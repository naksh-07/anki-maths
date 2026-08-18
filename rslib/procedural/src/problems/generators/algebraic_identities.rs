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

    /// Level 1: Direct difference of squares: a^2 - b^2 = (a - b)(a + b)
    fn generate_level_1(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let a = rng.random_range(51..=99);
        let b = a - rng.random_range(1..=5) * 2; // (a - b) is 2, 4, 6, 8, or 10
        let diff = a - b;
        let sum = a + b;
        let result = diff * sum;

        let prompt = format!(
            "Evaluate using algebraic identities:\n\n\\[ {}^{{2}} - {}^{{2}} \\]",
            a, b
        );

        let solution = format!(
            "**Step 1:** Apply the difference of squares identity:\n\
             \\[ a^2 - b^2 = (a - b)(a + b) \\]\n\n\
             **Step 2:** Substitute \\(a = {}\\) and \\(b = {}\\):\n\
             \\[ ({} - {}) \\times ({} + {}) = {} \\times {} = **{}** \\]",
            a, b, a, b, a, b, diff, sum, result
        );

        let parameters = serde_json::json!({
            "variant": "direct_expansion",
            "a": a,
            "b": b,
            "diff": diff,
            "sum": sum,
            "result": result,
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
        .with_expected_value((diff * sum) as f64)
        .with_hints(vec![
            StepHint::principle("Apply the identity: a^2 - b^2 = (a - b)(a + b)."),
            StepHint::operation(format!("Compute ({} - {}) and ({} + {}).", a, b, a, b)),
            StepHint::intermediate_relation(format!("Factors are {} and {}", diff, sum)),
        ]);

        let step2 = StepNode::new(
            "calc_result",
            StepType::FinalAnswer,
            "Multiply factors",
            format!("{} * {} = {}", diff, sum, result),
            format!("{}", result),
        )
        .with_expected_value(result as f64)
        .with_dependencies(vec!["factor_identity".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Multiply the two simplified factors."),
            StepHint::operation(format!("Multiply {} * {}.", diff, sum)),
            StepHint::intermediate_relation(format!("Result = {}", result)),
        ]);

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
            "learning_object_level": "procedural_execution",
        }))
    }

    /// Level 2: Sum & Product evaluation: given a + b and ab, find a^2 + b^2 = (a+b)^2 - 2ab
    fn generate_level_2(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let sum = rng.random_range(7..=15);
        let prod = rng.random_range(6..=((sum * sum) / 4)); // ensure real solutions
        let sum_sq = sum * sum;
        let ans = sum_sq - 2 * prod;

        let prompt = format!(
            "If \\(a + b = {}\\) and \\(ab = {}\\), find the value of:\n\n\\[ a^2 + b^2 \\]",
            sum, prod
        );

        let solution = format!(
            "**Step 1:** Use the square identity \\((a + b)^2 = a^2 + b^2 + 2ab\\):\n\
             \\[ a^2 + b^2 = (a + b)^2 - 2ab \\]\n\n\
             **Step 2:** Substitute the known values:\n\
             \\[ a^2 + b^2 = ({})^2 - 2({}) = {} - {} = **{}** \\]",
            sum, prod, sum_sq, 2 * prod, ans
        );

        let parameters = serde_json::json!({
            "variant": "sum_product_evaluation",
            "sum": sum,
            "product": prod,
            "result": ans,
        });

        let correct_answer = serde_json::json!({
            "value": ans as f64,
            "formatted": format!("{}", ans),
            "solution": solution,
        });

        let step1 = StepNode::new(
            "identity_formula",
            StepType::Transformation,
            "Rearrange square identity",
            format!("a^2 + b^2 = {}^2 - 2*{} = {} - {}", sum, prod, sum_sq, 2 * prod),
            format!("{}^2 - 2*{}", sum, prod),
        )
        .with_expected_value(ans as f64)
        .with_hints(vec![
            StepHint::principle("Rearrange (a + b)^2 = a^2 + 2ab + b^2 into a^2 + b^2 = (a + b)^2 - 2ab."),
            StepHint::operation(format!("Compute ({})^2 - 2 * {}.", sum, prod)),
            StepHint::intermediate_relation(format!("{} - {}", sum_sq, 2 * prod)),
        ]);

        let step2 = StepNode::new(
            "calc_ans",
            StepType::FinalAnswer,
            "Subtract 2ab from sum squared",
            format!("{} - {} = {}", sum_sq, 2 * prod, ans),
            format!("{}", ans),
        )
        .with_expected_value(ans as f64)
        .with_dependencies(vec!["identity_formula".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Subtract the product term from the squared sum."),
            StepHint::operation(format!("Subtract {} from {}.", 2 * prod, sum_sq)),
            StepHint::intermediate_relation(format!("a^2 + b^2 = {}", ans)),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2], "calc_ans");

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
            "learning_object_level": "procedural_execution",
        }))
    }

    /// Level 3: Reciprocal squares: x + 1/x = k ==> x^2 + 1/x^2 = k^2 - 2
    fn generate_level_3(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let k = rng.random_range(3..=9);
        let ans = k * k - 2;

        let prompt = format!(
            "If \\(x + \\frac{{1}}{{x}} = {}\\), find the value of:\n\n\\[ x^2 + \\frac{{1}}{{x^2}} \\]",
            k
        );

        let solution = format!(
            "**Step 1:** Square both sides of the equation:\n\
             \\[ \\left(x + \\frac{{1}}{{x}}\\right)^2 = x^2 + \\frac{{1}}{{x^2}} + 2 \\cdot x \\cdot \\frac{{1}}{{x}} = x^2 + \\frac{{1}}{{x^2}} + 2 \\]\n\n\
             **Step 2:** Substitute \\(x + 1/x = {}\\):\n\
             \\[ ({})^2 = x^2 + \\frac{{1}}{{x^2}} + 2 \\implies {} = x^2 + \\frac{{1}}{{x^2}} + 2 \\]\n\n\
             **Step 3:** Subtract 2 from both sides:\n\
             \\[ x^2 + \\frac{{1}}{{x^2}} = {} - 2 = **{}** \\]",
            k, k, k * k, k * k, ans
        );

        let parameters = serde_json::json!({
            "variant": "reciprocal_squares",
            "k": k,
            "result": ans,
        });

        let correct_answer = serde_json::json!({
            "value": ans as f64,
            "formatted": format!("{}", ans),
            "solution": solution,
        });

        let step1 = StepNode::new(
            "square_identity",
            StepType::Transformation,
            "Apply reciprocal square formula k^2 - 2",
            format!("{}^2 - 2 = {} - 2 = {}", k, k * k, ans),
            format!("{}^2 - 2", k),
        )
        .with_expected_value(ans as f64)
        .with_hints(vec![
            StepHint::principle("Identity: If x + 1/x = k, then x^2 + 1/x^2 = k^2 - 2."),
            StepHint::operation(format!("Square {} and subtract 2.", k)),
            StepHint::intermediate_relation(format!("{} - 2 = {}", k * k, ans)),
        ]);

        let step2 = StepNode::new(
            "final_value",
            StepType::FinalAnswer,
            "Evaluate final value",
            format!("{}", ans),
            format!("{}", ans),
        )
        .with_expected_value(ans as f64)
        .with_dependencies(vec!["square_identity".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Complete the subtraction."),
            StepHint::operation(format!("Subtract 2 from {}.", k * k)),
            StepHint::intermediate_relation(format!("Value = {}", ans)),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2], "final_value");

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
            "learning_object_level": "variation",
        }))
    }

    /// Level 4: Reciprocal cubes: x + 1/x = k ==> x^3 + 1/x^3 = k^3 - 3k
    fn generate_level_4(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let k = rng.random_range(3..=6);
        let k_cubed = k * k * k;
        let ans = k_cubed - 3 * k;

        let prompt = format!(
            "If \\(x + \\frac{{1}}{{x}} = {}\\), find the value of:\n\n\\[ x^3 + \\frac{{1}}{{x^3}} \\]",
            k
        );

        let solution = format!(
            "**Step 1:** Cube both sides using the binomial identity:\n\
             \\[ \\left(x + \\frac{{1}}{{x}}\\right)^3 = x^3 + \\frac{{1}}{{x^3}} + 3 \\cdot x \\cdot \\frac{{1}}{{x}} \\cdot \\left(x + \\frac{{1}}{{x}}\\right) \\]\n\
             \\[ \\left(x + \\frac{{1}}{{x}}\\right)^3 = x^3 + \\frac{{1}}{{x^3}} + 3 \\cdot \\left(x + \\frac{{1}}{{x}}\\right) \\]\n\n\
             **Step 2:** Substitute \\(x + 1/x = {}\\):\n\
             \\[ ({})^3 = x^3 + \\frac{{1}}{{x^3}} + 3({}) \\implies {} = x^3 + \\frac{{1}}{{x^3}} + {} \\]\n\n\
             **Step 3:** Rearrange to solve:\n\
             \\[ x^3 + \\frac{{1}}{{x^3}} = {} - {} = **{}** \\]",
            k, k, k, k_cubed, 3 * k, k_cubed, 3 * k, ans
        );

        let parameters = serde_json::json!({
            "variant": "reciprocal_cubes",
            "k": k,
            "k_cubed": k_cubed,
            "result": ans,
        });

        let correct_answer = serde_json::json!({
            "value": ans as f64,
            "formatted": format!("{}", ans),
            "solution": solution,
        });

        let step1 = StepNode::new(
            "cube_identity",
            StepType::Transformation,
            "Apply reciprocal cube formula k^3 - 3k",
            format!("{}^3 - 3*{} = {} - {}", k, k, k_cubed, 3 * k),
            format!("{}^3 - 3*{}", k, k),
        )
        .with_expected_value(ans as f64)
        .with_hints(vec![
            StepHint::principle("Identity: If x + 1/x = k, then x^3 + 1/x^3 = k^3 - 3k."),
            StepHint::operation(format!("Compute ({})^3 - 3 * {}.", k, k)),
            StepHint::intermediate_relation(format!("{} - {}", k_cubed, 3 * k)),
        ]);

        let step2 = StepNode::new(
            "calc_cube_value",
            StepType::FinalAnswer,
            "Subtract 3k from k cubed",
            format!("{} - {} = {}", k_cubed, 3 * k, ans),
            format!("{}", ans),
        )
        .with_expected_value(ans as f64)
        .with_dependencies(vec!["cube_identity".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Complete the arithmetic subtraction."),
            StepHint::operation(format!("Subtract {} from {}.", 3 * k, k_cubed)),
            StepHint::intermediate_relation(format!("x^3 + 1/x^3 = {}", ans)),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2], "calc_cube_value");

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
            "learning_object_level": "variation",
        }))
    }

    /// Level 5: Conditional identity: if a + b + c = 0 ==> a^3 + b^3 + c^3 = 3abc
    fn generate_level_5(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let a = rng.random_range(12..=35);
        let b = rng.random_range(10..=25);
        let c = -(a + b); // so a + b + c = 0
        let result = 3 * a * b * c;

        let prompt = format!(
            "Without expanding directly, find the value of:\n\n\\[ ({})^{{3}} + ({})^{{3}} + ({})^{{3}} \\]",
            a, b, c
        );

        let solution = format!(
            "**Step 1:** Check the conditional property \\(a + b + c\\):\n\
             \\[ {} + {} + ({}) = 0 \\]\n\n\
             **Step 2:** By the three-variable cubic identity, if \\(a + b + c = 0\\), then:\n\
             \\[ a^3 + b^3 + c^3 = 3abc \\]\n\n\
             **Step 3:** Calculate \\(3abc\\):\n\
             \\[ 3 \\times ({}) \\times ({}) \\times ({}) = **{}** \\]",
            a, b, c, a, b, c, result
        );

        let parameters = serde_json::json!({
            "variant": "conditional_identities",
            "a": a,
            "b": b,
            "c": c,
            "result": result,
        });

        let correct_answer = serde_json::json!({
            "value": result as f64,
            "formatted": format!("{}", result),
            "solution": solution,
        });

        let step1 = StepNode::new(
            "check_condition",
            StepType::Transformation,
            "Verify sum equals zero and apply identity 3abc",
            format!("3 * {} * {} * ({}) = {}", a, b, c, result),
            format!("3 * {} * {} * {}", a, b, c),
        )
        .with_expected_value(result as f64)
        .with_hints(vec![
            StepHint::principle("Fundamental algebraic identity: If a + b + c = 0, then a^3 + b^3 + c^3 = 3abc."),
            StepHint::operation(format!("Compute 3 * {} * {} * ({}).", a, b, c)),
            StepHint::intermediate_relation(format!("Product = {}", result)),
        ]);

        let step2 = StepNode::new(
            "calc_product",
            StepType::FinalAnswer,
            "Multiply 3 * a * b * c",
            format!("{}", result),
            format!("{}", result),
        )
        .with_expected_value(result as f64)
        .with_dependencies(vec!["check_condition".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Compute the signed product."),
            StepHint::operation(format!("Multiply 3 * {} * {} * {}.", a, b, c)),
            StepHint::intermediate_relation(format!("Final value = {}", result)),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2], "calc_product");

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
            "learning_object_level": "transfer",
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

#[derive(Debug, Clone, Default)]
pub struct AlgebraicIdentitiesValidator;

impl ProblemValidator for AlgebraicIdentitiesValidator {
    fn family_id(&self) -> &str {
        FAMILY_ALGEBRAIC_IDENTITIES
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
            let is_correct = diff <= 0.01;

            if is_correct {
                let score = if target_time_ms > 0 && time_taken_ms > target_time_ms {
                    0.85
                } else {
                    1.0
                };
                AnswerEvaluation::correct(score, time_taken_ms, target_time_ms)
                    .with_parsed_values(student_num, expected_val)
                    .with_diagnostic("✓ Correct algebraic identity evaluation.")
            } else {
                // Check if student forgot cross term 2ab or 2 in reciprocal identity (e.g. k^2 instead of k^2 - 2)
                let k = instance.parameters.get("k").and_then(|v| v.as_f64()).unwrap_or(0.0);
                if k > 0.0 && (student_num - (k * k)).abs() <= 0.01 {
                    return AnswerEvaluation::incorrect(
                        ErrorCategory::Concept,
                        "Missing cross-term: (x + 1/x)^2 = x^2 + 1/x^2 + 2, so x^2 + 1/x^2 = k^2 - 2 (you forgot to subtract 2).",
                    )
                    .with_parsed_values(student_num, expected_val);
                }

                AnswerEvaluation::incorrect(
                    ErrorCategory::Calculation,
                    format!("Calculation error: Expected {:.0}, but received {:.0}.", expected_val, student_num),
                )
                .with_parsed_values(student_num, expected_val)
            }
        } else {
            AnswerEvaluation::incorrect(
                ErrorCategory::Careless,
                "Unable to parse response. Please submit a valid integer or numerical value.",
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
    fn test_algebraic_identities_generation_all_levels() {
        let gen = AlgebraicIdentitiesGenerator;
        let validator = AlgebraicIdentitiesValidator;

        for level in 1..=5 {
            let inst = gen.generate(&ProblemFamilyId::new(FAMILY_ALGEBRAIC_IDENTITIES), 42 + level as u64, level, None).unwrap();
            assert!(!inst.rendered_prompt.is_empty(), "Prompt non-empty for L{}", level);

            let graph = inst.solution_graph();
            assert!(graph.is_some(), "SolutionGraph exists for L{}", level);
            assert!(graph.unwrap().validate_topology(), "Topology valid for L{}", level);

            let correct_ans = inst.correct_answer.get("value").unwrap();
            let eval = validator.evaluate(&inst, correct_ans, 15000, 30000);
            assert!(eval.is_correct, "Self-eval succeeds for L{}", level);
        }
    }

    #[test]
    fn test_algebraic_identities_missing_cross_term_diagnostic() {
        let gen = AlgebraicIdentitiesGenerator;
        let validator = AlgebraicIdentitiesValidator;

        let inst = gen.generate(&ProblemFamilyId::new(FAMILY_ALGEBRAIC_IDENTITIES), 100, 3, Some("reciprocal_squares")).unwrap();
        let k = inst.parameters.get("k").unwrap().as_f64().unwrap();

        // Submit k^2 without subtracting 2
        let eval = validator.evaluate(&inst, &serde_json::json!(k * k), 20000, 40000);
        assert!(!eval.is_correct);
        assert_eq!(eval.error_category, Some(ErrorCategory::Concept));
        assert!(eval.diagnostic_message.unwrap().contains("Missing cross-term"));
    }
}
