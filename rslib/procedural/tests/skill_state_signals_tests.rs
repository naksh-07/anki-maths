// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use procedural::diagnostics::ErrorCategory;
use procedural::skills::signals::{IndependenceLevel, MasteryEvidence};
use procedural::skills::{PracticeProgressionState, SkillState};

/// Helper: build a simple correct MasteryEvidence
fn ev_ok(latency: u64, variant: &str) -> MasteryEvidence {
    MasteryEvidence {
        final_correctness: true,
        latency_evidence: latency,
        variant_exposure: Some(variant.to_string()),
        independence: IndependenceLevel::Independent,
        ..Default::default()
    }
}

/// Helper: build a failure MasteryEvidence
fn ev_fail(latency: u64, variant: Option<&str>, error: Option<ErrorCategory>) -> MasteryEvidence {
    MasteryEvidence {
        final_correctness: false,
        latency_evidence: latency,
        variant_exposure: variant.map(|s| s.to_string()),
        diagnostic_errors: error.map(|e| vec![e]).unwrap_or_default(), domain_evidence: None,
        ..Default::default()
    }
}

#[test]
fn test_sliding_window_accuracy_and_latency() {
    let mut state = SkillState::new("skill.maths.percentages").with_window_size(4);

    // Initial state
    assert_eq!(state.practice_state, PracticeProgressionState::New);
    assert_eq!(state.recent_accuracy(), 0.0);
    assert_eq!(state.moving_average_latency_ms(), 0.0);

    // Add 4 consecutive correct attempts
    state.record_attempt_outcome(&ev_ok(10_000, "forward_two_step"), 1.0, 30_000, 100);
    state.record_attempt_outcome(&ev_ok(20_000, "forward_two_step"), 1.0, 30_000, 200);
    state.record_attempt_outcome(&ev_ok(15_000, "forward_two_step"), 1.0, 30_000, 300);
    state.record_attempt_outcome(&ev_ok(25_000, "forward_two_step"), 1.0, 30_000, 400);

    assert_eq!(state.total_attempts, 4);
    assert_eq!(state.successful_attempts, 4);
    assert_eq!(state.failed_attempts, 0);
    assert_eq!(state.consecutive_successes, 4);
    assert_eq!(state.consecutive_failures, 0);
    assert_eq!(state.recent_accuracy(), 1.0);
    // Mean of 10, 20, 15, 25 = 70 / 4 = 17.5s (17500ms)
    assert_eq!(state.moving_average_latency_ms(), 17_500.0);
    assert_eq!(state.latency_stats.min_latency_ms, Some(10_000));
    assert_eq!(state.latency_stats.max_latency_ms, Some(25_000));
    assert!(state.latency_stats.moving_variance.is_some());

    // 5th attempt: failure (pushes out 1st attempt)
    state.record_attempt_outcome(
        &ev_fail(35_000, Some("reverse_initial"), Some(ErrorCategory::Strategy)),
        0.0,
        30_000,
        500,
    );

    assert_eq!(state.total_attempts, 5);
    assert_eq!(state.successful_attempts, 4);
    assert_eq!(state.failed_attempts, 1);
    assert_eq!(state.consecutive_successes, 0);
    assert_eq!(state.consecutive_failures, 1);
    // Window has [20000, 15000, 25000, 35000]: 3 correct out of 4 -> 0.75
    assert_eq!(state.recent_accuracy(), 0.75);
    // Mean of 20, 15, 25, 35 = 95 / 4 = 23750ms
    assert_eq!(state.moving_average_latency_ms(), 23_750.0);
}

#[test]
fn test_error_frequency_tracking() {
    let mut state = SkillState::new("skill.maths.percentages");

    state.record_attempt_outcome(&ev_fail(30_000, None, Some(ErrorCategory::Concept)), 0.0, 30_000, 1);
    state.record_attempt_outcome(&ev_fail(30_000, None, Some(ErrorCategory::Concept)), 0.0, 30_000, 2);
    state.record_attempt_outcome(&ev_fail(30_000, None, Some(ErrorCategory::Calculation)), 0.0, 30_000, 3);
    state.record_attempt_outcome(&ev_fail(30_000, None, Some(ErrorCategory::Careless)), 0.0, 30_000, 4);
    state.record_attempt_outcome(&ev_fail(30_000, None, Some(ErrorCategory::Time)), 0.0, 30_000, 5);

    assert_eq!(state.error_counts.get_count(&ErrorCategory::Concept), 2);
    assert_eq!(state.error_counts.get_count(&ErrorCategory::Calculation), 1);
    assert_eq!(state.error_counts.get_count(&ErrorCategory::Careless), 1);
    assert_eq!(state.error_counts.get_count(&ErrorCategory::Time), 1);
    assert_eq!(state.error_counts.get_count(&ErrorCategory::Unknown), 0);
    assert_eq!(state.error_counts.total_errors(), 5);
    assert_eq!(state.error_counts.primary_error_category(), Some(ErrorCategory::Concept));
}

#[test]
fn test_variant_performance_exposure() {
    let mut state = SkillState::new("skill.maths.percentages");

    // Practice ForwardTwoStep 3 times
    state.record_attempt_outcome(&ev_ok(20_000, "forward_two_step"), 1.0, 30_000, 1);
    state.record_attempt_outcome(&ev_ok(22_000, "forward_two_step"), 1.0, 30_000, 2);
    state.record_attempt_outcome(&ev_fail(30_000, Some("forward_two_step"), Some(ErrorCategory::Careless)), 0.0, 30_000, 3);

    // Practice ReverseInitial twice
    state.record_attempt_outcome(&ev_ok(40_000, "reverse_initial"), 1.0, 45_000, 4);
    state.record_attempt_outcome(&ev_ok(38_000, "reverse_initial"), 1.0, 45_000, 5);

    let fwd = state.variant_stats.get("forward_two_step").unwrap();
    assert_eq!(fwd.total_attempts, 3);
    assert_eq!(fwd.successful_attempts, 2);
    assert_eq!(fwd.failed_attempts, 1);
    assert!((fwd.success_rate() - (2.0 / 3.0)).abs() < 1e-6);

    let rev = state.variant_stats.get("reverse_initial").unwrap();
    assert_eq!(rev.total_attempts, 2);
    assert_eq!(rev.successful_attempts, 2);
    assert_eq!(rev.failed_attempts, 0);
    assert_eq!(rev.success_rate(), 1.0);
    assert_eq!(rev.average_latency_ms, 39_000.0);
}

#[test]
fn test_deterministic_progression_state_transitions() {
    let mut state = SkillState::new("skill.maths.percentages").with_window_size(5);
    assert_eq!(state.practice_state, PracticeProgressionState::New);

    // 1st attempt advances to Learning
    state.record_attempt_outcome(&ev_ok(25_000, "forward_two_step"), 1.0, 30_000, 1);
    assert_eq!(state.practice_state, PracticeProgressionState::Learning);

    // 2 more correct attempts (3 in window, 100% acc, 3 consecutive) -> Fluent
    state.record_attempt_outcome(&ev_ok(24_000, "forward_two_step"), 1.0, 30_000, 2);
    state.record_attempt_outcome(&ev_ok(22_000, "forward_two_step"), 1.0, 30_000, 3);
    assert_eq!(state.practice_state, PracticeProgressionState::Fluent);

    // Practice another variant -> advances to Variation
    state.record_attempt_outcome(&ev_ok(35_000, "reverse_initial"), 1.0, 40_000, 4);
    assert_eq!(state.practice_state, PracticeProgressionState::Variation);

    // Solid performance on multiple variants -> advances to Transfer
    state.record_attempt_outcome(&ev_ok(34_000, "reverse_initial"), 1.0, 40_000, 5);
    assert_eq!(state.practice_state, PracticeProgressionState::Transfer);

    // Multiple consecutive failures step down state
    state.record_attempt_outcome(&ev_fail(45_000, Some("forward_two_step"), Some(ErrorCategory::Concept)), 0.0, 30_000, 6);
    state.record_attempt_outcome(&ev_fail(45_000, Some("forward_two_step"), Some(ErrorCategory::Concept)), 0.0, 30_000, 7);
    state.record_attempt_outcome(&ev_fail(45_000, Some("forward_two_step"), Some(ErrorCategory::Concept)), 0.0, 30_000, 8);
    assert_eq!(state.practice_state, PracticeProgressionState::Variation);
}