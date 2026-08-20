// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! # Executable Remediation Engine Subsystem
//!
//! Transforms diagnostic error signals into explicit, structured, and executable
//! learning interventions across Mathematics, Physics, Chemistry, and Reasoning,
//! feeding verified evidence into R1 Mastery Signals.

pub mod actions;
pub mod audit;
pub mod objects;
pub mod policy;
pub mod queue;
pub mod selector;

pub use actions::{RemediationAction, RemediationActionKind, RemediationUrgency};
pub use audit::{RemediationAuditLog, RemediationAuditRecord, RemediationOutcomeStatus};
pub use objects::{
    CircuitBreakerObject, ConceptCheckEvaluation, ConceptCheckObject, ConceptCheckOption, DeclarativeRecallBridge,
    PrerequisiteReviewObject, RemediationIntervention, RepresentationDrillEvaluation,
    RepresentationDrillObject, RepresentationOption, StrategyDrillEvaluation, StrategyDrillObject,
    StrategyOption, WorkedExampleObject,
};
pub use policy::{RemediationContext, RemediationPolicy};
pub use queue::RemediationQueue;
pub use selector::RemediationSelector;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{AttemptId, Domain, SchemaId, SkillId};
    use crate::diagnostics::ErrorCategory;
    use crate::problems::steps::StepErrorType;
    use crate::skills::signals::{IndependenceLevel, PracticeProgressionState};

    #[test]
    fn test_remediation_policy_concept_error_to_concept_check() {
        let skill = SkillId::new("math.percentage.successive");
        let schema = SchemaId::new("successive_percentage");
        let att_id = AttemptId::new("att-101");

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
            recurrence_count: 1,
            is_transfer_attempt: false,
        };

        let action = RemediationPolicy::evaluate(&ctx);
        assert_eq!(action.kind, RemediationActionKind::ConceptCheck);
        assert_eq!(action.urgency, RemediationUrgency::Normal);
    }

    #[test]
    fn test_remediation_policy_strategy_error_to_strategy_drill() {
        let skill = SkillId::new("reasoning.seating.constraint");
        let schema = SchemaId::new("seating_linear");
        let att_id = AttemptId::new("att-102");

        let ctx = RemediationContext {
            skill_id: &skill,
            schema_id: &schema,
            domain: Domain::Reasoning,
            primary_error: ErrorCategory::Strategy,
            step_error: Some(StepErrorType::StrategySelectionError),
            decision_point_correct: None,
            independence: IndependenceLevel::NonIndependent,
            progression_state: PracticeProgressionState::Fluent,
            recent_attempts: &[],
            source_attempt_id: &att_id,
            recurrence_count: 1,
            is_transfer_attempt: false,
        };

        let action = RemediationPolicy::evaluate(&ctx);
        assert_eq!(action.kind, RemediationActionKind::StrategyDrill);
    }

    #[test]
    fn test_remediation_escalation_path() {
        let skill = SkillId::new("physics.kinematics.1d");
        let schema = SchemaId::new("kinematics_1d");
        let att_id = AttemptId::new("att-103");

        // Attempt 1: Concept check
        let ctx1 = RemediationContext {
            skill_id: &skill,
            schema_id: &schema,
            domain: Domain::Physics,
            primary_error: ErrorCategory::Concept,
            step_error: Some(StepErrorType::ModelSelectionError),
            decision_point_correct: None,
            independence: IndependenceLevel::NonIndependent,
            progression_state: PracticeProgressionState::Learning,
            recent_attempts: &[],
            source_attempt_id: &att_id,
            recurrence_count: 1,
            is_transfer_attempt: false,
        };
        let act1 = RemediationPolicy::evaluate(&ctx1);
        assert_eq!(act1.kind, RemediationActionKind::ConceptCheck);

        // Attempt 2: Strategy drill with Critical urgency
        let ctx2 = RemediationContext {
            recurrence_count: 2,
            ..ctx1.clone()
        };
        let act2 = RemediationPolicy::evaluate(&ctx2);
        assert_eq!(act2.kind, RemediationActionKind::StrategyDrill);
        assert_eq!(act2.urgency, RemediationUrgency::Critical);

        // Attempt 3: Worked example
        let ctx3 = RemediationContext {
            recurrence_count: 3,
            ..ctx1.clone()
        };
        let act3 = RemediationPolicy::evaluate(&ctx3);
        assert_eq!(act3.kind, RemediationActionKind::WorkedExample);
        assert_eq!(act3.urgency, RemediationUrgency::Critical);

        // Attempt 4+: Advisory prerequisite review
        let ctx4 = RemediationContext {
            recurrence_count: 4,
            ..ctx1.clone()
        };
        let act4 = RemediationPolicy::evaluate(&ctx4);
        assert_eq!(act4.kind, RemediationActionKind::PrerequisiteReview);
    }

    #[test]
    fn test_concept_check_evaluation_evidence() {
        let cc = ConceptCheckObject::new(
            "cc_1",
            SkillId::new("math.percentage.successive"),
            SchemaId::new("successive_percentage"),
            Domain::Mathematics,
            "Successive percentage compounding question",
            vec![
                ConceptCheckOption::new("opt_a", "Compounding multipliers", true, "compound", "Correct!"),
                ConceptCheckOption::new("opt_b", "Additive sum", false, "additive", "Wrong!"),
            ],
            "opt_a",
            "Multipliers compound.",
        );

        // Correct choice
        let eval_pass = cc.evaluate_choice("opt_a", 5000);
        assert!(eval_pass.is_correct);
        assert!(eval_pass.evidence.final_correctness);
        assert_eq!(eval_pass.evidence.decision_quality, Some(1.0));
        assert_eq!(eval_pass.evidence.independence, IndependenceLevel::Independent);
        assert!(eval_pass.evidence.diagnostic_errors.is_empty());

        // Incorrect choice
        let eval_fail = cc.evaluate_choice("opt_b", 7000);
        assert!(!eval_fail.is_correct);
        assert!(!eval_fail.evidence.final_correctness);
        assert_eq!(eval_fail.evidence.decision_quality, Some(0.0));
        assert_eq!(eval_fail.evidence.diagnostic_errors, vec![ErrorCategory::Concept]);
    }

    #[test]
    fn test_worked_example_no_false_mastery() {
        let we = WorkedExampleObject::new(
            "we_1",
            SkillId::new("chem.stoichiometry.moles"),
            SchemaId::new("stoichiometry_moles"),
            Domain::Chemistry,
            "Title",
            "Context",
            vec!["Step 1".to_string(), "Step 2".to_string()],
            "Decision",
            "Rationale",
            vec!["Common mistake".to_string()],
        );

        let evidence = we.generate_viewing_evidence(12000);
        // Viewing worked example provides exposure, but does NOT award correctness or mastery!
        assert!(!evidence.final_correctness);
        assert_eq!(evidence.decision_quality, None);
        assert_eq!(evidence.independence, IndependenceLevel::NonIndependent);
        assert_eq!(evidence.variant_exposure, Some("worked_example_view".to_string()));
    }

    #[test]
    fn test_remediation_queue_user_intent_gating() {
        let mut queue = RemediationQueue::new();

        let act_math = RemediationAction::new(
            "rem-math",
            RemediationActionKind::ConceptCheck,
            SkillId::new("math.percentage.successive"),
            SchemaId::new("successive_percentage"),
            Domain::Mathematics,
            ErrorCategory::Concept,
            AttemptId::new("att-1"),
            "Math concept issue",
        );

        let act_phys = RemediationAction::new(
            "rem-phys",
            RemediationActionKind::StrategyDrill,
            SkillId::new("physics.kinematics.1d"),
            SchemaId::new("kinematics_1d"),
            Domain::Physics,
            ErrorCategory::Strategy,
            AttemptId::new("att-2"),
            "Physics strategy issue",
        );

        queue.enqueue(act_math);
        queue.enqueue(act_phys);

        // User explicitly requests focused practice on Physics
        let mode_phys = crate::scheduling::PracticeMode::FocusedSkill {
            skill_id: SkillId::new("physics.kinematics.1d"),
        };

        // Queue should select the physics remediation and NOT hijack with the math one
        let selected = queue.select_next_remediation(&mode_phys).unwrap();
        assert_eq!(selected.skill_id.as_str(), "physics.kinematics.1d");
    }
}
