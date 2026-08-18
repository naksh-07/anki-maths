// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use serde::{Deserialize, Serialize};

use crate::problems::steps::step_graph::{SolutionGraph, StepHint};

/// Result of requesting a deterministic hint from a problem's solution graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HintResponse {
    pub step_index: usize,
    pub step_id: String,
    pub hint_level: u32,
    pub hint_type: String,
    pub title: String,
    pub content: String,
    pub hints_remaining_for_step: usize,
    pub total_hints_used: u32,
}

/// Deterministic hint manager providing rule-based progressive mathematical assistance.
pub struct DeterministicHintSystem;

impl DeterministicHintSystem {
    /// Maximum allowed hints per step before full intermediate relation is revealed.
    pub const MAX_HINTS_PER_STEP: u32 = 3;

    /// Retrieve the next deterministic hint for the given step index and current hint count on that step.
    pub fn get_next_hint(
        graph: &SolutionGraph,
        step_index: usize,
        hints_used_on_step: u32,
        total_hints_used_so_far: u32,
    ) -> Option<HintResponse> {
        let step = graph.get_step_by_index(step_index)?;
        let requested_level = (hints_used_on_step + 1).min(Self::MAX_HINTS_PER_STEP);

        // Find hint matching the requested level or generate default deterministic fallback
        let hint = step
            .hints
            .iter()
            .find(|h| h.level == requested_level)
            .cloned()
            .unwrap_or_else(|| Self::generate_fallback_hint(step, requested_level));

        let remaining = step.hints.len().saturating_sub(requested_level as usize);

        Some(HintResponse {
            step_index,
            step_id: step.id.clone(),
            hint_level: requested_level,
            hint_type: match requested_level {
                1 => "principle".to_string(),
                2 => "operation".to_string(),
                _ => "intermediate_relation".to_string(),
            },
            title: hint.title,
            content: hint.content,
            hints_remaining_for_step: remaining,
            total_hints_used: total_hints_used_so_far + 1,
        })
    }

    /// Generate deterministic fallback hint if specific step hints are not pre-populated.
    fn generate_fallback_hint(step: &crate::problems::steps::step_graph::StepNode, level: u32) -> StepHint {
        match level {
            1 => StepHint::principle(format!(
                "Goal for this step: Focus on {} ({})",
                step.title, step.step_type
            )),
            2 => StepHint::operation(format!(
                "Operation: {}",
                step.description
            )),
            _ => StepHint::intermediate_relation(format!(
                "Target equation / relationship: {}",
                step.expected_expression
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::problems::steps::step_graph::{StepNode, StepType};

    #[test]
    fn test_deterministic_hint_progression() {
        let step = StepNode::new(
            "step_1",
            StepType::EquationRearrangement,
            "Isolate variable",
            "Subtract 5 from both sides",
            "3x = 12",
        )
        .with_hints(vec![
            StepHint::principle("Balance equations symmetrically."),
            StepHint::operation("Subtract 5 from both sides."),
            StepHint::intermediate_relation("3x = 17 - 5 = 12"),
        ]);

        let graph = SolutionGraph::new(vec![step], "step_1");

        // 1st Hint (Principle)
        let h1 = DeterministicHintSystem::get_next_hint(&graph, 0, 0, 0).unwrap();
        assert_eq!(h1.hint_level, 1);
        assert_eq!(h1.hint_type, "principle");
        assert!(h1.content.contains("Balance equations"));
        assert_eq!(h1.total_hints_used, 1);

        // 2nd Hint (Operation)
        let h2 = DeterministicHintSystem::get_next_hint(&graph, 0, 1, 1).unwrap();
        assert_eq!(h2.hint_level, 2);
        assert_eq!(h2.hint_type, "operation");
        assert!(h2.content.contains("Subtract 5"));
        assert_eq!(h2.total_hints_used, 2);

        // 3rd Hint (Intermediate Relation)
        let h3 = DeterministicHintSystem::get_next_hint(&graph, 0, 2, 2).unwrap();
        assert_eq!(h3.hint_level, 3);
        assert_eq!(h3.hint_type, "intermediate_relation");
        assert!(h3.content.contains("3x = 17 - 5 = 12"));
        assert_eq!(h3.total_hints_used, 3);
    }
}
