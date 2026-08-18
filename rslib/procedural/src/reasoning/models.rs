// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use serde::{Deserialize, Serialize};

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
        }
    }
}

impl std::fmt::Display for SchemaKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A structured option for a cognitive decision point.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecisionOption {
    pub id: String,
    pub label: String,
    pub strategy: StrategyKind,
    pub is_valid: bool,
    pub feedback: String,
}

impl DecisionOption {
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        strategy: StrategyKind,
        is_valid: bool,
        feedback: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            strategy,
            is_valid,
            feedback: feedback.into(),
        }
    }
}

/// Micro learning object capturing strategic reasoning choice before or during problem solving.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CognitiveDecisionPoint {
    pub id: String,
    pub prompt: String,
    pub options: Vec<DecisionOption>,
    pub preferred_option_id: String,
    pub preferred_strategy: StrategyKind,
    pub explanation: String,
}

impl CognitiveDecisionPoint {
    pub fn new(
        id: impl Into<String>,
        prompt: impl Into<String>,
        options: Vec<DecisionOption>,
        preferred_option_id: impl Into<String>,
        preferred_strategy: StrategyKind,
        explanation: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            prompt: prompt.into(),
            options,
            preferred_option_id: preferred_option_id.into(),
            preferred_strategy,
            explanation: explanation.into(),
        }
    }

    /// Evaluate a learner's decision choice.
    pub fn evaluate_choice(&self, chosen_id: &str) -> (bool, Option<StrategyKind>, String) {
        if let Some(opt) = self.options.iter().find(|o| o.id == chosen_id) {
            (opt.is_valid, Some(opt.strategy), opt.feedback.clone())
        } else {
            (false, None, "Invalid decision option chosen.".to_string())
        }
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

    pub fn as_strategy_drill(mut self) -> Self {
        self.is_strategy_drill = true;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decision_point_evaluation() {
        let opt1 = DecisionOption::new(
            "opt_a",
            "Anchor fixed person C at position 3",
            StrategyKind::AnchorFixed,
            true,
            "Correct: Fixed positions immediately reduce variable domains.",
        );
        let opt2 = DecisionOption::new(
            "opt_b",
            "Try placing flexible person A randomly",
            StrategyKind::BranchCases,
            false,
            "Sub-optimal: Guessing without anchoring fixed positions increases branching.",
        );

        let dp = CognitiveDecisionPoint::new(
            "dp_seating_1",
            "Which step should you perform first?",
            vec![opt1, opt2],
            "opt_a",
            StrategyKind::AnchorFixed,
            "Always anchor invariant fixed positions before relative constraints.",
        );

        let (is_valid, strategy, feedback) = dp.evaluate_choice("opt_a");
        assert!(is_valid);
        assert_eq!(strategy, Some(StrategyKind::AnchorFixed));
        assert!(feedback.contains("Fixed positions"));

        let (is_valid2, strategy2, _) = dp.evaluate_choice("opt_b");
        assert!(!is_valid2);
        assert_eq!(strategy2, Some(StrategyKind::BranchCases));
    }
}
