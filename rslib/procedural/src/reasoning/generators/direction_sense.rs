// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde_json::json;

use crate::core::{ProblemFamilyId, Result};
use crate::core::decision::{CognitiveDecisionPoint, DecisionOption};
use crate::diagnostics::ErrorCategory;
use crate::problems::generator::ProblemGenerator;
use crate::problems::steps::{HintLevel, SolutionGraph, StepHint, StepNode, StepType};
use crate::problems::validator::{AnswerEvaluation, ProblemValidator};
use crate::problems::ProblemInstance;
use crate::reasoning::models::{ReasoningProblemMetadata, SchemaKind, StrategyKind};
use crate::reasoning::relations::DirectionPuzzle;

pub const FAMILY_REASONING_DIRECTION_SENSE: &str = "family.reasoning.direction_sense";
pub const TEMPLATE_REASONING_DIRECTION_SENSE_V1: &str = "reasoning.direction_sense.v1";

/// Generator for Direction Sense 2D spatial trajectory and compass orientation problems.
pub struct DirectionSenseGenerator;

impl DirectionSenseGenerator {
    pub fn target_latency(difficulty_level: u32) -> u64 {
        match difficulty_level {
            1 => 20_000,
            2 => 25_000,
            3 => 30_000,
            4 => 35_000,
            _ => 40_000,
        }
    }

    pub fn generate_problem(seed: u64, difficulty_level: u32, variant: Option<&str>) -> ProblemInstance {
        let mut rng = StdRng::seed_from_u64(seed);
        let is_strategy_drill = variant == Some("strategy_drill") || variant == Some("decision_point");

        let puzzle = if let Some(v) = variant {
            match v {
                "orthogonal_2step" => {
                    let d1 = rng.random_range(5..=40);
                    let d2 = rng.random_range(5..=40);
                    DirectionPuzzle::create_2step_path(d1, d2)
                }
                "standard_3step" => {
                    let d1 = rng.random_range(6..=50);
                    let d2 = rng.random_range(4..=40);
                    let d3 = rng.random_range(2..=(d1 + 20));
                    DirectionPuzzle::create_path(d1, d2, d3)
                }
                "multiturn_4step" => {
                    let d1 = rng.random_range(8..=50);
                    let d2 = rng.random_range(6..=40);
                    let d3 = rng.random_range(4..=35);
                    let d4 = rng.random_range(5..=30);
                    DirectionPuzzle::create_4step_path(d1, d2, d3, d4)
                }
                "complex_5step" => {
                    let d1 = rng.random_range(10..=60);
                    let d2 = rng.random_range(8..=50);
                    let d3 = rng.random_range(6..=45);
                    let d4 = rng.random_range(5..=40);
                    let d5 = rng.random_range(4..=35);
                    DirectionPuzzle::create_5step_path(d1, d2, d3, d4, d5)
                }
                _ => Self::generate_by_level(&mut rng, difficulty_level),
            }
        } else {
            Self::generate_by_level(&mut rng, difficulty_level)
        };

        let target_time_ms = Self::target_latency(difficulty_level);
        let prompt = format!(
            "A person starts from a fixed point O and performs the following walk:\n{}\n\n\
             **Question:** In which **direction** is the person located relative to the starting point?",
            puzzle.steps_text.iter().map(|s| format!("- {}", s)).collect::<Vec<_>>().join("\n")
        );

        let heading_str = puzzle.target_heading.as_str().to_string();

        let dp = CognitiveDecisionPoint::new(
            "dp_direction_vector",
            "Which method should you use to calculate the net displacement?",
            vec![
                DecisionOption::new(
                    "opt_vector",
                    "Sum horizontal (East/West) and vertical (North/South) displacement components separately",
                    StrategyKind::TraceDirectionVectors.as_str(),
                    true,
                    "Component summation (Δx, Δy) simplifies 2D trajectories into a single coordinate point.",
                ),
                DecisionOption::new(
                    "opt_sketch",
                    "Estimate the direction visually from a rough hand sketch without numbers",
                    StrategyKind::BranchCases.as_str(),
                    false,
                    "Sub-optimal: Qualitative sketches fail when distances in opposite directions are close.",
                ),
            ],
            "opt_vector",
            StrategyKind::TraceDirectionVectors.as_str(),
            "Decompose each path segment into East/West (X) and North/South (Y) components.",
        );

        let mut meta = ReasoningProblemMetadata::new(SchemaKind::DirectionSense, StrategyKind::TraceDirectionVectors)
            .with_decision_point(dp)
            .with_constraint_count(puzzle.steps_text.len());

        if is_strategy_drill {
            meta = meta.as_strategy_drill();
        }

        let step1 = StepNode::new(
            "calc_components",
            StepType::Transformation,
            "Calculate Net Coordinate Displacement",
            format!("Δx = {} m, Δy = {} m", puzzle.displacement_x, puzzle.displacement_y),
            format!("({}, {})", puzzle.displacement_x, puzzle.displacement_y),
        )
        .with_hints(vec![
            StepHint::new(HintLevel::Principle, "2D Displacement", "Track net East (+x) / West (-x) and net North (+y) / South (-y)."),
            StepHint::new(HintLevel::Operation, "Sum Components", format!("Net Δx = {} m, Δy = {} m.", puzzle.displacement_x, puzzle.displacement_y)),
            StepHint::new(HintLevel::IntermediateRelation, "Coordinates", format!("({}, {})", puzzle.displacement_x, puzzle.displacement_y)),
        ]);

        let step2 = StepNode::new(
            "determine_direction",
            StepType::FinalAnswer,
            "Determine Cardinal/Ordinal Direction",
            format!("Vector ({}, {}) corresponds to direction {}.", puzzle.displacement_x, puzzle.displacement_y, heading_str),
            heading_str.clone(),
        )
        .with_alternates(vec![
            heading_str.to_lowercase(),
            heading_str.replace('-', "").to_lowercase(),
            heading_str.replace('-', " ").to_lowercase(),
        ])
        .with_dependencies(vec!["calc_components".to_string()])
        .as_final();

        let graph = SolutionGraph::new(vec![step1, step2], "determine_direction");

        let parameters = json!({
            "difficulty": difficulty_level,
            "dx": puzzle.displacement_x,
            "dy": puzzle.displacement_y,
            "straight_distance": puzzle.straight_distance_m,
            "target_heading": heading_str,
            "reasoning_metadata": meta,
        });

        let correct_answer = json!({
            "value": heading_str,
            "formatted": heading_str,
            "solution": puzzle.explanation,
        });

        let instance_id = format!("inst-direction-l{}-{}", difficulty_level, seed);

        ProblemInstance::new(
            instance_id,
            FAMILY_REASONING_DIRECTION_SENSE,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(json!({
            "difficulty_level": difficulty_level,
            "target_time_ms": target_time_ms,
            "domain": "reasoning",
            "generator": TEMPLATE_REASONING_DIRECTION_SENSE_V1,
        }))
    }

    fn generate_by_level(rng: &mut StdRng, difficulty_level: u32) -> DirectionPuzzle {
        match difficulty_level {
            1 => {
                let d1 = rng.random_range(5..=30);
                let d2 = rng.random_range(5..=30);
                DirectionPuzzle::create_2step_path(d1, d2)
            }
            2 => {
                let d1 = rng.random_range(6..=40);
                let d2 = rng.random_range(5..=35);
                let d3 = rng.random_range(2..=(d1 + 15));
                DirectionPuzzle::create_path(d1, d2, d3)
            }
            3 => {
                let d1 = rng.random_range(8..=45);
                let d2 = rng.random_range(6..=40);
                let d3 = rng.random_range(4..=35);
                let d4 = rng.random_range(5..=30);
                DirectionPuzzle::create_4step_path(d1, d2, d3, d4)
            }
            4 => {
                let d1 = rng.random_range(10..=55);
                let d2 = rng.random_range(8..=45);
                let d3 = rng.random_range(6..=40);
                let d4 = rng.random_range(7..=35);
                DirectionPuzzle::create_4step_path(d1, d2, d3, d4)
            }
            _ => {
                let d1 = rng.random_range(10..=60);
                let d2 = rng.random_range(8..=50);
                let d3 = rng.random_range(6..=45);
                let d4 = rng.random_range(5..=40);
                let d5 = rng.random_range(4..=35);
                DirectionPuzzle::create_5step_path(d1, d2, d3, d4, d5)
            }
        }
    }
}

impl ProblemGenerator for DirectionSenseGenerator {
    fn family_id(&self) -> &str {
        FAMILY_REASONING_DIRECTION_SENSE
    }

    fn template_ref(&self) -> &str {
        TEMPLATE_REASONING_DIRECTION_SENSE_V1
    }

    fn supported_variants(&self) -> Vec<String> {
        vec![
            "orthogonal_2step".to_string(),
            "standard_3step".to_string(),
            "multiturn_4step".to_string(),
            "complex_5step".to_string(),
            "strategy_drill".to_string(),
        ]
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

/// Validator for Direction Sense problems.
pub struct DirectionSenseValidator;

impl ProblemValidator for DirectionSenseValidator {
    fn family_id(&self) -> &str {
        FAMILY_REASONING_DIRECTION_SENSE
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

        let clean_sub = student_str.to_lowercase().replace('-', " ").replace('_', " ");
        let clean_exp = expected_str.to_lowercase().replace('-', " ").replace('_', " ");

        let is_correct = clean_sub == clean_exp
            || clean_sub.contains(&clean_exp)
            || (clean_exp.contains("north") && clean_sub == "north")
            || (clean_exp.contains("south") && clean_sub == "south")
            || (clean_exp.contains("east") && clean_sub == "east")
            || (clean_exp.contains("west") && clean_sub == "west");

        if is_correct {
            AnswerEvaluation::correct(1.0, time_taken_ms, target_time_ms)
        } else {
            AnswerEvaluation::incorrect(
                ErrorCategory::Concept,
                format!(
                    "Incorrect direction. Submitted '{}', expected '{}'.",
                    student_str, expected_str
                ),
            )
        }
    }
}
