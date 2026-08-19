// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use serde::{Deserialize, Serialize};

use crate::core::Domain;
use crate::skills::signals::PracticeProgressionState;

/// Categorical evaluation of attempt solution speed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpeedRating {
    /// Solution was executed efficiently within expected automaticity bounds.
    Optimal,
    /// Solution latency is within reasonable working limits for the domain and stage.
    Acceptable,
    /// Solution required noticeable deliberation or mechanical hesitation.
    Slow,
    /// Solution latency exceeded multiple multiples of target, indicating struggle.
    SeverelyDelayed,
}

/// Domain-specific timing parameters for procedural problem solving.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DomainSpeedConfig {
    pub domain: Domain,
    /// Baseline target latency (ms) for typical single-step or standard problems.
    pub target_latency_ms: u64,
    /// Upper acceptable latency bound (ms) before marking as slow.
    pub warning_threshold_ms: u64,
    /// Maximum allowed latency (ms) before severe delay penalty applies.
    pub max_expected_ms: u64,
}

impl DomainSpeedConfig {
    pub fn for_domain(domain: Domain) -> Self {
        match domain {
            Domain::Mathematics => Self {
                domain,
                target_latency_ms: 30_000,    // 30s
                warning_threshold_ms: 60_000, // 60s
                max_expected_ms: 120_000,     // 120s
            },
            Domain::Physics => Self {
                domain,
                target_latency_ms: 45_000,    // 45s (model selection & units take time)
                warning_threshold_ms: 90_000, // 90s
                max_expected_ms: 180_000,     // 180s
            },
            Domain::Chemistry => Self {
                domain,
                target_latency_ms: 45_000,    // 45s (stoichiometric setup takes time)
                warning_threshold_ms: 90_000, // 90s
                max_expected_ms: 180_000,     // 180s
            },
            Domain::Reasoning => Self {
                domain,
                target_latency_ms: 40_000,    // 40s (constraint elimination search)
                warning_threshold_ms: 80_000, // 80s
                max_expected_ms: 160_000,     // 160s
            },
            Domain::Custom(_) => Self {
                domain,
                target_latency_ms: 35_000,
                warning_threshold_ms: 70_000,
                max_expected_ms: 140_000,
            },
        }
    }
}

/// Structured outcome of stage-aware speed interpretation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpeedEvaluation {
    pub speed_rating: SpeedRating,
    pub effective_latency_weight: f64,
    pub fluency_score: f64,
    pub is_acceptable: bool,
    pub advisory_message: Option<String>,
}

/// Centralized stage-aware speed policy interpreter.
#[derive(Debug, Clone, Default)]
pub struct StageSpeedPolicy;

impl StageSpeedPolicy {
    /// Returns the stage-dependent latency weight (0.0 = completely informational, 1.0 = full weight).
    pub fn latency_weight_for_stage(stage: PracticeProgressionState) -> f64 {
        match stage {
            PracticeProgressionState::New => 0.0,
            PracticeProgressionState::Learning => 0.10,
            PracticeProgressionState::Fluent => 0.50,
            PracticeProgressionState::Variation => 0.70,
            PracticeProgressionState::Transfer => 0.30, // Generalization prioritizes correctness over speed
            PracticeProgressionState::Mastered
            | PracticeProgressionState::Retired
            | PracticeProgressionState::Hibernating => 0.85,
        }
    }

    /// Evaluates latency performance for an attempt given its progression stage and domain.
    pub fn evaluate(
        stage: PracticeProgressionState,
        domain: Domain,
        actual_latency_ms: u64,
        target_override_ms: Option<u64>,
    ) -> SpeedEvaluation {
        let config = DomainSpeedConfig::for_domain(domain);
        let target_ms = target_override_ms.unwrap_or(config.target_latency_ms);
        let warning_ms = target_ms.saturating_mul(2).max(config.warning_threshold_ms);
        let max_ms = target_ms.saturating_mul(4).max(config.max_expected_ms);

        let weight = Self::latency_weight_for_stage(stage);

        let (rating, fluency, advisory) = if actual_latency_ms <= target_ms {
            (SpeedRating::Optimal, 1.0, None)
        } else if actual_latency_ms <= warning_ms {
            let ratio = (actual_latency_ms - target_ms) as f64 / (warning_ms - target_ms) as f64;
            let score = 1.0 - (0.25 * ratio);
            (SpeedRating::Acceptable, score, None)
        } else if actual_latency_ms <= max_ms {
            let ratio = (actual_latency_ms - warning_ms) as f64 / (max_ms - warning_ms) as f64;
            let score = 0.75 - (0.45 * ratio);
            let msg = if weight >= 0.4 {
                Some(format!(
                    "Solution speed is slow ({:.1}s vs target {:.1}s). Practice will reinforce automaticity.",
                    actual_latency_ms as f64 / 1000.0,
                    target_ms as f64 / 1000.0
                ))
            } else {
                None
            };
            (SpeedRating::Slow, score, msg)
        } else {
            let msg = Some(format!(
                "Latency was significantly delayed ({:.1}s).",
                actual_latency_ms as f64 / 1000.0
            ));
            (SpeedRating::SeverelyDelayed, 0.20, msg)
        };

        // For early learning stages, speed rating never marks as unacceptable unless severely delayed
        let is_acceptable = match stage {
            PracticeProgressionState::New | PracticeProgressionState::Learning => {
                rating != SpeedRating::SeverelyDelayed
            }
            PracticeProgressionState::Transfer => rating != SpeedRating::SeverelyDelayed,
            _ => rating == SpeedRating::Optimal || rating == SpeedRating::Acceptable,
        };

        SpeedEvaluation {
            speed_rating: rating,
            effective_latency_weight: weight,
            fluency_score: fluency,
            is_acceptable,
            advisory_message: advisory,
        }
    }
}
