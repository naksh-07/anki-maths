// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use procedural::core::{Domain, SchemaId, SkillId};
use procedural::diagnostics::ErrorCategory;
use procedural::practice::{PracticeObjective, PracticeRequest, PracticeScope};
use procedural::problems::catalog::*;
use procedural::remediation::{
    RemediationAction, RemediationActionKind, RemediationIntervention, RemediationUrgency,
};
use procedural::scheduling::SessionReadiness;
use procedural::service::ProceduralService;

#[test]
fn test_scope_isolation_in_focused_mode_preserves_user_intent() {
    let service = ProceduralService::open_in_memory().unwrap();

    // User explicitly requests focused practice on multi-concept math (which requires percentage and ratio)
    let multi_skill = SkillId::from(SKILL_COMBINED_MULTI_CONCEPT);
    let req = PracticeRequest::new(
        PracticeScope::SingleSkill(multi_skill.clone()),
        PracticeObjective::Practice,
    );

    // Learner has NOT mastered percentage or ratio yet (empty state)
    let session = service.prepare_unified_practice_session(&req, None, None, Some(42)).unwrap();

    // Verification: Must STILL deliver the multi_concept schema, with advisory warning attached, NOT escaping scope
    assert_eq!(session.schema.skill_id, multi_skill);
    assert_eq!(session.schema.id.as_str(), SCHEMA_COMBINED_MULTI_CONCEPT);

    // Readiness status reflects prerequisite needs advisably without hard blocking
    match session.readiness {
        SessionReadiness::PrerequisitesNeeded { ref missing_skills } => {
            assert!(missing_skills.contains(&SkillId::from(SKILL_PERCENTAGE_SUCCESSIVE)));
            assert!(missing_skills.contains(&SkillId::from(SKILL_RATIO)));
        }
        _ => panic!("Expected PrerequisitesNeeded advisory in session readiness"),
    }
}

#[test]
fn test_priority_tier_precedence_critical_remediation_overrides_rotation() {
    let service = ProceduralService::open_in_memory().unwrap();

    // Enqueue a critical remediation action on Chemistry Equilibrium
    let chem_skill = SkillId::from(SKILL_CHEMISTRY_EQUILIBRIUM);
    let chem_schema = SchemaId::from(SCHEMA_CHEMISTRY_EQUILIBRIUM);

    let action = RemediationAction::new(
        "rem-chem-urgent",
        RemediationActionKind::ConceptCheck,
        chem_skill.clone(),
        chem_schema.clone(),
        Domain::Chemistry,
        ErrorCategory::Concept,
        "att-123",
        "Persistent Le Chatelier principle misconception",
    )
    .with_urgency(RemediationUrgency::Critical);

    service.enqueue_remediation_action(action).unwrap();

    // Learner requests general mixed practice
    let req = PracticeRequest::new(PracticeScope::AllDomains, PracticeObjective::Practice);
    let session = service.prepare_unified_practice_session(&req, None, None, Some(99)).unwrap();

    // Critical remediation takes Tier 3 priority and selects Chemistry Equilibrium
    assert_eq!(session.schema.skill_id, chem_skill);
    assert_eq!(session.schema.id, chem_schema);
}

#[test]
fn test_executable_prerequisite_review_generates_foundational_problem() {
    let service = ProceduralService::open_in_memory().unwrap();

    // Enqueue PrerequisiteReview for Physics Work & Energy (prerequisite is Kinematics)
    let we_skill = SkillId::from(SKILL_PHYSICS_WORK_ENERGY);
    let we_schema = SchemaId::from(SCHEMA_PHYSICS_WORK_ENERGY);

    let action = RemediationAction::new(
        "rem-we-prereq",
        RemediationActionKind::PrerequisiteReview,
        we_skill.clone(),
        we_schema.clone(),
        Domain::Physics,
        ErrorCategory::Concept,
        "att-456",
        "Foundational 1D Kinematics review needed",
    );

    let intervention = service.select_remediation_intervention(&action, 777).unwrap();

    match intervention {
        RemediationIntervention::PrerequisiteReview(review_obj) => {
            assert_eq!(review_obj.target_skill_id, we_skill);
            assert!(review_obj.prerequisite_skill_ids.contains(&SkillId::from(SKILL_PHYSICS_KINEMATICS)));
            assert_eq!(
                review_obj.primary_missing_prerequisite,
                Some(SkillId::from(SKILL_PHYSICS_KINEMATICS))
            );
            assert_eq!(
                review_obj.executable_schema_id,
                Some(SchemaId::from(SCHEMA_PHYSICS_KINEMATICS))
            );
            // Must have generated an authentic foundational problem instance for Kinematics
            assert!(review_obj.executable_problem.is_some());
            let problem = review_obj.executable_problem.unwrap();
            assert!(problem.rendered_prompt.len() > 10);
        }
        _ => panic!("Expected RemediationIntervention::PrerequisiteReview"),
    }
}

#[test]
fn test_anti_priming_penalty_in_open_mode_vs_disabled_in_focused_mode() {
    let service = ProceduralService::open_in_memory().unwrap();

    let last_schema = SchemaId::from(SCHEMA_SUCCESSIVE_PERCENTAGE);

    // In open MixedMaths mode with last_schema, the engine interleaves to a different math schema
    let req_open = PracticeRequest::new(
        PracticeScope::SingleDomain(Domain::Mathematics),
        PracticeObjective::Practice,
    );
    let session_open = service
        .prepare_unified_practice_session(&req_open, None, Some(&last_schema), Some(123))
        .unwrap();
    // Anti-priming prevents immediate repetition of percentage
    assert_ne!(session_open.schema.id, last_schema);

    // In focused mode on SuccessivePercentage, the engine MUST deliver percentage despite being last_schema
    let req_focused = PracticeRequest::new(
        PracticeScope::SingleSchema(last_schema.clone()),
        PracticeObjective::Practice,
    );
    let session_focused = service
        .prepare_unified_practice_session(&req_focused, None, Some(&last_schema), Some(123))
        .unwrap();
    assert_eq!(session_focused.schema.id, last_schema);
}
