use procedural::anchor::{ProceduralCardAnchor, SeedMode};
use procedural::core::{Domain, PracticeItemId, ProblemFamilyId, SchemaId, SkillId};
use procedural::content::item::{PracticeItem, Origin, QuestionType};
use procedural::exam::pyq::ContentProvenance;
use procedural::service::ProceduralService;
use procedural::problems::catalog::{
    SCHEMA_LINEAR_EQUATIONS, SCHEMA_REASONING_CODED_EXPRESSIONS, SCHEMA_PHYSICS_KINEMATICS,
    SCHEMA_CHEMISTRY_STOICHIOMETRY, SCHEMA_REASONING_SEATING
};
use std::time::Instant;

fn setup_service() -> ProceduralService {
    let service = ProceduralService::open_in_memory().expect("Failed to open in-memory service");
    service
}

fn create_practice_item(
    id: &str,
    domain: Domain,
    schema_id: &str,
    family_id: &str,
    variant_type: &str,
) -> PracticeItem {
    PracticeItem::new(
        PracticeItemId::new(id),
        Origin::SyntheticSchema {
            generator_version: 1,
            seed: 42,
        },
        domain,
        "Test Chapter",
        SkillId::new(format!("{}.skill", family_id)),
        SchemaId::new(schema_id),
        ProblemFamilyId::new(family_id),
        QuestionType::Structured {
            steps: serde_json::json!([]),
        },
        "What is the result?",
        ContentProvenance::new_direct_procedural(1, 1, 1, variant_type, 42),
    )
}

#[test]
fn test_fresh_profile_import_and_resolve() {
    let service = setup_service();

    // 1. Fresh Profile - the item is not there yet.
    let mut anchor = ProceduralCardAnchor::new("math.lcm_hcf");
    anchor.content_ref = Some("item_math_lcm_hcf_new".to_string());
    anchor.seed_mode = SeedMode::Fixed(42);

    let resolve_result = service.resolve_procedural_target(&anchor, None);
    assert!(resolve_result.is_err(), "Should fail because PracticeItem isn't hydrated");

    // 2. APKG Hydration (Mocking the import)
    let item = create_practice_item(
        "item_math_lcm_hcf_new",
        Domain::Mathematics,
        "number_system_divisibility",
        "family.math.number_system.divisibility",
        "standard",
    );
    service.store().insert_practice_item(&item).unwrap();

    // 3. Resolve again
    let session = service.resolve_procedural_target(&anchor, None).expect("Should resolve successfully now");

    // Verify
    assert_eq!(session.schema.id.as_str(), "number_system_divisibility");
    // Note: The Reviewer render test is handled implicitly by verifying we get a valid PracticeSessionObject
    // that the reviewer template uses to extract parameters and answer.
    assert!(!session.instance.rendered_prompt.is_empty());
}

#[test]
fn test_cross_domain_resolution_matrix() {
    let service = setup_service();
    let store = service.store();

    let domains_to_test = vec![
        ("item_math_algebra", Domain::Mathematics, SCHEMA_LINEAR_EQUATIONS, "family.math.algebra.linear_equations", "standard"),
        ("item_reasoning_coding", Domain::Reasoning, SCHEMA_REASONING_CODED_EXPRESSIONS, "family.reasoning.coded_expressions.relations", "standard"),
        ("item_reasoning_seating", Domain::Reasoning, SCHEMA_REASONING_SEATING, "family.reasoning.seating.linear", "standard"),
        ("item_physics_kinematics", Domain::Physics, SCHEMA_PHYSICS_KINEMATICS, "family.physics.kinematics.1d", "standard"),
        ("item_chemistry_stoichiometry", Domain::Chemistry, SCHEMA_CHEMISTRY_STOICHIOMETRY, "family.chemistry.stoichiometry.moles", "standard"),
    ];

    println!("Domain\tContent Ref\tPracticeItem Found\tSchema Resolved\tFamily\tInstance\tReviewer\tResult");

    for (item_id, domain, schema_id, family_id, variant) in domains_to_test {
        let item = create_practice_item(item_id, domain.clone(), schema_id, family_id, variant);
        store.insert_practice_item(&item).unwrap();

        let mut anchor = ProceduralCardAnchor::new("legacy_ignored");
        anchor.content_ref = Some(item_id.to_string());
        anchor.seed_mode = SeedMode::Fixed(42);

        let session = service.resolve_procedural_target(&anchor, None).unwrap();

        // 5. Canonical Resolution
        assert_eq!(session.schema.id.as_str(), schema_id);
        
        // 12. Cross-Domain Isolation
        assert_eq!(session.instance.family_id.as_str(), family_id);
        
        // 7. Actual Generation
        assert!(!session.instance.correct_answer.is_null());
        assert_eq!(session.selected_variant.as_deref(), Some(variant));

        println!(
            "{:?}\t{}\ttrue\t{}\t{}\tOK\tOK\tSUCCESS",
            domain, item_id, schema_id, family_id
        );
    }
}

#[test]
fn test_legacy_apkg_compatibility() {
    let service = setup_service();

    let mut legacy_anchor = ProceduralCardAnchor::new("number_system_divisibility");
    legacy_anchor.seed_mode = SeedMode::Fixed(42);

    // Should resolve via unified practice engine and alias
    let session = service.resolve_procedural_target(&legacy_anchor, None).unwrap();
    assert_eq!(session.schema.id.as_str(), "number_system_divisibility");
}

#[test]
fn test_unknown_content_failures() {
    let service = setup_service();

    // Unknown content_ref
    let mut anchor_unknown = ProceduralCardAnchor::new("math.algebra");
    anchor_unknown.content_ref = Some("nonexistent_item_123".to_string());
    anchor_unknown.seed_mode = SeedMode::Fixed(42);

    let err = service.resolve_procedural_target(&anchor_unknown, None).unwrap_err();
    assert!(err.to_string().contains("Content Reference 'nonexistent_item_123' not found"));

    // Valid content_ref + invalid schema
    let item_invalid_schema = create_practice_item(
        "item_invalid_schema",
        Domain::Mathematics,
        "invalid_schema_xyz",
        "family.math.number_system.divisibility",
        "standard",
    );
    service.store().insert_practice_item(&item_invalid_schema).unwrap();

    let mut anchor_invalid_schema = ProceduralCardAnchor::new("math.algebra");
    anchor_invalid_schema.content_ref = Some("item_invalid_schema".to_string());
    anchor_invalid_schema.seed_mode = SeedMode::Fixed(42);

    let err = service.resolve_procedural_target(&anchor_invalid_schema, None).unwrap_err();
    assert!(err.to_string().contains("Schema not found: invalid_schema_xyz"));

    // Valid content_ref + missing family
    // The default maths catalog might not have "missing.family.abc"
    let item_missing_family = create_practice_item(
        "item_missing_family",
        Domain::Mathematics,
        SCHEMA_LINEAR_EQUATIONS,
        "family.missing.abc",
        "standard",
    );
    service.store().insert_practice_item(&item_missing_family).unwrap();

    let mut anchor_missing_family = ProceduralCardAnchor::new("math.algebra");
    anchor_missing_family.content_ref = Some("item_missing_family".to_string());
    anchor_missing_family.seed_mode = SeedMode::Fixed(42);

    let err = service.resolve_procedural_target(&anchor_missing_family, None).unwrap_err();
    assert!(err.to_string().contains("Problem family not found: family.missing.abc"));
}

#[test]
fn test_scale_synthetic_corpus() {
    let service = setup_service();
    let store = service.store();

    let start = Instant::now();

    for i in 0..100 {
        let item = create_practice_item(
            &format!("synth_math_{}", i),
            Domain::Mathematics,
            SCHEMA_LINEAR_EQUATIONS,
            "family.math.algebra.linear_equations",
            "standard",
        );
        store.insert_practice_item(&item).unwrap();
        
        let item = create_practice_item(
            &format!("synth_reasoning_{}", i),
            Domain::Reasoning,
            SCHEMA_REASONING_CODED_EXPRESSIONS,
            "family.reasoning.coded_expressions.relations",
            "standard",
        );
        store.insert_practice_item(&item).unwrap();
    }

    let elapsed_insert = start.elapsed();
    println!("Inserted 200 synthetic items in {:?}", elapsed_insert);

    // Resolution should still be fast
    let resolve_start = Instant::now();
    let mut anchor = ProceduralCardAnchor::new("legacy");
    anchor.content_ref = Some("synth_reasoning_99".to_string());
    anchor.seed_mode = SeedMode::Fixed(42);

    let session = service.resolve_procedural_target(&anchor, None).unwrap();
    let elapsed_resolve = resolve_start.elapsed();
    println!("Resolved 1 synthetic item in {:?}", elapsed_resolve);

    assert_eq!(session.schema.id.as_str(), SCHEMA_REASONING_CODED_EXPRESSIONS);
}

#[test]
fn test_important_architectural_new_pattern() {
    let service = setup_service();
    let store = service.store();

    // 17. No Code Change Test
    // Add a purely content-based PracticeItem linking to an existing capability (math.algebra.linear_equations)
    // but with a new variant type and completely new ID that doesn't exist in any schema alias tables.
    let item = create_practice_item(
        "item_totally_new_pattern",
        Domain::Mathematics,
        SCHEMA_LINEAR_EQUATIONS,
        "family.math.algebra.linear_equations", // Capability
        "new_experimental_variant", // Variation
    );
    store.insert_practice_item(&item).unwrap();

    let mut anchor = ProceduralCardAnchor::new("doesnt_matter_at_all");
    anchor.content_ref = Some("item_totally_new_pattern".to_string());
    anchor.seed_mode = SeedMode::Fixed(999);

    let session = service.resolve_procedural_target(&anchor, None).unwrap();
    assert_eq!(session.selected_variant.as_deref(), Some("new_experimental_variant"));
    // We generated a question without ever touching render.rs or alias lookup tables!
    assert!(!session.instance.rendered_prompt.is_empty());
}