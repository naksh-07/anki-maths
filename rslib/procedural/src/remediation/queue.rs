// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::core::SkillId;
use crate::diagnostics::ErrorCategory;
use crate::remediation::actions::{RemediationAction, RemediationUrgency};
use crate::scheduling::PracticeMode;

/// Priority queue and loop-prevention manager for procedural remediation interventions.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RemediationQueue {
    /// Pending remediation actions ordered by priority and urgency.
    pub pending_actions: Vec<RemediationAction>,
    /// Recurrence counters tracking consecutive failures for (SkillId, ErrorCategory).
    pub recurrence_tracker: HashMap<(SkillId, ErrorCategory), u32>,
    /// Maximum allowed attempts before forced escalation to prevent infinite loops.
    pub max_loop_limit: u32,
}

impl RemediationQueue {
    pub fn new() -> Self {
        Self {
            pending_actions: Vec::new(),
            recurrence_tracker: HashMap::new(),
            max_loop_limit: 4,
        }
    }

    /// Enqueue a new remediation action with loop-prevention tracking and priority sorting.
    pub fn enqueue(&mut self, mut action: RemediationAction) {
        let key = (action.skill_id.clone(), action.primary_error.clone());
        let current_recurrence = self.recurrence_tracker.entry(key.clone()).or_insert(0);
        *current_recurrence += 1;
        action.recurrence_count = *current_recurrence;

        // Remove any existing pending action for the exact same skill and error category to avoid stale duplicates
        self.pending_actions.retain(|a| !(a.skill_id == action.skill_id && a.primary_error == action.primary_error));

        self.pending_actions.push(action);
        self.sort_by_priority();
    }

    /// Sort pending actions by urgency (Critical > Normal > Advisory) then newest.
    fn sort_by_priority(&mut self) {
        self.pending_actions.sort_by(|a, b| {
            b.urgency.cmp(&a.urgency)
                .then_with(|| b.recurrence_count.cmp(&a.recurrence_count))
                .then_with(|| b.created_at.cmp(&a.created_at))
        });
    }

    /// Read-only iterator over pending actions in priority order.
    pub fn iter_pending(&self) -> impl Iterator<Item = &RemediationAction> {
        self.pending_actions.iter()
    }

    /// Select the next applicable remediation action respecting practice mode and user intent.
    pub fn select_next_remediation(&mut self, mode: &PracticeMode) -> Option<RemediationAction> {
        if self.pending_actions.is_empty() {
            return None;
        }

        // Check if user specified a focused practice mode
        let focused_skill = match mode {
            PracticeMode::FocusedSkill { skill_id } | PracticeMode::FocusedReasoningSkill { skill_id } => Some(skill_id),
            _ => None,
        };

        if let Some(target_skill) = focused_skill {
            // In focused mode: Only serve if action matches target skill OR if top action is Critical
            if let Some(idx) = self.pending_actions.iter().position(|a| &a.skill_id == target_skill) {
                return Some(self.pending_actions.remove(idx));
            }

            // If there's a Critical urgency action from another topic, let's see if we should interrupt:
            // Only interrupt if urgency == Critical AND recurrence >= 3
            if let Some(first) = self.pending_actions.first() {
                if first.urgency == RemediationUrgency::Critical && first.recurrence_count >= 3 {
                    return Some(self.pending_actions.remove(0));
                }
            }

            // Otherwise respect user intent and let focused practice proceed without hijacking
            return None;
        }

        // In mixed or open practice modes: Pop top priority remediation
        Some(self.pending_actions.remove(0))
    }

    /// Record successful resolution of a remediation, decrementing or resetting recurrence tracker.
    pub fn record_resolution(&mut self, skill_id: &SkillId, category: &ErrorCategory) {
        let key = (skill_id.clone(), category.clone());
        self.recurrence_tracker.remove(&key);
        self.pending_actions.retain(|a| !(a.skill_id == *skill_id && a.primary_error == *category));
    }

    /// Retrieve the current failure recurrence count for a skill and category.
    pub fn get_recurrence_count(&self, skill_id: &SkillId, category: &ErrorCategory) -> u32 {
        let key = (skill_id.clone(), category.clone());
        self.recurrence_tracker.get(&key).copied().unwrap_or(0)
    }

    /// Check if queue has any pending actions.
    pub fn is_empty(&self) -> bool {
        self.pending_actions.is_empty()
    }

    /// Number of pending remediation actions.
    pub fn len(&self) -> usize {
        self.pending_actions.len()
    }

    /// Clear all pending actions and recurrence counters.
    pub fn clear(&mut self) {
        self.pending_actions.clear();
        self.recurrence_tracker.clear();
    }
}
