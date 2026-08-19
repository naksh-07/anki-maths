// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::collections::HashMap;

use procedural::core::{Domain, SkillId};
use procedural::problems::catalog::*;
use procedural::skills::prerequisites::{
    PrerequisiteEvaluation, PrerequisiteGraphService, PrerequisitePolicy, PrerequisiteReadiness,
};
use procedural::skills::{PracticeProgressionState, RecentAttemptRecord, Skill, SkillState};
use procedural::storage::ProceduralStore;

#[test]
fn test_catalog_prerequisite_topology() {
    let store = ProceduralStore::open_in_memory().unwrap();
    MathsCatalog::init_all(&store).unwrap();

    let service = PrerequisiteGraphService::new();
    service.sync_from_store(&store).unwrap();

    // 1. Math multi-concept depends on percentage and ratio
    let multi_concept = SkillId::from(SKILL_COMBINED_MULTI_CONCEPT);
    let direct_mc = service.get_direct_prerequisites(&multi_concept);
    assert_eq!(direct_mc.len(), 2);
    assert!(direct_mc.contains(&SkillId::from(SKILL_PERCENTAGE_SUCCESSIVE)));
    assert!(direct_mc.contains(&SkillId::from(SKILL_RATIO)));

    // 2. Profit loss depends on percentage
    let pl_direct = service.get_direct_prerequisites(&SkillId::from(SKILL_PROFIT_LOSS));
    assert_eq!(pl_direct, vec![SkillId::from(SKILL_PERCENTAGE_SUCCESSIVE)]);

    // 3. Physics: work energy depends on kinematics
    let we_direct = service.get_direct_prerequisites(&SkillId::from(SKILL_PHYSICS_WORK_ENERGY));
    assert_eq!(we_direct, vec![SkillId::from(SKILL_PHYSICS_KINEMATICS)]);

    // 4. Chemistry: equilibrium depends on stoichiometry
    let eq_direct = service.get_direct_prerequisites(&SkillId::from(SKILL_CHEMISTRY_EQUILIBRIUM));
    assert_eq!(eq_direct, vec![SkillId::from(SKILL_CHEMISTRY_STOICHIOMETRY)]);

    // 5. Reasoning: seating depends on series
    let seat_direct = service.get_direct_prerequisites(&SkillId::from(SKILL_REASONING_SEATING));
    assert_eq!(seat_direct, vec![SkillId::from(SKILL_REASONING_SERIES)]);
}

#[test]
fn test_transitive_prerequisite_traversal_and_caching() {
    let service = PrerequisiteGraphService::new();

    let s1 = SkillId::new("skill_level_1");
    let s2 = SkillId::new("skill_level_2");
    let s3 = SkillId::new("skill_level_3");
    let s4 = SkillId::new("skill_level_4");

    // s4 -> s3 -> s2 -> s1
    service.register_skill_prerequisites(s2.clone(), vec![s1.clone()]);
    service.register_skill_prerequisites(s3.clone(), vec![s2.clone()]);
    service.register_skill_prerequisites(s4.clone(), vec![s3.clone()]);

    let (transitive_s4, _) = service.get_transitive_prerequisites(&s4);
    assert_eq!(transitive_s4.len(), 3);
    assert_eq!(transitive_s4, vec![s3.clone(), s2.clone(), s1.clone()]);

    // Check memoization works
    let (transitive_cached, _) = service.get_transitive_prerequisites(&s4);
    assert_eq!(transitive_cached, transitive_s4);

    // Invalidate cache and update
    let s0 = SkillId::new("skill_level_0");
    service.register_skill_prerequisites(s1.clone(), vec![s0.clone()]);

    let (updated_s4, _) = service.get_transitive_prerequisites(&s4);
    assert_eq!(updated_s4.len(), 4);
    assert!(updated_s4.contains(&s0));
}

#[test]
fn test_defensive_cycle_detection_and_cycle_breaking() {
    let service = PrerequisiteGraphService::new();

    let a = SkillId::new("skill_A");
    let b = SkillId::new("skill_B");
    let c = SkillId::new("skill_C");

    // 1. Direct Self-loop: A -> A
    service.register_skill_prerequisites(a.clone(), vec![a.clone()]);
    let direct = service.get_direct_prerequisites(&a);
    assert_eq!(direct, vec![a.clone()]);

    let (transitive, _) = service.get_transitive_prerequisites(&a);
    // Cycle is detected, terminates safely without infinite loop
    assert_eq!(transitive.len(), 1);

    let (has_cycle, cycles) = service.detect_cycles(&a);
    assert!(has_cycle);
    assert!(!cycles.is_empty());

    // 2. Multi-node cycle: A -> B -> C -> A
    service.register_skill_prerequisites(a.clone(), vec![b.clone()]);
    service.register_skill_prerequisites(b.clone(), vec![c.clone()]);
    service.register_skill_prerequisites(c.clone(), vec![a.clone()]);

    let (cycle_detected, cycle_paths) = service.detect_cycles(&a);
    assert!(cycle_detected);
    assert!(!cycle_paths.is_empty());

    // Transitive traversal handles the cycle gracefully without stack overflow (prerequisites of A are B and C)
    let (transitive_cycle, _) = service.get_transitive_prerequisites(&a);
    assert_eq!(transitive_cycle.len(), 2);
}

#[test]
fn test_max_depth_boundary_limiting() {
    let service = PrerequisiteGraphService::new().with_max_depth(3);

    // Chain of 10 skills: s10 -> s9 -> ... -> s1
    let skills: Vec<SkillId> = (1..=10).map(|i| SkillId::new(format!("chain_s{}", i))).collect();
    for i in 1..skills.len() {
        service.register_skill_prerequisites(skills[i].clone(), vec![skills[i - 1].clone()]);
    }

    let (result, _) = service.get_transitive_prerequisites(&skills[9]); // s10
    // With max_depth = 3, traversal is strictly bounded
    assert_eq!(result.len(), 3);
    assert_eq!(result, vec![skills[8].clone(), skills[7].clone(), skills[6].clone()]);
}

#[test]
fn test_readiness_evaluation_states_and_accuracy_signals() {
    let service = PrerequisiteGraphService::new();

    let target = SkillId::new("advanced_mechanics");
    let prereq_1 = SkillId::new("vector_algebra");
    let prereq_2 = SkillId::new("kinematics_1d");

    service.register_skill_prerequisites(target.clone(), vec![prereq_1.clone(), prereq_2.clone()]);

    let mut states = HashMap::new();

    // 1. Both prerequisites completely missing -> PrerequisitesNeeded
    let eval_missing = service.evaluate_readiness(&target, &states);
    assert!(matches!(eval_missing.readiness, PrerequisiteReadiness::PrerequisitesNeeded { .. }));
    assert!(eval_missing.requires_intervention());
    assert_eq!(eval_missing.missing_prerequisites.len(), 2);

    // 2. Prereq 1 Mastered, Prereq 2 in Learning with low accuracy -> PrerequisitesNeeded
    let mut state_1 = SkillState::new(prereq_1.clone());
    state_1.mastery = 0.9;
    state_1.confidence = 0.9;
    state_1.practice_state = PracticeProgressionState::Mastered;
    states.insert(prereq_1.clone(), state_1);

    let mut state_2_weak = SkillState::new(prereq_2.clone());
    state_2_weak.mastery = 0.4;
    state_2_weak.confidence = 0.4;
    state_2_weak.practice_state = PracticeProgressionState::Learning;
    state_2_weak.recent_attempts.push(RecentAttemptRecord {
        is_correct: false,
        score: 0.0,
        latency_ms: 40_000,
        target_latency_ms: 35_000,
        variant: None,
        error_category: None,
        max_hint_level: None,
        hint_count: None,
        timestamp: 1000,
    });
    states.insert(prereq_2.clone(), state_2_weak);

    let eval_weak = service.evaluate_readiness(&target, &states);
    assert!(matches!(eval_weak.readiness, PrerequisiteReadiness::PrerequisitesNeeded { .. }));

    // 3. Prereq 2 in Learning with strong recent accuracy (>= 75%, 2 consecutive successes) -> ReadyWithWarnings
    let mut state_2_improving = SkillState::new(prereq_2.clone());
    state_2_improving.mastery = 0.7;
    state_2_improving.confidence = 0.7;
    state_2_improving.practice_state = PracticeProgressionState::Learning;
    state_2_improving.consecutive_successes = 2;
    for _ in 0..3 {
        state_2_improving.recent_attempts.push(RecentAttemptRecord {
            is_correct: true,
            score: 1.0,
            latency_ms: 25_000,
            target_latency_ms: 35_000,
            variant: None,
            error_category: None,
            max_hint_level: None,
            hint_count: None,
            timestamp: 2000,
        });
    }
    states.insert(prereq_2.clone(), state_2_improving);

    let eval_improving = service.evaluate_readiness(&target, &states);
    assert!(matches!(eval_improving.readiness, PrerequisiteReadiness::ReadyWithWarnings { .. }));
    assert!(!eval_improving.requires_intervention());

    // 4. Both prerequisites Fluent/Mastered -> Ready
    let mut state_2_fluent = SkillState::new(prereq_2.clone());
    state_2_fluent.mastery = 0.85;
    state_2_fluent.confidence = 0.85;
    state_2_fluent.practice_state = PracticeProgressionState::Fluent;
    states.insert(prereq_2.clone(), state_2_fluent);

    let eval_ready = service.evaluate_readiness(&target, &states);
    assert_eq!(eval_ready.readiness, PrerequisiteReadiness::Ready);
    assert!(!eval_ready.requires_intervention());
}

#[test]
fn test_synthetic_graph_scale_and_performance() {
    let service = PrerequisiteGraphService::new().with_max_depth(10);

    // Build synthetic DAG of 100 skills
    for i in 1..=100 {
        let skill_id = SkillId::new(format!("synthetic_skill_{}", i));
        let prereqs = if i > 1 {
            vec![
                SkillId::new(format!("synthetic_skill_{}", i - 1)),
                SkillId::new(format!("synthetic_skill_{}", (i / 2).max(1))),
            ]
        } else {
            vec![]
        };
        service.register_skill_prerequisites(skill_id, prereqs);
    }

    let top_skill = SkillId::new("synthetic_skill_100");
    let (transitive, _) = service.get_transitive_prerequisites(&top_skill);
    assert!(!transitive.is_empty());
    assert!(transitive.len() <= 100);

    let (has_cycle, _) = service.detect_cycles(&top_skill);
    assert!(!has_cycle);
}
