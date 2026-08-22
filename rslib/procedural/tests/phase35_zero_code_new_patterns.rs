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
fn test_zero_code_new_patterns_all_domains() {
    let service = ProceduralService::open_in_memory().expect("failed to create in-memory service");

    // -----------------------------------------------------------------------
    // New Pattern 1 (Math): 3-Number LCM Modular Remainders
    // -----------------------------------------------------------------------
    let new_math_contract = DeclarativeFamilyContract::new(
        ProblemFamilyContract::new(
            "family.math.number_system.lcm_modular_remainders",
            "skill.math.number_system.lcm_modular_remainders",
            Domain::Mathematics,
            "schema.math.number_system.lcm_modular_remainders",
            ProblemFamilyCapability::Declarative,
        )
        .with_difficulty_range(1.0, 5.0),
        vec![DeclarativeArchetype::new(
            "arch_lcm_remainder_3num",
            3,
            VariantCategory::Structural,
            "lcm_remainder_3num",
            vec![
                ParameterSpec::integer_range("d1", 6, 12),
                ParameterSpec::integer_range("d2", 15, 20),
                ParameterSpec::integer_range("d3", 24, 30),
                ParameterSpec::integer_range("remainder", 2, 5),
            ],
            "Find the smallest positive integer which when divided by {d1}, {d2}, and {d3} leaves a remainder of {remainder} in each case.",
            AnswerDerivation::LcmArray {
                params: vec!["d1".into(), "d2".into(), "d3".into()],
            },
            "LCM({d1}, {d2}, {d3}) + {remainder} = {answer}",
            "Required number = LCM({d1}, {d2}, {d3}) + remainder. LCM is {answer}.",
            45_000,
        )
        .with_step_nodes(vec![
            StepNodeSpec::new(
                "step_lcm",
                StepType::Arithmetic,
                "Compute 3-number LCM",
                "Calculate LCM({d1}, {d2}, {d3}).",
                "{answer}",
                vec![],
                "The common dividend before remainder is the LCM.",
                "Factorize the 3 numbers and take the max prime powers.",
                "LCM({d1}, {d2}, {d3}) = {answer}",
            ),
        ])],
    );

    // -----------------------------------------------------------------------
    // New Pattern 2 (Reasoning): Direction Sense Vector Displacement
    // -----------------------------------------------------------------------
    let new_reasoning_contract = DeclarativeFamilyContract::new(
        ProblemFamilyContract::new(
            "family.reasoning.direction_sense.vector_path",
            "skill.reasoning.direction_sense.vector_path",
            Domain::Reasoning,
            "schema.reasoning.direction_sense.vector_path",
            ProblemFamilyCapability::Declarative,
        )
        .with_difficulty_range(1.0, 5.0),
        vec![DeclarativeArchetype::new(
            "arch_direction_displacement",
            2,
            VariantCategory::Contextual,
            "direction_displacement",
            vec![
                ParameterSpec::permutation_choice("traveler", vec!["Rohan".into(), "Suresh".into(), "Priya".into()], 1),
                ParameterSpec::integer_range("north_dist", 10, 50),
                ParameterSpec::integer_range("turn_dist", 10, 50),
                ParameterSpec::derived_sum("total_path", "north_dist", "turn_dist"),
            ],
            "{traveler_0} walks {north_dist} meters North, then turns right and walks {turn_dist} meters East. What is the total distance walked by {traveler_0}?",
            AnswerDerivation::DirectParam {
                param_name: "total_path".into(),
            },
            "{answer} meters",
            "Total distance walked is the scalar path length: {north_dist}m + {turn_dist}m = {answer}m.",
            35_000,
        )],
    );

    // -----------------------------------------------------------------------
    // New Pattern 3 (Physics): Gravitational Maximum Height Projectile
    // -----------------------------------------------------------------------
    let new_physics_contract = DeclarativeFamilyContract::new(
        ProblemFamilyContract::new(
            "family.physics.kinematics.max_height",
            "skill.physics.kinematics.max_height",
            Domain::Physics,
            "schema.physics.kinematics.max_height",
            ProblemFamilyCapability::DomainPhysics,
        )
        .with_difficulty_range(1.0, 5.0),
        vec![DeclarativeArchetype::new(
            "arch_vertical_max_height",
            2,
            VariantCategory::Transfer,
            "vertical_max_height",
            vec![
                ParameterSpec::integer_range("u", 20, 60),
                ParameterSpec::integer_range("g", 10, 10), // standard g = 10 m/s^2
            ],
            "A ball is thrown vertically upward with an initial velocity u = {u} m/s. Assuming g = {g} m/s^2, find the maximum height reached by the ball.",
            AnswerDerivation::KinematicStoppingDistance {
                u_param: "u".into(),
                a_param: "g".into(),
            },
            "H_max = {answer} m",
            "At maximum height, v = 0. Using v^2 = u^2 - 2gH -> H = u^2 / (2g) = ({u})^2 / (2 * {g}) = {answer} m.",
            40_000,
        )],
    );

    // -----------------------------------------------------------------------
    // New Pattern 4 (Chemistry): Gas Law Molar Volume Conversion
    // -----------------------------------------------------------------------
    let new_chemistry_contract = DeclarativeFamilyContract::new(
        ProblemFamilyContract::new(
            "family.chemistry.gas_laws.stp_volume",
            "skill.chemistry.gas_laws.stp_volume",
            Domain::Chemistry,
            "schema.chemistry.gas_laws.stp_volume",
            ProblemFamilyCapability::DomainChemistry,
        )
        .with_difficulty_range(1.0, 5.0),
        vec![DeclarativeArchetype::new(
            "arch_stp_gas_volume",
            2,
            VariantCategory::Parameter,
            "stp_gas_volume",
            vec![
                ParameterSpec::integer_range("moles", 2, 8),
                ParameterSpec::integer_range("molar_volume_stp", 22, 22), // 22.4 L/mol rounded
                ParameterSpec::derived_product("volume_l", "moles", "molar_volume_stp"),
            ],
            "Calculate the volume occupied by {moles} moles of an ideal gas at Standard Temperature and Pressure (STP), given standard molar volume = {molar_volume_stp} L/mol.",
            AnswerDerivation::DirectParam {
                param_name: "volume_l".into(),
            },
            "{answer} Liters",
            "Volume V = n * V_m = ({moles} mol) * ({molar_volume_stp} L/mol) = {answer} L.",
            30_000,
        )],
    );

    let new_patterns = vec![
        ("New Math (3-num LCM Remainders)", new_math_contract),
        ("New Reasoning (Vector Path)", new_reasoning_contract),
        ("New Physics (Max Height Projectile)", new_physics_contract),
        ("New Chemistry (STP Gas Volume)", new_chemistry_contract),
    ];

    for (name, contract) in new_patterns {
        println!("==> Testing Unseen Zero-Code Pattern: {}", name);

        // 1. Contract Validation
        contract.validate().expect("New contract validation failed");

        // 2. Package into APKG Card Anchor
        let anchor = ProceduralCardAnchor::new(contract.contract.default_schema.clone())
            .with_seed_mode(SeedMode::Fixed(99))
            .with_difficulty_override(2.0)
            .with_inline_contract(contract.clone());

        // 3. Roundtrip Anchor JSON
        let json_str = anchor.to_json_string().unwrap();
        let parsed = ProceduralCardAnchor::from_json_str(&json_str).unwrap().unwrap();

        // 4. Resolve and generate with ZERO topic-specific Rust code
        let session = service
            .resolve_procedural_target(&parsed, Some(2002))
            .unwrap_or_else(|e| panic!("Failed to resolve unseen pattern {}: {:?}", name, e));

        assert_eq!(session.schema.id, contract.contract.default_schema);
        assert!(!session.instance.rendered_prompt.is_empty());
        assert!(session.instance.correct_answer.get("value").is_some());

        // 5. Verify Reviewer HTML Render
        let html = render_reviewer_html(&session);
        assert!(!html.is_empty());

        println!("    [ZERO-CODE SUCCESS] {} generated and rendered cleanly with NO new generator code!", name);
    }
}
