// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use procedural::core::{ProblemFamilyId, SchemaId, SkillId};
use procedural::problems::catalog::{
    MathsCatalog, FAMILY_REASONING_BLOOD_RELATIONS, FAMILY_REASONING_CODED_EXPRESSIONS,
    FAMILY_REASONING_DATA_SUFFICIENCY, FAMILY_REASONING_DIRECTION_SENSE,
    FAMILY_REASONING_FLOOR_GRID, FAMILY_REASONING_LOGIC_DAG, FAMILY_REASONING_RELATIONS,
    FAMILY_REASONING_SEATING, FAMILY_REASONING_SERIES, FAMILY_REASONING_SYLLOGISM,
    SCHEMA_REASONING_BLOOD_RELATIONS, SCHEMA_REASONING_DIRECTION_SENSE,
    SKILL_REASONING_BLOOD_RELATIONS, SKILL_REASONING_DIRECTION_SENSE,
};
use procedural::problems::generator::ProblemGenerator;
use procedural::problems::registry::ProblemRegistry;
use procedural::problems::validator::ProblemValidator;
use procedural::reasoning::generators::{
    BloodRelationsGenerator, BloodRelationsValidator, CodedExpressionsGenerator,
    DataSufficiencyGenerator, DirectionSenseGenerator, DirectionSenseValidator, FloorGridGenerator,
    LogicDagGenerator, RelationsGenerator, SeatingGenerator, SeriesGenerator, SyllogismGenerator,
};
use procedural::service::ProceduralService;
use procedural::ProceduralStore;

#[test]
fn test_blood_relations_generator_isolation_and_progression() {
    let generator = BloodRelationsGenerator;
    let validator = BloodRelationsValidator;
    let family_id = ProblemFamilyId::from(FAMILY_REASONING_BLOOD_RELATIONS);

    for diff in 1..=5 {
        for seed in 100..110 {
            let instance = generator
                .generate(&family_id, seed, diff, None)
                .expect("Generation should succeed");

            assert_eq!(instance.family_id.as_str(), FAMILY_REASONING_BLOOD_RELATIONS);
            assert!(
                instance.rendered_prompt.contains("related to"),
                "Prompt must be a blood relations question: {}",
                instance.rendered_prompt
            );
            assert!(
                !instance.rendered_prompt.contains("starts from a fixed point"),
                "Blood relations must NEVER contain direction sense steps"
            );

            // Verify reasoning metadata schema is BloodRelations
            let meta = instance
                .parameters
                .get("reasoning_metadata")
                .expect("Metadata must be present");
            assert_eq!(meta["schema_kind"], "blood_relations");

            // Verify target latency synchronization
            let expected_latency = BloodRelationsGenerator::target_latency(diff);
            assert_eq!(generator.target_latency_ms(diff), expected_latency);
            assert_eq!(
                instance.metadata["target_time_ms"].as_u64().unwrap(),
                expected_latency,
                "Instance metadata target_time_ms must equal target_latency_ms"
            );

            // Verify validator evaluates correct answer
            let answer = instance.correct_answer.get("value").unwrap();
            let eval = validator.evaluate(&instance, answer, 15_000, expected_latency);
            assert!(eval.is_correct, "Correct answer must validate: {:?}", answer);
        }
    }
}

#[test]
fn test_direction_sense_generator_isolation_and_progression() {
    let generator = DirectionSenseGenerator;
    let validator = DirectionSenseValidator;
    let family_id = ProblemFamilyId::from(FAMILY_REASONING_DIRECTION_SENSE);

    for diff in 1..=5 {
        for seed in 200..210 {
            let instance = generator
                .generate(&family_id, seed, diff, None)
                .expect("Generation should succeed");

            assert_eq!(instance.family_id.as_str(), FAMILY_REASONING_DIRECTION_SENSE);
            assert!(
                instance.rendered_prompt.contains("starts from a fixed point")
                    || instance.rendered_prompt.contains("direction"),
                "Prompt must be a direction sense question: {}",
                instance.rendered_prompt
            );
            assert!(
                !instance.rendered_prompt.contains("related to"),
                "Direction sense must NEVER contain blood relation questions"
            );

            // Verify reasoning metadata schema is DirectionSense
            let meta = instance
                .parameters
                .get("reasoning_metadata")
                .expect("Metadata must be present");
            assert_eq!(meta["schema_kind"], "direction_sense");

            // Verify target latency synchronization
            let expected_latency = DirectionSenseGenerator::target_latency(diff);
            assert_eq!(generator.target_latency_ms(diff), expected_latency);
            assert_eq!(
                instance.metadata["target_time_ms"].as_u64().unwrap(),
                expected_latency,
                "Instance metadata target_time_ms must equal target_latency_ms"
            );

            // Verify validator evaluates correct answer
            let answer = instance.correct_answer.get("value").unwrap();
            let eval = validator.evaluate(&instance, answer, 15_000, expected_latency);
            assert!(eval.is_correct, "Correct answer must validate: {:?}", answer);
        }
    }
}

#[test]
fn test_syllogism_deterministic_difficulty_contract() {
    let generator = SyllogismGenerator;
    let family_id = ProblemFamilyId::from(FAMILY_REASONING_SYLLOGISM);

    for seed in 300..310 {
        // Level 1: Barbara AAA-1 -> Both I and II follow
        let inst_l1 = generator.generate(&family_id, seed, 1, None).unwrap();
        assert_eq!(
            inst_l1.correct_answer["value"], "Both I and II follow",
            "L1 Syllogism must generate Barbara (Both follow)"
        );

        // Level 2: Celarent / Camestres -> Only I follows
        let inst_l2 = generator.generate(&family_id, seed, 2, None).unwrap();
        assert_eq!(
            inst_l2.correct_answer["value"], "Only I follows",
            "L2 Syllogism must generate Celarent/Camestres (Only I follows)"
        );

        // Level 3: Darii / Only II follows
        let inst_l3 = generator.generate(&family_id, seed, 3, None).unwrap();
        let val_l3 = inst_l3.correct_answer["value"].as_str().unwrap();
        assert!(
            val_l3 == "Only I follows" || val_l3 == "Only II follows",
            "L3 Syllogism must generate particular affirmative (Only I or Only II follows), got: {}",
            val_l3
        );

        // Level 4: Ferio -> Only I follows (particular negative)
        let inst_l4 = generator.generate(&family_id, seed, 4, None).unwrap();
        assert_eq!(
            inst_l4.correct_answer["value"], "Only I follows",
            "L4 Syllogism must generate Ferio (Only I follows)"
        );

        // Level 5: Disjoint / Fallacy of undistributed middle -> Neither follows
        let inst_l5 = generator.generate(&family_id, seed, 5, None).unwrap();
        assert_eq!(
            inst_l5.correct_answer["value"], "Neither follows",
            "L5 Syllogism must generate disjoint/invalid (Neither follows)"
        );
    }
}

#[test]
fn test_reasoning_target_latency_synchronization_across_all_generators() {
    let generators: Vec<(Box<dyn ProblemGenerator>, &str)> = vec![
        (Box::new(BloodRelationsGenerator), FAMILY_REASONING_BLOOD_RELATIONS),
        (Box::new(DirectionSenseGenerator), FAMILY_REASONING_DIRECTION_SENSE),
        (Box::new(RelationsGenerator), FAMILY_REASONING_RELATIONS),
        (Box::new(SyllogismGenerator), FAMILY_REASONING_SYLLOGISM),
        (Box::new(FloorGridGenerator), FAMILY_REASONING_FLOOR_GRID),
        (Box::new(LogicDagGenerator), FAMILY_REASONING_LOGIC_DAG),
        (Box::new(DataSufficiencyGenerator), FAMILY_REASONING_DATA_SUFFICIENCY),
        (Box::new(CodedExpressionsGenerator), FAMILY_REASONING_CODED_EXPRESSIONS),
        (Box::new(SeatingGenerator), FAMILY_REASONING_SEATING),
        (Box::new(SeriesGenerator), FAMILY_REASONING_SERIES),
    ];

    for (gen, family_str) in generators {
        let family_id = ProblemFamilyId::from(family_str);
        let mut prev_latency = 0;

        for diff in 1..=5 {
            let latency = gen.target_latency_ms(diff);
            assert!(
                latency >= prev_latency,
                "Latency must be monotonically non-decreasing for family {}: L{} ({}ms) vs L{} ({}ms)",
                family_str,
                diff,
                latency,
                diff - 1,
                prev_latency
            );
            prev_latency = latency;

            let inst = gen
                .generate(&family_id, 42, diff, None)
                .expect("Generation must succeed");
            let meta_target = inst.metadata["target_time_ms"].as_u64().unwrap();

            assert_eq!(
                meta_target, latency,
                "Instance target_time_ms must be synchronized with generator.target_latency_ms for family {} at L{}",
                family_str, diff
            );
        }
    }
}

#[test]
fn test_catalog_and_registry_full_registration() {
    let store = ProceduralStore::open_in_memory().unwrap();
    MathsCatalog::init_all(&store).expect("Catalog initialization must succeed");

    // Verify blood relations and direction sense are in store
    assert!(store.get_skill(&SkillId::from(SKILL_REASONING_BLOOD_RELATIONS)).unwrap().is_some());
    assert!(store.get_skill(&SkillId::from(SKILL_REASONING_DIRECTION_SENSE)).unwrap().is_some());
    assert!(store.get_problem_family(&ProblemFamilyId::from(FAMILY_REASONING_BLOOD_RELATIONS)).unwrap().is_some());
    assert!(store.get_problem_family(&ProblemFamilyId::from(FAMILY_REASONING_DIRECTION_SENSE)).unwrap().is_some());

    // Verify registry contains both generators and validators
    let registry = ProblemRegistry::default_registry();
    let gen_blood = registry.get_generator(FAMILY_REASONING_BLOOD_RELATIONS);
    assert!(gen_blood.is_some(), "BloodRelationsGenerator must be registered");
    let val_blood = registry.get_validator(FAMILY_REASONING_BLOOD_RELATIONS);
    assert!(val_blood.is_some(), "BloodRelationsValidator must be registered");

    let gen_dir = registry.get_generator(FAMILY_REASONING_DIRECTION_SENSE);
    assert!(gen_dir.is_some(), "DirectionSenseGenerator must be registered");
    let val_dir = registry.get_validator(FAMILY_REASONING_DIRECTION_SENSE);
    assert!(val_dir.is_some(), "DirectionSenseValidator must be registered");

    // Verify service schema resolution
    let service = ProceduralService::new(store);
    let resolved_blood = service.resolve_schema(&SchemaId::from("reasoning.blood_relations")).unwrap();
    assert!(resolved_blood.is_some());
    assert_eq!(resolved_blood.unwrap().id.as_str(), SCHEMA_REASONING_BLOOD_RELATIONS);

    let resolved_dir = service.resolve_schema(&SchemaId::from("reasoning.direction_sense")).unwrap();
    assert!(resolved_dir.is_some());
    assert_eq!(resolved_dir.unwrap().id.as_str(), SCHEMA_REASONING_DIRECTION_SENSE);
}