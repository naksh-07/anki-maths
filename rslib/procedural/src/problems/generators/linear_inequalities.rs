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

pub const FAMILY_LINEAR_INEQUALITIES: &str = "family.math.algebra.linear_inequalities";
pub const TEMPLATE_LINEAR_INEQUALITIES_V1: &str = "math.algebra.linear_inequalities.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinearInequalitiesVariant {
    OneStep,
    TwoStepPositive,
    NegativeCoefficient,
    VariablesBothSides,
    CompoundInequality,
}

impl LinearInequalitiesVariant {
    pub fn as_str(&self) -> &'static str {
        match self {
            LinearInequalitiesVariant::OneStep => "one_step",
            LinearInequalitiesVariant::TwoStepPositive => "two_step_positive",
            LinearInequalitiesVariant::NegativeCoefficient => "negative_coefficient",
            LinearInequalitiesVariant::VariablesBothSides => "variables_both_sides",
            LinearInequalitiesVariant::CompoundInequality => "compound_inequality",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct LinearInequalitiesGenerator;

impl LinearInequalitiesGenerator {
    pub fn generate_problem(seed: u64, difficulty_level: u32, variant: Option<&str>) -> ProblemInstance {
        let mut rng = StdRng::seed_from_u64(seed);
        let chosen_variant = if let Some(v) = variant {
            match v {
                "one_step" => LinearInequalitiesVariant::OneStep,
                "two_step_positive" => LinearInequalitiesVariant::TwoStepPositive,
                "negative_coefficient" => LinearInequalitiesVariant::NegativeCoefficient,
                "variables_both_sides" => LinearInequalitiesVariant::VariablesBothSides,
                "compound_inequality" => LinearInequalitiesVariant::CompoundInequality,
                _ => LinearInequalitiesVariant::OneStep,
            }
        } else {
            match difficulty_level {
                1 => LinearInequalitiesVariant::OneStep,
                2 => LinearInequalitiesVariant::TwoStepPositive,
                3 => LinearInequalitiesVariant::NegativeCoefficient,
                4 => LinearInequalitiesVariant::VariablesBothSides,
                _ => LinearInequalitiesVariant::CompoundInequality,
            }
        };

        match chosen_variant {
            LinearInequalitiesVariant::OneStep => Self::generate_level_1(&mut rng, seed),
            LinearInequalitiesVariant::TwoStepPositive => Self::generate_level_2(&mut rng, seed),
            LinearInequalitiesVariant::NegativeCoefficient => Self::generate_level_3(&mut rng, seed),
            LinearInequalitiesVariant::VariablesBothSides => Self::generate_level_4(&mut rng, seed),
            LinearInequalitiesVariant::CompoundInequality => Self::generate_level_5(&mut rng, seed),
        }
    }

    /// Level 1: One-step inequality: x + a <= b or ax <= b (a > 0)
    fn generate_level_1(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let a = rng.random_range(2..=8);
        let bound = rng.random_range(-10..=15);
        let c = a * bound;

        let prompt = format!(
            "Solve the linear inequality for \\(x\\):\n\n\\[ {}x \\le {} \\]",
            a, c
        );

        let solution = format!(
            "**Step 1:** Divide both sides by the positive coefficient {}:\n\
             \\[ x \\le \\frac{{{}}}{{{}}} \\implies **x \\le {}** \\]",
            a, c, a, bound
        );

        let parameters = serde_json::json!({
            "variant": "one_step",
            "coefficient": a,
            "constant": c,
            "bound": bound,
            "operator": "<=",
        });

        let correct_answer = serde_json::json!({
            "value": bound as f64,
            "formatted": format!("x <= {}", bound),
            "operator": "<=",
            "solution": solution,
        });

        let step1 = StepNode::new(
            "solve_bound",
            StepType::FinalAnswer,
            "Divide by positive coefficient",
            format!("x <= {} / {} = {}", c, a, bound),
            format!("x <= {}", bound),
        )
        .with_expected_value(bound as f64)
        .with_alternates(vec![
            format!("{}", bound),
            format!("x <= {}", bound),
            format!("x<={}", bound),
        ])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Dividing by a positive number preserves the direction of the inequality."),
            StepHint::operation(format!("Divide {} by {}.", c, a)),
            StepHint::intermediate_relation(format!("x <= {}", bound)),
        ]);

        let graph = SolutionGraph::new(vec![step1], "solve_bound");

        ProblemInstance::new(
            format!("inst-ineq-l1-{}", seed),
            FAMILY_LINEAR_INEQUALITIES,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(serde_json::json!({
            "target_time_ms": 25_000,
            "difficulty_level": 1,
            "variant": "one_step",
            "learning_object_level": "procedural_execution",
        }))
    }

    /// Level 2: Two-step inequality with positive coefficient: ax + b >= c
    fn generate_level_2(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let a: i32 = rng.random_range(2..=7);
        let bound: i32 = rng.random_range(-8..=12);
        let b: i32 = rng.random_range(-15..=15);
        let c: i32 = a * bound + b;

        let b_sign = if b >= 0 { format!("+ {}", b) } else { format!("- {}", b.abs()) };
        let prompt = format!(
            "Solve the linear inequality for \\(x\\):\n\n\\[ {}x {} \\ge {} \\]",
            a, b_sign, c
        );

        let solution = format!(
            "**Step 1:** Subtract {} from both sides:\n\
             \\[ {}x \\ge {} - ({}) = {} \\]\n\n\
             **Step 2:** Divide by {}:\n\
             \\[ x \\ge \\frac{{{}}}{{{}}} \\implies **x \\ge {}** \\]",
            b, a, c, b, c - b, a, c - b, a, bound
        );

        let parameters = serde_json::json!({
            "variant": "two_step_positive",
            "a": a,
            "b": b,
            "c": c,
            "bound": bound,
            "operator": ">=",
        });

        let correct_answer = serde_json::json!({
            "value": bound as f64,
            "formatted": format!("x >= {}", bound),
            "operator": ">=",
            "solution": solution,
        });

        let step1 = StepNode::new(
            "isolate_var_term",
            StepType::EquationRearrangement,
            "Subtract constant",
            format!("{}x >= {} - ({}) = {}", a, c, b, c - b),
            format!("{}x >= {}", a, c - b),
        )
        .with_alternates(vec![format!("{}x >= {}", a, c - b), format!("{}x>={}", a, c - b)])
        .with_hints(vec![
            StepHint::principle("Subtract the constant term from both sides without changing the inequality sign."),
            StepHint::operation(format!("Subtract {} from both sides.", b)),
            StepHint::intermediate_relation(format!("{}x >= {}", a, c - b)),
        ]);

        let step2 = StepNode::new(
            "divide_positive",
            StepType::FinalAnswer,
            "Divide by positive coefficient",
            format!("x >= {} / {} = {}", c - b, a, bound),
            format!("x >= {}", bound),
        )
        .with_expected_value(bound as f64)
        .with_dependencies(vec!["isolate_var_term".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Divide by the coefficient to isolate x."),
            StepHint::operation(format!("Divide both sides by {}.", a)),
            StepHint::intermediate_relation(format!("x >= {}", bound)),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2], "divide_positive");

        ProblemInstance::new(
            format!("inst-ineq-l2-{}", seed),
            FAMILY_LINEAR_INEQUALITIES,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(serde_json::json!({
            "target_time_ms": 30_000,
            "difficulty_level": 2,
            "variant": "two_step_positive",
            "learning_object_level": "procedural_execution",
        }))
    }

    /// Level 3: Negative coefficient requiring sign flip: -ax + b <= c ==> x >= (b-c)/a
    fn generate_level_3(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let a: i32 = rng.random_range(2..=6); // we'll use -a as coefficient
        let bound: i32 = rng.random_range(-8..=10);
        let b: i32 = rng.random_range(-12..=15);
        let c: i32 = -a * bound + b;

        let b_sign = if b >= 0 { format!("+ {}", b) } else { format!("- {}", b.abs()) };
        let prompt = format!(
            "Solve the linear inequality for \\(x\\):\n\n\\[ -{}x {} \\le {} \\]",
            a, b_sign, c
        );

        let solution = format!(
            "**Step 1:** Subtract {} from both sides:\n\
             \\[ -{}x \\le {} - ({}) = {} \\]\n\n\
             **Step 2:** Divide both sides by **-{}** and **REVERSE** the inequality sign:\n\
             \\[ x \\ge \\frac{{{}}}{{-{}}} \\implies **x \\ge {}** \\]",
            b, a, c, b, c - b, a, c - b, a, bound
        );

        let parameters = serde_json::json!({
            "variant": "negative_coefficient",
            "neg_a": -a,
            "b": b,
            "c": c,
            "bound": bound,
            "operator": ">=",
        });

        let correct_answer = serde_json::json!({
            "value": bound as f64,
            "formatted": format!("x >= {}", bound),
            "operator": ">=",
            "solution": solution,
        });

        let step1 = StepNode::new(
            "isolate_neg_term",
            StepType::EquationRearrangement,
            "Subtract constant",
            format!("-{}x <= {} - ({}) = {}", a, c, b, c - b),
            format!("-{}x <= {}", a, c - b),
        )
        .with_alternates(vec![format!("-{}x <= {}", a, c - b), format!("-{}x<={}", a, c - b)])
        .with_hints(vec![
            StepHint::principle("Subtract the constant from both sides first."),
            StepHint::operation(format!("Subtract {} from both sides.", b)),
            StepHint::intermediate_relation(format!("-{}x <= {}", a, c - b)),
        ]);

        let step2 = StepNode::new(
            "divide_negative_flip",
            StepType::FinalAnswer,
            "Divide by negative and reverse inequality sign",
            format!("x >= {} / (-{}) = {}", c - b, a, bound),
            format!("x >= {}", bound),
        )
        .with_expected_value(bound as f64)
        .with_dependencies(vec!["isolate_neg_term".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("CRITICAL RULE: Dividing or multiplying both sides of an inequality by a negative number REVERSES the inequality symbol (<= becomes >=)."),
            StepHint::operation(format!("Divide by -{} and flip <= to >=.", a)),
            StepHint::intermediate_relation(format!("x >= {}", bound)),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2], "divide_negative_flip");

        ProblemInstance::new(
            format!("inst-ineq-l3-{}", seed),
            FAMILY_LINEAR_INEQUALITIES,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(serde_json::json!({
            "target_time_ms": 35_000,
            "difficulty_level": 3,
            "variant": "negative_coefficient",
            "learning_object_level": "variation",
        }))
    }

    /// Level 4: Variables on both sides: ax + b < cx + d
    fn generate_level_4(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let a: i32 = rng.random_range(5..=9);
        let c: i32 = rng.random_range(2..=4); // a > c so net coeff (a - c) > 0
        let bound: i32 = rng.random_range(-6..=8);
        let b: i32 = rng.random_range(-10..=10);
        let d: i32 = (a - c) * bound + b;

        let b_sign = if b >= 0 { format!("+ {}", b) } else { format!("- {}", b.abs()) };
        let d_sign = if d >= 0 { format!("+ {}", d) } else { format!("- {}", d.abs()) };

        let prompt = format!(
            "Solve the linear inequality for \\(x\\):\n\n\\[ {}x {} < {}x {} \\]",
            a, b_sign, c, d_sign
        );

        let solution = format!(
            "**Step 1:** Collect variable terms on the left side:\n\
             \\[ ({} - {})x {} < {} \\implies {}x {} < {} \\]\n\n\
             **Step 2:** Collect constant terms on the right side:\n\
             \\[ {}x < {} - ({}) = {} \\]\n\n\
             **Step 3:** Divide by {}:\n\
             \\[ x < \\frac{{{}}}{{{}}} \\implies **x < {}** \\]",
            a, c, b_sign, d, a - c, b_sign, d, a - c, d, b, d - b, a - c, d - b, a - c, bound
        );

        let parameters = serde_json::json!({
            "variant": "variables_both_sides",
            "a": a,
            "b": b,
            "c": c,
            "d": d,
            "bound": bound,
            "operator": "<",
        });

        let correct_answer = serde_json::json!({
            "value": bound as f64,
            "formatted": format!("x < {}", bound),
            "operator": "<",
            "solution": solution,
        });

        let step1 = StepNode::new(
            "collect_terms",
            StepType::EquationRearrangement,
            "Collect variable terms",
            format!("{}x - {}x < {} - ({}) = {}", a, c, d, b, d - b),
            format!("{}x < {}", a - c, d - b),
        )
        .with_alternates(vec![format!("{}x < {}", a - c, d - b), format!("{}x<{}", a - c, d - b)])
        .with_hints(vec![
            StepHint::principle("Group all variable terms on one side and constant terms on the other side."),
            StepHint::operation(format!("Subtract {}x and subtract {} from both sides.", c, b)),
            StepHint::intermediate_relation(format!("{}x < {}", a - c, d - b)),
        ]);

        let step2 = StepNode::new(
            "solve_x",
            StepType::FinalAnswer,
            "Divide by coefficient",
            format!("x < {} / {} = {}", d - b, a - c, bound),
            format!("x < {}", bound),
        )
        .with_expected_value(bound as f64)
        .with_dependencies(vec!["collect_terms".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Divide by the positive net coefficient."),
            StepHint::operation(format!("Divide {} by {}.", d - b, a - c)),
            StepHint::intermediate_relation(format!("x < {}", bound)),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2], "solve_x");

        ProblemInstance::new(
            format!("inst-ineq-l4-{}", seed),
            FAMILY_LINEAR_INEQUALITIES,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(serde_json::json!({
            "target_time_ms": 40_000,
            "difficulty_level": 4,
            "variant": "variables_both_sides",
            "learning_object_level": "variation",
        }))
    }

    /// Level 5: Compound inequality & integer solution counting: L <= bx + c <= R
    fn generate_level_5(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let b: i32 = rng.random_range(2..=5);
        let c: i32 = rng.random_range(-5..=10);
        let min_x: i32 = rng.random_range(-3..=2);
        let max_x: i32 = min_x + rng.random_range(3..=7);

        let left_bound = b * min_x + c;
        let right_bound = b * max_x + c;
        let integer_count = max_x - min_x + 1;

        let c_sign = if c >= 0 { format!("+ {}", c) } else { format!("- {}", c.abs()) };
        let prompt = format!(
            "How many **integers** \\(x\\) satisfy the double inequality:\n\n\\[ {} \\le {}x {} \\le {} \\]?",
            left_bound, b, c_sign, right_bound
        );

        let solution = format!(
            "**Step 1:** Subtract {} from all three parts:\n\
             \\[ {} - ({}) \\le {}x \\le {} - ({}) \\]\n\
             \\[ {} \\le {}x \\le {} \\]\n\n\
             **Step 2:** Divide all parts by {}:\n\
             \\[ \\frac{{{}}}{{{}}} \\le x \\le \\frac{{{}}}{{{}}} \\implies {} \\le x \\le {} \\]\n\n\
             **Step 3:** Count the number of integers in the closed interval \\([{}, {}]\\):\n\
             \\[ \\text{{Count}} = \\text{{Max}} - \\text{{Min}} + 1 = {} - ({}) + 1 = **{}** \\]",
            c, left_bound, c, b, right_bound, c, left_bound - c, b, right_bound - c,
            b, left_bound - c, b, right_bound - c, b, min_x, max_x, min_x, max_x, max_x, min_x, integer_count
        );

        let parameters = serde_json::json!({
            "variant": "compound_inequality",
            "b": b,
            "c": c,
            "left_bound": left_bound,
            "right_bound": right_bound,
            "min_x": min_x,
            "max_x": max_x,
            "integer_count": integer_count,
        });

        let correct_answer = serde_json::json!({
            "value": integer_count as f64,
            "formatted": format!("{}", integer_count),
            "solution": solution,
        });

        let step1 = StepNode::new(
            "isolate_interval",
            StepType::Transformation,
            "Subtract constant and divide to find interval [min_x, max_x]",
            format!("{} <= x <= {}", min_x, max_x),
            format!("{} <= x <= {}", min_x, max_x),
        )
        .with_alternates(vec![
            format!("[{}, {}]", min_x, max_x),
            format!("{} <= x <= {}", min_x, max_x),
        ])
        .with_hints(vec![
            StepHint::principle("Perform operations on all 3 parts of the double inequality: first subtract c, then divide by b."),
            StepHint::operation(format!("Subtract {} then divide by {}.", c, b)),
            StepHint::intermediate_relation(format!("{} <= x <= {}", min_x, max_x)),
        ]);

        let step2 = StepNode::new(
            "count_integers",
            StepType::FinalAnswer,
            "Count integers in closed interval",
            format!("{} - ({}) + 1 = {}", max_x, min_x, integer_count),
            format!("{}", integer_count),
        )
        .with_expected_value(integer_count as f64)
        .with_dependencies(vec!["isolate_interval".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("The number of integers in [A, B] inclusive is B - A + 1."),
            StepHint::operation(format!("Calculate {} - ({}) + 1.", max_x, min_x)),
            StepHint::intermediate_relation(format!("Count = {}", integer_count)),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2], "count_integers");

        ProblemInstance::new(
            format!("inst-ineq-l5-{}", seed),
            FAMILY_LINEAR_INEQUALITIES,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(serde_json::json!({
            "target_time_ms": 45_000,
            "difficulty_level": 5,
            "variant": "compound_inequality",
            "learning_object_level": "transfer",
        }))
    }
}

impl ProblemGenerator for LinearInequalitiesGenerator {
    fn family_id(&self) -> &str {
        FAMILY_LINEAR_INEQUALITIES
    }

    fn template_ref(&self) -> &str {
        TEMPLATE_LINEAR_INEQUALITIES_V1
    }

    fn supported_variants(&self) -> Vec<String> {
        vec![
            "one_step".to_string(),
            "two_step_positive".to_string(),
            "negative_coefficient".to_string(),
            "variables_both_sides".to_string(),
            "compound_inequality".to_string(),
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

#[derive(Debug, Clone, Default)]
pub struct LinearInequalitiesValidator;

impl ProblemValidator for LinearInequalitiesValidator {
    fn family_id(&self) -> &str {
        FAMILY_LINEAR_INEQUALITIES
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

        let exp_op = instance
            .correct_answer
            .get("operator")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Check string inequality matching (e.g. "x >= 4")
        if let Some(s_str) = student_answer.as_str() {
            let norm_sub = s_str.trim().replace(' ', "");
            let formatted_exp = instance.correct_answer.get("formatted").and_then(|v| v.as_str()).unwrap_or("");
            let norm_exp = formatted_exp.trim().replace(' ', "");

            if norm_sub == norm_exp {
                return AnswerEvaluation::correct(1.0, time_taken_ms, target_time_ms)
                    .with_diagnostic("✓ Correct inequality solution.");
            }

            // Check if student forgot to flip inequality sign on negative division
            if !exp_op.is_empty() {
                let flipped_op = match exp_op {
                    ">=" => "<=",
                    "<=" => ">=",
                    ">" => "<",
                    "<" => ">",
                    _ => "",
                };
                let wrong_flipped_str = format!("x{}{}", flipped_op, expected_val as i64);
                if norm_sub == wrong_flipped_str {
                    return AnswerEvaluation::incorrect(
                        ErrorCategory::Sign,
                        "Inequality sign error: You forgot to reverse the inequality direction when dividing by a negative coefficient.",
                    )
                    .with_parsed_values(expected_val, expected_val);
                }
            }
        }

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
                    .with_diagnostic("✓ Correct inequality boundary value.")
            } else {
                AnswerEvaluation::incorrect(
                    ErrorCategory::Calculation,
                    format!("Calculation error: Expected boundary {}, but received {:.0}.", expected_val, student_num),
                )
                .with_parsed_values(student_num, expected_val)
            }
        } else {
            AnswerEvaluation::incorrect(
                ErrorCategory::Careless,
                "Unable to parse response. Submit as 'x <= 5' or as a boundary number.",
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
    fn test_linear_inequalities_generation_all_levels() {
        let gen = LinearInequalitiesGenerator;
        let validator = LinearInequalitiesValidator;

        for level in 1..=5 {
            let inst = gen.generate(&ProblemFamilyId::new(FAMILY_LINEAR_INEQUALITIES), 42 + level as u64, level, None).unwrap();
            assert!(!inst.rendered_prompt.is_empty(), "Prompt non-empty for L{}", level);

            let graph = inst.solution_graph();
            assert!(graph.is_some(), "SolutionGraph exists for L{}", level);
            assert!(graph.unwrap().validate_topology(), "Topology valid for L{}", level);

            let correct_ans = inst.correct_answer.get("formatted").unwrap();
            let eval = validator.evaluate(&inst, correct_ans, 15000, 30000);
            assert!(eval.is_correct, "Self-eval succeeds for L{}", level);
        }
    }

    #[test]
    fn test_linear_inequalities_sign_flip_diagnostic() {
        let gen = LinearInequalitiesGenerator;
        let validator = LinearInequalitiesValidator;

        let inst = gen.generate(&ProblemFamilyId::new(FAMILY_LINEAR_INEQUALITIES), 100, 3, Some("negative_coefficient")).unwrap();
        let bound = inst.parameters.get("bound").unwrap().as_i64().unwrap();

        // If correct is "x >= bound", submit "x <= bound"
        let wrong_flip = format!("x <= {}", bound);
        let eval = validator.evaluate(&inst, &serde_json::json!(wrong_flip), 20000, 40000);
        assert!(!eval.is_correct);
        assert_eq!(eval.error_category, Some(ErrorCategory::Sign));
        assert!(eval.diagnostic_message.unwrap().contains("reverse the inequality"));
    }
}
