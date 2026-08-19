// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use serde::{Deserialize, Serialize};
use crate::core::Domain;

/// Mathematics-specific variation dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MathsVariationDimension {
    /// Pure numeric/parameter adjustments with identical algebraic form.
    Parameter,
    /// Target variable inverted (e.g. given final quantity & rate, solve for initial).
    Reverse,
    /// Algebraic or geometric structure altered (e.g., additional step, non-monic).
    Structural,
    /// Boundary conditions, zero cases, edge constraints, or algebraic traps.
    BoundaryTrap,
    /// Compound multi-topic schema synthesis.
    MultiConcept,
    /// Cross-domain application (e.g. rate problems in physics/chemistry).
    Transfer,
}

/// Physics-specific variation dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PhysicsVariationDimension {
    /// Numerical magnitudes and unit conversions.
    Parameter,
    /// Non-zero initial velocity, angle, or frame of reference changes.
    InitialCondition,
    /// Graphical, vector, or diagrammatic representation vs text.
    Representation,
    /// Multiple physical laws applicable (e.g., energy conservation vs kinematic equations).
    ModelSelection,
    /// Multi-stage motions, compound forces, or coupled systems.
    Structural,
    /// Inter-domain physical modeling (e.g. thermodynamics with stoichiometry).
    Transfer,
}

/// Chemistry-specific variation dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChemistryVariationDimension {
    /// Molar mass, stoichiometric coefficients, concentration values.
    Quantity,
    /// Different chemical species, acids/bases, or polyatomic ions.
    Species,
    /// Limiting reagent present, excess reagent, or equilibrium direction shifts.
    Regime,
    /// Temperature/pressure bounds, solubility limits, or non-ideal behavior.
    Constraint,
    /// Multi-step synthesis or coupled equilibrium equations.
    Structural,
    /// Chemical thermodynamics and reasoning transfer.
    Transfer,
}

/// Reasoning-specific variation dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasoningVariationDimension {
    /// Concrete entity names, alphabetic sets, or seating labels.
    Entity,
    /// Directional, conditional, or exclusionary constraints.
    Constraint,
    /// Strategy choice (e.g., direct elimination vs branch-and-bound vs parity).
    Strategy,
    /// Grid topology (circular vs linear seating, syllogistic quantifier mix).
    Structure,
    /// State-space search complexity and backtrack depth.
    Search,
    /// Cross-context deductive modeling.
    Transfer,
}

/// Domain-unified representation of variation dimensions.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "domain", content = "dimension", rename_all = "snake_case")]
pub enum DomainVariationDimension {
    Mathematics(MathsVariationDimension),
    Physics(PhysicsVariationDimension),
    Chemistry(ChemistryVariationDimension),
    Reasoning(ReasoningVariationDimension),
}

impl DomainVariationDimension {
    pub fn domain(&self) -> Domain {
        match self {
            DomainVariationDimension::Mathematics(_) => Domain::Mathematics,
            DomainVariationDimension::Physics(_) => Domain::Physics,
            DomainVariationDimension::Chemistry(_) => Domain::Chemistry,
            DomainVariationDimension::Reasoning(_) => Domain::Reasoning,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            DomainVariationDimension::Mathematics(d) => match d {
                MathsVariationDimension::Parameter => "parameter",
                MathsVariationDimension::Reverse => "reverse",
                MathsVariationDimension::Structural => "structural",
                MathsVariationDimension::BoundaryTrap => "boundary_trap",
                MathsVariationDimension::MultiConcept => "multi_concept",
                MathsVariationDimension::Transfer => "transfer",
            },
            DomainVariationDimension::Physics(d) => match d {
                PhysicsVariationDimension::Parameter => "parameter",
                PhysicsVariationDimension::InitialCondition => "initial_condition",
                PhysicsVariationDimension::Representation => "representation",
                PhysicsVariationDimension::ModelSelection => "model_selection",
                PhysicsVariationDimension::Structural => "structural",
                PhysicsVariationDimension::Transfer => "transfer",
            },
            DomainVariationDimension::Chemistry(d) => match d {
                ChemistryVariationDimension::Quantity => "quantity",
                ChemistryVariationDimension::Species => "species",
                ChemistryVariationDimension::Regime => "regime",
                ChemistryVariationDimension::Constraint => "constraint",
                ChemistryVariationDimension::Structural => "structural",
                ChemistryVariationDimension::Transfer => "transfer",
            },
            DomainVariationDimension::Reasoning(d) => match d {
                ReasoningVariationDimension::Entity => "entity",
                ReasoningVariationDimension::Constraint => "constraint",
                ReasoningVariationDimension::Strategy => "strategy",
                ReasoningVariationDimension::Structure => "structure",
                ReasoningVariationDimension::Search => "search",
                ReasoningVariationDimension::Transfer => "transfer",
            },
        }
    }
}

/// Categorical structural distance between a problem variant and standard baseline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VariationDistance {
    /// Exact surface and parameter match.
    Exact = 0,
    /// Surface parameter variations with identical solution graph topology.
    Near = 1,
    /// Meaningful change in solution graph nodes, algebraic inversion, or additional step.
    Structural = 2,
    /// Same underlying domain skill applied in an alternate physical/conceptual context.
    Contextual = 3,
    /// Interaction of 2+ discrete skill schemas requiring composite decision points.
    MultiConcept = 4,
    /// Novel structure requiring deep schema generalization.
    Far = 5,
}

impl Default for VariationDistance {
    fn default() -> Self {
        VariationDistance::Near
    }
}

impl VariationDistance {
    pub fn as_str(&self) -> &'static str {
        match self {
            VariationDistance::Exact => "exact",
            VariationDistance::Near => "near",
            VariationDistance::Structural => "structural",
            VariationDistance::Contextual => "contextual",
            VariationDistance::MultiConcept => "multi_concept",
            VariationDistance::Far => "far",
        }
    }

    /// Computes the categorical variation distance from variation dimension and node count delta.
    pub fn from_dimension(dimension: &DomainVariationDimension, node_count_diff: usize) -> Self {
        match dimension {
            DomainVariationDimension::Mathematics(d) => match d {
                MathsVariationDimension::Parameter => {
                    if node_count_diff == 0 {
                        VariationDistance::Near
                    } else {
                        VariationDistance::Structural
                    }
                }
                MathsVariationDimension::Reverse => VariationDistance::Structural,
                MathsVariationDimension::Structural => VariationDistance::Structural,
                MathsVariationDimension::BoundaryTrap => VariationDistance::Contextual,
                MathsVariationDimension::MultiConcept => VariationDistance::MultiConcept,
                MathsVariationDimension::Transfer => VariationDistance::Far,
            },
            DomainVariationDimension::Physics(d) => match d {
                PhysicsVariationDimension::Parameter => VariationDistance::Near,
                PhysicsVariationDimension::InitialCondition => VariationDistance::Near,
                PhysicsVariationDimension::Representation => VariationDistance::Contextual,
                PhysicsVariationDimension::ModelSelection => VariationDistance::Structural,
                PhysicsVariationDimension::Structural => VariationDistance::Structural,
                PhysicsVariationDimension::Transfer => VariationDistance::Far,
            },
            DomainVariationDimension::Chemistry(d) => match d {
                ChemistryVariationDimension::Quantity => VariationDistance::Near,
                ChemistryVariationDimension::Species => VariationDistance::Near,
                ChemistryVariationDimension::Regime => VariationDistance::Structural,
                ChemistryVariationDimension::Constraint => VariationDistance::Contextual,
                ChemistryVariationDimension::Structural => VariationDistance::Structural,
                ChemistryVariationDimension::Transfer => VariationDistance::Far,
            },
            DomainVariationDimension::Reasoning(d) => match d {
                ReasoningVariationDimension::Entity => VariationDistance::Near,
                ReasoningVariationDimension::Constraint => VariationDistance::Structural,
                ReasoningVariationDimension::Strategy => VariationDistance::Structural,
                ReasoningVariationDimension::Structure => VariationDistance::Contextual,
                ReasoningVariationDimension::Search => VariationDistance::MultiConcept,
                ReasoningVariationDimension::Transfer => VariationDistance::Far,
            },
        }
    }
}
