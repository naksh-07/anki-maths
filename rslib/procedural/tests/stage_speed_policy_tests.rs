// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use procedural::core::Domain;
use procedural::scheduling::speed::{DomainSpeedConfig, SpeedRating, StageSpeedPolicy};
use procedural::skills::signals::PracticeProgressionState;

#[test]
fn test_stage_latency_weights_progression() {
    // 1. New: Latency is informational only (0.0)
    assert_eq!(StageSpeedPolicy::latency_weight_for_stage(PracticeProgressionState::New), 0.0);

    // 2. Learning: Minimal influence (0.10)
    assert_eq!(StageSpeedPolicy::latency_weight_for_stage(PracticeProgressionState::Learning), 0.10);

    // 3. Fluent: Fluency evidence begins (0.50)
    assert_eq!(StageSpeedPolicy::latency_weight_for_stage(PracticeProgressionState::Fluent), 0.50);

    // 4. Variation: Meaningful fluency (0.70)
    assert_eq!(StageSpeedPolicy::latency_weight_for_stage(PracticeProgressionState::Variation), 0.70);

    // 5. Transfer: Generalization prioritized over speed (0.30)
    assert_eq!(StageSpeedPolicy::latency_weight_for_stage(PracticeProgressionState::Transfer), 0.30);

    // 6. Mastered & Retired: High maintenance automaticity (0.85)
    assert_eq!(StageSpeedPolicy::latency_weight_for_stage(PracticeProgressionState::Mastered), 0.85);
    assert_eq!(StageSpeedPolicy::latency_weight_for_stage(PracticeProgressionState::Retired), 0.85);
}

#[test]
fn test_domain_speed_baselines() {
    let maths_cfg = DomainSpeedConfig::for_domain(Domain::Mathematics);
    assert_eq!(maths_cfg.target_latency_ms, 30_000);

    let physics_cfg = DomainSpeedConfig::for_domain(Domain::Physics);
    assert_eq!(physics_cfg.target_latency_ms, 45_000);

    let chem_cfg = DomainSpeedConfig::for_domain(Domain::Chemistry);
    assert_eq!(chem_cfg.target_latency_ms, 45_000);

    let reasoning_cfg = DomainSpeedConfig::for_domain(Domain::Reasoning);
    assert_eq!(reasoning_cfg.target_latency_ms, 40_000);
}

#[test]
fn test_speed_evaluation_ratings_and_advisory() {
    // 1. Optimal speed in Maths (20s <= 30s target)
    let eval_optimal = StageSpeedPolicy::evaluate(
        PracticeProgressionState::Fluent,
        Domain::Mathematics,
        20_000,
        None,
    );
    assert_eq!(eval_optimal.speed_rating, SpeedRating::Optimal);
    assert_eq!(eval_optimal.fluency_score, 1.0);
    assert!(eval_optimal.is_acceptable);
    assert!(eval_optimal.advisory_message.is_none());

    // 2. Slow speed in Learning stage (90s): is_acceptable remains TRUE because early stage latency is informational
    let eval_learning_slow = StageSpeedPolicy::evaluate(
        PracticeProgressionState::Learning,
        Domain::Mathematics,
        90_000,
        None,
    );
    assert_eq!(eval_learning_slow.speed_rating, SpeedRating::Slow);
    assert!(eval_learning_slow.is_acceptable);

    // 3. Slow speed in Mastered stage (90s): is_acceptable is FALSE (demands fluency)
    let eval_mastered_slow = StageSpeedPolicy::evaluate(
        PracticeProgressionState::Mastered,
        Domain::Mathematics,
        90_000,
        None,
    );
    assert_eq!(eval_mastered_slow.speed_rating, SpeedRating::Slow);
    assert!(!eval_mastered_slow.is_acceptable);
    assert!(eval_mastered_slow.advisory_message.is_some());
}
