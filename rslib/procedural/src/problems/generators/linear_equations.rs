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

pub const FAMILY_LINEAR_EQUATIONS: &str = "family.math.algebra.linear_equations";
pub const TEMPLATE_LINEAR_EQUATIONS_V1: &str = "math.algebra.linear_equations.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinearEquationVariant {
    TwoStepBasic,
    VariablesBothSides,
    Distributive,
    FractionalCoefficients,
    WordProblem,
}

impl LinearEquationVariant {
    pub fn as_str(&self) -> &'static str {
        match self {
            LinearEquationVariant::TwoStepBasic => "two_step_basic",
            LinearEquationVariant::VariablesBothSides => "variables_both_sides",
            LinearEquationVariant::Distributive => "distributive",
            LinearEquationVariant::FractionalCoefficients => "fractional_coefficients",
            LinearEquationVariant::WordProblem => "word_problem",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct LinearEquationsGenerator;

impl LinearEquationsGenerator {
    pub fn generate_problem(seed: u64, difficulty_level: u32, variant: Option<&str>) -> ProblemInstance {
        let mut rng = StdRng::seed_from_u64(seed);
        let chosen_variant = if let Some(v) = variant {
            match v {
                "two_step_basic" => LinearEquationVariant::TwoStepBasic,
                "variables_both_sides" => LinearEquationVariant::VariablesBothSides,
                "distributive" => LinearEquationVariant::Distributive,
                "fractional_coefficients" => LinearEquationVariant::FractionalCoefficients,
                "word_problem" => LinearEquationVariant::WordProblem,
                _ => LinearEquationVariant::TwoStepBasic,
            }
        } else {
            match difficulty_level {
                1 => LinearEquationVariant::TwoStepBasic,
                2 => LinearEquationVariant::VariablesBothSides,
                3 => LinearEquationVariant::Distributive,
                4 => LinearEquationVariant::FractionalCoefficients,
                _ => LinearEquationVariant::WordProblem,
            }
        };

        match chosen_variant {
            LinearEquationVariant::TwoStepBasic => Self::generate_level_1(&mut rng, seed),
            LinearEquationVariant::VariablesBothSides => Self::generate_level_2(&mut rng, seed),
            LinearEquationVariant::Distributive => Self::generate_level_3(&mut rng, seed),
            LinearEquationVariant::FractionalCoefficients => Self::generate_level_4(&mut rng, seed),
            LinearEquationVariant::WordProblem => Self::generate_level_5(&mut rng, seed),
        }
    }

    /// Level 1: ax + b = c
    fn generate_level_1(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let a: i32 = rng.random_range(2..=9);
        let x: i32 = rng.random_range(-12..=12);
        let b: i32 = rng.random_range(-20..=20);
        let c: i32 = a * x + b;

        let b_sign = if b >= 0 { format!("+ {}", b) } else { format!("- {}", b.abs()) };
        let prompt = format!("Solve for \\(x\\):\n\n\\[ {}x {} = {} \\]", a, b_sign, c);
        let solution = format!(
            "**Step 1:** Subtract {} from both sides:\n\
             \\[ {}x = {} - ({}) = {} \\]\n\n\
             **Step 2:** Divide both sides by {}:\n\
             \\[ x = \\frac{{{}}}{{{}}} = **{}** \\]",
            b, a, c, b, c - b, a, c - b, a, x
        );

        let parameters = serde_json::json!({
            "variant": "two_step_basic",
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
            "isolate_var",
            StepType::EquationRearrangement,
            "Isolate variable term",
            format!("Subtract {} from both sides", b),
            format!("{}x = {}", a, c - b),
        )
        .with_alternates(vec![
            format!("{}x = {} - ({})", a, c, b),
            format!("{}x = {} - {}", a, c, b),
        ])
        .with_hints(vec![
            StepHint::principle("To isolate the variable term, perform the inverse arithmetic operation on both sides."),
            StepHint::operation(format!("Subtract {} from both sides of the equation.", b)),
            StepHint::intermediate_relation(format!("{}x = {} - ({}) = {}", a, c, b, c - b)),
        ]);

        let step2 = StepNode::new(
            "solve_x",
            StepType::FinalAnswer,
            "Solve for x",
            format!("Divide both sides by {}", a),
            format!("x = {}", x),
        )
        .with_expected_value(x as f64)
        .with_alternates(vec![format!("{}", x)])
        .with_dependencies(vec!["isolate_var".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Divide both sides by the coefficient of x to find its value."),
            StepHint::operation(format!("Divide both sides by {}.", a)),
            StepHint::intermediate_relation(format!("x = {} / {} = {}", c - b, a, x)),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2], "solve_x");

        let metadata = serde_json::json!({
            "difficulty": 1.0,
            "target_time_ms": 25_000,
            "generator": TEMPLATE_LINEAR_EQUATIONS_V1,
        });

        ProblemInstance::new(
            format!("inst-linear-{}-{}", 1, seed),
            FAMILY_LINEAR_EQUATIONS,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_metadata(metadata)
        .with_solution_graph(graph)
    }

    /// Level 2: ax + b = cx + d
    fn generate_level_2(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let x: i32 = rng.random_range(-10..=10);
        let a: i32 = rng.random_range(3..=9);
        let mut c: i32 = rng.random_range(1..=8);
        if a == c {
            c = a + 1;
        }
        let b: i32 = rng.random_range(-15..=15);
        let d: i32 = a * x + b - c * x;

        let b_sign = if b >= 0 { format!("+ {}", b) } else { format!("- {}", b.abs()) };
        let d_sign = if d >= 0 { format!("+ {}", d) } else { format!("- {}", d.abs()) };

        let prompt = format!("Solve for \\(x\\):\n\n\\[ {}x {} = {}x {} \\]", a, b_sign, c, d_sign);
        let solution = format!(
            "**Step 1:** Collect variable terms on one side:\n\
             \\[ {}x - {}x = {} - ({}) \\]\n\
             \\[ {}x = {} \\]\n\n\
             **Step 2:** Solve for \\(x\\):\n\
             \\[ x = \\frac{{{}}}{{{}}} = **{}** \\]",
            a, c, d, b, a - c, d - b, d - b, a - c, x
        );

        let parameters = serde_json::json!({
            "variant": "variables_both_sides",
            "a": a,
            "b": b,
            "c": c,
            "d": d,
            "solution_x": x,
        });

        let correct_answer = serde_json::json!({
            "value": x as f64,
            "formatted": format!("{}", x),
            "solution": solution,
        });

        let step1 = StepNode::new(
            "collect_terms",
            StepType::EquationRearrangement,
            "Collect variable terms on one side",
            format!("Subtract {}x and subtract {} from both sides", c, b),
            format!("{}x = {}", a - c, d - b),
        )
        .with_alternates(vec![
            format!("{}x - {}x = {} - ({})", a, c, d, b),
            format!("{}x = {}", a - c, d - b),
        ])
        .with_hints(vec![
            StepHint::principle("Group all variable terms on one side and constant terms on the other side."),
            StepHint::operation(format!("Subtract {}x from both sides and subtract {} from both sides.", c, b)),
            StepHint::intermediate_relation(format!("{}x = {}", a - c, d - b)),
        ]);

        let step2 = StepNode::new(
            "solve_x",
            StepType::FinalAnswer,
            "Solve for x",
            format!("Divide both sides by {}", a - c),
            format!("x = {}", x),
        )
        .with_expected_value(x as f64)
        .with_alternates(vec![format!("{}", x)])
        .with_dependencies(vec!["collect_terms".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Divide the constant by the net coefficient of x."),
            StepHint::operation(format!("Divide {} by {}.", d - b, a - c)),
            StepHint::intermediate_relation(format!("x = {} / {} = {}", d - b, a - c, x)),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2], "solve_x");

        let metadata = serde_json::json!({
            "difficulty": 2.0,
            "target_time_ms": 35_000,
            "generator": TEMPLATE_LINEAR_EQUATIONS_V1,
        });

        ProblemInstance::new(
            format!("inst-linear-{}-{}", 2, seed),
            FAMILY_LINEAR_EQUATIONS,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_metadata(metadata)
        .with_solution_graph(graph)
    }

    /// Level 3: a(x + b) = c(x + d) + e
    fn generate_level_3(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let x: i32 = rng.random_range(-8..=8);
        let a: i32 = rng.random_range(2..=5);
        let c: i32 = rng.random_range(1..=4);
        let b: i32 = rng.random_range(-6..=6);
        let d: i32 = rng.random_range(-6..=6);
        let e: i32 = a * (x + b) - c * (x + d);

        let b_sign = if b >= 0 { format!("+ {}", b) } else { format!("- {}", b.abs()) };
        let d_sign = if d >= 0 { format!("+ {}", d) } else { format!("- {}", d.abs()) };
        let e_sign = if e >= 0 { format!("+ {}", e) } else { format!("- {}", e.abs()) };

        let prompt = format!("Solve for \\(x\\):\n\n\\[ {}(x {}) = {}(x {}) {} \\]", a, b_sign, c, d_sign, e_sign);
        let solution = format!(
            "**Step 1:** Expand both sides:\n\
             \\[ {}x {} = {}x {} {} \\]\n\
             \\[ {}x {} = {}x {} \\]\n\n\
             **Step 2:** Group like terms:\n\
             \\[ {}x - {}x = {} - ({}) \\]\n\
             \\[ {}x = {} \\]\n\n\
             **Step 3:** Solve for \\(x\\):\n\
             \\[ x = **{}** \\]",
            a, a * b, c, c * d, e_sign,
            a, a * b, c, c * d + e,
            a, c, c * d + e, a * b,
            a - c, (c * d + e) - (a * b),
            x
        );

        let parameters = serde_json::json!({
            "variant": "distributive",
            "a": a,
            "b": b,
            "c": c,
            "d": d,
            "e": e,
            "solution_x": x,
        });

        let correct_answer = serde_json::json!({
            "value": x as f64,
            "formatted": format!("{}", x),
            "solution": solution,
        });

        let step1 = StepNode::new(
            "expand_brackets",
            StepType::Simplification,
            "Expand brackets",
            "Apply distributive property on both sides",
            format!("{}x + {} = {}x + {}", a, a * b, c, c * d + e),
        )
        .with_hints(vec![
            StepHint::principle("Use the distributive property: multiply the factor outside by every term inside."),
            StepHint::operation(format!("Multiply {}(x {}) and {}(x {}).", a, b_sign, c, d_sign)),
            StepHint::intermediate_relation(format!("{}x + {} = {}x + {}", a, a * b, c, c * d + e)),
        ]);

        let step2 = StepNode::new(
            "isolate_x_term",
            StepType::EquationRearrangement,
            "Group like terms",
            "Transpose variable terms to left and constants to right",
            format!("{}x = {}", a - c, (c * d + e) - (a * b)),
        )
        .with_dependencies(vec!["expand_brackets".to_string()])
        .with_hints(vec![
            StepHint::principle("Group like variable terms on one side and numerical constants on the other."),
            StepHint::operation(format!("Subtract {}x from both sides and subtract {} from both sides.", c, a * b)),
            StepHint::intermediate_relation(format!("{}x = {}", a - c, (c * d + e) - (a * b))),
        ]);

        let step3 = StepNode::new(
            "solve_x",
            StepType::FinalAnswer,
            "Solve for x",
            "Divide by the coefficient of x",
            format!("x = {}", x),
        )
        .with_expected_value(x as f64)
        .with_alternates(vec![format!("{}", x)])
        .with_dependencies(vec!["isolate_x_term".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Divide both sides by the net coefficient of x."),
            StepHint::operation(format!("Divide {} by {}.", (c * d + e) - (a * b), a - c)),
            StepHint::intermediate_relation(format!("x = {}", x)),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2, step3], "solve_x");

        let metadata = serde_json::json!({
            "difficulty": 3.0,
            "target_time_ms": 50_000,
            "generator": TEMPLATE_LINEAR_EQUATIONS_V1,
        });

        ProblemInstance::new(
            format!("inst-linear-{}-{}", 3, seed),
            FAMILY_LINEAR_EQUATIONS,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_metadata(metadata)
        .with_solution_graph(graph)
    }

    /// Level 4: (x + a)/b + (x + c)/d = e
    fn generate_level_4(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let x: i32 = rng.random_range(1..=12);
        let b: i32 = 2;
        let d: i32 = 3;
        let a: i32 = rng.random_range(1..=6) * 2 - (x % 2); // ensure clean sum
        let c: i32 = rng.random_range(1..=6) * 3 - (x % 3);
        let e: i32 = (x + a) / b + (x + c) / d;

        let a_sign = if a >= 0 { format!("+ {}", a) } else { format!("- {}", a.abs()) };
        let c_sign = if c >= 0 { format!("+ {}", c) } else { format!("- {}", c.abs()) };

        let prompt = format!(
            "Solve for \\(x\\):\n\n\\[ \\frac{{x {}}}{{{}}} + \\frac{{x {}}}{{{}}} = {} \\]",
            a_sign, b, c_sign, d, e
        );

        let solution = format!(
            "**Step 1:** Multiply both sides by the LCM of {} and {} (which is 6):\n\
             \\[ 3(x {}) + 2(x {}) = 6 \\times {} \\]\n\
             \\[ 3x {} + 2x {} = {} \\]\n\n\
             **Step 2:** Combine like terms:\n\
             \\[ 5x {} = {} \\]\n\
             \\[ 5x = {} \\]\n\n\
             **Step 3:** Divide by 5:\n\
             \\[ x = **{}** \\]",
            b, d, a_sign, c_sign, e,
            3 * a, 2 * c, 6 * e,
            3 * a + 2 * c, 6 * e,
            6 * e - (3 * a + 2 * c),
            x
        );

        let parameters = serde_json::json!({
            "variant": "fractional_coefficients",
            "a": a,
            "b": b,
            "c": c,
            "d": d,
            "e": e,
            "solution_x": x,
        });

        let correct_answer = serde_json::json!({
            "value": x as f64,
            "formatted": format!("{}", x),
            "solution": solution,
        });

        let step1 = StepNode::new(
            "eliminate_denominators",
            StepType::Transformation,
            "Eliminate denominators",
            "Multiply entire equation by LCM(2,3) = 6",
            format!("3(x {}) + 2(x {}) = {}", a_sign, c_sign, 6 * e),
        )
        .with_hints(vec![
            StepHint::principle("Clear fractional denominators by multiplying every term by the LCM of the denominators (6)."),
            StepHint::operation("Multiply both sides by 6."),
            StepHint::intermediate_relation(format!("3(x {}) + 2(x {}) = {}", a_sign, c_sign, 6 * e)),
        ]);

        let step2 = StepNode::new(
            "combine_like_terms",
            StepType::Simplification,
            "Combine like terms",
            "Expand and combine terms",
            format!("5x = {}", 6 * e - (3 * a + 2 * c)),
        )
        .with_dependencies(vec!["eliminate_denominators".to_string()])
        .with_hints(vec![
            StepHint::principle("Expand parentheses and group x-terms and constant numbers."),
            StepHint::operation("Combine 3x + 2x = 5x and move constant to right."),
            StepHint::intermediate_relation(format!("5x = {}", 6 * e - (3 * a + 2 * c))),
        ]);

        let step3 = StepNode::new(
            "solve_x",
            StepType::FinalAnswer,
            "Solve for x",
            "Divide by 5",
            format!("x = {}", x),
        )
        .with_expected_value(x as f64)
        .with_alternates(vec![format!("{}", x)])
        .with_dependencies(vec!["combine_like_terms".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Divide by the coefficient 5."),
            StepHint::operation(format!("Divide {} by 5.", 6 * e - (3 * a + 2 * c))),
            StepHint::intermediate_relation(format!("x = {}", x)),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2, step3], "solve_x");

        let metadata = serde_json::json!({
            "difficulty": 4.0,
            "target_time_ms": 65_000,
            "generator": TEMPLATE_LINEAR_EQUATIONS_V1,
        });

        ProblemInstance::new(
            format!("inst-linear-{}-{}", 4, seed),
            FAMILY_LINEAR_EQUATIONS,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_metadata(metadata)
        .with_solution_graph(graph)
    }

    /// Level 5: Word Problem Formulation
    fn generate_level_5(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let x = rng.random_range(4..=25);
        let mult1 = rng.random_range(2..=5);
        let mult2 = mult1 + rng.random_range(1..=3);
        let add1 = rng.random_range(5..=30);
        let sub2 = mult2 * x - (mult1 * x + add1);

        let prompt = format!(
            "If {} times a number increased by {} is equal to {} times the number decreased by {}.\n\nWhat is the number?",
            mult1, add1, mult2, sub2
        );

        let solution = format!(
            "**Step 1:** Set up the equation with unknown \\(x\\):\n\
             \\[ {}x + {} = {}x - {} \\]\n\n\
             **Step 2:** Rearrange to isolate \\(x\\):\n\
             \\[ {} + {} = {}x - {}x \\]\n\
             \\[ {} = {}x \\]\n\n\
             **Step 3:** Solve for \\(x\\):\n\
             \\[ x = \\frac{{{}}}{{{}}} = **{}** \\]",
            mult1, add1, mult2, sub2,
            add1, sub2, mult2, mult1,
            add1 + sub2, mult2 - mult1,
            add1 + sub2, mult2 - mult1,
            x
        );

        let parameters = serde_json::json!({
            "variant": "word_problem",
            "mult1": mult1,
            "add1": add1,
            "mult2": mult2,
            "sub2": sub2,
            "solution_x": x,
        });

        let correct_answer = serde_json::json!({
            "value": x as f64,
            "formatted": format!("{}", x),
            "solution": solution,
        });

        let step1 = StepNode::new(
            "formulate_equation",
            StepType::FormulaSelection,
            "Formulate algebraic equation",
            "Translate word problem into algebraic equation",
            format!("{}x + {} = {}x - {}", mult1, add1, mult2, sub2),
        )
        .with_hints(vec![
            StepHint::principle("Translate words into algebra: 'times' means multiplication, 'increased by' is addition, 'decreased by' is subtraction."),
            StepHint::operation(format!("Let x be the unknown number: write {}x + {} = {}x - {}.", mult1, add1, mult2, sub2)),
            StepHint::intermediate_relation(format!("{}x + {} = {}x - {}", mult1, add1, mult2, sub2)),
        ]);

        let step2 = StepNode::new(
            "isolate_variable",
            StepType::EquationRearrangement,
            "Rearrange to group terms",
            "Group x-terms and constants",
            format!("{}x = {}", mult2 - mult1, add1 + sub2),
        )
        .with_dependencies(vec!["formulate_equation".to_string()])
        .with_hints(vec![
            StepHint::principle("Transpose terms to collect variables on one side."),
            StepHint::operation(format!("Subtract {}x and add {}.", mult1, sub2)),
            StepHint::intermediate_relation(format!("{} = {}x", add1 + sub2, mult2 - mult1)),
        ]);

        let step3 = StepNode::new(
            "solve_x",
            StepType::FinalAnswer,
            "Solve for x",
            "Divide by coefficient",
            format!("x = {}", x),
        )
        .with_expected_value(x as f64)
        .with_alternates(vec![format!("{}", x)])
        .with_dependencies(vec!["isolate_variable".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Divide the total sum by the net difference in multipliers."),
            StepHint::operation(format!("Divide {} by {}.", add1 + sub2, mult2 - mult1)),
            StepHint::intermediate_relation(format!("x = {}", x)),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2, step3], "solve_x");

        let metadata = serde_json::json!({
            "difficulty": 5.0,
            "target_time_ms": 60_000,
            "generator": TEMPLATE_LINEAR_EQUATIONS_V1,
        });

        ProblemInstance::new(
            format!("inst-linear-{}-{}", 5, seed),
            FAMILY_LINEAR_EQUATIONS,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_metadata(metadata)
        .with_solution_graph(graph)
    }
}

impl ProblemGenerator for LinearEquationsGenerator {
    fn family_id(&self) -> &str {
        FAMILY_LINEAR_EQUATIONS
    }

    fn template_ref(&self) -> &str {
        TEMPLATE_LINEAR_EQUATIONS_V1
    }

    fn supported_variants(&self) -> Vec<String> {
        vec![
            "two_step_basic".to_string(),
            "variables_both_sides".to_string(),
            "distributive".to_string(),
            "fractional_coefficients".to_string(),
            "word_problem".to_string(),
        ]
    }

    fn target_latency_ms(&self, difficulty_level: u32) -> u64 {
        match difficulty_level {
            1 => 25_000,
            2 => 35_000,
            3 => 50_000,
            4 => 65_000,
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

/// Deterministic validator and misconception classifier for linear equations.
pub struct LinearEquationsValidator;

impl ProblemValidator for LinearEquationsValidator {
    fn family_id(&self) -> &str {
        FAMILY_LINEAR_EQUATIONS
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

impl LinearEquationsValidator {
    fn classify_misconception(
        student_val: f64,
        params: &serde_json::Value,
        expected_val: f64,
    ) -> (ErrorCategory, String) {
        let variant = params.get("variant").and_then(|v| v.as_str()).unwrap_or("");

        // Check for Transposition Sign Error in level 1 (ax + b = c => ax = c + b instead of c - b)
        if variant == "two_step_basic" {
            if let (Some(a), Some(b), Some(c)) = (
                params.get("a").and_then(|v| v.as_f64()),
                params.get("b").and_then(|v| v.as_f64()),
                params.get("c").and_then(|v| v.as_f64()),
            ) {
                let sign_error_x = (c + b) / a;
                if (student_val - sign_error_x).abs() <= 0.01 {
                    return (
                        ErrorCategory::Strategy,
                        "Transposition sign error: Added constant to right side instead of subtracting (or vice versa).".to_string(),
                    );
                }
            }
        }

        // Check for negation error (e.g. -x instead of x)
        if (student_val + expected_val).abs() <= 0.01 {
            return (
                ErrorCategory::Strategy,
                "Sign error: The magnitude is correct, but the sign is inverted.".to_string(),
            );
        }

        if (student_val - expected_val).abs() < 5.0 {
            (
                ErrorCategory::Calculation,
                format!("Arithmetic slip: Expected {} but received {}.", expected_val, student_val),
            )
        } else {
            (
                ErrorCategory::Unknown,
                format!("Algebra manipulation error: Expected {}.", expected_val),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_equations_generation_all_levels() {
        let gen = LinearEquationsGenerator;
        for level in 1..=5 {
            let inst = gen
                .generate(&ProblemFamilyId::new(FAMILY_LINEAR_EQUATIONS), 12345, level, None)
                .unwrap();
            assert!(!inst.rendered_prompt.is_empty());
            assert!(inst.correct_answer.get("value").is_some());
        }
    }

    #[test]
    fn test_linear_equations_seed_reproducibility() {
        let gen = LinearEquationsGenerator;
        let inst1 = gen
            .generate(&ProblemFamilyId::new(FAMILY_LINEAR_EQUATIONS), 9999, 2, None)
            .unwrap();
        let inst2 = gen
            .generate(&ProblemFamilyId::new(FAMILY_LINEAR_EQUATIONS), 9999, 2, None)
            .unwrap();
        assert_eq!(inst1.rendered_prompt, inst2.rendered_prompt);
        assert_eq!(inst1.correct_answer, inst2.correct_answer);
    }

    #[test]
    fn test_linear_equations_validation_correct_and_sign_error() {
        let validator = LinearEquationsValidator;
        let gen = LinearEquationsGenerator;
        let inst = gen
            .generate(&ProblemFamilyId::new(FAMILY_LINEAR_EQUATIONS), 42, 1, Some("two_step_basic"))
            .unwrap();

        let ans = inst.correct_answer.get("value").unwrap().as_f64().unwrap();

        // Correct answer
        let res = validator.evaluate(&inst, &serde_json::json!(ans), 15000, 25000);
        assert!(res.is_correct);

        // Sign inverted error
        let res_sign = validator.evaluate(&inst, &serde_json::json!(-ans), 15000, 25000);
        assert!(!res_sign.is_correct);
        assert_eq!(res_sign.error_category, Some(ErrorCategory::Strategy));
    }
}
