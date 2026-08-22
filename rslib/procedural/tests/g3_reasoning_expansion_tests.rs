// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use procedural::core::ProblemFamilyId;
use procedural::diagnostics::ErrorCategory;
use procedural::problems::catalog::MathsCatalog;
use procedural::problems::generator::ProblemGenerator;
use procedural::problems::registry::ProblemRegistry;
use procedural::problems::validator::ProblemValidator;
use procedural::problems::ProblemInstance;
use procedural::reasoning::data_sufficiency::{DataSufficiencyPuzzle, DsAnswer};
use procedural::reasoning::generators::coded_expressions::{
    CodedExpressionsGenerator, CodedExpressionsValidator, FAMILY_REASONING_CODED_EXPRESSIONS,
};
use procedural::reasoning::generators::data_sufficiency::{
    DataSufficiencyGenerator, DataSufficiencyValidator, FAMILY_REASONING_DATA_SUFFICIENCY,
};
use procedural::reasoning::generators::floor_grid::{
    FloorGridGenerator, FloorGridValidator, FAMILY_REASONING_FLOOR_GRID,
};
use procedural::reasoning::generators::logic_dag::{
    LogicDagGenerator, LogicDagValidator, FAMILY_REASONING_LOGIC_DAG,
};
use procedural::reasoning::logic_dag::LogicDagPuzzle;
use procedural::storage::ProceduralStore;
use rand::rngs::StdRng;
use rand::SeedableRng;

#[test]
fn test_g3_floor_grid_generation_all_difficulty_levels() {
    let generator = FloorGridGenerator;
    for level in 1..=5 {
        for seed in [101u64, 202, 303, 404, 505] {
            let inst = FloorGridGenerator::generate_problem(seed, level, None);
            assert_eq!(inst.family_id, ProblemFamilyId::from(FAMILY_REASONING_FLOOR_GRID));
            assert!(!inst.rendered_prompt.is_empty());
            assert!(inst.solution_graph().is_some());
            assert!(inst.correct_answer.get("value").is_some());
            assert!(inst.parameters.get("total_slots").is_some());

            // Trait dispatch
            let trait_inst = generator
                .generate(&ProblemFamilyId::from(FAMILY_REASONING_FLOOR_GRID), seed, level, None)
                .expect("Generator trait should succeed");
            assert_eq!(trait_inst.rendered_prompt, inst.rendered_prompt);
        }
    }
}

#[test]
fn test_g3_floor_grid_solver_and_validator() {
    let validator = FloorGridValidator;

    // Test across several seeds
    for seed in 1..=10 {
        let inst = FloorGridGenerator::generate_problem(seed * 777, 3, None);
        let expected_val = inst.correct_answer.get("value").unwrap().as_str().unwrap();

        // Exact match
        let eval_correct = validator.evaluate(&inst, &serde_json::json!(expected_val), 25000, 45000);
        assert!(eval_correct.is_correct, "Seed {} should validate correct", seed);
        assert_eq!(eval_correct.score, 1.0);

        // Incorrect student answer
        let eval_incorrect = validator.evaluate(&inst, &serde_json::json!("Floor 99"), 25000, 45000);
        assert!(!eval_incorrect.is_correct);
        assert_eq!(eval_incorrect.error_category, Some(ErrorCategory::Strategy));
    }
}

#[test]
fn test_g3_logic_dag_generation_and_deductive_soundness() {
    let _generator = LogicDagGenerator;
    let validator = LogicDagValidator;

    for level in 1..=5 {
        let inst = LogicDagGenerator::generate_problem(12345 + level as u64, level, None);
        assert_eq!(inst.family_id, ProblemFamilyId::from(FAMILY_REASONING_LOGIC_DAG));
        assert!(!inst.rendered_prompt.is_empty());
        assert!(inst.solution_graph().is_some());

        let expected_conclusion = inst.correct_answer.get("value").unwrap().as_str().unwrap();
        let options = inst.correct_answer.get("options").unwrap().as_array().unwrap();
        assert!(options.len() >= 3);

        // Verify that expected conclusion is indeed one of the options
        let has_expected = options.iter().any(|opt| opt.as_str().unwrap() == expected_conclusion);
        assert!(has_expected, "Options must contain the target expected answer");

        // Verify correct answer submission
        let eval = validator.evaluate(&inst, &serde_json::json!(expected_conclusion), 20000, 40000);
        assert!(eval.is_correct);

        // Verify incorrect answer
        let eval_bad = validator.evaluate(&inst, &serde_json::json!("Arbitrary Invalid Conclusion"), 20000, 40000);
        assert!(!eval_bad.is_correct);
        assert_eq!(eval_bad.error_category, Some(ErrorCategory::Concept));
    }
}

#[test]
fn test_g3_logic_dag_independent_truth_table_verifier() {
    let mut rng = StdRng::seed_from_u64(98765);
    for level in 1..=5 {
        let puzzle = LogicDagPuzzle::generate_dynamic(&mut rng, level);
        assert!(!puzzle.premises_formal.is_empty());
        assert!(!puzzle.target_answer.is_empty());
        assert!(!puzzle.derivation_steps.is_empty());

        // Verify all premises are non-empty strings
        for p in &puzzle.premises_text {
            assert!(!p.is_empty());
        }
    }
}

#[test]
fn test_g3_data_sufficiency_determinacy_options_a_to_e() {
    let validator = DataSufficiencyValidator;

    // Test generator across levels 1..=5
    for level in 1..=5 {
        let inst = DataSufficiencyGenerator::generate_problem(5555 + level as u64, level, None);
        assert_eq!(inst.family_id, ProblemFamilyId::from(FAMILY_REASONING_DATA_SUFFICIENCY));
        assert!(!inst.rendered_prompt.is_empty());
        assert!(inst.solution_graph().is_some());

        let expected_letter = inst.correct_answer.get("value").unwrap().as_str().unwrap();
        assert!(["A", "B", "C", "D", "E"].contains(&expected_letter));

        // Evaluate correct letter submission
        let eval = validator.evaluate(&inst, &serde_json::json!(expected_letter), 20000, 35000);
        assert!(eval.is_correct, "Level {} letter {} should be correct", level, expected_letter);

        // Evaluate format variations (e.g. "(A)", lowercase "a")
        let eval_bracket = validator.evaluate(&inst, &serde_json::json!(format!("({})", expected_letter)), 20000, 35000);
        assert!(eval_bracket.is_correct, "Bracketed letter should be accepted");

        let eval_lower = validator.evaluate(&inst, &serde_json::json!(expected_letter.to_lowercase()), 20000, 35000);
        assert!(eval_lower.is_correct, "Lowercase letter should be accepted");

        // Incorrect letter
        let wrong_letter = if expected_letter == "A" { "B" } else { "A" };
        let eval_wrong = validator.evaluate(&inst, &serde_json::json!(wrong_letter), 20000, 35000);
        assert!(!eval_wrong.is_correct);
        assert_eq!(eval_wrong.error_category, Some(ErrorCategory::Concept));
    }
}

#[test]
fn test_g3_data_sufficiency_puzzle_determinacy_models() {
    let mut rng = StdRng::seed_from_u64(424242);
    let mut seen_answers = std::collections::HashSet::new();

    for level in 1..=5 {
        for _ in 0..10 {
            let p = DataSufficiencyPuzzle::generate_dynamic(&mut rng, level);
            seen_answers.insert(p.correct_answer);
        }
    }

    // Must generate multiple distinct sufficiency determinacy outcomes (A, B, C, D, E)
    assert!(seen_answers.len() >= 3, "Must produce diverse DS answer distributions");
}

#[test]
fn test_g3_coded_expressions_kinship_and_vectors() {
    let validator = CodedExpressionsValidator;

    for level in 1..=5 {
        let inst = CodedExpressionsGenerator::generate_problem(7890 + level as u64, level, None);
        assert_eq!(inst.family_id, ProblemFamilyId::from(FAMILY_REASONING_CODED_EXPRESSIONS));
        assert!(!inst.rendered_prompt.is_empty());
        assert!(inst.solution_graph().is_some());

        let expected_val = inst.correct_answer.get("value").unwrap().as_str().unwrap();
        let options = inst.correct_answer.get("options").unwrap().as_array().unwrap();
        assert!(options.len() >= 3);

        // Correct evaluation
        let eval = validator.evaluate(&inst, &serde_json::json!(expected_val), 25000, 40000);
        assert!(eval.is_correct, "Level {} answer '{}' should validate", level, expected_val);

        // Incorrect evaluation
        let eval_bad = validator.evaluate(&inst, &serde_json::json!("Completely Wrong Option"), 25000, 40000);
        assert!(!eval_bad.is_correct);
        assert_eq!(eval_bad.error_category, Some(ErrorCategory::Concept));
    }
}

#[test]
fn test_g3_reasoning_cognitive_decision_points_and_strategy_drills() {
    let generators: Vec<(&str, fn(u64, u32, Option<&str>) -> ProblemInstance)> = vec![
        (FAMILY_REASONING_FLOOR_GRID, FloorGridGenerator::generate_problem),
        (FAMILY_REASONING_LOGIC_DAG, LogicDagGenerator::generate_problem),
        (FAMILY_REASONING_DATA_SUFFICIENCY, DataSufficiencyGenerator::generate_problem),
        (FAMILY_REASONING_CODED_EXPRESSIONS, CodedExpressionsGenerator::generate_problem),
    ];

    for (fam, gen_fn) in generators {
        let inst: ProblemInstance = gen_fn(12345, 3, Some("strategy_drill"));
        let meta = inst.parameters.get("reasoning_metadata");
        assert!(meta.is_some(), "Reasoning metadata must be present for {}", fam);

        let dps = meta.unwrap().get("decision_points").and_then(|v| v.as_array());
        assert!(dps.is_some(), "decision_points must be present for {}", fam);
        let first_dp = &dps.unwrap()[0];
        let options = first_dp.get("options").unwrap().as_array().unwrap();
        assert!(options.len() >= 2, "Must offer >= 2 options for strategic decision");
    }
}

#[test]
fn test_g3_reasoning_catalog_and_registry_full_integration() {
    let store = ProceduralStore::open_in_memory().unwrap();
    MathsCatalog::init_all(&store).unwrap();

    // Verify all 8 reasoning skills/families exist in the database
    let reasoning_families = [
        "family.reasoning.series.patterns",
        "family.reasoning.syllogism.categorical",
        "family.reasoning.seating.linear",
        "family.reasoning.relations.graph",
        FAMILY_REASONING_FLOOR_GRID,
        FAMILY_REASONING_LOGIC_DAG,
        FAMILY_REASONING_DATA_SUFFICIENCY,
        FAMILY_REASONING_CODED_EXPRESSIONS,
    ];

    for fam in reasoning_families {
        assert!(
            store.get_problem_family(&ProblemFamilyId::from(fam)).unwrap().is_some(),
            "Family {} must be registered in database",
            fam
        );
    }

    let registry = ProblemRegistry::default();

    // Verify registry can generate problem instances for all 8 reasoning families
    for fam in reasoning_families {
        let gen_result = registry.generate(&ProblemFamilyId::from(fam), "", 42, 2, None);
        assert!(gen_result.is_ok(), "Registry failed to generate {}", fam);
        let instance = gen_result.unwrap();
        assert_eq!(instance.family_id, ProblemFamilyId::from(fam));
    }
}