// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::collections::HashMap;
use std::time::Instant;

use procedural::core::{AttemptId, Domain, ExamProfileId, ProblemFamilyId, SchemaId, SkillId};
use procedural::diagnostics::ErrorCategory;
use procedural::exam::{ExamObjective, ExamProfile};
use procedural::practice::{
    PracticeObjective, PracticeRequest, PracticeScope, SchemaPracticeObject, SessionBudget,
};
use procedural::problems::registry::ProblemRegistry;
use procedural::remediation::{
    RemediationAction, RemediationActionKind, RemediationQueue, RemediationUrgency,
};
use procedural::scheduling::{
    BacklogSeverity, BacklogTriageEngine, MacroBudgetPlanner, MacroPlanningContext,
    SessionBudgetTracker, StructuralCoverageEvaluator, StructuralCoverageProfile,
    UnifiedPracticeEngine, DEFAULT_ANTI_STARVATION_FLOOR,
};
use procedural::skills::signals::{IndependenceLevel, VariantCategory};
use procedural::skills::{
    MasteryEvidence, PracticeProgressionState, PrerequisiteGraphService, RecentAttemptRecord,
    SkillState,
};
use procedural::storage::ProceduralStore;

/// Helper initializing in-memory store and schemas for testing.
fn setup_test_environment() -> (
    ProceduralStore,
    ProblemRegistry,
    PrerequisiteGraphService,
    Vec<SchemaPracticeObject>,
    HashMap<SchemaId, Domain>,
    HashMap<SkillId, SkillState>,
) {
    let store = ProceduralStore::open_in_memory().unwrap();
    let registry = ProblemRegistry::new();
    let prereq_service = PrerequisiteGraphService::new();

    let domains = [
        Domain::Mathematics,
        Domain::Physics,
        Domain::Chemistry,
        Domain::Reasoning,
    ];

    let mut schemas = Vec::new();
    let mut schema_domains = HashMap::new();
    let mut skill_states = HashMap::new();

    for (_d_idx, domain) in domains.iter().enumerate() {
        for s_idx in 1..=3 {
            let skill_id = SkillId::new(format!("skill_{:?}_{}", domain, s_idx).to_lowercase());
            let schema_id = SchemaId::new(format!("schema_{:?}_{}", domain, s_idx).to_lowercase());
            let family_id = ProblemFamilyId::new(format!("family_{:?}_{}", domain, s_idx).to_lowercase());

            let skill = procedural::Skill::new(
                skill_id.clone(),
                domain.clone(),
                format!("{:?} Skill {}", domain, s_idx),
                format!("Description for {:?}", domain),
            );
            store.insert_skill(&skill).unwrap();

            let family = procedural::ProblemFamily::new(
                family_id.clone(),
                skill_id.clone(),
                domain.clone(),
                format!("{:?} Family {}", domain, s_idx),
                "default.v1",
            );
            store.insert_problem_family(&family).unwrap();

            let schema = SchemaPracticeObject::new(
                schema_id.clone(),
                skill_id.clone(),
                family_id,
                format!("{:?} Schema {}", domain, s_idx),
                format!("Description for {:?}", domain),
            );
            store.insert_schema(&schema).unwrap();

            let mut state = SkillState::new(skill_id.clone());
            state.practice_state = PracticeProgressionState::Learning;
            state.recent_attempts = vec![
                RecentAttemptRecord {
                    is_correct: true,
                    score: 1.0,
                    latency_ms: 30_000,
                    target_latency_ms: 35_000,
                    variant: Some("param_1".into()),
                    variant_category: Some(VariantCategory::Parameter),
                    error_category: None,
                    max_hint_level: None,
                    hint_count: Some(0),
                    independence: Some(IndependenceLevel::Independent),
                    solution_graph_fingerprint: Some("sg-1".into()),
                    cognitive_decision_correct: Some(true),
                    timestamp: 1000,
                },
            ];

            schemas.push(schema);
            schema_domains.insert(schema_id, domain.clone());
            skill_states.insert(skill_id, state);
        }
    }

    // Set up prerequisite chain: Maths Skill 1 -> Physics Skill 1
    prereq_service.register_skill_prerequisites(
        SkillId::new("skill_physics_1"),
        vec![SkillId::new("skill_mathematics_1")],
    );

    (store, registry, prereq_service, schemas, schema_domains, skill_states)
}

#[test]
fn test_gate_1_no_workload_explosion() {
    let (_, _, _, _schemas, schema_domains, skill_states) = setup_test_environment();

    let request = PracticeRequest::new(PracticeScope::AllDomains, PracticeObjective::Practice)
        .with_session_budget(SessionBudget::TimeLimitMs {
            max_time_ms: 3_600_000, // 60 mins
        });

    let effective_prereqs = HashMap::new();
    let capacities = HashMap::new();

    let ctx = MacroPlanningContext {
        total_time_budget_ms: 3_600_000,
        item_budget: None,
        request: &request,
        exam_profile: None,
        skill_states: &skill_states,
        schema_domains: &schema_domains,
        remediation_queue: None,
        effective_prereq_values: &effective_prereqs,
        domain_structural_capacities: &capacities,
        anti_starvation_floor: DEFAULT_ANTI_STARVATION_FLOOR,
    };

    let plan = MacroBudgetPlanner::plan_session(&ctx);

    // Total allocated time must equal session time budget exactly
    let sum_time: u64 = plan.domain_allocations.values().map(|b| b.allocated_time_ms).sum();
    assert_eq!(sum_time, 3_600_000);
    assert_eq!(plan.active_domains.len(), 4);

    // Total items estimated should be reasonable (~75-85 items max for 60m across domains)
    let total_items: usize = plan.domain_allocations.values().map(|b| b.target_item_count).sum();
    assert!(total_items >= 50 && total_items <= 120, "Total items {} outside safe workload window", total_items);
}

#[test]
fn test_gate_2_no_domain_starvation_with_anti_starvation_floor() {
    let (_, _, _, _schemas, schema_domains, skill_states) = setup_test_environment();

    // Create an extreme exam profile where Mathematics is 85% and Chemistry is 5%
    let mut profile = ExamProfile::new(
        ExamProfileId::new("extreme_math_exam"),
        "Math Heavy Blueprint",
        "85% Math exam",
        vec![
            Domain::Mathematics,
            Domain::Physics,
            Domain::Chemistry,
            Domain::Reasoning,
        ],
        ExamObjective::ConceptMastery,
    );
    profile.domain_weights.insert(Domain::Mathematics, 0.85);
    profile.domain_weights.insert(Domain::Physics, 0.05);
    profile.domain_weights.insert(Domain::Chemistry, 0.05);
    profile.domain_weights.insert(Domain::Reasoning, 0.05);

    let request = PracticeRequest::new(PracticeScope::AllDomains, PracticeObjective::Exam);
    let effective_prereqs = HashMap::new();
    let capacities = HashMap::new();

    let ctx = MacroPlanningContext {
        total_time_budget_ms: 3_600_000, // 60 mins
        item_budget: None,
        request: &request,
        exam_profile: Some(&profile),
        skill_states: &skill_states,
        schema_domains: &schema_domains,
        remediation_queue: None,
        effective_prereq_values: &effective_prereqs,
        domain_structural_capacities: &capacities,
        anti_starvation_floor: 0.15,
    };

    let plan = MacroBudgetPlanner::plan_session(&ctx);

    // Chemistry and Physics must each receive AT LEAST 15% floor (540,000 ms) despite 5% exam weight
    let chem_alloc = plan.domain_allocations.get(&Domain::Chemistry).unwrap();
    let phys_alloc = plan.domain_allocations.get(&Domain::Physics).unwrap();
    let math_alloc = plan.domain_allocations.get(&Domain::Mathematics).unwrap();

    assert!(
        chem_alloc.allocated_time_ms >= 540_000,
        "Chemistry starved! Allocated {} ms (< 540,000 ms)",
        chem_alloc.allocated_time_ms
    );
    assert!(
        phys_alloc.allocated_time_ms >= 540_000,
        "Physics starved! Allocated {} ms (< 540,000 ms)",
        phys_alloc.allocated_time_ms
    );

    // Mathematics gets the largest surplus share
    assert!(
        math_alloc.allocated_time_ms > chem_alloc.allocated_time_ms,
        "Math should receive surplus: Math={}, Chem={}",
        math_alloc.allocated_time_ms,
        chem_alloc.allocated_time_ms
    );
}

#[test]
fn test_gate_3_long_break_backlog_triage() {
    let (_, _, _, _, _, skill_states) = setup_test_environment();

    // Create 30 overdue items after a 30-day break
    let mut overdue_items = Vec::new();
    for i in 1..=30 {
        let domain = match i % 4 {
            0 => Domain::Mathematics,
            1 => Domain::Physics,
            2 => Domain::Chemistry,
            _ => Domain::Reasoning,
        };
        let schema_id = SchemaId::new(format!("overdue_schema_{}", i));
        let skill_id = SkillId::new(format!("overdue_skill_{}", i));
        let elapsed_days = 30.0;
        overdue_items.push((schema_id, skill_id, domain, elapsed_days));
    }

    let effective_prereqs = HashMap::new();
    let plan = BacklogTriageEngine::triage_backlog(
        &overdue_items,
        &skill_states,
        &effective_prereqs,
        12, // Max session memory quota
    );

    assert_eq!(plan.severity, BacklogSeverity::Severe);
    assert_eq!(plan.total_overdue_count, 30);
    assert_eq!(plan.active_quota, 12);
    assert_eq!(plan.deferred_count, 18);

    // Verify prioritized items are sorted by forgetting risk (retrievability)
    for i in 0..plan.prioritized_items.len() - 1 {
        assert!(
            plan.prioritized_items[i].triage_priority_score >= plan.prioritized_items[i + 1].triage_priority_score
        );
    }

    // Active items must not be deferred; deferred items must be marked deferred
    for (idx, item) in plan.prioritized_items.iter().enumerate() {
        if idx < 12 {
            assert!(!item.is_deferred);
        } else {
            assert!(item.is_deferred);
            assert!(item.deferral_reason.is_some());
        }
    }
}

#[test]
fn test_gate_4_effective_prerequisite_value_propagation() {
    let prereq_service = PrerequisiteGraphService::new();

    let math_foundational = SkillId::new("math_algebra_fundamentals");
    let math_intermediate = SkillId::new("math_polynomials");
    let physics_kinematics = SkillId::new("physics_kinematics_advanced");

    // DAG: Fundamentals -> Polynomials -> Advanced Kinematics
    prereq_service.register_skill_prerequisites(math_intermediate.clone(), vec![math_foundational.clone()]);
    prereq_service.register_skill_prerequisites(physics_kinematics.clone(), vec![math_intermediate.clone()]);

    // Exam profile gives 0 direct weight to fundamentals, 0.5 to polynomials, 3.0 to advanced kinematics
    let mut direct_values = HashMap::new();
    direct_values.insert(math_foundational.clone(), 0.0);
    direct_values.insert(math_intermediate.clone(), 0.5);
    direct_values.insert(physics_kinematics.clone(), 3.0);

    let effective_values = prereq_service.compute_effective_prerequisite_values(&direct_values, 0.80);

    let fund_eff = effective_values.get(&math_foundational).copied().unwrap_or(0.0);
    let inter_eff = effective_values.get(&math_intermediate).copied().unwrap_or(0.0);
    let phys_eff = effective_values.get(&physics_kinematics).copied().unwrap_or(0.0);

    // Foundational algebra had direct value 0.0, but should have substantial propagated value (> 2.0)
    assert!(
        fund_eff >= 2.0,
        "Foundational algebra should accumulate effective value from downstream (got {:.2})",
        fund_eff
    );
    assert!(inter_eff > 2.5, "Intermediate algebra should accumulate value (got {:.2})", inter_eff);
    assert_eq!(phys_eff, 3.0, "Leaf node retains direct value");
}

#[test]
fn test_gate_5_procedural_instance_queue_control() {
    let mut state = SkillState::new(SkillId::new("ratio_proportions"));
    state.practice_state = PracticeProgressionState::Learning;

    // Simulate 3 parameter solves
    for i in 1..=3 {
        let evidence = MasteryEvidence {
            final_correctness: true,
            decision_quality: Some(1.0),
            step_quality: None,
            independence: IndependenceLevel::Independent,
            max_hint_level: None,
            hint_dependence: 0,
            retry_dependence: 0,
            variant_exposure: Some(format!("param_{}", i)),
            variant_category: VariantCategory::Parameter,
            solution_graph_fingerprint: Some(format!("sg-param-{}", i)),
            cognitive_decision_correct: Some(true),
            time_since_last_ms: Some(10_000),
            transfer_evidence: false,
            domain_competence_verified: Some(true),
            latency_evidence: 25_000,
            diagnostic_errors: Vec::new(),
        };
        state.record_attempt_outcome(&evidence, 1.0, 30_000, 1000 + i * 60);
    }

    let profile = StructuralCoverageProfile::from_skill_state(&state);
    assert_eq!(profile.parameter_count, 3);

    // Novelty multiplier should now dampen parameter category and encourage isomorphic/structural
    let param_mult = StructuralCoverageEvaluator::compute_novelty_multiplier(Some(&state), &VariantCategory::Parameter);
    let isom_mult = StructuralCoverageEvaluator::compute_novelty_multiplier(Some(&state), &VariantCategory::Isomorphic);

    assert!(
        isom_mult > param_mult,
        "Isomorphic multiplier ({:.2}) should exceed parameter ({:.2}) after parameter quota reached",
        isom_mult,
        param_mult
    );
}

#[test]
fn test_gate_6_soft_jit_prerequisite_warnings() {
    let (store, registry, prereq_service, schemas, schema_domains, skill_states) = setup_test_environment();

    // Request practice on Physics Skill 1, which requires Maths Skill 1
    // Mark Maths Skill 1 as New with 0 history
    let mut empty_skill_states = skill_states.clone();
    empty_skill_states.insert(SkillId::new("skill_mathematics_1"), SkillState::new(SkillId::new("skill_mathematics_1")));

    let request = PracticeRequest::new(
        PracticeScope::SingleSkill(SkillId::new("skill_physics_1")),
        PracticeObjective::Practice,
    );

    let pyqs = HashMap::new();
    let decision = UnifiedPracticeEngine::select_next(
        &request,
        &schemas,
        &schema_domains,
        &empty_skill_states,
        &prereq_service,
        None,
        None,
        &pyqs,
        None,
        &registry,
        &store,
        42,
    ).unwrap();

    // User is NOT hard blocked from practicing Physics 1; soft advisory warning is attached
    assert_eq!(decision.skill_id, SkillId::new("skill_physics_1"));
    assert!(decision.advisory_warning.is_some(), "Soft advisory warning expected");
    assert!(decision.advisory_warning.unwrap().contains("Foundational gaps"));
}

#[test]
fn test_gate_7_user_intent_scope_authoritative() {
    let (store, registry, prereq_service, schemas, schema_domains, skill_states) = setup_test_environment();

    // Create an exam profile where Mathematics is 90%
    let mut profile = ExamProfile::new(
        ExamProfileId::new("math_exam"),
        "Math Blueprint",
        "Desc",
        vec![Domain::Mathematics, Domain::Physics, Domain::Chemistry, Domain::Reasoning],
        ExamObjective::BalancedPreparation,
    );
    profile.domain_weights.insert(Domain::Mathematics, 0.90);
    profile.domain_weights.insert(Domain::Chemistry, 0.05);

    // User explicitly requests SingleDomain(Chemistry)
    let request = PracticeRequest::new(
        PracticeScope::SingleDomain(Domain::Chemistry),
        PracticeObjective::Practice,
    );

    let effective_prereqs = HashMap::new();
    let capacities = HashMap::new();

    let ctx = MacroPlanningContext {
        total_time_budget_ms: 1_800_000,
        item_budget: None,
        request: &request,
        exam_profile: Some(&profile),
        skill_states: &skill_states,
        schema_domains: &schema_domains,
        remediation_queue: None,
        effective_prereq_values: &effective_prereqs,
        domain_structural_capacities: &capacities,
        anti_starvation_floor: DEFAULT_ANTI_STARVATION_FLOOR,
    };

    let plan = MacroBudgetPlanner::plan_session(&ctx);

    // 100% must be allocated to Chemistry without escaping to Mathematics
    assert_eq!(plan.active_domains, vec![Domain::Chemistry]);
    assert_eq!(plan.domain_allocations.len(), 1);
    assert_eq!(
        plan.domain_allocations.get(&Domain::Chemistry).unwrap().allocated_time_ms,
        1_800_000
    );

    // Selection engine under this plan must pick a Chemistry schema
    let budget_tracker = SessionBudgetTracker::new(Some(SessionBudget::TimeLimitMs { max_time_ms: 1_800_000 }));
    let pyqs = HashMap::new();

    let decision = UnifiedPracticeEngine::select_next_with_macro_plan(
        &request,
        &plan,
        &budget_tracker,
        &schemas,
        &schema_domains,
        &skill_states,
        &prereq_service,
        None,
        Some(&profile),
        &pyqs,
        None,
        &registry,
        &store,
        42,
    ).unwrap();

    assert_eq!(decision.domain, Domain::Chemistry);
}

#[test]
fn test_gate_8_mature_skills_receive_meaningful_transfer() {
    let mut state = SkillState::new(SkillId::new("kinematics_work_energy"));
    state.practice_state = PracticeProgressionState::Fluent;
    state.structural_forms_seen.insert("param_1".into(), 2);
    state.structural_forms_seen.insert("isomorphic_1".into(), 1);

    // In Fluent stage, structural and contextual multipliers are elevated
    let struct_mult = StructuralCoverageEvaluator::compute_novelty_multiplier(Some(&state), &VariantCategory::Structural);
    let param_mult = StructuralCoverageEvaluator::compute_novelty_multiplier(Some(&state), &VariantCategory::Parameter);

    assert!(
        struct_mult >= 1.30,
        "Structural novelty multiplier should be elevated for fluent skill (got {:.2})",
        struct_mult
    );
    assert!(
        struct_mult > param_mult,
        "Structural form should be prioritized over parameter form for fluent skill"
    );

    // Once in Variation stage, Transfer and Multi-Concept are strongly favored
    state.practice_state = PracticeProgressionState::Variation;
    let transfer_mult = StructuralCoverageEvaluator::compute_novelty_multiplier(Some(&state), &VariantCategory::Transfer);
    assert!(transfer_mult >= 1.40, "Transfer multiplier should be elevated in Variation stage");
}

#[test]
fn test_gate_9_domain_block_anti_switching() {
    let (_, _, _, _schemas, schema_domains, skill_states) = setup_test_environment();

    let request = PracticeRequest::new(PracticeScope::AllDomains, PracticeObjective::Practice);
    let effective_prereqs = HashMap::new();
    let capacities = HashMap::new();

    let ctx = MacroPlanningContext {
        total_time_budget_ms: 3_600_000, // 60 mins
        item_budget: None,
        request: &request,
        exam_profile: None,
        skill_states: &skill_states,
        schema_domains: &schema_domains,
        remediation_queue: None,
        effective_prereq_values: &effective_prereqs,
        domain_structural_capacities: &capacities,
        anti_starvation_floor: DEFAULT_ANTI_STARVATION_FLOOR,
    };

    let plan = MacroBudgetPlanner::plan_session(&ctx);

    // Should create up to 2 blocks per active domain (anti-starvation pass + surplus pass)
    // For 4 domains, this means up to 8 structured contiguous domain blocks.
    assert!(
        plan.domain_blocks.len() >= 4 && plan.domain_blocks.len() <= 8,
        "Domain blocks count {} outside bounded 4-8 range (expected up to 2 per domain)",
        plan.domain_blocks.len()
    );

    // Verify time sequencing: at 0ms, block 1 domain is active
    let domain_t0 = plan.active_domain_at_elapsed(0).unwrap();
    
    assert_eq!(domain_t0, plan.domain_blocks[0].domain);
    // Anti-starvation blocks might be as short as MIN_DOMAIN_BLOCK_DURATION_MS (180_000ms / 3 mins)
    assert!(plan.domain_blocks[0].duration_ms >= 180_000, "Block duration should be at least 3 minutes (minimum domain block)");
}

#[test]
fn test_gate_10_g1_learning_dynamics_intact_with_g2_scheduler() {
    let (store, registry, prereq_service, schemas, schema_domains, skill_states) = setup_test_environment();

    // Test Remediation Circuit Breaker integration under G2
    let mut queue = RemediationQueue::new();
    let mut cb_action = RemediationAction::new(
        "rem-cb-1",
        RemediationActionKind::CircuitBreaker,
        SkillId::new("skill_mathematics_1"),
        SchemaId::new("schema_mathematics_1"),
        Domain::Mathematics,
        ErrorCategory::Concept,
        AttemptId::new("att-1"),
        "Multiple repeated failures cooldown",
    );
    cb_action.urgency = RemediationUrgency::Advisory;
    cb_action.recurrence_count = 5;
    queue.pending_actions.push(cb_action);

    let request = PracticeRequest::new(PracticeScope::AllDomains, PracticeObjective::Practice);
    let pyqs = HashMap::new();

    // Circuit breaker produces Advisory / Cooldown learning object without infinite loop
    let decision = UnifiedPracticeEngine::select_next(
        &request,
        &schemas,
        &schema_domains,
        &skill_states,
        &prereq_service,
        Some(&mut queue),
        None,
        &pyqs,
        None,
        &registry,
        &store,
        42,
    ).unwrap();

    assert!(decision.selection_reason.contains("remediation_circuit_breaker"));
}

#[test]
fn test_session_matrix_and_cognitive_load() {
    let durations = [
        1_200_000,  // 20 mins
        2_700_000,  // 45 mins
        3_600_000,  // 60 mins
        7_200_000,  // 120 mins
    ];

    let (_, _, _, _schemas, schema_domains, skill_states) = setup_test_environment();
    let effective_prereqs = HashMap::new();
    let capacities = HashMap::new();

    for &duration in &durations {
        let request = PracticeRequest::new(PracticeScope::AllDomains, PracticeObjective::Practice)
            .with_session_budget(SessionBudget::TimeLimitMs { max_time_ms: duration });

        let ctx = MacroPlanningContext {
            total_time_budget_ms: duration,
            item_budget: None,
            request: &request,
            exam_profile: None,
            skill_states: &skill_states,
            schema_domains: &schema_domains,
            remediation_queue: None,
            effective_prereq_values: &effective_prereqs,
            domain_structural_capacities: &capacities,
            anti_starvation_floor: DEFAULT_ANTI_STARVATION_FLOOR,
        };

        let plan = MacroBudgetPlanner::plan_session(&ctx);
        assert_eq!(plan.domain_allocations.len(), 4);

        let mut tracker = SessionBudgetTracker::new(Some(SessionBudget::TimeLimitMs { max_time_ms: duration }))
            .with_max_cognitive_load(50.0);

        // Record a sequence of mixed problems
        tracker.record_item_with_domain(&Domain::Mathematics, 30_000, false, 2);
        tracker.record_item_with_domain(&Domain::Physics, 60_000, false, 4);
        tracker.record_item_with_domain(&Domain::Chemistry, 45_000, true, 3);

        assert_eq!(tracker.items_completed, 3);
        assert_eq!(tracker.remediations_served, 1);
        assert!(tracker.total_cognitive_load > 3.0);
        assert!(!tracker.is_exhausted);
    }
}

#[test]
fn test_performance_scaling_benchmark() {
    let prereq_service = PrerequisiteGraphService::new();

    // Benchmark scaling across 22, 100, 300, 1000 skills in realistic syllabus hierarchy
    for &skill_count in &[22, 100, 300, 1000] {
        let mut direct_values = HashMap::new();
        for i in 1..=skill_count {
            let skill_id = SkillId::new(format!("bench_skill_{}", i));
            direct_values.insert(skill_id.clone(), (i % 5) as f64);
            let parent_idx = (i / 8).max(1);
            if i > 1 && parent_idx != i {
                let parent = SkillId::new(format!("bench_skill_{}", parent_idx));
                prereq_service.register_skill_prerequisites(skill_id, vec![parent]);
            }
        }

        let start = Instant::now();
        let eff_values = prereq_service.compute_effective_prerequisite_values(&direct_values, 0.75);
        let elapsed = start.elapsed();

        assert_eq!(eff_values.len(), skill_count);
        // Ensure execution is sub-millisecond to low milliseconds (well within budget)
        assert!(
            elapsed.as_millis() < 50,
            "Scaling calculation for {} skills took too long: {:?}",
            skill_count,
            elapsed
        );
    }
}

// =========================================================================
// LONGITUDINAL MULTI-DOMAIN TARGETED SIMULATIONS (30d, 90d, 180d)
// =========================================================================

#[test]
fn test_simulation_30_day_multi_domain_cohort() {
    let (store, registry, prereq_service, schemas, schema_domains, mut skill_states) = setup_test_environment();

    let mut domain_exposure_seconds: HashMap<Domain, u64> = HashMap::new();
    let mut total_problems_solved = 0;

    // Simulate 30 days of daily 45-minute multi-domain practice
    let session_duration_ms = 2_700_000; // 45 mins
    let mut remediation_queue = RemediationQueue::new();
    let pyqs = HashMap::new();

    for day in 1..=30 {
        let request = PracticeRequest::new(PracticeScope::AllDomains, PracticeObjective::Practice)
            .with_session_budget(SessionBudget::TimeLimitMs { max_time_ms: session_duration_ms });

        let effective_prereqs = HashMap::new();
        let capacities = HashMap::new();

        let ctx = MacroPlanningContext {
            total_time_budget_ms: session_duration_ms,
            item_budget: None,
            request: &request,
            exam_profile: None,
            skill_states: &skill_states,
            schema_domains: &schema_domains,
            remediation_queue: Some(&remediation_queue),
            effective_prereq_values: &effective_prereqs,
            domain_structural_capacities: &capacities,
            anti_starvation_floor: DEFAULT_ANTI_STARVATION_FLOOR,
        };

        let plan = MacroBudgetPlanner::plan_session(&ctx);
        let mut tracker = SessionBudgetTracker::new(Some(SessionBudget::TimeLimitMs { max_time_ms: session_duration_ms }));

        while !tracker.is_exhausted && tracker.items_completed < 40 {
            if let Some(decision) = UnifiedPracticeEngine::select_next_with_macro_plan(
                &request,
                &plan,
                &tracker,
                &schemas,
                &schema_domains,
                &skill_states,
                &prereq_service,
                Some(&mut remediation_queue),
                None,
                &pyqs,
                None,
                &registry,
                &store,
                (day * 100 + tracker.items_completed) as u64,
            ) {
                let domain = decision.domain.clone();
                let latency_ms = decision.target_time_ms;
                let is_remed = matches!(decision.learning_object, procedural::LearningObjectKind::Remediation(_));

                tracker.record_item_with_domain(&domain, latency_ms, is_remed, decision.difficulty_level);
                *domain_exposure_seconds.entry(domain.clone()).or_insert(0) += latency_ms / 1000;
                total_problems_solved += 1;

                // Update skill state with simulated solve outcome (Physics lower accuracy 65%, others 85%)
                let is_correct = match domain {
                    Domain::Physics => (total_problems_solved % 3) != 0, // 66% accuracy
                    _ => (total_problems_solved % 7) != 0, // 86% accuracy
                };

                let state = skill_states.entry(decision.skill_id.clone()).or_insert_with(|| SkillState::new(decision.skill_id.clone()));
                let evidence = MasteryEvidence {
                    final_correctness: is_correct,
                    decision_quality: Some(1.0),
                    step_quality: None,
                    independence: IndependenceLevel::Independent,
                    max_hint_level: None,
                    hint_dependence: 0,
                    retry_dependence: 0,
                    variant_exposure: Some("param_sim".into()),
                    variant_category: VariantCategory::Parameter,
                    solution_graph_fingerprint: Some("sg-sim".into()),
                    cognitive_decision_correct: Some(true),
                    time_since_last_ms: Some(86_400_000),
                    transfer_evidence: false,
                    domain_competence_verified: Some(true),
                    latency_evidence: latency_ms,
                    diagnostic_errors: if is_correct { vec![] } else { vec![ErrorCategory::Calculation] },
                };
                state.record_attempt_outcome(&evidence, if is_correct { 1.0 } else { 0.0 }, latency_ms, (day * 86_400) as i64);
            } else {
                break;
            }
        }
    }

    // Over 30 days (1,350 total study minutes):
    let total_practiced_secs: u64 = domain_exposure_seconds.values().sum();
    assert!(total_practiced_secs >= 60_000, "Total practiced time must be substantial");

    // Gate 2 Verification: Every single domain must have received at least 12% lifetime exposure (anti-starvation)
    for (dom, &secs) in &domain_exposure_seconds {
        let share = secs as f64 / total_practiced_secs as f64;
        assert!(
            share >= 0.12,
            "Domain {:?} starved over 30 days! Share was only {:.1}%",
            dom,
            share * 100.0
        );
    }
}

#[test]
fn test_simulation_90_day_exam_crammer_anti_starvation() {
    let (store, registry, prereq_service, schemas, schema_domains, mut skill_states) = setup_test_environment();

    // 75% Maths exam blueprint
    let mut profile = ExamProfile::new(
        ExamProfileId::new("maths_dominant_exam"),
        "Maths Focus Exam",
        "75% syllabus in maths",
        vec![Domain::Mathematics, Domain::Physics, Domain::Chemistry, Domain::Reasoning],
        ExamObjective::ConceptMastery,
    );
    profile.domain_weights.insert(Domain::Mathematics, 0.75);
    profile.domain_weights.insert(Domain::Physics, 0.10);
    profile.domain_weights.insert(Domain::Chemistry, 0.10);
    profile.domain_weights.insert(Domain::Reasoning, 0.05);

    let mut domain_minutes: HashMap<Domain, f64> = HashMap::new();
    let session_duration_ms = 3_600_000; // 60 mins/day
    let mut remediation_queue = RemediationQueue::new();
    let pyqs = HashMap::new();

    for day in 1..=90 {
        let request = PracticeRequest::new(PracticeScope::AllDomains, PracticeObjective::Exam)
            .with_session_budget(SessionBudget::TimeLimitMs { max_time_ms: session_duration_ms });

        let effective_prereqs = HashMap::new();
        let capacities = HashMap::new();

        let ctx = MacroPlanningContext {
            total_time_budget_ms: session_duration_ms,
            item_budget: None,
            request: &request,
            exam_profile: Some(&profile),
            skill_states: &skill_states,
            schema_domains: &schema_domains,
            remediation_queue: Some(&remediation_queue),
            effective_prereq_values: &effective_prereqs,
            domain_structural_capacities: &capacities,
            anti_starvation_floor: DEFAULT_ANTI_STARVATION_FLOOR,
        };

        let plan = MacroBudgetPlanner::plan_session(&ctx);
        let mut tracker = SessionBudgetTracker::new(Some(SessionBudget::TimeLimitMs { max_time_ms: session_duration_ms }));

        while !tracker.is_exhausted && tracker.items_completed < 120 {
            if let Some(decision) = UnifiedPracticeEngine::select_next_with_macro_plan(
                &request,
                &plan,
                &tracker,
                &schemas,
                &schema_domains,
                &skill_states,
                &prereq_service,
                Some(&mut remediation_queue),
                Some(&profile),
                &pyqs,
                None,
                &registry,
                &store,
                (day * 100 + tracker.items_completed) as u64,
            ) {
                let domain = decision.domain.clone();
                let latency_ms = decision.target_time_ms;
                let is_remed = matches!(decision.learning_object, procedural::LearningObjectKind::Remediation(_));

                tracker.record_item_with_domain(&domain, latency_ms, is_remed, decision.difficulty_level);
                *domain_minutes.entry(domain.clone()).or_insert(0.0) += (latency_ms as f64) / 60_000.0;

                let state = skill_states.entry(decision.skill_id.clone()).or_insert_with(|| SkillState::new(decision.skill_id.clone()));
                let evidence = MasteryEvidence {
                    final_correctness: true,
                    decision_quality: Some(1.0),
                    step_quality: None,
                    independence: IndependenceLevel::Independent,
                    max_hint_level: None,
                    hint_dependence: 0,
                    retry_dependence: 0,
                    variant_exposure: Some("param_sim".into()),
                    variant_category: VariantCategory::Parameter,
                    solution_graph_fingerprint: Some("sg-sim".into()),
                    cognitive_decision_correct: Some(true),
                    time_since_last_ms: Some(86_400_000),
                    transfer_evidence: false,
                    domain_competence_verified: Some(true),
                    latency_evidence: latency_ms,
                    diagnostic_errors: vec![],
                };
                state.record_attempt_outcome(&evidence, 1.0, latency_ms, (day * 86_400) as i64);
            } else {
                break;
            }
        }
    }

    let total_mins: f64 = domain_minutes.values().sum();
    let math_share = domain_minutes.get(&Domain::Mathematics).copied().unwrap_or(0.0) / total_mins;
    let chem_share = domain_minutes.get(&Domain::Chemistry).copied().unwrap_or(0.0) / total_mins;
    let phys_share = domain_minutes.get(&Domain::Physics).copied().unwrap_or(0.0) / total_mins;
    let reas_share = domain_minutes.get(&Domain::Reasoning).copied().unwrap_or(0.0) / total_mins;

    // Maths receives the primary surplus (>35%, substantially larger than other domains)
    assert!(math_share > 0.35, "Maths should receive substantial surplus (got {:.1}%)", math_share * 100.0);

    // Non-exam subjects are protected by the anti-starvation floor
    assert!(chem_share >= 0.10, "Chemistry floor protected (got {:.1}%)", chem_share * 100.0);
    assert!(phys_share >= 0.10, "Physics floor protected (got {:.1}%)", phys_share * 100.0);
    assert!(reas_share >= 0.10, "Reasoning floor protected (got {:.1}%)", reas_share * 100.0);
}

#[test]
fn test_simulation_180_day_long_break_and_maintenance_cohort() {
    let (_store, _registry, _prereq_service, _schemas, schema_domains, skill_states) = setup_test_environment();

    let mut overdue_items = Vec::new();
    // Simulate 180 days with a 30-day inactivity gap at day 60
    for day in 1..=180 {
        if day > 60 && day <= 90 {
            // Learner is away on a 30-day break; accumulation of overdue reviews (1 per day)
            let schema_id = SchemaId::new(format!("accum_schema_{}", day));
            let skill_id = SkillId::new(format!("accum_skill_{}", day));
            overdue_items.push((schema_id, skill_id, Domain::Mathematics, (90 - day) as f64));
            continue;
        }

        // On day 91 (return day), triage backlog
        if day == 91 && !overdue_items.is_empty() {
            let effective_prereqs = HashMap::new();
            let triage_plan = BacklogTriageEngine::triage_backlog(&overdue_items, &skill_states, &effective_prereqs, 15);

            assert_eq!(triage_plan.severity, BacklogSeverity::Severe);
            assert!(triage_plan.active_quota <= 15, "Learner not overwhelmed on return");
            assert!(triage_plan.deferred_count > 0, "Non-critical reviews staged for future days");
            overdue_items.clear();
        }

        // Standard maintenance and progression
        let request = PracticeRequest::new(PracticeScope::AllDomains, PracticeObjective::Practice)
            .with_session_budget(SessionBudget::TimeLimitMs { max_time_ms: 1_800_000 }); // 30 mins

        let effective_prereqs = HashMap::new();
        let capacities = HashMap::new();

        let ctx = MacroPlanningContext {
            total_time_budget_ms: 1_800_000,
            item_budget: None,
            request: &request,
            exam_profile: None,
            skill_states: &skill_states,
            schema_domains: &schema_domains,
            remediation_queue: None,
            effective_prereq_values: &effective_prereqs,
            domain_structural_capacities: &capacities,
            anti_starvation_floor: DEFAULT_ANTI_STARVATION_FLOOR,
        };

        let plan = MacroBudgetPlanner::plan_session(&ctx);
        assert_eq!(plan.domain_allocations.len(), 4);
    }
}

