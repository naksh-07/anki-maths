// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use procedural::{
    BloodRelationPuzzle, CognitiveDecisionPoint, CspConstraint, CspProblem, CspSolver,
    DecisionOption, DirectionPuzzle, Heading, KinshipRelation, PracticeMode, ProceduralCardAnchor,
    ProceduralService, Rating, SeatingPuzzle, SeriesProblem, SeriesRule, SolutionGraph, StepNode,
    StepType, StepValidationStatus, StepValidator, StepwiseSubmission, StrategyKind,
    SubmittedStep, SyllogismProblem, TransferEligibilityEngine,
};
use procedural::core::SchemaId;
use procedural::problems::catalog::{
    SCHEMA_REASONING_RELATIONS, SCHEMA_REASONING_SEATING, SCHEMA_REASONING_SERIES,
    SCHEMA_REASONING_SYLLOGISM,
};
use procedural::skills::signals::{IndependenceLevel, MasteryEvidence};

#[test]
fn test_reasoning_models_and_decision_points() {
    let opt1 = DecisionOption::new(
        "opt_anchor",
        "Place fixed person at slot 1 first",
        StrategyKind::AnchorFixed.as_str(),
        true,
        "Anchor positions bound variable domains immediately.",
    );
    let opt2 = DecisionOption::new(
        "opt_random",
        "Place flexible person at slot 3",
        StrategyKind::BranchCases.as_str(),
        false,
        "Sub-optimal: Branching before anchoring fixed elements increases search space.",
    );

    let dp = CognitiveDecisionPoint::new(
        "dp_1",
        "Which constraint should be applied first?",
        vec![opt1, opt2],
        "opt_anchor",
        StrategyKind::AnchorFixed.as_str(),
        "Always anchor invariant elements first.",
    );

    let (is_valid, strategy, feedback) = dp.evaluate_choice("opt_anchor");
    assert!(is_valid);
    assert_eq!(strategy, Some(StrategyKind::AnchorFixed.as_str().to_string()));
    assert!(feedback.contains("Anchor positions"));

    let (is_valid2, strategy2, _) = dp.evaluate_choice("opt_random");
    assert!(!is_valid2);
    assert_eq!(strategy2, Some(StrategyKind::BranchCases.as_str().to_string()));
}

#[test]
fn test_csp_solver_arc_consistency_and_uniqueness() {
    // Linear seating CSP: 4 people (A, B, C, D) in slots 1..4
    let mut problem = CspProblem::new(
        vec!["A".to_string(), "B".to_string(), "C".to_string(), "D".to_string()],
        4,
    );
    problem.add_constraint(CspConstraint::Fixed {
        var: "A".to_string(),
        slot: 1,
    });
    problem.add_constraint(CspConstraint::ImmediateLeft {
        v1: "B".to_string(),
        v2: "C".to_string(),
    });
    problem.add_constraint(CspConstraint::Fixed {
        var: "D".to_string(),
        slot: 4,
    });

    let solver = CspSolver;
    assert!(solver.is_unambiguous(&problem));

    let sol = solver.solve_unique(&problem).unwrap();
    assert_eq!(sol.get("A"), Some(&1));
    assert_eq!(sol.get("B"), Some(&2));
    assert_eq!(sol.get("C"), Some(&3));
    assert_eq!(sol.get("D"), Some(&4));

    // Contradictory problem: A is at slot 1, B is at slot 1
    let mut bad_problem = CspProblem::new(vec!["A".to_string(), "B".to_string()], 2);
    bad_problem.add_constraint(CspConstraint::Fixed {
        var: "A".to_string(),
        slot: 1,
    });
    bad_problem.add_constraint(CspConstraint::Fixed {
        var: "B".to_string(),
        slot: 1,
    });

    assert!(!solver.is_unambiguous(&bad_problem));
    assert!(solver.solve_all(&bad_problem).is_empty());
}

#[test]
fn test_syllogism_formal_inference_models() {
    // Barbara: All cats are mammals, All mammals are animals -> Both follow
    let barbara = SyllogismProblem::create_barbara("cats", "mammals", "animals");
    assert!(barbara.is_correct("Both I and II follow"));
    assert!(barbara.is_correct("both"));
    assert!(barbara.is_correct("option c"));

    // Celarent: All roses are flowers, No flowers are rocks -> Only I follows
    let celarent = SyllogismProblem::create_celarent("roses", "flowers", "rocks");
    assert!(celarent.is_correct("Only I follows"));
    assert!(celarent.is_correct("1"));

    // Two Particulars: Some apples are fruits, Some fruits are red -> Neither follows
    let disjoint = SyllogismProblem::create_disjoint_some("apples", "fruits", "red objects");
    assert!(disjoint.is_correct("Neither follows"));
    assert!(disjoint.is_correct("none"));
}

#[test]
fn test_series_pattern_detection_and_unambiguous_answer() {
    // Constant difference
    let p1 = SeriesProblem::generate_numeric(SeriesRule::ConstantDifference { diff: 4 }, 2, 4);
    assert_eq!(p1.terms_numeric, vec![2, 6, 10, 14]);
    assert_eq!(p1.expected_next_numeric, Some(18));
    assert!(p1.is_correct("18"));
    assert!(!p1.is_correct("19"));

    // Geometric sequence
    let p2 = SeriesProblem::generate_numeric(SeriesRule::Geometric { ratio: 3 }, 2, 4);
    assert_eq!(p2.terms_numeric, vec![2, 6, 18, 54]);
    assert_eq!(p2.expected_next_numeric, Some(162));
    assert!(p2.is_correct("162"));

    // Alphabet progression
    let p3 = SeriesProblem::generate_alphabet('C', 2, 4);
    assert_eq!(p3.terms_string, vec!["C", "E", "G", "I"]);
    assert_eq!(p3.expected_next_string, "K");
    assert!(p3.is_correct("K"));
    assert!(p3.is_correct("k"));
}

#[test]
fn test_seating_csp_formulation_and_generator() {
    let puzzle = SeatingPuzzle::build_5person_anchor_puzzle("Alice", 1, "Bob", "Charlie", &["David", "Emma"], 3);
    assert!(puzzle.is_some());
    let p = puzzle.unwrap();
    assert_eq!(p.target_answer, "Charlie");
    assert!(p.is_correct("Charlie"));
    assert!(p.is_correct("charlie"));
}

#[test]
fn test_relations_kinship_and_direction_vectors() {
    // Maternal uncle chain
    let blood = BloodRelationPuzzle::create_uncle_chain("Rohan", "Priya", "Amit");
    assert_eq!(blood.target_relation, KinshipRelation::MaternalUncle);
    assert!(blood.is_correct("Maternal Uncle"));
    assert!(blood.is_correct("Uncle"));

    // 2D Direction displacement
    let dir = DirectionPuzzle::create_path(12, 5, 0); // 12m North, 5m East -> North-East, 13m
    assert_eq!(dir.shortest_distance_meters, 13);
    assert_eq!(dir.final_direction_from_start, Heading::NorthEast);
    assert!(dir.is_correct("North-East"));
    assert!(dir.is_correct("13"));
}

#[test]
fn test_reasoning_catalog_and_schema_resolution() {
    let service = ProceduralService::open_in_memory().unwrap();

    // 1. Series schema resolution
    let schema_ser = service
        .resolve_schema(&SchemaId::from("reasoning.series"))
        .unwrap();
    assert!(schema_ser.is_some());
    assert_eq!(schema_ser.unwrap().id.as_str(), SCHEMA_REASONING_SERIES);

    // 2. Syllogism schema resolution
    let schema_syl = service
        .resolve_schema(&SchemaId::from("reasoning.syllogism"))
        .unwrap();
    assert!(schema_syl.is_some());
    assert_eq!(schema_syl.unwrap().id.as_str(), SCHEMA_REASONING_SYLLOGISM);

    // 3. Seating schema resolution
    let schema_seat = service
        .resolve_schema(&SchemaId::from("reasoning.seating"))
        .unwrap();
    assert!(schema_seat.is_some());
    assert_eq!(schema_seat.unwrap().id.as_str(), SCHEMA_REASONING_SEATING);

    // 4. Relations schema resolution
    let schema_rel = service
        .resolve_schema(&SchemaId::from("reasoning.relations"))
        .unwrap();
    assert!(schema_rel.is_some());
    assert_eq!(schema_rel.unwrap().id.as_str(), SCHEMA_REASONING_RELATIONS);
}

#[test]
fn test_reasoning_multi_schema_selection_and_interleaving() {
    let service = ProceduralService::open_in_memory().unwrap();

    // MixedReasoning mode session
    let session = service
        .prepare_multi_schema_session(&PracticeMode::MixedReasoning, None, None, Some(42))
        .unwrap();

    assert!(!session.instance.rendered_prompt.is_empty());
    assert!(session.difficulty_level.is_some());
    assert!(session.target_latency_ms.is_some());

    // Anti-priming interleaving
    let chosen_id = session.schema.id.clone();
    let next_session = service
        .prepare_multi_schema_session(
            &PracticeMode::MixedReasoning,
            None,
            Some(&chosen_id),
            Some(43),
        )
        .unwrap();

    assert_ne!(next_session.schema.id, chosen_id);
}

#[test]
fn test_reasoning_stepwise_solution_graph_validation_and_hints() {
    // Construct a reasoning seating solution graph
    let step1 = StepNode::new(
        "anchor_step",
        StepType::ApplyConstraint,
        "Anchor Fixed Position",
        "Place Alice at slot 1",
        "Slot 1 = Alice",
    )
    .with_hints(vec![
        procedural::problems::steps::StepHint::new(
            procedural::problems::steps::HintLevel::Principle,
            "Anchor Principle",
            "Place the fixed person first.",
        ),
        procedural::problems::steps::StepHint::new(
            procedural::problems::steps::HintLevel::Operation,
            "Operation",
            "Set Slot 1 to Alice.",
        ),
        procedural::problems::steps::StepHint::new(
            procedural::problems::steps::HintLevel::IntermediateRelation,
            "Intermediate",
            "Slot 1 = Alice",
        ),
    ]);

    let step2 = StepNode::new(
        "final_step",
        StepType::FinalAnswer,
        "Target Person",
        "Identify person at slot 3",
        "Charlie",
    )
    .with_dependencies(vec!["anchor_step".to_string()])
    .as_final();

    let graph = SolutionGraph::new(vec![step1, step2], "final_step");
    assert!(graph.validate_topology());
    assert_eq!(graph.hints_for_step(0).len(), 3);

    // Correct stepwise submission
    let steps = vec![
        SubmittedStep::new(0, "Slot 1 = Alice", 3000),
        SubmittedStep::new(1, "Charlie", 2000),
    ];
    let submission = StepwiseSubmission::stepwise(steps, Some("Charlie".to_string()), 5000);
    let eval = StepValidator::evaluate_submission(&graph, &submission, 30000);
    assert!(eval.is_correct);
    assert_eq!(eval.score, 1.0);
    assert_eq!(eval.first_error_step, None);
    assert_eq!(eval.step_evaluations[0].status, StepValidationStatus::Valid);
    assert_eq!(eval.step_evaluations[1].status, StepValidationStatus::Valid);
}

#[test]
fn test_end_to_end_reasoning_seating_quick_solve_and_fsrs_rating() {
    let service = ProceduralService::open_in_memory().unwrap();

    // 1. Anchor card resolution
    let anchor = ProceduralCardAnchor::new("reasoning.seating");
    let session = service.prepare_practice_session(&anchor, Some(99001)).unwrap();
    assert_eq!(session.schema.id.as_str(), SCHEMA_REASONING_SEATING);

    // 2. Extract correct answer
    let target_answer = session
        .instance
        .correct_answer
        .get("formatted")
        .unwrap()
        .as_str()
        .unwrap();

    // 3. Evaluate and record attempt
    let outcome = service
        .evaluate_and_record_attempt(
            &session.instance.id,
            session.card_id,
            serde_json::json!(target_answer),
            20_000,
            0,
            1,
        )
        .unwrap();

    assert!(outcome.is_correct);
    assert_eq!(outcome.score, 1.0);
    assert_eq!(outcome.error_category, None);

    // 4. Derive FSRS rating
    let rating = service.derive_fsrs_rating(&outcome).unwrap();
    assert!(matches!(rating, Rating::Good | Rating::Easy));

    // 5. Verify skill state update
    let state = service.load_skill_state(&outcome.skill_id).unwrap().unwrap();
    assert_eq!(state.total_attempts, 1);
    assert_eq!(state.successful_attempts, 1);
    assert_eq!(state.consecutive_successes, 1);
}

#[test]
fn test_end_to_end_reasoning_series_strategy_drill_and_fsrs_rating() {
    let service = ProceduralService::open_in_memory().unwrap();

    // 1. Prepare session in StrategyDrill mode
    let session = service
        .prepare_multi_schema_session(&PracticeMode::StrategyDrill, None, None, Some(777))
        .unwrap();

    assert!(!session.instance.rendered_prompt.is_empty());
    assert_eq!(session.target_latency_ms, Some(15_000)); // Strategy drill 15s target

    let expected_answer = session
        .instance
        .correct_answer
        .get("formatted")
        .unwrap()
        .as_str()
        .unwrap();

    // 2. Evaluate correct response
    let outcome = service
        .evaluate_and_record_attempt(
            &session.instance.id,
            None,
            serde_json::json!(expected_answer),
            8_000,
            0,
            1,
        )
        .unwrap();

    assert!(outcome.is_correct);
    assert_eq!(outcome.score, 1.0);

    // 3. Derive FSRS rating
    let rating = service.derive_fsrs_rating(&outcome).unwrap();
    assert!(matches!(rating, Rating::Good | Rating::Easy));
}

#[test]
fn test_reasoning_transfer_eligibility_gating() {
    let mut state = procedural::SkillState::new(procedural::SkillId::from("reasoning.seating.constraint_satisfaction"));
    
    // Cold start is not eligible
    let elig = TransferEligibilityEngine::evaluate_eligibility(Some(&state));
    assert!(!elig.is_eligible);

    // Add 2 consecutive successful attempts
    let ev1 = MasteryEvidence {
        final_correctness: true,
        latency_evidence: 20_000,
        variant_exposure: Some("linear_4person".to_string()),
        independence: IndependenceLevel::Independent,
        ..Default::default()
    };
    state.record_attempt_outcome(&ev1, 1.0, 35_000, 1000);
    let ev2 = MasteryEvidence {
        final_correctness: true,
        latency_evidence: 22_000,
        variant_exposure: Some("linear_5person".to_string()),
        independence: IndependenceLevel::Independent,
        ..Default::default()
    };
    state.record_attempt_outcome(&ev2, 1.0, 35_000, 2000);

    let elig_after = TransferEligibilityEngine::evaluate_eligibility(Some(&state));
    assert!(elig_after.is_eligible);
}
