// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use procedural::core::{AttemptId, Domain, SchemaId, SkillId};
use procedural::diagnostics::ErrorCategory;
use procedural::problems::catalog::SCHEMA_SUCCESSIVE_PERCENTAGE;
use procedural::problems::steps::StepErrorType;
use procedural::remediation::{
    RemediationAction, RemediationActionKind, RemediationContext, RemediationPolicy,
    RemediationQueue, RemediationUrgency,
};
use procedural::scheduling::PracticeMode;
use procedural::service::ProceduralService;
use procedural::skills::signals::{IndependenceLevel, MasteryEvidence, PracticeProgressionState};
use procedural::skills::SkillState;

#[test]
fn test_adversarial_persistent_concept_failure_escalates_without_infinite_loops() {
    let skill = SkillId::new("math.percentage.successive");
    let schema = SchemaId::new(SCHEMA_SUCCESSIVE_PERCENTAGE);
    let att_id = AttemptId::new("att-adv-1");

    let mut queue = RemediationQueue::new();

    // Loop through 5 consecutive failures on the same conceptual error
    let kinds: Vec<RemediationActionKind> = (1..=5)
        .map(|rec| {
            let ctx = RemediationContext {
                skill_id: &skill,
                schema_id: &schema,
                domain: Domain::Mathematics,
                primary_error: ErrorCategory::Concept,
                step_error: Some(StepErrorType::FormulaSelectionError),
                decision_point_correct: None,
                independence: IndependenceLevel::NonIndependent,
                progression_state: PracticeProgressionState::Learning,
                recent_attempts: &[],
                source_attempt_id: &att_id,
                recurrence_count: rec,
                is_transfer_attempt: false,
            };

            let action = RemediationPolicy::evaluate(&ctx);
            queue.enqueue(action.clone());
            action.kind
        })
        .collect();

    // Verify escalation path:
    // 1st failure -> ConceptCheck
    assert_eq!(kinds[0], RemediationActionKind::ConceptCheck);
    // 2nd failure -> StrategyDrill (escalated from basic concept check)
    assert_eq!(kinds[1], RemediationActionKind::StrategyDrill);
    // 3rd failure -> WorkedExample
    assert_eq!(kinds[2], RemediationActionKind::WorkedExample);
    // 4th failure -> PrerequisiteReview
    assert_eq!(kinds[3], RemediationActionKind::PrerequisiteReview);
    // 5th failure -> CircuitBreaker (halts wheel-spinning after repeated prerequisite failures)
    assert_eq!(kinds[4], RemediationActionKind::CircuitBreaker);

    // Verify queue only holds 1 active action (deduplicated to latest escalation)
    assert_eq!(queue.len(), 1);
    let top = queue.select_next_remediation(&PracticeMode::MixedMaths).unwrap();
    assert_eq!(top.kind, RemediationActionKind::CircuitBreaker);
    assert_eq!(top.urgency, RemediationUrgency::Advisory);
}

#[test]
fn test_adversarial_hint_dependent_learner_avoids_false_mastery() {
    let mut state = SkillState::new("math.percentage.successive");
    assert_eq!(state.practice_state, PracticeProgressionState::New);

    // Learner gets correct answers but relies on heavy hints (SignificantSupport)
    for i in 1..=5 {
        let ev = MasteryEvidence {
            final_correctness: true,
            decision_quality: Some(1.0),
            step_quality: Some(1.0),
            independence: IndependenceLevel::SignificantSupport,
            max_hint_level: Some(3),
            hint_dependence: 4,
            retry_dependence: 2,
            variant_exposure: Some("standard".to_string()),
            variant_category: procedural::VariantCategory::Parameter,
            solution_graph_fingerprint: None,
            cognitive_decision_correct: Some(true),
            time_since_last_ms: None,
            transfer_evidence: false,
            domain_competence_verified: Some(true),
            latency_evidence: 45000,
            diagnostic_errors: vec![],
            domain_evidence: None,
        };
        state.record_attempt_outcome(&ev, 0.7, 35000, 1000 * i);
    }

    // Despite 5 consecutive successes, state must NOT advance to Fluent because independence was not demonstrated
    assert_ne!(state.practice_state, PracticeProgressionState::Fluent);
    assert_eq!(state.practice_state, PracticeProgressionState::Learning);
}

#[test]
fn test_adversarial_correct_final_answer_but_wrong_strategy_triggers_remediation() {
    let skill = SkillId::new("reasoning.seating.constraint");
    let schema = SchemaId::new("seating_linear");
    let att_id = AttemptId::new("att-adv-2");

    // Student got lucky with final answer, but made an invalid strategy decision
    let ctx = RemediationContext {
        skill_id: &skill,
        schema_id: &schema,
        domain: Domain::Reasoning,
        primary_error: ErrorCategory::Strategy,
        step_error: Some(StepErrorType::StrategySelectionError),
        decision_point_correct: Some(false), // Strategy was WRONG
        independence: IndependenceLevel::LightSupport,
        progression_state: PracticeProgressionState::Fluent,
        recent_attempts: &[],
        source_attempt_id: &att_id,
        recurrence_count: 1,
        is_transfer_attempt: false,
    };

    let action = RemediationPolicy::evaluate(&ctx);
    // Remediation must still trigger a StrategyDrill to address the faulty strategic reasoning
    assert_eq!(action.kind, RemediationActionKind::StrategyDrill);
}

#[test]
fn test_adversarial_correct_strategy_wrong_execution_avoids_conceptual_interruption() {
    let skill = SkillId::new("math.percentage.successive");
    let schema = SchemaId::new(SCHEMA_SUCCESSIVE_PERCENTAGE);
    let att_id = AttemptId::new("att-adv-3");

    // Student chose optimal strategy, but made a pure arithmetic slip
    let ctx = RemediationContext {
        skill_id: &skill,
        schema_id: &schema,
        domain: Domain::Mathematics,
        primary_error: ErrorCategory::Calculation,
        step_error: Some(StepErrorType::ArithmeticError),
        decision_point_correct: Some(true), // Strategy was sound!
        independence: IndependenceLevel::Independent,
        progression_state: PracticeProgressionState::Fluent,
        recent_attempts: &[],
        source_attempt_id: &att_id,
        recurrence_count: 1,
        is_transfer_attempt: false,
    };

    let action = RemediationPolicy::evaluate(&ctx);
    // Should NOT trigger a ConceptCheck or StrategyDrill, but rather a targeted simpler procedural variant
    assert_eq!(action.kind, RemediationActionKind::ProceduralVariant);
    assert_eq!(action.preferred_variant, Some("simpler_numbers".to_string()));
}

#[test]
fn test_adversarial_user_intent_respected_in_focused_practice() {
    let mut queue = RemediationQueue::new();

    // Enqueue a Normal-urgency remediation for Chemistry
    let act_chem = RemediationAction::new(
        "rem-chem",
        RemediationActionKind::ProceduralVariant,
        SkillId::new("chem.stoichiometry.moles"),
        SchemaId::new("stoichiometry_moles"),
        Domain::Chemistry,
        ErrorCategory::Calculation,
        AttemptId::new("att-c1"),
        "Calculation error in chem",
    );
    queue.enqueue(act_chem);

    // User explicitly selects focused practice on Maths (Linear Equations)
    let mode_math = PracticeMode::FocusedSkill {
        skill_id: SkillId::new("math.algebra.linear_equations"),
    };

    // Queue must NOT pop the unrelated Chemistry remediation
    let selected = queue.select_next_remediation(&mode_math);
    assert!(selected.is_none(), "Focused practice should not be hijacked by non-critical cross-skill remediation");
}