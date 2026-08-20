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

        // Count how many conceptual / strategy errors exist in the recent window
        let recent_conceptual_errors = state.recent_attempts.iter().filter(|a| {
            matches!(
                a.error_category,
                Some(ErrorCategory::Concept) | Some(ErrorCategory::Conceptual)
            )
        }).count();

        let recent_strategy_errors = state.recent_attempts.iter().filter(|a| {
            matches!(a.error_category, Some(ErrorCategory::Strategy))
        }).count();

        match state.practice_state {
            PracticeProgressionState::New => {
                if state.total_attempts >= 1 {
                    state.practice_state = PracticeProgressionState::Learning;
                }
            }
            PracticeProgressionState::Learning => {
                // Advance to Fluent if high accuracy, independent, and NO persistent conceptual errors.
                // Speed does NOT penalize conceptual learning in early stages.
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
                let acc_ok = attempts_in_window == 0 || recent_acc >= 0.80;
                let indep_ok = state.historical_independent_count > 0 || state.total_attempts == 0 || state.longitudinal_independence_ratio() >= 0.50;

                // If learner fails multiple times, or makes conceptual errors, drop back to Learning
                if state.consecutive_failures >= 3 || (attempts_in_window >= 4 && recent_acc < 0.4) || recent_conceptual_errors >= 2 {
                    state.practice_state = PracticeProgressionState::Learning;
                } else if state.variant_stats.len() >= 2 
                    && state.consecutive_successes >= 2 
                    && acc_ok
                    && evidence.independence == IndependenceLevel::Independent 
                    && indep_ok
                {
                    // Explored variations independently with adequate independence
                    state.practice_state = PracticeProgressionState::Variation;
                }
            }
            PracticeProgressionState::Variation => {
                let acc_ok = attempts_in_window == 0 || recent_acc >= 0.80;
                let indep_ok = state.historical_independent_count > 0 || state.total_attempts == 0 || state.longitudinal_independence_ratio() >= 0.60;

                if state.consecutive_failures >= 3 || (attempts_in_window >= 4 && recent_acc < 0.4) {
                    state.practice_state = PracticeProgressionState::Fluent;
                } else {
                    // Structural Diversity Gate for Transition to Transfer:
                    // Must have proven performance on at least 2 distinct structural/contextual forms
                    let distinct_structural = state.distinct_structural_forms_passed();
                    let distinct_variants_passed = state
                        .variant_stats
                        .values()
                        .filter(|v| v.independent_successes >= 1 || v.successful_attempts >= 2)
                        .count();
                    
                    let structural_gate_met = distinct_structural >= 2 || distinct_variants_passed >= 2 || state.variant_stats.len() >= 2;

                    if structural_gate_met 
                        && acc_ok 
                        && state.consecutive_successes >= 2
                        && recent_conceptual_errors == 0
                        && evidence.independence == IndependenceLevel::Independent 
                        && indep_ok
                    {
                        state.practice_state = PracticeProgressionState::Transfer;
                    }
                }
            }
            PracticeProgressionState::Transfer => {
                let acc_ok = attempts_in_window == 0 || (recent_acc >= 0.90 && state.consecutive_successes >= 4);
                let indep_ok = state.total_attempts == 0 || state.longitudinal_independence_ratio() >= 0.70;

                if state.consecutive_failures >= 3 || (attempts_in_window >= 4 && recent_acc < 0.4) {
                    state.practice_state = PracticeProgressionState::Variation;
                } else {
                    // MASTERY PROMOTION COMPOSITE GATE:
                    // 1. High recent accuracy and consecutive streak
                    let accuracy_met = acc_ok && (attempts_in_window == 0 || state.consecutive_successes >= 4);

                    // 2. Structural Diversity Gate: >= 3 distinct structural forms or variants passed independently
                    let structural_diversity_met = state.distinct_structural_forms_passed() >= 3 
                        || (state.variant_stats.len() >= 3 && state.variant_stats.values().all(|v| v.successful_attempts >= 1));

                    // 3. Transfer Gate: Far/transfer evidence must be verified
                    let transfer_gate_met = evidence.transfer_evidence;

                    // 4. Longitudinal Independence Gate: >= 70% unassisted lifetime success
                    let independence_met = evidence.independence == IndependenceLevel::Independent 
                        && indep_ok;

                    // 5. Delayed Retention Gate: retention survived time separation or robust multi-session history
                    let retention_met = state.delayed_retention_successes >= 1 
                        || state.has_delayed_retention_evidence(43_200_000)
                        || state.total_attempts >= 8;

                    // 6. Cognitive Decision / Strategy Quality Gate
                    let decision_quality_met = evidence.decision_quality.map_or(true, |q| q >= 0.80) 
                        && recent_strategy_errors == 0;

                    if accuracy_met 
                        && structural_diversity_met 
                        && transfer_gate_met 
                        && independence_met 
                        && retention_met 
                        && decision_quality_met
                        && recent_conceptual_errors == 0
                    {
                        state.practice_state = PracticeProgressionState::Mastered;
                    }
                }
            }
            PracticeProgressionState::Mastered => {
                if state.consecutive_failures >= 3 || (attempts_in_window >= 4 && recent_acc < 0.5) || recent_conceptual_errors >= 2 {
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
