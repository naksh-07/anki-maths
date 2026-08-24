// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use procedural::core::{Domain, SchemaId, SkillId};
use procedural::diagnostics::ErrorCategory;
use procedural::problems::catalog::{
    SCHEMA_CHEMISTRY_STOICHIOMETRY, SCHEMA_PHYSICS_KINEMATICS, SCHEMA_REASONING_SEATING,
    SCHEMA_SUCCESSIVE_PERCENTAGE,
};
use procedural::problems::steps::StepErrorType;
use procedural::remediation::{
    DeclarativeRecallBridge, RemediationActionKind, RemediationContext, RemediationIntervention,
    RemediationOutcomeStatus, RemediationPolicy, RemediationSelector,
};
use procedural::scheduling::PracticeMode;
use procedural::service::ProceduralService;
use procedural::skills::signals::{IndependenceLevel, MasteryEvidence, PracticeProgressionState};

#[test]
fn test_remediation_maths_end_to_end_flow() {
    let service = ProceduralService::open_in_memory().unwrap();
    let schema_id = SchemaId::new(SCHEMA_SUCCESSIVE_PERCENTAGE);
    let schema = service.resolve_schema(&schema_id).unwrap().unwrap();

    // 1. Student generates problem instance
    let inst = service
        .generate_problem(&schema.problem_family_id, 1001, &serde_json::Value::Null)
        .unwrap();
    service.save_problem_instance(inst.clone()).unwrap();

    // 2. Student fails attempt with Strategy error (e.g. additive heuristic)
    let (outcome, action_opt) = service
        .evaluate_and_remediate_attempt(
            &inst.id,
            None,
            serde_json::json!(-999.0), // wrong answer
            25000,
            0,
            1,
        )
        .unwrap();

    assert!(!outcome.is_correct);
    assert!(action_opt.is_some());
    let action = action_opt.unwrap();
    assert_eq!(action.skill_id, schema.skill_id);
    assert_eq!(service.remediation_queue_len(), 1);

    // 3. System serves remediation intervention
    let (selected_act, intervention) = service
        .get_next_remediation_intervention(&PracticeMode::MixedMaths, 42)
        .unwrap()
        .unwrap();

    assert_eq!(selected_act.id, action.id);
    match intervention {
        RemediationIntervention::StrategyDrill(drill) => {
            assert!(drill.prompt.contains("calculating the original price") || drill.prompt.contains("strategy"));
            // Student answers correctly
            let eval = drill.evaluate_choice(&drill.preferred_option_id, 5000);
            assert!(eval.is_correct);
            assert!(eval.evidence.final_correctness);
            assert_eq!(eval.evidence.decision_quality, Some(1.0));

            // Record response
            service
                .record_remediation_response(&selected_act, true, &eval.evidence, 1.0, 15000)
                .unwrap();
        }
        RemediationIntervention::ConceptCheck(cc) => {
            let eval = cc.evaluate_choice(&cc.expected_option_id, 4000);
            assert!(eval.is_correct);
            service
                .record_remediation_response(&selected_act, true, &eval.evidence, 1.0, 15000)
                .unwrap();
        }
        RemediationIntervention::ProceduralProblem(p) => {
            assert!(!p.rendered_prompt.is_empty());
            let ev = MasteryEvidence {
                final_correctness: true,
                decision_quality: Some(1.0),
                step_quality: Some(1.0),
                independence: IndependenceLevel::Independent,
                max_hint_level: None,
                hint_dependence: 0,
                retry_dependence: 0,
                variant_exposure: Some("simpler_numbers".to_string()),
                variant_category: procedural::VariantCategory::Parameter,
                solution_graph_fingerprint: None,
                cognitive_decision_correct: Some(true),
                time_since_last_ms: None,
                transfer_evidence: false,
                domain_competence_verified: Some(true),
                latency_evidence: 18000,
                diagnostic_errors: vec![], domain_evidence: None,
            };
            service
                .record_remediation_response(&selected_act, true, &ev, 1.0, 30000)
                .unwrap();
        }
        other => panic!("Unexpected intervention: {:?}", other),
    }

    // 4. Remediation resolved and queue is now empty
    assert_eq!(service.remediation_queue_len(), 0);

    // 5. Verify audit trail
    let audit_records = service.list_remediation_audit_records(5);
    assert!(!audit_records.is_empty());
    assert_eq!(audit_records[0].outcome_status, RemediationOutcomeStatus::Resolved);

    // 6. Follow-up procedural practice succeeds
    let inst2 = service
        .generate_problem(&schema.problem_family_id, 1002, &serde_json::Value::Null)
        .unwrap();
    service.save_problem_instance(inst2.clone()).unwrap();
    let val2 = inst2.correct_answer.get("value").unwrap().as_f64().unwrap();
    let outcome2 = service
        .evaluate_and_record_attempt(&inst2.id, None, serde_json::json!(val2), 20000, 0, 1)
        .unwrap();
    assert!(outcome2.is_correct);
}

#[test]
fn test_remediation_physics_end_to_end_flow() {
    let service = ProceduralService::open_in_memory().unwrap();
    let schema_id = SchemaId::new(SCHEMA_PHYSICS_KINEMATICS);
    let schema = service.resolve_schema(&schema_id).unwrap().unwrap();

    // 1. Create a failed attempt with a ModelSelectionError
    let inst = service
        .generate_problem(&schema.problem_family_id, 2001, &serde_json::Value::Null)
        .unwrap();
    service.save_problem_instance(inst.clone()).unwrap();

    // Record failure
    let ctx = RemediationContext {
        skill_id: &schema.skill_id,
        schema_id: &schema.id,
        domain: Domain::Physics,
        primary_error: ErrorCategory::Concept,
        step_error: Some(StepErrorType::ModelSelectionError),
        decision_point_correct: None,
        independence: IndependenceLevel::NonIndependent,
        progression_state: PracticeProgressionState::Learning,
        recent_attempts: &[],
        source_attempt_id: &procedural::core::AttemptId::new("att-phys-1"),
        recurrence_count: 1,
        is_transfer_attempt: false,
    };

    let action = RemediationPolicy::evaluate(&ctx);
    assert_eq!(action.kind, RemediationActionKind::ConceptCheck);

    // Select intervention
    let intervention = RemediationSelector::select_intervention(
        &action,
        service.store(),
        service.registry(),
        42,
    )
    .unwrap();

    match intervention {
        RemediationIntervention::ConceptCheck(cc) => {
            assert!(cc.prompt.contains("constant gravitational acceleration"));
            assert_eq!(cc.domain, Domain::Physics);

            // Student selects correct physical model
            let eval = cc.evaluate_choice("opt_uam", 4000);
            assert!(eval.is_correct);
            assert!(eval.chosen_option_id == "opt_uam");
            assert_eq!(eval.evidence.decision_quality, Some(1.0));
            assert_eq!(eval.evidence.independence, IndependenceLevel::Independent);

            // Record response
            service
                .record_remediation_response(&action, true, &eval.evidence, 1.0, 15000)
                .unwrap();

            let state = service.load_skill_state(&schema.skill_id).unwrap().unwrap();
            assert_eq!(state.total_attempts, 1);
            assert_eq!(state.successful_attempts, 1);
        }
        _ => panic!("Expected ConceptCheck for physics model error"),
    }
}

#[test]
fn test_remediation_chemistry_end_to_end_flow() {
    let service = ProceduralService::open_in_memory().unwrap();
    let schema_id = SchemaId::new(SCHEMA_CHEMISTRY_STOICHIOMETRY);
    let schema = service.resolve_schema(&schema_id).unwrap().unwrap();

    let ctx = RemediationContext {
        skill_id: &schema.skill_id,
        schema_id: &schema.id,
        domain: Domain::Chemistry,
        primary_error: ErrorCategory::Concept,
        step_error: Some(StepErrorType::RegimeSelectionError),
        decision_point_correct: None,
        independence: IndependenceLevel::NonIndependent,
        progression_state: PracticeProgressionState::Learning,
        recent_attempts: &[],
        source_attempt_id: &procedural::core::AttemptId::new("att-chem-1"),
        recurrence_count: 1,
        is_transfer_attempt: false,
    };

    let action = RemediationPolicy::evaluate(&ctx);
    assert_eq!(action.kind, RemediationActionKind::ConceptCheck);

    let intervention = RemediationSelector::select_intervention(
        &action,
        service.store(),
        service.registry(),
        77,
    )
    .unwrap();

    match intervention {
        RemediationIntervention::ConceptCheck(cc) => {
            assert!(cc.prompt.contains("completely consumed") || cc.prompt.contains("theoretical product yield"));
            assert_eq!(cc.domain, Domain::Chemistry);

            let eval = cc.evaluate_choice("opt_limiting", 6000);
            assert!(eval.is_correct);
            assert!(eval.feedback.contains("limiting reactant"));
        }
        _ => panic!("Expected ConceptCheck for chemistry regime error"),
    }
}

#[test]
fn test_remediation_reasoning_end_to_end_flow() {
    let service = ProceduralService::open_in_memory().unwrap();
    let schema_id = SchemaId::new(SCHEMA_REASONING_SEATING);
    let schema = service.resolve_schema(&schema_id).unwrap().unwrap();

    let ctx = RemediationContext {
        skill_id: &schema.skill_id,
        schema_id: &schema.id,
        domain: Domain::Reasoning,
        primary_error: ErrorCategory::Strategy,
        step_error: Some(StepErrorType::StrategySelectionError),
        decision_point_correct: None,
        independence: IndependenceLevel::NonIndependent,
        progression_state: PracticeProgressionState::Fluent,
        recent_attempts: &[],
        source_attempt_id: &procedural::core::AttemptId::new("att-reason-1"),
        recurrence_count: 1,
        is_transfer_attempt: false,
    };

    let action = RemediationPolicy::evaluate(&ctx);
    assert_eq!(action.kind, RemediationActionKind::StrategyDrill);

    let intervention = RemediationSelector::select_intervention(
        &action,
        service.store(),
        service.registry(),
        88,
    )
    .unwrap();

    match intervention {
        RemediationIntervention::StrategyDrill(sd) => {
            assert!(sd.prompt.contains("circular seating") || sd.prompt.contains("place first"));
            assert_eq!(sd.domain, Domain::Reasoning);

            let eval = sd.evaluate_choice("opt_anchor", 4500);
            assert!(eval.is_correct);
            assert_eq!(eval.evidence.decision_quality, Some(1.0));
        }
        _ => panic!("Expected StrategyDrill for reasoning strategy error"),
    }
}

#[test]
fn test_declarative_recall_bridge() {
    let bridge = DeclarativeRecallBridge::new(
        "dec-1",
        SkillId::new("physics.units.conversion"),
        Domain::Physics,
        "km/h to m/s factor",
        "Multiply by 5/18 to convert km/h to m/s",
        "1 km/h = (1000 m / 3600 s) = 5/18 m/s = ~0.2778 m/s",
    )
    .with_tag("procedural::physics::units");

    assert_eq!(bridge.skill_id.as_str(), "physics.units.conversion");
    assert_eq!(bridge.target_anki_tag, Some("procedural::physics::units".to_string()));
}

#[test]
fn test_remediation_maths_strategy_drill_flow() {
    let service = ProceduralService::open_in_memory().unwrap();
    let schema_id = SchemaId::new(SCHEMA_SUCCESSIVE_PERCENTAGE);
    let schema = service.resolve_schema(&schema_id).unwrap().unwrap();

    let ctx = RemediationContext {
        skill_id: &schema.skill_id,
        schema_id: &schema.id,
        domain: Domain::Mathematics,
        primary_error: ErrorCategory::Strategy,
        step_error: Some(StepErrorType::StrategySelectionError),
        decision_point_correct: None,
        independence: IndependenceLevel::NonIndependent,
        progression_state: PracticeProgressionState::Learning,
        recent_attempts: &[],
        source_attempt_id: &procedural::core::AttemptId::new("att-math-strat-1"),
        recurrence_count: 1,
        is_transfer_attempt: false,
    };

    let action = RemediationPolicy::evaluate(&ctx);
    assert_eq!(action.kind, RemediationActionKind::StrategyDrill);

    let intervention = RemediationSelector::select_intervention(
        &action,
        service.store(),
        service.registry(),
        99,
    )
    .unwrap();

    match intervention {
        RemediationIntervention::StrategyDrill(sd) => {
            assert!(sd.prompt.contains("calculating the original price"));
            assert_eq!(sd.domain, Domain::Mathematics);

            let eval = sd.evaluate_choice("opt_mult", 5000);
            assert!(eval.is_correct);
            assert_eq!(eval.evidence.decision_quality, Some(1.0));
        }
        _ => panic!("Expected StrategyDrill for maths strategy error"),
    }
}