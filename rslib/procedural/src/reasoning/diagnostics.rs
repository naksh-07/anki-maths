// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use serde::{Deserialize, Serialize};
use crate::diagnostics::ErrorCategory;
use crate::problems::steps::StepErrorType;

/// Specialized taxonomic error categories for Reasoning learning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasoningErrorCategory {
    /// Failed to identify or classify the correct structural problem schema.
    SchemaRecognitionError,
    /// Chose the wrong strategy, wrong starting constraint, or inappropriate anchor.
    StrategySelectionError,
    /// Constructed flawed diagram, mental model, slot allocation, or graph structure.
    RepresentationError,
    /// Misapplied a puzzle rule, ignored a condition, or violated an explicit constraint.
    ConstraintApplicationError,
    /// Made an invalid logical deduction or unjustified relational leap.
    InferenceError,
    /// Missed, improperly merged, or mishandled branching search cases.
    SearchCaseError,
    /// Failed to detect, test, or utilize logical contradiction.
    ContradictionHandlingError,
    /// Fell for a predictable logical distractor, inverse phrasing, or trap wording.
    ReadingTrapError,
    /// Reasoning and strategy are sound, but arithmetic or clerical execution slip occurred.
    ExecutionError,
    /// Solution is correct, but solving time exceeded pacing target.
    TimeError,
    /// Unclassified or fallback error.
    Unknown,
}

impl ReasoningErrorCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            ReasoningErrorCategory::SchemaRecognitionError => "schema_recognition_error",
            ReasoningErrorCategory::StrategySelectionError => "strategy_selection_error",
            ReasoningErrorCategory::RepresentationError => "representation_error",
            ReasoningErrorCategory::ConstraintApplicationError => "constraint_application_error",
            ReasoningErrorCategory::InferenceError => "inference_error",
            ReasoningErrorCategory::SearchCaseError => "search_case_error",
            ReasoningErrorCategory::ContradictionHandlingError => "contradiction_handling_error",
            ReasoningErrorCategory::ReadingTrapError => "reading_trap_error",
            ReasoningErrorCategory::ExecutionError => "execution_error",
            ReasoningErrorCategory::TimeError => "time_error",
            ReasoningErrorCategory::Unknown => "unknown",
        }
    }

    /// Map reasoning-specific error category to the common engine ErrorCategory.
    pub fn to_common_error_category(&self) -> ErrorCategory {
        match self {
            ReasoningErrorCategory::SchemaRecognitionError => ErrorCategory::Concept,
            ReasoningErrorCategory::StrategySelectionError => ErrorCategory::Strategy,
            ReasoningErrorCategory::RepresentationError => ErrorCategory::Concept,
            ReasoningErrorCategory::ConstraintApplicationError => ErrorCategory::Strategy,
            ReasoningErrorCategory::InferenceError => ErrorCategory::Concept,
            ReasoningErrorCategory::SearchCaseError => ErrorCategory::Strategy,
            ReasoningErrorCategory::ContradictionHandlingError => ErrorCategory::Strategy,
            ReasoningErrorCategory::ReadingTrapError => ErrorCategory::Careless,
            ReasoningErrorCategory::ExecutionError => ErrorCategory::Calculation,
            ReasoningErrorCategory::TimeError => ErrorCategory::Time,
            ReasoningErrorCategory::Unknown => ErrorCategory::Unknown,
        }
    }

    /// Convert from common StepErrorType.
    pub fn from_step_error_type(err: StepErrorType) -> Self {
        match err {
            StepErrorType::SchemaRecognitionError => ReasoningErrorCategory::SchemaRecognitionError,
            StepErrorType::StrategySelectionError | StepErrorType::FormulaSelectionError => {
                ReasoningErrorCategory::StrategySelectionError
            }
            StepErrorType::RepresentationError => ReasoningErrorCategory::RepresentationError,
            StepErrorType::ConstraintApplicationError | StepErrorType::SetupError => {
                ReasoningErrorCategory::ConstraintApplicationError
            }
            StepErrorType::InferenceError => ReasoningErrorCategory::InferenceError,
            StepErrorType::SearchCaseError => ReasoningErrorCategory::SearchCaseError,
            StepErrorType::ContradictionHandlingError => {
                ReasoningErrorCategory::ContradictionHandlingError
            }
            StepErrorType::ReadingTrapError => ReasoningErrorCategory::ReadingTrapError,
            StepErrorType::ExecutionSlipError
            | StepErrorType::ArithmeticError
            | StepErrorType::TransformationError => ReasoningErrorCategory::ExecutionError,
            _ => ReasoningErrorCategory::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reasoning_error_category_mappings() {
        assert_eq!(
            ReasoningErrorCategory::StrategySelectionError.to_common_error_category(),
            ErrorCategory::Strategy
        );
        assert_eq!(
            ReasoningErrorCategory::SchemaRecognitionError.to_common_error_category(),
            ErrorCategory::Concept
        );
        assert_eq!(
            ReasoningErrorCategory::ExecutionError.to_common_error_category(),
            ErrorCategory::Calculation
        );
        assert_eq!(
            ReasoningErrorCategory::from_step_error_type(StepErrorType::StrategySelectionError),
            ReasoningErrorCategory::StrategySelectionError
        );
    }
}
