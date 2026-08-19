// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::skills::signals::PracticeProgressionState;
use crate::skills::SkillState;

/// Outcome of evaluating a skill for retirement / hibernation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RetirementEvaluation {
    pub is_eligible: bool,
    pub current_mastery_score: f64,
    pub days_since_active_learning: i64,
    pub variant_coverage_count: usize,
    pub reasons: Vec<String>,
}

/// Structured outcome of a maintenance review attempt on a retired skill.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MaintenanceReviewOutcome {
    /// Skill confirmed robust fluency; remains in retired maintenance state.
    ConfirmedFluency { new_maintenance_timestamp: i64 },
    /// Minor calculation slip or single retry; remains retired with advisory note.
    AcceptableWithNote { note: String },
    /// Meaningful failure or conceptual breakdown; reactivated to active practice stage.
    ReactivatedToActiveStage {
        reactivated_stage: PracticeProgressionState,
        reason: String,
    },
}

/// Centralized lifecycle policy for long-term skill retirement and maintenance.
#[derive(Debug, Clone)]
pub struct RetirementPolicy {
    pub min_mastery_score: f64,
    pub min_stability_days: i64,
    pub min_variant_coverage: usize,
    pub max_recent_error_rate: f64,
}

impl Default for RetirementPolicy {
    fn default() -> Self {
        Self {
            min_mastery_score: 0.85,
            min_stability_days: 14,
            min_variant_coverage: 3,
            max_recent_error_rate: 0.10,
        }
    }
}

impl RetirementPolicy {
    /// Evaluates whether a skill qualifies for retirement into low-frequency maintenance.
    pub fn evaluate_retirement_eligibility(&self, state: &SkillState) -> RetirementEvaluation {
        let mut reasons = Vec::new();
        let mastery = state.mastery;
        let variant_count = state.variant_stats.len();
        let recent_err = 1.0 - state.recent_accuracy();

        let now = Utc::now().timestamp();
        let days_since = match state.last_practiced_at {
            Some(ts) => (now - ts).max(0) / 86400,
            None => 0,
        };

        if state.practice_state != PracticeProgressionState::Mastered {
            reasons.push(format!(
                "Skill must be in Mastered state before retirement (currently {}).",
                state.practice_state.as_str()
            ));
        }

        if mastery < self.min_mastery_score {
            reasons.push(format!(
                "Mastery score ({:.2}) is below retirement threshold ({:.2}).",
                mastery, self.min_mastery_score
            ));
        }

        if variant_count < self.min_variant_coverage {
            reasons.push(format!(
                "Variant coverage ({}) is below required distinct variant count ({}).",
                variant_count, self.min_variant_coverage
            ));
        }

        if recent_err > self.max_recent_error_rate {
            reasons.push(format!(
                "Recent error rate ({:.1}%) exceeds allowed maximum ({:.1}%).",
                recent_err * 100.0,
                self.max_recent_error_rate * 100.0
            ));
        }

        let is_eligible = reasons.is_empty();

        RetirementEvaluation {
            is_eligible,
            current_mastery_score: mastery,
            days_since_active_learning: days_since,
            variant_coverage_count: variant_count,
            reasons,
        }
    }

    /// Evaluates the outcome of a maintenance review on a retired skill.
    pub fn evaluate_maintenance_attempt(
        state: &mut SkillState,
        is_correct: bool,
        is_conceptual_error: bool,
    ) -> MaintenanceReviewOutcome {
        let now = Utc::now().timestamp();

        if is_correct {
            state.last_practiced_at = Some(now);
            MaintenanceReviewOutcome::ConfirmedFluency {
                new_maintenance_timestamp: now,
            }
        } else if is_conceptual_error || state.consecutive_failures >= 2 {
            // Significant breakdown on retired skill: reactivate to Learning or Variation
            let target_stage = if is_conceptual_error {
                PracticeProgressionState::Learning
            } else {
                PracticeProgressionState::Variation
            };
            state.practice_state = target_stage;
            MaintenanceReviewOutcome::ReactivatedToActiveStage {
                reactivated_stage: target_stage,
                reason: "Maintenance check encountered conceptual or recurring error.".into(),
            }
        } else {
            // Single non-conceptual slip: remain retired but log note
            MaintenanceReviewOutcome::AcceptableWithNote {
                note: "Single isolated execution slip on maintenance; remaining in retired state.".into(),
            }
        }
    }
}
