// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

pub mod coded_expressions;
pub mod data_sufficiency;
pub mod floor_grid;
pub mod logic_dag;
pub mod relations;
pub mod seating;
pub mod series;
pub mod syllogism;

pub use coded_expressions::{
    CodedExpressionsGenerator, CodedExpressionsValidator, FAMILY_REASONING_CODED_EXPRESSIONS,
    TEMPLATE_REASONING_CODED_EXPRESSIONS_V1,
};
pub use data_sufficiency::{
    DataSufficiencyGenerator, DataSufficiencyValidator, FAMILY_REASONING_DATA_SUFFICIENCY,
    TEMPLATE_REASONING_DATA_SUFFICIENCY_V1,
};
pub use floor_grid::{
    FloorGridGenerator, FloorGridValidator, FAMILY_REASONING_FLOOR_GRID,
    TEMPLATE_REASONING_FLOOR_GRID_V1,
};
pub use logic_dag::{
    LogicDagGenerator, LogicDagValidator, FAMILY_REASONING_LOGIC_DAG,
    TEMPLATE_REASONING_LOGIC_DAG_V1,
};
pub use relations::{RelationsGenerator, RelationsValidator};
pub use seating::{SeatingGenerator, SeatingValidator};
pub use series::{SeriesGenerator, SeriesValidator};
pub use syllogism::{SyllogismGenerator, SyllogismValidator};

// Problem Family IDs
pub const FAMILY_REASONING_SERIES: &str = "family.reasoning.series.patterns";
pub const TEMPLATE_REASONING_SERIES_V1: &str = "reasoning.series.patterns.v1";

pub const FAMILY_REASONING_SYLLOGISM: &str = "family.reasoning.syllogism.categorical";
pub const TEMPLATE_REASONING_SYLLOGISM_V1: &str = "reasoning.syllogism.categorical.v1";

pub const FAMILY_REASONING_SEATING: &str = "family.reasoning.seating.linear";
pub const TEMPLATE_REASONING_SEATING_V1: &str = "reasoning.seating.linear.v1";

pub const FAMILY_REASONING_RELATIONS: &str = "family.reasoning.relations.graph";
pub const TEMPLATE_REASONING_RELATIONS_V1: &str = "reasoning.relations.graph.v1";
