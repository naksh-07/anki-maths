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
use crate::reasoning::coded_expressions::CodedExpressionsPuzzle;
use crate::reasoning::models::{ReasoningProblemMetadata, SchemaKind, StrategyKind};

pub const FAMILY_REASONING_CODED_EXPRESSIONS: &str = "family.reasoning.coded_expressions.relations";
pub const TEMPLATE_REASONING_CODED_EXPRESSIONS_V1: &str = "reasoning.coded_expressions.relations.v1";

/// Generator for Symbolic Coded Expressions reasoning problems.
pub struct CodedExpressionsGenerator;

impl CodedExpressionsGenerator {
    pub fn generate_problem(seed: u64, difficulty_level: u32, variant: Option<&str>) -> ProblemInstance {
        let mut rng = StdRng::seed_from_u64(seed);
        let is_strategy_drill = variant == Some("strategy_drill") || variant == Some("decision_point");

        let puzzle = CodedExpressionsPuzzle::generate_dynamic(&mut rng, difficulty_level);

        let operators_formatted: Vec<String> = puzzle
            .operator_definitions
            .iter()
            .enumerate()
            .map(|(i, op)| format!("{}. {}", i + 1, op))
            .collect();

        let options_formatted: Vec<String> = puzzle
            .options
            .iter()
            .enumerate()
            .map(|(i, opt)| format!("({}) {}", (b'A' + i as u8) as char, opt))
            .collect();

        let prompt = format!(
            "**Operator Definitions:**\n{}\n\n\
            **Given Expression:**\n`{}`\n\n\
            **Question:**\n{}\n\n\
            **Options:**\n{}",
            operators_formatted.join("\n"),
            puzzle.given_expression,
            puzzle.target_query,
            options_formatted.join("\n")
        );

        let dp = CognitiveDecisionPoint::new(
            "dp_coded_operator_decode",
            "What is the required strategy to decode the symbolic expression chain?",
            vec![
                DecisionOption::new(
                    "opt_substitute_and_trace",
                    "Decode operators sequentially from left to right into a relational tree or 2D vector graph",
                    StrategyKind::ConstructKinshipGraph.as_str(),
                    true,
                    "Stepwise translation into concrete graph nodes prevents operator transposition errors.",
                ),
                DecisionOption::new(
                    "opt_guess_pattern",
                    "Rely on superficial operator repetition without building a relational graph",
                    StrategyKind::BranchCases.as_str(),
                    false,
                    "Sub-optimal: Guessing relationships without explicit graph construction fails on multi-step chains.",
                ),
            ],
            "opt_substitute_and_trace",
            StrategyKind::ConstructKinshipGraph.as_str(),
            "Always construct the relational tree or coordinate displacement vector step by step.",
        );

        let mut meta = ReasoningProblemMetadata::new(SchemaKind::CodedExpressions, StrategyKind::ConstructKinshipGraph)
            .with_decision_point(dp)
            .with_constraint_count(puzzle.operator_definitions.len());

        if is_strategy_drill {
            meta = meta.as_strategy_drill();
        }

        let step1 = StepNode::new(
            "decode_operators",
            StepType::ApplyConstraint,
            "Decode Symbolic Operators",
            format!("Map symbols in `{}` to concrete relations/vectors.", puzzle.given_expression),
            puzzle.step_by_step_trace.first().cloned().unwrap_or_default(),
        )
        .with_hints(vec![
            StepHint::new(HintLevel::Principle, "Decoding Strategy", "Translate each symbolic token into its semantic relation or direction."),
            StepHint::new(HintLevel::Operation, "First Token Translation", puzzle.step_by_step_trace.first().cloned().unwrap_or_default()),
            StepHint::new(HintLevel::IntermediateRelation, "Graph Seed", "Initialize the root node of the relational graph."),
        ]);

        let step2 = StepNode::new(
            "trace_relational_path",
            StepType::PropagateConstraint,
            "Trace Relational Path",
            "Compose the intermediate relations across all nodes in the expression.",
            puzzle.step_by_step_trace.join(" -> "),
        )
        .with_dependencies(vec!["decode_operators".to_string()]);

        let step3 = StepNode::new(
            "conclude_relationship",
            StepType::FinalAnswer,
            "Conclude Relationship / Vector",
            format!("Target result: {}", puzzle.target_answer),
            puzzle.target_answer.clone(),
        )
        .with_dependencies(vec!["trace_relational_path".to_string()])
        .as_final();

        let solution_graph = SolutionGraph::new(vec![step1, step2, step3], "conclude_relationship");

        let parameters = json!({
            "difficulty": difficulty_level,
            "given_expression": puzzle.given_expression,
            "options": puzzle.options,
            "reasoning_metadata": meta,
        });

        let correct_answer = json!({
            "value": puzzle.target_answer,
            "formatted": puzzle.target_answer,
            "options": puzzle.options,
            "trace": puzzle.step_by_step_trace,
        });

        let instance_id = format!("inst-ce-l{}-{}", difficulty_level, seed);

        ProblemInstance::new(
            instance_id,
            FAMILY_REASONING_CODED_EXPRESSIONS,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(solution_graph)
        .with_metadata(json!({
            "difficulty_level": difficulty_level,
            "target_time_ms": 40_000,
            "domain": "reasoning",
            "generator": TEMPLATE_REASONING_CODED_EXPRESSIONS_V1,
        }))
    }
}

impl ProblemGenerator for CodedExpressionsGenerator {
    fn family_id(&self) -> &str {
        FAMILY_REASONING_CODED_EXPRESSIONS
    }

    fn template_ref(&self) -> &str {
        TEMPLATE_REASONING_CODED_EXPRESSIONS_V1
    }

    fn supported_variants(&self) -> Vec<String> {
        vec!["default".to_string(), "strategy_drill".to_string()]
    }

    fn target_latency_ms(&self, _difficulty_level: u32) -> u64 {
        40_000
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

/// Validator for Coded Expressions reasoning problems.
pub struct CodedExpressionsValidator;

impl ProblemValidator for CodedExpressionsValidator {
    fn family_id(&self) -> &str {
        FAMILY_REASONING_CODED_EXPRESSIONS
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

        let options = instance
            .correct_answer
            .get("options")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let clean_student = match student_answer {
            serde_json::Value::String(s) => s.trim().to_string(),
            serde_json::Value::Object(map) => {
                map.get("formatted")
                    .or_else(|| map.get("value"))
                    .or_else(|| map.get("answer"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .trim()
                    .to_string()
            }
            _ => "".to_string(),
        };

        // Check if student provided letter A, B, C, D
        let mut letter_matched = false;
        if clean_student.len() == 1 {
            let ch = clean_student.chars().next().unwrap().to_ascii_uppercase();
            if ch >= 'A' && ((ch as usize - 'A' as usize) < options.len()) {
                let idx = ch as usize - 'A' as usize;
                if options[idx].eq_ignore_ascii_case(expected) {
                    letter_matched = true;
                }
            }
        }

        let text_matched = clean_student.eq_ignore_ascii_case(expected)
            || expected.to_lowercase().contains(&clean_student.to_lowercase());

        if letter_matched || text_matched {
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
                    "Coded Relation Error: Expected '{}', but received '{}'.",
                    expected, clean_student
                ),
            )
        }
    }
}
