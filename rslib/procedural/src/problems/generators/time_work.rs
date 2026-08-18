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

pub const FAMILY_TIME_WORK: &str = "family.math.time_work.basic";
pub const TEMPLATE_TIME_WORK_V1: &str = "math.time_work.basic.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeWorkVariant {
    SingleWorkerRate,
    TwoWorkersTogether,
    WorkerLeavesEarly,
    RelativeEfficiency,
    PipesInletOutlet,
}

impl TimeWorkVariant {
    pub fn as_str(&self) -> &'static str {
        match self {
            TimeWorkVariant::SingleWorkerRate => "single_worker_rate",
            TimeWorkVariant::TwoWorkersTogether => "two_workers_together",
            TimeWorkVariant::WorkerLeavesEarly => "worker_leaves_early",
            TimeWorkVariant::RelativeEfficiency => "relative_efficiency",
            TimeWorkVariant::PipesInletOutlet => "pipes_inlet_outlet",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TimeWorkGenerator;

impl TimeWorkGenerator {
    pub fn generate_problem(seed: u64, difficulty_level: u32, variant: Option<&str>) -> ProblemInstance {
        let mut rng = StdRng::seed_from_u64(seed);
        let chosen_variant = if let Some(v) = variant {
            match v {
                "single_worker_rate" => TimeWorkVariant::SingleWorkerRate,
                "two_workers_together" => TimeWorkVariant::TwoWorkersTogether,
                "worker_leaves_early" => TimeWorkVariant::WorkerLeavesEarly,
                "relative_efficiency" => TimeWorkVariant::RelativeEfficiency,
                "pipes_inlet_outlet" => TimeWorkVariant::PipesInletOutlet,
                _ => TimeWorkVariant::TwoWorkersTogether,
            }
        } else {
            match difficulty_level {
                1 => TimeWorkVariant::SingleWorkerRate,
                2 => TimeWorkVariant::TwoWorkersTogether,
                3 => TimeWorkVariant::WorkerLeavesEarly,
                4 => TimeWorkVariant::RelativeEfficiency,
                _ => TimeWorkVariant::PipesInletOutlet,
            }
        };

        match chosen_variant {
            TimeWorkVariant::SingleWorkerRate => Self::generate_level_1(&mut rng, seed),
            TimeWorkVariant::TwoWorkersTogether => Self::generate_level_2(&mut rng, seed),
            TimeWorkVariant::WorkerLeavesEarly => Self::generate_level_3(&mut rng, seed),
            TimeWorkVariant::RelativeEfficiency => Self::generate_level_4(&mut rng, seed),
            TimeWorkVariant::PipesInletOutlet => Self::generate_level_5(&mut rng, seed),
        }
    }

    /// Level 1: Single worker rate: A finishes in N days, how many days for fraction of work?
    fn generate_level_1(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let total_days = rng.random_range(12..=36);
        let part_k = 3; // 1/3 of the work
        let required_days = total_days / part_k;

        let prompt = format!(
            "Alice can complete a full project alone in {} days.\n\nHow many days will she take to complete 1/3 of the project at the same rate?",
            total_days
        );

        let solution = format!(
            "**Step 1:** Work done per day = \\(1/{}\\)\n\n\
             **Step 2:** Days required for \\(1/3\\) work = \\( \\frac{{1/3}}{{1/{}}} = \\frac{{{}}}{{3}} = **{}** \\text{{ days}} \\)",
            total_days, total_days, total_days, required_days
        );

        let parameters = serde_json::json!({
            "variant": "single_worker_rate",
            "total_days": total_days,
            "required_days": required_days,
        });

        let correct_answer = serde_json::json!({
            "value": required_days as f64,
            "formatted": format!("{}", required_days),
            "unit": "days",
            "solution": solution,
        });

        let metadata = serde_json::json!({
            "difficulty": 1.0,
            "target_time_ms": 20_000,
            "generator": TEMPLATE_TIME_WORK_V1,
        });

        let step1 = crate::problems::steps::StepNode::new(
            "calc_fraction_days",
            crate::problems::steps::StepType::FinalAnswer,
            "Multiply days by fraction",
            format!("(1/2) * {} = {}", total_days, required_days),
            format!("{}", required_days),
        )
        .with_expected_value(required_days as f64)
        .as_final();

        let graph = crate::problems::steps::SolutionGraph::new(vec![step1], "calc_fraction_days");

        ProblemInstance::new(
            format!("inst-work-1-{}", seed),
            FAMILY_TIME_WORK,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(metadata)
    }

    /// Level 2: Two workers together: A in X days, B in Y days
    fn generate_level_2(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        // Pairs with integer or clean answers: (10, 15 => 6), (12, 24 => 8), (20, 30 => 12), (15, 30 => 10), (12, 6 => 4)
        let pairs = [(10.0, 15.0, 6.0), (12.0, 24.0, 8.0), (20.0, 30.0, 12.0), (15.0, 30.0, 10.0), (12.0, 6.0, 4.0), (18.0, 9.0, 6.0)];
        let idx = rng.random_range(0..pairs.len());
        let (days_a, days_b, together_days) = pairs[idx];

        let prompt = format!(
            "Worker A can complete a task in {:.0} days and Worker B can complete the same task in {:.0} days.\n\nHow many days will they take to complete the task working together?",
            days_a, days_b
        );

        let solution = format!(
            "**Step 1:** Work done per day:\n\
             A's 1-day work = \\(1/{:.0}\\)\n\
             B's 1-day work = \\(1/{:.0}\\)\n\n\
             **Step 2:** Combined 1-day rate:\n\
             \\[ \\frac{{1}}{{{:.0}}} + \\frac{{1}}{{{:.0}}} = \\frac{{{:.0} + {:.0}}}{{{:.0} \\times {:.0}}} = \\frac{{{:.0}}}{{{:.0}}} = \\frac{{1}}{{{:.0}}} \\]\n\n\
             **Step 3:** Total time together = **{:.0}** days.",
            days_a, days_b, days_a, days_b, days_b, days_a, days_a, days_b, days_a + days_b, days_a * days_b, together_days, together_days
        );

        let parameters = serde_json::json!({
            "variant": "two_workers_together",
            "days_a": days_a,
            "days_b": days_b,
            "together_days": together_days,
        });

        let correct_answer = serde_json::json!({
            "value": together_days,
            "formatted": format!("{:.0}", together_days),
            "unit": "days",
            "solution": solution,
        });

        let metadata = serde_json::json!({
            "difficulty": 2.0,
            "target_time_ms": 35_000,
            "generator": TEMPLATE_TIME_WORK_V1,
        });

        let step1 = crate::problems::steps::StepNode::new(
            "calc_together_days",
            crate::problems::steps::StepType::FinalAnswer,
            "Apply reciprocal work formula (a*b)/(a+b)",
            format!("({} * {}) / ({} + {}) = {:.0}", days_a, days_b, days_a, days_b, together_days),
            format!("{:.0}", together_days),
        )
        .with_expected_value(together_days)
        .as_final();

        let graph = crate::problems::steps::SolutionGraph::new(vec![step1], "calc_together_days");

        ProblemInstance::new(
            format!("inst-work-2-{}", seed),
            FAMILY_TIME_WORK,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(metadata)
    }

    /// Level 3: Worker leaves early
    fn generate_level_3(_rng: &mut StdRng, seed: u64) -> ProblemInstance {
        // A in 20 days, B in 30 days. Work together for 5 days. Then A leaves.
        // Total work = 60 units. A rate = 3/day, B rate = 2/day.
        // In 5 days: (3+2)*5 = 25 units done. Remaining = 35 units.
        // B finishes in 35/2 = 17.5 days.
        // Let's use clean numbers: A in 12 days, B in 18 days => Total = 36 units. A = 3, B = 2.
        // Together 4 days => 5*4 = 20 units. Remaining = 16 units. B takes 16/2 = 8 days.
        let days_a = 12.0;
        let days_b = 18.0;
        let together_worked = 4.0;
        let remaining_days_b = 8.0;

        let prompt = format!(
            "A can finish a work in {:.0} days and B in {:.0} days. They start working together, but after {:.0} days A leaves. How many more days will B take to complete the remaining work alone?",
            days_a, days_b, together_worked
        );

        let solution = format!(
            "**Step 1:** Total work = LCM(12, 18) = 36 units.\n\
             A's efficiency = 36/12 = 3 units/day.\n\
             B's efficiency = 36/18 = 2 units/day.\n\n\
             **Step 2:** Work completed in 4 days together:\n\
             (3 + 2) × 4 = 20 units.\n\n\
             **Step 3:** Remaining work = 36 - 20 = 16 units.\n\
             Days B takes alone = 16 / 2 = **{:.0}** days.",
            remaining_days_b
        );

        let parameters = serde_json::json!({
            "variant": "worker_leaves_early",
            "days_a": days_a,
            "days_b": days_b,
            "together_worked": together_worked,
            "remaining_days_b": remaining_days_b,
        });

        let correct_answer = serde_json::json!({
            "value": remaining_days_b,
            "formatted": format!("{:.0}", remaining_days_b),
            "unit": "days",
            "solution": solution,
        });

        let metadata = serde_json::json!({
            "difficulty": 3.0,
            "target_time_ms": 50_000,
            "generator": TEMPLATE_TIME_WORK_V1,
        });

        let step1 = crate::problems::steps::StepNode::new(
            "calc_remaining_work",
            crate::problems::steps::StepType::IntermediateResult,
            "Calculate remaining units after joint work",
            "36 - (3 + 2)*4 = 16".to_string(),
            "16".to_string(),
        )
        .with_expected_value(16.0);

        let step2 = crate::problems::steps::StepNode::new(
            "calc_b_alone_days",
            crate::problems::steps::StepType::FinalAnswer,
            "Divide remaining units by B's rate",
            format!("16 / 2 = {:.0}", remaining_days_b),
            format!("{:.0}", remaining_days_b),
        )
        .with_expected_value(remaining_days_b)
        .with_dependencies(vec!["calc_remaining_work".to_string()])
        .as_final();

        let graph = crate::problems::steps::SolutionGraph::new(vec![step1, step2], "calc_b_alone_days");

        ProblemInstance::new(
            format!("inst-work-3-{}", seed),
            FAMILY_TIME_WORK,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(metadata)
    }

    /// Level 4: Relative Efficiency (A is 2x as efficient as B)
    fn generate_level_4(_rng: &mut StdRng, seed: u64) -> ProblemInstance {
        // A is twice as fast as B. Together they take 12 days.
        // A = 2 units/day, B = 1 unit/day => Combined = 3 units/day.
        // Total work = 3 * 12 = 36 units.
        // A alone takes 36 / 2 = 18 days.
        let eff_ratio = 2.0;
        let together_days = 12.0;
        let days_a_alone = 18.0;

        let prompt = format!(
            "Worker A is twice as efficient as Worker B. Working together, they can finish a job in {:.0} days. In how many days can Worker A finish the job working alone?",
            together_days
        );

        let solution = format!(
            "**Step 1:** Ratio of efficiency \\(A:B = 2:1\\).\n\
             Combined daily efficiency = 2 + 1 = 3 units/day.\n\n\
             **Step 2:** Total work = 3 units/day × {:.0} days = 36 units.\n\n\
             **Step 3:** Time for A alone = 36 / 2 = **{:.0}** days.",
            together_days, days_a_alone
        );

        let parameters = serde_json::json!({
            "variant": "relative_efficiency",
            "eff_ratio": eff_ratio,
            "together_days": together_days,
            "days_a_alone": days_a_alone,
        });

        let correct_answer = serde_json::json!({
            "value": days_a_alone,
            "formatted": format!("{:.0}", days_a_alone),
            "unit": "days",
            "solution": solution,
        });

        let metadata = serde_json::json!({
            "difficulty": 4.0,
            "target_time_ms": 60_000,
            "generator": TEMPLATE_TIME_WORK_V1,
        });

        let step1 = crate::problems::steps::StepNode::new(
            "calc_a_alone",
            crate::problems::steps::StepType::FinalAnswer,
            "Calculate A's time from combined work and ratio",
            format!("(3 * {}) / 2 = {:.0}", together_days, days_a_alone),
            format!("{:.0}", days_a_alone),
        )
        .with_expected_value(days_a_alone)
        .as_final();

        let graph = crate::problems::steps::SolutionGraph::new(vec![step1], "calc_a_alone");

        ProblemInstance::new(
            format!("inst-work-4-{}", seed),
            FAMILY_TIME_WORK,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(metadata)
    }

    /// Level 5: Pipes & Cisterns (Inlet + Outlet)
    fn generate_level_5(_rng: &mut StdRng, seed: u64) -> ProblemInstance {
        // Pipe A fills in 10 hours, Pipe B empties in 15 hours.
        // Capacity = 30 units. A rate = +3/hr, B rate = -2/hr. Net rate = +1/hr.
        // Time to fill = 30 / 1 = 30 hours.
        let fill_hours = 10.0;
        let empty_hours = 15.0;
        let net_hours = 30.0;

        let prompt = format!(
            "Pipe A can fill a tank in {:.0} hours, while Pipe B can empty the full tank in {:.0} hours. If both pipes are opened simultaneously, in how many hours will the empty tank be completely filled?",
            fill_hours, empty_hours
        );

        let solution = format!(
            "**Step 1:** In 1 hour:\n\
             Pipe A fills \\(1/{:.0}\\) of the tank.\n\
             Pipe B empties \\(1/{:.0}\\) of the tank.\n\n\
             **Step 2:** Net filling per hour:\n\
             \\[ \\frac{{1}}{{{:.0}}} - \\frac{{1}}{{{:.0}}} = \\frac{{3 - 2}}{{30}} = \\frac{{1}}{{30}} \\]\n\n\
             **Step 3:** Time required = **{:.0}** hours.",
            fill_hours, empty_hours, fill_hours, empty_hours, net_hours
        );

        let parameters = serde_json::json!({
            "variant": "pipes_inlet_outlet",
            "fill_hours": fill_hours,
            "empty_hours": empty_hours,
            "net_hours": net_hours,
        });

        let correct_answer = serde_json::json!({
            "value": net_hours,
            "formatted": format!("{:.0}", net_hours),
            "unit": "hours",
            "solution": solution,
        });

        let metadata = serde_json::json!({
            "difficulty": 5.0,
            "target_time_ms": 70_000,
            "generator": TEMPLATE_TIME_WORK_V1,
        });

        let step1 = crate::problems::steps::StepNode::new(
            "calc_net_pipes",
            crate::problems::steps::StepType::FinalAnswer,
            "Calculate net filling time 1/(1/A - 1/B)",
            format!("1 / (1/{} - 1/{}) = {:.0}", fill_hours, empty_hours, net_hours),
            format!("{:.0}", net_hours),
        )
        .with_expected_value(net_hours)
        .as_final();

        let graph = crate::problems::steps::SolutionGraph::new(vec![step1], "calc_net_pipes");

        ProblemInstance::new(
            format!("inst-work-5-{}", seed),
            FAMILY_TIME_WORK,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(metadata)
    }
}

impl ProblemGenerator for TimeWorkGenerator {
    fn family_id(&self) -> &str {
        FAMILY_TIME_WORK
    }

    fn template_ref(&self) -> &str {
        TEMPLATE_TIME_WORK_V1
    }

    fn supported_variants(&self) -> Vec<String> {
        vec![
            "single_worker_rate".to_string(),
            "two_workers_together".to_string(),
            "worker_leaves_early".to_string(),
            "relative_efficiency".to_string(),
            "pipes_inlet_outlet".to_string(),
        ]
    }

    fn target_latency_ms(&self, difficulty_level: u32) -> u64 {
        match difficulty_level {
            1 => 20_000,
            2 => 35_000,
            3 => 50_000,
            4 => 60_000,
            _ => 70_000,
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

pub struct TimeWorkValidator;

impl ProblemValidator for TimeWorkValidator {
    fn family_id(&self) -> &str {
        FAMILY_TIME_WORK
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

impl TimeWorkValidator {
    fn classify_misconception(
        student_val: f64,
        params: &serde_json::Value,
        expected_val: f64,
    ) -> (ErrorCategory, String) {
        let variant = params.get("variant").and_then(|v| v.as_str()).unwrap_or("");

        // Reciprocal / rate setup error in level 2 (student simply added the days: A + B)
        if variant == "two_workers_together" {
            if let (Some(da), Some(db)) = (
                params.get("days_a").and_then(|v| v.as_f64()),
                params.get("days_b").and_then(|v| v.as_f64()),
            ) {
                if (student_val - (da + db)).abs() <= 0.01 {
                    return (
                        ErrorCategory::Concept,
                        "Reciprocal rate error: Added individual days directly (A + B) instead of combining work rates (1/A + 1/B).".to_string(),
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
    fn test_time_work_generation_all_levels() {
        let gen = TimeWorkGenerator;
        for lvl in 1..=5 {
            let inst = gen
                .generate(&ProblemFamilyId::new(FAMILY_TIME_WORK), 555, lvl, None)
                .unwrap();
            assert!(!inst.rendered_prompt.is_empty());
            assert!(inst.correct_answer.get("value").is_some());
        }
    }

    #[test]
    fn test_time_work_reciprocal_error_diagnostic() {
        let validator = TimeWorkValidator;
        let gen = TimeWorkGenerator;
        let inst = gen
            .generate(&ProblemFamilyId::new(FAMILY_TIME_WORK), 123, 2, Some("two_workers_together"))
            .unwrap();

        let da = inst.parameters.get("days_a").unwrap().as_f64().unwrap();
        let db = inst.parameters.get("days_b").unwrap().as_f64().unwrap();

        let eval = validator.evaluate(&inst, &serde_json::json!(da + db), 15000, 35000);
        assert!(!eval.is_correct);
        assert_eq!(eval.error_category, Some(ErrorCategory::Concept));
        assert!(eval.diagnostic_message.unwrap().contains("Reciprocal rate error"));
    }
}
