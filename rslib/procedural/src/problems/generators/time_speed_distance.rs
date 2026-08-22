// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

use crate::core::{ProblemFamilyId, Result};
use crate::diagnostics::ErrorCategory;
use crate::problems::generator::ProblemGenerator;
use crate::problems::steps::{
    DiagnosticConfidence, SolutionGraph, StepGraphEvaluation, StepHint, StepNode, StepType,
    StepValidator, StepwiseSubmission,
};
use crate::problems::validator::{AnswerEvaluation, NumericAnswerParser, ProblemValidator};
use crate::problems::ProblemInstance;

pub const FAMILY_TIME_SPEED_DISTANCE: &str = "family.math.arithmetic.time_speed_distance";
pub const TEMPLATE_TIME_SPEED_DISTANCE_V1: &str = "math.arithmetic.time_speed_distance.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeSpeedDistanceVariant {
    DirectFormula,
    UnitConversion,
    AverageSpeed,
    RelativeSpeed,
    TransferTravel,
}

impl TimeSpeedDistanceVariant {
    pub fn as_str(&self) -> &'static str {
        match self {
            TimeSpeedDistanceVariant::DirectFormula => "direct_formula",
            TimeSpeedDistanceVariant::UnitConversion => "unit_conversion",
            TimeSpeedDistanceVariant::AverageSpeed => "average_speed",
            TimeSpeedDistanceVariant::RelativeSpeed => "relative_speed",
            TimeSpeedDistanceVariant::TransferTravel => "transfer_travel",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TimeSpeedDistanceGenerator;

impl TimeSpeedDistanceGenerator {
    pub fn generate_problem(seed: u64, difficulty_level: u32, variant: Option<&str>) -> ProblemInstance {
        let mut rng = StdRng::seed_from_u64(seed);
        let chosen_variant = if let Some(v) = variant {
            match v {
                "direct_formula" => TimeSpeedDistanceVariant::DirectFormula,
                "unit_conversion" => TimeSpeedDistanceVariant::UnitConversion,
                "average_speed" => TimeSpeedDistanceVariant::AverageSpeed,
                "relative_speed" => TimeSpeedDistanceVariant::RelativeSpeed,
                "transfer_travel" => TimeSpeedDistanceVariant::TransferTravel,
                _ => TimeSpeedDistanceVariant::DirectFormula,
            }
        } else {
            match difficulty_level {
                1 => TimeSpeedDistanceVariant::DirectFormula,
                2 => TimeSpeedDistanceVariant::UnitConversion,
                3 => TimeSpeedDistanceVariant::AverageSpeed,
                4 => TimeSpeedDistanceVariant::RelativeSpeed,
                _ => TimeSpeedDistanceVariant::TransferTravel,
            }
        };

        match chosen_variant {
            TimeSpeedDistanceVariant::DirectFormula => Self::generate_level_1(&mut rng, seed),
            TimeSpeedDistanceVariant::UnitConversion => Self::generate_level_2(&mut rng, seed),
            TimeSpeedDistanceVariant::AverageSpeed => Self::generate_level_3(&mut rng, seed),
            TimeSpeedDistanceVariant::RelativeSpeed => Self::generate_level_4(&mut rng, seed),
            TimeSpeedDistanceVariant::TransferTravel => Self::generate_level_5(&mut rng, seed),
        }
    }

    /// Level 1: Direct D = S * T with clean integers and randomized missing variable
    fn generate_level_1(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let mode = rng.random_range(0..3);
        let speed = rng.random_range(20..=120); 
        let time = rng.random_range(2..=10);
        let distance = speed * time;

        let prompt;
        let solution;
        let expected_val: f64;
        let unit: &str;
        
        if mode == 0 {
            prompt = format!("A car travels at a constant speed of **{} km/h** for **{} hours**.\n\nFind the total distance covered in kilometers.", speed, time);
            solution = format!("**Step 1:** \\text{{Distance}} = \\text{{Speed}} \\times \\text{{Time}}\n\\[ \\text{{Distance}} = {} \\times {} = **{}** \\text{{ km}} \\]", speed, time, distance);
            expected_val = distance as f64;
            unit = "km";
        } else if mode == 1 {
            prompt = format!("A car covers a distance of **{} km** traveling at a constant speed of **{} km/h**.\n\nHow many hours does the journey take?", distance, speed);
            solution = format!("**Step 1:** \\text{{Time}} = \\frac{{\\text{{Distance}}}}{{\\text{{Speed}}}}\n\\[ \\text{{Time}} = \\frac{{{}}}{{{}}} = **{}** \\text{{ hours}} \\]", distance, speed, time);
            expected_val = time as f64;
            unit = "hours";
        } else {
            prompt = format!("A car covers a distance of **{} km** in **{} hours** at a constant speed.\n\nWhat is the speed of the car in km/h?", distance, time);
            solution = format!("**Step 1:** \\text{{Speed}} = \\frac{{\\text{{Distance}}}}{{\\text{{Time}}}}\n\\[ \\text{{Speed}} = \\frac{{{}}}{{{}}} = **{}** \\text{{ km/h}} \\]", distance, time, speed);
            expected_val = speed as f64;
            unit = "km/h";
        }

        let parameters = serde_json::json!({
            "variant": "direct_formula",
            "mode": mode,
            "speed": speed,
            "time": time,
            "distance": distance,
        });

        let correct_answer = serde_json::json!({
            "value": expected_val,
            "formatted": format!("{}", expected_val),
            "unit": unit,
            "solution": solution,
        });

        let step1 = StepNode::new(
            "calc_target",
            StepType::FinalAnswer,
            "Calculate missing value",
            "Apply D = S * T or its variations",
            format!("Result = {}", expected_val),
        )
        .with_expected_value(expected_val)
        .with_alternates(vec![format!("{}", expected_val)])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Use the relationship: Distance = Speed * Time."),
        ]);

        let graph = SolutionGraph::new(vec![step1], "calc_target");

        ProblemInstance::new(
            format!("inst-tsd-l1-{}", seed),
            FAMILY_TIME_SPEED_DISTANCE,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(serde_json::json!({
            "target_time_ms": 25_000,
            "difficulty_level": 1,
            "variant": "direct_formula",
            "learning_object_level": "procedural_execution",
        }))
    }

    /// Level 2: Unit conversion (km/h <-> m/s) and sub-minute calculation
    fn generate_level_2(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let mode = rng.random_range(0..2);
        
        let prompt;
        let solution;
        let expected_val: f64;
        let unit: &str;
        
        if mode == 0 {
            // km/h to m/s
            let speed_kmh_multiplier = rng.random_range(2..=12);
            let speed_kmh = speed_kmh_multiplier * 18; 
            let speed_ms = speed_kmh_multiplier * 5;   
            let time_sec = rng.random_range(10..=60);  
            let distance_m = speed_ms * time_sec;

            prompt = format!("A train travels at **{} km/h**.\n\nHow many meters does it cover in **{} seconds**?", speed_kmh, time_sec);
            solution = format!("**Step 1:** Convert speed to m/s:\n\\[ {} \\times \\frac{{5}}{{18}} = {} \\text{{ m/s}} \\]\n\n**Step 2:** Distance = Speed \\times Time\n\\[ {} \\times {} = **{}** \\text{{ meters}} \\]", speed_kmh, speed_ms, speed_ms, time_sec, distance_m);
            expected_val = distance_m as f64;
            unit = "meters";
        } else {
            // m/s to km/h
            let speed_ms_multiplier = rng.random_range(2..=8);
            let speed_ms = speed_ms_multiplier * 5;
            let speed_kmh = speed_ms_multiplier * 18;
            let time_hours = rng.random_range(2..=8);
            let distance_km = speed_kmh * time_hours;
            
            prompt = format!("A bird flies at a speed of **{} m/s**.\n\nHow many kilometers does it cover in **{} hours**?", speed_ms, time_hours);
            solution = format!("**Step 1:** Convert speed to km/h:\n\\[ {} \\times \\frac{{18}}{{5}} = {} \\text{{ km/h}} \\]\n\n**Step 2:** Distance = Speed \\times Time\n\\[ {} \\times {} = **{}** \\text{{ kilometers}} \\]", speed_ms, speed_kmh, speed_kmh, time_hours, distance_km);
            expected_val = distance_km as f64;
            unit = "km";
        }

        let parameters = serde_json::json!({
            "variant": "unit_conversion",
            "mode": mode,
        });

        let correct_answer = serde_json::json!({
            "value": expected_val,
            "formatted": format!("{}", expected_val),
            "unit": unit,
            "solution": solution,
        });

        let step1 = StepNode::new(
            "calc_distance",
            StepType::FinalAnswer,
            "Calculate distance",
            "Convert units and multiply by time",
            format!("Result = {}", expected_val),
        )
        .with_expected_value(expected_val)
        .with_alternates(vec![format!("{}", expected_val)])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Remember the conversion factors: 18 km/h = 5 m/s."),
        ]);

        let graph = SolutionGraph::new(vec![step1], "calc_distance");

        ProblemInstance::new(
            format!("inst-tsd-l2-{}", seed),
            FAMILY_TIME_SPEED_DISTANCE,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(serde_json::json!({
            "target_time_ms": 35_000,
            "difficulty_level": 2,
            "variant": "unit_conversion",
            "learning_object_level": "procedural_execution",
        }))
    }

    /// Level 3: Average speed (Harmonic mean for equal distances)
    fn generate_level_3(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let s1 = rng.random_range(20..=120) as f64;
        let s2 = rng.random_range(20..=120) as f64;
        let avg_speed = (2.0 * s1 * s2) / (s1 + s2);
        let avg_speed_rounded = (avg_speed * 10.0).round() / 10.0;

        let prompt = format!(
            "A motorist drives from Town A to Town B at **{} km/h** and returns along the exact same route at **{} km/h**.\n\nCalculate the average speed for the entire round trip in km/h (round to 1 decimal place).",
            s1, s2
        );

        let solution = format!(
            "**Step 1:** Because distance in both directions is equal, average speed is given by the harmonic formula:\n\
             \\[ \\text{{Average Speed}} = \\frac{{2 \\cdot S_1 \\cdot S_2}}{{S_1 + S_2}} \\]\n\n\
             **Step 2:** Compute:\n\
             \\[ \\text{{Average Speed}} = \\frac{{2 \\cdot {} \\cdot {}}}{{{} + {}}} = **{}** \\text{{ km/h}} \\]",
            s1, s2, s1, s2, avg_speed_rounded
        );

        let parameters = serde_json::json!({
            "variant": "average_speed",
            "speed1": s1,
            "speed2": s2,
            "average_speed": avg_speed_rounded,
        });

        let correct_answer = serde_json::json!({
            "value": avg_speed_rounded,
            "formatted": format!("{}", avg_speed_rounded),
            "unit": "km/h",
            "solution": solution,
        });

        let step1 = StepNode::new(
            "calc_avg_speed",
            StepType::FinalAnswer,
            "Compute average speed",
            "Apply harmonic mean formula",
            format!("{}", avg_speed_rounded),
        )
        .with_expected_value(avg_speed_rounded)
        .with_alternates(vec![format!("{}", avg_speed_rounded)])
        .as_final()
        .with_hints(vec![
            StepHint::principle("For equal distances, the average speed is the harmonic mean: (2 * S1 * S2) / (S1 + S2). Avoid the simple arithmetic mean (S1+S2)/2!"),
        ]);

        let graph = SolutionGraph::new(vec![step1], "calc_avg_speed");

        ProblemInstance::new(
            format!("inst-tsd-l3-{}", seed),
            FAMILY_TIME_SPEED_DISTANCE,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(serde_json::json!({
            "target_time_ms": 40_000,
            "difficulty_level": 3,
            "variant": "average_speed",
            "learning_object_level": "variation",
        }))
    }

    /// Level 4: Relative speed & Trains crossing
    fn generate_level_4(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let same_direction = rng.random_bool(0.5);
        let s1_kmh = rng.random_range(60..=120);
        let s2_kmh = if same_direction {
            // make sure they have a clean difference in m/s
            s1_kmh - rng.random_range(1..=3) * 18
        } else {
            // make sure they have a clean sum in m/s
            180 - s1_kmh + rng.random_range(0..=2) * 18
        };
        
        let rel_speed_kmh = if same_direction { s1_kmh - s2_kmh } else { s1_kmh + s2_kmh };
        let rel_speed_ms = rel_speed_kmh * 5 / 18;

        let len1_m = rng.random_range(100..=250);
        let len2_m = rng.random_range(100..=250);
        let total_dist_m = len1_m + len2_m;
        
        let rem = total_dist_m % rel_speed_ms;
        let final_len2 = if rem != 0 { len2_m + (rel_speed_ms - rem) } else { len2_m };
        let final_total_dist = len1_m + final_len2;
        let crossing_time_sec = final_total_dist / rel_speed_ms;

        let dir_str = if same_direction { "in the same direction" } else { "towards each other in opposite directions" };
        let prompt = format!(
            "Two trains of lengths **{} meters** and **{} meters** are traveling {} on parallel tracks at **{} km/h** and **{} km/h** respectively.\n\nIn how many seconds will they completely cross each other?",
            len1_m, final_len2, dir_str, s1_kmh, s2_kmh
        );

        let solution = format!(
            "**Step 1:** Total distance to cover = Sum of train lengths:\n\
             \\[ \\text{{Total Distance}} = {} + {} = {} \\text{{ m}} \\]\n\n\
             **Step 2:** Relative speed in m/s:\n\
             \\[ \\text{{Relative Speed}} = {} \\times \\frac{{5}}{{18}} = {} \\text{{ m/s}} \\]\n\n\
             **Step 3:** Time to cross completely:\n\
             \\[ \\text{{Time}} = \\frac{{{}}}{{{}}} = **{}** \\text{{ seconds}} \\]",
            len1_m, final_len2, final_total_dist, rel_speed_kmh, rel_speed_ms, final_total_dist, rel_speed_ms, crossing_time_sec
        );

        let parameters = serde_json::json!({
            "variant": "relative_speed",
            "len1": len1_m,
            "len2": final_len2,
            "speed1_kmh": s1_kmh,
            "speed2_kmh": s2_kmh,
            "same_direction": same_direction,
            "rel_speed_ms": rel_speed_ms,
            "crossing_time_sec": crossing_time_sec,
        });

        let correct_answer = serde_json::json!({
            "value": crossing_time_sec as f64,
            "formatted": format!("{}", crossing_time_sec),
            "unit": "seconds",
            "solution": solution,
        });

        let step1 = StepNode::new(
            "calc_time",
            StepType::FinalAnswer,
            "Calculate crossing time",
            "Divide total distance by relative speed",
            format!("{}", crossing_time_sec),
        )
        .with_expected_value(crossing_time_sec as f64)
        .with_alternates(vec![format!("{}", crossing_time_sec)])
        .as_final()
        .with_hints(vec![
            StepHint::principle(if same_direction { "For same direction, subtract speeds." } else { "For opposite directions, add speeds." }),
        ]);

        let graph = SolutionGraph::new(vec![step1], "calc_time");

        ProblemInstance::new(
            format!("inst-tsd-l4-{}", seed),
            FAMILY_TIME_SPEED_DISTANCE,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(serde_json::json!({
            "target_time_ms": 50_000,
            "difficulty_level": 4,
            "variant": "relative_speed",
            "learning_object_level": "variation",
        }))
    }

    /// Level 5: Transfer word problem (Early/Late arrival time difference equation)
    fn generate_level_5(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let s1 = rng.random_range(10..=30);
        let s2 = s1 + rng.random_range(4..=10);
        let late_mins = rng.random_range(5..=15);
        let early_mins = rng.random_range(5..=15);
        let total_diff_mins = late_mins + early_mins;
        let delta_t_hours = total_diff_mins as f64 / 60.0;
        let d_val = (s1 as f64 * s2 as f64 * delta_t_hours) / (s2 - s1) as f64;
        let d_rounded = (d_val * 10.0).round() / 10.0;

        let prompt = format!(
            "If a student cycles from home to school at **{} km/h**, she arrives **{} minutes late**.\n\
             If she increases her speed to **{} km/h**, she arrives **{} minutes early**.\n\n\
             Find the distance from her home to the school in kilometers.",
            s1, late_mins, s2, early_mins
        );

        let solution = format!(
            "**Step 1:** Calculate total time difference between the two trips in hours:\n\
             \\[ \\Delta T = {} \\text{{ min (late)}} + {} \\text{{ min (early)}} = {} \\text{{ min}} = \\frac{{{}}}{{60}} \\text{{ hours}} \\]\n\n\
             **Step 2:** Formulate the travel time equation:\n\
             \\[ \\frac{{D}}{{{}}} - \\frac{{D}}{{{}}} = \\frac{{{}}}{{60}} \\]\n\n\
             **Step 3:** Solve for distance \\(D\\):\n\
             \\[ D \\cdot \\left(\\frac{{{}}}{{{} \\cdot {}}}\\right) = \\frac{{{}}}{{60}} \\]\n\
             \\[ D = \\frac{{{} \\cdot {} \\cdot {}}}{{{} \\cdot 60}} = **{:.1}** \\text{{ km}} \\]",
            late_mins, early_mins, total_diff_mins, total_diff_mins, s1, s2, total_diff_mins,
            s2 - s1, s1, s2, total_diff_mins, s1, s2, total_diff_mins, s2 - s1, d_rounded
        );

        let parameters = serde_json::json!({
            "variant": "transfer_travel",
            "speed1": s1,
            "speed2": s2,
            "late_mins": late_mins,
            "early_mins": early_mins,
            "distance": d_rounded,
        });

        let correct_answer = serde_json::json!({
            "value": d_rounded,
            "formatted": format!("{:.1}", d_rounded),
            "unit": "km",
            "solution": solution,
        });

        let step1 = StepNode::new(
            "time_diff",
            StepType::Transformation,
            "Compute time difference in hours",
            format!("({} + {}) / 60 = {:.2} hours", late_mins, early_mins, delta_t_hours),
            format!("{}/60", total_diff_mins),
        )
        .with_expected_value(delta_t_hours)
        .with_hints(vec![
            StepHint::principle("The total time difference between being late and early is the sum of both offsets divided by 60."),
            StepHint::operation(format!("Add {} + {} = {} minutes, then divide by 60.", late_mins, early_mins, total_diff_mins)),
            StepHint::intermediate_relation(format!("Time difference = {}/60 hours", total_diff_mins)),
        ]);

        let step2 = StepNode::new(
            "solve_distance",
            StepType::FinalAnswer,
            "Solve for distance D",
            format!("D = ({} * {} * {}) / (({} - {}) * 60)", s1, s2, total_diff_mins, s2, s1),
            format!("{:.1}", d_rounded),
        )
        .with_expected_value(d_rounded)
        .with_dependencies(vec!["time_diff".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Use the distance formula for speed change: D = (S1 * S2 * ΔT) / (S2 - S1)."),
            StepHint::operation(format!("Calculate ({} * {} * {}) / ({} * 60).", s1, s2, total_diff_mins, s2 - s1)),
            StepHint::intermediate_relation(format!("Distance D = {:.1} km", d_rounded)),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2], "solve_distance");

        ProblemInstance::new(
            format!("inst-tsd-l5-{}", seed),
            FAMILY_TIME_SPEED_DISTANCE,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(serde_json::json!({
            "target_time_ms": 60_000,
            "difficulty_level": 5,
            "variant": "transfer_travel",
            "learning_object_level": "transfer",
        }))
    }
}

impl ProblemGenerator for TimeSpeedDistanceGenerator {
    fn family_id(&self) -> &str {
        FAMILY_TIME_SPEED_DISTANCE
    }

    fn template_ref(&self) -> &str {
        TEMPLATE_TIME_SPEED_DISTANCE_V1
    }

    fn supported_variants(&self) -> Vec<String> {
        vec![
            "direct_formula".to_string(),
            "unit_conversion".to_string(),
            "average_speed".to_string(),
            "relative_speed".to_string(),
            "transfer_travel".to_string(),
        ]
    }

    fn target_latency_ms(&self, difficulty_level: u32) -> u64 {
        match difficulty_level {
            1 => 25_000,
            2 => 35_000,
            3 => 40_000,
            4 => 50_000,
            _ => 60_000,
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

#[derive(Debug, Clone, Default)]
pub struct TimeSpeedDistanceValidator;

impl ProblemValidator for TimeSpeedDistanceValidator {
    fn family_id(&self) -> &str {
        FAMILY_TIME_SPEED_DISTANCE
    }

    fn evaluate(
        &self,
        instance: &ProblemInstance,
        student_answer: &serde_json::Value,
        time_taken_ms: u64,
        target_time_ms: u64,
    ) -> AnswerEvaluation {
        let expected_val = instance
            .correct_answer
            .get("value")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let parsed_val = NumericAnswerParser::parse_student_answer(student_answer);

        if let Some(student_num) = parsed_val {
            let diff = (student_num - expected_val).abs();
            let is_correct = diff <= 0.1;

            if is_correct {
                let score = if target_time_ms > 0 && time_taken_ms > target_time_ms {
                    0.85
                } else {
                    1.0
                };
                AnswerEvaluation::correct(score, time_taken_ms, target_time_ms)
                    .with_parsed_values(student_num, expected_val)
                    .with_diagnostic("✓ Correct distance / speed calculation.")
            } else {
                let s1 = instance.parameters.get("speed1").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let s2 = instance.parameters.get("speed2").and_then(|v| v.as_f64()).unwrap_or(0.0);

                // Check for arithmetic mean trap in average speed: (S1 + S2)/2
                if s1 > 0.0 && s2 > 0.0 && (student_num - (s1 + s2) / 2.0).abs() <= 0.1 {
                    return AnswerEvaluation::incorrect(
                        ErrorCategory::Concept,
                        "Misconception trap: Calculated arithmetic mean (S1+S2)/2 instead of harmonic average speed (2*S1*S2)/(S1+S2).",
                    )
                    .with_parsed_values(student_num, expected_val);
                }

                // Check for unit conversion factor omission (e.g. 18/5 instead of 5/18 or missing conversion)
                if (student_num - expected_val * 3.6).abs() <= 1.0 || (student_num - expected_val / 3.6).abs() <= 1.0 {
                    return AnswerEvaluation::incorrect(
                        ErrorCategory::Calculation,
                        "Unit conversion error: Inverted or missing km/h to m/s conversion factor (5/18).",
                    )
                    .with_parsed_values(student_num, expected_val);
                }

                AnswerEvaluation::incorrect(
                    ErrorCategory::Calculation,
                    format!("Calculation error: Expected {:.1}, but received {:.1}.", expected_val, student_num),
                )
                .with_parsed_values(student_num, expected_val)
            }
        } else {
            AnswerEvaluation::incorrect(
                ErrorCategory::Careless,
                "Unable to parse numerical answer. Please submit a valid number.",
            )
        }
    }

    fn evaluate_stepwise(
        &self,
        instance: &ProblemInstance,
        submission: &StepwiseSubmission,
        target_time_ms: u64,
    ) -> StepGraphEvaluation {
        if let Some(graph) = instance.solution_graph() {
            StepValidator::evaluate_submission(&graph, submission, target_time_ms)
        } else {
            StepGraphEvaluation {
                is_correct: false,
                score: 0.0,
                first_error_step: None,
                first_error_type: None,
                confidence: DiagnosticConfidence::Uncertain,
                steps_completed: submission.steps.len(),
                steps_correct: 0,
                step_evaluations: Vec::new(),
                overall_feedback: "Solution graph missing for stepwise evaluation.".to_string(),
                remediation_recommendation: None,
                first_action_latency_ms: submission.first_action_latency_ms,
                step_latencies_ms: submission.steps.iter().map(|s| s.time_taken_ms).collect(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_speed_distance_generation_all_levels() {
        let gen = TimeSpeedDistanceGenerator;
        let validator = TimeSpeedDistanceValidator;

        for level in 1..=5 {
            let inst = gen.generate(&ProblemFamilyId::new(FAMILY_TIME_SPEED_DISTANCE), 42 + level as u64, level, None).unwrap();
            assert!(!inst.rendered_prompt.is_empty(), "Prompt non-empty for L{}", level);

            let graph = inst.solution_graph();
            assert!(graph.is_some(), "SolutionGraph exists for L{}", level);
            assert!(graph.unwrap().validate_topology(), "SolutionGraph topological acyclicity for L{}", level);

            // Self-validation
            let correct_ans = inst.correct_answer.get("value").unwrap();
            let eval = validator.evaluate(&inst, correct_ans, 15000, 30000);
            assert!(eval.is_correct, "Self-eval succeeds for L{}", level);
        }
    }

    #[test]
    fn test_time_speed_distance_average_speed_trap_diagnostic() {
        let gen = TimeSpeedDistanceGenerator;
        let validator = TimeSpeedDistanceValidator;

        let inst = gen.generate(&ProblemFamilyId::new(FAMILY_TIME_SPEED_DISTANCE), 100, 3, Some("average_speed")).unwrap();
        let s1 = inst.parameters.get("speed1").unwrap().as_f64().unwrap();
        let s2 = inst.parameters.get("speed2").unwrap().as_f64().unwrap();

        // Submit arithmetic mean (s1+s2)/2
        let wrong_mean = (s1 + s2) / 2.0;
        let eval = validator.evaluate(&inst, &serde_json::json!(wrong_mean), 20000, 40000);
        assert!(!eval.is_correct);
        assert_eq!(eval.error_category, Some(ErrorCategory::Concept));
        assert!(eval.diagnostic_message.unwrap().contains("harmonic average speed"));
    }
}
