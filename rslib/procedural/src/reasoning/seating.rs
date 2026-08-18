// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::reasoning::csp::{CspConstraint, CspProblem, CspSolver};

/// A linear seating arrangement puzzle instance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SeatingPuzzle {
    pub total_slots: usize,
    pub people: Vec<String>,
    pub conditions_text: Vec<String>,
    pub csp_problem: CspProblem,
    pub unique_solution: HashMap<String, usize>,
    pub target_question: String,
    pub target_answer: String,
    pub explanation: String,
}

impl SeatingPuzzle {
    /// Build a verified 5-person linear seating puzzle.
    /// People: A, B, C, D, E in slots 1..5 (left to right, all facing North).
    pub fn build_5person_anchor_puzzle(
        anchor_person: &str,
        anchor_slot: usize,
        pair_left: &str,
        pair_right: &str,
        other_people: &[&str],
        query_slot: usize,
    ) -> Option<Self> {
        let mut people = vec![
            anchor_person.to_string(),
            pair_left.to_string(),
            pair_right.to_string(),
        ];
        for p in other_people {
            if !people.contains(&p.to_string()) {
                people.push(p.to_string());
            }
        }
        people.sort();

        let total_slots = people.len();
        let mut csp = CspProblem::new(people.clone(), total_slots);
        let mut conditions = Vec::new();

        // 1. Anchor condition
        csp.add_constraint(CspConstraint::Fixed {
            var: anchor_person.to_string(),
            slot: anchor_slot,
        });
        conditions.push(format!(
            "{} sits at position {} from the left.",
            anchor_person, anchor_slot
        ));

        // 2. Immediate left pair condition
        csp.add_constraint(CspConstraint::ImmediateLeft {
            v1: pair_left.to_string(),
            v2: pair_right.to_string(),
        });
        conditions.push(format!(
            "{} sits immediately to the left of {}.",
            pair_left, pair_right
        ));

        // 3. Constrain other individuals to ensure an unambiguous, fully-determined layout
        if other_people.len() == 1 {
            let last_person = other_people[0];
            csp.add_constraint(CspConstraint::Fixed {
                var: last_person.to_string(),
                slot: total_slots,
            });
            conditions.push(format!(
                "{} sits at position {} from the left (extreme right).",
                last_person, total_slots
            ));
        } else if other_people.len() >= 2 {
            let p_slot4 = other_people[0];
            csp.add_constraint(CspConstraint::Fixed {
                var: p_slot4.to_string(),
                slot: 4,
            });
            conditions.push(format!(
                "{} sits at position 4 from the left.",
                p_slot4
            ));

            let p_slot5 = other_people[1];
            csp.add_constraint(CspConstraint::Fixed {
                var: p_slot5.to_string(),
                slot: 5,
            });
            conditions.push(format!(
                "{} sits at position 5 from the left (extreme right).",
                p_slot5
            ));
        }

        // Solve and verify uniqueness
        let solver = CspSolver;
        let sol = solver.solve_unique(&csp)?;

        // Find person at query slot
        let target_person = sol
            .iter()
            .find(|(_, &slot)| slot == query_slot)
            .map(|(p, _)| p.clone())?;

        let target_question = format!("Who sits at position {} from the left?", query_slot);
        let mut slot_summary: Vec<(&String, &usize)> = sol.iter().collect();
        slot_summary.sort_by_key(|(_, &s)| s);
        let arrangement_str: Vec<String> = slot_summary
            .iter()
            .map(|(p, s)| format!("Slot {}: {}", s, p))
            .collect();

        let explanation = format!(
            "Arrangement deduced via constraint satisfaction: {}. Therefore, {} sits at position {}.",
            arrangement_str.join(", "),
            target_person,
            query_slot
        );

        Some(Self {
            total_slots,
            people,
            conditions_text: conditions,
            csp_problem: csp,
            unique_solution: sol,
            target_question,
            target_answer: target_person,
            explanation,
        })
    }

    /// Check if a submitted answer is correct.
    pub fn is_correct(&self, submission: &str) -> bool {
        let clean = submission.trim().to_uppercase();
        clean == self.target_answer.to_uppercase()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seating_puzzle_generation_and_uniqueness() {
        // 4 people: A, B, C, D in slots 1..4
        // A is at 1, B is immediately left of C, D must be at 4
        let puzzle = SeatingPuzzle::build_5person_anchor_puzzle("A", 1, "B", "C", &["D"], 3);
        assert!(puzzle.is_some());

        let p = puzzle.unwrap();
        assert_eq!(p.unique_solution.get("A"), Some(&1));
        assert_eq!(p.unique_solution.get("B"), Some(&2));
        assert_eq!(p.unique_solution.get("C"), Some(&3));
        assert_eq!(p.unique_solution.get("D"), Some(&4));
        assert_eq!(p.target_answer, "C");
        assert!(p.is_correct("C"));
        assert!(p.is_correct("c"));
        assert!(!p.is_correct("B"));
    }
}
