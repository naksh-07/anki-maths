// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use serde::{Deserialize, Serialize};

/// Graduated hint levels distinguishing light retrieval cues from heavy procedural scaffolding.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HintLevel {
    /// Level 0: No hint requested, fully independent execution.
    Level0_None = 0,
    /// Level 1: Light retrieval cue, formula reminder, unit definition, or notation hint.
    Level1_RetrievalCue = 1,
    /// Level 2: Procedural scaffold, intermediate sub-goal, variable isolation, or equation setup.
    Level2_ProceduralScaffold = 2,
    /// Level 3: Near-solution support, concrete calculation walkthrough, or direct resolution.
    Level3_NearSolutionSupport = 3,
}

impl Default for HintLevel {
    fn default() -> Self {
        HintLevel::Level0_None
    }
}

impl HintLevel {
    pub fn from_u32(val: u32) -> Self {
        match val {
            0 => HintLevel::Level0_None,
            1 => HintLevel::Level1_RetrievalCue,
            2 => HintLevel::Level2_ProceduralScaffold,
            _ => HintLevel::Level3_NearSolutionSupport,
        }
    }

    pub fn as_u32(&self) -> u32 {
        *self as u32
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            HintLevel::Level0_None => "none",
            HintLevel::Level1_RetrievalCue => "retrieval_cue",
            HintLevel::Level2_ProceduralScaffold => "procedural_scaffold",
            HintLevel::Level3_NearSolutionSupport => "near_solution_support",
        }
    }

    /// Independence score multiplier (1.0 = full independence, 0.3 = heavily scaffolded).
    pub fn independence_multiplier(&self) -> f64 {
        match self {
            HintLevel::Level0_None => 1.0,
            HintLevel::Level1_RetrievalCue => 0.92,
            HintLevel::Level2_ProceduralScaffold => 0.65,
            HintLevel::Level3_NearSolutionSupport => 0.35,
        }
    }
}

/// Structured record of hint interactions for a single practice attempt.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HintUsageRecord {
    pub max_hint_level: HintLevel,
    pub hint_count: u32,
    pub independent_success: bool,
    pub timestamp: i64,
}

impl HintUsageRecord {
    pub fn new(max_hint_level: HintLevel, hint_count: u32, independent_success: bool) -> Self {
        Self {
            max_hint_level,
            hint_count,
            independent_success,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    pub fn none(independent_success: bool) -> Self {
        Self {
            max_hint_level: HintLevel::Level0_None,
            hint_count: 0,
            independent_success,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }
}

/// Longitudinal summary of hint dependence across recent practice attempts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HintDependencyStats {
    pub total_attempts: usize,
    pub attempts_with_hints: usize,
    pub level_counts: [usize; 4],
    pub consecutive_hint_attempts: usize,
    pub hint_dependency_ratio: f64,
    pub heavy_scaffold_ratio: f64,
    pub composite_independence_score: f64,
}

impl Default for HintDependencyStats {
    fn default() -> Self {
        Self {
            total_attempts: 0,
            attempts_with_hints: 0,
            level_counts: [0; 4],
            consecutive_hint_attempts: 0,
            hint_dependency_ratio: 0.0,
            heavy_scaffold_ratio: 0.0,
            composite_independence_score: 1.0,
        }
    }
}

impl HintDependencyStats {
    /// Compute longitudinal hint dependency stats from recent hint records.
    pub fn from_records(records: &[HintUsageRecord]) -> Self {
        if records.is_empty() {
            return Self::default();
        }

        let total = records.len();
        let mut level_counts = [0usize; 4];
        let mut with_hints = 0;
        let mut heavy_scaffold_count = 0;
        let mut consecutive_hints = 0;
        let mut counting_consecutive = true;

        let mut multiplier_sum = 0.0;

        for r in records.iter().rev() {
            let lvl_idx = r.max_hint_level.as_u32() as usize;
            level_counts[lvl_idx.min(3)] += 1;
            multiplier_sum += r.max_hint_level.independence_multiplier();

            if r.max_hint_level != HintLevel::Level0_None {
                with_hints += 1;
                if counting_consecutive {
                    consecutive_hints += 1;
                }
            } else {
                counting_consecutive = false;
            }

            if r.max_hint_level >= HintLevel::Level2_ProceduralScaffold {
                heavy_scaffold_count += 1;
            }
        }

        let hint_ratio = with_hints as f64 / total as f64;
        let heavy_ratio = heavy_scaffold_count as f64 / total as f64;
        let avg_independence = multiplier_sum / total as f64;

        Self {
            total_attempts: total,
            attempts_with_hints: with_hints,
            level_counts,
            consecutive_hint_attempts: consecutive_hints,
            hint_dependency_ratio: hint_ratio,
            heavy_scaffold_ratio: heavy_ratio,
            composite_independence_score: avg_independence,
        }
    }

    /// Evaluates whether the learner exhibits unhealthy persistent hint dependence.
    pub fn has_chronic_dependence(&self) -> bool {
        self.total_attempts >= 3
            && (self.consecutive_hint_attempts >= 3
                || self.heavy_scaffold_ratio >= 0.50
                || self.composite_independence_score < 0.60)
    }
}
