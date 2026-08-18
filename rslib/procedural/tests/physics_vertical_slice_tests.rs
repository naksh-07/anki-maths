// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::collections::HashMap;

use procedural::core::{Domain, SchemaId, SkillId};
use procedural::diagnostics::ErrorCategory;
use procedural::physics::{
    DimensionalValidator, PhysicalDimension, PhysicalSanityValidator,
    PhysicsErrorCategory, PhysicsUnit,
};
use procedural::problems::catalog::{
    SCHEMA_PHYSICS_KINEMATICS, SCHEMA_PHYSICS_WORK_ENERGY,
    SKILL_PHYSICS_KINEMATICS, SKILL_PHYSICS_WORK_ENERGY,
};
use procedural::problems::steps::{
    HintLevel, StepType, StepValidator, StepwiseSubmission, SubmittedStep,
};
use procedural::scheduling::{
    derive_fsrs_rating, MultiSchemaSelector, PracticeMode, Rating,
};
use procedural::service::ProceduralService;

#[test]
fn test_physics_units_dimensional_analysis_and_conversion() {
    // 1. Dimensions
    assert_eq!(PhysicsUnit::Meter.dimension(), PhysicalDimension::LENGTH);
    assert_eq!(PhysicsUnit::Second.dimension(), PhysicalDimension::TIME);
    assert_eq!(PhysicsUnit::Kilogram.dimension(), PhysicalDimension::MASS);
    assert_eq!(PhysicsUnit::MeterPerSecond.dimension(), PhysicalDimension::VELOCITY);
    assert_eq!(PhysicsUnit::KilometerPerHour.dimension(), PhysicalDimension::VELOCITY);
    assert_eq!(PhysicsUnit::MeterPerSecondSquared.dimension(), PhysicalDimension::ACCELERATION);
    assert_eq!(PhysicsUnit::Newton.dimension(), PhysicalDimension::FORCE);
    assert_eq!(PhysicsUnit::Joule.dimension(), PhysicalDimension::ENERGY);
    assert_eq!(PhysicsUnit::Watt.dimension(), PhysicalDimension::POWER);

    // 2. Dimensional compatibility
    assert!(DimensionalValidator::are_compatible(
        &PhysicsUnit::KilometerPerHour,
        &PhysicsUnit::MeterPerSecond
    ));
    assert!(!DimensionalValidator::are_compatible(
        &PhysicsUnit::Newton,
        &PhysicsUnit::Joule
    ));
    assert!(!DimensionalValidator::are_compatible(
        &PhysicsUnit::MeterPerSecond,
        &PhysicsUnit::MeterPerSecondSquared
    ));

    // 3. Conversion accuracy
    // 72 km/h == 20 m/s
    let v_si = DimensionalValidator::convert(72.0, &PhysicsUnit::KilometerPerHour, &PhysicsUnit::MeterPerSecond).unwrap();
    assert!((v_si - 20.0).abs() < 1e-6);

    // 20 m/s == 72 km/h
    let v_kmh = DimensionalValidator::convert(20.0, &PhysicsUnit::MeterPerSecond, &PhysicsUnit::KilometerPerHour).unwrap();
    assert!((v_kmh - 72.0).abs() < 1e-6);

    // 500 grams == 0.5 kg
    let m_kg = DimensionalValidator::convert(500.0, &PhysicsUnit::Gram, &PhysicsUnit::Kilogram).unwrap();
    assert!((m_kg - 0.5).abs() < 1e-6);

    // Incompatible conversion returns None
    assert!(DimensionalValidator::convert(100.0, &PhysicsUnit::Joule, &PhysicsUnit::Watt).is_none());
}

#[test]
fn test_physics_physical_sanity_validator_constraints() {
    // 1. Time must be non-negative
    assert!(PhysicalSanityValidator::check_time(0.0).is_ok());
    assert!(PhysicalSanityValidator::check_time(12.5).is_ok());
    assert!(PhysicalSanityValidator::check_time(-3.0).is_err());

    // 2. Mass must be strictly positive
    assert!(PhysicalSanityValidator::check_mass(0.1).is_ok());
    assert!(PhysicalSanityValidator::check_mass(100.0).is_ok());
    assert!(PhysicalSanityValidator::check_mass(0.0).is_err());
    assert!(PhysicalSanityValidator::check_mass(-5.0).is_err());

    // 3. Sub-light speed limit
    assert!(PhysicalSanityValidator::check_sublight_speed(5000.0).is_ok());
    assert!(PhysicalSanityValidator::check_sublight_speed(300_000_000.0).is_err());

    // 4. Kinetic energy must be non-negative
    assert!(PhysicalSanityValidator::check_kinetic_energy(0.0).is_ok());
    assert!(PhysicalSanityValidator::check_kinetic_energy(1500.0).is_ok());
    assert!(PhysicalSanityValidator::check_kinetic_energy(-10.0).is_err());

    // 5. Energy conservation checking
    assert!(PhysicalSanityValidator::check_energy_conservation(500.0, 500.02, 0.05).is_ok());
    assert!(PhysicalSanityValidator::check_energy_conservation(500.0, 450.0, 0.05).is_err());
}

#[test]
fn test_physics_error_taxonomy_and_mappings() {
    assert_eq!(
        PhysicsErrorCategory::ModelSelectionError.to_common_error_category(),
        ErrorCategory::Strategy
    );
    assert_eq!(
        PhysicsErrorCategory::UnitError.to_common_error_category(),
        ErrorCategory::Unit
    );
    assert_eq!(
        PhysicsErrorCategory::SignConventionError.to_common_error_category(),
        ErrorCategory::Sign
    );
    assert_eq!(
        PhysicsErrorCategory::AlgebraExecutionError.to_common_error_category(),
        ErrorCategory::Calculation
    );
    assert_eq!(
        PhysicsErrorCategory::PhysicalPlausibilityError.to_common_error_category(),
        ErrorCategory::Concept
    );
}

#[test]
fn test_physics_catalog_and_schema_resolution() {
    let service = ProceduralService::open_in_memory().unwrap();

    // 1. Resolve Kinematics schema by canonical and aliases
    let schema_kin = service.resolve_schema(&SchemaId::new(SCHEMA_PHYSICS_KINEMATICS)).unwrap();
    assert!(schema_kin.is_some());
    let s_kin = schema_kin.unwrap();
    assert_eq!(s_kin.skill_id.as_str(), SKILL_PHYSICS_KINEMATICS);

    let alias_kin = service.resolve_schema(&SchemaId::new("physics.kinematics.1d")).unwrap();
    assert!(alias_kin.is_some());

    // 2. Resolve Work-Energy schema by canonical and aliases
    let schema_nrg = service.resolve_schema(&SchemaId::new(SCHEMA_PHYSICS_WORK_ENERGY)).unwrap();
    assert!(schema_nrg.is_some());
    let s_nrg = schema_nrg.unwrap();
    assert_eq!(s_nrg.skill_id.as_str(), SKILL_PHYSICS_WORK_ENERGY);

    let alias_nrg = service.resolve_schema(&SchemaId::new("work_energy")).unwrap();
    assert!(alias_nrg.is_some());

    // 3. Verify skills registered with Domain::Physics
    let skill_kin = service.store().get_skill(&SkillId::new(SKILL_PHYSICS_KINEMATICS)).unwrap();
    assert!(skill_kin.is_some());
    assert_eq!(skill_kin.unwrap().domain, Domain::Physics);

    let skill_nrg = service.store().get_skill(&SkillId::new(SKILL_PHYSICS_WORK_ENERGY)).unwrap();
    assert!(skill_nrg.is_some());
    assert_eq!(skill_nrg.unwrap().domain, Domain::Physics);
}

#[test]
fn test_physics_end_to_end_kinematics_quick_solve_and_fsrs_rating() {
    let service = ProceduralService::open_in_memory().unwrap();

    let schema = service
        .resolve_schema(&SchemaId::new(SCHEMA_PHYSICS_KINEMATICS))
        .unwrap()
        .unwrap();

    // Generate Level 2 Problem (Kinematics with km/h unit conversion)
    let inst = service
        .generate_problem(&schema.problem_family_id, 1002, &serde_json::json!({"difficulty": 2}))
        .unwrap();
    service.save_problem_instance(inst.clone()).unwrap();

    // 1. Evaluate correct answer quickly -> Good / Easy rating
    let correct_val = inst.correct_answer.get("value").unwrap().as_f64().unwrap();
    let outcome = service
        .evaluate_and_record_attempt(
            &inst.id,
            Some(101),
            serde_json::json!(correct_val),
            15000,
            0,
            1,
        )
        .unwrap();

    assert!(outcome.is_correct);
    assert_eq!(outcome.score, 1.0);
    assert!(matches!(derive_fsrs_rating(&outcome, None), Rating::Good | Rating::Easy));

    // 2. Evaluate unit conversion misconception: student submitted unscaled km/h + at
    let inst2 = service
        .generate_problem(&schema.problem_family_id, 1003, &serde_json::json!({"difficulty": 2}))
        .unwrap();
    service.save_problem_instance(inst2.clone()).unwrap();

    let u_kmh = inst2.parameters.get("u_kmh").unwrap().as_f64().unwrap();
    let a = inst2.parameters.get("a").unwrap().as_f64().unwrap();
    let t = inst2.parameters.get("t").unwrap().as_f64().unwrap();
    let bad_val = u_kmh + a * t;

    let bad_outcome = service
        .evaluate_and_record_attempt(
            &inst2.id,
            Some(101),
            serde_json::json!(bad_val),
            12000,
            0,
            1,
        )
        .unwrap();

    assert!(!bad_outcome.is_correct);
    assert_eq!(bad_outcome.error_category, Some(ErrorCategory::Unit));
    assert_eq!(derive_fsrs_rating(&bad_outcome, None), Rating::Again);
}

#[test]
fn test_physics_stepwise_solution_graph_validation_and_hints() {
    let service = ProceduralService::open_in_memory().unwrap();

    let schema = service
        .resolve_schema(&SchemaId::new(SCHEMA_PHYSICS_WORK_ENERGY))
        .unwrap()
        .unwrap();

    // Generate Level 1 Problem (Direct Kinetic Energy KE = 1/2 m v^2)
    let inst = service
        .generate_problem(&schema.problem_family_id, 2001, &serde_json::json!({"difficulty": 1}))
        .unwrap();
    let graph = inst.solution_graph().expect("SolutionGraph must exist");

    assert_eq!(graph.steps.len(), 2);
    assert_eq!(graph.steps[0].step_type, StepType::SelectEquation);
    assert_eq!(graph.steps[1].step_type, StepType::FinalAnswer);

    // Verify hint ladder
    let hints = graph.hints_for_step(0);
    assert_eq!(hints.len(), 3);
    assert_eq!(hints[0].hint_level, HintLevel::Principle);
    assert_eq!(hints[1].hint_level, HintLevel::Operation);
    assert_eq!(hints[2].hint_level, HintLevel::IntermediateRelation);

    // 1. Submit all steps correct stepwise
    let correct_ke = inst.correct_answer.get("value").unwrap().as_f64().unwrap();
    let valid_sub = StepwiseSubmission::stepwise(
        vec![
            SubmittedStep::new(0, "KE = 0.5*m*v^2", 5000).with_step_id("ke_formula"),
            SubmittedStep::new(1, format!("{:.0}", correct_ke), 8000).with_step_id("calc_ke"),
        ],
        Some(format!("{:.0}", correct_ke)),
        13000,
    );

    let step_eval = StepValidator::evaluate_submission(&graph, &valid_sub, 25000);
    println!("step_eval result: {:#?}", step_eval);
    assert!(step_eval.is_correct);
    assert_eq!(step_eval.steps_correct, 2);
    assert_eq!(step_eval.score, 1.0);
    assert!(step_eval.first_error_step.is_none());

    // 2. Submit wrong formula (forgot half factor or wrong setup)
    let invalid_sub = StepwiseSubmission::stepwise(
        vec![
            SubmittedStep::new(0, "KE = m * v", 5000).with_step_id("ke_formula"),
        ],
        None,
        5000,
    );

    let step_eval2 = StepValidator::evaluate_submission(&graph, &invalid_sub, 25000);
    assert!(!step_eval2.is_correct);
    assert_eq!(step_eval2.first_error_step, Some(0));
    assert!(step_eval2.remediation_recommendation.is_some());
}

#[test]
fn test_physics_multi_schema_selection_and_interleaving() {
    let service = ProceduralService::open_in_memory().unwrap();

    let schemas = service.store().list_all_schemas().unwrap();
    let physics_schemas: Vec<_> = schemas
        .into_iter()
        .filter(|s| s.id.as_str().starts_with("physics_"))
        .collect();

    assert_eq!(physics_schemas.len(), 2);

    let state_map: HashMap<SkillId, procedural::SkillState> = HashMap::new();

    // Select problem in MixedPhysics mode
    let decision = MultiSchemaSelector::select_next_schema(
        &PracticeMode::MixedPhysics,
        &physics_schemas,
        &state_map,
        None,
        42,
    );

    assert!(decision.is_some());
    let dec = decision.unwrap();
    assert!(
        dec.schema.id.as_str() == SCHEMA_PHYSICS_KINEMATICS
            || dec.schema.id.as_str() == SCHEMA_PHYSICS_WORK_ENERGY
    );
    assert_eq!(dec.difficulty_level, 1); // Cold start defaults to L1
}
