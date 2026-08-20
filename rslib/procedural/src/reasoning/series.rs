// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use serde::{Deserialize, Serialize};

/// Bounded taxonomy of deterministic series pattern operators.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SeriesRule {
    /// Arithmetic progression with constant difference: a_{n} = a_{n-1} + diff.
    ConstantDifference { diff: i64 },
    /// Progressive difference: diff_{k} = start_diff + k * step.
    IncreasingDifference { start_diff: i64, step: i64 },
    /// Geometric progression with integer multiplier: a_{n} = a_{n-1} * ratio.
    Geometric { ratio: i64 },
    /// Alternating operations: +diff1 on odd steps, +diff2 on even steps.
    Alternating { diff1: i64, diff2: i64 },
    /// Differences are consecutive perfect squares: diff_k = (start + k)^2.
    SquareDifference { start_k: i64 },
    /// Linear recurrence / Multiply and Add: a_{n} = a_{n-1} * mult + add.
    MultiplyAndAdd { mult: i64, add: i64 },
    /// Fibonacci-like sum of previous two terms.
    FibonacciLike,
    /// Alphabet letter position progression with character shift: char -> (char - 'A' + shift) % 26 + 'A'.
    AlphabetShift { shift: i32 },
}

impl SeriesRule {
    pub fn description(&self) -> String {
        match self {
            SeriesRule::ConstantDifference { diff } => {
                if *diff >= 0 {
                    format!("Add {} to each successive term", diff)
                } else {
                    format!("Subtract {} from each successive term", -diff)
                }
            }
            SeriesRule::IncreasingDifference { start_diff, step } => {
                format!(
                    "Differences increase successively by {} (starting difference: {})",
                    step, start_diff
                )
            }
            SeriesRule::Geometric { ratio } => {
                format!("Multiply each successive term by {}", ratio)
            }
            SeriesRule::Alternating { diff1, diff2 } => {
                format!("Alternating operations: {:+} followed by {:+}", diff1, diff2)
            }
            SeriesRule::SquareDifference { start_k } => {
                format!("Differences are consecutive squares starting from {}^2", start_k)
            }
            SeriesRule::MultiplyAndAdd { mult, add } => {
                format!("Multiply by {} and add {}", mult, add)
            }
            SeriesRule::FibonacciLike => {
                "Each term is the sum of the two preceding terms".to_string()
            }
            SeriesRule::AlphabetShift { shift } => {
                format!("Shift each letter forward by {} alphabetical positions", shift)
            }
        }
    }
}

/// A concrete generated series problem with terms and deterministic solution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SeriesProblem {
    pub rule: SeriesRule,
    pub terms_numeric: Vec<i64>,
    pub terms_string: Vec<String>,
    pub expected_next_numeric: Option<i64>,
    pub expected_next_string: String,
    pub is_alphabet: bool,
    pub explanation: String,
}

impl SeriesProblem {
    /// Generate a number series from a given rule and start value.
    pub fn generate_numeric(rule: SeriesRule, start: i64, count: usize) -> Self {
        let mut terms = Vec::with_capacity(count + 1);
        let mut curr = start;
        terms.push(curr);

        match &rule {
            SeriesRule::ConstantDifference { diff } => {
                for _ in 1..=count {
                    curr += diff;
                    terms.push(curr);
                }
            }
            SeriesRule::IncreasingDifference { start_diff, step } => {
                let mut current_diff = *start_diff;
                for _ in 1..=count {
                    curr += current_diff;
                    terms.push(curr);
                    current_diff += step;
                }
            }
            SeriesRule::Geometric { ratio } => {
                for _ in 1..=count {
                    curr *= ratio;
                    terms.push(curr);
                }
            }
            SeriesRule::Alternating { diff1, diff2 } => {
                for i in 0..count {
                    let d = if i % 2 == 0 { *diff1 } else { *diff2 };
                    curr += d;
                    terms.push(curr);
                }
            }
            SeriesRule::SquareDifference { start_k } => {
                let mut k = *start_k;
                for _ in 1..=count {
                    curr += k * k;
                    terms.push(curr);
                    k += 1;
                }
            }
            SeriesRule::MultiplyAndAdd { mult, add } => {
                for _ in 1..=count {
                    curr = curr * mult + add;
                    terms.push(curr);
                }
            }
            SeriesRule::FibonacciLike => {
                let second = start + 3;
                terms = vec![start, second];
                for i in 2..=count {
                    let next = terms[i - 1] + terms[i - 2];
                    terms.push(next);
                }
            }
            SeriesRule::AlphabetShift { shift } => {
                let mut char_code = ((start.rem_euclid(26)) as u8) + b'A';
                let mut char_terms = Vec::new();
                char_terms.push((char_code as char).to_string());
                for _ in 1..=count {
                    let offset = (char_code - b'A' + (*shift as u8)).rem_euclid(26);
                    char_code = b'A' + offset;
                    char_terms.push((char_code as char).to_string());
                }
                let next_str = char_terms.pop().unwrap();
                let explanation = format!(
                    "Pattern rule: {}. Sequence: {}. Next character is **{}**.",
                    rule.description(),
                    char_terms.join(", "),
                    next_str
                );
                return Self {
                    rule,
                    terms_numeric: Vec::new(),
                    terms_string: char_terms,
                    expected_next_numeric: None,
                    expected_next_string: next_str,
                    is_alphabet: true,
                    explanation,
                };
            }
        }

        let next_val = terms.pop().unwrap();
        let terms_str: Vec<String> = terms.iter().map(|t| t.to_string()).collect();
        let explanation = format!(
            "Pattern rule: {}. Sequence: {}. Next term is **{}**.",
            rule.description(),
            terms_str.join(", "),
            next_val
        );

        Self {
            rule,
            terms_numeric: terms,
            terms_string: terms_str,
            expected_next_numeric: Some(next_val),
            expected_next_string: next_val.to_string(),
            is_alphabet: false,
            explanation,
        }
    }

    /// Check if a submitted string answer matches the expected series term.
    pub fn is_correct(&self, submission: &str) -> bool {
        submission.trim().eq_ignore_ascii_case(self.expected_next_string.trim())
    }

    /// Generate an alphabet series.
    pub fn generate_alphabet(start_char: char, shift: i32, count: usize) -> Self {
        let start_idx = ((start_char.to_ascii_uppercase() as u8) - b'A') as i64;
        Self::generate_numeric(SeriesRule::AlphabetShift { shift }, start_idx, count)
    }
}
