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

pub const FAMILY_AVERAGE: &str = "family.math.arithmetic.average";
pub const TEMPLATE_AVERAGE_V1: &str = "math.arithmetic.average.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AverageVariant {
    DirectAverage,
    MissingValue,
    InclusionExclusion,
    WeightedAverage,
    AverageSpeed,
}

impl AverageVariant {
    pub fn as_str(&self) -> &'static str {
        match self {
            AverageVariant::DirectAverage => "direct_average",
            AverageVariant::MissingValue => "missing_value",
            AverageVariant::InclusionExclusion => "inclusion_exclusion",
            AverageVariant::WeightedAverage => "weighted_average",
            AverageVariant::AverageSpeed => "average_speed",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct AverageGenerator;

impl AverageGenerator {
    pub fn generate_problem(seed: u64, difficulty_level: u32, variant: Option<&str>) -> ProblemInstance {
        let mut rng = StdRng::seed_from_u64(seed);
        let chosen_variant = if let Some(v) = variant {
            match v {
                "direct_average" => AverageVariant::DirectAverage,
                "missing_value" => AverageVariant::MissingValue,
                "inclusion_exclusion" => AverageVariant::InclusionExclusion,
                "weighted_average" => AverageVariant::WeightedAverage,
                "average_speed" => AverageVariant::AverageSpeed,
                _ => AverageVariant::DirectAverage,
            }
        } else {
            match difficulty_level {
                1 => AverageVariant::DirectAverage,
                2 => AverageVariant::MissingValue,
                3 => AverageVariant::InclusionExclusion,
                4 => AverageVariant::WeightedAverage,
                _ => AverageVariant::AverageSpeed,
            }
        };

        match chosen_variant {
            AverageVariant::DirectAverage => Self::generate_level_1(&mut rng, seed),
            AverageVariant::MissingValue => Self::generate_level_2(&mut rng, seed),
            AverageVariant::InclusionExclusion => Self::generate_level_3(&mut rng, seed),
            AverageVariant::WeightedAverage => Self::generate_level_4(&mut rng, seed),
            AverageVariant::AverageSpeed => Self::generate_level_5(&mut rng, seed),
        }
    }

    /// Level 1: Direct average of 4 to 6 numbers with random symmetric/asymmetric distributions
    fn generate_level_1(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let target_avg = rng.random_range(15..=120) as i64;
        let count = rng.random_range(4..=6) as usize;

        let mut values: Vec<i64> = Vec::new();
        for _ in 0..(count - 1) {
            let offset = rng.random_range(-20..=20) as i64;
            values.push(target_avg + offset);
        }
        // Adjust last so sum is exact count * target_avg
        let current_sum: i64 = values.iter().sum();
        let last_val = (target_avg * count as i64) - current_sum;
        values.push(last_val);

        let total_sum = target_avg * count as i64;
        let values_str = values.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(", ");
        let sum_expr = values.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(" + ");

        let prompt = format!(
            "Find the arithmetic average (mean) of the following set of **{} numbers**:\n\n\\[ {} \\]",
            count, values_str
        );

        let solution = format!(
            "**Step 1:** Calculate total sum:\n\
             \\[ {} = {} \\]\n\n\
             **Step 2:** Divide total sum by count ({}):\n\
             \\[ \\text{{Average}} = \\frac{{{}}}{{{}}} = **{}** \\]",
            sum_expr, total_sum, count, total_sum, count, target_avg
        );

        let parameters = serde_json::json!({
            "variant": "direct_average",
            "values": values,
            "count": count,
            "expected_avg": target_avg,
        });

        let correct_answer = serde_json::json!({
            "value": target_avg as f64,
            "formatted": format!("{}", target_avg),
            "solution": solution,
        });

        let step1 = StepNode::new(
            "calc_sum",
            StepType::Transformation,
            format!("Sum the {} numbers", count),
            format!("{} = {}", sum_expr, total_sum),
            format!("{}", total_sum),
        )
        .with_expected_value(total_sum as f64)
        .with_hints(vec![
            StepHint::principle("Average = Sum of items / Count of items."),
            StepHint::operation(format!("Compute {}.", sum_expr)),
            StepHint::intermediate_relation(format!("Sum = {}", total_sum)),
        ]);

        let step2 = StepNode::new(
            "calc_avg",
            StepType::FinalAnswer,
            format!("Divide sum by count {}", count),
            format!("{} / {} = {}", total_sum, count, target_avg),
            format!("{}", target_avg),
        )
        .with_expected_value(target_avg as f64)
        .with_dependencies(vec!["calc_sum".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Divide total sum by number of items."),
            StepHint::operation(format!("Divide {} by {}.", total_sum, count)),
            StepHint::intermediate_relation(format!("Average = {}", target_avg)),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2], "calc_avg");

        ProblemInstance::new(
            format!("inst-avg-1-{}", seed),
            FAMILY_AVERAGE,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(serde_json::json!({
            "difficulty_level": 1,
            "target_time_ms": 25_000,
            "variant": "direct_average",
        }))
    }

    /// Level 2: Finding missing value from average
    fn generate_level_2(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let count = rng.random_range(4..=6) as usize;
        let avg = rng.random_range(25..=100) as i64;

        let mut known_values = Vec::new();
        for _ in 0..(count - 1) {
            let offset = rng.random_range(-25..=25) as i64;
            known_values.push(avg + offset);
        }
        let total_sum = (count as i64) * avg;
        let known_sum: i64 = known_values.iter().sum();
        let target_x = total_sum - known_sum;

        let known_str = known_values.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(", ");
        let known_sum_expr = known_values.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(" + ");

        let prompt = format!(
            "The average of **{} numbers** is **{}**. **{} of the numbers** are: **{}**.\n\n\
             What is the value of the remaining number?",
            count, avg, count - 1, known_str
        );

        let solution = format!(
            "**Step 1:** Calculate required total sum:\n\
             \\[ {} \\times {} = {} \\]\n\n\
             **Step 2:** Sum the {} known numbers:\n\
             \\[ {} = {} \\]\n\n\
             **Step 3:** Subtract known sum from total sum:\n\
             \\[ x = {} - {} = **{}** \\]",
            count, avg, total_sum, count - 1, known_sum_expr, known_sum,
            total_sum, known_sum, target_x
        );

        let parameters = serde_json::json!({
            "variant": "missing_value",
            "count": count,
            "avg": avg,
            "known_values": known_values,
            "target_x": target_x,
        });

        let correct_answer = serde_json::json!({
            "value": target_x as f64,
            "formatted": format!("{}", target_x),
            "solution": solution,
        });

        let step1 = StepNode::new(
            "calc_total_sum",
            StepType::Transformation,
            format!("Calculate required total sum ({} * {})", count, avg),
            format!("{} * {} = {}", count, avg, total_sum),
            format!("{}", total_sum),
        )
        .with_expected_value(total_sum as f64);

        let step2 = StepNode::new(
            "calc_known_sum",
            StepType::IntermediateResult,
            "Sum the known numbers",
            format!("{} = {}", known_sum_expr, known_sum),
            format!("{}", known_sum),
        )
        .with_expected_value(known_sum as f64);

        let step3 = StepNode::new(
            "calc_missing_x",
            StepType::FinalAnswer,
            "Subtract known sum from total sum",
            format!("{} - {} = {}", total_sum, known_sum, target_x),
            format!("{}", target_x),
        )
        .with_expected_value(target_x as f64)
        .with_dependencies(vec!["calc_total_sum".to_string(), "calc_known_sum".to_string()])
        .as_final();

        let graph = SolutionGraph::new(vec![step1, step2, step3], "calc_missing_x");

        ProblemInstance::new(
            format!("inst-avg-2-{}", seed),
            FAMILY_AVERAGE,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(serde_json::json!({
            "difficulty_level": 2,
            "target_time_ms": 35_000,
            "variant": "missing_value",
        }))
    }

    /// Level 3: Inclusion / Exclusion / Replacement with dynamic variations
    fn generate_level_3(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let mode = rng.random_range(0..3);

        if mode == 0 {
            // Replacement: person of weight W_old is replaced by new person, changing avg by delta
            let n_members = rng.random_range(8..=30) as i64;
            let delta = rng.random_range(1..=4) as i64; // e.g. average increases by 2 kg
            let old_weight = rng.random_range(40..=85) as i64;
            let new_weight = old_weight + (n_members * delta);

            let prompt = format!(
                "The average weight of a group of **{} people** increases by **{} kg** when one member weighing **{} kg** is replaced by a new person.\n\n\
                 What is the weight of the new person in kg?",
                n_members, delta, old_weight
            );

            let solution = format!(
                "**Step 1:** Formula for replacement change:\n\
                 \\[ \\text{{Weight of New Person}} = \\text{{Weight of Replaced Person}} + (\\text{{Number of People}} \\times \\text{{Change in Average}}) \\]\n\n\
                 **Step 2:** Calculate total net increase across all {} members:\n\
                 \\[ {} \\times {} = {} \\text{{ kg}} \\]\n\n\
                 **Step 3:** Compute new person's weight:\n\
                 \\[ \\text{{New Weight}} = {} + {} = **{} kg** \\]",
                n_members, n_members, delta, n_members * delta,
                old_weight, n_members * delta, new_weight
            );

            let parameters = serde_json::json!({
                "variant": "replacement",
                "n_members": n_members,
                "delta": delta,
                "old_weight": old_weight,
                "new_weight": new_weight,
            });

            let correct_answer = serde_json::json!({
                "value": new_weight as f64,
                "formatted": format!("{}", new_weight),
                "unit": "kg",
                "solution": solution,
            });

            let step1 = StepNode::new(
                "calc_net_change",
                StepType::Transformation,
                "Calculate total net change N * delta",
                format!("{} * {} = {}", n_members, delta, n_members * delta),
                format!("{}", n_members * delta),
            )
            .with_expected_value((n_members * delta) as f64);

            let step2 = StepNode::new(
                "calc_new_weight",
                StepType::FinalAnswer,
                "Add net change to old weight",
                format!("{} + {} = {}", old_weight, n_members * delta, new_weight),
                format!("{}", new_weight),
            )
            .with_expected_value(new_weight as f64)
            .with_dependencies(vec!["calc_net_change".to_string()])
            .as_final();

            let graph = SolutionGraph::new(vec![step1, step2], "calc_new_weight");

            ProblemInstance::new(
                format!("inst-avg-3-{}", seed),
                FAMILY_AVERAGE,
                seed,
                parameters,
                prompt,
                correct_answer,
            )
            .with_solution_graph(graph)
            .with_metadata(serde_json::json!({
                "difficulty_level": 3,
                "target_time_ms": 35_000,
                "variant": "inclusion_exclusion",
            }))
        } else {
            // Inclusion: Teacher/Leader joins
            let n_students = rng.random_range(12..=35) as i64;
            let avg_age = rng.random_range(11..=22) as i64;
            let inc = rng.random_range(1..=3) as i64;
            let new_avg = avg_age + inc;
            let leader_age = (n_students + 1) * new_avg - (n_students * avg_age);

            let student_sum = n_students * avg_age;
            let total_sum = (n_students + 1) * new_avg;

            let prompt = format!(
                "The average age of a class of **{} students** is **{} years**. When the teacher's age is included, the average age increases by **{} year{}**.\n\n\
                 What is the **teacher's age** in years?",
                n_students, avg_age, inc, if inc > 1 { "s" } else { "" }
            );

            let solution = format!(
                "**Step 1:** Total age of {} students = \\({} \\times {} = {}\\) years.\n\n\
                 **Step 2:** New total with teacher ({} people) = \\({} \\times {} = {}\\) years.\n\n\
                 **Step 3:** Teacher's age = \\({} - {} = **{}**\\) years.",
                n_students, n_students, avg_age, student_sum,
                n_students + 1, n_students + 1, new_avg, total_sum,
                total_sum, student_sum, leader_age
            );

            let parameters = serde_json::json!({
                "variant": "inclusion",
                "n_students": n_students,
                "avg_age": avg_age,
                "inc": inc,
                "leader_age": leader_age,
            });

            let correct_answer = serde_json::json!({
                "value": leader_age as f64,
                "formatted": format!("{}", leader_age),
                "unit": "years",
                "solution": solution,
            });

            let step1 = StepNode::new(
                "student_total",
                StepType::Transformation,
                "Calculate students total age",
                format!("{} * {} = {}", n_students, avg_age, student_sum),
                format!("{}", student_sum),
            )
            .with_expected_value(student_sum as f64);

            let step2 = StepNode::new(
                "new_total",
                StepType::IntermediateResult,
                "Calculate new total with teacher",
                format!("{} * {} = {}", n_students + 1, new_avg, total_sum),
                format!("{}", total_sum),
            )
            .with_expected_value(total_sum as f64);

            let step3 = StepNode::new(
                "calc_teacher_age",
                StepType::FinalAnswer,
                "Subtract student total from new total",
                format!("{} - {} = {}", total_sum, student_sum, leader_age),
                format!("{}", leader_age),
            )
            .with_expected_value(leader_age as f64)
            .with_dependencies(vec!["student_total".to_string(), "new_total".to_string()])
            .as_final();

            let graph = SolutionGraph::new(vec![step1, step2, step3], "calc_teacher_age");

            ProblemInstance::new(
                format!("inst-avg-3-{}", seed),
                FAMILY_AVERAGE,
                seed,
                parameters,
                prompt,
                correct_answer,
            )
            .with_solution_graph(graph)
            .with_metadata(serde_json::json!({
                "difficulty_level": 3,
                "target_time_ms": 40_000,
                "variant": "inclusion_exclusion",
            }))
        }
    }

    /// Level 4: Weighted average of two or three groups with dynamic counts and means
    fn generate_level_4(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let n1 = rng.random_range(10..=50) as i64;
        let avg1 = (rng.random_range(40..=85)) as f64;
        let n2 = rng.random_range(10..=50) as i64;
        let avg2 = (rng.random_range(40..=90)) as f64;

        let total_n = n1 + n2;
        let sum1 = n1 as f64 * avg1;
        let sum2 = n2 as f64 * avg2;
        let total_sum = sum1 + sum2;
        let combined_avg = ((total_sum / total_n as f64) * 100.0).round() / 100.0;

        let prompt = format!(
            "Section A has **{} students** with an average score of **{:.1}**. Section B has **{} students** with an average score of **{:.1}**.\n\n\
             What is the **combined average score** of all students together? (Round to 2 decimal places)",
            n1, avg1, n2, avg2
        );

        let solution = format!(
            "**Step 1:** Total score for Section A = \\({} \\times {:.1} = {:.1}\\)\n\
             **Step 2:** Total score for Section B = \\({} \\times {:.1} = {:.1}\\)\n\n\
             **Step 3:** Combined total score = \\({:.1} + {:.1} = {:.1}\\)\n\
             **Step 4:** Combined average = \\(\\frac{{{:.1}}}{{{} + {}}} = \\frac{{{:.1}}}{{{}}} = **{:.2}** \\)",
            n1, avg1, sum1, n2, avg2, sum2, sum1, sum2, total_sum, total_sum, n1, n2, total_sum, total_n, combined_avg
        );

        let parameters = serde_json::json!({
            "variant": "weighted_average",
            "n1": n1, "avg1": avg1, "n2": n2, "avg2": avg2,
            "combined_avg": combined_avg,
        });

        let correct_answer = serde_json::json!({
            "value": combined_avg,
            "formatted": format!("{:.2}", combined_avg),
            "solution": solution,
        });

        let step1 = StepNode::new(
            "weighted_total",
            StepType::Transformation,
            "Calculate combined sum of products",
            format!("{} * {:.1} + {} * {:.1} = {:.1}", n1, avg1, n2, avg2, total_sum),
            format!("{:.1}", total_sum),
        )
        .with_expected_value(total_sum);

        let step2 = StepNode::new(
            "calc_weighted_avg",
            StepType::FinalAnswer,
            "Divide by total student count",
            format!("{:.1} / {} = {:.2}", total_sum, total_n, combined_avg),
            format!("{:.2}", combined_avg),
        )
        .with_expected_value(combined_avg)
        .with_dependencies(vec!["weighted_total".to_string()])
        .as_final();

        let graph = SolutionGraph::new(vec![step1, step2], "calc_weighted_avg");

        ProblemInstance::new(
            format!("inst-avg-4-{}", seed),
            FAMILY_AVERAGE,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(serde_json::json!({
            "difficulty_level": 4,
            "target_time_ms": 45_000,
            "variant": "weighted_average",
        }))
    }

    /// Level 5: Round trip and multi-segment average speeds
    fn generate_level_5(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let is_three_segments = rng.random_bool(0.3);

        if is_three_segments {
            // Three equal distance segments at speeds v1, v2, v3
            let v1 = (rng.random_range(2..=6) * 10) as f64; // 20 to 60 km/h
            let v2 = (rng.random_range(3..=8) * 10) as f64;
            let v3 = (rng.random_range(4..=10) * 10) as f64;

            let harmonic_denom = (1.0 / v1) + (1.0 / v2) + (1.0 / v3);
            let avg_speed = ((3.0 / harmonic_denom) * 100.0).round() / 100.0;

            let prompt = format!(
                "A car travels one-third of a total journey distance at **{:.0} km/h**, the second one-third at **{:.0} km/h**, and the final one-third at **{:.0} km/h**.\n\n\
                 What is the **average speed** for the entire journey in km/h? (Round to 2 decimal places)",
                v1, v2, v3
            );

            let solution = format!(
                "**Step 1:** Formula for average speed across 3 equal distance segments:\n\
                 \\[ \\text{{Average Speed}} = \\frac{{3}}{{\\frac{{1}}{{v_1}} + \\frac{{1}}{{v_2}} + \\frac{{1}}{{v_3}}}} = \\frac{{3}}{{\\frac{{1}}{{{:.0}}} + \\frac{{1}}{{{:.0}}} + \\frac{{1}}{{{:.0}}}}} \\]\n\n\
                 **Step 2:** Evaluate reciprocal sum:\n\
                 \\[ \\text{{Denominator}} = {:.5} \\]\n\n\
                 **Step 3:** Calculate Average Speed:\n\
                 \\[ \\text{{Average Speed}} = \\frac{{3}}{{{:.5}}} = **{:.2} km/h** \\]",
                v1, v2, v3, harmonic_denom, harmonic_denom, avg_speed
            );

            let parameters = serde_json::json!({
                "variant": "three_segment_speed",
                "v1": v1, "v2": v2, "v3": v3,
                "avg_speed": avg_speed,
            });

            let correct_answer = serde_json::json!({
                "value": avg_speed,
                "formatted": format!("{:.2}", avg_speed),
                "unit": "km/h",
                "solution": solution,
            });

            let step1 = StepNode::new(
                "calc_harmonic_3",
                StepType::FinalAnswer,
                "Apply 3-segment harmonic mean formula 3 / (1/v1 + 1/v2 + 1/v3)",
                format!("3 / (1/{} + 1/{} + 1/{}) = {:.2}", v1, v2, v3, avg_speed),
                format!("{:.2}", avg_speed),
            )
            .with_expected_value(avg_speed)
            .as_final();

            let graph = SolutionGraph::new(vec![step1], "calc_harmonic_3");

            ProblemInstance::new(
                format!("inst-avg-5-{}", seed),
                FAMILY_AVERAGE,
                seed,
                parameters,
                prompt,
                correct_answer,
            )
            .with_solution_graph(graph)
            .with_metadata(serde_json::json!({
                "difficulty_level": 5,
                "target_time_ms": 50_000,
                "variant": "average_speed",
            }))
        } else {
            // Dynamic round trip: 2 * v1 * v2 / (v1 + v2)
            let v1 = (rng.random_range(3..=10) * 10) as f64; // 30 to 100 km/h
            let v2 = (rng.random_range(2..=8) * 10) as f64;  // 20 to 80 km/h

            let avg_speed = ((2.0 * v1 * v2 / (v1 + v2)) * 100.0).round() / 100.0;

            let prompt = format!(
                "A traveler drives from City A to City B at **{:.0} km/h** and returns along the exact same route at **{:.0} km/h**.\n\n\
                 What is the **average speed** for the entire round trip in km/h? (Round to 2 decimal places)",
                v1, v2
            );

            let solution = format!(
                "**Step 1:** For equal distances in both directions, average speed is the harmonic mean:\n\
                 \\[ \\text{{Average Speed}} = \\frac{{2 v_1 v_2}}{{v_1 + v_2}} \\]\n\n\
                 **Step 2:** Substitute speeds:\n\
                 \\[ \\text{{Average Speed}} = \\frac{{2 \\times {:.0} \\times {:.0}}}{{{:.0} + {:.0}}} = \\frac{{{:.0}}}{{{:.0}}} = **{:.2} km/h** \\]",
                v1, v2, v1, v2, 2.0 * v1 * v2, v1 + v2, avg_speed
            );

            let parameters = serde_json::json!({
                "variant": "round_trip_speed",
                "speed1": v1, "speed2": v2,
                "avg_speed": avg_speed,
            });

            let correct_answer = serde_json::json!({
                "value": avg_speed,
                "formatted": format!("{:.2}", avg_speed),
                "unit": "km/h",
                "solution": solution,
            });

            let step1 = StepNode::new(
                "calc_harmonic_avg",
                StepType::FinalAnswer,
                "Apply harmonic mean formula 2*s1*s2/(s1+s2)",
                format!("(2 * {} * {}) / ({} + {}) = {:.2}", v1, v2, v1, v2, avg_speed),
                format!("{:.2}", avg_speed),
            )
            .with_expected_value(avg_speed)
            .as_final();

            let graph = SolutionGraph::new(vec![step1], "calc_harmonic_avg");

            ProblemInstance::new(
                format!("inst-avg-5-{}", seed),
                FAMILY_AVERAGE,
                seed,
                parameters,
                prompt,
                correct_answer,
            )
            .with_solution_graph(graph)
            .with_metadata(serde_json::json!({
                "difficulty_level": 5,
                "target_time_ms": 45_000,
                "variant": "average_speed",
            }))
        }
    }
}

impl ProblemGenerator for AverageGenerator {
    fn family_id(&self) -> &str {
        FAMILY_AVERAGE
    }

    fn template_ref(&self) -> &str {
        TEMPLATE_AVERAGE_V1
    }

    fn supported_variants(&self) -> Vec<String> {
        vec![
            "direct_average".to_string(),
            "missing_value".to_string(),
            "inclusion_exclusion".to_string(),
            "weighted_average".to_string(),
            "average_speed".to_string(),
        ]
    }

    fn target_latency_ms(&self, difficulty_level: u32) -> u64 {
        match difficulty_level {
            1 => 25_000,
            2 => 35_000,
            3 => 40_000,
            4 => 45_000,
            _ => 50_000,
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

pub struct AverageValidator;

impl ProblemValidator for AverageValidator {
    fn family_id(&self) -> &str {
        FAMILY_AVERAGE
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
