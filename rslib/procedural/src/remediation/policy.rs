// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use crate::core::{AttemptId, Domain, SchemaId, SkillId};
use crate::diagnostics::ErrorCategory;
use crate::problems::steps::StepErrorType;
use crate::remediation::actions::{RemediationAction, RemediationActionKind, RemediationUrgency};
use crate::skills::signals::{IndependenceLevel, PracticeProgressionState, RecentAttemptRecord};

/// Context provided to the RemediationPolicy to evaluate the optimal next remediation action.
#[derive(Debug, Clone)]
pub struct RemediationContext<'a> {
    pub skill_id: &'a SkillId,
    pub schema_id: &'a SchemaId,
    pub domain: Domain,
    pub primary_error: ErrorCategory,
    pub step_error: Option<StepErrorType>,
    pub decision_point_correct: Option<bool>,
    pub independence: IndependenceLevel,
    pub progression_state: PracticeProgressionState,
    pub recent_attempts: &'a [RecentAttemptRecord],
    pub source_attempt_id: &'a AttemptId,
    pub recurrence_count: u32,
    pub is_transfer_attempt: bool,
}

/// Centralized, deterministic learning remediation policy.
pub struct RemediationPolicy;

impl RemediationPolicy {
    /// Evaluate a diagnostic failure context and return a typed, structured RemediationAction.
    pub fn evaluate(ctx: &RemediationContext) -> RemediationAction {
        let action_id = format!("rem-{}-{}", ctx.skill_id, ctx.source_attempt_id);
        let rec = ctx.recurrence_count;

        // 1. ESCALATION FOR PERSISTENT REPEATED FAILURES (Recurrence >= 4)
        if rec >= 4 {
            let action = RemediationAction::new(
                action_id,
                RemediationActionKind::PrerequisiteReview,
                ctx.skill_id,
                ctx.schema_id,
                ctx.domain.clone(),
                ctx.primary_error.clone(),
                ctx.source_attempt_id,
                format!("Persistent failure recurrence ({}) detected. Advisory prerequisite review recommended.", rec),
            )
            .with_recurrence(rec)
            .with_urgency(RemediationUrgency::Critical)
            .with_step_error(ctx.step_error);

            return action;
        }

        // 2. ESCALATION FOR REPEATED CONCEPTUAL/STRATEGY FAILURES (Recurrence == 3)
        if rec == 3 {
            let action = RemediationAction::new(
                action_id,
                RemediationActionKind::WorkedExample,
                ctx.skill_id,
                ctx.schema_id,
                ctx.domain.clone(),
                ctx.primary_error.clone(),
                ctx.source_attempt_id,
                "Repeated misconception or strategy error. Escalating to canonical worked example.",
            )
            .with_recurrence(rec)
            .with_urgency(RemediationUrgency::Critical)
            .with_acknowledgement(true)
            .with_step_error(ctx.step_error);

            return action;
        }

        // 3. TRANSFER FAILURE HANDLING
        if ctx.is_transfer_attempt {
            let action = RemediationAction::new(
                action_id,
                RemediationActionKind::TransferRetry,
                ctx.skill_id,
                ctx.schema_id,
                ctx.domain.clone(),
                ctx.primary_error.clone(),
                ctx.source_attempt_id,
                "Transfer attempt failed. Returning to earlier standard structural level before next transfer retry.",
            )
            .with_difficulty(2)
            .with_recurrence(rec)
            .with_urgency(RemediationUrgency::Normal)
            .with_step_error(ctx.step_error);

            return action;
        }

        // 4. STEP-LEVEL ERROR TYPE SPECIFIC MAPPINGS
        if let Some(step_err) = ctx.step_error {
            match step_err {
                // Concept / Schema Recognition
                StepErrorType::FormulaSelectionError
                | StepErrorType::SchemaRecognitionError
                | StepErrorType::RegimeSelectionError
                | StepErrorType::ModelSelectionError
                | StepErrorType::EquationSetupError
                | StepErrorType::PhysicalPlausibilityError
                | StepErrorType::ConservationViolationError
                | StepErrorType::InferenceError
                | StepErrorType::RatioInversionError
                | StepErrorType::RateInversionError
                | StepErrorType::IdentityCrossTermError
                | StepErrorType::PythagoreanLegConfusion => {
                    let kind = if rec == 2 {
                        RemediationActionKind::StrategyDrill
                    } else {
                        RemediationActionKind::ConceptCheck
                    };
                    return RemediationAction::new(
                        action_id,
                        kind,
                        ctx.skill_id,
                        ctx.schema_id,
                        ctx.domain.clone(),
                        ctx.primary_error.clone(),
                        ctx.source_attempt_id,
                        format!("Conceptual/schema recognition breakdown ({:?}). Triggering targeted concept evaluation.", step_err),
                    )
                    .with_step_error(Some(step_err))
                    .with_recurrence(rec)
                    .with_urgency(if rec > 1 { RemediationUrgency::Critical } else { RemediationUrgency::Normal });
                }

                // Strategy Selection & Structural Setup
                StepErrorType::StrategySelectionError
                | StepErrorType::SetupError
                | StepErrorType::TransformationError
                | StepErrorType::EquationBalanceError
                | StepErrorType::StoichiometricRatioError
                | StepErrorType::LimitingReagentError
                | StepErrorType::AlligationSwapError
                | StepErrorType::ConstraintApplicationError
                | StepErrorType::SearchCaseError
                | StepErrorType::ContradictionHandlingError => {
                    return RemediationAction::new(
                        action_id,
                        RemediationActionKind::StrategyDrill,
                        ctx.skill_id,
                        ctx.schema_id,
                        ctx.domain.clone(),
                        ctx.primary_error.clone(),
                        ctx.source_attempt_id,
                        format!("Strategy selection failure ({:?}). Isolating approach choice via strategy drill.", step_err),
                    )
                    .with_step_error(Some(step_err))
                    .with_recurrence(rec)
                    .with_urgency(if rec > 1 { RemediationUrgency::Critical } else { RemediationUrgency::Normal });
                }

                // Representation
                StepErrorType::RepresentationError
                | StepErrorType::ChemicalRepresentationError => {
                    return RemediationAction::new(
                        action_id,
                        RemediationActionKind::RepresentationDrill,
                        ctx.skill_id,
                        ctx.schema_id,
                        ctx.domain.clone(),
                        ctx.primary_error.clone(),
                        ctx.source_attempt_id,
                        format!("Representation breakdown ({:?}). Triggering structured notation/diagram check.", step_err),
                    )
                    .with_step_error(Some(step_err))
                    .with_recurrence(rec);
                }

                // Sign / Direction
                StepErrorType::SignError
                | StepErrorType::SignConventionError
                | StepErrorType::InequalitySignFlipError => {
                    return RemediationAction::new(
                        action_id,
                        RemediationActionKind::ProceduralVariant,
                        ctx.skill_id,
                        ctx.schema_id,
                        ctx.domain.clone(),
                        ctx.primary_error.clone(),
                        ctx.source_attempt_id,
                        "Sign/directional error. Providing sign-focused structural variant.",
                    )
                    .with_step_error(Some(step_err))
                    .with_variant(Some("directional_sign".to_string()))
                    .with_difficulty(1)
                    .with_recurrence(rec);
                }

                // Units
                StepErrorType::UnitError => {
                    let kind = if rec > 1 {
                        RemediationActionKind::DeclarativeRecall
                    } else {
                        RemediationActionKind::ProceduralVariant
                    };
                    return RemediationAction::new(
                        action_id,
                        kind,
                        ctx.skill_id,
                        ctx.schema_id,
                        ctx.domain.clone(),
                        ctx.primary_error.clone(),
                        ctx.source_attempt_id,
                        "Unit conversion error. Practicing dimensional scaling.",
                    )
                    .with_step_error(Some(step_err))
                    .with_variant(Some("unit_conversion".to_string()))
                    .with_difficulty(1)
                    .with_recurrence(rec);
                }

                // Calculation / Algebra Execution
                StepErrorType::ArithmeticError
                | StepErrorType::AlgebraExecutionError
                | StepErrorType::ModularReductionError
                | StepErrorType::ExecutionSlipError => {
                    return RemediationAction::new(
                        action_id,
                        RemediationActionKind::ProceduralVariant,
                        ctx.skill_id,
                        ctx.schema_id,
                        ctx.domain.clone(),
                        ctx.primary_error.clone(),
                        ctx.source_attempt_id,
                        "Operational execution slip. Practicing same schema with simpler numbers / reduced distraction.",
                    )
                    .with_step_error(Some(step_err))
                    .with_variant(Some("simpler_numbers".to_string()))
                    .with_difficulty(1)
                    .with_recurrence(rec);
                }

                // Careless / Reading
                StepErrorType::PrematureCompletion
                | StepErrorType::FinalAnswerFormattingError
                | StepErrorType::ReadingTrapError => {
                    return RemediationAction::new(
                        action_id,
                        RemediationActionKind::ProceduralVariant,
                        ctx.skill_id,
                        ctx.schema_id,
                        ctx.domain.clone(),
                        ctx.primary_error.clone(),
                        ctx.source_attempt_id,
                        "Careless reading or premature stopping. Providing guided step-by-step variant.",
                    )
                    .with_step_error(Some(step_err))
                    .with_variant(Some("guided_steps".to_string()))
                    .with_difficulty(1)
                    .with_recurrence(rec);
                }

                StepErrorType::Unknown => {}
            }
        }

        // 5. TAXONOMIC ERROR CATEGORY MAPPINGS
        match ctx.primary_error {
            ErrorCategory::Concept | ErrorCategory::Conceptual => {
                let kind = if rec == 2 {
                    RemediationActionKind::StrategyDrill
                } else {
                    RemediationActionKind::ConceptCheck
                };
                RemediationAction::new(
                    action_id,
                    kind,
                    ctx.skill_id,
                    ctx.schema_id,
                    ctx.domain.clone(),
                    ctx.primary_error.clone(),
                    ctx.source_attempt_id,
                    "Conceptual breakdown. Triggering concept check to diagnose misconception.",
                )
                .with_recurrence(rec)
                .with_urgency(if rec > 1 { RemediationUrgency::Critical } else { RemediationUrgency::Normal })
            }

            ErrorCategory::Strategy => RemediationAction::new(
                action_id,
                RemediationActionKind::StrategyDrill,
                ctx.skill_id,
                ctx.schema_id,
                ctx.domain.clone(),
                ctx.primary_error.clone(),
                ctx.source_attempt_id,
                "Strategy selection error. Drilling initial approach choice.",
            )
            .with_recurrence(rec)
            .with_urgency(if rec > 1 { RemediationUrgency::Critical } else { RemediationUrgency::Normal }),

            ErrorCategory::Sign => RemediationAction::new(
                action_id,
                RemediationActionKind::ProceduralVariant,
                ctx.skill_id,
                ctx.schema_id,
                ctx.domain.clone(),
                ctx.primary_error.clone(),
                ctx.source_attempt_id,
                "Sign error. Presenting sign-focused directional variant.",
            )
            .with_variant(Some("directional_sign".to_string()))
            .with_difficulty(1)
            .with_recurrence(rec),

            ErrorCategory::Unit => {
                let kind = if rec > 1 {
                    RemediationActionKind::DeclarativeRecall
                } else {
                    RemediationActionKind::ProceduralVariant
                };
                RemediationAction::new(
                    action_id,
                    kind,
                    ctx.skill_id,
                    ctx.schema_id,
                    ctx.domain.clone(),
                    ctx.primary_error.clone(),
                    ctx.source_attempt_id,
                    "Unit confusion. Practicing unit conversions.",
                )
                .with_variant(Some("unit_conversion".to_string()))
                .with_difficulty(1)
                .with_recurrence(rec)
            }

            ErrorCategory::Calculation => RemediationAction::new(
                action_id,
                RemediationActionKind::ProceduralVariant,
                ctx.skill_id,
                ctx.schema_id,
                ctx.domain.clone(),
                ctx.primary_error.clone(),
                ctx.source_attempt_id,
                "Calculation slip. Serving same schema with lower numerical complexity.",
            )
            .with_variant(Some("simpler_numbers".to_string()))
            .with_difficulty(1)
            .with_recurrence(rec),

            ErrorCategory::Careless | ErrorCategory::ProceduralSlip | ErrorCategory::Syntax => RemediationAction::new(
                action_id,
                RemediationActionKind::ProceduralVariant,
                ctx.skill_id,
                ctx.schema_id,
                ctx.domain.clone(),
                ctx.primary_error.clone(),
                ctx.source_attempt_id,
                "Procedural slip or syntax issue. Serving standard baseline problem.",
            )
            .with_difficulty(1)
            .with_recurrence(rec),

            ErrorCategory::Time | ErrorCategory::Timeout => RemediationAction::new(
                action_id,
                RemediationActionKind::ProceduralVariant,
                ctx.skill_id,
                ctx.schema_id,
                ctx.domain.clone(),
                ctx.primary_error.clone(),
                ctx.source_attempt_id,
                "Time limit exceeded. Reinforcing fluency on simpler variant.",
            )
            .with_difficulty(1)
            .with_recurrence(rec),

            ErrorCategory::Unknown | ErrorCategory::DomainSpecific(_) => RemediationAction::new(
                action_id,
                RemediationActionKind::ProceduralVariant,
                ctx.skill_id,
                ctx.schema_id,
                ctx.domain.clone(),
                ctx.primary_error.clone(),
                ctx.source_attempt_id,
                "Safe standard fallback procedural practice.",
            )
            .with_difficulty(1)
            .with_recurrence(rec),
        }
    }
}
