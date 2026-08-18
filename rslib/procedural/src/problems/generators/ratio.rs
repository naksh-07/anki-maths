// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

use crate::core::{ProblemFamilyId, Result};
use crate::diagnostics::ErrorCategory;
use crate::problems::generator::ProblemGenerator;
use crate::problems::steps::{SolutionGraph, StepHint, StepNode, StepType};
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

    /// Level 1: Divide Total into A:B
    fn generate_level_1(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let a = rng.random_range(2..=7);
        let b = rng.random_range(3..=9);
        let sum_parts = a + b;
        let mult = rng.random_range(10..=50) * 10;
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
            "a": a,
            "b": b,
            "total": total,
            "share_a": share_a,
            "share_b": share_b,
            "ask_first": ask_first,
        });

        let correct_answer = serde_json::json!({
            "value": target_val,
            "formatted": format!("${:.0}", target_val),
            "solution": solution,
        });

        let step1 = StepNode::new(
            "total_parts",
            StepType::Arithmetic,
            "Find total number of ratio parts",
            format!("Add {} + {}", a, b),
            format!("{}", sum_parts),
        )
        .with_expected_value(sum_parts as f64)
        .with_hints(vec![
            StepHint::principle("In ratio division problems, first determine the total number of proportional parts."),
            StepHint::operation(format!("Add the ratio terms: {} + {} = {}.", a, b, sum_parts)),
            StepHint::intermediate_relation(format!("Total Parts = {} + {} = {}", a, b, sum_parts)),
        ]);

        let step2 = StepNode::new(
            "unit_value",
            StepType::IntermediateResult,
            "Calculate value of one unit part",
            format!("Divide ${:.0} by {}", total, sum_parts),
            format!("{:.0}", mult as f64),
        )
        .with_expected_value(mult as f64)
        .with_dependencies(vec!["total_parts".to_string()])
        .with_hints(vec![
            StepHint::principle("Divide total amount by total parts to find the monetary value of one part."),
            StepHint::operation(format!("Divide ${:.0} by {}.", total, sum_parts)),
            StepHint::intermediate_relation(format!("1 Part = ${:.0} / {} = ${:.0}", total, sum_parts, mult as f64)),
        ]);

        let step3 = StepNode::new(
            "target_share",
            StepType::FinalAnswer,
            "Calculate requested share",
            format!("Multiply {} by ${:.0}", if ask_first { a } else { b }, mult as f64),
            format!("{:.0}", target_val),
        )
        .with_expected_value(target_val)
        .with_alternates(vec![format!("${:.0}", target_val)])
        .with_dependencies(vec!["unit_value".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Multiply the unit value by the number of parts for the specified person."),
            StepHint::operation(format!("Multiply {} parts * ${:.0}.", if ask_first { a } else { b }, mult as f64)),
            StepHint::intermediate_relation(format!("Share = {} * ${:.0} = ${:.0}", if ask_first { a } else { b }, mult as f64, target_val)),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2, step3], "target_share");

        let metadata = serde_json::json!({
            "difficulty": 1.0,
            "target_time_ms": 25_000,
            "generator": TEMPLATE_RATIO_V1,
        });

        ProblemInstance::new(
            format!("inst-ratio-1-{}", seed),
            FAMILY_RATIO,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_metadata(metadata)
        .with_solution_graph(graph)
    }

    /// Level 2: Missing proportion term A : B = C : x
    fn generate_level_2(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let a = rng.random_range(3..=9);
        let b = rng.random_range(2..=8);
        let factor = rng.random_range(3..=12);
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
            "a": a,
            "b": b,
            "c": c,
            "solution_x": x,
        });

        let correct_answer = serde_json::json!({
            "value": x as f64,
            "formatted": format!("{}", x),
            "solution": solution,
        });

        let step1 = StepNode::new(
            "cross_multiply",
            StepType::EquationRearrangement,
            "Cross multiply proportion",
            format!("Means and extremes: {} * x = {} * {}", a, b, c),
            format!("{}x = {}", a, b * c),
        )
        .with_hints(vec![
            StepHint::principle("In proportion a:b = c:d, product of extremes equals product of means: a * d = b * c."),
            StepHint::operation(format!("Write {} * x = {} * {}.", a, b, c)),
            StepHint::intermediate_relation(format!("{}x = {}", a, b * c)),
        ]);

        let step2 = StepNode::new(
            "solve_x",
            StepType::FinalAnswer,
            "Solve for x",
            format!("Divide {} by {}", b * c, a),
            format!("{}", x),
        )
        .with_expected_value(x as f64)
        .with_alternates(vec![format!("x = {}", x)])
        .with_dependencies(vec!["cross_multiply".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Divide product of means by known extreme term."),
            StepHint::operation(format!("Divide {} by {}.", b * c, a)),
            StepHint::intermediate_relation(format!("x = {} / {} = {}", b * c, a, x)),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2], "solve_x");

        let metadata = serde_json::json!({
            "difficulty": 2.0,
            "target_time_ms": 30_000,
            "generator": TEMPLATE_RATIO_V1,
        });

        ProblemInstance::new(
            format!("inst-ratio-2-{}", seed),
            FAMILY_RATIO,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_metadata(metadata)
        .with_solution_graph(graph)
    }

    /// Level 3: Combining Ratios A:B and B:C
    fn generate_level_3(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let a = rng.random_range(2..=5);
        let b1 = rng.random_range(3..=6);
        let b2 = rng.random_range(2..=5);
        let c = rng.random_range(3..=7);

        // A:B = a:b1, B:C = b2:c => A:B:C = (a*b2) : (b1*b2) : (b1*c)
        let a_term = a * b2;
        let b_term = b1 * b2;
        let c_term = b1 * c;

        let prompt = format!(
            "If the ratio of \\(A:B\\) is {}:{} and the ratio of \\(B:C\\) is {}:{}.\n\nIf \\(C = {}\\), what is the value of \\(A\\)?",
            a, b1, b2, c, c_term * 5
        );

        let ans_a = (a_term * 5) as f64;
        let solution = format!(
            "**Step 1:** Normalize the common term \\(B\\):\n\
             \\(A:B = ({}\\times{}) : ({}\\times{}) = {}:{} \\)\n\
             \\(B:C = ({}\\times{}) : ({}\\times{}) = {}:{} \\)\n\
             \\(A:B:C = {}:{}:{} \\)\n\n\
             **Step 2:** Scale with given \\(C = {}\\):\n\
             \\(1\\text{{ part}} = {} / {} = 5 \\)\n\
             \\(A = {} \\times 5 = **{:.0}** \\)",
            a, b2, b1, b2, a_term, b_term,
            b2, b1, c, b1, b_term, c_term,
            a_term, b_term, c_term,
            c_term * 5, c_term * 5, c_term,
            a_term, ans_a
        );

        let parameters = serde_json::json!({
            "variant": "three_part_ratio",
            "a": a,
            "b1": b1,
            "b2": b2,
            "c": c,
            "c_val": c_term * 5,
            "ans_a": ans_a,
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
            format!("Combine to form A:B:C = {}:{}:{}", a_term, b_term, c_term),
            format!("{}:{}:{}", a_term, b_term, c_term),
        )
        .with_hints(vec![
            StepHint::principle("Make the common term B identical in both ratios by multiplying across."),
            StepHint::operation(format!("Multiply ratio 1 by {} and ratio 2 by {}.", b2, b1)),
            StepHint::intermediate_relation(format!("A:B:C = {}:{}:{}", a_term, b_term, c_term)),
        ]);

        let step2 = StepNode::new(
            "solve_a",
            StepType::FinalAnswer,
            "Solve for A",
            format!("Multiply A's parts ({}) by scale factor (5)", a_term),
            format!("{:.0}", ans_a),
        )
        .with_expected_value(ans_a)
        .with_dependencies(vec!["combine_ratios".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Find unit scale = C / C_parts and multiply by A_parts."),
            StepHint::operation(format!("Scale is {} / {} = 5. Multiply {} * 5.", c_term * 5, c_term, a_term)),
            StepHint::intermediate_relation(format!("A = {} * 5 = {:.0}", a_term, ans_a)),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2], "solve_a");

        let metadata = serde_json::json!({
            "difficulty": 3.0,
            "target_time_ms": 45_000,
            "generator": TEMPLATE_RATIO_V1,
        });

        ProblemInstance::new(
            format!("inst-ratio-3-{}", seed),
            FAMILY_RATIO,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_metadata(metadata)
        .with_solution_graph(graph)
    }

    /// Level 4: Ratio Shift after adding a constant
    fn generate_level_4(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let x = rng.random_range(3..=10);
        let a = 3;
        let b = 4;
        let c = 4;
        let d = 5;
        let k = (c * b * x - d * a * x) / (d - c);
        let num1 = a * x;
        let _num2 = b * x;

        let prompt = format!(
            "Two numbers are in the ratio {}:{}. If {} is added to each number, the ratio becomes {}:{}.\n\nWhat is the smaller number?",
            a, b, k, c, d
        );

        let x_a = format!("{}x", a);
        let x_b = format!("{}x", b);
        let d_xa = format!("{}x", d * a);
        let c_xb = format!("{}x", c * b);
        let dk = d * k;
        let ck = c * k;

        let solution = format!(
            "**Step 1:** Let numbers be \\({a}x\\) and \\({b}x\\).\n\
             \\[ \\frac{{{x_a} + {k}}}{{{x_b} + {k}}} = \\frac{{{c}}}{{{d}}} \\]\n\n\
             **Step 2:** Cross multiply:\n\
             \\[ {d}({x_a} + {k}) = {c}({x_b} + {k}) \\]\n\
             \\[ {d_xa} + {dk} = {c_xb} + {ck} \\]\n\
             \\[ x = {x} \\]\n\n\
             **Step 3:** Smaller number = \\({a} \\times {x} = **{num1}** \\)"
        );

        let parameters = serde_json::json!({
            "variant": "ratio_shift",
            "a": a,
            "b": b,
            "c": c,
            "d": d,
            "k": k,
            "smaller_num": num1,
        });

        let correct_answer = serde_json::json!({
            "value": num1 as f64,
            "formatted": format!("{}", num1),
            "solution": solution,
        });

        let step1 = StepNode::new(
            "cross_multiply_shift",
            StepType::EquationRearrangement,
            "Cross multiply ratio shift",
            format!("{} * ({}x + {}) = {} * ({}x + {})", d, a, k, c, b, k),
            format!("{} + {} = {} + {}", d_xa, dk, c_xb, ck),
        )
        .with_hints(vec![
            StepHint::principle("Represent the numbers as ax and bx, then cross multiply the new ratio fraction."),
            StepHint::operation(format!("Cross multiply: {}({}x + {}) = {}({}x + {}).", d, a, k, c, b, k)),
            StepHint::intermediate_relation(format!("{} + {} = {} + {}", d_xa, dk, c_xb, ck)),
        ]);

        let step2 = StepNode::new(
            "solve_multiplier",
            StepType::IntermediateResult,
            "Solve multiplier x",
            "Isolate variable x",
            format!("x = {}", x),
        )
        .with_expected_value(x as f64)
        .with_dependencies(vec!["cross_multiply_shift".to_string()])
        .with_hints(vec![
            StepHint::principle("Subtract like terms to isolate x."),
            StepHint::operation(format!("Subtract {} from {} to find x.", d_xa, c_xb)),
            StepHint::intermediate_relation(format!("x = {}", x)),
        ]);

        let step3 = StepNode::new(
            "smaller_number",
            StepType::FinalAnswer,
            "Calculate smaller number",
            format!("Multiply {} * {}", a, x),
            format!("{}", num1),
        )
        .with_expected_value(num1 as f64)
        .with_dependencies(vec!["solve_multiplier".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Multiply the ratio coefficient of the smaller number by x."),
            StepHint::operation(format!("Compute {} * {}.", a, x)),
            StepHint::intermediate_relation(format!("Smaller number = {} * {} = {}", a, x, num1)),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2, step3], "smaller_number");

        let metadata = serde_json::json!({
            "difficulty": 4.0,
            "target_time_ms": 60_000,
            "generator": TEMPLATE_RATIO_V1,
        });

        ProblemInstance::new(
            format!("inst-ratio-4-{}", seed),
            FAMILY_RATIO,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_metadata(metadata)
        .with_solution_graph(graph)
    }

    /// Level 5: Mixture Replacement / Ratio Adjustment
    fn generate_level_5(_rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let total_liters = 60.0;
        let m_ratio = 7.0;
        let w_ratio = 3.0; // 7:3 => Milk = 42L, Water = 18L
        let milk = total_liters * (m_ratio / (m_ratio + w_ratio));
        let water = total_liters * (w_ratio / (m_ratio + w_ratio));
        let added_water = 10.0;

        let prompt = format!(
            "A mixture of {:.0} liters contains milk and water in the ratio 7:3.\n\nHow many liters of water must be added to make the ratio of milk to water 3:2?",
            total_liters
        );

        let solution = format!(
            "**Step 1:** Calculate initial quantities:\n\
             Milk = {:.0} × (7/10) = {:.0} L\n\
             Water = {:.0} × (3/10) = {:.0} L\n\n\
             **Step 2:** Set up equation with added water \\(w\\):\n\
             \\[ \\frac{{{:.0}}}{{{:.0} + w}} = \\frac{{3}}{{2}} \\]\n\
             \\[ 2 \\times {:.0} = 3({:.0} + w) \\]\n\
             \\[ 84 = 54 + 3w \\]\n\
             \\[ 3w = 30 \\implies w = **{:.0}** \\text{{ liters}} \\]",
            total_liters, milk, total_liters, water, milk, water, milk, water, added_water
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
            "initial_components",
            StepType::IntermediateResult,
            "Calculate initial component volumes",
            "Compute initial milk and water volumes",
            format!("Milk={:.0}, Water={:.0}", milk, water),
        )
        .with_hints(vec![
            StepHint::principle("Find the exact quantity of each liquid before anything is added."),
            StepHint::operation(format!("Milk = {:.0} * (7/10) = {:.0}L, Water = {:.0} * (3/10) = {:.0}L.", total_liters, milk, total_liters, water)),
            StepHint::intermediate_relation(format!("Milk = {:.0}L, Water = {:.0}L", milk, water)),
        ]);

        let step2 = StepNode::new(
            "solve_added_water",
            StepType::FinalAnswer,
            "Solve for added water volume",
            "Cross multiply new ratio equation",
            format!("{:.0}", added_water),
        )
        .with_expected_value(added_water)
        .with_alternates(vec![
            format!("{:.0} liters", added_water),
            format!("{:.0}L", added_water),
        ])
        .with_dependencies(vec!["initial_components".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Set Milk / (Water + w) = 3/2 and solve for w."),
            StepHint::operation(format!("84 = 54 + 3w => 3w = 30 => w = {:.0}L.", added_water)),
            StepHint::intermediate_relation(format!("Added water = {:.0} liters", added_water)),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2], "solve_added_water");

        let metadata = serde_json::json!({
            "difficulty": 5.0,
            "target_time_ms": 70_000,
            "generator": TEMPLATE_RATIO_V1,
        });

        ProblemInstance::new(
            format!("inst-ratio-5-{}", seed),
            FAMILY_RATIO,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_metadata(metadata)
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
            _ => 70_000,
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
        _time_taken_ms: u64,
        _target_time_ms: u64,
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
        let is_correct = diff <= 0.01;

        if is_correct {
            AnswerEvaluation {
                is_correct: true,
                score: 1.0,
                parsed_student_value: Some(student_num),
                canonical_value: expected_val,
                error_category: None,
                diagnostic_message: None,
            }
        } else {
            let (cat, msg) = Self::classify_misconception(student_num, &instance.parameters, expected_val);
            AnswerEvaluation {
                is_correct: false,
                score: 0.0,
                parsed_student_value: Some(student_num),
                canonical_value: expected_val,
                error_category: Some(cat),
                diagnostic_message: Some(msg),
            }
        }
    }
}

impl RatioValidator {
    fn classify_misconception(
        student_val: f64,
        params: &serde_json::Value,
        expected_val: f64,
    ) -> (ErrorCategory, String) {
        let variant = params.get("variant").and_then(|v| v.as_str()).unwrap_or("");

        // Inverted share error in level 1 (e.g. answered person B's share instead of A's)
        if variant == "divide_amount" {
            let share_a = params.get("share_a").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let share_b = params.get("share_b").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let ask_first = params.get("ask_first").and_then(|v| v.as_bool()).unwrap_or(true);

            let alternate = if ask_first { share_b } else { share_a };
            if (student_val - alternate).abs() <= 0.01 {
                return (
                    ErrorCategory::Careless,
                    "Inverted share: Calculated the share for the other person in the ratio.".to_string(),
                );
            }
        }

        (
            ErrorCategory::Unknown,
            format!("Incorrect answer: Expected {}.", expected_val),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ratio_generation_all_levels() {
        let gen = RatioGenerator;
        for lvl in 1..=5 {
            let inst = gen
                .generate(&ProblemFamilyId::new(FAMILY_RATIO), 4242, lvl, None)
                .unwrap();
            assert!(!inst.rendered_prompt.is_empty());
            assert!(inst.correct_answer.get("value").is_some());
        }
    }

    #[test]
    fn test_ratio_inverted_share_diagnostic() {
        let validator = RatioValidator;
        let gen = RatioGenerator;
        let inst = gen
            .generate(&ProblemFamilyId::new(FAMILY_RATIO), 1001, 1, Some("divide_amount"))
            .unwrap();

        let share_a = inst.parameters.get("share_a").unwrap().as_f64().unwrap();
        let share_b = inst.parameters.get("share_b").unwrap().as_f64().unwrap();
        let ask_first = inst.parameters.get("ask_first").unwrap().as_bool().unwrap();

        let wrong_alternate = if ask_first { share_b } else { share_a };
        let eval = validator.evaluate(&inst, &serde_json::json!(wrong_alternate), 15000, 25000);
        assert!(!eval.is_correct);
        assert_eq!(eval.error_category, Some(ErrorCategory::Careless));
        assert!(eval.diagnostic_message.unwrap().contains("Inverted share"));
    }
}
