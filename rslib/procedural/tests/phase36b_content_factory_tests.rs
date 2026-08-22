// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::time::Instant;

use serde_json::json;

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

// ===========================================================================
// Domain Contract Builders for Six-Domain Proof
// ===========================================================================

fn create_math_lcm_hcf_contract() -> DeclarativeFamilyContract {
    let contract = ProblemFamilyContract::new(
        "family.math.number_system.lcm_hcf",
        "skill.math.number_system.lcm_hcf",
        Domain::Mathematics,
        "schema.math.number_system.lcm_hcf",
        ProblemFamilyCapability::Declarative,
    )
    .with_difficulty_range(1.0, 5.0)
    .with_variants(vec!["lcm_two_numbers", "hcf_two_numbers"])
    .with_target_latency(1, 25_000)
    .with_target_latency(2, 35_000);

    let arch_lcm = DeclarativeArchetype::new(
        "arch_lcm_two_numbers",
        1,
        VariantCategory::Parameter,
        "lcm_two_numbers",
        vec![
            ParameterSpec::integer_range("num1", 6, 36),
            ParameterSpec::integer_range("num2", 8, 48),
        ],
        "Find the Least Common Multiple (LCM) of {num1} and {num2}.",
        AnswerDerivation::LcmArray {
            params: vec!["num1".into(), "num2".into()],
        },
        "{answer}",
        "To find LCM({num1}, {num2}), factorize each into prime powers and take the highest power of each prime. Result = {answer}.",
        25_000,
    )
    .with_step_nodes(vec![
        StepNodeSpec::new(
            "step_prime_fact",
            StepType::Arithmetic,
            "Prime Factorization",
            "Find the prime factors of {num1} and {num2}.",
            "LCM({num1}, {num2}) = {answer}",
            vec![],
            "Every composite number can be factored into a product of primes.",
            "Write the prime powers for each number.",
            "Take the maximum exponent for each prime factor.",
        ),
    ]);

    let arch_hcf = DeclarativeArchetype::new(
        "arch_hcf_two_numbers",
        2,
        VariantCategory::Parameter,
        "hcf_two_numbers",
        vec![
            ParameterSpec::integer_range("num1", 12, 96),
            ParameterSpec::integer_range("num2", 18, 120),
        ],
        "Find the Highest Common Factor (HCF / GCD) of {num1} and {num2}.",
        AnswerDerivation::GcdArray {
            params: vec!["num1".into(), "num2".into()],
        },
        "{answer}",
        "To find HCF({num1}, {num2}), compute the greatest common divisor. HCF = {answer}.",
        35_000,
    );

    DeclarativeFamilyContract::new(contract, vec![arch_lcm, arch_hcf])
}

fn create_math_algebra_contract() -> DeclarativeFamilyContract {
    let contract = ProblemFamilyContract::new(
        "family.math.algebra.linear_equations",
        "skill.math.algebra.linear_equations",
        Domain::Mathematics,
        "schema.math.algebra.linear_equations",
        ProblemFamilyCapability::Declarative,
    )
    .with_difficulty_range(1.0, 5.0)
    .with_variants(vec!["two_step_basic"])
    .with_target_latency(1, 25_000);

    let arch = DeclarativeArchetype::new(
        "arch_two_step_basic",
        1,
        VariantCategory::Parameter,
        "two_step_basic",
        vec![
            ParameterSpec::integer_range("a", 2, 8),
            ParameterSpec::integer_range("x", 1, 12),
            ParameterSpec::integer_range("b", 1, 15),
            ParameterSpec::derived_linear("c", "a", "x", "b"),
        ],
        "Solve the linear equation: {a}x + {b} = {c}",
        AnswerDerivation::LinearTwoStep {
            c_param: "c".into(),
            b_param: "b".into(),
            a_param: "a".into(),
        },
        "x = {answer}",
        "Subtract {b} from both sides: {a}x = {c_minus_b}, then divide by {a}: x = {answer}.",
        25_000,
    )
    .with_step_nodes(vec![
        StepNodeSpec::new(
            "step_isolate",
            StepType::EquationRearrangement,
            "Isolate variable term",
            "Subtract {b} from both sides",
            "{a}x = {c_minus_b}",
            vec![],
            "Inverse arithmetic operation.",
            "Subtract {b} from both sides.",
            "{a}x = {c_minus_b}",
        ),
    ]);

    DeclarativeFamilyContract::new(contract, vec![arch])
}

fn create_reasoning_coding_contract() -> DeclarativeFamilyContract {
    let contract = ProblemFamilyContract::new(
        "family.reasoning.coded_expressions",
        "skill.reasoning.coded_expressions",
        Domain::Reasoning,
        "schema.reasoning.coded_expressions",
        ProblemFamilyCapability::Declarative,
    )
    .with_difficulty_range(1.0, 5.0)
    .with_variants(vec!["coded_kinship"])
    .with_target_latency(1, 30_000);

    let arch = DeclarativeArchetype::new(
        "arch_coded_kinship",
        1,
        VariantCategory::Structural,
        "coded_kinship",
        vec![
            ParameterSpec::permutation_choice("actors", vec!["P".into(), "Q".into(), "R".into(), "S".into()], 3),
            ParameterSpec::discrete_choice("relation", vec![json!("Father"), json!("Mother"), json!("Brother")]),
        ],
        "Given the code: A @ B means 'A is the {relation} of B'. In expression {actors_0} @ {actors_1}, how is {actors_0} related to {actors_1}?",
        AnswerDerivation::DirectStringParam {
            param_name: "relation".into(),
        },
        "{answer}",
        "Decoding the operator '@' according to given rules: {actors_0} is the {answer} of {actors_1}.",
        30_000,
    );

    DeclarativeFamilyContract::new(contract, vec![arch])
}

fn create_reasoning_seating_contract() -> DeclarativeFamilyContract {
    let contract = ProblemFamilyContract::new(
        "family.reasoning.seating",
        "skill.reasoning.seating",
        Domain::Reasoning,
        "schema.reasoning.seating",
        ProblemFamilyCapability::ConstraintSolver,
    )
    .with_difficulty_range(1.0, 5.0)
    .with_variants(vec!["linear_seating"])
    .with_target_latency(1, 35_000);

    let arch = DeclarativeArchetype::new(
        "arch_linear_seating",
        1,
        VariantCategory::Structural,
        "linear_seating",
        vec![
            ParameterSpec::permutation_choice("names", vec!["Alice".into(), "Bob".into(), "Charlie".into(), "David".into()], 4),
            ParameterSpec::integer_range("anchor_slot", 1, 4),
        ],
        "Four friends {names} sit in a single row from left to right (slots 1 to 4). {names_0} sits at position {anchor_slot}. At which position is {names_0} sitting?",
        AnswerDerivation::DirectParam {
            param_name: "anchor_slot".into(),
        },
        "Position {answer}",
        "From the given ground truth constraint, {names_0} is placed at slot {answer}.",
        35_000,
    );

    DeclarativeFamilyContract::new(contract, vec![arch])
}

fn create_physics_kinematics_contract() -> DeclarativeFamilyContract {
    let contract = ProblemFamilyContract::new(
        "family.physics.kinematics.1d",
        "skill.physics.kinematics",
        Domain::Physics,
        "schema.physics.kinematics.1d",
        ProblemFamilyCapability::DomainPhysics,
    )
    .with_difficulty_range(1.0, 5.0)
    .with_variants(vec!["final_velocity", "stopping_distance"])
    .with_target_latency(1, 30_000)
    .with_target_latency(2, 40_000);

    let arch_v = DeclarativeArchetype::new(
        "arch_kinematic_velocity",
        1,
        VariantCategory::Parameter,
        "final_velocity",
        vec![
            ParameterSpec::integer_range("u", 0, 20),
            ParameterSpec::integer_range("a", 2, 8),
            ParameterSpec::integer_range("t", 1, 10),
        ],
        "A particle moves in a straight line with initial velocity u = {u} m/s and constant acceleration a = {a} m/s^2. Calculate its velocity after t = {t} seconds.",
        AnswerDerivation::KinematicVelocity {
            u_param: "u".into(),
            a_param: "a".into(),
            t_param: "t".into(),
        },
        "{answer} m/s",
        "Using first equation of motion: v = u + at = {u} + ({a})({t}) = {answer} m/s.",
        30_000,
    );

    let arch_stop = DeclarativeArchetype::new(
        "arch_stopping_distance",
        2,
        VariantCategory::Parameter,
        "stopping_distance",
        vec![
            ParameterSpec::integer_range("u", 10, 40),
            ParameterSpec::integer_range("a", 2, 8),
        ],
        "A vehicle traveling at initial speed u = {u} m/s brakes with deceleration a = {a} m/s^2 until coming to rest. Find the stopping distance.",
        AnswerDerivation::KinematicStoppingDistance {
            u_param: "u".into(),
            a_param: "a".into(),
        },
        "{answer} m",
        "Using v^2 = u^2 - 2as where v = 0 -> s = u^2 / (2a) = {answer} m.",
        40_000,
    );

    DeclarativeFamilyContract::new(contract, vec![arch_v, arch_stop])
}

fn create_chemistry_stoichiometry_contract() -> DeclarativeFamilyContract {
    let contract = ProblemFamilyContract::new(
        "family.chemistry.stoichiometry.moles",
        "skill.chemistry.stoichiometry",
        Domain::Chemistry,
        "schema.chemistry.stoichiometry.moles",
        ProblemFamilyCapability::DomainChemistry,
    )
    .with_difficulty_range(1.0, 5.0)
    .with_variants(vec!["moles_to_mass"])
    .with_target_latency(1, 25_000);

    let arch = DeclarativeArchetype::new(
        "arch_stoich_moles_to_mass",
        1,
        VariantCategory::Parameter,
        "moles_to_mass",
        vec![
            ParameterSpec::integer_range("moles", 1, 10),
            ParameterSpec::integer_range("molar_mass", 18, 44),
        ],
        "Calculate the mass (in grams) of {moles} moles of a chemical substance with molar mass M = {molar_mass} g/mol.",
        AnswerDerivation::StoichiometricMolesToMass {
            moles_param: "moles".into(),
            molar_mass_param: "molar_mass".into(),
        },
        "{answer} g",
        "Mass m = n * M = ({moles} mol) * ({molar_mass} g/mol) = {answer} g.",
        25_000,
    );

    DeclarativeFamilyContract::new(contract, vec![arch])
}

// ===========================================================================
// Test 1: Six-Domain Proof (Section 10)
// ===========================================================================

#[test]
fn test_section_10_six_domain_factory_proof() {
    let service = ProceduralService::open_in_memory().expect("failed to open service");

    let domains = vec![
        ("Mathematics: LCM/HCF", create_math_lcm_hcf_contract()),
        ("Mathematics: Linear Algebra", create_math_algebra_contract()),
        ("Reasoning: Coding-Decoding", create_reasoning_coding_contract()),
        ("Reasoning: Seating Arrangement (CSP)", create_reasoning_seating_contract()),
        ("Physics: Kinematics 1D", create_physics_kinematics_contract()),
        ("Chemistry: Stoichiometry", create_chemistry_stoichiometry_contract()),
    ];

    for (domain_title, contract) in domains {
        // 1. Contract validation
        assert!(contract.validate().is_ok(), "Validation failed for {}", domain_title);

        // 2. Package into APKG card anchor
        let anchor = ProceduralCardAnchor::new(contract.contract.default_schema.clone())
            .with_seed_mode(SeedMode::Fixed(42))
            .with_difficulty_override(1.0)
            .with_inline_contract(contract.clone());

        // 3. Serialize and deserialize simulating APKG payload
        let json_str = anchor.to_json_string().expect("serialization failed");
        let parsed_anchor = ProceduralCardAnchor::from_json_str(&json_str)
            .expect("parsing failed")
            .expect("missing anchor");
        assert!(parsed_anchor.inline_contract.is_some());

        // 4. Resolve procedural target on clean profile
        let session = service
            .resolve_procedural_target(&parsed_anchor, Some(101))
            .unwrap_or_else(|e| panic!("Resolution failed for {}: {:?}", domain_title, e));

        // 5. Verify session properties
        assert_eq!(session.schema.id, contract.contract.default_schema);
        assert!(!session.instance.rendered_prompt.is_empty());
        assert!(session.instance.correct_answer.get("value").is_some());
        assert_eq!(session.target_latency_ms, Some(contract.contract.target_latency(1)));

        // 6. Verify Reviewer HTML rendering
        let html = render_reviewer_html(&session);
        assert!(!html.is_empty(), "Reviewer HTML empty for {}", domain_title);
        assert!(
            html.contains("studylab-card-container") || html.contains("problem-card") || html.contains("procedural"),
            "Reviewer HTML missing StudyLab container for {}", domain_title
        );
    }
}

// ===========================================================================
// Test 2: Zero-Code Unseen Pattern Proof (Section 11)
// ===========================================================================

#[test]
fn test_section_11_four_unseen_patterns_proof() {
    let service = ProceduralService::open_in_memory().expect("failed to open service");

    // Pattern 1 (Math): Right Triangle Pythagoras & Geometry
    let unseen_math = DeclarativeFamilyContract::new(
        ProblemFamilyContract::new(
            "family.math.geometry.pythagoras_hypotenuse",
            "skill.math.geometry.pythagoras",
            Domain::Mathematics,
            "schema.math.geometry.pythagoras_hypotenuse",
            ProblemFamilyCapability::DomainGeometry,
        )
        .with_difficulty_range(1.0, 5.0)
        .with_target_latency(1, 30_000),
        vec![DeclarativeArchetype::new(
            "arch_pythagoras_3_4_5",
            1,
            VariantCategory::Structural,
            "pythagoras_calc",
            vec![
                ParameterSpec::integer_range("leg_a", 3, 12),
                ParameterSpec::integer_range("leg_b", 4, 16),
            ],
            "In a right-angled triangle, the lengths of the two perpendicular legs are a = {leg_a} cm and b = {leg_b} cm. Calculate the length of the hypotenuse c.",
            AnswerDerivation::PythagorasHypotenuse {
                a_param: "leg_a".into(),
                b_param: "leg_b".into(),
            },
            "c = {answer} cm",
            "Using Pythagoras theorem: c = sqrt(a^2 + b^2) = sqrt({leg_a}^2 + {leg_b}^2) = {answer} cm.",
            30_000,
        )
        .with_step_nodes(vec![
            StepNodeSpec::new(
                "step_pythagoras",
                StepType::FormulaSelection,
                "Apply Pythagoras Theorem",
                "Compute c = sqrt(a^2 + b^2)",
                "c = {answer}",
                vec![],
                "In a right triangle, the square of the hypotenuse equals the sum of squares of legs.",
                "Square both legs, sum them, and take the square root.",
                "c = sqrt({leg_a}^2 + {leg_b}^2) = {answer}",
            ),
        ])],
    );

    // Pattern 2 (Reasoning): Propositional Logic Inference (SymbolicLogic)
    let unseen_reasoning = DeclarativeFamilyContract::new(
        ProblemFamilyContract::new(
            "family.reasoning.symbolic_logic.implication",
            "skill.reasoning.symbolic_logic",
            Domain::Reasoning,
            "schema.reasoning.symbolic_logic.implication",
            ProblemFamilyCapability::SymbolicLogic,
        )
        .with_difficulty_range(1.0, 5.0)
        .with_target_latency(1, 20_000),
        vec![DeclarativeArchetype::new(
            "arch_prop_logic_implication",
            1,
            VariantCategory::Isomorphic,
            "implication_eval",
            vec![
                ParameterSpec::discrete_choice("p", vec![json!(true), json!(false)]),
                ParameterSpec::discrete_choice("q", vec![json!(true), json!(false)]),
            ],
            "Determine the truth value of the conditional proposition P -> Q when P is {p} and Q is {q}.",
            AnswerDerivation::SymbolicLogicEvaluation {
                p_param: "p".into(),
                q_param: "q".into(),
                operator: "IMPLIES".into(),
            },
            "{answer}",
            "A conditional statement P -> Q is False only when P is True and Q is False. Here truth value is {answer}.",
            20_000,
        )],
    );

    // Pattern 3 (Physics): Kinetic Work-Energy Theorem
    let unseen_physics = DeclarativeFamilyContract::new(
        ProblemFamilyContract::new(
            "family.physics.work_energy.kinetic_energy",
            "skill.physics.work_energy",
            Domain::Physics,
            "schema.physics.work_energy.kinetic_energy",
            ProblemFamilyCapability::DomainPhysics,
        )
        .with_difficulty_range(1.0, 5.0)
        .with_target_latency(1, 25_000),
        vec![DeclarativeArchetype::new(
            "arch_kinetic_energy_calc",
            1,
            VariantCategory::Parameter,
            "kinetic_energy_calc",
            vec![
                ParameterSpec::integer_range("mass", 2, 20),
                ParameterSpec::integer_range("velocity", 3, 15),
            ],
            "Calculate the kinetic energy (in Joules) of an object with mass m = {mass} kg moving at a velocity v = {velocity} m/s.",
            AnswerDerivation::KinematicWorkEnergy {
                mass_param: "mass".into(),
                velocity_param: "velocity".into(),
            },
            "E_k = {answer} J",
            "Kinetic Energy E_k = (1/2) * m * v^2 = 0.5 * {mass} * ({velocity})^2 = {answer} Joules.",
            25_000,
        )],
    );

    // Pattern 4 (Chemistry): Ideal Gas Law Pressure
    let unseen_chemistry = DeclarativeFamilyContract::new(
        ProblemFamilyContract::new(
            "family.chemistry.gas_laws.ideal_gas_pressure",
            "skill.chemistry.gas_laws",
            Domain::Chemistry,
            "schema.chemistry.gas_laws.ideal_gas_pressure",
            ProblemFamilyCapability::DomainChemistry,
        )
        .with_difficulty_range(1.0, 5.0)
        .with_target_latency(1, 30_000),
        vec![DeclarativeArchetype::new(
            "arch_ideal_gas_pressure",
            1,
            VariantCategory::Transfer,
            "ideal_gas_pressure",
            vec![
                ParameterSpec::integer_range("moles", 1, 5),
                ParameterSpec::integer_range("temp", 300, 400),
                ParameterSpec::integer_range("volume", 10, 50),
            ],
            "Using the ideal gas equation PV = nRT with R = 8.314 J/(mol·K), find the pressure (in Pascals) exerted by {moles} moles of gas in a {volume} m^3 container at T = {temp} K.",
            AnswerDerivation::IdealGasLawPressure {
                moles_param: "moles".into(),
                temp_param: "temp".into(),
                vol_param: "volume".into(),
                r_const: Some(8.314),
            },
            "P = {answer} Pa",
            "P = (n * R * T) / V = ({moles} * 8.314 * {temp}) / {volume} = {answer} Pa.",
            30_000,
        )],
    );

    let unseen_patterns = vec![
        ("Unseen Math: Pythagoras Geometry", unseen_math),
        ("Unseen Reasoning: Symbolic Logic", unseen_reasoning),
        ("Unseen Physics: Kinetic Energy", unseen_physics),
        ("Unseen Chemistry: Ideal Gas Law", unseen_chemistry),
    ];

    for (name, contract) in unseen_patterns {
        assert!(contract.validate().is_ok(), "Validation failed for {}", name);

        let anchor = ProceduralCardAnchor::new(contract.contract.default_schema.clone())
            .with_seed_mode(SeedMode::Fixed(77))
            .with_inline_contract(contract.clone());

        let session = service
            .resolve_procedural_target(&anchor, Some(202))
            .unwrap_or_else(|e| panic!("Resolution failed for {}: {:?}", name, e));

        assert_eq!(session.schema.id, contract.contract.default_schema);
        assert!(!session.instance.rendered_prompt.is_empty());
        assert!(session.instance.correct_answer.get("value").is_some());

        let html = render_reviewer_html(&session);
        assert!(!html.is_empty());
    }
}

// ===========================================================================
// Test 3: 100-Pattern Batch Factory Stress Test (Section 13)
// ===========================================================================

#[test]
fn test_section_13_100_pattern_batch_stress_test() {
    let service = ProceduralService::open_in_memory().expect("failed to open service");
    let mut total_generated = 0;
    let start_time = Instant::now();

    // 25 Math Patterns
    for i in 1..=25 {
        let family_id = format!("family.math.batch_stress.pattern_{:02}", i);
        let schema_id = format!("schema.math.batch_stress.pattern_{:02}", i);
        let contract = DeclarativeFamilyContract::new(
            ProblemFamilyContract::new(
                family_id.as_str(),
                format!("skill.math.batch.{}", i),
                Domain::Mathematics,
                schema_id.as_str(),
                ProblemFamilyCapability::Declarative,
            )
            .with_difficulty_range(1.0, 5.0)
            .with_target_latency(1, 20_000 + (i as u64) * 1000),
            vec![DeclarativeArchetype::new(
                format!("arch_math_{:02}", i),
                1,
                VariantCategory::Parameter,
                format!("variant_math_{:02}", i),
                vec![
                    ParameterSpec::integer_range("a", 2 + (i as i64), 10 + (i as i64)),
                    ParameterSpec::integer_range("b", 1 + (i as i64), 20 + (i as i64)),
                    ParameterSpec::derived_sum("sum", "a", "b"),
                ],
                format!("Math Pattern {}: Compute the sum of {{a}} and {{b}}.", i),
                AnswerDerivation::DirectParam { param_name: "sum".into() },
                "{answer}",
                "Sum = {a} + {b} = {answer}.",
                25_000,
            )],
        );

        assert!(contract.validate().is_ok());
        let anchor = ProceduralCardAnchor::new(schema_id.as_str())
            .with_seed_mode(SeedMode::Fixed(1000 + i as u64))
            .with_inline_contract(contract);

        let session = service.resolve_procedural_target(&anchor, Some(1000 + i as i64)).unwrap();
        assert!(!session.instance.rendered_prompt.is_empty());
        assert!(session.instance.correct_answer.get("value").is_some());
        let html = render_reviewer_html(&session);
        assert!(!html.is_empty());
        total_generated += 1;
    }

    // 25 Reasoning Patterns
    for i in 1..=25 {
        let family_id = format!("family.reasoning.batch_stress.pattern_{:02}", i);
        let schema_id = format!("schema.reasoning.batch_stress.pattern_{:02}", i);
        let contract = DeclarativeFamilyContract::new(
            ProblemFamilyContract::new(
                family_id.as_str(),
                format!("skill.reasoning.batch.{}", i),
                Domain::Reasoning,
                schema_id.as_str(),
                ProblemFamilyCapability::Declarative,
            )
            .with_difficulty_range(1.0, 5.0)
            .with_target_latency(1, 30_000),
            vec![DeclarativeArchetype::new(
                format!("arch_reasoning_{:02}", i),
                1,
                VariantCategory::Structural,
                format!("variant_reasoning_{:02}", i),
                vec![
                    ParameterSpec::permutation_choice("items", vec!["Alpha".into(), "Beta".into(), "Gamma".into(), "Delta".into()], 3),
                    ParameterSpec::integer_range("rank", 1, 3),
                ],
                format!("Reasoning Pattern {}: In order {{items}}, item {{items_0}} is ranked at {{rank}}. What is {{items_0}}'s rank?", i),
                AnswerDerivation::DirectParam { param_name: "rank".into() },
                "Rank {answer}",
                "Direct constraint lookup yields rank {answer}.",
                30_000,
            )],
        );

        assert!(contract.validate().is_ok());
        let anchor = ProceduralCardAnchor::new(schema_id.as_str())
            .with_seed_mode(SeedMode::Fixed(2000 + i as u64))
            .with_inline_contract(contract);

        let session = service.resolve_procedural_target(&anchor, Some(2000 + i as i64)).unwrap();
        assert!(!session.instance.rendered_prompt.is_empty());
        let html = render_reviewer_html(&session);
        assert!(!html.is_empty());
        total_generated += 1;
    }

    // 25 Physics Patterns
    for i in 1..=25 {
        let family_id = format!("family.physics.batch_stress.pattern_{:02}", i);
        let schema_id = format!("schema.physics.batch_stress.pattern_{:02}", i);
        let contract = DeclarativeFamilyContract::new(
            ProblemFamilyContract::new(
                family_id.as_str(),
                format!("skill.physics.batch.{}", i),
                Domain::Physics,
                schema_id.as_str(),
                ProblemFamilyCapability::DomainPhysics,
            )
            .with_difficulty_range(1.0, 5.0)
            .with_target_latency(1, 35_000),
            vec![DeclarativeArchetype::new(
                format!("arch_physics_{:02}", i),
                1,
                VariantCategory::Parameter,
                format!("variant_physics_{:02}", i),
                vec![
                    ParameterSpec::integer_range("mass", 1 + (i as i64), 10 + (i as i64)),
                    ParameterSpec::integer_range("acc", 2, 8),
                    ParameterSpec::derived_product("force", "mass", "acc"),
                ],
                format!("Physics Pattern {}: An object of mass {{mass}} kg accelerates at {{acc}} m/s^2. Find the net force F = m*a.", i),
                AnswerDerivation::DirectParam { param_name: "force".into() },
                "{answer} N",
                "Force F = m * a = {mass} * {acc} = {answer} N.",
                35_000,
            )],
        );

        assert!(contract.validate().is_ok());
        let anchor = ProceduralCardAnchor::new(schema_id.as_str())
            .with_seed_mode(SeedMode::Fixed(3000 + i as u64))
            .with_inline_contract(contract);

        let session = service.resolve_procedural_target(&anchor, Some(3000 + i as i64)).unwrap();
        assert!(!session.instance.rendered_prompt.is_empty());
        let html = render_reviewer_html(&session);
        assert!(!html.is_empty());
        total_generated += 1;
    }

    // 25 Chemistry Patterns
    for i in 1..=25 {
        let family_id = format!("family.chemistry.batch_stress.pattern_{:02}", i);
        let schema_id = format!("schema.chemistry.batch_stress.pattern_{:02}", i);
        let contract = DeclarativeFamilyContract::new(
            ProblemFamilyContract::new(
                family_id.as_str(),
                format!("skill.chemistry.batch.{}", i),
                Domain::Chemistry,
                schema_id.as_str(),
                ProblemFamilyCapability::DomainChemistry,
            )
            .with_difficulty_range(1.0, 5.0)
            .with_target_latency(1, 30_000),
            vec![DeclarativeArchetype::new(
                format!("arch_chemistry_{:02}", i),
                1,
                VariantCategory::Parameter,
                format!("variant_chemistry_{:02}", i),
                vec![
                    ParameterSpec::integer_range("moles", 1 + (i as i64 % 5), 10),
                    ParameterSpec::integer_range("molar_mass", 16, 60),
                ],
                format!("Chemistry Pattern {}: Calculate the mass in grams for {{moles}} moles of compound with molar mass {{molar_mass}} g/mol.", i),
                AnswerDerivation::StoichiometricMolesToMass {
                    moles_param: "moles".into(),
                    molar_mass_param: "molar_mass".into(),
                },
                "{answer} g",
                "Mass m = n * M = {moles} * {molar_mass} = {answer} g.",
                30_000,
            )],
        );

        assert!(contract.validate().is_ok());
        let anchor = ProceduralCardAnchor::new(schema_id.as_str())
            .with_seed_mode(SeedMode::Fixed(4000 + i as u64))
            .with_inline_contract(contract);

        let session = service.resolve_procedural_target(&anchor, Some(4000 + i as i64)).unwrap();
        assert!(!session.instance.rendered_prompt.is_empty());
        let html = render_reviewer_html(&session);
        assert!(!html.is_empty());
        total_generated += 1;
    }

    let elapsed = start_time.elapsed();
    assert_eq!(total_generated, 100);
    println!("==> [100-PATTERN BATCH PASS] 100 patterns generated and rendered in {:?}", elapsed);
    println!("    Average generation & render latency: {:.3} ms/pattern", (elapsed.as_secs_f64() * 1000.0) / 100.0);
}

// ===========================================================================
// Test 4: Security & Validation Guardrails (Section 14)
// ===========================================================================

#[test]
fn test_section_14_security_and_validation() {
    // 1. Rejects empty family_id
    let invalid_contract_1 = DeclarativeFamilyContract::new(
        ProblemFamilyContract::new("", "skill.invalid", Domain::Mathematics, "schema.test", ProblemFamilyCapability::Declarative),
        vec![DeclarativeArchetype::new("arch", 1, VariantCategory::Parameter, "v", vec![], "prompt", AnswerDerivation::DirectParam { param_name: "a".into() }, "", "", 10_000)],
    );
    assert!(invalid_contract_1.validate().is_err());

    // 2. Rejects invalid difficulty range
    let mut invalid_pf = ProblemFamilyContract::new("fam.test", "skill.test", Domain::Mathematics, "schema.test", ProblemFamilyCapability::Declarative);
    invalid_pf.min_difficulty = 5.0;
    invalid_pf.max_difficulty = 1.0;
    let invalid_contract_2 = DeclarativeFamilyContract::new(
        invalid_pf,
        vec![DeclarativeArchetype::new("arch", 1, VariantCategory::Parameter, "v", vec![], "prompt", AnswerDerivation::DirectParam { param_name: "a".into() }, "", "", 10_000)],
    );
    assert!(invalid_contract_2.validate().is_err());

    // 3. Rejects empty archetypes
    let invalid_contract_3 = DeclarativeFamilyContract::new(
        ProblemFamilyContract::new("fam.test", "skill.test", Domain::Mathematics, "schema.test", ProblemFamilyCapability::Declarative),
        vec![],
    );
    assert!(invalid_contract_3.validate().is_err());

    // 4. Rejects zero target_time_ms
    let invalid_contract_4 = DeclarativeFamilyContract::new(
        ProblemFamilyContract::new("fam.test", "skill.test", Domain::Mathematics, "schema.test", ProblemFamilyCapability::Declarative),
        vec![DeclarativeArchetype::new("arch", 1, VariantCategory::Parameter, "v", vec![], "prompt", AnswerDerivation::DirectParam { param_name: "a".into() }, "", "", 0)],
    );
    assert!(invalid_contract_4.validate().is_err());
}

// ===========================================================================
// Test 5: Legacy & Precedence Integration (Section 15 & 6)
// ===========================================================================

#[test]
fn test_section_15_legacy_and_precedence() {
    let service = ProceduralService::open_in_memory().expect("failed to open service");

    // Tier 1: inline_contract with explicit difficulty override
    let rich_contract = create_math_algebra_contract();
    let rich_anchor = ProceduralCardAnchor::new("schema.math.algebra.linear_equations")
        .with_difficulty_override(3.0)
        .with_inline_contract(rich_contract);

    let session_rich = service.resolve_procedural_target(&rich_anchor, Some(501)).unwrap();
    assert_eq!(session_rich.difficulty_level, Some(3));
    assert_eq!(session_rich.selection_reason, Some("Inline Declarative Contract".to_string()));

    // Tier 3: legacy proc_schema fallback
    let legacy_anchor = ProceduralCardAnchor::new("math.percentage.successive");
    let session_legacy = service.resolve_procedural_target(&legacy_anchor, Some(502)).unwrap();
    assert_eq!(session_legacy.schema.id.as_str(), "successive_percentage");
    assert!(!session_legacy.instance.rendered_prompt.is_empty());
}
