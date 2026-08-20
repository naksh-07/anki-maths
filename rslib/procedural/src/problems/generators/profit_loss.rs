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

const ARTICLE_NAMES: &[&str] = &[
    "laptop", "smartphone", "bicycle", "wrist watch", "camera", "tablet",
    "television", "refrigerator", "microwave", "leather jacket", "sofa set", "desk",
];

impl ProfitLossGenerator {
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

    /// Level 1: Direct Profit or Loss Percentage given Cost Price and Selling Price
    fn generate_level_1(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let item = ARTICLE_NAMES[rng.random_range(0..ARTICLE_NAMES.len())];

        let cp_base = rng.random_range(4..=100) * 25; // $100 to $2500
        let cp = cp_base as f64;

        let pct = (rng.random_range(2..=25) * 2) as f64; // 4% to 50%
        let is_profit = rng.random_bool(0.65);

        let sp = if is_profit {
            cp * (1.0 + pct / 100.0)
        } else {
            cp * (1.0 - pct / 100.0)
        };

        let kind_str = if is_profit { "profit" } else { "loss" };
        let diff_amount = (sp - cp).abs();

        let prompt = format!(
            "A {} is purchased for a cost price of **${:.0}** and sold for **${:.0}**.\n\n\
             What is the percentage **{}** on the cost price?",
            item, cp, sp, kind_str
        );

        let solution = format!(
            "**Step 1:** Calculate the {} amount:\n\
             \\[ |\\text{{SP}} - \\text{{CP}}| = |{:.0} - {:.0}| = \\${:.0} \\]\n\n\
             **Step 2:** Calculate percentage on Cost Price:\n\
             \\[ \\text{{{kind_str} \\%}} = \\frac{{{:.0}}}{{{:.0}}} \\times 100\\% = **{:.0}%** \\]",
            kind_str, sp, cp, diff_amount,
            diff_amount, cp, pct
        );

        let parameters = serde_json::json!({
            "variant": "direct_profit_loss",
            "item": item,
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

        let step1 = StepNode::new(
            "calc_diff",
            StepType::Transformation,
            format!("Calculate {} amount", kind_str),
            format!("|{:.0} - {:.0}| = {:.0}", sp, cp, diff_amount),
            format!("{}", diff_amount),
        )
        .with_expected_value(diff_amount)
        .with_hints(vec![
            StepHint::principle(format!("{} amount = |Selling Price - Cost Price|.", kind_str)),
            StepHint::operation(format!("Compute |{:.0} - {:.0}|.", sp, cp)),
            StepHint::intermediate_relation(format!("{} = ${:.0}", kind_str, diff_amount)),
        ]);

        let step2 = StepNode::new(
            "calc_pct",
            StepType::FinalAnswer,
            "Compute percentage on Cost Price",
            format!("({:.0} / {:.0}) * 100 = {:.0}%", diff_amount, cp, pct),
            format!("{:.0}%", pct),
        )
        .with_expected_value(pct)
        .with_dependencies(vec!["calc_diff".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Percentage = (Difference / Cost Price) * 100."),
            StepHint::operation(format!("Compute ({:.0} / {:.0}) * 100.", diff_amount, cp)),
            StepHint::intermediate_relation(format!("{}%", pct)),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2], "calc_pct");

        ProblemInstance::new(
            format!("inst-profit-1-{}", seed),
            FAMILY_PROFIT_LOSS,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(serde_json::json!({
            "difficulty_level": 1,
            "target_time_ms": 30_000,
            "variant": "direct_profit_loss",
        }))
    }

    /// Level 2: Calculate Cost Price from Selling Price and Profit/Loss %
    fn generate_level_2(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let item = ARTICLE_NAMES[rng.random_range(0..ARTICLE_NAMES.len())];

        let cp_base = rng.random_range(10..=200) * 20; // $200 to $4000
        let cp = cp_base as f64;

        let pct = (rng.random_range(1..=10) * 5) as f64; // 5%, 10%, 15%, ..., 50%
        let is_profit = rng.random_bool(0.6);

        let multiplier = if is_profit { 1.0 + pct / 100.0 } else { 1.0 - pct / 100.0 };
        let sp = cp * multiplier;
        let kind_str = if is_profit { "profit" } else { "loss" };

        let prompt = format!(
            "A {} is sold for **${:.0}**, incurring a **{:.0}% {}** on the cost price.\n\n\
             What was the original **cost price** in dollars?",
            item, sp, pct, kind_str
        );

        let solution = format!(
            "**Step 1:** Formulate the relationship:\n\
             \\[ \\text{{SP}} = \\text{{CP}} \\times (1 {} \\frac{{{:.0}}}{{100}}) = \\text{{CP}} \\times {:.2} \\]\n\n\
             **Step 2:** Solve for Cost Price:\n\
             \\[ \\text{{CP}} = \\frac{{\\text{{SP}}}}{{{:.2}}} = \\frac{{{:.0}}}{{{:.2}}} = **${:.0}** \\]",
            if is_profit { "+" } else { "-" }, pct, multiplier,
            multiplier, sp, multiplier, cp
        );

        let parameters = serde_json::json!({
            "variant": "calculate_cp_sp",
            "item": item,
            "sp": sp,
            "pct": pct,
            "is_profit": is_profit,
            "cp": cp,
        });

        let correct_answer = serde_json::json!({
            "value": cp,
            "formatted": format!("${:.0}", cp),
            "unit": "$",
            "solution": solution,
        });

        let step1 = StepNode::new(
            "rate_multiplier",
            StepType::Transformation,
            "Express Selling Price as CP * (1 ± rate)",
            format!("Multiplier = 1 {} {:.2} = {:.2}", if is_profit { "+" } else { "-" }, pct / 100.0, multiplier),
            format!("{:.2}", multiplier),
        )
        .with_hints(vec![
            StepHint::principle("If sold at p% profit: SP = CP * (1 + p/100). If p% loss: SP = CP * (1 - p/100)."),
            StepHint::operation(format!("Compute 1 {} ({:.0}/100).", if is_profit { "+" } else { "-" }, pct)),
            StepHint::intermediate_relation(format!("Multiplier = {:.2}", multiplier)),
        ]);

        let step2 = StepNode::new(
            "calc_cp",
            StepType::FinalAnswer,
            "Divide SP by rate multiplier",
            format!("{:.0} / {:.2} = ${:.0}", sp, multiplier, cp),
            format!("${:.0}", cp),
        )
        .with_expected_value(cp)
        .with_dependencies(vec!["rate_multiplier".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Cost Price = Selling Price / Multiplier."),
            StepHint::operation(format!("Divide {:.0} by {:.2}.", sp, multiplier)),
            StepHint::intermediate_relation(format!("${:.0}", cp)),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2], "calc_cp");

        ProblemInstance::new(
            format!("inst-profit-2-{}", seed),
            FAMILY_PROFIT_LOSS,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(serde_json::json!({
            "difficulty_level": 2,
            "target_time_ms": 35_000,
            "variant": "calculate_cp_sp",
        }))
    }

    /// Level 3: Marked Price, Discount, and Net Profit % with dynamic parameters.
    fn generate_level_3(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let item = ARTICLE_NAMES[rng.random_range(0..ARTICLE_NAMES.len())];

        let cp = (rng.random_range(5..=50) * 100) as f64; // $500 to $5000
        let markup_pct = (rng.random_range(4..=16) * 5) as f64; // 20% to 80%
        let discount_pct = (rng.random_range(2..=8) * 5) as f64; // 10% to 40%

        let mp = cp * (1.0 + markup_pct / 100.0);
        let sp = mp * (1.0 - discount_pct / 100.0);
        let profit_pct = ((sp - cp) / cp) * 100.0;
        let is_profit = profit_pct >= 0.0;

        let prompt = format!(
            "A merchant marks a {} **{:.0}% above its cost price** and allows a **discount of {:.0}%** on the marked price.\n\n\
             What is the merchant's net percentage **{}**?",
            item, markup_pct, discount_pct,
            if is_profit { "profit" } else { "loss" }
        );

        let solution = format!(
            "**Step 1:** Let Cost Price = \\$100 (or use \\${:.0}).\n\
             \\[ \\text{{Marked Price}} = \\$100 \\times (1 + \\frac{{{:.0}}}{{100}}) = \\${:.0} \\]\n\n\
             **Step 2:** Apply {:.0}% discount on Marked Price:\n\
             \\[ \\text{{Selling Price}} = {:.0} \\times (1 - \\frac{{{:.0}}}{{100}}) = {:.0} \\times {:.2} = \\${:.2} \\]\n\n\
             **Step 3:** Calculate Net Percentage:\n\
             \\[ \\text{{Net \\%}} = {:.2} - 100 = **{:.1}%** \\]",
            cp, markup_pct, 100.0 * (1.0 + markup_pct / 100.0),
            discount_pct,
            100.0 * (1.0 + markup_pct / 100.0), discount_pct, 100.0 * (1.0 + markup_pct / 100.0), 1.0 - discount_pct / 100.0,
            100.0 * (1.0 + markup_pct / 100.0) * (1.0 - discount_pct / 100.0),
            100.0 * (1.0 + markup_pct / 100.0) * (1.0 - discount_pct / 100.0), profit_pct
        );

        let parameters = serde_json::json!({
            "variant": "marked_price_discount",
            "item": item,
            "cp": cp,
            "markup_pct": markup_pct,
            "discount_pct": discount_pct,
            "profit_pct": profit_pct,
        });

        let correct_answer = serde_json::json!({
            "value": profit_pct,
            "formatted": format!("{:.1}%", profit_pct),
            "unit": "%",
            "solution": solution,
        });

        let step1 = StepNode::new(
            "calc_mp",
            StepType::Transformation,
            "Compute Marked Price multiplier",
            format!("1 + {:.0}/100 = {:.2}", markup_pct, 1.0 + markup_pct / 100.0),
            format!("{:.2}", 1.0 + markup_pct / 100.0),
        )
        .with_hints(vec![
            StepHint::principle("Marked Price factor = 1 + (markup / 100)."),
            StepHint::operation(format!("Compute 1 + {:.0}/100.", markup_pct)),
            StepHint::intermediate_relation(format!("MP factor = {:.2}", 1.0 + markup_pct / 100.0)),
        ]);

        let step2 = StepNode::new(
            "calc_sp_factor",
            StepType::IntermediateResult,
            "Apply discount factor",
            format!("{:.2} * (1 - {:.0}/100) = {:.4}", 1.0 + markup_pct / 100.0, discount_pct, (1.0 + markup_pct / 100.0) * (1.0 - discount_pct / 100.0)),
            format!("{:.4}", (1.0 + markup_pct / 100.0) * (1.0 - discount_pct / 100.0)),
        )
        .with_dependencies(vec!["calc_mp".to_string()]);

        let step3 = StepNode::new(
            "calc_net_profit",
            StepType::FinalAnswer,
            "Compute net percentage change",
            format!("({:.4} - 1) * 100 = {:.1}%", (1.0 + markup_pct / 100.0) * (1.0 - discount_pct / 100.0), profit_pct),
            format!("{:.1}%", profit_pct),
        )
        .with_expected_value(profit_pct)
        .with_dependencies(vec!["calc_sp_factor".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Net Profit % = ((SP factor - 1) * 100)."),
            StepHint::operation("Subtract 1 and multiply by 100."),
            StepHint::intermediate_relation(format!("{:.1}%", profit_pct)),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2, step3], "calc_net_profit");

        ProblemInstance::new(
            format!("inst-profit-3-{}", seed),
            FAMILY_PROFIT_LOSS,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(serde_json::json!({
            "difficulty_level": 3,
            "target_time_ms": 40_000,
            "variant": "marked_price_discount",
        }))
    }

    /// Level 4: Successive Discounts (Two or Three discounts)
    fn generate_level_4(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let item = ARTICLE_NAMES[rng.random_range(0..ARTICLE_NAMES.len())];
        let mp = (rng.random_range(4..=40) * 100) as f64; // $400 to $4000

        let d1 = (rng.random_range(2..=8) * 5) as f64; // 10%, 15%, 20%, 25%, 30%, 35%, 40%
        let d2 = (rng.random_range(1..=6) * 5) as f64; // 5%, 10%, 15%, 20%, 25%, 30%

        let factor1 = 1.0 - d1 / 100.0;
        let factor2 = 1.0 - d2 / 100.0;
        let retention_factor = factor1 * factor2;
        let single_eq_discount = (1.0 - retention_factor) * 100.0;

        let prompt = format!(
            "An item ({}) with a marked price of **${:.0}** is offered with two successive discounts of **{:.0}%** and **{:.0}%**.\n\n\
             What is the **single equivalent discount percentage**?",
            item, mp, d1, d2
        );

        let solution = format!(
            "**Step 1:** Calculate the combined retention factor:\n\
             \\[ \\text{{Factor}} = (1 - \\frac{{{:.0}}}{{100}}) \\times (1 - \\frac{{{:.0}}}{{100}}) = {:.2} \\times {:.2} = {:.4} \\]\n\n\
             **Step 2:** Convert retention factor into single equivalent discount:\n\
             \\[ \\text{{Single Discount}} = (1 - {:.4}) \\times 100\\% = **{:.1}%** \\]",
            d1, d2, factor1, factor2, retention_factor,
            retention_factor, single_eq_discount
        );

        let parameters = serde_json::json!({
            "variant": "successive_discounts",
            "item": item,
            "mp": mp,
            "d1": d1,
            "d2": d2,
            "single_eq_discount": single_eq_discount,
        });

        let correct_answer = serde_json::json!({
            "value": single_eq_discount,
            "formatted": format!("{:.1}%", single_eq_discount),
            "unit": "%",
            "solution": solution,
        });

        let step1 = StepNode::new(
            "mult_factors",
            StepType::Transformation,
            "Multiply discount retention factors (1 - d1/100)*(1 - d2/100)",
            format!("{:.2} * {:.2} = {:.4}", factor1, factor2, retention_factor),
            format!("{:.4}", retention_factor),
        )
        .with_expected_value(retention_factor)
        .with_hints(vec![
            StepHint::principle("Each discount d retains (1 - d/100) of the preceding price."),
            StepHint::operation(format!("Compute (1 - {:.0}/100) * (1 - {:.0}/100).", d1, d2)),
            StepHint::intermediate_relation(format!("Retention factor = {:.4}", retention_factor)),
        ]);

        let step2 = StepNode::new(
            "calc_eq_discount",
            StepType::FinalAnswer,
            "Convert retention to equivalent discount percentage",
            format!("(1 - {:.4}) * 100 = {:.1}%", retention_factor, single_eq_discount),
            format!("{:.1}%", single_eq_discount),
        )
        .with_expected_value(single_eq_discount)
        .with_dependencies(vec!["mult_factors".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Single equivalent discount = (1 - Retention Factor) * 100."),
            StepHint::operation(format!("Compute (1 - {:.4}) * 100.", retention_factor)),
            StepHint::intermediate_relation(format!("{:.1}%", single_eq_discount)),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2], "calc_eq_discount");

        ProblemInstance::new(
            format!("inst-profit-4-{}", seed),
            FAMILY_PROFIT_LOSS,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(serde_json::json!({
            "difficulty_level": 4,
            "target_time_ms": 45_000,
            "variant": "successive_discounts",
        }))
    }

    /// Level 5: Chained Transactions / Dishonest Dealer with dynamic parameters
    fn generate_level_5(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let is_dishonest_dealer = rng.random_bool(0.4);

        if is_dishonest_dealer {
            let true_weight = 1000; // 1000 grams
            let false_weight = rng.random_range(16..=19) * 50; // 800g to 950g
            let error = true_weight - false_weight;
            let profit_pct = (error as f64 / false_weight as f64) * 100.0;

            let prompt = format!(
                "A dishonest shopkeeper professes to sell goods at cost price, but uses a false weight of **{} grams** instead of the standard **1 kg (1000 grams)** weight.\n\n\
                 What is the shopkeeper's gain percentage?",
                false_weight
            );

            let solution = format!(
                "**Step 1:** Formula for false weight profit percentage:\n\
                 \\[ \\text{{Gain \\%}} = \\frac{{\\text{{Error}}}}{{\\text{{False Weight}}}} \\times 100\\% \\]\n\n\
                 **Step 2:** Substitute values:\n\
                 \\[ \\text{{Error}} = 1000 - {} = {} \\text{{ g}} \\]\n\
                 \\[ \\text{{Gain \\%}} = \\frac{{{}}}{{{}}} \\times 100\\% = **{:.2}%** \\]",
                false_weight, error,
                error, false_weight, profit_pct
            );

            let parameters = serde_json::json!({
                "variant": "dishonest_dealer",
                "true_weight": true_weight,
                "false_weight": false_weight,
                "error": error,
                "profit_pct": profit_pct,
            });

            let correct_answer = serde_json::json!({
                "value": profit_pct,
                "formatted": format!("{:.2}%", profit_pct),
                "unit": "%",
                "solution": solution,
            });

            let step1 = StepNode::new(
                "calc_weight_error",
                StepType::Transformation,
                "Compute weight error (True Weight - False Weight)",
                format!("1000 - {} = {}", false_weight, error),
                format!("{}", error),
            )
            .with_hints(vec![
                StepHint::principle("Error = Standard True Weight - False Weight given to customer."),
                StepHint::operation(format!("Compute 1000 - {}.", false_weight)),
                StepHint::intermediate_relation(format!("Error = {} g", error)),
            ]);

            let step2 = StepNode::new(
                "calc_gain_pct",
                StepType::FinalAnswer,
                "Divide error by false weight and multiply by 100",
                format!("({} / {}) * 100 = {:.2}%", error, false_weight, profit_pct),
                format!("{:.2}%", profit_pct),
            )
            .with_expected_value(profit_pct)
            .with_dependencies(vec!["calc_weight_error".to_string()])
            .as_final()
            .with_hints(vec![
                StepHint::principle("Gain % = (Error / False Weight) * 100."),
                StepHint::operation(format!("Compute ({} / {}) * 100.", error, false_weight)),
                StepHint::intermediate_relation(format!("{:.2}%", profit_pct)),
            ]);

            let graph = SolutionGraph::new(vec![step1, step2], "calc_gain_pct");

            ProblemInstance::new(
                format!("inst-profit-5-{}", seed),
                FAMILY_PROFIT_LOSS,
                seed,
                parameters,
                prompt,
                correct_answer,
            )
            .with_solution_graph(graph)
            .with_metadata(serde_json::json!({
                "difficulty_level": 5,
                "target_time_ms": 50_000,
                "variant": "dishonest_dealer",
            }))
        } else {
            // Chained transaction: Person A -> Person B -> Person C
            let cp_a = (rng.random_range(10..=60) * 100) as f64; // $1000 to $6000
            let p1 = (rng.random_range(2..=8) * 5) as f64; // 10% to 40% profit
            let p2 = (rng.random_range(1..=5) * 5) as f64; // 5% to 25% loss/profit
            let p2_is_profit = rng.random_bool(0.5);

            let sp_a = cp_a * (1.0 + p1 / 100.0); // Cost to B
            let sp_b = if p2_is_profit {
                sp_a * (1.0 + p2 / 100.0)
            } else {
                sp_a * (1.0 - p2 / 100.0)
            }; // Cost to C

            let prompt = format!(
                "Person A sells an item to Person B at a **profit of {:.0}%**. Person B sells it to Person C at a **{} of {:.0}%** for **${:.2}**.\n\n\
                 How much did Person A originally pay for the item in dollars?",
                p1, if p2_is_profit { "profit" } else { "loss" }, p2, sp_b
            );

            let mult1 = 1.0 + p1 / 100.0;
            let mult2 = if p2_is_profit { 1.0 + p2 / 100.0 } else { 1.0 - p2 / 100.0 };
            let combined_mult = mult1 * mult2;

            let solution = format!(
                "**Step 1:** Express transaction chain algebraically:\n\
                 \\[ \\text{{Price to C}} = \\text{{Cost to A}} \\times ({:.2}) \\times ({:.2}) = \\text{{Cost to A}} \\times {:.4} \\]\n\n\
                 **Step 2:** Solve for Cost to A:\n\
                 \\[ \\text{{Cost to A}} = \\frac{{{:.2}}}{{{:.4}}} = **${:.0}** \\]",
                mult1, mult2, combined_mult,
                sp_b, combined_mult, cp_a
            );

            let parameters = serde_json::json!({
                "variant": "chained_transaction",
                "cp_a": cp_a,
                "p1": p1,
                "p2": p2,
                "p2_is_profit": p2_is_profit,
                "sp_b": sp_b,
            });

            let correct_answer = serde_json::json!({
                "value": cp_a,
                "formatted": format!("${:.0}", cp_a),
                "unit": "$",
                "solution": solution,
            });

            let step1 = StepNode::new(
                "calc_chain_multiplier",
                StepType::Transformation,
                "Multiply sequential profit/loss factors",
                format!("{:.2} * {:.2} = {:.4}", mult1, mult2, combined_mult),
                format!("{:.4}", combined_mult),
            )
            .with_hints(vec![
                StepHint::principle("Sequential transactions multiply their respective price factors: P_final = CP_initial * Factor1 * Factor2."),
                StepHint::operation(format!("Compute {:.2} * {:.2}.", mult1, mult2)),
                StepHint::intermediate_relation(format!("Multiplier = {:.4}", combined_mult)),
            ]);

            let step2 = StepNode::new(
                "calc_initial_cp",
                StepType::FinalAnswer,
                "Divide final price by combined chain multiplier",
                format!("{:.2} / {:.4} = ${:.0}", sp_b, combined_mult, cp_a),
                format!("${:.0}", cp_a),
            )
            .with_expected_value(cp_a)
            .with_dependencies(vec!["calc_chain_multiplier".to_string()])
            .as_final()
            .with_hints(vec![
                StepHint::principle("Initial CP = Final Selling Price / Combined Multiplier."),
                StepHint::operation(format!("Divide {:.2} by {:.4}.", sp_b, combined_mult)),
                StepHint::intermediate_relation(format!("${:.0}", cp_a)),
            ]);

            let graph = SolutionGraph::new(vec![step1, step2], "calc_initial_cp");

            ProblemInstance::new(
                format!("inst-profit-5-{}", seed),
                FAMILY_PROFIT_LOSS,
                seed,
                parameters,
                prompt,
                correct_answer,
            )
            .with_solution_graph(graph)
            .with_metadata(serde_json::json!({
                "difficulty_level": 5,
                "target_time_ms": 55_000,
                "variant": "chained_transaction",
            }))
        }
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
            2 => 35_000,
            3 => 40_000,
            4 => 45_000,
            _ => 55_000,
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
        let is_correct = diff <= 0.1 || (expected_val > 0.0 && diff / expected_val <= 0.01);

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
