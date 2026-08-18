// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use serde::{Deserialize, Serialize};
use crate::diagnostics::ErrorCategory;
use crate::problems::steps::StepErrorType;

/// Specialized taxonomic error categories for Physics learning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PhysicsErrorCategory {
    /// Inappropriate physical model or governing law selected for the situation.
    ModelSelectionError,
    /// Misrepresented physical initial/final state, reference level, or coordinate system.
    RepresentationError,
    /// Correct physical law chosen, but mathematical equation construction was flawed.
    EquationSetupError,
    /// Missing, unscaled, or dimensionally incompatible physical units.
    UnitError,
    /// Inverted directional vector sign or gravitational orientation.
    SignConventionError,
    /// Physical formulation is sound, but mathematical execution / arithmetic slip occurred.
    AlgebraExecutionError,
    /// Numerical result violates fundamental physical constraints (e.g. negative time, negative energy).
    PhysicalPlausibilityError,
    /// Correct solution, but latency exceeded target pacing threshold.
    TimeError,
    /// Unclassified fallback.
    Unknown,
}

impl PhysicsErrorCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            PhysicsErrorCategory::ModelSelectionError => "model_selection_error",
            PhysicsErrorCategory::RepresentationError => "representation_error",
            PhysicsErrorCategory::EquationSetupError => "equation_setup_error",
            PhysicsErrorCategory::UnitError => "unit_error",
            PhysicsErrorCategory::SignConventionError => "sign_convention_error",
            PhysicsErrorCategory::AlgebraExecutionError => "algebra_execution_error",
            PhysicsErrorCategory::PhysicalPlausibilityError => "physical_plausibility_error",
            PhysicsErrorCategory::TimeError => "time_error",
            PhysicsErrorCategory::Unknown => "unknown",
        }
    }

    /// Map domain-specific physics error to common engine ErrorCategory.
    pub fn to_common_error_category(&self) -> ErrorCategory {
        match self {
            PhysicsErrorCategory::ModelSelectionError => ErrorCategory::Strategy,
            PhysicsErrorCategory::RepresentationError => ErrorCategory::Concept,
            PhysicsErrorCategory::EquationSetupError => ErrorCategory::Concept,
            PhysicsErrorCategory::UnitError => ErrorCategory::Unit,
            PhysicsErrorCategory::SignConventionError => ErrorCategory::Sign,
            PhysicsErrorCategory::AlgebraExecutionError => ErrorCategory::Calculation,
            PhysicsErrorCategory::PhysicalPlausibilityError => ErrorCategory::Concept,
            PhysicsErrorCategory::TimeError => ErrorCategory::Time,
            PhysicsErrorCategory::Unknown => ErrorCategory::Unknown,
        }
    }

    /// Convert from common StepErrorType.
    pub fn from_step_error_type(err: StepErrorType) -> Self {
        match err {
            StepErrorType::ModelSelectionError | StepErrorType::FormulaSelectionError => {
                PhysicsErrorCategory::ModelSelectionError
            }
            StepErrorType::RepresentationError => PhysicsErrorCategory::RepresentationError,
            StepErrorType::EquationSetupError | StepErrorType::SetupError => {
                PhysicsErrorCategory::EquationSetupError
            }
            StepErrorType::UnitError => PhysicsErrorCategory::UnitError,
            StepErrorType::SignConventionError | StepErrorType::SignError => {
                PhysicsErrorCategory::SignConventionError
            }
            StepErrorType::AlgebraExecutionError
            | StepErrorType::TransformationError
            | StepErrorType::ArithmeticError => PhysicsErrorCategory::AlgebraExecutionError,
            StepErrorType::PhysicalPlausibilityError => PhysicsErrorCategory::PhysicalPlausibilityError,
            _ => PhysicsErrorCategory::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_physics_error_category_mappings() {
        assert_eq!(
            PhysicsErrorCategory::ModelSelectionError.to_common_error_category(),
            ErrorCategory::Strategy
        );
        assert_eq!(
            PhysicsErrorCategory::UnitError.to_common_error_category(),
            ErrorCategory::Unit
        );
        assert_eq!(
            PhysicsErrorCategory::SignConventionError.to_common_error_category(),
            ErrorCategory::Sign
        );
        assert_eq!(
            PhysicsErrorCategory::AlgebraExecutionError.to_common_error_category(),
            ErrorCategory::Calculation
        );

        assert_eq!(
            PhysicsErrorCategory::from_step_error_type(StepErrorType::ModelSelectionError),
            PhysicsErrorCategory::ModelSelectionError
        );
    }
}
