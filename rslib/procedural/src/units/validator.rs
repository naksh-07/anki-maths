// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::str::FromStr;
use crate::diagnostics::ErrorCategory;
use crate::problems::validator::AnswerEvaluation;
use super::parser::UnitParser;
use super::tolerance::Tolerance;
use super::unit_def::Unit;

/// Comprehensive evaluation engine for Physics, Chemistry, and Numerical STEM items.
pub struct UnitAnswerValidator;

impl UnitAnswerValidator {
    pub fn evaluate(
        student_input: &serde_json::Value,
        expected_val: f64,
        expected_unit: Unit,
        tolerance: Tolerance,
        time_taken_ms: u64,
        target_time_ms: u64,
    ) -> AnswerEvaluation {
        Self::evaluate_advanced(
            student_input,
            expected_val,
            expected_unit,
            tolerance,
            false,
            true,
            time_taken_ms,
            target_time_ms,
        )
    }

    pub fn evaluate_advanced(
        student_input: &serde_json::Value,
        expected_val: f64,
        expected_unit: Unit,
        tolerance: Tolerance,
        require_unit: bool,
        enforce_non_negative: bool,
        time_taken_ms: u64,
        target_time_ms: u64,
    ) -> AnswerEvaluation {
        let parsed = UnitParser::parse_json(student_input);
        let Some(p) = parsed else {
            return AnswerEvaluation::incorrect(
                ErrorCategory::Calculation,
                "Unable to parse numeric answer. Please provide a valid scalar, fraction, or physical quantity.",
            )
            .with_parsed_values(0.0, expected_val);
        };

        let is_naturally_non_negative = expected_unit.dimension().mass > 0
            || expected_unit.dimension().length > 0
            || expected_unit.dimension().amount > 0
            || (expected_unit.dimension().time > 0 && expected_val >= 0.0);

        if enforce_non_negative && is_naturally_non_negative && p.value < -1e-6 {
            return AnswerEvaluation::incorrect(
                ErrorCategory::Concept,
                format!(
                    "Physical Sanity Violation: Quantity cannot be negative (received {}).",
                    p.value
                ),
            )
            .with_parsed_values(p.value, expected_val);
        }

        if require_unit && expected_unit != Unit::Dimensionless && !p.has_explicit_unit {
            return AnswerEvaluation::incorrect(
                ErrorCategory::Unit,
                format!(
                    "Missing Unit: An explicit unit is required for this answer (e.g. {}).",
                    expected_unit.symbol()
                ),
            )
            .with_parsed_values(p.value, expected_val);
        }

        if let (None, Some(raw_u)) = (p.unit, &p.raw_unit_str) {
            return AnswerEvaluation::incorrect(
                ErrorCategory::Unit,
                format!(
                    "Unrecognized unit '{}'. Expected physical dimension: {} (e.g. {}).",
                    raw_u,
                    expected_unit.dimension(),
                    expected_unit.symbol()
                ),
            )
            .with_parsed_values(p.value, expected_val);
        }

        if let Some(student_unit) = p.unit {
            if student_unit != Unit::Dimensionless {
                if !student_unit.is_compatible_with(&expected_unit) {
                    return AnswerEvaluation::incorrect(
                        ErrorCategory::Unit,
                        format!(
                            "Dimensional Incompatibility: Received unit '{}' with dimension {}, but expected dimension {} ({})",
                            student_unit.symbol(),
                            student_unit.dimension(),
                            expected_unit.dimension(),
                            expected_unit.symbol()
                        ),
                    )
                    .with_parsed_values(p.value, expected_val);
                }

                if let Some(converted_val) = student_unit.convert_to(p.value, &expected_unit) {
                    if tolerance.is_within(converted_val, expected_val) {
                        let score = if target_time_ms > 0 && time_taken_ms > target_time_ms * 2 {
                            0.8
                        } else {
                            1.0
                        };
                        return AnswerEvaluation::correct(score, time_taken_ms, target_time_ms)
                            .with_parsed_values(converted_val, expected_val)
                            .with_diagnostic(format!(
                                "✓ Correct ({:.4} {} is equivalent to {:.4} {})",
                                p.value,
                                student_unit.symbol(),
                                expected_val,
                                expected_unit.symbol()
                            ));
                    } else {
                        return AnswerEvaluation::incorrect(
                            ErrorCategory::Calculation,
                            format!(
                                "Calculation Error: {:.4} {} converts to {:.4} {}, but expected {:.4} {}.",
                                p.value,
                                student_unit.symbol(),
                                converted_val,
                                expected_unit.symbol(),
                                expected_val,
                                expected_unit.symbol()
                            ),
                        )
                        .with_parsed_values(converted_val, expected_val);
                    }
                }
            }
        }

        if tolerance.is_within(p.value, expected_val) {
            let score = if target_time_ms > 0 && time_taken_ms > target_time_ms * 2 {
                0.8
            } else {
                1.0
            };
            return AnswerEvaluation::correct(score, time_taken_ms, target_time_ms)
                .with_parsed_values(p.value, expected_val);
        }

        if expected_unit == Unit::MeterPerSecond {
            if let Some(converted) = Unit::KilometerPerHour.convert_to(p.value, &Unit::MeterPerSecond) {
                if tolerance.is_within(converted, expected_val) {
                    return AnswerEvaluation::incorrect(
                        ErrorCategory::Unit,
                        format!(
                            "Missing Unit Conversion: You answered {} (km/h) without converting to SI unit m/s (expected {} m/s = {} * 5/18).",
                            p.value, expected_val, p.value
                        ),
                    )
                    .with_parsed_values(p.value, expected_val);
                }
            }
        }

        if expected_unit == Unit::Kilogram {
            if let Some(converted) = Unit::Gram.convert_to(p.value, &Unit::Kilogram) {
                if tolerance.is_within(converted, expected_val) {
                    return AnswerEvaluation::incorrect(
                        ErrorCategory::Unit,
                        format!(
                            "Missing Unit Conversion: You answered {} (g) without converting to SI unit kg (expected {} kg = {} / 1000).",
                            p.value, expected_val, p.value
                        ),
                    )
                    .with_parsed_values(p.value, expected_val);
                }
            }
        }

        if expected_unit == Unit::Molar {
            if let Some(converted) = Unit::Millimolar.convert_to(p.value, &Unit::Molar) {
                if tolerance.is_within(converted, expected_val) {
                    return AnswerEvaluation::incorrect(
                        ErrorCategory::Unit,
                        format!(
                            "Missing Unit Conversion: You answered {} (mM) without converting to Molar (expected {} M = {} / 1000).",
                            p.value, expected_val, p.value
                        ),
                    )
                    .with_parsed_values(p.value, expected_val);
                }
            }
        }

        AnswerEvaluation::incorrect(
            ErrorCategory::Calculation,
            format!(
                "Calculation Error: Expected {:.4} {}, but received {:.4}.",
                expected_val,
                expected_unit.symbol(),
                p.value
            ),
        )
        .with_parsed_values(p.value, expected_val)
    }

    pub fn evaluate_instance_answer(
        correct_answer: &serde_json::Value,
        student_input: &serde_json::Value,
        time_taken_ms: u64,
        target_time_ms: u64,
    ) -> AnswerEvaluation {
        let expected_val = correct_answer
            .get("value")
            .or_else(|| correct_answer.get("answer"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let unit_str = correct_answer
            .get("unit")
            .and_then(|u| u.as_str())
            .unwrap_or("");

        let expected_unit = Unit::from_str(unit_str).unwrap_or(Unit::Dimensionless);

        let tol = Tolerance::from_json_or_default(
            correct_answer.get("tolerance"),
            if expected_unit.dimension().amount != 0 {
                Tolerance::default_chemistry()
            } else if !expected_unit.dimension().is_dimensionless() {
                Tolerance::default_physics()
            } else {
                Tolerance::default_math()
            },
        );

        Self::evaluate(
            student_input,
            expected_val,
            expected_unit,
            tol,
            time_taken_ms,
            target_time_ms,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unit_answer_validator_physics_conversions() {
        let tol = Tolerance::default_physics();
        let res1 = UnitAnswerValidator::evaluate(
            &serde_json::json!("72 km/h"),
            20.0,
            Unit::MeterPerSecond,
            tol,
            15000,
            30000,
        );
        assert!(res1.is_correct);
        assert_eq!(res1.score, 1.0);

        let res2 = UnitAnswerValidator::evaluate(
            &serde_json::json!("20 m/s"),
            20.0,
            Unit::MeterPerSecond,
            tol,
            15000,
            30000,
        );
        assert!(res2.is_correct);

        let res3 = UnitAnswerValidator::evaluate(
            &serde_json::json!("72"),
            20.0,
            Unit::MeterPerSecond,
            tol,
            15000,
            30000,
        );
        assert!(!res3.is_correct);
        assert_eq!(res3.error_category, Some(ErrorCategory::Unit));

        let res4 = UnitAnswerValidator::evaluate(
            &serde_json::json!("20 kg"),
            20.0,
            Unit::MeterPerSecond,
            tol,
            15000,
            30000,
        );
        assert!(!res4.is_correct);
        assert_eq!(res4.error_category, Some(ErrorCategory::Unit));
    }

    #[test]
    fn test_unit_answer_validator_chemistry_conversions() {
        let tol = Tolerance::default_chemistry();
        let res1 = UnitAnswerValidator::evaluate(
            &serde_json::json!("1.2 mM"),
            0.0012,
            Unit::Molar,
            tol,
            10000,
            25000,
        );
        assert!(res1.is_correct);

        let res2 = UnitAnswerValidator::evaluate(
            &serde_json::json!("1.2e-3 mol/L"),
            0.0012,
            Unit::Molar,
            tol,
            10000,
            25000,
        );
        assert!(res2.is_correct);

        let res3 = UnitAnswerValidator::evaluate(
            &serde_json::json!("2500 g"),
            2.5,
            Unit::Kilogram,
            tol,
            10000,
            25000,
        );
        assert!(res3.is_correct);

        let res4 = UnitAnswerValidator::evaluate(
            &serde_json::json!("-2.5 mol"),
            2.5,
            Unit::Mole,
            tol,
            10000,
            25000,
        );
        assert!(!res4.is_correct);
        assert_eq!(res4.error_category, Some(ErrorCategory::Concept));
    }
}
