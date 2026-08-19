// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use procedural::diagnostics::ErrorCategory;
use procedural::problems::generators::percentage_successive::PercentageVariant;
use procedural::scheduling::VariantSelector;
use procedural::skills::signals::{IndependenceLevel, MasteryEvidence};
use procedural::skills::{PracticeProgressionState, SkillState};

fn ev_ok(latency: u64, variant: &str) -> MasteryEvidence {
    MasteryEvidence {
        final_correctness: true,
        latency_evidence: latency,
        variant_exposure: Some(variant.to_string()),
        independence: IndependenceLevel::Independent,
        ..Default::default()
    }
}

fn ev_fail(latency: u64, variant: &str, error: ErrorCategory) -> MasteryEvidence {
    MasteryEvidence {
        final_correctness: false,
        latency_evidence: latency,
        variant_exposure: Some(variant.to_string()),
        diagnostic_errors: vec![error],
        ..Default::default()
    }
}

#[test]
fn test_concept_failure_triggers_standard_variant() {
    let mut state = SkillState::new("skill.percentage.successive");
    state.practice_state = PracticeProgressionState::Fluent;
    // Failed on a hard variant due to concept error
    state.record_attempt_outcome(
        &ev_fail(30_000, "forward_three_step", ErrorCategory::Concept),
        0.0,
        45_000,
        1000,
    );

    let decision = VariantSelector::select_variant(Some(&state), None, 123);
    assert_eq!(decision.variant, PercentageVariant::ForwardTwoStep);
    assert_eq!(
        decision.selection_reason,
        "remediate_concept_error_standard_variant"
    );
}

#[test]
fn test_reverse_initial_failure_remediation() {
    let mut state = SkillState::new("skill.percentage.successive");
    state.practice_state = PracticeProgressionState::Variation;
    // Failed on ReverseInitial due to calculation
    state.record_attempt_outcome(
        &ev_fail(40_000, "reverse_initial", ErrorCategory::Calculation),
        0.0,
        45_000,
        1000,
    );

    let decision = VariantSelector::select_variant(Some(&state), None, 456);
    assert_eq!(decision.variant, PercentageVariant::ReverseInitial);
    assert_eq!(
        decision.selection_reason,
        "remediate_failed_variant:reverse_initial"
    );
}

#[test]
fn test_slow_success_reinforces_fluency_without_structural_jump() {
    let mut state = SkillState::new("skill.percentage.successive");
    state.practice_state = PracticeProgressionState::Fluent;
    // Target 35s, learner took 60s (> 1.25x 35s = 43.75s)
    state.record_attempt_outcome(
        &ev_ok(60_000, "forward_two_step"),
        1.0,
        35_000,
        1000,
    );

    let decision = VariantSelector::select_variant(Some(&state), None, 789);
    assert_eq!(decision.variant, PercentageVariant::ForwardTwoStep);
    assert_eq!(
        decision.selection_reason,
        "fluency_reinforcement_slow_latency"
    );
}

#[test]
fn test_fast_strong_performance_introduces_advanced_variation() {
    let mut state = SkillState::new("skill.percentage.successive");
    state.practice_state = PracticeProgressionState::Fluent;
    // 2 consecutive fast successes (12s and 14s on 35s target)
    state.record_attempt_outcome(&ev_ok(12_000, "forward_two_step"), 1.0, 35_000, 1000);
    state.record_attempt_outcome(&ev_ok(14_000, "forward_two_step"), 1.0, 35_000, 1050);

    let decision = VariantSelector::select_variant(Some(&state), None, 999);
    assert_ne!(decision.variant, PercentageVariant::ForwardTwoStep);
    assert_eq!(
        decision.selection_reason,
        "introduce_structural_variation"
    );
}

#[test]
fn test_anti_priming_prevents_immediate_sibling_repetition() {
    let mut state = SkillState::new("skill.percentage.successive");
    state.practice_state = PracticeProgressionState::Variation;
    // Just solved ReverseInitial successfully
    state.record_attempt_outcome(&ev_ok(35_000, "reverse_initial"), 1.0, 45_000, 1000);

    // Anti-priming should suppress ReverseInitial from being selected next
    let decision = VariantSelector::select_variant(Some(&state), None, 54321);
    assert_ne!(decision.variant, PercentageVariant::ReverseInitial);
}

#[test]
fn test_deterministic_seed_selection_reproducibility() {
    let mut state = SkillState::new("skill.percentage.successive");
    state.practice_state = PracticeProgressionState::Variation;
    state.record_attempt_outcome(&ev_ok(25_000, "forward_two_step"), 1.0, 35_000, 1000);

    let d1 = VariantSelector::select_variant(Some(&state), None, 88888);
    let d2 = VariantSelector::select_variant(Some(&state), None, 88888);
    let d3 = VariantSelector::select_variant(Some(&state), None, 88888);

    assert_eq!(d1, d2);
    assert_eq!(d2, d3);
}
