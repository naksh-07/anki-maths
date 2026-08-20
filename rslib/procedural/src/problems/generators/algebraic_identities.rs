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

pub const FAMILY_ALGEBRAIC_IDENTITIES: &str = "family.math.algebra.algebraic_identities";
pub const TEMPLATE_ALGEBRAIC_IDENTITIES_V1: &str = "math.algebra.algebraic_identities.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlgebraicIdentitiesVariant {
    DirectExpansion,
    SumProductEvaluation,
    ReciprocalSquares,
    ReciprocalCubes,
    ConditionalIdentities,
}

impl AlgebraicIdentitiesVariant {
    pub fn as_str(&self) -> &'static str {
        match self {
            AlgebraicIdentitiesVariant::DirectExpansion => "direct_expansion",
            AlgebraicIdentitiesVariant::SumProductEvaluation => "sum_product_evaluation",
            AlgebraicIdentitiesVariant::ReciprocalSquares => "reciprocal_squares",
            AlgebraicIdentitiesVariant::ReciprocalCubes => "reciprocal_cubes",
            AlgebraicIdentitiesVariant::ConditionalIdentities => "conditional_identities",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct AlgebraicIdentitiesGenerator;

impl AlgebraicIdentitiesGenerator {
    pub fn generate_problem(seed: u64, difficulty_level: u32, variant: Option<&str>) -> ProblemInstance {
        let mut rng = StdRng::seed_from_u64(seed);
        let chosen_variant = if let Some(v) = variant {
            match v {
                "direct_expansion" => AlgebraicIdentitiesVariant::DirectExpansion,
                "sum_product_evaluation" => AlgebraicIdentitiesVariant::SumProductEvaluation,
                "reciprocal_squares" => AlgebraicIdentitiesVariant::ReciprocalSquares,
                "reciprocal_cubes" => AlgebraicIdentitiesVariant::ReciprocalCubes,
                "conditional_identities" => AlgebraicIdentitiesVariant::ConditionalIdentities,
                _ => AlgebraicIdentitiesVariant::DirectExpansion,
            }
        } else {
            match difficulty_level {
                1 => AlgebraicIdentitiesVariant::DirectExpansion,
                2 => AlgebraicIdentitiesVariant::SumProductEvaluation,
                3 => AlgebraicIdentitiesVariant::ReciprocalSquares,
                4 => AlgebraicIdentitiesVariant::ReciprocalCubes,
                _ => AlgebraicIdentitiesVariant::ConditionalIdentities,
            }
        };

        match chosen_variant {
            AlgebraicIdentitiesVariant::DirectExpansion => Self::generate_level_1(&mut rng, seed),
            AlgebraicIdentitiesVariant::SumProductEvaluation => Self::generate_level_2(&mut rng, seed),
            AlgebraicIdentitiesVariant::ReciprocalSquares => Self::generate_level_3(&mut rng, seed),
            AlgebraicIdentitiesVariant::ReciprocalCubes => Self::generate_level_4(&mut rng, seed),
            AlgebraicIdentitiesVariant::ConditionalIdentities => Self::generate_level_5(&mut rng, seed),
        }
    }

    /// Level 1: Difference of squares / Square expansions
    fn generate_level_1(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let mode = rng.random_range(0..2);

        if mode == 0 {
            let a = rng.random_range(25..=400);
            let diff_factor = rng.random_range(1..=20) * 2;
            let b = (a - diff_factor).max(1);
            let diff = a - b;
            let sum = a + b;
            let result = diff * sum;

            let prompt = format!(
                "Evaluate using algebraic identities:\n\n\\[ {}^{{2}} - {}^{{2}} \\]",
                a, b
            );

            let solution = format!(
                "**Step 1:** Apply difference of squares identity \\(a^2 - b^2 = (a - b)(a + b)\\):\n\
                 \\[ ({} - {}) \\times ({} + {}) = {} \\times {} = **{}** \\]",
                a, b, a, b, diff, sum, result
            );

            let parameters = serde_json::json!({
                "variant": "direct_expansion",
                "a": a, "b": b, "result": result,
            });

            let correct_answer = serde_json::json!({
                "value": result as f64,
                "formatted": format!("{}", result),
                "solution": solution,
            });

            let step1 = StepNode::new(
                "factor_identity",
                StepType::FormulaSelection,
                "Factor as (a - b)(a + b)",
                format!("({} - {}) * ({} + {}) = {} * {}", a, b, a, b, diff, sum),
                format!("{} * {}", diff, sum),
            )
            .with_expected_value((diff * sum) as f64);

            let step2 = StepNode::new(
                "calc_result",
                StepType::FinalAnswer,
                "Multiply factors",
                format!("{} * {} = {}", diff, sum, result),
                format!("{}", result),
            )
            .with_expected_value(result as f64)
            .with_dependencies(vec!["factor_identity".to_string()])
            .as_final();

            let graph = SolutionGraph::new(vec![step1, step2], "calc_result");

            ProblemInstance::new(
                format!("inst-id-l1-{}", seed),
                FAMILY_ALGEBRAIC_IDENTITIES,
                seed,
                parameters,
                prompt,
                correct_answer,
            )
            .with_solution_graph(graph)
            .with_metadata(serde_json::json!({
                "target_time_ms": 25_000,
                "difficulty_level": 1,
                "variant": "direct_expansion",
            }))
        } else {
            // (a + b)^2 - (a - b)^2 = 4ab
            let a = rng.random_range(10..=150);
            let b = rng.random_range(5..=90);
            let result = 4 * a * b;

            let prompt = format!(
                "Evaluate without manual long arithmetic:\n\n\\[ ({0} + {1})^2 - ({0} - {1})^2 \\]",
                a, b
            );

            let solution = format!(
                "**Step 1:** Use identity \\((a + b)^2 - (a - b)^2 = 4ab\\):\n\
                 \\[ 4 \\times {} \\times {} = **{}** \\]",
                a, b, result
            );

            let parameters = serde_json::json!({
                "variant": "four_ab_identity",
                "a": a, "b": b, "result": result,
            });

            let correct_answer = serde_json::json!({
                "value": result as f64,
                "formatted": format!("{}", result),
                "solution": solution,
            });

            let step1 = StepNode::new(
                "calc_4ab",
                StepType::FinalAnswer,
                "Apply 4ab identity",
                format!("4 * {} * {} = {}", a, b, result),
                format!("{}", result),
            )
            .with_expected_value(result as f64)
            .as_final();

            let graph = SolutionGraph::new(vec![step1], "calc_4ab");

            ProblemInstance::new(
                format!("inst-id-l1-{}", seed),
                FAMILY_ALGEBRAIC_IDENTITIES,
                seed,
                parameters,
                prompt,
                correct_answer,
            )
            .with_solution_graph(graph)
            .with_metadata(serde_json::json!({
                "target_time_ms": 25_000,
                "difficulty_level": 1,
                "variant": "direct_expansion",
            }))
        }
    }

    /// Level 2: Sum & Product evaluation: given a + b and ab, or a - b and ab
    fn generate_level_2(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let is_minus = rng.random_bool(0.4);

        if is_minus {
            let diff = rng.random_range(3..=50);
            let prod = rng.random_range(4..=150);
            let diff_sq = diff * diff;
            let ans = diff_sq + 2 * prod;

            let prompt = format!(
                "If \\(a - b = {}\\) and \\(ab = {}\\), find the value of:\n\n\\[ a^2 + b^2 \\]",
                diff, prod
            );

            let solution = format!(
                "**Step 1:** Use identity \\((a - b)^2 = a^2 + b^2 - 2ab\\):\n\
                 \\[ a^2 + b^2 = (a - b)^2 + 2ab \\]\n\n\
                 **Step 2:** Substitute values:\n\
                 \\[ a^2 + b^2 = ({})^2 + 2({}) = {} + {} = **{}** \\]",
                diff, prod, diff_sq, 2 * prod, ans
            );

            let parameters = serde_json::json!({
                "variant": "diff_product_evaluation",
                "diff": diff, "product": prod, "result": ans,
            });

            let correct_answer = serde_json::json!({
                "value": ans as f64,
                "formatted": format!("{}", ans),
                "solution": solution,
            });

            let step1 = StepNode::new(
                "calc_sum_squares_diff",
                StepType::FinalAnswer,
                "Compute (a - b)^2 + 2ab",
                format!("{}^2 + 2*{} = {}", diff, prod, ans),
                format!("{}", ans),
            )
            .with_expected_value(ans as f64)
            .as_final();

            let graph = SolutionGraph::new(vec![step1], "calc_sum_squares_diff");

            ProblemInstance::new(
                format!("inst-id-l2-{}", seed),
                FAMILY_ALGEBRAIC_IDENTITIES,
                seed,
                parameters,
                prompt,
                correct_answer,
            )
            .with_solution_graph(graph)
            .with_metadata(serde_json::json!({
                "target_time_ms": 30_000,
                "difficulty_level": 2,
                "variant": "sum_product_evaluation",
            }))
        } else {
            let sum = rng.random_range(6..=60);
            let prod = rng.random_range(4..=((sum * sum) / 4).max(5).min(400));
            let sum_sq = sum * sum;
            let ans = sum_sq - 2 * prod;

            let prompt = format!(
                "If \\(a + b = {}\\) and \\(ab = {}\\), find the value of:\n\n\\[ a^2 + b^2 \\]",
                sum, prod
            );

            let solution = format!(
                "**Step 1:** Use identity \\(a^2 + b^2 = (a + b)^2 - 2ab\\):\n\
                 \\[ a^2 + b^2 = ({})^2 - 2({}) = {} - {} = **{}** \\]",
                sum, prod, sum_sq, 2 * prod, ans
            );

            let parameters = serde_json::json!({
                "variant": "sum_product_evaluation",
                "sum": sum, "product": prod, "result": ans,
            });

            let correct_answer = serde_json::json!({
                "value": ans as f64,
                "formatted": format!("{}", ans),
                "solution": solution,
            });

            let step1 = StepNode::new(
                "calc_sum_squares",
                StepType::FinalAnswer,
                "Compute (a + b)^2 - 2ab",
                format!("{}^2 - 2*{} = {}", sum, prod, ans),
                format!("{}", ans),
            )
            .with_expected_value(ans as f64)
            .as_final();

            let graph = SolutionGraph::new(vec![step1], "calc_sum_squares");

            ProblemInstance::new(
                format!("inst-id-l2-{}", seed),
                FAMILY_ALGEBRAIC_IDENTITIES,
                seed,
                parameters,
                prompt,
                correct_answer,
            )
            .with_solution_graph(graph)
            .with_metadata(serde_json::json!({
                "target_time_ms": 30_000,
                "difficulty_level": 2,
                "variant": "sum_product_evaluation",
            }))
        }
    }

    /// Level 3: Reciprocal squares: x ± 1/x = k ==> x^2 + 1/x^2 = k^2 ∓ 2
    fn generate_level_3(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let is_minus = rng.random_bool(0.35);
        let k = rng.random_range(3..=150);

        let ans = if is_minus { k * k + 2 } else { k * k - 2 };

        let prompt = if is_minus {
            format!(
                "If \\(x - \\frac{{1}}{{x}} = {}\\), find the value of:\n\n\\[ x^2 + \\frac{{1}}{{x^2}} \\]",
                k
            )
        } else {
            format!(
                "If \\(x + \\frac{{1}}{{x}} = {}\\), find the value of:\n\n\\[ x^2 + \\frac{{1}}{{x^2}} \\]",
                k
            )
        };

        let solution = if is_minus {
            format!(
                "**Step 1:** Square both sides: \\((x - 1/x)^2 = x^2 + 1/x^2 - 2\\)\n\
                 **Step 2:** \\(x^2 + 1/x^2 = ({})^2 + 2 = {} + 2 = **{}** \\]",
                k, k * k, ans
            )
        } else {
            format!(
                "**Step 1:** Square both sides: \\((x + 1/x)^2 = x^2 + 1/x^2 + 2\\)\n\
                 **Step 2:** \\(x^2 + 1/x^2 = ({})^2 - 2 = {} - 2 = **{}** \\]",
                k, k * k, ans
            )
        };

        let parameters = serde_json::json!({
            "variant": "reciprocal_squares",
            "k": k,
            "is_minus": is_minus,
            "result": ans,
        });

        let correct_answer = serde_json::json!({
            "value": ans as f64,
            "formatted": format!("{}", ans),
            "solution": solution,
        });

        let step1 = StepNode::new(
            "calc_recip_sq",
            StepType::FinalAnswer,
            "Evaluate reciprocal square value",
            format!("{}", ans),
            format!("{}", ans),
        )
        .with_expected_value(ans as f64)
        .as_final();

        let graph = SolutionGraph::new(vec![step1], "calc_recip_sq");

        ProblemInstance::new(
            format!("inst-id-l3-{}", seed),
            FAMILY_ALGEBRAIC_IDENTITIES,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(serde_json::json!({
            "target_time_ms": 30_000,
            "difficulty_level": 3,
            "variant": "reciprocal_squares",
        }))
    }

    /// Level 4: Reciprocal cubes: x ± 1/x = k ==> x^3 ± 1/x^3
    fn generate_level_4(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let is_minus = rng.random_bool(0.35);
        let k = rng.random_range(2..=80);
        let k_cubed = k * k * k;

        let ans = if is_minus {
            k_cubed + 3 * k
        } else {
            k_cubed - 3 * k
        };

        let prompt = if is_minus {
            format!(
                "If \\(x - \\frac{{1}}{{x}} = {}\\), find the value of:\n\n\\[ x^3 - \\frac{{1}}{{x^3}} \\]",
                k
            )
        } else {
            format!(
                "If \\(x + \\frac{{1}}{{x}} = {}\\), find the value of:\n\n\\[ x^3 + \\frac{{1}}{{x^3}} \\]",
                k
            )
        };

        let solution = if is_minus {
            format!(
                "**Step 1:** Use cubic identity \\(x^3 - 1/x^3 = (x - 1/x)^3 + 3(x - 1/x)\\)\n\
                 **Step 2:** \\(({})^3 + 3({}) = {} + {} = **{}** \\]",
                k, k, k_cubed, 3 * k, ans
            )
        } else {
            format!(
                "**Step 1:** Use cubic identity \\(x^3 + 1/x^3 = (x + 1/x)^3 - 3(x + 1/x)\\)\n\
                 **Step 2:** \\(({})^3 - 3({}) = {} - {} = **{}** \\]",
                k, k, k_cubed, 3 * k, ans
            )
        };

        let parameters = serde_json::json!({
            "variant": "reciprocal_cubes",
            "k": k,
            "is_minus": is_minus,
            "result": ans,
        });

        let correct_answer = serde_json::json!({
            "value": ans as f64,
            "formatted": format!("{}", ans),
            "solution": solution,
        });

        let step1 = StepNode::new(
            "calc_recip_cube",
            StepType::FinalAnswer,
            "Evaluate reciprocal cube value",
            format!("{}", ans),
            format!("{}", ans),
        )
        .with_expected_value(ans as f64)
        .as_final();

        let graph = SolutionGraph::new(vec![step1], "calc_recip_cube");

        ProblemInstance::new(
            format!("inst-id-l4-{}", seed),
            FAMILY_ALGEBRAIC_IDENTITIES,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(serde_json::json!({
            "target_time_ms": 35_000,
            "difficulty_level": 4,
            "variant": "reciprocal_cubes",
        }))
    }

    /// Level 5: Conditional identity: if a + b + c = 0 ==> a^3 + b^3 + c^3 = 3abc
    fn generate_level_5(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let a = rng.random_range(10..=150);
        let b = rng.random_range(8..=120);
        let c = -(a + b);
        let result = 3 * a * b * c;

        let prompt = format!(
            "Without expanding directly, find the value of:\n\n\\[ ({})^{{3}} + ({})^{{3}} + ({})^{{3}} \\]",
            a, b, c
        );

        let solution = format!(
            "**Step 1:** Since \\({} + {} + ({}) = 0\\), use identity \\(a^3 + b^3 + c^3 = 3abc\\):\n\
             \\[ 3 \\times ({}) \\times ({}) \\times ({}) = **{}** \\]",
            a, b, c, a, b, c, result
        );

        let parameters = serde_json::json!({
            "variant": "conditional_identities",
            "a": a, "b": b, "c": c, "result": result,
        });

        let correct_answer = serde_json::json!({
            "value": result as f64,
            "formatted": format!("{}", result),
            "solution": solution,
        });

        let step1 = StepNode::new(
            "calc_3abc",
            StepType::FinalAnswer,
            "Compute 3abc",
            format!("3 * {} * {} * ({}) = {}", a, b, c, result),
            format!("{}", result),
        )
        .with_expected_value(result as f64)
        .as_final();

        let graph = SolutionGraph::new(vec![step1], "calc_3abc");

        ProblemInstance::new(
            format!("inst-id-l5-{}", seed),
            FAMILY_ALGEBRAIC_IDENTITIES,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(serde_json::json!({
            "target_time_ms": 40_000,
            "difficulty_level": 5,
            "variant": "conditional_identities",
        }))
    }
}

impl ProblemGenerator for AlgebraicIdentitiesGenerator {
    fn family_id(&self) -> &str {
        FAMILY_ALGEBRAIC_IDENTITIES
    }

    fn template_ref(&self) -> &str {
        TEMPLATE_ALGEBRAIC_IDENTITIES_V1
    }

    fn supported_variants(&self) -> Vec<String> {
        vec![
            "direct_expansion".to_string(),
            "sum_product_evaluation".to_string(),
            "reciprocal_squares".to_string(),
            "reciprocal_cubes".to_string(),
            "conditional_identities".to_string(),
        ]
    }

    fn target_latency_ms(&self, difficulty_level: u32) -> u64 {
        match difficulty_level {
            1 => 25_000,
            2 => 30_000,
            3 => 30_000,
            4 => 35_000,
            _ => 40_000,
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

pub struct AlgebraicIdentitiesValidator;

impl ProblemValidator for AlgebraicIdentitiesValidator {
    fn family_id(&self) -> &str {
        FAMILY_ALGEBRAIC_IDENTITIES
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
