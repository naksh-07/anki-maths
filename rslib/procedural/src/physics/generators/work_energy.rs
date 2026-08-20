// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

use crate::core::{ProblemFamilyId, Result};
use crate::diagnostics::ErrorCategory;
use crate::physics::diagnostics::PhysicsErrorCategory;
use crate::physics::models::{
    CoordinateSystem, PhysicalModelKind, PhysicalProblemMetadata, PhysicalQuantity, PhysicalRegime,
};
use crate::physics::sanity::PhysicalSanityValidator;
use crate::physics::units::PhysicsUnit;
use crate::problems::generator::ProblemGenerator;
use crate::problems::steps::{
    SolutionGraph, StepHint, StepNode, StepType, StepValidator, StepwiseSubmission,
};
use crate::problems::validator::{AnswerEvaluation, NumericAnswerParser, ProblemValidator};
use crate::problems::ProblemInstance;

pub const FAMILY_PHYSICS_WORK_ENERGY: &str = "family.physics.work_energy.mechanics";
pub const TEMPLATE_PHYSICS_WORK_ENERGY_V1: &str = "physics.work_energy.mechanics.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkEnergyVariant {
    DirectKineticPotentialEnergy,
    WorkDoneConstantForce,
    WorkEnergyTheorem,
    ConservationMechanicalEnergy,
    PowerResistiveInclineTransfer,
}

impl WorkEnergyVariant {
    pub fn as_str(&self) -> &'static str {
        match self {
            WorkEnergyVariant::DirectKineticPotentialEnergy => "direct_kinetic_potential_energy",
            WorkEnergyVariant::WorkDoneConstantForce => "work_done_constant_force",
            WorkEnergyVariant::WorkEnergyTheorem => "work_energy_theorem",
            WorkEnergyVariant::ConservationMechanicalEnergy => "conservation_mechanical_energy",
            WorkEnergyVariant::PowerResistiveInclineTransfer => "power_resistive_incline_transfer",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct WorkEnergyGenerator;

impl WorkEnergyGenerator {
    pub fn generate_problem(seed: u64, difficulty_level: u32, variant: Option<&str>) -> ProblemInstance {
        let mut rng = StdRng::seed_from_u64(seed);
        let chosen_variant = if let Some(v) = variant {
            match v {
                "direct_kinetic_potential_energy" => WorkEnergyVariant::DirectKineticPotentialEnergy,
                "work_done_constant_force" => WorkEnergyVariant::WorkDoneConstantForce,
                "work_energy_theorem" => WorkEnergyVariant::WorkEnergyTheorem,
                "conservation_mechanical_energy" => WorkEnergyVariant::ConservationMechanicalEnergy,
                "power_resistive_incline_transfer" => WorkEnergyVariant::PowerResistiveInclineTransfer,
                _ => WorkEnergyVariant::DirectKineticPotentialEnergy,
            }
        } else {
            match difficulty_level {
                1 => WorkEnergyVariant::DirectKineticPotentialEnergy,
                2 => WorkEnergyVariant::WorkDoneConstantForce,
                3 => WorkEnergyVariant::WorkEnergyTheorem,
                4 => WorkEnergyVariant::ConservationMechanicalEnergy,
                _ => WorkEnergyVariant::PowerResistiveInclineTransfer,
            }
        };

        match chosen_variant {
            WorkEnergyVariant::DirectKineticPotentialEnergy => Self::generate_level_1(&mut rng, seed),
            WorkEnergyVariant::WorkDoneConstantForce => Self::generate_level_2(&mut rng, seed),
            WorkEnergyVariant::WorkEnergyTheorem => Self::generate_level_3(&mut rng, seed),
            WorkEnergyVariant::ConservationMechanicalEnergy => Self::generate_level_4(&mut rng, seed),
            WorkEnergyVariant::PowerResistiveInclineTransfer => Self::generate_level_5(&mut rng, seed),
        }
    }

    /// Level 1: Direct Kinetic Energy: KE = 1/2 m v^2
    fn generate_level_1(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let mass: f64 = (rng.random_range(1..=50) * 2) as f64; // even mass for clean 1/2 m
        let velocity: f64 = rng.random_range(2..=50) as f64; // m/s
        let ke = 0.5 * mass * velocity * velocity; // J

        let prompt = format!(
            "An autonomous cart with mass **{:.0} kg** moves along a flat test track at a speed of **{:.0} m/s**.\n\n\
             What is the kinetic energy of the cart in Joules (J)?",
            mass, velocity
        );

        let solution = format!(
            "**Step 1 (Physical Model & Formula):**\n\
             Kinetic energy formula for mass \\(m\\) moving at velocity \\(v\\):\n\
             \\[ KE = \\frac{{1}}{{2}} m v^2 \\]\n\n\
             **Step 2 (Substitution & Calculation):**\n\
             \\[ KE = \\frac{{1}}{{2}}({:.0})({:.0}^2) = {:.0} \\times {:.0} = **{:.0} J** \\]",
            mass, velocity, 0.5 * mass, velocity * velocity, ke
        );

        let physics_meta = PhysicalProblemMetadata {
            regime: PhysicalRegime::default(),
            coordinate_system: CoordinateSystem::default(),
            candidate_models: vec![PhysicalModelKind::MechanicalEnergyConservation, PhysicalModelKind::WorkEnergyTheorem],
            governing_model: PhysicalModelKind::MechanicalEnergyConservation,
            known_quantities: vec![
                PhysicalQuantity::known("Mass", "m", mass, PhysicsUnit::Kilogram),
                PhysicalQuantity::known("Velocity", "v", velocity, PhysicsUnit::MeterPerSecond),
            ],
            target_quantity: PhysicalQuantity::unknown("Kinetic Energy", "KE", PhysicsUnit::Joule),
            governing_equations: vec!["KE = 0.5 * m * v^2".to_string()],
        };

        let parameters = serde_json::json!({
            "variant": "direct_kinetic_potential_energy",
            "mass": mass,
            "velocity": velocity,
            "ke": ke,
            "unit": "J",
            "physics_metadata": physics_meta,
        });

        let correct_answer = serde_json::json!({
            "value": ke,
            "unit": "J",
            "formatted": format!("{:.0} J", ke),
            "solution": solution,
        });

        let step1 = StepNode::new(
            "ke_formula",
            StepType::SelectEquation,
            "Identify Kinetic Energy Equation",
            "KE = (1/2) * m * v^2".to_string(),
            "KE = 0.5*m*v^2".to_string(),
        )
        .with_hints(vec![
            StepHint::principle("Kinetic energy is defined as half of mass times velocity squared."),
            StepHint::operation("Set up KE = (1/2) * m * v^2."),
            StepHint::intermediate_relation("KE = 0.5 * m * v^2"),
        ]);

        let step2 = StepNode::new(
            "calc_ke",
            StepType::FinalAnswer,
            "Calculate KE value",
            format!("0.5 * {} * {}^2 = {} J", mass, velocity, ke),
            format!("{:.0}", ke),
        )
        .with_expected_value(ke)
        .with_dependencies(vec!["ke_formula".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Square the velocity first, then multiply by half the mass."),
            StepHint::operation(format!("Compute 0.5 * {:.0} * {:.0}^2.", mass, velocity)),
            StepHint::intermediate_relation(format!("KE = {:.0} J", ke)),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2], "calc_ke");

        ProblemInstance::new(
            format!("inst-phys-nrg-l1-{}", seed),
            FAMILY_PHYSICS_WORK_ENERGY,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(serde_json::json!({
            "difficulty_level": 1,
            "target_time_ms": 25_000,
            "domain": "physics",
            "unit": "J",
        }))
    }

    /// Level 2: Work Done by Constant Force at an Angle: W = F * d * cos(theta)
    fn generate_level_2(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let force: f64 = rng.random_range(10..=250) as f64;
        let distance: f64 = rng.random_range(2..=50) as f64; // m
        let angle_deg: f64 = 60.0; // cos(60) = 0.5 exactly for clean arithmetic
        let cos_theta = 0.5;
        let work = force * distance * cos_theta; // J

        let prompt = format!(
            "A crate is pulled across a smooth horizontal warehouse floor by a tension force of **{:.0} N** acting at an angle of **{:.0}°** above the horizontal.\n\n\
             If the crate is pulled through a displacement of **{:.0} meters**, calculate the total work done by the tension force in Joules (J).\n\
             *(Note: \\(\\cos 60° = 0.5\\))*",
            force, angle_deg, distance
        );

        let solution = format!(
            "**Step 1 (Physical Model & Work Definition):**\n\
             Work done by a constant force at an angle \\(\\theta\\) to displacement \\(d\\):\n\
             \\[ W = F \\cdot d \\cdot \\cos\\theta \\]\n\n\
             **Step 2 (Substitution & Calculation):**\n\
             \\[ W = {:.0} \\text{{ N}} \\times {:.0} \\text{{ m}} \\times \\cos(60°) \\]\n\
             \\[ W = {:.0} \\times {:.0} \\times 0.5 = **{:.0} J** \\]",
            force, distance, force, distance, work
        );

        let physics_meta = PhysicalProblemMetadata {
            regime: PhysicalRegime::default(),
            coordinate_system: CoordinateSystem {
                dimension: "2D Planar / 1D Motion".to_string(),
                positive_direction: "horizontal displacement (+x)".to_string(),
                origin_reference: "starting position x = 0".to_string(),
            },
            candidate_models: vec![PhysicalModelKind::WorkEnergyTheorem],
            governing_model: PhysicalModelKind::WorkEnergyTheorem,
            known_quantities: vec![
                PhysicalQuantity::known("Force", "F", force, PhysicsUnit::Newton),
                PhysicalQuantity::known("Displacement", "d", distance, PhysicsUnit::Meter),
                PhysicalQuantity::known("Angle", "theta", angle_deg, PhysicsUnit::Dimensionless),
            ],
            target_quantity: PhysicalQuantity::unknown("Work Done", "W", PhysicsUnit::Joule),
            governing_equations: vec!["W = F * d * cos(theta)".to_string()],
        };

        let parameters = serde_json::json!({
            "variant": "work_done_constant_force",
            "force": force,
            "distance": distance,
            "angle_deg": angle_deg,
            "work": work,
            "unit": "J",
            "physics_metadata": physics_meta,
        });

        let correct_answer = serde_json::json!({
            "value": work,
            "unit": "J",
            "formatted": format!("{:.0} J", work),
            "solution": solution,
        });

        let step1 = StepNode::new(
            "work_formula",
            StepType::SelectEquation,
            "Identify Work Formula with Angle Component",
            "W = F * d * cos(theta)".to_string(),
            "W = F * d * cos(theta)".to_string(),
        )
        .with_hints(vec![
            StepHint::principle("Only the component of force parallel to the displacement (F * cos theta) does work."),
            StepHint::operation("Set up W = F * d * cos(60°)."),
            StepHint::intermediate_relation("W = F * d * 0.5"),
        ]);

        let step2 = StepNode::new(
            "calc_work",
            StepType::FinalAnswer,
            "Compute work value",
            format!("{} * {} * 0.5 = {} J", force, distance, work),
            format!("{:.0}", work),
        )
        .with_expected_value(work)
        .with_dependencies(vec!["work_formula".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Multiply force magnitude, distance, and 0.5 (cos 60°)."),
            StepHint::operation(format!("Compute {:.0} * {:.0} * 0.5.", force, distance)),
            StepHint::intermediate_relation(format!("W = {:.0} J", work)),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2], "calc_work");

        ProblemInstance::new(
            format!("inst-phys-nrg-l2-{}", seed),
            FAMILY_PHYSICS_WORK_ENERGY,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(serde_json::json!({
            "difficulty_level": 2,
            "target_time_ms": 30_000,
            "domain": "physics",
            "unit": "J",
        }))
    }

    /// Level 3: Work-Energy Theorem: W_net = Delta KE = 1/2 m (v_f^2 - v_i^2)
    fn generate_level_3(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let mass: f64 = (rng.random_range(1..=20) * 2) as f64; // kg
        let v_i: f64 = rng.random_range(2..=20) as f64; // m/s
        let v_f: f64 = v_i + rng.random_range(2..=20) as f64; // m/s
        let delta_ke = 0.5 * mass * (v_f * v_f - v_i * v_i); // J (work done)

        let prompt = format!(
            "A net horizontal force acts on a **{:.0} kg** robotic rover, accelerating it from an initial speed of **{:.0} m/s** to a final speed of **{:.0} m/s**.\n\n\
             Using the Work-Energy Theorem, what is the net work done on the rover in Joules (J)?",
            mass, v_i, v_f
        );

        let solution = format!(
            "**Step 1 (Work-Energy Theorem):**\n\
             \\[ W_{{\\text{{net}}}} = \\Delta KE = KE_f - KE_i = \\frac{{1}}{{2}} m (v_f^2 - v_i^2) \\]\n\n\
             **Step 2 (State Energy Calculations):**\n\
             - Initial KE: \\( \\frac{{1}}{{2}}({:.0})({:.0}^2) = {:.0} \\text{{ J}} \\)\n\
             - Final KE: \\( \\frac{{1}}{{2}}({:.0})({:.0}^2) = {:.0} \\text{{ J}} \\)\n\n\
             **Step 3 (Net Work Done):**\n\
             \\[ W_{{\\text{{net}}}} = {:.0} - {:.0} = **{:.0} J** \\]",
            mass, v_i, 0.5 * mass * v_i * v_i, mass, v_f, 0.5 * mass * v_f * v_f,
            0.5 * mass * v_f * v_f, 0.5 * mass * v_i * v_i, delta_ke
        );

        let physics_meta = PhysicalProblemMetadata {
            regime: PhysicalRegime::default(),
            coordinate_system: CoordinateSystem::default(),
            candidate_models: vec![PhysicalModelKind::WorkEnergyTheorem, PhysicalModelKind::KinematicsConstantAcceleration],
            governing_model: PhysicalModelKind::WorkEnergyTheorem,
            known_quantities: vec![
                PhysicalQuantity::known("Mass", "m", mass, PhysicsUnit::Kilogram),
                PhysicalQuantity::known("Initial Velocity", "v_i", v_i, PhysicsUnit::MeterPerSecond),
                PhysicalQuantity::known("Final Velocity", "v_f", v_f, PhysicsUnit::MeterPerSecond),
            ],
            target_quantity: PhysicalQuantity::unknown("Net Work Done", "W_net", PhysicsUnit::Joule),
            governing_equations: vec!["W_net = 0.5 * m * (v_f^2 - v_i^2)".to_string()],
        };

        let parameters = serde_json::json!({
            "variant": "work_energy_theorem",
            "mass": mass,
            "v_i": v_i,
            "v_f": v_f,
            "delta_ke": delta_ke,
            "unit": "J",
            "physics_metadata": physics_meta,
        });

        let correct_answer = serde_json::json!({
            "value": delta_ke,
            "unit": "J",
            "formatted": format!("{:.0} J", delta_ke),
            "solution": solution,
        });

        let step1 = StepNode::new(
            "work_energy_principle",
            StepType::SelectModel,
            "Apply Work-Energy Theorem W = Delta KE",
            "W = 0.5*m*(v_f^2 - v_i^2)".to_string(),
            "work_energy_theorem".to_string(),
        )
        .with_alternates(vec!["work_energy_theorem".to_string(), "W = Delta KE".to_string()])
        .with_hints(vec![
            StepHint::principle("The net work done on an object equals the change in its kinetic energy."),
            StepHint::operation("Compute W = KE_final - KE_initial."),
            StepHint::intermediate_relation("W = 0.5 * m * (v_f^2 - v_i^2)"),
        ]);

        let step2 = StepNode::new(
            "calc_net_work",
            StepType::FinalAnswer,
            "Compute Delta KE = 0.5 * m * (v_f^2 - v_i^2)",
            format!("0.5 * {} * ({}^2 - {}^2) = {} J", mass, v_f, v_i, delta_ke),
            format!("{:.0}", delta_ke),
        )
        .with_expected_value(delta_ke)
        .with_dependencies(vec!["work_energy_principle".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Compute the difference between final and initial kinetic energy."),
            StepHint::operation(format!("Compute 0.5 * {:.0} * ({:.0}^2 - {:.0}^2).", mass, v_f, v_i)),
            StepHint::intermediate_relation(format!("W = {:.0} J", delta_ke)),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2], "calc_net_work");

        ProblemInstance::new(
            format!("inst-phys-nrg-l3-{}", seed),
            FAMILY_PHYSICS_WORK_ENERGY,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(serde_json::json!({
            "difficulty_level": 3,
            "target_time_ms": 35_000,
            "domain": "physics",
            "unit": "J",
        }))
    }

    /// Level 4: Conservation of Mechanical Energy: mgh_i = 1/2 m v_f^2 => v_f = sqrt(2gh)
    fn generate_level_4(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let g: f64 = 9.8; // m/s^2
        // Use v = 7 * k. v^2 = 49 * k^2. h = v^2 / 19.6 = 2.5 * k^2
        let k = rng.random_range(1..=20) as f64;
        let height: f64 = 2.5 * k * k; 
        let mass: f64 = rng.random_range(2..=50) as f64; // kg
        let v_final = (2.0 * g * height).sqrt(); // m/s

        let prompt = format!(
            "A roller-coaster car of mass **{:.0} kg** starts from rest at the top of a frictionless drop of height **{:.1} meters** above ground level.\n\n\
             Taking \\(g = 9.8\\text{{ m/s}}^2\\), calculate the speed of the car at the bottom of the drop in **m/s** using Conservation of Mechanical Energy.",
            mass, height
        );

        let solution = format!(
            "**Step 1 (Model Selection: Conservation of Mechanical Energy):**\n\
             Because the track is frictionless (no non-conservative forces), total mechanical energy is conserved:\n\
             \\[ E_i = E_f \\implies PE_i + KE_i = PE_f + KE_f \\]\n\n\
             **Step 2 (State Energy Setup):**\n\
             - Initial state (apex): \\(KE_i = 0\\), \\(PE_i = mgh\\)\n\
             - Final state (bottom): \\(PE_f = 0\\), \\(KE_f = \\frac{{1}}{{2}}mv_f^2\\)\n\
             \\[ mgh = \\frac{{1}}{{2}} m v_f^2 \\implies v_f = \\sqrt{{2gh}} \\]\n\n\
             **Step 3 (Calculation):**\n\
             \\[ v_f = \\sqrt{{2 \\times 9.8 \\times {:.1}}} = \\sqrt{{{:.0}}} = **{:.0} m/s** \\]",
            height, 2.0 * g * height, v_final
        );

        let physics_meta = PhysicalProblemMetadata {
            regime: PhysicalRegime {
                regime_name: "Frictionless Conservative Gravity Field".to_string(),
                gravity_acceleration: 9.8,
                friction_present: false,
                air_resistance_neglected: true,
            },
            coordinate_system: CoordinateSystem {
                dimension: "1D Vertical Datum".to_string(),
                positive_direction: "height above track datum (+h)".to_string(),
                origin_reference: "bottom track level h = 0".to_string(),
            },
            candidate_models: vec![
                PhysicalModelKind::MechanicalEnergyConservation,
                PhysicalModelKind::KinematicsConstantAcceleration,
            ],
            governing_model: PhysicalModelKind::MechanicalEnergyConservation,
            known_quantities: vec![
                PhysicalQuantity::known("Mass", "m", mass, PhysicsUnit::Kilogram),
                PhysicalQuantity::known("Drop Height", "h", height, PhysicsUnit::Meter),
                PhysicalQuantity::known("Gravity", "g", 9.8, PhysicsUnit::MeterPerSecondSquared),
            ],
            target_quantity: PhysicalQuantity::unknown("Final Speed", "v_f", PhysicsUnit::MeterPerSecond),
            governing_equations: vec!["E_i = E_f".to_string(), "v_f = sqrt(2 * g * h)".to_string()],
        };

        let parameters = serde_json::json!({
            "variant": "conservation_mechanical_energy",
            "mass": mass,
            "height": height,
            "v_final": v_final,
            "unit": "m/s",
            "physics_metadata": physics_meta,
        });

        let correct_answer = serde_json::json!({
            "value": v_final,
            "unit": "m/s",
            "formatted": format!("{:.0} m/s", v_final),
            "solution": solution,
        });

        let step1 = StepNode::new(
            "energy_conservation_equation",
            StepType::SelectModel,
            "Apply Conservation of Mechanical Energy mgh = 1/2 m v^2",
            "mgh = 0.5 * m * v^2 => v = sqrt(2gh)".to_string(),
            "mechanical_energy_conservation".to_string(),
        )
        .with_alternates(vec!["mechanical_energy_conservation".to_string(), "v = sqrt(2gh)".to_string()])
        .with_hints(vec![
            StepHint::principle("Gravitational potential energy at the top converts entirely into kinetic energy at the bottom."),
            StepHint::operation("Cancel mass m: v = sqrt(2 * g * h)."),
            StepHint::intermediate_relation(format!("v = sqrt(2 * 9.8 * {:.1})", height)),
        ]);

        let step2 = StepNode::new(
            "calc_speed_at_bottom",
            StepType::FinalAnswer,
            "Compute final speed v = sqrt(2 * 9.8 * h)",
            format!("sqrt(2 * 9.8 * {:.1}) = {:.0} m/s", height, v_final),
            format!("{:.0}", v_final),
        )
        .with_expected_value(v_final)
        .with_dependencies(vec!["energy_conservation_equation".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Multiply 2 * 9.8 * height and take the square root."),
            StepHint::operation(format!("Compute sqrt({:.0}).", 2.0 * g * height)),
            StepHint::intermediate_relation(format!("v = {:.0} m/s", v_final)),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2], "calc_speed_at_bottom");

        ProblemInstance::new(
            format!("inst-phys-nrg-l4-{}", seed),
            FAMILY_PHYSICS_WORK_ENERGY,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(serde_json::json!({
            "difficulty_level": 4,
            "target_time_ms": 40_000,
            "domain": "physics",
            "unit": "m/s",
        }))
    }

    /// Level 5: Power & Mechanical Rate Transfer: P = W / t = F * v
    fn generate_level_5(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let mass: f64 = (rng.random_range(20..=200) * 10) as f64;
        let height: f64 = rng.random_range(10..=60) as f64; // m
        let time: f64 = rng.random_range(5..=60) as f64; // s
        let g = 9.8;
        let work = mass * g * height; // J
        let power = work / time; // W

        let prompt = format!(
            "An electric crane lifts a cargo container of mass **{:.0} kg** vertically through a height of **{:.0} meters** at constant speed in **{:.0} seconds**.\n\n\
             Taking \\(g = 9.8\\text{{ m/s}}^2\\), calculate the minimum mechanical power output of the crane motor in Watts (W).",
            mass, height, time
        );

        let solution = format!(
            "**Step 1 (Work Done Against Gravity):**\n\
             Since velocity is constant, lifting force equals weight \\(F = mg\\):\n\
             \\[ W = mgh = {:.0} \\times 9.8 \\times {:.0} = {:.0} \\text{{ J}} \\]\n\n\
             **Step 2 (Power as Rate of Work):**\n\
             \\[ P = \\frac{{W}}{{t}} = \\frac{{{:.0}}}{{{:.0}}} = **{:.1} W** \\]",
            mass, height, work, work, time, power
        );

        let physics_meta = PhysicalProblemMetadata {
            regime: PhysicalRegime::default(),
            coordinate_system: CoordinateSystem {
                dimension: "1D Vertical".to_string(),
                positive_direction: "upward (+y)".to_string(),
                origin_reference: "ground level y = 0".to_string(),
            },
            candidate_models: vec![PhysicalModelKind::PowerWorkRelation, PhysicalModelKind::WorkEnergyTheorem],
            governing_model: PhysicalModelKind::PowerWorkRelation,
            known_quantities: vec![
                PhysicalQuantity::known("Mass", "m", mass, PhysicsUnit::Kilogram),
                PhysicalQuantity::known("Lift Height", "h", height, PhysicsUnit::Meter),
                PhysicalQuantity::known("Time", "t", time, PhysicsUnit::Second),
                PhysicalQuantity::known("Gravity", "g", 9.8, PhysicsUnit::MeterPerSecondSquared),
            ],
            target_quantity: PhysicalQuantity::unknown("Power Output", "P", PhysicsUnit::Watt),
            governing_equations: vec!["W = m * g * h".to_string(), "P = W / t".to_string()],
        };

        let parameters = serde_json::json!({
            "variant": "power_resistive_incline_transfer",
            "mass": mass,
            "height": height,
            "time": time,
            "work": work,
            "power": power,
            "unit": "W",
            "physics_metadata": physics_meta,
        });

        let correct_answer = serde_json::json!({
            "value": power,
            "unit": "W",
            "formatted": format!("{:.1} W", power),
            "solution": solution,
        });

        let step1 = StepNode::new(
            "calc_lifting_work",
            StepType::Transformation,
            "Compute work done against gravity W = mgh",
            format!("{} * 9.8 * {} = {} J", mass, height, work),
            format!("{:.0}", work),
        )
        .with_expected_value(work)
        .with_hints(vec![
            StepHint::principle("To lift at constant speed, force equals weight F = mg. Work W = mgh."),
            StepHint::operation(format!("Multiply {:.0} * 9.8 * {:.0}.", mass, height)),
            StepHint::intermediate_relation(format!("W = {:.0} J", work)),
        ]);

        let step2 = StepNode::new(
            "calc_power_output",
            StepType::FinalAnswer,
            "Divide Work by Time P = W / t",
            format!("{} / {} = {:.1} W", work, time, power),
            format!("{:.1}", power),
        )
        .with_expected_value(power)
        .with_dependencies(vec!["calc_lifting_work".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Power is the time rate of doing work P = W / t."),
            StepHint::operation(format!("Compute {:.0} / {:.0}.", work, time)),
            StepHint::intermediate_relation(format!("P = {:.1} W", power)),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2], "calc_power_output");

        ProblemInstance::new(
            format!("inst-phys-nrg-l5-{}", seed),
            FAMILY_PHYSICS_WORK_ENERGY,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(serde_json::json!({
            "difficulty_level": 5,
            "target_time_ms": 45_000,
            "domain": "physics",
            "unit": "W",
            "learning_object_level": "transfer",
        }))
    }
}

impl ProblemGenerator for WorkEnergyGenerator {
    fn family_id(&self) -> &str {
        FAMILY_PHYSICS_WORK_ENERGY
    }

    fn template_ref(&self) -> &str {
        TEMPLATE_PHYSICS_WORK_ENERGY_V1
    }

    fn supported_variants(&self) -> Vec<String> {
        vec![
            "direct_kinetic_potential_energy".to_string(),
            "work_done_constant_force".to_string(),
            "work_energy_theorem".to_string(),
            "conservation_mechanical_energy".to_string(),
            "power_resistive_incline_transfer".to_string(),
        ]
    }

    fn target_latency_ms(&self, difficulty_level: u32) -> u64 {
        match difficulty_level {
            1 => 25_000,
            2 => 30_000,
            3 => 35_000,
            4 => 40_000,
            _ => 45_000,
        }
    }

    fn generate(
        &self,
        _family_id: &ProblemFamilyId,
        seed: u64,
        difficulty_level: u32,
        variant: Option<&str>,
    ) -> Result<ProblemInstance> {
        Ok(Self::generate_problem(seed, difficulty_level, variant))
    }
}

pub struct WorkEnergyValidator;

impl ProblemValidator for WorkEnergyValidator {
    fn family_id(&self) -> &str {
        FAMILY_PHYSICS_WORK_ENERGY
    }

    fn evaluate(
        &self,
        instance: &ProblemInstance,
        student_input: &serde_json::Value,
        time_taken_ms: u64,
        target_time_ms: u64,
    ) -> AnswerEvaluation {
        // 1. Check stepwise evaluation if submitted
        if let Ok(stepwise) = serde_json::from_value::<StepwiseSubmission>(student_input.clone()) {
            if let Some(graph) = instance.solution_graph() {
                let step_eval = StepValidator::evaluate_submission(&graph, &stepwise, target_time_ms);
                let first_err_cat = step_eval.first_error_type.map(|e| {
                    PhysicsErrorCategory::from_step_error_type(e).to_common_error_category()
                });
                return AnswerEvaluation {
                    is_correct: step_eval.is_correct,
                    score: step_eval.score,
                    parsed_student_value: None,
                    canonical_value: instance.correct_answer.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    error_category: first_err_cat,
                    diagnostic_message: Some(step_eval.overall_feedback),
                };
            }
        }

        // 2. Numerical evaluation with unit and physical sanity checking
        let expected_val = instance
            .correct_answer
            .get("value")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let parsed_num = NumericAnswerParser::parse_student_answer(student_input);
        let Some(num) = parsed_num else {
            return AnswerEvaluation::incorrect(
                ErrorCategory::Calculation,
                "Could not parse numeric answer. Please provide a valid physical quantity.",
            )
            .with_parsed_values(0.0, expected_val);
        };

        // Physical sanity check: Energy / Power cannot be negative in this context
        if let Err(sanity_err) = PhysicalSanityValidator::check_kinetic_energy(num) {
            return AnswerEvaluation::incorrect(
                ErrorCategory::Concept,
                format!("Physical Sanity Violation: {}", sanity_err),
            )
            .with_parsed_values(num, expected_val);
        }

        // Check common work-energy misconceptions
        let variant = instance.parameters.get("variant").and_then(|v| v.as_str()).unwrap_or("");
        if variant == "direct_kinetic_potential_energy" {
            let mass = instance.parameters.get("mass").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let velocity = instance.parameters.get("velocity").and_then(|v| v.as_f64()).unwrap_or(0.0);

            // Misconception 1: Forgot 1/2 factor (m * v^2)
            let forgot_half = mass * velocity * velocity;
            if (num - forgot_half).abs() < 1e-4 {
                return AnswerEvaluation::incorrect(
                    ErrorCategory::Concept,
                    "Equation Setup Error: Forgot the 1/2 factor in kinetic energy formula KE = 1/2 m v^2.",
                )
                .with_parsed_values(num, expected_val);
            }

            // Misconception 2: Forgot to square velocity (1/2 m v)
            let forgot_square = 0.5 * mass * velocity;
            if (num - forgot_square).abs() < 1e-4 {
                return AnswerEvaluation::incorrect(
                    ErrorCategory::Concept,
                    "Equation Setup Error: Velocity must be squared in kinetic energy formula KE = 1/2 m v^2 (did not square v).",
                )
                .with_parsed_values(num, expected_val);
            }
        }

        // Check numerical tolerance (0.5% or 0.1 abs)
        let tol = (expected_val.abs() * 0.005).max(0.1);
        if (num - expected_val).abs() <= tol {
            let score = if target_time_ms > 0 && time_taken_ms > target_time_ms * 2 {
                0.8
            } else {
                1.0
            };
            AnswerEvaluation::correct(score, time_taken_ms, target_time_ms).with_parsed_values(num, expected_val)
        } else {
            AnswerEvaluation::incorrect(
                ErrorCategory::Calculation,
                format!("Incorrect numerical answer. Expected {:.2}.", expected_val),
            )
            .with_parsed_values(num, expected_val)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_work_energy_generation_all_levels() {
        for level in 1..=5 {
            let inst = WorkEnergyGenerator::generate_problem(54321 + level as u64, level, None);
            assert_eq!(inst.family_id.as_str(), FAMILY_PHYSICS_WORK_ENERGY);
            assert!(!inst.rendered_prompt.is_empty());
            assert!(inst.correct_answer.get("value").is_some());
            assert!(inst.solution_graph().is_some());
        }
    }

    #[test]
    fn test_work_energy_forgot_half_diagnostic() {
        let generator = WorkEnergyGenerator;
        let validator = WorkEnergyValidator;
        let inst = generator.generate(&ProblemFamilyId::new(FAMILY_PHYSICS_WORK_ENERGY), 101, 1, None).unwrap();

        let mass = inst.parameters.get("mass").unwrap().as_f64().unwrap();
        let velocity = inst.parameters.get("velocity").unwrap().as_f64().unwrap();

        // Submit forgot half factor: m * v^2
        let bad_sub = serde_json::json!(mass * velocity * velocity);
        let eval = validator.evaluate(&inst, &bad_sub, 15000, 25000);

        assert!(!eval.is_correct);
        assert_eq!(eval.error_category, Some(ErrorCategory::Concept));
        assert!(eval.diagnostic_message.unwrap().contains("Forgot the 1/2 factor"));
    }
}
