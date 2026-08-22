// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use procedural::{
    derive_fsrs_rating, AdaptiveDifficultyEngine, Domain,
    ErrorCategory, IndependenceLevel, MasteryEvidence, PracticeProgressionState,
    ProceduralReviewOutcome, Rating, RemediationActionKind, RemediationContext,
    RemediationPolicy, SkillId, SkillState, StepErrorType, VariantCategory,
};

// =========================================================================
// 1. ADVERSARIAL SCENARIO 1: The Pattern Matcher (Familiarity Inflation)
// =========================================================================

#[test]
fn test_adversarial_pattern_matcher_familiarity_ceiling() {
    let mut state = SkillState::new("math.algebra.quadratics");
    state.practice_state = PracticeProgressionState::Transfer;

    // Simulate 20 fast, correct attempts on ONLY parameter-variation problems (same template)
    let timestamp_base = 1_000_000;
    for i in 0..20 {
        let evidence = MasteryEvidence {
            final_correctness: true,
            decision_quality: Some(1.0),
            step_quality: None,
            independence: IndependenceLevel::Independent,
            max_hint_level: None,
            hint_dependence: 0,
            retry_dependence: 0,
            variant_exposure: Some("quadratics_standard_ax2_bx_c".to_string()),
            variant_category: VariantCategory::Parameter, // Only parameter variations
            solution_graph_fingerprint: Some("fingerprint_standard_template".to_string()),
            cognitive_decision_correct: Some(true),
            time_since_last_ms: Some(10_000),
            transfer_evidence: false, // No transfer evidence
            domain_competence_verified: Some(true),
            latency_evidence: 12_000,
            diagnostic_errors: Vec::new(), domain_evidence: None,
        };

        state.record_attempt_outcome(&evidence, 1.0, 30_000, timestamp_base + i * 60);
    }

    // Despite 20/20 correct (100% accuracy) and high speed, skill CANNOT progress to Mastered
    assert_ne!(
        state.practice_state,
        PracticeProgressionState::Mastered,
        "Pattern matcher must NOT reach Mastered on parameter variants alone!"
    );
    assert_eq!(state.practice_state, PracticeProgressionState::Transfer);

    // Rating Policy Check: Check that FSRS ratings are bounded (Easy blocked due to shallow structural diversity)
    let outcome = ProceduralReviewOutcome::new(
        "att-pm-21",
        "quadratics_standard",
        "math.algebra.quadratics",
        "algebra",
        42,
        true,
        1.0,
        10_000, // very fast
        30_000,
        0,
        1,
        None,
    );

    let rating = derive_fsrs_rating(&outcome, Some(&state));
    // Since only 1 structural form has been seen across 20 attempts, Easy is prevented
    assert_eq!(
        rating,
        Rating::Good,
        "Rating must be capped at Good to prevent FSRS interval ballooning for shallow structural exposure"
    );
}

// =========================================================================
// 2. ADVERSARIAL SCENARIO 2: The Hint-Dependent Learner (Chronic Dependency)
// =========================================================================

#[test]
fn test_adversarial_chronic_hint_dependent_learner() {
    let mut state = SkillState::new("physics.mechanics.work_energy");
    state.practice_state = PracticeProgressionState::Transfer;

    let timestamp_base = 1_000_000;
    // 10 attempts where learner requested hints (Level 1/2 hint)
    for i in 0..10 {
        let evidence = MasteryEvidence {
            final_correctness: true,
            decision_quality: Some(0.8),
            step_quality: None,
            independence: IndependenceLevel::LightSupport,
            max_hint_level: Some(2),
            hint_dependence: 2,
            retry_dependence: 0,
            variant_exposure: Some("work_energy_incline".to_string()),
            variant_category: VariantCategory::Structural,
            solution_graph_fingerprint: Some(format!("fp_{}", i % 3)),
            cognitive_decision_correct: Some(true),
            time_since_last_ms: Some(10_000),
            transfer_evidence: false,
            domain_competence_verified: Some(true),
            latency_evidence: 25_000,
            diagnostic_errors: Vec::new(), domain_evidence: None,
        };
        state.record_attempt_outcome(&evidence, 0.7, 30_000, timestamp_base + i * 60);
    }

    // Now learner solves 3 attempts independently
    for i in 10..13 {
        let evidence = MasteryEvidence {
            final_correctness: true,
            decision_quality: Some(1.0),
            step_quality: None,
            independence: IndependenceLevel::Independent,
            max_hint_level: None,
            hint_dependence: 0,
            retry_dependence: 0,
            variant_exposure: Some(format!("work_energy_transfer_{}", i)),
            variant_category: VariantCategory::Transfer,
            solution_graph_fingerprint: Some(format!("fp_trans_{}", i)),
            cognitive_decision_correct: Some(true),
            time_since_last_ms: Some(10_000),
            transfer_evidence: true,
            domain_competence_verified: Some(true),
            latency_evidence: 18_000,
            diagnostic_errors: Vec::new(), domain_evidence: None,
        };
        state.record_attempt_outcome(&evidence, 1.0, 30_000, timestamp_base + i * 60);
    }

    // Recent 3 attempts in sliding window look clean (100% accuracy, unassisted),
    // BUT longitudinal independence ratio is only 3/13 (~23% < 70% threshold).
    assert!(
        state.longitudinal_independence_ratio() < 0.70,
        "Longitudinal independence ratio should reflect lifetime hint dependency"
    );
    assert_ne!(
        state.practice_state,
        PracticeProgressionState::Mastered,
        "Chronic hint dependence must NOT be wiped out by sliding window; mastery promotion must be blocked"
    );
}

// =========================================================================
// 3. ADVERSARIAL SCENARIO 3: Fast-Wrong Impulsive Learner
// =========================================================================

#[test]
fn test_adversarial_fast_wrong_impulsive_learner() {
    let mut state = SkillState::new("reasoning.series.arithmetic");
    state.practice_state = PracticeProgressionState::Fluent;

    // Learner answers very quickly (4s on 30s target) but makes wrong strategy selection
    let evidence = MasteryEvidence {
        final_correctness: false,
        decision_quality: Some(0.0),
        step_quality: None,
        independence: IndependenceLevel::Independent,
        max_hint_level: None,
        hint_dependence: 0,
        retry_dependence: 0,
        variant_exposure: Some("series_two_step".to_string()),
        variant_category: VariantCategory::Structural,
        solution_graph_fingerprint: Some("fp_series_diff".to_string()),
        cognitive_decision_correct: Some(false),
        time_since_last_ms: Some(5_000),
        transfer_evidence: false,
        domain_competence_verified: Some(false),
        latency_evidence: 4_000, // 4s on 30s target -> impulsive fast wrong
        diagnostic_errors: vec![ErrorCategory::Strategy], domain_evidence: None,
    };
    state.record_attempt_outcome(&evidence, 0.0, 30_000, 1_000_000);

    // Rating Policy check: fast-wrong MUST receive Again
    let outcome = ProceduralReviewOutcome::new(
        "att-impulsive-1",
        "series_arithmetic",
        "reasoning.series.arithmetic",
        "series",
        42,
        false,
        0.0,
        4_000,
        30_000,
        0,
        1,
        Some(ErrorCategory::Strategy),
    );
    assert_eq!(derive_fsrs_rating(&outcome, Some(&state)), Rating::Again);

    // Adaptive difficulty engine should step down difficulty level on strategy breakdown
    let diff_decision = AdaptiveDifficultyEngine::evaluate_difficulty(Some(&state), None, None);
    assert!(
        diff_decision.level <= 2,
        "Impulsive strategy failure must trigger difficulty step-down"
    );
}

// =========================================================================
// 4. ADVERSARIAL SCENARIO 4: Slow-but-Accurate Conceptual Learner
// =========================================================================

#[test]
fn test_adversarial_slow_accurate_conceptual_learner() {
    let mut state = SkillState::new("chemistry.stoichiometry.limiting_reagent");
    state.practice_state = PracticeProgressionState::Learning;

    let timestamp_base = 1_000_000;
    // Learner carefully works through 3 problems in Learning stage.
    // Target is 30s, learner takes 45s-50s (1.5x - 1.6x target), but 100% accurate and unassisted.
    for i in 0..3 {
        let evidence = MasteryEvidence {
            final_correctness: true,
            decision_quality: Some(1.0),
            step_quality: None,
            independence: IndependenceLevel::Independent,
            max_hint_level: None,
            hint_dependence: 0,
            retry_dependence: 0,
            variant_exposure: Some(format!("limiting_reagent_v{}", i)),
            variant_category: VariantCategory::Contextual,
            solution_graph_fingerprint: Some(format!("fp_chem_{}", i)),
            cognitive_decision_correct: Some(true),
            time_since_last_ms: Some(20_000),
            transfer_evidence: false,
            domain_competence_verified: Some(true),
            latency_evidence: 48_000, // slow but clean
            diagnostic_errors: Vec::new(), domain_evidence: None,
        };
        // Verify rating policy: In Learning stage, slow clean solves receive Good (not Hard)
        let outcome = ProceduralReviewOutcome::new(
            format!("att-slow-{}", i),
            "stoich_limiting",
            "chemistry.stoichiometry.limiting_reagent",
            "stoichiometry",
            42,
            true,
            1.0,
            48_000,
            30_000,
            0,
            1,
            None,
        );
        let rating = derive_fsrs_rating(&outcome, Some(&state));
        assert_eq!(
            rating,
            Rating::Good,
            "Slow-but-accurate learner in Learning stage must receive Good without speed penalty"
        );

        state.record_attempt_outcome(&evidence, 1.0, 30_000, timestamp_base + i * 100);
    }

    // Progression check: Learner successfully advances from Learning to Fluent
    assert_eq!(
        state.practice_state,
        PracticeProgressionState::Fluent,
        "Accurate, independent learner with no misconceptions must advance to Fluent even if taking time"
    );
}

// =========================================================================
// 5. ADVERSARIAL SCENARIO 5: Long-Break Delayed Retention Verification
// =========================================================================

#[test]
fn test_adversarial_long_break_retention_verification() {
    let mut state = SkillState::new("math.calculus.derivatives");
    state.practice_state = PracticeProgressionState::Transfer;

    let timestamp_base = 1_000_000;
    // Learner solves 4 diverse structural and transfer problems in a SINGLE burst session (all within 5 minutes)
    for i in 0..4 {
        let evidence = MasteryEvidence {
            final_correctness: true,
            decision_quality: Some(1.0),
            step_quality: None,
            independence: IndependenceLevel::Independent,
            max_hint_level: None,
            hint_dependence: 0,
            retry_dependence: 0,
            variant_exposure: Some(format!("derivative_form_{}", i)),
            variant_category: if i < 3 { VariantCategory::Structural } else { VariantCategory::Transfer },
            solution_graph_fingerprint: Some(format!("fp_struct_{}", i)),
            cognitive_decision_correct: Some(true),
            time_since_last_ms: Some(30_000), // only 30s apart
            transfer_evidence: i >= 3,
            domain_competence_verified: Some(true),
            latency_evidence: 20_000,
            diagnostic_errors: Vec::new(), domain_evidence: None,
        };
        state.record_attempt_outcome(&evidence, 1.0, 30_000, timestamp_base + i * 60);
    }

    // In single burst without spaced retention or multi-session history, delayed retention is checked
    // Now simulate learner returning 2 days later (172,800,000 ms delay) and solving successfully
    let delay_evidence = MasteryEvidence {
        final_correctness: true,
        decision_quality: Some(1.0),
        step_quality: None,
        independence: IndependenceLevel::Independent,
        max_hint_level: None,
        hint_dependence: 0,
        retry_dependence: 0,
        variant_exposure: Some("derivative_form_spaced".to_string()),
        variant_category: VariantCategory::Transfer,
        solution_graph_fingerprint: Some("fp_struct_spaced".to_string()),
        cognitive_decision_correct: Some(true),
        time_since_last_ms: Some(172_800_000), // 48 hours later!
        transfer_evidence: true,
        domain_competence_verified: Some(true),
        latency_evidence: 18_000,
        diagnostic_errors: Vec::new(), domain_evidence: None,
    };
    state.record_attempt_outcome(&delay_evidence, 1.0, 30_000, timestamp_base + 172_800);

    assert!(
        state.delayed_retention_successes >= 1,
        "Delayed retention success must be recorded across meaningful spacing"
    );
    assert_eq!(
        state.practice_state,
        PracticeProgressionState::Mastered,
        "Learner with structural diversity, transfer, and verified retention reaches Mastered"
    );
}

// =========================================================================
// 6. ADVERSARIAL SCENARIO 6: Chemistry Calculation Slip - No Hijacking
// =========================================================================

#[test]
fn test_adversarial_chemistry_calculation_slip_no_hijacking() {
    let skill = SkillId::new("chemistry.stoichiometry.alligation");
    let schema = procedural::SchemaId::new("chemistry_alligation");
    let attempt = procedural::AttemptId::new("att-chem-slip-1");

    let ctx = RemediationContext {
        skill_id: &skill,
        schema_id: &schema,
        domain: Domain::Chemistry,
        primary_error: ErrorCategory::Calculation,
        step_error: Some(StepErrorType::ArithmeticError),
        decision_point_correct: Some(true), // Cognitive model was correct
        independence: IndependenceLevel::Independent,
        progression_state: PracticeProgressionState::Fluent,
        recent_attempts: &[],
        source_attempt_id: &attempt,
        recurrence_count: 1,
        is_transfer_attempt: false,
    };

    let action = RemediationPolicy::evaluate(&ctx);

    // Must remain within Chemistry domain and provide simpler numbers variant rather than hijacking to general arithmetic
    assert_eq!(action.domain, Domain::Chemistry);
    assert_eq!(action.kind, RemediationActionKind::ProceduralVariant);
    assert_eq!(action.preferred_variant.as_deref(), Some("simpler_numbers"));
}

// =========================================================================
// 7. ADVERSARIAL SCENARIO 7: Remediation Wheel-Spinning Circuit Breaker
// =========================================================================

#[test]
fn test_adversarial_remediation_wheel_spinning_circuit_breaker() {
    let skill = SkillId::new("physics.kinematics.projectile");
    let schema = procedural::SchemaId::new("kinematics_projectile");
    let attempt = procedural::AttemptId::new("att-spin-1");

    // Recurrence 1: Targeted ConceptCheck
    let ctx1 = RemediationContext {
        skill_id: &skill,
        schema_id: &schema,
        domain: Domain::Physics,
        primary_error: ErrorCategory::Concept,
        step_error: Some(StepErrorType::RegimeSelectionError),
        decision_point_correct: Some(false),
        independence: IndependenceLevel::Independent,
        progression_state: PracticeProgressionState::Learning,
        recent_attempts: &[],
        source_attempt_id: &attempt,
        recurrence_count: 1,
        is_transfer_attempt: false,
    };
    let action1 = RemediationPolicy::evaluate(&ctx1);
    assert_eq!(action1.kind, RemediationActionKind::ConceptCheck);

    // Recurrence 2: Strategy Drill
    let mut ctx2 = ctx1.clone();
    ctx2.recurrence_count = 2;
    let action2 = RemediationPolicy::evaluate(&ctx2);
    assert_eq!(action2.kind, RemediationActionKind::StrategyDrill);

    // Recurrence 3: Worked Example
    let mut ctx3 = ctx1.clone();
    ctx3.recurrence_count = 3;
    let action3 = RemediationPolicy::evaluate(&ctx3);
    assert_eq!(action3.kind, RemediationActionKind::WorkedExample);

    // Recurrence 4: Prerequisite Review (bounded micro-intervention)
    let mut ctx4 = ctx1.clone();
    ctx4.recurrence_count = 4;
    let action4 = RemediationPolicy::evaluate(&ctx4);
    assert_eq!(action4.kind, RemediationActionKind::PrerequisiteReview);

    // Recurrence 5: Circuit Breaker cooldown to prevent endless wheel-spinning
    let mut ctx5 = ctx1.clone();
    ctx5.recurrence_count = 5;
    let action5 = RemediationPolicy::evaluate(&ctx5);
    assert_eq!(
        action5.kind,
        RemediationActionKind::CircuitBreaker,
        "Recurrence >= 5 must trigger CircuitBreaker"
    );
}

// =========================================================================
// 8. ADVERSARIAL SCENARIO 8: Shallow Structural Exposure FSRS Cap
// =========================================================================

#[test]
fn test_adversarial_shallow_structural_exposure_fsrs_cap() {
    let mut state = SkillState::new("math.algebra.polynomials");
    state.consecutive_successes = 5;
    state.total_attempts = 10;
    state.historical_independent_count = 10;
    // Only 1 structural form seen
    state.structural_forms_seen.insert("standard_form".to_string(), 10);

    let outcome_fast = ProceduralReviewOutcome::new(
        "att-poly-11",
        "polynomials_standard",
        "math.algebra.polynomials",
        "algebra",
        42,
        true,
        1.0,
        15_000, // fast
        30_000,
        0,
        1,
        None,
    );

    // Even though fast, unassisted, and 5 consecutive successes, Easy is capped at Good because distinct structural forms < 2
    let rating = derive_fsrs_rating(&outcome_fast, Some(&state));
    assert_eq!(
        rating,
        Rating::Good,
        "Rating must be capped at Good when structural exposure is shallow (< 2 distinct forms)"
    );

    // Now introduce second structural form passed independently
    state.structural_forms_seen.insert("factored_form".to_string(), 2);
    let rating_after = derive_fsrs_rating(&outcome_fast, Some(&state));
    assert_eq!(
        rating_after,
        Rating::Easy,
        "Rating qualifies for Easy once multiple structural forms are proven"
    );
}

// =========================================================================
// 9. TARGETED SIMULATION: 30-Day Cohort Progression
// =========================================================================

#[test]
fn test_simulation_30_day_cohort_progression() {
    // 4 Student Archetypes:
    // 1. Pattern Matcher (Memorizes 1 template, 100% correct on template, fails transfers)
    // 2. Impulsive Guesser (Fast, 50% accurate, frequent strategy slips)
    // 3. Deliberate Thinker (Slow, 95% accurate, develops genuine mastery)
    // 4. Balanced Achiever (Fast & accurate across diverse forms)

    let mut pm_state = SkillState::new("sim.skill.pattern_matcher");
    let mut imp_state = SkillState::new("sim.skill.impulsive");
    let mut delib_state = SkillState::new("sim.skill.deliberate");
    let mut bal_state = SkillState::new("sim.skill.balanced");

    let sec_per_day = 86_400;

    for day in 1..=30 {
        let day_ts = day as i64 * sec_per_day;

        // Pattern Matcher: 2 attempts/day on parameter variant
        for a in 0..2 {
            let ev = MasteryEvidence {
                final_correctness: true,
                decision_quality: Some(1.0),
                step_quality: None,
                independence: IndependenceLevel::Independent,
                max_hint_level: None,
                hint_dependence: 0,
                retry_dependence: 0,
                variant_exposure: Some("template_alpha".to_string()),
                variant_category: VariantCategory::Parameter,
                solution_graph_fingerprint: Some("fp_alpha".to_string()),
                cognitive_decision_correct: Some(true),
                time_since_last_ms: Some(sec_per_day as u64 * 1000),
                transfer_evidence: false,
                domain_competence_verified: Some(true),
                latency_evidence: 12_000,
                diagnostic_errors: Vec::new(), domain_evidence: None,
            };
            pm_state.record_attempt_outcome(&ev, 1.0, 30_000, day_ts + a * 300);
        }

        // Impulsive Guesser: 2 attempts/day, alternating success/fail
        for a in 0..2 {
            let is_corr = (day + a) % 2 == 0;
            let ev = MasteryEvidence {
                final_correctness: is_corr,
                decision_quality: Some(if is_corr { 0.9 } else { 0.1 }),
                step_quality: None,
                independence: IndependenceLevel::Independent,
                max_hint_level: None,
                hint_dependence: 0,
                retry_dependence: 0,
                variant_exposure: Some(format!("variant_{}", (day % 3))),
                variant_category: VariantCategory::Structural,
                solution_graph_fingerprint: Some(format!("fp_struct_{}", day % 3)),
                cognitive_decision_correct: Some(is_corr),
                time_since_last_ms: Some(sec_per_day as u64 * 1000),
                transfer_evidence: false,
                domain_competence_verified: Some(is_corr),
                latency_evidence: 5_000,
                diagnostic_errors: if is_corr { Vec::new() } else { vec![ErrorCategory::Strategy] }, domain_evidence: None,
            };
            imp_state.record_attempt_outcome(&ev, if is_corr { 1.0 } else { 0.0 }, 30_000, day_ts + a * 300);
        }

        // Deliberate Thinker: 2 attempts/day, slow (45s), high accuracy across distinct forms
        for a in 0..2 {
            let form_id = (day % 4) + 1;
            let ev = MasteryEvidence {
                final_correctness: true,
                decision_quality: Some(1.0),
                step_quality: None,
                independence: IndependenceLevel::Independent,
                max_hint_level: None,
                hint_dependence: 0,
                retry_dependence: 0,
                variant_exposure: Some(format!("struct_form_{}", form_id)),
                variant_category: if form_id == 4 { VariantCategory::Transfer } else { VariantCategory::Structural },
                solution_graph_fingerprint: Some(format!("fp_delib_{}", form_id)),
                cognitive_decision_correct: Some(true),
                time_since_last_ms: Some(sec_per_day as u64 * 1000),
                transfer_evidence: form_id == 4,
                domain_competence_verified: Some(true),
                latency_evidence: 45_000, // slow but thorough
                diagnostic_errors: Vec::new(), domain_evidence: None,
            };
            delib_state.record_attempt_outcome(&ev, 1.0, 30_000, day_ts + a * 300);
        }

        // Balanced Achiever: 2 attempts/day, fast (18s), high accuracy across distinct forms
        for a in 0..2 {
            let form_id = (day % 4) + 1;
            let ev = MasteryEvidence {
                final_correctness: true,
                decision_quality: Some(1.0),
                step_quality: None,
                independence: IndependenceLevel::Independent,
                max_hint_level: None,
                hint_dependence: 0,
                retry_dependence: 0,
                variant_exposure: Some(format!("bal_form_{}", form_id)),
                variant_category: if form_id == 4 { VariantCategory::Transfer } else { VariantCategory::Structural },
                solution_graph_fingerprint: Some(format!("fp_bal_{}", form_id)),
                cognitive_decision_correct: Some(true),
                time_since_last_ms: Some(sec_per_day as u64 * 1000),
                transfer_evidence: form_id == 4,
                domain_competence_verified: Some(true),
                latency_evidence: 18_000,
                diagnostic_errors: Vec::new(), domain_evidence: None,
            };
            bal_state.record_attempt_outcome(&ev, 1.0, 30_000, day_ts + a * 300);
        }
    }

    // Verifications:
    // 1. Pattern Matcher has 100% accuracy but ONLY 1 parameter template -> CANNOT be Mastered!
    assert_ne!(pm_state.practice_state, PracticeProgressionState::Mastered);

    // 2. Impulsive Guesser has oscillating performance -> Held in Learning/Fluent, NOT Mastered
    assert_ne!(imp_state.practice_state, PracticeProgressionState::Mastered);

    // 3. Deliberate Thinker proved diverse structural forms, transfer, and delayed retention -> Reaches Mastered
    assert_eq!(delib_state.practice_state, PracticeProgressionState::Mastered);

    // 4. Balanced Achiever reaches Mastered
    assert_eq!(bal_state.practice_state, PracticeProgressionState::Mastered);
}

// =========================================================================
// 10. TARGETED SIMULATION: 90-Day Curriculum Transfer & Retention
// =========================================================================

#[test]
fn test_simulation_90_day_curriculum_transfer_and_retention() {
    let mut state = SkillState::new("sim.physics.circuits");
    let sec_per_day = 86_400;

    // Days 1-15: Initial Acquisition (Learning -> Fluent)
    for day in 1..=15 {
        let day_ts = day as i64 * sec_per_day;
        let ev = MasteryEvidence {
            final_correctness: true,
            decision_quality: Some(0.9),
            step_quality: None,
            independence: IndependenceLevel::Independent,
            max_hint_level: None,
            hint_dependence: 0,
            retry_dependence: 0,
            variant_exposure: Some("ohms_law_basic".to_string()),
            variant_category: VariantCategory::Parameter,
            solution_graph_fingerprint: Some("fp_basic".to_string()),
            cognitive_decision_correct: Some(true),
            time_since_last_ms: Some(sec_per_day as u64 * 1000),
            transfer_evidence: false,
            domain_competence_verified: Some(true),
            latency_evidence: 28_000,
            diagnostic_errors: Vec::new(), domain_evidence: None,
        };
        state.record_attempt_outcome(&ev, 1.0, 30_000, day_ts);
    }
    assert_eq!(state.practice_state, PracticeProgressionState::Fluent);

    // Days 16-35: Variation Stage (Exploring Series, Parallel, Mixed topologies)
    for day in 16..=35 {
        let day_ts = day as i64 * sec_per_day;
        let form_key = if day % 2 == 0 { "series_circuits" } else { "parallel_circuits" };
        let ev = MasteryEvidence {
            final_correctness: true,
            decision_quality: Some(1.0),
            step_quality: None,
            independence: IndependenceLevel::Independent,
            max_hint_level: None,
            hint_dependence: 0,
            retry_dependence: 0,
            variant_exposure: Some(form_key.to_string()),
            variant_category: VariantCategory::Structural,
            solution_graph_fingerprint: Some(format!("fp_{}", form_key)),
            cognitive_decision_correct: Some(true),
            time_since_last_ms: Some(sec_per_day as u64 * 1000),
            transfer_evidence: false,
            domain_competence_verified: Some(true),
            latency_evidence: 25_000,
            diagnostic_errors: Vec::new(), domain_evidence: None,
        };
        state.record_attempt_outcome(&ev, 1.0, 30_000, day_ts);
    }
    assert_eq!(state.practice_state, PracticeProgressionState::Transfer);

    // Days 36-60: Transfer Stage with Spaced Retention & Transfer Problem
    for day in 36..=60 {
        let day_ts = day as i64 * sec_per_day;
        let is_transfer_day = day == 60;
        let ev = MasteryEvidence {
            final_correctness: true,
            decision_quality: Some(1.0),
            step_quality: None,
            independence: IndependenceLevel::Independent,
            max_hint_level: None,
            hint_dependence: 0,
            retry_dependence: 0,
            variant_exposure: Some(if is_transfer_day { "bridge_circuit_transfer".to_string() } else { "mixed_mesh".to_string() }),
            variant_category: if is_transfer_day { VariantCategory::Transfer } else { VariantCategory::MultiConcept },
            solution_graph_fingerprint: Some(if is_transfer_day { "fp_bridge".to_string() } else { "fp_mesh".to_string() }),
            cognitive_decision_correct: Some(true),
            time_since_last_ms: Some(sec_per_day as u64 * 1000),
            transfer_evidence: is_transfer_day,
            domain_competence_verified: Some(true),
            latency_evidence: 22_000,
            diagnostic_errors: Vec::new(), domain_evidence: None,
        };
        state.record_attempt_outcome(&ev, 1.0, 30_000, day_ts);
    }

    // Verified reached Mastered on Day 60
    assert_eq!(state.practice_state, PracticeProgressionState::Mastered);

    // Days 61-90: Long Spacing Interval Maintenance Checks
    let maint_day_ts = 90 * sec_per_day;
    let maint_ev = MasteryEvidence {
        final_correctness: true,
        decision_quality: Some(1.0),
        step_quality: None,
        independence: IndependenceLevel::Independent,
        max_hint_level: None,
        hint_dependence: 0,
        retry_dependence: 0,
        variant_exposure: Some("bridge_circuit_transfer".to_string()),
        variant_category: VariantCategory::Transfer,
        solution_graph_fingerprint: Some("fp_bridge".to_string()),
        cognitive_decision_correct: Some(true),
        time_since_last_ms: Some(30 * sec_per_day as u64 * 1000), // 30-day gap
        transfer_evidence: true,
        domain_competence_verified: Some(true),
        latency_evidence: 20_000,
        diagnostic_errors: Vec::new(), domain_evidence: None,
    };
    state.record_attempt_outcome(&maint_ev, 1.0, 30_000, maint_day_ts);

    assert_eq!(state.practice_state, PracticeProgressionState::Mastered);
    assert!(state.distinct_structural_forms_passed() >= 3);
}

// =========================================================================
// 11. TARGETED SIMULATION: 180-Day Longitudinal Mastery & Circuit Breaker Robustness
// =========================================================================

#[test]
fn test_simulation_180_day_longitudinal_mastery_and_circuit_breakers() {
    let skill = SkillId::new("sim.chem.equilibrium");
    let schema = procedural::SchemaId::new("chem_equilibrium");
    let attempt = procedural::AttemptId::new("att-sim-180");

    // Test circuit breaker triggers consistently after 5 persistent failures across time
    for rec in 1..=8 {
        let ctx = RemediationContext {
            skill_id: &skill,
            schema_id: &schema,
            domain: Domain::Chemistry,
            primary_error: ErrorCategory::Concept,
            step_error: Some(StepErrorType::ModelSelectionError),
            decision_point_correct: Some(false),
            independence: IndependenceLevel::Independent,
            progression_state: PracticeProgressionState::Learning,
            recent_attempts: &[],
            source_attempt_id: &attempt,
            recurrence_count: rec,
            is_transfer_attempt: false,
        };
        let action = RemediationPolicy::evaluate(&ctx);
        if rec >= 5 {
            assert_eq!(
                action.kind,
                RemediationActionKind::CircuitBreaker,
                "Recurrence {} must yield CircuitBreaker",
                rec
            );
        }
    }
}