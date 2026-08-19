// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use serde::{Deserialize, Serialize};

/// A structured option for a cognitive decision point.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecisionOption {
    pub id: String,
    pub label: String,
    /// String identifier of the chosen strategy (e.g. "anchor_fixed", "propagate_adjacency").
    pub strategy: String,
    pub is_valid: bool,
    pub feedback: String,
}

impl DecisionOption {
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        strategy: impl Into<String>,
        is_valid: bool,
        feedback: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            strategy: strategy.into(),
            is_valid,
            feedback: feedback.into(),
        }
    }
}

/// Micro learning object capturing strategic reasoning choice before or during problem solving.
/// This acts as mastery evidence for the learner's procedural intent, distinct from their
/// ability to calculate the final answer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CognitiveDecisionPoint {
    pub id: String,
    pub prompt: String,
    pub options: Vec<DecisionOption>,
    pub preferred_option_id: String,
    pub preferred_strategy: String,
    pub explanation: String,
}

impl CognitiveDecisionPoint {
    pub fn new(
        id: impl Into<String>,
        prompt: impl Into<String>,
        options: Vec<DecisionOption>,
        preferred_option_id: impl Into<String>,
        preferred_strategy: impl Into<String>,
        explanation: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            prompt: prompt.into(),
            options,
            preferred_option_id: preferred_option_id.into(),
            preferred_strategy: preferred_strategy.into(),
            explanation: explanation.into(),
        }
    }

    /// Evaluate a learner's decision choice.
    pub fn evaluate_choice(&self, chosen_id: &str) -> (bool, Option<String>, String) {
        if let Some(opt) = self.options.iter().find(|o| o.id == chosen_id) {
            (opt.is_valid, Some(opt.strategy.clone()), opt.feedback.clone())
        } else {
            (false, None, "Invalid decision option chosen.".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decision_point_evaluation() {
        let opt1 = DecisionOption::new(
            "opt_a",
            "Anchor fixed person C at position 3",
            "anchor_fixed",
            true,
            "Correct: Fixed positions immediately reduce variable domains.",
        );
        let opt2 = DecisionOption::new(
            "opt_b",
            "Try placing flexible person A randomly",
            "branch_cases",
            false,
            "Sub-optimal: Guessing without anchoring fixed positions increases branching.",
        );

        let dp = CognitiveDecisionPoint::new(
            "dp_seating_1",
            "Which step should you perform first?",
            vec![opt1, opt2],
            "opt_a",
            "anchor_fixed",
            "Always anchor invariant fixed positions before relative constraints.",
        );

        let (is_valid, strategy, feedback) = dp.evaluate_choice("opt_a");
        assert!(is_valid);
        assert_eq!(strategy, Some("anchor_fixed".to_string()));
        assert!(feedback.contains("Fixed positions"));

        let (is_valid2, strategy2, _) = dp.evaluate_choice("opt_b");
        assert!(!is_valid2);
        assert_eq!(strategy2, Some("branch_cases".to_string()));
    }
}
