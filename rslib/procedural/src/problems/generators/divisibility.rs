// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

use crate::core::{ProblemFamilyId, Result};
use crate::diagnostics::ErrorCategory;
use crate::problems::generator::ProblemGenerator;
use crate::problems::validator::{AnswerEvaluation, NumericAnswerParser, ProblemValidator};
use crate::problems::ProblemInstance;

pub const FAMILY_DIVISIBILITY: &str = "family.math.number_system.divisibility";
pub const TEMPLATE_DIVISIBILITY_V1: &str = "math.number_system.divisibility.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DivisibilityVariant {
    SingleRuleCheck,
    SingleMissingDigit,
    CompositeDivisibility,
    RemainderProblem,
    TwoMissingDigits,
}

impl DivisibilityVariant {
    pub fn as_str(&self) -> &'static str {
        match self {
            DivisibilityVariant::SingleRuleCheck => "single_rule_check",
            DivisibilityVariant::SingleMissingDigit => "single_missing_digit",
            DivisibilityVariant::CompositeDivisibility => "composite_divisibility",
            DivisibilityVariant::RemainderProblem => "remainder_problem",
            DivisibilityVariant::TwoMissingDigits => "two_missing_digits",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct DivisibilityGenerator;

impl DivisibilityGenerator {
    pub fn generate_problem(seed: u64, difficulty_level: u32, variant: Option<&str>) -> ProblemInstance {
        let mut rng = StdRng::seed_from_u64(seed);
        let chosen_variant = if let Some(v) = variant {
            match v {
                "single_rule_check" => DivisibilityVariant::SingleRuleCheck,
                "single_missing_digit" => DivisibilityVariant::SingleMissingDigit,
                "composite_divisibility" => DivisibilityVariant::CompositeDivisibility,
                "remainder_problem" => DivisibilityVariant::RemainderProblem,
                "two_missing_digits" => DivisibilityVariant::TwoMissingDigits,
                _ => DivisibilityVariant::SingleRuleCheck,
            }
        } else {
            match difficulty_level {
                1 => DivisibilityVariant::SingleRuleCheck,
                2 => DivisibilityVariant::SingleMissingDigit,
                3 => DivisibilityVariant::CompositeDivisibility,
                4 => DivisibilityVariant::RemainderProblem,
                _ => DivisibilityVariant::TwoMissingDigits,
            }
        };

        match chosen_variant {
            DivisibilityVariant::SingleRuleCheck => Self::generate_level_1(&mut rng, seed),
            DivisibilityVariant::SingleMissingDigit => Self::generate_level_2(&mut rng, seed),
            DivisibilityVariant::CompositeDivisibility => Self::generate_level_3(&mut rng, seed),
            DivisibilityVariant::RemainderProblem => Self::generate_level_4(&mut rng, seed),
            DivisibilityVariant::TwoMissingDigits => Self::generate_level_5(&mut rng, seed),
        }
    }

    /// Level 1: Divisibility remainder check for diverse divisors (3, 4, 7, 8, 9, 11, 13)
    fn generate_level_1(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let num = rng.random_range(10_000..=999_999) as i64;
        let divisors = [3, 4, 7, 8, 9, 11, 13];
        let divisor = divisors[rng.random_range(0..divisors.len())];
        let remainder = num % divisor;

        let prompt = format!(
            "What is the remainder when the number **{}** is divided by **{}**?",
            num, divisor
        );

        let solution = format!(
            "**Divisibility Evaluation:**\n\
             \\[ {} \\div {} = {} \\text{{ with remainder }} **{}** \\]\n\
             \\[ {} \\equiv {} \\pmod{{{}}} \\]",
            num, divisor, num / divisor, remainder, num, remainder, divisor
        );

        let parameters = serde_json::json!({
            "variant": "single_rule_check",
            "number": num,
            "divisor": divisor,
            "remainder": remainder,
        });

        let correct_answer = serde_json::json!({
            "value": remainder as f64,
            "formatted": format!("{}", remainder),
            "solution": solution,
        });

        let step1 = crate::problems::steps::StepNode::new(
            "calc_rem",
            crate::problems::steps::StepType::FinalAnswer,
            "Apply divisibility rule",
            format!("{} mod {} = {}", num, divisor, remainder),
            format!("{}", remainder),
        )
        .with_expected_value(remainder as f64)
        .as_final();

        let graph = crate::problems::steps::SolutionGraph::new(vec![step1], "calc_rem");

        ProblemInstance::new(
            format!("inst-div-1-{}", seed),
            FAMILY_DIVISIBILITY,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(serde_json::json!({
            "difficulty": 1.0,
            "target_time_ms": 25_000,
            "generator": TEMPLATE_DIVISIBILITY_V1,
        }))
    }

    /// Level 2: Find single missing digit x so that number is divisible by 3, 9, or 11
    fn generate_level_2(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let d1 = rng.random_range(1..=9);
        let d2 = rng.random_range(0..=9);
        let d3 = rng.random_range(0..=9);
        let d4 = rng.random_range(0..=9);

        let divisor = if rng.random_bool(0.5) { 9 } else { 11 };

        let mut target_x = 0;
        for x in 0..=9 {
            let candidate_num = d1 * 10_000 + d2 * 1_000 + d3 * 100 + d4 * 10 + x;
            if candidate_num % divisor == 0 {
                target_x = x;
                break;
            }
        }

        let prompt = format!(
            "Find the smallest single digit \\(x\\) such that the 5-digit number \\({}{}{}{}x\\) is completely divisible by **{}**.",
            d1, d2, d3, d4, divisor
        );

        let solution = format!(
            "**Divisibility by {}:**\n\
             Testing digits \\(x \\in [0..9]\\) reveals that \\(x = **{}**\\) creates the valid multiple \\({}{}{}{}{}\\) divisible by {}.",
            divisor, target_x, d1, d2, d3, d4, target_x, divisor
        );

        let parameters = serde_json::json!({
            "variant": "single_missing_digit",
            "d1": d1, "d2": d2, "d3": d3, "d4": d4,
            "divisor": divisor,
            "solution_x": target_x,
        });

        let correct_answer = serde_json::json!({
            "value": target_x as f64,
            "formatted": format!("{}", target_x),
            "solution": solution,
        });

        let step1 = crate::problems::steps::StepNode::new(
            "find_digit",
            crate::problems::steps::StepType::FinalAnswer,
            "Determine missing digit x",
            format!("x = {}", target_x),
            format!("{}", target_x),
        )
        .with_expected_value(target_x as f64)
        .as_final();

        let graph = crate::problems::steps::SolutionGraph::new(vec![step1], "find_digit");

        ProblemInstance::new(
            format!("inst-div-2-{}", seed),
            FAMILY_DIVISIBILITY,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(serde_json::json!({
            "difficulty": 2.0,
            "target_time_ms": 35_000,
            "generator": TEMPLATE_DIVISIBILITY_V1,
        }))
    }

    /// Level 3: Composite Divisibility (12, 15, 18, 36, 72)
    fn generate_level_3(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let composites = [(12, 3, 4), (15, 3, 5), (18, 2, 9), (36, 4, 9), (72, 8, 9)];
        let (comp, f1, f2) = composites[rng.random_range(0..composites.len())];

        let base_prefix = rng.random_range(100..=999) as i64;
        let last_digit = rng.random_range(0..=9) as i64;

        let mut target_x = None;
        for x in 0..=9 {
            let test_num = base_prefix * 100 + x * 10 + last_digit;
            if test_num % comp == 0 {
                target_x = Some(x);
                break;
            }
        }

        let x_val = target_x.unwrap_or(2);
        let valid_num = base_prefix * 100 + x_val * 10 + last_digit;

        let prompt = format!(
            "Find the smallest digit \\(x\\) such that the number \\({}x{}\\) is divisible by **{}**.",
            base_prefix, last_digit, comp
        );

        let solution = format!(
            "**Rule for {}:** The number must be coprime-divisible by both {} and {}.\n\n\
             Testing \\(x = **{}**\\) produces \\({}\\), which is divisible by {}.",
            comp, f1, f2, x_val, valid_num, comp
        );

        let parameters = serde_json::json!({
            "variant": "composite_divisibility",
            "base_prefix": base_prefix,
            "last_digit": last_digit,
            "composite": comp,
            "solution_x": x_val,
        });

        let correct_answer = serde_json::json!({
            "value": x_val as f64,
            "formatted": format!("{}", x_val),
            "solution": solution,
        });

        let step1 = crate::problems::steps::StepNode::new(
            "solve_x",
            crate::problems::steps::StepType::FinalAnswer,
            "Find digit satisfying composite factors",
            format!("x = {}", x_val),
            format!("{}", x_val),
        )
        .with_expected_value(x_val as f64)
        .as_final();

        let graph = crate::problems::steps::SolutionGraph::new(vec![step1], "solve_x");

        ProblemInstance::new(
            format!("inst-div-3-{}", seed),
            FAMILY_DIVISIBILITY,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(serde_json::json!({
            "difficulty": 3.0,
            "target_time_ms": 45_000,
            "generator": TEMPLATE_DIVISIBILITY_V1,
        }))
    }

    /// Level 4: Remainder problem: number gives remainder r1 mod m1
    fn generate_level_4(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let m = rng.random_range(12..=48) as i64;
        let r = rng.random_range(3..=(m - 2)) as i64;
        let d = rng.random_range(3..=11) as i64;

        let rem_ans = r % d;

        let prompt = format!(
            "A number \\(N\\) when divided by **{}** leaves a remainder of **{}**.\n\n\
             What will be the remainder when the same number \\(N\\) is divided by **{}**?",
            m, r, d
        );

        let solution = format!(
            "**Step 1:** Express \\(N\\) in division form:\n\
             \\[ N = {}k + {} \\]\n\n\
             **Step 2:** Modulo by {}:\n\
             \\[ N \\pmod{{{}}} = ({}k + {}) \\pmod{{{}}} \\]\n\
             \\[ N \\pmod{{{}}} = {} \\pmod{{{}}} = **{}** \\]",
            m, r, d, d, m, r, d, d, r, d, rem_ans
        );

        let parameters = serde_json::json!({
            "variant": "remainder_problem",
            "m": m, "r": r, "d": d, "rem_ans": rem_ans,
        });

        let correct_answer = serde_json::json!({
            "value": rem_ans as f64,
            "formatted": format!("{}", rem_ans),
            "solution": solution,
        });

        let step1 = crate::problems::steps::StepNode::new(
            "solve_rem",
            crate::problems::steps::StepType::FinalAnswer,
            "Evaluate remainder modulo d",
            format!("{} mod {} = {}", r, d, rem_ans),
            format!("{}", rem_ans),
        )
        .with_expected_value(rem_ans as f64)
        .as_final();

        let graph = crate::problems::steps::SolutionGraph::new(vec![step1], "solve_rem");

        ProblemInstance::new(
            format!("inst-div-4-{}", seed),
            FAMILY_DIVISIBILITY,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(serde_json::json!({
            "difficulty": 4.0,
            "target_time_ms": 30_000,
            "generator": TEMPLATE_DIVISIBILITY_V1,
        }))
    }

    /// Level 5: Two missing digits x and y for divisibility by 72 or 88
    fn generate_level_5(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let div = if rng.random_bool(0.5) { 72 } else { 88 };
        let p1 = rng.random_range(1..=9) as i64;
        let p2 = rng.random_range(0..=9) as i64;
        let p3 = rng.random_range(0..=9) as i64;

        let mut best_pair = (1, 2);
        for x in 0..=9 {
            for y in 0..=9 {
                let num = p1 * 100_000 + p2 * 10_000 + x * 1_000 + p3 * 100 + 40 + y;
                if num % div == 0 {
                    best_pair = (x, y);
                    break;
                }
            }
        }

        let (x_sol, y_sol) = best_pair;
        let target_val = (x_sol + y_sol) as f64;

        let prompt = format!(
            "If the 6-digit number \\({}{}x{}4y\\) is completely divisible by **{}**, find the value of \\((x + y)\\).",
            p1, p2, p3, div
        );

        let solution = format!(
            "**Divisibility by {}:**\n\
             The values satisfying both cofactor conditions are \\(x = {}\\) and \\(y = {}\\).\n\
             \\[ x + y = {} + {} = **{}** \\]",
            div, x_sol, y_sol, x_sol, y_sol, (x_sol + y_sol)
        );

        let parameters = serde_json::json!({
            "variant": "two_missing_digits",
            "p1": p1, "p2": p2, "p3": p3, "div": div,
            "x": x_sol, "y": y_sol, "ans": target_val,
        });

        let correct_answer = serde_json::json!({
            "value": target_val,
            "formatted": format!("{:.0}", target_val),
            "solution": solution,
        });

        let step1 = crate::problems::steps::StepNode::new(
            "solve_sum",
            crate::problems::steps::StepType::FinalAnswer,
            "Compute x + y",
            format!("{} + {} = {:.0}", x_sol, y_sol, target_val),
            format!("{:.0}", target_val),
        )
        .with_expected_value(target_val)
        .as_final();

        let graph = crate::problems::steps::SolutionGraph::new(vec![step1], "solve_sum");

        ProblemInstance::new(
            format!("inst-div-5-{}", seed),
            FAMILY_DIVISIBILITY,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(serde_json::json!({
            "difficulty": 5.0,
            "target_time_ms": 60_000,
            "generator": TEMPLATE_DIVISIBILITY_V1,
        }))
    }
}

impl ProblemGenerator for DivisibilityGenerator {
    fn family_id(&self) -> &str {
        FAMILY_DIVISIBILITY
    }

    fn template_ref(&self) -> &str {
        TEMPLATE_DIVISIBILITY_V1
    }

    fn supported_variants(&self) -> Vec<String> {
        vec![
            "single_rule_check".to_string(),
            "single_missing_digit".to_string(),
            "composite_divisibility".to_string(),
            "remainder_problem".to_string(),
            "two_missing_digits".to_string(),
        ]
    }

    fn target_latency_ms(&self, difficulty_level: u32) -> u64 {
        match difficulty_level {
            1 => 25_000,
            2 => 35_000,
            3 => 45_000,
            4 => 30_000,
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

pub struct DivisibilityValidator;

impl ProblemValidator for DivisibilityValidator {
    fn family_id(&self) -> &str {
        FAMILY_DIVISIBILITY
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
        let is_correct = diff <= 0.1;

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
