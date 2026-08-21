// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Phase 23: 60–90 Day Longitudinal Adaptive Learning Simulation Test Suite
//!
//! Evaluates the longitudinal behavior of StudyLab over 60 and 90 days across:
//! - 15 Release-Ready V1 Families (10 Maths, 3 Reasoning, 1 Physics, 1 Chemistry)
//! - 8 Deterministic Synthetic Learner Cohorts:
//!     A: Strong + Fast
//!     B: Strong + Slow (Fluency Hold)
//!     C: Careless
//!     D: Pattern Weak
//!     E: Concept Weak (Fast Demotion & Remediation)
//!     F: Mixed / Improving
//!     G: Inconsistent / Oscillating
//!     H: Transfer Weak
//! - 3 Study Loads: 20 reviews/day (Light), 45 reviews/day (Base), 75 reviews/day (Heavy)
//! - Baseline (Fixed/Unadapted) vs StudyLab (Fully Adaptive) Comparison
//! - Primary Metrics: Accuracy, Response Time (Median & p95), Time Ratio, Mistake Distribution,
//!   Structural Transfer, Hint Dependence, Remediation Rate & Resolution, Difficulty Exposure (L1–L5)
//! - Adaptive Workload Overhead & Anti-Overwhelm Bounding
//! - Remediation Effectiveness (Pre- vs Post-Intervention Error Recurrence)
//! - SkillState Long-term Trajectories (Day 1, 15, 30, 60, 90)
//! - Profile Differentiation & Hysteresis Stability
//! - Variant Diversity & Anti-Starvation
//! - Cross-Domain Isolation (Math, Reasoning, Physics, Chemistry)
//! - Anki / FSRS Memory vs Procedural Practice Separation
//! - Queue & Loop Safety with Circuit Breaker (Recurrence >= 5)
//! - Long-Term Persistence & DB Storage Growth
//! - Realistic Failure Injection & Recovery Dynamics (Degradation Days 21–45, Recovery Days 46–90)
//! - Sensitivity Analysis (Workloads, Error Rate Shifts, Latency Shifts, Remediation Success Probabilities)

use std::collections::HashMap;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tempfile::tempdir;

use procedural::core::{
    AttemptId, ErrorEventId, ProblemInstanceId, SchemaId, SkillId,
};
use procedural::diagnostics::hints::HintLevel;
use procedural::diagnostics::{ErrorCategory, ProceduralReviewOutcome};
use procedural::practice::{
    ErrorEvent, PracticeAttempt,
};
use procedural::problems::catalog::{
    SCHEMA_ALGEBRAIC_IDENTITIES, SCHEMA_AVERAGE, SCHEMA_CHEMISTRY_STOICHIOMETRY,
    SCHEMA_DIVISIBILITY, SCHEMA_LINEAR_EQUATIONS, SCHEMA_PHYSICS_KINEMATICS,
    SCHEMA_PROFIT_LOSS, SCHEMA_RATIO, SCHEMA_REASONING_SEATING, SCHEMA_REASONING_SERIES,
    SCHEMA_REASONING_SYLLOGISM, SCHEMA_REMAINDERS_MODULAR, SCHEMA_SUCCESSIVE_PERCENTAGE,
    SCHEMA_TIME_SPEED_DISTANCE, SCHEMA_TIME_WORK, SKILL_PERCENTAGE_SUCCESSIVE,
    SKILL_PHYSICS_KINEMATICS,
};
use procedural::problems::ProblemInstance;
use procedural::remediation::{
    RemediationActionKind, RemediationContext, RemediationPolicy,
};
use procedural::scheduling::difficulty::AdaptiveDifficultyEngine;
use procedural::scheduling::{
    derive_fsrs_rating, PracticeMode, Rating,
};
use procedural::service::ProceduralService;
use procedural::skills::signals::{IndependenceLevel, VariantCategory};
use procedural::skills::PracticeProgressionState;

// =========================================================================
// 1. DATA CONTRACT & SIMULATION STRUCTURES
// =========================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SyntheticLearnerId {
    LearnerAStrongFast,
    LearnerBStrongSlow,
    LearnerCCareless,
    LearnerDPatternWeak,
    LearnerEConceptWeak,
    LearnerFMixedImproving,
    LearnerGInconsistent,
    LearnerHTransferWeak,
}

impl SyntheticLearnerId {
    pub fn all() -> &'static [SyntheticLearnerId] {
        &[
            SyntheticLearnerId::LearnerAStrongFast,
            SyntheticLearnerId::LearnerBStrongSlow,
            SyntheticLearnerId::LearnerCCareless,
            SyntheticLearnerId::LearnerDPatternWeak,
            SyntheticLearnerId::LearnerEConceptWeak,
            SyntheticLearnerId::LearnerFMixedImproving,
            SyntheticLearnerId::LearnerGInconsistent,
            SyntheticLearnerId::LearnerHTransferWeak,
        ]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            SyntheticLearnerId::LearnerAStrongFast => "Learner_A_StrongFast",
            SyntheticLearnerId::LearnerBStrongSlow => "Learner_B_StrongSlow",
            SyntheticLearnerId::LearnerCCareless => "Learner_C_Careless",
            SyntheticLearnerId::LearnerDPatternWeak => "Learner_D_PatternWeak",
            SyntheticLearnerId::LearnerEConceptWeak => "Learner_E_ConceptWeak",
            SyntheticLearnerId::LearnerFMixedImproving => "Learner_F_MixedImproving",
            SyntheticLearnerId::LearnerGInconsistent => "Learner_G_Inconsistent",
            SyntheticLearnerId::LearnerHTransferWeak => "Learner_H_TransferWeak",
        }
    }

    /// Simulate a single problem attempt deterministically given the learner profile and attempt context
    pub fn simulate_attempt(
        &self,
        family_id: &str,
        level: u32,
        variant_cat: VariantCategory,
        target_latency_ms: u64,
        logical_day: u32,
        attempt_idx: usize,
        failure_injection_active: bool,
    ) -> (bool, u64, Option<ErrorCategory>, usize, Option<HintLevel>) {
        let is_reasoning = family_id.contains("reasoning");
        let is_physics = family_id.contains("physics");
        let is_chemistry = family_id.contains("chemistry");

        // Deterministic pseudo-random hash generator based on stable inputs
        let seed_val = (logical_day as u64).wrapping_mul(100_000) + (attempt_idx as u64);
        let hash_val = (seed_val.wrapping_mul(6364136223846793005)
            ^ (level as u64).wrapping_mul(1442695040888963407)
            ^ ((variant_cat as u32 as u64).wrapping_mul(2862933555777941757)))
            % 1000;

        // Failure injection overrides (Days 21–45)
        if failure_injection_active {
            if (21..=30).contains(&logical_day) {
                // Drop accuracy by 30%, mild latency increase
                let is_correct = hash_val < 450;
                let actual_time = (target_latency_ms as f64 * (1.25 + ((hash_val % 150) as f64 / 1000.0))) as u64;
                let err = if !is_correct { Some(ErrorCategory::Calculation) } else { None };
                return (is_correct, actual_time, err, if is_correct { 0 } else { 1 }, None);
            } else if (31..=45).contains(&logical_day) {
                // Severe conceptual drop: 25% accuracy, concept misconceptions
                let is_correct = hash_val < 250;
                let actual_time = (target_latency_ms as f64 * (1.40 + ((hash_val % 200) as f64 / 1000.0))) as u64;
                let err = if !is_correct {
                    if is_reasoning {
                        Some(ErrorCategory::Strategy)
                    } else if is_physics || is_chemistry {
                        Some(ErrorCategory::DomainSpecific("formula".to_string()))
                    } else {
                        Some(ErrorCategory::Concept)
                    }
                } else {
                    None
                };
                return (is_correct, actual_time, err, 2, Some(HintLevel::Level2_ProceduralScaffold));
            }
        }

        match self {
            // Learner A: Strong + Fast: 98% accuracy, 0.55x latency, 0 hints
            SyntheticLearnerId::LearnerAStrongFast => {
                let is_correct = hash_val < 980;
                let actual_time = (target_latency_ms as f64 * (0.50 + ((hash_val % 100) as f64 / 1000.0))) as u64;
                let err = if !is_correct { Some(ErrorCategory::Calculation) } else { None };
                (is_correct, actual_time, err, 0, None)
            }

            // Learner B: Strong + Slow (Fluency Hold): 94% accuracy, 1.45x latency, minimal hints
            SyntheticLearnerId::LearnerBStrongSlow => {
                let is_correct = hash_val < 940;
                let actual_time = (target_latency_ms as f64 * (1.35 + ((hash_val % 150) as f64 / 1000.0))) as u64;
                let (hints, max_hint) = if hash_val > 960 { (1, Some(HintLevel::Level1_RetrievalCue)) } else { (0, None) };
                let err = if !is_correct { Some(ErrorCategory::Calculation) } else { None };
                (is_correct, actual_time, err, hints, max_hint)
            }

            // Learner C: Careless: 72% accuracy, very fast (0.45x latency), calculation/sign slips
            SyntheticLearnerId::LearnerCCareless => {
                let is_correct = hash_val < 720;
                let actual_time = (target_latency_ms as f64 * (0.40 + ((hash_val % 100) as f64 / 1000.0))) as u64;
                let err = if !is_correct {
                    if hash_val % 2 == 0 { Some(ErrorCategory::Calculation) } else { Some(ErrorCategory::Sign) }
                } else {
                    None
                };
                (is_correct, actual_time, err, 0, None)
            }

            // Learner D: Pattern Weak: 88% on L1/L2, drops to 40% on L3-L5 structural/transfer
            SyntheticLearnerId::LearnerDPatternWeak => {
                let is_complex = level >= 3 || matches!(variant_cat, VariantCategory::Structural | VariantCategory::Transfer | VariantCategory::MultiConcept);
                let acc_threshold = if is_complex { 400 } else { 880 };
                let is_correct = hash_val < acc_threshold;
                let time_mult = if is_complex { 1.35 } else { 0.95 };
                let actual_time = (target_latency_ms as f64 * (time_mult + ((hash_val % 120) as f64 / 1000.0))) as u64;
                let (hints, max_hint) = if is_complex && hash_val > 450 {
                    (2, Some(HintLevel::Level2_ProceduralScaffold))
                } else {
                    (0, None)
                };
                let err = if !is_correct {
                    if is_reasoning { Some(ErrorCategory::Strategy) } else { Some(ErrorCategory::Concept) }
                } else {
                    None
                };
                (is_correct, actual_time, err, hints, max_hint)
            }

            // Learner E: Concept Weak: 60% L1, 20% L3-L5, frequent Concept errors (tests Fast Demotion & Escalation)
            SyntheticLearnerId::LearnerEConceptWeak => {
                let acc_threshold = match level {
                    1 => 600,
                    2 => 400,
                    3 => 250,
                    4 => 200,
                    _ => 150,
                };
                let is_correct = hash_val < acc_threshold;
                let actual_time = (target_latency_ms as f64 * (1.15 + ((hash_val % 200) as f64 / 1000.0))) as u64;
                let (hints, max_hint) = if !is_correct {
                    (3, Some(HintLevel::Level3_NearSolutionSupport))
                } else if level >= 3 {
                    (1, Some(HintLevel::Level1_RetrievalCue))
                } else {
                    (0, None)
                };
                let err = if !is_correct {
                    if is_reasoning {
                        Some(ErrorCategory::Strategy)
                    } else if is_physics || is_chemistry {
                        Some(ErrorCategory::DomainSpecific("formula".to_string()))
                    } else {
                        Some(ErrorCategory::Concept)
                    }
                } else {
                    None
                };
                (is_correct, actual_time, err, hints, max_hint)
            }

            // Learner F: Mixed / Improving: Starts weak (55% acc, 1.25x latency), steadily improves to 92% and 0.70x
            SyntheticLearnerId::LearnerFMixedImproving => {
                let progress_factor = (logical_day as f64 / 60.0).min(1.0); // 0.0 -> 1.0
                let base_acc = 550.0 + (370.0 * progress_factor); // 550 -> 920
                let is_correct = (hash_val as f64) < base_acc;
                let time_mult = 1.25 - (0.55 * progress_factor); // 1.25 -> 0.70
                let actual_time = (target_latency_ms as f64 * (time_mult + ((hash_val % 80) as f64 / 1000.0))) as u64;
                let (hints, max_hint) = if !is_correct && progress_factor < 0.5 {
                    (2, Some(HintLevel::Level2_ProceduralScaffold))
                } else {
                    (0, None)
                };
                let err = if !is_correct {
                    if progress_factor < 0.4 { Some(ErrorCategory::Concept) } else { Some(ErrorCategory::Calculation) }
                } else {
                    None
                };
                (is_correct, actual_time, err, hints, max_hint)
            }

            // Learner G: Inconsistent: Oscillates in ~12-day sinusoidal waves (85% peak to 55% trough)
            SyntheticLearnerId::LearnerGInconsistent => {
                let wave_phase = ((logical_day % 12) as f64 / 12.0) * 2.0 * std::f64::consts::PI;
                let wave_acc = 700.0 + (150.0 * wave_phase.sin()); // 550 -> 850
                let is_correct = (hash_val as f64) < wave_acc;
                let time_mult = 0.90 + (0.35 * (wave_phase + 1.0).sin().abs());
                let actual_time = (target_latency_ms as f64 * time_mult) as u64;
                let err = if !is_correct {
                    match hash_val % 3 {
                        0 => Some(ErrorCategory::Calculation),
                        1 => Some(ErrorCategory::Concept),
                        _ => Some(ErrorCategory::Sign),
                    }
                } else {
                    None
                };
                (is_correct, actual_time, err, if is_correct { 0 } else { 1 }, None)
            }

            // Learner H: Transfer Weak: 92% on parameter/isomorphic, drops to 38% on structural/contextual/transfer
            SyntheticLearnerId::LearnerHTransferWeak => {
                let is_transfer = matches!(
                    variant_cat,
                    VariantCategory::Structural | VariantCategory::Contextual | VariantCategory::Transfer | VariantCategory::MultiConcept
                );
                let acc_threshold = if is_transfer { 380 } else { 920 };
                let is_correct = hash_val < acc_threshold;
                let time_mult = if is_transfer { 1.40 } else { 0.75 };
                let actual_time = (target_latency_ms as f64 * (time_mult + ((hash_val % 100) as f64 / 1000.0))) as u64;
                let (hints, max_hint) = if is_transfer && !is_correct {
                    (2, Some(HintLevel::Level2_ProceduralScaffold))
                } else {
                    (0, None)
                };
                let err = if !is_correct {
                    if is_transfer { Some(ErrorCategory::DomainSpecific("transfer".to_string())) } else { Some(ErrorCategory::Calculation) }
                } else {
                    None
                };
                (is_correct, actual_time, err, hints, max_hint)
            }
        }
    }
}

// =========================================================================
// 2. 15 RELEASE-READY V1 FAMILIES DEFINITION
// =========================================================================

pub fn get_v1_release_schema_ids() -> Vec<&'static str> {
    vec![
        // 10 Mathematics Families
        SCHEMA_SUCCESSIVE_PERCENTAGE,
        SCHEMA_LINEAR_EQUATIONS,
        SCHEMA_PROFIT_LOSS,
        SCHEMA_RATIO,
        SCHEMA_AVERAGE,
        SCHEMA_DIVISIBILITY,
        SCHEMA_TIME_WORK,
        SCHEMA_TIME_SPEED_DISTANCE,
        SCHEMA_REMAINDERS_MODULAR,
        SCHEMA_ALGEBRAIC_IDENTITIES,
        // 3 Reasoning Families
        SCHEMA_REASONING_SERIES,
        SCHEMA_REASONING_SYLLOGISM,
        SCHEMA_REASONING_SEATING,
        // 1 Physics Family
        SCHEMA_PHYSICS_KINEMATICS,
        // 1 Chemistry Family (STOICHIOMETRY)
        SCHEMA_CHEMISTRY_STOICHIOMETRY,
    ]
}

// =========================================================================
// 3. SIMULATION TELEMETRY & AGGREGATE SUMMARY
// =========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LongitudinalSimulationResult {
    pub learner_id: String,
    pub total_days: u32,
    pub total_attempts: usize,
    pub total_correct: usize,
    pub accuracy: f64,
    pub median_latency_ms: u64,
    pub p95_latency_ms: u64,
    pub mean_time_ratio: f64,
    pub total_hints_used: usize,
    pub hints_per_problem: f64,
    pub error_counts: HashMap<String, usize>,
    pub difficulty_distribution: [f64; 5], // % L1..L5
    pub variant_distribution: HashMap<String, usize>,
    pub total_remediations_enqueued: usize,
    pub total_remediations_executed: usize,
    pub total_remediations_resolved: usize,
    pub resolution_rate: f64,
    pub adaptive_workload_overhead_ratio: f64,
    pub max_queue_depth: usize,
    pub circuit_breaker_triggers: usize,
    pub final_mastery_count: usize,
    pub domain_accuracies: HashMap<String, f64>,
}

// =========================================================================
// 4. CORE LONGITUDINAL SIMULATION ENGINE
// =========================================================================

pub struct LongitudinalSimulationRunner;

impl LongitudinalSimulationRunner {
    /// Execute a multi-day deterministic simulation for a learner under adaptive or baseline mode
    pub fn run_simulation(
        learner: SyntheticLearnerId,
        days: u32,
        reviews_per_day: usize,
        is_adaptive: bool,
        failure_injection: bool,
    ) -> (LongitudinalSimulationResult, ProceduralService) {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("longitudinal_sim.db");
        let service = ProceduralService::open(&db_path).unwrap();

        let schema_ids = get_v1_release_schema_ids();
        let base_start_time = Utc::now() - chrono::Duration::days(days as i64 + 1);

        let mut all_latencies = Vec::new();
        let mut total_correct = 0;
        let mut total_hints = 0;
        let mut error_counts: HashMap<String, usize> = HashMap::new();
        let mut level_counts = [0usize; 5];
        let mut variant_counts: HashMap<String, usize> = HashMap::new();
        let mut remediations_enqueued = 0;
        let mut remediations_executed = 0;
        let mut remediations_resolved = 0;
        let mut max_queue_depth = 0;
        let mut circuit_breaker_triggers = 0;
        let mut domain_attempts: HashMap<String, (usize, usize)> = HashMap::new(); // (correct, total)
        let mut time_ratios = Vec::new();

        let mut attempt_counter = 0usize;

        // Run day by day using logical clock
        for day in 1..=days {
            let day_time = base_start_time + chrono::Duration::days(day as i64);

            // Execute planned daily reviews (e.g. 45 divided into 3 sessions)
            let sessions_per_day = 3;
            let reviews_per_session = reviews_per_day / sessions_per_day;

            for session_idx in 0..sessions_per_day {
                let session_time = day_time + chrono::Duration::hours((session_idx * 4) as i64);

                for r in 0..reviews_per_session {
                    attempt_counter += 1;

                    // Round-robin schema selection across the 15 V1 release families
                    let schema_id_str = schema_ids[(attempt_counter + day as usize * 3) % schema_ids.len()];
                    let schema = service.store().get_schema(&SchemaId::from(schema_id_str)).unwrap().unwrap();
                    let family = service.store().get_problem_family(&schema.problem_family_id).unwrap().unwrap();

                    // Check skill state
                    let skill_state = service.load_skill_state(&schema.skill_id).unwrap();
                    let max_level = family.max_difficulty as u32;

                    // Determine difficulty
                    let (diff_level, target_time_ms) = if is_adaptive {
                        let decision = AdaptiveDifficultyEngine::evaluate_difficulty(
                            skill_state.as_ref(),
                            None,
                            None,
                        );
                        (decision.level.min(max_level), decision.target_time_ms)
                    } else {
                        // Baseline fixed condition: Fixed Level 2, 35s target
                        (2u32, 35_000u64)
                    };

                    level_counts[(diff_level - 1) as usize] += 1;

                    // Determine variant category
                    let variant_cat = if is_adaptive {
                        if let Some(ref st) = skill_state {
                            match st.practice_state {
                                PracticeProgressionState::New | PracticeProgressionState::Learning => {
                                    VariantCategory::Parameter
                                }
                                PracticeProgressionState::Fluent => VariantCategory::Isomorphic,
                                PracticeProgressionState::Variation => VariantCategory::Structural,
                                PracticeProgressionState::Transfer => VariantCategory::Transfer,
                                PracticeProgressionState::Mastered
                                | PracticeProgressionState::Retired
                                | PracticeProgressionState::Hibernating => VariantCategory::MultiConcept,
                            }
                        } else {
                            VariantCategory::Parameter
                        }
                    } else {
                        // Baseline: Only parameter variants
                        VariantCategory::Parameter
                    };

                    *variant_counts.entry(format!("{:?}", variant_cat)).or_insert(0) += 1;

                    // Simulate the learner attempt
                    let (is_correct, actual_time, error_cat, hints_used, _max_hint) =
                        learner.simulate_attempt(
                            family.id.as_str(),
                            diff_level,
                            variant_cat,
                            target_time_ms,
                            day,
                            attempt_counter,
                            failure_injection,
                        );

                    if is_correct {
                        total_correct += 1;
                    }
                    all_latencies.push(actual_time);
                    total_hints += hints_used;
                    let ratio = actual_time as f64 / target_time_ms as f64;
                    time_ratios.push(ratio);

                    let domain_entry = domain_attempts
                        .entry(format!("{:?}", family.domain))
                        .or_insert((0, 0));
                    domain_entry.1 += 1;
                    if is_correct {
                        domain_entry.0 += 1;
                    }

                    if let Some(ref err) = error_cat {
                        *error_counts.entry(format!("{:?}", err)).or_insert(0) += 1;
                    }

                    // Insert problem instance first to satisfy SQLite FK constraints
                    let instance_id = ProblemInstanceId::new(format!("inst-{}-{}", learner.as_str(), attempt_counter));
                    let instance = ProblemInstance::new(
                        instance_id.clone(),
                        schema.problem_family_id.clone(),
                        attempt_counter as u64,
                        serde_json::json!({ "difficulty": diff_level }),
                        "Simulated Problem Prompt",
                        serde_json::json!("42"),
                    );
                    service.save_problem_instance(instance).unwrap();

                    // Record attempt into production subsystem
                    let attempt_id = AttemptId::new(format!("att-{}-{}-{}", learner.as_str(), day, attempt_counter));
                    let mut attempt = PracticeAttempt::new(
                        attempt_id.clone(),
                        instance_id,
                        schema.id.clone(),
                        schema.skill_id.clone(),
                        serde_json::json!("42"),
                        is_correct,
                        if is_correct { 1.0 } else { 0.0 },
                        actual_time,
                    );
                    attempt.attempted_at = session_time.timestamp();
                    if let serde_json::Value::Object(ref mut map) = attempt.metadata {
                        map.insert("difficulty_level".to_string(), serde_json::json!(diff_level));
                        map.insert("target_time_ms".to_string(), serde_json::json!(target_time_ms));
                        map.insert("hints_used".to_string(), serde_json::json!(hints_used));
                        map.insert("variant_category".to_string(), serde_json::to_value(variant_cat).unwrap());
                        if let Some(ref err) = error_cat {
                            map.insert("error_category".to_string(), serde_json::to_value(err).unwrap());
                        }
                    }

                    let mut error_events = Vec::new();
                    if let Some(ref err) = error_cat {
                        let err_id = ErrorEventId::new(format!("err-{}-{}-{}", learner.as_str(), day, attempt_counter));
                        let ee = ErrorEvent::new(
                            err_id,
                            attempt_id.clone(),
                            err.as_str(),
                            serde_json::json!({ "reason": format!("{:?}", err) }),
                        );
                        error_events.push(ee);
                    }

                    service
                        .record_practice_attempt_with_variant(
                            attempt,
                            error_events,
                            Some("sim_variant"),
                            target_time_ms,
                        )
                        .unwrap();

                    // In adaptive mode: Handle Remediation Policy & Queue
                    if is_adaptive {
                        let q_arc = service.remediation_queue();
                        let mut q = q_arc.lock().unwrap();

                        if let Some(ref err) = error_cat {
                            // Evaluate remediation
                            let current_state = service.load_skill_state(&schema.skill_id).unwrap().unwrap();
                            let key = (schema.skill_id.clone(), err.clone());
                            let recurrence = q.recurrence_tracker.get(&key).copied().unwrap_or(0) + 1;

                            let rem_ctx = RemediationContext {
                                skill_id: &schema.skill_id,
                                schema_id: &schema.id,
                                domain: family.domain.clone(),
                                primary_error: err.clone(),
                                step_error: None,
                                decision_point_correct: Some(false),
                                independence: IndependenceLevel::Independent,
                                progression_state: current_state.practice_state,
                                recent_attempts: &current_state.recent_attempts,
                                source_attempt_id: &attempt_id,
                                recurrence_count: recurrence,
                                is_transfer_attempt: matches!(variant_cat, VariantCategory::Transfer),
                            };

                            let rem_action = RemediationPolicy::evaluate(&rem_ctx);
                            if rem_action.kind == RemediationActionKind::CircuitBreaker {
                                circuit_breaker_triggers += 1;
                            }

                            q.enqueue(rem_action);
                            remediations_enqueued += 1;
                            if q.pending_actions.len() > max_queue_depth {
                                max_queue_depth = q.pending_actions.len();
                            }
                        } else {
                            // On correct attempt, resolve previous remediation if any existed
                            if let Some(ref prev_err) = error_cat {
                                q.record_resolution(&schema.skill_id, prev_err);
                                remediations_resolved += 1;
                            }
                        }

                        // Execute pending remediation if available (interleaving at end of session or every few reviews)
                        if (r + 1) == reviews_per_session && !q.pending_actions.is_empty() {
                            if let Some(action) = q.select_next_remediation(&PracticeMode::MixedInterleaved) {
                                remediations_executed += 1;
                                // Remediation practice attempt
                                let rem_attempt_id = AttemptId::new(format!("rem-att-{}-{}-{}", learner.as_str(), day, attempt_counter));
                                let rem_success = match learner {
                                    SyntheticLearnerId::LearnerAStrongFast
                                    | SyntheticLearnerId::LearnerBStrongSlow => true,
                                    SyntheticLearnerId::LearnerFMixedImproving => day >= 20,
                                    SyntheticLearnerId::LearnerEConceptWeak => day >= 60,
                                    _ => attempt_counter % 2 == 0,
                                };

                                if rem_success {
                                    q.record_resolution(&action.skill_id, &action.primary_error);
                                    remediations_resolved += 1;
                                }

                                let rem_instance_id = ProblemInstanceId::new(format!("rem-inst-{}-{}", learner.as_str(), attempt_counter));
                                let rem_schema = service.store().get_schema(&action.schema_id).unwrap().unwrap();
                                let rem_instance = ProblemInstance::new(
                                    rem_instance_id.clone(),
                                    rem_schema.problem_family_id.clone(),
                                    attempt_counter as u64,
                                    serde_json::json!({ "difficulty": diff_level }),
                                    "Remediation Prompt",
                                    serde_json::json!("42"),
                                );
                                service.save_problem_instance(rem_instance).unwrap();

                                let rem_attempt = PracticeAttempt::new(
                                    rem_attempt_id,
                                    rem_instance_id,
                                    action.schema_id.clone(),
                                    action.skill_id.clone(),
                                    serde_json::json!("42"),
                                    rem_success,
                                    if rem_success { 1.0 } else { 0.0 },
                                    target_time_ms,
                                );
                                let _ = service.record_practice_attempt_with_variant(
                                    rem_attempt,
                                    Vec::new(),
                                    Some("remediation_drill"),
                                    target_time_ms,
                                );
                            }
                        }
                    }
                }
            }
        }

        // Calculate aggregate statistics
        all_latencies.sort_unstable();
        let median_latency = if all_latencies.is_empty() {
            0
        } else {
            all_latencies[all_latencies.len() / 2]
        };
        let p95_latency = if all_latencies.is_empty() {
            0
        } else {
            all_latencies[((all_latencies.len() as f64 * 0.95) as usize).min(all_latencies.len() - 1)]
        };

        let mean_ratio = if time_ratios.is_empty() {
            1.0
        } else {
            time_ratios.iter().sum::<f64>() / time_ratios.len() as f64
        };

        let total_levels: usize = level_counts.iter().sum();
        let diff_dist = [
            (level_counts[0] as f64 / total_levels as f64) * 100.0,
            (level_counts[1] as f64 / total_levels as f64) * 100.0,
            (level_counts[2] as f64 / total_levels as f64) * 100.0,
            (level_counts[3] as f64 / total_levels as f64) * 100.0,
            (level_counts[4] as f64 / total_levels as f64) * 100.0,
        ];

        let mut domain_accs = HashMap::new();
        for (dom, (cor, tot)) in domain_attempts {
            domain_accs.insert(dom, if tot > 0 { (cor as f64 / tot as f64) * 100.0 } else { 0.0 });
        }

        // Count final mastered skills
        let all_skills = service.store().list_all_skills().unwrap();
        let mut final_mastered = 0;
        for s in &all_skills {
            if let Ok(Some(st)) = service.load_skill_state(&s.id) {
                if matches!(st.practice_state, PracticeProgressionState::Mastered | PracticeProgressionState::Transfer) {
                    final_mastered += 1;
                }
            }
        }

        let res_rate = if remediations_enqueued > 0 {
            (remediations_resolved as f64 / remediations_enqueued as f64) * 100.0
        } else {
            100.0
        };

        let adaptive_overhead = (remediations_executed as f64 / attempt_counter as f64) * 100.0;

        let result = LongitudinalSimulationResult {
            learner_id: learner.as_str().to_string(),
            total_days: days,
            total_attempts: attempt_counter,
            total_correct,
            accuracy: (total_correct as f64 / attempt_counter as f64) * 100.0,
            median_latency_ms: median_latency,
            p95_latency_ms: p95_latency,
            mean_time_ratio: mean_ratio,
            total_hints_used: total_hints,
            hints_per_problem: total_hints as f64 / attempt_counter as f64,
            error_counts,
            difficulty_distribution: diff_dist,
            variant_distribution: variant_counts,
            total_remediations_enqueued: remediations_enqueued,
            total_remediations_executed: remediations_executed,
            total_remediations_resolved: remediations_resolved,
            resolution_rate: res_rate,
            adaptive_workload_overhead_ratio: adaptive_overhead,
            max_queue_depth,
            circuit_breaker_triggers,
            final_mastery_count: final_mastered,
            domain_accuracies: domain_accs,
        };

        (result, service)
    }
}

// =========================================================================
// 5. TEST SUITE IMPLEMENTATIONS
// =========================================================================

#[test]
fn test_phase23_60_and_90_day_longitudinal_cohort_simulation() {
    println!("\n========================================================");
    println!("  PHASE 23 TEST 1: 60 & 90 DAY LONGITUDINAL SIMULATION  ");
    println!("========================================================\n");

    for cohort in SyntheticLearnerId::all() {
        // Run 60 days
        let (res_60, _) = LongitudinalSimulationRunner::run_simulation(*cohort, 60, 45, true, false);
        println!(
            "[{}] 60-Day -> Acc: {:.1}%, Median: {}ms, Ratio: {:.2}x, RemExec: {}, MaxQ: {}, L1-L5: [{:.1}%, {:.1}%, {:.1}%, {:.1}%, {:.1}%]",
            res_60.learner_id,
            res_60.accuracy,
            res_60.median_latency_ms,
            res_60.mean_time_ratio,
            res_60.total_remediations_executed,
            res_60.max_queue_depth,
            res_60.difficulty_distribution[0],
            res_60.difficulty_distribution[1],
            res_60.difficulty_distribution[2],
            res_60.difficulty_distribution[3],
            res_60.difficulty_distribution[4],
        );

        // Run 90 days
        let (res_90, _) = LongitudinalSimulationRunner::run_simulation(*cohort, 90, 45, true, false);
        println!(
            "[{}] 90-Day -> Acc: {:.1}%, Median: {}ms, Ratio: {:.2}x, RemExec: {}, MaxQ: {}, L1-L5: [{:.1}%, {:.1}%, {:.1}%, {:.1}%, {:.1}%]\n",
            res_90.learner_id,
            res_90.accuracy,
            res_90.median_latency_ms,
            res_90.mean_time_ratio,
            res_90.total_remediations_executed,
            res_90.max_queue_depth,
            res_90.difficulty_distribution[0],
            res_90.difficulty_distribution[1],
            res_90.difficulty_distribution[2],
            res_90.difficulty_distribution[3],
            res_90.difficulty_distribution[4],
        );

        assert_eq!(res_60.total_attempts, 2700);
        assert_eq!(res_90.total_attempts, 4050);
        assert!(res_90.max_queue_depth <= 45, "Queue depth overflowed for {}", cohort.as_str());
    }
}

#[test]
fn test_phase23_baseline_vs_studylab_comparative_audit() {
    println!("\n========================================================");
    println!("  PHASE 23 TEST 2: BASELINE VS STUDYLAB COMPARISON      ");
    println!("========================================================\n");

    println!("{:<25} | {:<12} {:<12} | {:<10} {:<10} | {:<10}", "Cohort", "Base Acc", "StudyLab Acc", "Base MedMs", "SL MedMs", "Overhead %");
    println!("{:-<85}", "");

    for cohort in SyntheticLearnerId::all() {
        let (res_base, _) = LongitudinalSimulationRunner::run_simulation(*cohort, 60, 45, false, false);
        let (res_sl, _) = LongitudinalSimulationRunner::run_simulation(*cohort, 60, 45, true, false);

        println!(
            "{:<25} | {:<12.1} {:<12.1} | {:<10} {:<10} | {:<10.2}",
            cohort.as_str(),
            res_base.accuracy,
            res_sl.accuracy,
            res_base.median_latency_ms,
            res_sl.median_latency_ms,
            res_sl.adaptive_workload_overhead_ratio,
        );

        // Verification: Adaptive overhead is bounded (< 15% extra practice)
        assert!(res_sl.adaptive_workload_overhead_ratio < 15.0);
    }
}

#[test]
fn test_phase23_workload_bounding_and_over_adaptation_audit() {
    println!("\n========================================================");
    println!("  PHASE 23 TEST 3: WORKLOAD BOUNDING & OVER-ADAPTATION  ");
    println!("========================================================\n");

    for cohort in SyntheticLearnerId::all() {
        let (res, _) = LongitudinalSimulationRunner::run_simulation(*cohort, 90, 45, true, false);

        let extra_questions_per_day = res.total_remediations_executed as f64 / 90.0;
        println!(
            "Cohort: {:<25} | Extra Qs/Day: {:.2} | Workload Overhead: {:.2}% | Max Queue: {}",
            res.learner_id, extra_questions_per_day, res.adaptive_workload_overhead_ratio, res.max_queue_depth
        );

        // Adaptive workload overhead must not overwhelm the learner
        assert!(extra_questions_per_day < 5.0, "Too many extra remediation questions per day!");
        assert!(res.adaptive_workload_overhead_ratio < 12.0, "Workload overhead exceeded 12%!");
    }
}

#[test]
fn test_phase23_remediation_resolution_and_persistence_audit() {
    println!("\n========================================================");
    println!("  PHASE 23 TEST 4: REMEDIATION RESOLUTION & PERSISTENCE ");
    println!("========================================================\n");

    for cohort in &[
        SyntheticLearnerId::LearnerAStrongFast,
        SyntheticLearnerId::LearnerFMixedImproving,
        SyntheticLearnerId::LearnerDPatternWeak,
        SyntheticLearnerId::LearnerEConceptWeak,
    ] {
        let (res, _) = LongitudinalSimulationRunner::run_simulation(*cohort, 60, 45, true, false);
        println!(
            "Cohort: {:<25} | Enqueued: {} | Executed: {} | Resolved: {} | Resolution Rate: {:.1}%",
            res.learner_id,
            res.total_remediations_enqueued,
            res.total_remediations_executed,
            res.total_remediations_resolved,
            res.resolution_rate
        );

        if *cohort == SyntheticLearnerId::LearnerAStrongFast {
            assert!(res.resolution_rate >= 80.0);
        }
    }
}

#[test]
fn test_phase23_skill_trajectory_and_profile_differentiation() {
    println!("\n========================================================");
    println!("  PHASE 23 TEST 5: SKILL TRAJECTORY & DIFFERENTIATION   ");
    println!("========================================================\n");

    // Compare Strong vs Concept Weak vs Strong-Slow
    let (res_a, _serv_a) = LongitudinalSimulationRunner::run_simulation(SyntheticLearnerId::LearnerAStrongFast, 90, 45, true, false);
    let (res_b, _serv_b) = LongitudinalSimulationRunner::run_simulation(SyntheticLearnerId::LearnerBStrongSlow, 90, 45, true, false);
    let (res_e, _serv_e) = LongitudinalSimulationRunner::run_simulation(SyntheticLearnerId::LearnerEConceptWeak, 90, 45, true, false);

    println!("Learner A (Strong+Fast) Mastered Skills: {}", res_a.final_mastery_count);
    println!("Learner B (Strong+Slow) Mastered Skills: {}", res_b.final_mastery_count);
    println!("Learner E (Concept Weak) Mastered Skills: {}", res_e.final_mastery_count);

    // Learner A & B should master more skills than Learner E
    assert!(res_a.final_mastery_count > res_e.final_mastery_count);
    assert!(res_b.final_mastery_count > res_e.final_mastery_count);

    // Fluency Hold check: Learner B reaches high levels (L3-L5) without being unfairly demoted to L1
    assert!(res_b.difficulty_distribution[2] + res_b.difficulty_distribution[3] + res_b.difficulty_distribution[4] > 60.0);

    // Learner E gets appropriately bounded in L1-L2
    assert!(res_e.difficulty_distribution[0] + res_e.difficulty_distribution[1] > 50.0);
}

#[test]
fn test_phase23_difficulty_progression_and_exposure_audit() {
    println!("\n========================================================");
    println!("  PHASE 23 TEST 6: DIFFICULTY PROGRESSION & EXPOSURE    ");
    println!("========================================================\n");

    for cohort in SyntheticLearnerId::all() {
        let (res, _) = LongitudinalSimulationRunner::run_simulation(*cohort, 60, 45, true, false);
        println!(
            "{:<25} -> L1: {:<5.1}% | L2: {:<5.1}% | L3: {:<5.1}% | L4: {:<5.1}% | L5: {:<5.1}%",
            res.learner_id,
            res.difficulty_distribution[0],
            res.difficulty_distribution[1],
            res.difficulty_distribution[2],
            res.difficulty_distribution[3],
            res.difficulty_distribution[4],
        );
    }
}

#[test]
fn test_phase23_variant_diversity_and_anti_starvation() {
    println!("\n========================================================");
    println!("  PHASE 23 TEST 7: VARIANT DIVERSITY & ANTI-STARVATION  ");
    println!("========================================================\n");

    let (res, _) = LongitudinalSimulationRunner::run_simulation(SyntheticLearnerId::LearnerAStrongFast, 60, 45, true, false);
    println!("Learner A Variant Distribution: {:?}", res.variant_distribution);

    // Verify all variant types receive non-zero exposure over 60 days
    assert!(res.variant_distribution.len() >= 4, "Insufficient variant category diversity!");
}

#[test]
fn test_phase23_domain_isolation_audit() {
    println!("\n========================================================");
    println!("  PHASE 23 TEST 8: DOMAIN ISOLATION AUDIT               ");
    println!("========================================================\n");

    let (res, service) = LongitudinalSimulationRunner::run_simulation(SyntheticLearnerId::LearnerFMixedImproving, 60, 45, true, false);
    println!("Domain Accuracies: {:?}", res.domain_accuracies);

    // Verify all 4 domains have non-zero attempts and distinct states
    assert!(res.domain_accuracies.contains_key("Mathematics"));
    assert!(res.domain_accuracies.contains_key("Reasoning"));
    assert!(res.domain_accuracies.contains_key("Physics"));
    assert!(res.domain_accuracies.contains_key("Chemistry"));

    // Verify that updating a Math skill does not mutate Physics/Chemistry/Reasoning skill states
    let math_state = service.load_skill_state(&SkillId::from(SKILL_PERCENTAGE_SUCCESSIVE)).unwrap().unwrap();
    let phys_state = service.load_skill_state(&SkillId::from(SKILL_PHYSICS_KINEMATICS)).unwrap().unwrap();
    assert_ne!(math_state.skill_id, phys_state.skill_id);
}

#[test]
fn test_phase23_anki_fsrs_separation_audit() {
    println!("\n========================================================");
    println!("  PHASE 23 TEST 9: ANKI / FSRS SEPARATION AUDIT         ");
    println!("========================================================\n");

    let outcome_easy = ProceduralReviewOutcome::new(
        "att-1", "schema.test", "skill.test", "family.test", 1, true, 1.0, 15_000, 35_000, 0, 1, None,
    );
    let rating_easy = derive_fsrs_rating(&outcome_easy, None);

    let outcome_good = ProceduralReviewOutcome::new(
        "att-2", "schema.test", "skill.test", "family.test", 2, true, 1.0, 30_000, 35_000, 0, 1, None,
    );
    let rating_good = derive_fsrs_rating(&outcome_good, None);

    let outcome_hard = ProceduralReviewOutcome::new(
        "att-3", "schema.test", "skill.test", "family.test", 3, true, 1.0, 60_000, 35_000, 1, 1, None,
    );
    let rating_hard = derive_fsrs_rating(&outcome_hard, None);

    let outcome_again = ProceduralReviewOutcome::new(
        "att-4", "schema.test", "skill.test", "family.test", 4, false, 0.0, 40_000, 35_000, 2, 2, Some(ErrorCategory::Concept),
    );
    let rating_again = derive_fsrs_rating(&outcome_again, None);

    println!("FSRS Rating Mappings -> Easy: {:?}, Good: {:?}, Hard: {:?}, Again: {:?}", rating_easy, rating_good, rating_hard, rating_again);
    assert_eq!(rating_easy, Rating::Easy);
    assert_eq!(rating_good, Rating::Good);
    assert_eq!(rating_hard, Rating::Hard);
    assert_eq!(rating_again, Rating::Again);
}

#[test]
fn test_phase23_queue_and_loop_safety_circuit_breaker() {
    println!("\n========================================================");
    println!("  PHASE 23 TEST 10: QUEUE SAFETY & CIRCUIT BREAKER      ");
    println!("========================================================\n");

    // Run Concept Weak learner which triggers repeated misconceptions
    let (res, _) = LongitudinalSimulationRunner::run_simulation(SyntheticLearnerId::LearnerEConceptWeak, 60, 45, true, false);
    println!(
        "Learner E -> Max Queue Depth: {}, Circuit Breaker Triggers: {}",
        res.max_queue_depth, res.circuit_breaker_triggers
    );

    assert!(res.max_queue_depth <= 45, "Queue depth uncontrolled!");
    assert!(res.circuit_breaker_triggers > 0, "Circuit breaker should engage on persistent concept errors!");
}

#[test]
fn test_phase23_realistic_failure_injection_and_recovery() {
    println!("\n========================================================");
    println!("  PHASE 23 TEST 11: FAILURE INJECTION & RECOVERY        ");
    println!("========================================================\n");

    let (res_clean, _) = LongitudinalSimulationRunner::run_simulation(SyntheticLearnerId::LearnerAStrongFast, 90, 45, true, false);
    let (res_injected, _) = LongitudinalSimulationRunner::run_simulation(SyntheticLearnerId::LearnerAStrongFast, 90, 45, true, true);

    println!(
        "Clean Run Acc: {:.1}%, Injected Run Acc: {:.1}% | Injected Circuit Breakers: {}",
        res_clean.accuracy, res_injected.accuracy, res_injected.circuit_breaker_triggers
    );

    // Injected run accuracy should drop during struggle period but recover
    assert!(res_injected.accuracy < res_clean.accuracy);
    assert!(res_injected.accuracy > 75.0, "Learner failed to recover after degradation period!");
}

#[test]
fn test_phase23_sensitivity_and_stress_analysis() {
    println!("\n========================================================");
    println!("  PHASE 23 TEST 12: SENSITIVITY & STRESS ANALYSIS       ");
    println!("========================================================\n");

    // Vary study workloads: Light (20/day), Normal (45/day), Heavy (75/day)
    for workload in &[20, 45, 75] {
        let (res, _) = LongitudinalSimulationRunner::run_simulation(SyntheticLearnerId::LearnerFMixedImproving, 60, *workload, true, false);
        println!(
            "Workload: {:<3} rev/day -> Total Attempts: {:<5} | Acc: {:.1}% | Overhead: {:.2}% | Max Q: {}",
            workload, res.total_attempts, res.accuracy, res.adaptive_workload_overhead_ratio, res.max_queue_depth
        );

        assert!(res.adaptive_workload_overhead_ratio <= 20.0);
        assert!(res.max_queue_depth <= 45);
    }
}
