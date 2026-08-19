// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use serde::{Deserialize, Serialize};

use crate::practice::SessionBudget;

/// Classification of current practice queue pressure and learner workload.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkloadState {
    /// Workload is manageable; standard advancement, transfer, and new content allowed.
    Sustainable,
    /// Backlog or queue is elevated; throttling non-essential new content.
    Heavy,
    /// Critical backlog; strictly limiting selection to critical remediation and maintenance.
    Overloaded,
}

impl Default for WorkloadState {
    fn default() -> Self {
        WorkloadState::Sustainable
    }
}

/// Workload load metrics estimating current pedagogical queue pressure.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct WorkloadSnapshot {
    pub pending_remediation_count: usize,
    pub critical_remediation_count: usize,
    pub due_memory_reviews: usize,
    pub active_learning_skills: usize,
    pub transfer_pending_count: usize,
    pub total_composite_load: usize,
}

impl WorkloadSnapshot {
    pub fn compute_state(&self) -> WorkloadState {
        if self.critical_remediation_count >= 3
            || self.pending_remediation_count >= 6
            || self.total_composite_load >= 25
        {
            WorkloadState::Overloaded
        } else if self.critical_remediation_count >= 1
            || self.pending_remediation_count >= 3
            || self.total_composite_load >= 12
        {
            WorkloadState::Heavy
        } else {
            WorkloadState::Sustainable
        }
    }
}

/// Safeguards preventing infinite remediation chains and review explosions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkloadSafeguards {
    /// Maximum remediation interventions allowed within a single study session.
    pub max_remediations_per_session: usize,
    /// Maximum depth of prerequisite chain expansion.
    pub max_prerequisite_depth: usize,
    /// Maximum concurrent new skills in active Learning state.
    pub max_concurrent_new_skills: usize,
}

impl Default for WorkloadSafeguards {
    fn default() -> Self {
        Self {
            max_remediations_per_session: 2,
            max_prerequisite_depth: 10,
            max_concurrent_new_skills: 4,
        }
    }
}

/// State tracker for active session budget enforcement.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionBudgetTracker {
    pub budget: Option<SessionBudget>,
    pub items_completed: usize,
    pub elapsed_ms: u64,
    pub remediations_served: usize,
    pub is_exhausted: bool,
}

impl SessionBudgetTracker {
    pub fn new(budget: Option<SessionBudget>) -> Self {
        Self {
            budget,
            items_completed: 0,
            elapsed_ms: 0,
            remediations_served: 0,
            is_exhausted: false,
        }
    }

    /// Records an item completed with its latency and updates exhaustion status.
    pub fn record_item(&mut self, latency_ms: u64, is_remediation: bool) {
        self.items_completed += 1;
        self.elapsed_ms = self.elapsed_ms.saturating_add(latency_ms);
        if is_remediation {
            self.remediations_served += 1;
        }

        self.check_exhaustion();
    }

    /// Checks if budget limits have been met.
    pub fn check_exhaustion(&mut self) -> bool {
        match self.budget {
            None => {
                self.is_exhausted = false;
            }
            Some(SessionBudget::ItemCount { max_items }) => {
                self.is_exhausted = self.items_completed >= max_items;
            }
            Some(SessionBudget::TimeLimitMs { max_time_ms }) => {
                self.is_exhausted = self.elapsed_ms >= max_time_ms;
            }
            Some(SessionBudget::Bounded {
                max_items,
                max_time_ms,
            }) => {
                self.is_exhausted =
                    self.items_completed >= max_items || self.elapsed_ms >= max_time_ms;
            }
        }
        self.is_exhausted
    }

    /// Checks whether another remediation intervention is permitted under session safeguards.
    pub fn can_serve_remediation(&self, safeguards: &WorkloadSafeguards) -> bool {
        self.remediations_served < safeguards.max_remediations_per_session
    }
}
