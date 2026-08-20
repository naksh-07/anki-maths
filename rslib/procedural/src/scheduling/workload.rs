// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use serde::{Deserialize, Serialize};

use std::collections::HashMap;

use crate::core::Domain;
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

/// State tracker for active session budget enforcement with multi-domain and cognitive load awareness.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionBudgetTracker {
    pub budget: Option<SessionBudget>,
    pub items_completed: usize,
    pub elapsed_ms: u64,
    pub remediations_served: usize,
    pub remediation_elapsed_ms: u64,
    pub domain_elapsed_ms: HashMap<Domain, u64>,
    pub domain_items: HashMap<Domain, usize>,
    pub total_cognitive_load: f64,
    pub max_cognitive_load: Option<f64>,
    pub is_exhausted: bool,
}

impl SessionBudgetTracker {
    pub fn new(budget: Option<SessionBudget>) -> Self {
        Self {
            budget,
            items_completed: 0,
            elapsed_ms: 0,
            remediations_served: 0,
            remediation_elapsed_ms: 0,
            domain_elapsed_ms: HashMap::new(),
            domain_items: HashMap::new(),
            total_cognitive_load: 0.0,
            max_cognitive_load: None,
            is_exhausted: false,
        }
    }

    pub fn with_max_cognitive_load(mut self, max_load: f64) -> Self {
        self.max_cognitive_load = Some(max_load);
        self
    }

    /// Evaluates cognitive load weight based on subject complexity and difficulty level.
    pub fn calculate_cognitive_weight(domain: &Domain, difficulty_level: u32) -> f64 {
        let base_weight = match domain {
            Domain::Mathematics => 1.0,
            Domain::Physics => 1.4,
            Domain::Chemistry => 1.3,
            Domain::Reasoning => 1.2,
            Domain::Custom(_) => 1.0,
        };
        let diff_multiplier = match difficulty_level {
            1 => 0.8,
            2 => 1.0,
            3 => 1.3,
            4 => 1.7,
            _ => 2.2, // Level 5 (Multi-concept / transfer)
        };
        base_weight * diff_multiplier
    }

    /// Records an item completed with its domain, latency, and difficulty.
    pub fn record_item_with_domain(
        &mut self,
        domain: &Domain,
        latency_ms: u64,
        is_remediation: bool,
        difficulty_level: u32,
    ) {
        self.items_completed += 1;
        self.elapsed_ms = self.elapsed_ms.saturating_add(latency_ms);

        let dom_ms = self.domain_elapsed_ms.entry(domain.clone()).or_insert(0);
        *dom_ms = dom_ms.saturating_add(latency_ms);

        let dom_count = self.domain_items.entry(domain.clone()).or_insert(0);
        *dom_count += 1;

        if is_remediation {
            self.remediations_served += 1;
            self.remediation_elapsed_ms = self.remediation_elapsed_ms.saturating_add(latency_ms);
        }

        let cog_weight = Self::calculate_cognitive_weight(domain, difficulty_level);
        self.total_cognitive_load += cog_weight;

        self.check_exhaustion();
    }

    /// Records an item completed with its latency and updates exhaustion status.
    pub fn record_item(&mut self, latency_ms: u64, is_remediation: bool) {
        self.record_item_with_domain(&Domain::Mathematics, latency_ms, is_remediation, 2);
    }

    /// Checks if budget limits have been met.
    pub fn check_exhaustion(&mut self) -> bool {
        let budget_exhausted = match self.budget {
            None => false,
            Some(SessionBudget::ItemCount { max_items }) => self.items_completed >= max_items,
            Some(SessionBudget::TimeLimitMs { max_time_ms }) => self.elapsed_ms >= max_time_ms,
            Some(SessionBudget::Bounded {
                max_items,
                max_time_ms,
            }) => self.items_completed >= max_items || self.elapsed_ms >= max_time_ms,
        };

        let cog_exhausted = self
            .max_cognitive_load
            .map_or(false, |limit| self.total_cognitive_load >= limit);

        self.is_exhausted = budget_exhausted || cog_exhausted;
        self.is_exhausted
    }

    /// Checks whether a specific domain has exceeded its allocated time budget.
    pub fn is_domain_exhausted(&self, domain: &Domain, allocated_time_ms: u64) -> bool {
        let elapsed = self.domain_elapsed_ms.get(domain).copied().unwrap_or(0);
        elapsed >= allocated_time_ms
    }

    /// Checks whether another remediation intervention is permitted under session safeguards.
    pub fn can_serve_remediation(&self, safeguards: &WorkloadSafeguards) -> bool {
        self.can_serve_remediation_with_cap(safeguards, None)
    }

    /// Checks whether another remediation intervention is permitted under session safeguards and budget caps.
    pub fn can_serve_remediation_with_cap(&self, safeguards: &WorkloadSafeguards, max_remediation_ms: Option<u64>) -> bool {
        if self.remediations_served >= safeguards.max_remediations_per_session {
            return false;
        }
        if let Some(max_ms) = max_remediation_ms {
            if self.remediation_elapsed_ms >= max_ms {
                return false;
            }
        }
        true
    }
}
