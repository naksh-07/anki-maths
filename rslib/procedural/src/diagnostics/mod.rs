// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

pub mod hints;

use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::core::{AttemptId, ProblemFamilyId, SchemaId, SkillId};
use crate::practice::{ErrorEvent, PracticeAttempt};
use crate::skills::signals::IndependenceLevel;

pub use hints::{HintDependencyStats, HintLevel, HintUsageRecord};

/// Common taxonomic categories for procedural practice errors.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCategory {
    Concept,
    Strategy,
    Calculation,
    Careless,
    Time,
    Unknown,
    // Aliases and backwards compatibility:
    Conceptual,
    Sign,
    Syntax,
    Unit,
    ProceduralSlip,
    Timeout,
    #[serde(untagged)]
    DomainSpecific(String),
}

impl ErrorCategory {
    pub fn as_str(&self) -> &str {
        match self {
            ErrorCategory::Concept => "concept",
            ErrorCategory::Strategy => "strategy",
            ErrorCategory::Calculation => "calculation",
            ErrorCategory::Careless => "careless",
            ErrorCategory::Time => "time",
            ErrorCategory::Unknown => "unknown",
            ErrorCategory::Conceptual => "conceptual",
            ErrorCategory::Sign => "sign",
            ErrorCategory::Syntax => "syntax",
            ErrorCategory::Unit => "unit",
            ErrorCategory::ProceduralSlip => "procedural_slip",
            ErrorCategory::Timeout => "timeout",
            ErrorCategory::DomainSpecific(s) => s.as_str(),
        }
    }
}

impl fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Structured outcome object containing performance signals from a procedural practice attempt.
/// Serves as the clean data carrier for the future FSRS calibration bridge.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProceduralReviewOutcome {
    pub attempt_id: AttemptId,
    pub schema_id: SchemaId,
    pub skill_id: SkillId,
    pub family_id: ProblemFamilyId,
    pub seed: u64,
    pub is_correct: bool,
    pub score: f64,
    pub latency_ms: u64,
    pub target_latency_ms: u64,
    pub hints_used: u32,
    pub attempt_count: u32,
    pub error_category: Option<ErrorCategory>,
    pub diagnostic_message: Option<String>,
    pub timestamp: i64,
    #[serde(default)]
    pub first_error_step: Option<usize>,
    #[serde(default)]
    pub steps_completed: usize,
    #[serde(default)]
    pub steps_correct: usize,
    #[serde(default)]
    pub step_latencies_ms: Vec<u64>,
    #[serde(default)]
    pub first_action_latency_ms: Option<u64>,
    #[serde(default)]
    pub diagnostic_confidence: Option<String>,
    #[serde(default)]
    pub remediation_recommendation: Option<String>,
    #[serde(default)]
    pub decision_points_presented: usize,
    #[serde(default)]
    pub decision_points_correct: usize,
    #[serde(default)]
    pub independence_level: IndependenceLevel,
}

impl ProceduralReviewOutcome {
    pub fn new(
        attempt_id: impl Into<AttemptId>,
        schema_id: impl Into<SchemaId>,
        skill_id: impl Into<SkillId>,
        family_id: impl Into<ProblemFamilyId>,
        seed: u64,
        is_correct: bool,
        score: f64,
        latency_ms: u64,
        target_latency_ms: u64,
        hints_used: u32,
        attempt_count: u32,
        error_category: Option<ErrorCategory>,
    ) -> Self {
        Self {
            attempt_id: attempt_id.into(),
            schema_id: schema_id.into(),
            skill_id: skill_id.into(),
            family_id: family_id.into(),
            seed,
            is_correct,
            score,
            latency_ms,
            target_latency_ms,
            hints_used,
            attempt_count,
            error_category,
            diagnostic_message: None,
            timestamp: chrono::Utc::now().timestamp(),
            first_error_step: None,
            steps_completed: 0,
            steps_correct: 0,
            step_latencies_ms: Vec::new(),
            first_action_latency_ms: None,
            diagnostic_confidence: None,
            remediation_recommendation: None,
            decision_points_presented: 0,
            decision_points_correct: 0,
            independence_level: IndependenceLevel::default(),
        }
    }

    pub fn with_step_diagnostics(
        mut self,
        first_error_step: Option<usize>,
        steps_completed: usize,
        steps_correct: usize,
        step_latencies: Vec<u64>,
        first_action_latency: Option<u64>,
        confidence: Option<String>,
        remediation: Option<String>,
    ) -> Self {
        self.first_error_step = first_error_step;
        self.steps_completed = steps_completed;
        self.steps_correct = steps_correct;
        self.step_latencies_ms = step_latencies;
        self.first_action_latency_ms = first_action_latency;
        self.diagnostic_confidence = confidence;
        self.remediation_recommendation = remediation;
        self
    }
}

/// Aggregate diagnostic statistics across practice attempts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttemptDiagnosticSummary {
    pub total_attempts: usize,
    pub correct_attempts: usize,
    pub accuracy: f64,
    pub average_time_ms: f64,
    pub error_breakdown: HashMap<String, usize>,
}

impl AttemptDiagnosticSummary {
    pub fn compute(attempts: &[PracticeAttempt], errors: &[ErrorEvent]) -> Self {
        let total_attempts = attempts.len();
        let correct_attempts = attempts.iter().filter(|a| a.is_correct).count();
        let accuracy = if total_attempts > 0 {
            correct_attempts as f64 / total_attempts as f64
        } else {
            0.0
        };

        let total_time: u64 = attempts.iter().map(|a| a.time_taken_ms).sum();
        let average_time_ms = if total_attempts > 0 {
            total_time as f64 / total_attempts as f64
        } else {
            0.0
        };

        let mut error_breakdown = HashMap::new();
        for err in errors {
            *error_breakdown.entry(err.error_category.clone()).or_insert(0) += 1;
        }

        Self {
            total_attempts,
            correct_attempts,
            accuracy,
            average_time_ms,
            error_breakdown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::AttemptId;

    #[test]
    fn test_diagnostic_summary_computation() {
        let attempts = vec![
            PracticeAttempt::new(
                "a1",
                "i1",
                "s1",
                "sk1",
                serde_json::json!(4),
                true,
                1.0,
                2000,
            ),
            PracticeAttempt::new(
                "a2",
                "i2",
                "s1",
                "sk1",
                serde_json::json!(5),
                false,
                0.0,
                4000,
            ),
        ];

        let errors = vec![ErrorEvent::new(
            "e1",
            AttemptId::from("a2"),
            ErrorCategory::Calculation.to_string(),
            serde_json::json!({}),
        )];

        let summary = AttemptDiagnosticSummary::compute(&attempts, &errors);
        assert_eq!(summary.total_attempts, 2);
        assert_eq!(summary.correct_attempts, 1);
        assert!((summary.accuracy - 0.5).abs() < f64::EPSILON);
        assert_eq!(summary.average_time_ms, 3000.0);
        assert_eq!(summary.error_breakdown.get("calculation"), Some(&1));
    }
}
