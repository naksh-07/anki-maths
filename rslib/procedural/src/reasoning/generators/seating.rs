// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use rand::rngs::StdRng;
use rand::SeedableRng;
use serde_json::json;

use crate::core::{ProblemFamilyId, Result};
use crate::diagnostics::ErrorCategory;
use crate::problems::generator::ProblemGenerator;
use crate::problems::steps::{HintLevel, SolutionGraph, StepHint, StepNode, StepType};
use crate::problems::validator::{AnswerEvaluation, ProblemValidator};
use crate::problems::ProblemInstance;
use crate::reasoning::generators::{FAMILY_REASONING_SEATING, TEMPLATE_REASONING_SEATING_V1};
use crate::core::decision::{CognitiveDecisionPoint, DecisionOption};
use crate::reasoning::models::{ReasoningProblemMetadata, SchemaKind, StrategyKind};
use crate::reasoning::seating::SeatingPuzzle;

/// Generator for Linear Seating Arrangement CSP problems.
pub struct SeatingGenerator;

impl SeatingGenerator {
    pub fn generate_problem(seed: u64, difficulty_level: u32, variant: Option<&str>) -> ProblemInstance {
        let mut rng = StdRng::seed_from_u64(seed);

        let is_strategy_drill = variant == Some("strategy_drill") || variant == Some("decision_point");

        let total_slots = match difficulty_level {
            1 => 4,
            2 => 5,
            3 => 5,
            4 => 6,
            _ => 7,
        };

        let puzzle = SeatingPuzzle::generate_dynamic(&mut rng, total_slots, difficulty_level)
            .unwrap_or_else(|| {
                SeatingPuzzle::build_5person_anchor_puzzle(
                    "Alice", 1, "Bob", "Charlie", &["David", "Emma"], 3,
                ).unwrap()
            });

        let anchor_person = puzzle.anchor_person.clone();
        let query_slot = puzzle.query_slot;
        let target_answer = puzzle.target_answer.clone();

        let conditions_formatted: Vec<String> = puzzle
            .conditions_text
            .iter()
            .enumerate()
            .map(|(i, c)| format!("{}. {}", i + 1, c))
            .collect();

        let prompt = format!(
            "**{} people** ({}) sit in a single row facing North (positions 1 to {} from left to right).\n\n\
            **Conditions:**\n{}\n\n\
            **Question:**\n{}",
            puzzle.total_slots,
            puzzle.people.join(", "),
            puzzle.total_slots,
            conditions_formatted.join("\n"),
            puzzle.target_question
        );

        let dp = CognitiveDecisionPoint::new(
            "dp_seating_anchor",
            "Which condition should be placed onto the arrangement slots first?",
            vec![
                DecisionOption::new(
                    "opt_anchor",
                    "Anchor the person whose absolute position is given",
                    StrategyKind::AnchorFixed.as_str(),
                    true,
                    "Anchoring fixed positions drastically reduces the domain for relative adjacent constraints.",
                ),
                DecisionOption::new(
                    "opt_guess",
                    "Randomly test arbitrary positions for the relative pair",
                    StrategyKind::BranchCases.as_str(),
                    false,
                    "Sub-optimal: Guessing before placing fixed anchors increases unnecessary search branching.",
                ),
            ],
            "opt_anchor",
            StrategyKind::AnchorFixed.as_str(),
            "Always anchor invariant fixed positions before relative adjacent constraints.",
        );

        let mut meta = ReasoningProblemMetadata::new(SchemaKind::LinearSeating, StrategyKind::AnchorFixed)
            .with_decision_point(dp)
            .with_constraint_count(puzzle.conditions_text.len());

        if is_strategy_drill {
            meta = meta.as_strategy_drill();
        }

        let step1 = StepNode::new(
            "place_anchor",
            StepType::ApplyConstraint,
            "Place Fixed Anchor",
            format!("Fix {} at specified invariant anchor slot.", anchor_person),
            format!("Anchor: {}", anchor_person),
        )
        .with_hints(vec![
            StepHint::new(HintLevel::Principle, "Constraint Principle", "Start with the most restrictive or fixed position condition."),
            StepHint::new(HintLevel::Operation, "Strategy Operation", format!("Identify the fixed position for {}.", anchor_person)),
            StepHint::new(HintLevel::IntermediateRelation, "Anchor Placed", format!("Anchor person: {}", anchor_person)),
        ]);

        let step2 = StepNode::new(
            "propagate_relative",
            StepType::PropagateConstraint,
            "Propagate Relative Conditions",
            "Place remaining people into available slots based on adjacency and ordering.",
            format!("Slot {} = {}", query_slot, target_answer),
        )
        .with_dependencies(vec!["place_anchor".to_string()]);

        let step3 = StepNode::new(
            "identify_target",
            StepType::FinalAnswer,
            "Final Answer",
            format!("Identify person at slot {}.", query_slot),
            target_answer.clone(),
        )
        .with_alternates(vec![
            target_answer.to_lowercase(),
            target_answer.to_uppercase(),
        ])
        .with_dependencies(vec!["propagate_relative".to_string()])
        .as_final();

        let graph = SolutionGraph::new(vec![step1, step2, step3], "identify_target");

        let parameters = json!({
            "difficulty": difficulty_level,
            "total_slots": puzzle.total_slots,
            "target_answer": target_answer,
            "people": puzzle.people,
            "query_slot": query_slot,
            "reasoning_metadata": meta,
        });

        let correct_answer = json!({
            "value": target_answer,
            "formatted": target_answer,
            "solution": puzzle.explanation,
        });

        let instance_id = format!("inst-seating-l{}-{}", difficulty_level, seed);

        ProblemInstance::new(
            instance_id,
            FAMILY_REASONING_SEATING,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(json!({
            "difficulty_level": difficulty_level,
            "target_time_ms": 35_000,
            "domain": "reasoning",
            "generator": TEMPLATE_REASONING_SEATING_V1,
        }))
    }
}

impl ProblemGenerator for SeatingGenerator {
    fn family_id(&self) -> &str {
        FAMILY_REASONING_SEATING
    }

    fn template_ref(&self) -> &str {
        TEMPLATE_REASONING_SEATING_V1
    }

    fn supported_variants(&self) -> Vec<String> {
        vec![
            "linear_4person".to_string(),
            "linear_5person_anchor".to_string(),
            "linear_6person_adjacent".to_string(),
            "strategy_drill".to_string(),
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

/// Validator for Linear Seating Arrangement problems.
pub struct SeatingValidator;

impl ProblemValidator for SeatingValidator {
    fn family_id(&self) -> &str {
        FAMILY_REASONING_SEATING
    }

    fn evaluate(
        &self,
        instance: &ProblemInstance,
        student_answer: &serde_json::Value,
        time_taken_ms: u64,
        target_time_ms: u64,
    ) -> AnswerEvaluation {
        let expected_str = instance
            .correct_answer
            .get("formatted")
            .or_else(|| instance.correct_answer.get("value"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let student_str = match student_answer {
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

        let is_correct = !student_str.is_empty() && student_str.eq_ignore_ascii_case(expected_str);

        if is_correct {
            AnswerEvaluation::correct(1.0, time_taken_ms, target_time_ms)
        } else {
            AnswerEvaluation::incorrect(
                ErrorCategory::Strategy,
                format!(
                    "Incorrect person identified. Submitted '{}', expected '{}'.",
                    student_str, expected_str
                ),
            )
        }
    }
}
