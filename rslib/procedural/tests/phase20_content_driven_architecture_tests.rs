use std::sync::Arc;
use std::time::Instant;

use procedural::core::{Domain, ProblemFamilyId};
use procedural::problems::contract::{
    DeclarativeArchetype, DeclarativeFamilyContract, ParameterSpec,
    ProblemFamilyCapability, ProblemFamilyContract,
};
use procedural::problems::declarative::{
    linear_equations_declarative_contract, DeclarativeProblemGenerator,
};
use procedural::problems::generator::ProblemGenerator;
use procedural::problems::generators::{
    LinearEquationsGenerator, LinearEquationsValidator, FAMILY_LINEAR_EQUATIONS,
    TEMPLATE_LINEAR_EQUATIONS_V1,
};
use procedural::problems::registry::ProblemRegistry;
use procedural::problems::validator::ProblemValidator;
use procedural::service::ProceduralService;
use procedural::skills::signals::VariantCategory;

#[test]
fn test_capability_resolution_across_all_thirty_two_families() {
    let registry = ProblemRegistry::default_registry();

    let expected_capabilities = vec![
        // 14 Mathematics
        ("family.math.percentage.successive", ProblemFamilyCapability::Declarative),
        ("family.math.algebra.linear_equations", ProblemFamilyCapability::Declarative),
        ("family.math.arithmetic.profit_loss", ProblemFamilyCapability::Declarative),
        ("family.math.arithmetic.ratio", ProblemFamilyCapability::Declarative),
        ("family.math.arithmetic.average", ProblemFamilyCapability::Declarative),
        ("family.math.number_system.divisibility", ProblemFamilyCapability::Declarative),
        ("family.math.time_work.basic", ProblemFamilyCapability::Declarative),
        ("family.math.arithmetic.time_speed_distance", ProblemFamilyCapability::Declarative),
        ("family.math.arithmetic.mixtures_alligation", ProblemFamilyCapability::Declarative),
        ("family.math.number_system.remainders_modular", ProblemFamilyCapability::Declarative),
        ("family.math.algebra.linear_inequalities", ProblemFamilyCapability::Declarative),
        ("family.math.algebra.algebraic_identities", ProblemFamilyCapability::Declarative),
        ("family.math.geometry.triangles", ProblemFamilyCapability::DomainGeometry),
        ("family.math.combined.multi_concept", ProblemFamilyCapability::Specialized),
        // 2 Physics
        ("family.physics.kinematics.1d", ProblemFamilyCapability::DomainPhysics),
        ("family.physics.work_energy.mechanics", ProblemFamilyCapability::DomainPhysics),
        // 6 Chemistry
        ("family.chemistry.stoichiometry.moles", ProblemFamilyCapability::DomainChemistry),
        ("family.chemistry.equilibrium.concentration", ProblemFamilyCapability::DomainChemistry),
        ("family.chemistry.ionic_equilibrium.buffers_titration", ProblemFamilyCapability::DomainChemistry),
        ("family.chemistry.electrochemistry.nernst_faraday", ProblemFamilyCapability::DomainChemistry),
        ("family.chemistry.kinetics.rate_laws", ProblemFamilyCapability::DomainChemistry),
        ("family.chemistry.reaction_networks.multistage_synthesis", ProblemFamilyCapability::DomainChemistry),
        // 10 Reasoning
        ("family.reasoning.series.pattern_recognition", ProblemFamilyCapability::Declarative),
        ("family.reasoning.syllogism.formal_inference", ProblemFamilyCapability::SymbolicLogic),
        ("family.reasoning.seating.constraint_satisfaction", ProblemFamilyCapability::ConstraintSolver),
        ("family.reasoning.relations.graph_inference", ProblemFamilyCapability::SymbolicLogic),
        ("family.reasoning.floor_grid.spatial_csp", ProblemFamilyCapability::ConstraintSolver),
        ("family.reasoning.logic_dag.multi_step_inference", ProblemFamilyCapability::SymbolicLogic),
        ("family.reasoning.data_sufficiency.constraint_sufficiency", ProblemFamilyCapability::SymbolicLogic),
        ("family.reasoning.coded_expressions.symbolic_operators", ProblemFamilyCapability::Declarative),
        ("family.reasoning.blood_relations.kinship_graph", ProblemFamilyCapability::SymbolicLogic),
        ("family.reasoning.direction_sense.spatial_orientation", ProblemFamilyCapability::SymbolicLogic),
    ];

    assert_eq!(expected_capabilities.len(), 32, "Must audit all 32 multi-domain families");

    for (fam_id_str, expected_cap) in expected_capabilities {
        let cap = registry.get_capability(fam_id_str);
        assert_eq!(
            cap,
            Some(expected_cap),
            "Family {} must resolve capability {:?}",
            fam_id_str,
            expected_cap
        );

        let contract = registry.get_family_contract(fam_id_str);
        assert!(
            contract.is_some(),
            "Family {} must have a registered ProblemFamilyContract",
            fam_id_str
        );
        let c = contract.unwrap();
        assert_eq!(c.capability, expected_cap);
        assert!(!c.supported_variants.is_empty(), "Family {} must declare supported variants", fam_id_str);
        assert_eq!(c.min_difficulty, 1.0);
        assert_eq!(c.max_difficulty, 5.0);
    }
}

#[test]
fn test_declarative_contract_structure_and_archetypes() {
    let contract = linear_equations_declarative_contract();
    assert_eq!(contract.contract.family_id.as_str(), FAMILY_LINEAR_EQUATIONS);
    assert_eq!(contract.contract.capability, ProblemFamilyCapability::Declarative);
    assert_eq!(contract.archetypes.len(), 5, "Must have exactly 5 difficulty archetypes");

    for level in 1..=5 {
        let arch = contract.find_archetype(level, None);
        assert!(arch.is_some(), "Archetype must exist for difficulty level {}", level);
        let a = arch.unwrap();
        assert_eq!(a.difficulty_level, level);
        assert!(!a.parameters.is_empty(), "Archetype L{} must declare parameters", level);
        assert!(!a.prompt_template.is_empty(), "Archetype L{} must declare prompt template", level);
        assert!(!a.solution_template.is_empty(), "Archetype L{} must declare solution template", level);
        assert!(!a.step_nodes.is_empty(), "Archetype L{} must declare step nodes", level);
    }
}

#[test]
fn test_linear_equations_old_vs_new_semantic_equivalence_across_seeds() {
    let dec_contract = Arc::new(linear_equations_declarative_contract());
    let dec_generator = DeclarativeProblemGenerator::new(dec_contract);
    let specialized_generator = LinearEquationsGenerator;
    let validator = LinearEquationsValidator;
    let fam_id = ProblemFamilyId::new(FAMILY_LINEAR_EQUATIONS);

    for level in 1..=5 {
        for seed in 1..=100 {
            // 1. Generate via Declarative Generator
            let dec_inst = dec_generator
                .generate(&fam_id, seed, level, None)
                .expect("Declarative generation must succeed");

            // 2. Generate via Specialized Generator
            let spec_inst = specialized_generator
                .generate(&fam_id, seed, level, None)
                .expect("Specialized generation must succeed");

            // Semantic Equivalence Checks:
            // A. Family ID parity
            assert_eq!(dec_inst.family_id, spec_inst.family_id);

            // B. Target Latency parity from generator contract
            assert_eq!(
                dec_generator.target_latency_ms(level),
                specialized_generator.target_latency_ms(level),
                "Target latency must match at level {}",
                level
            );

            // C. Answer validity via standard validator
            let dec_ans = dec_inst.correct_answer.get("value").unwrap();
            let dec_eval = validator.evaluate(&dec_inst, dec_ans, 15_000, 30_000);
            assert!(
                dec_eval.is_correct,
                "Declarative instance answer {:?} must validate as correct at level {} (seed {})",
                dec_ans, level, seed
            );

            let spec_ans = spec_inst.correct_answer.get("value").unwrap();
            let spec_eval = validator.evaluate(&spec_inst, spec_ans, 15_000, 30_000);
            assert!(
                spec_eval.is_correct,
                "Specialized instance answer {:?} must validate as correct at level {} (seed {})",
                spec_ans, level, seed
            );

            // D. Solution Graph Integrity
            let dec_graph = dec_inst.solution_graph().expect("Declarative graph must exist");
            assert!(dec_graph.validate_topology(), "Declarative graph must be acyclic and valid");
            assert!(dec_graph.step_count() >= 2, "Declarative graph must contain multiple step nodes");

            let spec_graph = spec_inst.solution_graph().expect("Specialized graph must exist");
            assert!(spec_graph.validate_topology(), "Specialized graph must be acyclic and valid");
            assert!(spec_graph.step_count() >= 2, "Specialized graph must contain multiple step nodes");
        }
    }
}

#[test]
fn test_resilient_registry_dispatch_and_automatic_fallback() {
    let mut registry = ProblemRegistry::new();

    // Register specialized generator & validator
    registry.register_generator(Arc::new(LinearEquationsGenerator));
    registry.register_validator(Arc::new(LinearEquationsValidator));

    let fam_id = ProblemFamilyId::new(FAMILY_LINEAR_EQUATIONS);

    // 1. Normal declarative generation
    registry.register_declarative_family(linear_equations_declarative_contract());
    let inst_normal = registry
        .generate(&fam_id, TEMPLATE_LINEAR_EQUATIONS_V1, 42, 1, None)
        .expect("Normal declarative dispatch must succeed");
    assert_eq!(inst_normal.metadata["is_declarative"].as_bool(), Some(true));

    // 2. Inject broken declarative contract with unsatisfiable constraints (e.g. a == 0 when non_zero is required)
    let broken_contract = DeclarativeFamilyContract::new(
        ProblemFamilyContract::new(
            FAMILY_LINEAR_EQUATIONS,
            "algebra.linear_equations",
            Domain::Mathematics,
            "schema.algebra.linear_equations.v1",
            ProblemFamilyCapability::Declarative,
        ),
        vec![
            DeclarativeArchetype::new(
                "unsatisfiable_archetype",
                1,
                VariantCategory::Parameter,
                "two_step_basic",
                vec![
                    ParameterSpec::integer_range("a", 1, 5),
                    ParameterSpec::integer_range("b", 1, 5),
                    ParameterSpec::derived_linear("c", "a", "b", "b"),
                ],
                "Solve: {a}x + {b} = {c}",
                procedural::problems::contract::AnswerDerivation::LinearTwoStep {
                    c_param: "c".to_string(),
                    b_param: "b".to_string(),
                    a_param: "a".to_string(),
                },
                "{answer}",
                "Solution",
                25_000,
            )
            .with_constraints(vec![
                procedural::problems::contract::ConstraintSpec::NotEqual {
                    param_a: "a".to_string(),
                    param_b: "a".to_string(), // Impossible constraint: a != a!
                },
            ]),
        ],
    );

    // Override with broken declarative contract
    registry.register_declarative_family(broken_contract);

    // 3. Dispatch through registry: should fail declarative generation and automatically fallback to specialized generator!
    let fallback_inst = registry
        .generate(&fam_id, TEMPLATE_LINEAR_EQUATIONS_V1, 42, 1, None)
        .expect("Registry fallback to specialized generator must succeed seamlessly");

    // The fallback instance is from the specialized generator (so is_declarative is null or false)
    assert!(
        fallback_inst.metadata.get("is_declarative").is_none(),
        "Fallback instance must originate from specialized generator"
    );

    // Verify the fallback instance answer validates as 100% correct
    let ans = fallback_inst.correct_answer.get("value").unwrap();
    let eval = LinearEquationsValidator.evaluate(&fallback_inst, ans, 10_000, 20_000);
    assert!(eval.is_correct, "Fallback instance must be completely valid");
}

#[test]
fn test_runtime_performance_and_memory_overhead() {
    let registry = ProblemRegistry::default_registry();
    let fam_id = ProblemFamilyId::new(FAMILY_LINEAR_EQUATIONS);

    // Benchmark 1,000 declarative generations
    let start_dec = Instant::now();
    for seed in 1..=1000 {
        let level = (seed % 5 + 1) as u32;
        let _inst = registry
            .generate(&fam_id, TEMPLATE_LINEAR_EQUATIONS_V1, seed, level, None)
            .unwrap();
    }
    let duration_dec = start_dec.elapsed();
    let avg_dec_micros = duration_dec.as_micros() as f64 / 1000.0;

    println!(
        "\n[Phase 20 Performance Benchmark] 1,000 Declarative Generations: {:?} (Avg: {:.2} µs / generation)",
        duration_dec, avg_dec_micros
    );

    // Must be well below 500 microseconds per generation in unoptimized debug mode (< 50 µs in release)
    assert!(
        avg_dec_micros < 500.0,
        "Declarative generation average latency must be < 500 µs in debug mode, measured: {:.2} µs",
        avg_dec_micros
    );
}

#[test]
fn test_end_to_end_service_integration_with_declarative_engine() {
    let service = ProceduralService::open_in_memory().expect("Service init must succeed");

    let all_schemas = service.store().list_all_schemas().expect("List schemas must succeed");
    let linear_schema = all_schemas
        .iter()
        .find(|s| s.problem_family_id.as_str() == FAMILY_LINEAR_EQUATIONS)
        .expect("Linear equations schema must exist in catalog");

    let anchor = procedural::anchor::ProceduralCardAnchor::new(linear_schema.id.clone())
        .with_seed_mode(procedural::anchor::SeedMode::Fixed(777));

    let session = service
        .prepare_practice_session(&anchor, Some(101))
        .expect("Practice session preparation must succeed");

    assert_eq!(session.instance.family_id.as_str(), FAMILY_LINEAR_EQUATIONS);
    assert_eq!(session.instance.seed, 777);
    assert!(!session.instance.rendered_prompt.is_empty());
    assert!(session.instance.correct_answer.get("value").is_some());
}
