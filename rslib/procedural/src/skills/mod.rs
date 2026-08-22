// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

pub mod signals;
pub mod progression;
pub mod prerequisites;
pub mod lifecycle;
pub mod domain_evidence;

use std::collections::HashMap;
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::core::{Domain, SkillId};

pub use signals::{
    ErrorFrequencyCounts, MovingLatencyStats, PracticeProgressionState,
    RecentAttemptRecord, VariantPerformance, MasteryEvidence, IndependenceLevel, VariantCategory
};
pub use progression::ProgressionPolicy;
pub use prerequisites::{
    DEFAULT_MAX_PREREQUISITE_DEPTH, PrerequisiteEvaluation, PrerequisiteGraphService,
    PrerequisitePolicy, PrerequisiteReadiness,
};
pub use lifecycle::{MaintenanceReviewOutcome, RetirementEvaluation, RetirementPolicy};
pub use domain_evidence::{
    ChemistryEvidence, DomainEvidencePayload, MathEvidence, PhysicsEvidence, ReasoningEvidence,
    VersionedDomainEvidence,
};

/// Discrete skill node representing a specific concept or operational competency.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Skill {
    pub id: SkillId,
    pub domain: Domain,
    pub name: String,
    pub description: String,
    pub prerequisites: Vec<SkillId>,
    pub metadata: serde_json::Value,
    pub created_at: i64,
}

impl Skill {
    pub fn new(
        id: impl Into<SkillId>,
        domain: Domain,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            domain,
            name: name.into(),
            description: description.into(),
            prerequisites: Vec::new(),
            metadata: serde_json::Value::Object(Default::default()),
            created_at: Utc::now().timestamp(),
        }
    }

    pub fn with_prerequisites(mut self, prerequisites: Vec<SkillId>) -> Self {
        self.prerequisites = prerequisites;
        self
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Learner's progress, mastery state, and learning signals for a given skill.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkillState {
    pub skill_id: SkillId,
    /// Estimated mastery score (0.0 to 1.0)
    pub mastery: f64,
    /// Uncertainty/confidence in current mastery estimation (0.0 to 1.0)
    pub confidence: f64,
    pub total_attempts: u32,
    pub successful_attempts: u32,
    pub failed_attempts: u32,
    pub last_practiced_at: Option<i64>,
    pub last_success_at: Option<i64>,
    pub consecutive_successes: u32,
    pub consecutive_failures: u32,
    pub practice_state: PracticeProgressionState,
    pub window_size: usize,
    pub recent_attempts: Vec<RecentAttemptRecord>,
    pub latency_stats: MovingLatencyStats,
    pub error_counts: ErrorFrequencyCounts,
    pub variant_stats: HashMap<String, VariantPerformance>,
    /// Map of distinct structural/transfer forms successfully solved independently to pass count.
    #[serde(default)]
    pub structural_forms_seen: HashMap<String, u32>,
    /// Lifetime count of attempts where hints were requested.
    #[serde(default)]
    pub historical_hint_count: u32,
    /// Lifetime count of completely unassisted successful attempts.
    #[serde(default)]
    pub historical_independent_count: u32,
    /// Count of successful independent solves after meaningful delay separation (e.g. >= 12h).
    #[serde(default)]
    pub delayed_retention_successes: u32,
    /// Time separation of the most recent retention check in ms.
    #[serde(default)]
    pub last_retention_delay_ms: Option<u64>,
    /// Evidence-based progression level (0=Authentic, 1=Parameter, 2=Isomorphic, 3=Structural, 4=Contextual, 5=Transfer)
    #[serde(default)]
    pub variant_progression_level: u8,
    /// Extensible state for future knowledge tracing algorithms (BKT, DKT, Elo)
    pub custom_state: serde_json::Value,
    pub updated_at: i64,
}

impl SkillState {
    pub fn new(skill_id: impl Into<SkillId>) -> Self {
        Self {
            skill_id: skill_id.into(),
            mastery: 0.0,
            confidence: 0.0,
            total_attempts: 0,
            successful_attempts: 0,
            failed_attempts: 0,
            last_practiced_at: None,
            last_success_at: None,
            consecutive_successes: 0,
            consecutive_failures: 0,
            practice_state: PracticeProgressionState::New,
            window_size: 5,
            recent_attempts: Vec::new(),
            latency_stats: MovingLatencyStats::default(),
            error_counts: ErrorFrequencyCounts::default(),
            variant_stats: HashMap::new(),
            structural_forms_seen: HashMap::new(),
            historical_hint_count: 0,
            historical_independent_count: 0,
            delayed_retention_successes: 0,
            last_retention_delay_ms: None,
            variant_progression_level: 0,
            custom_state: serde_json::Value::Object(Default::default()),
            updated_at: Utc::now().timestamp(),
        }
    }

    pub fn with_window_size(mut self, size: usize) -> Self {
        self.window_size = size.max(1);
        self
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_attempts == 0 {
            0.0
        } else {
            self.successful_attempts as f64 / self.total_attempts as f64
        }
    }

    /// Accuracy computed exclusively over the configured sliding window of recent attempts.
    pub fn recent_accuracy(&self) -> f64 {
        if self.recent_attempts.is_empty() {
            0.0
        } else {
            let correct = self.recent_attempts.iter().filter(|a| a.is_correct).count();
            correct as f64 / self.recent_attempts.len() as f64
        }
    }

    /// Moving average latency across recent practice attempts in milliseconds.
    pub fn moving_average_latency_ms(&self) -> f64 {
        self.latency_stats.moving_average_ms
    }

    /// Number of distinct structural or transfer problem forms successfully passed independently.
    pub fn distinct_structural_forms_passed(&self) -> usize {
        self.structural_forms_seen.len()
    }

    /// Ratio of lifetime independent successful attempts vs total attempts (longitudinal independence).
    pub fn longitudinal_independence_ratio(&self) -> f64 {
        if self.total_attempts == 0 {
            1.0
        } else {
            self.historical_independent_count as f64 / self.total_attempts as f64
        }
    }

    /// Calculates a structural diversity score (0.0 to 1.0) based on distinct structural forms passed.
    pub fn structural_diversity_score(&self) -> f64 {
        (self.structural_forms_seen.len() as f64 / 3.0).min(1.0)
    }

    /// Whether the learner has demonstrated evidence of retained competence across time separation.
    pub fn has_delayed_retention_evidence(&self, min_delay_ms: u64) -> bool {
        self.delayed_retention_successes >= 1 
            || self.last_retention_delay_ms.map_or(false, |d| d >= min_delay_ms)
    }

    /// Record a practice attempt and update all moving windows, error counters, variant stats, and progression.
    pub fn record_attempt_outcome(
        &mut self,
        evidence: &MasteryEvidence,
        score: f64,
        target_latency_ms: u64,
        timestamp: i64,
    ) {
        let prev_practiced_at = self.last_practiced_at;
        self.total_attempts += 1;
        self.last_practiced_at = Some(timestamp);
        self.updated_at = Utc::now().timestamp();

        let is_correct = evidence.final_correctness;
        let latency_ms = evidence.latency_evidence;
        let error_category = evidence.diagnostic_errors.first();
        let variant = evidence.variant_exposure.as_deref();

        // Calculate time separation since previous attempt
        let time_since_last_ms = evidence.time_since_last_ms.or_else(|| {
            prev_practiced_at.map(|prev| (timestamp - prev).max(0) as u64 * 1000)
        });

        if is_correct {
            self.successful_attempts += 1;
            self.consecutive_successes += 1;
            self.consecutive_failures = 0;
            self.last_success_at = Some(timestamp);

            if evidence.independence == IndependenceLevel::Independent {
                self.historical_independent_count += 1;

                // Check delayed retention: >= 12 hours (43_200_000 ms) delay
                if let Some(delay) = time_since_last_ms {
                    if delay >= 43_200_000 {
                        self.delayed_retention_successes += 1;
                    }
                    self.last_retention_delay_ms = Some(delay);
                }

                // Record structural form exposure if structural, contextual, multi-concept, or transfer
                if evidence.variant_category.is_structural_or_transfer() {
                    let form_key = evidence
                        .solution_graph_fingerprint
                        .clone()
                        .or_else(|| evidence.variant_exposure.clone())
                        .unwrap_or_else(|| format!("{:?}", evidence.variant_category));
                    *self.structural_forms_seen.entry(form_key).or_insert(0) += 1;
                }
            }
        } else {
            self.failed_attempts += 1;
            self.consecutive_failures += 1;
            self.consecutive_successes = 0;
        }

        if evidence.hint_dependence > 0 || evidence.max_hint_level.map_or(false, |l| l > 0) {
            self.historical_hint_count += 1;
        }

        // Record error categories
        for cat in &evidence.diagnostic_errors {
            self.error_counts.record(cat);
        }

        // Record latency stats in sliding window
        self.latency_stats.update(latency_ms, self.window_size);

        // Record sliding recent attempt record
        self.recent_attempts.push(RecentAttemptRecord {
            is_correct,
            score,
            latency_ms,
            target_latency_ms,
            variant: variant.map(|s| s.to_string()),
            variant_category: Some(evidence.variant_category),
            error_category: error_category.cloned(),
            max_hint_level: evidence.max_hint_level,
            hint_count: Some(evidence.hint_dependence),
            independence: Some(evidence.independence),
            solution_graph_fingerprint: evidence.solution_graph_fingerprint.clone(),
            cognitive_decision_correct: evidence.cognitive_decision_correct,
            domain_evidence: evidence.domain_evidence.clone(),
            timestamp,
        });
        if self.recent_attempts.len() > self.window_size {
            self.recent_attempts.remove(0);
        }

        // Record variant performance
        if let Some(var_key) = variant {
            let perf = self
                .variant_stats
                .entry(var_key.to_string())
                .or_insert_with(VariantPerformance::default);
            perf.category = evidence.variant_category;
            if is_correct && evidence.independence == IndependenceLevel::Independent {
                perf.independent_successes += 1;
            }
            perf.record(is_correct, latency_ms, error_category.cloned(), timestamp);
        }

        // Smooth mastery estimation update
        let weight = 0.2;
        let outcome_val = if is_correct { 1.0 } else { 0.0 };
        self.mastery = (1.0 - weight) * self.mastery + weight * outcome_val;
        self.confidence = (self.total_attempts as f64 / 10.0).min(1.0);

        // Evaluate variant progression level based on evidence
        if is_correct && evidence.independence == IndependenceLevel::Independent {
            if self.consecutive_successes >= 1 && self.variant_progression_level < 5 {
                self.variant_progression_level += 1;
            }
        } else if !is_correct {
            // Demote on failure, especially conceptual
            if matches!(error_category, Some(crate::diagnostics::ErrorCategory::Concept) | Some(crate::diagnostics::ErrorCategory::Conceptual)) {
                self.variant_progression_level = 0; // Drop to authentic
            } else if self.variant_progression_level > 0 {
                self.variant_progression_level -= 1;
            }
        }

        // Evaluate state progression
        ProgressionPolicy::evaluate(self, evidence);

        // Sync custom_state for serialization compatibility
        self.sync_custom_state();
    }

    /// Directly record variant performance exposure.
    pub fn record_variant_exposure(
        &mut self,
        variant_key: impl Into<String>,
        is_correct: bool,
        latency_ms: u64,
        error_category: Option<crate::diagnostics::ErrorCategory>,
        timestamp: i64,
    ) {
        let perf = self
            .variant_stats
            .entry(variant_key.into())
            .or_insert_with(VariantPerformance::default);
        perf.record(is_correct, latency_ms, error_category, timestamp);
    }

    /// Synchronize rich structured signals into `custom_state` JSON for database persistence.
    pub fn sync_custom_state(&mut self) {
        let mut obj = match self.custom_state.as_object() {
            Some(map) => map.clone(),
            None => serde_json::Map::new(),
        };

        if let Some(last_lat) = self.latency_stats.recent_latencies.last() {
            obj.insert("recent_latency_ms".to_string(), serde_json::json!(last_lat));
        }
        obj.insert("recent_accuracy".to_string(), serde_json::json!(self.recent_accuracy()));
        obj.insert("failed_attempts".to_string(), serde_json::json!(self.failed_attempts));
        obj.insert("last_success_at".to_string(), serde_json::json!(self.last_success_at));
        obj.insert("consecutive_successes".to_string(), serde_json::json!(self.consecutive_successes));
        obj.insert("consecutive_failures".to_string(), serde_json::json!(self.consecutive_failures));
        obj.insert("practice_state".to_string(), serde_json::json!(self.practice_state));
        obj.insert("window_size".to_string(), serde_json::json!(self.window_size));
        obj.insert("recent_attempts".to_string(), serde_json::json!(self.recent_attempts));
        obj.insert("latency_stats".to_string(), serde_json::json!(self.latency_stats));
        obj.insert("error_counts".to_string(), serde_json::json!(self.error_counts));
        obj.insert("variant_stats".to_string(), serde_json::json!(self.variant_stats));
        obj.insert("structural_forms_seen".to_string(), serde_json::json!(self.structural_forms_seen));
        obj.insert("historical_hint_count".to_string(), serde_json::json!(self.historical_hint_count));
        obj.insert("historical_independent_count".to_string(), serde_json::json!(self.historical_independent_count));
        obj.insert("delayed_retention_successes".to_string(), serde_json::json!(self.delayed_retention_successes));
        obj.insert("last_retention_delay_ms".to_string(), serde_json::json!(self.last_retention_delay_ms));

        self.custom_state = serde_json::Value::Object(obj);
    }

    /// Restore rich structured signals from `custom_state` JSON if present.
    pub fn restore_from_custom_state(&mut self) {
        if let Some(obj) = self.custom_state.as_object() {
            if let Some(val) = obj.get("failed_attempts").and_then(|v| v.as_u64()) {
                self.failed_attempts = val as u32;
            }
            if let Some(val) = obj.get("last_success_at").and_then(|v| v.as_i64()) {
                self.last_success_at = Some(val);
            }
            if let Some(val) = obj.get("consecutive_successes").and_then(|v| v.as_u64()) {
                self.consecutive_successes = val as u32;
            }
            if let Some(val) = obj.get("consecutive_failures").and_then(|v| v.as_u64()) {
                self.consecutive_failures = val as u32;
            }
            if let Some(val) = obj.get("practice_state") {
                if let Ok(state) = serde_json::from_value::<PracticeProgressionState>(val.clone()) {
                    self.practice_state = state;
                }
            }
            if let Some(val) = obj.get("window_size").and_then(|v| v.as_u64()) {
                self.window_size = val as usize;
            }
            if let Some(val) = obj.get("recent_attempts") {
                if let Ok(attempts) = serde_json::from_value::<Vec<RecentAttemptRecord>>(val.clone()) {
                    self.recent_attempts = attempts;
                }
            }
            if let Some(val) = obj.get("latency_stats") {
                if let Ok(lat) = serde_json::from_value::<MovingLatencyStats>(val.clone()) {
                    self.latency_stats = lat;
                }
            }
            if let Some(val) = obj.get("error_counts") {
                if let Ok(errs) = serde_json::from_value::<ErrorFrequencyCounts>(val.clone()) {
                    self.error_counts = errs;
                }
            }
            if let Some(val) = obj.get("variant_stats") {
                if let Ok(vars) = serde_json::from_value::<HashMap<String, VariantPerformance>>(val.clone()) {
                    self.variant_stats = vars;
                }
            }
            if let Some(val) = obj.get("structural_forms_seen") {
                if let Ok(forms) = serde_json::from_value::<HashMap<String, u32>>(val.clone()) {
                    self.structural_forms_seen = forms;
                }
            }
            if let Some(val) = obj.get("historical_hint_count").and_then(|v| v.as_u64()) {
                self.historical_hint_count = val as u32;
            }
            if let Some(val) = obj.get("historical_independent_count").and_then(|v| v.as_u64()) {
                self.historical_independent_count = val as u32;
            }
            if let Some(val) = obj.get("delayed_retention_successes").and_then(|v| v.as_u64()) {
                self.delayed_retention_successes = val as u32;
            }
            if let Some(val) = obj.get("last_retention_delay_ms").and_then(|v| v.as_u64()) {
                self.last_retention_delay_ms = Some(val);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::ErrorCategory;

    #[test]
    fn test_skill_creation() {
        let skill = Skill::new(
            "math.calc.derivatives",
            Domain::Mathematics,
            "Power Rule",
            "Applying the power rule for derivatives",
        )
        .with_prerequisites(vec![SkillId::from("math.calc.limits")]);

        assert_eq!(skill.id.as_str(), "math.calc.derivatives");
        assert_eq!(skill.domain, Domain::Mathematics);
        assert_eq!(skill.prerequisites.len(), 1);
        assert_eq!(skill.prerequisites[0].as_str(), "math.calc.limits");
    }

    #[test]
    fn test_skill_state_success_rate() {
        let mut state = SkillState::new("chem.stoichiometry.moles");
        assert_eq!(state.success_rate(), 0.0);

        state.total_attempts = 10;
        state.successful_attempts = 7;
        assert!((state.success_rate() - 0.7).abs() < f64::EPSILON);
    }

    #[test]
    fn test_moving_window_and_signals_recording() {
        let mut state = SkillState::new("math.percentage.successive").with_window_size(3);
        assert_eq!(state.practice_state, PracticeProgressionState::New);

        // 1st attempt: correct
        let ev1 = MasteryEvidence {
            final_correctness: true,
            latency_evidence: 20_000,
            variant_exposure: Some("forward_two_step".to_string()),
            independence: IndependenceLevel::Independent,
            ..Default::default()
        };
        state.record_attempt_outcome(&ev1, 1.0, 30_000, 1000);
        assert_eq!(state.total_attempts, 1);
        assert_eq!(state.successful_attempts, 1);
        assert_eq!(state.consecutive_successes, 1);
        assert_eq!(state.practice_state, PracticeProgressionState::Learning);
        assert_eq!(state.recent_accuracy(), 1.0);
        assert_eq!(state.moving_average_latency_ms(), 20_000.0);

        // 2nd attempt: correct
        let ev2 = MasteryEvidence {
            final_correctness: true,
            latency_evidence: 25_000,
            variant_exposure: Some("forward_two_step".to_string()),
            independence: IndependenceLevel::Independent,
            ..Default::default()
        };
        state.record_attempt_outcome(&ev2, 1.0, 30_000, 1050);

        // 3rd attempt: correct
        let ev3 = MasteryEvidence {
            final_correctness: true,
            latency_evidence: 15_000,
            variant_exposure: Some("forward_two_step".to_string()),
            independence: IndependenceLevel::Independent,
            ..Default::default()
        };
        state.record_attempt_outcome(&ev3, 1.0, 30_000, 1100);
        assert_eq!(state.practice_state, PracticeProgressionState::Fluent);
        assert_eq!(state.recent_accuracy(), 1.0);
        assert_eq!(state.moving_average_latency_ms(), 20_000.0);
        assert_eq!(state.latency_stats.min_latency_ms, Some(15_000));
        assert_eq!(state.latency_stats.max_latency_ms, Some(25_000));

        // 4th attempt: fail with concept error
        let ev4 = MasteryEvidence {
            final_correctness: false,
            latency_evidence: 40_000,
            variant_exposure: Some("forward_two_step".to_string()),
            diagnostic_errors: vec![ErrorCategory::Concept],
            ..Default::default()
        };
        state.record_attempt_outcome(
            &ev4,
            0.0,
            30_000,
            1150,
        );
        // Window size is 3, so window contains: 25s (pass), 15s (pass), 40s (fail) -> 2/3 accuracy
        assert_eq!(state.recent_attempts.len(), 3);
        assert!((state.recent_accuracy() - (2.0 / 3.0)).abs() < 1e-6);
        assert_eq!(state.consecutive_failures, 1);
        assert_eq!(state.consecutive_successes, 0);
        assert_eq!(state.error_counts.get_count(&ErrorCategory::Concept), 1);
    }
}
