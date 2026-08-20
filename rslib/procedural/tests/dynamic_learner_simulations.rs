// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use procedural::core::{Domain, SchemaId, SkillId};
use procedural::diagnostics::hints::{HintDependencyStats, HintLevel, HintUsageRecord};
use procedural::diagnostics::ErrorCategory;
use procedural::practice::{PracticeObjective, PracticeRequest, PracticeScope, SessionBudget};
use procedural::remediation::RemediationActionKind;
use procedural::scheduling::speed::{DomainSpeedConfig, SpeedRating, StageSpeedPolicy};
use procedural::scheduling::transfer::{TransferEngine, TransferLevel};
use procedural::scheduling::workload::{SessionBudgetTracker, WorkloadSnapshot, WorkloadState};
use procedural::skills::lifecycle::{MaintenanceReviewOutcome, RetirementPolicy};
use procedural::skills::signals::{IndependenceLevel, MasteryEvidence, PracticeProgressionState};
use procedural::skills::{ProgressionPolicy, SkillState};

#[test]
fn test_simulation_learner_a_new_slow_correct() {
    let skill_id = SkillId::new("maths.percentage.successive");
    let mut state = SkillState::new(skill_id);
    assert_eq!(state.practice_state, PracticeProgressionState::New);

    // Learner A takes 75s (slower than 30s target), but solves correctly
    let speed_eval = StageSpeedPolicy::evaluate(
        state.practice_state,
        Domain::Mathematics,
        75_000,
        None,
    );

    // In New stage, latency weight is 0.0 (informational only) and attempt is acceptable
    assert_eq!(speed_eval.effective_latency_weight, 0.0);
    assert!(speed_eval.is_acceptable);

    let evidence = MasteryEvidence {
        final_correctness: true,
        decision_quality: Some(1.0),
        step_quality: None,
        independence: IndependenceLevel::Independent,
        max_hint_level: None,
        hint_dependence: 0,
        retry_dependence: 0,
        variant_exposure: Some("standard".into()),
        variant_category: procedural::VariantCategory::Parameter,
        solution_graph_fingerprint: None,
        cognitive_decision_correct: Some(true),
        time_since_last_ms: None,
        transfer_evidence: false,
        domain_competence_verified: Some(true),
        latency_evidence: 75_000,
        diagnostic_errors: vec![],
    };

    state.total_attempts = 1;
    ProgressionPolicy::evaluate(&mut state, &evidence);
    // Advances to Learning without being blocked by early latency
    assert_eq!(state.practice_state, PracticeProgressionState::Learning);
}

#[test]
fn test_simulation_learner_b_fluent_fast_correct() {
    let skill_id = SkillId::new("physics.kinematics.1d");
    let mut state = SkillState::new(skill_id);
    state.practice_state = PracticeProgressionState::Fluent;
    state.total_attempts = 5;
    state.historical_independent_count = 5;

    // Learner B takes 25s (well under 45s target)
    let speed_eval = StageSpeedPolicy::evaluate(
        state.practice_state,
        Domain::Physics,
        25_000,
        None,
    );
    assert_eq!(speed_eval.speed_rating, SpeedRating::Optimal);
    assert_eq!(speed_eval.fluency_score, 1.0);

    // Practice variants with high independence
    state.record_variant_exposure("v_angle", true, 25_000, None, 100);
    state.record_variant_exposure("v_gravity", true, 26_000, None, 101);
    state.consecutive_successes = 3;

    let evidence = MasteryEvidence {
        final_correctness: true,
        decision_quality: Some(1.0),
        step_quality: None,
        independence: IndependenceLevel::Independent,
        max_hint_level: None,
        hint_dependence: 0,
        retry_dependence: 0,
        variant_exposure: Some("v_gravity".into()),
        variant_category: procedural::VariantCategory::Structural,
        solution_graph_fingerprint: None,
        cognitive_decision_correct: Some(true),
        time_since_last_ms: None,
        transfer_evidence: false,
        domain_competence_verified: Some(true),
        latency_evidence: 26_000,
        diagnostic_errors: vec![],
    };

    ProgressionPolicy::evaluate(&mut state, &evidence);
    assert_eq!(state.practice_state, PracticeProgressionState::Variation);
}

#[test]
fn test_simulation_learner_c_heavy_hints() {
    let records = vec![
        HintUsageRecord::new(HintLevel::Level3_NearSolutionSupport, 2, true),
        HintUsageRecord::new(HintLevel::Level3_NearSolutionSupport, 3, true),
        HintUsageRecord::new(HintLevel::Level3_NearSolutionSupport, 1, true),
    ];

    let stats = HintDependencyStats::from_records(&records);
    assert!(stats.has_chronic_dependence());
    assert!(stats.composite_independence_score <= 0.35);

    let skill_id = SkillId::new("reasoning.seating.linear");
    let mut state = SkillState::new(skill_id);
    state.practice_state = PracticeProgressionState::Transfer;

    let evidence = MasteryEvidence {
        final_correctness: true,
        decision_quality: Some(1.0),
        step_quality: None,
        independence: IndependenceLevel::SignificantSupport,
        max_hint_level: Some(3),
        hint_dependence: 3,
        retry_dependence: 0,
        variant_exposure: Some("seating_circular".into()),
        variant_category: procedural::VariantCategory::Transfer,
        solution_graph_fingerprint: None,
        cognitive_decision_correct: Some(true),
        time_since_last_ms: None,
        transfer_evidence: true,
        domain_competence_verified: Some(true),
        latency_evidence: 30_000,
        diagnostic_errors: vec![],
    };

    ProgressionPolicy::evaluate(&mut state, &evidence);
    // Blocked from false Mastered state due to hint dependency
    assert_ne!(state.practice_state, PracticeProgressionState::Mastered);
}

#[test]
fn test_simulation_learner_d_strong_standard_weak_transfer() {
    let skill_id = SkillId::new("chemistry.stoichiometry.moles");
    let schema_id = SchemaId::new("chemistry_stoichiometry");

    let mut state = SkillState::new(skill_id.clone());
    state.practice_state = PracticeProgressionState::Variation;
    state.total_attempts = 4;
    state.consecutive_successes = 3;
    for _ in 0..3 {
        state.recent_attempts.push(procedural::skills::signals::RecentAttemptRecord {
            is_correct: true,
            score: 1.0,
            latency_ms: 20_000,
            target_latency_ms: 45_000,
            variant: Some("standard_moles".into()),
            variant_category: Some(procedural::VariantCategory::Parameter),
            error_category: None,
            max_hint_level: None,
            hint_count: None,
            independence: Some(procedural::IndependenceLevel::Independent),
            solution_graph_fingerprint: None,
            cognitive_decision_correct: Some(true),
            timestamp: 100,
        });
    }
    state.record_variant_exposure("standard_moles", true, 20_000, None, 100);
    state.record_variant_exposure("limiting_reagent", true, 22_000, None, 101);

    // Meets NearTransfer eligibility
    let eval_transfer = TransferEngine::evaluate_eligibility(&state, TransferLevel::NearTransfer, false);
    assert!(eval_transfer.is_eligible);

    // Learner attempts transfer but makes a strategic mistake
    let remediation = TransferEngine::classify_transfer_failure(
        &skill_id,
        &schema_id,
        Domain::Chemistry,
        TransferLevel::StructuralTransfer,
        Some(ErrorCategory::Strategy),
        None,
    );

    assert_eq!(remediation.kind, RemediationActionKind::StrategyDrill);
    assert_eq!(remediation.skill_id, skill_id);
}

#[test]
fn test_simulation_learner_e_long_break_mastered_retirement_and_reactivation() {
    let skill_id = SkillId::new("maths.algebra.linear");
    let mut state = SkillState::new(skill_id);
    state.practice_state = PracticeProgressionState::Mastered;
    state.mastery = 0.92;
    state.record_variant_exposure("v1", true, 15_000, None, 100);
    state.record_variant_exposure("v2", true, 16_000, None, 101);
    state.record_variant_exposure("v3", true, 14_000, None, 102);

    for _ in 0..5 {
        state.recent_attempts.push(procedural::skills::signals::RecentAttemptRecord {
            is_correct: true,
            score: 1.0,
            latency_ms: 15_000,
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
        });
    }

    let policy = RetirementPolicy::default();
    let ret_eval = policy.evaluate_retirement_eligibility(&state);
    assert!(ret_eval.is_eligible);

    // Transition to Retired
    state.practice_state = PracticeProgressionState::Retired;

    // Maintenance check encounters a conceptual error -> Reactivates to Learning
    let outcome = RetirementPolicy::evaluate_maintenance_attempt(&mut state, false, true);
    assert!(matches!(
        outcome,
        MaintenanceReviewOutcome::ReactivatedToActiveStage {
            reactivated_stage: PracticeProgressionState::Learning,
            ..
        }
    ));
    assert_eq!(state.practice_state, PracticeProgressionState::Learning);
}

#[test]
fn test_simulation_learner_f_exam_cram_overloaded_protection() {
    let snapshot = WorkloadSnapshot {
        pending_remediation_count: 5,
        critical_remediation_count: 2,
        due_memory_reviews: 18,
        active_learning_skills: 5,
        transfer_pending_count: 3,
        total_composite_load: 28,
    };

    assert_eq!(snapshot.compute_state(), WorkloadState::Overloaded);

    // Budget constraints protect session duration
    let mut tracker = SessionBudgetTracker::new(Some(SessionBudget::ItemCount { max_items: 2 }));
    tracker.record_item(30_000, true);
    assert!(!tracker.is_exhausted);
    tracker.record_item(30_000, false);
    assert!(tracker.is_exhausted);
}
