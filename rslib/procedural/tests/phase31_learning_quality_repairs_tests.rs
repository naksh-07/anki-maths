// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use procedural::core::{AttemptId, Domain, SchemaId, SkillId};
use procedural::diagnostics::ErrorCategory;
use procedural::problems::steps::{
    DiagnosticConfidence, StepErrorType, StepEvaluation, StepGraphEvaluation, StepValidationStatus,
};
use procedural::reasoning::generators::{
    BloodRelationsGenerator, FloorGridGenerator, LogicDagGenerator, SeatingGenerator,
};
use procedural::reasoning::models::{ReasoningProblemMetadata, SchemaKind, StrategyKind};
use procedural::remediation::{
    RemediationAction, RemediationActionKind, RemediationContext, RemediationPolicy,
    RemediationQueue, RemediationUrgency,
};
use procedural::skills::domain_evidence::{
    ChemistryEvidence, VersionedDomainEvidence,
};
use procedural::skills::signals::{
    IndependenceLevel, PracticeProgressionState, RecentAttemptRecord,
};

// =========================================================================
// 1. REASONING REPRESENTATION SCAFFOLDING & FADING TESTS
// =========================================================================

#[test]
fn test_reasoning_metadata_serialization_and_backward_compatibility() {
    let meta = ReasoningProblemMetadata::new(SchemaKind::LogicDag, StrategyKind::DirectSyllogisticDeduction)
        .with_scaffolding_level(2)
        .with_constraint_density(1.5)
        .with_branching_factor(2)
        .with_trap_density(0.25);

    let json = serde_json::to_string(&meta).unwrap();
    assert!(json.contains(r#""scaffolding_level":2"#));
    assert!(json.contains(r#""constraint_density":1.5"#));

    // Deserialization with full fields
    let deserialized: ReasoningProblemMetadata = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.scaffolding_level, 2);
    assert_eq!(deserialized.constraint_density, 1.5);
    assert_eq!(deserialized.branching_factor, 2);
    assert_eq!(deserialized.trap_density, 0.25);

    // Backward compatibility: old JSON without new fields
    let old_json = r#"{
        "schema_kind": "logic_dag",
        "strategy_kind": "direct_syllogistic_deduction",
        "decision_points": [],
        "constraint_count": 3,
        "search_depth": 1,
        "is_unambiguous": true,
        "is_strategy_drill": false
    }"#;
    let old_deserialized: ReasoningProblemMetadata = serde_json::from_str(old_json).unwrap();
    assert_eq!(old_deserialized.scaffolding_level, 0);
    assert_eq!(old_deserialized.constraint_density, 0.0);
    assert_eq!(old_deserialized.branching_factor, 0);
    assert_eq!(old_deserialized.trap_density, 0.0);
}

#[test]
fn test_logic_dag_scaffolding_progression_and_fading() {
    let p_l1 = LogicDagGenerator::generate_problem(1001, 1, None);
    let p_l2 = LogicDagGenerator::generate_problem(1001, 2, None);
    let p_l3 = LogicDagGenerator::generate_problem(1001, 3, None);
    let p_l4 = LogicDagGenerator::generate_problem(1001, 4, None);
    let p_l5 = LogicDagGenerator::generate_problem(1001, 5, None);

    // L1 contains explicit scaffold
    assert!(p_l1.rendered_prompt.contains("Explicit Scaffold"));
    assert!(p_l1.rendered_prompt.contains("Premise Structure"));
    let meta_l1: ReasoningProblemMetadata =
        serde_json::from_value(p_l1.parameters["reasoning_metadata"].clone()).unwrap();
    assert_eq!(meta_l1.scaffolding_level, 2);

    // L2 contains partial scaffold
    assert!(p_l2.rendered_prompt.contains("Partial Scaffold"));
    let meta_l2: ReasoningProblemMetadata =
        serde_json::from_value(p_l2.parameters["reasoning_metadata"].clone()).unwrap();
    assert_eq!(meta_l2.scaffolding_level, 1);

    // L3-L5 fade scaffolds
    assert!(!p_l3.rendered_prompt.contains("Scaffold"));
    assert!(!p_l4.rendered_prompt.contains("Scaffold"));
    assert!(!p_l5.rendered_prompt.contains("Scaffold"));

    let meta_l3: ReasoningProblemMetadata =
        serde_json::from_value(p_l3.parameters["reasoning_metadata"].clone()).unwrap();
    let meta_l5: ReasoningProblemMetadata =
        serde_json::from_value(p_l5.parameters["reasoning_metadata"].clone()).unwrap();
    assert_eq!(meta_l3.scaffolding_level, 0);
    assert_eq!(meta_l5.scaffolding_level, 0);
}

#[test]
fn test_floor_grid_scaffolding_progression_and_fading() {
    let p_l1 = FloorGridGenerator::generate_problem(2001, 1, None);
    let p_l2 = FloorGridGenerator::generate_problem(2001, 2, None);
    let p_l3 = FloorGridGenerator::generate_problem(2001, 3, None);
    let p_l5 = FloorGridGenerator::generate_problem(2001, 5, None);

    // L1 contains explicit floor slots scaffold
    assert!(p_l1.rendered_prompt.contains("Floor Layout (Explicit Scaffold)"));
    assert!(p_l1.rendered_prompt.contains("Floor 1: ["));

    // L2 contains partial slots scaffold
    assert!(p_l2.rendered_prompt.contains("Floor Layout (Partial Scaffold)"));

    // L3 and L5 fade scaffold
    assert!(!p_l3.rendered_prompt.contains("Scaffold"));
    assert!(!p_l5.rendered_prompt.contains("Scaffold"));
}

#[test]
fn test_seating_scaffolding_progression_and_fading() {
    let p_l1 = SeatingGenerator::generate_problem(3001, 1, None);
    let p_l2 = SeatingGenerator::generate_problem(3001, 2, None);
    let p_l3 = SeatingGenerator::generate_problem(3001, 3, None);
    let p_l5 = SeatingGenerator::generate_problem(3001, 5, None);

    // L1 contains explicit position slots scaffold
    assert!(p_l1.rendered_prompt.contains("Arrangement (Explicit Scaffold)"));
    assert!(p_l1.rendered_prompt.contains("Pos 1:"));

    // L2 contains partial position slots scaffold
    assert!(p_l2.rendered_prompt.contains("Arrangement (Partial Scaffold)"));

    // L3 and L5 fade scaffold
    assert!(!p_l3.rendered_prompt.contains("Scaffold"));
    assert!(!p_l5.rendered_prompt.contains("Scaffold"));
}

#[test]
fn test_blood_relations_scaffolding_progression_and_fading() {
    let p_l1 = BloodRelationsGenerator::generate_problem(4001, 1, None);
    let p_l2 = BloodRelationsGenerator::generate_problem(4001, 2, None);
    let p_l3 = BloodRelationsGenerator::generate_problem(4001, 3, None);

    // L1 contains generational tree scaffold
    assert!(p_l1.rendered_prompt.contains("Kinship Graph (Explicit Scaffold)"));
    assert!(p_l1.rendered_prompt.contains("Generation +1"));

    // L2 contains relational bridge scaffold
    assert!(p_l2.rendered_prompt.contains("Relational Bridge (Partial Scaffold)"));

    // L3 fades scaffold
    assert!(!p_l3.rendered_prompt.contains("Scaffold"));
}

// =========================================================================
// 2. CHEMISTRY INTERMEDIATE-STEP EVIDENCE TESTS
// =========================================================================

#[test]
fn test_chemistry_intermediate_quantity_evidence_serialization() {
    let chem_ev = ChemistryEvidence::Physical {
        model_setup: Some(true),
        equation_selection: Some(true),
        intermediate_quantity: Some(true),
        calculation: Some(false),
        conservation: Some(true),
        verification: Some(true),
        transfer: None,
    };
    let domain_ev = VersionedDomainEvidence::new_chemistry(chem_ev);

    assert!(domain_ev.is_execution_error());
    assert!(!domain_ev.is_conceptual_error());
    assert!(!domain_ev.is_intermediate_error());

    let json = serde_json::to_string(&domain_ev).unwrap();
    assert!(json.contains(r#""intermediate_quantity":true"#));
    assert!(json.contains(r#""calculation":false"#));

    let deserialized: VersionedDomainEvidence = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, domain_ev);
}

#[test]
fn test_chemistry_intermediate_step_evaluation_diagnostics() {
    // Case 1: Intermediate step 1 is VALID, final step 2 is INVALID (Arithmetic slip on final answer)
    let eval_final_slip = StepGraphEvaluation {
        is_correct: false,
        score: 0.5,
        first_error_step: Some(1),
        first_error_type: Some(StepErrorType::ArithmeticError),
        confidence: DiagnosticConfidence::Deterministic,
        steps_completed: 2,
        steps_correct: 1,
        step_evaluations: vec![
            StepEvaluation {
                step_id: "step_intermediate_moles".to_string(),
                step_index: 0,
                status: StepValidationStatus::Valid,
                submitted_text: "0.25 mol".to_string(),
                expected_expression: "0.25".to_string(),
                parsed_value: Some(0.25),
                error_type: None,
                confidence: DiagnosticConfidence::Deterministic,
                feedback: Some("✓ Correct intermediate moles".to_string()),
                is_downstream_consistent: false,
            },
            StepEvaluation {
                step_id: "step_final_mass".to_string(),
                step_index: 1,
                status: StepValidationStatus::Invalid,
                submitted_text: "15.0 g".to_string(),
                expected_expression: "14.2 g".to_string(),
                parsed_value: Some(15.0),
                error_type: Some(StepErrorType::ArithmeticError),
                confidence: DiagnosticConfidence::Deterministic,
                feedback: Some("Arithmetic calculation slip".to_string()),
                is_downstream_consistent: false,
            },
        ],
        overall_feedback: "Intermediate step correct, final arithmetic slip".to_string(),
        remediation_recommendation: Some("remediate:simpler_numbers_variant".to_string()),
        first_action_latency_ms: Some(12000),
        step_latencies_ms: vec![12000, 8000],
    };

    let chem_ev1 = eval_final_slip.to_chemistry_physical_evidence().unwrap();
    match &chem_ev1 {
        ChemistryEvidence::Physical {
            model_setup,
            intermediate_quantity,
            calculation,
            ..
        } => {
            assert_eq!(*model_setup, Some(true));
            assert_eq!(*intermediate_quantity, Some(true));
            assert_eq!(*calculation, Some(false));
        }
        _ => panic!("Expected Physical variant"),
    }
    let ver_ev1 = VersionedDomainEvidence::new_chemistry(chem_ev1);
    assert!(!ver_ev1.is_intermediate_error());
    assert!(ver_ev1.is_execution_error());

    // Case 2: Intermediate step 1 is INVALID (Intermediate conversion failure)
    let eval_intermediate_fail = StepGraphEvaluation {
        is_correct: false,
        score: 0.0,
        first_error_step: Some(0),
        first_error_type: Some(StepErrorType::StoichiometricRatioError),
        confidence: DiagnosticConfidence::Deterministic,
        steps_completed: 1,
        steps_correct: 0,
        step_evaluations: vec![StepEvaluation {
            step_id: "step_intermediate_moles".to_string(),
            step_index: 0,
            status: StepValidationStatus::Invalid,
            submitted_text: "0.50 mol".to_string(),
            expected_expression: "0.25".to_string(),
            parsed_value: Some(0.50),
            error_type: Some(StepErrorType::StoichiometricRatioError),
            confidence: DiagnosticConfidence::Deterministic,
            feedback: Some("Inverted stoichiometric ratio".to_string()),
            is_downstream_consistent: false,
        }],
        overall_feedback: "Failed at intermediate ratio step".to_string(),
        remediation_recommendation: Some("remediate:ratio_mapping_drill".to_string()),
        first_action_latency_ms: Some(15000),
        step_latencies_ms: vec![15000],
    };

    let chem_ev2 = eval_intermediate_fail.to_chemistry_physical_evidence().unwrap();
    match &chem_ev2 {
        ChemistryEvidence::Physical {
            model_setup,
            intermediate_quantity,
            calculation,
            ..
        } => {
            assert_eq!(*model_setup, Some(true));
            assert_eq!(*intermediate_quantity, Some(false));
            assert_eq!(*calculation, Some(false));
        }
        _ => panic!("Expected Physical variant"),
    }
    let ver_ev2 = VersionedDomainEvidence::new_chemistry(chem_ev2);
    assert!(ver_ev2.is_intermediate_error());
}

#[test]
fn test_chemistry_remediation_routing_by_intermediate_evidence() {
    let attempts = vec![RecentAttemptRecord {
        is_correct: false,
        score: 0.0,
        latency_ms: 25000,
        target_latency_ms: 30000,
        variant: None,
        variant_category: None,
        error_category: Some(ErrorCategory::Calculation),
        max_hint_level: None,
        hint_count: None,
        independence: None,
        solution_graph_fingerprint: None,
        cognitive_decision_correct: None,
        domain_evidence: None,
        timestamp: 1000,
    }];

    // Scenario A: Intermediate was correct, final arithmetic slipped -> ProceduralVariant with simpler_numbers
    let chem_slip = ChemistryEvidence::Physical {
        model_setup: Some(true),
        equation_selection: Some(true),
        intermediate_quantity: Some(true),
        calculation: Some(false),
        conservation: Some(true),
        verification: Some(true),
        transfer: None,
    };
    let dom_ev_slip = VersionedDomainEvidence::new_chemistry(chem_slip);
    let mut attempts_slip = attempts.clone();
    attempts_slip[0].domain_evidence = Some(dom_ev_slip.clone());

    let ctx_slip = RemediationContext {
        skill_id: &SkillId::from("chemistry.stoichiometry"),
        schema_id: &SchemaId::from("schema.chemistry.stoich"),
        domain: Domain::Chemistry,
        primary_error: ErrorCategory::Calculation,
        step_error: Some(StepErrorType::ArithmeticError),
        decision_point_correct: Some(true),
        independence: IndependenceLevel::Independent,
        progression_state: PracticeProgressionState::Learning,
        recent_attempts: &attempts_slip,
        source_attempt_id: &AttemptId::new("attempt-chem-slip"),
        recurrence_count: 1,
        is_transfer_attempt: false,
    };
    let action_slip = RemediationPolicy::evaluate(&ctx_slip);
    assert_eq!(action_slip.kind, RemediationActionKind::ProceduralVariant);
    assert_eq!(action_slip.preferred_variant, Some("simpler_numbers".to_string()));

    // Scenario B: Intermediate conversion was wrong -> ProceduralVariant with guided_steps
    let chem_inter = ChemistryEvidence::Physical {
        model_setup: Some(true),
        equation_selection: Some(true),
        intermediate_quantity: Some(false),
        calculation: Some(false),
        conservation: Some(true),
        verification: Some(false),
        transfer: None,
    };
    let dom_ev_inter = VersionedDomainEvidence::new_chemistry(chem_inter);
    let mut attempts_inter = attempts.clone();
    attempts_inter[0].domain_evidence = Some(dom_ev_inter.clone());

    let ctx_inter = RemediationContext {
        skill_id: &SkillId::from("chemistry.stoichiometry"),
        schema_id: &SchemaId::from("schema.chemistry.stoich"),
        domain: Domain::Chemistry,
        primary_error: ErrorCategory::Calculation,
        step_error: Some(StepErrorType::StoichiometricRatioError),
        decision_point_correct: Some(true),
        independence: IndependenceLevel::Independent,
        progression_state: PracticeProgressionState::Learning,
        recent_attempts: &attempts_inter,
        source_attempt_id: &AttemptId::new("attempt-chem-inter"),
        recurrence_count: 1,
        is_transfer_attempt: false,
    };
    let action_inter = RemediationPolicy::evaluate(&ctx_inter);
    assert_eq!(action_inter.kind, RemediationActionKind::ProceduralVariant);
    assert_eq!(action_inter.preferred_variant, Some("guided_steps".to_string()));

    // Scenario C: Setup / Conservation was wrong -> StrategyDrill
    let chem_setup = ChemistryEvidence::Physical {
        model_setup: Some(false),
        equation_selection: Some(false),
        intermediate_quantity: Some(false),
        calculation: Some(false),
        conservation: Some(false),
        verification: Some(false),
        transfer: None,
    };
    let dom_ev_setup = VersionedDomainEvidence::new_chemistry(chem_setup);
    let mut attempts_setup = attempts.clone();
    attempts_setup[0].domain_evidence = Some(dom_ev_setup.clone());

    let ctx_setup = RemediationContext {
        skill_id: &SkillId::from("chemistry.stoichiometry"),
        schema_id: &SchemaId::from("schema.chemistry.stoich"),
        domain: Domain::Chemistry,
        primary_error: ErrorCategory::Concept,
        step_error: Some(StepErrorType::ChemicalRepresentationError),
        decision_point_correct: Some(false),
        independence: IndependenceLevel::Independent,
        progression_state: PracticeProgressionState::Learning,
        recent_attempts: &attempts_setup,
        source_attempt_id: &AttemptId::new("attempt-chem-setup"),
        recurrence_count: 1,
        is_transfer_attempt: false,
    };
    let action_setup = RemediationPolicy::evaluate(&ctx_setup);
    assert_eq!(action_setup.kind, RemediationActionKind::StrategyDrill);
}

// =========================================================================
// 3. REMEDIATION QUEUE COMPACTION TESTS
// =========================================================================

#[test]
fn test_remediation_queue_same_skill_compaction() {
    let mut queue = RemediationQueue::new();
    let skill_a = SkillId::from("chemistry.stoichiometry");
    let schema_a = SchemaId::from("schema.stoichiometry.1");

    // Action 1: Procedural Variant (Normal urgency, rec 1, tier 20)
    let action1 = RemediationAction::new(
        "act-1",
        RemediationActionKind::ProceduralVariant,
        &skill_a,
        &schema_a,
        Domain::Chemistry,
        ErrorCategory::Calculation,
        AttemptId::new("att-1"),
        "Calculation slip",
    );
    queue.enqueue(action1);
    assert_eq!(queue.len(), 1);

    // Action 2: Strategy Drill on the SAME skill (Normal urgency, rec 2, tier 60)
    let action2 = RemediationAction::new(
        "act-2",
        RemediationActionKind::StrategyDrill,
        &skill_a,
        &schema_a,
        Domain::Chemistry,
        ErrorCategory::Strategy,
        AttemptId::new("att-2"),
        "Strategy drill needed",
    );
    queue.enqueue(action2);

    // Must be compacted into 1 authoritative item
    assert_eq!(queue.len(), 1);
    let top = queue.pending_actions.first().unwrap();
    assert_eq!(top.skill_id, skill_a);
    assert_eq!(top.kind, RemediationActionKind::StrategyDrill); // Higher tier preserved!
    assert_eq!(top.recurrence_count, 2); // Recurrence preserved!

    // Action 3: Circuit Breaker on the SAME skill (Critical urgency, rec 3, tier 90)
    let mut action3 = RemediationAction::new(
        "act-3",
        RemediationActionKind::CircuitBreaker,
        &skill_a,
        &schema_a,
        Domain::Chemistry,
        ErrorCategory::Concept,
        AttemptId::new("att-3"),
        "Circuit breaker triggered",
    );
    action3.urgency = RemediationUrgency::Critical;
    queue.enqueue(action3);

    assert_eq!(queue.len(), 1);
    let top_final = queue.pending_actions.first().unwrap();
    assert_eq!(top_final.kind, RemediationActionKind::CircuitBreaker);
    assert_eq!(top_final.urgency, RemediationUrgency::Critical);
    assert_eq!(top_final.recurrence_count, 3);
}

#[test]
fn test_remediation_queue_distinct_skill_isolation() {
    let mut queue = RemediationQueue::new();

    let skill_math = SkillId::from("math.divisibility");
    let skill_chem = SkillId::from("chemistry.stoichiometry");
    let skill_phys = SkillId::from("physics.kinematics");
    let skill_reas = SkillId::from("reasoning.logic_dag");

    let act_math = RemediationAction::new(
        "act-math",
        RemediationActionKind::ProceduralVariant,
        &skill_math,
        SchemaId::from("schema.math.1"),
        Domain::Mathematics,
        ErrorCategory::Calculation,
        AttemptId::new("att-m"),
        "Math calculation",
    );

    let act_chem = RemediationAction::new(
        "act-chem",
        RemediationActionKind::StrategyDrill,
        &skill_chem,
        SchemaId::from("schema.chem.1"),
        Domain::Chemistry,
        ErrorCategory::Strategy,
        AttemptId::new("att-c"),
        "Chem strategy",
    );

    let mut act_phys = RemediationAction::new(
        "act-phys",
        RemediationActionKind::CircuitBreaker,
        &skill_phys,
        SchemaId::from("schema.phys.1"),
        Domain::Physics,
        ErrorCategory::Concept,
        AttemptId::new("att-p"),
        "Phys circuit breaker",
    );
    act_phys.urgency = RemediationUrgency::Critical;

    let act_reas = RemediationAction::new(
        "act-reas",
        RemediationActionKind::RepresentationDrill,
        &skill_reas,
        SchemaId::from("schema.reas.1"),
        Domain::Reasoning,
        ErrorCategory::Strategy,
        AttemptId::new("att-r"),
        "Reasoning scaffold",
    );

    queue.enqueue(act_math);
    queue.enqueue(act_chem);
    queue.enqueue(act_phys);
    queue.enqueue(act_reas);

    // 4 distinct skills must remain distinct in queue
    assert_eq!(queue.len(), 4);

    // Critical urgency physics must be ordered first
    let first = queue.pending_actions.first().unwrap();
    assert_eq!(first.skill_id, skill_phys);
    assert_eq!(first.urgency, RemediationUrgency::Critical);

    // Resolve math
    queue.record_resolution(&skill_math, &ErrorCategory::Calculation);
    assert_eq!(queue.len(), 3);
    assert!(queue.pending_actions.iter().all(|a| a.skill_id != skill_math));
}

#[test]
fn test_remediation_queue_oscillation_depth_bound() {
    let mut queue = RemediationQueue::new();
    let num_skills = 32;

    // Simulate 500 failed attempts with high oscillation across 32 skills and random error categories
    let error_categories = [
        ErrorCategory::Calculation,
        ErrorCategory::Concept,
        ErrorCategory::Strategy,
        ErrorCategory::Unit,
        ErrorCategory::Sign,
    ];

    for attempt_idx in 0..500 {
        let skill_idx = attempt_idx % num_skills;
        let err_idx = (attempt_idx / 3) % error_categories.len();
        let skill_id = SkillId::from(format!("skill.domain.{}", skill_idx));
        let schema_id = SchemaId::from(format!("schema.domain.{}", skill_idx));
        let err = error_categories[err_idx].clone();

        let action_kind = match err_idx {
            0 => RemediationActionKind::ProceduralVariant,
            1 => RemediationActionKind::ConceptCheck,
            2 => RemediationActionKind::StrategyDrill,
            3 => RemediationActionKind::WorkedExample,
            _ => RemediationActionKind::RepresentationDrill,
        };

        let action = RemediationAction::new(
            format!("act-{}", attempt_idx),
            action_kind,
            &skill_id,
            &schema_id,
            Domain::Mathematics,
            err,
            AttemptId::new(format!("att-{}", attempt_idx)),
            "Oscillation attempt",
        );

        queue.enqueue(action);
    }

    // With 32 canonical skills, the compacted queue MUST be strictly <= 32!
    assert!(queue.len() <= num_skills);
    assert_eq!(queue.len(), num_skills);
}
