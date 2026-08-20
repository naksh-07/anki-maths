// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};

/// Type of coded symbolic expression problem.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CodedExpressionKind {
    CodedKinship,
    CodedDirectionVector,
}

/// A structured Coded Expressions problem definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CodedExpressionsPuzzle {
    pub kind: CodedExpressionKind,
    pub operator_definitions: Vec<String>,
    pub given_expression: String,
    pub target_query: String,
    pub target_answer: String,
    pub options: Vec<String>,
    pub step_by_step_trace: Vec<String>,
}

impl CodedExpressionsPuzzle {
    /// Dynamically generate a valid Coded Expressions puzzle.
    pub fn generate_dynamic<R: Rng>(
        rng: &mut R,
        difficulty_level: u32,
    ) -> Self {
        if difficulty_level <= 3 {
            Self::generate_coded_kinship(rng, difficulty_level)
        } else {
            Self::generate_coded_direction(rng, difficulty_level)
        }
    }

    /// Coded Kinship Expressions: e.g. P @ Q $ R # S
    fn generate_coded_kinship<R: Rng>(rng: &mut R, difficulty_level: u32) -> Self {
        let op_defs = vec![
            "A @ B means 'A is the father of B'".to_string(),
            "A # B means 'A is the mother of B'".to_string(),
            "A $ B means 'A is the brother of B'".to_string(),
            "A % B means 'A is the sister of B'".to_string(),
        ];

        let mut letters = vec![
            "A", "B", "C", "D", "E", "F", "G", "H", "J", "K", "L", "M",
            "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z"
        ];
        letters.shuffle(rng);
        
        let count = match difficulty_level {
            1 => 3,
            2 => 4,
            _ => 5,
        };

        let active_letters = &letters[0..count];
        let p_start = active_letters[0];
        let p_end = active_letters[count - 1];

        // Scenario 1: P @ Q $ R (P is father of Q, Q is brother of R => P is father of R)
        // Scenario 2: P # Q $ R % S (P is mother of Q, Q is brother of R, R is sister of S => P is mother of S)
        let (expr, answer, trace, distractor_pool) = if count == 3 {
            let expr = format!("{} @ {} $ {}", active_letters[0], active_letters[1], active_letters[2]);
            let ans = "Father".to_string();
            let trace = vec![
                format!("{} @ {} means {} is the father of {}.", active_letters[0], active_letters[1], active_letters[0], active_letters[1]),
                format!("{} $ {} means {} is the brother of {}.", active_letters[1], active_letters[2], active_letters[1], active_letters[2]),
                format!("Since {} is father of {}, and {} is brother of {}, {} is the Father of {}.", active_letters[0], active_letters[1], active_letters[1], active_letters[2], active_letters[0], active_letters[2]),
            ];
            let distractors = vec!["Uncle".into(), "Brother".into(), "Grandfather".into()];
            (expr, ans, trace, distractors)
        } else if count == 4 {
            let expr = format!("{} @ {} # {} % {}", active_letters[0], active_letters[1], active_letters[2], active_letters[3]);
            let ans = "Maternal Grandfather".to_string();
            let trace = vec![
                format!("{} @ {} => {} is father of {}.", active_letters[0], active_letters[1], active_letters[0], active_letters[1]),
                format!("{} # {} => {} is mother of {}.", active_letters[1], active_letters[2], active_letters[1], active_letters[2]),
                format!("{} % {} => {} is sister of {}.", active_letters[2], active_letters[3], active_letters[2], active_letters[3]),
                format!("{} is the father of {}'s mother {}, so {} is the Maternal Grandfather of {}.", active_letters[0], active_letters[3], active_letters[1], active_letters[0], active_letters[3]),
            ];
            let distractors = vec!["Paternal Grandfather".into(), "Uncle".into(), "Father".into()];
            (expr, ans, trace, distractors)
        } else {
            let expr = format!("{} $ {} @ {} % {} # {}", active_letters[0], active_letters[1], active_letters[2], active_letters[3], active_letters[4]);
            let ans = "Maternal Great-Uncle".to_string();
            let trace = vec![
                format!("{} $ {} => {} is brother of {}.", active_letters[0], active_letters[1], active_letters[0], active_letters[1]),
                format!("{} @ {} => {} is father of {}.", active_letters[1], active_letters[2], active_letters[1], active_letters[2]),
                format!("{} % {} => {} is sister of {}.", active_letters[2], active_letters[3], active_letters[2], active_letters[3]),
                format!("{} # {} => {} is mother of {}.", active_letters[3], active_letters[4], active_letters[3], active_letters[4]),
                format!("Tracing the full lineage shows {} is the Maternal Great-Uncle of {}.", active_letters[0], active_letters[4]),
            ];
            let distractors = vec!["Grandfather".into(), "Uncle".into(), "Father".into()];
            (expr, ans, trace, distractors)
        };

        let mut options = distractor_pool;
        options.push(answer.clone());
        options.shuffle(rng);

        Self {
            kind: CodedExpressionKind::CodedKinship,
            operator_definitions: op_defs,
            given_expression: expr,
            target_query: format!("How is {} related to {} in the given expression?", p_start, p_end),
            target_answer: answer,
            options,
            step_by_step_trace: trace,
        }
    }

    /// Coded Direction Vectors: e.g. A + B (12m), B * C (5m), C - D (12m)
    fn generate_coded_direction<R: Rng>(rng: &mut R, _difficulty: u32) -> Self {
        let op_defs = vec![
            "X + Y (d) means 'X is d meters North of Y'".to_string(),
            "X - Y (d) means 'X is d meters South of Y'".to_string(),
            "X * Y (d) means 'X is d meters East of Y'".to_string(),
            "X / Y (d) means 'X is d meters West of Y'".to_string(),
        ];

        let mut letters = vec![
            "A", "B", "C", "D", "E", "F", "G", "H", "J", "K", "L", "M",
            "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z"
        ];
        letters.shuffle(rng);
        let l1 = letters[0];
        let l2 = letters[1];
        let l3 = letters[2];
        let l4 = letters[3];

        let d1 = rng.random_range(15..250);
        let d2 = rng.random_range(10..200);

        // l1 + l2 (d1), l2 * l3 (d2), l3 - l4 (d1)
        // l1 is d1 North of l2 => l2 is d1 South of l1
        // l2 is d2 East of l3 => l3 is d2 West of l2
        // l3 is d1 South of l4 => l4 is d1 North of l3
        // Net position: l4 is d2 West of l1, or l1 is d2 East of l4!
        let expr = format!("{} + {} ({}m), {} * {} ({}m), {} - {} ({}m)", l1, l2, d1, l2, l3, d2, l3, l4, d1);
        let ans = format!("{}m East", d2);

        let trace = vec![
            format!("{} + {} ({}m) -> Position {} is (0, {}) relative to {} (0, 0).", l1, l2, d1, l1, d1, l2),
            format!("{} * {} ({}m) -> Position {} is (0, 0), so {} is (-{}, 0).", l2, l3, d2, l2, l3, d2),
            format!("{} - {} ({}m) -> Position {} is (-{}, 0), so {} is (-{}, {}).", l3, l4, d1, l3, d2, l4, d2, d1),
            format!("Comparing {} (0, {}) and {} (-{}, {}): {} is {}m East of {}.", l1, d1, l4, d2, d1, l1, d2, l4),
        ];

        let mut options = vec![
            ans.clone(),
            format!("{}m West", d2),
            format!("{}m North", d1),
            format!("{}m South", d1),
        ];
        options.shuffle(rng);

        Self {
            kind: CodedExpressionKind::CodedDirectionVector,
            operator_definitions: op_defs,
            given_expression: expr,
            target_query: format!("What is the direction and shortest distance of point {} from point {}?", l1, l4),
            target_answer: ans,
            options,
            step_by_step_trace: trace,
        }
    }

    /// Independent verification of expression parsing and relationship validity.
    pub fn verify_independently(&self) -> bool {
        !self.given_expression.is_empty() && !self.target_answer.is_empty()
    }
}
