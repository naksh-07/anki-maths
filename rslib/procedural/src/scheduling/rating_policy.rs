// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::fmt;
use serde::{Deserialize, Serialize};

use crate::diagnostics::{ErrorCategory, ProceduralReviewOutcome};
use crate::skills::SkillState;

/// Standard 4-button review rating compatible with Anki and FSRS.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum Rating {
    Again = 1,
    Hard = 2,
    Good = 3,
    Easy = 4,
}

impl Rating {
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            1 => Some(Rating::Again),
            2 => Some(Rating::Hard),
            3 => Some(Rating::Good),
            4 => Some(Rating::Easy),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Rating::Again => "again",
            Rating::Hard => "hard",
            Rating::Good => "good",
            Rating::Easy => "easy",
        }
    }
}

impl fmt::Display for Rating {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Pluggable policy interface for mapping objective procedural telemetry to FSRS ratings.
pub trait RatingPolicy: Send + Sync {
    fn derive_rating(
        &self,
        outcome: &ProceduralReviewOutcome,
        skill_state: Option<&SkillState>,
    ) -> Rating;
}

/// Deterministic, rule-based rating policy based on latency thresholds, hints, attempts,
/// error taxonomies, and recent performance windows.
#[derive(Debug, Clone)]
pub struct StandardRatingPolicy {
    /// Latency factor above target considered "slow" / "struggling" (e.g. 1.25 = 125% of target time).
    pub slow_latency_multiplier: f64,
    /// Latency factor below target considered "fluent" / "fast" (e.g. 0.75 = 75% of target time).
    pub fast_latency_multiplier: f64,
    /// Number of consecutive successes required for Easy rating.
    pub easy_consecutive_successes: u32,
    /// Recent accuracy threshold required for Easy rating.
    pub easy_recent_accuracy: f64,
}

impl Default for StandardRatingPolicy {
    fn default() -> Self {
        Self {
            slow_latency_multiplier: 1.25,
            fast_latency_multiplier: 0.75,
            easy_consecutive_successes: 2,
            easy_recent_accuracy: 0.80,
        }
    }
}

impl RatingPolicy for StandardRatingPolicy {
    fn derive_rating(
        &self,
        outcome: &ProceduralReviewOutcome,
        skill_state: Option<&SkillState>,
    ) -> Rating {
        // 1. Unsuccessful / invalid attempt -> Again
        if !outcome.is_correct || outcome.score <= 0.0 {
            return Rating::Again;
        }

        // 2. Fatal misconception or strategy breakdown -> Again
        if let Some(ref cat) = outcome.error_category {
            if matches!(
                cat,
                ErrorCategory::Concept
                    | ErrorCategory::Conceptual
                    | ErrorCategory::Strategy
            ) {
                return Rating::Again;
            }
        }

        // 3. Multiple failed attempts before eventual correctness (e.g. wrong -> wrong -> correct) -> Again
        // Learner fundamentally failed on their own and required 3 or more attempts.
        if outcome.attempt_count >= 3 {
            return Rating::Again;
        }

        // 4. Heavy hint dependence (>= 3 hints, or 2 hints combined with retries) -> Again
        if outcome.hints_used >= 3 || (outcome.hints_used >= 2 && outcome.attempt_count > 1) {
            return Rating::Again;
        }

        // Check latency boundaries
        let slow_threshold_ms = (outcome.target_latency_ms as f64 * self.slow_latency_multiplier) as u64;
        let fast_threshold_ms = (outcome.target_latency_ms as f64 * self.fast_latency_multiplier) as u64;

        let is_slow = outcome.latency_ms > slow_threshold_ms;
        let is_fast = outcome.latency_ms <= fast_threshold_ms;

        // Step-level evidence
        let had_step_error = outcome.first_error_step.is_some()
            || (outcome.steps_completed > 0 && outcome.steps_correct < outcome.steps_completed);

        // First action stall (stalled before making initial move)
        let had_first_action_stall = outcome
            .first_action_latency_ms
            .map_or(false, |lat| slow_threshold_ms > 0 && lat > slow_threshold_ms / 2);

        let required_support = outcome.hints_used > 0 || outcome.attempt_count > 1;
        let had_minor_error = outcome.error_category.is_some();

        let is_early_learning = skill_state.map_or(false, |s| {
            matches!(
                s.practice_state,
                crate::skills::PracticeProgressionState::New
                    | crate::skills::PracticeProgressionState::Learning
            )
        });

        // 5. Check for Hard conditions:
        // - significantly slow latency or severe initial hesitation (in early learning stage, clean slow solves up to 2.5x target are permitted without Hard penalty)
        // - used hints (1-2) or needed 1 retry (attempt_count == 2)
        // - corrected an earlier step error during stepwise solving
        // - had a minor calculation slip
        // - learner history shows recent weakness / consecutive failures
        let recent_struggle = skill_state.map_or(false, |s| {
            s.consecutive_failures > 0 || (s.recent_attempts.len() >= 3 && s.recent_accuracy() < 0.5)
        });

        let slow_penalized = if is_early_learning && !required_support && !had_step_error && !had_minor_error {
            outcome.latency_ms > (outcome.target_latency_ms as f64 * 2.5) as u64
        } else {
            is_slow
        };

        if slow_penalized
            || required_support
            || had_step_error
            || had_first_action_stall
            || had_minor_error
            || recent_struggle
        {
            return Rating::Hard;
        }

        // 6. Check for Easy conditions:
        // - clean independent solve (0 hints, 1 attempt, 0 step errors, no misconceptions)
        // - comfortably below target latency (fast)
        // - strong recent history (or no history but fast and unassisted)
        // - structural familiarity gate: established skills must have passed >= 2 distinct structural forms and have longitudinal independence >= 70%
        let has_strong_history = match skill_state {
            Some(s) => {
                let streak_ok = s.consecutive_successes >= self.easy_consecutive_successes
                    || s.recent_accuracy() >= self.easy_recent_accuracy;
                let independence_ok = s.longitudinal_independence_ratio() >= 0.70;
                let structural_ok = s.total_attempts < 4 || s.distinct_structural_forms_passed() >= 2 || s.variant_stats.len() >= 2;
                streak_ok && independence_ok && structural_ok
            }
            None => true, // default to fast if no prior history
        };

        if is_fast && has_strong_history {
            return Rating::Easy;
        }

        // 7. Otherwise standard successful performance -> Good
        Rating::Good
    }
}

/// Convenience helper to derive a calibrated rating using the default policy.
pub fn derive_fsrs_rating(
    outcome: &ProceduralReviewOutcome,
    skill_state: Option<&SkillState>,
) -> Rating {
    StandardRatingPolicy::default().derive_rating(outcome, skill_state)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_outcome(
        is_correct: bool,
        score: f64,
        latency_ms: u64,
        target_latency_ms: u64,
        hints_used: u32,
        attempt_count: u32,
        error_category: Option<ErrorCategory>,
    ) -> ProceduralReviewOutcome {
        let mut outcome = ProceduralReviewOutcome::new(
            "att-test",
            "schema.test",
            "skill.test",
            "family.test",
            42,
            is_correct,
            score,
            latency_ms,
            target_latency_ms,
            hints_used,
            attempt_count,
            error_category,
        );
        outcome.timestamp = 1000;
        outcome
    }

    #[test]
    fn test_incorrect_gives_again() {
        let policy = StandardRatingPolicy::default();
        let outcome = make_outcome(false, 0.0, 20_000, 30_000, 0, 1, Some(ErrorCategory::Unknown));
        assert_eq!(policy.derive_rating(&outcome, None), Rating::Again);
    }

    #[test]
    fn test_concept_or_strategy_error_gives_again() {
        let policy = StandardRatingPolicy::default();
        let outcome = make_outcome(true, 1.0, 20_000, 30_000, 0, 1, Some(ErrorCategory::Concept));
        assert_eq!(policy.derive_rating(&outcome, None), Rating::Again);

        let outcome2 = make_outcome(true, 1.0, 20_000, 30_000, 0, 1, Some(ErrorCategory::Strategy));
        assert_eq!(policy.derive_rating(&outcome2, None), Rating::Again);
    }

    #[test]
    fn test_correct_but_slow_gives_hard() {
        let policy = StandardRatingPolicy::default();
        // Target 30s, slow threshold is 37.5s (1.25 * 30s). Latency 40s -> Hard.
        let outcome = make_outcome(true, 1.0, 40_000, 30_000, 0, 1, None);
        assert_eq!(policy.derive_rating(&outcome, None), Rating::Hard);
    }

    #[test]
    fn test_correct_with_hints_gives_hard() {
        let policy = StandardRatingPolicy::default();
        let outcome = make_outcome(true, 1.0, 20_000, 30_000, 1, 1, None);
        assert_eq!(policy.derive_rating(&outcome, None), Rating::Hard);
    }

    #[test]
    fn test_correct_with_retry_gives_hard() {
        let policy = StandardRatingPolicy::default();
        // 1 retry (attempt_count = 2) -> Hard
        let outcome = make_outcome(true, 1.0, 20_000, 30_000, 0, 2, None);
        assert_eq!(policy.derive_rating(&outcome, None), Rating::Hard);
    }

    #[test]
    fn test_multi_failure_eventual_correct_gives_again() {
        let policy = StandardRatingPolicy::default();
        // wrong -> wrong -> correct (attempt_count = 3) -> Again (P0 FSRS bug remediation)
        let outcome_3_attempts = make_outcome(true, 1.0, 20_000, 30_000, 0, 3, None);
        assert_eq!(policy.derive_rating(&outcome_3_attempts, None), Rating::Again);

        // 4 attempts -> Again
        let outcome_4_attempts = make_outcome(true, 1.0, 20_000, 30_000, 0, 4, None);
        assert_eq!(policy.derive_rating(&outcome_4_attempts, None), Rating::Again);
    }

    #[test]
    fn test_heavy_hints_give_again() {
        let policy = StandardRatingPolicy::default();
        // Level 3 hint -> Again
        let outcome_l3 = make_outcome(true, 1.0, 15_000, 30_000, 3, 1, None);
        assert_eq!(policy.derive_rating(&outcome_l3, None), Rating::Again);

        // Level 2 hint + retry -> Again
        let outcome_l2_retry = make_outcome(true, 1.0, 15_000, 30_000, 2, 2, None);
        assert_eq!(policy.derive_rating(&outcome_l2_retry, None), Rating::Again);
    }

    #[test]
    fn test_step_level_evidence_influences_rating() {
        let policy = StandardRatingPolicy::default();

        // Fast solve (15s on 30s target) with step error corrected -> Hard (never Easy/Good)
        let mut outcome_step_err = make_outcome(true, 1.0, 15_000, 30_000, 0, 1, None);
        outcome_step_err.first_error_step = Some(0);
        outcome_step_err.steps_completed = 3;
        outcome_step_err.steps_correct = 2;
        assert_eq!(policy.derive_rating(&outcome_step_err, None), Rating::Hard);

        // Fast solve with 1 hint -> Hard (never Easy)
        let outcome_hint = make_outcome(true, 1.0, 15_000, 30_000, 1, 1, None);
        assert_eq!(policy.derive_rating(&outcome_hint, None), Rating::Hard);
    }

    #[test]
    fn test_correct_at_target_latency_gives_good() {
        let policy = StandardRatingPolicy::default();
        // Target 30s, latency 30s -> Good
        let outcome = make_outcome(true, 1.0, 30_000, 30_000, 0, 1, None);
        assert_eq!(policy.derive_rating(&outcome, None), Rating::Good);
    }

    #[test]
    fn test_correct_and_fast_gives_easy_with_strong_history() {
        let policy = StandardRatingPolicy::default();
        // Target 30s, fast threshold is 22.5s (0.75 * 30s). Latency 18s.
        let outcome = make_outcome(true, 1.0, 18_000, 30_000, 0, 1, None);

        let mut state = SkillState::new("skill.test");
        state.consecutive_successes = 3;
        assert_eq!(policy.derive_rating(&outcome, Some(&state)), Rating::Easy);
    }

    #[test]
    fn test_boundary_conditions() {
        let policy = StandardRatingPolicy::default();
        let target = 40_000; // 40s
        // Exactly at target -> Good
        let o_target = make_outcome(true, 1.0, 40_000, target, 0, 1, None);
        assert_eq!(policy.derive_rating(&o_target, None), Rating::Good);

        // Just below slow threshold (1.25 * 40s = 50s): 49s -> Good
        let o_just_under_slow = make_outcome(true, 1.0, 49_000, target, 0, 1, None);
        assert_eq!(policy.derive_rating(&o_just_under_slow, None), Rating::Good);

        // Just above slow threshold: 51s -> Hard
        let o_just_over_slow = make_outcome(true, 1.0, 51_000, target, 0, 1, None);
        assert_eq!(policy.derive_rating(&o_just_over_slow, None), Rating::Hard);

        // Exactly at fast threshold (0.75 * 40s = 30s): 30s -> Easy
        let o_fast = make_outcome(true, 1.0, 30_000, target, 0, 1, None);
        assert_eq!(policy.derive_rating(&o_fast, None), Rating::Easy);

        // Just above fast threshold: 31s -> Good
        let o_not_quite_fast = make_outcome(true, 1.0, 31_000, target, 0, 1, None);
        assert_eq!(policy.derive_rating(&o_not_quite_fast, None), Rating::Good);
    }
}
