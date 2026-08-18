// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use serde::{Deserialize, Serialize};

/// Standard vocabulary of discrete mathematical step types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepType {
    /// Choosing or stating the appropriate governing formula, theorem, or method.
    FormulaSelection,
    /// Applying an algebraic or domain-specific transformation.
    Transformation,
    /// Substituting known values into an algebraic expression or formula.
    Substitution,
    /// Executing pure numerical arithmetic calculations.
    Arithmetic,
    /// Simplifying terms, fractions, or combined expressions.
    Simplification,
    /// Rearranging terms, transposing constants, or isolating variables in an equation.
    EquationRearrangement,
    /// Comparing numerical values, quantities, or conditions.
    Comparison,
    /// Converting between units or scaling quantities.
    UnitConversion,
    /// Computing an intermediate sub-goal state or variable value.
    IntermediateResult,
    /// Producing the final verified answer.
    FinalAnswer,
    // --- Physics-specific step types ---
    /// Identifying and extracting known variables and target unknown.
    IdentifyKnowns,
    /// Selecting the governing physical model or law (e.g. Energy Conservation vs Kinematics).
    SelectModel,
    /// Choosing positive direction, origin, or coordinate system orientation.
    ChooseCoordinateSystem,
    /// Selecting or setting up the specific governing equation before substitution.
    SelectEquation,
    /// Evaluating physical plausibility (units, positivity, boundary bounds).
    PhysicalSanityCheck,
    // --- Chemistry-specific step types ---
    /// Identifying and extracting participating chemical species and formulas.
    IdentifyChemicalSpecies,
    /// Balancing the stoichiometric chemical equation.
    BalanceEquation,
    /// Converting given mass or volume to amount of substance in moles.
    ConvertMassToMoles,
    /// Applying stoichiometric molar ratios between reactants and products.
    ApplyStoichiometricRatio,
    /// Identifying the limiting reagent among initial reactant quantities.
    IdentifyLimitingReagent,
    /// Formulating the equilibrium constant mass-action expression (Kc/Kp).
    ConstructEquilibriumExpression,
    /// Validating chemical invariants (element conservation, concentration positivity).
    ChemicalSanityCheck,
    // --- Reasoning-specific step types ---
    /// Recognizing the underlying problem schema (e.g. Linear Seating vs Syllogism).
    IdentifySchema,
    /// Selecting the optimal solving strategy or starting constraint.
    SelectStrategy,
    /// Constructing the structured mental or diagrammatic representation (slots, tree, vectors, truth assignment).
    BuildRepresentation,
    /// Placing or enforcing an explicit constraint onto the representation.
    ApplyConstraint,
    /// Propagating deterministic deductions across the constraint network.
    PropagateConstraint,
    /// Making a direct deductive or relational logical inference.
    MakeInference,
    /// Opening a hypothetical branch/case for split analysis.
    CreateCase,
    /// Discarding a case due to logical contradiction or impossibility.
    EliminateCase,
    /// Verifying whether derived conditions contain a contradiction.
    CheckContradiction,
    /// Verifying conclusions against established facts or premises.
    VerifyConclusion,
}

impl StepType {
    pub fn as_str(&self) -> &'static str {
        match self {
            StepType::FormulaSelection => "formula_selection",
            StepType::Transformation => "transformation",
            StepType::Substitution => "substitution",
            StepType::Arithmetic => "arithmetic",
            StepType::Simplification => "simplification",
            StepType::EquationRearrangement => "equation_rearrangement",
            StepType::Comparison => "comparison",
            StepType::UnitConversion => "unit_conversion",
            StepType::IntermediateResult => "intermediate_result",
            StepType::FinalAnswer => "final_answer",
            StepType::IdentifyKnowns => "identify_knowns",
            StepType::SelectModel => "select_model",
            StepType::ChooseCoordinateSystem => "choose_coordinate_system",
            StepType::SelectEquation => "select_equation",
            StepType::PhysicalSanityCheck => "physical_sanity_check",
            StepType::IdentifyChemicalSpecies => "identify_chemical_species",
            StepType::BalanceEquation => "balance_equation",
            StepType::ConvertMassToMoles => "convert_mass_to_moles",
            StepType::ApplyStoichiometricRatio => "apply_stoichiometric_ratio",
            StepType::IdentifyLimitingReagent => "identify_limiting_reagent",
            StepType::ConstructEquilibriumExpression => "construct_equilibrium_expression",
            StepType::ChemicalSanityCheck => "chemical_sanity_check",
            StepType::IdentifySchema => "identify_schema",
            StepType::SelectStrategy => "select_strategy",
            StepType::BuildRepresentation => "build_representation",
            StepType::ApplyConstraint => "apply_constraint",
            StepType::PropagateConstraint => "propagate_constraint",
            StepType::MakeInference => "make_inference",
            StepType::CreateCase => "create_case",
            StepType::EliminateCase => "eliminate_case",
            StepType::CheckContradiction => "check_contradiction",
            StepType::VerifyConclusion => "verify_conclusion",
        }
    }
}

impl std::fmt::Display for StepType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Category / level of hint guidance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HintLevel {
    /// Level 1: Identifies the governing principle or formula.
    Principle = 1,
    /// Level 2: Identifies the specific operation type to perform next.
    Operation = 2,
    /// Level 3: Reveals the intermediate relationship or equation setup.
    IntermediateRelation = 3,
}

/// A deterministic hint associated with a specific step in the solution graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StepHint {
    pub level: u32,
    pub hint_level: HintLevel,
    pub title: String,
    pub content: String,
}

impl StepHint {
    pub fn new(level: HintLevel, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            level: level as u32,
            hint_level: level,
            title: title.into(),
            content: content.into(),
        }
    }

    pub fn principle(content: impl Into<String>) -> Self {
        Self::new(HintLevel::Principle, "Principle / Rule", content)
    }

    pub fn operation(content: impl Into<String>) -> Self {
        Self::new(HintLevel::Operation, "Next Operation", content)
    }

    pub fn intermediate_relation(content: impl Into<String>) -> Self {
        Self::new(HintLevel::IntermediateRelation, "Intermediate Setup", content)
    }
}

/// Node representing a discrete step in a structured solution graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StepNode {
    pub id: String,
    pub step_type: StepType,
    pub title: String,
    pub description: String,
    /// Canonical mathematical expression or relation (e.g. "3x = 12", "x = 4", "1.20 * 500 = 600")
    pub expected_expression: String,
    /// Canonical numeric target value if this step produces a scalar number
    pub expected_value: Option<f64>,
    /// Acceptable alternative expressions (e.g. "2x + 6 = 16", "2(x + 3) = 16")
    pub alternate_expressions: Vec<String>,
    /// Step IDs on which this step depends (topological predecessor links)
    pub dependencies: Vec<String>,
    pub is_final: bool,
    /// Deterministic 3-level hints for this step
    pub hints: Vec<StepHint>,
}

impl StepNode {
    pub fn new(
        id: impl Into<String>,
        step_type: StepType,
        title: impl Into<String>,
        description: impl Into<String>,
        expected_expression: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            step_type,
            title: title.into(),
            description: description.into(),
            expected_expression: expected_expression.into(),
            expected_value: None,
            alternate_expressions: Vec::new(),
            dependencies: Vec::new(),
            is_final: false,
            hints: Vec::new(),
        }
    }

    pub fn with_expected_value(mut self, val: f64) -> Self {
        self.expected_value = Some(val);
        self
    }

    pub fn with_alternates(mut self, alternates: Vec<String>) -> Self {
        self.alternate_expressions = alternates;
        self
    }

    pub fn with_dependencies(mut self, deps: Vec<String>) -> Self {
        self.dependencies = deps;
        self
    }

    pub fn as_final(mut self) -> Self {
        self.is_final = true;
        self
    }

    pub fn with_hints(mut self, hints: Vec<StepHint>) -> Self {
        self.hints = hints;
        self
    }
}

/// Generic mathematical solution graph / step graph abstraction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SolutionGraph {
    pub steps: Vec<StepNode>,
    pub final_step_id: String,
    pub metadata: serde_json::Value,
}

impl SolutionGraph {
    pub fn new(steps: Vec<StepNode>, final_step_id: impl Into<String>) -> Self {
        Self {
            steps,
            final_step_id: final_step_id.into(),
            metadata: serde_json::Value::Object(Default::default()),
        }
    }

    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    pub fn get_step(&self, id: &str) -> Option<&StepNode> {
        self.steps.iter().find(|s| s.id == id)
    }

    pub fn get_step_by_index(&self, index: usize) -> Option<&StepNode> {
        self.steps.get(index)
    }

    pub fn final_step(&self) -> Option<&StepNode> {
        self.get_step(&self.final_step_id)
            .or_else(|| self.steps.iter().find(|s| s.is_final))
            .or_else(|| self.steps.last())
    }

    /// Retrieve hints for a given step index (1-indexed hint levels up to 3).
    pub fn hints_for_step(&self, step_idx: usize) -> Vec<StepHint> {
        self.steps
            .get(step_idx)
            .map(|s| s.hints.clone())
            .unwrap_or_default()
    }

    /// Validate graph topology ensuring all dependency references exist and there are no circular dependencies.
    pub fn validate_topology(&self) -> bool {
        let step_ids: std::collections::HashSet<&str> = self.steps.iter().map(|s| s.id.as_str()).collect();

        // Ensure all dependencies exist
        for step in &self.steps {
            for dep in &step.dependencies {
                if !step_ids.contains(dep.as_str()) {
                    return false;
                }
            }
        }

        // Verify topological acyclicity
        let mut visited = std::collections::HashSet::new();
        let mut in_stack = std::collections::HashSet::new();

        for step in &self.steps {
            if !visited.contains(&step.id) {
                if self.has_cycle(&step.id, &mut visited, &mut in_stack) {
                    return false;
                }
            }
        }

        true
    }

    fn has_cycle(
        &self,
        node_id: &str,
        visited: &mut std::collections::HashSet<String>,
        in_stack: &mut std::collections::HashSet<String>,
    ) -> bool {
        visited.insert(node_id.to_string());
        in_stack.insert(node_id.to_string());

        if let Some(step) = self.get_step(node_id) {
            for dep in &step.dependencies {
                if !visited.contains(dep) {
                    if self.has_cycle(dep, visited, in_stack) {
                        return true;
                    }
                } else if in_stack.contains(dep) {
                    return true;
                }
            }
        }

        in_stack.remove(node_id);
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solution_graph_construction_and_topology() {
        let step1 = StepNode::new(
            "isolate_x",
            StepType::EquationRearrangement,
            "Isolate variable term",
            "Subtract 5 from both sides",
            "3x = 12",
        )
        .with_hints(vec![
            StepHint::principle("Balance equations by performing identical operations on both sides."),
            StepHint::operation("Subtract the constant 5 from both sides."),
            StepHint::intermediate_relation("3x = 17 - 5 = 12"),
        ]);

        let step2 = StepNode::new(
            "solve_x",
            StepType::FinalAnswer,
            "Solve for x",
            "Divide both sides by 3",
            "x = 4",
        )
        .with_expected_value(4.0)
        .with_dependencies(vec!["isolate_x".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Divide by coefficient to isolate x."),
            StepHint::operation("Divide both sides by 3."),
            StepHint::intermediate_relation("x = 12 / 3 = 4"),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2], "solve_x");
        assert_eq!(graph.step_count(), 2);
        assert!(graph.validate_topology());
        assert_eq!(graph.final_step().unwrap().id, "solve_x");
        assert_eq!(graph.hints_for_step(0).len(), 3);
    }
}
