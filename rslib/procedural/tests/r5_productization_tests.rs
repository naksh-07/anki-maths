// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use chrono::Utc;
use procedural::core::{Domain, ExamProfileId};
use procedural::diagnostics::ErrorCategory;
use procedural::exam::{
    ContentProvenance, ExamObjective, ExamProfile, MockBlueprint, MockFollowUpEngine,
    MockQuestionItem, MockSession, PyqMasteryAction, PyqMasteryBridge,
};
use procedural::practice::{
    PracticeAttempt, PracticeObjective, PracticeRequest, PracticeScope, RemediationPrecedence,
};
use procedural::problems::catalog::MathsCatalog;
use procedural::problems::generators::{
    LinearEquationsGenerator, PercentageSuccessiveConfig, PercentageSuccessiveGenerator,
};
use procedural::problems::ProblemInstance;
use procedural::remediation::{
    ConceptCheckObject, ConceptCheckOption, DeclarativeRecallBridge, StrategyDrillObject,
    StrategyOption, WorkedExampleObject,
};
use procedural::reviewer::render_reviewer_html;
use procedural::scheduling::PracticeSessionObject;
use procedural::scheduling::Rating;
use procedural::service::ProceduralService;
use procedural::skills::SkillState;

#[test]
fn test_r5_native_reviewer_rendering_matrix() {
    // 1. Maths ConceptCheck
    let math_schema = MathsCatalog::successive_percentage_schema();
    let cc = ConceptCheckObject::new(
        "cc-math-01",
        math_schema.skill_id.clone(),
        math_schema.id.clone(),
        Domain::Mathematics,
        "What is the effective change for +20% followed by -20%?",
        vec![
            ConceptCheckOption::new("opt-1", "-4%", true, "effective_formula", "Correct: 20 - 20 - 400/100 = -4%"),
            ConceptCheckOption::new("opt-2", "0%", false, "zero_sum_trap", "Incorrect: Percentages apply to changing base"),
        ],
        "opt-1",
        "Successive percentages multiply factors: 1.2 * 0.8 = 0.96 (-4%).",
    );

    let mut cc_instance = PercentageSuccessiveGenerator::generate_instance(
        &math_schema.problem_family_id,
        501,
        &PercentageSuccessiveConfig::default(),
    );
    cc_instance.rendered_prompt = cc.prompt.clone();
    cc_instance.metadata = serde_json::json!({
        "object_type": "concept_check",
        "concept_check": cc,
        "remediation_message": "💡 Concept Reinforcement: Verify successive multiplier principle."
    });

    let cc_session = PracticeSessionObject::new(math_schema.clone(), cc_instance, Some(1), None);
    let cc_html = render_reviewer_html(&cc_session);
    assert!(cc_html.contains("proc-option-group"));
    assert!(cc_html.contains("data-opt-id=\"opt-1\""));
    assert!(cc_html.contains("💡 Concept Reinforcement"));

    // 2. Physics StrategyDrill
    let phys_schema = MathsCatalog::kinematics_schema();
    let sd = StrategyDrillObject::new(
        "sd-phys-01",
        phys_schema.skill_id.clone(),
        phys_schema.id.clone(),
        Domain::Physics,
        "A rocket accelerates upwards with constant fuel burn. Which kinematics model applies?",
        "Rocket starting from rest with continuous acceleration",
        vec![
            StrategyOption::new("opt-1", "Constant acceleration model (v = u + at)", "const_accel", true, "Optimal under constant thrust."),
            StrategyOption::new("opt-2", "Energy conservation without work", "no_work_trap", false, "Suboptimal: Thrust does work."),
        ],
        "opt-1",
        "With constant acceleration, standard equations of motion apply directly.",
    );

    let mut sd_instance = ProblemInstance::new(
        procedural::core::ProblemInstanceId::new("inst-sd-01"),
        phys_schema.problem_family_id.clone(),
        1,
        serde_json::json!({}),
        sd.prompt.clone(),
        serde_json::json!({"preferred_option_id": "opt-1"}),
    );
    sd_instance.metadata = serde_json::json!({
        "object_type": "strategy_drill",
        "strategy_drill": sd,
        "remediation_message": "🧭 Strategy Selection: Identify the governing physical model."
    });

    let sd_session = PracticeSessionObject::new(phys_schema.clone(), sd_instance, Some(2), None);
    let sd_html = render_reviewer_html(&sd_session);
    assert!(sd_html.contains("proc-option-group"));
    assert!(sd_html.contains("Rocket starting from rest"));
    assert!(sd_html.contains("🧭 Strategy Selection"));

    // 3. Chemistry WorkedExample
    let chem_schema = MathsCatalog::stoichiometry_schema();
    let we = WorkedExampleObject::new(
        "we-chem-01",
        chem_schema.skill_id.clone(),
        chem_schema.id.clone(),
        Domain::Chemistry,
        "Calculate mass of CO2 produced by complete combustion of 16g CH4.",
        "Combustion reaction: CH4 + 2O2 -> CO2 + 2H2O",
        vec![
            "Step 1: Calculate moles of CH4 = 16g / 16g/mol = 1.0 mol.".into(),
            "Step 2: Stoichiometric ratio CH4 : CO2 is 1 : 1, yielding 1.0 mol CO2.".into(),
            "Step 3: Mass of CO2 = 1.0 mol * 44g/mol = 44g.".into(),
        ],
        "Convert given mass to moles before applying stoichiometric coefficients",
        "Direct mass-to-mass stoichiometry requires mole intermediary.",
        vec!["Multiplying mass directly by stoichiometric coefficients".into()],
    );

    let mut we_instance = ProblemInstance::new(
        procedural::core::ProblemInstanceId::new("inst-we-01"),
        chem_schema.problem_family_id.clone(),
        1,
        serde_json::json!({}),
        we.prompt.clone(),
        serde_json::json!({}),
    );
    we_instance.metadata = serde_json::json!({
        "object_type": "worked_example",
        "worked_example": we,
        "remediation_message": "📖 Step-by-Step Method Breakdown"
    });

    let we_session = PracticeSessionObject::new(chem_schema.clone(), we_instance, Some(3), None);
    let we_html = render_reviewer_html(&we_session);
    assert!(we_html.contains("proc-worked-example-card"));
    assert!(we_html.contains("proc-try-similar-btn"));
    assert!(we_html.contains("Convert given mass to moles before applying stoichiometric coefficients"));
    assert!(we_html.contains("proc-pitfall-box"));

    // 4. Reasoning DeclarativeRecall
    let reason_schema = MathsCatalog::syllogism_schema();
    let dr = DeclarativeRecallBridge::new(
        "dr-reason-01",
        reason_schema.skill_id.clone(),
        Domain::Reasoning,
        "Universal Affirmative Inference",
        "Standard syllogistic conversion for 'All A are B'",
        "'All A are B' does NOT imply 'All B are A' (valid converse is 'Some B are A')",
    );

    let mut dr_instance = ProblemInstance::new(
        procedural::core::ProblemInstanceId::new("inst-dr-01"),
        reason_schema.problem_family_id.clone(),
        1,
        serde_json::json!({}),
        "Recall Universal Affirmative Conversion",
        serde_json::json!({}),
    );
    dr_instance.metadata = serde_json::json!({
        "object_type": "declarative_recall",
        "declarative_recall": dr,
        "remediation_message": "🧠 Memory Bridge: Review formal logic rule."
    });

    let dr_session = PracticeSessionObject::new(reason_schema.clone(), dr_instance, Some(4), None);
    let dr_html = render_reviewer_html(&dr_session);
    assert!(dr_html.contains("proc-anki-recall-btn"));
    assert!(dr_html.contains("Universal Affirmative Inference"));
    assert!(dr_html.contains("All A are B"));
}

#[test]
fn test_r5_mock_session_full_lifecycle_and_actionable_follow_up() {
    let profile = ExamProfile::new(
        ExamProfileId::new("exam-gate-cs"),
        "GATE CS",
        "Graduate Aptitude Test in Engineering",
        vec![Domain::Mathematics, Domain::Reasoning],
        ExamObjective::ComprehensiveMock,
    );

    let blueprint = MockBlueprint::from_exam_profile(&profile, 6, 600_000);
    assert_eq!(blueprint.total_questions, 6);

    let math_schema = MathsCatalog::linear_equations_schema();
    let reason_schema = MathsCatalog::series_schema();

    let mut questions = Vec::new();
    for i in 0..6 {
        let is_math = i < 3;
        let schema = if is_math { &math_schema } else { &reason_schema };
        let domain = if is_math { Domain::Mathematics } else { Domain::Reasoning };
        let inst = LinearEquationsGenerator::generate_problem(200 + i as u64, 2, None);

        questions.push(MockQuestionItem {
            question_index: i,
            schema_id: schema.id.clone(),
            skill_id: schema.skill_id.clone(),
            domain,
            schema_title: schema.title.clone(),
            instance: inst,
            difficulty_level: 2,
            target_time_ms: 40_000,
            is_pyq: i % 2 == 0,
            provenance: Some(ContentProvenance::new_direct_procedural(1, 1, 1, "practice_variant", 200 + i as u64)),
        });
    }

    let mut mock = MockSession::new("mock-session-gate-01", blueprint, questions);
    assert!(!mock.is_submitted);

    // Answer Q0 correctly (20s)
    let ans0 = mock.questions[0].instance.correct_answer["formatted"].as_str().unwrap_or("x = 5").to_string();
    mock.record_answer(0, ans0, 20_000);

    // Answer Q1 incorrectly (50s - slow)
    mock.record_answer(1, "wrong_ans", 50_000);

    // Answer Q2 incorrectly (30s)
    mock.record_answer(2, "wrong_ans", 30_000);

    // Answer Q3 correctly (15s)
    let ans3 = mock.questions[3].instance.correct_answer["formatted"].as_str().unwrap_or("10").to_string();
    mock.record_answer(3, ans3, 15_000);

    // Q4 and Q5 unanswered

    // Submit mock
    let scoring = mock.submit(Utc::now().timestamp_millis());

    assert_eq!(scoring.total_questions, 6);
    assert_eq!(scoring.answered_count, 4);
    assert_eq!(scoring.unanswered_count, 2);
    assert_eq!(scoring.correct_count, 2);
    assert_eq!(scoring.incorrect_count, 2);

    // Score calculation: 2 * 1.0 - 2 * 0.25 = 1.5 out of 6.0 = 25%
    assert!((scoring.raw_score - 1.5).abs() < 1e-6);
    assert!((scoring.percentage - 25.0).abs() < 1e-6);
    assert_eq!(scoring.accuracy, 50.0);

    // Generate actionable learning follow-up
    let follow_up = MockFollowUpEngine::generate_follow_up_request(&scoring);
    assert_eq!(follow_up.objective, PracticeObjective::Practice);
    assert_eq!(follow_up.remediation_policy, RemediationPrecedence::AllEligible);
    match follow_up.scope {
        PracticeScope::MultipleSchemas(schemas) => {
            assert!(schemas.contains(&math_schema.id));
        }
        _ => panic!("Expected MultipleSchemas scope targeting diagnosed weak areas"),
    }
}

#[test]
fn test_r5_pyq_review_policy_and_familiarity_ceiling() {
    let attempt_1 = PracticeAttempt::new(
        "att.pyq.1",
        "inst.pyq.1",
        "math.ratio",
        "skill.ratio",
        serde_json::json!({"answer": 4}),
        true,
        1.0,
        25_000,
    );

    let state = SkillState::new("skill.ratio");

    // 1st authentic success -> requires isomorphic variant confirmation
    let action_1 = PyqMasteryBridge::evaluate_pyq_attempt(&attempt_1, &state, None, Rating::Good, 1);
    match action_1 {
        PyqMasteryAction::VariantConfirmationRequired { suggested_variant_type, target_success_count, .. } => {
            assert_eq!(suggested_variant_type, "isomorphic");
            assert_eq!(target_success_count, 2);
        }
        _ => panic!("Expected VariantConfirmationRequired on 1st exposure"),
    }

    // 4th authentic success without variant validation -> triggers FamiliarityCapReached
    let action_4 = PyqMasteryBridge::evaluate_pyq_attempt(&attempt_1, &state, None, Rating::Good, 4);
    match action_4 {
        PyqMasteryAction::FamiliarityCapReached { authentic_exposures, suggested_variant_type, .. } => {
            assert_eq!(authentic_exposures, 4);
            assert_eq!(suggested_variant_type, "structural_variant");
        }
        _ => panic!("Expected FamiliarityCapReached on repeated authentic exposure"),
    }

    // Failure on authentic PYQ -> triggers targeted foundational remediation
    let attempt_fail = PracticeAttempt::new(
        "att.pyq.fail",
        "inst.pyq.fail",
        "math.ratio",
        "skill.ratio",
        serde_json::json!({"answer": 99}),
        false,
        0.0,
        45_000,
    );
    let action_fail = PyqMasteryBridge::evaluate_pyq_attempt(
        &attempt_fail,
        &state,
        Some(&ErrorCategory::Concept),
        Rating::Again,
        1,
    );
    match action_fail {
        PyqMasteryAction::TargetedRemediationRequired { remediation_difficulty, primary_error, .. } => {
            assert_eq!(remediation_difficulty, 1);
            assert_eq!(primary_error, Some(ErrorCategory::Concept));
        }
        _ => panic!("Expected TargetedRemediationRequired on PYQ failure"),
    }
}

#[test]
fn test_r5_user_intent_authoritative_precedence_in_service() {
    let service = ProceduralService::open_in_memory().unwrap();

    // User explicitly requests single schema focus on Physics kinematics
    let phys_schema = MathsCatalog::kinematics_schema();
    let focused_request = PracticeRequest::new(
        PracticeScope::SingleSchema(phys_schema.id.clone()),
        PracticeObjective::Practice,
    )
    .with_exact_difficulty(4)
    .with_target_latency_ms(35_000);

    let session = service
        .prepare_unified_practice_session(&focused_request, None, None, Some(999))
        .expect("Selection should succeed");

    assert_eq!(session.schema.id, phys_schema.id);
    assert_eq!(session.difficulty_level, Some(4));
    assert_eq!(session.target_latency_ms, Some(35_000));
}