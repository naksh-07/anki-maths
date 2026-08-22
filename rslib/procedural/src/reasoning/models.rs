// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use serde::{Deserialize, Serialize};

use crate::core::decision::CognitiveDecisionPoint;

/// High-level taxonomy of reasoning solving strategies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StrategyKind {
    /// Anchor a fixed entity or invariant slot first (e.g. "X sits at slot 3").
    AnchorFixed,
    /// Resolve immediate adjacency and non-adjacency constraints.
    PropagateAdjacency,
    /// Inspect successive differences in sequences (first, second differences).
    InspectDifferences,
    /// Inspect multiplicative ratios in geometric sequences.
    InspectRatios,
    /// Inspect alternating dual sub-sequences.
    InspectAlternating,
    /// Inspect positional character shifts in alphabet series.
    InspectAlphabetShift,
    /// Construct kinship relational family tree graph from reference person.
    ConstructKinshipGraph,
    /// Trace 2D displacement coordinate vectors and heading orientation.
    TraceDirectionVectors,
    /// Direct formal categorical syllogistic inference.
    DirectSyllogisticDeduction,
    /// Test truth assignment or conclusion validity via contradiction/counter-example.
    TestContradiction,
    /// Split search into exhaustive branching cases.
    BranchCases,
    /// Eliminate contradictory or invalid candidates.
    EliminateInvalid,
}

impl StrategyKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            StrategyKind::AnchorFixed => "anchor_fixed",
            StrategyKind::PropagateAdjacency => "propagate_adjacency",
            StrategyKind::InspectDifferences => "inspect_differences",
            StrategyKind::InspectRatios => "inspect_ratios",
            StrategyKind::InspectAlternating => "inspect_alternating",
            StrategyKind::InspectAlphabetShift => "inspect_alphabet_shift",
            StrategyKind::ConstructKinshipGraph => "construct_kinship_graph",
            StrategyKind::TraceDirectionVectors => "trace_direction_vectors",
            StrategyKind::DirectSyllogisticDeduction => "direct_syllogistic_deduction",
            StrategyKind::TestContradiction => "test_contradiction",
            StrategyKind::BranchCases => "branch_cases",
            StrategyKind::EliminateInvalid => "eliminate_invalid",
        }
    }
}

impl std::fmt::Display for StrategyKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Recognized structural schema classifications for reasoning problems.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SchemaKind {
    LinearSeating,
    CircularSeating,
    CategoricalSyllogism,
    NumberSeries,
    AlphabetSeries,
    BloodRelations,
    DirectionSense,
    FloorGridCsp,
    LogicDag,
    DataSufficiency,
    CodedExpressions,
}

impl SchemaKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            SchemaKind::LinearSeating => "linear_seating",
            SchemaKind::CircularSeating => "circular_seating",
            SchemaKind::CategoricalSyllogism => "categorical_syllogism",
            SchemaKind::NumberSeries => "number_series",
            SchemaKind::AlphabetSeries => "alphabet_series",
            SchemaKind::BloodRelations => "blood_relations",
            SchemaKind::DirectionSense => "direction_sense",
            SchemaKind::FloorGridCsp => "floor_grid_csp",
            SchemaKind::LogicDag => "logic_dag",
            SchemaKind::DataSufficiency => "data_sufficiency",
            SchemaKind::CodedExpressions => "coded_expressions",
        }
    }
}

impl std::fmt::Display for SchemaKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}



/// Metadata carrier attached to generated reasoning problem instances.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReasoningProblemMetadata {
    pub schema_kind: SchemaKind,
    pub strategy_kind: StrategyKind,
    pub decision_points: Vec<CognitiveDecisionPoint>,
    pub constraint_count: usize,
    pub search_depth: usize,
    pub is_unambiguous: bool,
    pub is_strategy_drill: bool,
    #[serde(default)]
    pub scaffolding_level: u32,
    #[serde(default)]
    pub constraint_density: f64,
    #[serde(default)]
    pub branching_factor: usize,
    #[serde(default)]
    pub trap_density: f64,
}

impl ReasoningProblemMetadata {
    pub fn new(schema_kind: SchemaKind, strategy_kind: StrategyKind) -> Self {
        Self {
            schema_kind,
            strategy_kind,
            decision_points: Vec::new(),
            constraint_count: 0,
            search_depth: 0,
            is_unambiguous: true,
            is_strategy_drill: false,
            scaffolding_level: 0,
            constraint_density: 0.0,
            branching_factor: 1,
            trap_density: 0.0,
        }
    }

    pub fn with_decision_point(mut self, dp: CognitiveDecisionPoint) -> Self {
        self.decision_points.push(dp);
        self
    }

    pub fn with_constraint_count(mut self, count: usize) -> Self {
        self.constraint_count = count;
        self
    }

    pub fn with_search_depth(mut self, depth: usize) -> Self {
        self.search_depth = depth;
        self
    }

    pub fn with_scaffolding_level(mut self, level: u32) -> Self {
        self.scaffolding_level = level;
        self
    }

    pub fn with_constraint_density(mut self, density: f64) -> Self {
        self.constraint_density = density;
        self
    }

    pub fn with_branching_factor(mut self, factor: usize) -> Self {
        self.branching_factor = factor;
        self
    }

    pub fn with_trap_density(mut self, trap_density: f64) -> Self {
        self.trap_density = trap_density;
        self
    }

    pub fn as_strategy_drill(mut self) -> Self {
        self.is_strategy_drill = true;
        self
    }
}

