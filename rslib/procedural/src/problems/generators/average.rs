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

    /// Level 1: Direct average of 5 clean integers
    fn generate_level_1(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let target_avg = rng.random_range(20..=80);
        let offsets = [-12, -4, 2, 6, 8];
        let mut values: Vec<i32> = offsets.iter().map(|&off| target_avg + off).collect();
        // Adjust last so sum is exact 5 * target_avg
        let sum: i32 = values.iter().sum();
        let diff = (target_avg * 5) - sum;
        values[4] += diff;

        let prompt = format!(
            "Find the average of the following set of numbers:\n\n\\[ {}, {}, {}, {}, {} \\]",
            values[0], values[1], values[2], values[3], values[4]
        );

        let total_sum = target_avg * 5;
        let solution = format!(
            "**Step 1:** Calculate total sum:\n\
             \\[ {} + {} + {} + {} + {} = {} \\]\n\n\
             **Step 2:** Divide by count (5):\n\
             \\[ \\text{{Average}} = \\frac{{{}}}{{5}} = **{}** \\]",
            values[0], values[1], values[2], values[3], values[4], total_sum, total_sum, target_avg
        );

        let parameters = serde_json::json!({
            "variant": "direct_average",
            "values": values,
            "count": 5,
            "expected_avg": target_avg,
        });

        let correct_answer = serde_json::json!({
            "value": target_avg as f64,
            "formatted": format!("{}", target_avg),
            "solution": solution,
        });

        let metadata = serde_json::json!({
            "difficulty": 1.0,
            "target_time_ms": 30_000,
            "generator": TEMPLATE_AVERAGE_V1,
        });

        let step1 = crate::problems::steps::StepNode::new(
            "calc_sum",
            crate::problems::steps::StepType::Transformation,
            "Sum the 5 numbers",
            format!("{} + {} + {} + {} + {} = {}", values[0], values[1], values[2], values[3], values[4], total_sum),
            format!("{}", total_sum),
        )
        .with_expected_value(total_sum as f64);

        let step2 = crate::problems::steps::StepNode::new(
            "calc_avg",
            crate::problems::steps::StepType::FinalAnswer,
            "Divide sum by count 5",
            format!("{} / 5 = {}", total_sum, target_avg),
            format!("{}", target_avg),
        )
        .with_expected_value(target_avg as f64)
        .with_dependencies(vec!["calc_sum".to_string()])
        .as_final();

        let graph = crate::problems::steps::SolutionGraph::new(vec![step1, step2], "calc_avg");

        ProblemInstance::new(
            format!("inst-avg-1-{}", seed),
            FAMILY_AVERAGE,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(metadata)
    }

    /// Level 2: Finding missing value from average
    fn generate_level_2(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let avg = rng.random_range(30..=90);
        let v1 = avg - rng.random_range(5..=20);
        let v2 = avg + rng.random_range(3..=15);
        let v3 = avg - rng.random_range(2..=10);
        let v4 = avg + rng.random_range(5..=18);
        let target_x = 5 * avg - (v1 + v2 + v3 + v4);

        let prompt = format!(
            "The average of 5 numbers is {}. Four of the numbers are {}, {}, {}, and {}.\n\nWhat is the fifth number?",
            avg, v1, v2, v3, v4
        );

        let known_sum = v1 + v2 + v3 + v4;
        let total_sum = 5 * avg;
        let solution = format!(
            "**Step 1:** Calculate required total sum:\n\
             \\[ 5 \\times {} = {} \\]\n\n\
             **Step 2:** Sum the 4 known numbers:\n\
             \\[ {} + {} + {} + {} = {} \\]\n\n\
             **Step 3:** Subtract to find the 5th number:\n\
             \\[ x = {} - {} = **{}** \\]",
            avg, total_sum, v1, v2, v3, v4, known_sum, total_sum, known_sum, target_x
        );

        let parameters = serde_json::json!({
            "variant": "missing_value",
            "avg": avg,
            "known_values": [v1, v2, v3, v4],
            "solution_x": target_x,
        });

        let correct_answer = serde_json::json!({
            "value": target_x as f64,
            "formatted": format!("{}", target_x),
            "solution": solution,
        });

        let metadata = serde_json::json!({
            "difficulty": 2.0,
            "target_time_ms": 40_000,
            "generator": TEMPLATE_AVERAGE_V1,
        });

        let step1 = crate::problems::steps::StepNode::new(
            "calc_total_sum",
            crate::problems::steps::StepType::Transformation,
            "Calculate required total sum (5 * avg)",
            format!("5 * {} = {}", avg, total_sum),
            format!("{}", total_sum),
        )
        .with_expected_value(total_sum as f64);

        let step2 = crate::problems::steps::StepNode::new(
            "calc_known_sum",
            crate::problems::steps::StepType::IntermediateResult,
            "Sum the 4 known numbers",
            format!("{} + {} + {} + {} = {}", v1, v2, v3, v4, known_sum),
            format!("{}", known_sum),
        )
        .with_expected_value(known_sum as f64);

        let step3 = crate::problems::steps::StepNode::new(
            "calc_missing_x",
            crate::problems::steps::StepType::FinalAnswer,
            "Subtract known sum from total sum",
            format!("{} - {} = {}", total_sum, known_sum, target_x),
            format!("{}", target_x),
        )
        .with_expected_value(target_x as f64)
        .with_dependencies(vec!["calc_total_sum".to_string(), "calc_known_sum".to_string()])
        .as_final();

        let graph = crate::problems::steps::SolutionGraph::new(vec![step1, step2, step3], "calc_missing_x");

        ProblemInstance::new(
            format!("inst-avg-2-{}", seed),
            FAMILY_AVERAGE,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(metadata)
    }

    /// Level 3: Inclusion / exclusion of a member
    fn generate_level_3(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let n_students = rng.random_range(15..=25);
        let avg_age = rng.random_range(12..=16);
        let inc = 1; // avg age increases by 1 year when teacher joins
        let new_avg = avg_age + inc;
        let teacher_age = (n_students + 1) * new_avg - (n_students * avg_age);

        let prompt = format!(
            "The average age of a class of {} students is {} years. When the teacher's age is included, the average age increases by {} year.\n\nWhat is the teacher's age?",
            n_students, avg_age, inc
        );

        let student_sum = n_students * avg_age;
        let total_sum = (n_students + 1) * new_avg;
        let solution = format!(
            "**Step 1:** Total age of {} students = {} × {} = {} years\n\n\
             **Step 2:** New total with teacher ({} people) = {} × {} = {} years\n\n\
             **Step 3:** Teacher's age = {} - {} = **{}** years",
            n_students, n_students, avg_age, student_sum,
            n_students + 1, n_students + 1, new_avg, total_sum,
            total_sum, student_sum, teacher_age
        );

        let parameters = serde_json::json!({
            "variant": "inclusion_exclusion",
            "n_students": n_students,
            "avg_age": avg_age,
            "teacher_age": teacher_age,
        });

        let correct_answer = serde_json::json!({
            "value": teacher_age as f64,
            "formatted": format!("{}", teacher_age),
            "solution": solution,
        });

        let metadata = serde_json::json!({
            "difficulty": 3.0,
            "target_time_ms": 50_000,
            "generator": TEMPLATE_AVERAGE_V1,
        });

        let step1 = crate::problems::steps::StepNode::new(
            "student_total",
            crate::problems::steps::StepType::Transformation,
            "Calculate students total age",
            format!("{} * {} = {}", n_students, avg_age, student_sum),
            format!("{}", student_sum),
        )
        .with_expected_value(student_sum as f64);

        let step2 = crate::problems::steps::StepNode::new(
            "new_total",
            crate::problems::steps::StepType::IntermediateResult,
            "Calculate new total with teacher",
            format!("{} * {} = {}", n_students + 1, new_avg, total_sum),
            format!("{}", total_sum),
        )
        .with_expected_value(total_sum as f64);

        let step3 = crate::problems::steps::StepNode::new(
            "calc_teacher_age",
            crate::problems::steps::StepType::FinalAnswer,
            "Subtract student total from new total",
            format!("{} - {} = {}", total_sum, student_sum, teacher_age),
            format!("{}", teacher_age),
        )
        .with_expected_value(teacher_age as f64)
        .with_dependencies(vec!["student_total".to_string(), "new_total".to_string()])
        .as_final();

        let graph = crate::problems::steps::SolutionGraph::new(vec![step1, step2, step3], "calc_teacher_age");

        ProblemInstance::new(
            format!("inst-avg-3-{}", seed),
            FAMILY_AVERAGE,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(metadata)
    }

    /// Level 4: Weighted average of two groups
    fn generate_level_4(_rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let n1 = 20;
        let avg1 = 60.0;
        let n2 = 30;
        let avg2 = 75.0;
        let combined_avg = (n1 as f64 * avg1 + n2 as f64 * avg2) / (n1 + n2) as f64; // (1200 + 2250) / 50 = 69.0

        let prompt = format!(
            "Class A with {} students has an average test score of {:.0}. Class B with {} students has an average score of {:.0}.\n\nWhat is the combined average score of both classes together?",
            n1, avg1, n2, avg2
        );

        let sum1 = n1 as f64 * avg1;
        let sum2 = n2 as f64 * avg2;
        let solution = format!(
            "**Step 1:** Total score for Class A = {} × {:.0} = {:.0}\n\
             **Step 2:** Total score for Class B = {} × {:.0} = {:.0}\n\n\
             **Step 3:** Combined total score = {:.0} + {:.0} = {:.0}\n\
             **Step 4:** Combined average = {:.0} / ({}+{}) = **{:.1}**",
            n1, avg1, sum1, n2, avg2, sum2, sum1, sum2, sum1 + sum2, sum1 + sum2, n1, n2, combined_avg
        );

        let parameters = serde_json::json!({
            "variant": "weighted_average",
            "n1": n1,
            "avg1": avg1,
            "n2": n2,
            "avg2": avg2,
            "combined_avg": combined_avg,
        });

        let correct_answer = serde_json::json!({
            "value": combined_avg,
            "formatted": format!("{:.1}", combined_avg),
            "solution": solution,
        });

        let metadata = serde_json::json!({
            "difficulty": 4.0,
            "target_time_ms": 60_000,
            "generator": TEMPLATE_AVERAGE_V1,
        });

        let step1 = crate::problems::steps::StepNode::new(
            "weighted_total",
            crate::problems::steps::StepType::Transformation,
            "Calculate sum of products",
            format!("{} * {:.0} + {} * {:.0} = {:.0}", n1, avg1, n2, avg2, sum1 + sum2),
            format!("{:.0}", sum1 + sum2),
        )
        .with_expected_value(sum1 + sum2);

        let step2 = crate::problems::steps::StepNode::new(
            "calc_weighted_avg",
            crate::problems::steps::StepType::FinalAnswer,
            "Divide by total student count",
            format!("{:.0} / {} = {:.1}", sum1 + sum2, n1 + n2, combined_avg),
            format!("{:.1}", combined_avg),
        )
        .with_expected_value(combined_avg)
        .with_dependencies(vec!["weighted_total".to_string()])
        .as_final();

        let graph = crate::problems::steps::SolutionGraph::new(vec![step1, step2], "calc_weighted_avg");

        ProblemInstance::new(
            format!("inst-avg-4-{}", seed),
            FAMILY_AVERAGE,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(metadata)
    }

    /// Level 5: Round trip average speed (Harmonic mean formula 2xy/(x+y))
    fn generate_level_5(_rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let speed1 = 40.0;
        let speed2 = 60.0;
        let avg_speed = (2.0 * speed1 * speed2) / (speed1 + speed2); // (4800) / 100 = 48.0 km/h

        let prompt = format!(
            "A traveler drives from City A to City B at a speed of {:.0} km/h and returns along the same route at {:.0} km/h.\n\nWhat is the average speed for the entire round trip in km/h?",
            speed1, speed2
        );

        let solution = format!(
            "**Formula:** For equal distances, \\( \\text{{Average Speed}} = \\frac{{2xy}}{{x + y}} \\)\n\n\
             \\[ \\text{{Average Speed}} = \\frac{{2 \\times {:.0} \\times {:.0}}}{{{:.0} + {:.0}}} = \\frac{{{:.0}}}{{{:.0}}} = **{:.0}** \\text{{ km/h}} \\]",
            speed1, speed2, speed1, speed2, 2.0 * speed1 * speed2, speed1 + speed2, avg_speed
        );

        let parameters = serde_json::json!({
            "variant": "average_speed",
            "speed1": speed1,
            "speed2": speed2,
            "avg_speed": avg_speed,
        });

        let correct_answer = serde_json::json!({
            "value": avg_speed,
            "formatted": format!("{:.0}", avg_speed),
            "unit": "km/h",
            "solution": solution,
        });

        let metadata = serde_json::json!({
            "difficulty": 5.0,
            "target_time_ms": 65_000,
            "generator": TEMPLATE_AVERAGE_V1,
        });

        let step1 = crate::problems::steps::StepNode::new(
            "calc_harmonic_avg",
            crate::problems::steps::StepType::FinalAnswer,
            "Apply harmonic mean formula 2*s1*s2/(s1+s2)",
            format!("(2 * {} * {}) / ({} + {}) = {:.0}", speed1, speed2, speed1, speed2, avg_speed),
            format!("{:.0}", avg_speed),
        )
        .with_expected_value(avg_speed)
        .as_final();

        let graph = crate::problems::steps::SolutionGraph::new(vec![step1], "calc_harmonic_avg");

        ProblemInstance::new(
            format!("inst-avg-5-{}", seed),
            FAMILY_AVERAGE,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(metadata)
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
            1 => 30_000,
            2 => 40_000,
            3 => 50_000,
            4 => 60_000,
            _ => 65_000,
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

impl AverageValidator {
    fn classify_misconception(
        student_val: f64,
        params: &serde_json::Value,
        expected_val: f64,
    ) -> (ErrorCategory, String) {
        let variant = params.get("variant").and_then(|v| v.as_str()).unwrap_or("");

        // Arithmetic mean error for speed in level 5 (student calculated (x+y)/2 instead of harmonic mean 2xy/(x+y))
        if variant == "average_speed" {
            if let (Some(s1), Some(s2)) = (
                params.get("speed1").and_then(|v| v.as_f64()),
                params.get("speed2").and_then(|v| v.as_f64()),
            ) {
                let wrong_arithmetic_mean = (s1 + s2) / 2.0;
                if (student_val - wrong_arithmetic_mean).abs() <= 0.01 {
                    return (
                        ErrorCategory::Concept,
                        "Harmonic vs Arithmetic mean error: Directly averaged the two speeds without accounting for time spent at each speed.".to_string(),
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
    fn test_average_generation_all_levels() {
        let gen = AverageGenerator;
        for lvl in 1..=5 {
            let inst = gen
                .generate(&ProblemFamilyId::new(FAMILY_AVERAGE), 777, lvl, None)
                .unwrap();
            assert!(!inst.rendered_prompt.is_empty());
            assert!(inst.correct_answer.get("value").is_some());
        }
    }

    #[test]
    fn test_average_speed_concept_fallacy_diagnostic() {
        let validator = AverageValidator;
        let gen = AverageGenerator;
        let inst = gen
            .generate(&ProblemFamilyId::new(FAMILY_AVERAGE), 42, 5, Some("average_speed"))
            .unwrap();

        // 40 and 60 arithmetic mean is 50. Harmonic is 48.
        let eval = validator.evaluate(&inst, &serde_json::json!(50.0), 20000, 65000);
        assert!(!eval.is_correct);
        assert_eq!(eval.error_category, Some(ErrorCategory::Concept));
        assert!(eval.diagnostic_message.unwrap().contains("Harmonic vs Arithmetic"));
    }
}
