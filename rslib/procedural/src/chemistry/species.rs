// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::fmt;
use serde::{Deserialize, Serialize};

/// Physical state of matter of a chemical entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StateOfMatter {
    Solid,
    Liquid,
    Gas,
    Aqueous,
}

impl StateOfMatter {
    pub fn symbol(&self) -> &'static str {
        match self {
            StateOfMatter::Solid => "(s)",
            StateOfMatter::Liquid => "(l)",
            StateOfMatter::Gas => "(g)",
            StateOfMatter::Aqueous => "(aq)",
        }
    }
}

impl fmt::Display for StateOfMatter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.symbol())
    }
}

/// Explicit chemical species descriptor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChemicalSpecies {
    pub formula: String,
    pub name: String,
    /// Molar mass in grams per mole (g/mol)
    pub molar_mass: f64,
    pub state: StateOfMatter,
    pub charge: i8,
}

impl ChemicalSpecies {
    pub fn new(
        formula: impl Into<String>,
        name: impl Into<String>,
        molar_mass: f64,
        state: StateOfMatter,
        charge: i8,
    ) -> Self {
        Self {
            formula: formula.into(),
            name: name.into(),
            molar_mass,
            state,
            charge,
        }
    }

    pub fn formatted_formula(&self) -> String {
        format!("{}{}", self.formula, self.state.symbol())
    }
}

/// Standard verified catalog of chemical compounds for deterministic generation.
pub struct SpeciesCatalog;

impl SpeciesCatalog {
    pub fn water() -> ChemicalSpecies {
        ChemicalSpecies::new("H2O", "Water", 18.015, StateOfMatter::Liquid, 0)
    }

    pub fn carbon_dioxide() -> ChemicalSpecies {
        ChemicalSpecies::new("CO2", "Carbon Dioxide", 44.01, StateOfMatter::Gas, 0)
    }

    pub fn methane() -> ChemicalSpecies {
        ChemicalSpecies::new("CH4", "Methane", 16.04, StateOfMatter::Gas, 0)
    }

    pub fn oxygen() -> ChemicalSpecies {
        ChemicalSpecies::new("O2", "Oxygen Gas", 32.00, StateOfMatter::Gas, 0)
    }

    pub fn nitrogen() -> ChemicalSpecies {
        ChemicalSpecies::new("N2", "Nitrogen Gas", 28.014, StateOfMatter::Gas, 0)
    }

    pub fn hydrogen() -> ChemicalSpecies {
        ChemicalSpecies::new("H2", "Hydrogen Gas", 2.016, StateOfMatter::Gas, 0)
    }

    pub fn ammonia() -> ChemicalSpecies {
        ChemicalSpecies::new("NH3", "Ammonia", 17.031, StateOfMatter::Gas, 0)
    }

    pub fn hydrochloric_acid() -> ChemicalSpecies {
        ChemicalSpecies::new("HCl", "Hydrochloric Acid", 36.46, StateOfMatter::Aqueous, 0)
    }

    pub fn sodium_hydroxide() -> ChemicalSpecies {
        ChemicalSpecies::new("NaOH", "Sodium Hydroxide", 39.997, StateOfMatter::Aqueous, 0)
    }

    pub fn sodium_chloride() -> ChemicalSpecies {
        ChemicalSpecies::new("NaCl", "Sodium Chloride", 58.44, StateOfMatter::Aqueous, 0)
    }

    pub fn calcium_carbonate() -> ChemicalSpecies {
        ChemicalSpecies::new("CaCO3", "Calcium Carbonate", 100.086, StateOfMatter::Solid, 0)
    }

    pub fn calcium_oxide() -> ChemicalSpecies {
        ChemicalSpecies::new("CaO", "Calcium Oxide", 56.077, StateOfMatter::Solid, 0)
    }

    pub fn aluminum() -> ChemicalSpecies {
        ChemicalSpecies::new("Al", "Aluminum", 26.982, StateOfMatter::Solid, 0)
    }

    pub fn aluminum_oxide() -> ChemicalSpecies {
        ChemicalSpecies::new("Al2O3", "Aluminum Oxide", 101.96, StateOfMatter::Solid, 0)
    }

    pub fn iron_iii_oxide() -> ChemicalSpecies {
        ChemicalSpecies::new("Fe2O3", "Iron(III) Oxide", 159.69, StateOfMatter::Solid, 0)
    }

    pub fn iron() -> ChemicalSpecies {
        ChemicalSpecies::new("Fe", "Iron", 55.845, StateOfMatter::Solid, 0)
    }

    pub fn sulfur_dioxide() -> ChemicalSpecies {
        ChemicalSpecies::new("SO2", "Sulfur Dioxide", 64.066, StateOfMatter::Gas, 0)
    }

    pub fn sulfur_trioxide() -> ChemicalSpecies {
        ChemicalSpecies::new("SO3", "Sulfur Trioxide", 80.066, StateOfMatter::Gas, 0)
    }

    pub fn nitrogen_dioxide() -> ChemicalSpecies {
        ChemicalSpecies::new("NO2", "Nitrogen Dioxide", 46.006, StateOfMatter::Gas, 0)
    }

    pub fn dinitrogen_tetroxide() -> ChemicalSpecies {
        ChemicalSpecies::new("N2O4", "Dinitrogen Tetroxide", 92.011, StateOfMatter::Gas, 0)
    }

    pub fn phosphorus_pentachloride() -> ChemicalSpecies {
        ChemicalSpecies::new("PCl5", "Phosphorus Pentachloride", 208.24, StateOfMatter::Gas, 0)
    }

    pub fn phosphorus_trichloride() -> ChemicalSpecies {
        ChemicalSpecies::new("PCl3", "Phosphorus Trichloride", 137.33, StateOfMatter::Gas, 0)
    }

    pub fn chlorine() -> ChemicalSpecies {
        ChemicalSpecies::new("Cl2", "Chlorine Gas", 70.90, StateOfMatter::Gas, 0)
    }

    pub fn hydrogen_iodide() -> ChemicalSpecies {
        ChemicalSpecies::new("HI", "Hydrogen Iodide", 127.91, StateOfMatter::Gas, 0)
    }

    pub fn iodine() -> ChemicalSpecies {
        ChemicalSpecies::new("I2", "Iodine", 253.808, StateOfMatter::Gas, 0)
    }

    /// Lookup species by chemical formula or common name.
    pub fn find(identifier: &str) -> Option<ChemicalSpecies> {
        let norm = identifier.trim().to_uppercase();
        match norm.as_str() {
            "H2O" | "WATER" => Some(Self::water()),
            "CO2" | "CARBON DIOXIDE" => Some(Self::carbon_dioxide()),
            "CH4" | "METHANE" => Some(Self::methane()),
            "O2" | "OXYGEN" => Some(Self::oxygen()),
            "N2" | "NITROGEN" => Some(Self::nitrogen()),
            "H2" | "HYDROGEN" => Some(Self::hydrogen()),
            "NH3" | "AMMONIA" => Some(Self::ammonia()),
            "HCL" | "HYDROCHLORIC ACID" => Some(Self::hydrochloric_acid()),
            "NAOH" | "SODIUM HYDROXIDE" => Some(Self::sodium_hydroxide()),
            "NACL" | "SODIUM CHLORIDE" => Some(Self::sodium_chloride()),
            "CACO3" | "CALCIUM CARBONATE" => Some(Self::calcium_carbonate()),
            "CAO" | "CALCIUM OXIDE" => Some(Self::calcium_oxide()),
            "AL" | "ALUMINUM" => Some(Self::aluminum()),
            "AL2O3" | "ALUMINUM OXIDE" => Some(Self::aluminum_oxide()),
            "FE2O3" | "IRON(III) OXIDE" => Some(Self::iron_iii_oxide()),
            "FE" | "IRON" => Some(Self::iron()),
            "SO2" | "SULFUR DIOXIDE" => Some(Self::sulfur_dioxide()),
            "SO3" | "SULFUR TRIOXIDE" => Some(Self::sulfur_trioxide()),
            "NO2" | "NITROGEN DIOXIDE" => Some(Self::nitrogen_dioxide()),
            "N2O4" | "DINITROGEN TETROXIDE" => Some(Self::dinitrogen_tetroxide()),
            "PCL5" | "PHOSPHORUS PENTACHLORIDE" => Some(Self::phosphorus_pentachloride()),
            "PCL3" | "PHOSPHORUS TRICHLORIDE" => Some(Self::phosphorus_trichloride()),
            "CL2" | "CHLORINE" => Some(Self::chlorine()),
            "HI" | "HYDROGEN IODIDE" => Some(Self::hydrogen_iodide()),
            "I2" | "IODINE" => Some(Self::iodine()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_species_catalog_lookup() {
        let h2o = SpeciesCatalog::find("H2O").unwrap();
        assert_eq!(h2o.name, "Water");
        assert!((h2o.molar_mass - 18.015).abs() < 1e-3);
        assert_eq!(h2o.state, StateOfMatter::Liquid);

        let nh3 = SpeciesCatalog::find("Ammonia").unwrap();
        assert_eq!(nh3.formula, "NH3");
        assert_eq!(nh3.state, StateOfMatter::Gas);
    }
}
