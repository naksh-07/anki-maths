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

    /// Level 1: Divisibility by 9 or 11 check (remainder)
    fn generate_level_1(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let d1 = rng.random_range(1..=9);
        let d2 = rng.random_range(0..=9);
        let d3 = rng.random_range(0..=9);
        let d4 = rng.random_range(0..=9);
        let num = d1 * 1000 + d2 * 100 + d3 * 10 + d4;
        let divisor = if rng.random_bool(0.5) { 9 } else { 11 };
        let remainder = num % divisor;

        let prompt = format!(
            "What is the remainder when the 4-digit number {} is divided by {}?",
            num, divisor
        );

        let sum_digits = d1 + d2 + d3 + d4;
        let solution = if divisor == 9 {
            format!(
                "**Rule for 9:** A number has the same remainder as the sum of its digits modulo 9.\n\n\
                 \\[ \\text{{Sum}} = {} + {} + {} + {} = {} \\]\n\
                 \\[ {} \\pmod 9 = **{}** \\]",
                d1, d2, d3, d4, sum_digits, sum_digits, remainder
            )
        } else {
            format!(
                "**Rule for 11:** Alternating sum of digits:\n\
                 \\[ ({}+{}) - ({}+{}) = {} - {} = {} \\]\n\
                 \\[ {} \\pmod{{11}} = **{}** \\]",
                d2, d4, d1, d3, d2 + d4, d1 + d3, (d2 + d4) - (d1 + d3), (d2 + d4) - (d1 + d3), remainder
            )
        };

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

        let metadata = serde_json::json!({
            "difficulty": 1.0,
            "target_time_ms": 25_000,
            "generator": TEMPLATE_DIVISIBILITY_V1,
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
        .with_metadata(metadata)
    }

    /// Level 2: Find single missing digit x so that number is divisible by 9
    fn generate_level_2(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let d1 = rng.random_range(2..=9);
        let d2 = rng.random_range(1..=9);
        let d3 = rng.random_range(1..=9);
        let sum_3 = d1 + d2 + d3;
        // Find x in 0..=9 such that (sum_3 + x) % 9 == 0
        let target_x = (9 - (sum_3 % 9)) % 9;

        let prompt = format!(
            "Find the digit \\(x\\) such that the 4-digit number \\({}{}{}x\\) is completely divisible by 9.",
            d1, d2, d3
        );

        let solution = format!(
            "**Rule for 9:** Sum of digits must be a multiple of 9.\n\n\
             \\[ {} + {} + {} + x = {} + x \\]\n\
             For \\({} + x\\) to be a multiple of 9, the smallest digit is \\(x = **{}**\\) (sum = {}).",
            d1, d2, d3, sum_3, sum_3, target_x, sum_3 + target_x
        );

        let parameters = serde_json::json!({
            "variant": "single_missing_digit",
            "d1": d1,
            "d2": d2,
            "d3": d3,
            "solution_x": target_x,
        });

        let correct_answer = serde_json::json!({
            "value": target_x as f64,
            "formatted": format!("{}", target_x),
            "solution": solution,
        });

        let metadata = serde_json::json!({
            "difficulty": 2.0,
            "target_time_ms": 35_000,
            "generator": TEMPLATE_DIVISIBILITY_V1,
        });

        let step1 = crate::problems::steps::StepNode::new(
            "sum_digits",
            crate::problems::steps::StepType::Transformation,
            "Sum known digits",
            format!("{} + {} + {} = {}", d1, d2, d3, sum_3),
            format!("{}", sum_3),
        )
        .with_expected_value(sum_3 as f64);

        let step2 = crate::problems::steps::StepNode::new(
            "find_digit",
            crate::problems::steps::StepType::FinalAnswer,
            "Find complement to next multiple of 9",
            format!("9 - ({} mod 9) = {}", sum_3, target_x),
            format!("{}", target_x),
        )
        .with_expected_value(target_x as f64)
        .with_dependencies(vec!["sum_digits".to_string()])
        .as_final();

        let graph = crate::problems::steps::SolutionGraph::new(vec![step1, step2], "find_digit");

        ProblemInstance::new(
            format!("inst-div-2-{}", seed),
            FAMILY_DIVISIBILITY,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(metadata)
    }

    /// Level 3: Composite Divisibility (Divisible by 12 = 3 * 4)
    fn generate_level_3(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let d1 = rng.random_range(3..=8);
        let d2 = rng.random_range(1..=7);
        // We want number d1 d2 x 2 to be divisible by 12 (divisible by 4 and 3)
        // Last 2 digits: x2 must be divisible by 4 => x can be 1, 3, 5, 7, 9 (since 12, 32, 52, 72, 92 are div by 4)
        // Also (d1 + d2 + x + 2) must be divisible by 3.
        let valid_candidates: Vec<i32> = [1, 3, 5, 7, 9]
            .iter()
            .copied()
            .filter(|&x| (d1 + d2 + x + 2) % 3 == 0)
            .collect();

        let target_x = if !valid_candidates.is_empty() {
            valid_candidates[0]
        } else {
            1
        };

        let prompt = format!(
            "Find the smallest digit \\(x\\) such that the number \\({}{}x2\\) is divisible by 12.",
            d1, d2
        );

        let solution = format!(
            "**Rule for 12:** The number must be simultaneously divisible by 3 and 4 (coprime factors).\n\n\
             **Condition 1 (Divisibility by 4):** Last two digits \\(x2\\) must be divisible by 4 \\(\\implies x \\in \\{{1, 3, 5, 7, 9\\}}\\).\n\n\
             **Condition 2 (Divisibility by 3):** Sum of digits \\({} + {} + x + 2 = {} + x\\) must be divisible by 3.\n\n\
             Checking candidates \\(x \\in \\{{1, 3, 5, 7, 9\\}}\\), the smallest valid digit is \\(x = **{}**\\).",
            d1, d2, d1 + d2 + 2, target_x
        );

        let parameters = serde_json::json!({
            "variant": "composite_divisibility",
            "d1": d1,
            "d2": d2,
            "solution_x": target_x,
        });

        let correct_answer = serde_json::json!({
            "value": target_x as f64,
            "formatted": format!("{}", target_x),
            "solution": solution,
        });

        let metadata = serde_json::json!({
            "difficulty": 3.0,
            "target_time_ms": 45_000,
            "generator": TEMPLATE_DIVISIBILITY_V1,
        });

        let step1 = crate::problems::steps::StepNode::new(
            "composite_check",
            crate::problems::steps::StepType::FinalAnswer,
            "Test candidates against mod 4 and mod 3",
            format!("Smallest valid x = {}", target_x),
            format!("{}", target_x),
        )
        .with_expected_value(target_x as f64)
        .as_final();

        let graph = crate::problems::steps::SolutionGraph::new(vec![step1], "composite_check");

        ProblemInstance::new(
            format!("inst-div-3-{}", seed),
            FAMILY_DIVISIBILITY,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(metadata)
    }

    /// Level 4: Remainder with powers
    fn generate_level_4(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let base = rng.random_range(2..=5);
        let exp = rng.random_range(10..=30);
        let mod_val = 7;
        // Compute (base^exp) % 7
        let mut rem = 1;
        for _ in 0..exp {
            rem = (rem * base) % mod_val;
        }

        let prompt = format!(
            "What is the remainder when \\({}^{{{}}}\\) is divided by {}?",
            base, exp, mod_val
        );

        let solution = format!(
            "**Step 1:** Find the period (cyclicity) of powers of {} modulo {}.\n\
             Using modular arithmetic exponent reduction:\n\
             \\[ {}^{{{}}} \\pmod{{{}}} = **{}** \\]",
            base, mod_val, base, exp, mod_val, rem
        );

        let parameters = serde_json::json!({
            "variant": "remainder_problem",
            "base": base,
            "exp": exp,
            "mod_val": mod_val,
            "remainder": rem,
        });

        let correct_answer = serde_json::json!({
            "value": rem as f64,
            "formatted": format!("{}", rem),
            "solution": solution,
        });

        let metadata = serde_json::json!({
            "difficulty": 4.0,
            "target_time_ms": 50_000,
            "generator": TEMPLATE_DIVISIBILITY_V1,
        });

        let step1 = crate::problems::steps::StepNode::new(
            "power_mod",
            crate::problems::steps::StepType::FinalAnswer,
            "Compute power modulo",
            format!("{}^{} mod {} = {}", base, exp, mod_val, rem),
            format!("{}", rem),
        )
        .with_expected_value(rem as f64)
        .as_final();

        let graph = crate::problems::steps::SolutionGraph::new(vec![step1], "power_mod");

        ProblemInstance::new(
            format!("inst-div-4-{}", seed),
            FAMILY_DIVISIBILITY,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(metadata)
    }

    /// Level 5: Two missing digits x and y in 56x34y divisible by 72 (find x + y)
    fn generate_level_5(_rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let _d1 = 5;
        let _d2 = 6;
        let _d3 = 3;
        let _d4 = 4;
        // Number is 56x34y. Divisible by 72 means div by 8 and 9.
        // Div by 8: last 3 digits 34y must be div by 8.
        // 340 / 8 = 42 rem 4 => 344 is div by 8 (344 = 43 * 8). So y = 4.
        let y = 4;
        // Div by 9: 5 + 6 + x + 3 + 4 + 4 = 22 + x must be div by 9 => x = 5 (22 + 5 = 27).
        let x = 5;
        let sum_xy = x + y; // 9

        let prompt = "If the 6-digit number \\(56x34y\\) is completely divisible by 72, what is the value of \\(x + y\\)?".to_string();

        let solution = format!(
            "**Rule for 72:** The number must be divisible by both 8 and 9 (coprime factors).\n\n\
             **Step 1 (Divisibility by 8):** Last three digits \\(34y\\) must be divisible by 8.\n\
             \\(340 \\div 8 = 42\\text{{ rem }}4 \\implies y = 4\\) (since 344 is divisible by 8).\n\n\
             **Step 2 (Divisibility by 9):** Sum of digits must be a multiple of 9.\n\
             \\[ 5 + 6 + x + 3 + 4 + 4 = 22 + x \\]\n\
             \\(22 + x = 27 \\implies x = 5\\).\n\n\
             **Step 3:** \\(x + y = 5 + 4 = **{}**\\)",
            sum_xy
        );

        let parameters = serde_json::json!({
            "variant": "two_missing_digits",
            "x": x,
            "y": y,
            "sum_xy": sum_xy,
        });

        let correct_answer = serde_json::json!({
            "value": sum_xy as f64,
            "formatted": format!("{}", sum_xy),
            "solution": solution,
        });

        let metadata = serde_json::json!({
            "difficulty": 5.0,
            "target_time_ms": 65_000,
            "generator": TEMPLATE_DIVISIBILITY_V1,
        });

        let step1 = crate::problems::steps::StepNode::new(
            "find_y",
            crate::problems::steps::StepType::Transformation,
            "Find y using last 3 digits mod 8",
            "344 mod 8 == 0 => y = 4".to_string(),
            "4".to_string(),
        )
        .with_expected_value(4.0);

        let step2 = crate::problems::steps::StepNode::new(
            "find_x",
            crate::problems::steps::StepType::IntermediateResult,
            "Find x using sum of digits mod 9",
            "22 + x = 27 => x = 5".to_string(),
            "5".to_string(),
        )
        .with_expected_value(5.0)
        .with_dependencies(vec!["find_y".to_string()]);

        let step3 = crate::problems::steps::StepNode::new(
            "calc_sum_xy",
            crate::problems::steps::StepType::FinalAnswer,
            "Compute x + y",
            format!("5 + 4 = {}", sum_xy),
            format!("{}", sum_xy),
        )
        .with_expected_value(sum_xy as f64)
        .with_dependencies(vec!["find_x".to_string()])
        .as_final();

        let graph = crate::problems::steps::SolutionGraph::new(vec![step1, step2, step3], "calc_sum_xy");

        ProblemInstance::new(
            format!("inst-div-5-{}", seed),
            FAMILY_DIVISIBILITY,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(metadata)
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
            4 => 50_000,
            _ => 65_000,
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
                diagnostic_message: Some("Could not parse answer as a valid number.".to_string()),
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

impl DivisibilityValidator {
    fn classify_misconception(
        _student_val: f64,
        _params: &serde_json::Value,
        expected_val: f64,
    ) -> (ErrorCategory, String) {
        (
            ErrorCategory::Unknown,
            format!("Divisibility rule error: Expected {}.", expected_val),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_divisibility_generation_all_levels() {
        let gen = DivisibilityGenerator;
        for lvl in 1..=5 {
            let inst = gen
                .generate(&ProblemFamilyId::new(FAMILY_DIVISIBILITY), 9876, lvl, None)
                .unwrap();
            assert!(!inst.rendered_prompt.is_empty());
            assert!(inst.correct_answer.get("value").is_some());
        }
    }
}
