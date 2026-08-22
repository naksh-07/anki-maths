// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use procedural::core::{AttemptId, Domain, SchemaId, SkillId};
use procedural::diagnostics::ErrorCategory;
use procedural::remediation::{RemediationActionKind, RemediationContext, RemediationPolicy};
use procedural::scheduling::difficulty::AdaptiveDifficultyEngine;
use procedural::skills::domain_evidence::{
    ChemistryEvidence, DomainEvidencePayload, MathEvidence, PhysicsEvidence, ReasoningEvidence,
    VersionedDomainEvidence,
};
use procedural::skills::signals::{
    IndependenceLevel, MasteryEvidence, PracticeProgressionState, RecentAttemptRecord,
};
use procedural::skills::SkillState;

#[test]
fn test_math_calculation_slip_does_not_demote_concept() {
    let mut state = SkillState::new("math.algebra");
    state.practice_state = PracticeProgressionState::Fluent;
    state.custom_state = serde_json::json!({ "current_difficulty_level": 4 });

    let math_ev = MathEvidence {
        execution: Some(false),
        method_selection: Some(true),
        pattern_recognition: Some(true),
        ..Default::default()
    };
    let domain_ev = VersionedDomainEvidence::new_math(math_ev);

    let ev = MasteryEvidence {
        final_correctness: false,
        latency_evidence: 45_000,
        diagnostic_errors: vec![ErrorCategory::Concept], // Evaluator guessed Concept, but domain evidence overrides it
        domain_evidence: Some(domain_ev.clone()),
        ..Default::default()
    };
    state.record_attempt_outcome(&ev, 0.0, 65_000, 1000);

    let dec = AdaptiveDifficultyEngine::evaluate_difficulty(Some(&state), None, None);
    // Should NOT be demoted_on_concept_breakdown, it's just a general failure. 
    // Since recent_accuracy drops (1 attempt, 0% correct), but window is 5. 
    // It should stay at 4 or demote via other rules, but not the FAST concept demotion.
    assert!(!dec.reason.contains("demoted_on_concept_breakdown"));

    // Check remediation policy
    let ctx = RemediationContext {
        skill_id: &SkillId::from("math.algebra"),
        schema_id: &SchemaId::from("schema.algebra.1"),
        domain: Domain::Mathematics,
        primary_error: ErrorCategory::Concept,
        step_error: None,
        decision_point_correct: None,
        independence: IndependenceLevel::Independent,
        progression_state: PracticeProgressionState::Fluent,
        recent_attempts: &state.recent_attempts,
        source_attempt_id: &AttemptId::new("test"),
        recurrence_count: 1,
        is_transfer_attempt: false,
    };
    let action = RemediationPolicy::evaluate(&ctx);
    assert_eq!(action.kind, RemediationActionKind::ProceduralVariant);
    assert_eq!(action.preferred_variant, Some("simpler_numbers".to_string()));
}

#[test]
fn test_physics_unit_error_remediation() {
    let mut state = SkillState::new("physics.kinematics");

    let phys_ev = PhysicsEvidence {
        unit_validity: Some(false),
        ..Default::default()
    };
    let domain_ev = VersionedDomainEvidence::new_physics(phys_ev);

    let mut attempt = RecentAttemptRecord {
        is_correct: false,
        score: 0.0,
        latency_ms: 30000,
        target_latency_ms: 30000,
        variant: None,
        variant_category: None,
        error_category: Some(ErrorCategory::Calculation),
        max_hint_level: None,
        hint_count: None,
        independence: None,
        solution_graph_fingerprint: None,
        cognitive_decision_correct: None,
        domain_evidence: Some(domain_ev.clone()),
        timestamp: 1000,
    };

    let attempts = vec![attempt.clone()];
    let ctx = RemediationContext {
        skill_id: &SkillId::from("physics.kinematics"),
        schema_id: &SchemaId::from("schema.physics.1"),
        domain: Domain::Physics,
        primary_error: ErrorCategory::Calculation,
        step_error: None,
        decision_point_correct: None,
        independence: IndependenceLevel::Independent,
        progression_state: PracticeProgressionState::Learning,
        recent_attempts: &attempts,
        source_attempt_id: &AttemptId::new("test-physics-1"),
        recurrence_count: 1,
        is_transfer_attempt: false,
    };

    let action1 = RemediationPolicy::evaluate(&ctx);
    // First time -> Procedural Variant with unit_conversion
    assert_eq!(action1.kind, RemediationActionKind::ProceduralVariant);
    assert_eq!(action1.preferred_variant, Some("unit_conversion".to_string()));

    // Second time -> Escalates to Declarative Recall
    attempt.timestamp = 2000;
    let attempts2 = vec![attempt.clone(), attempt.clone()];
    let ctx2 = RemediationContext {
        recent_attempts: &attempts2,
        recurrence_count: 2,
        ..ctx
    };
    let action2 = RemediationPolicy::evaluate(&ctx2);
    assert_eq!(action2.kind, RemediationActionKind::DeclarativeRecall);
    assert_eq!(action2.preferred_variant, Some("unit_conversion".to_string()));
}

#[test]
fn test_chemistry_stoichiometry_error() {
    let mut state = SkillState::new("chemistry.moles");

    let chem_ev = ChemistryEvidence::Physical {
        model_setup: Some(true),
        equation_selection: Some(true),
        intermediate_quantity: Some(true),
        calculation: Some(true),
        conservation: Some(false), // Setup/conservation error
        verification: Some(false),
        transfer: Some(false),
    };
    let domain_ev = VersionedDomainEvidence::new_chemistry(chem_ev);

    let ev = MasteryEvidence {
        final_correctness: false,
        latency_evidence: 45_000,
        diagnostic_errors: vec![ErrorCategory::Calculation],
        domain_evidence: Some(domain_ev),
        ..Default::default()
    };
    state.record_attempt_outcome(&ev, 0.0, 65_000, 1000);

    let ctx = RemediationContext {
        skill_id: &SkillId::from("chemistry.moles"),
        schema_id: &SchemaId::from("schema.chem.1"),
        domain: Domain::Chemistry,
        primary_error: ErrorCategory::Calculation,
        step_error: None,
        decision_point_correct: None,
        independence: IndependenceLevel::Independent,
        progression_state: PracticeProgressionState::Learning,
        recent_attempts: &state.recent_attempts,
        source_attempt_id: &AttemptId::new("test"),
        recurrence_count: 1,
        is_transfer_attempt: false,
    };
    let action = RemediationPolicy::evaluate(&ctx);
    // Stoichiometry/conservation setup error maps to StrategyDrill, overriding Calculation
    assert_eq!(action.kind, RemediationActionKind::StrategyDrill);
}

#[test]
fn test_reasoning_representation_issue() {
    let mut state = SkillState::new("reasoning.logic");

    let reas_ev = ReasoningEvidence {
        representation: Some(false),
        ..Default::default()
    };
    let domain_ev = VersionedDomainEvidence::new_reasoning(reas_ev);

    let ev = MasteryEvidence {
        final_correctness: false,
        latency_evidence: 45_000,
        diagnostic_errors: vec![ErrorCategory::Unknown],
        domain_evidence: Some(domain_ev),
        ..Default::default()
    };
    state.record_attempt_outcome(&ev, 0.0, 65_000, 1000);

    let ctx = RemediationContext {
        skill_id: &SkillId::from("reasoning.logic"),
        schema_id: &SchemaId::from("schema.reasoning.1"),
        domain: Domain::Reasoning,
        primary_error: ErrorCategory::Unknown,
        step_error: None,
        decision_point_correct: None,
        independence: IndependenceLevel::Independent,
        progression_state: PracticeProgressionState::Learning,
        recent_attempts: &state.recent_attempts,
        source_attempt_id: &AttemptId::new("test"),
        recurrence_count: 1,
        is_transfer_attempt: false,
    };
    let action = RemediationPolicy::evaluate(&ctx);
    // Representation failure maps to RepresentationDrill
    assert_eq!(action.kind, RemediationActionKind::RepresentationDrill);
}
