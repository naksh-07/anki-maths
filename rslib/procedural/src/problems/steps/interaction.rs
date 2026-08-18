// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use serde::{Deserialize, Serialize};

/// Interaction style / mode for procedural problem practice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InteractionMode {
    /// Fast direct final answer input only (standard UX).
    FinalAnswerOnly,
    /// Structured step-by-step input where intermediate reasoning is entered.
    StepwiseAttempt,
    /// Guided solve mode with structured steps and deterministic progressive hints.
    GuidedSolve,
    /// Deep diagnostic mode with mandatory step-level error localization.
    Diagnostic,
    /// Rapid speed drill with final answer and strict time bounds.
    Speed,
}

impl Default for InteractionMode {
    fn default() -> Self {
        InteractionMode::FinalAnswerOnly
    }
}

impl InteractionMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            InteractionMode::FinalAnswerOnly => "final_answer_only",
            InteractionMode::StepwiseAttempt => "stepwise_attempt",
            InteractionMode::GuidedSolve => "guided_solve",
            InteractionMode::Diagnostic => "diagnostic",
            InteractionMode::Speed => "speed",
        }
    }
}

/// A single step submitted by the learner during a stepwise practice attempt.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubmittedStep {
    /// Optional step ID matching a known node in the solution graph
    pub step_id: Option<String>,
    /// Step order index (0-indexed)
    pub step_index: usize,
    /// Learner's typed mathematical expression, equation, or intermediate answer
    pub content: String,
    /// Milliseconds spent writing/submitting this specific step
    pub time_taken_ms: u64,
}

impl SubmittedStep {
    pub fn new(step_index: usize, content: impl Into<String>, time_taken_ms: u64) -> Self {
        Self {
            step_id: None,
            step_index,
            content: content.into(),
            time_taken_ms,
        }
    }

    pub fn with_step_id(mut self, step_id: impl Into<String>) -> Self {
        self.step_id = Some(step_id.into());
        self
    }
}

/// Complete submission payload for a procedural problem attempt (supporting both final-answer and stepwise modes).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StepwiseSubmission {
    pub mode: InteractionMode,
    /// Submitted intermediate steps if stepwise mode was used
    pub steps: Vec<SubmittedStep>,
    /// Final answer string or JSON value if provided
    pub final_answer: Option<String>,
    /// Number of deterministic hints requested during this attempt
    pub hints_used: u32,
    /// Total duration of attempt in milliseconds
    pub total_time_ms: u64,
    /// Latency before the user performed their first action / typed first step
    pub first_action_latency_ms: Option<u64>,
    /// Timestamps (offsets in ms from start) when hints were requested
    pub hint_timestamps_ms: Vec<u64>,
}

impl StepwiseSubmission {
    pub fn final_answer_only(answer: impl Into<String>, time_taken_ms: u64) -> Self {
        Self {
            mode: InteractionMode::FinalAnswerOnly,
            steps: Vec::new(),
            final_answer: Some(answer.into()),
            hints_used: 0,
            total_time_ms: time_taken_ms,
            first_action_latency_ms: None,
            hint_timestamps_ms: Vec::new(),
        }
    }

    pub fn stepwise(steps: Vec<SubmittedStep>, final_answer: Option<String>, total_time_ms: u64) -> Self {
        Self {
            mode: InteractionMode::StepwiseAttempt,
            steps,
            final_answer,
            hints_used: 0,
            total_time_ms,
            first_action_latency_ms: None,
            hint_timestamps_ms: Vec::new(),
        }
    }

    pub fn with_hints(mut self, hints_used: u32, hint_timestamps: Vec<u64>) -> Self {
        self.hints_used = hints_used;
        self.hint_timestamps_ms = hint_timestamps;
        self
    }

    pub fn with_first_action_latency(mut self, ms: u64) -> Self {
        self.first_action_latency_ms = Some(ms);
        self
    }

    pub fn with_mode(mut self, mode: InteractionMode) -> Self {
        self.mode = mode;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interaction_submission_modes() {
        let final_sub = StepwiseSubmission::final_answer_only("42", 15_000);
        assert_eq!(final_sub.mode, InteractionMode::FinalAnswerOnly);
        assert_eq!(final_sub.final_answer.as_deref(), Some("42"));
        assert!(final_sub.steps.is_empty());

        let steps = vec![
            SubmittedStep::new(0, "3x = 12", 5000),
            SubmittedStep::new(1, "x = 4", 3000),
        ];
        let stepwise_sub = StepwiseSubmission::stepwise(steps, Some("4".to_string()), 8000)
            .with_hints(1, vec![2000]);

        assert_eq!(stepwise_sub.mode, InteractionMode::StepwiseAttempt);
        assert_eq!(stepwise_sub.steps.len(), 2);
        assert_eq!(stepwise_sub.hints_used, 1);
        assert_eq!(stepwise_sub.hint_timestamps_ms, vec![2000]);
    }
}
