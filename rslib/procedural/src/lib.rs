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
pub mod core;
pub mod diagnostics;
pub mod exam;
pub mod physics;
pub mod practice;
pub mod problems;
pub mod reasoning;
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
pub use diagnostics::{AttemptDiagnosticSummary, ErrorCategory, ProceduralReviewOutcome};
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
    BloodRelationPuzzle, CognitiveDecisionPoint, CspConstraint, CspProblem, CspSolver,
    DecisionOption, DirectionPuzzle, Heading, KinshipRelation, KinshipStatement,
    ReasoningErrorCategory, ReasoningProblemMetadata, RelationsGenerator, RelationsValidator,
    SchemaKind as ReasoningSchemaKind, SeatingGenerator, SeatingPuzzle, SeatingValidator,
    SeriesGenerator, SeriesProblem, SeriesRule, SeriesValidator, StrategyKind,
    SyllogismGenerator, SyllogismProblem, SyllogismValidator,
};
pub use practice::{ErrorEvent, PracticeAttempt, SchemaPracticeObject};
pub use problems::{
    catalog::*,
    generator::{ProblemGenerator, VariantType},
    generators::*,
    registry::ProblemRegistry,
    steps::*,
    validator::{AnswerEvaluation, NumericAnswerParser, PercentageSuccessiveValidator, ProblemValidator},
    ProblemFamily, ProblemInstance,
};
pub use reviewer::render_reviewer_html;
pub use scheduling::{
    derive_fsrs_rating, AdaptiveDifficultyEngine, DifficultyDecision,
    MultiSchemaSelectionDecision, MultiSchemaSelector, PracticeMode, PracticeSessionObject,
    Rating, RatingPolicy, SelectionDecision, SessionReadiness, StandardRatingPolicy,
    TransferEligibility, TransferEligibilityEngine, VariantSelector,
};
pub use service::ProceduralService;
pub use skills::{
    ErrorFrequencyCounts, MovingLatencyStats, PracticeProgressionState, RecentAttemptRecord, Skill,
    SkillState, VariantPerformance,
};
pub use storage::{MigrationRunner, ProceduralStore};
