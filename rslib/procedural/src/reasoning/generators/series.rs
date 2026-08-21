// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde_json::json;

use crate::core::{ProblemFamilyId, Result};
use crate::diagnostics::ErrorCategory;
use crate::problems::generator::ProblemGenerator;
use crate::problems::steps::{HintLevel, SolutionGraph, StepHint, StepNode, StepType};
use crate::problems::validator::{AnswerEvaluation, ProblemValidator};
use crate::problems::ProblemInstance;
use crate::reasoning::generators::{FAMILY_REASONING_SERIES, TEMPLATE_REASONING_SERIES_V1};
use crate::core::decision::{CognitiveDecisionPoint, DecisionOption};
use crate::reasoning::models::{ReasoningProblemMetadata, SchemaKind, StrategyKind};
use crate::reasoning::series::{SeriesProblem, SeriesRule};

/// Generator for Number and Alphabetical Pattern Series reasoning problems.
pub struct SeriesGenerator;

impl SeriesGenerator {
    pub fn target_latency(difficulty_level: u32) -> u64 {
        match difficulty_level {
            1 => 15_000,
            2 => 20_000,
            3 => 25_000,
            4 => 30_000,
            _ => 30_000,
        }
    }

    pub fn generate_problem(seed: u64, difficulty_level: u32, variant: Option<&str>) -> ProblemInstance {
        let mut rng = StdRng::seed_from_u64(seed);

        let is_strategy_drill = variant == Some("strategy_drill") || variant == Some("decision_point");
        let target_time_ms = Self::target_latency(difficulty_level);

        let (prob, strategy_kind) = match difficulty_level {
            1 => {
                let diff = (rng.random_range(2..=15) as i64) * if rng.random_bool(0.3) { -1 } else { 1 };
                let start = rng.random_range(5..=100) as i64;
                (
                    SeriesProblem::generate_numeric(SeriesRule::ConstantDifference { diff }, start, 5),
                    StrategyKind::InspectDifferences,
                )
            }
            2 => {
                let start_diff = rng.random_range(2..=8) as i64;
                let step = rng.random_range(2..=6) as i64;
                let start = rng.random_range(1..=40) as i64;
                (
                    SeriesProblem::generate_numeric(
                        SeriesRule::IncreasingDifference { start_diff, step },
                        start,
                        5,
                    ),
                    StrategyKind::InspectDifferences,
                )
            }
            3 => {
                let mode = rng.random_range(0..2);
                if mode == 0 {
                    let ratio = (rng.random_range(2..=5) as i64) * if rng.random_bool(0.5) { 1 } else { -1 };
                    let start = (rng.random_range(2..=20) as i64) * if rng.random_bool(0.5) { 1 } else { -1 };
                    (
                        SeriesProblem::generate_numeric(SeriesRule::Geometric { ratio }, start, 5),
                        StrategyKind::InspectRatios,
                    )
                } else {
                    let start_k = rng.random_range(1..=10) as i64;
                    let start = rng.random_range(1..=100) as i64;
                    (
                        SeriesProblem::generate_numeric(SeriesRule::SquareDifference { start_k }, start, 5),
                        StrategyKind::InspectDifferences,
                    )
                }
            }
            4 => {
                let mode = rng.random_range(0..3);
                if mode == 0 {
                    let d1 = rng.random_range(3..=15) as i64;
                    let d2 = (rng.random_range(2..=12) as i64) * -1;
                    let start = rng.random_range(20..=120) as i64;
                    (
                        SeriesProblem::generate_numeric(SeriesRule::Alternating { diff1: d1, diff2: d2 }, start, 6),
                        StrategyKind::InspectAlternating,
                    )
                } else if mode == 1 {
                    let mult = rng.random_range(2..=5) as i64;
                    let add = (rng.random_range(1..=10) as i64) * if rng.random_bool(0.5) { 1 } else { -1 };
                    let start = rng.random_range(2..=20) as i64;
                    (
                        SeriesProblem::generate_numeric(SeriesRule::MultiplyAndAdd { mult, add }, start, 5),
                        StrategyKind::InspectRatios,
                    )
                } else {
                    let start = rng.random_range(1..=100) as i64;
                    (
                        SeriesProblem::generate_numeric(SeriesRule::FibonacciLike, start, 6),
                        StrategyKind::InspectDifferences,
                    )
                }
            }
            _ => {
                let shift = rng.random_range(1..=25) as i32;
                let start_char = (b'A' + rng.random_range(0..26)) as char;
                (
                    SeriesProblem::generate_alphabet(start_char, shift, 5),
                    StrategyKind::InspectAlphabetShift,
                )
            }
        };

        let seq_str = if prob.is_alphabet {
            prob.terms_string.join(", ")
        } else {
            prob.terms_numeric
                .iter()
                .map(|t| t.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        };

        let prompt = format!(
            "Find the next term in the following {}:\n\n\\[ {}, \\; \\mathbf{{?}} \\]",
            if prob.is_alphabet { "alphabet sequence" } else { "number sequence" },
            seq_str
        );

        let target_str = prob.expected_next_string.clone();

        let dp = CognitiveDecisionPoint::new(
            "dp_series_first_step",
            "What is the first analytical step to discover the underlying sequence pattern?",
            vec![
                DecisionOption::new(
                    "opt_diff",
                    "Compute first-order differences (Δ = a_{n} - a_{n-1}) between successive terms",
                    StrategyKind::InspectDifferences.as_str(),
                    true,
                    "Calculating successive differences immediately classifies arithmetic, progressive, and alternating progressions.",
                ),
                DecisionOption::new(
                    "opt_skip",
                    "Guess numbers randomly until one looks plausible",
                    StrategyKind::BranchCases.as_str(),
                    false,
                    "Sub-optimal: Guessing fails on non-trivial polynomial and alternating series.",
                ),
            ],
            "opt_diff",
            StrategyKind::InspectDifferences.as_str(),
            "Always inspect the differences or ratios between consecutive terms.",
        );

        let mut meta = ReasoningProblemMetadata::new(
            if prob.is_alphabet { SchemaKind::AlphabetSeries } else { SchemaKind::NumberSeries },
            strategy_kind,
        )
        .with_decision_point(dp)
        .with_constraint_count(prob.terms_string.len());

        if is_strategy_drill {
            meta = meta.as_strategy_drill();
        }

        let step1 = StepNode::new(
            "find_pattern",
            StepType::Transformation,
            "Discover Pattern Rule",
            prob.rule.description(),
            prob.rule.description(),
        )
        .with_hints(vec![
            StepHint::new(HintLevel::Principle, "Pattern Principle", "Look for common difference, common ratio, or progressive steps."),
            StepHint::new(HintLevel::Operation, "Strategy Operation", "Calculate the relationship between each consecutive pair."),
            StepHint::new(HintLevel::IntermediateRelation, "Rule Found", &prob.rule.description()),
        ]);

        let step2 = StepNode::new(
            "apply_pattern",
            StepType::FinalAnswer,
            "Compute Next Term",
            format!("Apply rule to obtain next term: {}.", target_str),
            target_str.clone(),
        )
        .with_alternates(vec![
            target_str.to_lowercase(),
            target_str.to_uppercase(),
        ])
        .with_dependencies(vec!["find_pattern".to_string()])
        .as_final();

        let graph = SolutionGraph::new(vec![step1, step2], "apply_pattern");

        let parameters = json!({
            "difficulty": difficulty_level,
            "is_alphabet": prob.is_alphabet,
            "target": target_str,
            "reasoning_metadata": meta,
        });

        let correct_answer = json!({
            "value": target_str,
            "formatted": target_str,
            "solution": prob.explanation,
        });

        let instance_id = format!("inst-series-l{}-{}", difficulty_level, seed);

        ProblemInstance::new(
            instance_id,
            FAMILY_REASONING_SERIES,
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
            "generator": TEMPLATE_REASONING_SERIES_V1,
        }))
    }
}

impl ProblemGenerator for SeriesGenerator {
    fn family_id(&self) -> &str {
        FAMILY_REASONING_SERIES
    }

    fn template_ref(&self) -> &str {
        TEMPLATE_REASONING_SERIES_V1
    }

    fn supported_variants(&self) -> Vec<String> {
        vec![
            "constant_diff".to_string(),
            "increasing_diff".to_string(),
            "geometric".to_string(),
            "alternating".to_string(),
            "alphabet_shift".to_string(),
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

/// Validator for Number and Alphabetical Pattern Series problems.
pub struct SeriesValidator;

impl ProblemValidator for SeriesValidator {
    fn family_id(&self) -> &str {
        FAMILY_REASONING_SERIES
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
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::String(s) => s.trim().to_string(),
            serde_json::Value::Object(map) => {
                map.get("formatted")
                    .or_else(|| map.get("value"))
                    .or_else(|| map.get("answer"))
                    .map(|v| {
                        if let Some(s) = v.as_str() {
                            s.trim().to_string()
                        } else if let Some(n) = v.as_i64() {
                            n.to_string()
                        } else if let Some(f) = v.as_f64() {
                            format!("{:.0}", f)
                        } else {
                            "".to_string()
                        }
                    })
                    .unwrap_or_else(|| "".to_string())
            }
            _ => "".to_string(),
        };

        let is_correct = !student_str.is_empty()
            && student_str.trim().eq_ignore_ascii_case(expected_str.trim());

        if is_correct {
            AnswerEvaluation::correct(1.0, time_taken_ms, target_time_ms)
        } else {
            AnswerEvaluation::incorrect(
                ErrorCategory::Calculation,
                format!(
                    "Incorrect series term. Submitted '{}', expected '{}'.",
                    student_str, expected_str
                ),
            )
        }
    }
}
