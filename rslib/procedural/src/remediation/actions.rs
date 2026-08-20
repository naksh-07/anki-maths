// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::core::{AttemptId, Domain, SchemaId, SkillId};
use crate::diagnostics::ErrorCategory;
use crate::problems::steps::StepErrorType;

/// Taxonomic kind of remediation intervention.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RemediationActionKind {
    /// Targeted procedural problem variant (e.g. simpler numbers, lower distraction, directional sign).
    ProceduralVariant,
    /// Discrete conceptual decision check (e.g. regime selection, physical model identification).
    ConceptCheck,
    /// Decision-point drill testing first strategy selection without full arithmetic execution.
    StrategyDrill,
    /// Structural or diagrammatic representation drill (e.g. coordinate system, chemical species).
    RepresentationDrill,
    /// Canonical worked example with step-by-step rationale and misconception warnings.
    WorkedExample,
    /// Bridge to native Anki card / tag for declarative fact or formula retrieval.
    DeclarativeRecall,
    /// Advisory recommendation to review foundational prerequisite skills.
    PrerequisiteReview,
    /// Return to foundational structural level after a transfer failure before retrying transfer.
    TransferRetry,
    /// Controlled cooldown / deferral halting repetitive isomorphic wheel-spinning.
    CircuitBreaker,
}

impl RemediationActionKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            RemediationActionKind::ProceduralVariant => "procedural_variant",
            RemediationActionKind::ConceptCheck => "concept_check",
            RemediationActionKind::StrategyDrill => "strategy_drill",
            RemediationActionKind::RepresentationDrill => "representation_drill",
            RemediationActionKind::WorkedExample => "worked_example",
            RemediationActionKind::DeclarativeRecall => "declarative_recall",
            RemediationActionKind::PrerequisiteReview => "prerequisite_review",
            RemediationActionKind::TransferRetry => "transfer_retry",
            RemediationActionKind::CircuitBreaker => "circuit_breaker",
        }
    }
}

/// Urgency level of the remediation intervention.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RemediationUrgency {
    /// Non-blocking, optional reinforcement or review.
    Advisory = 1,
    /// Standard next-step remediation within normal practice interleaving.
    Normal = 2,
    /// High-priority intervention that should immediately intercept repeated or severe conceptual breakdowns.
    Critical = 3,
}

impl Default for RemediationUrgency {
    fn default() -> Self {
        Self::Normal
    }
}

/// Structured, typed learning remediation plan produced deterministically from diagnostic evidence.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RemediationAction {
    pub id: String,
    pub kind: RemediationActionKind,
    pub skill_id: SkillId,
    pub schema_id: SchemaId,
    pub domain: Domain,
    pub primary_error: ErrorCategory,
    pub step_error: Option<StepErrorType>,
    pub preferred_difficulty: u32,
    pub preferred_variant: Option<String>,
    pub source_attempt_id: AttemptId,
    pub urgency: RemediationUrgency,
    pub requires_acknowledgement: bool,
    pub recurrence_count: u32,
    pub rationale: String,
    pub created_at: i64,
}

impl RemediationAction {
    pub fn new(
        id: impl Into<String>,
        kind: RemediationActionKind,
        skill_id: impl Into<SkillId>,
        schema_id: impl Into<SchemaId>,
        domain: Domain,
        primary_error: ErrorCategory,
        source_attempt_id: impl Into<AttemptId>,
        rationale: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            kind,
            skill_id: skill_id.into(),
            schema_id: schema_id.into(),
            domain,
            primary_error,
            step_error: None,
            preferred_difficulty: 1,
            preferred_variant: None,
            source_attempt_id: source_attempt_id.into(),
            urgency: RemediationUrgency::Normal,
            requires_acknowledgement: false,
            recurrence_count: 1,
            rationale: rationale.into(),
            created_at: Utc::now().timestamp(),
        }
    }

    pub fn with_step_error(mut self, step_error: Option<StepErrorType>) -> Self {
        self.step_error = step_error;
        self
    }

    pub fn with_difficulty(mut self, difficulty: u32) -> Self {
        self.preferred_difficulty = difficulty;
        self
    }

    pub fn with_variant(mut self, variant: Option<String>) -> Self {
        self.preferred_variant = variant;
        self
    }

    pub fn with_urgency(mut self, urgency: RemediationUrgency) -> Self {
        self.urgency = urgency;
        self
    }

    pub fn with_acknowledgement(mut self, req: bool) -> Self {
        self.requires_acknowledgement = req;
        self
    }

    pub fn with_recurrence(mut self, count: u32) -> Self {
        self.recurrence_count = count;
        self
    }
}
