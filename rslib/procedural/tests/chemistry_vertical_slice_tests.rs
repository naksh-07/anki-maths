// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::collections::HashMap;

use procedural::anchor::{ProceduralCardAnchor, SeedMode};
use procedural::chemistry::{
    ChemicalDimensionalValidator, ChemicalInvariantValidator, ChemistryUnit, ReactionTemplates,
    SpeciesCatalog, StateOfMatter,
};
use procedural::core::SchemaId;
use procedural::diagnostics::ErrorCategory;
use procedural::problems::catalog::{
    SCHEMA_CHEMISTRY_EQUILIBRIUM, SCHEMA_CHEMISTRY_STOICHIOMETRY,
    SKILL_CHEMISTRY_EQUILIBRIUM, SKILL_CHEMISTRY_STOICHIOMETRY,
};
use procedural::problems::steps::{
    HintLevel, StepType, StepValidator, StepwiseSubmission, SubmittedStep,
};
use procedural::scheduling::{
    derive_fsrs_rating, MultiSchemaSelector, PracticeMode, Rating,
};
use procedural::service::ProceduralService;

#[test]
fn test_chemistry_units_and_dimensional_conversions() {
    // 1. Mass conversions
    let kg = ChemicalDimensionalValidator::convert(450.0, ChemistryUnit::Gram, ChemistryUnit::Kilogram).unwrap();
    assert!((kg - 0.45).abs() < 1e-6);

    let g = ChemicalDimensionalValidator::convert(0.125, ChemistryUnit::Kilogram, ChemistryUnit::Gram).unwrap();
    assert!((g - 125.0).abs() < 1e-6);

    // 2. Volume conversions
    let l = ChemicalDimensionalValidator::convert(250.0, ChemistryUnit::Milliliter, ChemistryUnit::Liter).unwrap();
    assert!((l - 0.25).abs() < 1e-6);

    // 3. Amount of substance conversions
    let mmol = ChemicalDimensionalValidator::convert(0.02, ChemistryUnit::Mole, ChemistryUnit::Millimole).unwrap();
    assert!((mmol - 20.0).abs() < 1e-6);

    // 4. Incompatible conversion rejected
    let err = ChemicalDimensionalValidator::convert(10.0, ChemistryUnit::Gram, ChemistryUnit::Molar);
    assert!(err.is_err());
}

#[test]
fn test_chemical_species_and_reaction_balancing() {
    // Species catalog lookups
    let caco3 = SpeciesCatalog::find("CaCO3").expect("CaCO3 should exist");
    assert_eq!(caco3.name, "Calcium Carbonate");
    assert!((caco3.molar_mass - 100.086).abs() < 1e-3);
    assert_eq!(caco3.state, StateOfMatter::Solid);

    let nh3 = SpeciesCatalog::find("NH3").expect("NH3 should exist");
    assert_eq!(nh3.name, "Ammonia");
    assert!((nh3.molar_mass - 17.031).abs() < 1e-3);
    assert_eq!(nh3.state, StateOfMatter::Gas);

    // Reaction stoichiometric ratios
    let haber = ReactionTemplates::haber_bosch();
    assert_eq!(haber.stoichiometric_ratio("N2", "NH3"), Some(2.0));
    assert_eq!(haber.stoichiometric_ratio("H2", "NH3"), Some(2.0 / 3.0));

    let al_ox = ReactionTemplates::aluminum_oxidation(); // 4 Al + 3 O2 -> 2 Al2O3
    assert_eq!(al_ox.stoichiometric_ratio("Al", "Al2O3"), Some(2.0 / 4.0));
}

#[test]
fn test_chemical_invariants_and_sanity_constraints() {
    // Mass & moles non-negativity
    assert!(ChemicalInvariantValidator::validate_mass_non_negative(0.0).is_ok());
    assert!(ChemicalInvariantValidator::validate_mass_non_negative(150.0).is_ok());
    assert!(ChemicalInvariantValidator::validate_mass_non_negative(-0.01).is_err());

    assert!(ChemicalInvariantValidator::validate_moles_non_negative(0.05).is_ok());
    assert!(ChemicalInvariantValidator::validate_moles_non_negative(-1.0).is_err());

    // Concentration & volume
    assert!(ChemicalInvariantValidator::validate_concentration_non_negative(0.5).is_ok());
    assert!(ChemicalInvariantValidator::validate_concentration_non_negative(-0.1).is_err());

    assert!(ChemicalInvariantValidator::validate_volume_positive(0.001).is_ok());
    assert!(ChemicalInvariantValidator::validate_volume_positive(0.0).is_err());

    // Equilibrium constant
    assert!(ChemicalInvariantValidator::validate_equilibrium_constant_positive(45.0).is_ok());
    assert!(ChemicalInvariantValidator::validate_equilibrium_constant_positive(0.0).is_err());

    // Limiting reagent check
    // 5 mol N2 (coeff 1) vs 9 mol H2 (coeff 3)
    // N2 ratio = 5/1 = 5.0; H2 ratio = 9/3 = 3.0 -> H2 is limiting
    assert!(ChemicalInvariantValidator::validate_limiting_reagent(5.0, 1, "N2", 9.0, 3, "H2", "H2").is_ok());
    assert!(ChemicalInvariantValidator::validate_limiting_reagent(5.0, 1, "N2", 9.0, 3, "H2", "N2").is_err());
}

#[test]
fn test_chemistry_catalog_and_schema_resolution() {
    let service = ProceduralService::open_in_memory().unwrap();

    // 1. Resolve Stoichiometry Schema
    let stk_schema = service
        .resolve_schema(&SchemaId::new("chemistry.stoichiometry.moles"))
        .unwrap()
        .expect("Stoichiometry schema should resolve");
    assert_eq!(stk_schema.id.as_str(), SCHEMA_CHEMISTRY_STOICHIOMETRY);
    assert_eq!(stk_schema.skill_id.as_str(), SKILL_CHEMISTRY_STOICHIOMETRY);

    // 2. Resolve Equilibrium Schema
    let eq_schema = service
        .resolve_schema(&SchemaId::new("chemistry.equilibrium.concentration"))
        .unwrap()
        .expect("Equilibrium schema should resolve");
    assert_eq!(eq_schema.id.as_str(), SCHEMA_CHEMISTRY_EQUILIBRIUM);
    assert_eq!(eq_schema.skill_id.as_str(), SKILL_CHEMISTRY_EQUILIBRIUM);

    // 3. Verify total schema count = 30 (14 Maths + 2 Physics + 6 Chemistry + 8 Reasoning)
    let all_schemas = service.store().list_all_schemas().unwrap();
    assert_eq!(all_schemas.len(), 30);
}

#[test]
fn test_chemistry_multi_schema_selection_and_interleaving() {
    let service = ProceduralService::open_in_memory().unwrap();

    let stk_schema = service
        .resolve_schema(&SchemaId::new(SCHEMA_CHEMISTRY_STOICHIOMETRY))
        .unwrap()
        .unwrap();
    let eq_schema = service
        .resolve_schema(&SchemaId::new(SCHEMA_CHEMISTRY_EQUILIBRIUM))
        .unwrap()
        .unwrap();

    let candidate_schemas = vec![stk_schema.clone(), eq_schema.clone()];
    let skill_states = HashMap::new();

    // Select with last_schema = Stoichiometry -> Should interleave to Equilibrium
    let decision = MultiSchemaSelector::select_next_schema(
        &PracticeMode::MixedChemistry,
        &candidate_schemas,
        &skill_states,
        Some(&stk_schema.id),
        1001,
    );

    assert!(decision.is_some());
    let dec = decision.unwrap();
    assert_eq!(dec.schema.id, eq_schema.id);
    assert_eq!(dec.difficulty_level, 1);
}

#[test]
fn test_chemistry_stepwise_solution_graph_validation_and_hints() {
    let service = ProceduralService::open_in_memory().unwrap();

    let schema = service
        .resolve_schema(&SchemaId::new(SCHEMA_CHEMISTRY_STOICHIOMETRY))
        .unwrap()
        .unwrap();

    // Generate Level 2 Problem (Reaction Mole Ratio)
    let inst = service
        .generate_problem(&schema.problem_family_id, 3001, &serde_json::json!({"difficulty": 2}))
        .unwrap();
    let graph = inst.solution_graph().expect("SolutionGraph must exist");

    assert_eq!(graph.steps.len(), 2);
    assert_eq!(graph.steps[0].step_type, StepType::ApplyStoichiometricRatio);
    assert_eq!(graph.steps[1].step_type, StepType::FinalAnswer);

    // Verify hint ladder
    let hints = graph.hints_for_step(0);
    assert_eq!(hints.len(), 3);
    assert_eq!(hints[0].hint_level, HintLevel::Principle);
    assert_eq!(hints[1].hint_level, HintLevel::Operation);
    assert_eq!(hints[2].hint_level, HintLevel::IntermediateRelation);

    // 1. Submit all steps correct stepwise
    let correct_moles = inst.correct_answer.get("value").unwrap().as_f64().unwrap();
    let a = inst.parameters.get("ratio_coeff_source").unwrap().as_f64().unwrap();
    let b = inst.parameters.get("ratio_coeff_target").unwrap().as_f64().unwrap();

    let valid_sub = StepwiseSubmission::stepwise(
        vec![
            SubmittedStep::new(0, format!("ratio = {}/{}", b, a), 5000).with_step_id("ratio"),
            SubmittedStep::new(1, format!("{:.2}", correct_moles), 8000).with_step_id("calc_moles"),
        ],
        Some(format!("{:.2}", correct_moles)),
        13000,
    );

    let step_eval = StepValidator::evaluate_submission(&graph, &valid_sub, 35000);
    assert!(step_eval.is_correct);
    assert_eq!(step_eval.steps_correct, 2);
    assert_eq!(step_eval.score, 1.0);
}

#[test]
fn test_chemistry_end_to_end_stoichiometry_quick_solve_and_fsrs_rating() {
    let service = ProceduralService::open_in_memory().unwrap();

    // 1. Card Anchor linking to Chemistry Stoichiometry
    let anchor = ProceduralCardAnchor::new("chemistry.stoichiometry.moles")
        .with_seed_mode(SeedMode::Fixed(4001));

    let session = service.prepare_practice_session(&anchor, Some(101)).unwrap();
    assert_eq!(session.schema.id.as_str(), SCHEMA_CHEMISTRY_STOICHIOMETRY);
    assert!(!session.instance.rendered_prompt.is_empty());

    let correct_val = session
        .instance
        .correct_answer
        .get("value")
        .unwrap()
        .as_f64()
        .unwrap();

    // 2. Student submits correct answer in 18 seconds (target = 25s)
    let outcome = service
        .evaluate_and_record_attempt(
            &session.instance.id,
            Some(101),
            serde_json::json!(correct_val),
            18000,
            0,
            1,
        )
        .unwrap();

    assert!(outcome.is_correct);
    assert_eq!(outcome.score, 1.0);
    assert!(matches!(derive_fsrs_rating(&outcome, None), Rating::Good | Rating::Easy));

    // 3. Evaluate inverted stoichiometric ratio misconception
    let inst2 = service
        .generate_problem(
            &session.schema.problem_family_id,
            4002,
            &serde_json::json!({"difficulty": 2}),
        )
        .unwrap();
    service.save_problem_instance(inst2.clone()).unwrap();

    let a = inst2.parameters.get("ratio_coeff_source").unwrap().as_f64().unwrap();
    let b = inst2.parameters.get("ratio_coeff_target").unwrap().as_f64().unwrap();
    let n_reactant = inst2.parameters.get("n_reactant").unwrap().as_f64().unwrap();
    let inverted_moles = n_reactant * (a / b);

    let bad_outcome = service
        .evaluate_and_record_attempt(
            &inst2.id,
            Some(101),
            serde_json::json!(inverted_moles),
            14000,
            0,
            1,
        )
        .unwrap();

    assert!(!bad_outcome.is_correct);
    assert_eq!(bad_outcome.error_category, Some(ErrorCategory::Strategy));
    assert_eq!(derive_fsrs_rating(&bad_outcome, None), Rating::Again);
}
