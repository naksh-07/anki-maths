// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use serde::{Deserialize, Serialize};
use crate::physics::units::PhysicsUnit;

/// Discrete physical model / law category applicable to a physical situation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PhysicalModelKind {
    /// 1D uniform motion with zero acceleration: s = v * t
    KinematicsUniform,
    /// 1D motion with constant acceleration: v = u + at, s = ut + 1/2 at^2, v^2 = u^2 + 2as
    KinematicsConstantAcceleration,
    /// Vertical motion under constant gravitational acceleration a = -g
    KinematicsVerticalFreeFall,
    /// Work-Energy Theorem: Net Work = Delta Kinetic Energy
    WorkEnergyTheorem,
    /// Conservation of Mechanical Energy: E_initial = E_final (PE_i + KE_i = PE_f + KE_f)
    MechanicalEnergyConservation,
    /// Power as rate of doing work: P = W / t = F * v
    PowerWorkRelation,
    /// Newton's Second Law: Net Force = mass * acceleration
    NewtonSecondLaw,
}

impl PhysicalModelKind {
    pub fn name(&self) -> &'static str {
        match self {
            PhysicalModelKind::KinematicsUniform => "Uniform Motion (Zero Acceleration)",
            PhysicalModelKind::KinematicsConstantAcceleration => "Constant Acceleration Kinematics",
            PhysicalModelKind::KinematicsVerticalFreeFall => "Vertical Motion Under Gravity",
            PhysicalModelKind::WorkEnergyTheorem => "Work-Energy Theorem",
            PhysicalModelKind::MechanicalEnergyConservation => "Conservation of Mechanical Energy",
            PhysicalModelKind::PowerWorkRelation => "Power & Work Rate",
            PhysicalModelKind::NewtonSecondLaw => "Newton's Second Law of Motion",
        }
    }

    pub fn identifier(&self) -> &'static str {
        match self {
            PhysicalModelKind::KinematicsUniform => "kinematics_uniform",
            PhysicalModelKind::KinematicsConstantAcceleration => "kinematics_constant_acceleration",
            PhysicalModelKind::KinematicsVerticalFreeFall => "kinematics_vertical_free_fall",
            PhysicalModelKind::WorkEnergyTheorem => "work_energy_theorem",
            PhysicalModelKind::MechanicalEnergyConservation => "mechanical_energy_conservation",
            PhysicalModelKind::PowerWorkRelation => "power_work_relation",
            PhysicalModelKind::NewtonSecondLaw => "newton_second_law",
        }
    }
}

/// A structured physical quantity with symbol, magnitude, and unit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhysicalQuantity {
    pub name: String,
    pub symbol: String,
    pub value: Option<f64>,
    pub unit: PhysicsUnit,
    pub is_target: bool,
}

impl PhysicalQuantity {
    pub fn known(name: impl Into<String>, symbol: impl Into<String>, value: f64, unit: PhysicsUnit) -> Self {
        Self {
            name: name.into(),
            symbol: symbol.into(),
            value: Some(value),
            unit,
            is_target: false,
        }
    }

    pub fn unknown(name: impl Into<String>, symbol: impl Into<String>, unit: PhysicsUnit) -> Self {
        Self {
            name: name.into(),
            symbol: symbol.into(),
            value: None,
            unit,
            is_target: true,
        }
    }
}

/// Coordinate system and directional convention.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoordinateSystem {
    pub dimension: String,
    pub positive_direction: String,
    pub origin_reference: String,
}

impl Default for CoordinateSystem {
    fn default() -> Self {
        Self {
            dimension: "1D".to_string(),
            positive_direction: "forward / right / upward".to_string(),
            origin_reference: "initial position x0 = 0".to_string(),
        }
    }
}

/// Physical regime constraints and environmental parameters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhysicalRegime {
    pub regime_name: String,
    pub gravity_acceleration: f64,
    pub friction_present: bool,
    pub air_resistance_neglected: bool,
}

impl Default for PhysicalRegime {
    fn default() -> Self {
        Self {
            regime_name: "Classical 1D Mechanics".to_string(),
            gravity_acceleration: 9.8,
            friction_present: false,
            air_resistance_neglected: true,
        }
    }
}

/// Comprehensive structured physical problem metadata stored inside a `ProblemInstance`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhysicalProblemMetadata {
    pub regime: PhysicalRegime,
    pub coordinate_system: CoordinateSystem,
    pub candidate_models: Vec<PhysicalModelKind>,
    pub governing_model: PhysicalModelKind,
    pub known_quantities: Vec<PhysicalQuantity>,
    pub target_quantity: PhysicalQuantity,
    pub governing_equations: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_physical_quantity_creation() {
        let u = PhysicalQuantity::known("Initial Velocity", "u", 10.0, PhysicsUnit::MeterPerSecond);
        let s = PhysicalQuantity::unknown("Displacement", "s", PhysicsUnit::Meter);

        assert_eq!(u.value, Some(10.0));
        assert!(!u.is_target);
        assert_eq!(s.value, None);
        assert!(s.is_target);
    }

    #[test]
    fn test_model_metadata_roundtrip() {
        let meta = PhysicalProblemMetadata {
            regime: PhysicalRegime::default(),
            coordinate_system: CoordinateSystem::default(),
            candidate_models: vec![
                PhysicalModelKind::KinematicsUniform,
                PhysicalModelKind::KinematicsConstantAcceleration,
            ],
            governing_model: PhysicalModelKind::KinematicsConstantAcceleration,
            known_quantities: vec![
                PhysicalQuantity::known("Initial Velocity", "u", 0.0, PhysicsUnit::MeterPerSecond),
                PhysicalQuantity::known("Acceleration", "a", 2.0, PhysicsUnit::MeterPerSecondSquared),
                PhysicalQuantity::known("Time", "t", 5.0, PhysicsUnit::Second),
            ],
            target_quantity: PhysicalQuantity::unknown("Final Velocity", "v", PhysicsUnit::MeterPerSecond),
            governing_equations: vec!["v = u + at".to_string()],
        };

        let json = serde_json::to_value(&meta).unwrap();
        assert_eq!(json["governing_model"], "kinematics_constant_acceleration");
        assert_eq!(json["candidate_models"].as_array().unwrap().len(), 2);
    }
}
