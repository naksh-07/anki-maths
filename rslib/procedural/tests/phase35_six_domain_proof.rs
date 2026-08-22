// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

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

// ---------------------------------------------------------------------------
// 1. Math Domain: LCM / HCF Contract
// ---------------------------------------------------------------------------
fn create_math_lcm_hcf_contract() -> DeclarativeFamilyContract {
    let contract = ProblemFamilyContract::new(
        "family.math.number_system.lcm_hcf",
        "skill.math.number_system.lcm_hcf",
        Domain::Mathematics,
        "schema.math.number_system.lcm_hcf",
        ProblemFamilyCapability::Declarative,
    )
    .with_difficulty_range(1.0, 5.0)
    .with_variants(vec!["lcm_two_numbers", "hcf_two_numbers"]);

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
        "To find LCM({num1}, {num2}), factorize each number into primes and take the highest power of each prime. Result = {answer}.",
        30_000,
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

// ---------------------------------------------------------------------------
// 2. Math Domain: Algebra (Linear Equations) Contract
// ---------------------------------------------------------------------------
fn create_math_algebra_contract() -> DeclarativeFamilyContract {
    let contract = ProblemFamilyContract::new(
        "family.math.algebra.linear_equations",
        "skill.math.algebra.linear_equations",
        Domain::Mathematics,
        "schema.math.algebra.linear_equations",
        ProblemFamilyCapability::Declarative,
    )
    .with_difficulty_range(1.0, 5.0)
    .with_variants(vec!["two_step_basic"]);

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
    );

    DeclarativeFamilyContract::new(contract, vec![arch])
}

// ---------------------------------------------------------------------------
// 3. Reasoning Domain: Coding-Decoding (Coded Expressions) Contract
// ---------------------------------------------------------------------------
fn create_reasoning_coding_contract() -> DeclarativeFamilyContract {
    let contract = ProblemFamilyContract::new(
        "family.reasoning.coded_expressions",
        "skill.reasoning.coded_expressions",
        Domain::Reasoning,
        "schema.reasoning.coded_expressions",
        ProblemFamilyCapability::Declarative,
    )
    .with_difficulty_range(1.0, 5.0)
    .with_variants(vec!["coded_kinship"]);

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

// ---------------------------------------------------------------------------
// 4. Reasoning Domain: Seating Arrangement (CSP) Contract
// ---------------------------------------------------------------------------
fn create_reasoning_seating_contract() -> DeclarativeFamilyContract {
    let contract = ProblemFamilyContract::new(
        "family.reasoning.seating",
        "skill.reasoning.seating",
        Domain::Reasoning,
        "schema.reasoning.seating",
        ProblemFamilyCapability::ConstraintSolver,
    )
    .with_difficulty_range(1.0, 5.0)
    .with_variants(vec!["linear_seating"]);

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

// ---------------------------------------------------------------------------
// 5. Physics Domain: Kinematics Contract
// ---------------------------------------------------------------------------
fn create_physics_kinematics_contract() -> DeclarativeFamilyContract {
    let contract = ProblemFamilyContract::new(
        "family.physics.kinematics.1d",
        "skill.physics.kinematics",
        Domain::Physics,
        "schema.physics.kinematics.1d",
        ProblemFamilyCapability::DomainPhysics,
    )
    .with_difficulty_range(1.0, 5.0)
    .with_variants(vec!["final_velocity", "stopping_distance"]);

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

// ---------------------------------------------------------------------------
// 6. Chemistry Domain: Stoichiometry & Equilibrium Contract
// ---------------------------------------------------------------------------
fn create_chemistry_contract() -> DeclarativeFamilyContract {
    let contract = ProblemFamilyContract::new(
        "family.chemistry.stoichiometry.moles",
        "skill.chemistry.stoichiometry",
        Domain::Chemistry,
        "schema.chemistry.stoichiometry.moles",
        ProblemFamilyCapability::DomainChemistry,
    )
    .with_difficulty_range(1.0, 5.0)
    .with_variants(vec!["moles_to_mass", "mass_to_mass"]);

    let arch_mol_mass = DeclarativeArchetype::new(
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

    DeclarativeFamilyContract::new(contract, vec![arch_mol_mass])
}

// ===========================================================================
// Six-Domain Verification Test Suite
// ===========================================================================

#[test]
fn test_six_domain_declarative_proof() {
    let service = ProceduralService::open_in_memory().expect("failed to create in-memory service");

    let contracts = vec![
        ("Math: LCM/HCF", create_math_lcm_hcf_contract()),
        ("Math: Algebra", create_math_algebra_contract()),
        ("Reasoning: Coding-Decoding", create_reasoning_coding_contract()),
        ("Reasoning: Seating Arrangement", create_reasoning_seating_contract()),
        ("Physics: Kinematics", create_physics_kinematics_contract()),
        ("Chemistry: Stoichiometry", create_chemistry_contract()),
    ];

    for (domain_name, contract) in contracts {
        println!("==> Verifying Domain Contract: {}", domain_name);

        // 1. Contract Validation
        assert!(contract.validate().is_ok(), "Contract validation failed for {}", domain_name);

        // 2. Package into APKG Anchor
        let anchor = ProceduralCardAnchor::new(contract.contract.default_schema.clone())
            .with_seed_mode(SeedMode::Fixed(42))
            .with_difficulty_override(1.0)
            .with_inline_contract(contract.clone());

        // 3. Serialize Anchor to JSON (simulating APKG payload in card field)
        let anchor_json = anchor.to_json_string().expect("failed to serialize anchor");
        assert!(anchor_json.contains("inline_contract"));

        // 4. Ingest & Deserialization on clean profile
        let parsed_anchor = ProceduralCardAnchor::from_json_str(&anchor_json)
            .expect("parsing failed")
            .expect("anchor is None");
        assert!(parsed_anchor.inline_contract.is_some());

        // 5. Precedence Resolution: Dynamic Registration & Problem Generation
        let session = service
            .resolve_procedural_target(&parsed_anchor, Some(1001))
            .unwrap_or_else(|e| panic!("Failed to resolve target for {}: {:?}", domain_name, e));

        // 6. Verify Instance Properties
        assert_eq!(session.schema.id, contract.contract.default_schema);
        assert!(!session.instance.rendered_prompt.is_empty());
        assert!(session.instance.correct_answer.get("value").is_some());

        // 7. Verify Target Latency from Contract (Not 45s hardcoded fallback)
        let expected_latency = contract.contract.target_latency(1);
        assert_eq!(session.target_latency_ms, Some(expected_latency));

        // 8. Verify Reviewer HTML Render
        let html = render_reviewer_html(&session);
        assert!(!html.is_empty(), "Reviewer HTML should not be empty for {}", domain_name);
        assert!(
            html.contains("studylab-card-container") || html.contains("problem-card") || html.contains("procedural"),
            "Reviewer HTML should contain standard StudyLab markup"
        );

        println!("    [PASS] {} successfully generated, solved, and rendered!", domain_name);
    }
}
