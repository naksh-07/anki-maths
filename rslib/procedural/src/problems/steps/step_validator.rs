// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use serde::{Deserialize, Serialize};

use crate::problems::steps::interaction::{SubmittedStep, StepwiseSubmission};
use crate::problems::steps::step_graph::{SolutionGraph, StepNode, StepType};
use crate::problems::validator::NumericAnswerParser;

/// Validation status of an individual submitted step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepValidationStatus {
    /// Step is mathematically sound and advances toward the canonical solution.
    Valid,
    /// Step contains a mathematical, sign, or algebraic error.
    Invalid,
    /// Step is derived correctly from an earlier incorrect intermediate state (downstream consistent).
    PartiallyValid,
    /// Step is mathematically valid but not strictly necessary in the standard path.
    UnnecessaryButValid,
    /// Step cannot be unambiguously classified.
    Unresolved,
}

impl StepValidationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            StepValidationStatus::Valid => "valid",
            StepValidationStatus::Invalid => "invalid",
            StepValidationStatus::PartiallyValid => "partially_valid",
            StepValidationStatus::UnnecessaryButValid => "unnecessary_but_valid",
            StepValidationStatus::Unresolved => "unresolved",
        }
    }
}

/// Precise taxonomic error category identified at the step level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepErrorType {
    /// Selected an inappropriate formula, theorem, or method for this step.
    FormulaSelectionError,
    /// Selected correct concept but set up initial equation/relation incorrectly.
    SetupError,
    /// Performed invalid algebraic manipulation or simplification.
    TransformationError,
    /// Made a pure arithmetic calculation slip (e.g. 7 * 8 = 54).
    ArithmeticError,
    /// Inverted sign, direction, or transposition sign during algebraic rearrangement.
    SignError,
    /// Stopped solving prematurely before completing required transformations.
    PrematureCompletion,
    /// Omitted or incorrectly converted units (e.g. km/h vs m/s).
    UnitError,
    /// Inverted a ratio proportion (e.g. substituted a:b as b:a).
    RatioInversionError,
    /// Swapped alligation weights or concentration differences in mixture calculations.
    AlligationSwapError,
    /// Inverted rate / speed relation (e.g. D = S/T instead of D = S*T).
    RateInversionError,
    /// Failed to reverse inequality sign when multiplying/dividing by negative number.
    InequalitySignFlipError,
    /// Dropped cross term or sign in algebraic identities (e.g. (a+b)^2 vs a^2+b^2).
    IdentityCrossTermError,
    /// Confused triangle legs with hypotenuse in Pythagorean theorem.
    PythagoreanLegConfusion,
    /// Mishandled negative remainder or modular reduction cyclicity.
    ModularReductionError,
    /// Reasoning mathematically correct but final submitted format/notation was invalid.
    FinalAnswerFormattingError,
    // --- Physics-specific step error types ---
    /// Selected wrong physical law/governing model (e.g. uniform motion when acceleration != 0).
    ModelSelectionError,
    /// Incorrect physical state, coordinate system, or diagrammatic representation.
    RepresentationError,
    /// Model correct, but equation construction / relationship setup was incorrect.
    EquationSetupError,
    /// Vector direction or sign convention inverted (e.g. gravity +g vs -g).
    SignConventionError,
    /// Physics model is sound, but algebraic or arithmetic manipulation was wrong.
    AlgebraExecutionError,
    /// Final result violates a fundamental physical constraint (e.g. negative time or mass).
    PhysicalPlausibilityError,
    // --- Chemistry-specific step error types ---
    /// Incorrect chemical species, molecular formula, or molar mass.
    ChemicalRepresentationError,
    /// Chemical reaction balancing error or invalid stoichiometric coefficients.
    EquationBalanceError,
    /// Applied incorrect mole ratio between reactant and product coefficients.
    StoichiometricRatioError,
    /// Incorrectly identified limiting reagent or inverted limiting ratio comparison.
    LimitingReagentError,
    /// Selected inappropriate chemical regime (e.g. mole-ratio vs equilibrium ICE table).
    RegimeSelectionError,
    /// Result violates chemical conservation laws (element conservation, charge, or positive mass).
    ConservationViolationError,
    // --- Reasoning-specific step error types ---
    /// Failed to recognize or identify the correct problem schema.
    SchemaRecognitionError,
    /// Selected an inappropriate strategy or wrong starting constraint.
    StrategySelectionError,
    /// Applied a constraint incorrectly or violated an explicit puzzle rule.
    ConstraintApplicationError,
    /// Made an invalid deductive or relational logical inference.
    InferenceError,
    /// Missed, mishandled, or improperly branched search cases.
    SearchCaseError,
    /// Failed to recognize or handle logical contradiction.
    ContradictionHandlingError,
    /// Fell for a logical trap, distractor, or inverse reading error.
    ReadingTrapError,
    /// Correct strategic reasoning, but arithmetic or clerical operational slip occurred.
    ExecutionSlipError,
    /// Unclassified or fallback error.
    Unknown,
}

impl StepErrorType {
    pub fn as_str(&self) -> &'static str {
        match self {
            StepErrorType::FormulaSelectionError => "formula_selection_error",
            StepErrorType::SetupError => "setup_error",
            StepErrorType::TransformationError => "transformation_error",
            StepErrorType::ArithmeticError => "arithmetic_error",
            StepErrorType::SignError => "sign_error",
            StepErrorType::PrematureCompletion => "premature_completion",
            StepErrorType::UnitError => "unit_error",
            StepErrorType::RatioInversionError => "ratio_inversion_error",
            StepErrorType::AlligationSwapError => "alligation_swap_error",
            StepErrorType::RateInversionError => "rate_inversion_error",
            StepErrorType::InequalitySignFlipError => "inequality_sign_flip_error",
            StepErrorType::IdentityCrossTermError => "identity_cross_term_error",
            StepErrorType::PythagoreanLegConfusion => "pythagorean_leg_confusion",
            StepErrorType::ModularReductionError => "modular_reduction_error",
            StepErrorType::FinalAnswerFormattingError => "final_answer_formatting_error",
            StepErrorType::ModelSelectionError => "model_selection_error",
            StepErrorType::RepresentationError => "representation_error",
            StepErrorType::EquationSetupError => "equation_setup_error",
            StepErrorType::SignConventionError => "sign_convention_error",
            StepErrorType::AlgebraExecutionError => "algebra_execution_error",
            StepErrorType::PhysicalPlausibilityError => "physical_plausibility_error",
            StepErrorType::ChemicalRepresentationError => "chemical_representation_error",
            StepErrorType::EquationBalanceError => "equation_balance_error",
            StepErrorType::StoichiometricRatioError => "stoichiometric_ratio_error",
            StepErrorType::LimitingReagentError => "limiting_reagent_error",
            StepErrorType::RegimeSelectionError => "regime_selection_error",
            StepErrorType::ConservationViolationError => "conservation_violation_error",
            StepErrorType::SchemaRecognitionError => "schema_recognition_error",
            StepErrorType::StrategySelectionError => "strategy_selection_error",
            StepErrorType::ConstraintApplicationError => "constraint_application_error",
            StepErrorType::InferenceError => "inference_error",
            StepErrorType::SearchCaseError => "search_case_error",
            StepErrorType::ContradictionHandlingError => "contradiction_handling_error",
            StepErrorType::ReadingTrapError => "reading_trap_error",
            StepErrorType::ExecutionSlipError => "execution_slip_error",
            StepErrorType::Unknown => "unknown",
        }
    }
}

impl std::fmt::Display for StepErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Confidence / certainty level of the inferred step diagnosis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticConfidence {
    /// Exact mathematical identity or deterministic misconception pattern match.
    Deterministic,
    /// Highly probable mistake classification based on step context and magnitude.
    StronglyInferred,
    /// Plausible classification with limited evidence.
    Uncertain,
}

impl DiagnosticConfidence {
    pub fn as_str(&self) -> &'static str {
        match self {
            DiagnosticConfidence::Deterministic => "deterministic",
            DiagnosticConfidence::StronglyInferred => "strongly_inferred",
            DiagnosticConfidence::Uncertain => "uncertain",
        }
    }
}

/// Detailed evaluation of an individual step attempt.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StepEvaluation {
    pub step_id: String,
    pub step_index: usize,
    pub status: StepValidationStatus,
    pub submitted_text: String,
    pub expected_expression: String,
    pub parsed_value: Option<f64>,
    pub error_type: Option<StepErrorType>,
    pub confidence: DiagnosticConfidence,
    pub feedback: Option<String>,
    pub is_downstream_consistent: bool,
}

/// Comprehensive outcome of evaluating a complete stepwise attempt across a solution graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StepGraphEvaluation {
    pub is_correct: bool,
    pub score: f64,
    /// Index of the first incorrect step (0-indexed), if any error occurred.
    pub first_error_step: Option<usize>,
    /// Error type identified at the first error location.
    pub first_error_type: Option<StepErrorType>,
    pub confidence: DiagnosticConfidence,
    pub steps_completed: usize,
    pub steps_correct: usize,
    pub step_evaluations: Vec<StepEvaluation>,
    pub overall_feedback: String,
    pub remediation_recommendation: Option<String>,
    pub first_action_latency_ms: Option<u64>,
    pub step_latencies_ms: Vec<u64>,
}

impl StepGraphEvaluation {
    /// Extract Chemistry domain diagnostic evidence from stepwise evaluation.
    pub fn to_chemistry_physical_evidence(&self) -> Option<crate::skills::domain_evidence::ChemistryEvidence> {
        if self.step_evaluations.is_empty() {
            return None;
        }

        let first_err_idx = self.first_error_step;
        let is_all_correct = self.is_correct;

        if is_all_correct {
            return Some(crate::skills::domain_evidence::ChemistryEvidence::Physical {
                model_setup: Some(true),
                equation_selection: Some(true),
                intermediate_quantity: Some(true),
                calculation: Some(true),
                conservation: Some(true),
                verification: Some(true),
                transfer: None,
            });
        }

        let (model_setup, intermediate_quantity, calculation, conservation) = match first_err_idx {
            Some(0) => {
                let is_concept = match self.first_error_type {
                    Some(StepErrorType::ChemicalRepresentationError)
                    | Some(StepErrorType::EquationBalanceError)
                    | Some(StepErrorType::RegimeSelectionError)
                    | Some(StepErrorType::FormulaSelectionError)
                    | Some(StepErrorType::SetupError) => true,
                    _ => false,
                };
                if is_concept {
                    (Some(false), Some(false), Some(false), None)
                } else {
                    (Some(true), Some(false), Some(false), Some(true))
                }
            }
            Some(i) if i >= 1 && i < self.step_evaluations.len().saturating_sub(1) => {
                (Some(true), Some(false), Some(false), Some(true))
            }
            _ => {
                let is_cons_violation = self.first_error_type == Some(StepErrorType::ConservationViolationError);
                if is_cons_violation {
                    (Some(true), Some(true), Some(false), Some(false))
                } else {
                    (Some(true), Some(true), Some(false), Some(true))
                }
            }
        };

        Some(crate::skills::domain_evidence::ChemistryEvidence::Physical {
            model_setup,
            equation_selection: Some(true),
            intermediate_quantity,
            calculation,
            conservation,
            verification: Some(false),
            transfer: None,
        })
    }
}

/// Mathematical semantic comparator for lightweight, bounded algebraic and numeric equivalence.
pub struct MathSemanticComparator;

impl MathSemanticComparator {
    pub const FLOAT_TOLERANCE: f64 = 0.01;

    /// Clean and normalize a mathematical expression string for canonical comparison.
    pub fn normalize_expr(expr: &str) -> String {
        expr.trim()
            .replace('\\', "")
            .replace('$', "")
            .replace('€', "")
            .replace('£', "")
            .replace('₹', "")
            .replace('%', "")
            .replace(',', "")
            .replace(' ', "")
            .to_lowercase()
    }

    /// Check if two expressions are equivalent either numerically, algebraically, or literally.
    pub fn is_equivalent(submitted: &str, expected: &str, alternates: &[String], expected_val: Option<f64>) -> bool {
        let norm_sub = Self::normalize_expr(submitted);
        let norm_exp = Self::normalize_expr(expected);

        // 1. Literal normalized match
        if norm_sub == norm_exp {
            return true;
        }

        // 2. Alternate forms match
        for alt in alternates {
            if norm_sub == Self::normalize_expr(alt) {
                return true;
            }
        }

        // 3. Numeric value equivalence
        if let (Some(sub_num), Some(exp_num)) = (
            NumericAnswerParser::parse_string(submitted),
            expected_val.or_else(|| NumericAnswerParser::parse_string(expected)),
        ) {
            if (sub_num - exp_num).abs() <= Self::FLOAT_TOLERANCE {
                return true;
            }
        }

        // 4. Linear equation equivalence (e.g. "ax + b = c" or "x = val" or "val = x")
        if Self::check_equation_equivalence(submitted, expected) {
            return true;
        }

        // 5. Commutative expression match (e.g. "2x + 6" == "6 + 2x")
        if Self::check_commutative_addition(&norm_sub, &norm_exp) {
            return true;
        }

        // 6. Multiplier vs percentage equivalence (e.g. "1.20" vs "120%" vs "+20%")
        if Self::check_multiplier_equivalence(submitted, expected) {
            return true;
        }

        false
    }

    /// Check linear equation semantic equivalence (e.g. "3x = 12", "x = 4", "12 = 3x").
    pub fn check_equation_equivalence(submitted: &str, expected: &str) -> bool {
        let sub_parts: Vec<&str> = submitted.split('=').collect();
        let exp_parts: Vec<&str> = expected.split('=').collect();

        if sub_parts.len() != 2 || exp_parts.len() != 2 {
            return false;
        }

        let sub_lhs = Self::normalize_expr(sub_parts[0]);
        let sub_rhs = Self::normalize_expr(sub_parts[1]);
        let exp_lhs = Self::normalize_expr(exp_parts[0]);
        let exp_rhs = Self::normalize_expr(exp_parts[1]);

        // Exact sides or swapped sides (e.g. x = 4 vs 4 = x)
        if (sub_lhs == exp_lhs && sub_rhs == exp_rhs) || (sub_lhs == exp_rhs && sub_rhs == exp_lhs) {
            return true;
        }

        // Solve standard form Ax = B
        if let (Some((sub_a, sub_b)), Some((exp_a, exp_b))) = (
            Self::parse_linear_one_var(&sub_lhs, &sub_rhs),
            Self::parse_linear_one_var(&exp_lhs, &exp_rhs),
        ) {
            if exp_a.abs() > f64::EPSILON && sub_a.abs() > f64::EPSILON {
                let sub_sol = sub_b / sub_a;
                let exp_sol = exp_b / exp_a;
                if (sub_sol - exp_sol).abs() <= Self::FLOAT_TOLERANCE {
                    return true;
                }
            }
        }

        false
    }

    /// Extract root of algebraic linear equation or parsed numeric value from expression.
    pub fn extract_root_or_value(expr: &str) -> Option<f64> {
        let norm = Self::normalize_expr(expr);
        if let Some((lhs, rhs)) = norm.split_once('=') {
            if let Some((coeff, constant)) = Self::parse_linear_one_var(lhs, rhs) {
                if coeff.abs() > f64::EPSILON {
                    return Some(constant / coeff);
                }
            }
        }
        NumericAnswerParser::parse_string(expr)
    }

    fn parse_linear_one_var(lhs: &str, rhs: &str) -> Option<(f64, f64)> {
        // Simple linear extractor: looking for pattern `Ax + B = C` or `Ax = C`
        // Returns (coeff_x, constant) such that coeff_x * x = constant
        let (lhs_a, lhs_c) = Self::extract_linear_terms(lhs);
        let (rhs_a, rhs_c) = Self::extract_linear_terms(rhs);

        let net_a = lhs_a - rhs_a;
        let net_c = rhs_c - lhs_c;

        if net_a.abs() > f64::EPSILON {
            Some((net_a, net_c))
        } else {
            None
        }
    }

    fn extract_linear_terms(expr: &str) -> (f64, f64) {
        let norm = Self::normalize_expr(expr);
        if norm.is_empty() {
            return (0.0, 0.0);
        }

        // Check simple variable `x` presence
        if !norm.contains('x') {
            let val = NumericAnswerParser::parse_string(&norm).unwrap_or(0.0);
            return (0.0, val);
        }

        let mut coeff = 0.0;
        let mut constant = 0.0;

        // Tokenize by + and - preserving signs
        let mut curr_term = String::new();
        let mut chars = norm.chars().peekable();

        while let Some(ch) = chars.next() {
            if (ch == '+' || ch == '-') && !curr_term.is_empty() {
                Self::accumulate_term(&curr_term, &mut coeff, &mut constant);
                curr_term.clear();
            }
            curr_term.push(ch);
        }
        if !curr_term.is_empty() {
            Self::accumulate_term(&curr_term, &mut coeff, &mut constant);
        }

        (coeff, constant)
    }

    fn accumulate_term(term: &str, coeff: &mut f64, constant: &mut f64) {
        let trimmed = term.trim();
        if trimmed.is_empty() {
            return;
        }

        if trimmed.contains('x') {
            let without_x = trimmed.replace('x', "");
            let c = match without_x.as_str() {
                "" | "+" => 1.0,
                "-" => -1.0,
                other => other.parse::<f64>().unwrap_or(1.0),
            };
            *coeff += c;
        } else if let Ok(val) = trimmed.parse::<f64>() {
            *constant += val;
        }
    }

    pub fn check_commutative_addition(s1: &str, s2: &str) -> bool {
        let mut parts1: Vec<String> = s1.split('+').map(|p| p.trim().to_string()).collect();
        let mut parts2: Vec<String> = s2.split('+').map(|p| p.trim().to_string()).collect();
        if parts1.len() > 1 && parts1.len() == parts2.len() {
            parts1.sort();
            parts2.sort();
            return parts1 == parts2;
        }
        false
    }

    pub fn check_multiplier_equivalence(sub: &str, exp: &str) -> bool {
        let sub_val = NumericAnswerParser::parse_string(sub);
        let exp_val = NumericAnswerParser::parse_string(exp);

        if let (Some(s), Some(e)) = (sub_val, exp_val) {
            // Check decimal vs percentage (e.g. 1.20 vs 120 or 0.20 vs 20)
            if (s * 100.0 - e).abs() <= Self::FLOAT_TOLERANCE || (s - e * 100.0).abs() <= Self::FLOAT_TOLERANCE {
                return true;
            }
        }
        false
    }

    /// Diagnose the likely error type when a submitted step fails validation.
    pub fn diagnose_step_error(
        submitted: &str,
        expected_step: &StepNode,
        prev_submitted: Option<&SubmittedStep>,
    ) -> (StepErrorType, DiagnosticConfidence, String) {
        let sub_val = Self::extract_root_or_value(submitted);
        let exp_val = expected_step
            .expected_value
            .or_else(|| Self::extract_root_or_value(&expected_step.expected_expression));

        // 1. Sign error check
        if let (Some(s), Some(e)) = (sub_val, exp_val) {
            if (s + e).abs() <= Self::FLOAT_TOLERANCE && e.abs() > Self::FLOAT_TOLERANCE {
                return (
                    StepErrorType::SignError,
                    DiagnosticConfidence::Deterministic,
                    format!("Sign reversal detected: Received {:+.2}, expected {:+.2}.", s, e),
                );
            }
        }

        // 2. Check for transposition sign error in linear equations (e.g. 3x = 17 + 5 = 22 instead of 17 - 5 = 12)
        if expected_step.step_type == StepType::EquationRearrangement || expected_step.step_type == StepType::Simplification {
            if submitted.contains('=') {
                return (
                    StepErrorType::SignError,
                    DiagnosticConfidence::StronglyInferred,
                    "Transposition or sign error: Incorrect addition/subtraction across equals sign.".to_string(),
                );
            }
        }

        // 3. Formula selection error
        if expected_step.step_type == StepType::FormulaSelection {
            return (
                StepErrorType::FormulaSelectionError,
                DiagnosticConfidence::StronglyInferred,
                "Formula selection error: Inappropriate formula or principle applied.".to_string(),
            );
        }

        // 4. Arithmetic calculation error
        if let (Some(s), Some(e)) = (sub_val, exp_val) {
            if (s - e).abs() < 20.0 {
                return (
                    StepErrorType::ArithmeticError,
                    DiagnosticConfidence::StronglyInferred,
                    format!("Arithmetic slip: Expected {:.2}, but calculated {:.2}.", e, s),
                );
            }
        }

        // 5. Transformation error
        if expected_step.step_type == StepType::Transformation || expected_step.step_type == StepType::EquationRearrangement {
            return (
                StepErrorType::TransformationError,
                DiagnosticConfidence::StronglyInferred,
                format!(
                    "Transformation error: Expected operation '{}', but received '{}'.",
                    expected_step.description, submitted
                ),
            );
        }

        // 6. Premature completion
        if !expected_step.is_final && prev_submitted.is_none() && expected_step.expected_value.is_some() {
            return (
                StepErrorType::PrematureCompletion,
                DiagnosticConfidence::Uncertain,
                "Premature completion: Stopped before completing all necessary algebraic steps.".to_string(),
            );
        }

        (
            StepErrorType::Unknown,
            DiagnosticConfidence::Uncertain,
            format!("Incorrect step: Expected '{}'.", expected_step.expected_expression),
        )
    }
}

/// Deterministic step-aware validation engine.
pub struct StepValidator;

impl StepValidator {
    /// Evaluate a stepwise submission against a problem instance's solution graph.
    pub fn evaluate_submission(
        graph: &SolutionGraph,
        submission: &StepwiseSubmission,
        target_time_ms: u64,
    ) -> StepGraphEvaluation {
        let mut step_evaluations = Vec::new();
        let mut first_error_step: Option<usize> = None;
        let mut first_error_type: Option<StepErrorType> = None;
        let mut first_error_conf = DiagnosticConfidence::Deterministic;
        let mut prev_step_had_error = false;
        let mut prev_erroneous_value: Option<f64> = None;

        let expected_step_count = graph.step_count();
        let mut correct_steps_count = 0;

        let step_latencies: Vec<u64> = submission.steps.iter().map(|s| s.time_taken_ms).collect();

        // 1. Evaluate each submitted step against expected steps in graph
        for (idx, sub_step) in submission.steps.iter().enumerate() {
            let exp_step = graph.get_step_by_index(idx);

            let Some(expected) = exp_step else {
                // Submitted extra steps beyond graph
                step_evaluations.push(StepEvaluation {
                    step_id: format!("extra_step_{}", idx),
                    step_index: idx,
                    status: StepValidationStatus::UnnecessaryButValid,
                    submitted_text: sub_step.content.clone(),
                    expected_expression: "".to_string(),
                    parsed_value: NumericAnswerParser::parse_string(&sub_step.content),
                    error_type: None,
                    confidence: DiagnosticConfidence::Uncertain,
                    feedback: Some("Additional intermediate step.".to_string()),
                    is_downstream_consistent: false,
                });
                continue;
            };

            let is_valid = MathSemanticComparator::is_equivalent(
                &sub_step.content,
                &expected.expected_expression,
                &expected.alternate_expressions,
                expected.expected_value,
            );

            let parsed_curr = MathSemanticComparator::extract_root_or_value(&sub_step.content);

            if is_valid {
                correct_steps_count += 1;
                step_evaluations.push(StepEvaluation {
                    step_id: expected.id.clone(),
                    step_index: idx,
                    status: StepValidationStatus::Valid,
                    submitted_text: sub_step.content.clone(),
                    expected_expression: expected.expected_expression.clone(),
                    parsed_value: parsed_curr,
                    error_type: None,
                    confidence: DiagnosticConfidence::Deterministic,
                    feedback: Some("✓ Correct step".to_string()),
                    is_downstream_consistent: false,
                });
                prev_step_had_error = false;
                prev_erroneous_value = None;
            } else {
                // Check if this step is downstream consistent with previous error
                let is_downstream = if prev_step_had_error && prev_erroneous_value.is_some() && parsed_curr.is_some() {
                    let prev_val = prev_erroneous_value.unwrap();
                    let curr_val = parsed_curr.unwrap();
                    (prev_val - curr_val).abs() <= MathSemanticComparator::FLOAT_TOLERANCE
                } else {
                    false
                };

                let status = if is_downstream {
                    StepValidationStatus::PartiallyValid
                } else {
                    StepValidationStatus::Invalid
                };

                let prev_sub = if idx > 0 { submission.steps.get(idx - 1) } else { None };
                let (err_type, conf, feedback) =
                    MathSemanticComparator::diagnose_step_error(&sub_step.content, expected, prev_sub);

                if first_error_step.is_none() {
                    first_error_step = Some(idx);
                    first_error_type = Some(err_type);
                    first_error_conf = conf;
                }

                prev_step_had_error = true;
                prev_erroneous_value = parsed_curr;

                step_evaluations.push(StepEvaluation {
                    step_id: expected.id.clone(),
                    step_index: idx,
                    status,
                    submitted_text: sub_step.content.clone(),
                    expected_expression: expected.expected_expression.clone(),
                    parsed_value: parsed_curr,
                    error_type: Some(err_type),
                    confidence: conf,
                    feedback: Some(feedback),
                    is_downstream_consistent: is_downstream,
                });
            }
        }

        // 2. Check final answer if provided directly or exclusively
        let final_step_node = graph.final_step();
        let final_ans_correct = if let Some(ref ans) = submission.final_answer {
            if let Some(fin) = final_step_node {
                MathSemanticComparator::is_equivalent(
                    ans,
                    &fin.expected_expression,
                    &fin.alternate_expressions,
                    fin.expected_value,
                )
            } else {
                false
            }
        } else if let Some(last_eval) = step_evaluations.last() {
            last_eval.status == StepValidationStatus::Valid
        } else {
            false
        };

        // Determine completeness and score
        let all_steps_valid = first_error_step.is_none() && (submission.steps.len() >= expected_step_count || submission.steps.is_empty());
        let is_overall_correct = final_ans_correct && (all_steps_valid || submission.steps.is_empty());

        let score = if is_overall_correct {
            1.0
        } else if correct_steps_count > 0 && expected_step_count > 0 {
            (correct_steps_count as f64 / expected_step_count as f64).min(0.9)
        } else {
            0.0
        };

        let overall_feedback = if is_overall_correct {
            if target_time_ms > 0 && submission.total_time_ms > target_time_ms {
                format!(
                    "✓ All steps correct! Completed in {:.1}s (Target: {:.1}s).",
                    submission.total_time_ms as f64 / 1000.0,
                    target_time_ms as f64 / 1000.0
                )
            } else {
                "✓ Excellent! All procedural steps executed correctly.".to_string()
            }
        } else if let Some(first_err_idx) = first_error_step {
            let err_name = first_error_type.map(|e| e.to_string()).unwrap_or_else(|| "error".to_string());
            format!(
                "First error localized at Step {}: {} ({})",
                first_err_idx + 1,
                step_evaluations.get(first_err_idx).and_then(|e| e.feedback.as_deref()).unwrap_or(""),
                err_name
            )
        } else {
            "Final answer incorrect.".to_string()
        };

        let remediation_recommendation = first_error_type.map(|err| match err {
            StepErrorType::FormulaSelectionError => "remediate:simpler_schema_trigger".to_string(),
            StepErrorType::SetupError => "remediate:guided_problem".to_string(),
            StepErrorType::TransformationError => "remediate:lower_complexity_variant".to_string(),
            StepErrorType::ArithmeticError => "remediate:simpler_numbers_variant".to_string(),
            StepErrorType::SignError => "remediate:sign_focused_variant".to_string(),
            StepErrorType::PrematureCompletion => "remediate:multi_step_guided".to_string(),
            StepErrorType::UnitError => "remediate:unit_conversion_drill".to_string(),
            StepErrorType::RatioInversionError => "remediate:ratio_mapping_drill".to_string(),
            StepErrorType::AlligationSwapError => "remediate:alligation_setup_drill".to_string(),
            StepErrorType::RateInversionError => "remediate:rate_relation_drill".to_string(),
            StepErrorType::InequalitySignFlipError => "remediate:inequality_sign_flip_drill".to_string(),
            StepErrorType::IdentityCrossTermError => "remediate:algebraic_expansion_drill".to_string(),
            StepErrorType::PythagoreanLegConfusion => "remediate:pythagorean_hypotenuse_drill".to_string(),
            StepErrorType::ModularReductionError => "remediate:modular_arithmetic_drill".to_string(),
            StepErrorType::FinalAnswerFormattingError => "remediate:notation_practice".to_string(),
            StepErrorType::ModelSelectionError => "remediate:physics_model_discrimination".to_string(),
            StepErrorType::RepresentationError => "remediate:coordinate_system_setup".to_string(),
            StepErrorType::EquationSetupError => "remediate:equation_formulation_guided".to_string(),
            StepErrorType::SignConventionError => "remediate:vector_sign_convention_drill".to_string(),
            StepErrorType::AlgebraExecutionError => "remediate:algebraic_rearrangement_drill".to_string(),
            StepErrorType::PhysicalPlausibilityError => "remediate:physical_constraint_sanity_check".to_string(),
            StepErrorType::ChemicalRepresentationError => "remediate:chemical_species_representation_drill".to_string(),
            StepErrorType::EquationBalanceError => "remediate:reaction_balancing_drill".to_string(),
            StepErrorType::StoichiometricRatioError => "remediate:stoichiometric_mole_ratio_drill".to_string(),
            StepErrorType::LimitingReagentError => "remediate:limiting_reagent_comparison_drill".to_string(),
            StepErrorType::RegimeSelectionError => "remediate:chemical_regime_discrimination".to_string(),
            StepErrorType::ConservationViolationError => "remediate:chemical_conservation_sanity_check".to_string(),
            StepErrorType::SchemaRecognitionError => "remediate:schema_recognition_drill".to_string(),
            StepErrorType::StrategySelectionError => "remediate:strategy_selection_drill".to_string(),
            StepErrorType::ConstraintApplicationError => "remediate:constraint_propagation_guided".to_string(),
            StepErrorType::InferenceError => "remediate:formal_inference_drill".to_string(),
            StepErrorType::SearchCaseError => "remediate:case_branching_guided".to_string(),
            StepErrorType::ContradictionHandlingError => "remediate:contradiction_detection_drill".to_string(),
            StepErrorType::ReadingTrapError => "remediate:reading_precision_drill".to_string(),
            StepErrorType::ExecutionSlipError => "remediate:operational_accuracy_drill".to_string(),
            StepErrorType::Unknown => "remediate:standard_variant".to_string(),
        });

        StepGraphEvaluation {
            is_correct: is_overall_correct,
            score,
            first_error_step,
            first_error_type,
            confidence: first_error_conf,
            steps_completed: submission.steps.len(),
            steps_correct: correct_steps_count,
            step_evaluations,
            overall_feedback,
            remediation_recommendation,
            first_action_latency_ms: submission.first_action_latency_ms,
            step_latencies_ms: step_latencies,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::problems::steps::step_graph::{StepNode, StepType};

    fn make_test_graph() -> SolutionGraph {
        let step1 = StepNode::new(
            "isolate_x",
            StepType::EquationRearrangement,
            "Isolate variable term",
            "Subtract 6 from both sides",
            "2x = 10",
        )
        .with_alternates(vec!["2x + 6 - 6 = 16 - 6".to_string(), "2x = 16 - 6".to_string()]);

        let step2 = StepNode::new(
            "solve_x",
            StepType::FinalAnswer,
            "Solve for x",
            "Divide both sides by 2",
            "x = 5",
        )
        .with_expected_value(5.0)
        .with_alternates(vec!["5".to_string()])
        .with_dependencies(vec!["isolate_x".to_string()])
        .as_final();

        SolutionGraph::new(vec![step1, step2], "solve_x")
    }

    #[test]
    fn test_step_validator_all_steps_correct() {
        let graph = make_test_graph();
        let steps = vec![
            SubmittedStep::new(0, "2x = 10", 3000),
            SubmittedStep::new(1, "x = 5", 2000),
        ];
        let sub = StepwiseSubmission::stepwise(steps, Some("5".to_string()), 5000);

        let eval = StepValidator::evaluate_submission(&graph, &sub, 25000);
        assert!(eval.is_correct);
        assert_eq!(eval.score, 1.0);
        assert_eq!(eval.first_error_step, None);
        assert_eq!(eval.steps_correct, 2);
        assert_eq!(eval.step_evaluations[0].status, StepValidationStatus::Valid);
        assert_eq!(eval.step_evaluations[1].status, StepValidationStatus::Valid);
    }

    #[test]
    fn test_step_validator_first_error_localization() {
        let graph = make_test_graph();
        // Step 1 wrong: 2x = 12 instead of 2x = 10
        // Step 2: x = 6 (derived correctly from wrong step 1)
        let steps = vec![
            SubmittedStep::new(0, "2x = 12", 4000),
            SubmittedStep::new(1, "x = 6", 3000),
        ];
        let sub = StepwiseSubmission::stepwise(steps, Some("6".to_string()), 7000);

        let eval = StepValidator::evaluate_submission(&graph, &sub, 25000);
        assert!(!eval.is_correct);
        assert_eq!(eval.first_error_step, Some(0));
        assert_eq!(eval.step_evaluations[0].status, StepValidationStatus::Invalid);
        assert_eq!(eval.step_evaluations[1].status, StepValidationStatus::PartiallyValid);
        assert!(eval.overall_feedback.contains("First error localized at Step 1"));
    }

    #[test]
    fn test_math_semantic_comparator_algebraic_equivalence() {
        // "2x + 6 = 16" should be equivalent to "2x = 10" and "x = 5"
        assert!(MathSemanticComparator::check_equation_equivalence("2x + 6 = 16", "2x = 10"));
        assert!(MathSemanticComparator::check_equation_equivalence("x = 5", "5 = x"));
        assert!(MathSemanticComparator::check_equation_equivalence("3x = 12", "x = 4"));
    }
}
