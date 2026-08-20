// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use procedural::scheduling::difficulty::AdaptiveDifficultyEngine;
use procedural::skills::signals::{IndependenceLevel, MasteryEvidence};
use procedural::{
    ErrorCategory, MultiSchemaSelector, PracticeMode, ProceduralService,
    Rating, SkillId, SkillState, FAMILY_AVERAGE, FAMILY_DIVISIBILITY, FAMILY_LINEAR_EQUATIONS,
    FAMILY_PROFIT_LOSS, FAMILY_RATIO, FAMILY_TIME_WORK, SKILL_AVERAGE, SKILL_DIVISIBILITY,
    SKILL_LINEAR_EQUATIONS, SKILL_PERCENTAGE_SUCCESSIVE, SKILL_PROFIT_LOSS, SKILL_RATIO,
    SKILL_TIME_WORK,
};

#[test]
fn test_catalog_expansion_all_seven_topics_registered() {
    let service = ProceduralService::open_in_memory().unwrap();

    let expected_skills = [
        SKILL_PERCENTAGE_SUCCESSIVE,
        SKILL_LINEAR_EQUATIONS,
        SKILL_PROFIT_LOSS,
        SKILL_RATIO,
        SKILL_AVERAGE,
        SKILL_DIVISIBILITY,
        SKILL_TIME_WORK,
    ];

    for skill_id_str in expected_skills {
        let skill_id = SkillId::new(skill_id_str);
        let skill = service.store().get_skill(&skill_id).unwrap();
        assert!(skill.is_some(), "Skill {} should be registered in database", skill_id_str);
    }

    let all_schemas = service.store().list_all_schemas().unwrap();
    assert_eq!(all_schemas.len(), 30, "All 30 schemas (14 Maths + 2 Physics + 6 Chemistry + 8 Reasoning) should be present in catalog");
}

#[test]
fn test_generator_and_validator_registry_dispatch_for_all_topics() {
    let service = ProceduralService::open_in_memory().unwrap();

    let topics = [
        ("family.math.percentage.successive", 2),
        (FAMILY_LINEAR_EQUATIONS, 1),
        (FAMILY_PROFIT_LOSS, 2),
        (FAMILY_RATIO, 3),
        (FAMILY_AVERAGE, 1),
        (FAMILY_DIVISIBILITY, 2),
        (FAMILY_TIME_WORK, 2),
    ];

    for (fam_str, diff_level) in topics {
        let fam_id = procedural::ProblemFamilyId::new(fam_str);
        let inst = service
            .registry()
            .generate(&fam_id, "test.template", 12345, diff_level, None)
            .unwrap();

        assert!(!inst.rendered_prompt.is_empty(), "Prompt should not be empty for {}", fam_str);
        assert!(inst.correct_answer.get("value").is_some(), "Correct answer value should exist for {}", fam_str);

        service.save_problem_instance(inst.clone()).unwrap();

        // Check evaluation with correct answer
        let val = inst.correct_answer.get("value").unwrap();
        let outcome = service
            .evaluate_and_record_attempt(&inst.id, None, val.clone(), 15000, 0, 1)
            .unwrap();

        assert!(outcome.is_correct, "Evaluation should be correct for {}", fam_str);
        assert_eq!(outcome.error_category, None);
    }
}

#[test]
fn test_adaptive_difficulty_hysteresis_and_bounded_transitions() {
    let mut state = SkillState::new(SKILL_LINEAR_EQUATIONS);
    state.custom_state = serde_json::json!({ "current_difficulty_level": 2 });

    // Step 1: 1st fast success -> Hysteresis keeps level 2
    let ev1 = MasteryEvidence { final_correctness: true, latency_evidence: 15000, variant_exposure: Some("standard".to_string()), independence: IndependenceLevel::Independent, ..Default::default() };
    state.record_attempt_outcome(&ev1, 1.0, 35000, 1000);
    let dec1 = AdaptiveDifficultyEngine::evaluate_difficulty(Some(&state), None, None);
    assert_eq!(dec1.level, 2);

    // Step 2: 2nd fast success -> Hysteresis promotes to Level 3
    let ev2 = MasteryEvidence { final_correctness: true, latency_evidence: 16000, variant_exposure: Some("standard".to_string()), independence: IndependenceLevel::Independent, ..Default::default() };
    state.record_attempt_outcome(&ev2, 1.0, 35000, 1050);
    let dec2 = AdaptiveDifficultyEngine::evaluate_difficulty(Some(&state), None, None);
    assert_eq!(dec2.level, 3);
    assert_eq!(dec2.target_time_ms, 50000);

    // Update state to level 3
    state.custom_state = serde_json::json!({ "current_difficulty_level": 3 });

    // Step 3: Success but slow on Level 3 (took 70s on 50s target) -> Fluency hold keeps Level 3
    let ev3 = MasteryEvidence { final_correctness: true, latency_evidence: 70000, variant_exposure: Some("standard".to_string()), independence: IndependenceLevel::Independent, ..Default::default() };
    state.record_attempt_outcome(&ev3, 1.0, 50000, 1100);
    let dec3 = AdaptiveDifficultyEngine::evaluate_difficulty(Some(&state), None, None);
    assert_eq!(dec3.level, 3);
    assert!(dec3.reason.contains("fluency_hold"));

    // Step 4: Concept breakdown failure on Level 3 -> Fast demotion drops to Level 2
    let ev4 = MasteryEvidence { final_correctness: false, latency_evidence: 30000, variant_exposure: Some("standard".to_string()), diagnostic_errors: vec![ErrorCategory::Concept], ..Default::default() };
    state.record_attempt_outcome(&ev4, 0.0, 50000, 1150);
    let dec4 = AdaptiveDifficultyEngine::evaluate_difficulty(Some(&state), None, None);
    assert_eq!(dec4.level, 2);
    assert!(dec4.reason.contains("demoted_on_concept_breakdown"));
}

#[test]
fn test_multi_schema_selection_interleaving_and_focused_mode() {
    let service = ProceduralService::open_in_memory().unwrap();

    let all_schemas = service.store().list_all_schemas().unwrap();
    let states = std::collections::HashMap::new();

    // 1. In MixedMaths mode, after practicing Schema 0, selector should NOT immediately repeat Schema 0
    let last_schema = &all_schemas[0];
    let decision1 = MultiSchemaSelector::select_next_schema(
        &PracticeMode::MixedMaths,
        &all_schemas,
        &states,
        Some(&last_schema.id),
        12345,
    )
    .unwrap();

    assert_ne!(decision1.schema.id, last_schema.id, "Mixed mode must interleave schemas");

    // 2. In FocusedSkill mode, selector must pick the requested skill regardless of last_schema_id
    let target_skill = SkillId::new(SKILL_AVERAGE);
    let focused_mode = PracticeMode::FocusedSkill {
        skill_id: target_skill.clone(),
    };
    let decision2 = MultiSchemaSelector::select_next_schema(
        &focused_mode,
        &all_schemas,
        &states,
        Some(&decision1.schema.id),
        12345,
    )
    .unwrap();

    assert_eq!(decision2.schema.skill_id, target_skill, "Focused mode must honor selected skill");
}

#[test]
fn test_end_to_end_multi_schema_study_session() {
    let service = ProceduralService::open_in_memory().unwrap();

    // Problem 1: Mixed Maths -> Starts with a topic (e.g. Percentage)
    let s1 = service
        .prepare_multi_schema_session(&PracticeMode::MixedMaths, None, None, Some(101))
        .unwrap();
    let ans1 = s1.instance.correct_answer.get("value").unwrap().clone();
    let out1 = service
        .evaluate_and_record_attempt(&s1.instance.id, None, ans1, 20000, 0, 1)
        .unwrap();
    assert!(out1.is_correct);

    // Problem 2: Next mixed problem -> Anti-priming chooses a different schema (e.g. Ratio or Linear)
    let s2 = service
        .prepare_multi_schema_session(&PracticeMode::MixedMaths, None, Some(&s1.schema.id), Some(102))
        .unwrap();
    assert_ne!(s2.schema.id, s1.schema.id, "Problem 2 should interleave from Problem 1");

    // Fail problem 2 with a concept error
    let out2 = service
        .evaluate_and_record_attempt(&s2.instance.id, None, serde_json::json!(-9999), 25000, 0, 1)
        .unwrap();
    assert!(!out2.is_correct);
    let rating2 = service.derive_fsrs_rating(&out2).unwrap();
    assert_eq!(rating2, Rating::Again);

    // Problem 3: Next mixed problem -> Priority scoring MUST prioritize the failed skill for remediation!
    let s3 = service
        .prepare_multi_schema_session(&PracticeMode::MixedMaths, None, Some(&s2.schema.id), Some(103))
        .unwrap();
    assert_eq!(
        s3.schema.id, s2.schema.id,
        "Critical remediation must prioritize the failed schema"
    );

    // Solve problem 3 correctly
    let ans3 = s3.instance.correct_answer.get("value").unwrap().clone();
    let out3 = service
        .evaluate_and_record_attempt(&s3.instance.id, None, ans3, 18000, 0, 1)
        .unwrap();
    assert!(out3.is_correct);

    // Problem 4: With remediation complete, next step interleaves to another schema
    let s4 = service
        .prepare_multi_schema_session(&PracticeMode::MixedMaths, None, Some(&s3.schema.id), Some(104))
        .unwrap();
    assert_ne!(s4.schema.id, s3.schema.id, "Should interleave after remediation");
}
