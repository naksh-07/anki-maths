// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use serde::{Deserialize, Serialize};

use crate::skills::signals::VariantCategory;
use crate::skills::{PracticeProgressionState, SkillState};

/// Structural diversity profile for a skill or schema.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct StructuralCoverageProfile {
    pub parameter_count: u32,
    pub isomorphic_count: u32,
    pub structural_count: u32,
    pub contextual_count: u32,
    pub multi_concept_count: u32,
    pub transfer_count: u32,
    pub total_structural_forms_passed: usize,
}

impl StructuralCoverageProfile {
    pub fn from_skill_state(state: &SkillState) -> Self {
        let mut profile = Self::default();
        for perf in state.variant_stats.values() {
            match perf.category {
                VariantCategory::Parameter => profile.parameter_count += perf.successful_attempts,
                VariantCategory::Isomorphic => profile.isomorphic_count += perf.successful_attempts,
                VariantCategory::Structural => profile.structural_count += perf.successful_attempts,
                VariantCategory::Contextual => profile.contextual_count += perf.successful_attempts,
                VariantCategory::MultiConcept => profile.multi_concept_count += perf.successful_attempts,
                VariantCategory::Transfer => profile.transfer_count += perf.successful_attempts,
            }
        }
        for (form_key, &count) in &state.structural_forms_seen {
            if form_key.contains("param") {
                profile.parameter_count = profile.parameter_count.max(count);
            } else if form_key.contains("isomorphic") {
                profile.isomorphic_count = profile.isomorphic_count.max(count);
            } else if form_key.contains("contextual") {
                profile.contextual_count = profile.contextual_count.max(count);
            } else if form_key.contains("multi_concept") {
                profile.multi_concept_count = profile.multi_concept_count.max(count);
            } else if form_key.contains("transfer") {
                profile.transfer_count = profile.transfer_count.max(count);
            } else {
                profile.structural_count = profile.structural_count.max(count);
            }
        }
        profile.total_structural_forms_passed = state.distinct_structural_forms_passed();
        profile
    }

    /// Recommends the next optimal variant category for structural progression.
    pub fn recommend_next_category(&self, progression_stage: PracticeProgressionState) -> VariantCategory {
        match progression_stage {
            PracticeProgressionState::New | PracticeProgressionState::Learning => {
                if self.parameter_count < 2 {
                    VariantCategory::Parameter
                } else {
                    VariantCategory::Isomorphic
                }
            }
            PracticeProgressionState::Fluent => {
                if self.structural_count == 0 {
                    VariantCategory::Structural
                } else if self.contextual_count == 0 {
                    VariantCategory::Contextual
                } else {
                    VariantCategory::MultiConcept
                }
            }
            PracticeProgressionState::Variation => {
                if self.contextual_count == 0 {
                    VariantCategory::Contextual
                } else if self.multi_concept_count == 0 {
                    VariantCategory::MultiConcept
                } else {
                    VariantCategory::Transfer
                }
            }
            PracticeProgressionState::Transfer => VariantCategory::Transfer,
            PracticeProgressionState::Mastered
            | PracticeProgressionState::Retired
            | PracticeProgressionState::Hibernating => {
                // Maintenance probes favor structural or transfer
                VariantCategory::Structural
            }
        }
    }
}

/// Evaluator enforcing bounded procedural queue quotas and structural novelty.
pub struct StructuralCoverageEvaluator;

impl StructuralCoverageEvaluator {
    /// Computes structural novelty multiplier for candidate ranking.
    /// Boosts underexposed structural/transfer forms and dampens over-practiced parameter templates.
    pub fn compute_novelty_multiplier(
        state: Option<&SkillState>,
        target_category: &VariantCategory,
    ) -> f64 {
        let Some(s) = state else {
            // New skill: parameter/isomorphic forms favored
            return if matches!(target_category, VariantCategory::Parameter | VariantCategory::Isomorphic) {
                1.2
            } else {
                0.8
            };
        };

        let profile = StructuralCoverageProfile::from_skill_state(s);

        match s.practice_state {
            PracticeProgressionState::New | PracticeProgressionState::Learning => {
                // Focus on parameter acquisition if under quota, otherwise encourage isomorphic
                if matches!(target_category, VariantCategory::Parameter) {
                    if profile.parameter_count < 3 { 1.25 } else { 0.90 }
                } else if matches!(target_category, VariantCategory::Isomorphic) {
                    1.15
                } else {
                    0.75
                }
            }
            PracticeProgressionState::Fluent => {
                // Need structural variation
                if matches!(target_category, VariantCategory::Structural | VariantCategory::Contextual) {
                    1.40
                } else if matches!(target_category, VariantCategory::Parameter) {
                    0.70 // Penalize repeated parameter solving once fluent
                } else {
                    1.10
                }
            }
            PracticeProgressionState::Variation => {
                if matches!(target_category, VariantCategory::Contextual | VariantCategory::MultiConcept | VariantCategory::Transfer) {
                    1.50
                } else if matches!(target_category, VariantCategory::Parameter | VariantCategory::Isomorphic) {
                    0.60
                } else {
                    1.10
                }
            }
            PracticeProgressionState::Transfer => {
                if matches!(target_category, VariantCategory::Transfer | VariantCategory::MultiConcept) {
                    1.60
                } else {
                    0.70
                }
            }
            PracticeProgressionState::Mastered
            | PracticeProgressionState::Retired
            | PracticeProgressionState::Hibernating => {
                // Maintenance probes prefer structural or transfer
                if matches!(target_category, VariantCategory::Structural | VariantCategory::Transfer) {
                    1.30
                } else {
                    0.80
                }
            }
        }
    }
}
