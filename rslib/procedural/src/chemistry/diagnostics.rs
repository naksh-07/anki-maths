// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::fmt;
use serde::{Deserialize, Serialize};

use crate::diagnostics::ErrorCategory;
use crate::problems::steps::StepErrorType;

/// Specialized diagnostic error taxonomy for the Chemistry domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChemistryErrorCategory {
    /// Incorrect chemical species, molecular formula, or molar mass representation.
    ChemicalRepresentationError,
    /// Chemical reaction balancing error or invalid stoichiometric coefficients.
    EquationBalanceError,
    /// Applied incorrect mole ratio between reactant and product coefficients.
    StoichiometricRatioError,
    /// Incorrectly identified limiting reagent or inverted limiting ratio comparison.
    LimitingReagentError,
    /// Selected inappropriate chemical regime (e.g. mole-ratio vs equilibrium ICE table).
    RegimeSelectionError,
    /// Incorrect chemical quantity or concentration unit conversion (e.g. mL vs L, g vs kg).
    UnitConversionError,
    /// Chemistry setup is sound, but arithmetic or algebraic execution was wrong.
    NumericalExecutionError,
    /// Result violates chemical conservation laws (element conservation, charge, or positive mass).
    ConservationViolationError,
    /// Mathematical result is correct but misinterpreted (e.g. reported moles instead of mass).
    InterpretationError,
    /// Unclassified or fallback error.
    Unknown,
}

impl ChemistryErrorCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            ChemistryErrorCategory::ChemicalRepresentationError => "chemical_representation_error",
            ChemistryErrorCategory::EquationBalanceError => "equation_balance_error",
            ChemistryErrorCategory::StoichiometricRatioError => "stoichiometric_ratio_error",
            ChemistryErrorCategory::LimitingReagentError => "limiting_reagent_error",
            ChemistryErrorCategory::RegimeSelectionError => "regime_selection_error",
            ChemistryErrorCategory::UnitConversionError => "unit_conversion_error",
            ChemistryErrorCategory::NumericalExecutionError => "numerical_execution_error",
            ChemistryErrorCategory::ConservationViolationError => "conservation_violation_error",
            ChemistryErrorCategory::InterpretationError => "interpretation_error",
            ChemistryErrorCategory::Unknown => "unknown",
        }
    }

    /// Map domain-specific chemistry category into general ErrorCategory for unified telemetry.
    pub fn to_common_error_category(&self) -> ErrorCategory {
        match self {
            ChemistryErrorCategory::ChemicalRepresentationError => ErrorCategory::Concept,
            ChemistryErrorCategory::EquationBalanceError => ErrorCategory::Strategy,
            ChemistryErrorCategory::StoichiometricRatioError => ErrorCategory::Strategy,
            ChemistryErrorCategory::LimitingReagentError => ErrorCategory::Strategy,
            ChemistryErrorCategory::RegimeSelectionError => ErrorCategory::Concept,
            ChemistryErrorCategory::UnitConversionError => ErrorCategory::Unit,
            ChemistryErrorCategory::NumericalExecutionError => ErrorCategory::Calculation,
            ChemistryErrorCategory::ConservationViolationError => ErrorCategory::Concept,
            ChemistryErrorCategory::InterpretationError => ErrorCategory::Syntax,
            ChemistryErrorCategory::Unknown => ErrorCategory::Unknown,
        }
    }

    /// Map to StepErrorType for StepGraph stepwise validation.
    pub fn to_step_error_type(&self) -> StepErrorType {
        match self {
            ChemistryErrorCategory::ChemicalRepresentationError => StepErrorType::ChemicalRepresentationError,
            ChemistryErrorCategory::EquationBalanceError => StepErrorType::EquationBalanceError,
            ChemistryErrorCategory::StoichiometricRatioError => StepErrorType::StoichiometricRatioError,
            ChemistryErrorCategory::LimitingReagentError => StepErrorType::LimitingReagentError,
            ChemistryErrorCategory::RegimeSelectionError => StepErrorType::RegimeSelectionError,
            ChemistryErrorCategory::UnitConversionError => StepErrorType::UnitError,
            ChemistryErrorCategory::NumericalExecutionError => StepErrorType::ArithmeticError,
            ChemistryErrorCategory::ConservationViolationError => StepErrorType::ConservationViolationError,
            ChemistryErrorCategory::InterpretationError => StepErrorType::FinalAnswerFormattingError,
            ChemistryErrorCategory::Unknown => StepErrorType::Unknown,
        }
    }
}

impl fmt::Display for ChemistryErrorCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chemistry_error_category_mappings() {
        assert_eq!(
            ChemistryErrorCategory::StoichiometricRatioError.to_common_error_category(),
            ErrorCategory::Strategy
        );
        assert_eq!(
            ChemistryErrorCategory::LimitingReagentError.to_step_error_type(),
            StepErrorType::LimitingReagentError
        );
        assert_eq!(
            ChemistryErrorCategory::UnitConversionError.to_common_error_category(),
            ErrorCategory::Unit
        );
    }
}
