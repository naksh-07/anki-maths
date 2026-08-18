// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::core::{ProceduralError, Result};
use crate::exam::pipeline::PyqVariantPipeline;
use crate::exam::pyq::{ContentProvenance, MappingConfidence, MappingStatus, PYQSource, PyqMapping};
use crate::problems::registry::ProblemRegistry;
use crate::problems::ProblemInstance;

/// Action executed by a content reviewer or educator during curation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum ReviewAction {
    /// Approve the mapping and mark it Verified for active study rotation
    Approve,
    /// Reject the mapping or question as invalid or poorly aligned
    Reject { reason: String },
    /// Re-assign the PYQ to a different schema, skill, or difficulty level
    Remap { mapping: PyqMapping },
    /// Regenerate a test variant with a new random or specified seed
    Regenerate { seed: u64 },
}

/// Comprehensive inspection view for reviewing a PYQ source, its mapping,
/// and its generated practice variant.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReviewInspection {
    pub source_pyq: PYQSource,
    pub mapping: Option<PyqMapping>,
    pub generated_variant: Option<ProblemInstance>,
    pub validator_passed: bool,
    pub validation_notes: Option<String>,
    pub difficulty_level: u32,
    pub target_latency_ms: u64,
    pub provenance: ContentProvenance,
}

/// Minimal internal workflow engine for human review and curation.
pub struct HumanReviewWorkflow;

impl HumanReviewWorkflow {
    /// Prepare a complete inspection object for a reviewer.
    pub fn inspect_pyq(
        registry: &ProblemRegistry,
        pyq: &PYQSource,
        mapping: Option<&PyqMapping>,
        sample_seed: u64,
    ) -> ReviewInspection {
        let mut sample_instance = None;
        let mut validator_passed = false;
        let mut validation_notes = None;
        let mut diff = 1;
        let mut lat = 45_000;

        if let Some(m) = mapping {
            diff = m.difficulty_level;
            lat = m.target_latency_ms;
            match PyqVariantPipeline::generate_and_validate_variant(
                registry,
                Some(pyq),
                m,
                sample_seed,
                None,
            ) {
                Ok(inst) => {
                    sample_instance = Some(inst);
                    validator_passed = true;
                    validation_notes = Some("Passed domain validation gate successfully.".into());
                }
                Err((err, _)) => {
                    validator_passed = false;
                    validation_notes = Some(format!("Validation failed: {}", err));
                }
            }
        }

        ReviewInspection {
            source_pyq: pyq.clone(),
            mapping: mapping.cloned(),
            generated_variant: sample_instance,
            validator_passed,
            validation_notes,
            difficulty_level: diff,
            target_latency_ms: lat,
            provenance: pyq.provenance.clone(),
        }
    }

    /// Apply a reviewer action to an existing mapping.
    pub fn apply_review_action(
        current_mapping: &mut PyqMapping,
        action: ReviewAction,
    ) -> Result<()> {
        match action {
            ReviewAction::Approve => {
                current_mapping.status = MappingStatus::Verified;
                current_mapping.confidence = MappingConfidence::Deterministic;
                current_mapping.updated_at = Utc::now().timestamp();
            }
            ReviewAction::Reject { reason } => {
                current_mapping.status = MappingStatus::Rejected;
                current_mapping.reviewer_notes = Some(reason);
                current_mapping.updated_at = Utc::now().timestamp();
            }
            ReviewAction::Remap { mapping } => {
                if mapping.pyq_id != current_mapping.pyq_id {
                    return Err(ProceduralError::Validation(format!(
                        "Cannot remap PYQ ID mismatch: {} vs {}",
                        mapping.pyq_id, current_mapping.pyq_id
                    )));
                }
                *current_mapping = mapping;
                current_mapping.status = MappingStatus::Mapped;
                current_mapping.confidence = MappingConfidence::HighConfidence;
                current_mapping.updated_at = Utc::now().timestamp();
            }
            ReviewAction::Regenerate { .. } => {
                // Regeneration is an inspection step; mapping remains in current status
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Domain;

    #[test]
    fn test_human_review_workflow_actions() {
        let mut mapping = PyqMapping::new(
            "pyq.test.002",
            Domain::Mathematics,
            "arithmetic.profit_loss",
            "schema.math.arithmetic.profit_loss",
            "family.math.arithmetic.profit_loss",
            2,
            40_000,
        )
        .with_status(MappingStatus::Unreviewed)
        .with_confidence(MappingConfidence::NeedsReview);

        assert!(!mapping.is_eligible_for_practice());

        // Approve action
        HumanReviewWorkflow::apply_review_action(&mut mapping, ReviewAction::Approve).unwrap();
        assert_eq!(mapping.status, MappingStatus::Verified);
        assert!(mapping.is_eligible_for_practice());

        // Reject action
        HumanReviewWorkflow::apply_review_action(
            &mut mapping,
            ReviewAction::Reject {
                reason: "Ambiguous question formulation in original shift paper".into(),
            },
        )
        .unwrap();
        assert_eq!(mapping.status, MappingStatus::Rejected);
        assert!(!mapping.is_eligible_for_practice());
        assert_eq!(
            mapping.reviewer_notes.as_deref(),
            Some("Ambiguous question formulation in original shift paper")
        );
    }
}
