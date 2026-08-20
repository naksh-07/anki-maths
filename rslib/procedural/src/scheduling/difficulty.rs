// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use serde::{Deserialize, Serialize};

use crate::diagnostics::ErrorCategory;
use crate::skills::{PracticeProgressionState, SkillState};

/// Outcome decision from the adaptive difficulty evaluation engine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DifficultyDecision {
    pub level: u32,
    pub target_time_ms: u64,
    pub reason: String,
}

impl DifficultyDecision {
    pub fn new(level: u32, target_time_ms: u64, reason: impl Into<String>) -> Self {
        Self {
            level: level.clamp(1, 5),
            target_time_ms,
            reason: reason.into(),
        }
    }
}

/// Deterministic, explainable Adaptive Difficulty Engine v1 with hysteresis and bounded transitions.
#[derive(Debug, Clone, Default)]
pub struct AdaptiveDifficultyEngine;

impl AdaptiveDifficultyEngine {
    /// Default target latencies for discrete difficulty levels (L1 to L5) in milliseconds.
    pub fn default_target_latency_for_level(level: u32) -> u64 {
        match level {
            1 => 25_000,
            2 => 35_000,
            3 => 50_000,
            4 => 65_000,
            _ => 80_000,
        }
    }

    /// Extract current discrete difficulty level (1..=5) from skill state metadata or recent attempts.
    pub fn current_difficulty_level(skill_state: Option<&SkillState>) -> u32 {
        let Some(state) = skill_state else {
            return 1;
        };

        if let Some(diff) = state
            .custom_state
            .get("current_difficulty_level")
            .and_then(|v| v.as_u64())
        {
            return (diff as u32).clamp(1, 5);
        }

        // Infer from last attempt if available
        if let Some(last) = state.recent_attempts.last() {
            if let Some(diff) = state
                .custom_state
                .get("last_difficulty")
                .and_then(|v| v.as_u64())
            {
                return (diff as u32).clamp(1, 5);
            }
            if last.target_latency_ms <= 28_000 {
                return 1;
            } else if last.target_latency_ms <= 40_000 {
                return 2;
            } else if last.target_latency_ms <= 55_000 {
                return 3;
            } else if last.target_latency_ms <= 70_000 {
                return 4;
            } else {
                return 5;
            }
        }

        match state.practice_state {
            PracticeProgressionState::New => 1,
            PracticeProgressionState::Learning => 1,
            PracticeProgressionState::Fluent => 2,
            PracticeProgressionState::Variation => 3,
            PracticeProgressionState::Transfer => 4,
            PracticeProgressionState::Mastered
            | PracticeProgressionState::Retired
            | PracticeProgressionState::Hibernating => 5,
        }
    }

    /// Personalize difficulty level (1..=5) deterministically based on learner signals and hysteresis rules.
    pub fn evaluate_difficulty(
        skill_state: Option<&SkillState>,
        forced_level: Option<u32>,
        latency_override: Option<u64>,
    ) -> DifficultyDecision {
        if let Some(lvl) = forced_level {
            let level = lvl.clamp(1, 5);
            let target = latency_override.unwrap_or_else(|| Self::default_target_latency_for_level(level));
            return DifficultyDecision::new(level, target, "forced_difficulty_override");
        }

        let Some(state) = skill_state else {
            // Cold start: default to Level 1
            return DifficultyDecision::new(
                1,
                latency_override.unwrap_or_else(|| Self::default_target_latency_for_level(1)),
                "cold_start_level_1",
            );
        };

        let curr_level = Self::current_difficulty_level(Some(state));
        let last_attempt = state.recent_attempts.last();
        let last_failed = last_attempt.map_or(false, |a| !a.is_correct);
        let recent_acc = state.recent_accuracy();
        let attempts_in_window = state.recent_attempts.len();

        // 1. CRITICAL DEMOTION ON CONCEPT / STRATEGY BREAKDOWN (Fast Demotion)
        if last_failed {
            if let Some(err_cat) = last_attempt.and_then(|a| a.error_category.as_ref()) {
                if matches!(err_cat, ErrorCategory::Concept | ErrorCategory::Conceptual | ErrorCategory::Strategy) {
                    let new_level = (curr_level.saturating_sub(1)).max(1);
                    let target = latency_override.unwrap_or_else(|| Self::default_target_latency_for_level(new_level));
                    return DifficultyDecision::new(
                        new_level,
                        target,
                        format!("demoted_on_concept_breakdown:L{}->L{}", curr_level, new_level),
                    );
                }
            }
        }

        // 2. REPEATED FAILURES (>= 2 consecutive failures) -> Step down by 1 level
        if state.consecutive_failures >= 2 {
            let new_level = (curr_level.saturating_sub(1)).max(1);
            let target = latency_override.unwrap_or_else(|| Self::default_target_latency_for_level(new_level));
            return DifficultyDecision::new(
                new_level,
                target,
                format!("demoted_on_consecutive_failures:L{}->L{}", curr_level, new_level),
            );
        }

        // 3. LOW RECENT ACCURACY (< 50% in window >= 3 attempts) -> Step down by 1 level
        if attempts_in_window >= 3 && recent_acc < 0.5 {
            let new_level = (curr_level.saturating_sub(1)).max(1);
            let target = latency_override.unwrap_or_else(|| Self::default_target_latency_for_level(new_level));
            return DifficultyDecision::new(
                new_level,
                target,
                format!("demoted_on_low_accuracy:L{}->L{}", curr_level, new_level),
            );
        }

        // 4. FLUENCY HOLD (Correct but slow: Latency > 1.25x target latency)
        // Keep difficulty steady to build speed without cognitive overload
        let last_latency = last_attempt.map_or(0, |a| a.latency_ms);
        let last_target = last_attempt.map_or(35_000, |a| a.target_latency_ms);
        let was_slow = last_latency > (last_target as f64 * 1.25) as u64;

        if !last_failed && was_slow {
            let target = latency_override.unwrap_or_else(|| Self::default_target_latency_for_level(curr_level));
            return DifficultyDecision::new(
                curr_level,
                target,
                format!("fluency_hold_slow_latency:L{}", curr_level),
            );
        }

        // 5. HYSTERESIS-BOUNDED PROMOTION
        // Require repeated evidence: >= 2 consecutive successes AND recent accuracy >= 0.8 AND not excessively slow
        let is_learning = state.practice_state == PracticeProgressionState::Learning || state.practice_state == PracticeProgressionState::New;
        let speed_tolerance = if is_learning { 1.50 } else { 1.15 };
        let was_fast_or_on_target = last_latency <= (last_target as f64 * speed_tolerance) as u64;
        let ready_for_promotion = state.consecutive_successes >= 2
            && recent_acc >= 0.8
            && was_fast_or_on_target
            && curr_level < 5;

        if ready_for_promotion {
            let new_level = (curr_level + 1).min(5);
            let target = latency_override.unwrap_or_else(|| Self::default_target_latency_for_level(new_level));
            return DifficultyDecision::new(
                new_level,
                target,
                format!("hysteresis_promoted:L{}->L{}", curr_level, new_level),
            );
        }

        // 6. STABLE PERFORMANCE MAINTAINED
        let target = latency_override.unwrap_or_else(|| Self::default_target_latency_for_level(curr_level));
        DifficultyDecision::new(
            curr_level,
            target,
            format!("maintained_stable_performance:L{}", curr_level),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skills::signals::{IndependenceLevel, MasteryEvidence};
    use crate::diagnostics::ErrorCategory;

    #[test]
    fn test_cold_start_defaults_to_level_1() {
        let dec = AdaptiveDifficultyEngine::evaluate_difficulty(None, None, None);
        assert_eq!(dec.level, 1);
        assert_eq!(dec.target_time_ms, 25_000);
        assert_eq!(dec.reason, "cold_start_level_1");
    }

    #[test]
    fn test_consecutive_successes_promote_with_hysteresis() {
        let mut state = SkillState::new("skill.algebra");
        state.practice_state = PracticeProgressionState::Fluent;
        // Set initial difficulty level in custom_state
        state.custom_state = serde_json::json!({ "current_difficulty_level": 2 });

        // 1st success: not enough to promote (hysteresis requires >= 2)
        let ev1 = MasteryEvidence {
            final_correctness: true,
            latency_evidence: 20_000,
            variant_exposure: Some("standard".to_string()),
            independence: IndependenceLevel::Independent,
            ..Default::default()
        };
        state.record_attempt_outcome(&ev1, 1.0, 35_000, 1000);
        let dec1 = AdaptiveDifficultyEngine::evaluate_difficulty(Some(&state), None, None);
        assert_eq!(dec1.level, 2);
        assert!(dec1.reason.contains("maintained_stable_performance"));

        // 2nd consecutive fast success: promoted to Level 3
        let ev2 = MasteryEvidence {
            final_correctness: true,
            latency_evidence: 18_000,
            variant_exposure: Some("standard".to_string()),
            independence: IndependenceLevel::Independent,
            ..Default::default()
        };
        state.record_attempt_outcome(&ev2, 1.0, 35_000, 1050);
        let dec2 = AdaptiveDifficultyEngine::evaluate_difficulty(Some(&state), None, None);
        assert_eq!(dec2.level, 3);
        assert_eq!(dec2.target_time_ms, 50_000);
        assert!(dec2.reason.contains("hysteresis_promoted"));
    }

    #[test]
    fn test_concept_error_demotes_immediately() {
        let mut state = SkillState::new("skill.algebra");
        state.custom_state = serde_json::json!({ "current_difficulty_level": 3 });

        let ev_fail = MasteryEvidence {
            final_correctness: false,
            latency_evidence: 30_000,
            variant_exposure: Some("standard".to_string()),
            diagnostic_errors: vec![ErrorCategory::Concept],
            ..Default::default()
        };
        state.record_attempt_outcome(&ev_fail, 0.0, 50_000, 1000);

        let dec = AdaptiveDifficultyEngine::evaluate_difficulty(Some(&state), None, None);
        assert_eq!(dec.level, 2);
        assert!(dec.reason.contains("demoted_on_concept_breakdown"));
    }

    #[test]
    fn test_slow_success_holds_difficulty_for_fluency() {
        let mut state = SkillState::new("skill.algebra");
        state.custom_state = serde_json::json!({ "current_difficulty_level": 3 });

        // Target is 50s, took 70s (> 1.25 * 50 = 62.5s)
        let ev_slow = MasteryEvidence {
            final_correctness: true,
            latency_evidence: 70_000,
            variant_exposure: Some("standard".to_string()),
            independence: IndependenceLevel::Independent,
            ..Default::default()
        };
        state.record_attempt_outcome(&ev_slow, 1.0, 50_000, 1000);

        let dec = AdaptiveDifficultyEngine::evaluate_difficulty(Some(&state), None, None);
        assert_eq!(dec.level, 3);
        assert!(dec.reason.contains("fluency_hold_slow_latency"));
    }

    #[test]
    fn test_difficulty_bounded_within_1_to_5() {
        let mut state = SkillState::new("skill.algebra");
        state.custom_state = serde_json::json!({ "current_difficulty_level": 1 });

        // Demoting at level 1 stays at level 1
        let ev_d1 = MasteryEvidence {
            final_correctness: false,
            latency_evidence: 30_000,
            variant_exposure: Some("standard".to_string()),
            diagnostic_errors: vec![ErrorCategory::Concept],
            ..Default::default()
        };
        state.record_attempt_outcome(&ev_d1, 0.0, 25_000, 1000);
        let dec = AdaptiveDifficultyEngine::evaluate_difficulty(Some(&state), None, None);
        assert_eq!(dec.level, 1);

        // Promoting at level 5 stays at level 5
        state.custom_state = serde_json::json!({ "current_difficulty_level": 5 });
        state.consecutive_successes = 5;
        let ev_p5a = MasteryEvidence {
            final_correctness: true,
            latency_evidence: 15_000,
            variant_exposure: Some("standard".to_string()),
            independence: IndependenceLevel::Independent,
            ..Default::default()
        };
        state.record_attempt_outcome(&ev_p5a, 1.0, 80_000, 1050);
        let ev_p5b = MasteryEvidence {
            final_correctness: true,
            latency_evidence: 15_000,
            variant_exposure: Some("standard".to_string()),
            independence: IndependenceLevel::Independent,
            ..Default::default()
        };
        state.record_attempt_outcome(&ev_p5b, 1.0, 80_000, 1100);
        let dec5 = AdaptiveDifficultyEngine::evaluate_difficulty(Some(&state), None, None);
        assert_eq!(dec5.level, 5);
    }
}
