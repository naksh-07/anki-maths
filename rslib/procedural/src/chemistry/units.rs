// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::fmt;
use std::str::FromStr;
use serde::{Deserialize, Serialize};

/// Fundamental dimensional vector for chemical and physical units:
/// [Mass]^m · [Length]^l · [Time]^t · [AmountOfSubstance]^n · [Temperature]^k
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChemicalDimension {
    pub mass: i8,
    pub length: i8,
    pub time: i8,
    pub amount: i8,
    pub temperature: i8,
}

impl ChemicalDimension {
    pub const DIMENSIONLESS: Self = Self { mass: 0, length: 0, time: 0, amount: 0, temperature: 0 };
    pub const MASS: Self = Self { mass: 1, length: 0, time: 0, amount: 0, temperature: 0 };
    pub const AMOUNT: Self = Self { mass: 0, length: 0, time: 0, amount: 1, temperature: 0 };
    pub const VOLUME: Self = Self { mass: 0, length: 3, time: 0, amount: 0, temperature: 0 };
    pub const CONCENTRATION: Self = Self { mass: 0, length: -3, time: 0, amount: 1, temperature: 0 }; // mol/m^3 or mol/L
    pub const MOLAR_MASS: Self = Self { mass: 1, length: 0, time: 0, amount: -1, temperature: 0 };    // kg/mol or g/mol
    pub const ENERGY: Self = Self { mass: 1, length: 2, time: -2, amount: 0, temperature: 0 };        // J
    pub const MOLAR_ENERGY: Self = Self { mass: 1, length: 2, time: -2, amount: -1, temperature: 0 }; // J/mol or kJ/mol

    pub fn is_compatible_with(&self, other: &Self) -> bool {
        self == other
    }
}

/// Supported Chemistry units with exact base conversion scales.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChemistryUnit {
    // Mass
    Gram,
    Kilogram,
    Milligram,
    // Amount of substance
    Mole,
    Millimole,
    // Volume
    Liter,
    Milliliter,
    // Concentration
    Molar,       // mol/L
    Millimolar,  // mmol/L
    // Molar Mass
    GramPerMole,
    // Energy
    Joule,
    Kilojoule,
    KilojoulePerMole,
    // Dimensionless
    Dimensionless,
    Percent,
}

impl ChemistryUnit {
    pub fn dimension(&self) -> ChemicalDimension {
        match self {
            ChemistryUnit::Gram | ChemistryUnit::Kilogram | ChemistryUnit::Milligram => {
                ChemicalDimension::MASS
            }
            ChemistryUnit::Mole | ChemistryUnit::Millimole => ChemicalDimension::AMOUNT,
            ChemistryUnit::Liter | ChemistryUnit::Milliliter => ChemicalDimension::VOLUME,
            ChemistryUnit::Molar | ChemistryUnit::Millimolar => ChemicalDimension::CONCENTRATION,
            ChemistryUnit::GramPerMole => ChemicalDimension::MOLAR_MASS,
            ChemistryUnit::Joule | ChemistryUnit::Kilojoule => ChemicalDimension::ENERGY,
            ChemistryUnit::KilojoulePerMole => ChemicalDimension::MOLAR_ENERGY,
            ChemistryUnit::Dimensionless | ChemistryUnit::Percent => ChemicalDimension::DIMENSIONLESS,
        }
    }

    /// Factor to multiply a numeric value by to convert to canonical base unit (g, mol, L, M, J).
    pub fn to_base_scale(&self) -> f64 {
        match self {
            ChemistryUnit::Gram => 1.0,
            ChemistryUnit::Kilogram => 1000.0,
            ChemistryUnit::Milligram => 0.001,
            ChemistryUnit::Mole => 1.0,
            ChemistryUnit::Millimole => 0.001,
            ChemistryUnit::Liter => 1.0,
            ChemistryUnit::Milliliter => 0.001,
            ChemistryUnit::Molar => 1.0,
            ChemistryUnit::Millimolar => 0.001,
            ChemistryUnit::GramPerMole => 1.0,
            ChemistryUnit::Joule => 1.0,
            ChemistryUnit::Kilojoule => 1000.0,
            ChemistryUnit::KilojoulePerMole => 1000.0,
            ChemistryUnit::Dimensionless => 1.0,
            ChemistryUnit::Percent => 0.01,
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            ChemistryUnit::Gram => "g",
            ChemistryUnit::Kilogram => "kg",
            ChemistryUnit::Milligram => "mg",
            ChemistryUnit::Mole => "mol",
            ChemistryUnit::Millimole => "mmol",
            ChemistryUnit::Liter => "L",
            ChemistryUnit::Milliliter => "mL",
            ChemistryUnit::Molar => "M",
            ChemistryUnit::Millimolar => "mM",
            ChemistryUnit::GramPerMole => "g/mol",
            ChemistryUnit::Joule => "J",
            ChemistryUnit::Kilojoule => "kJ",
            ChemistryUnit::KilojoulePerMole => "kJ/mol",
            ChemistryUnit::Dimensionless => "",
            ChemistryUnit::Percent => "%",
        }
    }
}

impl fmt::Display for ChemistryUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.symbol())
    }
}

impl FromStr for ChemistryUnit {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let norm = s.trim().to_lowercase();
        match norm.as_str() {
            "g" | "gram" | "grams" => Ok(ChemistryUnit::Gram),
            "kg" | "kilogram" | "kilograms" => Ok(ChemistryUnit::Kilogram),
            "mg" | "milligram" => Ok(ChemistryUnit::Milligram),
            "mol" | "mole" | "moles" => Ok(ChemistryUnit::Mole),
            "mmol" | "millimole" | "millimoles" => Ok(ChemistryUnit::Millimole),
            "l" | "liter" | "liters" | "litre" | "dm3" | "dm^3" => Ok(ChemistryUnit::Liter),
            "ml" | "milliliter" | "milliliters" | "cm3" | "cm^3" => Ok(ChemistryUnit::Milliliter),
            "m" | "molar" | "mol/l" | "mol/dm3" | "mol*l^-1" => Ok(ChemistryUnit::Molar),
            "mm" | "millimolar" | "mmol/l" => Ok(ChemistryUnit::Millimolar),
            "g/mol" | "g*mol^-1" => Ok(ChemistryUnit::GramPerMole),
            "j" | "joule" | "joules" => Ok(ChemistryUnit::Joule),
            "kj" | "kilojoule" | "kilojoules" => Ok(ChemistryUnit::Kilojoule),
            "kj/mol" | "kj*mol^-1" => Ok(ChemistryUnit::KilojoulePerMole),
            "" | "dimensionless" | "ratio" => Ok(ChemistryUnit::Dimensionless),
            "%" | "percent" | "percentage" => Ok(ChemistryUnit::Percent),
            _ => Err(format!("Unrecognized chemistry unit: {}", s)),
        }
    }
}

/// Helper for dimensional analysis and unit conversions in Chemistry problems.
pub struct ChemicalDimensionalValidator;

impl ChemicalDimensionalValidator {
    pub const TOLERANCE: f64 = 1e-4;

    /// Convert a value from `from_unit` to `to_unit`. Returns Err if units are dimensionally incompatible.
    pub fn convert(val: f64, from_unit: ChemistryUnit, to_unit: ChemistryUnit) -> Result<f64, String> {
        if !from_unit.dimension().is_compatible_with(&to_unit.dimension()) {
            return Err(format!(
                "Incompatible chemical units: {} ({:?}) cannot be converted to {} ({:?})",
                from_unit,
                from_unit.dimension(),
                to_unit,
                to_unit.dimension()
            ));
        }

        let base_val = val * from_unit.to_base_scale();
        Ok(base_val / to_unit.to_base_scale())
    }

    /// Check if two unit-attached values are equivalent after conversion to base units.
    pub fn is_equivalent(val_a: f64, unit_a: ChemistryUnit, val_b: f64, unit_b: ChemistryUnit) -> bool {
        if let Ok(converted_b) = Self::convert(val_b, unit_b, unit_a) {
            (val_a - converted_b).abs() <= Self::TOLERANCE.max(val_a.abs() * 0.01)
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chemistry_unit_conversions() {
        // Grams to Kilograms
        let kg = ChemicalDimensionalValidator::convert(2500.0, ChemistryUnit::Gram, ChemistryUnit::Kilogram).unwrap();
        assert!((kg - 2.5).abs() < 1e-6);

        // Milliliters to Liters
        let l = ChemicalDimensionalValidator::convert(750.0, ChemistryUnit::Milliliter, ChemistryUnit::Liter).unwrap();
        assert!((l - 0.75).abs() < 1e-6);

        // Moles to Millimoles
        let mmol = ChemicalDimensionalValidator::convert(0.045, ChemistryUnit::Mole, ChemistryUnit::Millimole).unwrap();
        assert!((mmol - 45.0).abs() < 1e-6);

        // Incompatible conversion rejected
        let err = ChemicalDimensionalValidator::convert(10.0, ChemistryUnit::Gram, ChemistryUnit::Mole);
        assert!(err.is_err());
    }

    #[test]
    fn test_chemistry_unit_parsing() {
        assert_eq!("g".parse::<ChemistryUnit>().unwrap(), ChemistryUnit::Gram);
        assert_eq!("mol".parse::<ChemistryUnit>().unwrap(), ChemistryUnit::Mole);
        assert_eq!("mL".parse::<ChemistryUnit>().unwrap(), ChemistryUnit::Milliliter);
        assert_eq!("M".parse::<ChemistryUnit>().unwrap(), ChemistryUnit::Molar);
        assert_eq!("kJ/mol".parse::<ChemistryUnit>().unwrap(), ChemistryUnit::KilojoulePerMole);
    }
}
