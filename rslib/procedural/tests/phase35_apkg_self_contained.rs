// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use procedural::anchor::{ProceduralCardAnchor, SeedMode};
use procedural::core::Domain;
use procedural::problems::contract::{
    AnswerDerivation, DeclarativeArchetype, DeclarativeFamilyContract,
    ParameterSpec, ProblemFamilyCapability, ProblemFamilyContract, StepNodeSpec,
};
use procedural::problems::steps::StepType;
use procedural::reviewer::render_reviewer_html;
use procedural::service::ProceduralService;
use procedural::skills::signals::VariantCategory;

#[test]
fn test_apkg_self_contained_zero_pre_seeding() {
    // 1. Create fresh completely empty in-memory service
    let service = ProceduralService::open_in_memory().expect("failed to create in-memory service");

    // 2. Construct a rich declarative contract
    let rich_contract = DeclarativeFamilyContract::new(
        ProblemFamilyContract::new(
            "family.math.arithmetic.unit_conversion_speed",
            "skill.math.arithmetic.speed",
            Domain::Mathematics,
            "schema.math.arithmetic.unit_conversion_speed",
            ProblemFamilyCapability::Declarative,
        )
        .with_difficulty_range(1.0, 5.0),
        vec![DeclarativeArchetype::new(
            "arch_speed_kmh_to_ms",
            1,
            VariantCategory::Parameter,
            "kmh_to_ms",
            vec![
                ParameterSpec::integer_range("speed_kmh", 18, 180),
                ParameterSpec::derived_quotient("speed_ms", "speed_kmh", "conversion_factor", Some(2)),
                ParameterSpec::integer_range("conversion_factor", 3, 3), // (approx 3.6, integer step test)
            ],
            "Convert a speed of {speed_kmh} km/h into m/s (using division factor {conversion_factor}).",
            AnswerDerivation::Quotient {
                numerator_param: "speed_kmh".into(),
                denominator_param: "conversion_factor".into(),
            },
            "{answer} m/s",
            "Speed in m/s = {speed_kmh} / {conversion_factor} = {answer} m/s.",
            25_000,
        )
        .with_step_nodes(vec![
            StepNodeSpec::new(
                "step_div",
                StepType::Arithmetic,
                "Perform Division",
                "Divide {speed_kmh} by {conversion_factor}.",
                "{answer}",
                vec![],
                "Dividing km/h by conversion factor yields m/s.",
                "Execute the quotient.",
                "{speed_kmh} / {conversion_factor} = {answer}",
            ),
        ])],
    );

    // 3. Package as an inline contract anchor
    let anchor = ProceduralCardAnchor::new("schema.math.arithmetic.unit_conversion_speed")
        .with_seed_mode(SeedMode::Fixed(777))
        .with_difficulty_override(1.0)
        .with_inline_contract(rich_contract);

    let anchor_json = anchor.to_json_string().unwrap();

    // 4. Simulate card note fields in an imported APKG
    let card_fields = vec![
        "What is the speed conversion?".to_string(),
        anchor_json,
        "See answer on flip side.".to_string(),
    ];

    // 5. Extract anchor from card note fields
    let extracted = ProceduralCardAnchor::extract_from_card_fields(&card_fields)
        .expect("extraction failed")
        .expect("anchor missing in note fields");

    assert!(extracted.inline_contract.is_some());

    // 6. Direct Reviewer Resolution
    let session = service
        .resolve_procedural_target(&extracted, Some(3003))
        .expect("resolution on fresh profile failed");

    // 7. Verify session and HTML render
    assert_eq!(session.schema.id.as_str(), "schema.math.arithmetic.unit_conversion_speed");
    assert_eq!(session.target_latency_ms, Some(25_000));
    assert!(session.instance.solution_graph().is_some());

    let html = render_reviewer_html(&session);
    assert!(!html.is_empty());
    assert!(html.contains("studylab-card-container") || html.contains("problem-card") || html.contains("procedural"));

    println!("[SELF-CONTAINED APKG PASS] Card resolved and rendered on clean empty profile without pre-seeding!");
}

#[test]
fn test_legacy_apkg_compatibility() {
    let service = ProceduralService::open_in_memory().expect("failed to create in-memory service");

    // Legacy card note field containing only proc_schema
    let legacy_fields = vec![
        "Legacy Question".to_string(),
        r#"{"proc_schema":"math.percentage.successive"}"#.to_string(),
        "Legacy Answer".to_string(),
    ];

    let extracted = ProceduralCardAnchor::extract_from_card_fields(&legacy_fields)
        .expect("extraction failed")
        .expect("legacy anchor missing");

    assert_eq!(extracted.proc_schema.as_str(), "math.percentage.successive");
    assert!(extracted.inline_contract.is_none());
    assert!(extracted.content_ref.is_none());

    // Should resolve cleanly via legacy practice request path
    let session = service
        .resolve_procedural_target(&extracted, Some(4004))
        .expect("legacy card resolution failed");

    assert_eq!(session.schema.id.as_str(), "successive_percentage");
    assert!(!session.instance.rendered_prompt.is_empty());

    let html = render_reviewer_html(&session);
    assert!(!html.is_empty());

    println!("[LEGACY COMPATIBILITY PASS] Legacy proc_schema cards continue working with zero errors!");
}
