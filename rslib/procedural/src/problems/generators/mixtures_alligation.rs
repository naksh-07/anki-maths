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

pub const FAMILY_MIXTURES_ALLIGATION: &str = "family.math.arithmetic.mixtures_alligation";
pub const TEMPLATE_MIXTURES_ALLIGATION_V1: &str = "math.arithmetic.mixtures_alligation.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MixturesAlligationVariant {
    TwoComponentBlend,
    AlligationRatio,
    DilutionAddition,
    RepeatedReplacement,
    TransferCommercial,
}

impl MixturesAlligationVariant {
    pub fn as_str(&self) -> &'static str {
        match self {
            MixturesAlligationVariant::TwoComponentBlend => "two_component_blend",
            MixturesAlligationVariant::AlligationRatio => "alligation_ratio",
            MixturesAlligationVariant::DilutionAddition => "dilution_addition",
            MixturesAlligationVariant::RepeatedReplacement => "repeated_replacement",
            MixturesAlligationVariant::TransferCommercial => "transfer_commercial",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct MixturesAlligationGenerator;

impl MixturesAlligationGenerator {
    pub fn generate_problem(seed: u64, difficulty_level: u32, variant: Option<&str>) -> ProblemInstance {
        let mut rng = StdRng::seed_from_u64(seed);
        let chosen_variant = if let Some(v) = variant {
            match v {
                "two_component_blend" => MixturesAlligationVariant::TwoComponentBlend,
                "alligation_ratio" => MixturesAlligationVariant::AlligationRatio,
                "dilution_addition" => MixturesAlligationVariant::DilutionAddition,
                "repeated_replacement" => MixturesAlligationVariant::RepeatedReplacement,
                "transfer_commercial" => MixturesAlligationVariant::TransferCommercial,
                _ => MixturesAlligationVariant::TwoComponentBlend,
            }
        } else {
            match difficulty_level {
                1 => MixturesAlligationVariant::TwoComponentBlend,
                2 => MixturesAlligationVariant::AlligationRatio,
                3 => MixturesAlligationVariant::DilutionAddition,
                4 => MixturesAlligationVariant::RepeatedReplacement,
                _ => MixturesAlligationVariant::TransferCommercial,
            }
        };

        match chosen_variant {
            MixturesAlligationVariant::TwoComponentBlend => Self::generate_level_1(&mut rng, seed),
            MixturesAlligationVariant::AlligationRatio => Self::generate_level_2(&mut rng, seed),
            MixturesAlligationVariant::DilutionAddition => Self::generate_level_3(&mut rng, seed),
            MixturesAlligationVariant::RepeatedReplacement => Self::generate_level_4(&mut rng, seed),
            MixturesAlligationVariant::TransferCommercial => Self::generate_level_5(&mut rng, seed),
        }
    }

    /// Level 1: Two component blend -> find mean cost/concentration
    fn generate_level_1(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let items = ["tea", "coffee", "sugar", "rice", "flour", "pulses", "grains"];
        let item = items[rng.random_range(0..items.len())];

        let q1 = rng.random_range(2..=20) * 5; // e.g. 10 to 100 kg
        let q2 = rng.random_range(2..=20) * 5; // e.g. 10 to 100 kg
        let p1 = rng.random_range(20..=80);   // $/kg
        let p2 = p1 + rng.random_range(10..=45); // $/kg
        let total_weight = q1 + q2;
        let total_cost = q1 * p1 + q2 * p2;
        let mean_price = total_cost as f64 / total_weight as f64;
        let rounded_mean = (mean_price * 10.0).round() / 10.0;

        let prompt = format!(
            "A merchant mixes **{} kg** of {} priced at **${}/kg** with **{} kg** of {} priced at **${}/kg**.\n\nFind the average price per kilogram of the resulting mixture in dollars.",
            q1, item, p1, q2, item, p2
        );

        let solution = format!(
            "**Step 1:** Calculate total cost of both varieties:\n\
             \\[ \\text{{Total Cost}} = ({} \\times {}) + ({} \\times {}) = {} + {} = \\${} \\]\n\n\
             **Step 2:** Calculate total quantity:\n\
             \\[ \\text{{Total Quantity}} = {} + {} = {} \\text{{ kg}} \\]\n\n\
             **Step 3:** Calculate average price per kg:\n\
             \\[ \\text{{Mean Price}} = \\frac{{{}}}{{{}}} = **{:.1}** \\text{{ \\$/kg}} \\]",
            q1, p1, q2, p2, q1 * p1, q2 * p2, total_cost, q1, q2, total_weight, total_cost, total_weight, rounded_mean
        );

        let parameters = serde_json::json!({
            "variant": "two_component_blend",
            "item": item,
            "q1": q1,
            "q2": q2,
            "p1": p1,
            "p2": p2,
            "mean_price": rounded_mean,
        });

        let correct_answer = serde_json::json!({
            "value": rounded_mean,
            "formatted": format!("{:.1}", rounded_mean),
            "unit": "$/kg",
            "solution": solution,
        });

        let step1 = StepNode::new(
            "total_cost",
            StepType::IntermediateResult,
            "Calculate total cost",
            format!("({} * {}) + ({} * {}) = {}", q1, p1, q2, p2, total_cost),
            format!("{}", total_cost),
        )
        .with_expected_value(total_cost as f64)
        .with_hints(vec![
            StepHint::principle("Total cost = Sum of individual costs: (Q1 * P1) + (Q2 * P2)."),
            StepHint::operation(format!("Multiply and add: ({} * {}) + ({} * {}).", q1, p1, q2, p2)),
            StepHint::intermediate_relation(format!("Total cost = ${}", total_cost)),
        ]);

        let step2 = StepNode::new(
            "mean_price",
            StepType::FinalAnswer,
            "Calculate mean price per kg",
            format!("{} / {} = {:.1}", total_cost, total_weight, rounded_mean),
            format!("{:.1}", rounded_mean),
        )
        .with_expected_value(rounded_mean)
        .with_dependencies(vec!["total_cost".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Mean price = Total Cost / Total Quantity."),
            StepHint::operation(format!("Divide {} by {}.", total_cost, total_weight)),
            StepHint::intermediate_relation(format!("Mean price = ${:.1}/kg", rounded_mean)),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2], "mean_price");

        ProblemInstance::new(
            format!("inst-mix-l1-{}", seed),
            FAMILY_MIXTURES_ALLIGATION,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(serde_json::json!({
            "target_time_ms": 30_000,
            "difficulty_level": 1,
            "variant": "two_component_blend",
            "learning_object_level": "procedural_execution",
        }))
    }

    /// Level 2: Rule of Alligation -> Find ratio to mix two prices to achieve target mean price
    fn generate_level_2(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let items = ["pulses", "tea", "coffee", "sugar", "rice", "wheat", "salt"];
        let item = items[rng.random_range(0..items.len())];

        let p_cheap = rng.random_range(10..=100); // Cheaper price
        let p_dear = p_cheap + rng.random_range(10..=80); // Dearer price
        // Target mean price strictly between cheap and dear
        let offset = rng.random_range(2..=(p_dear - p_cheap - 1));
        let p_mean = p_cheap + offset;

        let diff_cheap = p_mean - p_cheap; // Part of dearer variety
        let diff_dear = p_dear - p_mean;   // Part of cheaper variety

        // Reduce ratio (diff_dear : diff_cheap)
        fn gcd(a: u32, b: u32) -> u32 {
            if b == 0 { a } else { gcd(b, a % b) }
        }
        let g = gcd(diff_dear, diff_cheap);
        let ratio_cheap = diff_dear / g;
        let ratio_dear = diff_cheap / g;

        let prompt = format!(
            "In what ratio must a grocer mix two varieties of {} costing **${}/kg** and **${}/kg** so that the mixture costs **${}/kg**?",
            item, p_cheap, p_dear, p_mean
        );

        let solution = format!(
            "**Step 1:** Apply the Rule of Alligation:\n\
             \\[ \\frac{{\\text{{Quantity of Cheaper}}}}{{\\text{{Quantity of Dearer}}}} = \\frac{{\\text{{Price of Dearer}} - \\text{{Mean Price}}}}{{\\text{{Mean Price}} - \\text{{Price of Cheaper}}}} \\]\n\n\
             **Step 2:** Compute differences:\n\
             \\[ \\text{{Numerator (Cheaper ratio)}} = {} - {} = {} \\]\n\
             \\[ \\text{{Denominator (Dearer ratio)}} = {} - {} = {} \\]\n\n\
             **Step 3:** Simplify the ratio:\n\
             \\[ {} : {} = **{}:{}** \\]",
            p_dear, p_mean, diff_dear, p_mean, p_cheap, diff_cheap, diff_dear, diff_cheap, ratio_cheap, ratio_dear
        );

        let parameters = serde_json::json!({
            "variant": "alligation_ratio",
            "item": item,
            "p_cheap": p_cheap,
            "p_dear": p_dear,
            "p_mean": p_mean,
            "ratio_cheap": ratio_cheap,
            "ratio_dear": ratio_dear,
        });

        let correct_answer = serde_json::json!({
            "value": ratio_cheap as f64 / ratio_dear as f64,
            "formatted": format!("{}:{}", ratio_cheap, ratio_dear),
            "ratio": [ratio_cheap, ratio_dear],
            "solution": solution,
        });

        let step1 = StepNode::new(
            "alligation_setup",
            StepType::FormulaSelection,
            "Calculate alligation cross differences",
            format!("(Dearer - Mean) : (Mean - Cheaper) = ({} - {}) : ({} - {}) = {} : {}", p_dear, p_mean, p_mean, p_cheap, diff_dear, diff_cheap),
            format!("{}:{}", diff_dear, diff_cheap),
        )
        .with_alternates(vec![
            format!("{}:{}", ratio_cheap, ratio_dear),
            format!("{}/{}", diff_dear, diff_cheap),
        ])
        .with_hints(vec![
            StepHint::principle("Rule of Alligation: Ratio of (Cheaper : Dearer) = (Dearer Price - Mean Price) : (Mean Price - Cheaper Price)."),
            StepHint::operation(format!("Compute ({} - {}) : ({} - {}).", p_dear, p_mean, p_mean, p_cheap)),
            StepHint::intermediate_relation(format!("Unsimplified ratio = {} : {}", diff_dear, diff_cheap)),
        ]);

        let step2 = StepNode::new(
            "simplify_ratio",
            StepType::FinalAnswer,
            "Simplify ratio",
            format!("Simplify {} : {} by dividing by GCD {} = {}:{}", diff_dear, diff_cheap, g, ratio_cheap, ratio_dear),
            format!("{}:{}", ratio_cheap, ratio_dear),
        )
        .with_alternates(vec![
            format!("{}:{}", ratio_cheap, ratio_dear),
            format!("{}/{}", ratio_cheap, ratio_dear),
        ])
        .with_dependencies(vec!["alligation_setup".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Divide both terms of the ratio by their greatest common divisor."),
            StepHint::operation(format!("Divide {} and {} by {}.", diff_dear, diff_cheap, g)),
            StepHint::intermediate_relation(format!("Simplified ratio = {}:{}", ratio_cheap, ratio_dear)),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2], "simplify_ratio");

        ProblemInstance::new(
            format!("inst-mix-l2-{}", seed),
            FAMILY_MIXTURES_ALLIGATION,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(serde_json::json!({
            "target_time_ms": 35_000,
            "difficulty_level": 2,
            "variant": "alligation_ratio",
            "learning_object_level": "procedural_execution",
        }))
    }

    /// Level 3: Dilution / Pure substance addition to adjust concentration
    fn generate_level_3(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let liquids = [("alcohol", "pure alcohol"), ("acid", "pure acid"), ("antifreeze", "pure antifreeze"), ("juice", "pure juice extract")];
        let (solute, pure_solute) = liquids[rng.random_range(0..liquids.len())];

        let initial_volume = rng.random_range(10..=150) * 2; // 20 to 300 liters
        let c1 = rng.random_range(5..=45); // Initial %
        let target_c = c1 + rng.random_range(5..=45); // Target %

        // Solute initially = (c1 / 100) * initial_volume
        // Non-solute (constant) = ((100 - c1) / 100) * initial_volume
        // In final mixture, non-solute is (100 - target_c)%
        // Final Volume = Non-solute / ((100 - target_c)/100)
        let non_solute = ((100 - c1) as f64 / 100.0) * initial_volume as f64;
        let final_volume = non_solute / ((100 - target_c) as f64 / 100.0);
        let added_solute = final_volume - initial_volume as f64;
        let rounded_added = (added_solute * 10.0).round() / 10.0;

        let prompt = format!(
            "A **{} liter** solution contains **{}%** {} by volume.\n\nHow many liters of {} must be added to increase the concentration to **{}%**?",
            initial_volume, c1, solute, pure_solute, target_c
        );

        let solution = format!(
            "**Step 1:** Identify the quantity that remains constant (the water/solvent component):\n\
             \\[ \\text{{Solvent Volume}} = (100\\% - {}%) \\times {} = {}% \\times {} = {:.1} \\text{{ liters}} \\]\n\n\
             **Step 2:** In the final solution with {}% {}, solvent represents \\(100\\% - {}% = {}%\\):\n\
             \\[ \\text{{Total Final Volume}} = \\frac{{{:.1}}}{{{}\\%}} = \\frac{{{:.1}}}{{{:.2}}} = {:.1} \\text{{ liters}} \\]\n\n\
             **Step 3:** Calculate added pure {}:\n\
             \\[ \\text{{Added Volume}} = {:.1} - {} = **{:.1}** \\text{{ liters}} \\]",
            c1, initial_volume, 100 - c1, initial_volume, non_solute, target_c, solute, target_c, 100 - target_c,
            non_solute, 100 - target_c, non_solute, (100 - target_c) as f64 / 100.0, final_volume, solute, final_volume, initial_volume, rounded_added
        );

        let parameters = serde_json::json!({
            "variant": "dilution_addition",
            "solute": solute,
            "initial_volume": initial_volume,
            "initial_concentration": c1,
            "target_concentration": target_c,
            "added_volume": rounded_added,
        });

        let correct_answer = serde_json::json!({
            "value": rounded_added,
            "formatted": format!("{:.1}", rounded_added),
            "unit": "liters",
            "solution": solution,
        });

        let step1 = StepNode::new(
            "constant_component",
            StepType::IntermediateResult,
            format!("Find volume of constant non-{} component", solute).as_str(),
            format!("{} * (100 - {})% = {:.1}", initial_volume, c1, non_solute),
            format!("{:.1}", non_solute),
        )
        .with_expected_value(non_solute)
        .with_hints(vec![
            StepHint::principle("When adding pure solute, the amount of solvent (water) remains invariant."),
            StepHint::operation(format!("Calculate {} * ({} / 100).", initial_volume, 100 - c1).as_str()),
            StepHint::intermediate_relation(format!("Solvent volume = {:.1} liters", non_solute).as_str()),
        ]);

        let step2 = StepNode::new(
            "solve_added",
            StepType::FinalAnswer,
            format!("Calculate added pure {}", solute).as_str(),
            format!("{:.1} / ({}%) - {} = {:.1}", non_solute, 100 - target_c, initial_volume, rounded_added),
            format!("{:.1}", rounded_added),
        )
        .with_expected_value(rounded_added)
        .with_dependencies(vec!["constant_component".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Final Volume = Invariant Solvent / Final Solvent Fraction, then subtract initial volume."),
            StepHint::operation(format!("Compute {:.1} / {:.2} - {}.", non_solute, (100 - target_c) as f64 / 100.0, initial_volume).as_str()),
            StepHint::intermediate_relation(format!("Added volume = {:.1} liters", rounded_added).as_str()),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2], "solve_added");

        ProblemInstance::new(
            format!("inst-mix-l3-{}", seed),
            FAMILY_MIXTURES_ALLIGATION,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(serde_json::json!({
            "target_time_ms": 40_000,
            "difficulty_level": 3,
            "variant": "dilution_addition",
            "learning_object_level": "variation",
        }))
    }

    /// Level 4: Repeated replacement formula Q = Q0 * (1 - x/V)^n
    fn generate_level_4(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let total_vol = rng.random_range(40..=200); 
        let replaced_x = rng.random_range(5..=30);
        let n_ops = rng.random_range(2..=4); 

        let fraction_remaining = 1.0 - (replaced_x as f64 / total_vol as f64);
        let final_pure = total_vol as f64 * fraction_remaining.powi(n_ops);
        let rounded_final = (final_pure * 100.0).round() / 100.0;

        let prompt = format!(
            "A container initially holds **{} liters of pure milk**.\n\
             **{} liters** of milk is drawn out and replaced with water. This process is repeated {} more time(s) (total {} replacements).\n\n\
             How many liters of pure milk remain in the container?",
            total_vol, replaced_x, n_ops - 1, n_ops
        );

        let solution = format!(
            "**Step 1:** Apply the standard repeated replacement formula:\n\
             \\[ \\text{{Remaining Pure Liquid}} = V \\times \\left(1 - \\frac{{x}}{{V}}\\right)^n \\]\n\n\
             **Step 2:** Substitute \\(V = {}\\), \\(x = {}\\), \\(n = {}\\):\n\
             \\[ 1 - \\frac{{{}}}{{{}}} = 1 - {:.3} = {:.3} \\]\n\n\
             **Step 3:** Compute final milk quantity:\n\
             \\[ \\text{{Remaining Milk}} = {} \\times ({:.3})^{{{}}} = **{:.2}** \\text{{ liters}} \\]",
            total_vol, replaced_x, n_ops, replaced_x, total_vol, replaced_x as f64 / total_vol as f64, fraction_remaining,
            total_vol, fraction_remaining, n_ops, rounded_final
        );

        let parameters = serde_json::json!({
            "variant": "repeated_replacement",
            "total_volume": total_vol,
            "replaced_per_step": replaced_x,
            "operations_count": n_ops,
            "final_pure_liquid": rounded_final,
        });

        let correct_answer = serde_json::json!({
            "value": rounded_final,
            "formatted": format!("{:.2}", rounded_final),
            "unit": "liters",
            "solution": solution,
        });

        let step1 = StepNode::new(
            "formula_ratio",
            StepType::Transformation,
            "Calculate (1 - x/V)",
            format!("1 - {}/{} = {:.3}", replaced_x, total_vol, fraction_remaining),
            format!("{:.3}", fraction_remaining),
        )
        .with_expected_value(fraction_remaining)
        .with_hints(vec![
            StepHint::principle("The fraction of pure liquid remaining after each operation is (1 - x/V)."),
            StepHint::operation(format!("Compute 1 - {} / {}.", replaced_x, total_vol)),
            StepHint::intermediate_relation(format!("1 - x/V = {:.3}", fraction_remaining)),
        ]);

        let step2 = StepNode::new(
            "calc_final_pure",
            StepType::FinalAnswer,
            "Compute final liquid V * (1 - x/V)^2",
            format!("{} * ({:.3})^2 = {:.2}", total_vol, fraction_remaining, rounded_final),
            format!("{:.2}", rounded_final),
        )
        .with_expected_value(rounded_final)
        .with_dependencies(vec!["formula_ratio".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Multiply initial volume by (1 - x/V)^2."),
            StepHint::operation(format!("Calculate {} * ({:.3})^2.", total_vol, fraction_remaining)),
            StepHint::intermediate_relation(format!("Remaining milk = {:.2} liters", rounded_final)),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2], "calc_final_pure");

        ProblemInstance::new(
            format!("inst-mix-l4-{}", seed),
            FAMILY_MIXTURES_ALLIGATION,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(serde_json::json!({
            "target_time_ms": 40_000,
            "difficulty_level": 4,
            "variant": "repeated_replacement",
            "learning_object_level": "variation",
        }))
    }

    /// Level 5: Transfer commercial / alloy composition problem
    fn generate_level_5(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let ratios = [
            ("1 : 4", 20),
            ("1 : 3", 25),
            ("3 : 7", 30),
            ("1 : 2", 33), // Approx but not exact, skip 33 for target calculations
            ("2 : 3", 40),
            ("1 : 1", 50),
            ("3 : 2", 60),
            ("7 : 3", 70),
            ("3 : 1", 75),
            ("4 : 1", 80),
            ("9 : 1", 90),
        ];

        let idx_a = rng.random_range(0..ratios.len() - 3);
        let idx_b = rng.random_range((idx_a + 2)..ratios.len());
        
        let (ratio_a_str, copper_pct_a) = ratios[idx_a];
        let (ratio_b_str, copper_pct_b) = ratios[idx_b];
        
        // Pick target
        let min_target = copper_pct_a + 5;
        let max_target = copper_pct_b - 5;
        let target_steps = (max_target - min_target) / 5;
        let step = rng.random_range(0..=target_steps);
        let target_copper_pct = min_target + step * 5;

        let metals = ["copper and tin", "gold and silver", "zinc and copper", "nickel and iron", "lead and tin"];
        let target_metals = ["copper", "gold", "zinc", "nickel", "lead"];
        let m_idx = rng.random_range(0..metals.len());
        let metal_pair = metals[m_idx];
        let target_metal = target_metals[m_idx];

        let diff_a = (copper_pct_b as i32 - target_copper_pct as i32).abs() as u32;
        let diff_b = (target_copper_pct as i32 - copper_pct_a as i32).abs() as u32;

        fn gcd(a: u32, b: u32) -> u32 {
            if b == 0 { a } else { gcd(b, a % b) }
        }
        let g = gcd(diff_a, diff_b);
        let ratio_a = diff_a / g;
        let ratio_b = diff_b / g;

        let prompt = format!(
            "Alloy \\(A\\) contains {} in the ratio **{}** ({}% {}).\n\
             Alloy \\(B\\) contains {} in the ratio **{}** ({}% {}).\n\n\
             In what ratio must Alloy \\(A\\) and Alloy \\(B\\) be melted together to produce a new alloy containing **{}% {}**?",
             metal_pair, ratio_a_str, copper_pct_a, target_metal,
             metal_pair, ratio_b_str, copper_pct_b, target_metal,
             target_copper_pct, target_metal
        );

        let solution = format!(
            "**Step 1:** Express the {} concentration in both alloys:\n\
             \\[ \\text{{Alloy }} A = {}\\%, \\quad \\text{{Alloy }} B = {}\\% \\]\n\n\
             **Step 2:** Apply Alligation to the target concentration of {}% {}:\n\
             \\[ \\frac{{\\text{{Weight of }} A}}{{\\text{{Weight of }} B}} = \\frac{{{} - {}}}{{{} - {}}} = \\frac{{{}}}{{{}}} \\]\n\n\
             **Step 3:** Simplify the ratio:\n\
             \\[ \\frac{{{}}}{{{}}} = **{}:{}** \\]",
            target_metal, copper_pct_a, copper_pct_b, 
            target_copper_pct, target_metal, 
            copper_pct_b, target_copper_pct, target_copper_pct, copper_pct_a, diff_a, diff_b,
            diff_a, diff_b, ratio_a, ratio_b
        );

        let parameters = serde_json::json!({
            "variant": "transfer_commercial",
            "pct_a": copper_pct_a,
            "pct_b": copper_pct_b,
            "target_pct": target_copper_pct,
            "ratio_a": ratio_a,
            "ratio_b": ratio_b,
        });

        let correct_answer = serde_json::json!({
            "value": ratio_a as f64 / ratio_b as f64,
            "formatted": format!("{}:{}", ratio_a, ratio_b),
            "ratio": [ratio_a, ratio_b],
            "solution": solution,
        });

        let step1 = StepNode::new(
            "alligation_calc",
            StepType::Transformation,
            "Apply alligation to alloy concentrations",
            format!("({} - {}) : ({} - {}) = {} : {}", copper_pct_b, target_copper_pct, target_copper_pct, copper_pct_a, diff_a, diff_b),
            format!("{}:{}", diff_a, diff_b),
        )
        .with_alternates(vec![format!("{}:{}", ratio_a, ratio_b), format!("{}/{}", diff_a, diff_b)])
        .with_hints(vec![
            StepHint::principle(format!("Apply Rule of Alligation using the percentage of the common metal ({}).", target_metal).as_str()),
            StepHint::operation(format!("Compute ({}% - {}%) : ({}% - {}%).", copper_pct_b, target_copper_pct, target_copper_pct, copper_pct_a).as_str()),
            StepHint::intermediate_relation(format!("Ratio = {} : {}", diff_a, diff_b).as_str()),
        ]);

        let step2 = StepNode::new(
            "simplify_alloy_ratio",
            StepType::FinalAnswer,
            "Simplify alloy mixing ratio",
            format!("{}/{} : {}/{} = {} : {}", diff_a, g, diff_b, g, ratio_a, ratio_b),
            format!("{}:{}", ratio_a, ratio_b),
        )
        .with_alternates(vec![format!("{}:{}", ratio_a, ratio_b), format!("{}/{}", ratio_a, ratio_b)])
        .with_dependencies(vec!["alligation_calc".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle(format!("Reduce the ratio {}:{} to lowest terms.", diff_a, diff_b).as_str()),
            StepHint::operation(format!("Divide both terms by {}.", g).as_str()),
            StepHint::intermediate_relation(format!("Ratio = {}:{}", ratio_a, ratio_b).as_str()),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2], "simplify_alloy_ratio");

        ProblemInstance::new(
            format!("inst-mix-l5-{}", seed),
            FAMILY_MIXTURES_ALLIGATION,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(serde_json::json!({
            "target_time_ms": 45_000,
            "difficulty_level": 5,
            "variant": "transfer_commercial",
            "learning_object_level": "transfer",
        }))
    }
}

impl ProblemGenerator for MixturesAlligationGenerator {
    fn family_id(&self) -> &str {
        FAMILY_MIXTURES_ALLIGATION
    }

    fn template_ref(&self) -> &str {
        TEMPLATE_MIXTURES_ALLIGATION_V1
    }

    fn supported_variants(&self) -> Vec<String> {
        vec![
            "two_component_blend".to_string(),
            "alligation_ratio".to_string(),
            "dilution_addition".to_string(),
            "repeated_replacement".to_string(),
            "transfer_commercial".to_string(),
        ]
    }

    fn target_latency_ms(&self, difficulty_level: u32) -> u64 {
        match difficulty_level {
            1 => 30_000,
            2 => 35_000,
            3 => 40_000,
            4 => 45_000,
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
pub struct MixturesAlligationValidator;

impl ProblemValidator for MixturesAlligationValidator {
    fn family_id(&self) -> &str {
        FAMILY_MIXTURES_ALLIGATION
    }

    fn evaluate(
        &self,
        instance: &ProblemInstance,
        student_answer: &serde_json::Value,
        time_taken_ms: u64,
        target_time_ms: u64,
    ) -> AnswerEvaluation {
        // Support both ratio formats ("1:3") and numeric values
        let formatted_exp = instance
            .correct_answer
            .get("formatted")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let expected_val = instance
            .correct_answer
            .get("value")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        if let Some(s_str) = student_answer.as_str() {
            let norm_sub = s_str.trim().replace(' ', "");
            let norm_exp = formatted_exp.trim().replace(' ', "");
            if norm_sub == norm_exp {
                return AnswerEvaluation::correct(1.0, time_taken_ms, target_time_ms)
                    .with_diagnostic("✓ Correct mixture / alligation solution.");
            }
        }

        let parsed_val = NumericAnswerParser::parse_student_answer(student_answer);

        if let Some(student_num) = parsed_val {
            let diff = (student_num - expected_val).abs();
            let is_correct = diff <= 0.05 || (expected_val > 0.0 && diff / expected_val <= 0.02);

            if is_correct {
                let score = if target_time_ms > 0 && time_taken_ms > target_time_ms {
                    0.85
                } else {
                    1.0
                };
                AnswerEvaluation::correct(score, time_taken_ms, target_time_ms)
                    .with_parsed_values(student_num, expected_val)
                    .with_diagnostic("✓ Correct numerical answer.")
            } else {
                // Check if ratio was inverted (e.g. 3:1 instead of 1:3)
                if expected_val > 0.0 && (student_num - (1.0 / expected_val)).abs() <= 0.05 {
                    return AnswerEvaluation::incorrect(
                        ErrorCategory::Strategy,
                        "Inverted ratio: You swapped the quantities of the two mixture components.",
                    )
                    .with_parsed_values(student_num, expected_val);
                }

                AnswerEvaluation::incorrect(
                    ErrorCategory::Calculation,
                    format!("Calculation error: Expected {}, but received {:.2}.", formatted_exp, student_num),
                )
                .with_parsed_values(student_num, expected_val)
            }
        } else {
            AnswerEvaluation::incorrect(
                ErrorCategory::Careless,
                "Unable to parse response. Submit as a ratio (e.g. '1:3') or decimal number.",
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
    fn test_mixtures_alligation_generation_all_levels() {
        let gen = MixturesAlligationGenerator;
        let validator = MixturesAlligationValidator;

        for level in 1..=5 {
            let inst = gen.generate(&ProblemFamilyId::new(FAMILY_MIXTURES_ALLIGATION), 42 + level as u64, level, None).unwrap();
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
    fn test_mixtures_alligation_inverted_ratio_diagnostic() {
        let gen = MixturesAlligationGenerator;
        let validator = MixturesAlligationValidator;

        let inst = gen.generate(&ProblemFamilyId::new(FAMILY_MIXTURES_ALLIGATION), 100, 5, Some("transfer_commercial")).unwrap();
        // Correct is 1:3 -> value 0.333. Inverted is 3:1 -> value 3.0
        let eval = validator.evaluate(&inst, &serde_json::json!(3.0), 20000, 40000);
        assert!(!eval.is_correct);
        assert_eq!(eval.error_category, Some(ErrorCategory::Strategy));
        assert!(eval.diagnostic_message.unwrap().contains("Inverted ratio"));
    }
}
