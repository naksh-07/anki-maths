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
use crate::reasoning::logic_dag::LogicDagPuzzle;
use crate::reasoning::models::{ReasoningProblemMetadata, SchemaKind, StrategyKind};

pub const FAMILY_REASONING_LOGIC_DAG: &str = "family.reasoning.logic_dag.deduction";
pub const TEMPLATE_REASONING_LOGIC_DAG_V1: &str = "reasoning.logic_dag.deduction.v1";

/// Generator for Deductive Multi-Premise Logic DAG reasoning problems.
pub struct LogicDagGenerator;

impl LogicDagGenerator {
    pub fn generate_problem(seed: u64, difficulty_level: u32, variant: Option<&str>) -> ProblemInstance {
        let mut rng = StdRng::seed_from_u64(seed);
        let is_strategy_drill = variant == Some("strategy_drill") || variant == Some("decision_point");

        let puzzle = LogicDagPuzzle::generate_dynamic(&mut rng, difficulty_level);

        let premises_formatted: Vec<String> = puzzle
            .premises_text
            .iter()
            .enumerate()
            .map(|(i, p)| format!("{}. {}", i + 1, p))
            .collect();

        let options_formatted: Vec<String> = puzzle
            .options
            .iter()
            .enumerate()
            .map(|(i, opt)| format!("({}) {}", (b'A' + i as u8) as char, opt))
            .collect();

        let prompt = format!(
            "**Logical Premises:**\n{}\n\n\
            **Question:**\n{}\n\n\
            **Options:**\n{}",
            premises_formatted.join("\n"),
            puzzle.target_query,
            options_formatted.join("\n")
        );

        let dp = CognitiveDecisionPoint::new(
            "dp_logic_dag_premise",
            "Which premise or deduction rule should be applied first?",
            vec![
                DecisionOption::new(
                    "opt_atomic_unconditional",
                    "Begin with the categorical/unconditional ground fact or disjunction",
                    StrategyKind::DirectSyllogisticDeduction.as_str(),
                    true,
                    "Starting with the unconditional ground premise (or negated leaf) allows deterministic forward/backward propagation.",
                ),
                DecisionOption::new(
                    "opt_unfounded_assumption",
                    "Assume arbitrary truth values without grounding in premises",
                    StrategyKind::BranchCases.as_str(),
                    false,
                    "Sub-optimal: Unfounded assumptions introduce false paths without semantic entailment.",
                ),
            ],
            "opt_atomic_unconditional",
            StrategyKind::DirectSyllogisticDeduction.as_str(),
            "Always identify the unconditional fact or negated premise to trigger Modus Ponens or Modus Tollens.",
        );

        let mut meta = ReasoningProblemMetadata::new(SchemaKind::LogicDag, StrategyKind::DirectSyllogisticDeduction)
            .with_decision_point(dp)
            .with_constraint_count(puzzle.premises_text.len());

        if is_strategy_drill {
            meta = meta.as_strategy_drill();
        }

        let step1 = StepNode::new(
            "identify_ground_premise",
            StepType::ApplyConstraint,
            "Identify Ground Premise",
            "Locate the unconditional truth assertion or disjunction from premises.",
            puzzle.premises_formal.last().cloned().unwrap_or_default(),
        )
        .with_hints(vec![
            StepHint::new(HintLevel::Principle, "Deduction Rule", "Find the atomic statement that is asserted unconditionally."),
            StepHint::new(HintLevel::Operation, "Rule Application", "Apply Modus Ponens or Modus Tollens to derive the first intermediate lemma."),
            StepHint::new(HintLevel::IntermediateRelation, "Intermediate Lemma", puzzle.intermediate_lemmas.first().cloned().unwrap_or_default()),
        ]);

        let step2 = StepNode::new(
            "propagate_dag_chain",
            StepType::PropagateConstraint,
            "Propagate Logic DAG",
            "Chain intermediate conclusions through conditional implications.",
            puzzle.intermediate_lemmas.join(" -> "),
        )
        .with_dependencies(vec!["identify_ground_premise".to_string()]);

        let step3 = StepNode::new(
            "derive_valid_conclusion",
            StepType::FinalAnswer,
            "Derive Target Conclusion",
            format!("Logically necessary conclusion: {}", puzzle.target_answer),
            puzzle.target_answer.clone(),
        )
        .with_dependencies(vec!["propagate_dag_chain".to_string()])
        .as_final();

        let solution_graph = SolutionGraph::new(vec![step1, step2, step3], "derive_valid_conclusion");

        let parameters = json!({
            "difficulty": difficulty_level,
            "premises_formal": puzzle.premises_formal,
            "options": puzzle.options,
            "reasoning_metadata": meta,
        });

        let correct_answer = json!({
            "value": puzzle.target_answer,
            "formatted": puzzle.target_answer,
            "options": puzzle.options,
            "derivation_steps": puzzle.derivation_steps,
        });

        let instance_id = format!("inst-ld-l{}-{}", difficulty_level, seed);

        ProblemInstance::new(
            instance_id,
            FAMILY_REASONING_LOGIC_DAG,
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
            "generator": TEMPLATE_REASONING_LOGIC_DAG_V1,
        }))
    }
}

impl ProblemGenerator for LogicDagGenerator {
    fn family_id(&self) -> &str {
        FAMILY_REASONING_LOGIC_DAG
    }

    fn template_ref(&self) -> &str {
        TEMPLATE_REASONING_LOGIC_DAG_V1
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

/// Validator for Logic DAG reasoning problems.
pub struct LogicDagValidator;

impl ProblemValidator for LogicDagValidator {
    fn family_id(&self) -> &str {
        FAMILY_REASONING_LOGIC_DAG
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

        // Check if student selected letter (A, B, C, D)
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
                    "Logical Fallacy: The necessary conclusion is '{}', but received '{}'.",
                    expected, clean_student
                ),
            )
        }
    }
}
