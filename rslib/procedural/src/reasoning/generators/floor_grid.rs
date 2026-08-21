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
use crate::reasoning::floor_grid::FloorGridPuzzle;
use crate::reasoning::models::{ReasoningProblemMetadata, SchemaKind, StrategyKind};

pub const FAMILY_REASONING_FLOOR_GRID: &str = "family.reasoning.floor_grid.csp";
pub const TEMPLATE_REASONING_FLOOR_GRID_V1: &str = "reasoning.floor_grid.csp.v1";

/// Generator for Spatial/Analytical Floor & Grid CSP reasoning problems.
pub struct FloorGridGenerator;

impl FloorGridGenerator {
    pub fn target_latency(difficulty_level: u32) -> u64 {
        match difficulty_level {
            1 => 30_000,
            2 => 35_000,
            3 => 45_000,
            4 => 55_000,
            _ => 65_000,
        }
    }

    pub fn generate_problem(seed: u64, difficulty_level: u32, variant: Option<&str>) -> ProblemInstance {
        let mut rng = StdRng::seed_from_u64(seed);
        let is_strategy_drill = variant == Some("strategy_drill") || variant == Some("decision_point");
        let target_time_ms = Self::target_latency(difficulty_level);

        let total_slots = match difficulty_level {
            1 => 5,
            2 => 6,
            3 => 6,
            4 => 7,
            _ => 8,
        };

        let puzzle = FloorGridPuzzle::generate_dynamic(&mut rng, total_slots, difficulty_level)
            .unwrap_or_else(FloorGridPuzzle::build_canonical_floor_puzzle);

        let anchor_entity = puzzle.anchor_entity.clone();
        let target_entity = puzzle.target_entity.clone();
        let query_slot = puzzle.target_slot;
        let target_answer = puzzle.target_answer.clone();

        let conditions_formatted: Vec<String> = puzzle
            .conditions_text
            .iter()
            .enumerate()
            .map(|(i, c)| format!("{}. {}", i + 1, c))
            .collect();

        let structure_desc = if puzzle.is_2d_grid {
            format!("**{} people** ({}) occupy offices in a {}×{} building grid.", puzzle.total_slots, puzzle.entities.join(", "), puzzle.grid_rows, puzzle.grid_cols)
        } else {
            format!("**{} people** ({}) live in an apartment building with floors numbered 1 (ground floor) to {} (top floor).", puzzle.total_slots, puzzle.entities.join(", "), puzzle.total_slots)
        };

        let prompt = format!(
            "{}\n\n\
            **Conditions:**\n{}\n\n\
            **Question:**\n{}",
            structure_desc,
            conditions_formatted.join("\n"),
            puzzle.target_question
        );

        let dp = CognitiveDecisionPoint::new(
            "dp_floor_grid_anchor",
            "Which constraint should be mapped onto the floor/grid layout first?",
            vec![
                DecisionOption::new(
                    "opt_anchor",
                    "Place the fixed or highest-constraint entity first",
                    StrategyKind::AnchorFixed.as_str(),
                    true,
                    "Anchoring fixed positions severely prunes the search space for relative above/below constraints.",
                ),
                DecisionOption::new(
                    "opt_branch",
                    "Arbitrarily test random floor assignments for relative pairs",
                    StrategyKind::BranchCases.as_str(),
                    false,
                    "Sub-optimal: Guessing before placing fixed floor anchors leads to combinatorial backtracking.",
                ),
            ],
            "opt_anchor",
            StrategyKind::AnchorFixed.as_str(),
            "Always position invariant fixed anchors or parity bounds before propagating relative distances.",
        );

        let mut meta = ReasoningProblemMetadata::new(SchemaKind::FloorGridCsp, StrategyKind::AnchorFixed)
            .with_decision_point(dp)
            .with_constraint_count(puzzle.conditions_text.len());

        if is_strategy_drill {
            meta = meta.as_strategy_drill();
        }

        let step1 = StepNode::new(
            "anchor_fixed",
            StepType::ApplyConstraint,
            "Anchor Fixed Placement",
            format!("Fix position of {} from initial conditions.", anchor_entity),
            format!("Fixed: {}", anchor_entity),
        )
        .with_hints(vec![
            StepHint::new(HintLevel::Principle, "Constraint Principle", "Begin with fixed floor assignments or parity bounds."),
            StepHint::new(HintLevel::Operation, "Strategy Operation", format!("Identify the explicit constraint for {}.", anchor_entity)),
            StepHint::new(HintLevel::IntermediateRelation, "Anchor Placed", format!("Position {} fixed.", anchor_entity)),
        ]);

        let step2 = StepNode::new(
            "propagate_distances",
            StepType::PropagateConstraint,
            "Propagate Relative Constraints",
            "Fill adjacent, above/below, and between distance constraints.",
            format!("{} at slot {}", target_entity, query_slot),
        )
        .with_dependencies(vec!["anchor_fixed".to_string()]);

        let step3 = StepNode::new(
            "derive_target",
            StepType::FinalAnswer,
            "Determine Target Placement",
            format!("Conclude position for {}: {}", target_entity, target_answer),
            target_answer.clone(),
        )
        .with_dependencies(vec!["propagate_distances".to_string()])
        .as_final();

        let solution_graph = SolutionGraph::new(vec![step1, step2, step3], "derive_target");

        let parameters = json!({
            "difficulty": difficulty_level,
            "total_slots": puzzle.total_slots,
            "is_2d_grid": puzzle.is_2d_grid,
            "target_entity": target_entity,
            "target_slot": query_slot,
            "reasoning_metadata": meta,
        });

        let correct_answer = json!({
            "value": target_answer,
            "formatted": target_answer,
            "target_slot": query_slot,
            "solution_map": puzzle.solution_map,
        });

        let instance_id = format!("inst-fg-l{}-{}", difficulty_level, seed);

        ProblemInstance::new(
            instance_id,
            FAMILY_REASONING_FLOOR_GRID,
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
            "generator": TEMPLATE_REASONING_FLOOR_GRID_V1,
        }))
    }
}

impl ProblemGenerator for FloorGridGenerator {
    fn family_id(&self) -> &str {
        FAMILY_REASONING_FLOOR_GRID
    }

    fn template_ref(&self) -> &str {
        TEMPLATE_REASONING_FLOOR_GRID_V1
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

/// Validator for Floor & Grid CSP problems.
pub struct FloorGridValidator;

impl ProblemValidator for FloorGridValidator {
    fn family_id(&self) -> &str {
        FAMILY_REASONING_FLOOR_GRID
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
            .unwrap_or("")
            .trim();

        let target_slot = instance
            .correct_answer
            .get("target_slot")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;

        let slot_str = target_slot.to_string();

        let student_str = match student_answer {
            serde_json::Value::String(s) => s.trim().to_string(),
            serde_json::Value::Number(n) => n.to_string(),
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

        let is_correct = !student_str.is_empty()
            && (student_str.eq_ignore_ascii_case(expected_str)
                || student_str == slot_str
                || student_str.to_lowercase() == format!("floor {}", target_slot).to_lowercase());

        if is_correct {
            let score = if time_taken_ms <= target_time_ms {
                1.0
            } else {
                (1.0 - ((time_taken_ms - target_time_ms) as f64 / target_time_ms as f64) * 0.5).max(0.5)
            };
            AnswerEvaluation::correct(score, time_taken_ms, target_time_ms)
        } else {
            AnswerEvaluation::incorrect(
                ErrorCategory::Strategy,
                format!(
                    "Incorrect floor placement. Submitted '{}', expected '{}'.",
                    student_str, expected_str
                ),
            )
        }
    }
}
