// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::collections::{HashMap, HashSet};

use procedural::chemistry::generators::{
    FAMILY_CHEMISTRY_BUFFERS_TITRATION, FAMILY_CHEMISTRY_ELECTROCHEMISTRY,
    FAMILY_CHEMISTRY_EQUILIBRIUM, FAMILY_CHEMISTRY_KINETICS, FAMILY_CHEMISTRY_REACTION_NETWORKS,
    FAMILY_CHEMISTRY_STOICHIOMETRY,
};
use procedural::core::{Domain, ProblemFamilyId, SchemaId, SkillId};
use procedural::physics::generators::{FAMILY_PHYSICS_KINEMATICS, FAMILY_PHYSICS_WORK_ENERGY};
use procedural::practice::{
    PracticeObjective, PracticeRequest, PracticeScope, SchemaPracticeObject, SessionBudget,
};
use procedural::problems::catalog::{
    MathsCatalog, FAMILY_PERCENTAGE_SUCCESSIVE,
};
use procedural::problems::generators::{
    FAMILY_ALGEBRAIC_IDENTITIES, FAMILY_AVERAGE, FAMILY_COMBINED_MULTI_CONCEPT,
    FAMILY_DIVISIBILITY, FAMILY_GEOMETRY_TRIANGLES, FAMILY_LINEAR_EQUATIONS,
    FAMILY_LINEAR_INEQUALITIES, FAMILY_MIXTURES_ALLIGATION, FAMILY_PROFIT_LOSS, FAMILY_RATIO,
    FAMILY_REMAINDERS_MODULAR, FAMILY_TIME_SPEED_DISTANCE, FAMILY_TIME_WORK,
};
use procedural::problems::registry::ProblemRegistry;
use procedural::reasoning::generators::{
    FAMILY_REASONING_CODED_EXPRESSIONS, FAMILY_REASONING_DATA_SUFFICIENCY,
    FAMILY_REASONING_FLOOR_GRID, FAMILY_REASONING_LOGIC_DAG, FAMILY_REASONING_RELATIONS,
    FAMILY_REASONING_SEATING, FAMILY_REASONING_SERIES, FAMILY_REASONING_SYLLOGISM,
};
use procedural::remediation::RemediationQueue;
use procedural::scheduling::{
    MacroBudgetPlanner, MacroPlanningContext, SessionBudgetTracker, UnifiedPracticeEngine,
    DEFAULT_ANTI_STARVATION_FLOOR,
};
use procedural::skills::signals::{IndependenceLevel, VariantCategory};
use procedural::skills::{
    PracticeProgressionState, PrerequisiteGraphService, RecentAttemptRecord, SkillState,
};
use procedural::storage::ProceduralStore;

fn all_30_families() -> Vec<(&'static str, Domain)> {
    vec![
        // 14 Mathematics
        (FAMILY_PERCENTAGE_SUCCESSIVE, Domain::Mathematics),
        (FAMILY_LINEAR_EQUATIONS, Domain::Mathematics),
        (FAMILY_PROFIT_LOSS, Domain::Mathematics),
        (FAMILY_RATIO, Domain::Mathematics),
        (FAMILY_AVERAGE, Domain::Mathematics),
        (FAMILY_DIVISIBILITY, Domain::Mathematics),
        (FAMILY_TIME_WORK, Domain::Mathematics),
        (FAMILY_TIME_SPEED_DISTANCE, Domain::Mathematics),
        (FAMILY_MIXTURES_ALLIGATION, Domain::Mathematics),
        (FAMILY_REMAINDERS_MODULAR, Domain::Mathematics),
        (FAMILY_LINEAR_INEQUALITIES, Domain::Mathematics),
        (FAMILY_ALGEBRAIC_IDENTITIES, Domain::Mathematics),
        (FAMILY_GEOMETRY_TRIANGLES, Domain::Mathematics),
        (FAMILY_COMBINED_MULTI_CONCEPT, Domain::Mathematics),
        // 2 Physics
        (FAMILY_PHYSICS_KINEMATICS, Domain::Physics),
        (FAMILY_PHYSICS_WORK_ENERGY, Domain::Physics),
        // 6 Chemistry
        (FAMILY_CHEMISTRY_STOICHIOMETRY, Domain::Chemistry),
        (FAMILY_CHEMISTRY_EQUILIBRIUM, Domain::Chemistry),
        (FAMILY_CHEMISTRY_BUFFERS_TITRATION, Domain::Chemistry),
        (FAMILY_CHEMISTRY_ELECTROCHEMISTRY, Domain::Chemistry),
        (FAMILY_CHEMISTRY_KINETICS, Domain::Chemistry),
        (FAMILY_CHEMISTRY_REACTION_NETWORKS, Domain::Chemistry),
        // 8 Reasoning
        (FAMILY_REASONING_SERIES, Domain::Reasoning),
        (FAMILY_REASONING_SYLLOGISM, Domain::Reasoning),
        (FAMILY_REASONING_SEATING, Domain::Reasoning),
        (FAMILY_REASONING_RELATIONS, Domain::Reasoning),
        (FAMILY_REASONING_FLOOR_GRID, Domain::Reasoning),
        (FAMILY_REASONING_LOGIC_DAG, Domain::Reasoning),
        (FAMILY_REASONING_DATA_SUFFICIENCY, Domain::Reasoning),
        (FAMILY_REASONING_CODED_EXPRESSIONS, Domain::Reasoning),
    ]
}

/// Sets up the complete 30-family procedural environment with actual catalog schemas.
fn setup_full_30_family_environment() -> (
    ProceduralStore,
    ProblemRegistry,
    PrerequisiteGraphService,
    Vec<SchemaPracticeObject>,
    HashMap<SchemaId, Domain>,
    HashMap<SkillId, SkillState>,
) {
    let store = ProceduralStore::open_in_memory().unwrap();
    MathsCatalog::init_all(&store).unwrap();

    let registry = ProblemRegistry::new();
    let prereq_service = PrerequisiteGraphService::new();
    prereq_service.sync_from_store(&store).unwrap();

    let mut schemas = Vec::new();
    let mut schema_domains = HashMap::new();
    let mut skill_states = HashMap::new();

    for (fam_str, domain) in all_30_families() {
        let fam_id = ProblemFamilyId::from(fam_str);
        let schema = store.get_schema_by_family(&fam_id).unwrap().unwrap_or_else(|| {
            panic!("Schema for family {} must exist in database", fam_str);
        });

        let skill_id = schema.skill_id.clone();
        let schema_id = schema.id.clone();
        schema_domains.insert(schema_id, domain);
        schemas.push(schema);

        let mut state = SkillState::new(skill_id.clone());
        state.practice_state = PracticeProgressionState::Learning;
        state.total_attempts = 1;
        state.successful_attempts = 1;
        state.consecutive_successes = 1;
        state.recent_attempts = vec![
            RecentAttemptRecord {
                is_correct: true,
                score: 1.0,
                latency_ms: 25_000,
                target_latency_ms: 35_000,
                variant: Some("param_init".into()),
                variant_category: Some(VariantCategory::Parameter),
                error_category: None,
                max_hint_level: None,
                hint_count: Some(0),
                independence: Some(IndependenceLevel::Independent),
                solution_graph_fingerprint: Some("sg-init".into()),
                cognitive_decision_correct: Some(true),
                timestamp: 1000,
            },
        ];

        skill_states.insert(skill_id, state);
    }

    (store, registry, prereq_service, schemas, schema_domains, skill_states)
}

#[test]
fn test_g3_catalog_30_family_composition() {
    let store = ProceduralStore::open_in_memory().unwrap();
    MathsCatalog::init_all(&store).unwrap();

    let families = all_30_families();
    assert_eq!(families.len(), 30, "Procedural catalog must contain exactly 30 total families");

    let mut domain_counts: HashMap<Domain, usize> = HashMap::new();
    for (f, dom) in &families {
        let fam_id = ProblemFamilyId::from(*f);
        assert!(store.get_problem_family(&fam_id).unwrap().is_some(), "Family {} must exist in database", f);
        *domain_counts.entry(dom.clone()).or_default() += 1;
    }

    assert_eq!(domain_counts.get(&Domain::Mathematics).copied().unwrap_or(0), 14, "Maths should have 14 families");
    assert_eq!(domain_counts.get(&Domain::Physics).copied().unwrap_or(0), 2, "Physics should have 2 families");
    assert_eq!(domain_counts.get(&Domain::Chemistry).copied().unwrap_or(0), 6, "Chemistry should have 6 families (expanded from 2)");
    assert_eq!(domain_counts.get(&Domain::Reasoning).copied().unwrap_or(0), 8, "Reasoning should have 8 families (expanded from 4)");
}

#[test]
fn test_g3_90_day_multi_domain_anti_starvation_and_capacity() {
    let (store, registry, prereq_service, schemas, schema_domains, mut skill_states) =
        setup_full_30_family_environment();

    let mut domain_attempts: HashMap<Domain, usize> = HashMap::new();
    let mut chemistry_families_practiced: HashSet<ProblemFamilyId> = HashSet::new();
    let mut reasoning_families_practiced: HashSet<ProblemFamilyId> = HashSet::new();
    let mut total_problems_generated = 0;

    let pyqs = HashMap::new();
    let effective_prereqs = HashMap::new();
    let capacities = HashMap::new();

    // Simulate 90 days with 6 problems practiced per day
    for day in 1..=90 {
        let request = PracticeRequest::new(PracticeScope::AllDomains, PracticeObjective::Practice)
            .with_session_budget(SessionBudget::ItemCount { max_items: 6 });

        let mut rem_queue = RemediationQueue::new();
        let mut tracker = SessionBudgetTracker::new(Some(SessionBudget::ItemCount { max_items: 6 }));

        let ctx = MacroPlanningContext {
            total_time_budget_ms: 180_000,
            item_budget: Some(6),
            request: &request,
            exam_profile: None,
            skill_states: &skill_states,
            schema_domains: &schema_domains,
            remediation_queue: None,
            effective_prereq_values: &effective_prereqs,
            domain_structural_capacities: &capacities,
            anti_starvation_floor: DEFAULT_ANTI_STARVATION_FLOOR,
        };

        let macro_plan = MacroBudgetPlanner::plan_session(&ctx);

        while !tracker.is_exhausted && tracker.items_completed < 6 {
            if let Some(decision) = UnifiedPracticeEngine::select_next_with_macro_plan(
                &request,
                &macro_plan,
                &tracker,
                &schemas,
                &schema_domains,
                &skill_states,
                &prereq_service,
                Some(&mut rem_queue),
                None,
                &pyqs,
                None,
                &registry,
                &store,
                (day * 100 + tracker.items_completed) as u64,
            ) {
                total_problems_generated += 1;
                *domain_attempts.entry(decision.domain.clone()).or_default() += 1;

                if decision.domain == Domain::Chemistry {
                    chemistry_families_practiced.insert(decision.schema.problem_family_id.clone());
                } else if decision.domain == Domain::Reasoning {
                    reasoning_families_practiced.insert(decision.schema.problem_family_id.clone());
                }

                tracker.record_item_with_domain(&decision.domain, decision.target_time_ms, false, decision.difficulty_level);

                // Update skill state with simulated successful attempt
                let skill_id = decision.skill_id.clone();
                if let Some(state) = skill_states.get_mut(&skill_id) {
                    state.total_attempts += 1;
                    state.successful_attempts += 1;
                    state.consecutive_successes += 1;
                    state.last_success_at = Some(1000 + day as i64 * 86400);
                    state.last_practiced_at = Some(1000 + day as i64 * 86400);
                    if state.successful_attempts >= 2 {
                        state.practice_state = PracticeProgressionState::Fluent;
                    }
                    if state.successful_attempts >= 4 {
                        state.practice_state = PracticeProgressionState::Variation;
                    }
                    if state.successful_attempts >= 6 {
                        state.practice_state = PracticeProgressionState::Transfer;
                    }
                    if state.successful_attempts >= 8 {
                        state.practice_state = PracticeProgressionState::Mastered;
                    }
                    state.recent_attempts.push(RecentAttemptRecord {
                        is_correct: true,
                        score: 1.0,
                        latency_ms: 28_000,
                        target_latency_ms: 35_000,
                        variant: Some(format!("variant_d{}_{}", day, total_problems_generated)),
                        variant_category: Some(VariantCategory::Structural),
                        error_category: None,
                        max_hint_level: None,
                        hint_count: Some(0),
                        independence: Some(IndependenceLevel::Independent),
                        solution_graph_fingerprint: Some(format!("sg-d{}", day)),
                        cognitive_decision_correct: Some(true),
                        timestamp: 1000 + day as i64 * 86400,
                    });
                    if state.recent_attempts.len() > 10 {
                        state.recent_attempts.remove(0);
                    }
                }
            } else {
                break;
            }
        }
    }

    // Gate 1: No starvation across all 4 domains
    for domain in [Domain::Mathematics, Domain::Physics, Domain::Chemistry, Domain::Reasoning] {
        let count = domain_attempts.get(&domain).copied().unwrap_or(0);
        assert!(
            count >= 20,
            "Domain {:?} starved! Only {} attempts in 90 days",
            domain,
            count
        );
    }

    // Gate 2: High capacity in Chemistry (all 6 chemistry families engaged)
    assert_eq!(
        chemistry_families_practiced.len(),
        6,
        "All 6 chemistry families must be practiced over 90 days. Found: {:?}",
        chemistry_families_practiced
    );

    // Gate 3: High capacity in Reasoning (all 8 reasoning families engaged)
    assert_eq!(
        reasoning_families_practiced.len(),
        8,
        "All 8 reasoning families must be practiced over 90 days. Found: {:?}",
        reasoning_families_practiced
    );
}

#[test]
fn test_g3_180_day_longitudinal_transfer_and_retention() {
    let (store, registry, prereq_service, schemas, schema_domains, mut skill_states) =
        setup_full_30_family_environment();

    let mut generated_instances = 0;
    let mut decision_points_seen = 0;

    let pyqs = HashMap::new();
    let effective_prereqs = HashMap::new();
    let capacities = HashMap::new();

    // Simulate 180 days with session budget
    for day in 1..=180 {
        let request = PracticeRequest::new(PracticeScope::AllDomains, PracticeObjective::Practice)
            .with_session_budget(SessionBudget::ItemCount { max_items: 8 });

        let mut rem_queue = RemediationQueue::new();
        let mut tracker = SessionBudgetTracker::new(Some(SessionBudget::ItemCount { max_items: 8 }));

        let ctx = MacroPlanningContext {
            total_time_budget_ms: 240_000,
            item_budget: Some(8),
            request: &request,
            exam_profile: None,
            skill_states: &skill_states,
            schema_domains: &schema_domains,
            remediation_queue: None,
            effective_prereq_values: &effective_prereqs,
            domain_structural_capacities: &capacities,
            anti_starvation_floor: DEFAULT_ANTI_STARVATION_FLOOR,
        };

        let macro_plan = MacroBudgetPlanner::plan_session(&ctx);

        while !tracker.is_exhausted && tracker.items_completed < 8 {
            if let Some(decision) = UnifiedPracticeEngine::select_next_with_macro_plan(
                &request,
                &macro_plan,
                &tracker,
                &schemas,
                &schema_domains,
                &skill_states,
                &prereq_service,
                Some(&mut rem_queue),
                None,
                &pyqs,
                None,
                &registry,
                &store,
                (day * 100 + tracker.items_completed) as u64,
            ) {
                generated_instances += 1;

                // Generate concrete instance from registry to verify zero generation failures
                let inst = registry.generate(
                    &decision.schema.problem_family_id,
                    "",
                    (day * 1000 + generated_instances) as u64,
                    decision.difficulty_level,
                    None,
                )
                .expect("Problem instance generation must never fail");

                assert!(!inst.rendered_prompt.is_empty());
                assert!(inst.solution_graph().is_some());

                // Check if instance contains CognitiveDecisionPoint
                if inst.parameters.get("decision_point").is_some()
                    || inst.parameters.get("reasoning_metadata").is_some()
                {
                    decision_points_seen += 1;
                }

                tracker.record_item_with_domain(&decision.domain, decision.target_time_ms, false, decision.difficulty_level);

                // Update learner skill state
                if let Some(state) = skill_states.get_mut(&decision.skill_id) {
                    state.total_attempts += 1;
                    state.successful_attempts += 1;
                    state.consecutive_successes += 1;
                    state.last_success_at = Some(1000 + day as i64 * 86400);
                    state.last_practiced_at = Some(1000 + day as i64 * 86400);
                    if state.successful_attempts >= 2 {
                        state.practice_state = PracticeProgressionState::Fluent;
                    }
                    if state.successful_attempts >= 4 {
                        state.practice_state = PracticeProgressionState::Variation;
                    }
                    if state.successful_attempts >= 6 {
                        state.practice_state = PracticeProgressionState::Transfer;
                    }
                    if state.successful_attempts >= 8 {
                        state.practice_state = PracticeProgressionState::Mastered;
                    }
                    state.recent_attempts.push(RecentAttemptRecord {
                        is_correct: true,
                        score: 1.0,
                        latency_ms: 25_000,
                        target_latency_ms: 35_000,
                        variant: Some(format!("var_{}", generated_instances)),
                        variant_category: Some(VariantCategory::Structural),
                        error_category: None,
                        max_hint_level: None,
                        hint_count: Some(0),
                        independence: Some(IndependenceLevel::Independent),
                        solution_graph_fingerprint: Some(format!("sg_{}", generated_instances)),
                        cognitive_decision_correct: Some(true),
                        timestamp: 1000 + day as i64 * 86400,
                    });
                    if state.recent_attempts.len() > 10 {
                        state.recent_attempts.remove(0);
                    }
                }
            } else {
                break;
            }
        }
    }

    assert!(generated_instances >= 500, "Must generate over 500 practice items across 180 days (got {})", generated_instances);
    assert!(decision_points_seen >= 100, "Must actively traverse cognitive decision points in procedural practice (got {})", decision_points_seen);
}
