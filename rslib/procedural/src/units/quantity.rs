// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::fmt;
use serde::{Deserialize, Serialize};

use super::dimension::Dimension;
use super::tolerance::Tolerance;
use super::unit_def::Unit;

/// A physical or chemical quantity with an associated scalar value and unit.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Quantity {
    pub value: f64,
    pub unit: Unit,
}

impl Quantity {
    pub const fn new(value: f64, unit: Unit) -> Self {
        Self { value, unit }
    }

    pub const fn dimensionless(value: f64) -> Self {
        Self {
            value,
            unit: Unit::Dimensionless,
        }
    }

    pub fn dimension(&self) -> Dimension {
        self.unit.dimension()
    }

    pub fn is_compatible_with(&self, other: &Self) -> bool {
        self.unit.is_compatible_with(&other.unit)
    }

    pub fn convert_to(&self, target_unit: Unit) -> Option<Self> {
        let converted_val = self.unit.convert_to(self.value, &target_unit)?;
        Some(Self::new(converted_val, target_unit))
    }

    pub fn is_equivalent(&self, other: &Self, tolerance: &Tolerance) -> bool {
        if !self.is_compatible_with(other) {
            return false;
        }
        let Some(converted_other) = other.convert_to(self.unit) else {
            return false;
        };
        tolerance.is_within(converted_other.value, self.value)
    }
}

impl fmt::Display for Quantity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.unit == Unit::Dimensionless {
            write!(f, "{}", self.value)
        } else if self.unit == Unit::Percent {
            write!(f, "{}%", self.value)
        } else {
            write!(f, "{} {}", self.value, self.unit.symbol())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quantity_equivalence() {
        let q1 = Quantity::new(72.0, Unit::KilometerPerHour);
        let q2 = Quantity::new(20.0, Unit::MeterPerSecond);
        let tol = Tolerance::relative(0.001);
        assert!(q1.is_equivalent(&q2, &tol));
        let q3 = Quantity::new(2500.0, Unit::Gram);
        let q4 = Quantity::new(2.5, Unit::Kilogram);
        assert!(q3.is_equivalent(&q4, &tol));
        let q5 = Quantity::new(1.2e-3, Unit::Molar);
        let q6 = Quantity::new(1.2, Unit::Millimolar);
        assert!(q5.is_equivalent(&q6, &tol));
        let q7 = Quantity::new(10.0, Unit::Second);
        let q8 = Quantity::new(10.0, Unit::Meter);
        assert!(!q7.is_equivalent(&q8, &tol));
    }
}
