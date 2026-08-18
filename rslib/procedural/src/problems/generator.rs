// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use serde::{Deserialize, Serialize};

use crate::core::{ProblemFamilyId, Result};
use crate::problems::ProblemInstance;

/// Standard taxonomy of procedural variation types across Maths problem families.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VariantType {
    /// Exact / replay variation for initial learning and parameter grounding.
    ExactReplay,
    /// Isomorphic variation (same mathematical structure and steps, different numerical values).
    Isomorphic,
    /// Structural variation (different algebraic form, alternate step sequencing, or additional operations).
    Structural,
    /// Reverse variation (solve for initial unknown or parameter given the final output).
    Reverse,
    /// Boundary / trap variation (edge cases, extreme bounds, negative constraints, zero rates).
    BoundaryTrap,
    /// Transfer variation (word-problem context, composite multi-domain application).
    Transfer,
}

impl VariantType {
    pub fn as_str(&self) -> &'static str {
        match self {
            VariantType::ExactReplay => "exact_replay",
            VariantType::Isomorphic => "isomorphic",
            VariantType::Structural => "structural",
            VariantType::Reverse => "reverse",
            VariantType::BoundaryTrap => "boundary_trap",
            VariantType::Transfer => "transfer",
        }
    }
}

impl std::fmt::Display for VariantType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Pedagogical level of the mathematical learning object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LearningObjectLevel {
    /// Declarative Trigger: "What principle/formula is relevant?"
    DeclarativeTrigger,
    /// Strategy Selection: "What method/schema should I use?"
    StrategySelection,
    /// Procedural Execution: "Carry out the method."
    ProceduralExecution,
    /// Variation: "Same schema, changed structure or parameter range."
    Variation,
    /// Transfer: "Same underlying skill, substantially novel surface context."
    Transfer,
}

impl LearningObjectLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LearningObjectLevel::DeclarativeTrigger => "declarative_trigger",
            LearningObjectLevel::StrategySelection => "strategy_selection",
            LearningObjectLevel::ProceduralExecution => "procedural_execution",
            LearningObjectLevel::Variation => "variation",
            LearningObjectLevel::Transfer => "transfer",
        }
    }
}

impl std::fmt::Display for LearningObjectLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Exam-style problem metadata for realistic practice and timed benchmark testing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExamMetadata {
    pub target_latency_ms: u64,
    pub difficulty_level: u32,
    pub source_style: String,
    pub question_format: String,
    pub distractors: Vec<String>,
}

impl ExamMetadata {
    pub fn new(
        target_latency_ms: u64,
        difficulty_level: u32,
        source_style: impl Into<String>,
        question_format: impl Into<String>,
        distractors: Vec<String>,
    ) -> Self {
        Self {
            target_latency_ms,
            difficulty_level,
            source_style: source_style.into(),
            question_format: question_format.into(),
            distractors,
        }
    }
}

/// Domain-agnostic generator interface for deterministic problem instance generation.
pub trait ProblemGenerator: Send + Sync {
    /// Unique canonical problem family ID (e.g. "family.math.algebra.linear_equations")
    fn family_id(&self) -> &str;

    /// Engine template reference (e.g. "math.algebra.linear_equations.v1")
    fn template_ref(&self) -> &str;

    /// Supported discrete difficulty level range, defaults to 1..=5.
    fn difficulty_range(&self) -> (u32, u32) {
        (1, 5)
    }

    /// List of supported variant names for this problem family.
    fn supported_variants(&self) -> Vec<String>;

    /// Target latency benchmark in milliseconds for a specific difficulty level (1..=5).
    fn target_latency_ms(&self, difficulty_level: u32) -> u64;

    /// Deterministically generate a problem instance from a 64-bit seed, difficulty level,
    /// and optional variant identifier.
    fn generate(
        &self,
        family_id: &ProblemFamilyId,
        seed: u64,
        difficulty_level: u32,
        variant: Option<&str>,
    ) -> Result<ProblemInstance>;
}
