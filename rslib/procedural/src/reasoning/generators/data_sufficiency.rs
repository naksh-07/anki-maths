// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use rand::rngs::StdRng;
use rand::SeedableRng;
use serde_json::json;

use crate::core::{ProblemFamilyId, Result};
use crate::core::decision::{CognitiveDecisionPoint, DecisionOption};
use crate::diagnostics::ErrorCategory;
use crate::problems::generator::ProblemGenerator;
use crate::problems::steps::{HintLevel, SolutionGraph, StepHint, StepNode, StepType};
use crate::problems::validator::{AnswerEvaluation, ProblemValidator};
use crate::problems::ProblemInstance;
use crate::reasoning::data_sufficiency::{DataSufficiencyPuzzle, DsAnswer};
use crate::reasoning::models::{ReasoningProblemMetadata, SchemaKind, StrategyKind};

pub const FAMILY_REASONING_DATA_SUFFICIENCY: &str = "family.reasoning.data_sufficiency.determinacy";
pub const TEMPLATE_REASONING_DATA_SUFFICIENCY_V1: &str = "reasoning.data_sufficiency.determinacy.v1";

/// Generator for Meta-Cognitive Data Sufficiency & Determinacy problems.
pub struct DataSufficiencyGenerator;

impl DataSufficiencyGenerator {
    pub fn target_latency(difficulty_level: u32) -> u64 {
        match difficulty_level {
            1 => 25_000,
            2 => 30_000,
            3 => 35_000,
            4 => 45_000,
            _ => 55_000,
        }
    }

    pub fn generate_problem(seed: u64, difficulty_level: u32, variant: Option<&str>) -> ProblemInstance {
        let mut rng = StdRng::seed_from_u64(seed);
        let is_strategy_drill = variant == Some("strategy_drill") || variant == Some("decision_point");
        let target_time_ms = Self::target_latency(difficulty_level);

        let puzzle = DataSufficiencyPuzzle::generate_dynamic(&mut rng, difficulty_level);

        let options_list = [
            DsAnswer::Statement1Alone.full_description(),
            DsAnswer::Statement2Alone.full_description(),
            DsAnswer::BothTogether.full_description(),
            DsAnswer::EachAlone.full_description(),
            DsAnswer::NeitherSufficient.full_description(),
        ];

        let prompt = format!(
            "**Problem:**\n{}\n\n\
            {}\n\
            {}\n\n\
            **Directions:** Select the correct determinacy option:\n\
            {}\n\
            {}\n\
            {}\n\
            {}\n\
            {}",
            puzzle.problem_prompt,
            puzzle.statement_1,
            puzzle.statement_2,
            options_list[0],
            options_list[1],
            options_list[2],
            options_list[3],
            options_list[4]
        );

        let correct_letter = puzzle.correct_answer.letter().to_string();

        let dp = CognitiveDecisionPoint::new(
            "dp_ds_evaluation_strategy",
            "What is the optimal systematic evaluation procedure for this Data Sufficiency problem?",
            vec![
                DecisionOption::new(
                    "opt_independent_then_combined",
                    "Evaluate Statement (1) alone first, then Statement (2) alone independently, before combining",
                    StrategyKind::EliminateInvalid.as_str(),
                    true,
                    "Evaluating statements independently first eliminates false assumptions from accidental information leakage.",
                ),
                DecisionOption::new(
                    "opt_assume_both_immediately",
                    "Immediately assume both statements together without testing individually",
                    StrategyKind::BranchCases.as_str(),
                    false,
                    "Sub-optimal: Assuming both statements together immediately causes false 'C' traps when 'A', 'B', or 'D' is correct.",
                ),
            ],
            "opt_independent_then_combined",
            StrategyKind::EliminateInvalid.as_str(),
            "Always test each statement independently before attempting to combine them.",
        );

        let mut meta = ReasoningProblemMetadata::new(SchemaKind::DataSufficiency, StrategyKind::EliminateInvalid)
            .with_decision_point(dp)
            .with_constraint_count(2);

        if is_strategy_drill {
            meta = meta.as_strategy_drill();
        }

        let step1 = StepNode::new(
            "eval_statement_1",
            StepType::ApplyConstraint,
            "Evaluate Statement (1) Alone",
            format!("Check whether {} uniquely determines the target question.", puzzle.statement_1),
            format!("Statement 1 Sufficiency Check: {:?}", puzzle.correct_answer == DsAnswer::Statement1Alone || puzzle.correct_answer == DsAnswer::EachAlone),
        )
        .with_hints(vec![
            StepHint::new(HintLevel::Principle, "Sufficiency Rule", "Test if Statement (1) yields exactly one unique answer or multiple values."),
            StepHint::new(HintLevel::Operation, "Statement 1 Check", "Solve for the variable or test counterexamples using only Statement (1)."),
            StepHint::new(HintLevel::IntermediateRelation, "Statement 1 Result", if puzzle.correct_answer == DsAnswer::Statement1Alone || puzzle.correct_answer == DsAnswer::EachAlone { "Sufficient alone" } else { "Insufficient alone" }.to_string()),
        ]);

        let step2 = StepNode::new(
            "eval_statement_2",
            StepType::ApplyConstraint,
            "Evaluate Statement (2) Alone",
            format!("Independently check whether {} uniquely determines the target question.", puzzle.statement_2),
            format!("Statement 2 Sufficiency Check: {:?}", puzzle.correct_answer == DsAnswer::Statement2Alone || puzzle.correct_answer == DsAnswer::EachAlone),
        )
        .with_dependencies(vec!["eval_statement_1".to_string()]);

        let step3 = StepNode::new(
            "conclude_determinacy",
            StepType::FinalAnswer,
            "Conclude Overall Sufficiency",
            puzzle.explanation.clone(),
            correct_letter.clone(),
        )
        .with_dependencies(vec!["eval_statement_2".to_string()])
        .as_final();

        let solution_graph = SolutionGraph::new(vec![step1, step2, step3], "conclude_determinacy");

        let parameters = json!({
            "difficulty": difficulty_level,
            "statement_1": puzzle.statement_1,
            "statement_2": puzzle.statement_2,
            "reasoning_metadata": meta,
        });

        let correct_answer = json!({
            "value": correct_letter,
            "formatted": correct_letter,
            "full_answer_text": puzzle.correct_answer.full_description(),
            "explanation": puzzle.explanation,
        });

        let instance_id = format!("inst-ds-l{}-{}", difficulty_level, seed);

        ProblemInstance::new(
            instance_id,
            FAMILY_REASONING_DATA_SUFFICIENCY,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(solution_graph)
        .with_metadata(json!({
            "difficulty_level": difficulty_level,
            "target_time_ms": target_time_ms,
            "domain": "reasoning",
            "generator": TEMPLATE_REASONING_DATA_SUFFICIENCY_V1,
        }))
    }
}

impl ProblemGenerator for DataSufficiencyGenerator {
    fn family_id(&self) -> &str {
        FAMILY_REASONING_DATA_SUFFICIENCY
    }

    fn template_ref(&self) -> &str {
        TEMPLATE_REASONING_DATA_SUFFICIENCY_V1
    }

    fn supported_variants(&self) -> Vec<String> {
        vec!["default".to_string(), "strategy_drill".to_string()]
    }

    fn target_latency_ms(&self, difficulty_level: u32) -> u64 {
        Self::target_latency(difficulty_level)
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

/// Validator for Data Sufficiency problems.
pub struct DataSufficiencyValidator;

impl ProblemValidator for DataSufficiencyValidator {
    fn family_id(&self) -> &str {
        FAMILY_REASONING_DATA_SUFFICIENCY
    }

    fn evaluate(
        &self,
        instance: &ProblemInstance,
        student_answer: &serde_json::Value,
        time_taken_ms: u64,
        target_time_ms: u64,
    ) -> AnswerEvaluation {
        let expected = instance
            .correct_answer
            .get("value")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim();

        let clean_student = match student_answer {
            serde_json::Value::String(s) => s.trim().to_uppercase(),
            serde_json::Value::Object(map) => {
                map.get("formatted")
                    .or_else(|| map.get("value"))
                    .or_else(|| map.get("answer"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .trim()
                    .to_uppercase()
            }
            _ => "".to_string(),
        };

        let is_correct = clean_student == expected
            || clean_student == format!("({})", expected)
            || clean_student.starts_with(expected);

        if is_correct {
            let score = if time_taken_ms <= target_time_ms {
                1.0
            } else {
                (1.0 - ((time_taken_ms - target_time_ms) as f64 / target_time_ms as f64) * 0.5).max(0.5)
            };
            AnswerEvaluation::correct(score, time_taken_ms, target_time_ms)
        } else {
            AnswerEvaluation::incorrect(
                ErrorCategory::Concept,
                format!(
                    "Data Sufficiency Error: Expected ({}), but received '{}'.",
                    expected, clean_student
                ),
            )
        }
    }
}
