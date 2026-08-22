// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use procedural::core::SkillId;
use procedural::skills::lifecycle::{MaintenanceReviewOutcome, RetirementPolicy};
use procedural::skills::signals::PracticeProgressionState;
use procedural::skills::SkillState;

#[test]
fn test_retirement_eligibility_evaluation() {
    let skill_id = SkillId::new("maths.percentage.successive");
    let mut state = SkillState::new(skill_id);
    let policy = RetirementPolicy::default();

    // 1. Not eligible if in Learning or Fluent stage
    state.practice_state = PracticeProgressionState::Fluent;
    state.mastery = 0.90;
    let eval1 = policy.evaluate_retirement_eligibility(&state);
    assert!(!eval1.is_eligible);

    // 2. Not eligible if variant coverage < 3
    state.practice_state = PracticeProgressionState::Mastered;
    state.mastery = 0.90;
    state.record_variant_exposure("v1", true, 20_000, None, 100);
    state.record_variant_exposure("v2", true, 20_000, None, 101);
    let eval2 = policy.evaluate_retirement_eligibility(&state);
    assert!(!eval2.is_eligible);

    // 3. Eligible when Mastered, high mastery, >= 3 variants, low error
    state.record_variant_exposure("v3", true, 20_000, None, 102);
    for _ in 0..5 {
        state.recent_attempts.push(procedural::skills::signals::RecentAttemptRecord {
            is_correct: true,
            score: 1.0,
            latency_ms: 20_000,
            target_latency_ms: 30_000,
            variant: Some("v1".into()),
            variant_category: Some(procedural::VariantCategory::Structural),
            error_category: None,
            max_hint_level: None,
            hint_count: None,
            independence: Some(procedural::IndependenceLevel::Independent),
            solution_graph_fingerprint: None,
            cognitive_decision_correct: Some(true),
            timestamp: 100,
            domain_evidence: None,
        });
    }

    let eval3 = policy.evaluate_retirement_eligibility(&state);
    assert!(eval3.is_eligible);
}

#[test]
fn test_maintenance_review_reactivation_on_failure() {
    let skill_id = SkillId::new("chemistry.stoichiometry.moles");
    let mut state = SkillState::new(skill_id);
    state.practice_state = PracticeProgressionState::Retired;

    // 1. Success on maintenance -> ConfirmedFluency (remains Retired)
    let outcome_success = RetirementPolicy::evaluate_maintenance_attempt(&mut state, true, false);
    assert!(matches!(outcome_success, MaintenanceReviewOutcome::ConfirmedFluency { .. }));
    assert_eq!(state.practice_state, PracticeProgressionState::Retired);

    // 2. Conceptual failure on maintenance -> Reactivated to Learning stage
    let outcome_concept_fail = RetirementPolicy::evaluate_maintenance_attempt(&mut state, false, true);
    assert!(matches!(
        outcome_concept_fail,
        MaintenanceReviewOutcome::ReactivatedToActiveStage {
            reactivated_stage: PracticeProgressionState::Learning,
            ..
        }
    ));
    assert_eq!(state.practice_state, PracticeProgressionState::Learning);

    // 3. Execution failure on retired skill after multiple failures -> Reactivated to Variation
    state.practice_state = PracticeProgressionState::Retired;
    state.consecutive_failures = 2;
    let outcome_exec_fail = RetirementPolicy::evaluate_maintenance_attempt(&mut state, false, false);
    assert!(matches!(
        outcome_exec_fail,
        MaintenanceReviewOutcome::ReactivatedToActiveStage {
            reactivated_stage: PracticeProgressionState::Variation,
            ..
        }
    ));
    assert_eq!(state.practice_state, PracticeProgressionState::Variation);
}