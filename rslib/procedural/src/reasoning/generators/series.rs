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
use crate::reasoning::models::{CognitiveDecisionPoint, DecisionOption, ReasoningProblemMetadata, SchemaKind, StrategyKind};
use crate::reasoning::series::{SeriesProblem, SeriesRule};

/// Generator for Number and Alphabet Series pattern problems.
pub struct SeriesGenerator;

impl SeriesGenerator {
    pub fn generate_problem(seed: u64, difficulty_level: u32, variant: Option<&str>) -> ProblemInstance {
        let mut rng = StdRng::seed_from_u64(seed);

        let is_strategy_drill = variant == Some("strategy_drill") || variant == Some("decision_point");

        let (rule, start, count) = match difficulty_level {
            1 => {
                let diff = rng.random_range(2..=9);
                let start = rng.random_range(1..=20);
                (SeriesRule::ConstantDifference { diff }, start, 4)
            }
            2 => {
                let start_diff = rng.random_range(2..=5);
                let step = rng.random_range(2..=4);
                let start = rng.random_range(1..=15);
                (SeriesRule::IncreasingDifference { start_diff, step }, start, 4)
            }
            3 => {
                if rng.random_bool(0.5) {
                    let ratio = rng.random_range(2..=4);
                    let start = rng.random_range(2..=6);
                    (SeriesRule::Geometric { ratio }, start, 4)
                } else {
                    let diff1 = rng.random_range(4..=8);
                    let diff2 = -rng.random_range(1..=3);
                    let start = rng.random_range(10..=30);
                    (SeriesRule::Alternating { diff1, diff2 }, start, 4)
                }
            }
            4 => {
                let shift = rng.random_range(2..=5);
                let start_char_idx = rng.random_range(0..=10);
                let start_char = (b'A' + start_char_idx as u8) as char;
                let prob = SeriesProblem::generate_alphabet(start_char, shift, 4);
                return Self::build_alphabet_instance(seed, prob, difficulty_level, is_strategy_drill);
            }
            _ => {
                // Level 5: Transfer / multi-stage
                let diff1 = rng.random_range(5..=10);
                let diff2 = -rng.random_range(2..=4);
                let start = rng.random_range(20..=50);
                (SeriesRule::Alternating { diff1, diff2 }, start, 5)
            }
        };

        let prob = SeriesProblem::generate_numeric(rule, start, count);
        Self::build_numeric_instance(seed, prob, difficulty_level, is_strategy_drill)
    }

    fn build_numeric_instance(
        seed: u64,
        prob: SeriesProblem,
        difficulty: u32,
        is_strategy_drill: bool,
    ) -> ProblemInstance {
        let terms_str = prob.terms_string.join(", ");
        let next_val = prob.expected_next_numeric.unwrap_or(0);

        let prompt = if is_strategy_drill {
            format!(
                "Consider the sequence: **{}**, **?**\n\n**Strategy Drill**: What is the primary pattern operator governing this sequence?",
                terms_str
            )
        } else {
            format!(
                "Find the missing next number in the sequence:\n\n$$\\mathbf{{{}}}, \\quad \\mathbf{{?}}$$",
                terms_str
            )
        };

        let preferred_strategy = match prob.rule {
            SeriesRule::ConstantDifference { .. } => StrategyKind::InspectDifferences,
            SeriesRule::IncreasingDifference { .. } => StrategyKind::InspectDifferences,
            SeriesRule::Geometric { .. } => StrategyKind::InspectRatios,
            SeriesRule::Alternating { .. } => StrategyKind::InspectAlternating,
            SeriesRule::AlphabetShift { .. } => StrategyKind::InspectAlphabetShift,
        };

        let dp = CognitiveDecisionPoint::new(
            "dp_series_strategy",
            "Which pattern strategy should you apply first?",
            vec![
                DecisionOption::new(
                    "opt_diff",
                    "Inspect successive first differences between terms",
                    StrategyKind::InspectDifferences,
                    matches!(prob.rule, SeriesRule::ConstantDifference { .. } | SeriesRule::IncreasingDifference { .. }),
                    "First differences reveal arithmetic and progressive difference patterns.",
                ),
                DecisionOption::new(
                    "opt_ratio",
                    "Inspect successive ratios / multiplication factors",
                    StrategyKind::InspectRatios,
                    matches!(prob.rule, SeriesRule::Geometric { .. }),
                    "Ratios reveal exponential / geometric growth patterns.",
                ),
                DecisionOption::new(
                    "opt_alt",
                    "Inspect alternating dual operations",
                    StrategyKind::InspectAlternating,
                    matches!(prob.rule, SeriesRule::Alternating { .. }),
                    "Alternating checks reveal multi-operation oscillation patterns.",
                ),
            ],
            "opt_diff",
            preferred_strategy,
            prob.rule.description(),
        );

        let mut meta = ReasoningProblemMetadata::new(SchemaKind::NumberSeries, preferred_strategy)
            .with_decision_point(dp);
        if is_strategy_drill {
            meta = meta.as_strategy_drill();
        }

        let step1 = StepNode::new(
            "identify_rule",
            StepType::SelectStrategy,
            "Identify Sequence Rule",
            "Determine the mathematical operation connecting successive terms.",
            prob.rule.description(),
        )
        .with_hints(vec![
            StepHint::new(HintLevel::Principle, "Pattern Principle", "Look at the rate of growth between successive terms."),
            StepHint::new(HintLevel::Operation, "Strategy Operation", "Calculate the differences between adjacent numbers."),
            StepHint::new(HintLevel::IntermediateRelation, "Rule Setup", prob.rule.description()),
        ]);

        let step2 = StepNode::new(
            "compute_next",
            StepType::FinalAnswer,
            "Compute Next Term",
            "Apply the identified rule to the last term to find the next value.",
            next_val.to_string(),
        )
        .with_expected_value(next_val as f64)
        .with_dependencies(vec!["identify_rule".to_string()])
        .as_final();

        let graph = SolutionGraph::new(vec![step1, step2], "compute_next");

        let parameters = json!({
            "difficulty": difficulty,
            "terms": prob.terms_numeric,
            "rule": prob.rule,
            "reasoning_metadata": meta,
        });

        let correct_answer = json!({
            "value": next_val,
            "formatted": next_val.to_string(),
            "solution": prob.explanation,
        });

        ProblemInstance::new(
            format!("inst-reas-ser-{}", seed),
            FAMILY_REASONING_SERIES,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(json!({
            "difficulty_level": difficulty,
            "target_time_ms": 25_000,
            "domain": "reasoning",
        }))
    }

    fn build_alphabet_instance(
        seed: u64,
        prob: SeriesProblem,
        difficulty: u32,
        is_strategy_drill: bool,
    ) -> ProblemInstance {
        let terms_str = prob.terms_string.join(", ");
        let next_char = prob.expected_next_string.clone();

        let prompt = format!(
            "Find the next letter in the alphabet sequence:\n\n$$\\mathbf{{{}}}, \\quad \\mathbf{{?}}$$",
            terms_str
        );

        let dp = CognitiveDecisionPoint::new(
            "dp_alphabet_shift",
            "What strategy applies to alphabet letter sequences?",
            vec![
                DecisionOption::new(
                    "opt_shift",
                    "Map characters to alphabetical numbers (1..26) and find constant shift",
                    StrategyKind::InspectAlphabetShift,
                    true,
                    "Alphabet sequences represent integer modular addition over position indices.",
                ),
            ],
            "opt_shift",
            StrategyKind::InspectAlphabetShift,
            prob.rule.description(),
        );

        let mut meta = ReasoningProblemMetadata::new(SchemaKind::AlphabetSeries, StrategyKind::InspectAlphabetShift)
            .with_decision_point(dp);
        if is_strategy_drill {
            meta = meta.as_strategy_drill();
        }

        let step1 = StepNode::new(
            "alphabet_shift",
            StepType::MakeInference,
            "Determine Character Shift",
            "Calculate the positional index shift between successive letters.",
            prob.rule.description(),
        );

        let step2 = StepNode::new(
            "next_letter",
            StepType::FinalAnswer,
            "Find Next Letter",
            "Apply shift to the last character.",
            next_char.clone(),
        )
        .with_alternates(vec![next_char.to_lowercase()])
        .with_dependencies(vec!["alphabet_shift".to_string()])
        .as_final();

        let graph = SolutionGraph::new(vec![step1, step2], "next_letter");

        let parameters = json!({
            "difficulty": difficulty,
            "terms": prob.terms_string,
            "rule": prob.rule,
            "reasoning_metadata": meta,
        });

        let correct_answer = json!({
            "value": next_char,
            "formatted": next_char.clone(),
            "solution": prob.explanation,
        });

        ProblemInstance::new(
            format!("inst-reas-alph-{}", seed),
            FAMILY_REASONING_SERIES,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(json!({
            "difficulty_level": difficulty,
            "target_time_ms": 20_000,
            "domain": "reasoning",
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
            "constant_difference".to_string(),
            "increasing_difference".to_string(),
            "geometric".to_string(),
            "alternating".to_string(),
            "alphabet_shift".to_string(),
            "strategy_drill".to_string(),
        ]
    }

    fn target_latency_ms(&self, difficulty_level: u32) -> u64 {
        match difficulty_level {
            1 => 18_000,
            2 => 22_000,
            3 => 25_000,
            4 => 25_000,
            _ => 30_000,
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

/// Validator for Number and Alphabet Series problems.
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
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let student_str = match student_answer {
            serde_json::Value::String(s) => s.trim().to_string(),
            serde_json::Value::Number(n) => n.to_string(),
            _ => "".to_string(),
        };

        let is_correct = student_str.eq_ignore_ascii_case(expected_str);

        if is_correct {
            AnswerEvaluation::correct(1.0, time_taken_ms, target_time_ms)
        } else {
            AnswerEvaluation::incorrect(
                ErrorCategory::Strategy,
                format!(
                    "Incorrect next term. Submitted '{}', expected '{}'.",
                    student_str, expected_str
                ),
            )
        }
    }
}
