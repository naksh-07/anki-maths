// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use procedural::chemistry::generators::buffers_titration::{
    BuffersTitrationGenerator, BuffersTitrationValidator, FAMILY_CHEMISTRY_BUFFERS_TITRATION,
};
use procedural::chemistry::generators::electrochemistry::{
    ElectrochemistryGenerator, ElectrochemistryValidator, FAMILY_CHEMISTRY_ELECTROCHEMISTRY,
};
use procedural::chemistry::generators::kinetics::{
    ChemicalKineticsGenerator, ChemicalKineticsValidator, FAMILY_CHEMISTRY_KINETICS,
};
use procedural::chemistry::generators::reaction_networks::{
    ReactionNetworksGenerator, ReactionNetworksValidator, FAMILY_CHEMISTRY_REACTION_NETWORKS,
};
use procedural::core::ProblemFamilyId;
use procedural::diagnostics::ErrorCategory;
use procedural::problems::catalog::MathsCatalog;
use procedural::problems::generator::ProblemGenerator;
use procedural::problems::registry::ProblemRegistry;
use procedural::problems::validator::ProblemValidator;
use procedural::problems::ProblemInstance;
use procedural::storage::ProceduralStore;

#[test]
fn test_g3_buffers_titration_generation_all_levels() {
    let generator = BuffersTitrationGenerator;
    for level in 1..=5 {
        for seed in [111u64, 222, 333, 444, 555] {
            let inst = BuffersTitrationGenerator::generate_problem(seed, level, None);
            assert_eq!(inst.family_id, ProblemFamilyId::from(FAMILY_CHEMISTRY_BUFFERS_TITRATION));
            assert!(!inst.rendered_prompt.is_empty());
            assert!(inst.solution_graph().is_some());
            let ph = inst.correct_answer.get("value").unwrap().as_f64().unwrap();

            // pH bounds check: 0.0 <= pH <= 14.0
            assert!(
                ph >= 0.0 && ph <= 14.0,
                "Calculated pH {} must fall in physical range [0, 14]",
                ph
            );

            // Trait dispatch
            let trait_inst = generator
                .generate(&ProblemFamilyId::from(FAMILY_CHEMISTRY_BUFFERS_TITRATION), seed, level, None)
                .expect("Generator trait should succeed");
            assert_eq!(trait_inst.rendered_prompt, inst.rendered_prompt);
        }
    }
}

#[test]
fn test_g3_buffers_titration_validator_and_tolerances() {
    let validator = BuffersTitrationValidator;

    for level in 1..=5 {
        let inst = BuffersTitrationGenerator::generate_problem(4321 + level as u64, level, None);
        let correct_ph = inst.correct_answer.get("value").unwrap().as_f64().unwrap();

        // Exact match
        let eval = validator.evaluate(&inst, &serde_json::json!(correct_ph), 25000, 40000);
        assert!(eval.is_correct, "Exact pH should validate: {}", correct_ph);
        assert_eq!(eval.score, 1.0);

        // Within tolerance (±0.03)
        let eval_tol = validator.evaluate(&inst, &serde_json::json!(correct_ph + 0.03), 25000, 40000);
        assert!(eval_tol.is_correct, "pH within tolerance should validate");

        // Out of tolerance (±0.50)
        let eval_bad = validator.evaluate(&inst, &serde_json::json!(correct_ph + 0.50), 25000, 40000);
        assert!(!eval_bad.is_correct);
        assert_eq!(eval_bad.error_category, Some(ErrorCategory::Calculation));
    }
}

#[test]
fn test_g3_electrochemistry_nernst_and_faraday_generation() {
    let generator = ElectrochemistryGenerator;
    for level in 1..=5 {
        for seed in [1001u64, 2002, 3003, 4004, 5005] {
            let inst = ElectrochemistryGenerator::generate_problem(seed, level, None);
            assert_eq!(inst.family_id, ProblemFamilyId::from(FAMILY_CHEMISTRY_ELECTROCHEMISTRY));
            assert!(!inst.rendered_prompt.is_empty());
            assert!(inst.solution_graph().is_some());
            let val = inst.correct_answer.get("value").unwrap().as_f64().unwrap();

            // Non-negative physical quantities (or voltage reasonable bounds)
            if level <= 3 {
                // Voltage in volts (-1.0 to 6.5 V for galvanic couples up to Li-F2)
                assert!(val >= -1.0 && val <= 6.5, "Cell potential must be physical: {}", val);
            } else {
                // Mass or time (> 0.0)
                assert!(val > 0.0, "Electrolytic mass/time must be strictly positive: {}", val);
            }

            let trait_inst = generator
                .generate(&ProblemFamilyId::from(FAMILY_CHEMISTRY_ELECTROCHEMISTRY), seed, level, None)
                .expect("Generator trait should succeed");
            assert_eq!(trait_inst.rendered_prompt, inst.rendered_prompt);
        }
    }
}

#[test]
fn test_g3_electrochemistry_validator_and_tolerances() {
    let validator = ElectrochemistryValidator;

    for level in 1..=5 {
        let inst = ElectrochemistryGenerator::generate_problem(8765 + level as u64, level, None);
        let correct_val = inst.correct_answer.get("value").unwrap().as_f64().unwrap();

        // Exact match
        let eval = validator.evaluate(&inst, &serde_json::json!(correct_val), 20000, 40000);
        assert!(eval.is_correct, "Exact electrochem value should validate: {}", correct_val);

        // Incorrect value
        let eval_bad = validator.evaluate(&inst, &serde_json::json!(correct_val * 2.5 + 5.0), 20000, 40000);
        assert!(!eval_bad.is_correct);
        assert_eq!(eval_bad.error_category, Some(ErrorCategory::Calculation));
    }
}

#[test]
fn test_g3_kinetics_integrated_rate_laws_and_arrhenius() {
    let generator = ChemicalKineticsGenerator;
    let validator = ChemicalKineticsValidator;

    for level in 1..=5 {
        for seed in [1122u64, 3344, 5566] {
            let inst = ChemicalKineticsGenerator::generate_problem(seed, level, None);
            assert_eq!(inst.family_id, ProblemFamilyId::from(FAMILY_CHEMISTRY_KINETICS));
            assert!(!inst.rendered_prompt.is_empty());
            assert!(inst.solution_graph().is_some());

            let correct_val = inst.correct_answer.get("value").unwrap().as_f64().unwrap();
            assert!(correct_val > 0.0, "Kinetics quantity must be positive: {}", correct_val);

            // Verify evaluation
            let eval = validator.evaluate(&inst, &serde_json::json!(correct_val), 20000, 35000);
            assert!(eval.is_correct);

            // Trait dispatch
            let trait_inst = generator
                .generate(&ProblemFamilyId::from(FAMILY_CHEMISTRY_KINETICS), seed, level, None)
                .expect("Generator trait should succeed");
            assert_eq!(trait_inst.rendered_prompt, inst.rendered_prompt);
        }
    }
}

#[test]
fn test_g3_reaction_networks_multistage_stoichiometry_and_flux() {
    let generator = ReactionNetworksGenerator;
    let validator = ReactionNetworksValidator;

    for level in 1..=5 {
        for seed in [9901u64, 9902, 9903] {
            let inst = ReactionNetworksGenerator::generate_problem(seed, level, None);
            assert_eq!(inst.family_id, ProblemFamilyId::from(FAMILY_CHEMISTRY_REACTION_NETWORKS));
            assert!(!inst.rendered_prompt.is_empty());
            assert!(inst.solution_graph().is_some());

            let mass = inst.correct_answer.get("value").unwrap().as_f64().unwrap();
            assert!(mass > 0.0, "Reaction product mass/percentage must be positive: {}", mass);

            // Verify evaluation
            let eval = validator.evaluate(&inst, &serde_json::json!(mass), 25000, 45000);
            assert!(eval.is_correct);

            // Trait dispatch
            let trait_inst = generator
                .generate(&ProblemFamilyId::from(FAMILY_CHEMISTRY_REACTION_NETWORKS), seed, level, None)
                .expect("Generator trait should succeed");
            assert_eq!(trait_inst.rendered_prompt, inst.rendered_prompt);
        }
    }
}

#[test]
fn test_g3_chemistry_cognitive_decision_points_and_regimes() {
    let generators: Vec<(&str, fn(u64, u32, Option<&str>) -> ProblemInstance)> = vec![
        (FAMILY_CHEMISTRY_BUFFERS_TITRATION, BuffersTitrationGenerator::generate_problem),
        (FAMILY_CHEMISTRY_ELECTROCHEMISTRY, ElectrochemistryGenerator::generate_problem),
        (FAMILY_CHEMISTRY_KINETICS, ChemicalKineticsGenerator::generate_problem),
        (FAMILY_CHEMISTRY_REACTION_NETWORKS, ReactionNetworksGenerator::generate_problem),
    ];

    for (fam, gen_fn) in generators {
        let inst: ProblemInstance = gen_fn(54321, 3, None);
        let dp = inst.parameters.get("decision_point");
        assert!(dp.is_some(), "CognitiveDecisionPoint must be present for {}", fam);
        let options = dp.unwrap().get("options").unwrap().as_array().unwrap();
        assert!(options.len() >= 2, "Must offer >= 2 options for strategic decision");
    }
}

#[test]
fn test_g3_chemistry_catalog_and_registry_full_integration() {
    let store = ProceduralStore::open_in_memory().unwrap();
    MathsCatalog::init_all(&store).unwrap();

    // Verify all 6 chemistry skills/families exist in the database
    let chem_families = [
        "family.chemistry.stoichiometry.moles",
        "family.chemistry.equilibrium.concentration",
        FAMILY_CHEMISTRY_BUFFERS_TITRATION,
        FAMILY_CHEMISTRY_ELECTROCHEMISTRY,
        FAMILY_CHEMISTRY_KINETICS,
        FAMILY_CHEMISTRY_REACTION_NETWORKS,
    ];

    for fam in chem_families {
        assert!(
            store.get_problem_family(&ProblemFamilyId::from(fam)).unwrap().is_some(),
            "Chemistry family {} must be registered in database",
            fam
        );
    }

    let registry = ProblemRegistry::new();

    // Verify registry can generate problem instances for all 6 chemistry families
    for fam in chem_families {
        let gen_result = registry.generate(&ProblemFamilyId::from(fam), "", 42, 2, None);
        assert!(gen_result.is_ok(), "Registry failed to generate {}", fam);
        let instance = gen_result.unwrap();
        assert_eq!(instance.family_id, ProblemFamilyId::from(fam));
    }
}
