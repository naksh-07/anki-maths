// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::core::{PyqId, SchemaId};
use crate::diagnostics::ErrorCategory;

/// Performance summary aggregated for a specific source PYQ across historical attempts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PyqSourcePerformance {
    pub pyq_id: PyqId,
    pub exam: String,
    pub year: u32,
    pub total_attempts: usize,
    pub successful_attempts: usize,
    pub accuracy: f64,
    pub mean_latency_ms: f64,
    pub error_breakdown: HashMap<String, usize>,
    pub last_attempted_at: Option<i64>,
}

/// Diagnostic summary of schemas exhibiting high failure rates under an ExamProfile.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExamFailingSchemaSummary {
    pub schema_id: SchemaId,
    pub total_attempts: usize,
    pub failure_rate: f64,
    pub mean_latency_ms: f64,
    pub dominant_error: Option<ErrorCategory>,
    pub recommendation: String,
}

/// Analytics processor computing source-level metrics and exam weakness clusters.
pub struct PyqAnalyticsEngine;

impl PyqAnalyticsEngine {
    pub fn compute_performance(
        pyq_id: PyqId,
        exam: String,
        year: u32,
        attempts: &[(bool, u64, Option<String>, i64)],
    ) -> PyqSourcePerformance {
        let total = attempts.len();
        if total == 0 {
            return PyqSourcePerformance {
                pyq_id,
                exam,
                year,
                total_attempts: 0,
                successful_attempts: 0,
                accuracy: 0.0,
                mean_latency_ms: 0.0,
                error_breakdown: HashMap::new(),
                last_attempted_at: None,
            };
        }

        let successes = attempts.iter().filter(|(c, _, _, _)| *c).count();
        let total_time: u64 = attempts.iter().map(|(_, t, _, _)| *t).sum();
        let mean_latency = total_time as f64 / total as f64;
        let accuracy = successes as f64 / total as f64;

        let mut errors = HashMap::new();
        let mut last_time = None;

        for (is_correct, _, err_opt, time) in attempts {
            if !is_correct {
                let err_str = err_opt.clone().unwrap_or_else(|| "unknown".to_string());
                *errors.entry(err_str).or_insert(0) += 1;
            }
            if last_time.map_or(true, |t| *time > t) {
                last_time = Some(*time);
            }
        }

        PyqSourcePerformance {
            pyq_id,
            exam,
            year,
            total_attempts: total,
            successful_attempts: successes,
            accuracy,
            mean_latency_ms: mean_latency,
            error_breakdown: errors,
            last_attempted_at: last_time,
        }
    }
}
