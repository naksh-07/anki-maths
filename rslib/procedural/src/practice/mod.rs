// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

pub mod request;

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::core::{AttemptId, ErrorEventId, ProblemFamilyId, ProblemInstanceId, SchemaId, SkillId};

pub use request::{
    DifficultyConstraint, PracticeObjective, PracticeRequest, PracticeScope, RemediationPrecedence,
    SessionBudget, TimeConstraint,
};

/// Procedural learning object schema referenced by Anki cards as practice targets.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SchemaPracticeObject {
    pub id: SchemaId,
    pub skill_id: SkillId,
    pub problem_family_id: ProblemFamilyId,
    pub title: String,
    pub description: String,
    pub target_mastery: f64,
    /// Domain or generator specific configuration parameters
    pub config: serde_json::Value,
    pub created_at: i64,
}

impl SchemaPracticeObject {
    pub fn new(
        id: impl Into<SchemaId>,
        skill_id: impl Into<SkillId>,
        problem_family_id: impl Into<ProblemFamilyId>,
        title: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            skill_id: skill_id.into(),
            problem_family_id: problem_family_id.into(),
            title: title.into(),
            description: description.into(),
            target_mastery: 0.85,
            config: serde_json::Value::Object(Default::default()),
            created_at: Utc::now().timestamp(),
        }
    }

    pub fn with_target_mastery(mut self, target: f64) -> Self {
        self.target_mastery = target;
        self
    }

    pub fn with_config(mut self, config: serde_json::Value) -> Self {
        self.config = config;
        self
    }
}

/// Record of a learner practicing a specific problem instance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PracticeAttempt {
    pub id: AttemptId,
    pub instance_id: ProblemInstanceId,
    pub schema_id: SchemaId,
    pub skill_id: SkillId,
    /// Associated Anki Card ID if this attempt originated from an Anki review
    pub card_id: Option<i64>,
    pub user_answer: serde_json::Value,
    pub is_correct: bool,
    pub score: f64,
    pub time_taken_ms: u64,
    pub attempted_at: i64,
    pub metadata: serde_json::Value,
}

impl PracticeAttempt {
    pub fn new(
        id: impl Into<AttemptId>,
        instance_id: impl Into<ProblemInstanceId>,
        schema_id: impl Into<SchemaId>,
        skill_id: impl Into<SkillId>,
        user_answer: serde_json::Value,
        is_correct: bool,
        score: f64,
        time_taken_ms: u64,
    ) -> Self {
        Self {
            id: id.into(),
            instance_id: instance_id.into(),
            schema_id: schema_id.into(),
            skill_id: skill_id.into(),
            card_id: None,
            user_answer,
            is_correct,
            score,
            time_taken_ms,
            attempted_at: Utc::now().timestamp(),
            metadata: serde_json::Value::Object(Default::default()),
        }
    }

    pub fn with_card_id(mut self, card_id: i64) -> Self {
        self.card_id = Some(card_id);
        self
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Diagnostic error event recorded during a practice attempt.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ErrorEvent {
    pub id: ErrorEventId,
    pub attempt_id: AttemptId,
    pub error_category: String,
    pub details: serde_json::Value,
    pub occurred_at: i64,
}

impl ErrorEvent {
    pub fn new(
        id: impl Into<ErrorEventId>,
        attempt_id: impl Into<AttemptId>,
        error_category: impl Into<String>,
        details: serde_json::Value,
    ) -> Self {
        Self {
            id: id.into(),
            attempt_id: attempt_id.into(),
            error_category: error_category.into(),
            details,
            occurred_at: Utc::now().timestamp(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_practice_attempt_and_error_event() {
        let attempt = PracticeAttempt::new(
            "att-1",
            "inst-1",
            "schema-1",
            "skill-1",
            serde_json::json!({ "answer": 15 }),
            false,
            0.0,
            4200,
        )
        .with_card_id(1600000000000);

        assert_eq!(attempt.card_id, Some(1600000000000));
        assert!(!attempt.is_correct);

        let error = ErrorEvent::new(
            "err-1",
            attempt.id.clone(),
            "sign_error",
            serde_json::json!({ "expected": -15, "given": 15 }),
        );

        assert_eq!(error.attempt_id, attempt.id);
        assert_eq!(error.error_category, "sign_error");
    }
}
