// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! # Reasoning Engine Subsystem (V1)
//!
//! Subsystem for schema recognition, strategy selection, structured representation,
//! constraint satisfaction, formal logic inference, and cognitive decision point practice.

pub mod csp;
pub mod diagnostics;
pub mod generators;
pub mod models;
pub mod relations;
pub mod seating;
pub mod series;
pub mod syllogism;

pub use csp::{CspConstraint, CspProblem, CspSolver, SearchCase};
pub use diagnostics::ReasoningErrorCategory;
pub use generators::{
    RelationsGenerator, RelationsValidator, SeatingGenerator, SeatingValidator,
    SeriesGenerator, SeriesValidator, SyllogismGenerator, SyllogismValidator,
    FAMILY_REASONING_RELATIONS, FAMILY_REASONING_SEATING, FAMILY_REASONING_SERIES,
    FAMILY_REASONING_SYLLOGISM, TEMPLATE_REASONING_RELATIONS_V1, TEMPLATE_REASONING_SEATING_V1,
    TEMPLATE_REASONING_SERIES_V1, TEMPLATE_REASONING_SYLLOGISM_V1,
};
pub use models::{
    CognitiveDecisionPoint, DecisionOption, ReasoningProblemMetadata, SchemaKind, StrategyKind,
};
pub use relations::{
    BloodRelationPuzzle, DirectionPuzzle, Heading, KinshipRelation, KinshipStatement,
};
pub use seating::SeatingPuzzle;
pub use series::{SeriesProblem, SeriesRule};
pub use syllogism::{
    ConclusionVerdict, EvaluatedConclusion, Proposition, Quantifier, SyllogismProblem,
};
