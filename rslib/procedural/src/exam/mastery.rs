// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use serde::{Deserialize, Serialize};

use crate::core::SkillId;
use crate::diagnostics::ErrorCategory;
use crate::practice::PracticeAttempt;
use crate::scheduling::Rating;
use crate::skills::SkillState;

/// Learning intervention action triggered following a learner's PYQ practice attempt.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum PyqMasteryAction {
    /// PYQ solved successfully: requires solving validated procedural variants of the same
    /// family to confirm generalized procedural competence beyond pattern memorization.
    VariantConfirmationRequired {
        skill_id: SkillId,
        suggested_variant_type: String,
        target_success_count: u32,
    },
    /// PYQ failed: requires targeted remediation on foundational procedural concepts.
    TargetedRemediationRequired {
        skill_id: SkillId,
        remediation_difficulty: u32,
        primary_error: Option<ErrorCategory>,
    },
    /// Routine mastery update recorded.
    MasteryProgress {
        skill_id: SkillId,
        new_mastery: f64,
    },
}

/// Bridge coordinating PYQ evidence with SkillState and adaptive progression.
pub struct PyqMasteryBridge;

impl PyqMasteryBridge {
    /// Evaluate the outcome of a PYQ practice attempt and determine the required pedagogical follow-up.
    pub fn evaluate_pyq_attempt(
        attempt: &PracticeAttempt,
        skill_state: &SkillState,
        error_category: Option<&ErrorCategory>,
        fsrs_rating: Rating,
    ) -> PyqMasteryAction {
        if attempt.is_correct {
            // Success on authentic PYQ: verify with structural/isomorphic variant confirmation
            let target_successes = if fsrs_rating == Rating::Easy { 1 } else { 2 };
            let variant = if skill_state.consecutive_successes >= 2 {
                "structural".to_string()
            } else {
                "isomorphic".to_string()
            };

            PyqMasteryAction::VariantConfirmationRequired {
                skill_id: attempt.skill_id.clone(),
                suggested_variant_type: variant,
                target_success_count: target_successes,
            }
        } else {
            // Failure on authentic PYQ: drop to foundational level and isolate error
            let rem_level = 1.max(skill_state.consecutive_successes.saturating_sub(1));
            PyqMasteryAction::TargetedRemediationRequired {
                skill_id: attempt.skill_id.clone(),
                remediation_difficulty: rem_level,
                primary_error: error_category.cloned(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pyq_mastery_bridge_actions() {
        let attempt_success = PracticeAttempt::new(
            "att_1",
            "inst_1",
            "schema_1",
            "skill.time_work",
            serde_json::json!({ "answer": 12 }),
            true,
            1.0,
            25_000,
        );

        let state = SkillState::new("skill.time_work");
        let action = PyqMasteryBridge::evaluate_pyq_attempt(
            &attempt_success,
            &state,
            None,
            Rating::Good,
        );

        match action {
            PyqMasteryAction::VariantConfirmationRequired { skill_id, suggested_variant_type, target_success_count } => {
                assert_eq!(skill_id.as_str(), "skill.time_work");
                assert_eq!(suggested_variant_type, "isomorphic");
                assert_eq!(target_success_count, 2);
            }
            _ => panic!("Expected VariantConfirmationRequired"),
        }

        let attempt_failure = PracticeAttempt::new(
            "att_2",
            "inst_2",
            "schema_1",
            "skill.time_work",
            serde_json::json!({ "answer": 10 }),
            false,
            0.0,
            45_000,
        );

        let action_fail = PyqMasteryBridge::evaluate_pyq_attempt(
            &attempt_failure,
            &state,
            Some(&ErrorCategory::Concept),
            Rating::Again,
        );

        match action_fail {
            PyqMasteryAction::TargetedRemediationRequired { skill_id, remediation_difficulty, primary_error } => {
                assert_eq!(skill_id.as_str(), "skill.time_work");
                assert_eq!(remediation_difficulty, 1);
                assert_eq!(primary_error, Some(ErrorCategory::Concept));
            }
            _ => panic!("Expected TargetedRemediationRequired"),
        }
    }
}
