// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use procedural::diagnostics::{ErrorCategory, ProceduralReviewOutcome};
use procedural::scheduling::{derive_fsrs_rating, Rating};
use procedural::skills::SkillState;

fn create_outcome(
    is_correct: bool,
    score: f64,
    latency_ms: u64,
    target_latency_ms: u64,
    hints_used: u32,
    attempt_count: u32,
    error_category: Option<ErrorCategory>,
) -> ProceduralReviewOutcome {
    let mut outcome = ProceduralReviewOutcome::new(
        "att-fsrs-test",
        "schema.test",
        "skill.test",
        "family.test",
        12345,
        is_correct,
        score,
        latency_ms,
        target_latency_ms,
        hints_used,
        attempt_count,
        error_category,
    );
    outcome.timestamp = 10000;
    outcome
}

#[test]
fn test_incorrect_maps_to_again() {
    let outcome = create_outcome(false, 0.0, 25_000, 30_000, 0, 1, None);
    assert_eq!(derive_fsrs_rating(&outcome, None), Rating::Again);
}

#[test]
fn test_concept_breakdown_maps_to_again() {
    let outcome = create_outcome(true, 1.0, 20_000, 30_000, 0, 1, Some(ErrorCategory::Concept));
    assert_eq!(derive_fsrs_rating(&outcome, None), Rating::Again);

    let outcome_strat = create_outcome(true, 1.0, 20_000, 30_000, 0, 1, Some(ErrorCategory::Strategy));
    assert_eq!(derive_fsrs_rating(&outcome_strat, None), Rating::Again);
}

#[test]
fn test_exhausted_attempts_or_excessive_hints_maps_to_again() {
    // 4 attempts -> Again
    let outcome_attempts = create_outcome(true, 1.0, 20_000, 30_000, 0, 4, None);
    assert_eq!(derive_fsrs_rating(&outcome_attempts, None), Rating::Again);

    // 3 hints -> Again
    let outcome_hints = create_outcome(true, 1.0, 20_000, 30_000, 3, 1, None);
    assert_eq!(derive_fsrs_rating(&outcome_hints, None), Rating::Again);
}

#[test]
fn test_correct_but_slow_maps_to_hard() {
    // Target 40s. 1.25x is 50s. 55s latency -> Hard
    let outcome = create_outcome(true, 1.0, 55_000, 40_000, 0, 1, None);
    assert_eq!(derive_fsrs_rating(&outcome, None), Rating::Hard);
}

#[test]
fn test_correct_with_hints_or_second_attempt_maps_to_hard() {
    let outcome_hint = create_outcome(true, 1.0, 25_000, 40_000, 1, 1, None);
    assert_eq!(derive_fsrs_rating(&outcome_hint, None), Rating::Hard);

    let outcome_att = create_outcome(true, 1.0, 25_000, 40_000, 0, 2, None);
    assert_eq!(derive_fsrs_rating(&outcome_att, None), Rating::Hard);
}

#[test]
fn test_correct_with_calculation_slip_maps_to_hard() {
    let outcome_slip = create_outcome(true, 1.0, 25_000, 40_000, 0, 1, Some(ErrorCategory::Calculation));
    assert_eq!(derive_fsrs_rating(&outcome_slip, None), Rating::Hard);
}

#[test]
fn test_correct_with_recent_struggles_maps_to_hard() {
    let outcome = create_outcome(true, 1.0, 35_000, 40_000, 0, 1, None);

    let mut state = SkillState::new("skill.test");
    state.consecutive_failures = 2;

    assert_eq!(derive_fsrs_rating(&outcome, Some(&state)), Rating::Hard);
}

#[test]
fn test_correct_normal_latency_maps_to_good() {
    // Target 40s. Latency 38s -> Good
    let outcome = create_outcome(true, 1.0, 38_000, 40_000, 0, 1, None);
    assert_eq!(derive_fsrs_rating(&outcome, None), Rating::Good);
}

#[test]
fn test_correct_and_fast_maps_to_easy() {
    // Target 40s. 0.75x is 30s. Latency 22s -> Easy (when history is strong)
    let outcome = create_outcome(true, 1.0, 22_000, 40_000, 0, 1, None);

    let mut state = SkillState::new("skill.test");
    state.consecutive_successes = 3;
    assert_eq!(derive_fsrs_rating(&outcome, Some(&state)), Rating::Easy);
}

#[test]
fn test_rating_boundary_conditions() {
    let target = 40_000;

    // Exactly at target (40s) -> Good
    let o_at_target = create_outcome(true, 1.0, 40_000, target, 0, 1, None);
    assert_eq!(derive_fsrs_rating(&o_at_target, None), Rating::Good);

    // Exactly at slow threshold (50s) -> Good
    let o_at_slow = create_outcome(true, 1.0, 50_000, target, 0, 1, None);
    assert_eq!(derive_fsrs_rating(&o_at_slow, None), Rating::Good);

    // 1ms past slow threshold (50_001ms) -> Hard
    let o_past_slow = create_outcome(true, 1.0, 50_001, target, 0, 1, None);
    assert_eq!(derive_fsrs_rating(&o_past_slow, None), Rating::Hard);

    // Exactly at fast threshold (30s) -> Easy with history
    let mut strong_state = SkillState::new("skill.test");
    strong_state.consecutive_successes = 2;
    let o_at_fast = create_outcome(true, 1.0, 30_000, target, 0, 1, None);
    assert_eq!(derive_fsrs_rating(&o_at_fast, Some(&strong_state)), Rating::Easy);

    // 1ms above fast threshold (30_001ms) -> Good
    let o_above_fast = create_outcome(true, 1.0, 30_001, target, 0, 1, None);
    assert_eq!(derive_fsrs_rating(&o_above_fast, Some(&strong_state)), Rating::Good);
}