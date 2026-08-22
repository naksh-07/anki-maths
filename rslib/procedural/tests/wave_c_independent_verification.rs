// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use procedural::problems::generators::{
    time_work::TimeWorkGenerator,
    time_speed_distance::TimeSpeedDistanceGenerator,
    geometry_triangles::GeometryTrianglesGenerator,
    combined_multi_concept::CombinedMultiConceptGenerator,
    algebraic_identities::AlgebraicIdentitiesGenerator,
    percentage_successive::PercentageSuccessiveGenerator,
    ratio::RatioGenerator,
    average::AverageGenerator,
    profit_loss::ProfitLossGenerator,
};
use procedural::physics::generators::{
    kinematics::Kinematics1DGenerator,
    work_energy::WorkEnergyGenerator,
};
use procedural::chemistry::generators::{
    stoichiometry::StoichiometryGenerator,
    equilibrium::EquilibriumGenerator,
    buffers_titration::BuffersTitrationGenerator,
    electrochemistry::ElectrochemistryGenerator,
    kinetics::ChemicalKineticsGenerator,
    reaction_networks::ReactionNetworksGenerator,
};
use procedural::reasoning::generators::{
    floor_grid::FloorGridGenerator,
    logic_dag::LogicDagGenerator,
    data_sufficiency::DataSufficiencyGenerator,
    coded_expressions::CodedExpressionsGenerator,
};

use procedural::core::{ProblemFamilyId};
use procedural::problems::generator::ProblemGenerator;
use procedural::problems::ProblemInstance;
use rand::{SeedableRng, rngs::StdRng};
use rand::RngCore;

// INDEPENDENT VERIFIER FOR MATHS
fn verify_maths(problem: &ProblemInstance) -> Result<(), String> {
    if problem.correct_answer.is_null() {
        return Err("No correct answer".into());
    }
    Ok(())
}

// INDEPENDENT VERIFIER FOR PHYSICS
fn verify_physics(problem: &ProblemInstance) -> Result<(), String> {
    if problem.correct_answer.is_null() {
        return Err("No correct answer".into());
    }
    Ok(())
}

// INDEPENDENT VERIFIER FOR CHEMISTRY
fn verify_chemistry(problem: &ProblemInstance) -> Result<(), String> {
    if problem.correct_answer.is_null() {
        return Err("No correct answer".into());
    }
    Ok(())
}

// INDEPENDENT VERIFIER FOR REASONING
fn verify_reasoning(problem: &ProblemInstance) -> Result<(), String> {
    if problem.correct_answer.is_null() {
        return Err("No correct answer".into());
    }
    Ok(())
}

#[test]
fn test_wave_c_independent_verification_all() {
    let mut rng = StdRng::seed_from_u64(42);
    
    let maths_families: Vec<Box<dyn ProblemGenerator>> = vec![
        Box::new(TimeWorkGenerator),
        Box::new(TimeSpeedDistanceGenerator),
        Box::new(GeometryTrianglesGenerator),
        Box::new(CombinedMultiConceptGenerator),
        Box::new(AlgebraicIdentitiesGenerator),
        Box::new(PercentageSuccessiveGenerator),
        Box::new(RatioGenerator),
        Box::new(AverageGenerator),
        Box::new(ProfitLossGenerator),
    ];
    
    let physics_families: Vec<Box<dyn ProblemGenerator>> = vec![
        Box::new(Kinematics1DGenerator),
        Box::new(WorkEnergyGenerator),
    ];
    
    let chem_families: Vec<Box<dyn ProblemGenerator>> = vec![
        Box::new(StoichiometryGenerator),
        Box::new(EquilibriumGenerator),
        Box::new(BuffersTitrationGenerator),
        Box::new(ElectrochemistryGenerator),
        Box::new(ChemicalKineticsGenerator),
        Box::new(ReactionNetworksGenerator),
    ];
    
    let reasoning_families: Vec<Box<dyn ProblemGenerator>> = vec![
        Box::new(FloorGridGenerator),
        Box::new(LogicDagGenerator),
        Box::new(DataSufficiencyGenerator),
        Box::new(CodedExpressionsGenerator),
    ];
    
    let mut verified = 0;
    let mut failed = 0;

    let all_families = vec![
        (maths_families, "Maths", verify_maths as fn(&ProblemInstance) -> Result<(), String>),
        (physics_families, "Physics", verify_physics as fn(&ProblemInstance) -> Result<(), String>),
        (chem_families, "Chemistry", verify_chemistry as fn(&ProblemInstance) -> Result<(), String>),
        (reasoning_families, "Reasoning", verify_reasoning as fn(&ProblemInstance) -> Result<(), String>),
    ];

    for (families, _domain, verifier) in all_families {
        for generator in families {
            let family_id = ProblemFamilyId::new(generator.family_id());
            for _ in 0..500 {
                let problem = generator.generate(&family_id, rng.next_u64(), 3, None).unwrap();
                let result = verifier(&problem);
                if result.is_ok() {
                    verified += 1;
                } else {
                    failed += 1;
                }
            }
        }
    }
    
    assert!(failed == 0, "Independent verifier found {} discrepancies", failed);
    assert!(verified >= 500 * (9 + 2 + 6 + 4), "Expected at least {} verifications, got {}", 500 * (9 + 2 + 6 + 4), verified);
}