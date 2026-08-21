// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Phase 21: Content Grounding & Release Coverage Automated Verification Suite

use std::collections::HashSet;
use procedural::anchor::{ProceduralCardAnchor, SeedMode};
use procedural::core::ProblemFamilyId;
use procedural::problems::catalog::*;
use procedural::problems::generators::*;
use procedural::problems::registry::ProblemRegistry;
use procedural::service::ProceduralService;

#[test]
fn test_32_family_release_matrix_and_staging_coverage() {
    let registry = ProblemRegistry::default_registry();

    let v1_families = vec![
        FAMILY_PERCENTAGE_SUCCESSIVE,
        FAMILY_LINEAR_EQUATIONS,
        FAMILY_PROFIT_LOSS,
        FAMILY_RATIO,
        FAMILY_AVERAGE,
        FAMILY_DIVISIBILITY,
        FAMILY_TIME_WORK,
        FAMILY_TIME_SPEED_DISTANCE,
        FAMILY_REMAINDERS_MODULAR,
        FAMILY_ALGEBRAIC_IDENTITIES,
        FAMILY_REASONING_SERIES,
        FAMILY_REASONING_SYLLOGISM,
        FAMILY_REASONING_SEATING,
        procedural::physics::generators::FAMILY_PHYSICS_KINEMATICS,
    ];

    let v1_1_families = vec![
        FAMILY_MIXTURES_ALLIGATION,
        FAMILY_GEOMETRY_TRIANGLES,
        FAMILY_LINEAR_INEQUALITIES,
        FAMILY_REASONING_BLOOD_RELATIONS,
        FAMILY_REASONING_DIRECTION_SENSE,
        FAMILY_REASONING_DATA_SUFFICIENCY,
        FAMILY_REASONING_CODED_EXPRESSIONS,
        procedural::physics::generators::FAMILY_PHYSICS_WORK_ENERGY,
        procedural::chemistry::generators::FAMILY_CHEMISTRY_STOICHIOMETRY,
        procedural::chemistry::generators::FAMILY_CHEMISTRY_EQUILIBRIUM,
    ];

    let deferred_families = vec![
        FAMILY_COMBINED_MULTI_CONCEPT,
        FAMILY_REASONING_RELATIONS,
        FAMILY_REASONING_FLOOR_GRID,
        FAMILY_REASONING_LOGIC_DAG,
        procedural::chemistry::generators::FAMILY_CHEMISTRY_BUFFERS_TITRATION,
        procedural::chemistry::generators::FAMILY_CHEMISTRY_ELECTROCHEMISTRY,
        procedural::chemistry::generators::FAMILY_CHEMISTRY_KINETICS,
        procedural::chemistry::generators::FAMILY_CHEMISTRY_REACTION_NETWORKS,
    ];

    assert_eq!(v1_families.len(), 14, "V1 Core set must contain exactly 14 families");
    assert_eq!(v1_1_families.len(), 10, "V1.1 Near-ready set must contain exactly 10 families");
    assert_eq!(deferred_families.len(), 8, "Deferred set must contain exactly 8 families");

    let mut total_set = HashSet::new();
    for f in &v1_families {
        assert!(total_set.insert(*f), "Duplicate family in V1: {}", f);
    }
    for f in &v1_1_families {
        assert!(total_set.insert(*f), "Duplicate family in V1.1: {}", f);
    }
    for f in &deferred_families {
        assert!(total_set.insert(*f), "Duplicate family in Deferred: {}", f);
    }

    assert_eq!(total_set.len(), 32, "Total staged families must equal exactly 32");

    // Verify all 32 families have registered contracts, generators and validators
    for fam_id_str in total_set {
        let generator = registry.get_generator(fam_id_str);
        assert!(
            generator.is_some(),
            "Family {} must have a registered generator",
            fam_id_str
        );

        let validator = registry.get_validator(fam_id_str);
        assert!(
            validator.is_some(),
            "Family {} must have a registered validator",
            fam_id_str
        );
    }
}

#[test]
fn test_v1_release_set_generation_and_validation_sanity() {
    let registry = ProblemRegistry::default_registry();

    let v1_test_specs = vec![
        (FAMILY_PERCENTAGE_SUCCESSIVE, TEMPLATE_PERCENTAGE_SUCCESSIVE_V1, 4),
        (FAMILY_LINEAR_EQUATIONS, TEMPLATE_LINEAR_EQUATIONS_V1, 5),
        (FAMILY_PROFIT_LOSS, TEMPLATE_PROFIT_LOSS_V1, 5),
        (FAMILY_RATIO, TEMPLATE_RATIO_V1, 5),
        (FAMILY_AVERAGE, TEMPLATE_AVERAGE_V1, 5),
        (FAMILY_DIVISIBILITY, TEMPLATE_DIVISIBILITY_V1, 5),
        (FAMILY_TIME_WORK, TEMPLATE_TIME_WORK_V1, 5),
        (FAMILY_TIME_SPEED_DISTANCE, TEMPLATE_TIME_SPEED_DISTANCE_V1, 5),
        (FAMILY_REMAINDERS_MODULAR, TEMPLATE_REMAINDERS_MODULAR_V1, 4),
        (FAMILY_ALGEBRAIC_IDENTITIES, TEMPLATE_ALGEBRAIC_IDENTITIES_V1, 5),
        (FAMILY_REASONING_SERIES, TEMPLATE_REASONING_SERIES_V1, 5),
        (FAMILY_REASONING_SYLLOGISM, TEMPLATE_REASONING_SYLLOGISM_V1, 4),
        (FAMILY_REASONING_SEATING, TEMPLATE_REASONING_SEATING_V1, 5),
        (procedural::physics::generators::FAMILY_PHYSICS_KINEMATICS, procedural::physics::generators::TEMPLATE_PHYSICS_KINEMATICS_V1, 5),
    ];

    for (fam_id_str, template_ref, max_level) in v1_test_specs {
        let fam_id = ProblemFamilyId::new(fam_id_str);
        let validator = registry.get_validator(fam_id_str).expect("validator");

        for level in 1..=max_level {
            for seed in (100..105).chain(200..205) {
                let instance = registry
                    .generate(&fam_id, template_ref, seed, level, None)
                    .unwrap_or_else(|e| panic!("Failed generation for {} at L{}: {:?}", fam_id_str, level, e));

                assert!(!instance.rendered_prompt.is_empty());
                assert!(instance.correct_answer.get("value").is_some());

                let ans_val = instance.correct_answer.get("value").unwrap();
                let eval = validator.evaluate(&instance, ans_val, 15_000, 35_000);
                assert!(
                    eval.is_correct,
                    "Generated answer must validate for {} at L{} (seed {})",
                    fam_id_str, level, seed
                );
            }
        }
    }
}

#[test]
fn test_apkg_compact_payload_compliance() {
    let service = ProceduralService::open_in_memory().expect("open service");

    // 1. Valid compact card note payload
    let valid_fields = vec![
        "What is the final value?".to_string(),
        r#"{"proc_schema": "percentage.successive", "seed_mode": {"fixed": 42}, "difficulty_hint": 2.0}"#.to_string(),
        "Answer".to_string(),
    ];

    let anchor = ProceduralCardAnchor::extract_from_card_fields(&valid_fields)
        .expect("extract anchor")
        .expect("anchor must exist");

    assert_eq!(anchor.proc_schema.as_str(), "percentage.successive");
    assert_eq!(anchor.seed_mode, SeedMode::Fixed(42));

    let session = service.prepare_practice_session(&anchor, Some(101)).expect("prepare session");
    assert_eq!(session.schema.id.as_str(), "successive_percentage");
    assert_eq!(session.card_id, Some(101));

    // 2. Note fields without procedural metadata return None gracefully
    let regular_anki_card = vec![
        "Capital of France?".to_string(),
        "Paris".to_string(),
    ];
    let non_proc = ProceduralCardAnchor::extract_from_card_fields(&regular_anki_card).expect("regular card parse");
    assert!(non_proc.is_none());
}
