// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use procedural::core::{Domain, SchemaId, SkillId};
use procedural::diagnostics::ErrorCategory;
use procedural::remediation::RemediationActionKind;
use procedural::scheduling::transfer::{TransferEngine, TransferLevel};
use procedural::skills::signals::PracticeProgressionState;
use procedural::skills::SkillState;

#[test]
fn test_transfer_level_eligibility_progression() {
    let skill_id = SkillId::new("maths.algebra.linear");
    let mut state = SkillState::new(skill_id);

    // 1. New skill is not eligible for transfer
    let eval_new = TransferEngine::evaluate_eligibility(&state, TransferLevel::NearTransfer, false);
    assert!(!eval_new.is_eligible);

    // 2. Fluent stage with 80% accuracy -> eligible for NearTransfer
    state.practice_state = PracticeProgressionState::Fluent;
    state.total_attempts = 5;
    for _ in 0..5 {
        state.recent_attempts.push(procedural::skills::signals::RecentAttemptRecord {
            is_correct: true,
            score: 1.0,
            latency_ms: 25_000,
            target_latency_ms: 30_000,
            variant: Some("standard".into()),
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

    let eval_fluent = TransferEngine::evaluate_eligibility(&state, TransferLevel::NearTransfer, false);
    assert!(eval_fluent.is_eligible);
    assert_eq!(eval_fluent.max_eligible_level, Some(TransferLevel::NearTransfer));

    // 3. Variation stage with 2 variants practiced -> eligible for StructuralTransfer
    state.practice_state = PracticeProgressionState::Variation;
    state.consecutive_successes = 3;
    state.record_variant_exposure("var_reverse", true, 20_000, None, 100);
    state.record_variant_exposure("var_boundary", true, 22_000, None, 101);

    let eval_var = TransferEngine::evaluate_eligibility(&state, TransferLevel::StructuralTransfer, false);
    assert!(eval_var.is_eligible);
    assert_eq!(eval_var.max_eligible_level, Some(TransferLevel::StructuralTransfer));

    // 4. Mastered stage with supporting schemas stable -> eligible for MultiConcept & FarTransfer
    state.practice_state = PracticeProgressionState::Mastered;
    state.consecutive_successes = 6;
    state.record_variant_exposure("var_context", true, 19_000, None, 102);

    let eval_mastered = TransferEngine::evaluate_eligibility(&state, TransferLevel::MultiConceptTransfer, true);
    assert!(eval_mastered.is_eligible);

    let eval_far = TransferEngine::evaluate_eligibility(&state, TransferLevel::FarTransfer, true);
    assert!(eval_far.is_eligible);
}

#[test]
fn test_transfer_failure_remediation_routing() {
    let skill_id = SkillId::new("physics.kinematics.1d");
    let schema_id = SchemaId::new("physics_kinematics");

    // 1. Concept failure in transfer -> ConceptCheck
    let action_concept = TransferEngine::classify_transfer_failure(
        &skill_id,
        &schema_id,
        Domain::Physics,
        TransferLevel::StructuralTransfer,
        Some(ErrorCategory::Concept),
        None,
    );
    assert_eq!(action_concept.kind, RemediationActionKind::ConceptCheck);

    // 2. Strategy failure in transfer -> StrategyDrill
    let action_strat = TransferEngine::classify_transfer_failure(
        &skill_id,
        &schema_id,
        Domain::Physics,
        TransferLevel::StructuralTransfer,
        Some(ErrorCategory::Strategy),
        None,
    );
    assert_eq!(action_strat.kind, RemediationActionKind::StrategyDrill);

    // 3. Calculation slip in transfer -> WorkedExample
    let action_calc = TransferEngine::classify_transfer_failure(
        &skill_id,
        &schema_id,
        Domain::Physics,
        TransferLevel::StructuralTransfer,
        Some(ErrorCategory::Calculation),
        None,
    );
    assert_eq!(action_calc.kind, RemediationActionKind::WorkedExample);
}
