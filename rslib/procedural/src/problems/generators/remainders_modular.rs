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

pub const FAMILY_REMAINDERS_MODULAR: &str = "family.math.number_system.remainders_modular";
pub const TEMPLATE_REMAINDERS_MODULAR_V1: &str = "math.number_system.remainders_modular.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RemaindersModularVariant {
    DivisionAlgorithm,
    ExpressionRemainder,
    CyclicityPowers,
    CommonRemainder,
    TransferScheduling,
}

impl RemaindersModularVariant {
    pub fn as_str(&self) -> &'static str {
        match self {
            RemaindersModularVariant::DivisionAlgorithm => "division_algorithm",
            RemaindersModularVariant::ExpressionRemainder => "expression_remainder",
            RemaindersModularVariant::CyclicityPowers => "cyclicity_powers",
            RemaindersModularVariant::CommonRemainder => "common_remainder",
            RemaindersModularVariant::TransferScheduling => "transfer_scheduling",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct RemaindersModularGenerator;

impl RemaindersModularGenerator {
    pub fn generate_problem(seed: u64, difficulty_level: u32, variant: Option<&str>) -> ProblemInstance {
        let mut rng = StdRng::seed_from_u64(seed);
        let chosen_variant = if let Some(v) = variant {
            match v {
                "division_algorithm" => RemaindersModularVariant::DivisionAlgorithm,
                "expression_remainder" => RemaindersModularVariant::ExpressionRemainder,
                "cyclicity_powers" => RemaindersModularVariant::CyclicityPowers,
                "common_remainder" => RemaindersModularVariant::CommonRemainder,
                "transfer_scheduling" => RemaindersModularVariant::TransferScheduling,
                _ => RemaindersModularVariant::DivisionAlgorithm,
            }
        } else {
            match difficulty_level {
                1 => RemaindersModularVariant::DivisionAlgorithm,
                2 => RemaindersModularVariant::ExpressionRemainder,
                3 => RemaindersModularVariant::CyclicityPowers,
                4 => RemaindersModularVariant::CommonRemainder,
                _ => RemaindersModularVariant::TransferScheduling,
            }
        };

        match chosen_variant {
            RemaindersModularVariant::DivisionAlgorithm => Self::generate_level_1(&mut rng, seed),
            RemaindersModularVariant::ExpressionRemainder => Self::generate_level_2(&mut rng, seed),
            RemaindersModularVariant::CyclicityPowers => Self::generate_level_3(&mut rng, seed),
            RemaindersModularVariant::CommonRemainder => Self::generate_level_4(&mut rng, seed),
            RemaindersModularVariant::TransferScheduling => Self::generate_level_5(&mut rng, seed),
        }
    }

    /// Level 1: Division algorithm: Dividend = Divisor * Quotient + Remainder
    fn generate_level_1(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let divisor = rng.random_range(7..=99);
        let quotient = rng.random_range(12..=99);
        let remainder = rng.random_range(1..divisor);
        let dividend = divisor * quotient + remainder;

        let prompt = format!(
            "When a positive integer \\(N\\) is divided by **{}**, the quotient is **{}** and the remainder is **{}**.\n\nFind the value of \\(N\\).",
            divisor, quotient, remainder
        );

        let solution = format!(
            "**Step 1:** Apply Euclidean division algorithm:\n\
             \\[ \\text{{Dividend}} = (\\text{{Divisor}} \\times \\text{{Quotient}}) + \\text{{Remainder}} \\]\n\n\
             **Step 2:** Substitute values:\n\
             \\[ N = ({} \\times {}) + {} = {} + {} = **{}** \\]",
            divisor, quotient, remainder, divisor * quotient, remainder, dividend
        );

        let parameters = serde_json::json!({
            "variant": "division_algorithm",
            "divisor": divisor,
            "quotient": quotient,
            "remainder": remainder,
            "dividend": dividend,
        });

        let correct_answer = serde_json::json!({
            "value": dividend as f64,
            "formatted": format!("{}", dividend),
            "solution": solution,
        });

        let step1 = StepNode::new(
            "calc_product",
            StepType::Arithmetic,
            "Multiply divisor by quotient",
            format!("{} * {} = {}", divisor, quotient, divisor * quotient),
            format!("{}", divisor * quotient),
        )
        .with_expected_value((divisor * quotient) as f64)
        .with_hints(vec![
            StepHint::principle("In the division identity, calculate Divisor * Quotient first."),
            StepHint::operation(format!("Multiply {} by {}.", divisor, quotient)),
            StepHint::intermediate_relation(format!("Product = {}", divisor * quotient)),
        ]);

        let step2 = StepNode::new(
            "calc_dividend",
            StepType::FinalAnswer,
            "Add remainder to get dividend N",
            format!("{} + {} = {}", divisor * quotient, remainder, dividend),
            format!("{}", dividend),
        )
        .with_expected_value(dividend as f64)
        .with_dependencies(vec!["calc_product".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Add the remainder to the product to find the dividend."),
            StepHint::operation(format!("Add {} to {}.", remainder, divisor * quotient)),
            StepHint::intermediate_relation(format!("N = {}", dividend)),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2], "calc_dividend");

        ProblemInstance::new(
            format!("inst-rem-l1-{}", seed),
            FAMILY_REMAINDERS_MODULAR,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(serde_json::json!({
            "target_time_ms": 25_000,
            "difficulty_level": 1,
            "variant": "division_algorithm",
            "learning_object_level": "procedural_execution",
        }))
    }

    /// Level 2: Expression remainder: (A * B) mod M using individual modular arithmetic
    fn generate_level_2(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let m = rng.random_range(7..=25); // Modulus
        let k1 = rng.random_range(10..=50);
        let r1 = rng.random_range(2..m);
        let a = k1 * m + r1;

        let k2 = rng.random_range(10..=50);
        let r2 = rng.random_range(2..m);
        let b = k2 * m + r2;

        let rem_product = (r1 * r2) % m;

        let prompt = format!(
            "Find the remainder when the product \\({} \\times {}\\) is divided by **{}**.",
            a, b, m
        );

        let solution = format!(
            "**Step 1:** Find the individual remainders modulo {}:\n\
             \\[ {} \\equiv {} \\pmod{{{}}} \\]\n\
             \\[ {} \\equiv {} \\pmod{{{}}} \\]\n\n\
             **Step 2:** Multiply the individual remainders:\n\
             \\[ ({} \\times {}) \\pmod{{{}}} = {} \\pmod{{{}}} \\]\n\n\
             **Step 3:** Reduce to final remainder:\n\
             \\[ {} = ({} \\times {}) + **{}** \\]",
            m, a, r1, m, b, r2, m, r1, r2, m, r1 * r2, m, r1 * r2, (r1 * r2) / m, m, rem_product
        );

        let parameters = serde_json::json!({
            "variant": "expression_remainder",
            "a": a,
            "b": b,
            "modulus": m,
            "r1": r1,
            "r2": r2,
            "remainder": rem_product,
        });

        let correct_answer = serde_json::json!({
            "value": rem_product as f64,
            "formatted": format!("{}", rem_product),
            "solution": solution,
        });

        let step1 = StepNode::new(
            "indiv_remainders",
            StepType::Transformation,
            "Find individual remainders",
            format!("{} mod {} = {}, {} mod {} = {}", a, m, r1, b, m, r2),
            format!("{} * {}", r1, r2),
        )
        .with_alternates(vec![format!("{}", r1 * r2)])
        .with_hints(vec![
            StepHint::principle("Modular arithmetic property: (A * B) mod M = ((A mod M) * (B mod M)) mod M."),
            StepHint::operation(format!("Find {} mod {} and {} mod {}.", a, m, b, m)),
            StepHint::intermediate_relation(format!("Remainders are {} and {}, their product is {}", r1, r2, r1 * r2)),
        ]);

        let step2 = StepNode::new(
            "calc_final_remainder",
            StepType::FinalAnswer,
            "Reduce product modulo M",
            format!("{} mod {} = {}", r1 * r2, m, rem_product),
            format!("{}", rem_product),
        )
        .with_expected_value(rem_product as f64)
        .with_dependencies(vec!["indiv_remainders".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Compute the remainder of the product of remainders divided by the modulus."),
            StepHint::operation(format!("Divide {} by {} and take the remainder.", r1 * r2, m)),
            StepHint::intermediate_relation(format!("Remainder = {}", rem_product)),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2], "calc_final_remainder");

        ProblemInstance::new(
            format!("inst-rem-l2-{}", seed),
            FAMILY_REMAINDERS_MODULAR,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(serde_json::json!({
            "target_time_ms": 30_000,
            "difficulty_level": 2,
            "variant": "expression_remainder",
            "learning_object_level": "procedural_execution",
        }))
    }

    /// Level 3: Cyclicity of powers & Unit digits (a^k mod 10)
    fn generate_level_3(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        // Base numbers with period 4 cyclicity modulo 10: ends in 2, 3, 7, 8
        let unit_bases = [2, 3, 7, 8];
        let unit = unit_bases[rng.random_range(0..unit_bases.len())];
        let prefix = rng.random_range(0..=9);
        let base = if prefix == 0 { unit } else { prefix * 10 + unit };
        let exp = rng.random_range(35..=999);

        let rem_exp = exp % 4;
        let effective_exp = if rem_exp == 0 { 4 } else { rem_exp };
        let unit_digit = (base as u64).pow(effective_exp) % 10;

        let prompt = format!(
            "Find the unit digit (remainder when divided by **10**) of the expression:\n\n\\[ {}^{{{}}} \\]",
            base, exp
        );

        let solution = format!(
            "**Step 1:** The powers of {} repeat unit digits in cycles of length 4:\n\
             \\[ {}^{{1}} \\equiv {}, \\quad {}^{{2}} \\equiv {}, \\quad {}^{{3}} \\equiv {}, \\quad {}^{{4}} \\equiv {} \\pmod{{10}} \\]\n\n\
             **Step 2:** Find exponent modulo 4:\n\
             \\[ {} \\pmod{{4}} = {} \\implies \\text{{Effective exponent}} = {} \\]\n\n\
             **Step 3:** Calculate the unit digit:\n\
             \\[ {}^{{{}}} \\equiv **{}** \\pmod{{10}} \\]",
            base, base, base % 10, base, ((base as u64) * (base as u64)) % 10, base, ((base as u64) * (base as u64) * (base as u64)) % 10, base, ((base as u64) * (base as u64) * (base as u64) * (base as u64)) % 10,
            exp, rem_exp, effective_exp, base, effective_exp, unit_digit
        );

        let parameters = serde_json::json!({
            "variant": "cyclicity_powers",
            "base": base,
            "exponent": exp,
            "cycle_length": 4,
            "effective_exp": effective_exp,
            "unit_digit": unit_digit,
        });

        let correct_answer = serde_json::json!({
            "value": unit_digit as f64,
            "formatted": format!("{}", unit_digit),
            "solution": solution,
        });

        let step1 = StepNode::new(
            "exp_cyclicity",
            StepType::Transformation,
            "Find exponent remainder modulo 4",
            format!("{} mod 4 = {} (effective power = {})", exp, rem_exp, effective_exp),
            format!("{}", effective_exp),
        )
        .with_expected_value(effective_exp as f64)
        .with_hints(vec![
            StepHint::principle("The unit digits of 2, 3, 7, 8 have a cyclicity period of 4."),
            StepHint::operation(format!("Divide the exponent {} by 4 to find the position in the cycle.", exp)),
            StepHint::intermediate_relation(format!("{} mod 4 = {} (effective power = {})", exp, rem_exp, effective_exp)),
        ]);

        let step2 = StepNode::new(
            "calc_unit_digit",
            StepType::FinalAnswer,
            "Compute unit digit",
            format!("{}^{} mod 10 = {}", base, effective_exp, unit_digit),
            format!("{}", unit_digit),
        )
        .with_expected_value(unit_digit as f64)
        .with_dependencies(vec!["exp_cyclicity".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle(format!("Compute {}^{} modulo 10.", base, effective_exp)),
            StepHint::operation(format!("Calculate {}^{} mod 10.", base, effective_exp)),
            StepHint::intermediate_relation(format!("Unit digit = {}", unit_digit)),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2], "calc_unit_digit");

        ProblemInstance::new(
            format!("inst-rem-l3-{}", seed),
            FAMILY_REMAINDERS_MODULAR,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(serde_json::json!({
            "target_time_ms": 35_000,
            "difficulty_level": 3,
            "variant": "cyclicity_powers",
            "learning_object_level": "variation",
        }))
    }

    fn gcd(mut a: u32, mut b: u32) -> u32 {
        while b != 0 {
            let t = b;
            b = a % b;
            a = t;
        }
        a
    }

    fn lcm(a: u32, b: u32) -> u32 {
        if a == 0 || b == 0 { return 0; }
        (a / Self::gcd(a, b)) * b
    }

    /// Level 4: Common remainder: Smallest number leaving remainder R for divisors D1, D2, D3
    fn generate_level_4(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let mut d1 = rng.random_range(4..=25);
        let mut d2 = rng.random_range(4..=25);
        let mut d3 = rng.random_range(4..=25);
        
        while d1 == d2 || d2 == d3 || d1 == d3 {
            d1 = rng.random_range(4..=25);
            d2 = rng.random_range(4..=25);
            d3 = rng.random_range(4..=25);
        }
        
        let mut d_vec = vec![d1, d2, d3];
        d_vec.sort_unstable();
        let d1 = d_vec[0];
        let d2 = d_vec[1];
        let d3 = d_vec[2];

        let lcm = Self::lcm(Self::lcm(d1, d2), d3);
        let r = rng.random_range(2..d1); // common remainder (e.g. 5)
        let smallest_n = lcm + r;

        let prompt = format!(
            "Find the smallest positive integer which, when divided by **{}**, **{}**, and **{}**, leaves a remainder of **{}** in each case.",
            d1, d2, d3, r
        );

        let solution = format!(
            "**Step 1:** Find the Lowest Common Multiple (LCM) of the divisors ({}, {}, {}):\n\
             \\[ \\text{{LCM}}({}, {}, {}) = {} \\]\n\n\
             **Step 2:** The general form of such a number is \\( N = \\text{{LCM}} \\times k + R \\).\n\n\
             **Step 3:** For the smallest positive integer (\\(k = 1\\)):\n\
             \\[ N = {} + {} = **{}** \\]",
            d1, d2, d3, d1, d2, d3, lcm, lcm, r, smallest_n
        );

        let parameters = serde_json::json!({
            "variant": "common_remainder",
            "divisors": [d1, d2, d3],
            "lcm": lcm,
            "remainder": r,
            "smallest_n": smallest_n,
        });

        let correct_answer = serde_json::json!({
            "value": smallest_n as f64,
            "formatted": format!("{}", smallest_n),
            "solution": solution,
        });

        let step1 = StepNode::new(
            "calc_lcm",
            StepType::IntermediateResult,
            "Calculate LCM of divisors",
            format!("LCM({}, {}, {}) = {}", d1, d2, d3, lcm),
            format!("{}", lcm),
        )
        .with_expected_value(lcm as f64)
        .with_hints(vec![
            StepHint::principle("Any number leaving a common remainder R must be of the form LCM(d1, d2, d3)*k + R."),
            StepHint::operation(format!("Find the least common multiple of {}, {}, {}.", d1, d2, d3)),
            StepHint::intermediate_relation(format!("LCM = {}", lcm)),
        ]);

        let step2 = StepNode::new(
            "add_remainder",
            StepType::FinalAnswer,
            "Add common remainder to LCM",
            format!("{} + {} = {}", lcm, r, smallest_n),
            format!("{}", smallest_n),
        )
        .with_expected_value(smallest_n as f64)
        .with_dependencies(vec!["calc_lcm".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Add the remainder R to the LCM to obtain the smallest valid number."),
            StepHint::operation(format!("Add {} + {}.", lcm, r)),
            StepHint::intermediate_relation(format!("Smallest number N = {}", smallest_n)),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2], "add_remainder");

        ProblemInstance::new(
            format!("inst-rem-l4-{}", seed),
            FAMILY_REMAINDERS_MODULAR,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(serde_json::json!({
            "target_time_ms": 40_000,
            "difficulty_level": 4,
            "variant": "common_remainder",
            "learning_object_level": "variation",
        }))
    }

    /// Level 5: Transfer scheduling / Recurring calendar modular problem
    fn generate_level_5(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let days = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];
        let start_day_idx = rng.random_range(0..7);
        let start_day = days[start_day_idx];
        let n_days = rng.random_range(45..=9999);
        let rem_days = n_days % 7;
        let target_day_idx = (start_day_idx + rem_days) % 7;
        let target_day = days[target_day_idx];

        let prompt = format!(
            "Today is **{}**.\n\nWhat day of the week will it be exactly **{} days** from today?",
            start_day, n_days
        );

        let solution = format!(
            "**Step 1:** Days of the week cycle every 7 days (modulo 7 arithmetic).\n\n\
             **Step 2:** Find remainder of {} divided by 7:\n\
             \\[ {} \\div 7 = {} \\text{{ weeks remainder }} **{}** \\text{{ days}} \\]\n\n\
             **Step 3:** Advance {} days forward from {}:\n\
             \\[ {} + {} \\text{{ days}} = **{}** \\]",
            n_days, n_days, n_days / 7, rem_days, rem_days, start_day, start_day, rem_days, target_day
        );

        let parameters = serde_json::json!({
            "variant": "transfer_scheduling",
            "start_day": start_day,
            "n_days": n_days,
            "rem_days": rem_days,
            "target_day": target_day,
            "day_index": target_day_idx,
        });

        let correct_answer = serde_json::json!({
            "value": target_day_idx as f64,
            "formatted": target_day,
            "day": target_day,
            "solution": solution,
        });

        let step1 = StepNode::new(
            "calc_day_mod",
            StepType::Transformation,
            "Find offset modulo 7",
            format!("{} mod 7 = {}", n_days, rem_days),
            format!("{}", rem_days),
        )
        .with_expected_value(rem_days as f64)
        .with_hints(vec![
            StepHint::principle("The calendar week repeats every 7 days; compute N modulo 7 to find the forward offset."),
            StepHint::operation(format!("Compute {} mod 7.", n_days)),
            StepHint::intermediate_relation(format!("Offset = {} days", rem_days)),
        ]);

        let step2 = StepNode::new(
            "target_day_result",
            StepType::FinalAnswer,
            "Advance from starting day",
            format!("{} + {} days = {}", start_day, rem_days, target_day),
            target_day,
        )
        .with_alternates(vec![target_day.to_string(), target_day.to_lowercase()])
        .with_dependencies(vec!["calc_day_mod".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Count forward the remainder number of days from the starting day."),
            StepHint::operation(format!("Count {} days forward from {}.", rem_days, start_day)),
            StepHint::intermediate_relation(format!("Result = {}", target_day)),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2], "target_day_result");

        ProblemInstance::new(
            format!("inst-rem-l5-{}", seed),
            FAMILY_REMAINDERS_MODULAR,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(serde_json::json!({
            "target_time_ms": 30_000,
            "difficulty_level": 5,
            "variant": "transfer_scheduling",
            "learning_object_level": "transfer",
        }))
    }
}

impl ProblemGenerator for RemaindersModularGenerator {
    fn family_id(&self) -> &str {
        FAMILY_REMAINDERS_MODULAR
    }

    fn template_ref(&self) -> &str {
        TEMPLATE_REMAINDERS_MODULAR_V1
    }

    fn supported_variants(&self) -> Vec<String> {
        vec![
            "division_algorithm".to_string(),
            "expression_remainder".to_string(),
            "cyclicity_powers".to_string(),
            "common_remainder".to_string(),
            "transfer_scheduling".to_string(),
        ]
    }

    fn target_latency_ms(&self, difficulty_level: u32) -> u64 {
        match difficulty_level {
            1 => 25_000,
            2 => 30_000,
            3 => 35_000,
            4 => 40_000,
            _ => 30_000,
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
pub struct RemaindersModularValidator;

impl ProblemValidator for RemaindersModularValidator {
    fn family_id(&self) -> &str {
        FAMILY_REMAINDERS_MODULAR
    }

    fn evaluate(
        &self,
        instance: &ProblemInstance,
        student_answer: &serde_json::Value,
        time_taken_ms: u64,
        target_time_ms: u64,
    ) -> AnswerEvaluation {
        let formatted_exp = instance
            .correct_answer
            .get("formatted")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Check text match for days of the week (e.g. "Wednesday")
        if let Some(s_str) = student_answer.as_str() {
            let norm_sub = s_str.trim().to_lowercase();
            let norm_exp = formatted_exp.trim().to_lowercase();
            if norm_sub == norm_exp {
                return AnswerEvaluation::correct(1.0, time_taken_ms, target_time_ms)
                    .with_diagnostic("✓ Correct day of the week.");
            }
        }

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
                    .with_diagnostic("✓ Correct remainder / modular calculation.")
            } else {
                AnswerEvaluation::incorrect(
                    ErrorCategory::Calculation,
                    format!("Calculation error: Expected {}, but received {:.0}.", formatted_exp, student_num),
                )
                .with_parsed_values(student_num, expected_val)
            }
        } else {
            AnswerEvaluation::incorrect(
                ErrorCategory::Careless,
                "Unable to parse response. Please submit a valid number or day name.",
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
    fn test_remainders_modular_generation_all_levels() {
        let gen = RemaindersModularGenerator;
        let validator = RemaindersModularValidator;

        for level in 1..=5 {
            let inst = gen.generate(&ProblemFamilyId::new(FAMILY_REMAINDERS_MODULAR), 42 + level as u64, level, None).unwrap();
            assert!(!inst.rendered_prompt.is_empty(), "Prompt non-empty for L{}", level);

            let graph = inst.solution_graph();
            assert!(graph.is_some(), "SolutionGraph exists for L{}", level);
            assert!(graph.unwrap().validate_topology(), "Topology valid for L{}", level);

            let correct_ans = inst.correct_answer.get("formatted").unwrap();
            let eval = validator.evaluate(&inst, correct_ans, 15000, 30000);
            assert!(eval.is_correct, "Self-eval succeeds for L{}", level);
        }
    }
}
