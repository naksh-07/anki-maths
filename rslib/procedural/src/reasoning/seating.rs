// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::collections::HashMap;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::reasoning::csp::{CspConstraint, CspProblem, CspSolver};

pub const SEATING_NAMES_POOL: &[&str] = &[
    "Alice", "Bob", "Charlie", "David", "Emma", "Frank", "Grace", "Henry",
    "Ivy", "Jack", "Karan", "Liam", "Maya", "Noah", "Olivia", "Priya",
    "Quentin", "Rita", "Sam", "Tara", "Uma", "Victor", "Wendy", "Xander",
];

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
    pub anchor_person: String,
    pub query_slot: usize,
}

impl SeatingPuzzle {
    /// Dynamically generate a verified unique linear seating puzzle with varied constraints.
    pub fn generate_dynamic(rng: &mut StdRng, total_slots: usize, _difficulty: u32) -> Option<Self> {
        let total_slots = total_slots.clamp(4, 7);

        // Pick distinct names
        let mut name_pool = SEATING_NAMES_POOL.to_vec();
        name_pool.shuffle(rng);
        let chosen_names: Vec<String> = name_pool[..total_slots].iter().map(|&s| s.to_string()).collect();

        // Generate a random ground-truth permutation: assignment of slots 1..=total_slots
        let mut slots: Vec<usize> = (1..=total_slots).collect();
        slots.shuffle(rng);

        let mut ground_truth: HashMap<String, usize> = HashMap::new();
        let mut slot_to_person: HashMap<usize, String> = HashMap::new();
        for (i, name) in chosen_names.iter().enumerate() {
            ground_truth.insert(name.clone(), slots[i]);
            slot_to_person.insert(slots[i], name.clone());
        }

        let solver = CspSolver;

        for _attempt in 0..30 {
            let mut csp = CspProblem::new(chosen_names.clone(), total_slots);
            let mut conditions = Vec::new();

            // Always pick at least one fixed anchor to ground the puzzle
            let anchor_idx = rng.random_range(0..chosen_names.len());
            let anchor_name = chosen_names[anchor_idx].clone();
            let anchor_slot = ground_truth[&anchor_name];

            csp.add_constraint(CspConstraint::Fixed {
                var: anchor_name.clone(),
                slot: anchor_slot,
            });
            if anchor_slot == 1 {
                conditions.push(format!("{} sits at the extreme left end (position 1).", anchor_name));
            } else if anchor_slot == total_slots {
                conditions.push(format!("{} sits at the extreme right end (position {}).", anchor_name, total_slots));
            } else {
                conditions.push(format!("{} sits at position {} from the left.", anchor_name, anchor_slot));
            }

            // Generate candidate constraints derived from ground truth
            let mut candidate_constraints: Vec<(CspConstraint, String)> = Vec::new();

            // 1. Immediate adjacency / Immediate left or right
            for i in 1..total_slots {
                let p_left = &slot_to_person[&i];
                let p_right = &slot_to_person[&(i + 1)];
                candidate_constraints.push((
                    CspConstraint::ImmediateLeft {
                        v1: p_left.clone(),
                        v2: p_right.clone(),
                    },
                    format!("{} sits immediately to the left of {}.", p_left, p_right),
                ));
                candidate_constraints.push((
                    CspConstraint::Adjacent {
                        v1: p_left.clone(),
                        v2: p_right.clone(),
                    },
                    format!("{} and {} sit adjacent to each other.", p_left, p_right),
                ));
            }

            // 2. Not adjacent constraints
            for i in 1..=total_slots {
                for j in (i + 2)..=total_slots {
                    let p1 = &slot_to_person[&i];
                    let p2 = &slot_to_person[&j];
                    candidate_constraints.push((
                        CspConstraint::NotAdjacent {
                            v1: p1.clone(),
                            v2: p2.clone(),
                        },
                        format!("{} and {} do not sit adjacent to each other.", p1, p2),
                    ));
                }
            }

            // 3. LeftOf / RightOf relative ordering
            for i in 1..=total_slots {
                for j in (i + 1)..=total_slots {
                    let p1 = &slot_to_person[&i];
                    let p2 = &slot_to_person[&j];
                    candidate_constraints.push((
                        CspConstraint::LeftOf {
                            v1: p1.clone(),
                            v2: p2.clone(),
                        },
                        format!("{} sits somewhere to the left of {}.", p1, p2),
                    ));
                }
            }

            // 4. Fixed other positions
            for (&slot, name) in &slot_to_person {
                if name != &anchor_name {
                    candidate_constraints.push((
                        CspConstraint::Fixed {
                            var: name.clone(),
                            slot,
                        },
                        if slot == 1 {
                            format!("{} sits at the extreme left (position 1).", name)
                        } else if slot == total_slots {
                            format!("{} sits at the extreme right (position {}).", name, total_slots)
                        } else {
                            format!("{} sits at position {} from the left.", name, slot)
                        },
                    ));
                }
            }

            candidate_constraints.shuffle(rng);

            // Greedily add constraints until unique
            for (c, text) in candidate_constraints {
                if solver.solve_unique(&csp).is_some() {
                    break;
                }
                csp.add_constraint(c);
                conditions.push(text);
            }

            if let Some(sol) = solver.solve_unique(&csp) {
                // Choose query slot (different from anchor if possible)
                let query_slot = if total_slots > 1 {
                    let other_slots: Vec<usize> = (1..=total_slots).filter(|&s| s != anchor_slot).collect();
                    if other_slots.is_empty() {
                        anchor_slot
                    } else {
                        other_slots[rng.random_range(0..other_slots.len())]
                    }
                } else {
                    1
                };

                let target_person = sol
                    .iter()
                    .find(|(_, &s)| s == query_slot)
                    .map(|(p, _)| p.clone())
                    .unwrap_or_else(|| chosen_names[0].clone());

                let target_question = format!("Who sits at position {} from the left?", query_slot);

                let mut slot_summary: Vec<(&String, &usize)> = sol.iter().collect();
                slot_summary.sort_by_key(|(_, &s)| s);
                let arrangement_str: Vec<String> = slot_summary
                    .iter()
                    .map(|(p, s)| format!("Slot {}: {}", s, p))
                    .collect();

                let explanation = format!(
                    "Arrangement deduced via constraint satisfaction: {}. Therefore, **{}** sits at position {}.",
                    arrangement_str.join(", "),
                    target_person,
                    query_slot
                );

                return Some(Self {
                    total_slots,
                    people: chosen_names,
                    conditions_text: conditions,
                    csp_problem: csp,
                    unique_solution: sol,
                    target_question,
                    target_answer: target_person,
                    explanation,
                    anchor_person: anchor_name,
                    query_slot,
                });
            }
        }

        // Fallback to verified anchor builder if random loop didn't converge
        Self::build_5person_anchor_puzzle(
            &chosen_names[0], 1, &chosen_names[1], &chosen_names[2],
            &chosen_names[3..].iter().map(|s| s.as_str()).collect::<Vec<_>>(),
            2,
        )
    }

    /// Build a verified linear seating puzzle fallback.
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

        // 3. Constrain other individuals to ensure an unambiguous layout
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
            anchor_person: anchor_person.to_string(),
            query_slot,
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

    #[test]
    fn test_dynamic_seating_generation_entropy() {
        use rand::SeedableRng;
        let mut rng = StdRng::seed_from_u64(98765);
        for slots in 4..=6 {
            let pz = SeatingPuzzle::generate_dynamic(&mut rng, slots, 2);
            assert!(pz.is_some(), "Dynamic generation failed for slots {}", slots);
            let puzzle = pz.unwrap();
            assert_eq!(puzzle.unique_solution.len(), puzzle.total_slots);
            assert!(puzzle.is_correct(&puzzle.target_answer));
        }
    }
}
