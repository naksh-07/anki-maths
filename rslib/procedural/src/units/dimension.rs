// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::fmt;
use std::ops::{Div, Mul};
use serde::{Deserialize, Serialize};

/// Fundamental dimensional vector for physical and chemical dimensional analysis:
/// [Mass]^m * [Length]^l * [Time]^t * [AmountOfSubstance]^n * [Temperature]^k
///
/// Corresponds to SI base dimensions:
/// - M: Mass (kg)
/// - L: Length (m)
/// - T: Time (s)
/// - N: Amount of substance (mol)
/// - K: Thermodynamic temperature (K)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Dimension {
    pub mass: i8,        // [M]
    pub length: i8,      // [L]
    pub time: i8,        // [T]
    pub amount: i8,      // [N]
    pub temperature: i8, // [K]
}

impl Dimension {
    pub const DIMENSIONLESS: Self = Self { mass: 0, length: 0, time: 0, amount: 0, temperature: 0 };
    pub const MASS: Self = Self { mass: 1, length: 0, time: 0, amount: 0, temperature: 0 };
    pub const LENGTH: Self = Self { mass: 0, length: 1, time: 0, amount: 0, temperature: 0 };
    pub const TIME: Self = Self { mass: 0, length: 0, time: 1, amount: 0, temperature: 0 };
    pub const AMOUNT: Self = Self { mass: 0, length: 0, time: 0, amount: 1, temperature: 0 };
    pub const TEMPERATURE: Self = Self { mass: 0, length: 0, time: 0, amount: 0, temperature: 1 };

    // Derived kinematics & mechanics
    pub const AREA: Self = Self { mass: 0, length: 2, time: 0, amount: 0, temperature: 0 };
    pub const VOLUME: Self = Self { mass: 0, length: 3, time: 0, amount: 0, temperature: 0 };
    pub const VELOCITY: Self = Self { mass: 0, length: 1, time: -1, amount: 0, temperature: 0 };
    pub const ACCELERATION: Self = Self { mass: 0, length: 1, time: -2, amount: 0, temperature: 0 };
    pub const FORCE: Self = Self { mass: 1, length: 1, time: -2, amount: 0, temperature: 0 };
    pub const ENERGY: Self = Self { mass: 1, length: 2, time: -2, amount: 0, temperature: 0 };
    pub const POWER: Self = Self { mass: 1, length: 2, time: -3, amount: 0, temperature: 0 };
    pub const PRESSURE: Self = Self { mass: 1, length: -1, time: -2, amount: 0, temperature: 0 };
    pub const DENSITY: Self = Self { mass: 1, length: -3, time: 0, amount: 0, temperature: 0 };
    pub const FREQUENCY: Self = Self { mass: 0, length: 0, time: -1, amount: 0, temperature: 0 };

    // Derived chemistry & thermodynamics
    pub const CONCENTRATION: Self = Self { mass: 0, length: -3, time: 0, amount: 1, temperature: 0 };
    pub const MOLAR_MASS: Self = Self { mass: 1, length: 0, time: 0, amount: -1, temperature: 0 };
    pub const MOLAR_ENERGY: Self = Self { mass: 1, length: 2, time: -2, amount: -1, temperature: 0 };
    pub const MOLAR_VOLUME: Self = Self { mass: 0, length: 3, time: 0, amount: -1, temperature: 0 };
    pub const MOLAR_HEAT_CAPACITY: Self = Self { mass: 1, length: 2, time: -2, amount: -1, temperature: -1 };
    pub const SPECIFIC_HEAT_CAPACITY: Self = Self { mass: 0, length: 2, time: -2, amount: 0, temperature: -1 };

    pub const fn new(mass: i8, length: i8, time: i8, amount: i8, temperature: i8) -> Self {
        Self { mass, length, time, amount, temperature }
    }

    #[inline]
    pub fn is_dimensionless(&self) -> bool {
        self.mass == 0 && self.length == 0 && self.time == 0 && self.amount == 0 && self.temperature == 0
    }

    #[inline]
    pub fn is_compatible_with(&self, other: &Self) -> bool {
        self == other
    }

    pub fn multiply(self, other: Self) -> Self {
        Self {
            mass: self.mass + other.mass,
            length: self.length + other.length,
            time: self.time + other.time,
            amount: self.amount + other.amount,
            temperature: self.temperature + other.temperature,
        }
    }

    pub fn divide(self, other: Self) -> Self {
        Self {
            mass: self.mass - other.mass,
            length: self.length - other.length,
            time: self.time - other.time,
            amount: self.amount - other.amount,
            temperature: self.temperature - other.temperature,
        }
    }

    pub fn pow(self, exp: i8) -> Self {
        Self {
            mass: self.mass * exp,
            length: self.length * exp,
            time: self.time * exp,
            amount: self.amount * exp,
            temperature: self.temperature * exp,
        }
    }
}

impl Mul for Dimension {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        self.multiply(rhs)
    }
}

impl Div for Dimension {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        self.divide(rhs)
    }
}

impl fmt::Display for Dimension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_dimensionless() {
            return write!(f, "1 (dimensionless)");
        }
        let mut parts = Vec::new();
        if self.mass != 0 { parts.push(format!("[M]^{}", self.mass)); }
        if self.length != 0 { parts.push(format!("[L]^{}", self.length)); }
        if self.time != 0 { parts.push(format!("[T]^{}", self.time)); }
        if self.amount != 0 { parts.push(format!("[N]^{}", self.amount)); }
        if self.temperature != 0 { parts.push(format!("[K]^{}", self.temperature)); }
        write!(f, "{}", parts.join(" * "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dimension_algebra() {
        assert_eq!(Dimension::LENGTH / Dimension::TIME, Dimension::VELOCITY);
        assert_eq!(Dimension::VELOCITY / Dimension::TIME, Dimension::ACCELERATION);
        assert_eq!(Dimension::MASS * Dimension::ACCELERATION, Dimension::FORCE);
        assert_eq!(Dimension::FORCE * Dimension::LENGTH, Dimension::ENERGY);
        assert_eq!(Dimension::ENERGY / Dimension::TIME, Dimension::POWER);
        assert_eq!(Dimension::FORCE / Dimension::AREA, Dimension::PRESSURE);
        assert_eq!(Dimension::ENERGY / Dimension::AMOUNT, Dimension::MOLAR_ENERGY);
        assert_eq!(Dimension::AMOUNT / Dimension::VOLUME, Dimension::CONCENTRATION);
    }

    #[test]
    fn test_dimensionless() {
        assert!(Dimension::DIMENSIONLESS.is_dimensionless());
        let ratio = Dimension::LENGTH / Dimension::LENGTH;
        assert!(ratio.is_dimensionless());
    }
}
