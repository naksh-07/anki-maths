// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

/// High-level constraint definition over discrete integer slot positions (1-indexed or 0-indexed).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CspConstraint {
    /// Exact fixed position: pos(var) == slot.
    Fixed { var: String, slot: usize },
    /// All variables must occupy distinct positions (all-different).
    AllDifferent,
    /// Inequality: pos(v1) != pos(v2).
    NotEqual { v1: String, v2: String },
    /// v1 sits immediately to the left of v2: pos(v1) + 1 == pos(v2).
    ImmediateLeft { v1: String, v2: String },
    /// v1 sits immediately to the right of v2: pos(v1) == pos(v2) + 1.
    ImmediateRight { v1: String, v2: String },
    /// v1 and v2 sit next to each other: |pos(v1) - pos(v2)| == 1.
    Adjacent { v1: String, v2: String },
    /// v1 and v2 do NOT sit next to each other: |pos(v1) - pos(v2)| != 1.
    NotAdjacent { v1: String, v2: String },
    /// v1 sits somewhere to the left of v2: pos(v1) < pos(v2).
    LeftOf { v1: String, v2: String },
    /// v1 sits somewhere to the right of v2: pos(v1) > pos(v2).
    RightOf { v1: String, v2: String },
    /// v1 is between v2 and v3 (either v2 < v1 < v3 or v3 < v1 < v2).
    Between { v1: String, v2: String, v3: String },
    /// Exact distance between v1 and v2: |pos(v1) - pos(v2)| == dist.
    Distance { v1: String, v2: String, dist: usize },
    /// v1 must be in one of the allowed slots.
    OneOf { var: String, allowed: Vec<usize> },
}

impl CspConstraint {
    /// Evaluate whether a constraint is satisfied given a partial or complete assignment.
    /// Returns true if satisfied or if not all participating variables are assigned yet.
    pub fn is_satisfied(&self, assignment: &HashMap<String, usize>) -> bool {
        match self {
            CspConstraint::Fixed { var, slot } => {
                if let Some(&p) = assignment.get(var) {
                    p == *slot
                } else {
                    true
                }
            }
            CspConstraint::AllDifferent => {
                let mut seen = HashSet::new();
                for &val in assignment.values() {
                    if !seen.insert(val) {
                        return false;
                    }
                }
                true
            }
            CspConstraint::NotEqual { v1, v2 } => {
                if let (Some(&p1), Some(&p2)) = (assignment.get(v1), assignment.get(v2)) {
                    p1 != p2
                } else {
                    true
                }
            }
            CspConstraint::ImmediateLeft { v1, v2 } => {
                if let (Some(&p1), Some(&p2)) = (assignment.get(v1), assignment.get(v2)) {
                    p1 + 1 == p2
                } else {
                    true
                }
            }
            CspConstraint::ImmediateRight { v1, v2 } => {
                if let (Some(&p1), Some(&p2)) = (assignment.get(v1), assignment.get(v2)) {
                    p1 == p2 + 1
                } else {
                    true
                }
            }
            CspConstraint::Adjacent { v1, v2 } => {
                if let (Some(&p1), Some(&p2)) = (assignment.get(v1), assignment.get(v2)) {
                    (p1 as isize - p2 as isize).abs() == 1
                } else {
                    true
                }
            }
            CspConstraint::NotAdjacent { v1, v2 } => {
                if let (Some(&p1), Some(&p2)) = (assignment.get(v1), assignment.get(v2)) {
                    (p1 as isize - p2 as isize).abs() != 1
                } else {
                    true
                }
            }
            CspConstraint::LeftOf { v1, v2 } => {
                if let (Some(&p1), Some(&p2)) = (assignment.get(v1), assignment.get(v2)) {
                    p1 < p2
                } else {
                    true
                }
            }
            CspConstraint::RightOf { v1, v2 } => {
                if let (Some(&p1), Some(&p2)) = (assignment.get(v1), assignment.get(v2)) {
                    p1 > p2
                } else {
                    true
                }
            }
            CspConstraint::Between { v1, v2, v3 } => {
                if let (Some(&p1), Some(&p2), Some(&p3)) = (
                    assignment.get(v1),
                    assignment.get(v2),
                    assignment.get(v3),
                ) {
                    (p2 < p1 && p1 < p3) || (p3 < p1 && p1 < p2)
                } else {
                    true
                }
            }
            CspConstraint::Distance { v1, v2, dist } => {
                if let (Some(&p1), Some(&p2)) = (assignment.get(v1), assignment.get(v2)) {
                    (p1 as isize - p2 as isize).abs() == *dist as isize
                } else {
                    true
                }
            }
            CspConstraint::OneOf { var, allowed } => {
                if let Some(&p) = assignment.get(var) {
                    allowed.contains(&p)
                } else {
                    true
                }
            }
        }
    }
}

/// A structured CSP problem formulation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CspProblem {
    pub variables: Vec<String>,
    pub initial_domains: HashMap<String, Vec<usize>>,
    pub constraints: Vec<CspConstraint>,
}

impl CspProblem {
    pub fn new(variables: Vec<String>, total_slots: usize) -> Self {
        let default_domain: Vec<usize> = (1..=total_slots).collect();
        let mut initial_domains = HashMap::new();
        for v in &variables {
            initial_domains.insert(v.clone(), default_domain.clone());
        }

        Self {
            variables,
            initial_domains,
            constraints: vec![CspConstraint::AllDifferent],
        }
    }

    pub fn add_constraint(&mut self, constraint: CspConstraint) {
        self.constraints.push(constraint);
    }
}

/// Search case record for diagnosing search branching and contradiction handling.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchCase {
    pub case_id: String,
    pub variable: String,
    pub assigned_val: usize,
    pub is_contradiction: bool,
    pub derived_assignments: HashMap<String, usize>,
}

/// Lightweight, deterministic constraint satisfaction solver.
pub struct CspSolver;

impl CspSolver {
    /// Solve the CSP problem and return all valid complete assignments.
    pub fn solve_all(&self, problem: &CspProblem) -> Vec<HashMap<String, usize>> {
        let mut solutions = Vec::new();
        let mut current_assignment = HashMap::new();
        let mut domains = problem.initial_domains.clone();

        // 1. Initial domain reduction via unary constraints
        if !self.propagate_unary_constraints(problem, &mut domains) {
            return solutions;
        }

        self.backtrack(
            problem,
            &mut current_assignment,
            &domains,
            &mut solutions,
            0,
        );

        solutions
    }

    /// Check if the problem has exactly one unique valid solution.
    pub fn is_unambiguous(&self, problem: &CspProblem) -> bool {
        let solutions = self.solve_all(problem);
        solutions.len() == 1
    }

    /// Retrieve the single canonical solution if unique.
    pub fn solve_unique(&self, problem: &CspProblem) -> Option<HashMap<String, usize>> {
        let mut solutions = self.solve_all(problem);
        if solutions.len() == 1 {
            Some(solutions.remove(0))
        } else {
            None
        }
    }

    fn propagate_unary_constraints(
        &self,
        problem: &CspProblem,
        domains: &mut HashMap<String, Vec<usize>>,
    ) -> bool {
        for constraint in &problem.constraints {
            match constraint {
                CspConstraint::Fixed { var, slot } => {
                    if let Some(dom) = domains.get_mut(var) {
                        dom.retain(|&s| s == *slot);
                        if dom.is_empty() {
                            return false;
                        }
                    }
                }
                CspConstraint::OneOf { var, allowed } => {
                    if let Some(dom) = domains.get_mut(var) {
                        dom.retain(|s| allowed.contains(s));
                        if dom.is_empty() {
                            return false;
                        }
                    }
                }
                _ => {}
            }
        }
        true
    }

    fn backtrack(
        &self,
        problem: &CspProblem,
        assignment: &mut HashMap<String, usize>,
        domains: &HashMap<String, Vec<usize>>,
        solutions: &mut Vec<HashMap<String, usize>>,
        depth: usize,
    ) {
        // Stop search if exceeded safe bound (e.g. 50 solutions)
        if solutions.len() > 50 {
            return;
        }

        if assignment.len() == problem.variables.len() {
            // Check all constraints
            if self.is_completely_valid(assignment, &problem.constraints) {
                solutions.push(assignment.clone());
            }
            return;
        }

        // Variable selection: Minimum Remaining Values (MRV) heuristic
        let next_var = self.select_unassigned_variable(problem, assignment, domains);
        let Some(var_name) = next_var else {
            return;
        };

        let var_domain = match domains.get(&var_name) {
            Some(d) => d.clone(),
            None => return,
        };

        for val in var_domain {
            assignment.insert(var_name.clone(), val);

            // Check if partial assignment violates any constraint
            if self.is_partially_valid(assignment, &problem.constraints) {
                // Forward checking: prune domain of remaining variables
                let mut next_domains = domains.clone();
                if self.forward_check(problem, assignment, &mut next_domains) {
                    self.backtrack(problem, assignment, &next_domains, solutions, depth + 1);
                }
            }

            assignment.remove(&var_name);
        }
    }

    fn select_unassigned_variable(
        &self,
        problem: &CspProblem,
        assignment: &HashMap<String, usize>,
        domains: &HashMap<String, Vec<usize>>,
    ) -> Option<String> {
        problem
            .variables
            .iter()
            .filter(|v| !assignment.contains_key(v.as_str()))
            .min_by_key(|v| domains.get(v.as_str()).map(|d| d.len()).unwrap_or(usize::MAX))
            .cloned()
    }

    fn is_partially_valid(
        &self,
        assignment: &HashMap<String, usize>,
        constraints: &[CspConstraint],
    ) -> bool {
        for c in constraints {
            if !c.is_satisfied(assignment) {
                return false;
            }
        }
        true
    }

    fn is_completely_valid(
        &self,
        assignment: &HashMap<String, usize>,
        constraints: &[CspConstraint],
    ) -> bool {
        for c in constraints {
            if !c.is_satisfied(assignment) {
                return false;
            }
        }
        true
    }

    fn forward_check(
        &self,
        problem: &CspProblem,
        assignment: &HashMap<String, usize>,
        domains: &mut HashMap<String, Vec<usize>>,
    ) -> bool {
        // Assigned positions are unavailable for remaining variables
        let assigned_slots: HashSet<usize> = assignment.values().copied().collect();

        for var in &problem.variables {
            if !assignment.contains_key(var) {
                if let Some(dom) = domains.get_mut(var) {
                    dom.retain(|s| !assigned_slots.contains(s));
                    if dom.is_empty() {
                        return false;
                    }
                }
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csp_solver_linear_seating_unique_solution() {
        // 4 people: A, B, C, D in slots 1, 2, 3, 4
        // Constraints:
        // 1. A is in slot 1 (Fixed)
        // 2. B is immediately left of C (ImmediateLeft)
        // 3. D is in slot 4 (Fixed)
        let mut problem = CspProblem::new(
            vec!["A".to_string(), "B".to_string(), "C".to_string(), "D".to_string()],
            4,
        );
        problem.add_constraint(CspConstraint::Fixed {
            var: "A".to_string(),
            slot: 1,
        });
        problem.add_constraint(CspConstraint::ImmediateLeft {
            v1: "B".to_string(),
            v2: "C".to_string(),
        });
        problem.add_constraint(CspConstraint::Fixed {
            var: "D".to_string(),
            slot: 4,
        });

        let solver = CspSolver;
        assert!(solver.is_unambiguous(&problem));

        let sol = solver.solve_unique(&problem).unwrap();
        assert_eq!(sol.get("A"), Some(&1));
        assert_eq!(sol.get("B"), Some(&2));
        assert_eq!(sol.get("C"), Some(&3));
        assert_eq!(sol.get("D"), Some(&4));
    }

    #[test]
    fn test_csp_solver_contradiction_detection() {
        // A is at 1, B is at 1 -> Contradiction via AllDifferent
        let mut problem = CspProblem::new(vec!["A".to_string(), "B".to_string()], 2);
        problem.add_constraint(CspConstraint::Fixed {
            var: "A".to_string(),
            slot: 1,
        });
        problem.add_constraint(CspConstraint::Fixed {
            var: "B".to_string(),
            slot: 1,
        });

        let solver = CspSolver;
        let solutions = solver.solve_all(&problem);
        assert!(solutions.is_empty());
        assert!(!solver.is_unambiguous(&problem));
    }
}
