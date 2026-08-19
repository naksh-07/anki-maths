// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::core::{AttemptId, SchemaId, SkillId};
use crate::diagnostics::ErrorCategory;
use crate::problems::steps::StepErrorType;
use crate::remediation::actions::RemediationActionKind;
use crate::skills::signals::MasteryEvidence;

/// Final resolution outcome status of a remediation intervention.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RemediationOutcomeStatus {
    /// Learner demonstrated intended capability (concept, strategy, or procedural execution).
    Resolved,
    /// Learner failed intervention, triggering escalation to higher pedagogical tier.
    Escalated,
    /// Remediation presented or waiting for learner response.
    Pending,
    /// Remediation superseded or deferred by subsequent practice.
    Deferred,
}

/// Traceable audit log entry recording the lifecycle of a remediation intervention.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RemediationAuditRecord {
    pub id: String,
    pub source_attempt_id: AttemptId,
    pub skill_id: SkillId,
    pub schema_id: SchemaId,
    pub error_category: ErrorCategory,
    pub step_error: Option<StepErrorType>,
    pub action_kind: RemediationActionKind,
    pub intervention_type: String,
    pub learner_result: Option<bool>,
    pub follow_up_evidence: Option<MasteryEvidence>,
    pub outcome_status: RemediationOutcomeStatus,
    pub recurrence_count: u32,
    pub timestamp: i64,
}

impl RemediationAuditRecord {
    pub fn new(
        id: impl Into<String>,
        source_attempt_id: impl Into<AttemptId>,
        skill_id: impl Into<SkillId>,
        schema_id: impl Into<SchemaId>,
        error_category: ErrorCategory,
        action_kind: RemediationActionKind,
        intervention_type: impl Into<String>,
        recurrence_count: u32,
    ) -> Self {
        Self {
            id: id.into(),
            source_attempt_id: source_attempt_id.into(),
            skill_id: skill_id.into(),
            schema_id: schema_id.into(),
            error_category,
            step_error: None,
            action_kind,
            intervention_type: intervention_type.into(),
            learner_result: None,
            follow_up_evidence: None,
            outcome_status: RemediationOutcomeStatus::Pending,
            recurrence_count,
            timestamp: Utc::now().timestamp(),
        }
    }

    pub fn with_step_error(mut self, step_err: Option<StepErrorType>) -> Self {
        self.step_error = step_err;
        self
    }

    pub fn mark_completed(
        &mut self,
        is_correct: bool,
        evidence: MasteryEvidence,
        status: RemediationOutcomeStatus,
    ) {
        self.learner_result = Some(is_correct);
        self.follow_up_evidence = Some(evidence);
        self.outcome_status = status;
    }
}

/// In-memory and inspectable audit log tracking remediation history across sessions.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RemediationAuditLog {
    pub records: Vec<RemediationAuditRecord>,
}

impl RemediationAuditLog {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    pub fn record_event(&mut self, record: RemediationAuditRecord) {
        self.records.push(record);
    }

    pub fn get_record_mut(&mut self, id: &str) -> Option<&mut RemediationAuditRecord> {
        self.records.iter_mut().find(|r| r.id == id)
    }

    pub fn list_for_skill(&self, skill_id: &SkillId) -> Vec<&RemediationAuditRecord> {
        self.records.iter().filter(|r| &r.skill_id == skill_id).collect()
    }

    pub fn recent_records(&self, limit: usize) -> Vec<&RemediationAuditRecord> {
        self.records.iter().rev().take(limit).collect()
    }
}
