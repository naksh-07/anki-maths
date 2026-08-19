// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use serde::{Deserialize, Serialize};

use crate::core::{Result, SkillId};
use crate::diagnostics::ErrorCategory;
use crate::skills::signals::PracticeProgressionState;
use crate::skills::SkillState;
use crate::storage::ProceduralStore;

/// Maximum bounded search depth for transitive prerequisite DAG traversal.
pub const DEFAULT_MAX_PREREQUISITE_DEPTH: usize = 10;

/// Soft prerequisite readiness evaluation status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum PrerequisiteReadiness {
    /// All direct and transitive prerequisites satisfy the required mastery thresholds.
    Ready,
    /// Learner can proceed, but advisory warnings indicate partially established prerequisites.
    ReadyWithWarnings { warnings: Vec<String> },
    /// One or more required prerequisites are unmastered or have critical breakdowns.
    PrerequisitesNeeded { missing_skills: Vec<SkillId> },
    /// Skill or prerequisite definitions are unknown/unregistered in the graph.
    Unknown,
}

/// Comprehensive evaluation report for a skill's prerequisite dependencies.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PrerequisiteEvaluation {
    pub skill_id: SkillId,
    pub readiness: PrerequisiteReadiness,
    pub direct_prerequisites: Vec<SkillId>,
    pub transitive_prerequisites: Vec<SkillId>,
    pub missing_prerequisites: Vec<SkillId>,
    pub cycle_warnings: Vec<String>,
    pub advisory_message: Option<String>,
}

impl PrerequisiteEvaluation {
    pub fn is_ready(&self) -> bool {
        matches!(
            self.readiness,
            PrerequisiteReadiness::Ready | PrerequisiteReadiness::ReadyWithWarnings { .. }
        )
    }

    pub fn requires_intervention(&self) -> bool {
        matches!(self.readiness, PrerequisiteReadiness::PrerequisitesNeeded { .. })
    }
}

/// Configurable threshold evaluator for prerequisite readiness and mastery.
pub struct PrerequisitePolicy;

impl PrerequisitePolicy {
    /// Evaluate whether a single prerequisite skill state meets the soft readiness threshold.
    pub fn evaluate_single_prerequisite(
        prereq_id: &SkillId,
        state_opt: Option<&SkillState>,
    ) -> (bool, Option<String>) {
        let Some(state) = state_opt else {
            return (
                false,
                Some(format!("Prerequisite skill '{}' has no practice history (unseen).", prereq_id)),
            );
        };

        // 1. Check for recent critical conceptual breakdowns
        let has_recent_concept_error = state.recent_attempts.iter().rev().take(3).any(|a| {
            !a.is_correct
                && matches!(
                    a.error_category,
                    Some(ErrorCategory::Concept) | Some(ErrorCategory::Conceptual)
                )
        });

        if has_recent_concept_error {
            return (
                false,
                Some(format!(
                    "Prerequisite skill '{}' has recent unresolved conceptual breakdowns.",
                    prereq_id
                )),
            );
        }

        // 2. High progression stages automatically satisfy threshold
        match state.practice_state {
            PracticeProgressionState::Mastered
            | PracticeProgressionState::Retired
            | PracticeProgressionState::Hibernating
            | PracticeProgressionState::Transfer
            | PracticeProgressionState::Variation => (true, None),
            PracticeProgressionState::Fluent => {
                let acc = state.recent_accuracy();
                if state.recent_attempts.is_empty() || acc >= 0.6 {
                    (true, None)
                } else {
                    (
                        true,
                        Some(format!(
                            "Prerequisite skill '{}' is Fluent but recent accuracy is moderate ({:.0}%).",
                            prereq_id,
                            acc * 100.0
                        )),
                    )
                }
            }
            PracticeProgressionState::Learning => {
                let acc = state.recent_accuracy();
                let attempts = state.recent_attempts.len();

                if attempts >= 3 && acc >= 0.75 && state.consecutive_successes >= 2 {
                    (
                        true,
                        Some(format!(
                            "Prerequisite skill '{}' is in Learning stage but demonstrates recent accuracy ({:.0}%).",
                            prereq_id,
                            acc * 100.0
                        )),
                    )
                } else {
                    (
                        false,
                        Some(format!(
                            "Prerequisite skill '{}' requires additional practice (currently Learning, recent acc: {:.0}%).",
                            prereq_id,
                            acc * 100.0
                        )),
                    )
                }
            }
            PracticeProgressionState::New => (
                false,
                Some(format!("Prerequisite skill '{}' is New and has not reached foundational fluency.", prereq_id)),
            ),
        }
    }
}

/// Memoized cache entry for prerequisite graph queries.
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct CachedGraphEntry {
    direct: Vec<SkillId>,
    transitive: Vec<SkillId>,
    has_cycle: bool,
    cycles: Vec<String>,
}

/// Thread-safe service for managing, querying, and evaluating the skill prerequisite DAG.
#[derive(Clone)]
pub struct PrerequisiteGraphService {
    graph: Arc<RwLock<HashMap<SkillId, Vec<SkillId>>>>,
    cache: Arc<RwLock<HashMap<SkillId, CachedGraphEntry>>>,
    max_depth: usize,
}

impl Default for PrerequisiteGraphService {
    fn default() -> Self {
        Self::new()
    }
}

impl PrerequisiteGraphService {
    pub fn new() -> Self {
        Self {
            graph: Arc::new(RwLock::new(HashMap::new())),
            cache: Arc::new(RwLock::new(HashMap::new())),
            max_depth: DEFAULT_MAX_PREREQUISITE_DEPTH,
        }
    }

    pub fn with_max_depth(mut self, max_depth: usize) -> Self {
        self.max_depth = max_depth.max(1);
        self
    }

    /// Populate or sync the prerequisite graph from the store.
    pub fn sync_from_store(&self, store: &ProceduralStore) -> Result<()> {
        let skills = store.list_all_skills()?;
        let mut g = self.graph.write().unwrap();
        g.clear();
        for skill in skills {
            g.insert(skill.id, skill.prerequisites);
        }
        self.invalidate_cache();
        Ok(())
    }

    /// Register or update a single skill's prerequisites in the in-memory graph.
    pub fn register_skill_prerequisites(&self, skill_id: SkillId, prerequisites: Vec<SkillId>) {
        let mut g = self.graph.write().unwrap();
        g.insert(skill_id, prerequisites);
        self.invalidate_cache();
    }

    /// Invalidate memoized traversal caches.
    pub fn invalidate_cache(&self) {
        let mut cache = self.cache.write().unwrap();
        cache.clear();
    }

    /// Return direct prerequisites for a skill.
    pub fn get_direct_prerequisites(&self, skill_id: &SkillId) -> Vec<SkillId> {
        let g = self.graph.read().unwrap();
        g.get(skill_id).cloned().unwrap_or_default()
    }

    /// Return all transitive prerequisites for a skill with cycle protection and bounded depth.
    pub fn get_transitive_prerequisites(&self, skill_id: &SkillId) -> (Vec<SkillId>, Vec<String>) {
        // Check cache first
        {
            let cache = self.cache.read().unwrap();
            if let Some(entry) = cache.get(skill_id) {
                return (entry.transitive.clone(), entry.cycles.clone());
            }
        }

        let g = self.graph.read().unwrap();
        let mut visited = HashSet::new();
        let mut path_stack = Vec::new();
        let mut result = Vec::new();
        let mut cycle_warnings = Vec::new();

        Self::dfs_transitive(
            skill_id,
            &g,
            0,
            self.max_depth,
            &mut visited,
            &mut path_stack,
            &mut result,
            &mut cycle_warnings,
        );

        let direct = g.get(skill_id).cloned().unwrap_or_default();
        let entry = CachedGraphEntry {
            direct,
            transitive: result.clone(),
            has_cycle: !cycle_warnings.is_empty(),
            cycles: cycle_warnings.clone(),
        };

        drop(g);
        let mut cache = self.cache.write().unwrap();
        cache.insert(skill_id.clone(), entry);

        (result, cycle_warnings)
    }

    /// Check if the prerequisite subtree rooted at `skill_id` contains any cycles.
    pub fn detect_cycles(&self, skill_id: &SkillId) -> (bool, Vec<String>) {
        let (_transitive, cycles) = self.get_transitive_prerequisites(skill_id);
        (!cycles.is_empty(), cycles)
    }

    /// Recursive bounded DFS with back-edge cycle detection.
    fn dfs_transitive(
        current: &SkillId,
        graph: &HashMap<SkillId, Vec<SkillId>>,
        depth: usize,
        max_depth: usize,
        visited: &mut HashSet<SkillId>,
        path_stack: &mut Vec<SkillId>,
        result: &mut Vec<SkillId>,
        cycle_warnings: &mut Vec<String>,
    ) {
        if depth >= max_depth {
            return;
        }

        if let Some(prereqs) = graph.get(current) {
            for prereq in prereqs {
                if path_stack.contains(prereq) {
                    cycle_warnings.push(format!(
                        "Cycle detected in prerequisite graph: {} -> {} (path: {:?})",
                        current, prereq, path_stack
                    ));
                    continue;
                }

                if !visited.contains(prereq) {
                    visited.insert(prereq.clone());
                    result.push(prereq.clone());
                    path_stack.push(current.clone());

                    Self::dfs_transitive(
                        prereq,
                        graph,
                        depth + 1,
                        max_depth,
                        visited,
                        path_stack,
                        result,
                        cycle_warnings,
                    );

                    path_stack.pop();
                }
            }
        }
    }

    /// Evaluates complete prerequisite readiness for a target skill given a set of learner skill states.
    pub fn evaluate_readiness(
        &self,
        skill_id: &SkillId,
        skill_states: &HashMap<SkillId, SkillState>,
    ) -> PrerequisiteEvaluation {
        let direct = self.get_direct_prerequisites(skill_id);
        let (transitive, cycles) = self.get_transitive_prerequisites(skill_id);

        if direct.is_empty() && transitive.is_empty() {
            return PrerequisiteEvaluation {
                skill_id: skill_id.clone(),
                readiness: PrerequisiteReadiness::Ready,
                direct_prerequisites: Vec::new(),
                transitive_prerequisites: Vec::new(),
                missing_prerequisites: Vec::new(),
                cycle_warnings: cycles,
                advisory_message: None,
            };
        }

        let mut missing = Vec::new();
        let mut warnings = Vec::new();

        // Evaluate all dependencies (direct + transitive)
        let mut all_deps = direct.clone();
        for t in &transitive {
            if !all_deps.contains(t) {
                all_deps.push(t.clone());
            }
        }

        for prereq_id in &all_deps {
            let state_opt = skill_states.get(prereq_id);
            let (is_met, warning_opt) = PrerequisitePolicy::evaluate_single_prerequisite(prereq_id, state_opt);

            if !is_met {
                missing.push(prereq_id.clone());
            }
            if let Some(msg) = warning_opt {
                warnings.push(msg);
            }
        }

        let readiness = if !missing.is_empty() {
            PrerequisiteReadiness::PrerequisitesNeeded {
                missing_skills: missing.clone(),
            }
        } else if !warnings.is_empty() {
            PrerequisiteReadiness::ReadyWithWarnings {
                warnings: warnings.clone(),
            }
        } else {
            PrerequisiteReadiness::Ready
        };

        let advisory_message = if !missing.is_empty() {
            let names: Vec<String> = missing.iter().map(|s| format!("'{}'", s)).collect();
            Some(format!(
                "Foundational gaps detected in {}. Reviewing these prerequisites will improve success on '{}'.",
                names.join(", "),
                skill_id
            ))
        } else if !warnings.is_empty() {
            Some(warnings.join(" "))
        } else {
            None
        };

        PrerequisiteEvaluation {
            skill_id: skill_id.clone(),
            readiness,
            direct_prerequisites: direct,
            transitive_prerequisites: transitive,
            missing_prerequisites: missing,
            cycle_warnings: cycles,
            advisory_message,
        }
    }
}
