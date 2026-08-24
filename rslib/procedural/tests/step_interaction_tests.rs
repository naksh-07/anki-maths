// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use tempfile::tempdir;

use procedural::anchor::{ProceduralCardAnchor, SeedMode};
use procedural::core::{ProblemFamilyId, Result};
use procedural::problems::generators::{
    LinearEquationsGenerator, PercentageSuccessiveConfig, PercentageSuccessiveGenerator,
    PercentageVariant, RatioGenerator,
};
use procedural::problems::steps::{
    DeterministicHintSystem, DiagnosticConfidence, MathSemanticComparator, SolutionGraph,
    StepErrorType, StepNode, StepType, StepValidationStatus, StepValidator, StepwiseSubmission,
    SubmittedStep,
};
use procedural::problems::validator::NumericAnswerParser;
use procedural::scheduling::{derive_fsrs_rating, Rating};
use procedural::service::ProceduralService;

#[test]
fn test_solution_graph_generation_and_topology_across_three_families() {
    // 1. Linear Equations (Levels 1 to 5)
    for level in 1..=5 {
        let instance = LinearEquationsGenerator::generate_problem(42 + level as u64, level, None);
        let graph = instance.solution_graph().expect("Linear equations instance must have solution graph");
        assert!(graph.validate_topology(), "Graph topology must be valid and acyclic for Level {}", level);
        assert!(!graph.steps.is_empty(), "Graph must contain steps for Level {}", level);
        assert!(graph.final_step().is_some(), "Graph must have a designated final step for Level {}", level);

        for step in &graph.steps {
            assert!(!step.hints.is_empty(), "Step {} must have deterministic hints", step.id);
            assert!(!step.title.is_empty());
            assert!(!step.expected_expression.is_empty());
        }
    }

    // 2. Successive Percentage (All Variants)
    let variants = vec![
        PercentageVariant::ForwardTwoStep,
        PercentageVariant::ReverseInitial,
        PercentageVariant::NetEquivalentChange,
        PercentageVariant::ForwardThreeStep,
    ];
    let fam_id = ProblemFamilyId::new("family.math.arithmetic.percentage_successive");
    for v in variants {
        let config = PercentageSuccessiveConfig {
            allowed_variants: Some(vec![v]),
            min_difficulty: None,
            max_difficulty: None,
        };
        let instance = PercentageSuccessiveGenerator::generate_instance(&fam_id, 1001, &config);
        let graph = instance.solution_graph().expect("Percentage problem must have solution graph");
        assert!(graph.validate_topology());
        assert!(!graph.steps.is_empty());
        assert!(graph.final_step().is_some());
    }

    // 3. Ratio (Levels 1 to 5)
    for level in 1..=5 {
        let instance = RatioGenerator::generate_problem(2002 + level as u64, level, None);
        let graph = instance.solution_graph().expect("Ratio problem must have solution graph");
        assert!(graph.validate_topology());
        assert!(!graph.steps.is_empty());
        assert!(graph.final_step().is_some());
    }
}

#[test]
fn test_math_semantic_comparator_algebraic_and_numeric_equivalence() {
    // Linear Equations Equivalence
    assert!(MathSemanticComparator::check_equation_equivalence("2x + 6 = 16", "2x = 10"));
    assert!(MathSemanticComparator::check_equation_equivalence("2x = 10", "x = 5"));
    assert!(MathSemanticComparator::check_equation_equivalence("x = 5", "5 = x"));
    assert!(MathSemanticComparator::check_equation_equivalence("3x - 4 = 11", "3x = 15"));

    // Commutative Addition
    assert!(MathSemanticComparator::check_commutative_addition("2x + 6", "6 + 2x"));
    assert!(MathSemanticComparator::check_commutative_addition("a + b + c", "c + b + a"));

    // Numeric & Fractions
    assert_eq!(NumericAnswerParser::parse_string("3/4"), Some(0.75));
    assert_eq!(NumericAnswerParser::parse_string("  $1,250.50 "), Some(1250.50));
    assert_eq!(NumericAnswerParser::parse_string("25%"), Some(25.0));

    // Multiplier Equivalence
    assert!(MathSemanticComparator::check_multiplier_equivalence("1.25", "125%"));
    assert!(MathSemanticComparator::check_multiplier_equivalence("0.20", "20%"));
}

#[test]
fn test_step_validator_error_localization_and_downstream_carryover() {
    let step1 = StepNode::new(
        "subtract_constant",
        StepType::EquationRearrangement,
        "Subtract 6 from both sides",
        "Subtract 6",
        "2x = 10",
    );
    let step2 = StepNode::new(
        "divide_coeff",
        StepType::FinalAnswer,
        "Divide by 2",
        "Divide both sides by 2",
        "x = 5",
    )
    .with_expected_value(5.0)
    .with_dependencies(vec!["subtract_constant".to_string()])
    .as_final();

    let graph = SolutionGraph::new(vec![step1, step2], "divide_coeff");

    // Case A: Perfect Submission
    let sub_perfect = StepwiseSubmission::stepwise(
        vec![
            SubmittedStep::new(0, "2x = 10", 3000),
            SubmittedStep::new(1, "x = 5", 2000),
        ],
        Some("5".to_string()),
        5000,
    );
    let eval_perfect = StepValidator::evaluate_submission(&graph, &sub_perfect, 30000);
    assert!(eval_perfect.is_correct);
    assert_eq!(eval_perfect.score, 1.0);
    assert_eq!(eval_perfect.first_error_step, None);
    assert_eq!(eval_perfect.step_evaluations[0].status, StepValidationStatus::Valid);
    assert_eq!(eval_perfect.step_evaluations[1].status, StepValidationStatus::Valid);

    // Case B: Step 1 Error with Consistent Downstream Derivation
    // Student writes 2x = 12 (instead of 2x = 10), then correctly computes x = 6 from 2x = 12
    let sub_carryover = StepwiseSubmission::stepwise(
        vec![
            SubmittedStep::new(0, "2x = 12", 4000),
            SubmittedStep::new(1, "x = 6", 3000),
        ],
        Some("6".to_string()),
        7000,
    );
    let eval_carryover = StepValidator::evaluate_submission(&graph, &sub_carryover, 30000);
    assert!(!eval_carryover.is_correct);
    assert_eq!(eval_carryover.first_error_step, Some(0));
    assert_eq!(eval_carryover.step_evaluations[0].status, StepValidationStatus::Invalid);
    assert_eq!(eval_carryover.step_evaluations[1].status, StepValidationStatus::PartiallyValid);
    assert!(eval_carryover.step_evaluations[1].is_downstream_consistent);
    assert!(eval_carryover.overall_feedback.contains("First error localized at Step 1"));

    // Case C: Sign Error Identification
    // Expected x = 5, student enters x = -5
    let (err_type, conf, feedback) = MathSemanticComparator::diagnose_step_error("x = -5", &graph.steps[1], None);
    assert_eq!(err_type, StepErrorType::SignError);
    assert_eq!(conf, DiagnosticConfidence::Deterministic);
    assert!(feedback.contains("Sign reversal detected"));
}

#[test]
fn test_deterministic_hint_system_and_rating_penalties() {
    let instance = LinearEquationsGenerator::generate_problem(5555, 2, None);
    let graph = instance.solution_graph().unwrap();

    // 1. Progressive Hints Request
    let hint1 = DeterministicHintSystem::get_next_hint(&graph, 0, 0, 0).expect("Hint 1 should exist");
    assert_eq!(hint1.hint_level, 1);
    assert_eq!(hint1.title, "Principle / Rule");

    let hint2 = DeterministicHintSystem::get_next_hint(&graph, 0, 1, 1).expect("Hint 2 should exist");
    assert_eq!(hint2.hint_level, 2);
    assert_eq!(hint2.title, "Next Operation");

    let hint3 = DeterministicHintSystem::get_next_hint(&graph, 0, 2, 2).expect("Hint 3 should exist");
    assert_eq!(hint3.hint_level, 3);
    assert_eq!(hint3.title, "Intermediate Setup");

    // 2. FSRS Rating Penalties
    let outcome_0_hints = procedural::diagnostics::ProceduralReviewOutcome::new(
        "att-h0", "schema.linear", "skill.linear", "family.linear", 1, true, 1.0, 15000, 30000, 0, 1, None,
    );
    assert_eq!(derive_fsrs_rating(&outcome_0_hints, None), Rating::Easy);

    let outcome_1_hint = procedural::diagnostics::ProceduralReviewOutcome::new(
        "att-h1", "schema.linear", "skill.linear", "family.linear", 1, true, 1.0, 15000, 30000, 1, 1, None,
    );
    assert_eq!(derive_fsrs_rating(&outcome_1_hint, None), Rating::Hard);

    let outcome_2_hints = procedural::diagnostics::ProceduralReviewOutcome::new(
        "att-h2", "schema.linear", "skill.linear", "family.linear", 1, true, 1.0, 15000, 30000, 2, 1, None,
    );
    assert_eq!(derive_fsrs_rating(&outcome_2_hints, None), Rating::Hard);

    let outcome_3_hints = procedural::diagnostics::ProceduralReviewOutcome::new(
        "att-h3", "schema.linear", "skill.linear", "family.linear", 1, true, 1.0, 15000, 30000, 3, 1, None,
    );
    assert_eq!(derive_fsrs_rating(&outcome_3_hints, None), Rating::Again);
}

#[test]
fn test_end_to_end_stepwise_service_workflow() -> Result<()> {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("procedural_steps.db");
    let service = ProceduralService::open(&db_path)?;

    let anchor = ProceduralCardAnchor::new("algebra.linear_equations").with_seed_mode(SeedMode::Fixed(9999));
    let session = service.prepare_practice_session(&anchor, Some(101))?;

    let graph = session.instance.solution_graph().expect("Session instance must have solution graph");
    assert!(!graph.steps.is_empty());

    // Submit valid stepwise solution
    let mut steps = Vec::new();
    for (i, step_node) in graph.steps.iter().enumerate() {
        steps.push(SubmittedStep::new(i, &step_node.expected_expression, 4000));
    }
    let fin_ans = graph.final_step().map(|s| s.expected_expression.clone());

    let submission = StepwiseSubmission::stepwise(steps, fin_ans, 12000);
    let outcome = service.evaluate_stepwise_attempt(&session.instance.id, Some(101), &submission)?;

    assert!(outcome.is_correct);
    assert_eq!(outcome.score, 1.0);
    assert_eq!(outcome.first_error_step, None);
    assert_eq!(outcome.steps_completed, graph.steps.len());
    assert_eq!(outcome.steps_correct, graph.steps.len());
    assert_eq!(outcome.diagnostic_confidence.as_deref(), Some("deterministic"));

    // Verify rating
    let rating = service.derive_fsrs_rating(&outcome)?;
    assert!(matches!(rating, Rating::Good | Rating::Easy));

    // Verify SkillState updated
    let skill_state = service.load_skill_state(&outcome.skill_id)?.expect("Skill state should exist");
    assert_eq!(skill_state.recent_attempts.len(), 1);
    assert!(skill_state.recent_attempts[0].is_correct);

    Ok(())
}

#[test]
fn test_backward_compatibility_final_answer_only() -> Result<()> {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("procedural_compat.db");
    let service = ProceduralService::open(&db_path)?;

    let anchor = ProceduralCardAnchor::new("percentage.successive").with_seed_mode(SeedMode::Fixed(7777));
    let session = service.prepare_practice_session(&anchor, Some(202))?;

    let target_val = session.instance.correct_answer.get("value").and_then(|v| v.as_f64()).unwrap();

    // 1. Traditional evaluate_and_record_attempt
    let outcome1 = service.evaluate_and_record_attempt(
        &session.instance.id,
        Some(202),
        serde_json::json!(target_val),
        25000,
        0,
        1,
    )?;
    assert!(outcome1.is_correct);
    assert_eq!(outcome1.score, 1.0);

    // 2. FinalAnswerOnly StepwiseSubmission
    let sub_final = StepwiseSubmission::final_answer_only(target_val.to_string(), 24000);
    let outcome2 = service.evaluate_stepwise_attempt(&session.instance.id, Some(202), &sub_final)?;
    assert!(outcome2.is_correct);
    assert_eq!(outcome2.score, 1.0);

    Ok(())
}

#[test]
fn test_reasoning_pedagogical_structures_and_step_validation() {
    use procedural::reasoning::generators::{SeatingGenerator, SyllogismGenerator};
    use procedural::skills::domain_evidence::DomainEvidencePayload;

    // 1. Seating Arrangement CSP Stepwise Validation
    let seating_inst = SeatingGenerator::generate_problem(4242, 2, None);
    let seating_graph = seating_inst.solution_graph().expect("Seating instance must have solution graph");
    assert!(seating_graph.validate_topology());
    assert_eq!(seating_graph.steps.len(), 3);
    assert_eq!(seating_graph.steps[0].step_type, StepType::ApplyConstraint);
    assert_eq!(seating_graph.steps[1].step_type, StepType::PropagateConstraint);
    assert_eq!(seating_graph.steps[2].step_type, StepType::FinalAnswer);

    // Case A: Correct Seating Stepwise Submission
    let mut sub_steps = Vec::new();
    for (i, node) in seating_graph.steps.iter().enumerate() {
        sub_steps.push(SubmittedStep::new(i, &node.expected_expression, 4000));
    }
    let fin_ans = seating_graph.final_step().map(|s| s.expected_expression.clone());
    let sub_valid = StepwiseSubmission::stepwise(sub_steps, fin_ans, 12000);
    let eval_valid = StepValidator::evaluate_submission(&seating_graph, &sub_valid, 35000);
    assert!(eval_valid.is_correct);
    assert_eq!(eval_valid.score, 1.0);
    assert_eq!(eval_valid.first_error_step, None);
    assert_eq!(eval_valid.steps_correct, 3);

    let reasoning_ev = eval_valid.to_reasoning_evidence().expect("Must generate reasoning evidence");
    assert_eq!(reasoning_ev.pattern_recognition, Some(true));
    assert_eq!(reasoning_ev.representation, Some(true));
    assert_eq!(reasoning_ev.constraint_extraction, Some(true));
    assert_eq!(reasoning_ev.decision_path, Some(true));
    assert_eq!(reasoning_ev.deduction, Some(true));

    // Case B: Constraint Violation at Step 0
    let sub_err_step0 = StepwiseSubmission::stepwise(
        vec![
            SubmittedStep::new(0, "Anchor: InvalidPerson", 3000),
            SubmittedStep::new(1, &seating_graph.steps[1].expected_expression, 3000),
            SubmittedStep::new(2, &seating_graph.steps[2].expected_expression, 3000),
        ],
        Some(seating_graph.steps[2].expected_expression.clone()),
        9000,
    );
    let eval_err0 = StepValidator::evaluate_submission(&seating_graph, &sub_err_step0, 35000);
    assert!(!eval_err0.is_correct);
    assert_eq!(eval_err0.first_error_step, Some(0));
    assert_eq!(eval_err0.first_error_type, Some(StepErrorType::ConstraintApplicationError));

    let ev_err0 = eval_err0.to_reasoning_evidence().expect("Must generate reasoning evidence");
    assert_eq!(ev_err0.constraint_extraction, Some(false));

    // 2. Syllogism Representation & Deduction Validation
    let syllogism_inst = SyllogismGenerator::generate_problem(8888, 1, None);
    let syllogism_graph = syllogism_inst.solution_graph().expect("Syllogism instance must have solution graph");
    assert!(syllogism_graph.validate_topology());
    assert_eq!(syllogism_graph.steps[0].step_type, StepType::BuildRepresentation);
    assert_eq!(syllogism_graph.steps[1].step_type, StepType::FinalAnswer);

    // Flawed representation at step 0
    let (err_type, conf, feedback) = MathSemanticComparator::diagnose_step_error("Flawed Model", &syllogism_graph.steps[0], None);
    assert_eq!(err_type, StepErrorType::RepresentationError);
    assert_eq!(conf, DiagnosticConfidence::StronglyInferred);
    assert!(feedback.contains("Representation error"));

    // 3. Domain Evidence Wrapper Integration
    let versioned_ev = eval_valid.to_domain_evidence("family.reasoning.seating").expect("Versioned domain evidence must exist");
    assert_eq!(versioned_ev.version, 1);
    assert!(matches!(versioned_ev.payload, DomainEvidencePayload::Reasoning(_)));
}

#[test]
fn test_multi_domain_evidence_generation() {
    use procedural::skills::domain_evidence::DomainEvidencePayload;

    // 1. Math Domain Evidence
    let math_graph = SolutionGraph::new(
        vec![
            StepNode::new("f1", StepType::FormulaSelection, "Formula", "Select formula", "a^2 + b^2 = c^2"),
            StepNode::new("s1", StepType::Arithmetic, "Calc", "Calculate c", "c = 5").as_final(),
        ],
        "s1",
    );
    let sub_math_err = StepwiseSubmission::stepwise(
        vec![
            SubmittedStep::new(0, "wrong formula", 2000),
            SubmittedStep::new(1, "c = 5", 2000),
        ],
        Some("c = 5".to_string()),
        4000,
    );
    let eval_math = StepValidator::evaluate_submission(&math_graph, &sub_math_err, 20000);
    assert_eq!(eval_math.first_error_type, Some(StepErrorType::FormulaSelectionError));
    let dev_math = eval_math.to_domain_evidence("family.math.geometry").unwrap();
    if let DomainEvidencePayload::Math(m) = dev_math.payload {
        assert_eq!(m.pattern_recognition, Some(false));
        assert_eq!(m.method_selection, Some(false));
    } else {
        panic!("Expected Math domain evidence");
    }

    // 2. Physics Domain Evidence
    let phys_graph = SolutionGraph::new(
        vec![
            StepNode::new("m1", StepType::SelectModel, "Model", "Select model", "F = ma"),
            StepNode::new("u1", StepType::UnitConversion, "Units", "Convert units", "5 m/s"),
            StepNode::new("c1", StepType::PhysicalSanityCheck, "Check", "Sanity check", "a > 0").as_final(),
        ],
        "c1",
    );
    let sub_phys_err = StepwiseSubmission::stepwise(
        vec![
            SubmittedStep::new(0, "F = ma", 2000),
            SubmittedStep::new(1, "5000 km/s", 2000),
            SubmittedStep::new(2, "a > 0", 2000),
        ],
        Some("a > 0".to_string()),
        6000,
    );
    let eval_phys = StepValidator::evaluate_submission(&phys_graph, &sub_phys_err, 30000);
    assert_eq!(eval_phys.first_error_step, Some(1));
    assert_eq!(eval_phys.first_error_type, Some(StepErrorType::UnitError));
    let dev_phys = eval_phys.to_domain_evidence("family.physics.mechanics").unwrap();
    if let DomainEvidencePayload::Physics(p) = dev_phys.payload {
        assert_eq!(p.unit_validity, Some(false));
        assert_eq!(p.physical_model_selection, Some(true));
    } else {
        panic!("Expected Physics domain evidence");
    }
}