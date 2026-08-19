// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use crate::skills::signals::{IndependenceLevel, MasteryEvidence, PracticeProgressionState};
use crate::skills::SkillState;
use crate::diagnostics::ErrorCategory;

/// Evaluates mastery evidence against deterministic progression rules.
pub struct ProgressionPolicy;

impl ProgressionPolicy {
    /// Evaluates deterministic progression between skill development stages using comprehensive evidence.
    pub fn evaluate(state: &mut SkillState, evidence: &MasteryEvidence) {
        let recent_acc = state.recent_accuracy();
        let attempts_in_window = state.recent_attempts.len();

        // Count how many conceptual errors exist in the recent window
        let recent_conceptual_errors = state.recent_attempts.iter().filter(|a| {
            matches!(
                a.error_category,
                Some(ErrorCategory::Concept) | Some(ErrorCategory::Conceptual)
            )
        }).count();

        match state.practice_state {
            PracticeProgressionState::New => {
                if state.total_attempts >= 1 {
                    state.practice_state = PracticeProgressionState::Learning;
                }
            }
            PracticeProgressionState::Learning => {
                // Advance to Fluent if high accuracy, independent, and NO persistent conceptual errors
                if attempts_in_window >= 3 
                    && recent_acc >= 0.8 
                    && state.consecutive_successes >= 3 
                    && recent_conceptual_errors == 0 
                    && (evidence.independence == IndependenceLevel::Independent || evidence.independence == IndependenceLevel::LightSupport)
                {
                    state.practice_state = PracticeProgressionState::Fluent;
                }
            }
            PracticeProgressionState::Fluent => {
                // If learner fails multiple times, or makes conceptual errors, drop back to Learning
                if state.consecutive_failures >= 3 || (attempts_in_window >= 4 && recent_acc < 0.4) || recent_conceptual_errors >= 2 {
                    state.practice_state = PracticeProgressionState::Learning;
                } else if state.variant_stats.len() >= 2 && state.consecutive_successes >= 2 && evidence.independence == IndependenceLevel::Independent {
                    // Explored variations independently
                    state.practice_state = PracticeProgressionState::Variation;
                }
            }
            PracticeProgressionState::Variation => {
                if state.consecutive_failures >= 3 || (attempts_in_window >= 4 && recent_acc < 0.4) {
                    state.practice_state = PracticeProgressionState::Fluent;
                } else {
                    let distinct_passed = state
                        .variant_stats
                        .values()
                        .filter(|v| v.successful_attempts >= 2)
                        .count();
                    
                    if distinct_passed >= 2 && recent_acc >= 0.8 && evidence.independence == IndependenceLevel::Independent {
                        state.practice_state = PracticeProgressionState::Transfer;
                    }
                }
            }
            PracticeProgressionState::Transfer => {
                if state.consecutive_failures >= 3 || (attempts_in_window >= 4 && recent_acc < 0.4) {
                    state.practice_state = PracticeProgressionState::Variation;
                } else if state.consecutive_successes >= 5 
                    && recent_acc >= 0.9 
                    && state.variant_stats.len() >= 3 
                    && evidence.transfer_evidence 
                    && evidence.independence == IndependenceLevel::Independent 
                {
                    state.practice_state = PracticeProgressionState::Mastered;
                }
            }
            PracticeProgressionState::Mastered => {
                if state.consecutive_failures >= 3 || (attempts_in_window >= 4 && recent_acc < 0.5) {
                    state.practice_state = PracticeProgressionState::Transfer;
                }
            }
            PracticeProgressionState::Retired | PracticeProgressionState::Hibernating => {
                if state.consecutive_failures >= 2 || (attempts_in_window >= 3 && recent_acc < 0.6) {
                    state.practice_state = PracticeProgressionState::Variation;
                }
            }
        }
    }
}
