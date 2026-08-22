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

    /// Enqueue a new remediation action with loop-prevention tracking, same-skill consolidation, and priority sorting.
    pub fn enqueue(&mut self, mut action: RemediationAction) {
        let key = (action.skill_id.clone(), action.primary_error.clone());
        let current_recurrence = self.recurrence_tracker.entry(key.clone()).or_insert(0);
        *current_recurrence += 1;
        let total_skill_recurrence: u32 = self
            .recurrence_tracker
            .iter()
            .filter(|((s, _), _)| s == &action.skill_id)
            .map(|(_, count)| *count)
            .sum();
        action.recurrence_count = total_skill_recurrence;

        // Same-skill consolidation: if an action for the same skill already exists, compact into one authoritative action
        if let Some(pos) = self.pending_actions.iter().position(|a| a.skill_id == action.skill_id) {
            let existing = self.pending_actions.remove(pos);
            let merged = Self::consolidate_same_skill_actions(existing, action);
            self.pending_actions.push(merged);
        } else {
            self.pending_actions.push(action);
        }

        self.sort_by_priority();
    }

    /// Consolidate two same-skill remediation actions preserving highest urgency, highest recurrence,
    /// strongest intervention tier, and most recent evidence.
    fn consolidate_same_skill_actions(existing: RemediationAction, new_action: RemediationAction) -> RemediationAction {
        debug_assert_eq!(existing.skill_id, new_action.skill_id);

        let merged_urgency = existing.urgency.max(new_action.urgency);
        let merged_recurrence = existing.recurrence_count.max(new_action.recurrence_count);
        let merged_created_at = existing.created_at.max(new_action.created_at);
        let merged_ack = existing.requires_acknowledgement || new_action.requires_acknowledgement;
        let merged_difficulty = existing.preferred_difficulty.min(new_action.preferred_difficulty);

        let existing_tier = existing.kind.precedence_tier();
        let new_tier = new_action.kind.precedence_tier();

        if new_tier >= existing_tier {
            RemediationAction {
                id: new_action.id,
                kind: new_action.kind,
                skill_id: new_action.skill_id,
                schema_id: new_action.schema_id,
                domain: new_action.domain,
                primary_error: new_action.primary_error,
                step_error: new_action.step_error.or(existing.step_error),
                preferred_difficulty: merged_difficulty,
                preferred_variant: new_action.preferred_variant.or(existing.preferred_variant),
                source_attempt_id: new_action.source_attempt_id,
                urgency: merged_urgency,
                requires_acknowledgement: merged_ack,
                recurrence_count: merged_recurrence,
                rationale: new_action.rationale,
                created_at: merged_created_at,
            }
        } else {
            RemediationAction {
                id: existing.id,
                kind: existing.kind,
                skill_id: existing.skill_id,
                schema_id: existing.schema_id,
                domain: existing.domain,
                primary_error: existing.primary_error,
                step_error: existing.step_error.or(new_action.step_error),
                preferred_difficulty: merged_difficulty,
                preferred_variant: existing.preferred_variant.or(new_action.preferred_variant),
                source_attempt_id: new_action.source_attempt_id,
                urgency: merged_urgency,
                requires_acknowledgement: merged_ack,
                recurrence_count: merged_recurrence,
                rationale: existing.rationale,
                created_at: merged_created_at,
            }
        }
    }

    /// Compact the entire queue, ensuring at most one authoritative pending action per skill.
    pub fn compact(&mut self) {
        if self.pending_actions.len() <= 1 {
            return;
        }

        let mut compacted_map: HashMap<SkillId, RemediationAction> = HashMap::new();
        for action in self.pending_actions.drain(..) {
            compacted_map
                .entry(action.skill_id.clone())
                .and_modify(|existing| {
                    let merged = Self::consolidate_same_skill_actions(existing.clone(), action.clone());
                    *existing = merged;
                })
                .or_insert(action);
        }

        self.pending_actions = compacted_map.into_values().collect();
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
        let has_other_recurrences = self.recurrence_tracker.keys().any(|(s, _)| s == skill_id);
        if !has_other_recurrences {
            self.pending_actions.retain(|a| a.skill_id != *skill_id);
        } else {
            self.pending_actions.retain(|a| !(a.skill_id == *skill_id && a.primary_error == *category));
        }
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
