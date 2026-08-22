// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Phase 24: Adaptive Learning Quality & Longitudinal Outcome Audit Suite
//!
//! Targeted simulation and deep empirical audit analyzing:
//! 1. Replays for Cohorts D (Pattern Weak), H (Transfer Weak), A (Strong+Fast), E (Concept Weak), F (Mixed/Improving)
//! 2. Pattern-Weak Failure & Remediation Chains (Learner D)
//! 3. Transfer-Weak Multi-Tier Variant Analysis (Learner H)
//! 4. Diagnostic Exposure vs Learning Failure Classification
//! 5. Strong+Fast Latency & Difficulty Distribution (Learner A)
//! 6. Concept-Weak Reference Control Mechanism (Learner E)
//! 7. Remediation Effectiveness & Resolution Across All 8 Kinds
//! 8. Skill-Level vs Question-Level Generalization
//! 9. Difficulty Response (Promotions, Demotions, Holds, Recovery)
//! 10. Workload Quality Classification (Useful, Neutral, Redundant, Counterproductive)
//! 11. Variant Progression & Anti-Explosion Value
//! 12. Hint / Solution Dependence
//! 13. Controlled 4-Phase Recovery Verification
//! 14. Over-Adaptation & Conservatism Risk Audit

use std::collections::HashMap;
use chrono::Utc;
use tempfile::tempdir;

use procedural::core::{
    AttemptId, ErrorEventId, ProblemInstanceId, SchemaId, SkillId,
};
use procedural::diagnostics::hints::HintLevel;
use procedural::diagnostics::ErrorCategory;
use procedural::practice::{ErrorEvent, PracticeAttempt};
use procedural::problems::catalog::*;
use procedural::problems::ProblemInstance;
use procedural::remediation::{
    RemediationActionKind, RemediationContext, RemediationPolicy,
};
use procedural::scheduling::difficulty::AdaptiveDifficultyEngine;
use procedural::scheduling::PracticeMode;
use procedural::service::ProceduralService;
use procedural::skills::signals::{IndependenceLevel, VariantCategory};
use procedural::skills::PracticeProgressionState;

pub fn get_v1_schema_ids() -> Vec<&'static str> {
    vec![
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
        SCHEMA_REASONING_SERIES,
        SCHEMA_REASONING_SYLLOGISM,
        SCHEMA_REASONING_SEATING,
        SCHEMA_PHYSICS_KINEMATICS,
        SCHEMA_CHEMISTRY_STOICHIOMETRY,
    ]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Cohort {
    LearnerAStrongFast,
    LearnerDPatternWeak,
    LearnerEConceptWeak,
    LearnerFMixedImproving,
    LearnerHTransferWeak,
}

impl Cohort {
    pub fn name(&self) -> &'static str {
        match self {
            Cohort::LearnerAStrongFast => "Learner A (Strong+Fast)",
            Cohort::LearnerDPatternWeak => "Learner D (Pattern Weak)",
            Cohort::LearnerEConceptWeak => "Learner E (Concept Weak)",
            Cohort::LearnerFMixedImproving => "Learner F (Mixed/Improving)",
            Cohort::LearnerHTransferWeak => "Learner H (Transfer Weak)",
        }
    }

    pub fn simulate_attempt(
        &self,
        family_id: &str,
        level: u32,
        variant_cat: VariantCategory,
        target_latency_ms: u64,
        logical_day: u32,
        attempt_idx: usize,
    ) -> (bool, u64, Option<ErrorCategory>, usize, Option<HintLevel>) {
        let is_reasoning = family_id.contains("reasoning");
        let is_physics = family_id.contains("physics");
        let is_chemistry = family_id.contains("chemistry");

        let seed_val = (logical_day as u64).wrapping_mul(100_000) + (attempt_idx as u64);
        let hash_val = (seed_val.wrapping_mul(6364136223846793005)
            ^ (level as u64).wrapping_mul(1442695040888963407)
            ^ ((variant_cat as u32 as u64).wrapping_mul(2862933555777941757)))
            % 1000;

        match self {
            Cohort::LearnerAStrongFast => {
                let is_correct = hash_val < 980;
                let actual_time = (target_latency_ms as f64 * (0.50 + ((hash_val % 100) as f64 / 1000.0))) as u64;
                let err = if !is_correct { Some(ErrorCategory::Calculation) } else { None };
                (is_correct, actual_time, err, 0, None)
            }
            Cohort::LearnerDPatternWeak => {
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
            Cohort::LearnerEConceptWeak => {
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
            Cohort::LearnerFMixedImproving => {
                let progress_factor = (logical_day as f64 / 60.0).min(1.0);
                let base_acc = 550.0 + (370.0 * progress_factor);
                let is_correct = (hash_val as f64) < base_acc;
                let time_mult = 1.25 - (0.55 * progress_factor);
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
            Cohort::LearnerHTransferWeak => {
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

#[derive(Debug, Default, Clone)]
pub struct DayMetrics {
    pub day: u32,
    pub attempts: usize,
    pub correct: usize,
    pub familiar_attempts: usize,
    pub familiar_correct: usize,
    pub structural_attempts: usize,
    pub structural_correct: usize,
    pub transfer_attempts: usize,
    pub transfer_correct: usize,
    pub hints_used: usize,
    pub latencies: Vec<u64>,
    pub target_latencies: Vec<u64>,
    pub levels: [usize; 5],
    pub errors: HashMap<String, usize>,
    pub remediations_enqueued: usize,
    pub remediations_executed: usize,
    pub circuit_breakers: usize,
}

impl DayMetrics {
    pub fn accuracy(&self) -> f64 {
        if self.attempts == 0 { 0.0 } else { (self.correct as f64 / self.attempts as f64) * 100.0 }
    }
    pub fn familiar_accuracy(&self) -> f64 {
        if self.familiar_attempts == 0 { 0.0 } else { (self.familiar_correct as f64 / self.familiar_attempts as f64) * 100.0 }
    }
    pub fn structural_accuracy(&self) -> f64 {
        if self.structural_attempts == 0 { 0.0 } else { (self.structural_correct as f64 / self.structural_attempts as f64) * 100.0 }
    }
    pub fn transfer_accuracy(&self) -> f64 {
        if self.transfer_attempts == 0 { 0.0 } else { (self.transfer_correct as f64 / self.transfer_attempts as f64) * 100.0 }
    }
    pub fn median_latency(&self) -> u64 {
        if self.latencies.is_empty() { return 0; }
        let mut sorted = self.latencies.clone();
        sorted.sort_unstable();
        sorted[sorted.len() / 2]
    }
    pub fn median_target_latency(&self) -> u64 {
        if self.target_latencies.is_empty() { return 0; }
        let mut sorted = self.target_latencies.clone();
        sorted.sort_unstable();
        sorted[sorted.len() / 2]
    }
}

pub struct AuditHarness;

impl AuditHarness {
    pub fn run_longitudinal_audit(
        cohort: Cohort,
        days: u32,
        reviews_per_day: usize,
        is_adaptive: bool,
    ) -> (Vec<DayMetrics>, ProceduralService, HashMap<String, usize>, HashMap<String, usize>) {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("audit_sim.db");
        let service = ProceduralService::open(&db_path).unwrap();
        let schema_ids = get_v1_schema_ids();
        let base_start_time = Utc::now() - chrono::Duration::days(days as i64 + 1);

        let mut daily_metrics = Vec::new();
        let mut remediation_kind_counts = HashMap::new();
        let mut remediation_resolution_counts = HashMap::new();
        let mut attempt_counter = 0usize;

        for day in 1..=days {
            let day_time = base_start_time + chrono::Duration::days(day as i64);
            let mut dm = DayMetrics { day, ..Default::default() };
            let reviews_per_session = reviews_per_day / 3;

            for session_idx in 0..3 {
                let session_time = day_time + chrono::Duration::hours((session_idx * 4) as i64);

                for r in 0..reviews_per_session {
                    attempt_counter += 1;
                    dm.attempts += 1;

                    let schema_id_str = schema_ids[(attempt_counter + day as usize * 3) % schema_ids.len()];
                    let schema = service.store().get_schema(&SchemaId::from(schema_id_str)).unwrap().unwrap();
                    let family = service.store().get_problem_family(&schema.problem_family_id).unwrap().unwrap();

                    let skill_state = service.load_skill_state(&schema.skill_id).unwrap();
                    let max_level = family.max_difficulty as u32;

                    let (diff_level, target_time_ms) = if is_adaptive {
                        let decision = AdaptiveDifficultyEngine::evaluate_difficulty(
                            skill_state.as_ref(),
                            None,
                            None,
                        );
                        (decision.level.min(max_level), decision.target_time_ms)
                    } else {
                        (2u32, 35_000u64)
                    };

                    dm.levels[(diff_level - 1) as usize] += 1;

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
                        VariantCategory::Parameter
                    };

                    match variant_cat {
                        VariantCategory::Parameter | VariantCategory::Isomorphic => {
                            dm.familiar_attempts += 1;
                        }
                        VariantCategory::Structural | VariantCategory::Contextual => {
                            dm.structural_attempts += 1;
                        }
                        VariantCategory::Transfer | VariantCategory::MultiConcept => {
                            dm.transfer_attempts += 1;
                        }
                    }

                    let (is_correct, actual_time, error_cat, hints_used, _max_hint) =
                        cohort.simulate_attempt(
                            family.id.as_str(),
                            diff_level,
                            variant_cat,
                            target_time_ms,
                            day,
                            attempt_counter,
                        );

                    if is_correct {
                        dm.correct += 1;
                        match variant_cat {
                            VariantCategory::Parameter | VariantCategory::Isomorphic => dm.familiar_correct += 1,
                            VariantCategory::Structural | VariantCategory::Contextual => dm.structural_correct += 1,
                            VariantCategory::Transfer | VariantCategory::MultiConcept => dm.transfer_correct += 1,
                        }
                    }

                    dm.latencies.push(actual_time);
                    dm.target_latencies.push(target_time_ms);
                    dm.hints_used += hints_used;

                    if let Some(ref err) = error_cat {
                        *dm.errors.entry(format!("{:?}", err)).or_insert(0) += 1;
                    }

                    let instance_id = ProblemInstanceId::new(format!("inst-{}-{}", cohort.name(), attempt_counter));
                    let instance = ProblemInstance::new(
                        instance_id.clone(),
                        schema.problem_family_id.clone(),
                        attempt_counter as u64,
                        serde_json::json!({ "difficulty": diff_level }),
                        "Audit Problem Prompt",
                        serde_json::json!("42"),
                    );
                    service.save_problem_instance(instance).unwrap();

                    let attempt_id = AttemptId::new(format!("att-{}-{}-{}", cohort.name(), day, attempt_counter));
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

                    let mut error_events = Vec::new();
                    if let Some(ref err) = error_cat {
                        let err_id = ErrorEventId::new(format!("err-{}-{}-{}", cohort.name(), day, attempt_counter));
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
                            Some("audit_variant"),
                            target_time_ms,
                        )
                        .unwrap();

                    if is_adaptive {
                        let q_arc = service.remediation_queue();
                        let mut q = q_arc.lock().unwrap();

                        if let Some(ref err) = error_cat {
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
                            *remediation_kind_counts.entry(format!("{:?}", rem_action.kind)).or_insert(0) += 1;

                            if rem_action.kind == RemediationActionKind::CircuitBreaker {
                                dm.circuit_breakers += 1;
                            }

                            q.enqueue(rem_action);
                            dm.remediations_enqueued += 1;
                        }

                        if (r + 1) == reviews_per_session && !q.pending_actions.is_empty() {
                            if let Some(action) = q.select_next_remediation(&PracticeMode::MixedInterleaved) {
                                dm.remediations_executed += 1;
                                let rem_success = match cohort {
                                    Cohort::LearnerAStrongFast => true,
                                    Cohort::LearnerFMixedImproving => day >= 20,
                                    Cohort::LearnerEConceptWeak => day >= 60,
                                    _ => attempt_counter % 2 == 0,
                                };

                                if rem_success {
                                    q.record_resolution(&action.skill_id, &action.primary_error);
                                    *remediation_resolution_counts.entry(format!("{:?}", action.kind)).or_insert(0) += 1;
                                }

                                let rem_instance_id = ProblemInstanceId::new(format!("rem-inst-{}-{}", cohort.name(), attempt_counter));
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
                                    AttemptId::new(format!("rem-att-{}-{}-{}", cohort.name(), day, attempt_counter)),
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
            daily_metrics.push(dm);
        }

        (daily_metrics, service, remediation_kind_counts, remediation_resolution_counts)
    }
}

#[test]
fn test_audit_section1_and_2_pattern_weak_cohort_d() {
    println!("\n=== AUDIT SECTION 1 & 2: PATTERN WEAK COHORT D ===");
    let (metrics_sl, _, kinds, res) = AuditHarness::run_longitudinal_audit(Cohort::LearnerDPatternWeak, 90, 45, true);
    let (metrics_base, _, _, _) = AuditHarness::run_longitudinal_audit(Cohort::LearnerDPatternWeak, 90, 45, false);

    println!("Sample Days Trajectory for Learner D (StudyLab):");
    println!("{:<6} | {:<10} {:<10} {:<10} {:<10} | {:<10} | {:<10} | {:<15}", "Day", "Acc %", "Fam Acc%", "Struct Acc%", "Trans Acc%", "Med Lat(ms)", "Hints/Prob", "L1..L5 Dist");
    for &d in &[1, 15, 30, 60, 90] {
        let m = &metrics_sl[d - 1];
        let tot_lvl: usize = m.levels.iter().sum();
        println!(
            "Day {:<2} | {:<10.1} {:<10.1} {:<10.1} {:<10.1} | {:<10} | {:<10.2} | [{:.0}%, {:.0}%, {:.0}%, {:.0}%, {:.0}%]",
            d,
            m.accuracy(),
            m.familiar_accuracy(),
            m.structural_accuracy(),
            m.transfer_accuracy(),
            m.median_latency(),
            m.hints_used as f64 / m.attempts as f64,
            (m.levels[0] as f64 / tot_lvl as f64) * 100.0,
            (m.levels[1] as f64 / tot_lvl as f64) * 100.0,
            (m.levels[2] as f64 / tot_lvl as f64) * 100.0,
            (m.levels[3] as f64 / tot_lvl as f64) * 100.0,
            (m.levels[4] as f64 / tot_lvl as f64) * 100.0,
        );
    }

    let tot_att_sl: usize = metrics_sl.iter().map(|m| m.attempts).sum();
    let tot_cor_sl: usize = metrics_sl.iter().map(|m| m.correct).sum();
    let tot_att_base: usize = metrics_base.iter().map(|m| m.attempts).sum();
    let tot_cor_base: usize = metrics_base.iter().map(|m| m.correct).sum();

    println!("\nLearner D 90-Day Aggregate: StudyLab Acc = {:.1}%, Baseline Acc = {:.1}%",
        (tot_cor_sl as f64 / tot_att_sl as f64) * 100.0,
        (tot_cor_base as f64 / tot_att_base as f64) * 100.0
    );
    println!("Triggered Remediation Kinds for Learner D: {:?}", kinds);
    println!("Resolved Remediation Kinds for Learner D: {:?}", res);
}

#[test]
fn test_audit_section3_transfer_weak_cohort_h() {
    println!("\n=== AUDIT SECTION 3: TRANSFER WEAK COHORT H ===");
    let (metrics_sl, _, kinds, res) = AuditHarness::run_longitudinal_audit(Cohort::LearnerHTransferWeak, 90, 45, true);
    let (metrics_base, _, _, _) = AuditHarness::run_longitudinal_audit(Cohort::LearnerHTransferWeak, 90, 45, false);

    println!("Sample Days Trajectory for Learner H (StudyLab):");
    println!("{:<6} | {:<10} {:<10} {:<10} {:<10} | {:<10} | {:<10} | {:<15}", "Day", "Acc %", "Fam Acc%", "Struct Acc%", "Trans Acc%", "Med Lat(ms)", "Hints/Prob", "L1..L5 Dist");
    for &d in &[1, 15, 30, 60, 90] {
        let m = &metrics_sl[d - 1];
        let tot_lvl: usize = m.levels.iter().sum();
        println!(
            "Day {:<2} | {:<10.1} {:<10.1} {:<10.1} {:<10.1} | {:<10} | {:<10.2} | [{:.0}%, {:.0}%, {:.0}%, {:.0}%, {:.0}%]",
            d,
            m.accuracy(),
            m.familiar_accuracy(),
            m.structural_accuracy(),
            m.transfer_accuracy(),
            m.median_latency(),
            m.hints_used as f64 / m.attempts as f64,
            (m.levels[0] as f64 / tot_lvl as f64) * 100.0,
            (m.levels[1] as f64 / tot_lvl as f64) * 100.0,
            (m.levels[2] as f64 / tot_lvl as f64) * 100.0,
            (m.levels[3] as f64 / tot_lvl as f64) * 100.0,
            (m.levels[4] as f64 / tot_lvl as f64) * 100.0,
        );
    }

    let tot_att_sl: usize = metrics_sl.iter().map(|m| m.attempts).sum();
    let tot_cor_sl: usize = metrics_sl.iter().map(|m| m.correct).sum();
    let tot_att_base: usize = metrics_base.iter().map(|m| m.attempts).sum();
    let tot_cor_base: usize = metrics_base.iter().map(|m| m.correct).sum();

    println!("\nLearner H 90-Day Aggregate: StudyLab Acc = {:.1}%, Baseline Acc = {:.1}%",
        (tot_cor_sl as f64 / tot_att_sl as f64) * 100.0,
        (tot_cor_base as f64 / tot_att_base as f64) * 100.0
    );
    println!("Triggered Remediation Kinds for Learner H: {:?}", kinds);
    println!("Resolved Remediation Kinds for Learner H: {:?}", res);
}

#[test]
fn test_audit_section5_strong_fast_cohort_a() {
    println!("\n=== AUDIT SECTION 5: STRONG + FAST COHORT A ===");
    let (metrics_sl, _, _, _) = AuditHarness::run_longitudinal_audit(Cohort::LearnerAStrongFast, 90, 45, true);
    let (metrics_base, _, _, _) = AuditHarness::run_longitudinal_audit(Cohort::LearnerAStrongFast, 90, 45, false);

    let m_sl_day1 = &metrics_sl[0];
    let m_sl_day90 = &metrics_sl[89];
    let m_base_day90 = &metrics_base[89];

    println!("Learner A Comparison:");
    println!("StudyLab Day 1  : Median Latency = {}ms, Median Target = {}ms, Acc = {:.1}%", m_sl_day1.median_latency(), m_sl_day1.median_target_latency(), m_sl_day1.accuracy());
    println!("StudyLab Day 90 : Median Latency = {}ms, Median Target = {}ms, Acc = {:.1}%", m_sl_day90.median_latency(), m_sl_day90.median_target_latency(), m_sl_day90.accuracy());
    println!("Baseline Day 90 : Median Latency = {}ms, Median Target = {}ms, Acc = {:.1}%", m_base_day90.median_latency(), m_base_day90.median_target_latency(), m_base_day90.accuracy());

    let tot_levels_sl: [usize; 5] = metrics_sl.iter().fold([0; 5], |mut acc, m| {
        for i in 0..5 { acc[i] += m.levels[i]; }
        acc
    });
    let sum_sl: usize = tot_levels_sl.iter().sum();
    println!("StudyLab 90-Day Difficulty Exposure: L1={:.1}%, L2={:.1}%, L3={:.1}%, L4={:.1}%, L5={:.1}%",
        (tot_levels_sl[0] as f64 / sum_sl as f64) * 100.0,
        (tot_levels_sl[1] as f64 / sum_sl as f64) * 100.0,
        (tot_levels_sl[2] as f64 / sum_sl as f64) * 100.0,
        (tot_levels_sl[3] as f64 / sum_sl as f64) * 100.0,
        (tot_levels_sl[4] as f64 / sum_sl as f64) * 100.0,
    );
}

#[test]
fn test_audit_section6_concept_weak_cohort_e() {
    println!("\n=== AUDIT SECTION 6: CONCEPT WEAK COHORT E ===");
    let (metrics_sl, _, kinds, res) = AuditHarness::run_longitudinal_audit(Cohort::LearnerEConceptWeak, 90, 45, true);
    let (metrics_base, _, _, _) = AuditHarness::run_longitudinal_audit(Cohort::LearnerEConceptWeak, 90, 45, false);

    let tot_att_sl: usize = metrics_sl.iter().map(|m| m.attempts).sum();
    let tot_cor_sl: usize = metrics_sl.iter().map(|m| m.correct).sum();
    let tot_att_base: usize = metrics_base.iter().map(|m| m.attempts).sum();
    let tot_cor_base: usize = metrics_base.iter().map(|m| m.correct).sum();

    println!("Learner E 90-Day Aggregate: StudyLab Acc = {:.1}%, Baseline Acc = {:.1}%",
        (tot_cor_sl as f64 / tot_att_sl as f64) * 100.0,
        (tot_cor_base as f64 / tot_att_base as f64) * 100.0
    );

    let tot_levels_sl: [usize; 5] = metrics_sl.iter().fold([0; 5], |mut acc, m| {
        for i in 0..5 { acc[i] += m.levels[i]; }
        acc
    });
    let sum_sl: usize = tot_levels_sl.iter().sum();
    println!("Learner E Difficulty Exposure: L1={:.1}%, L2={:.1}%, L3={:.1}%, L4={:.1}%, L5={:.1}%",
        (tot_levels_sl[0] as f64 / sum_sl as f64) * 100.0,
        (tot_levels_sl[1] as f64 / sum_sl as f64) * 100.0,
        (tot_levels_sl[2] as f64 / sum_sl as f64) * 100.0,
        (tot_levels_sl[3] as f64 / sum_sl as f64) * 100.0,
        (tot_levels_sl[4] as f64 / sum_sl as f64) * 100.0,
    );
    println!("Triggered Remediation Kinds for Learner E: {:?}", kinds);
    println!("Resolved Remediation Kinds for Learner E: {:?}", res);
}

#[test]
fn test_audit_section14_controlled_recovery_test() {
    println!("\n=== AUDIT SECTION 14: CONTROLLED RECOVERY TEST ===");
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("recovery_test.db");
    let service = ProceduralService::open(&db_path).unwrap();
    let schema_id = SchemaId::from(SCHEMA_SUCCESSIVE_PERCENTAGE);
    let schema = service.store().get_schema(&schema_id).unwrap().unwrap();
    let target_time_ms = 35_000u64;

    let mut q = service.remediation_queue().lock().unwrap().clone();

    // Phase 1: Failures
    for i in 1..=5 {
        let instance_id = ProblemInstanceId::new(format!("inst-rec-{}", i));
        let instance = ProblemInstance::new(
            instance_id.clone(),
            schema.problem_family_id.clone(),
            i as u64,
            serde_json::json!({ "difficulty": 3 }),
            "Prompt",
            serde_json::json!("42"),
        );
        service.save_problem_instance(instance).unwrap();

        let attempt_id = AttemptId::new(format!("att-rec-{}", i));
        let attempt = PracticeAttempt::new(
            attempt_id.clone(),
            instance_id,
            schema.id.clone(),
            schema.skill_id.clone(),
            serde_json::json!("42"),
            false,
            0.0,
            45_000,
        );
        let error_events = vec![ErrorEvent::new(
            ErrorEventId::new(format!("err-rec-{}", i)),
            attempt_id.clone(),
            "concept_error",
            serde_json::json!({ "reason": "misconception" }),
        )];
        service.record_practice_attempt_with_variant(attempt, error_events, Some("struct_1"), target_time_ms).unwrap();

        let family = service.store().get_problem_family(&schema.problem_family_id).unwrap().unwrap();
        let rem_ctx = RemediationContext {
            skill_id: &schema.skill_id,
            schema_id: &schema.id,
            domain: family.domain.clone(),
            primary_error: ErrorCategory::Concept,
            step_error: None,
            decision_point_correct: Some(false),
            independence: IndependenceLevel::Independent,
            progression_state: PracticeProgressionState::Variation,
            recent_attempts: &[],
            source_attempt_id: &attempt_id,
            recurrence_count: i,
            is_transfer_attempt: true,
        };
        let action = RemediationPolicy::evaluate(&rem_ctx);
        q.enqueue(action);
    }

    println!("Phase 1: Enqueued {} pending remediation actions with recurrence = {}", q.pending_actions.len(), q.get_recurrence_count(&schema.skill_id, &ErrorCategory::Concept));
    assert_eq!(q.pending_actions.len(), 1);
    assert_eq!(q.get_recurrence_count(&schema.skill_id, &ErrorCategory::Concept), 5);
    // At recurrence 5, action is circuit breaker
    assert_eq!(q.pending_actions[0].kind, RemediationActionKind::CircuitBreaker);

    // Phase 2 & 3: Resolving remediations
    while let Some(action) = q.select_next_remediation(&PracticeMode::MixedInterleaved) {
        q.record_resolution(&action.skill_id, &action.primary_error);
    }
    println!("Phase 2 & 3: Completed and resolved all pending remediations. Pending left: {}", q.pending_actions.len());
    assert_eq!(q.pending_actions.len(), 0);

    // Phase 4: Successful practice on novel transfer questions
    for i in 6..=10 {
        let instance_id = ProblemInstanceId::new(format!("inst-rec-{}", i));
        let instance = ProblemInstance::new(
            instance_id.clone(),
            schema.problem_family_id.clone(),
            i as u64,
            serde_json::json!({ "difficulty": 3 }),
            "Prompt",
            serde_json::json!("42"),
        );
        service.save_problem_instance(instance).unwrap();

        let attempt_id = AttemptId::new(format!("att-rec-{}", i));
        let attempt = PracticeAttempt::new(
            attempt_id.clone(),
            instance_id,
            schema.id.clone(),
            schema.skill_id.clone(),
            serde_json::json!("42"),
            true,
            1.0,
            25_000,
        );
        service.record_practice_attempt_with_variant(attempt, Vec::new(), Some("novel_transfer"), target_time_ms).unwrap();
    }

    let final_st = service.load_skill_state(&schema.skill_id).unwrap().unwrap();
    let dec = AdaptiveDifficultyEngine::evaluate_difficulty(Some(&final_st), None, None);
    println!("Phase 4: SkillState practice_state = {:?}, recent_accuracy = {:.1}%, next difficulty decision = L{} ({})",
        final_st.practice_state,
        final_st.recent_accuracy() * 100.0,
        dec.level,
        dec.reason
    );
    assert!(final_st.recent_accuracy() >= 0.80);
}

#[test]
fn test_audit_section7_and_8_remediation_effectiveness_and_resolution() {
    println!("\n=== AUDIT SECTION 7 & 8: REMEDIATION EFFECTIVENESS & RESOLUTION ===");
    // Run all 5 cohorts and aggregate remediation actions triggered, completed, resolved
    let cohorts = [
        Cohort::LearnerAStrongFast,
        Cohort::LearnerDPatternWeak,
        Cohort::LearnerEConceptWeak,
        Cohort::LearnerFMixedImproving,
        Cohort::LearnerHTransferWeak,
    ];

    let mut aggregate_kinds: HashMap<String, usize> = HashMap::new();
    let mut aggregate_resolved: HashMap<String, usize> = HashMap::new();

    for c in &cohorts {
        let (_, _, kinds, resolved) = AuditHarness::run_longitudinal_audit(*c, 60, 45, true);
        for (k, v) in kinds {
            *aggregate_kinds.entry(k).or_insert(0) += v;
        }
        for (k, v) in resolved {
            *aggregate_resolved.entry(k).or_insert(0) += v;
        }
    }

    println!("{:<25} | {:<12} | {:<12} | {:<10}", "Remediation Kind", "Triggered", "Resolved", "Res Rate %");
    println!("{:-<65}", "");
    for (k, triggered) in &aggregate_kinds {
        let res = aggregate_resolved.get(k).copied().unwrap_or(0);
        let rate = if *triggered > 0 { (res as f64 / *triggered as f64) * 100.0 } else { 0.0 };
        println!("{:<25} | {:<12} | {:<12} | {:<10.1}", k, triggered, res, rate);
    }
}

#[test]
fn test_audit_section10_difficulty_response_and_hysteresis() {
    println!("\n=== AUDIT SECTION 10: DIFFICULTY RESPONSE & HYSTERESIS AUDIT ===");
    use procedural::skills::SkillState;
    use procedural::skills::signals::MasteryEvidence;

    let mut state = SkillState::new("test.diff.audit");
    state.custom_state = serde_json::json!({ "current_difficulty_level": 3 });

    // 1. Correct + fast -> Hysteresis requires >= 2 consecutive
    let ev_fast = MasteryEvidence {
        final_correctness: true,
        latency_evidence: 20_000,
        independence: IndependenceLevel::Independent,
        ..Default::default()
    };
    state.record_attempt_outcome(&ev_fast, 1.0, 50_000, 1000);
    let d1 = AdaptiveDifficultyEngine::evaluate_difficulty(Some(&state), None, None);
    println!("1 Fast Correct: Level = {}, Reason = {}", d1.level, d1.reason);
    assert_eq!(d1.level, 3); // Not promoted yet (needs 2 consecutive)

    state.record_attempt_outcome(&ev_fast, 1.0, 50_000, 1050);
    let d2 = AdaptiveDifficultyEngine::evaluate_difficulty(Some(&state), None, None);
    println!("2nd Fast Correct: Level = {}, Reason = {}", d2.level, d2.reason);
    assert_eq!(d2.level, 4); // Promoted!

    // 2. Correct + slow (> 1.25x target) -> Fluency Hold
    let mut state_slow = SkillState::new("test.diff.slow");
    state_slow.custom_state = serde_json::json!({ "current_difficulty_level": 3 });
    let ev_slow = MasteryEvidence {
        final_correctness: true,
        latency_evidence: 70_000, // target is 50_000, 70s > 62.5s
        independence: IndependenceLevel::Independent,
        ..Default::default()
    };
    state_slow.record_attempt_outcome(&ev_slow, 1.0, 50_000, 1000);
    let d_slow = AdaptiveDifficultyEngine::evaluate_difficulty(Some(&state_slow), None, None);
    println!("Correct + Slow: Level = {}, Reason = {}", d_slow.level, d_slow.reason);
    assert_eq!(d_slow.level, 3);
    assert!(d_slow.reason.contains("fluency_hold"));

    // 3. Wrong + Careless (Calculation slip) -> Single slip does NOT immediately demote
    let mut state_slip = SkillState::new("test.diff.slip");
    state_slip.custom_state = serde_json::json!({ "current_difficulty_level": 3 });
    let ev_slip = MasteryEvidence {
        final_correctness: false,
        latency_evidence: 20_000,
        diagnostic_errors: vec![ErrorCategory::Calculation], domain_evidence: None,
        ..Default::default()
    };
    state_slip.record_attempt_outcome(&ev_slip, 0.0, 50_000, 1000);
    let d_slip = AdaptiveDifficultyEngine::evaluate_difficulty(Some(&state_slip), None, None);
    println!("Wrong + Calculation Slip: Level = {}, Reason = {}", d_slip.level, d_slip.reason);
    assert_eq!(d_slip.level, 3); // Level maintained, not demoted on 1 calculation slip!

    // 4. Wrong + Concept / Strategy -> Demotes immediately (Fast Demotion)
    let mut state_concept = SkillState::new("test.diff.concept");
    state_concept.custom_state = serde_json::json!({ "current_difficulty_level": 3 });
    let ev_concept = MasteryEvidence {
        final_correctness: false,
        latency_evidence: 45_000,
        diagnostic_errors: vec![ErrorCategory::Concept], domain_evidence: None,
        ..Default::default()
    };
    state_concept.record_attempt_outcome(&ev_concept, 0.0, 50_000, 1000);
    let d_concept = AdaptiveDifficultyEngine::evaluate_difficulty(Some(&state_concept), None, None);
    println!("Wrong + Concept: Level = {}, Reason = {}", d_concept.level, d_concept.reason);
    assert_eq!(d_concept.level, 2); // Demoted L3 -> L2

    // 5. Repeated failures (consecutive failures >= 2) -> Demotes
    let mut state_rep = SkillState::new("test.diff.rep");
    state_rep.custom_state = serde_json::json!({ "current_difficulty_level": 3 });
    state_rep.record_attempt_outcome(&ev_slip, 0.0, 50_000, 1000);
    state_rep.record_attempt_outcome(&ev_slip, 0.0, 50_000, 1050);
    let d_rep = AdaptiveDifficultyEngine::evaluate_difficulty(Some(&state_rep), None, None);
    println!("2 Consecutive Calculation Slips: Level = {}, Reason = {}", d_rep.level, d_rep.reason);
    assert_eq!(d_rep.level, 2); // Demoted after 2nd consecutive failure
}

#[test]
fn test_audit_section15_over_adaptation_risk_audit() {
    println!("\n=== AUDIT SECTION 15: OVER-ADAPTATION & CONSERVATISM RISK AUDIT ===");
    use procedural::skills::SkillState;
    use procedural::skills::signals::MasteryEvidence;

    let mut state = SkillState::new("test.overadapt");
    state.practice_state = PracticeProgressionState::Transfer;
    state.custom_state = serde_json::json!({ "current_difficulty_level": 4 });

    // Learner fails 1 transfer problem with domain-specific transfer error
    let ev_trans_fail = MasteryEvidence {
        final_correctness: false,
        latency_evidence: 55_000,
        variant_category: VariantCategory::Transfer,
        diagnostic_errors: vec![ErrorCategory::DomainSpecific("transfer".to_string())], domain_evidence: None,
        ..Default::default()
    };
    state.record_attempt_outcome(&ev_trans_fail, 0.0, 65_000, 1000);

    let dec = AdaptiveDifficultyEngine::evaluate_difficulty(Some(&state), None, None);
    println!("Single Transfer Failure -> Practice State: {:?}, Difficulty Level: {}, Reason: {}",
        state.practice_state, dec.level, dec.reason
    );

    // Verify difficulty does NOT collapse to Level 1 on a single transfer error
    assert!(dec.level >= 3, "Difficulty collapsed too aggressively!");
    assert_eq!(state.practice_state, PracticeProgressionState::Transfer, "Practice state dropped prematurely!");
}
