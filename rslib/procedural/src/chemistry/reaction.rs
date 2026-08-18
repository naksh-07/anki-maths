// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::fmt;
use serde::{Deserialize, Serialize};

use super::species::{ChemicalSpecies, SpeciesCatalog};

/// A participant species with its stoichiometric integer coefficient.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReactionParticipant {
    pub species: ChemicalSpecies,
    pub coefficient: u32,
}

impl ReactionParticipant {
    pub fn new(species: ChemicalSpecies, coefficient: u32) -> Self {
        Self { species, coefficient }
    }

    pub fn formatted(&self) -> String {
        if self.coefficient == 1 {
            self.species.formatted_formula()
        } else {
            format!("{} {}", self.coefficient, self.species.formatted_formula())
        }
    }
}

/// A balanced chemical reaction with explicit stoichiometric coefficients.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChemicalReaction {
    pub name: String,
    pub reactants: Vec<ReactionParticipant>,
    pub products: Vec<ReactionParticipant>,
    pub is_reversible: bool,
}

impl ChemicalReaction {
    pub fn new(
        name: impl Into<String>,
        reactants: Vec<ReactionParticipant>,
        products: Vec<ReactionParticipant>,
        is_reversible: bool,
    ) -> Self {
        Self {
            name: name.into(),
            reactants,
            products,
            is_reversible,
        }
    }

    pub fn formatted_equation(&self) -> String {
        let r_str = self
            .reactants
            .iter()
            .map(|p| p.formatted())
            .collect::<Vec<_>>()
            .join(" + ");
        let p_str = self
            .products
            .iter()
            .map(|p| p.formatted())
            .collect::<Vec<_>>()
            .join(" + ");
        let arrow = if self.is_reversible { "⇌" } else { "→" };
        format!("{} {} {}", r_str, arrow, p_str)
    }

    pub fn coefficient_of(&self, formula: &str) -> Option<u32> {
        let norm = formula.trim().to_uppercase();
        for r in &self.reactants {
            if r.species.formula.to_uppercase() == norm {
                return Some(r.coefficient);
            }
        }
        for p in &self.products {
            if p.species.formula.to_uppercase() == norm {
                return Some(p.coefficient);
            }
        }
        None
    }

    /// Stoichiometric ratio: (coefficient of target) / (coefficient of source)
    pub fn stoichiometric_ratio(&self, source_formula: &str, target_formula: &str) -> Option<f64> {
        let source_coeff = self.coefficient_of(source_formula)? as f64;
        let target_coeff = self.coefficient_of(target_formula)? as f64;
        if source_coeff == 0.0 {
            None
        } else {
            Some(target_coeff / source_coeff)
        }
    }

    pub fn is_reactant(&self, formula: &str) -> bool {
        let norm = formula.trim().to_uppercase();
        self.reactants.iter().any(|r| r.species.formula.to_uppercase() == norm)
    }

    pub fn is_product(&self, formula: &str) -> bool {
        let norm = formula.trim().to_uppercase();
        self.products.iter().any(|p| p.species.formula.to_uppercase() == norm)
    }
}

impl fmt::Display for ChemicalReaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.formatted_equation())
    }
}

/// Standard verified reaction templates for deterministic problem generation.
pub struct ReactionTemplates;

impl ReactionTemplates {
    /// N2(g) + 3 H2(g) ⇌ 2 NH3(g)
    pub fn haber_bosch() -> ChemicalReaction {
        ChemicalReaction::new(
            "Haber-Bosch Ammonia Synthesis",
            vec![
                ReactionParticipant::new(SpeciesCatalog::nitrogen(), 1),
                ReactionParticipant::new(SpeciesCatalog::hydrogen(), 3),
            ],
            vec![ReactionParticipant::new(SpeciesCatalog::ammonia(), 2)],
            true,
        )
    }

    /// CH4(g) + 2 O2(g) → CO2(g) + 2 H2O(l)
    pub fn methane_combustion() -> ChemicalReaction {
        ChemicalReaction::new(
            "Methane Combustion",
            vec![
                ReactionParticipant::new(SpeciesCatalog::methane(), 1),
                ReactionParticipant::new(SpeciesCatalog::oxygen(), 2),
            ],
            vec![
                ReactionParticipant::new(SpeciesCatalog::carbon_dioxide(), 1),
                ReactionParticipant::new(SpeciesCatalog::water(), 2),
            ],
            false,
        )
    }

    /// CaCO3(s) → CaO(s) + CO2(g)
    pub fn calcium_carbonate_calcination() -> ChemicalReaction {
        ChemicalReaction::new(
            "Thermal Decomposition of Calcium Carbonate",
            vec![ReactionParticipant::new(SpeciesCatalog::calcium_carbonate(), 1)],
            vec![
                ReactionParticipant::new(SpeciesCatalog::calcium_oxide(), 1),
                ReactionParticipant::new(SpeciesCatalog::carbon_dioxide(), 1),
            ],
            false,
        )
    }

    /// 4 Al(s) + 3 O2(g) → 2 Al2O3(s)
    pub fn aluminum_oxidation() -> ChemicalReaction {
        ChemicalReaction::new(
            "Aluminum Oxidation",
            vec![
                ReactionParticipant::new(SpeciesCatalog::aluminum(), 4),
                ReactionParticipant::new(SpeciesCatalog::oxygen(), 3),
            ],
            vec![ReactionParticipant::new(SpeciesCatalog::aluminum_oxide(), 2)],
            false,
        )
    }

    /// 2 Al(s) + Fe2O3(s) → Al2O3(s) + 2 Fe(s)
    pub fn thermite_reaction() -> ChemicalReaction {
        ChemicalReaction::new(
            "Thermite Reaction",
            vec![
                ReactionParticipant::new(SpeciesCatalog::aluminum(), 2),
                ReactionParticipant::new(SpeciesCatalog::iron_iii_oxide(), 1),
            ],
            vec![
                ReactionParticipant::new(SpeciesCatalog::aluminum_oxide(), 1),
                ReactionParticipant::new(SpeciesCatalog::iron(), 2),
            ],
            false,
        )
    }

    /// 2 SO2(g) + O2(g) ⇌ 2 SO3(g)
    pub fn sulfur_trioxide_equilibrium() -> ChemicalReaction {
        ChemicalReaction::new(
            "Sulfur Trioxide Equilibrium",
            vec![
                ReactionParticipant::new(SpeciesCatalog::sulfur_dioxide(), 2),
                ReactionParticipant::new(SpeciesCatalog::oxygen(), 1),
            ],
            vec![ReactionParticipant::new(SpeciesCatalog::sulfur_trioxide(), 2)],
            true,
        )
    }

    /// 2 NO2(g) ⇌ N2O4(g)
    pub fn no2_dimerization() -> ChemicalReaction {
        ChemicalReaction::new(
            "Nitrogen Dioxide Dimerization",
            vec![ReactionParticipant::new(SpeciesCatalog::nitrogen_dioxide(), 2)],
            vec![ReactionParticipant::new(SpeciesCatalog::dinitrogen_tetroxide(), 1)],
            true,
        )
    }

    /// PCl5(g) ⇌ PCl3(g) + Cl2(g)
    pub fn pcl5_decomposition() -> ChemicalReaction {
        ChemicalReaction::new(
            "Phosphorus Pentachloride Dissociation",
            vec![ReactionParticipant::new(SpeciesCatalog::phosphorus_pentachloride(), 1)],
            vec![
                ReactionParticipant::new(SpeciesCatalog::phosphorus_trichloride(), 1),
                ReactionParticipant::new(SpeciesCatalog::chlorine(), 1),
            ],
            true,
        )
    }

    /// H2(g) + I2(g) ⇌ 2 HI(g)
    pub fn hydrogen_iodide_equilibrium() -> ChemicalReaction {
        ChemicalReaction::new(
            "Hydrogen Iodide Equilibrium",
            vec![
                ReactionParticipant::new(SpeciesCatalog::hydrogen(), 1),
                ReactionParticipant::new(SpeciesCatalog::iodine(), 1),
            ],
            vec![ReactionParticipant::new(SpeciesCatalog::hydrogen_iodide(), 2)],
            true,
        )
    }

    /// HCl(aq) + NaOH(aq) → NaCl(aq) + H2O(l)
    pub fn acid_base_neutralization() -> ChemicalReaction {
        ChemicalReaction::new(
            "Acid-Base Neutralization",
            vec![
                ReactionParticipant::new(SpeciesCatalog::hydrochloric_acid(), 1),
                ReactionParticipant::new(SpeciesCatalog::sodium_hydroxide(), 1),
            ],
            vec![
                ReactionParticipant::new(SpeciesCatalog::sodium_chloride(), 1),
                ReactionParticipant::new(SpeciesCatalog::water(), 1),
            ],
            false,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chemical_reaction_stoichiometric_ratios() {
        let haber = ReactionTemplates::haber_bosch();
        assert_eq!(haber.formatted_equation(), "N2(g) + 3 H2(g) ⇌ 2 NH3(g)");

        // 1 mol N2 produces 2 mol NH3
        assert_eq!(haber.stoichiometric_ratio("N2", "NH3"), Some(2.0));
        // 3 mol H2 produces 2 mol NH3
        assert!((haber.stoichiometric_ratio("H2", "NH3").unwrap() - 2.0 / 3.0).abs() < 1e-6);
        // Ratio of H2 to N2 required = 3 / 1 = 3
        assert_eq!(haber.stoichiometric_ratio("N2", "H2"), Some(3.0));
    }

    #[test]
    fn test_methane_combustion_participants() {
        let rxn = ReactionTemplates::methane_combustion();
        assert!(rxn.is_reactant("CH4"));
        assert!(rxn.is_reactant("O2"));
        assert!(rxn.is_product("CO2"));
        assert!(rxn.is_product("H2O"));
        assert_eq!(rxn.coefficient_of("O2"), Some(2));
        assert_eq!(rxn.coefficient_of("CO2"), Some(1));
    }
}
