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

pub const FAMILY_PROFIT_LOSS: &str = "family.math.arithmetic.profit_loss";
pub const TEMPLATE_PROFIT_LOSS_V1: &str = "math.arithmetic.profit_loss.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProfitLossVariant {
    DirectProfitLoss,
    CalculateCpSp,
    MarkedPriceDiscount,
    SuccessiveDiscounts,
    ChainedTransaction,
}

impl ProfitLossVariant {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProfitLossVariant::DirectProfitLoss => "direct_profit_loss",
            ProfitLossVariant::CalculateCpSp => "calculate_cp_sp",
            ProfitLossVariant::MarkedPriceDiscount => "marked_price_discount",
            ProfitLossVariant::SuccessiveDiscounts => "successive_discounts",
            ProfitLossVariant::ChainedTransaction => "chained_transaction",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ProfitLossGenerator;

impl ProfitLossGenerator {
    const CLEAN_CP_VALUES: &'static [f64] = &[
        50.0, 100.0, 150.0, 200.0, 250.0, 300.0, 400.0, 500.0, 600.0, 800.0, 1000.0, 1200.0,
        1500.0, 2000.0, 2500.0, 3000.0, 5000.0,
    ];

    const CLEAN_PERCENTAGES: &'static [f64] = &[5.0, 10.0, 15.0, 20.0, 25.0, 30.0, 40.0, 50.0];

    pub fn generate_problem(seed: u64, difficulty_level: u32, variant: Option<&str>) -> ProblemInstance {
        let mut rng = StdRng::seed_from_u64(seed);
        let chosen_variant = if let Some(v) = variant {
            match v {
                "direct_profit_loss" => ProfitLossVariant::DirectProfitLoss,
                "calculate_cp_sp" => ProfitLossVariant::CalculateCpSp,
                "marked_price_discount" => ProfitLossVariant::MarkedPriceDiscount,
                "successive_discounts" => ProfitLossVariant::SuccessiveDiscounts,
                "chained_transaction" => ProfitLossVariant::ChainedTransaction,
                _ => ProfitLossVariant::DirectProfitLoss,
            }
        } else {
            match difficulty_level {
                1 => ProfitLossVariant::DirectProfitLoss,
                2 => ProfitLossVariant::CalculateCpSp,
                3 => ProfitLossVariant::MarkedPriceDiscount,
                4 => ProfitLossVariant::SuccessiveDiscounts,
                _ => ProfitLossVariant::ChainedTransaction,
            }
        };

        match chosen_variant {
            ProfitLossVariant::DirectProfitLoss => Self::generate_level_1(&mut rng, seed),
            ProfitLossVariant::CalculateCpSp => Self::generate_level_2(&mut rng, seed),
            ProfitLossVariant::MarkedPriceDiscount => Self::generate_level_3(&mut rng, seed),
            ProfitLossVariant::SuccessiveDiscounts => Self::generate_level_4(&mut rng, seed),
            ProfitLossVariant::ChainedTransaction => Self::generate_level_5(&mut rng, seed),
        }
    }

    /// Level 1: Find Profit or Loss Percentage given Cost Price and Selling Price
    fn generate_level_1(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let cp_idx = rng.random_range(0..Self::CLEAN_CP_VALUES.len());
        let cp = Self::CLEAN_CP_VALUES[cp_idx];

        let pct_idx = rng.random_range(0..Self::CLEAN_PERCENTAGES.len());
        let pct = Self::CLEAN_PERCENTAGES[pct_idx];

        let is_profit = rng.random_bool(0.6);
        let sp = if is_profit {
            cp * (1.0 + pct / 100.0)
        } else {
            cp * (1.0 - pct / 100.0)
        };

        let kind_str = if is_profit { "profit" } else { "loss" };
        let prompt = format!(
            "An article is purchased for a cost price of ${:.0} and sold for ${:.0}.\n\nWhat is the percentage {}?",
            cp, sp, kind_str
        );

        let solution = format!(
            "**Step 1:** Calculate the {} amount:\n\
             ${:.0} - ${:.0} = ${:.0}\n\n\
             **Step 2:** Calculate percentage on Cost Price:\n\
             \\[ \\text{{{kind_str} \\%}} = \\frac{{{:.0}}}{{{:.0}}} \\times 100\\% = **{:.0}%** \\]",
            kind_str,
            if is_profit { sp } else { cp },
            if is_profit { cp } else { sp },
            (sp - cp).abs(),
            (sp - cp).abs(),
            cp,
            pct
        );

        let parameters = serde_json::json!({
            "variant": "direct_profit_loss",
            "cp": cp,
            "sp": sp,
            "is_profit": is_profit,
            "expected_percentage": pct,
        });

        let correct_answer = serde_json::json!({
            "value": pct,
            "formatted": format!("{:.0}%", pct),
            "unit": "%",
            "solution": solution,
        });

        let step1 = crate::problems::steps::StepNode::new(
            "calc_diff",
            crate::problems::steps::StepType::Transformation,
            format!("Calculate {} amount", kind_str),
            format!("|{} - {}| = {}", sp, cp, (sp - cp).abs()),
            format!("{}", (sp - cp).abs()),
        )
        .with_expected_value((sp - cp).abs())
        .with_hints(vec![
            crate::problems::steps::StepHint::principle(format!("Find absolute difference between Selling Price and Cost Price for {}.", kind_str)),
            crate::problems::steps::StepHint::operation(format!("Compute |{} - {}|.", sp, cp)),
            crate::problems::steps::StepHint::intermediate_relation(format!("{} amount = ${:.0}", kind_str, (sp - cp).abs())),
        ]);

        let step2 = crate::problems::steps::StepNode::new(
            "calc_pct",
            crate::problems::steps::StepType::FinalAnswer,
            "Compute percentage on Cost Price",
            format!("({} / {}) * 100 = {}%", (sp - cp).abs(), cp, pct),
            format!("{}%", pct),
        )
        .with_expected_value(pct)
        .with_dependencies(vec!["calc_diff".to_string()])
        .as_final()
        .with_hints(vec![
            crate::problems::steps::StepHint::principle("Divide the profit or loss amount by the Cost Price and multiply by 100%."),
            crate::problems::steps::StepHint::operation(format!("Compute ({:.0} / {:.0}) * 100.", (sp - cp).abs(), cp)),
            crate::problems::steps::StepHint::intermediate_relation(format!("Percentage = {:.0}%", pct)),
        ]);

        let metadata = serde_json::json!({
            "difficulty": 1.0,
            "target_time_ms": 30_000,
            "generator": TEMPLATE_PROFIT_LOSS_V1,
        });

        let graph = crate::problems::steps::SolutionGraph::new(vec![step1, step2], "calc_pct");

        ProblemInstance::new(
            format!("inst-profit-1-{}", seed),
            FAMILY_PROFIT_LOSS,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(metadata)
    }

    /// Level 2: Given Selling Price and Profit/Loss %, find Cost Price
    fn generate_level_2(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let cp_idx = rng.random_range(0..Self::CLEAN_CP_VALUES.len());
        let cp = Self::CLEAN_CP_VALUES[cp_idx];

        let pct_idx = rng.random_range(0..Self::CLEAN_PERCENTAGES.len());
        let pct = Self::CLEAN_PERCENTAGES[pct_idx];

        let is_profit = rng.random_bool(0.5);
        let sp = if is_profit {
            cp * (1.0 + pct / 100.0)
        } else {
            cp * (1.0 - pct / 100.0)
        };

        let kind_str = if is_profit { "profit" } else { "loss" };
        let prompt = format!(
            "By selling an item for ${:.0}, a merchant makes a {:.0}% {}.\n\nWhat was the original cost price?",
            sp, pct, kind_str
        );

        let multiplier = if is_profit { 1.0 + pct / 100.0 } else { 1.0 - pct / 100.0 };
        let solution = format!(
            "**Formula:** \\( \\text{{Cost Price}} = \\frac{{\\text{{Selling Price}}}}{{1 \\pm \\text{{rate}}}} \\)\n\n\
             **Step 1:** Formulate rate multiplier:\n\
             \\( 1 {} {:.2} = {:.2} \\)\n\n\
             **Step 2:** Compute Cost Price:\n\
             \\[ \\text{{CP}} = \\frac{{{:.0}}}{{{:.2}}} = **${:.0}** \\]",
            if is_profit { "+" } else { "-" },
            pct / 100.0,
            multiplier,
            sp,
            multiplier,
            cp
        );

        let parameters = serde_json::json!({
            "variant": "calculate_cp_sp",
            "cp": cp,
            "sp": sp,
            "is_profit": is_profit,
            "pct": pct,
        });

        let correct_answer = serde_json::json!({
            "value": cp,
            "formatted": format!("${:.0}", cp),
            "unit": "$",
            "solution": solution,
        });

        let metadata = serde_json::json!({
            "difficulty": 2.0,
            "target_time_ms": 45_000,
            "generator": TEMPLATE_PROFIT_LOSS_V1,
        });

        let step1 = crate::problems::steps::StepNode::new(
            "rate_multiplier",
            crate::problems::steps::StepType::Transformation,
            "Form rate multiplier",
            format!("1 {} {:.2} = {:.2}", if is_profit { "+" } else { "-" }, pct / 100.0, multiplier),
            format!("{:.2}", multiplier),
        )
        .with_expected_value(multiplier)
        .with_hints(vec![
            crate::problems::steps::StepHint::principle("Selling Price is Cost Price multiplied by (1 ± profit/loss rate)."),
            crate::problems::steps::StepHint::operation(format!("Compute 1 {} ({:.0}/100).", if is_profit { "+" } else { "-" }, pct)),
            crate::problems::steps::StepHint::intermediate_relation(format!("Multiplier = {:.2}", multiplier)),
        ]);

        let step2 = crate::problems::steps::StepNode::new(
            "calc_cp",
            crate::problems::steps::StepType::FinalAnswer,
            "Divide SP by rate multiplier",
            format!("{} / {:.2} = ${:.0}", sp, multiplier, cp),
            format!("${:.0}", cp),
        )
        .with_expected_value(cp)
        .with_dependencies(vec!["rate_multiplier".to_string()])
        .as_final()
        .with_hints(vec![
            crate::problems::steps::StepHint::principle("Cost Price = Selling Price / (1 ± rate)."),
            crate::problems::steps::StepHint::operation(format!("Divide {:.0} by {:.2}.", sp, multiplier)),
            crate::problems::steps::StepHint::intermediate_relation(format!("Cost Price = ${:.0}", cp)),
        ]);

        let graph = crate::problems::steps::SolutionGraph::new(vec![step1, step2], "calc_cp");

        ProblemInstance::new(
            format!("inst-profit-2-{}", seed),
            FAMILY_PROFIT_LOSS,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(metadata)
    }

    /// Level 3: Marked Price, Discount, and Net Profit %
    fn generate_level_3(_rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let cp = 1000.0;
        let markup_pct = 40.0; // Marked at $1400
        let discount_pct = 15.0; // 15% discount on $1400 = $1190
        let mp = cp * (1.0 + markup_pct / 100.0);
        let sp = mp * (1.0 - discount_pct / 100.0);
        let profit_pct = ((sp - cp) / cp) * 100.0; // 19%

        let prompt = format!(
            "A trader marks goods {:.0}% above cost price and allows a discount of {:.0}% on the marked price.\n\nWhat is the overall profit percentage?",
            markup_pct, discount_pct
        );

        let solution = format!(
            "**Step 1:** Assume Cost Price = $100.\n\
             Marked Price = $100 × (1 + 0.40) = $140.\n\n\
             **Step 2:** Apply {:.0}% discount on Marked Price:\n\
             Selling Price = $140 × (1 - 0.15) = $140 × 0.85 = $119.\n\n\
             **Step 3:** Calculate Profit Percentage:\n\
             Profit = $119 - $100 = **{:.0}%**",
            discount_pct, profit_pct
        );

        let parameters = serde_json::json!({
            "variant": "marked_price_discount",
            "cp": cp,
            "markup_pct": markup_pct,
            "discount_pct": discount_pct,
            "profit_pct": profit_pct,
        });

        let correct_answer = serde_json::json!({
            "value": profit_pct,
            "formatted": format!("{:.0}%", profit_pct),
            "unit": "%",
            "solution": solution,
        });

        let metadata = serde_json::json!({
            "difficulty": 3.0,
            "target_time_ms": 55_000,
            "generator": TEMPLATE_PROFIT_LOSS_V1,
        });

        let step1 = crate::problems::steps::StepNode::new(
            "calc_mp",
            crate::problems::steps::StepType::Transformation,
            "Compute Marked Price",
            format!("100 * 1.{:.0} = 140", markup_pct),
            "140".to_string(),
        )
        .with_expected_value(mp);

        let step2 = crate::problems::steps::StepNode::new(
            "calc_sp",
            crate::problems::steps::StepType::IntermediateResult,
            "Compute Selling Price after discount",
            format!("140 * 0.85 = 119"),
            "119".to_string(),
        )
        .with_expected_value(sp)
        .with_dependencies(vec!["calc_mp".to_string()]);

        let step3 = crate::problems::steps::StepNode::new(
            "calc_net_profit",
            crate::problems::steps::StepType::FinalAnswer,
            "Compute profit percentage",
            format!("119 - 100 = {:.0}%", profit_pct),
            format!("{:.0}%", profit_pct),
        )
        .with_expected_value(profit_pct)
        .with_dependencies(vec!["calc_sp".to_string()])
        .as_final();

        let graph = crate::problems::steps::SolutionGraph::new(vec![step1, step2, step3], "calc_net_profit");

        ProblemInstance::new(
            format!("inst-profit-3-{}", seed),
            FAMILY_PROFIT_LOSS,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(metadata)
    }

    /// Level 4: Successive Discounts
    fn generate_level_4(_rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let d1 = 20.0;
        let d2 = 10.0;
        let mp = 500.0;
        let sp = mp * (1.0 - d1 / 100.0) * (1.0 - d2 / 100.0); // 500 * 0.80 * 0.90 = 360
        let single_eq_discount = ((mp - sp) / mp) * 100.0; // 28%

        let prompt = format!(
            "An item with a marked price of ${:.0} is offered with two successive discounts of {:.0}% and {:.0}%.\n\nWhat is the single equivalent discount percentage?",
            mp, d1, d2
        );

        let solution = format!(
            "**Step 1:** Multiply discount factors:\n\
             Factor = (1 - 0.20) × (1 - 0.10) = 0.80 × 0.90 = 0.72\n\n\
             **Step 2:** Convert to single equivalent discount:\n\
             Discount = (1 - 0.72) × 100% = **{:.0}%**",
            single_eq_discount
        );

        let parameters = serde_json::json!({
            "variant": "successive_discounts",
            "mp": mp,
            "d1": d1,
            "d2": d2,
            "single_eq_discount": single_eq_discount,
        });

        let correct_answer = serde_json::json!({
            "value": single_eq_discount,
            "formatted": format!("{:.0}%", single_eq_discount),
            "unit": "%",
            "solution": solution,
        });

        let metadata = serde_json::json!({
            "difficulty": 4.0,
            "target_time_ms": 65_000,
            "generator": TEMPLATE_PROFIT_LOSS_V1,
        });

        let step1 = crate::problems::steps::StepNode::new(
            "mult_factors",
            crate::problems::steps::StepType::Transformation,
            "Multiply discount retention factors",
            "0.80 * 0.90 = 0.72".to_string(),
            "0.72".to_string(),
        )
        .with_expected_value(0.72);

        let step2 = crate::problems::steps::StepNode::new(
            "calc_eq_discount",
            crate::problems::steps::StepType::FinalAnswer,
            "Convert to equivalent discount percentage",
            format!("(1 - 0.72) * 100 = {:.0}%", single_eq_discount),
            format!("{:.0}%", single_eq_discount),
        )
        .with_expected_value(single_eq_discount)
        .with_dependencies(vec!["mult_factors".to_string()])
        .as_final();

        let graph = crate::problems::steps::SolutionGraph::new(vec![step1, step2], "calc_eq_discount");

        ProblemInstance::new(
            format!("inst-profit-4-{}", seed),
            FAMILY_PROFIT_LOSS,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(metadata)
    }

    /// Level 5: Chained Transactions
    fn generate_level_5(_rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let cp_a = 2000.0;
        let profit_a = 20.0; // A sells to B at +20% -> $2400
        let loss_b = 10.0; // B sells to C at -10% -> $2160
        let price_b = cp_a * (1.0 + profit_a / 100.0);
        let price_c = price_b * (1.0 - loss_b / 100.0);

        let prompt = format!(
            "A sells a bicycle to B at a profit of {:.0}%. B sells it to C at a loss of {:.0}%. If C pays ${:.0}, how much did A pay for it initially?",
            profit_a, loss_b, price_c
        );

        let solution = format!(
            "**Formula:** Price_C = Cost_A × (1 + r_A) × (1 - r_B)\n\n\
             **Step 1:** Compute combined factor:\n\
             Factor = 1.20 × 0.90 = 1.08\n\n\
             **Step 2:** Solve for Cost_A:\n\
             Cost_A = ${:.0} / 1.08 = **${:.0}**",
            price_c, cp_a
        );

        let parameters = serde_json::json!({
            "variant": "chained_transaction",
            "cp_a": cp_a,
            "profit_a": profit_a,
            "loss_b": loss_b,
            "price_c": price_c,
        });

        let correct_answer = serde_json::json!({
            "value": cp_a,
            "formatted": format!("${:.0}", cp_a),
            "unit": "$",
            "solution": solution,
        });

        let metadata = serde_json::json!({
            "difficulty": 5.0,
            "target_time_ms": 75_000,
            "generator": TEMPLATE_PROFIT_LOSS_V1,
        });

        let step1 = crate::problems::steps::StepNode::new(
            "combined_factor",
            crate::problems::steps::StepType::Transformation,
            "Multiply chained factors",
            "1.20 * 0.90 = 1.08".to_string(),
            "1.08".to_string(),
        )
        .with_expected_value(1.08);

        let step2 = crate::problems::steps::StepNode::new(
            "solve_initial_cp",
            crate::problems::steps::StepType::FinalAnswer,
            "Divide final price by combined factor",
            format!("{} / 1.08 = ${:.0}", price_c, cp_a),
            format!("${:.0}", cp_a),
        )
        .with_expected_value(cp_a)
        .with_dependencies(vec!["combined_factor".to_string()])
        .as_final();

        let graph = crate::problems::steps::SolutionGraph::new(vec![step1, step2], "solve_initial_cp");

        ProblemInstance::new(
            format!("inst-profit-5-{}", seed),
            FAMILY_PROFIT_LOSS,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(metadata)
    }
}

impl ProblemGenerator for ProfitLossGenerator {
    fn family_id(&self) -> &str {
        FAMILY_PROFIT_LOSS
    }

    fn template_ref(&self) -> &str {
        TEMPLATE_PROFIT_LOSS_V1
    }

    fn supported_variants(&self) -> Vec<String> {
        vec![
            "direct_profit_loss".to_string(),
            "calculate_cp_sp".to_string(),
            "marked_price_discount".to_string(),
            "successive_discounts".to_string(),
            "chained_transaction".to_string(),
        ]
    }

    fn target_latency_ms(&self, difficulty_level: u32) -> u64 {
        match difficulty_level {
            1 => 30_000,
            2 => 45_000,
            3 => 55_000,
            4 => 65_000,
            _ => 75_000,
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

pub struct ProfitLossValidator;

impl ProblemValidator for ProfitLossValidator {
    fn family_id(&self) -> &str {
        FAMILY_PROFIT_LOSS
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

impl ProfitLossValidator {
    fn classify_misconception(
        student_val: f64,
        params: &serde_json::Value,
        expected_val: f64,
    ) -> (ErrorCategory, String) {
        let variant = params.get("variant").and_then(|v| v.as_str()).unwrap_or("");

        // Base confusion in level 1: student computed (SP - CP) / SP instead of / CP
        if variant == "direct_profit_loss" {
            if let (Some(cp), Some(sp)) = (
                params.get("cp").and_then(|v| v.as_f64()),
                params.get("sp").and_then(|v| v.as_f64()),
            ) {
                let sp_base_pct = ((sp - cp).abs() / sp) * 100.0;
                if (student_val - sp_base_pct).abs() <= 0.01 {
                    return (
                        ErrorCategory::Concept,
                        "Percentage base confusion: Profit/Loss % was calculated over Selling Price instead of Cost Price.".to_string(),
                    );
                }
            }
        }

        // Additive discount fallacy in level 4 (e.g. 20% + 10% = 30% instead of 28%)
        if variant == "successive_discounts" {
            if let (Some(d1), Some(d2)) = (
                params.get("d1").and_then(|v| v.as_f64()),
                params.get("d2").and_then(|v| v.as_f64()),
            ) {
                if (student_val - (d1 + d2)).abs() <= 0.01 {
                    return (
                        ErrorCategory::Concept,
                        "Additive fallacy: Successive discounts were added together directly instead of compounded.".to_string(),
                    );
                }
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
    fn test_profit_loss_generation_levels() {
        let gen = ProfitLossGenerator;
        for lvl in 1..=5 {
            let inst = gen
                .generate(&ProblemFamilyId::new(FAMILY_PROFIT_LOSS), 42, lvl, None)
                .unwrap();
            assert!(!inst.rendered_prompt.is_empty());
            assert!(inst.correct_answer.get("value").is_some());
        }
    }

    #[test]
    fn test_profit_loss_base_confusion_diagnostic() {
        let validator = ProfitLossValidator;
        let gen = ProfitLossGenerator;
        let inst = gen
            .generate(&ProblemFamilyId::new(FAMILY_PROFIT_LOSS), 101, 1, Some("direct_profit_loss"))
            .unwrap();

        let cp = inst.parameters.get("cp").unwrap().as_f64().unwrap();
        let sp = inst.parameters.get("sp").unwrap().as_f64().unwrap();
        let wrong_sp_base = ((sp - cp).abs() / sp) * 100.0;

        let eval = validator.evaluate(&inst, &serde_json::json!(wrong_sp_base), 20000, 30000);
        assert!(!eval.is_correct);
        assert_eq!(eval.error_category, Some(ErrorCategory::Concept));
        assert!(eval.diagnostic_message.unwrap().contains("Percentage base confusion"));
    }
}
