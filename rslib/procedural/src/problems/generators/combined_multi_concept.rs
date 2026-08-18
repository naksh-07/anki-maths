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

pub const FAMILY_COMBINED_MULTI_CONCEPT: &str = "family.math.combined.multi_concept";
pub const TEMPLATE_COMBINED_MULTI_CONCEPT_V1: &str = "math.combined.multi_concept.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CombinedMultiConceptVariant {
    PercentageRatio,
    ProfitDiscountSuccessive,
    RatioAverage,
    TimeWorkRatio,
    SpeedRatioTransfer,
}

impl CombinedMultiConceptVariant {
    pub fn as_str(&self) -> &'static str {
        match self {
            CombinedMultiConceptVariant::PercentageRatio => "percentage_ratio",
            CombinedMultiConceptVariant::ProfitDiscountSuccessive => "profit_discount_successive",
            CombinedMultiConceptVariant::RatioAverage => "ratio_average",
            CombinedMultiConceptVariant::TimeWorkRatio => "time_work_ratio",
            CombinedMultiConceptVariant::SpeedRatioTransfer => "speed_ratio_transfer",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct CombinedMultiConceptGenerator;

impl CombinedMultiConceptGenerator {
    pub fn generate_problem(seed: u64, difficulty_level: u32, variant: Option<&str>) -> ProblemInstance {
        let mut rng = StdRng::seed_from_u64(seed);
        let chosen_variant = if let Some(v) = variant {
            match v {
                "percentage_ratio" => CombinedMultiConceptVariant::PercentageRatio,
                "profit_discount_successive" => CombinedMultiConceptVariant::ProfitDiscountSuccessive,
                "ratio_average" => CombinedMultiConceptVariant::RatioAverage,
                "time_work_ratio" => CombinedMultiConceptVariant::TimeWorkRatio,
                "speed_ratio_transfer" => CombinedMultiConceptVariant::SpeedRatioTransfer,
                _ => CombinedMultiConceptVariant::PercentageRatio,
            }
        } else {
            match difficulty_level {
                1 => CombinedMultiConceptVariant::PercentageRatio,
                2 => CombinedMultiConceptVariant::ProfitDiscountSuccessive,
                3 => CombinedMultiConceptVariant::RatioAverage,
                4 => CombinedMultiConceptVariant::TimeWorkRatio,
                _ => CombinedMultiConceptVariant::SpeedRatioTransfer,
            }
        };

        match chosen_variant {
            CombinedMultiConceptVariant::PercentageRatio => Self::generate_level_1(&mut rng, seed),
            CombinedMultiConceptVariant::ProfitDiscountSuccessive => Self::generate_level_2(&mut rng, seed),
            CombinedMultiConceptVariant::RatioAverage => Self::generate_level_3(&mut rng, seed),
            CombinedMultiConceptVariant::TimeWorkRatio => Self::generate_level_4(&mut rng, seed),
            CombinedMultiConceptVariant::SpeedRatioTransfer => Self::generate_level_5(&mut rng, seed),
        }
    }

    /// Level 1: Percentage + Ratio: Population percentage breakdown partitioned by sub-ratio
    fn generate_level_1(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let total_pop = rng.random_range(20..=60) * 1000; // e.g. 40,000
        let workforce_pct = rng.random_range(4..=8) * 10; // e.g. 60%
        let workforce_total = (total_pop * workforce_pct) / 100;
        let r1 = 3;
        let r2 = 2; // Ratio male:female = 3:2 (sum = 5)
        let male_workers = (workforce_total * r1) / (r1 + r2);

        let prompt = format!(
            "A city has a total population of **{}**.\n\
             **{}%** of the population constitutes the working labor force.\n\
             Among the working population, the ratio of male to female workers is **{} : {}**.\n\n\
             Find the total number of **male workers** in the city.",
            total_pop, workforce_pct, r1, r2
        );

        let solution = format!(
            "**Step 1:** Calculate the total workforce:\n\
             \\[ \\text{{Workforce}} = {} \\times {}% = {} \\times {:.2} = {} \\]\n\n\
             **Step 2:** Divide the workforce according to the ratio \\({}:{}\\) (sum = {}):\n\
             \\[ \\text{{Male Workers}} = {} \\times \\frac{{{}}}{{{}}} = **{}** \\]",
            total_pop, workforce_pct, total_pop, workforce_pct as f64 / 100.0, workforce_total,
            r1, r2, r1 + r2, workforce_total, r1, r1 + r2, male_workers
        );

        let parameters = serde_json::json!({
            "variant": "percentage_ratio",
            "total_population": total_pop,
            "workforce_pct": workforce_pct,
            "workforce_total": workforce_total,
            "ratio_male": r1,
            "ratio_female": r2,
            "male_workers": male_workers,
        });

        let correct_answer = serde_json::json!({
            "value": male_workers as f64,
            "formatted": format!("{}", male_workers),
            "solution": solution,
        });

        let step1 = StepNode::new(
            "calc_workforce",
            StepType::IntermediateResult,
            "Calculate total workforce from percentage",
            format!("{} * {}% = {}", total_pop, workforce_pct, workforce_total),
            format!("{}", workforce_total),
        )
        .with_expected_value(workforce_total as f64)
        .with_hints(vec![
            StepHint::principle("First find the intermediate total workforce by applying the percentage."),
            StepHint::operation(format!("Multiply {} by {}%.", total_pop, workforce_pct)),
            StepHint::intermediate_relation(format!("Total workforce = {}", workforce_total)),
        ]);

        let step2 = StepNode::new(
            "calc_male_workers",
            StepType::FinalAnswer,
            "Apply ratio proportion to workforce",
            format!("{} * {}/{} = {}", workforce_total, r1, r1 + r2, male_workers),
            format!("{}", male_workers),
        )
        .with_expected_value(male_workers as f64)
        .with_dependencies(vec!["calc_workforce".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Divide the workforce into (r1 / (r1 + r2)) parts for males."),
            StepHint::operation(format!("Multiply {} * ({}/{}).", workforce_total, r1, r1 + r2)),
            StepHint::intermediate_relation(format!("Male workers = {}", male_workers)),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2], "calc_male_workers");

        ProblemInstance::new(
            format!("inst-comb-l1-{}", seed),
            FAMILY_COMBINED_MULTI_CONCEPT,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(serde_json::json!({
            "target_time_ms": 30_000,
            "difficulty_level": 1,
            "variant": "percentage_ratio",
            "learning_object_level": "procedural_execution",
        }))
    }

    /// Level 2: Profit/Loss + Successive Discount: Marked price markup + 2 successive discounts
    fn generate_level_2(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let cp = rng.random_range(4..=15) * 100; // e.g. $800
        let markup_pct = 50; // Marked price = 1.50 * CP
        let mp = cp + (cp * markup_pct) / 100; // e.g. $1200
        let d1 = 20; // 20% trade discount -> $960
        let d2 = 10; // 10% cash discount -> $864
        let sp = ((mp as f64 * (1.0 - d1 as f64 / 100.0)) * (1.0 - d2 as f64 / 100.0)).round() as i64;
        let net_profit = sp - cp as i64;

        let prompt = format!(
            "A trader buys an item for **${}** and marks it up by **{}%** above Cost Price to establish the Marked Price.\n\
             He then offers two successive discounts of **{}%** and **{}%** on the Marked Price.\n\n\
             Calculate the trader's **net profit in dollars**.",
            cp, markup_pct, d1, d2
        );

        let solution = format!(
            "**Step 1:** Calculate Marked Price (MP):\n\
             \\[ \\text{{MP}} = {} \\times (1 + 0.{}) = \\${} \\]\n\n\
             **Step 2:** Apply successive discounts to find Selling Price (SP):\n\
             \\[ \\text{{SP}} = {} \\times (1 - 0.{}) \\times (1 - 0.{}) = {} \\times 0.{} \\times 0.{} = \\${} \\]\n\n\
             **Step 3:** Calculate Net Profit:\n\
             \\[ \\text{{Net Profit}} = \\text{{SP}} - \\text{{CP}} = {} - {} = **\\${}** \\]",
            cp, markup_pct, mp, mp, d1, d2, mp, 100 - d1, 100 - d2, sp, sp, cp, net_profit
        );

        let parameters = serde_json::json!({
            "variant": "profit_discount_successive",
            "cp": cp,
            "markup_pct": markup_pct,
            "mp": mp,
            "discount1": d1,
            "discount2": d2,
            "sp": sp,
            "net_profit": net_profit,
        });

        let correct_answer = serde_json::json!({
            "value": net_profit as f64,
            "formatted": format!("{}", net_profit),
            "unit": "$",
            "solution": solution,
        });

        let step1 = StepNode::new(
            "calc_mp",
            StepType::Transformation,
            "Calculate Marked Price",
            format!("{} * 1.{} = {}", cp, markup_pct, mp),
            format!("{}", mp),
        )
        .with_expected_value(mp as f64)
        .with_hints(vec![
            StepHint::principle("Marked Price = Cost Price * (1 + Markup%)."),
            StepHint::operation(format!("Multiply {} * 1.{}.", cp, markup_pct)),
            StepHint::intermediate_relation(format!("MP = ${}", mp)),
        ]);

        let step2 = StepNode::new(
            "calc_sp",
            StepType::IntermediateResult,
            "Apply successive discounts for SP",
            format!("{} * 0.{} * 0.{} = {}", mp, 100 - d1, 100 - d2, sp),
            format!("{}", sp),
        )
        .with_expected_value(sp as f64)
        .with_dependencies(vec!["calc_mp".to_string()])
        .with_hints(vec![
            StepHint::principle("Selling Price = MP * (1 - d1/100) * (1 - d2/100)."),
            StepHint::operation(format!("Compute {} * 0.{} * 0.{}.", mp, 100 - d1, 100 - d2)),
            StepHint::intermediate_relation(format!("SP = ${}", sp)),
        ]);

        let step3 = StepNode::new(
            "calc_net_profit",
            StepType::FinalAnswer,
            "Calculate Net Profit (SP - CP)",
            format!("{} - {} = {}", sp, cp, net_profit),
            format!("{}", net_profit),
        )
        .with_expected_value(net_profit as f64)
        .with_dependencies(vec!["calc_sp".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Net Profit = Selling Price - Cost Price."),
            StepHint::operation(format!("Subtract {} - {}.", sp, cp)),
            StepHint::intermediate_relation(format!("Profit = ${}", net_profit)),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2, step3], "calc_net_profit");

        ProblemInstance::new(
            format!("inst-comb-l2-{}", seed),
            FAMILY_COMBINED_MULTI_CONCEPT,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(serde_json::json!({
            "target_time_ms": 40_000,
            "difficulty_level": 2,
            "variant": "profit_discount_successive",
            "learning_object_level": "procedural_execution",
        }))
    }

    /// Level 3: Ratio + Average: Weighted class average from student ratio
    fn generate_level_3(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let r1 = rng.random_range(2..=4); // Section A weight
        let r2 = rng.random_range(3..=5); // Section B weight
        let avg1 = rng.random_range(60..=85);
        let avg2 = avg1 + rng.random_range(5..=15);

        let weighted_sum = r1 * avg1 + r2 * avg2;
        let total_parts = r1 + r2;
        let combined_avg = (weighted_sum as f64) / (total_parts as f64);
        let rounded_avg = (combined_avg * 10.0).round() / 10.0;

        let prompt = format!(
            "In an academy, the ratio of students in Section \\(A\\) to Section \\(B\\) is **{} : {}**.\n\
             The average test score of Section \\(A\\) is **{}**, while the average test score of Section \\(B\\) is **{}**.\n\n\
             Find the **combined average score** of all students across both sections.",
            r1, r2, avg1, avg2
        );

        let solution = format!(
            "**Step 1:** Use ratio weights \\(w_1 = {}\\) and \\(w_2 = {}\\) for the two sections.\n\n\
             **Step 2:** Calculate weighted sum of scores:\n\
             \\[ \\text{{Weighted Total}} = ({} \\times {}) + ({} \\times {}) = {} + {} = {} \\]\n\n\
             **Step 3:** Divide by total ratio parts \\({} + {} = {}\\):\n\
             \\[ \\text{{Combined Average}} = \\frac{{{}}}{{{}}} = **{:.1}** \\]",
            r1, r2, r1, avg1, r2, avg2, r1 * avg1, r2 * avg2, weighted_sum, r1, r2, total_parts, weighted_sum, total_parts, rounded_avg
        );

        let parameters = serde_json::json!({
            "variant": "ratio_average",
            "r1": r1,
            "r2": r2,
            "avg1": avg1,
            "avg2": avg2,
            "combined_avg": rounded_avg,
        });

        let correct_answer = serde_json::json!({
            "value": rounded_avg,
            "formatted": format!("{:.1}", rounded_avg),
            "solution": solution,
        });

        let step1 = StepNode::new(
            "weighted_sum",
            StepType::Transformation,
            "Calculate weighted score sum",
            format!("({} * {}) + ({} * {}) = {}", r1, avg1, r2, avg2, weighted_sum),
            format!("{}", weighted_sum),
        )
        .with_expected_value(weighted_sum as f64)
        .with_hints(vec![
            StepHint::principle("Multiply each section's ratio weight by its average score: (w1 * avg1) + (w2 * avg2)."),
            StepHint::operation(format!("Compute ({} * {}) + ({} * {}).", r1, avg1, r2, avg2)),
            StepHint::intermediate_relation(format!("Weighted sum = {}", weighted_sum)),
        ]);

        let step2 = StepNode::new(
            "calc_combined_avg",
            StepType::FinalAnswer,
            "Divide by total ratio parts",
            format!("{} / {} = {:.1}", weighted_sum, total_parts, rounded_avg),
            format!("{:.1}", rounded_avg),
        )
        .with_expected_value(rounded_avg)
        .with_dependencies(vec!["weighted_sum".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Divide the weighted sum by (r1 + r2)."),
            StepHint::operation(format!("Divide {} by {}.", weighted_sum, total_parts)),
            StepHint::intermediate_relation(format!("Combined average = {:.1}", rounded_avg)),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2], "calc_combined_avg");

        ProblemInstance::new(
            format!("inst-comb-l3-{}", seed),
            FAMILY_COMBINED_MULTI_CONCEPT,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(serde_json::json!({
            "target_time_ms": 35_000,
            "difficulty_level": 3,
            "variant": "ratio_average",
            "learning_object_level": "variation",
        }))
    }

    /// Level 4: Time/Work + Ratio: Efficiency ratio + collaborative duration
    fn generate_level_4(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        // A is twice as efficient as B (efficiency ratio 2:1)
        // A takes N days alone. How many days when working together?
        // Days = (A's days * B's days) / (A's days + B's days)
        // If efficiency of A is k times B, B takes k * A_days.
        // Combined time = (A_days * k * A_days) / (A_days * (k + 1)) = (k / (k + 1)) * A_days
        let eff_ratio_a = 2;
        let eff_ratio_b = 1;
        let b_alone_days = rng.random_range(6..=18) * 3; // multiple of 3 (e.g. 18 days)
        let a_alone_days = b_alone_days / 2;             // e.g. 9 days
        let combined_days = (a_alone_days * b_alone_days) / (a_alone_days + b_alone_days); // 9*18/27 = 6 days

        let prompt = format!(
            "Worker \\(A\\) is **twice as efficient** as worker \\(B\\) (efficiency ratio **2 : 1**).\n\
             If worker \\(B\\) alone can complete a job in **{} days**, how many days will both workers take to complete the job working together?",
            b_alone_days
        );

        let solution = format!(
            "**Step 1:** Since \\(A\\) is twice as efficient as \\(B\\), \\(A\\) takes half the time:\n\
             \\[ \\text{{Time for }} A = \\frac{{{}}}{{2}} = {} \\text{{ days}} \\]\n\n\
             **Step 2:** Combine daily work rates:\n\
             \\[ \\text{{Daily rate of }} (A + B) = \\frac{{1}}{{{}}} + \\frac{{1}}{{{}}} = \\frac{{2 + 1}}{{{}}} = \\frac{{3}}{{{}}} = \\frac{{1}}{{{}}} \\]\n\n\
             **Step 3:** Combined days required:\n\
             \\[ \\text{{Total Days}} = **{}** \\text{{ days}} \\]",
            b_alone_days, a_alone_days, a_alone_days, b_alone_days, b_alone_days, b_alone_days, combined_days, combined_days
        );

        let parameters = serde_json::json!({
            "variant": "time_work_ratio",
            "b_alone_days": b_alone_days,
            "a_alone_days": a_alone_days,
            "efficiency_ratio": [eff_ratio_a, eff_ratio_b],
            "combined_days": combined_days,
        });

        let correct_answer = serde_json::json!({
            "value": combined_days as f64,
            "formatted": format!("{}", combined_days),
            "unit": "days",
            "solution": solution,
        });

        let step1 = StepNode::new(
            "calc_a_time",
            StepType::Transformation,
            "Find time for A from efficiency ratio",
            format!("{} / 2 = {}", b_alone_days, a_alone_days),
            format!("{}", a_alone_days),
        )
        .with_expected_value(a_alone_days as f64)
        .with_hints(vec![
            StepHint::principle("Efficiency is inversely proportional to time taken: A is twice as fast, so A takes half the days."),
            StepHint::operation(format!("Divide {} by 2.", b_alone_days)),
            StepHint::intermediate_relation(format!("A takes {} days", a_alone_days)),
        ]);

        let step2 = StepNode::new(
            "calc_combined_time",
            StepType::FinalAnswer,
            "Calculate combined work time",
            format!("({} * {}) / ({} + {}) = {}", a_alone_days, b_alone_days, a_alone_days, b_alone_days, combined_days),
            format!("{}", combined_days),
        )
        .with_expected_value(combined_days as f64)
        .with_dependencies(vec!["calc_a_time".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Combined time = (Time_A * Time_B) / (Time_A + Time_B)."),
            StepHint::operation(format!("Compute ({} * {}) / ({} + {}).", a_alone_days, b_alone_days, a_alone_days, b_alone_days)),
            StepHint::intermediate_relation(format!("Combined time = {} days", combined_days)),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2], "calc_combined_time");

        ProblemInstance::new(
            format!("inst-comb-l4-{}", seed),
            FAMILY_COMBINED_MULTI_CONCEPT,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(serde_json::json!({
            "target_time_ms": 35_000,
            "difficulty_level": 4,
            "variant": "time_work_ratio",
            "learning_object_level": "variation",
        }))
    }

    /// Level 5: Speed + Ratio Transfer: Ratio of speeds inversely proportional to time for constant distance
    fn generate_level_5(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        // Ratio of speeds of A and B is 4:5
        // To cover a constant distance D, A takes 30 mins more than B. Find time taken by A.
        // Ratio of times = 5:4. Difference in parts = 5 - 4 = 1 part = 30 mins.
        // Time for A = 5 * 30 = 150 mins = 2.5 hours.
        let r_speed_a = 4;
        let r_speed_b = 5;
        let diff_mins = rng.random_range(3..=8) * 5; // e.g. 20, 25, 30, 35 mins
        let time_parts_a = r_speed_b; // 5 parts
        let time_parts_b = r_speed_a; // 4 parts
        let part_diff = time_parts_a - time_parts_b; // 1
        let time_a_mins = (time_parts_a * diff_mins) / part_diff;

        let prompt = format!(
            "The ratio of speeds of two travelers \\(A\\) and \\(B\\) is **{} : {}**.\n\
             To cover the same journey distance, \\(A\\) takes **{} minutes more** than \\(B\\).\n\n\
             Find the total time taken by traveler \\(A\\) to complete the journey in **minutes**.",
            r_speed_a, r_speed_b, diff_mins
        );

        let solution = format!(
            "**Step 1:** For a constant distance, time is inversely proportional to speed:\n\
             \\[ \\text{{Ratio of times }} (T_A : T_B) = {} : {} \\]\n\n\
             **Step 2:** The difference in ratio parts is \\({} - {} = {}\\) part.\n\
             \\[ 1 \\text{{ part}} = {} \\text{{ minutes}} \\]\n\n\
             **Step 3:** Calculate time taken by \\(A\\) ({} parts):\n\
             \\[ T_A = {} \\times {} = **{}** \\text{{ minutes}} \\]",
            r_speed_b, r_speed_a, time_parts_a, time_parts_b, part_diff, diff_mins, time_parts_a, time_parts_a, diff_mins, time_a_mins
        );

        let parameters = serde_json::json!({
            "variant": "speed_ratio_transfer",
            "speed_ratio": [r_speed_a, r_speed_b],
            "time_diff_mins": diff_mins,
            "time_a_mins": time_a_mins,
        });

        let correct_answer = serde_json::json!({
            "value": time_a_mins as f64,
            "formatted": format!("{}", time_a_mins),
            "unit": "minutes",
            "solution": solution,
        });

        let step1 = StepNode::new(
            "invert_time_ratio",
            StepType::Transformation,
            "Invert speed ratio to get time ratio",
            format!("Speed ratio {}:{} ==> Time ratio {}:{}", r_speed_a, r_speed_b, time_parts_a, time_parts_b),
            format!("{}:{}", time_parts_a, time_parts_b),
        )
        .with_hints(vec![
            StepHint::principle("When distance is constant, Time ratio is the inverse of Speed ratio: T_A : T_B = S_B : S_A."),
            StepHint::operation(format!("Invert {}:{} to get {}:{}.", r_speed_a, r_speed_b, time_parts_a, time_parts_b)),
            StepHint::intermediate_relation(format!("Time ratio = {}:{}", time_parts_a, time_parts_b)),
        ]);

        let step2 = StepNode::new(
            "calc_time_a",
            StepType::FinalAnswer,
            "Compute traveler A's time in minutes",
            format!("{} * {} = {}", time_parts_a, diff_mins, time_a_mins),
            format!("{}", time_a_mins),
        )
        .with_expected_value(time_a_mins as f64)
        .with_dependencies(vec!["invert_time_ratio".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Multiply A's time ratio parts by the minutes per part."),
            StepHint::operation(format!("Multiply {} * {}.", time_parts_a, diff_mins)),
            StepHint::intermediate_relation(format!("Time for A = {} minutes", time_a_mins)),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2], "calc_time_a");

        ProblemInstance::new(
            format!("inst-comb-l5-{}", seed),
            FAMILY_COMBINED_MULTI_CONCEPT,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(serde_json::json!({
            "target_time_ms": 40_000,
            "difficulty_level": 5,
            "variant": "speed_ratio_transfer",
            "learning_object_level": "transfer",
        }))
    }
}

impl ProblemGenerator for CombinedMultiConceptGenerator {
    fn family_id(&self) -> &str {
        FAMILY_COMBINED_MULTI_CONCEPT
    }

    fn template_ref(&self) -> &str {
        TEMPLATE_COMBINED_MULTI_CONCEPT_V1
    }

    fn supported_variants(&self) -> Vec<String> {
        vec![
            "percentage_ratio".to_string(),
            "profit_discount_successive".to_string(),
            "ratio_average".to_string(),
            "time_work_ratio".to_string(),
            "speed_ratio_transfer".to_string(),
        ]
    }

    fn target_latency_ms(&self, difficulty_level: u32) -> u64 {
        match difficulty_level {
            1 => 30_000,
            2 => 40_000,
            3 => 35_000,
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
pub struct CombinedMultiConceptValidator;

impl ProblemValidator for CombinedMultiConceptValidator {
    fn family_id(&self) -> &str {
        FAMILY_COMBINED_MULTI_CONCEPT
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
            let is_correct = diff <= 0.1 || (expected_val > 0.0 && diff / expected_val <= 0.01);

            if is_correct {
                let score = if target_time_ms > 0 && time_taken_ms > target_time_ms {
                    0.85
                } else {
                    1.0
                };
                AnswerEvaluation::correct(score, time_taken_ms, target_time_ms)
                    .with_parsed_values(student_num, expected_val)
                    .with_diagnostic("✓ Correct multi-concept solution.")
            } else {
                AnswerEvaluation::incorrect(
                    ErrorCategory::Calculation,
                    format!("Calculation error: Expected {:.1}, but received {:.1}.", expected_val, student_num),
                )
                .with_parsed_values(student_num, expected_val)
            }
        } else {
            AnswerEvaluation::incorrect(
                ErrorCategory::Careless,
                "Unable to parse response. Please submit a valid numerical value.",
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
    fn test_combined_multi_concept_generation_all_levels() {
        let gen = CombinedMultiConceptGenerator;
        let validator = CombinedMultiConceptValidator;

        for level in 1..=5 {
            let inst = gen.generate(&ProblemFamilyId::new(FAMILY_COMBINED_MULTI_CONCEPT), 42 + level as u64, level, None).unwrap();
            assert!(!inst.rendered_prompt.is_empty(), "Prompt non-empty for L{}", level);

            let graph = inst.solution_graph();
            assert!(graph.is_some(), "SolutionGraph exists for L{}", level);
            assert!(graph.unwrap().validate_topology(), "Topology valid for L{}", level);

            let correct_ans = inst.correct_answer.get("value").unwrap();
            let eval = validator.evaluate(&inst, correct_ans, 15000, 30000);
            assert!(eval.is_correct, "Self-eval succeeds for L{}", level);
        }
    }
}
