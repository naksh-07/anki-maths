// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! # Reasoning Engine Subsystem (V1 & G3 Structural Expansion)
//!
//! Subsystem for schema recognition, strategy selection, structured representation,
//! constraint satisfaction, formal logic inference, data sufficiency, symbolic coded operators,
//! and cognitive decision point practice.

pub mod coded_expressions;
pub mod csp;
pub mod data_sufficiency;
pub mod diagnostics;
pub mod floor_grid;
pub mod generators;
pub mod logic_dag;
pub mod models;
pub mod relations;
pub mod seating;
pub mod series;
pub mod syllogism;

pub use coded_expressions::{CodedExpressionKind, CodedExpressionsPuzzle};
pub use csp::{CspConstraint, CspProblem, CspSolver, SearchCase};
pub use data_sufficiency::{DataSufficiencyPuzzle, DsAnswer};
pub use diagnostics::ReasoningErrorCategory;
pub use floor_grid::FloorGridPuzzle;
pub use generators::{
    CodedExpressionsGenerator, CodedExpressionsValidator, DataSufficiencyGenerator,
    DataSufficiencyValidator, FloorGridGenerator, FloorGridValidator, LogicDagGenerator,
    LogicDagValidator, RelationsGenerator, RelationsValidator, SeatingGenerator, SeatingValidator,
    SeriesGenerator, SeriesValidator, SyllogismGenerator, SyllogismValidator,
    FAMILY_REASONING_CODED_EXPRESSIONS, FAMILY_REASONING_DATA_SUFFICIENCY,
    FAMILY_REASONING_FLOOR_GRID, FAMILY_REASONING_LOGIC_DAG, FAMILY_REASONING_RELATIONS,
    FAMILY_REASONING_SEATING, FAMILY_REASONING_SERIES, FAMILY_REASONING_SYLLOGISM,
    TEMPLATE_REASONING_CODED_EXPRESSIONS_V1, TEMPLATE_REASONING_DATA_SUFFICIENCY_V1,
    TEMPLATE_REASONING_FLOOR_GRID_V1, TEMPLATE_REASONING_LOGIC_DAG_V1,
    TEMPLATE_REASONING_RELATIONS_V1, TEMPLATE_REASONING_SEATING_V1, TEMPLATE_REASONING_SERIES_V1,
    TEMPLATE_REASONING_SYLLOGISM_V1,
};
pub use logic_dag::{LogicDagPuzzle, LogicRule};
pub use models::{ReasoningProblemMetadata, SchemaKind, StrategyKind};
pub use relations::{
    BloodRelationPuzzle, DirectionPuzzle, Heading, KinshipRelation, KinshipStatement,
};
pub use seating::SeatingPuzzle;
pub use series::{SeriesProblem, SeriesRule};
pub use syllogism::{
    ConclusionVerdict, EvaluatedConclusion, Proposition, Quantifier, SyllogismProblem,
};
