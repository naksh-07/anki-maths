// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::diagnostics::ErrorCategory;

/// Deterministic stage of procedural skill development.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PracticeProgressionState {
    /// Newly introduced skill without sufficient practice history.
    New,
    /// Building initial procedural knowledge and basic calculation steps.
    Learning,
    /// High accuracy and consistent execution on standard problem variants.
    Fluent,
    /// Practicing diverse structural and contextual problem variants.
    Variation,
    /// Robust performance across non-standard and multi-step variations.
    Transfer,
    /// High-speed, high-accuracy automaticity across all variant templates.
    Mastered,
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
        }
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
    pub error_category: Option<ErrorCategory>,
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
