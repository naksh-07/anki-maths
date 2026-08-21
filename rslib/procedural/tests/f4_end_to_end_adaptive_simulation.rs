// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use procedural::core::{AttemptId, Domain, SchemaId, SkillId, ErrorEventId};
use procedural::practice::{PracticeAttempt, ErrorEvent};
use procedural::diagnostics::ErrorCategory;
use procedural::problems::catalog::{SCHEMA_REASONING_SEATING, SCHEMA_SUCCESSIVE_PERCENTAGE};
use procedural::remediation::{RemediationContext, RemediationPolicy};
use procedural::service::ProceduralService;
use procedural::skills::signals::{IndependenceLevel, PracticeProgressionState};
use procedural::practice::{PracticeRequest, PracticeScope, PracticeObjective, RemediationPrecedence};
use procedural::scheduling::PracticeSessionObject;
use procedural::remediation::actions::{RemediationAction, RemediationActionKind};
use procedural::skills::SkillState;
use serde_json::json;
use chrono::Utc;
use std::str::FromStr;
use std::collections::HashMap;

pub struct SyntheticLearner {
    pub service: ProceduralService,
    pub attempt_counter: u64,
}

impl SyntheticLearner {
    pub fn new() -> Self {
        let mut service = ProceduralService::open_in_memory().unwrap();
        service.init_default_maths_catalog().unwrap();
        Self {
            service,
            attempt_counter: 0,
        }
    }

    pub fn simulate_attempt(
        &mut self,
        schema_id_str: &str,
        skill_id_str: &str,
        is_correct: bool,
        response_time_ms: u64,
        target_time_ms: u64,
        mistake_type_str: &str,
        variant_str: &str,
    ) -> Option<RemediationAction> {
        self.attempt_counter += 1;
        
        let schema_id = SchemaId::new(schema_id_str);
        let skill_id = SkillId::new(skill_id_str);
        
        let schema = self.service.resolve_schema(&schema_id).unwrap().unwrap();
        let custom_params = json!({});
        let instance = self.service.generate_problem(&schema.problem_family_id, self.attempt_counter as u64, &custom_params).unwrap();
        self.service.save_problem_instance(instance.clone()).unwrap();
        
        let attempt_id_str = format!("att-{}", self.attempt_counter);
        let attempt_id = AttemptId::new(attempt_id_str.clone());
        
        let score = if is_correct { 1.0 } else { 0.0 };
        
        let err_cat = match mistake_type_str {
            "silly_mistake" | "careless" => Some(ErrorCategory::Careless),
            "pattern_not_recognized" | "strategy" => Some(ErrorCategory::Strategy),
            "formula_or_concept_misapplied" | "concept_not_known" => Some(ErrorCategory::Concept),
            "slow" | "time" => Some(ErrorCategory::Time),
            "none" | "" => None,
            other => Some(ErrorCategory::DomainSpecific(other.to_string())),
        };

        let mut metadata = json!({
            "target_time_ms": target_time_ms,
            "variant": variant_str,
            "variant_category": "structural",
            "hints_used": 0,
            "attempt_count": 1,
        });
        
        if let Some(ref cat) = err_cat {
            metadata["error_category"] = json!(cat);
        }

        let attempt = PracticeAttempt::new(
            &attempt_id,
            &instance.id,
            &schema_id,
            &skill_id,
            json!("synthetic answer"),
            is_correct,
            score,
            response_time_ms,
        ).with_metadata(metadata);
        
        let mut errors = vec![];
        if let Some(ref cat) = err_cat {
            errors.push(ErrorEvent::new(
                ErrorEventId::new(format!("err-{}", self.attempt_counter)),
                &attempt_id,
                cat.as_str(),
                json!({}),
            ));
        }

        self.service.record_practice_attempt_with_variant(
            attempt,
            errors,
            Some(variant_str),
            target_time_ms
        ).unwrap();
        
        let domain = if schema_id_str.contains("reasoning") { Domain::Reasoning } else { Domain::Mathematics };
        let skill_state = self.service.store().get_skill_state(&skill_id).unwrap().unwrap();
        
        if !is_correct {
            let cat = err_cat.unwrap_or(ErrorCategory::Unknown);
            let queue_arc = self.service.remediation_queue();
            let queue_lock = queue_arc.lock().unwrap();
            let recurrence = queue_lock.get_recurrence_count(&skill_id, &cat) + 1;
            drop(queue_lock);
            
            let ctx = RemediationContext {
                skill_id: &skill_id,
                schema_id: &schema_id,
                domain,
                primary_error: cat,
                step_error: None,
                decision_point_correct: None,
                independence: IndependenceLevel::Independent,
                progression_state: skill_state.practice_state,
                recent_attempts: &skill_state.recent_attempts,
                source_attempt_id: &attempt_id,
                recurrence_count: recurrence,
                is_transfer_attempt: false,
            };
            
            let action = RemediationPolicy::evaluate(&ctx);
            self.service.enqueue_remediation_action(action.clone()).unwrap();
            Some(action)
        } else {
            if response_time_ms > target_time_ms {
                 let ctx = RemediationContext {
                    skill_id: &skill_id,
                    schema_id: &schema_id,
                    domain,
                    primary_error: ErrorCategory::Time,
                    step_error: None,
                    decision_point_correct: None,
                    independence: IndependenceLevel::Independent,
                    progression_state: skill_state.practice_state,
                    recent_attempts: &skill_state.recent_attempts,
                    source_attempt_id: &attempt_id,
                    recurrence_count: 1,
                    is_transfer_attempt: false,
                };
                let action = RemediationPolicy::evaluate(&ctx);
                self.service.enqueue_remediation_action(action.clone()).unwrap();
                Some(action)
            } else {
                None
            }
        }
    }
    
    pub fn get_skill_state(&self, skill_id_str: &str) -> Option<SkillState> {
        self.service.store().get_skill_state(&SkillId::new(skill_id_str)).unwrap()
    }
}

#[test]
fn test_phase10_longitudinal_profile_evolution() {
    let mut learner = SyntheticLearner::new();
    let skill = procedural::problems::catalog::SKILL_PERCENTAGE_SUCCESSIVE;
    let schema = SCHEMA_SUCCESSIVE_PERCENTAGE;
    
    println!("--- PROFILE EVOLUTION TEST ---");
    learner.simulate_attempt(schema, skill, false, 10000, 15000, "silly_mistake", "A");
    let s1 = learner.get_skill_state(skill).unwrap();
    println!("After Attempt 1 (Silly Mistake): Mastery={}, Total={}, Recents={}, Errors={:?}", s1.mastery, s1.total_attempts, s1.recent_attempts.len(), s1.error_counts);
    assert_eq!(s1.total_attempts, 1);
    
    learner.simulate_attempt(schema, skill, false, 15000, 15000, "pattern_not_recognized", "A");
    learner.simulate_attempt(schema, skill, false, 20000, 15000, "formula_or_concept_misapplied", "B");
    let s3 = learner.get_skill_state(skill).unwrap();
    println!("After Attempt 3 (Pattern + Concept): Mastery={}, Total={}, Errors={:?}", s3.mastery, s3.total_attempts, s3.error_counts);
    assert_eq!(s3.total_attempts, 3);
    
    learner.simulate_attempt(schema, skill, true, 25000, 15000, "slow", "B");
    learner.simulate_attempt(schema, skill, true, 22000, 15000, "slow", "C");
    let s5 = learner.get_skill_state(skill).unwrap();
    println!("After Attempt 5 (2x Slow Correct): Mastery={}, Latency SMA={}, PracticeState={:?}", s5.mastery, s5.latency_stats.moving_average_ms, s5.practice_state);
    
    for i in 6..=10 {
        learner.simulate_attempt(schema, skill, true, 12000, 15000, "none", "D");
    }
    
    let s10 = learner.get_skill_state(skill).unwrap();
    println!("After Attempt 10: Mastery={}, Latency SMA={}, Consecutive={}", s10.mastery, s10.latency_stats.moving_average_ms, s10.consecutive_successes);
    assert!(s10.mastery > s3.mastery, "Mastery must meaningfully evolve");
}

#[test]
fn test_phase10_history_must_influence_future_decisions() {
    println!("--- HISTORY INFLUENCE TEST ---");
    let schema = SCHEMA_SUCCESSIVE_PERCENTAGE;
    let skill = procedural::problems::catalog::SKILL_PERCENTAGE_SUCCESSIVE;
    
    let mut learner_a = SyntheticLearner::new();
    for _ in 0..5 {
        learner_a.simulate_attempt(schema, skill, false, 15000, 15000, "concept_not_known", "A");
    }
    
    let mut learner_b = SyntheticLearner::new();
    for _ in 0..5 {
        learner_b.simulate_attempt(schema, skill, true, 10000, 15000, "none", "A");
    }
    
    let action_a = learner_a.simulate_attempt(schema, skill, false, 15000, 15000, "concept_not_known", "B").unwrap();
    let action_b = learner_b.simulate_attempt(schema, skill, false, 15000, 15000, "concept_not_known", "B").unwrap();
    
    println!("Learner A Action: {:?}", action_a.kind);
    println!("Learner B Action: {:?}", action_b.kind);
    assert_ne!(action_a.kind, action_b.kind, "Same current mistake but different histories MUST yield different remediation");
}

#[test]
fn test_phase10_speed_fluency_history() {
    println!("--- SPEED / FLUENCY HISTORY TEST ---");
    let schema = SCHEMA_SUCCESSIVE_PERCENTAGE;
    let skill = procedural::problems::catalog::SKILL_PERCENTAGE_SUCCESSIVE;
    
    let mut learner_a = SyntheticLearner::new();
    let mut learner_b = SyntheticLearner::new();
    
    for _ in 0..5 {
        learner_a.simulate_attempt(schema, skill, true, 30000, 15000, "none", "A");
    }
    for _ in 0..5 {
        learner_b.simulate_attempt(schema, skill, true, 8000, 15000, "none", "A");
    }
    
    let s_a = learner_a.get_skill_state(skill).unwrap();
    let s_b = learner_b.get_skill_state(skill).unwrap();
    
    println!("Learner A Latency SMA: {}", s_a.latency_stats.moving_average_ms);
    println!("Learner B Latency SMA: {}", s_b.latency_stats.moving_average_ms);
    assert!(s_a.latency_stats.moving_average_ms > s_b.latency_stats.moving_average_ms);
}

#[test]
fn test_phase10_structural_diversity() {
    println!("--- STRUCTURAL DIVERSITY TEST ---");
    let schema = SCHEMA_SUCCESSIVE_PERCENTAGE;
    let skill = procedural::problems::catalog::SKILL_PERCENTAGE_SUCCESSIVE;
    
    let mut learner_same = SyntheticLearner::new();
    for _ in 0..5 {
        learner_same.simulate_attempt(schema, skill, true, 10000, 15000, "none", "Variant1");
    }
    
    let mut learner_diverse = SyntheticLearner::new();
    for i in 0..5 {
        learner_diverse.simulate_attempt(schema, skill, true, 10000, 15000, "none", &format!("Variant{}", i));
    }
    
    let s_same = learner_same.get_skill_state(skill).unwrap();
    let s_div = learner_diverse.get_skill_state(skill).unwrap();
    
    println!("Same forms seen: {}", s_same.structural_forms_seen.len());
    println!("Diverse forms seen: {}", s_div.structural_forms_seen.len());
    
    assert_eq!(s_same.structural_forms_seen.len(), 1);
    assert_eq!(s_div.structural_forms_seen.len(), 5);
}

#[test]
fn test_phase10_multi_skill_isolation() {
    println!("--- MULTI SKILL ISOLATION ---");
    let mut learner = SyntheticLearner::new();
    let schema_a = procedural::problems::catalog::SCHEMA_SUCCESSIVE_PERCENTAGE;
    let skill_a = procedural::problems::catalog::SKILL_PERCENTAGE_SUCCESSIVE;
    
    // We need another real schema from the catalog.
    let schema_b = procedural::problems::catalog::SCHEMA_LINEAR_EQUATIONS;
    let skill_b = procedural::problems::catalog::SKILL_LINEAR_EQUATIONS;
    
    for i in 0..15 {
        learner.simulate_attempt(schema_a, skill_a, true, 8000, 15000, "none", &format!("V{}", i));
    }
    for _ in 0..3 {
        learner.simulate_attempt(schema_b, skill_b, false, 15000, 15000, "concept_not_known", "V1");
    }
    
    let sa = learner.get_skill_state(skill_a).unwrap();
    let sb = learner.get_skill_state(skill_b).unwrap();
    
    assert!(sa.mastery > 0.8, "A should be strong");
    assert!(sb.mastery < 0.5, "B should be weak");
    
    let q_len = learner.service.remediation_queue_len();
    assert_eq!(q_len, 1, "Queue should only have B's single deduplicated remediation action");
}

#[test]
fn test_phase10_remediation_evolution_and_resolution() {
    println!("--- REMEDIATION EVOLUTION & RESOLUTION ---");
    let mut learner = SyntheticLearner::new();
    let schema = SCHEMA_SUCCESSIVE_PERCENTAGE;
    let skill = procedural::problems::catalog::SKILL_PERCENTAGE_SUCCESSIVE;
    
    let a1 = learner.simulate_attempt(schema, skill, false, 15000, 15000, "concept_not_known", "V1").unwrap();
    println!("Action 1: {:?}", a1.kind);
    let a2 = learner.simulate_attempt(schema, skill, false, 15000, 15000, "concept_not_known", "V2").unwrap();
    println!("Action 2: {:?}", a2.kind);
    assert_ne!(a1.kind, a2.kind, "Remediation should escalate");
    
    learner.simulate_attempt(schema, skill, true, 12000, 15000, "none", "V3");
    learner.simulate_attempt(schema, skill, true, 9000, 15000, "none", "V4");
    
    let state = learner.get_skill_state(skill).unwrap();
    println!("Final consecutive successes: {}", state.consecutive_successes);
    assert_eq!(state.consecutive_successes, 2);
}

#[test]
fn test_phase10_learning_effect() {
    println!("--- LEARNING EFFECT TEST ---");
    let mut learner = SyntheticLearner::new();
    let schema = SCHEMA_SUCCESSIVE_PERCENTAGE;
    let skill = procedural::problems::catalog::SKILL_PERCENTAGE_SUCCESSIVE;
    
    learner.simulate_attempt(schema, skill, false, 25000, 15000, "pattern_not_recognized", "V1");
    let s_before = learner.get_skill_state(skill).unwrap();
    
    learner.simulate_attempt(schema, skill, true, 10000, 15000, "none", "V2");
    let s_after = learner.get_skill_state(skill).unwrap();
    
    println!("Before: Mastery={}, Consecutive={}", s_before.mastery, s_before.consecutive_successes);
    println!("After: Mastery={}, Consecutive={}, Structural Forms={}", s_after.mastery, s_after.consecutive_successes, s_after.structural_forms_seen.len());
    
    assert!(s_after.mastery > s_before.mastery);
    assert_eq!(s_after.structural_forms_seen.len(), 1);
}

#[test]
fn test_phase10_long_run_stress() {
    println!("--- LONG RUN SIMULATION (200 REVIEWS) ---");
    let mut learner = SyntheticLearner::new();
    let schema = SCHEMA_SUCCESSIVE_PERCENTAGE;
    let skill = procedural::problems::catalog::SKILL_PERCENTAGE_SUCCESSIVE;
    
    for i in 0..200 {
        let is_correct = i % 4 != 0; // 75% correct
        let mistake = if is_correct { "none" } else { "careless" };
        learner.simulate_attempt(schema, skill, is_correct, 12000, 15000, mistake, &format!("V{}", i % 10));
    }
    
    let state = learner.get_skill_state(skill).unwrap();
    println!("After 200 reviews: Mastery={}, Total Attempts={}, Errors={:?}", state.mastery, state.total_attempts, state.error_counts);
    assert_eq!(state.total_attempts, 200);
}
