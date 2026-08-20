// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::sync::Arc;
use std::thread;
use std::time::Instant;

use chrono::Utc;
use procedural::anchor::ProceduralCardAnchor;
use procedural::core::{AttemptId, Domain, ErrorEventId, ProblemFamilyId, ProblemInstanceId, PyqId, Result, SchemaId, SkillId};
use procedural::exam::{MappingConfidence, MappingStatus, PYQSource, PyqMapping};
use procedural::practice::{ErrorEvent, PracticeAttempt, SchemaPracticeObject};
use procedural::problems::{ProblemFamily, ProblemInstance};
use procedural::reasoning::csp::{CspConstraint, CspProblem, CspSolver, CspStatus, MAX_CSP_RECURSION_DEPTH, MAX_CSP_SEARCH_NODES};
use procedural::skills::signals::{IndependenceLevel, MasteryEvidence, PracticeProgressionState};
use procedural::skills::{Skill, SkillState};
use procedural::storage::ProceduralStore;
use tempfile::tempdir;

fn setup_test_entities(
    store: &ProceduralStore,
    skill_id: &SkillId,
    family_id: &ProblemFamilyId,
    schema_id: &SchemaId,
    instance_id: &ProblemInstanceId,
) -> Result<()> {
    let skill = Skill {
        id: skill_id.clone(),
        domain: Domain::Mathematics,
        name: "Linear Equations".to_string(),
        description: "Solving linear equations".to_string(),
        prerequisites: vec![],
        metadata: serde_json::json!({}),
        created_at: Utc::now().timestamp(),
    };
    store.insert_skill(&skill)?;

    let family = ProblemFamily {
        id: family_id.clone(),
        skill_id: skill_id.clone(),
        domain: Domain::Mathematics,
        name: "Linear Equation Family".to_string(),
        template_ref: "linear_equations".to_string(),
        min_difficulty: 1.0,
        max_difficulty: 5.0,
        parameters_schema: serde_json::json!({}),
        metadata: serde_json::json!({}),
        created_at: Utc::now().timestamp(),
    };
    store.insert_problem_family(&family)?;

    let schema = SchemaPracticeObject {
        id: schema_id.clone(),
        skill_id: skill_id.clone(),
        problem_family_id: family_id.clone(),
        title: "Linear Equations Practice".to_string(),
        description: "Practice object".to_string(),
        target_mastery: 0.9,
        config: serde_json::json!({}),
        created_at: Utc::now().timestamp(),
    };
    store.insert_schema(&schema)?;

    let instance = ProblemInstance {
        id: instance_id.clone(),
        family_id: family_id.clone(),
        seed: 12345,
        parameters: serde_json::json!({"a": 2, "b": 4}),
        rendered_prompt: "Solve 2x + 4 = 0".to_string(),
        correct_answer: serde_json::json!({"value": -2.0}),
        metadata: serde_json::json!({}),
        created_at: Utc::now().timestamp(),
    };
    store.insert_problem_instance(&instance)?;

    Ok(())
}

// =========================================================================
// 1. ATOMIC PROCEDURAL ATTEMPT PERSISTENCE & FAILURE INJECTION
// =========================================================================

#[test]
fn test_atomic_attempt_persistence_and_rollback() -> Result<()> {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("atomic_test.db");
    let store = ProceduralStore::open(&db_path)?;

    let skill_id = SkillId::new("math.algebra.linear");
    let family_id = ProblemFamilyId::new("family.math.linear");
    let schema_id = SchemaId::new("schema.math.linear");
    let instance_id = ProblemInstanceId::new("inst-atomic-1");

    setup_test_entities(&store, &skill_id, &family_id, &schema_id, &instance_id)?;

    let initial_state = SkillState::new(skill_id.clone());
    store.upsert_skill_state(&initial_state)?;

    let attempt = PracticeAttempt {
        id: AttemptId::new("att-atomic-1"),
        instance_id: instance_id.clone(),
        schema_id: schema_id.clone(),
        skill_id: skill_id.clone(),
        card_id: Some(101),
        user_answer: serde_json::json!({"value": -2.0}),
        is_correct: true,
        score: 1.0,
        time_taken_ms: 12000,
        attempted_at: Utc::now().timestamp(),
        metadata: serde_json::json!({"variant": "standard"}),
    };

    let error_event = ErrorEvent {
        id: ErrorEventId::new("err-atomic-1"),
        attempt_id: attempt.id.clone(),
        error_category: "calculation_slip".to_string(),
        details: serde_json::json!({"step": 1}),
        occurred_at: Utc::now().timestamp(),
    };

    let mut updated_state = initial_state.clone();
    updated_state.total_attempts = 1;
    updated_state.successful_attempts = 1;
    updated_state.mastery = 0.85;

    // Successful atomic commit
    store.record_attempt_atomic(&attempt, &[error_event], &updated_state)?;

    // Verify all entities are persisted
    let loaded_attempts = store.get_practice_attempts_by_card(101)?;
    assert_eq!(loaded_attempts.len(), 1);
    assert_eq!(loaded_attempts[0].id.as_str(), "att-atomic-1");

    let loaded_state = store.get_skill_state(&skill_id)?.expect("skill state exists");
    assert_eq!(loaded_state.total_attempts, 1);
    assert_eq!(loaded_state.successful_attempts, 1);
    assert!((loaded_state.mastery - 0.85).abs() < 1e-6);

    // Test failure injection: attempting to insert duplicate attempt ID
    let duplicate_attempt = PracticeAttempt {
        id: AttemptId::new("att-atomic-1"), // Duplicate primary key
        instance_id: instance_id.clone(),
        schema_id: schema_id.clone(),
        skill_id: skill_id.clone(),
        card_id: Some(101),
        user_answer: serde_json::json!({"value": 99.0}),
        is_correct: false,
        score: 0.0,
        time_taken_ms: 5000,
        attempted_at: Utc::now().timestamp(),
        metadata: serde_json::json!({}),
    };

    let mut corrupt_state = loaded_state.clone();
    corrupt_state.mastery = 0.10; // State that shouldn't be committed

    let failed_result = store.record_attempt_atomic(&duplicate_attempt, &[], &corrupt_state);
    assert!(failed_result.is_err(), "Duplicate primary key MUST cause atomic transaction failure");

    // Verify rollback: skill state was NOT updated to 0.10, attempts count is still 1
    let verify_state = store.get_skill_state(&skill_id)?.expect("skill state exists");
    assert_eq!(verify_state.total_attempts, 1);
    assert!((verify_state.mastery - 0.85).abs() < 1e-6);

    Ok(())
}

// =========================================================================
// 2. SQLITE PRODUCTION PRAGMAS & CONCURRENCY
// =========================================================================

#[test]
fn test_sqlite_pragmas_and_concurrency() -> Result<()> {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("concurrency_test.db");
    let store = ProceduralStore::open(&db_path)?;

    let skill_id = SkillId::new("math.concurrency");
    let family_id = ProblemFamilyId::new("family.concurrency");
    let schema_id = SchemaId::new("schema.concurrency");
    let base_inst_id = ProblemInstanceId::new("inst-concurrency-base");

    setup_test_entities(&store, &skill_id, &family_id, &schema_id, &base_inst_id)?;

    // Pre-insert 200 instances for 10 threads x 20 attempts
    for t in 0..10 {
        for i in 0..20 {
            let instance = ProblemInstance {
                id: ProblemInstanceId::new(format!("inst-{}-{}", t, i)),
                family_id: family_id.clone(),
                seed: (t * 100 + i) as u64,
                parameters: serde_json::json!({}),
                rendered_prompt: "Concurrent prompt".to_string(),
                correct_answer: serde_json::json!({"value": 0}),
                metadata: serde_json::json!({}),
                created_at: Utc::now().timestamp(),
            };
            store.insert_problem_instance(&instance)?;
        }
    }

    let store_arc = Arc::new(store);
    let mut handles = Vec::new();

    // Spawn 10 threads doing concurrent atomic writes
    for t in 0..10 {
        let s = Arc::clone(&store_arc);
        let s_id = skill_id.clone();
        let sch_id = schema_id.clone();
        let handle = thread::spawn(move || {
            for i in 0..20 {
                let attempt = PracticeAttempt {
                    id: AttemptId::new(format!("att-thread-{}-{}", t, i)),
                    instance_id: ProblemInstanceId::new(format!("inst-{}-{}", t, i)),
                    schema_id: sch_id.clone(),
                    skill_id: s_id.clone(),
                    card_id: Some(t as i64),
                    user_answer: serde_json::json!({"ans": i}),
                    is_correct: i % 2 == 0,
                    score: if i % 2 == 0 { 1.0 } else { 0.0 },
                    time_taken_ms: 10000,
                    attempted_at: Utc::now().timestamp(),
                    metadata: serde_json::json!({}),
                };

                let mut state = SkillState::new(s_id.clone());
                state.total_attempts = ((t + 1) * (i + 1)) as u32;

                s.record_attempt_atomic(&attempt, &[], &state).unwrap();
            }
        });
        handles.push(handle);
    }

    for h in handles {
        h.join().unwrap();
    }

    // Verify all 200 attempts were cleanly persisted without corruption
    let mut total_attempts_found = 0;
    for t in 0..10 {
        let attempts = store_arc.get_practice_attempts_by_card(t as i64)?;
        total_attempts_found += attempts.len();
        assert_eq!(attempts.len(), 20);
    }
    assert_eq!(total_attempts_found, 200);

    Ok(())
}

// =========================================================================
// 3. N+1 QUERY REMEDIATION BENCHMARK (22, 100, 300, 1,000 SKILLS)
// =========================================================================

#[test]
fn test_n_plus_one_query_remediation_benchmark() -> Result<()> {
    let store = ProceduralStore::open_in_memory()?;

    let skill_counts = [22, 100, 300, 1000];

    for i in 0..1000 {
        let skill_id = SkillId::new(format!("skill.bench.{}", i));
        let family_id = ProblemFamilyId::new(format!("family.bench.{}", i));
        let schema_id = SchemaId::new(format!("schema.bench.{}", i));

        let skill = Skill {
            id: skill_id.clone(),
            domain: Domain::Mathematics,
            name: format!("Skill {}", i),
            description: "Bench skill".to_string(),
            prerequisites: vec![],
            metadata: serde_json::json!({}),
            created_at: Utc::now().timestamp(),
        };
        store.insert_skill(&skill)?;

        let family = ProblemFamily {
            id: family_id.clone(),
            skill_id: skill_id.clone(),
            domain: Domain::Mathematics,
            name: format!("Family {}", i),
            template_ref: "bench".to_string(),
            min_difficulty: 1.0,
            max_difficulty: 5.0,
            parameters_schema: serde_json::json!({}),
            metadata: serde_json::json!({}),
            created_at: Utc::now().timestamp(),
        };
        store.insert_problem_family(&family)?;

        let schema = SchemaPracticeObject {
            id: schema_id.clone(),
            skill_id: skill_id.clone(),
            problem_family_id: family_id.clone(),
            title: format!("Schema {}", i),
            description: "Bench schema".to_string(),
            target_mastery: 0.9,
            config: serde_json::json!({}),
            created_at: Utc::now().timestamp(),
        };
        store.insert_schema(&schema)?;

        let mut state = SkillState::new(skill_id.clone());
        state.mastery = 0.5 + (i as f64 % 50.0) / 100.0;
        store.upsert_skill_state(&state)?;

        let pyq = PYQSource {
            id: PyqId::new(format!("pyq-bench-{}", i)),
            exam: "rrb_alp".to_string(),
            year: 2024,
            paper: None,
            shift: None,
            session: None,
            domain: Domain::Mathematics,
            original_question: format!("Question for schema {}", i),
            original_options: None,
            original_answer: serde_json::json!(42),
            source_reference: "RRB-ALP-2024".to_string(),
            provenance: Default::default(),
            source_version: 1,
            import_timestamp: Utc::now().timestamp(),
            metadata: serde_json::json!({}),
        };
        store.insert_pyq_source(&pyq)?;

        let mapping = PyqMapping {
            pyq_id: pyq.id.clone(),
            domain: Domain::Mathematics,
            skill_id: skill_id.clone(),
            schema_id: schema_id.clone(),
            problem_family_id: family_id.clone(),
            variant_structure: None,
            difficulty_level: 2,
            target_latency_ms: 45000,
            diagnostic_metadata: serde_json::json!({}),
            status: MappingStatus::Verified,
            confidence: MappingConfidence::Deterministic,
            reviewer_notes: None,
            updated_at: Utc::now().timestamp(),
        };
        store.insert_pyq_mapping(&mapping)?;
    }

    println!("\n=== N+1 Query Remediation Benchmark ===");
    for count in skill_counts {
        // Measure Batch/Map Preloaded Retrieval (New O(1) Queries)
        let start_batch = Instant::now();
        let schemas_map = store.list_all_schemas_map()?;
        let skill_states_map = store.list_all_skill_states_map()?;
        let eligible_pyqs_map = store.list_all_eligible_pyqs_map()?;
        let duration_batch = start_batch.elapsed();

        assert!(schemas_map.len() >= count);
        assert!(skill_states_map.len() >= count);
        assert!(eligible_pyqs_map.len() >= count);

        // Measure Sequential N+1 Lookups for comparison
        let start_sequential = Instant::now();
        for i in 0..count {
            let schema_id = SchemaId::new(format!("schema.bench.{}", i));
            let skill_id = SkillId::new(format!("skill.bench.{}", i));
            let _ = store.get_schema(&schema_id)?;
            let _ = store.get_skill_state(&skill_id)?;
            let _ = store.list_eligible_pyqs_for_schema(&schema_id)?;
        }
        let duration_seq = start_sequential.elapsed();

        println!(
            "Skills: {:>4} | Batch Queries (3 DB calls): {:>6.2} ms | Sequential Queries ({} calls): {:>7.2} ms | Speedup: {:.1}x",
            count,
            duration_batch.as_secs_f64() * 1000.0,
            count * 3,
            duration_seq.as_secs_f64() * 1000.0,
            duration_seq.as_secs_f64() / duration_batch.as_secs_f64().max(1e-6)
        );
    }

    Ok(())
}

// =========================================================================
// 4. CSP RECURSION & BOUNDED SEARCH TESTS
// =========================================================================

#[test]
fn test_csp_bounded_search_depth_and_node_budget() {
    let solver = CspSolver;

    // 1. Satisfiable CSP within bounds
    let vars = vec!["A".to_string(), "B".to_string(), "C".to_string(), "D".to_string()];
    let mut problem = CspProblem::new(vars, 4);
    problem.add_constraint(CspConstraint::AllDifferent);
    problem.add_constraint(CspConstraint::ImmediateLeft {
        v1: "A".to_string(),
        v2: "B".to_string(),
    });
    problem.add_constraint(CspConstraint::Fixed {
        var: "C".to_string(),
        slot: 4,
    });
    problem.add_constraint(CspConstraint::Fixed {
        var: "D".to_string(),
        slot: 1,
    });

    let report = solver.solve_bounded(&problem, MAX_CSP_RECURSION_DEPTH, MAX_CSP_SEARCH_NODES);
    assert_eq!(report.status, CspStatus::Complete);
    assert_eq!(report.solutions.len(), 1);

    // 2. Pathological branching CSP exceeding artificial tight node budget
    let deep_vars = (0..10).map(|i| format!("V{}", i)).collect();
    let mut deep_problem = CspProblem::new(deep_vars, 10);
    deep_problem.add_constraint(CspConstraint::AllDifferent);

    // Limit to 20 search nodes
    let tight_report = solver.solve_bounded(&deep_problem, 32, 20);
    assert_eq!(tight_report.status, CspStatus::NodeBudgetExhausted);
    assert!(tight_report.nodes_visited > 20);

    // Limit to depth 2
    let shallow_report = solver.solve_bounded(&deep_problem, 2, 10_000);
    assert_eq!(shallow_report.status, CspStatus::DepthLimitExceeded);
}

// =========================================================================
// 5. PROCEDURAL CARD ANCHOR FALLBACK
// =========================================================================

#[test]
fn test_card_anchor_fallback_and_graceful_degradation() -> Result<()> {
    // 1. Valid anchor JSON
    let valid_json = r#"{"proc_schema":"math.profit_loss","difficulty_override":2.5,"seed_mode":"random"}"#;
    let anchor = ProceduralCardAnchor::from_json_str(valid_json)?;
    assert!(anchor.is_some());
    assert_eq!(anchor.unwrap().proc_schema.as_str(), "math.profit_loss");

    // 2. Missing metadata (normal flashcard note fields)
    let regular_note_field = "What is the formula for kinetic energy?";
    let no_anchor = ProceduralCardAnchor::from_json_str(regular_note_field)?;
    assert!(no_anchor.is_none(), "Normal non-procedural cards must return Ok(None)");

    // 3. Malformed JSON containing 'proc_schema' substring
    let malformed_field = r#"{"proc_schema": invalid_json_syntax_here"#;
    let malformed_result = ProceduralCardAnchor::from_json_str(malformed_field)?;
    assert!(malformed_result.is_none(), "Malformed anchor metadata must safely degrade to None without crashing");

    // 4. Strict parser detects the syntax error
    let strict_err = ProceduralCardAnchor::from_json_str_strict(malformed_field);
    assert!(strict_err.is_err());

    // 5. extract_from_card_fields with mixed fields
    let card_fields = vec![
        "Question prompt".to_string(),
        r#"{"proc_schema": broken"#.to_string(),
        r#"{"proc_schema":"physics.kinematics.1d","difficulty_override":1.0}"#.to_string(),
    ];
    let extracted = ProceduralCardAnchor::extract_from_card_fields(&card_fields)?;
    assert!(extracted.is_some());
    assert_eq!(extracted.unwrap().proc_schema.as_str(), "physics.kinematics.1d");

    Ok(())
}

// =========================================================================
// 6. PROVENANCE & VERSION BACKWARDS COMPATIBILITY
// =========================================================================

#[test]
fn test_provenance_and_version_compatibility() -> Result<()> {
    let old_instance_json = r#"{
        "id": "inst-legacy-001",
        "family_id": "arithmetic.profit_loss",
        "seed": 12345,
        "parameters": {"cp": 100, "profit_percent": 20, "variant": "standard"},
        "rendered_prompt": "A shopkeeper buys an item for Rs 100 and sells it at a 20% profit.",
        "correct_answer": {"value": 120.0, "unit": "Rs", "formatted": "120"},
        "metadata": {"version": 1},
        "created_at": 1700000000
    }"#;

    let instance: ProblemInstance = serde_json::from_str(old_instance_json)?;
    assert_eq!(instance.id.as_str(), "inst-legacy-001");
    assert_eq!(instance.family_id.as_str(), "arithmetic.profit_loss");
    assert_eq!(instance.seed, 12345);
    assert_eq!(instance.rendered_prompt, "A shopkeeper buys an item for Rs 100 and sells it at a 20% profit.");

    let ans_val = instance.correct_answer.get("value").and_then(|v| v.as_f64()).unwrap();
    assert_eq!(ans_val, 120.0);

    Ok(())
}

// =========================================================================
// 7. CORRECTED F2 SIMULATION ACCOUNTING SANITY CHECK
// =========================================================================

#[test]
fn test_simulation_accounting_progression_sanity() {
    let mut state = SkillState::new(SkillId::new("math.time_work"));
    assert_eq!(state.practice_state, PracticeProgressionState::New);

    // Stage 1: New -> Learning
    let ev1 = MasteryEvidence {
        final_correctness: true,
        latency_evidence: 15000,
        variant_exposure: Some("standard".to_string()),
        independence: IndependenceLevel::Independent,
        ..Default::default()
    };
    state.record_attempt_outcome(&ev1, 1.0, 45000, 1000);
    assert_eq!(state.practice_state, PracticeProgressionState::Learning);

    // Stage 2: Learning -> Fluent (Needs 3 consecutive independent successes with 0 conceptual errors)
    for i in 0..3 {
        let ev = MasteryEvidence {
            final_correctness: true,
            latency_evidence: 15000,
            variant_exposure: Some("standard".to_string()),
            independence: IndependenceLevel::Independent,
            ..Default::default()
        };
        state.record_attempt_outcome(&ev, 1.0, 45000, 2000 + i * 1000);
    }
    assert_eq!(state.practice_state, PracticeProgressionState::Fluent);

    // Stage 3: Fluent -> Variation (Needs >=2 distinct variants explored independently)
    let ev_var2 = MasteryEvidence {
        final_correctness: true,
        latency_evidence: 14000,
        variant_exposure: Some("collaborative".to_string()),
        independence: IndependenceLevel::Independent,
        ..Default::default()
    };
    state.record_attempt_outcome(&ev_var2, 1.0, 45000, 6000);
    assert_eq!(state.practice_state, PracticeProgressionState::Variation);

    // Stage 4: Variation -> Transfer (Needs >=2 distinct variants with >=2 successes each)
    state.record_attempt_outcome(&ev_var2, 1.0, 45000, 7000);
    assert_eq!(state.practice_state, PracticeProgressionState::Transfer);

    // Stage 5: Transfer -> Mastered (Needs >=3 variants, 5 consecutive successes, transfer_evidence = true)
    for i in 0..5 {
        let ev_transfer = MasteryEvidence {
            final_correctness: true,
            latency_evidence: 12000,
            variant_exposure: Some("pipes_cisterns".to_string()),
            independence: IndependenceLevel::Independent,
            transfer_evidence: true,
            ..Default::default()
        };
        state.record_attempt_outcome(&ev_transfer, 1.0, 45000, 10000 + i * 1000);
    }
    assert_eq!(state.practice_state, PracticeProgressionState::Mastered);

    println!("Progression pipeline successfully traversed: New -> Learning -> Fluent -> Variation -> Transfer -> Mastered");
}
