// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use procedural::diagnostics::hints::{HintDependencyStats, HintLevel, HintUsageRecord};
use procedural::skills::signals::{IndependenceLevel, MasteryEvidence, PracticeProgressionState};
use procedural::skills::{ProgressionPolicy, SkillState};

#[test]
fn test_graduated_hint_levels_and_multipliers() {
    assert_eq!(HintLevel::Level0_None.independence_multiplier(), 1.0);
    assert_eq!(HintLevel::Level1_RetrievalCue.independence_multiplier(), 0.92);
    assert_eq!(HintLevel::Level2_ProceduralScaffold.independence_multiplier(), 0.65);
    assert_eq!(HintLevel::Level3_NearSolutionSupport.independence_multiplier(), 0.35);
}

#[test]
fn test_hint_dependency_stats_longitudinal_tracking() {
    let records = vec![
        HintUsageRecord::new(HintLevel::Level0_None, 0, true),
        HintUsageRecord::new(HintLevel::Level1_RetrievalCue, 1, true),
        HintUsageRecord::new(HintLevel::Level0_None, 0, true),
        HintUsageRecord::new(HintLevel::Level0_None, 0, true),
    ];

    let stats = HintDependencyStats::from_records(&records);
    assert_eq!(stats.total_attempts, 4);
    assert_eq!(stats.attempts_with_hints, 1);
    assert!(!stats.has_chronic_dependence());
    assert!(stats.composite_independence_score > 0.90);
}

#[test]
fn test_chronic_hint_dependence_flagged() {
    // 3 consecutive heavy Level 3 hint attempts
    let heavy_records = vec![
        HintUsageRecord::new(HintLevel::Level3_NearSolutionSupport, 2, true),
        HintUsageRecord::new(HintLevel::Level3_NearSolutionSupport, 3, true),
        HintUsageRecord::new(HintLevel::Level2_ProceduralScaffold, 1, true),
    ];

    let stats = HintDependencyStats::from_records(&heavy_records);
    assert_eq!(stats.consecutive_hint_attempts, 3);
    assert!(stats.has_chronic_dependence());
    assert!(stats.composite_independence_score < 0.60);
}

#[test]
fn test_occasional_level1_hint_does_not_block_progression() {
    let skill_id = procedural::core::SkillId::new("percentage.successive");
    let mut state = SkillState::new(skill_id);
    state.practice_state = PracticeProgressionState::Learning;
    state.consecutive_successes = 3;
    state.total_attempts = 4;

    // Push 3 successful attempts in window
    for _ in 0..3 {
        state.recent_attempts.push(procedural::skills::signals::RecentAttemptRecord {
            is_correct: true,
            score: 1.0,
            latency_ms: 25_000,
            target_latency_ms: 30_000,
            variant: Some("standard".into()),
            variant_category: Some(procedural::VariantCategory::Parameter),
            error_category: None,
            max_hint_level: Some(1), // Level 1 retrieval cue
            hint_count: Some(1),
            independence: Some(procedural::IndependenceLevel::LightSupport),
            solution_graph_fingerprint: None,
            cognitive_decision_correct: Some(true),
            timestamp: 100,
        });
    }

    let evidence = MasteryEvidence {
        final_correctness: true,
        decision_quality: Some(1.0),
        step_quality: None,
        independence: IndependenceLevel::LightSupport, // Light support allowed for Fluent transition
        max_hint_level: Some(1),
        hint_dependence: 1,
        retry_dependence: 0,
        variant_exposure: Some("standard".into()),
        variant_category: procedural::VariantCategory::Parameter,
        solution_graph_fingerprint: None,
        cognitive_decision_correct: Some(true),
        time_since_last_ms: None,
        transfer_evidence: false,
        domain_competence_verified: Some(true),
        latency_evidence: 25_000,
        diagnostic_errors: vec![],
    };

    ProgressionPolicy::evaluate(&mut state, &evidence);
    assert_eq!(state.practice_state, PracticeProgressionState::Fluent);
}

#[test]
fn test_chronic_level3_hints_prevent_false_mastery() {
    let skill_id = procedural::core::SkillId::new("percentage.successive");
    let mut state = SkillState::new(skill_id);
    state.practice_state = PracticeProgressionState::Transfer;
    state.consecutive_successes = 5;
    state.total_attempts = 10;

    // Push successful attempts but all requiring Level 3 heavy hints
    for _ in 0..5 {
        state.recent_attempts.push(procedural::skills::signals::RecentAttemptRecord {
            is_correct: true,
            score: 1.0,
            latency_ms: 25_000,
            target_latency_ms: 30_000,
            variant: Some("transfer_v1".into()),
            variant_category: Some(procedural::VariantCategory::Transfer),
            error_category: None,
            max_hint_level: Some(3),
            hint_count: Some(3),
            independence: Some(procedural::IndependenceLevel::SignificantSupport),
            solution_graph_fingerprint: None,
            cognitive_decision_correct: Some(true),
            timestamp: 100,
        });
    }

    let evidence = MasteryEvidence {
        final_correctness: true,
        decision_quality: Some(1.0),
        step_quality: None,
        independence: IndependenceLevel::SignificantSupport, // Significant support prevents Mastered
        max_hint_level: Some(3),
        hint_dependence: 3,
        retry_dependence: 0,
        variant_exposure: Some("transfer_v1".into()),
        variant_category: procedural::VariantCategory::Transfer,
        solution_graph_fingerprint: None,
        cognitive_decision_correct: Some(true),
        time_since_last_ms: None,
        transfer_evidence: true,
        domain_competence_verified: Some(true),
        latency_evidence: 25_000,
        diagnostic_errors: vec![],
    };

    ProgressionPolicy::evaluate(&mut state, &evidence);
    // Should NOT advance to Mastered due to lack of independence
    assert_ne!(state.practice_state, PracticeProgressionState::Mastered);
    assert_eq!(state.practice_state, PracticeProgressionState::Transfer);
}
