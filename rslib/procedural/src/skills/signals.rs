// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::diagnostics::ErrorCategory;

/// Deterministic stage of procedural skill development.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PracticeProgressionState {
    /// Newly introduced skill without sufficient practice history.
    New = 0,
    /// Building initial procedural knowledge and basic calculation steps.
    Learning = 1,
    /// High accuracy and consistent execution on standard problem variants.
    Fluent = 2,
    /// Practicing diverse structural and contextual problem variants.
    Variation = 3,
    /// Robust performance across non-standard and multi-step variations.
    Transfer = 4,
    /// High-speed, high-accuracy automaticity across all variant templates.
    Mastered = 5,
    /// Retired from active practice into low-frequency maintenance.
    Retired = 6,
    /// Hibernating temporarily due to workload regulation.
    Hibernating = 7,
}

impl Default for PracticeProgressionState {
    fn default() -> Self {
        Self::New
    }
}

impl PracticeProgressionState {
    pub fn as_str(&self) -> &'static str {
        match self {
            PracticeProgressionState::New => "new",
            PracticeProgressionState::Learning => "learning",
            PracticeProgressionState::Fluent => "fluent",
            PracticeProgressionState::Variation => "variation",
            PracticeProgressionState::Transfer => "transfer",
            PracticeProgressionState::Mastered => "mastered",
            PracticeProgressionState::Retired => "retired",
            PracticeProgressionState::Hibernating => "hibernating",
        }
    }

    /// Whether this skill is in a mature or maintenance state.
    pub fn is_mature(&self) -> bool {
        matches!(
            self,
            PracticeProgressionState::Mastered
                | PracticeProgressionState::Retired
                | PracticeProgressionState::Hibernating
        )
    }
}

/// Category of problem variation distinguishing surface parameter shifts from genuine structural changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum VariantCategory {
    /// Pure parameter changes with identical algebraic/topological structure.
    #[default]
    Parameter = 0,
    /// Same structural steps with different surface context/wording.
    Isomorphic = 1,
    /// Modified solution graph topology, variable inversion, or additional step.
    Structural = 2,
    /// Alternate physical, conceptual, or domain representation.
    Contextual = 3,
    /// Coupled multi-schema integration requiring composite decision points.
    MultiConcept = 4,
    /// Far/novel transfer requiring deep schema abstraction.
    Transfer = 5,
}

impl VariantCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            VariantCategory::Parameter => "parameter",
            VariantCategory::Isomorphic => "isomorphic",
            VariantCategory::Structural => "structural",
            VariantCategory::Contextual => "contextual",
            VariantCategory::MultiConcept => "multi_concept",
            VariantCategory::Transfer => "transfer",
        }
    }

    /// Whether this variant provides structural or transfer evidence (i.e. beyond shallow template familiarity).
    pub fn is_structural_or_transfer(&self) -> bool {
        *self >= VariantCategory::Structural
    }
}

/// Snapshot of a single recent practice attempt used in sliding performance windows.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecentAttemptRecord {
    pub is_correct: bool,
    pub score: f64,
    pub latency_ms: u64,
    pub target_latency_ms: u64,
    pub variant: Option<String>,
    #[serde(default)]
    pub variant_category: Option<VariantCategory>,
    pub error_category: Option<ErrorCategory>,
    #[serde(default)]
    pub max_hint_level: Option<u32>,
    #[serde(default)]
    pub hint_count: Option<u32>,
    #[serde(default)]
    pub independence: Option<IndependenceLevel>,
    #[serde(default)]
    pub solution_graph_fingerprint: Option<String>,
    #[serde(default)]
    pub cognitive_decision_correct: Option<bool>,
    #[serde(default)]
    pub domain_evidence: Option<crate::skills::domain_evidence::VersionedDomainEvidence>,
    pub timestamp: i64,
}

/// Sliding window statistics for solution latencies.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MovingLatencyStats {
    pub recent_latencies: Vec<u64>,
    pub moving_average_ms: f64,
    pub min_latency_ms: Option<u64>,
    pub max_latency_ms: Option<u64>,
    pub moving_variance: Option<f64>,
}

impl MovingLatencyStats {
    pub fn update(&mut self, latency_ms: u64, window_size: usize) {
        self.recent_latencies.push(latency_ms);
        if self.recent_latencies.len() > window_size {
            self.recent_latencies.remove(0);
        }

        if self.recent_latencies.is_empty() {
            self.moving_average_ms = 0.0;
            self.min_latency_ms = None;
            self.max_latency_ms = None;
            self.moving_variance = None;
            return;
        }

        let count = self.recent_latencies.len() as f64;
        let sum: u64 = self.recent_latencies.iter().sum();
        let mean = sum as f64 / count;
        self.moving_average_ms = mean;

        let mut min_val = u64::MAX;
        let mut max_val = 0;
        let mut var_sum = 0.0;

        for &val in &self.recent_latencies {
            if val < min_val {
                min_val = val;
            }
            if val > max_val {
                max_val = val;
            }
            let diff = val as f64 - mean;
            var_sum += diff * diff;
        }

        self.min_latency_ms = Some(min_val);
        self.max_latency_ms = Some(max_val);
        self.moving_variance = Some(var_sum / count);
    }
}

/// Frequency counters for diagnostic error categories.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorFrequencyCounts {
    pub concept: u32,
    pub strategy: u32,
    pub calculation: u32,
    pub careless: u32,
    pub time: u32,
    pub unknown: u32,
    pub custom: HashMap<String, u32>,
}

impl ErrorFrequencyCounts {
    pub fn record(&mut self, category: &ErrorCategory) {
        match category {
            ErrorCategory::Concept | ErrorCategory::Conceptual => self.concept += 1,
            ErrorCategory::Strategy => self.strategy += 1,
            ErrorCategory::Calculation | ErrorCategory::Sign | ErrorCategory::Unit => self.calculation += 1,
            ErrorCategory::Careless | ErrorCategory::ProceduralSlip | ErrorCategory::Syntax => self.careless += 1,
            ErrorCategory::Time | ErrorCategory::Timeout => self.time += 1,
            ErrorCategory::Unknown => self.unknown += 1,
            ErrorCategory::DomainSpecific(key) => {
                *self.custom.entry(key.clone()).or_insert(0) += 1;
            }
        }
    }

    pub fn get_count(&self, category: &ErrorCategory) -> u32 {
        match category {
            ErrorCategory::Concept | ErrorCategory::Conceptual => self.concept,
            ErrorCategory::Strategy => self.strategy,
            ErrorCategory::Calculation | ErrorCategory::Sign | ErrorCategory::Unit => self.calculation,
            ErrorCategory::Careless | ErrorCategory::ProceduralSlip | ErrorCategory::Syntax => self.careless,
            ErrorCategory::Time | ErrorCategory::Timeout => self.time,
            ErrorCategory::Unknown => self.unknown,
            ErrorCategory::DomainSpecific(key) => self.custom.get(key).copied().unwrap_or(0),
        }
    }

    pub fn total_errors(&self) -> u32 {
        self.concept
            + self.strategy
            + self.calculation
            + self.careless
            + self.time
            + self.unknown
            + self.custom.values().sum::<u32>()
    }

    pub fn primary_error_category(&self) -> Option<ErrorCategory> {
        let mut best_count = 0;
        let mut best_cat = None;

        let checks = [
            (self.concept, ErrorCategory::Concept),
            (self.strategy, ErrorCategory::Strategy),
            (self.calculation, ErrorCategory::Calculation),
            (self.careless, ErrorCategory::Careless),
            (self.time, ErrorCategory::Time),
            (self.unknown, ErrorCategory::Unknown),
        ];

        for (cnt, cat) in checks {
            if cnt > best_count {
                best_count = cnt;
                best_cat = Some(cat);
            }
        }

        for (k, &cnt) in &self.custom {
            if cnt > best_count {
                best_count = cnt;
                best_cat = Some(ErrorCategory::DomainSpecific(k.clone()));
            }
        }

        best_cat
    }
}

/// Historical performance metrics for a specific problem variant.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct VariantPerformance {
    pub total_attempts: u32,
    pub successful_attempts: u32,
    pub failed_attempts: u32,
    pub average_latency_ms: f64,
    pub last_practiced_at: Option<i64>,
    pub last_error_category: Option<ErrorCategory>,
    #[serde(default)]
    pub category: VariantCategory,
    #[serde(default)]
    pub independent_successes: u32,
}

impl VariantPerformance {
    pub fn record(
        &mut self,
        is_correct: bool,
        latency_ms: u64,
        error_category: Option<ErrorCategory>,
        timestamp: i64,
    ) {
        let prev_total = self.total_attempts as f64;
        self.total_attempts += 1;
        if is_correct {
            self.successful_attempts += 1;
        } else {
            self.failed_attempts += 1;
        }

        // Running average latency
        self.average_latency_ms = (self.average_latency_ms * prev_total + latency_ms as f64) / (self.total_attempts as f64);
        self.last_practiced_at = Some(timestamp);
        if error_category.is_some() {
            self.last_error_category = error_category;
        }
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_attempts == 0 {
            0.0
        } else {
            self.successful_attempts as f64 / self.total_attempts as f64
        }
    }
}

/// The level of independence a learner demonstrated during a practice attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum IndependenceLevel {
    /// Solved completely independently with no hints, retries, or external assistance.
    #[default]
    Independent,
    /// Required minor assistance, such as a conceptual nudge or error-specific hint.
    LightSupport,
    /// Required significant step-by-step guidance or multiple retries to reach the solution.
    SignificantSupport,
    /// Could not solve the problem; relied entirely on bottom-out hints or gave up.
    NonIndependent,
}

/// A comprehensive compilation of evidence gathered during a practice attempt,
/// used to evaluate progression and mastery transitions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MasteryEvidence {
    /// Whether the final answer was correct.
    pub final_correctness: bool,
    /// The ratio of correct cognitive decisions made (if any decision points were present).
    pub decision_quality: Option<f64>,
    /// The ratio of correct steps executed (if step-level tracking is available).
    pub step_quality: Option<f64>,
    /// The level of independence demonstrated.
    pub independence: IndependenceLevel,
    /// Maximum hint level reached during this attempt (0=none, 1=cue, 2=scaffold, 3=bottom-out).
    #[serde(default)]
    pub max_hint_level: Option<u32>,
    /// Total number of hints explicitly requested.
    pub hint_dependence: u32,
    /// Total number of retry attempts required.
    pub retry_dependence: u32,
    /// The specific structural or contextual variant ID exposed.
    pub variant_exposure: Option<String>,
    /// Category of problem variation.
    #[serde(default)]
    pub variant_category: VariantCategory,
    /// Distinct topological fingerprint of the solution graph (if available).
    #[serde(default)]
    pub solution_graph_fingerprint: Option<String>,
    /// Whether a cognitive decision point was present and correctly decided.
    #[serde(default)]
    pub cognitive_decision_correct: Option<bool>,
    /// Whether the attempt provided valid evidence of far-transfer capabilities.
    pub transfer_evidence: bool,
    /// Latency (ms) taken to solve.
    pub latency_evidence: u64,
    /// Time elapsed since the previous practice attempt on this skill in ms (for delayed retention).
    #[serde(default)]
    pub time_since_last_ms: Option<u64>,
    /// Whether domain-specific procedural competence was verified (e.g. regime/model/schema choice).
    #[serde(default)]
    pub domain_competence_verified: Option<bool>,
    /// List of diagnostic errors encountered during the attempt.
    pub diagnostic_errors: Vec<ErrorCategory>,
    /// Domain-specific diagnostic evidence extensions.
    #[serde(default)]
    pub domain_evidence: Option<crate::skills::domain_evidence::VersionedDomainEvidence>,
}

impl Default for MasteryEvidence {
    fn default() -> Self {
        Self {
            final_correctness: false,
            decision_quality: None,
            step_quality: None,
            independence: IndependenceLevel::Independent,
            max_hint_level: None,
            hint_dependence: 0,
            retry_dependence: 0,
            variant_exposure: None,
            variant_category: VariantCategory::Parameter,
            solution_graph_fingerprint: None,
            cognitive_decision_correct: None,
            transfer_evidence: false,
            latency_evidence: 0,
            time_since_last_ms: None,
            domain_competence_verified: None,
            diagnostic_errors: vec![],
            domain_evidence: None,
        }
    }
}
