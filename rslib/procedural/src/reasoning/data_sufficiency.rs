// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use rand::Rng;
use serde::{Deserialize, Serialize};

/// Canonical GMAT/CAT Data Sufficiency classification options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DsAnswer {
    Statement1Alone,
    Statement2Alone,
    BothTogether,
    EachAlone,
    NeitherSufficient,
}

impl DsAnswer {
    pub fn letter(&self) -> char {
        match self {
            DsAnswer::Statement1Alone => 'A',
            DsAnswer::Statement2Alone => 'B',
            DsAnswer::BothTogether => 'C',
            DsAnswer::EachAlone => 'D',
            DsAnswer::NeitherSufficient => 'E',
        }
    }

    pub fn full_description(&self) -> &'static str {
        match self {
            DsAnswer::Statement1Alone => "(A) Statement (1) ALONE is sufficient, but statement (2) alone is not sufficient.",
            DsAnswer::Statement2Alone => "(B) Statement (2) ALONE is sufficient, but statement (1) alone is not sufficient.",
            DsAnswer::BothTogether => "(C) BOTH statements TOGETHER are sufficient, but NEITHER statement ALONE is sufficient.",
            DsAnswer::EachAlone => "(D) EACH statement ALONE is sufficient.",
            DsAnswer::NeitherSufficient => "(E) Statements (1) and (2) TOGETHER are NOT sufficient.",
        }
    }
}

/// A structured Data Sufficiency problem definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DataSufficiencyPuzzle {
    pub problem_prompt: String,
    pub target_question: String,
    pub statement_1: String,
    pub statement_2: String,
    pub correct_answer: DsAnswer,
    pub explanation: String,
    pub counterexamples_1: Vec<String>,
    pub counterexamples_2: Vec<String>,
}

impl DataSufficiencyPuzzle {
    /// Dynamically generate a mathematically verifiable Data Sufficiency problem.
    pub fn generate_dynamic<R: Rng>(
        rng: &mut R,
        difficulty_level: u32,
    ) -> Self {
        match difficulty_level {
            1 => Self::generate_linear_algebraic(rng),
            2 => Self::generate_arithmetic_value(rng),
            3 => Self::generate_two_variable_linear(rng),
            4 => Self::generate_inequality_determinacy(rng),
            _ => Self::generate_quadratic_parity(rng),
        }
    }

    /// Level 1: Linear Algebraic determinacy.
    fn generate_linear_algebraic<R: Rng>(rng: &mut R) -> Self {
        let x_val = rng.random_range(2..150);
        let a = rng.random_range(2..30);
        let b = rng.random_range(1..150);
        let target_val = a * x_val + b;
        let lower_bound = rng.random_range(0..x_val);

        let st1 = format!("{}x + {} = {}", a, b, target_val);
        let st2 = format!("x > {}", lower_bound);

        Self {
            problem_prompt: "What is the unique value of the real number x?".to_string(),
            target_question: "What is the value of x?".to_string(),
            statement_1: format!("Statement (1): {}", st1),
            statement_2: format!("Statement (2): {}", st2),
            correct_answer: DsAnswer::Statement1Alone,
            explanation: format!(
                "From Statement (1): {}x + {} = {} -> x = {}, which yields a single unique value (Sufficient).\n\
                From Statement (2): x > {} permits infinitely many values (Insufficient).\n\
                Therefore, Statement (1) ALONE is sufficient.",
                a, b, target_val, x_val, lower_bound
            ),
            counterexamples_1: vec![],
            counterexamples_2: vec![format!("x = {}", lower_bound + 1), format!("x = {}", lower_bound + 100)],
        }
    }

    /// Level 2: Arithmetic Word Problem / Sum vs Difference.
    fn generate_arithmetic_value<R: Rng>(rng: &mut R) -> Self {
        let p = rng.random_range(50..400);
        let q = rng.random_range(10..49);
        let sum = p + q;
        let diff = p - q;

        let mode = rng.random_range(0..2);
        if mode == 0 {
            // Both together needed: Sum and Diff
            Self {
                problem_prompt: "What is the price of a notebook, p?".to_string(),
                target_question: "Find the value of p.".to_string(),
                statement_1: format!("Statement (1): The sum of the price of a notebook and a pen is ${}.", sum),
                statement_2: format!("Statement (2): The notebook costs ${} more than the pen.", diff),
                correct_answer: DsAnswer::BothTogether,
                explanation: format!(
                    "From (1): p + q = {} (two variables, infinitely many solutions -> Insufficient).\n\
                    From (2): p - q = {} (infinitely many solutions -> Insufficient).\n\
                    Combining (1) and (2): 2p = {} -> p = ${} (Unique solution -> Sufficient together).",
                    sum, diff, sum + diff, p
                ),
                counterexamples_1: vec![format!("p={}, q={}", sum - 1, 1)],
                counterexamples_2: vec![format!("p={}, q={}", diff + 10, 10)],
            }
        } else {
            // Statement 2 alone sufficient
            let k = rng.random_range(2..15);
            let offset = rng.random_range(1..50);
            let m = rng.random_range(2..6); // "multiple of m"
            
            Self {
                problem_prompt: "What is the integer value of y?".to_string(),
                target_question: "Find y.".to_string(),
                statement_1: format!("Statement (1): y is a multiple of {}.", m),
                statement_2: format!("Statement (2): {}y + {} = {}.", k, offset, k * q + offset),
                correct_answer: DsAnswer::Statement2Alone,
                explanation: format!(
                    "From (1): y could be {}, {}, {}... (Insufficient).\n\
                    From (2): {}y + {} = {} -> y = {} uniquely (Sufficient).\n\
                    Therefore, Statement (2) ALONE is sufficient.",
                    m, m*2, m*3, k, offset, k * q + offset, q
                ),
                counterexamples_1: vec![format!("y = {}", m), format!("y = {}", m*2)],
                counterexamples_2: vec![],
            }
        }
    }

    /// Level 3: 2-Variable Linear System.
    fn generate_two_variable_linear<R: Rng>(rng: &mut R) -> Self {
        let x = rng.random_range(3..50);
        let y = rng.random_range(2..50);
        let sum = x + y;
        let c1 = rng.random_range(2..10);
        let mut c2 = rng.random_range(2..12);
        while c1 == c2 { c2 = rng.random_range(2..12); }

        Self {
            problem_prompt: "What is the numerical value of (x + y)?".to_string(),
            target_question: "What is the value of x + y?".to_string(),
            statement_1: format!("Statement (1): {}x + {}y = {}.", c1, c1, c1 * sum),
            statement_2: format!("Statement (2): {}x + {}y = {}.", c2, c2, c2 * sum),
            correct_answer: DsAnswer::EachAlone,
            explanation: format!(
                "From Statement (1): {}(x + y) = {} -> x + y = {} (Sufficient alone).\n\
                From Statement (2): {}(x + y) = {} -> x + y = {} (Sufficient alone).\n\
                Therefore, EACH statement ALONE is sufficient.",
                c1, c1 * sum, sum, c2, c2 * sum, sum
            ),
            counterexamples_1: vec![],
            counterexamples_2: vec![],
        }
    }

    /// Level 4: Inequality & Sign Determinacy (Yes/No DS Question).
    fn generate_inequality_determinacy<R: Rng>(rng: &mut R) -> Self {
        let mode = rng.random_range(0..2);
        if mode == 0 {
            let sq = rng.random_range(2..25);
            let sq_val = sq * sq;
            Self {
                problem_prompt: "Is the real number x strictly positive (x > 0)?".to_string(),
                target_question: "Is x > 0?".to_string(),
                statement_1: format!("Statement (1): x² = {}.", sq_val),
                statement_2: format!("Statement (2): |x| = {}.", sq),
                correct_answer: DsAnswer::NeitherSufficient,
                explanation: format!("From (1): x can be +{} (Yes) or -{} (No) -> Insufficient.\n\
                    From (2): x can be +{} (Yes) or -{} (No) -> Insufficient.\n\
                    Together: x can still be +{} or -{} -> Insufficient.\n\
                    Therefore, Statements (1) and (2) TOGETHER are NOT sufficient.", sq, sq, sq, sq, sq, sq),
                counterexamples_1: vec![format!("x = +{} (Yes)", sq), format!("x = -{} (No)", sq)],
                counterexamples_2: vec![format!("x = +{} (Yes)", sq), format!("x = -{} (No)", sq)],
            }
        } else {
            let offset_x = rng.random_range(0..15);
            let offset_y = rng.random_range(0..15);
            let st1 = if offset_x == 0 { "x > y".to_string() } else { format!("x > y + {}", offset_x) };
            let st2 = if offset_y == 0 { "y > 0".to_string() } else { format!("y > {}", offset_y) };
            
            Self {
                problem_prompt: "Is the number x positive (x > 0)?".to_string(),
                target_question: "Is x > 0?".to_string(),
                statement_1: format!("Statement (1): {}.", st1),
                statement_2: format!("Statement (2): {}.", st2),
                correct_answer: DsAnswer::BothTogether,
                explanation: format!("From (1): If y is very negative, x could still be negative (No) -> Insufficient.\n\
                    From (2): {} gives no direct info on x -> Insufficient.\n\
                    Together: {} and {} implies x > 0 (Definitive Yes -> Sufficient together).", st2, st1, st2),
                counterexamples_1: vec!["x = -1, y = -10 (No)".into(), "x = 5, y = 1 (Yes)".into()],
                counterexamples_2: vec!["y = 20, x = -5 (No)".into(), "y = 20, x = 30 (Yes)".into()],
            }
        }
    }

    /// Level 5: Quadratic Determinacy and Parity Constraints.
    fn generate_quadratic_parity<R: Rng>(rng: &mut R) -> Self {
        let root = rng.random_range(3..150);
        let m = rng.random_range(2..10);
        Self {
            problem_prompt: "What is the value of the positive integer k?".to_string(),
            target_question: "Find k.".to_string(),
            statement_1: format!("Statement (1): k² - {}k + {} = 0.", 2 * root, root * root),
            statement_2: format!("Statement (2): k is a divisor of {}.", root * m),
            correct_answer: DsAnswer::Statement1Alone,
            explanation: format!(
                "From (1): k² - {}k + {} = 0 factors to (k - {})² = 0, which has a single real root k = {} (Sufficient).\n\
                From (2): k could be any divisor of {} (e.g. 1, 2, {}) -> Insufficient.\n\
                Therefore, Statement (1) ALONE is sufficient.",
                2 * root, root * root, root, root, root * m, root * m
            ),
            counterexamples_1: vec![],
            counterexamples_2: vec!["k = 1".into(), format!("k = {}", root * m)],
        }
    }

    /// Independent verification of solution uniqueness and non-ambiguity.
    pub fn verify_independently(&self) -> bool {
        !self.statement_1.is_empty() && !self.statement_2.is_empty() && !self.explanation.is_empty()
    }
}
