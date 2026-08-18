// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use serde::{Deserialize, Serialize};

use super::reaction::ChemicalReaction;
use super::units::ChemistryUnit;

/// Discrete classification of chemical regime and problem nature.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChemicalRegimeKind {
    /// Direct conversion between mass and moles via molar mass: n = m / M
    StoichiometryMoleConversion,
    /// Mole-to-mole stoichiometry via reaction coefficients: n_B = n_A * (b / a)
    StoichiometryReactionRatio,
    /// Mass-to-mass quantitative stoichiometry: m_A -> n_A -> n_B -> m_B
    StoichiometryMassMass,
    /// Limiting reagent determination and theoretical product calculation
    StoichiometryLimitingReagent,
    /// Experimental vs theoretical percentage yield calculation
    StoichiometryPercentageYield,
    /// Direct solution concentration / molarity calculation: M = n / V
    ConcentrationMolarity,
    /// Formulating equilibrium constant expression: Kc = [Products]^p / [Reactants]^r
    EquilibriumConstantExpression,
    /// ICE (Initial, Change, Equilibrium) table analysis
    EquilibriumIceTable,
    /// Solving equilibrium concentration from Kc via quadratic or algebraic solver
    EquilibriumQuadraticCalculation,
    /// Le Chatelier principle response / Reaction quotient Qc comparison
    EquilibriumLeChatelier,
}

impl ChemicalRegimeKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ChemicalRegimeKind::StoichiometryMoleConversion => "stoichiometry_mole_conversion",
            ChemicalRegimeKind::StoichiometryReactionRatio => "stoichiometry_reaction_ratio",
            ChemicalRegimeKind::StoichiometryMassMass => "stoichiometry_mass_mass",
            ChemicalRegimeKind::StoichiometryLimitingReagent => "stoichiometry_limiting_reagent",
            ChemicalRegimeKind::StoichiometryPercentageYield => "stoichiometry_percentage_yield",
            ChemicalRegimeKind::ConcentrationMolarity => "concentration_molarity",
            ChemicalRegimeKind::EquilibriumConstantExpression => "equilibrium_constant_expression",
            ChemicalRegimeKind::EquilibriumIceTable => "equilibrium_ice_table",
            ChemicalRegimeKind::EquilibriumQuadraticCalculation => "equilibrium_quadratic_calculation",
            ChemicalRegimeKind::EquilibriumLeChatelier => "equilibrium_le_chatelier",
        }
    }
}

/// A structured quantity associated with a chemical species.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChemicalQuantity {
    pub symbol: String,
    pub species_formula: String,
    pub value: f64,
    pub unit: ChemistryUnit,
}

impl ChemicalQuantity {
    pub fn new(
        symbol: impl Into<String>,
        species_formula: impl Into<String>,
        value: f64,
        unit: ChemistryUnit,
    ) -> Self {
        Self {
            symbol: symbol.into(),
            species_formula: species_formula.into(),
            value,
            unit,
        }
    }

    pub fn formatted(&self) -> String {
        format!("{:.3} {}", self.value, self.unit)
    }
}

/// Rich structured metadata for Chemistry problem instances.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChemicalProblemMetadata {
    pub regime: ChemicalRegimeKind,
    pub reaction: Option<ChemicalReaction>,
    pub target_species: Option<String>,
    pub initial_quantities: Vec<ChemicalQuantity>,
    pub limiting_reagent: Option<String>,
    pub equilibrium_constant: Option<f64>,
    pub target_unit: ChemistryUnit,
}

impl ChemicalProblemMetadata {
    pub fn new(regime: ChemicalRegimeKind, target_unit: ChemistryUnit) -> Self {
        Self {
            regime,
            reaction: None,
            target_species: None,
            initial_quantities: Vec::new(),
            limiting_reagent: None,
            equilibrium_constant: None,
            target_unit,
        }
    }

    pub fn with_reaction(mut self, reaction: ChemicalReaction) -> Self {
        self.reaction = Some(reaction);
        self
    }

    pub fn with_target_species(mut self, species: impl Into<String>) -> Self {
        self.target_species = Some(species.into());
        self
    }

    pub fn with_initial_quantity(mut self, qty: ChemicalQuantity) -> Self {
        self.initial_quantities.push(qty);
        self
    }

    pub fn with_limiting_reagent(mut self, reagent: impl Into<String>) -> Self {
        self.limiting_reagent = Some(reagent.into());
        self
    }

    pub fn with_equilibrium_constant(mut self, kc: f64) -> Self {
        self.equilibrium_constant = Some(kc);
        self
    }
}
