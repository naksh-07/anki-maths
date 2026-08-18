// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use procedural::{
    ContentProvenance, Domain, MappingConfidence, MappingStatus, ProblemFamilyId, ProblemRegistry,
    ProceduralService, ProceduralStore, PyqId, PyqMapping, PyqVariantPipeline, PYQSource,
    RejectedVariantId, RejectedVariantRecord, ReviewAction, SchemaId,
};

#[test]
fn test_immutable_pyq_source_creation_and_metadata() {
    let original_options = vec![
        "30 seconds".to_string(),
        "25 seconds".to_string(),
        "20 seconds".to_string(),
        "35 seconds".to_string(),
    ];

    let pyq = PYQSource::new(
        "pyq.rrb_alp.2018.stage1.shift2.q14",
        "RRB ALP",
        2018,
        Domain::Mathematics,
        "A train 120m long passes a pole in 6 seconds. Find its speed in km/h.",
        serde_json::json!({ "speed_kmh": 72, "correct_option": "A" }),
        "RRB ALP Stage 1 - 09 Aug 2018 Shift 2 Q.14",
    )
    .with_shift_info(Some("Stage 1"), Some("Shift 2"), Some("Morning"))
    .with_options(original_options.clone())
    .with_metadata(serde_json::json!({ "topic": "Time & Distance", "language": "en" }))
    .with_source_version(1);

    // Verify metadata preservation
    assert_eq!(pyq.id.as_str(), "pyq.rrb_alp.2018.stage1.shift2.q14");
    assert_eq!(pyq.exam, "RRB ALP");
    assert_eq!(pyq.year, 2018);
    assert_eq!(pyq.paper.as_deref(), Some("Stage 1"));
    assert_eq!(pyq.shift.as_deref(), Some("Shift 2"));
    assert_eq!(pyq.session.as_deref(), Some("Morning"));
    assert_eq!(pyq.domain, Domain::Mathematics);
    assert_eq!(pyq.original_options, Some(original_options));
    assert_eq!(pyq.provenance.source_pyq_id, Some(pyq.id.clone()));
    assert_eq!(pyq.source_version, 1);
}

#[test]
fn test_pyq_mapping_transitions_and_confidence_gating() {
    let mut mapping = PyqMapping::new(
        "pyq.ssc_cgl.2021.q05",
        Domain::Mathematics,
        "arithmetic.profit_loss",
        "schema.math.arithmetic.profit_loss",
        "family.math.arithmetic.profit_loss",
        3,
        45_000,
    )
    .with_status(MappingStatus::Unreviewed)
    .with_confidence(MappingConfidence::NeedsReview);

    // 1. Newly imported / unreviewed: must NOT be eligible for autonomous practice
    assert!(!mapping.is_eligible_for_practice());

    // 2. Mapped with NeedsReview: still gated
    mapping.status = MappingStatus::Mapped;
    assert!(!mapping.is_eligible_for_practice());

    // 3. High confidence Mapped: eligible
    mapping.confidence = MappingConfidence::HighConfidence;
    assert!(mapping.is_eligible_for_practice());

    // 4. Deterministic Mapped: eligible
    mapping.confidence = MappingConfidence::Deterministic;
    assert!(mapping.is_eligible_for_practice());

    // 5. Verified: always eligible
    mapping.status = MappingStatus::Verified;
    assert!(mapping.is_eligible_for_practice());

    // 6. Rejected: strictly blocked
    mapping.status = MappingStatus::Rejected;
    assert!(!mapping.is_eligible_for_practice());
}

#[test]
fn test_content_provenance_and_deterministic_reproducibility() {
    let registry = ProblemRegistry::default_maths_registry();
    let mapping = PyqMapping::new(
        "pyq.rrb_alp.2018.q20",
        Domain::Mathematics,
        "percentage.successive",
        "schema.math.percentage.successive",
        "family.math.percentage.successive",
        2,
        35_000,
    )
    .with_variant_structure("reverse");

    let pyq = PYQSource::new(
        "pyq.rrb_alp.2018.q20",
        "RRB ALP",
        2018,
        Domain::Mathematics,
        "Original prompt",
        serde_json::json!({ "final": 20 }),
        "Reference 2018",
    );

    // Generate instance under fixed seed 424242
    let instance1 = PyqVariantPipeline::generate_and_validate_variant(
        &registry,
        Some(&pyq),
        &mapping,
        424242,
        Some("reverse"),
    )
    .expect("Variant generation should succeed");

    // Generate instance again under same seed 424242
    let instance2 = PyqVariantPipeline::generate_and_validate_variant(
        &registry,
        Some(&pyq),
        &mapping,
        424242,
        Some("reverse"),
    )
    .expect("Variant generation should succeed");

    // Must be bitwise and semantically identical
    assert_eq!(instance1.rendered_prompt, instance2.rendered_prompt);
    assert_eq!(instance1.correct_answer, instance2.correct_answer);
    assert_eq!(instance1.seed, 424242);

    // Provenance verification
    let prov1: ContentProvenance = serde_json::from_value(
        instance1.metadata.get("provenance").unwrap().clone(),
    )
    .unwrap();

    assert_eq!(prov1.source_pyq_id, Some(pyq.id));
    assert_eq!(prov1.variant_type, "reverse");
    assert_eq!(prov1.seed, Some(424242));
}

#[test]
fn test_domain_validation_gate_accepts_valid_and_stores_rejected() {
    let store = ProceduralStore::open_in_memory().unwrap();
    let registry = ProblemRegistry::default_maths_registry();

    let mapping = PyqMapping::new(
        "pyq.test.valid_and_reject",
        Domain::Mathematics,
        "percentage.successive",
        "schema.math.percentage.successive",
        "family.math.percentage.successive",
        2,
        35_000,
    );

    // 1. Valid variant generation passes
    let valid_inst = PyqVariantPipeline::generate_and_validate_variant(
        &registry,
        None,
        &mapping,
        12345,
        Some("isomorphic"),
    );
    assert!(valid_inst.is_ok());

    // 2. Persist a rejected variant record
    let rej_record = RejectedVariantRecord {
        id: RejectedVariantId::new("rej_sample_01"),
        source_pyq_id: Some(PyqId::new("pyq.test.valid_and_reject")),
        schema_id: SchemaId::new("schema.math.percentage.successive"),
        family_id: ProblemFamilyId::new("family.math.percentage.successive"),
        seed: 8888,
        variant_type: "trap".to_string(),
        failure_reason: "Sanity check: parameter produced degenerate zero division".to_string(),
        generated_instance_json: serde_json::json!({ "error_stage": "eval" }),
        rejected_at: 1700000000,
    };
    store.insert_rejected_variant(&rej_record).unwrap();

    let rejections = store.get_rejected_variants(5).unwrap();
    assert_eq!(rejections.len(), 1);
    assert_eq!(rejections[0].id.as_str(), "rej_sample_01");
    assert!(rejections[0].failure_reason.contains("degenerate zero division"));
}

#[test]
fn test_human_review_workflow_actions_inspection() {
    let service = ProceduralService::open_in_memory().unwrap();

    let pyq = PYQSource::new(
        "pyq.review.001",
        "SSC CGL",
        2020,
        Domain::Mathematics,
        "If a shopkeeper sells an article at 20% discount and gains 10%, find marked price ratio.",
        serde_json::json!({ "ratio": "11:8" }),
        "SSC CGL 2020 Tier 1",
    );
    let mapping = PyqMapping::new(
        &pyq.id,
        Domain::Mathematics,
        "arithmetic.profit_loss",
        "schema.math.arithmetic.profit_loss",
        "family.math.arithmetic.profit_loss",
        2,
        40_000,
    )
    .with_status(MappingStatus::Unreviewed)
    .with_confidence(MappingConfidence::NeedsReview);

    service.ingest_pyq(pyq.clone(), Some(mapping.clone())).unwrap();

    // 1. Inspect PYQ for review
    let inspection = service.inspect_pyq_for_review(&pyq.id, 999).unwrap().unwrap();
    assert_eq!(inspection.source_pyq.id, pyq.id);
    assert_eq!(inspection.mapping.as_ref().unwrap().status, MappingStatus::Unreviewed);
    assert!(inspection.validator_passed);

    // 2. Review action: Approve
    service.review_pyq_mapping(&pyq.id, ReviewAction::Approve).unwrap();
    let approved_mapping = service.get_pyq_mapping(&pyq.id).unwrap().unwrap();
    assert_eq!(approved_mapping.status, MappingStatus::Verified);
    assert_eq!(approved_mapping.confidence, MappingConfidence::Deterministic);
    assert!(approved_mapping.is_eligible_for_practice());

    // 3. Review action: Remap
    let mut new_mapping = approved_mapping.clone();
    new_mapping.difficulty_level = 4;
    new_mapping.variant_structure = Some("trap".to_string());
    service.review_pyq_mapping(&pyq.id, ReviewAction::Remap { mapping: new_mapping }).unwrap();

    let remapped = service.get_pyq_mapping(&pyq.id).unwrap().unwrap();
    assert_eq!(remapped.difficulty_level, 4);
    assert_eq!(remapped.variant_structure.as_deref(), Some("trap"));
}

#[test]
fn test_domain_variant_taxonomy_per_domain() {
    // Maths
    assert!(PyqVariantPipeline::is_variant_supported_for_domain(&Domain::Mathematics, "numerical"));
    assert!(PyqVariantPipeline::is_variant_supported_for_domain(&Domain::Mathematics, "structural"));
    assert!(PyqVariantPipeline::is_variant_supported_for_domain(&Domain::Mathematics, "reverse"));
    assert!(PyqVariantPipeline::is_variant_supported_for_domain(&Domain::Mathematics, "trap"));
    assert!(PyqVariantPipeline::is_variant_supported_for_domain(&Domain::Mathematics, "transfer"));

    // Physics
    assert!(PyqVariantPipeline::is_variant_supported_for_domain(&Domain::Physics, "parameter"));
    assert!(PyqVariantPipeline::is_variant_supported_for_domain(&Domain::Physics, "initial_condition"));
    assert!(PyqVariantPipeline::is_variant_supported_for_domain(&Domain::Physics, "representation"));
    assert!(PyqVariantPipeline::is_variant_supported_for_domain(&Domain::Physics, "model_selection"));
    assert!(PyqVariantPipeline::is_variant_supported_for_domain(&Domain::Physics, "transfer"));

    // Chemistry
    assert!(PyqVariantPipeline::is_variant_supported_for_domain(&Domain::Chemistry, "quantity"));
    assert!(PyqVariantPipeline::is_variant_supported_for_domain(&Domain::Chemistry, "species"));
    assert!(PyqVariantPipeline::is_variant_supported_for_domain(&Domain::Chemistry, "regime"));
    assert!(PyqVariantPipeline::is_variant_supported_for_domain(&Domain::Chemistry, "constraint"));
    assert!(PyqVariantPipeline::is_variant_supported_for_domain(&Domain::Chemistry, "transfer"));

    // Reasoning
    assert!(PyqVariantPipeline::is_variant_supported_for_domain(&Domain::Reasoning, "entity"));
    assert!(PyqVariantPipeline::is_variant_supported_for_domain(&Domain::Reasoning, "constraint"));
    assert!(PyqVariantPipeline::is_variant_supported_for_domain(&Domain::Reasoning, "strategy"));
    assert!(PyqVariantPipeline::is_variant_supported_for_domain(&Domain::Reasoning, "structural"));
    assert!(PyqVariantPipeline::is_variant_supported_for_domain(&Domain::Reasoning, "transfer"));
}
