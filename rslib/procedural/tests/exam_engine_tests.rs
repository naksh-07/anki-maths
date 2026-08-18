// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use procedural::{
    Domain, ErrorCategory, ExamObjective, ExamPracticeMode, ExamProfile, ExamProfileId,
    ExamRelevanceScorer, PracticeAttempt, ProceduralService, PyqMapping, PyqMasteryAction,
    PyqMasteryBridge, PYQSource, Rating, SchemaId, SchemaPracticeObject, SkillId, SkillState,
    SCHEMA_SUCCESSIVE_PERCENTAGE, SCHEMA_TIME_SPEED_DISTANCE, SCHEMA_TIME_WORK,
};

#[test]
fn test_exam_profiles_serialization_and_canonical_blueprints() {
    // 1. RRB ALP
    let rrb = ExamProfile::rrb_alp();
    assert_eq!(rrb.id.as_str(), "rrb_alp");
    assert_eq!(rrb.subjects.len(), 4);
    assert_eq!(rrb.get_domain_weight(&Domain::Mathematics), 0.35);
    assert_eq!(rrb.get_domain_weight(&Domain::Reasoning), 0.35);
    assert_eq!(rrb.get_domain_weight(&Domain::Physics), 0.20);
    assert_eq!(rrb.get_domain_weight(&Domain::Chemistry), 0.10);
    assert_eq!(rrb.objective, ExamObjective::SpeedAndAccuracy);

    // 2. SSC CGL
    let ssc = ExamProfile::ssc_cgl();
    assert_eq!(ssc.id.as_str(), "ssc_cgl");
    assert_eq!(ssc.get_domain_weight(&Domain::Mathematics), 0.40);
    assert_eq!(ssc.get_domain_weight(&Domain::Reasoning), 0.40);

    // 3. Banking PO
    let bank = ExamProfile::banking_po();
    assert_eq!(bank.id.as_str(), "banking_po");
    assert_eq!(bank.subjects, vec![Domain::Reasoning, Domain::Mathematics]);
    assert_eq!(bank.get_domain_weight(&Domain::Reasoning), 0.50);

    // 4. JEE Main
    let jee = ExamProfile::jee_main_foundation();
    assert_eq!(jee.id.as_str(), "jee_main_foundation");
    assert_eq!(jee.get_domain_weight(&Domain::Physics), 0.35);
    assert_eq!(jee.get_domain_weight(&Domain::Chemistry), 0.35);
    assert_eq!(jee.get_domain_weight(&Domain::Mathematics), 0.30);
    assert_eq!(jee.objective, ExamObjective::ConceptMastery);

    // JSON serialization roundtrip
    let json_str = serde_json::to_string(&rrb).unwrap();
    let deserialized: ExamProfile = serde_json::from_str(&json_str).unwrap();
    assert_eq!(deserialized.id, rrb.id);
    assert_eq!(deserialized.name, rrb.name);
}

#[test]
fn test_exam_relevance_scorer_and_topic_weighting() {
    let profile = ExamProfile::rrb_alp();
    let schema = SchemaPracticeObject::new(
        SCHEMA_TIME_SPEED_DISTANCE,
        "arithmetic.time_speed_distance",
        "family.math.arithmetic.time_speed_distance",
        "Time Speed Distance",
        "Train problems",
    );

    // 1. Unpracticed skill (cold start) -> should have mastery urgency bonus
    let score_cold = ExamRelevanceScorer::calculate_score(
        &profile,
        &schema,
        &Domain::Mathematics,
        None,
        true,
        false,
        &ExamPracticeMode::ExamPreparation,
    );
    assert!(score_cold.total_score > 200.0);
    assert!(score_cold.pyq_presence_bonus > 0.0);

    // 2. High mastery skill -> lower mastery urgency
    let mut state_mastered = SkillState::new("arithmetic.time_speed_distance");
    state_mastered.mastery = 0.95;
    state_mastered.total_attempts = 10;
    state_mastered.successful_attempts = 10;

    let score_mastered = ExamRelevanceScorer::calculate_score(
        &profile,
        &schema,
        &Domain::Mathematics,
        Some(&state_mastered),
        true,
        false,
        &ExamPracticeMode::ExamPreparation,
    );
    assert!(score_mastered.total_score < score_cold.total_score);

    // 3. Struggling skill with recent errors -> error urgency bonus
    let mut state_struggling = SkillState::new("arithmetic.time_speed_distance");
    state_struggling.mastery = 0.25;
    state_struggling.total_attempts = 5;
    state_struggling.consecutive_failures = 3;
    state_struggling.record_attempt_outcome(false, 0.0, 50_000, 35_000, None, Some(&ErrorCategory::Concept), 1000);

    let score_struggling = ExamRelevanceScorer::calculate_score(
        &profile,
        &schema,
        &Domain::Mathematics,
        Some(&state_struggling),
        true,
        false,
        &ExamPracticeMode::ExamPreparation,
    );
    assert!(score_struggling.total_score > score_mastered.total_score);
    assert!(score_struggling.error_urgency_component > 50.0);
}

#[test]
fn test_exam_practice_modes_and_selection() {
    let service = ProceduralService::open_in_memory().unwrap();
    service.init_default_exam_profiles().unwrap();

    let profile_id = ExamProfileId::new("rrb_alp");

    // Ingest authentic PYQ for Time Speed Distance
    let pyq = PYQSource::new(
        "pyq.rrb.tsd.01",
        "RRB ALP",
        2018,
        Domain::Mathematics,
        "A train crosses a pole in 9 seconds with speed 72 km/h. Find length of train.",
        serde_json::json!({ "length_m": 180 }),
        "RRB ALP 2018 Shift 1",
    );
    let mapping = PyqMapping::new(
        &pyq.id,
        Domain::Mathematics,
        "arithmetic.time_speed_distance",
        SCHEMA_TIME_SPEED_DISTANCE,
        "family.math.arithmetic.time_speed_distance",
        2,
        35_000,
    )
    .with_status(procedural::MappingStatus::Verified);

    service.ingest_pyq(pyq, Some(mapping)).unwrap();

    // 1. ExamPreparation Mode
    let session_prep = service
        .prepare_exam_practice_session(&profile_id, &ExamPracticeMode::ExamPreparation, None, 42)
        .unwrap()
        .expect("Should prepare session");
    assert!(session_prep.target_latency_ms.is_some());
    assert!(session_prep.selection_reason.unwrap().contains("RRB Assistant Loco Pilot"));

    // 2. PyqPractice Mode
    let session_pyq = service
        .prepare_exam_practice_session(&profile_id, &ExamPracticeMode::PyqPractice, None, 42)
        .unwrap()
        .expect("Should prepare PYQ session");
    assert!(session_pyq.instance.metadata.get("is_authentic_pyq").is_some());

    // 3. SpeedTraining Mode
    let session_speed = service
        .prepare_exam_practice_session(&profile_id, &ExamPracticeMode::SpeedTraining, None, 100)
        .unwrap()
        .expect("Should prepare Speed session");
    assert!(session_speed.difficulty_level.unwrap() <= 3);

    // 4. WeakAreas Mode
    let session_weak = service
        .prepare_exam_practice_session(&profile_id, &ExamPracticeMode::WeakAreas, None, 200)
        .unwrap()
        .expect("Should prepare Weak Areas session");
    assert!(session_weak.selection_reason.unwrap().contains("WeakAreas"));
}

#[test]
fn test_pyq_mastery_bridge_progression() {
    let attempt_pass = PracticeAttempt::new(
        "att.pyq.01",
        "inst.pyq.01",
        SCHEMA_TIME_WORK,
        "time_work.basic",
        serde_json::json!({ "answer_days": 6 }),
        true,
        1.0,
        22_000,
    );

    let state = SkillState::new("time_work.basic");
    let action_pass = PyqMasteryBridge::evaluate_pyq_attempt(
        &attempt_pass,
        &state,
        None,
        Rating::Good,
    );

    // Success on authentic PYQ demands variant confirmation
    match action_pass {
        PyqMasteryAction::VariantConfirmationRequired { skill_id, suggested_variant_type, target_success_count } => {
            assert_eq!(skill_id.as_str(), "time_work.basic");
            assert_eq!(suggested_variant_type, "isomorphic");
            assert_eq!(target_success_count, 2);
        }
        _ => panic!("Expected VariantConfirmationRequired on PYQ success"),
    }

    let attempt_fail = PracticeAttempt::new(
        "att.pyq.02",
        "inst.pyq.02",
        SCHEMA_TIME_WORK,
        "time_work.basic",
        serde_json::json!({ "answer_days": 12 }),
        false,
        0.0,
        48_000,
    );

    let action_fail = PyqMasteryBridge::evaluate_pyq_attempt(
        &attempt_fail,
        &state,
        Some(&ErrorCategory::Concept),
        Rating::Again,
    );

    // Failure on authentic PYQ triggers targeted remediation
    match action_fail {
        PyqMasteryAction::TargetedRemediationRequired { skill_id, remediation_difficulty, primary_error } => {
            assert_eq!(skill_id.as_str(), "time_work.basic");
            assert_eq!(remediation_difficulty, 1);
            assert_eq!(primary_error, Some(ErrorCategory::Concept));
        }
        _ => panic!("Expected TargetedRemediationRequired on PYQ failure"),
    }
}

#[test]
fn test_failing_schemas_analytics_query() {
    let service = ProceduralService::open_in_memory().unwrap();

    let pyq = PYQSource::new(
        "pyq.analytics.01",
        "RRB ALP",
        2018,
        Domain::Mathematics,
        "Calculate successive discount of 10% and 20%",
        serde_json::json!({ "effective": 28 }),
        "RRB ALP 2018",
    );
    let mapping = PyqMapping::new(
        &pyq.id,
        Domain::Mathematics,
        "percentage.successive",
        SCHEMA_SUCCESSIVE_PERCENTAGE,
        "family.math.percentage.successive",
        2,
        30_000,
    )
    .with_status(procedural::MappingStatus::Verified);

    service.ingest_pyq(pyq, Some(mapping)).unwrap();

    let instance = procedural::ProblemInstance::new(
        "inst_1",
        "family.math.percentage.successive",
        123,
        serde_json::json!({}),
        "Prompt",
        serde_json::json!({ "ans": 28 }),
    );
    service.save_problem_instance(instance).unwrap();

    // Record 2 failed attempts and 1 success for this schema
    let attempt_fail1 = PracticeAttempt::new(
        "att_f1",
        "inst_1",
        SCHEMA_SUCCESSIVE_PERCENTAGE,
        "percentage.successive",
        serde_json::json!({ "ans": 30 }),
        false,
        0.0,
        35_000,
    );
    service.record_practice_attempt(attempt_fail1, vec![]).unwrap();

    let attempt_fail2 = PracticeAttempt::new(
        "att_f2",
        "inst_1",
        SCHEMA_SUCCESSIVE_PERCENTAGE,
        "percentage.successive",
        serde_json::json!({ "ans": 32 }),
        false,
        0.0,
        40_000,
    );
    service.record_practice_attempt(attempt_fail2, vec![]).unwrap();

    let failing_schemas = service.get_exam_failing_schemas("RRB ALP").unwrap();
    assert_eq!(failing_schemas.len(), 1);
    assert_eq!(failing_schemas[0].0.as_str(), SCHEMA_SUCCESSIVE_PERCENTAGE);
    assert_eq!(failing_schemas[0].1, 1.0); // 100% failure rate
    assert_eq!(failing_schemas[0].2, 2);   // 2 attempts
}

#[test]
fn test_end_to_end_pyq_to_exam_adaptive_learning_loop() {
    let service = ProceduralService::open_in_memory().unwrap();
    service.init_default_exam_profiles().unwrap();

    // 1. Ingest authentic PYQ
    let pyq = PYQSource::new(
        "pyq.rrb.e2e.01",
        "RRB ALP",
        2018,
        Domain::Mathematics,
        "Two pipes A and B can fill a tank in 12 hours and 18 hours. Find combined time.",
        serde_json::json!({ "combined_hours": 7.2 }),
        "RRB ALP 2018 Shift 3",
    );
    let mapping = PyqMapping::new(
        &pyq.id,
        Domain::Mathematics,
        "time_work.basic",
        SCHEMA_TIME_WORK,
        "family.math.time_work.basic",
        2,
        40_000,
    )
    .with_status(procedural::MappingStatus::Verified)
    .with_variant_structure("reverse");

    service.ingest_pyq(pyq.clone(), Some(mapping)).unwrap();

    // 2. Generate validated variant
    let variant_instance = service
        .generate_validated_pyq_variant(&pyq.id, 99999, Some("reverse"))
        .expect("Should generate validated variant");
    assert_eq!(variant_instance.seed, 99999);

    // 3. Learner practices variant and succeeds
    let attempt = PracticeAttempt::new(
        "att_e2e_01",
        &variant_instance.id,
        SCHEMA_TIME_WORK,
        "time_work.basic",
        variant_instance.correct_answer.clone(),
        true,
        1.0,
        28_000,
    );

    let mastery_action = service
        .record_pyq_practice_attempt(attempt, vec![], Some(pyq.id.clone()))
        .expect("Should record attempt");

    // 4. Verify mastery progression
    match mastery_action {
        PyqMasteryAction::VariantConfirmationRequired { skill_id, .. } => {
            assert_eq!(skill_id.as_str(), "time_work.basic");
        }
        _ => panic!("Expected VariantConfirmationRequired"),
    }

    // 5. Verify SkillState updated
    let state = service.load_skill_state(&SkillId::new("time_work.basic")).unwrap().unwrap();
    assert_eq!(state.total_attempts, 1);
    assert_eq!(state.successful_attempts, 1);
    assert!(state.mastery > 0.0);

    // 6. Select next adaptive exam session
    let next_session = service
        .prepare_exam_practice_session(
            &ExamProfileId::new("rrb_alp"),
            &ExamPracticeMode::ExamPreparation,
            Some(&SchemaId::from(SCHEMA_TIME_WORK)),
            123,
        )
        .unwrap()
        .expect("Should select next practice session");

    // Anti-priming should avoid immediately repeating SCHEMA_TIME_WORK
    assert_ne!(next_session.schema.id.as_str(), SCHEMA_TIME_WORK);
}
