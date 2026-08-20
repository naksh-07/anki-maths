// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

use crate::core::{ProblemFamilyId, Result};
use crate::diagnostics::ErrorCategory;
use crate::problems::generator::ProblemGenerator;
use crate::problems::steps::{SolutionGraph, StepNode, StepType};
use crate::problems::validator::{AnswerEvaluation, NumericAnswerParser, ProblemValidator};
use crate::problems::ProblemInstance;

pub const FAMILY_RATIO: &str = "family.math.arithmetic.ratio";
pub const TEMPLATE_RATIO_V1: &str = "math.arithmetic.ratio.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RatioVariant {
    DivideAmount,
    MissingProportion,
    ThreePartRatio,
    RatioShift,
    MixtureProportion,
}

impl RatioVariant {
    pub fn as_str(&self) -> &'static str {
        match self {
            RatioVariant::DivideAmount => "divide_amount",
            RatioVariant::MissingProportion => "missing_proportion",
            RatioVariant::ThreePartRatio => "three_part_ratio",
            RatioVariant::RatioShift => "ratio_shift",
            RatioVariant::MixtureProportion => "mixture_proportion",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct RatioGenerator;

impl RatioGenerator {
    pub fn generate_problem(seed: u64, difficulty_level: u32, variant: Option<&str>) -> ProblemInstance {
        let mut rng = StdRng::seed_from_u64(seed);
        let chosen_variant = if let Some(v) = variant {
            match v {
                "divide_amount" => RatioVariant::DivideAmount,
                "missing_proportion" => RatioVariant::MissingProportion,
                "three_part_ratio" => RatioVariant::ThreePartRatio,
                "ratio_shift" => RatioVariant::RatioShift,
                "mixture_proportion" => RatioVariant::MixtureProportion,
                _ => RatioVariant::DivideAmount,
            }
        } else {
            match difficulty_level {
                1 => RatioVariant::DivideAmount,
                2 => RatioVariant::MissingProportion,
                3 => RatioVariant::ThreePartRatio,
                4 => RatioVariant::RatioShift,
                _ => RatioVariant::MixtureProportion,
            }
        };

        match chosen_variant {
            RatioVariant::DivideAmount => Self::generate_level_1(&mut rng, seed),
            RatioVariant::MissingProportion => Self::generate_level_2(&mut rng, seed),
            RatioVariant::ThreePartRatio => Self::generate_level_3(&mut rng, seed),
            RatioVariant::RatioShift => Self::generate_level_4(&mut rng, seed),
            RatioVariant::MixtureProportion => Self::generate_level_5(&mut rng, seed),
        }
    }

    /// Level 1: Divide Total into A:B with dynamic broad ranges
    fn generate_level_1(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let a = rng.random_range(2..=15);
        let b = rng.random_range(2..=18);
        let sum_parts = a + b;
        let mult = rng.random_range(10..=200) * 10;
        let total = (sum_parts * mult) as f64;
        let share_a = (a * mult) as f64;
        let share_b = (b * mult) as f64;

        let ask_first = rng.random_bool(0.5);
        let (target_val, target_name) = if ask_first { (share_a, "first") } else { (share_b, "second") };

        let prompt = format!(
            "Divide ${:.0} between two people in the ratio {}:{}.\n\nWhat is the share of the {} person?",
            total, a, b, target_name
        );

        let solution = format!(
            "**Step 1:** Total parts = {} + {} = {}\n\n\
             **Step 2:** Value of 1 part = ${:.0} / {} = ${:.0}\n\n\
             **Step 3:** {} share = {} × ${:.0} = **${:.0}**",
            a, b, sum_parts, total, sum_parts, mult as f64,
            target_name, if ask_first { a } else { b }, mult as f64, target_val
        );

        let parameters = serde_json::json!({
            "variant": "divide_amount",
            "a": a, "b": b, "total": total,
            "share_a": share_a, "share_b": share_b, "ask_first": ask_first,
        });

        let correct_answer = serde_json::json!({
            "value": target_val,
            "formatted": format!("${:.0}", target_val),
            "solution": solution,
        });

        let step1 = StepNode::new(
            "total_parts",
            StepType::Arithmetic,
            "Find total ratio parts",
            format!("{} + {} = {}", a, b, sum_parts),
            format!("{}", sum_parts),
        )
        .with_expected_value(sum_parts as f64);

        let step2 = StepNode::new(
            "unit_value",
            StepType::IntermediateResult,
            "Calculate value of one unit part",
            format!("${:.0} / {} = ${:.0}", total, sum_parts, mult),
            format!("{:.0}", mult as f64),
        )
        .with_expected_value(mult as f64)
        .with_dependencies(vec!["total_parts".to_string()]);

        let step3 = StepNode::new(
            "target_share",
            StepType::FinalAnswer,
            "Calculate requested share",
            format!("{} * ${:.0} = ${:.0}", if ask_first { a } else { b }, mult, target_val),
            format!("{:.0}", target_val),
        )
        .with_expected_value(target_val)
        .with_dependencies(vec!["unit_value".to_string()])
        .as_final();

        let graph = SolutionGraph::new(vec![step1, step2, step3], "target_share");

        ProblemInstance::new(
            format!("inst-ratio-1-{}", seed),
            FAMILY_RATIO,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_metadata(serde_json::json!({
            "difficulty": 1.0,
            "target_time_ms": 25_000,
            "generator": TEMPLATE_RATIO_V1,
        }))
        .with_solution_graph(graph)
    }

    /// Level 2: Missing proportion term A : B = C : x
    fn generate_level_2(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let a = rng.random_range(3..=25);
        let b = rng.random_range(2..=25);
        let factor = rng.random_range(3..=20);
        let c = a * factor;
        let x = b * factor;

        let prompt = format!("Find the missing term \\(x\\) in the proportion:\n\n\\[ {}:{} = {}:x \\]", a, b, c);

        let solution = format!(
            "**Formula:** In a proportion \\( a:b = c:d \\), the product of means equals product of extremes:\n\
             \\[ {} \\times x = {} \\times {} \\]\n\
             \\[ {}x = {} \\]\n\
             \\[ x = \\frac{{{}}}{{{}}} = **{}** \\]",
            a, b, c, a, b * c, b * c, a, x
        );

        let parameters = serde_json::json!({
            "variant": "missing_proportion",
            "a": a, "b": b, "c": c, "solution_x": x,
        });

        let correct_answer = serde_json::json!({
            "value": x as f64,
            "formatted": format!("{}", x),
            "solution": solution,
        });

        let step1 = StepNode::new(
            "solve_x",
            StepType::FinalAnswer,
            "Solve for x = (b * c) / a",
            format!("({} * {}) / {} = {}", b, c, a, x),
            format!("{}", x),
        )
        .with_expected_value(x as f64)
        .as_final();

        let graph = SolutionGraph::new(vec![step1], "solve_x");

        ProblemInstance::new(
            format!("inst-ratio-2-{}", seed),
            FAMILY_RATIO,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_metadata(serde_json::json!({
            "difficulty": 2.0,
            "target_time_ms": 30_000,
            "generator": TEMPLATE_RATIO_V1,
        }))
        .with_solution_graph(graph)
    }

    /// Level 3: Combining Ratios A:B and B:C with dynamic multipliers
    fn generate_level_3(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let a = rng.random_range(2..=8);
        let b1 = rng.random_range(3..=9);
        let b2 = rng.random_range(2..=8);
        let c = rng.random_range(3..=10);

        let a_term = a * b2;
        let b_term = b1 * b2;
        let c_term = b1 * c;

        let scale = rng.random_range(2..=25);
        let c_val = c_term * scale;
        let ans_a = (a_term * scale) as f64;

        let prompt = format!(
            "If the ratio of \\(A:B\\) is {}:{} and the ratio of \\(B:C\\) is {}:{}.\n\nIf \\(C = {}\\), what is the value of \\(A\\)?",
            a, b1, b2, c, c_val
        );

        let solution = format!(
            "**Step 1:** Normalize the common term \\(B\\):\n\
             \\(A:B = ({}\\times{}) : ({}\\times{}) = {}:{} \\)\n\
             \\(B:C = ({}\\times{}) : ({}\\times{}) = {}:{} \\)\n\
             \\(A:B:C = {}:{}:{} \\)\n\n\
             **Step 2:** Scale with given \\(C = {}\\):\n\
             \\(1\\text{{ part}} = {} / {} = {} \\)\n\
             \\(A = {} \\times {} = **{:.0}** \\)",
            a, b2, b1, b2, a_term, b_term,
            b2, b1, c, b1, b_term, c_term,
            a_term, b_term, c_term,
            c_val, c_val, c_term, scale,
            a_term, scale, ans_a
        );

        let parameters = serde_json::json!({
            "variant": "three_part_ratio",
            "a": a, "b1": b1, "b2": b2, "c": c, "c_val": c_val, "scale": scale, "ans_a": ans_a,
        });

        let correct_answer = serde_json::json!({
            "value": ans_a,
            "formatted": format!("{:.0}", ans_a),
            "solution": solution,
        });

        let step1 = StepNode::new(
            "combine_ratios",
            StepType::Transformation,
            "Normalize common term B",
            format!("A:B:C = {}:{}:{}", a_term, b_term, c_term),
            format!("{}:{}:{}", a_term, b_term, c_term),
        );

        let step2 = StepNode::new(
            "solve_a",
            StepType::FinalAnswer,
            "Solve for A",
            format!("{} * {} = {:.0}", a_term, scale, ans_a),
            format!("{:.0}", ans_a),
        )
        .with_expected_value(ans_a)
        .with_dependencies(vec!["combine_ratios".to_string()])
        .as_final();

        let graph = SolutionGraph::new(vec![step1, step2], "solve_a");

        ProblemInstance::new(
            format!("inst-ratio-3-{}", seed),
            FAMILY_RATIO,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_metadata(serde_json::json!({
            "difficulty": 3.0,
            "target_time_ms": 45_000,
            "generator": TEMPLATE_RATIO_V1,
        }))
        .with_solution_graph(graph)
    }

    /// Level 4: Ratio Shift after adding a constant with dynamic ratios
    fn generate_level_4(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let pairs = [
            (2, 3, 3, 4),
            (3, 4, 4, 5),
            (4, 5, 5, 6),
            (5, 7, 3, 4),
            (1, 2, 2, 3),
            (3, 5, 2, 3),
            (5, 8, 3, 4),
        ];
        let (a, b, c, d) = pairs[rng.random_range(0..pairs.len())];
        let x = rng.random_range(3..=20);
        let k = ((c * b * x - d * a * x) as f64 / (d - c) as f64).round() as i64;
        let num1 = a * x;

        let prompt = format!(
            "Two numbers are in the ratio {}:{}. If {} is added to each number, the ratio becomes {}:{}.\n\nWhat is the smaller number?",
            a, b, k, c, d
        );

        let solution = format!(
            "**Step 1:** Let numbers be \\({}x\\) and \\({}x\\).\n\
             \\[ \\frac{{{x_a} + {k}}}{{{x_b} + {k}}} = \\frac{{{c}}}{{{d}}} \\]\n\n\
             **Step 2:** Cross multiply and solve: \\(x = {}\\)\n\n\
             **Step 3:** Smaller number = \\({} \\times {} = **{}** \\)",
            a, b, x, a, x, num1, x_a = format!("{}x", a), x_b = format!("{}x", b)
        );

        let parameters = serde_json::json!({
            "variant": "ratio_shift",
            "a": a, "b": b, "c": c, "d": d, "k": k, "smaller_num": num1,
        });

        let correct_answer = serde_json::json!({
            "value": num1 as f64,
            "formatted": format!("{}", num1),
            "solution": solution,
        });

        let step1 = StepNode::new(
            "smaller_number",
            StepType::FinalAnswer,
            "Calculate smaller number",
            format!("{} * {} = {}", a, x, num1),
            format!("{}", num1),
        )
        .with_expected_value(num1 as f64)
        .as_final();

        let graph = SolutionGraph::new(vec![step1], "smaller_number");

        ProblemInstance::new(
            format!("inst-ratio-4-{}", seed),
            FAMILY_RATIO,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_metadata(serde_json::json!({
            "difficulty": 4.0,
            "target_time_ms": 60_000,
            "generator": TEMPLATE_RATIO_V1,
        }))
        .with_solution_graph(graph)
    }

    /// Level 5: Mixture Replacement / Ratio Adjustment with dynamic parameters
    fn generate_level_5(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let m_ratio = rng.random_range(3..=8) as f64;
        let w_ratio = rng.random_range(1..=4) as f64;
        let mult = rng.random_range(4..=15) as f64;

        let milk = m_ratio * mult;
        let water = w_ratio * mult;
        let total_liters = milk + water;

        // Target ratio: milk to (water + w) = target_m : target_w
        let target_m = m_ratio;
        let target_w = w_ratio + rng.random_range(1..=3) as f64;

        let added_water = (milk * target_w / target_m) - water;

        let prompt = format!(
            "A mixture of {:.0} liters contains milk and water in the ratio {:.0}:{:.0}.\n\n\
             How many liters of water must be added to make the ratio of milk to water {:.0}:{:.0}?",
            total_liters, m_ratio, w_ratio, target_m, target_w
        );

        let solution = format!(
            "**Step 1:** Initial volumes: Milk = {:.0} L, Water = {:.0} L.\n\n\
             **Step 2:** Set up new ratio: \\(\\frac{{{:.0}}}{{{:.0} + w}} = \\frac{{{:.0}}}{{{:.0}}}\\)\n\n\
             **Step 3:** Solve for added water \\(w\\) = **{:.0} liters**.",
            milk, water, milk, water, target_m, target_w, added_water
        );

        let parameters = serde_json::json!({
            "variant": "mixture_proportion",
            "total_liters": total_liters,
            "milk": milk,
            "water": water,
            "added_water": added_water,
        });

        let correct_answer = serde_json::json!({
            "value": added_water,
            "formatted": format!("{:.0}", added_water),
            "unit": "liters",
            "solution": solution,
        });

        let step1 = StepNode::new(
            "solve_added_water",
            StepType::FinalAnswer,
            "Solve for added water volume",
            format!("{:.0}", added_water),
            format!("{:.0}", added_water),
        )
        .with_expected_value(added_water)
        .as_final();

        let graph = SolutionGraph::new(vec![step1], "solve_added_water");

        ProblemInstance::new(
            format!("inst-ratio-5-{}", seed),
            FAMILY_RATIO,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_metadata(serde_json::json!({
            "difficulty": 5.0,
            "target_time_ms": 60_000,
            "generator": TEMPLATE_RATIO_V1,
        }))
        .with_solution_graph(graph)
    }
}

impl ProblemGenerator for RatioGenerator {
    fn family_id(&self) -> &str {
        FAMILY_RATIO
    }

    fn template_ref(&self) -> &str {
        TEMPLATE_RATIO_V1
    }

    fn supported_variants(&self) -> Vec<String> {
        vec![
            "divide_amount".to_string(),
            "missing_proportion".to_string(),
            "three_part_ratio".to_string(),
            "ratio_shift".to_string(),
            "mixture_proportion".to_string(),
        ]
    }

    fn target_latency_ms(&self, difficulty_level: u32) -> u64 {
        match difficulty_level {
            1 => 25_000,
            2 => 30_000,
            3 => 45_000,
            4 => 60_000,
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

pub struct RatioValidator;

impl ProblemValidator for RatioValidator {
    fn family_id(&self) -> &str {
        FAMILY_RATIO
    }

    fn evaluate(
        &self,
        instance: &ProblemInstance,
        student_input: &serde_json::Value,
        time_taken_ms: u64,
        target_time_ms: u64,
    ) -> AnswerEvaluation {
        let expected_val = instance
            .correct_answer
            .get("value")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let parsed_val = NumericAnswerParser::parse_value(student_input);
        let Some(student_num) = parsed_val else {
            return AnswerEvaluation {
                is_correct: false,
                score: 0.0,
                parsed_student_value: None,
                canonical_value: expected_val,
                error_category: Some(ErrorCategory::Calculation),
                diagnostic_message: Some("Could not parse answer as a number.".to_string()),
            };
        };

        let diff = (student_num - expected_val).abs();
        let is_correct = diff <= 0.1 || (expected_val != 0.0 && diff / expected_val.abs() <= 0.01);

        if is_correct {
            let score = if target_time_ms > 0 && time_taken_ms > target_time_ms * 2 {
                0.8
            } else {
                1.0
            };
            AnswerEvaluation::correct(score, time_taken_ms, target_time_ms)
                .with_parsed_values(student_num, expected_val)
        } else {
            AnswerEvaluation::incorrect(
                ErrorCategory::Calculation,
                format!("Incorrect answer. Submitted {:.2}, expected {:.2}.", student_num, expected_val),
            )
            .with_parsed_values(student_num, expected_val)
        }
    }
}
