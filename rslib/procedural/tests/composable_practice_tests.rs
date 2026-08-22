// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use procedural::core::{Domain, ExamProfileId, SchemaId, SkillId};
use procedural::exam::ExamPracticeMode;
use procedural::practice::{
    DifficultyConstraint, PracticeObjective, PracticeRequest, PracticeScope, RemediationPrecedence,
    SessionBudget, TimeConstraint,
};
use procedural::problems::catalog::*;
use procedural::scheduling::PracticeMode;
use procedural::service::ProceduralService;

#[test]
fn test_practice_scope_boundary_matching() {
    let math_schema = SchemaId::from(SCHEMA_SUCCESSIVE_PERCENTAGE);
    let math_skill = SkillId::from(SKILL_PERCENTAGE_SUCCESSIVE);
    let physics_schema = SchemaId::from(SCHEMA_PHYSICS_KINEMATICS);
    let physics_skill = SkillId::from(SKILL_PHYSICS_KINEMATICS);

    // 1. AllDomains
    let scope_all = PracticeScope::AllDomains;
    assert!(!scope_all.is_focused());
    assert!(scope_all.matches_skill(&math_skill, &Domain::Mathematics));
    assert!(scope_all.matches_skill(&physics_skill, &Domain::Physics));
    assert!(scope_all.matches_schema(&math_schema, &math_skill, &Domain::Mathematics));

    // 2. SingleDomain
    let scope_math = PracticeScope::SingleDomain(Domain::Mathematics);
    assert!(!scope_math.is_focused());
    assert!(scope_math.matches_skill(&math_skill, &Domain::Mathematics));
    assert!(!scope_math.matches_skill(&physics_skill, &Domain::Physics));

    // 3. SingleSkill
    let scope_skill = PracticeScope::SingleSkill(math_skill.clone());
    assert!(scope_skill.is_focused());
    assert!(scope_skill.matches_skill(&math_skill, &Domain::Mathematics));
    assert!(!scope_skill.matches_skill(&physics_skill, &Domain::Physics));

    // 4. SingleSchema
    let scope_schema = PracticeScope::SingleSchema(math_schema.clone());
    assert!(scope_schema.is_focused());
    assert!(scope_schema.matches_schema(&math_schema, &math_skill, &Domain::Mathematics));
    assert!(!scope_schema.matches_schema(&physics_schema, &physics_skill, &Domain::Physics));

    // 5. MultipleSkills
    let scope_multi = PracticeScope::MultipleSkills(vec![math_skill.clone(), physics_skill.clone()]);
    assert!(!scope_multi.is_focused());
    assert!(scope_multi.matches_skill(&math_skill, &Domain::Mathematics));
    assert!(scope_multi.matches_skill(&physics_skill, &Domain::Physics));
}

#[test]
fn test_difficulty_constraint_clamping() {
    // Exact difficulty 4
    let exact = DifficultyConstraint::Exact { level: 4 };
    assert_eq!(exact.clamp_level(1), 4);
    assert_eq!(exact.clamp_level(5), 4);

    // Range [2, 4]
    let range = DifficultyConstraint::Range { min: 2, max: 4 };
    assert_eq!(range.clamp_level(1), 2);
    assert_eq!(range.clamp_level(3), 3);
    assert_eq!(range.clamp_level(5), 4);

    // Min floor 3
    let min = DifficultyConstraint::Min { min: 3 };
    assert_eq!(min.clamp_level(1), 3);
    assert_eq!(min.clamp_level(4), 4);

    // Max ceiling 2
    let max = DifficultyConstraint::Max { max: 2 };
    assert_eq!(max.clamp_level(1), 1);
    assert_eq!(max.clamp_level(5), 2);
}

#[test]
fn test_legacy_mode_to_practice_request_mapping() {
    // 1. MixedMaths
    let req_math = PracticeRequest::from_legacy_mode(&PracticeMode::MixedMaths);
    assert_eq!(req_math.scope, PracticeScope::SingleDomain(Domain::Mathematics));
    assert_eq!(req_math.objective, PracticeObjective::Practice);

    // 2. FocusedSkill
    let skill_id = SkillId::from(SKILL_PHYSICS_KINEMATICS);
    let req_focused = PracticeRequest::from_legacy_mode(&PracticeMode::FocusedSkill {
        skill_id: skill_id.clone(),
    });
    assert_eq!(req_focused.scope, PracticeScope::SingleSkill(skill_id));
    assert!(req_focused.scope.is_focused());

    // 3. SpeedPractice
    let req_speed = PracticeRequest::from_legacy_mode(&PracticeMode::SpeedPractice);
    assert_eq!(req_speed.objective, PracticeObjective::Speed);
    assert_eq!(req_speed.difficulty_constraint, Some(DifficultyConstraint::Exact { level: 1 }));
    assert_eq!(req_speed.time_constraint.unwrap().target_latency_ms, Some(20_000));

    // 4. TransferPractice
    let req_transfer = PracticeRequest::from_legacy_mode(&PracticeMode::TransferPractice);
    assert_eq!(req_transfer.objective, PracticeObjective::Transfer);
    assert_eq!(req_transfer.difficulty_constraint, Some(DifficultyConstraint::Exact { level: 5 }));

    // 5. ExamPracticeMode::SpeedTraining
    let exam_profile = ExamProfileId::new("ssc_cgl");
    let req_exam_speed = PracticeRequest::from_exam_mode(&exam_profile, &ExamPracticeMode::SpeedTraining);
    assert_eq!(req_exam_speed.objective, PracticeObjective::Speed);
    assert_eq!(req_exam_speed.exam_profile, Some(exam_profile));
    assert_eq!(req_exam_speed.time_constraint.unwrap().target_latency_ms, Some(20_000));
}

#[test]
fn test_service_prepare_unified_practice_session_end_to_end() {
    let service = ProceduralService::open_in_memory().unwrap();

    // 1. Practice request for SingleDomain(Physics)
    let req_physics = PracticeRequest::new(
        PracticeScope::SingleDomain(Domain::Physics),
        PracticeObjective::Practice,
    );
    let session_physics = service.prepare_unified_practice_session(&req_physics, None, None, Some(42)).unwrap();
    assert_eq!(session_physics.schema.id.as_str(), SCHEMA_PHYSICS_KINEMATICS);
    assert!(session_physics.instance.rendered_prompt.len() > 10);

    // 2. Practice request with Exact difficulty constraint (Level 4)
    let req_diff = PracticeRequest::new(
        PracticeScope::SingleDomain(Domain::Mathematics),
        PracticeObjective::Practice,
    )
    .with_exact_difficulty(4);
    let session_diff = service.prepare_unified_practice_session(&req_diff, None, None, Some(101)).unwrap();
    assert_eq!(session_diff.difficulty_level, Some(4));

    // 3. Practice request with Target Latency Constraint (12,000 ms)
    let req_time = PracticeRequest::new(
        PracticeScope::SingleDomain(Domain::Reasoning),
        PracticeObjective::Speed,
    )
    .with_target_latency_ms(12_000);
    let session_time = service.prepare_unified_practice_session(&req_time, None, None, Some(202)).unwrap();
    assert_eq!(session_time.target_latency_ms, Some(12_000));
}