// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use serde::{Deserialize, Serialize};

/// Fundamental physical dimension represented as integer exponents: [M]^m [L]^l [T]^t
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PhysicalDimension {
    pub mass: i8,
    pub length: i8,
    pub time: i8,
}

impl PhysicalDimension {
    pub const DIMENSIONLESS: Self = Self { mass: 0, length: 0, time: 0 };
    pub const LENGTH: Self = Self { mass: 0, length: 1, time: 0 };
    pub const TIME: Self = Self { mass: 0, length: 0, time: 1 };
    pub const MASS: Self = Self { mass: 1, length: 0, time: 0 };
    pub const VELOCITY: Self = Self { mass: 0, length: 1, time: -1 };
    pub const ACCELERATION: Self = Self { mass: 0, length: 1, time: -2 };
    pub const FORCE: Self = Self { mass: 1, length: 1, time: -2 };
    pub const ENERGY: Self = Self { mass: 1, length: 2, time: -2 };
    pub const POWER: Self = Self { mass: 1, length: 2, time: -3 };

    pub fn multiply(self, other: Self) -> Self {
        Self {
            mass: self.mass + other.mass,
            length: self.length + other.length,
            time: self.time + other.time,
        }
    }

    pub fn divide(self, other: Self) -> Self {
        Self {
            mass: self.mass - other.mass,
            length: self.length - other.length,
            time: self.time - other.time,
        }
    }
}

/// Standard physical units supported in Physics Engine v1.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PhysicsUnit {
    // Base SI
    Meter,
    Second,
    Kilogram,
    // Derived SI
    MeterPerSecond,
    MeterPerSecondSquared,
    Newton,
    Joule,
    Watt,
    // Common non-SI units
    KilometerPerHour,
    Gram,
    Minute,
    Hour,
    Dimensionless,
}

impl PhysicsUnit {
    pub fn symbol(&self) -> &'static str {
        match self {
            PhysicsUnit::Meter => "m",
            PhysicsUnit::Second => "s",
            PhysicsUnit::Kilogram => "kg",
            PhysicsUnit::MeterPerSecond => "m/s",
            PhysicsUnit::MeterPerSecondSquared => "m/s²",
            PhysicsUnit::Newton => "N",
            PhysicsUnit::Joule => "J",
            PhysicsUnit::Watt => "W",
            PhysicsUnit::KilometerPerHour => "km/h",
            PhysicsUnit::Gram => "g",
            PhysicsUnit::Minute => "min",
            PhysicsUnit::Hour => "h",
            PhysicsUnit::Dimensionless => "",
        }
    }

    pub fn dimension(&self) -> PhysicalDimension {
        match self {
            PhysicsUnit::Meter => PhysicalDimension::LENGTH,
            PhysicsUnit::Second | PhysicsUnit::Minute | PhysicsUnit::Hour => PhysicalDimension::TIME,
            PhysicsUnit::Kilogram | PhysicsUnit::Gram => PhysicalDimension::MASS,
            PhysicsUnit::MeterPerSecond | PhysicsUnit::KilometerPerHour => PhysicalDimension::VELOCITY,
            PhysicsUnit::MeterPerSecondSquared => PhysicalDimension::ACCELERATION,
            PhysicsUnit::Newton => PhysicalDimension::FORCE,
            PhysicsUnit::Joule => PhysicalDimension::ENERGY,
            PhysicsUnit::Watt => PhysicalDimension::POWER,
            PhysicsUnit::Dimensionless => PhysicalDimension::DIMENSIONLESS,
        }
    }

    /// Scaling factor to convert a quantity expressed in this unit to standard base SI units.
    pub fn to_si_multiplier(&self) -> f64 {
        match self {
            PhysicsUnit::Meter | PhysicsUnit::Second | PhysicsUnit::Kilogram => 1.0,
            PhysicsUnit::MeterPerSecond | PhysicsUnit::MeterPerSecondSquared => 1.0,
            PhysicsUnit::Newton | PhysicsUnit::Joule | PhysicsUnit::Watt => 1.0,
            PhysicsUnit::KilometerPerHour => 5.0 / 18.0, // 1 km/h = 1000m / 3600s = 5/18 m/s
            PhysicsUnit::Gram => 0.001,
            PhysicsUnit::Minute => 60.0,
            PhysicsUnit::Hour => 3600.0,
            PhysicsUnit::Dimensionless => 1.0,
        }
    }
}

/// Deterministic dimensional compatibility and unit conversion validator.
#[derive(Debug, Clone, Default)]
pub struct DimensionalValidator;

impl DimensionalValidator {
    /// Parse unit from common string representations.
    pub fn parse_unit(s: &str) -> Option<PhysicsUnit> {
        let trimmed = s.trim().to_lowercase();
        match trimmed.as_str() {
            "m" | "meter" | "meters" | "metre" | "metres" => Some(PhysicsUnit::Meter),
            "s" | "sec" | "second" | "seconds" => Some(PhysicsUnit::Second),
            "kg" | "kilogram" | "kilograms" => Some(PhysicsUnit::Kilogram),
            "g" | "gram" | "grams" => Some(PhysicsUnit::Gram),
            "m/s" | "mps" | "m s^-1" | "meter/second" => Some(PhysicsUnit::MeterPerSecond),
            "m/s^2" | "m/s2" | "m/s²" | "m s^-2" => Some(PhysicsUnit::MeterPerSecondSquared),
            "km/h" | "kmph" | "km/hr" | "kph" => Some(PhysicsUnit::KilometerPerHour),
            "n" | "newton" | "newtons" => Some(PhysicsUnit::Newton),
            "j" | "joule" | "joules" => Some(PhysicsUnit::Joule),
            "w" | "watt" | "watts" => Some(PhysicsUnit::Watt),
            "min" | "minute" | "minutes" => Some(PhysicsUnit::Minute),
            "h" | "hr" | "hour" | "hours" => Some(PhysicsUnit::Hour),
            "" | "none" | "1" => Some(PhysicsUnit::Dimensionless),
            _ => None,
        }
    }

    /// Check if two units are dimensionally compatible.
    pub fn are_compatible(u1: &PhysicsUnit, u2: &PhysicsUnit) -> bool {
        u1.dimension() == u2.dimension()
    }

    /// Convert a numerical value from `source_unit` to standard SI units.
    pub fn convert_to_si(val: f64, source_unit: &PhysicsUnit) -> f64 {
        val * source_unit.to_si_multiplier()
    }

    /// Convert a numerical value from standard SI units to `target_unit`.
    pub fn convert_from_si(val_si: f64, target_unit: &PhysicsUnit) -> f64 {
        let mult = target_unit.to_si_multiplier();
        if mult == 0.0 {
            val_si
        } else {
            val_si / mult
        }
    }

    /// Convert directly between two dimensionally compatible units.
    pub fn convert(val: f64, from: &PhysicsUnit, to: &PhysicsUnit) -> Option<f64> {
        if !Self::are_compatible(from, to) {
            return None;
        }
        let si_val = Self::convert_to_si(val, from);
        Some(Self::convert_from_si(si_val, to))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unit_dimensions_and_compatibility() {
        assert_eq!(PhysicsUnit::Meter.dimension(), PhysicalDimension::LENGTH);
        assert_eq!(PhysicsUnit::Second.dimension(), PhysicalDimension::TIME);
        assert_eq!(PhysicsUnit::KilometerPerHour.dimension(), PhysicalDimension::VELOCITY);
        assert_eq!(PhysicsUnit::MeterPerSecond.dimension(), PhysicalDimension::VELOCITY);
        assert_eq!(PhysicsUnit::Newton.dimension(), PhysicalDimension::FORCE);
        assert_eq!(PhysicsUnit::Joule.dimension(), PhysicalDimension::ENERGY);
        assert_eq!(PhysicsUnit::Watt.dimension(), PhysicalDimension::POWER);

        assert!(DimensionalValidator::are_compatible(
            &PhysicsUnit::KilometerPerHour,
            &PhysicsUnit::MeterPerSecond
        ));
        assert!(!DimensionalValidator::are_compatible(
            &PhysicsUnit::MeterPerSecond,
            &PhysicsUnit::Newton
        ));
        assert!(!DimensionalValidator::are_compatible(
            &PhysicsUnit::Joule,
            &PhysicsUnit::Watt
        ));
    }

    #[test]
    fn test_unit_conversions_and_scaling() {
        // 72 km/h == 20 m/s
        let mps = DimensionalValidator::convert(72.0, &PhysicsUnit::KilometerPerHour, &PhysicsUnit::MeterPerSecond).unwrap();
        assert!((mps - 20.0).abs() < 1e-6);

        // 20 m/s == 72 km/h
        let kmh = DimensionalValidator::convert(20.0, &PhysicsUnit::MeterPerSecond, &PhysicsUnit::KilometerPerHour).unwrap();
        assert!((kmh - 72.0).abs() < 1e-6);

        // 500 g == 0.5 kg
        let kg = DimensionalValidator::convert(500.0, &PhysicsUnit::Gram, &PhysicsUnit::Kilogram).unwrap();
        assert!((kg - 0.5).abs() < 1e-6);

        // Incompatible conversion returns None
        assert!(DimensionalValidator::convert(10.0, &PhysicsUnit::Second, &PhysicsUnit::Meter).is_none());
    }

    #[test]
    fn test_parse_unit_symbols_and_synonyms() {
        assert_eq!(DimensionalValidator::parse_unit("km/h"), Some(PhysicsUnit::KilometerPerHour));
        assert_eq!(DimensionalValidator::parse_unit("m/s"), Some(PhysicsUnit::MeterPerSecond));
        assert_eq!(DimensionalValidator::parse_unit("Joules"), Some(PhysicsUnit::Joule));
        assert_eq!(DimensionalValidator::parse_unit("Newtons"), Some(PhysicsUnit::Newton));
        assert_eq!(DimensionalValidator::parse_unit("watts"), Some(PhysicsUnit::Watt));
        assert_eq!(DimensionalValidator::parse_unit("unknown_xyz"), None);
    }
}
