// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use serde::{Deserialize, Serialize};

use crate::core::{Domain, SchemaId, SkillId};
use crate::diagnostics::ErrorCategory;
use crate::remediation::RemediationAction;
use crate::skills::signals::PracticeProgressionState;
use crate::skills::SkillState;

/// Discrete levels of transfer challenge for procedural problem solving.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransferLevel {
    /// Surface parameter variations with identical core solution graphs.
    NearTransfer = 1,
    /// Meaningful algebraic/structural changes in step sequencing or sub-goal decomposition.
    StructuralTransfer = 2,
    /// Applying the foundational skill across an alternate physical or conceptual context.
    ContextTransfer = 3,
    /// Synthesis of 2+ distinct learned schemas in a single compound task.
    MultiConceptTransfer = 4,
    /// Rare, novel cross-domain problem formulation requiring deep abstraction.
    FarTransfer = 5,
}

impl Default for TransferLevel {
    fn default() -> Self {
        TransferLevel::NearTransfer
    }
}

impl TransferLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            TransferLevel::NearTransfer => "near_transfer",
            TransferLevel::StructuralTransfer => "structural_transfer",
            TransferLevel::ContextTransfer => "context_transfer",
            TransferLevel::MultiConceptTransfer => "multi_concept_transfer",
            TransferLevel::FarTransfer => "far_transfer",
        }
    }
}

/// Evaluation result determining if a skill is ready for a specific transfer level.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransferEligibilityEvaluation {
    pub target_level: TransferLevel,
    pub is_eligible: bool,
    pub max_eligible_level: Option<TransferLevel>,
    pub reasons: Vec<String>,
}

/// Transfer evaluation engine governing transfer readiness and remediation routing.
#[derive(Debug, Clone, Default)]
pub struct TransferEngine;

impl TransferEngine {
    /// Evaluates if a skill meets the rigorous pedagogical evidence for a transfer level.
    pub fn evaluate_eligibility(
        state: &SkillState,
        requested_level: TransferLevel,
        supporting_schemas_stable: bool,
    ) -> TransferEligibilityEvaluation {
        let mut reasons = Vec::new();
        let mut max_eligible = None;

        let recent_acc = state.recent_accuracy();
        let total_attempts = state.total_attempts;
        let variant_count = state.variant_stats.len();
        let has_recent_concept_errors = state.recent_attempts.iter().take(5).any(|a| {
            matches!(
                a.error_category,
                Some(ErrorCategory::Concept) | Some(ErrorCategory::Conceptual)
            )
        });

        // NearTransfer requires stable Fluent or higher
        if (state.practice_state >= PracticeProgressionState::Fluent || state.practice_state.is_mature())
            && recent_acc >= 0.75
            && total_attempts >= 3
            && !has_recent_concept_errors
        {
            max_eligible = Some(TransferLevel::NearTransfer);
        } else {
            reasons.push("Near transfer requires Fluent stage with >=75% accuracy and no recent conceptual breakdowns.".into());
        }

        // StructuralTransfer requires active Variation or Mastered with >= 2 distinct variants practiced
        if (state.practice_state >= PracticeProgressionState::Variation || state.practice_state.is_mature())
            && recent_acc >= 0.80
            && variant_count >= 2
            && state.consecutive_successes >= 2
            && !has_recent_concept_errors
        {
            max_eligible = Some(TransferLevel::StructuralTransfer);
        } else if requested_level >= TransferLevel::StructuralTransfer {
            reasons.push("Structural transfer requires Variation stage with >=2 variant exposures and stable recent accuracy.".into());
        }

        // ContextTransfer requires solid Structural Transfer success (e.g. stage Transfer or Mastered)
        if (state.practice_state >= PracticeProgressionState::Transfer || state.practice_state.is_mature())
            && recent_acc >= 0.85
            && variant_count >= 3
            && state.consecutive_successes >= 3
        {
            max_eligible = Some(TransferLevel::ContextTransfer);
        } else if requested_level >= TransferLevel::ContextTransfer {
            reasons.push("Context transfer requires established transfer performance across multiple variant families.".into());
        }

        // MultiConceptTransfer requires multiple schemas to be independently stable
        if max_eligible >= Some(TransferLevel::ContextTransfer) && supporting_schemas_stable {
            max_eligible = Some(TransferLevel::MultiConceptTransfer);
        } else if requested_level >= TransferLevel::MultiConceptTransfer && !supporting_schemas_stable {
            reasons.push("Multi-concept transfer requires supporting sub-schemas to be independently stable.".into());
        }

        // FarTransfer requires Mastered or mature state with flawless recent streak
        if state.practice_state.is_mature()
            && recent_acc >= 0.90
            && state.consecutive_successes >= 5
            && supporting_schemas_stable
        {
            max_eligible = Some(TransferLevel::FarTransfer);
        } else if requested_level == TransferLevel::FarTransfer {
            reasons.push("Far transfer requires verified mastery and sustained high performance.".into());
        }

        let is_eligible = match max_eligible {
            Some(max) => max >= requested_level,
            None => false,
        };

        TransferEligibilityEvaluation {
            target_level: requested_level,
            is_eligible,
            max_eligible_level: max_eligible,
            reasons,
        }
    }

    /// Classifies a transfer failure and returns the appropriate structured remediation action.
    pub fn classify_transfer_failure(
        skill_id: &SkillId,
        schema_id: &SchemaId,
        domain: Domain,
        transfer_level: TransferLevel,
        error_category: Option<ErrorCategory>,
        attempt_id: Option<crate::core::AttemptId>,
    ) -> RemediationAction {
        let err = error_category.unwrap_or(ErrorCategory::Unknown);
        let action_id = format!("rem-transfer-{}-{}", schema_id, attempt_id.as_ref().map_or("none", |a| a.as_str()));

        let (kind, rationale) = match err {
            // 1. Concept failure in transfer -> immediate ConceptCheck
            ErrorCategory::Concept | ErrorCategory::Conceptual => (
                crate::remediation::RemediationActionKind::ConceptCheck,
                "Transfer challenge encountered fundamental conceptual breakdown.".to_string(),
            ),
            // 2. Strategy failure in transfer -> StrategyDrill
            ErrorCategory::Strategy => (
                crate::remediation::RemediationActionKind::StrategyDrill,
                "Transfer challenge encountered strategic selection breakdown.".to_string(),
            ),
            // 3. Execution / Calculation slip -> Lower structural variant review
            ErrorCategory::Calculation | ErrorCategory::Careless | ErrorCategory::ProceduralSlip => (
                crate::remediation::RemediationActionKind::WorkedExample,
                "Transfer challenge encountered calculation slip; scaffolding intermediate steps.".to_string(),
            ),
            // 4. Time or other errors -> Strategy drill / scaffolding
            _ => {
                if transfer_level >= TransferLevel::ContextTransfer {
                    (
                        crate::remediation::RemediationActionKind::StrategyDrill,
                        "Complex contextual transfer failure; practicing strategy decomposition.".to_string(),
                    )
                } else {
                    (
                        crate::remediation::RemediationActionKind::WorkedExample,
                        "Transfer execution support with worked example.".to_string(),
                    )
                }
            }
        };

        RemediationAction::new(
            action_id,
            kind,
            skill_id.clone(),
            schema_id.clone(),
            domain,
            err,
            attempt_id.unwrap_or_else(|| crate::core::AttemptId::new("att-transfer-fail")),
            rationale,
        )
    }
}
