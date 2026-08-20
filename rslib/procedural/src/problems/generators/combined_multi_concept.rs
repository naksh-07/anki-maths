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
        let ratios = [(3, 2), (5, 3), (4, 1), (7, 3), (5, 4), (2, 1), (3, 1), (8, 3), (9, 2), (7, 5), (6, 5), (8, 7), (9, 4), (5, 2)];
        let (r1, r2) = ratios[rng.random_range(0..ratios.len())];
        let sum_r = r1 + r2;

        let total_pop = (rng.random_range(5..=200) * sum_r * 100) as i64;
        let workforce_pct = (rng.random_range(10..=45) * 2) as i64; // 20%, 22%, ..., 90%
        let workforce_total = (total_pop * workforce_pct) / 100;
        let male_workers = (workforce_total * r1) / sum_r;

        let prompt = format!(
            "A city has a total population of **{}**.\n\
             **{}%** of the population constitutes the working labor force.\n\
             Among the working population, the ratio of male to female workers is **{} : {}**.\n\n\
             Find the total number of **male workers** in the city.",
            total_pop, workforce_pct, r1, r2
        );

        let solution = format!(
            "**Step 1:** Calculate the total workforce:\n\
             \\[ \\text{{Workforce}} = {} \\times {}% = {} \\]\n\n\
             **Step 2:** Divide the workforce according to ratio \\({}:{}\\) (sum = {}):\n\
             \\[ \\text{{Male Workers}} = {} \\times \\frac{{{}}}{{{}}} = **{}** \\]",
            total_pop, workforce_pct, workforce_total,
            r1, r2, sum_r, workforce_total, r1, sum_r, male_workers
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
        .with_expected_value(workforce_total as f64);

        let step2 = StepNode::new(
            "calc_male_workers",
            StepType::FinalAnswer,
            "Apply ratio proportion to workforce",
            format!("{} * {}/{} = {}", workforce_total, r1, sum_r, male_workers),
            format!("{}", male_workers),
        )
        .with_expected_value(male_workers as f64)
        .with_dependencies(vec!["calc_workforce".to_string()])
        .as_final();

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
        }))
    }

    /// Level 2: Profit/Loss + Successive Discount: Marked price markup + 2 successive discounts
    fn generate_level_2(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let cp = (rng.random_range(2..=100) * 100) as i64; // $200 to $10000
        let markup_pct = (rng.random_range(4..=30) * 5) as i64; // 20% to 150%
        let mp = cp + (cp * markup_pct) / 100;
        let d1 = (rng.random_range(2..=10) * 5) as i64; // 10% to 50%
        let d2 = (rng.random_range(1..=8) * 5) as i64; // 5% to 40%

        let sp = ((mp as f64 * (1.0 - d1 as f64 / 100.0)) * (1.0 - d2 as f64 / 100.0)).round() as i64;
        let net_profit = sp - cp;

        let prompt = format!(
            "A trader buys an item for **${}** and marks it up by **{}%** above Cost Price to establish the Marked Price.\n\
             He then offers two successive discounts of **{}%** and **{}%** on the Marked Price.\n\n\
             Calculate the trader's **net profit (or loss) in dollars**.",
            cp, markup_pct, d1, d2
        );

        let solution = format!(
            "**Step 1:** Calculate Marked Price (MP):\n\
             \\[ \\text{{MP}} = {} \\times (1 + \\frac{{{}}}{{100}}) = \\${} \\]\n\n\
             **Step 2:** Apply successive discounts for Selling Price (SP):\n\
             \\[ \\text{{SP}} = {} \\times (1 - \\frac{{{}}}{{100}}) \\times (1 - \\frac{{{}}}{{100}}) = \\${} \\]\n\n\
             **Step 3:** Calculate Net Profit:\n\
             \\[ \\text{{Net Profit}} = \\text{{SP}} - \\text{{CP}} = {} - {} = **\\${}** \\]",
            cp, markup_pct, mp, mp, d1, d2, sp, sp, cp, net_profit
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
            format!("{} * (1 + {}/100) = {}", cp, markup_pct, mp),
            format!("{}", mp),
        )
        .with_expected_value(mp as f64);

        let step2 = StepNode::new(
            "calc_sp",
            StepType::IntermediateResult,
            "Apply successive discounts for SP",
            format!("{} * (1 - {}/100) * (1 - {}/100) = {}", mp, d1, d2, sp),
            format!("{}", sp),
        )
        .with_expected_value(sp as f64)
        .with_dependencies(vec!["calc_mp".to_string()]);

        let step3 = StepNode::new(
            "calc_net_profit",
            StepType::FinalAnswer,
            "Calculate Net Profit (SP - CP)",
            format!("{} - {} = {}", sp, cp, net_profit),
            format!("{}", net_profit),
        )
        .with_expected_value(net_profit as f64)
        .with_dependencies(vec!["calc_sp".to_string()])
        .as_final();

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
        }))
    }

    /// Level 3: Ratio + Average: Weighted class average from student ratio
    fn generate_level_3(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let r1 = rng.random_range(1..=15);
        let r2 = rng.random_range(1..=15);
        let avg1 = rng.random_range(10..=95);
        let avg2 = rng.random_range(10..=95);

        let weighted_sum = r1 * avg1 + r2 * avg2;
        let total_parts = r1 + r2;
        let combined_avg = (weighted_sum as f64) / (total_parts as f64);
        let rounded_avg = (combined_avg * 100.0).round() / 100.0;

        let prompt = format!(
            "In an academy, the ratio of students in Section \\(A\\) to Section \\(B\\) is **{} : {}**.\n\
             The average test score of Section \\(A\\) is **{}**, while the average test score of Section \\(B\\) is **{}**.\n\n\
             Find the **combined average score** across both sections. (Round to 2 decimal places)",
            r1, r2, avg1, avg2
        );

        let solution = format!(
            "**Step 1:** Calculate weighted total score:\n\
             \\[ \\text{{Total}} = ({} \\times {}) + ({} \\times {}) = {} + {} = {} \\]\n\n\
             **Step 2:** Divide by total ratio parts \\({} + {} = {}\\):\n\
             \\[ \\text{{Combined Average}} = \\frac{{{}}}{{{}}} = **{:.2}** \\]",
            r1, avg1, r2, avg2, r1 * avg1, r2 * avg2, weighted_sum,
            r1, r2, total_parts, weighted_sum, total_parts, rounded_avg
        );

        let parameters = serde_json::json!({
            "variant": "ratio_average",
            "r1": r1, "r2": r2, "avg1": avg1, "avg2": avg2,
            "combined_avg": rounded_avg,
        });

        let correct_answer = serde_json::json!({
            "value": rounded_avg,
            "formatted": format!("{:.2}", rounded_avg),
            "solution": solution,
        });

        let step1 = StepNode::new(
            "weighted_sum",
            StepType::Transformation,
            "Calculate weighted score sum",
            format!("({} * {}) + ({} * {}) = {}", r1, avg1, r2, avg2, weighted_sum),
            format!("{}", weighted_sum),
        )
        .with_expected_value(weighted_sum as f64);

        let step2 = StepNode::new(
            "calc_combined_avg",
            StepType::FinalAnswer,
            "Divide by total ratio parts",
            format!("{} / {} = {:.2}", weighted_sum, total_parts, rounded_avg),
            format!("{:.2}", rounded_avg),
        )
        .with_expected_value(rounded_avg)
        .with_dependencies(vec!["weighted_sum".to_string()])
        .as_final();

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
        }))
    }

    /// Level 4: Time/Work + Ratio: Efficiency ratio + collaborative duration
    fn generate_level_4(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let eff_k = rng.random_range(2..=15); // A is k times as efficient as B
        let a_alone_days = rng.random_range(2..=50) * (eff_k + 1); // ensures integer combined days
        let b_alone_days = a_alone_days * eff_k;
        let combined_days = (a_alone_days * b_alone_days) / (a_alone_days + b_alone_days);

        let prompt = format!(
            "Worker \\(A\\) is **{} times as efficient** as worker \\(B\\).\n\
             If worker \\(B\\) alone can complete a job in **{} days**, how many days will both workers take to complete the job working together?",
            eff_k, b_alone_days
        );

        let solution = format!(
            "**Step 1:** Since \\(A\\) is {} times as efficient, \\(A\\) takes \\({}/{} = {}\\) days.\n\n\
             **Step 2:** Combine daily work rates:\n\
             \\[ \\text{{Daily rate of }} (A + B) = \\frac{{1}}{{{}}} + \\frac{{1}}{{{}}} = \\frac{{{} + 1}}{{{}}} = \\frac{{{}}}{{{}}} = \\frac{{1}}{{{}}} \\]\n\n\
             **Step 3:** Combined days required:\n\
             \\[ \\text{{Total Days}} = **{}** \\text{{ days}} \\]",
            eff_k, b_alone_days, eff_k, a_alone_days,
            a_alone_days, b_alone_days, eff_k, b_alone_days, eff_k + 1, b_alone_days, combined_days, combined_days
        );

        let parameters = serde_json::json!({
            "variant": "time_work_ratio",
            "b_alone_days": b_alone_days,
            "a_alone_days": a_alone_days,
            "eff_k": eff_k,
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
            format!("{} / {} = {}", b_alone_days, eff_k, a_alone_days),
            format!("{}", a_alone_days),
        )
        .with_expected_value(a_alone_days as f64);

        let step2 = StepNode::new(
            "calc_combined_time",
            StepType::FinalAnswer,
            "Calculate combined work time",
            format!("({} * {}) / ({} + {}) = {}", a_alone_days, b_alone_days, a_alone_days, b_alone_days, combined_days),
            format!("{}", combined_days),
        )
        .with_expected_value(combined_days as f64)
        .with_dependencies(vec!["calc_a_time".to_string()])
        .as_final();

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
        }))
    }

    /// Level 5: Speed + Ratio Transfer: Ratio of speeds inversely proportional to time for constant distance
    fn generate_level_5(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let ratios = [
            (2, 3), (3, 4), (4, 5), (5, 6), (6, 7), (7, 8), (8, 9), (9, 10),
            (3, 5), (5, 7), (7, 9), (9, 11), (2, 5), (3, 7), (4, 7), (5, 8), (5, 9),
            (2, 7), (3, 8), (5, 11), (7, 10)
        ];
        let (r_speed_a, r_speed_b) = ratios[rng.random_range(0..ratios.len())];

        let diff_mins = (rng.random_range(1..=40) * 5) as i64; // 5 to 200 mins
        let time_parts_a = r_speed_b;
        let time_parts_b = r_speed_a;
        let part_diff = time_parts_a - time_parts_b;
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
             \\[ 1 \\text{{ part}} = \\frac{{{}}}{{{}}} = {} \\text{{ minutes}} \\]\n\n\
             **Step 3:** Calculate time taken by \\(A\\) ({} parts):\n\
             \\[ T_A = {} \\times {} = **{}** \\text{{ minutes}} \\]",
            r_speed_b, r_speed_a, time_parts_a, time_parts_b, part_diff, diff_mins, part_diff, diff_mins / part_diff,
            time_parts_a, time_parts_a, diff_mins / part_diff, time_a_mins
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
        );

        let step2 = StepNode::new(
            "calc_time_a",
            StepType::FinalAnswer,
            "Compute traveler A's time in minutes",
            format!("{} * ({}/{}) = {}", time_parts_a, diff_mins, part_diff, time_a_mins),
            format!("{}", time_a_mins),
        )
        .with_expected_value(time_a_mins as f64)
        .with_dependencies(vec!["invert_time_ratio".to_string()])
        .as_final();

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

pub struct CombinedMultiConceptValidator;

impl ProblemValidator for CombinedMultiConceptValidator {
    fn family_id(&self) -> &str {
        FAMILY_COMBINED_MULTI_CONCEPT
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
