// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::core::{Domain, SchemaId, SkillId};
use crate::skills::SkillState;

/// Severity classification of learner overdue review backlog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BacklogSeverity {
    /// 0 to 5 overdue items: Process normally without triage intervention.
    Mild,
    /// 6 to 15 overdue items: Prioritize critical forgetting risks and foundational dependencies.
    Moderate,
    /// > 15 overdue items: Staged reactivation quota; defer stable low-risk maintenance.
    Severe,
}

/// Triage candidate item evaluating retrievability and pedagogical urgency.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TriagedBacklogItem {
    pub schema_id: SchemaId,
    pub skill_id: SkillId,
    pub domain: Domain,
    pub elapsed_days: f64,
    pub estimated_stability_days: f64,
    pub estimated_retrievability: f64,
    pub effective_prereq_value: f64,
    pub triage_priority_score: f64,
    pub is_deferred: bool,
    pub deferral_reason: Option<String>,
}

/// Backlog triage plan produced after inactivity or heavy accumulated due cards.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BacklogTriagePlan {
    pub total_overdue_count: usize,
    pub severity: BacklogSeverity,
    pub active_quota: usize,
    pub deferred_count: usize,
    pub prioritized_items: Vec<TriagedBacklogItem>,
    pub explanation: String,
}

/// Retrievability-aware backlog triage engine.
pub struct BacklogTriageEngine;

impl BacklogTriageEngine {
    /// Calculate exponential retrievability R(Δt, S) = exp(-Δt / S).
    pub fn estimate_retrievability(elapsed_days: f64, stability_days: f64) -> f64 {
        let s = stability_days.max(0.5);
        let delta = elapsed_days.max(0.0);
        (-delta / s).exp().clamp(0.01, 1.0)
    }

    /// Triage overdue items according to retrievability, importance, and session capacity.
    pub fn triage_backlog(
        overdue_candidates: &[(SchemaId, SkillId, Domain, f64)], // (schema_id, skill_id, domain, elapsed_days)
        skill_states: &HashMap<SkillId, SkillState>,
        effective_prereq_values: &HashMap<SkillId, f64>,
        max_session_memory_quota: usize,
    ) -> BacklogTriagePlan {
        let total_count = overdue_candidates.len();

        let severity = if total_count <= 5 {
            BacklogSeverity::Mild
        } else if total_count <= 15 {
            BacklogSeverity::Moderate
        } else {
            BacklogSeverity::Severe
        };

        if total_count == 0 {
            return BacklogTriagePlan {
                total_overdue_count: 0,
                severity: BacklogSeverity::Mild,
                active_quota: 0,
                deferred_count: 0,
                prioritized_items: Vec::new(),
                explanation: "No overdue backlog detected.".to_string(),
            };
        }

        let mut scored_items = Vec::new();

        for (schema_id, skill_id, domain, elapsed_days) in overdue_candidates {
            let state_opt = skill_states.get(skill_id);
            let stability_days = match state_opt {
                Some(s) => {
                    // Estimate stability from consecutive successes and historical attempts
                    let base = 3.0 + (s.consecutive_successes as f64 * 4.0);
                    base.min(90.0)
                }
                None => 4.0, // Cold start default
            };

            let r = Self::estimate_retrievability(*elapsed_days, stability_days);
            let prereq_val = effective_prereq_values.get(skill_id).copied().unwrap_or(1.0);
            let past_failures = state_opt.map_or(0, |s| s.consecutive_failures);

            // Triage priority score: (1 - R) * PrereqValue * (1 + 0.5 * Failures)
            let priority_score = (1.0 - r) * prereq_val * (1.0 + 0.5 * past_failures as f64);

            scored_items.push(TriagedBacklogItem {
                schema_id: schema_id.clone(),
                skill_id: skill_id.clone(),
                domain: domain.clone(),
                elapsed_days: *elapsed_days,
                estimated_stability_days: stability_days,
                estimated_retrievability: r,
                effective_prereq_value: prereq_val,
                triage_priority_score: priority_score,
                is_deferred: false,
                deferral_reason: None,
            });
        }

        // Sort descending by priority score (highest forgetting risk / highest prerequisite value first)
        scored_items.sort_by(|a, b| {
            b.triage_priority_score
                .partial_cmp(&a.triage_priority_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Determine active quota based on severity and session budget
        let active_quota = match severity {
            BacklogSeverity::Mild => total_count,
            BacklogSeverity::Moderate => max_session_memory_quota.max(8).min(total_count),
            BacklogSeverity::Severe => max_session_memory_quota.max(10).min(total_count),
        };

        let mut deferred_count = 0;
        for (i, item) in scored_items.iter_mut().enumerate() {
            if i >= active_quota {
                item.is_deferred = true;
                item.deferral_reason = Some(format!(
                    "Deferred under {:?} backlog triage (R={:.2}, staged for subsequent session)",
                    severity, item.estimated_retrievability
                ));
                deferred_count += 1;
            }
        }

        let explanation = match severity {
            BacklogSeverity::Mild => format!("Mild backlog ({} items): processing all items directly.", total_count),
            BacklogSeverity::Moderate => format!(
                "Moderate backlog ({} items): prioritized {} highest-risk items, deferring {}.",
                total_count, active_quota, deferred_count
            ),
            BacklogSeverity::Severe => format!(
                "Severe backlog ({} items after inactivity): staged reactivation serving {} items, safely deferring {}.",
                total_count, active_quota, deferred_count
            ),
        };

        BacklogTriagePlan {
            total_overdue_count: total_count,
            severity,
            active_quota,
            deferred_count,
            prioritized_items: scored_items,
            explanation,
        }
    }
}
