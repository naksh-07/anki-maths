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
use crate::reasoning::models::{CognitiveDecisionPoint, DecisionOption, ReasoningProblemMetadata, SchemaKind, StrategyKind};
use crate::reasoning::seating::SeatingPuzzle;

/// Generator for Linear Seating Arrangement CSP problems.
pub struct SeatingGenerator;

const NAMES: &[&str] = &["Alice", "Bob", "Charlie", "David", "Emma", "Frank", "Grace"];

impl SeatingGenerator {
    pub fn generate_problem(seed: u64, difficulty_level: u32, variant: Option<&str>) -> ProblemInstance {
        let _rng = StdRng::seed_from_u64(seed);

        let is_strategy_drill = variant == Some("strategy_drill") || variant == Some("decision_point");

        let p1 = NAMES[0];
        let p2 = NAMES[1];
        let p3 = NAMES[2];
        let p4 = NAMES[3];
        let p5 = NAMES[4];

        let (puzzle, anchor_person, query_slot) = match difficulty_level {
            1 => {
                // 4 people: Alice at 1, Bob immediately left of Charlie, David at 4 -> Query 3 (Charlie)
                let query = 3;
                let pz = SeatingPuzzle::build_5person_anchor_puzzle(p1, 1, p2, p3, &[p4], query)
                    .unwrap_or_else(|| SeatingPuzzle::build_5person_anchor_puzzle(p1, 1, p2, p3, &[p4], query).unwrap());
                (pz, p1, query)
            }
            2 => {
                // 5 people: Charlie at slot 3, Alice immediately left of Bob, David & Emma remaining
                let query = 2;
                let pz = SeatingPuzzle::build_5person_anchor_puzzle(p1, 1, p2, p3, &[p4, p5], query)
                    .unwrap_or_else(|| SeatingPuzzle::build_5person_anchor_puzzle(p1, 1, p2, p3, &[p4], query).unwrap());
                (pz, p1, query)
            }
            _ => {
                let query = 4;
                let pz = SeatingPuzzle::build_5person_anchor_puzzle(p1, 1, p2, p3, &[p4, p5], query)
                    .unwrap_or_else(|| SeatingPuzzle::build_5person_anchor_puzzle(p1, 1, p2, p3, &[p4], query).unwrap());
                (pz, p1, query)
            }
        };

        let conditions_formatted: Vec<String> = puzzle
            .conditions_text
            .iter()
            .enumerate()
            .map(|(i, c)| format!("{}. {}", i + 1, c))
            .collect();

        let prompt = format!(
            "{} people ({}) sit in a single row facing North (positions 1 to {} from left to right).\n\n\
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
                    format!("Place the fixed anchor: {} at position 1", anchor_person),
                    StrategyKind::AnchorFixed,
                    true,
                    "Correct: Placing fixed invariant positions immediately bounds the remaining search space.",
                ),
                DecisionOption::new(
                    "opt_guess",
                    "Randomly test arbitrary positions for the relative pair",
                    StrategyKind::BranchCases,
                    false,
                    "Sub-optimal: Guessing before placing fixed anchors increases unnecessary search branching.",
                ),
            ],
            "opt_anchor",
            StrategyKind::AnchorFixed,
            "Always fix definite anchor positions first, then propagate relative adjacent constraints.",
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
            format!("Fix {} at slot 1.", anchor_person),
            format!("Slot 1 = {}", anchor_person),
        )
        .with_hints(vec![
            StepHint::new(HintLevel::Principle, "Constraint Principle", "Start with the most restrictive or fixed position condition."),
            StepHint::new(HintLevel::Operation, "Strategy Operation", format!("Place {} into position 1.", anchor_person)),
            StepHint::new(HintLevel::IntermediateRelation, "Slot Setup", format!("Slot 1: {}", anchor_person)),
        ]);

        let step2 = StepNode::new(
            "propagate_relative",
            StepType::PropagateConstraint,
            "Propagate Relative Conditions",
            "Place remaining people into available slots based on adjacency.",
            format!("Slot {} = {}", query_slot, puzzle.target_answer),
        )
        .with_dependencies(vec!["place_anchor".to_string()]);

        let step3 = StepNode::new(
            "identify_target",
            StepType::FinalAnswer,
            "Final Answer",
            format!("Identify person at slot {}.", query_slot),
            puzzle.target_answer.clone(),
        )
        .with_alternates(vec![puzzle.target_answer.to_lowercase()])
        .with_dependencies(vec!["propagate_relative".to_string()])
        .as_final();

        let graph = SolutionGraph::new(vec![step1, step2, step3], "identify_target");

        let parameters = json!({
            "difficulty": difficulty_level,
            "target_answer": puzzle.target_answer,
            "people": puzzle.people,
            "reasoning_metadata": meta,
        });

        let correct_answer = json!({
            "value": puzzle.target_answer,
            "formatted": puzzle.target_answer.clone(),
            "solution": puzzle.explanation,
        });

        ProblemInstance::new(
            format!("inst-reas-seat-{}", seed),
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
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let student_str = match student_answer {
            serde_json::Value::String(s) => s.trim().to_string(),
            _ => "".to_string(),
        };

        let is_correct = student_str.eq_ignore_ascii_case(expected_str);

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
