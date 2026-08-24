// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use serde::{Deserialize, Serialize};

use crate::diagnostics::ErrorCategory;
use crate::problems::ProblemInstance;

/// Result of deterministic evaluation of a student's answer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnswerEvaluation {
    pub is_correct: bool,
    pub score: f64,
    pub parsed_student_value: Option<f64>,
    pub canonical_value: f64,
    pub error_category: Option<ErrorCategory>,
    pub diagnostic_message: Option<String>,
}

impl AnswerEvaluation {
    pub fn correct(score: f64, _time_taken_ms: u64, _target_time_ms: u64) -> Self {
        Self {
            is_correct: true,
            score,
            parsed_student_value: None,
            canonical_value: 0.0,
            error_category: None,
            diagnostic_message: None,
        }
    }

    pub fn incorrect(error_category: ErrorCategory, diagnostic: impl Into<String>) -> Self {
        Self {
            is_correct: false,
            score: 0.0,
            parsed_student_value: None,
            canonical_value: 0.0,
            error_category: Some(error_category),
            diagnostic_message: Some(diagnostic.into()),
        }
    }

    pub fn with_parsed_values(mut self, student_val: f64, canonical_val: f64) -> Self {
        self.parsed_student_value = Some(student_val);
        self.canonical_value = canonical_val;
        self
    }

    pub fn with_diagnostic(mut self, msg: impl Into<String>) -> Self {
        self.diagnostic_message = Some(msg.into());
        self
    }
}

/// Domain-agnostic validator interface for evaluating problem instance answers.
pub trait ProblemValidator: Send + Sync {
    /// Unique canonical problem family ID (e.g. "family.math.algebra.linear_equations")
    fn family_id(&self) -> &str;

    /// Deterministically evaluate a student's answer against a problem instance.
    fn evaluate(
        &self,
        instance: &ProblemInstance,
        student_input: &serde_json::Value,
        time_taken_ms: u64,
        target_time_ms: u64,
    ) -> AnswerEvaluation;

    /// Stepwise evaluation against a problem instance's solution graph (default implementation provided).
    fn evaluate_stepwise(
        &self,
        instance: &ProblemInstance,
        submission: &crate::problems::steps::StepwiseSubmission,
        target_time_ms: u64,
    ) -> crate::problems::steps::StepGraphEvaluation {
        if let Some(graph) = instance.solution_graph() {
            crate::problems::steps::StepValidator::evaluate_submission(&graph, submission, target_time_ms)
        } else {
            let ans_json = submission
                .final_answer
                .as_ref()
                .map(|s| serde_json::json!(s))
                .unwrap_or(serde_json::Value::Null);

            let ans_eval = self.evaluate(instance, &ans_json, submission.total_time_ms, target_time_ms);
            crate::problems::steps::StepGraphEvaluation {
                is_correct: ans_eval.is_correct,
                score: ans_eval.score,
                first_error_step: None,
                first_error_type: ans_eval.error_category.map(|cat| match cat {
                    ErrorCategory::Concept | ErrorCategory::Conceptual => {
                        crate::problems::steps::StepErrorType::FormulaSelectionError
                    }
                    ErrorCategory::Strategy => crate::problems::steps::StepErrorType::TransformationError,
                    ErrorCategory::Calculation => crate::problems::steps::StepErrorType::ArithmeticError,
                    ErrorCategory::Sign => crate::problems::steps::StepErrorType::SignError,
                    ErrorCategory::Unit => crate::problems::steps::StepErrorType::UnitError,
                    _ => crate::problems::steps::StepErrorType::Unknown,
                }),
                confidence: crate::problems::steps::DiagnosticConfidence::Deterministic,
                steps_completed: submission.steps.len(),
                steps_correct: if ans_eval.is_correct { 1 } else { 0 },
                step_evaluations: Vec::new(),
                overall_feedback: ans_eval.diagnostic_message.unwrap_or_else(|| {
                    if ans_eval.is_correct {
                        "✓ Correct answer".to_string()
                    } else {
                        "Incorrect answer".to_string()
                    }
                }),
                remediation_recommendation: None,
                first_action_latency_ms: submission.first_action_latency_ms,
                step_latencies_ms: submission.steps.iter().map(|s| s.time_taken_ms).collect(),
            }
        }
    }
}

/// Standard numeric parsing helpers for mathematics domain answers.
pub struct NumericAnswerParser;

impl NumericAnswerParser {
    /// Extract a numeric value from string or JSON Value.
    pub fn parse_value(input: &serde_json::Value) -> Option<f64> {
        match input {
            serde_json::Value::Number(n) => n.as_f64(),
            serde_json::Value::String(s) => Self::parse_string(s),
            serde_json::Value::Object(map) => {
                if let Some(val) = map.get("value").or_else(|| map.get("answer")) {
                    Self::parse_value(val)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Alias for parse_value for backwards compatibility with student submissions.
    pub fn parse_student_answer(input: &serde_json::Value) -> Option<f64> {
        Self::parse_value(input)
    }

    /// Extract numeric value from string containing symbols like $, %, commas, +, fractions (e.g. "3/4"), 
    /// scientific notation, physical units (e.g. "12 m/s", "5 kg"), and simple variable assignment prefixes ("v = 12").
    pub fn parse_string(s: &str) -> Option<f64> {
        let mut trimmed = s.trim();
        if trimmed.is_empty() {
            return None;
        }

        // 1. Strip leading simple variable assignment prefix (e.g. "v = ", "ans = ", "x: ")
        // Must be purely alphabetic identifier (no digits or operations in LHS, e.g. NOT "2x = ")
        if let Some(pos) = trimmed.find(|c| c == '=' || c == ':') {
            let prefix = trimmed[..pos].trim();
            if !prefix.is_empty() && prefix.chars().all(|c| c.is_alphabetic() || c == '_' || c.is_whitespace()) {
                trimmed = trimmed[pos + 1..].trim();
            } else {
                // Complex LHS with digits/operators (like "2x = 10") is an equation, not a raw number
                return None;
            }
        }

        // 2. Remove currencies and common formatting
        let cleaned = trimmed
            .replace('$', "")
            .replace('€', "")
            .replace('£', "")
            .replace('₹', "")
            .replace('%', "")
            .replace(',', "")
            .replace(' ', "");

        if cleaned.is_empty() {
            return None;
        }

        // 3. Check for standard scientific notation with x10^ or *10^ or ×10^
        let lower = cleaned.to_lowercase();
        for marker in &["x10^", "*10^", "×10^", "x10", "*10", "×10"] {
            if let Some(pos) = lower.find(marker) {
                let mantissa_str = &lower[..pos];
                let exponent_str = &lower[pos + marker.len()..];
                let exp_digits: String = exponent_str
                    .chars()
                    .take_while(|c| c.is_ascii_digit() || *c == '-' || *c == '+')
                    .collect();
                if let (Ok(m), Ok(e)) = (mantissa_str.parse::<f64>(), exp_digits.parse::<i32>()) {
                    return Some(m * 10f64.powi(e));
                }
            }
        }

        // 4. Check for fraction format: "a/b" or "a/b m/s"
        if let Some((num_part, den_part)) = cleaned.split_once('/') {
            let num_clean = num_part.trim();
            let den_digits: String = den_part
                .chars()
                .take_while(|c| c.is_ascii_digit() || *c == '.' || *c == '-' || *c == '+')
                .collect();
            if !num_clean.is_empty() && !den_digits.is_empty() {
                if let (Ok(num), Ok(den)) = (num_clean.parse::<f64>(), den_digits.parse::<f64>()) {
                    if den.abs() > f64::EPSILON {
                        return Some(num / den);
                    }
                }
            }
        }

        // 5. Direct parse attempt (handles standard floats and standard "1.2e-3")
        if let Ok(val) = cleaned.parse::<f64>() {
            return Some(val);
        }

        // 6. Extract leading float and ignore trailing physical unit tokens (e.g. "12m/s", "5kg", "150J", "1.2e-3mol/L")
        let mut chars = cleaned.chars().peekable();
        let mut num_str = String::new();
        if let Some(&c) = chars.peek() {
            if c == '+' || c == '-' {
                num_str.push(chars.next().unwrap());
            }
        }
        let mut has_dot = false;
        let mut has_exp = false;
        while let Some(&c) = chars.peek() {
            if c.is_ascii_digit() {
                num_str.push(chars.next().unwrap());
            } else if c == '.' && !has_dot && !has_exp {
                has_dot = true;
                num_str.push(chars.next().unwrap());
            } else if (c == 'e' || c == 'E') && !has_exp && !num_str.is_empty() {
                has_exp = true;
                num_str.push(chars.next().unwrap());
                if let Some(&sign) = chars.peek() {
                    if sign == '+' || sign == '-' {
                        num_str.push(chars.next().unwrap());
                    }
                }
            } else {
                break;
            }
        }

        if num_str.chars().any(|c| c.is_ascii_digit()) {
            let remainder: String = chars.collect();
            let rem_lower = remainder.to_lowercase();
            // Valid unit characters include standard units, degree, slashes, exponents
            let is_valid_unit = !rem_lower.is_empty() && rem_lower.chars().all(|c| {
                c.is_alphabetic() || c == '/' || c == '^' || c == '°' || c == '*' || c == 'Ω' || c == 'ω' || c.is_ascii_digit()
            });
            // Reject if remainder is purely algebraic (e.g. single variable x, y, z without unit context) or contains '='
            if is_valid_unit && !rem_lower.contains('=') && rem_lower != "x" && rem_lower != "y" && rem_lower != "z" {
                if let Ok(val) = num_str.parse::<f64>() {
                    return Some(val);
                }
            }
        }

        None
    }
}

/// Validator for Successive Percentage problem family.
pub struct PercentageSuccessiveValidator;

impl PercentageSuccessiveValidator {
    pub const FLOAT_TOLERANCE: f64 = 0.01;

    /// Parse numerical student answer from string or JSON Value (backward compatible).
    pub fn parse_student_answer(input: &serde_json::Value) -> Option<f64> {
        NumericAnswerParser::parse_value(input)
    }

    /// Extract numeric value from string containing symbols (backward compatible).
    pub fn parse_numeric_string(s: &str) -> Option<f64> {
        NumericAnswerParser::parse_string(s)
    }

    /// Deterministically evaluate a student's answer (backward compatible static method).
    pub fn evaluate(
        correct_answer: &serde_json::Value,
        parameters: &serde_json::Value,
        student_input: &serde_json::Value,
        _time_taken_ms: u64,
        _target_time_ms: u64,
    ) -> AnswerEvaluation {
        let expected_val = correct_answer
            .get("value")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let parsed_val = Self::parse_student_answer(student_input);

        let Some(student_num) = parsed_val else {
            return AnswerEvaluation {
                is_correct: false,
                score: 0.0,
                parsed_student_value: None,
                canonical_value: expected_val,
                error_category: Some(ErrorCategory::Calculation),
                diagnostic_message: Some(
                    "Invalid input format: answer could not be parsed as a number.".to_string(),
                ),
            };
        };

        let diff = (student_num - expected_val).abs();
        let is_correct = diff <= Self::FLOAT_TOLERANCE;

        if is_correct {
            AnswerEvaluation {
                is_correct: true,
                score: 1.0,
                parsed_student_value: Some(student_num),
                canonical_value: expected_val,
                error_category: None,
                diagnostic_message: None,
            }
        } else {
            let (category, message) = Self::classify_misconception(student_num, parameters, expected_val);

            AnswerEvaluation {
                is_correct: false,
                score: 0.0,
                parsed_student_value: Some(student_num),
                canonical_value: expected_val,
                error_category: Some(category),
                diagnostic_message: Some(message),
            }
        }
    }

    /// Classify mistake without guessing: checks known deterministic misconceptions.
    pub fn classify_misconception(
        student_val: f64,
        parameters: &serde_json::Value,
        expected_val: f64,
    ) -> (ErrorCategory, String) {
        let initial_val = parameters.get("initial_value").and_then(|v| v.as_f64());
        let variant_str = parameters.get("variant").and_then(|v| v.as_str()).unwrap_or("");
        let steps = parameters.get("steps").and_then(|v| v.as_array());

        // 1. Additive percentage fallacy check (Concept error)
        if let (Some(init), Some(step_arr)) = (initial_val, steps) {
            if step_arr.len() >= 2 {
                let s1_pct = step_arr[0].get("percent").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let s1_dir = step_arr[0].get("direction").and_then(|v| v.as_str()).unwrap_or("");
                let s1_sign = if s1_dir == "decrease" { -1.0 } else { 1.0 };

                let s2_pct = step_arr[1].get("percent").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let s2_dir = step_arr[1].get("direction").and_then(|v| v.as_str()).unwrap_or("");
                let s2_sign = if s2_dir == "decrease" { -1.0 } else { 1.0 };

                let net_additive_pct = (s1_sign * s1_pct) + (s2_sign * s2_pct);

                if variant_str == "net_equivalent_change" {
                    if (student_val - net_additive_pct).abs() <= Self::FLOAT_TOLERANCE {
                        return (
                            ErrorCategory::Concept,
                            format!(
                                "Additive fallacy: Rates were simply added ({:+.0}%) instead of combined multiplicatively.",
                                net_additive_pct
                            ),
                        );
                    }
                } else if variant_str == "forward_two_step" || variant_str == "level_1_simple" || variant_str == "level_2_mixed" {
                    let additive_final = init * (1.0 + net_additive_pct / 100.0);
                    if (student_val - additive_final).abs() <= Self::FLOAT_TOLERANCE {
                        return (
                            ErrorCategory::Concept,
                            "Additive fallacy: Percentage changes were added together rather than applied successively to intermediate values.".to_string(),
                        );
                    }
                }

                // 2. Intermediate step answer check (Careless error)
                let step1_val = init * (1.0 + (s1_sign * s1_pct / 100.0));
                if (student_val - step1_val).abs() <= Self::FLOAT_TOLERANCE {
                    return (
                        ErrorCategory::Careless,
                        "Incomplete calculation: Answer matches value after step 1 without applying the second change.".to_string(),
                    );
                }

                // 3. Reversed direction / Sign error check (Strategy error)
                let reversed_s1 = init * (1.0 - (s1_sign * s1_pct / 100.0)) * (1.0 + (s2_sign * s2_pct / 100.0));
                let reversed_s2 = init * (1.0 + (s1_sign * s1_pct / 100.0)) * (1.0 - (s2_sign * s2_pct / 100.0));
                if (student_val - reversed_s1).abs() <= Self::FLOAT_TOLERANCE
                    || (student_val - reversed_s2).abs() <= Self::FLOAT_TOLERANCE
                {
                    return (
                        ErrorCategory::Strategy,
                        "Sign error: Inverted the increase/decrease direction of one of the percentage steps.".to_string(),
                    );
                }
            }
        }

        // 4. Reverse problem fallacy check (Concept error)
        if variant_str == "reverse_initial" || variant_str == "level_3_reverse" {
            let final_val = parameters.get("final_value").and_then(|v| v.as_f64());
            if let (Some(fin), Some(step_arr)) = (final_val, steps) {
                if step_arr.len() >= 2 {
                    let mut mult = 1.0;
                    for s in step_arr {
                        let pct = s.get("percent").and_then(|v| v.as_f64()).unwrap_or(0.0);
                        let dir = s.get("direction").and_then(|v| v.as_str()).unwrap_or("");
                        let sign = if dir == "decrease" { -1.0 } else { 1.0 };
                        mult *= 1.0 + (sign * pct / 100.0);
                    }
                    let wrong_multiplied = fin * mult;
                    if (student_val - wrong_multiplied).abs() <= Self::FLOAT_TOLERANCE {
                        return (
                            ErrorCategory::Concept,
                            "Inverse operation error: Multiplied the final value instead of dividing to recover initial value.".to_string(),
                        );
                    }
                }
            }
        }

        // 5. Calculation error if within plausible numerical range
        if student_val > 0.0 && student_val < expected_val * 10.0 {
            (
                ErrorCategory::Calculation,
                format!("Calculation error: Expected {} but received {}.", expected_val, student_val),
            )
        } else {
            // 6. Unknown fallback
            (
                ErrorCategory::Unknown,
                format!("Incorrect answer: Expected {}.", expected_val),
            )
        }
    }
}

impl ProblemValidator for PercentageSuccessiveValidator {
    fn family_id(&self) -> &str {
        "family.math.percentage.successive"
    }

    fn evaluate(
        &self,
        instance: &ProblemInstance,
        student_input: &serde_json::Value,
        time_taken_ms: u64,
        target_time_ms: u64,
    ) -> AnswerEvaluation {
        Self::evaluate(
            &instance.correct_answer,
            &instance.parameters,
            student_input,
            time_taken_ms,
            target_time_ms,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_numeric_strings_and_fractions() {
        assert_eq!(NumericAnswerParser::parse_string("540"), Some(540.0));
        assert_eq!(NumericAnswerParser::parse_string("$540.00"), Some(540.0));
        assert_eq!(NumericAnswerParser::parse_string("+32%"), Some(32.0));
        assert_eq!(NumericAnswerParser::parse_string("-8%"), Some(-8.0));
        assert_eq!(NumericAnswerParser::parse_string("  1,200.50  "), Some(1200.5));
        assert_eq!(NumericAnswerParser::parse_string("3/4"), Some(0.75));
        assert_eq!(NumericAnswerParser::parse_string("-5/2"), Some(-2.5));
        assert_eq!(NumericAnswerParser::parse_string("abc"), None);
    }

    #[test]
    fn test_evaluate_correct_and_time() {
        let correct_ans = serde_json::json!({ "value": 540.0 });
        let params = serde_json::json!({
            "variant": "forward_two_step",
            "initial_value": 500.0,
            "steps": [
                { "percent": 20.0, "direction": "increase" },
                { "percent": 10.0, "direction": "decrease" }
            ]
        });

        // Correct on time
        let res = PercentageSuccessiveValidator::evaluate(
            &correct_ans,
            &params,
            &serde_json::json!("540"),
            25000,
            45000,
        );
        assert!(res.is_correct);
        assert_eq!(res.score, 1.0);
        assert_eq!(res.error_category, None);

        // Correct but slow
        let res_slow = PercentageSuccessiveValidator::evaluate(
            &correct_ans,
            &params,
            &serde_json::json!("$540.00"),
            55000,
            45000,
        );
        assert!(res_slow.is_correct);
        assert_eq!(res_slow.score, 1.0);
        assert_eq!(res_slow.error_category, None);
    }

    #[test]
    fn test_classify_additive_fallacy() {
        let correct_ans = serde_json::json!({ "value": 540.0 });
        let params = serde_json::json!({
            "variant": "forward_two_step",
            "initial_value": 500.0,
            "steps": [
                { "percent": 20.0, "direction": "increase" },
                { "percent": 10.0, "direction": "decrease" }
            ]
        });

        // 500 * (1 + 0.20 - 0.10) = 500 * 1.10 = 550
        let res = PercentageSuccessiveValidator::evaluate(
            &correct_ans,
            &params,
            &serde_json::json!("550"),
            20000,
            45000,
        );
        assert!(!res.is_correct);
        assert_eq!(res.error_category, Some(ErrorCategory::Concept));
        assert!(res.diagnostic_message.unwrap().contains("Additive fallacy"));
    }

    #[test]
    fn test_classify_intermediate_step_careless() {
        let correct_ans = serde_json::json!({ "value": 540.0 });
        let params = serde_json::json!({
            "variant": "forward_two_step",
            "initial_value": 500.0,
            "steps": [
                { "percent": 20.0, "direction": "increase" },
                { "percent": 10.0, "direction": "decrease" }
            ]
        });

        // 500 * 1.20 = 600 (stopped after step 1)
        let res = PercentageSuccessiveValidator::evaluate(
            &correct_ans,
            &params,
            &serde_json::json!("600"),
            15000,
            45000,
        );
        assert!(!res.is_correct);
        assert_eq!(res.error_category, Some(ErrorCategory::Careless));
        assert!(res.diagnostic_message.unwrap().contains("Incomplete"));
    }

    #[test]
    fn test_numeric_answer_parser_units_and_formats() {
        assert_eq!(NumericAnswerParser::parse_string("12 m/s"), Some(12.0));
        assert_eq!(NumericAnswerParser::parse_string("5 kg"), Some(5.0));
        assert_eq!(NumericAnswerParser::parse_string("2.5 mol"), Some(2.5));
        assert_eq!(NumericAnswerParser::parse_string("150 J"), Some(150.0));
        assert_eq!(NumericAnswerParser::parse_string("v = 12 m/s"), Some(12.0));
        assert_eq!(NumericAnswerParser::parse_string("ans: 42"), Some(42.0));
        assert_eq!(NumericAnswerParser::parse_string("3/4 m/s"), Some(0.75));
        assert_eq!(NumericAnswerParser::parse_string("1.2e-3 mol/L"), Some(0.0012));
        assert_eq!(NumericAnswerParser::parse_string("3x10^4"), Some(30000.0));
        assert_eq!(NumericAnswerParser::parse_string("-9.8 m/s^2"), Some(-9.8));
    }
}
