// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

use crate::core::{ProblemFamilyId, Result};
use crate::core::decision::{CognitiveDecisionPoint, DecisionOption};
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

pub const FAMILY_PHYSICS_KINEMATICS: &str = "family.physics.kinematics.1d";
pub const TEMPLATE_PHYSICS_KINEMATICS_V1: &str = "physics.kinematics.1d.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KinematicsVariant {
    UniformMotion,
    ConstantAccelerationUnitConversion,
    KinematicEquationSelection,
    StoppingDistanceReverse,
    VerticalProjectileTransfer,
}

impl KinematicsVariant {
    pub fn as_str(&self) -> &'static str {
        match self {
            KinematicsVariant::UniformMotion => "uniform_motion",
            KinematicsVariant::ConstantAccelerationUnitConversion => "constant_acceleration_unit_conversion",
            KinematicsVariant::KinematicEquationSelection => "kinematic_equation_selection",
            KinematicsVariant::StoppingDistanceReverse => "stopping_distance_reverse",
            KinematicsVariant::VerticalProjectileTransfer => "vertical_projectile_transfer",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Kinematics1DGenerator;

impl Kinematics1DGenerator {
    pub fn generate_problem(seed: u64, difficulty_level: u32, variant: Option<&str>) -> ProblemInstance {
        let mut rng = StdRng::seed_from_u64(seed);
        let chosen_variant = if let Some(v) = variant {
            match v {
                "uniform_motion" => KinematicsVariant::UniformMotion,
                "constant_acceleration_unit_conversion" => KinematicsVariant::ConstantAccelerationUnitConversion,
                "kinematic_equation_selection" => KinematicsVariant::KinematicEquationSelection,
                "stopping_distance_reverse" => KinematicsVariant::StoppingDistanceReverse,
                "vertical_projectile_transfer" => KinematicsVariant::VerticalProjectileTransfer,
                _ => KinematicsVariant::UniformMotion,
            }
        } else {
            match difficulty_level {
                1 => KinematicsVariant::UniformMotion,
                2 => KinematicsVariant::ConstantAccelerationUnitConversion,
                3 => KinematicsVariant::KinematicEquationSelection,
                4 => KinematicsVariant::StoppingDistanceReverse,
                _ => KinematicsVariant::VerticalProjectileTransfer,
            }
        };

        let mut instance = match chosen_variant {
            KinematicsVariant::UniformMotion => Self::generate_level_1(&mut rng, seed),
            KinematicsVariant::ConstantAccelerationUnitConversion => Self::generate_level_2(&mut rng, seed),
            KinematicsVariant::KinematicEquationSelection => Self::generate_level_3(&mut rng, seed),
            KinematicsVariant::StoppingDistanceReverse => Self::generate_level_4(&mut rng, seed),
            KinematicsVariant::VerticalProjectileTransfer => Self::generate_level_5(&mut rng, seed),
        };

        let dp = CognitiveDecisionPoint::new(
            "dp_kinematic_equation",
            "Which kinematic equation is best suited to solve this problem?",
            vec![
                DecisionOption::new(
                    "opt_v_u_at",
                    "v = u + at",
                    "eq_v_u_at",
                    chosen_variant == KinematicsVariant::ConstantAccelerationUnitConversion,
                    "Use when displacement (s) is neither given nor requested.",
                ),
                DecisionOption::new(
                    "opt_s_ut",
                    "s = ut + 1/2 at^2",
                    "eq_s_ut",
                    chosen_variant == KinematicsVariant::StoppingDistanceReverse || chosen_variant == KinematicsVariant::VerticalProjectileTransfer,
                    "Use when final velocity (v) is neither given nor requested.",
                ),
                DecisionOption::new(
                    "opt_v2_u2",
                    "v^2 = u^2 + 2as",
                    "eq_v2_u2",
                    chosen_variant == KinematicsVariant::KinematicEquationSelection,
                    "Use when time (t) is neither given nor requested.",
                ),
            ],
            match chosen_variant {
                KinematicsVariant::ConstantAccelerationUnitConversion => "opt_v_u_at",
                KinematicsVariant::KinematicEquationSelection => "opt_v2_u2",
                _ => "opt_s_ut",
            },
            match chosen_variant {
                KinematicsVariant::ConstantAccelerationUnitConversion => "eq_v_u_at",
                KinematicsVariant::KinematicEquationSelection => "eq_v2_u2",
                _ => "eq_s_ut",
            },
            "Identify the missing variable (the one not given and not requested) to select the correct equation.",
        );

        if let Some(obj) = instance.metadata.as_object_mut() {
            obj.insert("decision_point".to_string(), serde_json::to_value(dp).unwrap());
        }

        instance
    }

    /// Level 1: Uniform Motion (Zero acceleration): s = v * t
    fn generate_level_1(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let speed: f64 = rng.random_range(2..=120) as f64; // m/s
        let time: f64 = rng.random_range(2..=120) as f64; // s
        let displacement = speed * time; // m

        let prompt = format!(
            "An electric vehicle cruises along a straight highway at a constant velocity of **{:.0} m/s** for **{:.0} seconds**.\n\n\
             Assuming zero acceleration, what is the total displacement traveled in meters?",
            speed, time
        );

        let solution = format!(
            "**Step 1 (Physical Model Selection):**\n\
             Since velocity is constant ($a = 0$), the governing model is **Uniform Motion** ($s = v \\cdot t$).\n\n\
             **Step 2 (Identify Knowns):**\n\
             - Velocity \\(v = {:.0}\\text{{ m/s}}\\)\n\
             - Time \\(t = {:.0}\\text{{ s}}\\)\n\
             - Target: Displacement \\(s\\)\n\n\
             **Step 3 (Calculation & Units):**\n\
             \\[ s = {:.0} \\text{{ m/s}} \\times {:.0} \\text{{ s}} = **{:.0} m** \\]",
            speed, time, speed, time, displacement
        );

        let physics_meta = PhysicalProblemMetadata {
            regime: PhysicalRegime::default(),
            coordinate_system: CoordinateSystem::default(),
            candidate_models: vec![
                PhysicalModelKind::KinematicsUniform,
                PhysicalModelKind::KinematicsConstantAcceleration,
            ],
            governing_model: PhysicalModelKind::KinematicsUniform,
            known_quantities: vec![
                PhysicalQuantity::known("Velocity", "v", speed, PhysicsUnit::MeterPerSecond),
                PhysicalQuantity::known("Time", "t", time, PhysicsUnit::Second),
            ],
            target_quantity: PhysicalQuantity::unknown("Displacement", "s", PhysicsUnit::Meter),
            governing_equations: vec!["s = v * t".to_string()],
        };

        let parameters = serde_json::json!({
            "variant": "uniform_motion",
            "speed": speed,
            "time": time,
            "displacement": displacement,
            "unit": "m",
            "physics_metadata": physics_meta,
        });

        let correct_answer = serde_json::json!({
            "value": displacement,
            "unit": "m",
            "formatted": format!("{:.0} m", displacement),
            "solution": solution,
        });

        let step1 = StepNode::new(
            "model_selection",
            StepType::SelectModel,
            "Select Physical Model",
            "Uniform Motion (a = 0)".to_string(),
            "kinematics_uniform".to_string(),
        )
        .with_alternates(vec!["Uniform Motion".to_string(), "s = vt".to_string(), "kinematics_uniform".to_string()])
        .with_hints(vec![
            StepHint::principle("Determine if acceleration is present. Constant velocity implies a = 0."),
            StepHint::operation("Select the Uniform Motion model s = v * t."),
            StepHint::intermediate_relation("s = v * t"),
        ]);

        let step2 = StepNode::new(
            "calc_displacement",
            StepType::FinalAnswer,
            "Calculate displacement s = v * t",
            format!("{} * {} = {} m", speed, time, displacement),
            format!("{:.0}", displacement),
        )
        .with_expected_value(displacement)
        .with_dependencies(vec!["model_selection".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Multiply constant speed by time elapsed."),
            StepHint::operation(format!("Compute {:.0} m/s * {:.0} s.", speed, time)),
            StepHint::intermediate_relation(format!("s = {:.0} m", displacement)),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2], "calc_displacement");

        ProblemInstance::new(
            format!("inst-phys-kin-l1-{}", seed),
            FAMILY_PHYSICS_KINEMATICS,
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
            "unit": "m",
        }))
    }

    /// Level 2: Constant Acceleration with km/h to m/s Unit Conversion: v = u + at
    fn generate_level_2(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let u_mps: f64 = rng.random_range(2..=60) as f64;
        let u_kmh = u_mps * 3.6;

        let a: f64 = rng.random_range(1..=15) as f64; // m/s^2
        let t: f64 = rng.random_range(2..=30) as f64; // s
        let v_mps = u_mps + a * t; // m/s

        let prompt = format!(
            "A high-speed train travels initially at **{:.0} km/h** and accelerates at **{:.0} m/s²** for **{:.0} seconds**.\n\n\
             What is the final velocity of the train in **m/s**?",
            u_kmh, a, t
        );

        let solution = format!(
            "**Step 1 (Unit Normalization to SI):**\n\
             Convert initial velocity from km/h to m/s:\n\
             \\[ u = {:.0} \\times \\frac{{5}}{{18}} = {:.0} \\text{{ m/s}} \\]\n\n\
             **Step 2 (Model & Equation Selection):**\n\
             Constant acceleration relation with knowns \\(u, a, t\\) and target \\(v\\):\n\
             \\[ v = u + at \\]\n\n\
             **Step 3 (Calculation):**\n\
             \\[ v = {:.0} + ({:.0} \\times {:.0}) = {:.0} + {:.0} = **{:.0} m/s** \\]",
            u_kmh, u_mps, u_mps, a, t, u_mps, a * t, v_mps
        );

        let physics_meta = PhysicalProblemMetadata {
            regime: PhysicalRegime::default(),
            coordinate_system: CoordinateSystem::default(),
            candidate_models: vec![
                PhysicalModelKind::KinematicsUniform,
                PhysicalModelKind::KinematicsConstantAcceleration,
            ],
            governing_model: PhysicalModelKind::KinematicsConstantAcceleration,
            known_quantities: vec![
                PhysicalQuantity::known("Initial Velocity", "u", u_kmh, PhysicsUnit::KilometerPerHour),
                PhysicalQuantity::known("Acceleration", "a", a, PhysicsUnit::MeterPerSecondSquared),
                PhysicalQuantity::known("Time", "t", t, PhysicsUnit::Second),
            ],
            target_quantity: PhysicalQuantity::unknown("Final Velocity", "v", PhysicsUnit::MeterPerSecond),
            governing_equations: vec!["v = u + a * t".to_string()],
        };

        let parameters = serde_json::json!({
            "variant": "constant_acceleration_unit_conversion",
            "u_kmh": u_kmh,
            "u_mps": u_mps,
            "a": a,
            "t": t,
            "v_mps": v_mps,
            "unit": "m/s",
            "physics_metadata": physics_meta,
        });

        let correct_answer = serde_json::json!({
            "value": v_mps,
            "unit": "m/s",
            "formatted": format!("{:.0} m/s", v_mps),
            "solution": solution,
        });

        let step1 = StepNode::new(
            "unit_conversion",
            StepType::UnitConversion,
            "Convert initial speed km/h to m/s",
            format!("{} * (5/18) = {} m/s", u_kmh, u_mps),
            format!("{:.0}", u_mps),
        )
        .with_expected_value(u_mps)
        .with_hints(vec![
            StepHint::principle("Always normalize units to SI base (m/s) before applying kinematic equations."),
            StepHint::operation(format!("Multiply {:.0} km/h by 5/18.", u_kmh)),
            StepHint::intermediate_relation(format!("u = {:.0} m/s", u_mps)),
        ]);

        let step2 = StepNode::new(
            "calc_final_velocity",
            StepType::FinalAnswer,
            "Calculate final velocity v = u + at",
            format!("{} + ({} * {}) = {} m/s", u_mps, a, t, v_mps),
            format!("{:.0}", v_mps),
        )
        .with_expected_value(v_mps)
        .with_dependencies(vec!["unit_conversion".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Apply the first kinematic equation v = u + at with normalized SI values."),
            StepHint::operation(format!("Compute {:.0} + ({:.0} * {:.0}).", u_mps, a, t)),
            StepHint::intermediate_relation(format!("v = {:.0} m/s", v_mps)),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2], "calc_final_velocity");

        ProblemInstance::new(
            format!("inst-phys-kin-l2-{}", seed),
            FAMILY_PHYSICS_KINEMATICS,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(serde_json::json!({
            "difficulty_level": 2,
            "target_time_ms": 35_000,
            "domain": "physics",
            "unit": "m/s",
        }))
    }

    /// Level 3: Kinematic Equation Selection & Displacement: s = ut + 1/2 a t^2
    fn generate_level_3(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let u: f64 = rng.random_range(2..=50) as f64; // m/s
        let a: f64 = (rng.random_range(1..=10) * 2) as f64; // even number for clean 1/2 a
        let t: f64 = rng.random_range(2..=25) as f64; // s
        let displacement = u * t + 0.5 * a * t * t; // m

        let prompt = format!(
            "A dragster starts with an initial velocity of **{:.0} m/s** and accelerates forward at **{:.0} m/s²** for **{:.0} seconds**.\n\n\
             Calculate the total distance traveled during this interval in meters.",
            u, a, t
        );

        let solution = format!(
            "**Step 1 (Equation Selection from Knowns):**\n\
             Knowns: \\(u = {:.0}\\text{{ m/s}}, a = {:.0}\\text{{ m/s}}^2, t = {:.0}\\text{{ s}}\\). Target: \\(s\\).\n\
             The appropriate equation relating \\(u, a, t, s\\) is:\n\
             \\[ s = ut + \\frac{{1}}{{2}}at^2 \\]\n\n\
             **Step 2 (Substitution & Calculation):**\n\
             \\[ s = ({:.0} \\times {:.0}) + \\frac{{1}}{{2}}({:.0})({:.0}^2) \\]\n\
             \\[ s = {:.0} + \\frac{{1}}{{2}}({:.0})({:.0}) = {:.0} + {:.0} = **{:.0} m** \\]",
            u, a, t, u, t, a, t, u * t, a, t * t, u * t, 0.5 * a * t * t, displacement
        );

        let physics_meta = PhysicalProblemMetadata {
            regime: PhysicalRegime::default(),
            coordinate_system: CoordinateSystem::default(),
            candidate_models: vec![PhysicalModelKind::KinematicsConstantAcceleration],
            governing_model: PhysicalModelKind::KinematicsConstantAcceleration,
            known_quantities: vec![
                PhysicalQuantity::known("Initial Velocity", "u", u, PhysicsUnit::MeterPerSecond),
                PhysicalQuantity::known("Acceleration", "a", a, PhysicsUnit::MeterPerSecondSquared),
                PhysicalQuantity::known("Time", "t", t, PhysicsUnit::Second),
            ],
            target_quantity: PhysicalQuantity::unknown("Displacement", "s", PhysicsUnit::Meter),
            governing_equations: vec!["s = u * t + 0.5 * a * t^2".to_string()],
        };

        let parameters = serde_json::json!({
            "variant": "kinematic_equation_selection",
            "u": u,
            "a": a,
            "t": t,
            "displacement": displacement,
            "unit": "m",
            "physics_metadata": physics_meta,
        });

        let correct_answer = serde_json::json!({
            "value": displacement,
            "unit": "m",
            "formatted": format!("{:.0} m", displacement),
            "solution": solution,
        });

        let step1 = StepNode::new(
            "select_equation",
            StepType::SelectEquation,
            "Select Kinematic Equation",
            "s = ut + (1/2)at^2".to_string(),
            "s = ut + 0.5at^2".to_string(),
        )
        .with_hints(vec![
            StepHint::principle("Identify the kinematic equation that directly relates displacement s to known quantities u, a, t."),
            StepHint::operation("Select s = ut + (1/2)at^2."),
            StepHint::intermediate_relation("s = ut + (1/2)at^2"),
        ]);

        let step2 = StepNode::new(
            "calc_distance",
            StepType::FinalAnswer,
            "Evaluate s = ut + 1/2 a t^2",
            format!("{}*{} + 0.5*{}*{}^2 = {} m", u, t, a, t, displacement),
            format!("{:.0}", displacement),
        )
        .with_expected_value(displacement)
        .with_dependencies(vec!["select_equation".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Compute the uniform term (u*t) and acceleration term (1/2 * a * t^2) separately."),
            StepHint::operation(format!("Compute ({:.0} * {:.0}) + 0.5 * {:.0} * {:.0}^2.", u, t, a, t)),
            StepHint::intermediate_relation(format!("s = {:.0} m", displacement)),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2], "calc_distance");

        ProblemInstance::new(
            format!("inst-phys-kin-l3-{}", seed),
            FAMILY_PHYSICS_KINEMATICS,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(serde_json::json!({
            "difficulty_level": 3,
            "target_time_ms": 40_000,
            "domain": "physics",
            "unit": "m",
        }))
    }

    /// Level 4: Reverse Variable / Stopping Distance under Deceleration: v^2 = u^2 + 2as (v = 0)
    fn generate_level_4(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let u: f64 = rng.random_range(5..=60) as f64; // m/s
        let a_mag: f64 = rng.random_range(1..=15) as f64; // magnitude of deceleration m/s^2
        // s = u^2 / (2 * a_mag)
        // Let's choose u such that u^2 is cleanly divisible by 2 * a_mag
        let stopping_dist = (u * u) / (2.0 * a_mag);

        let prompt = format!(
            "A motorist traveling at **{:.0} m/s** sees an obstacle and brakes with a constant deceleration of **{:.0} m/s²** until coming to a complete stop.\n\n\
             What is the minimum stopping distance in meters?",
            u, a_mag
        );

        let solution = format!(
            "**Step 1 (Sign Convention & Knowns):**\n\
             - Initial velocity \\(u = {:.0}\\text{{ m/s}}\\)\n\
             - Final velocity \\(v = 0\\text{{ m/s}}\\) (complete stop)\n\
             - Acceleration \\(a = -{:.0}\\text{{ m/s}}^2\\) (opposes motion)\n\n\
             **Step 2 (Third Kinematic Equation):**\n\
             \\[ v^2 = u^2 + 2as \\implies 0 = ({:.0})^2 + 2(-{:.0})s \\]\n\
             \\[ 0 = {:.0} - {:.0}s \\implies {:.0}s = {:.0} \\]\n\n\
             **Step 3 (Solve for Stopping Distance):**\n\
             \\[ s = \\frac{{{:.0}}}{{{:.0}}} = **{:.1} m** \\]",
            u, a_mag, u, a_mag, u * u, 2.0 * a_mag, 2.0 * a_mag, u * u, u * u, 2.0 * a_mag, stopping_dist
        );

        let physics_meta = PhysicalProblemMetadata {
            regime: PhysicalRegime::default(),
            coordinate_system: CoordinateSystem {
                dimension: "1D".to_string(),
                positive_direction: "direction of forward motion".to_string(),
                origin_reference: "point of braking application x = 0".to_string(),
            },
            candidate_models: vec![PhysicalModelKind::KinematicsConstantAcceleration],
            governing_model: PhysicalModelKind::KinematicsConstantAcceleration,
            known_quantities: vec![
                PhysicalQuantity::known("Initial Velocity", "u", u, PhysicsUnit::MeterPerSecond),
                PhysicalQuantity::known("Final Velocity", "v", 0.0, PhysicsUnit::MeterPerSecond),
                PhysicalQuantity::known("Acceleration", "a", -a_mag, PhysicsUnit::MeterPerSecondSquared),
            ],
            target_quantity: PhysicalQuantity::unknown("Stopping Distance", "s", PhysicsUnit::Meter),
            governing_equations: vec!["v^2 = u^2 + 2 * a * s".to_string(), "s = u^2 / (2 * a)".to_string()],
        };

        let parameters = serde_json::json!({
            "variant": "stopping_distance_reverse",
            "u": u,
            "a_mag": a_mag,
            "stopping_dist": stopping_dist,
            "unit": "m",
            "physics_metadata": physics_meta,
        });

        let correct_answer = serde_json::json!({
            "value": stopping_dist,
            "unit": "m",
            "formatted": format!("{:.1} m", stopping_dist),
            "solution": solution,
        });

        let step1 = StepNode::new(
            "stopping_formula",
            StepType::EquationRearrangement,
            "Rearrange v^2 = u^2 + 2as for v = 0",
            format!("0 = {}^2 - 2*{}*s => s = {} / {}", u, a_mag, u * u, 2.0 * a_mag),
            format!("{:.1}", stopping_dist),
        )
        .with_hints(vec![
            StepHint::principle("Use v^2 = u^2 + 2as with final velocity v = 0 and deceleration a = -a_mag."),
            StepHint::operation("Isolate stopping distance: s = u^2 / (2 * a_mag)."),
            StepHint::intermediate_relation(format!("s = {:.0}^2 / (2 * {:.0})", u, a_mag)),
        ]);

        let step2 = StepNode::new(
            "calc_stopping_distance",
            StepType::FinalAnswer,
            "Compute stopping distance value",
            format!("{:.0} / {:.0} = {:.1} m", u * u, 2.0 * a_mag, stopping_dist),
            format!("{:.1}", stopping_dist),
        )
        .with_expected_value(stopping_dist)
        .with_dependencies(vec!["stopping_formula".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Divide the square of initial speed by twice the braking deceleration."),
            StepHint::operation(format!("Compute {:.0} / {:.0}.", u * u, 2.0 * a_mag)),
            StepHint::intermediate_relation(format!("s = {:.1} m", stopping_dist)),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2], "calc_stopping_distance");

        ProblemInstance::new(
            format!("inst-phys-kin-l4-{}", seed),
            FAMILY_PHYSICS_KINEMATICS,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(serde_json::json!({
            "difficulty_level": 4,
            "target_time_ms": 45_000,
            "domain": "physics",
            "unit": "m",
        }))
    }

    /// Level 5: Vertical Projectile / Free Fall Transfer under Gravity (a = -g)
    fn generate_level_5(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let g = 9.8; // m/s^2
        let u: f64 = rng.random_range(5..=100) as f64;
        let max_height = (u * u) / (2.0 * g); // H = u^2 / (2g)
        let time_to_apex = u / g; // t = u / g

        let prompt = format!(
            "A research projectile is fired vertically upward from ground level with an initial launch speed of **{:.0} m/s**.\n\n\
             Taking acceleration due to gravity as \\(g = 9.8\\text{{ m/s}}^2\\) downwards and neglecting air resistance, what is the **maximum height** reached by the projectile in meters?",
            u
        );

        let solution = format!(
            "**Step 1 (Physical Model & Coordinate System):**\n\
             Model: **Vertical Motion Under Gravity** (Free Fall with \\(a = -g = -9.8\\text{{ m/s}}^2\\)).\n\
             Coordinate system: Upward is positive (\\(+y\\)).\n\n\
             **Step 2 (Apex Conditions & Governing Equation):**\n\
             At the maximum height apex, instantaneous velocity \\(v = 0\\).\n\
             \\[ v^2 = u^2 - 2gH \\implies 0 = ({:.0})^2 - 2(9.8)H \\]\n\n\
             **Step 3 (Solve for Maximum Height \\(H\\)):**\n\
             \\[ H = \\frac{{u^2}}{{2g}} = \\frac{{{:.0}}}{{19.6}} = **{:.2} m** \\]",
            u, u * u, max_height
        );

        let physics_meta = PhysicalProblemMetadata {
            regime: PhysicalRegime {
                regime_name: "1D Free Fall Under Gravity".to_string(),
                gravity_acceleration: 9.8,
                friction_present: false,
                air_resistance_neglected: true,
            },
            coordinate_system: CoordinateSystem {
                dimension: "1D Vertical".to_string(),
                positive_direction: "upward (+y)".to_string(),
                origin_reference: "ground level y = 0".to_string(),
            },
            candidate_models: vec![
                PhysicalModelKind::KinematicsVerticalFreeFall,
                PhysicalModelKind::MechanicalEnergyConservation,
            ],
            governing_model: PhysicalModelKind::KinematicsVerticalFreeFall,
            known_quantities: vec![
                PhysicalQuantity::known("Launch Speed", "u", u, PhysicsUnit::MeterPerSecond),
                PhysicalQuantity::known("Gravity", "g", 9.8, PhysicsUnit::MeterPerSecondSquared),
                PhysicalQuantity::known("Velocity at Apex", "v", 0.0, PhysicsUnit::MeterPerSecond),
            ],
            target_quantity: PhysicalQuantity::unknown("Maximum Height", "H", PhysicsUnit::Meter),
            governing_equations: vec!["H = u^2 / (2 * g)".to_string(), "v^2 = u^2 - 2 * g * H".to_string()],
        };

        let parameters = serde_json::json!({
            "variant": "vertical_projectile_transfer",
            "u": u,
            "g": g,
            "max_height": max_height,
            "time_to_apex": time_to_apex,
            "unit": "m",
            "physics_metadata": physics_meta,
        });

        let correct_answer = serde_json::json!({
            "value": max_height,
            "unit": "m",
            "formatted": format!("{:.2} m", max_height),
            "solution": solution,
        });

        let step1 = StepNode::new(
            "model_apex_condition",
            StepType::SelectModel,
            "Identify Apex Condition v = 0 under gravity a = -g",
            "v = 0 at apex => H = u^2 / (2g)".to_string(),
            "kinematics_vertical_free_fall".to_string(),
        )
        .with_alternates(vec!["kinematics_vertical_free_fall".to_string(), "free_fall".to_string(), "H = u^2/(2g)".to_string()])
        .with_hints(vec![
            StepHint::principle("At maximum vertical height, instantaneous vertical velocity is zero (v = 0)."),
            StepHint::operation("Apply kinematic relation v^2 = u^2 - 2gH with v = 0."),
            StepHint::intermediate_relation("H = u^2 / (2g)"),
        ]);

        let step2 = StepNode::new(
            "calc_max_height",
            StepType::FinalAnswer,
            "Compute Maximum Height H = u^2 / (2 * 9.8)",
            format!("{}^2 / (2 * 9.8) = {:.2} m", u, max_height),
            format!("{:.2}", max_height),
        )
        .with_expected_value(max_height)
        .with_dependencies(vec!["model_apex_condition".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Square launch velocity and divide by 2 * 9.8 = 19.6."),
            StepHint::operation(format!("Compute {:.0}^2 / 19.6.", u)),
            StepHint::intermediate_relation(format!("H = {:.2} m", max_height)),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2], "calc_max_height");

        ProblemInstance::new(
            format!("inst-phys-kin-l5-{}", seed),
            FAMILY_PHYSICS_KINEMATICS,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(serde_json::json!({
            "difficulty_level": 5,
            "target_time_ms": 50_000,
            "domain": "physics",
            "unit": "m",
            "learning_object_level": "transfer",
        }))
    }
}

impl ProblemGenerator for Kinematics1DGenerator {
    fn family_id(&self) -> &str {
        FAMILY_PHYSICS_KINEMATICS
    }

    fn template_ref(&self) -> &str {
        TEMPLATE_PHYSICS_KINEMATICS_V1
    }

    fn supported_variants(&self) -> Vec<String> {
        vec![
            "uniform_motion".to_string(),
            "constant_acceleration_unit_conversion".to_string(),
            "kinematic_equation_selection".to_string(),
            "stopping_distance_reverse".to_string(),
            "vertical_projectile_transfer".to_string(),
        ]
    }

    fn target_latency_ms(&self, difficulty_level: u32) -> u64 {
        match difficulty_level {
            1 => 25_000,
            2 => 35_000,
            3 => 40_000,
            4 => 45_000,
            _ => 50_000,
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

pub struct Kinematics1DValidator;

impl ProblemValidator for Kinematics1DValidator {
    fn family_id(&self) -> &str {
        FAMILY_PHYSICS_KINEMATICS
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

        // Physical sanity check: distance / speed must be non-negative
        if let Err(sanity_err) = PhysicalSanityValidator::check_time(num) {
            if instance.rendered_prompt.contains("seconds") || instance.rendered_prompt.contains("time") {
                return AnswerEvaluation::incorrect(
                    ErrorCategory::Concept,
                    format!("Physical Sanity Violation: {}", sanity_err),
                )
                .with_parsed_values(num, expected_val);
            }
        }

        // Check if student applied uniform motion when acceleration was present (ModelSelectionError)
        let variant = instance.parameters.get("variant").and_then(|v| v.as_str()).unwrap_or("");
        if variant == "constant_acceleration_unit_conversion" {
            let u_mps = instance.parameters.get("u_mps").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let u_kmh = instance.parameters.get("u_kmh").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let a = instance.parameters.get("a").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let t = instance.parameters.get("t").and_then(|v| v.as_f64()).unwrap_or(0.0);

            // Misconception 1: Forgot unit conversion (added km/h directly to a*t)
            let unscaled_val = u_kmh + a * t;
            if (num - unscaled_val).abs() < 1e-4 {
                return AnswerEvaluation::incorrect(
                    ErrorCategory::Unit,
                    "Unit Incompatibility Error: Initial velocity was given in km/h but acceleration is in m/s². You must multiply km/h by 5/18 before computing v = u + at.",
                )
                .with_parsed_values(num, expected_val);
            }

            // Misconception 2: Neglected acceleration entirely (v = u)
            if (num - u_mps).abs() < 1e-4 || (num - u_kmh).abs() < 1e-4 {
                return AnswerEvaluation::incorrect(
                    ErrorCategory::Strategy,
                    "Model Selection Error: The vehicle accelerates under constant acceleration a != 0. You cannot apply uniform velocity v = u.",
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
    fn test_kinematics_generation_all_levels() {
        for level in 1..=5 {
            let inst = Kinematics1DGenerator::generate_problem(12345 + level as u64, level, None);
            assert_eq!(inst.family_id.as_str(), FAMILY_PHYSICS_KINEMATICS);
            assert!(!inst.rendered_prompt.is_empty());
            assert!(inst.correct_answer.get("value").is_some());
            assert!(inst.solution_graph().is_some());
        }
    }

    #[test]
    fn test_kinematics_unit_conversion_diagnostic() {
        let generator = Kinematics1DGenerator;
        let validator = Kinematics1DValidator;
        let inst = generator.generate(&ProblemFamilyId::new(FAMILY_PHYSICS_KINEMATICS), 42, 2, None).unwrap();

        let u_kmh = inst.parameters.get("u_kmh").unwrap().as_f64().unwrap();
        let a = inst.parameters.get("a").unwrap().as_f64().unwrap();
        let t = inst.parameters.get("t").unwrap().as_f64().unwrap();

        // Submit unscaled unit error: u_kmh + a * t
        let bad_sub = serde_json::json!(u_kmh + a * t);
        let eval = validator.evaluate(&inst, &bad_sub, 15000, 35000);

        assert!(!eval.is_correct);
        assert_eq!(eval.error_category, Some(ErrorCategory::Unit));
        assert!(eval.diagnostic_message.unwrap().contains("Unit Incompatibility"));
    }

    #[test]
    fn test_kinematics_model_selection_diagnostic() {
        let generator = Kinematics1DGenerator;
        let validator = Kinematics1DValidator;
        let inst = generator.generate(&ProblemFamilyId::new(FAMILY_PHYSICS_KINEMATICS), 42, 2, None).unwrap();

        let u_mps = inst.parameters.get("u_mps").unwrap().as_f64().unwrap();
        // Submit neglected acceleration: v = u
        let bad_sub = serde_json::json!(u_mps);
        let eval = validator.evaluate(&inst, &bad_sub, 15000, 35000);

        assert!(!eval.is_correct);
        assert_eq!(eval.error_category, Some(ErrorCategory::Strategy));
        assert!(eval.diagnostic_message.unwrap().contains("Model Selection Error"));
    }
}
