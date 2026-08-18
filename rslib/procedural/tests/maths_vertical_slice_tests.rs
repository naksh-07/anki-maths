use procedural::{
    Domain, ErrorCategory, MathsCatalog, PercentageSuccessiveConfig, PercentageSuccessiveGenerator,
    PercentageSuccessiveValidator, PercentageVariant, ProceduralCardAnchor, ProceduralService,
    Result, SchemaId, SeedMode, Skill, SkillId,
};
use tempfile::tempdir;

#[test]
fn test_skill_successive_percentage_creation_and_serialization() {
    let skill = MathsCatalog::successive_percentage_skill();

    assert_eq!(skill.id.as_str(), "percentage.successive");
    assert_eq!(skill.domain, Domain::Mathematics);
    assert_eq!(skill.name, "Successive Percentage Changes");
    assert!(!skill.description.is_empty());
    assert_eq!(skill.prerequisites, vec![SkillId::from("percentage.basic")]);

    let target_time = skill
        .metadata
        .get("target_time_ms")
        .and_then(|v| v.as_u64())
        .expect("target_time_ms must be present");
    assert_eq!(target_time, 45_000);

    // Serialization roundtrip
    let json = serde_json::to_string(&skill).expect("serialize skill");
    let deserialized: Skill = serde_json::from_str(&json).expect("deserialize skill");
    assert_eq!(skill, deserialized);
}

#[test]
fn test_schema_lookup_and_anchor_resolution() -> Result<()> {
    let service = ProceduralService::open_in_memory()?;

    // Lookup schema by exact ID
    let schema = service
        .resolve_schema(&SchemaId::from("successive_percentage"))?
        .expect("schema must exist");
    assert_eq!(schema.title, "Successive Percentage Practice");
    assert_eq!(schema.skill_id.as_str(), "percentage.successive");

    // Lookup schema by alias
    let alias_schema = service
        .resolve_schema(&SchemaId::from("percentage.successive"))?
        .expect("schema alias must resolve");
    assert_eq!(alias_schema.id.as_str(), "successive_percentage");

    // Anchor resolution from note fields
    let card_fields = vec![
        "Solve the following percentage problem:".to_string(),
        r#"{"proc_schema": "percentage.successive", "seed_mode": {"fixed": 42}, "custom_params": {"difficulty": 2.0}}"#.to_string(),
        "Back of card answer placeholder".to_string(),
    ];

    let anchor = ProceduralCardAnchor::extract_from_card_fields(&card_fields)?
        .expect("anchor must be extracted");
    assert_eq!(anchor.proc_schema.as_str(), "percentage.successive");

    let session = service.prepare_practice_session(&anchor, Some(7788))?;
    assert_eq!(session.schema.id.as_str(), "successive_percentage");
    assert_eq!(session.card_id, Some(7788));
    assert!(!session.instance.rendered_prompt.is_empty());

    Ok(())
}

#[test]
fn test_generator_deterministic_seed_reproduction() {
    let config = PercentageSuccessiveConfig::default();

    // Verify 100 distinct seeds produce exactly identical results across repeated generations
    for seed in 1..=100 {
        let p1 = PercentageSuccessiveGenerator::generate(seed, &config);
        let p2 = PercentageSuccessiveGenerator::generate(seed, &config);

        assert_eq!(p1.variant, p2.variant, "Seed {seed}: variant mismatch");
        assert_eq!(p1.initial_value, p2.initial_value, "Seed {seed}: initial_value mismatch");
        assert_eq!(p1.final_value, p2.final_value, "Seed {seed}: final_value mismatch");
        assert_eq!(p1.rendered_prompt, p2.rendered_prompt, "Seed {seed}: prompt mismatch");
        assert_eq!(p1.canonical_answer_text, p2.canonical_answer_text, "Seed {seed}: canonical text mismatch");
        assert_eq!(p1.worked_solution, p2.worked_solution, "Seed {seed}: worked solution mismatch");
    }
}

#[test]
fn test_generator_parameter_constraints_and_non_degenerate() {
    let config = PercentageSuccessiveConfig::default();

    for seed in 1000..1500 {
        let p = PercentageSuccessiveGenerator::generate(seed, &config);

        // Positive initial and final values
        assert!(p.initial_value > 0.0, "Initial value must be > 0 (seed {seed})");
        assert!(p.final_value > 0.0, "Final value must be > 0 (seed {seed})");

        // Non-degenerate steps
        assert!(!p.steps.is_empty(), "Problem must have steps (seed {seed})");
        for step in &p.steps {
            assert!(step.percent > 0.0, "Percentage rate must be > 0% (seed {seed})");
            assert!(step.percent <= 100.0, "Percentage rate must be <= 100% (seed {seed})");
            assert!(step.multiplier() > 0.0, "Step multiplier must be > 0 (seed {seed})");
        }

        // Target answer correctness
        match p.variant {
            PercentageVariant::ForwardTwoStep => {
                let s1 = &p.steps[0];
                let s2 = &p.steps[1];
                let expected = p.initial_value * s1.multiplier() * s2.multiplier();
                let rounded_expected = (expected * 100.0).round() / 100.0;
                assert!(
                    (p.target_answer_value - rounded_expected).abs() < 0.001,
                    "ForwardTwoStep mathematical mismatch on seed {seed}"
                );
            }
            PercentageVariant::ReverseInitial => {
                let s1 = &p.steps[0];
                let s2 = &p.steps[1];
                let mult = s1.multiplier() * s2.multiplier();
                let computed_init = (p.final_value / mult * 100.0).round() / 100.0;
                assert!(
                    (p.target_answer_value - computed_init).abs() < 0.01,
                    "ReverseInitial mathematical mismatch on seed {seed}"
                );
            }
            PercentageVariant::NetEquivalentChange => {
                let s1 = &p.steps[0];
                let s2 = &p.steps[1];
                let mult = s1.multiplier() * s2.multiplier();
                let net_pct = ((mult - 1.0) * 100.0 * 100.0).round() / 100.0;
                assert!(
                    (p.target_answer_value - net_pct).abs() < 0.001,
                    "NetEquivalentChange mathematical mismatch on seed {seed}"
                );
            }
            PercentageVariant::ForwardThreeStep => {
                let s1 = &p.steps[0];
                let s2 = &p.steps[1];
                let s3 = &p.steps[2];
                let expected = p.initial_value * s1.multiplier() * s2.multiplier() * s3.multiplier();
                let rounded = (expected * 100.0).round() / 100.0;
                assert!(
                    (p.target_answer_value - rounded).abs() < 0.001,
                    "ForwardThreeStep mathematical mismatch on seed {seed}"
                );
            }
        }
    }
}

#[test]
fn test_validator_comprehensive_cases() {
    let params = serde_json::json!({
        "variant": "forward_two_step",
        "initial_value": 200.0,
        "steps": [
            { "percent": 25.0, "direction": "increase" },
            { "percent": 20.0, "direction": "decrease" }
        ]
    });
    // 200 * 1.25 = 250; 250 * 0.80 = 200.0
    let correct_answer = serde_json::json!({ "value": 200.0 });

    // 1. Exact numeric answer
    let res1 = PercentageSuccessiveValidator::evaluate(
        &correct_answer,
        &params,
        &serde_json::json!(200),
        20000,
        45000,
    );
    assert!(res1.is_correct);
    assert_eq!(res1.score, 1.0);
    assert_eq!(res1.error_category, None);

    // 2. Formatted string with currency symbol and decimal
    let res2 = PercentageSuccessiveValidator::evaluate(
        &correct_answer,
        &params,
        &serde_json::json!("$200.00"),
        25000,
        45000,
    );
    assert!(res2.is_correct);
    assert_eq!(res2.score, 1.0);

    // 3. Time limit exceeded on correct answer: validator returns is_correct with None error_category
    let res3 = PercentageSuccessiveValidator::evaluate(
        &correct_answer,
        &params,
        &serde_json::json!("200"),
        50000,
        45000,
    );
    assert!(res3.is_correct);
    assert_eq!(res3.score, 1.0);
    assert_eq!(res3.error_category, None);

    // 4. Additive fallacy: 200 * (1 + 0.25 - 0.20) = 200 * 1.05 = 210.0
    let res4 = PercentageSuccessiveValidator::evaluate(
        &correct_answer,
        &params,
        &serde_json::json!("210"),
        15000,
        45000,
    );
    assert!(!res4.is_correct);
    assert_eq!(res4.score, 0.0);
    assert_eq!(res4.error_category, Some(ErrorCategory::Concept));
    assert!(res4.diagnostic_message.unwrap().contains("Additive fallacy"));

    // 5. Incomplete step (Careless): 200 * 1.25 = 250.0
    let res5 = PercentageSuccessiveValidator::evaluate(
        &correct_answer,
        &params,
        &serde_json::json!("250"),
        10000,
        45000,
    );
    assert!(!res5.is_correct);
    assert_eq!(res5.error_category, Some(ErrorCategory::Careless));
    assert!(res5.diagnostic_message.unwrap().contains("Incomplete"));

    // 6. Sign/direction inversion: 200 * (1 - 0.25) * (1 - 0.20) = 200 * 0.75 * 0.80 = 120.0
    let res6 = PercentageSuccessiveValidator::evaluate(
        &correct_answer,
        &params,
        &serde_json::json!("120"),
        18000,
        45000,
    );
    assert!(!res6.is_correct);
    assert_eq!(res6.error_category, Some(ErrorCategory::Strategy));

    // 7. General calculation error
    let res7 = PercentageSuccessiveValidator::evaluate(
        &correct_answer,
        &params,
        &serde_json::json!("195"),
        22000,
        45000,
    );
    assert!(!res7.is_correct);
    assert_eq!(res7.error_category, Some(ErrorCategory::Calculation));

    // 8. Non-numeric / malformed input
    let res8 = PercentageSuccessiveValidator::evaluate(
        &correct_answer,
        &params,
        &serde_json::json!("not_a_number"),
        12000,
        45000,
    );
    assert!(!res8.is_correct);
    assert_eq!(res8.error_category, Some(ErrorCategory::Calculation));
}

#[test]
fn test_complete_end_to_end_vertical_slice() -> Result<()> {
    let dir = tempdir().expect("tempdir");
    let db_path = dir.path().join("procedural.db");

    let service = ProceduralService::open(&db_path)?;

    // 1. Anki Card Anchor
    let anchor = ProceduralCardAnchor::new("percentage.successive").with_seed_mode(SeedMode::Fixed(98765));
    let card_id = 99001;

    // 2. Prepare session
    let session = service.prepare_practice_session(&anchor, Some(card_id))?;
    assert_eq!(session.schema.id.as_str(), "successive_percentage");
    assert_eq!(session.instance.seed, 98765);
    assert!(!session.instance.rendered_prompt.is_empty());

    let canonical_val = session
        .instance
        .correct_answer
        .get("value")
        .unwrap()
        .as_f64()
        .unwrap();

    // 3. Attempt 1: Failed (Additive mistake)
    let additive_guess = canonical_val + 10.0;
    let outcome1 = service.evaluate_and_record_attempt(
        &session.instance.id,
        Some(card_id),
        serde_json::json!(additive_guess),
        32_000,
        0,
        1,
    )?;

    assert!(!outcome1.is_correct);
    assert_eq!(outcome1.score, 0.0);
    assert_eq!(outcome1.latency_ms, 32_000);
    assert_eq!(outcome1.target_latency_ms, 35_000);
    assert!(outcome1.error_category.is_some());

    // 4. Attempt 2: Correct on time
    let outcome2 = service.evaluate_and_record_attempt(
        &session.instance.id,
        Some(card_id),
        serde_json::json!(canonical_val),
        21_000,
        0,
        2,
    )?;

    assert!(outcome2.is_correct);
    assert_eq!(outcome2.score, 1.0);
    assert_eq!(outcome2.error_category, None);

    // 5. Verify database persistence in procedural.db
    let card_attempts = service.get_attempts_for_card(card_id)?;
    assert_eq!(card_attempts.len(), 2);
    assert!(card_attempts[0].is_correct); // Most recent first
    assert!(!card_attempts[1].is_correct);

    // 6. Verify SkillState telemetry
    let skill_state = service
        .load_skill_state(&outcome2.skill_id)?
        .expect("skill state must exist");
    assert_eq!(skill_state.total_attempts, 2);
    assert_eq!(skill_state.successful_attempts, 1);
    assert_eq!(skill_state.success_rate(), 0.5);
    assert!(skill_state.mastery > 0.0);
    assert!(skill_state.last_practiced_at.is_some());

    let recent_lat = skill_state
        .custom_state
        .get("recent_latency_ms")
        .and_then(|v| v.as_u64());
    assert_eq!(recent_lat, Some(21_000));

    // 7. Verify review outcome object matches requirements for future FSRS bridge
    assert_eq!(outcome2.schema_id.as_str(), "successive_percentage");
    assert_eq!(outcome2.skill_id.as_str(), "percentage.successive");
    assert_eq!(outcome2.seed, 98765);
    assert_eq!(outcome2.attempt_count, 2);
    assert_eq!(outcome2.hints_used, 0);

    Ok(())
}
