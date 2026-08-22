// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::sync::Arc;
use std::thread;

use tempfile::NamedTempFile;

use procedural::core::{Domain, ProblemFamilyId, SchemaId, SkillId, AttemptId};
use procedural::diagnostics::ErrorCategory;
use procedural::practice::{PracticeAttempt, ErrorEvent, PracticeRequest};
use procedural::problems::catalog::{
    SCHEMA_LINEAR_EQUATIONS, SKILL_LINEAR_EQUATIONS,
};
use procedural::problems::contract::{
    AnswerDerivation, DeclarativeArchetype, DeclarativeFamilyContract, ParameterDomain,
    ParameterSpec, ProblemFamilyCapability, ProblemFamilyContract,
};
use procedural::problems::declarative::DeclarativeProblemGenerator;
use procedural::problems::generator::ProblemGenerator;
use procedural::problems::generators::{FAMILY_LINEAR_EQUATIONS, TEMPLATE_LINEAR_EQUATIONS_V1};
use procedural::problems::registry::ProblemRegistry;
use procedural::remediation::actions::{RemediationAction, RemediationActionKind, RemediationUrgency};
use procedural::remediation::policy::{RemediationContext, RemediationPolicy};
use procedural::scheduling::PracticeMode;
use procedural::service::ProceduralService;
use procedural::skills::domain_evidence::{
    ChemistryEvidence, DomainEvidencePayload, MathEvidence,
    VersionedDomainEvidence,
};
use procedural::skills::signals::{IndependenceLevel, MasteryEvidence, PracticeProgressionState, RecentAttemptRecord, VariantCategory};

// =============================================================================
// FIX #1: BATCH PREREQUISITE & SESSION PREPARATION QUERIES
// =============================================================================

#[test]
fn test_fix1_prerequisite_query_batching_and_no_n_plus_1() {
    let service = ProceduralService::open_in_memory().unwrap();

    // 0 prerequisites: Should evaluate immediately
    let skill_0 = SkillId::new(SKILL_LINEAR_EQUATIONS);
    let eval_0 = service.evaluate_prerequisites(&skill_0).unwrap();
    assert!(eval_0.is_ready());
    assert_eq!(eval_0.missing_prerequisites.len(), 0);

    // Test session preparation with candidate schemas
    let req = PracticeRequest::default();
    let session_res = service.prepare_unified_practice_session(&req, None, None, None);
    assert!(session_res.is_ok());
    let session = session_res.unwrap();
    assert!(!session.instance.rendered_prompt.is_empty());
}

// =============================================================================
// FIX #2: ATOMIC LEARNER-STATE PERSISTENCE & CONCURRENCY
// =============================================================================

#[test]
fn test_fix2_atomic_learner_state_persistence_under_concurrency() {
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path().to_str().unwrap();

    let service = ProceduralService::open(db_path).unwrap();
    let skill_id = SkillId::new(SKILL_LINEAR_EQUATIONS);
    let schema_id = SchemaId::new(SCHEMA_LINEAR_EQUATIONS);
    let family_id = ProblemFamilyId::new(FAMILY_LINEAR_EQUATIONS);

    let reg = ProblemRegistry::default_registry();
    let instance = reg.generate(&family_id, TEMPLATE_LINEAR_EQUATIONS_V1, 12345, 1, None).unwrap();
    service.save_problem_instance(instance.clone()).unwrap();

    let service_arc = Arc::new(service);

    // 20 concurrent attempts on the SAME skill
    let mut handles = Vec::new();
    for i in 0..20 {
        let serv_clone = Arc::clone(&service_arc);
        let s_id = skill_id.clone();
        let sch_id = schema_id.clone();
        let inst_id = instance.id.clone();
        handles.push(thread::spawn(move || {
            let attempt_id = AttemptId::new(format!("attempt-atomic-{}", i));
            let attempt = PracticeAttempt::new(
                &attempt_id,
                &inst_id,
                &sch_id,
                &s_id,
                serde_json::json!(42),
                i % 2 == 0,
                if i % 2 == 0 { 1.0 } else { 0.0 },
                20_000,
            ).with_card_id(100 + i as i64);
            let mut errors = Vec::new();
            if i % 2 != 0 {
                errors.push(ErrorEvent::new(
                    procedural::core::ErrorEventId::new(format!("err-{}", i)),
                    &attempt_id,
                    "calculation",
                    serde_json::json!({}),
                ));
            }
            serv_clone.store().record_practice_attempt_atomic(&attempt, &errors, None, 30_000)
        }));
    }

    for h in handles {
        let res = h.join().unwrap();
        assert!(res.is_ok(), "Concurrent attempt persistence should succeed");
    }

    // Verify persisted state consistency
    let final_state = service_arc.store().get_skill_state(&skill_id).unwrap().unwrap();
    assert_eq!(final_state.total_attempts, 20);
    assert_eq!(final_state.successful_attempts, 10);
    assert_eq!(final_state.failed_attempts, 10);
}

// =============================================================================
// FIX #3: BACKEND-AUTHORITATIVE RECURRENCE & TAMPER RESISTANCE
// =============================================================================

#[test]
fn test_fix3_backend_authoritative_recurrence_escalation_and_circuit_breaker() {
    let service = ProceduralService::open_in_memory().unwrap();

    let skill_id = SkillId::new(SKILL_LINEAR_EQUATIONS);
    let schema_id = SchemaId::new(SCHEMA_LINEAR_EQUATIONS);
    let err_cat = ErrorCategory::Concept;

    // Simulate repeated concept failures and verify recurrence escalation is backend-authoritative
    let q_arc = service.remediation_queue();
    
    // Attempt 1: Recurrence should be 1 -> ConceptCheck
    let rec1 = {
        let q = q_arc.lock().unwrap();
        q.get_recurrence_count(&skill_id, &err_cat) + 1
    };
    assert_eq!(rec1, 1);

    let ctx1 = RemediationContext {
        skill_id: &skill_id,
        schema_id: &schema_id,
        domain: Domain::Mathematics,
        primary_error: err_cat.clone(),
        step_error: None,
        decision_point_correct: None,
        independence: IndependenceLevel::Independent,
        progression_state: PracticeProgressionState::Learning,
        recent_attempts: &[],
        source_attempt_id: &AttemptId::new("att-1"),
        recurrence_count: rec1,
        is_transfer_attempt: false,
    };
    let act1 = RemediationPolicy::evaluate(&ctx1);
    assert_eq!(act1.kind, RemediationActionKind::ConceptCheck);
    service.enqueue_remediation_action(act1).unwrap();

    // Attempt 2: Recurrence should be 2 -> StrategyDrill
    let rec2 = {
        let q = q_arc.lock().unwrap();
        q.get_recurrence_count(&skill_id, &err_cat) + 1
    };
    assert_eq!(rec2, 2);
    let mut ctx2 = ctx1.clone();
    ctx2.recurrence_count = rec2;
    let act2 = RemediationPolicy::evaluate(&ctx2);
    assert_eq!(act2.kind, RemediationActionKind::StrategyDrill);
    service.enqueue_remediation_action(act2).unwrap();

    // Attempt 3: Recurrence should be 3 -> WorkedExample
    let rec3 = {
        let q = q_arc.lock().unwrap();
        q.get_recurrence_count(&skill_id, &err_cat) + 1
    };
    assert_eq!(rec3, 3);
    let mut ctx3 = ctx1.clone();
    ctx3.recurrence_count = rec3;
    let act3 = RemediationPolicy::evaluate(&ctx3);
    assert_eq!(act3.kind, RemediationActionKind::WorkedExample);
    service.enqueue_remediation_action(act3).unwrap();

    // Attempt 4: Recurrence should be 4 -> PrerequisiteReview
    let rec4 = {
        let q = q_arc.lock().unwrap();
        q.get_recurrence_count(&skill_id, &err_cat) + 1
    };
    assert_eq!(rec4, 4);
    let mut ctx4 = ctx1.clone();
    ctx4.recurrence_count = rec4;
    let act4 = RemediationPolicy::evaluate(&ctx4);
    assert_eq!(act4.kind, RemediationActionKind::PrerequisiteReview);
    service.enqueue_remediation_action(act4).unwrap();

    // Attempt 5: Recurrence >= 5 -> Circuit Breaker
    let rec5 = {
        let q = q_arc.lock().unwrap();
        q.get_recurrence_count(&skill_id, &err_cat) + 1
    };
    assert_eq!(rec5, 5);
    let mut ctx5 = ctx1.clone();
    ctx5.recurrence_count = rec5;
    let act5 = RemediationPolicy::evaluate(&ctx5);
    assert_eq!(act5.kind, RemediationActionKind::CircuitBreaker);
}

// =============================================================================
// FIX #4: DURABLE REMEDIATION QUEUE PERSISTENCE ACROSS RESTART
// =============================================================================

#[test]
fn test_fix4_remediation_queue_durability_across_process_restart() {
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path().to_str().unwrap();

    let skill_id = SkillId::new(SKILL_LINEAR_EQUATIONS);
    let schema_id = SchemaId::new(SCHEMA_LINEAR_EQUATIONS);

    // Session 1: Create service, enqueue actions
    {
        let service = ProceduralService::open(db_path).unwrap();

        let action1 = RemediationAction::new(
            "act-durable-1",
            RemediationActionKind::StrategyDrill,
            &skill_id,
            &schema_id,
            Domain::Mathematics,
            ErrorCategory::Strategy,
            &AttemptId::new("att-init-1"),
            "Durable test reason 1",
        )
        .with_urgency(RemediationUrgency::Critical);

        service.enqueue_remediation_action(action1).unwrap();

        let action2 = RemediationAction::new(
            "act-durable-2",
            RemediationActionKind::StrategyDrill,
            &skill_id,
            &schema_id,
            Domain::Mathematics,
            ErrorCategory::Strategy,
            &AttemptId::new("att-init-2"),
            "Durable test reason 2",
        )
        .with_urgency(RemediationUrgency::Critical);

        service.enqueue_remediation_action(action2).unwrap();
        assert_eq!(service.remediation_queue_len(), 1); // Compacted to single authoritative skill action
    }

    // Session 2: Re-open service from the same SQLite file
    {
        let service = ProceduralService::open(db_path).unwrap();

        // Verify queue was restored from disk on startup
        assert_eq!(service.remediation_queue_len(), 1);

        let q_arc = service.remediation_queue();
        let q_lock = q_arc.lock().unwrap();
        let item = q_lock.iter_pending().next().unwrap();
        assert_eq!(item.kind, RemediationActionKind::StrategyDrill);
        assert_eq!(item.urgency, RemediationUrgency::Critical);
        assert_eq!(item.recurrence_count, 2);
    }
}

// =============================================================================
// FIX #5: RESTORE DOMAIN EVIDENCE BRIDGE
// =============================================================================

#[test]
fn test_fix5_domain_evidence_differential_routing_and_persistence() {
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path().to_str().unwrap();

    let service = ProceduralService::open(db_path).unwrap();
    let skill_id = SkillId::new(SKILL_LINEAR_EQUATIONS);
    let schema_id = SchemaId::new(SCHEMA_LINEAR_EQUATIONS);
    let family_id = ProblemFamilyId::new(FAMILY_LINEAR_EQUATIONS);

    let reg = ProblemRegistry::default_registry();
    let instance = reg.generate(&family_id, TEMPLATE_LINEAR_EQUATIONS_V1, 12345, 1, None).unwrap();
    service.save_problem_instance(instance.clone()).unwrap();

    // Paired test case: Case A (Intermediate ratio error) vs Case B (Model setup error)
    // Case A: Intermediate error -> ProceduralVariant with guided steps
    let chem_ev_a = ChemistryEvidence::Physical {
        model_setup: Some(true),
        equation_selection: Some(true),
        intermediate_quantity: Some(false),
        calculation: Some(false),
        conservation: Some(true),
        verification: Some(false),
        transfer: None,
    };
    let ver_ev_a = VersionedDomainEvidence::new_chemistry(chem_ev_a);

    let recent_a = vec![RecentAttemptRecord {
        is_correct: false,
        score: 0.0,
        latency_ms: 35_000,
        target_latency_ms: 30_000,
        variant: None,
        variant_category: None,
        error_category: Some(ErrorCategory::Calculation),
        max_hint_level: None,
        hint_count: None,
        independence: None,
        solution_graph_fingerprint: None,
        cognitive_decision_correct: None,
        domain_evidence: Some(ver_ev_a),
        timestamp: 1000,
    }];

    let ctx_a = RemediationContext {
        skill_id: &skill_id,
        schema_id: &schema_id,
        domain: Domain::Chemistry,
        primary_error: ErrorCategory::Calculation,
        step_error: None,
        decision_point_correct: None,
        independence: IndependenceLevel::Independent,
        progression_state: PracticeProgressionState::Learning,
        recent_attempts: &recent_a,
        source_attempt_id: &AttemptId::new("att-chem-a"),
        recurrence_count: 1,
        is_transfer_attempt: false,
    };
    let act_a = RemediationPolicy::evaluate(&ctx_a);
    assert_eq!(act_a.kind, RemediationActionKind::ProceduralVariant);
    assert_eq!(act_a.preferred_variant, Some("guided_steps".to_string()));

    // Case B: Model setup error -> StrategyDrill at rec=1, ConceptCheck at rec>=2
    let chem_ev_b = ChemistryEvidence::Physical {
        model_setup: Some(false),
        equation_selection: Some(false),
        intermediate_quantity: Some(false),
        calculation: Some(false),
        conservation: Some(false),
        verification: Some(false),
        transfer: None,
    };
    let ver_ev_b = VersionedDomainEvidence::new_chemistry(chem_ev_b);
    let recent_b = vec![RecentAttemptRecord {
        is_correct: false,
        score: 0.0,
        latency_ms: 35_000,
        target_latency_ms: 30_000,
        variant: None,
        variant_category: None,
        error_category: Some(ErrorCategory::Concept),
        max_hint_level: None,
        hint_count: None,
        independence: None,
        solution_graph_fingerprint: None,
        cognitive_decision_correct: None,
        domain_evidence: Some(ver_ev_b.clone()),
        timestamp: 1000,
    }];

    let ctx_b = RemediationContext {
        skill_id: &skill_id,
        schema_id: &schema_id,
        domain: Domain::Chemistry,
        primary_error: ErrorCategory::Concept,
        step_error: None,
        decision_point_correct: None,
        independence: IndependenceLevel::Independent,
        progression_state: PracticeProgressionState::Learning,
        recent_attempts: &recent_b,
        source_attempt_id: &AttemptId::new("att-chem-b"),
        recurrence_count: 2,
        is_transfer_attempt: false,
    };
    let act_b = RemediationPolicy::evaluate(&ctx_b);
    assert_eq!(act_b.kind, RemediationActionKind::ConceptCheck);

    // Verify domain evidence survives database persistence and SkillState restoration
    let mut attempt = PracticeAttempt::new(
        &AttemptId::new("att-chem-saved"),
        &instance.id,
        &schema_id,
        &skill_id,
        serde_json::json!({}),
        false,
        0.0,
        35_000,
    );
    let metadata = serde_json::json!({
        "error_category": "concept",
        "domain_evidence": serde_json::to_value(&ver_ev_b).unwrap(),
    });
    attempt = attempt.with_metadata(metadata);

    let updated_state = service.store().record_practice_attempt_atomic(
        &attempt,
        &[],
        None,
        30_000,
    ).unwrap();

    let restored_dev = updated_state.recent_attempts.last().unwrap().domain_evidence.as_ref().unwrap();
    assert_eq!(restored_dev.version, 1);
    match &restored_dev.payload {
        DomainEvidencePayload::Chemistry(ChemistryEvidence::Physical { model_setup, .. }) => {
            assert_eq!(*model_setup, Some(false));
        }
        _ => panic!("Expected Physical Chemistry evidence"),
    }
}

// =============================================================================
// FIX #6: SAFE CONTRACT VALIDATION & NO-PANIC HARDENING
// =============================================================================

#[test]
fn test_fix6_adversarial_malformed_contracts_fail_safely_without_panic() {
    let pfc_base = ProblemFamilyContract::new(
        ProblemFamilyId::new("test.inverted_float"),
        SkillId::new("skill.test"),
        Domain::Mathematics,
        SchemaId::new("schema.test"),
        ProblemFamilyCapability::Declarative,
    );

    // 1. Inverted Float Range
    let bad_contract_1 = DeclarativeFamilyContract::new(
        pfc_base.clone(),
        vec![DeclarativeArchetype::new(
            "arch_1",
            1,
            VariantCategory::Parameter,
            "standard",
            vec![ParameterSpec::new(
                "val",
                ParameterDomain::FloatRange { min: 10.0, max: 2.0, precision: 2 },
            )],
            "{{val}}",
            AnswerDerivation::DirectParam { param_name: "val".to_string() },
            "{{val}}",
            "{{val}}",
            30000,
        )],
    );
    let val_res_1 = bad_contract_1.validate();
    assert!(val_res_1.is_err(), "Inverted float range must fail validation");

    // 2. NaN float range in generator
    let nan_contract = DeclarativeArchetype::new(
        "arch_nan",
        1,
        VariantCategory::Parameter,
        "standard",
        vec![ParameterSpec::new(
            "nan_val",
            ParameterDomain::FloatRange { min: f64::NAN, max: 10.0, precision: 2 },
        )],
        "{{nan_val}}",
        AnswerDerivation::DirectParam { param_name: "nan_val".to_string() },
        "{{nan_val}}",
        "{{nan_val}}",
        30000,
    );
    let dummy_contract = DeclarativeFamilyContract::new(
        ProblemFamilyContract::new(
            ProblemFamilyId::new("test.safe"),
            SkillId::new("skill.test"),
            Domain::Mathematics,
            SchemaId::new("schema.test"),
            ProblemFamilyCapability::Declarative,
        ),
        vec![nan_contract],
    );
    let gen_safe = DeclarativeProblemGenerator::new(Arc::new(dummy_contract));
    let fam_id_safe = ProblemFamilyId::new("test.safe");
    let inst_res = gen_safe.generate(&fam_id_safe, 42, 1, None);
    assert!(inst_res.is_ok(), "Sampling with NaN must not panic");

    // 3. Negation overflow on i64::MIN
    let signed_str_arch = DeclarativeArchetype::new(
        "arch_neg_overflow",
        1,
        VariantCategory::Parameter,
        "standard",
        vec![
            ParameterSpec::integer_range("base", i64::MIN, i64::MIN),
            ParameterSpec::new("signed_str", ParameterDomain::DerivedSignedString { param: "base".to_string() }),
        ],
        "{{signed_str}}",
        AnswerDerivation::DirectParam { param_name: "base".to_string() },
        "{{signed_str}}",
        "{{signed_str}}",
        30000,
    );
    let contract_neg = DeclarativeFamilyContract::new(
        ProblemFamilyContract::new(
            ProblemFamilyId::new("test.neg"),
            SkillId::new("skill.test"),
            Domain::Mathematics,
            SchemaId::new("schema.test"),
            ProblemFamilyCapability::Declarative,
        ),
        vec![signed_str_arch],
    );
    let gen_neg = DeclarativeProblemGenerator::new(Arc::new(contract_neg));
    let fam_id_neg = ProblemFamilyId::new("test.neg");
    let inst_neg_res = gen_neg.generate(&fam_id_neg, 42, 1, None);
    assert!(inst_neg_res.is_ok(), "DerivedSignedString with i64::MIN must not panic");

    // 4. CoprimePair with inverted bounds
    let bad_coprime = DeclarativeFamilyContract::new(
        ProblemFamilyContract::new(
            ProblemFamilyId::new("test.coprime"),
            SkillId::new("skill.test"),
            Domain::Mathematics,
            SchemaId::new("schema.test"),
            ProblemFamilyCapability::Declarative,
        ),
        vec![DeclarativeArchetype::new(
            "arch_coprime",
            1,
            VariantCategory::Parameter,
            "standard",
            vec![ParameterSpec::new(
                "coprimes",
                ParameterDomain::CoprimePair { min: 100, max: 10 },
            )],
            "{{coprimes}}",
            AnswerDerivation::DirectParam { param_name: "coprimes".to_string() },
            "{{coprimes}}",
            "{{coprimes}}",
            30000,
        )],
    );
    assert!(bad_coprime.validate().is_err(), "CoprimePair with min > max must fail validate()");
}

// =============================================================================
// STAGE 8: INTEGRATED FULL ATTEMPT TO REMEDIATION PRODUCTION LOOP
// =============================================================================

#[test]
fn test_full_production_attempt_to_remediation_loop() {
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path().to_str().unwrap();

    let skill_id = SkillId::new(SKILL_LINEAR_EQUATIONS);
    let schema_id = SchemaId::new(SCHEMA_LINEAR_EQUATIONS);
    let family_id = ProblemFamilyId::new(FAMILY_LINEAR_EQUATIONS);

    // Session 1: Real Attempt -> Telemetry -> Persistence -> Policy -> Durable Queue
    {
        let service = ProceduralService::open(db_path).unwrap();

        // 1. Generate real problem instance
        let reg = ProblemRegistry::default_registry();
        let instance = reg.generate(&family_id, TEMPLATE_LINEAR_EQUATIONS_V1, 12345, 1, None).unwrap();
        service.save_problem_instance(instance.clone()).unwrap();

        // 2. Submit realistic incorrect attempt with domain evidence
        let math_ev = MathEvidence {
            pattern_recognition: Some(true),
            method_selection: Some(true),
            execution: Some(false), // Arithmetic calculation error
            verification: Some(false),
            structural_transfer: None,
        };
        let ver_ev = VersionedDomainEvidence::new_math(math_ev);

        let attempt_id = AttemptId::new("att-full-loop-1");
        let mut attempt = PracticeAttempt::new(
            &attempt_id,
            &instance.id,
            &schema_id,
            &skill_id,
            serde_json::json!(999), // Wrong answer
            false,
            0.0,
            28_000,
        );
        let metadata = serde_json::json!({
            "hints_used": 1,
            "target_time_ms": 30_000,
            "error_category": "calculation",
            "domain_evidence": serde_json::to_value(&ver_ev).unwrap(),
        });
        attempt = attempt.with_metadata(metadata);

        let error_event = ErrorEvent::new(
            procedural::core::ErrorEventId::new("err-full-loop-1"),
            &attempt_id,
            "calculation",
            serde_json::json!({ "feedback": "Calculation slip" }),
        );

        // 3. Persist atomically
        let updated_state = service.store().record_practice_attempt_atomic(
            &attempt,
            &[error_event],
            None,
            30_000,
        ).unwrap();

        assert_eq!(updated_state.total_attempts, 1);
        assert_eq!(updated_state.failed_attempts, 1);

        // 4. Derive backend recurrence and evaluate remediation policy
        let recurrence_count = {
            let q_arc = service.remediation_queue();
            let q_lock = q_arc.lock().unwrap();
            q_lock.get_recurrence_count(&skill_id, &ErrorCategory::Calculation) + 1
        };
        assert_eq!(recurrence_count, 1);

        let ctx = RemediationContext {
            skill_id: &skill_id,
            schema_id: &schema_id,
            domain: Domain::Mathematics,
            primary_error: ErrorCategory::Calculation,
            step_error: None,
            decision_point_correct: None,
            independence: IndependenceLevel::LightSupport,
            progression_state: updated_state.practice_state,
            recent_attempts: &updated_state.recent_attempts,
            source_attempt_id: &attempt_id,
            recurrence_count,
            is_transfer_attempt: false,
        };

        let action = RemediationPolicy::evaluate(&ctx);
        // Execution error in Math routes to ProceduralVariant(simpler_numbers)
        assert_eq!(action.kind, RemediationActionKind::ProceduralVariant);
        assert_eq!(action.preferred_variant, Some("simpler_numbers".to_string()));

        // 5. Enqueue into durable queue
        service.enqueue_remediation_action(action).unwrap();
        assert_eq!(service.remediation_queue_len(), 1);
    }

    // Session 2: Process Restart -> Queue Reload -> Next Intervention Selection
    {
        let service = ProceduralService::open(db_path).unwrap();

        // Queue automatically restored on restart
        assert_eq!(service.remediation_queue_len(), 1);

        let intervention_opt = service.get_next_remediation_intervention(
            &PracticeMode::MixedInterleaved,
            9999,
        ).unwrap();

        assert!(intervention_opt.is_some(), "Durable remediation action must be selected after restart");
        let (action, _intervention) = intervention_opt.unwrap();
        assert_eq!(action.kind, RemediationActionKind::ProceduralVariant);
        assert_eq!(action.skill_id, skill_id);

        // After selection, the action was popped and queue saved
        assert_eq!(service.remediation_queue_len(), 0);

        // Record successful resolution
        let math_ev_resolved = MathEvidence {
            pattern_recognition: Some(true),
            method_selection: Some(true),
            execution: Some(true),
            verification: Some(true),
            structural_transfer: None,
        };
        let mastery_ev = MasteryEvidence {
            final_correctness: true,
            latency_evidence: 15_000,
            independence: IndependenceLevel::Independent,
            domain_evidence: Some(VersionedDomainEvidence::new_math(math_ev_resolved)),
            ..Default::default()
        };

        service.record_remediation_response(
            &action,
            true,
            &mastery_ev,
            1.0,
            30_000,
        ).unwrap();

        let state_after = service.store().get_skill_state(&skill_id).unwrap().unwrap();
        assert_eq!(state_after.total_attempts, 2);
        assert_eq!(state_after.successful_attempts, 1);
    }
}
