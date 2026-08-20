// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

pub mod backlog_triage;
pub mod coverage;
pub mod difficulty;
pub mod interleaving;
pub mod macro_allocator;
pub mod rating_policy;
pub mod selector;
pub mod speed;
pub mod transfer;
pub mod unified;
pub mod workload;

use serde::{Deserialize, Serialize};

use crate::core::SkillId;
use crate::practice::SchemaPracticeObject;
use crate::problems::ProblemInstance;
use crate::skills::SkillState;

pub use backlog_triage::{
    BacklogSeverity, BacklogTriageEngine, BacklogTriagePlan, TriagedBacklogItem,
};
pub use coverage::{StructuralCoverageEvaluator, StructuralCoverageProfile};
pub use difficulty::{AdaptiveDifficultyEngine, DifficultyDecision};
pub use interleaving::InterleavingPolicy;
pub use macro_allocator::{
    DomainBlock, DomainBudget, MacroBudgetPlanner, MacroPlanningContext, MacroSessionPlan,
    DEFAULT_ANTI_STARVATION_FLOOR, MAX_REMEDIATION_SESSION_FRACTION,
};
pub use rating_policy::{derive_fsrs_rating, Rating, RatingPolicy, StandardRatingPolicy};
pub use selector::{
    MultiSchemaSelectionDecision, MultiSchemaSelector, PracticeMode, SelectionDecision,
    TransferEligibility, TransferEligibilityEngine, VariantSelector,
};
pub use speed::{DomainSpeedConfig, SpeedEvaluation, SpeedRating, StageSpeedPolicy};
pub use transfer::{TransferEligibilityEvaluation, TransferEngine, TransferLevel};
pub use unified::{
    LearningObjectKind, PriorityTier, UnifiedPracticeEngine, UnifiedSelectionDecision,
};
pub use workload::{
    SessionBudgetTracker, WorkloadSafeguards, WorkloadSnapshot, WorkloadState,
};

/// Status indicating whether a procedural practice object is ready for practice.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum SessionReadiness {
    Ready,
    PrerequisitesNeeded {
        missing_skills: Vec<SkillId>,
    },
    Mastered {
        current_mastery: f64,
        target_mastery: f64,
    },
    Cooldown {
        next_available_at: i64,
    },
}

/// Prepared practice session ready for presentation to a learner.
/// Contains ephemeral problem instance, selected variant metadata, and snapshot of learning state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PracticeSessionObject {
    pub schema: SchemaPracticeObject,
    pub instance: ProblemInstance,
    pub card_id: Option<i64>,
    pub skill_state: Option<SkillState>,
    pub readiness: SessionReadiness,
    pub selected_variant: Option<String>,
    pub target_latency_ms: Option<u64>,
    pub selection_reason: Option<String>,
    pub difficulty_level: Option<u32>,
}

impl PracticeSessionObject {
    pub fn new(
        schema: SchemaPracticeObject,
        instance: ProblemInstance,
        card_id: Option<i64>,
        skill_state: Option<SkillState>,
    ) -> Self {
        Self {
            schema,
            instance,
            card_id,
            skill_state,
            readiness: SessionReadiness::Ready,
            selected_variant: None,
            target_latency_ms: None,
            selection_reason: None,
            difficulty_level: None,
        }
    }

    pub fn with_readiness(mut self, readiness: SessionReadiness) -> Self {
        self.readiness = readiness;
        self
    }

    pub fn with_selection_decision(mut self, decision: &SelectionDecision) -> Self {
        self.selected_variant = Some(decision.variant.as_str().to_string());
        self.target_latency_ms = Some(decision.target_time_ms);
        self.selection_reason = Some(decision.selection_reason.clone());
        self
    }

    pub fn with_multi_schema_decision(mut self, decision: &MultiSchemaSelectionDecision) -> Self {
        self.difficulty_level = Some(decision.difficulty_level);
        self.target_latency_ms = Some(decision.target_time_ms);
        self.selection_reason = Some(decision.selection_reason.clone());
        self.selected_variant = decision.selected_variant.clone();
        self
    }
}
