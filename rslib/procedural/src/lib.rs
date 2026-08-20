// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! # Procedural Practice Engine (Subsystem)
//!
//! Isolated subsystem for procedural practice, multi-domain skill modeling,
//! problem seed/instance generation tracking, and learning diagnostics.
//!
//! This crate is deliberately isolated from Anki's collection database,
//! FSRS internals, and revlog semantics.

pub mod anchor;
pub mod chemistry;
pub mod content;
pub mod core;
pub mod diagnostics;
pub mod exam;
pub mod physics;
pub mod practice;
pub mod problems;
pub mod reasoning;
pub mod remediation;
pub mod reviewer;
pub mod scheduling;
pub mod service;
pub mod skills;
pub mod storage;

pub use anchor::{ProceduralCardAnchor, SeedMode};
pub use chemistry::{
    ChemicalDimension, ChemicalDimensionalValidator, ChemicalInvariantValidator,
    ChemicalProblemMetadata, ChemicalQuantity, ChemicalReaction, ChemicalRegimeKind,
    ChemicalSpecies, ChemistryErrorCategory, ChemistryUnit, EquilibriumGenerator,
    EquilibriumValidator, ReactionParticipant, ReactionTemplates, SpeciesCatalog,
    StateOfMatter, StoichiometryGenerator, StoichiometryValidator,
};
pub use core::{
    AttemptId, Domain, ErrorEventId, ExamProfileId, ProblemFamilyId, ProblemInstanceId,
    ProceduralError, PyqId, RejectedVariantId, Result, SchemaId, SkillId,
};
pub use core::decision::{CognitiveDecisionPoint, DecisionOption};
pub use diagnostics::{
    AttemptDiagnosticSummary, ErrorCategory, HintDependencyStats, HintLevel, HintUsageRecord,
    ProceduralReviewOutcome,
};
pub use exam::{
    ContentProvenance, ExamFailingSchemaSummary, ExamObjective, ExamPracticeMode, ExamProfile,
    ExamRelevanceScore, ExamRelevanceScorer, ExamSessionSelector, HumanReviewWorkflow,
    MappingConfidence, MappingStatus, PyqAnalyticsEngine, PyqMasteryAction, PyqMasteryBridge,
    PyqSourcePerformance, PyqVariantPipeline, PYQSource, PyqMapping, RejectedVariantRecord,
    ReviewAction, ReviewInspection, DEFAULT_CATALOG_VERSION, DEFAULT_GENERATOR_VERSION,
    DEFAULT_PYQ_SOURCE_VERSION, DEFAULT_SCHEMA_VERSION,
};
pub use physics::{
    CoordinateSystem, DimensionalValidator, Kinematics1DGenerator, Kinematics1DValidator,
    PhysicalDimension, PhysicalModelKind, PhysicalProblemMetadata, PhysicalQuantity,
    PhysicalRegime, PhysicalSanityValidator, PhysicsErrorCategory, PhysicsUnit,
    WorkEnergyGenerator, WorkEnergyValidator,
};
pub use reasoning::{
    BloodRelationPuzzle, CspConstraint, CspProblem, CspSolver,
    DirectionPuzzle, Heading, KinshipRelation, KinshipStatement,
    ReasoningErrorCategory, ReasoningProblemMetadata, RelationsGenerator, RelationsValidator,
    SchemaKind as ReasoningSchemaKind, SeatingGenerator, SeatingPuzzle, SeatingValidator,
    SeriesGenerator, SeriesProblem, SeriesRule, SeriesValidator, StrategyKind,
    SyllogismGenerator, SyllogismProblem, SyllogismValidator,
};
pub use practice::{
    DifficultyConstraint, ErrorEvent, PracticeAttempt, PracticeObjective, PracticeRequest,
    PracticeScope, RemediationPrecedence, SchemaPracticeObject, SessionBudget, TimeConstraint,
};
pub use remediation::{
    CircuitBreakerObject, ConceptCheckEvaluation, ConceptCheckObject, ConceptCheckOption, DeclarativeRecallBridge,
    PrerequisiteReviewObject, RemediationAction, RemediationActionKind, RemediationAuditLog,
    RemediationAuditRecord, RemediationContext, RemediationIntervention, RemediationOutcomeStatus,
    RemediationPolicy, RemediationQueue, RemediationSelector, RemediationUrgency,
    RepresentationDrillEvaluation, RepresentationDrillObject, RepresentationOption,
    StrategyDrillEvaluation, StrategyDrillObject, StrategyOption, WorkedExampleObject,
};
pub use problems::{
    catalog::*,
    generator::{ProblemGenerator, VariantType},
    generators::*,
    registry::ProblemRegistry,
    steps::*,
    validator::{AnswerEvaluation, NumericAnswerParser, PercentageSuccessiveValidator, ProblemValidator},
    variation::*,
    ProblemFamily, ProblemInstance,
};
pub use reviewer::render_reviewer_html;
pub use scheduling::{
    derive_fsrs_rating, AdaptiveDifficultyEngine, BacklogSeverity, BacklogTriageEngine,
    BacklogTriagePlan, DifficultyDecision, DomainBlock, DomainBudget, DomainSpeedConfig,
    InterleavingPolicy, LearningObjectKind, MacroBudgetPlanner, MacroPlanningContext,
    MacroSessionPlan, MultiSchemaSelectionDecision, MultiSchemaSelector, PracticeMode,
    PracticeSessionObject, PriorityTier, Rating, RatingPolicy, SelectionDecision,
    SessionBudgetTracker, SessionReadiness, SpeedEvaluation, SpeedRating, StandardRatingPolicy,
    StageSpeedPolicy, StructuralCoverageEvaluator, StructuralCoverageProfile,
    TransferEligibility, TransferEligibilityEngine, TransferEligibilityEvaluation,
    TransferEngine, TransferLevel, TriagedBacklogItem, UnifiedPracticeEngine,
    UnifiedSelectionDecision, VariantSelector, WorkloadSafeguards, WorkloadSnapshot, WorkloadState,
    DEFAULT_ANTI_STARVATION_FLOOR, MAX_REMEDIATION_SESSION_FRACTION,
};
pub use service::ProceduralService;
pub use skills::{
    DEFAULT_MAX_PREREQUISITE_DEPTH, ErrorFrequencyCounts, IndependenceLevel,
    MaintenanceReviewOutcome, MasteryEvidence, MovingLatencyStats, PracticeProgressionState,
    PrerequisiteEvaluation, PrerequisiteGraphService, PrerequisitePolicy, PrerequisiteReadiness,
    ProgressionPolicy, RecentAttemptRecord, RetirementEvaluation, RetirementPolicy, Skill,
    SkillState, VariantCategory, VariantPerformance,
};
pub use storage::{MigrationRunner, ProceduralStore};
