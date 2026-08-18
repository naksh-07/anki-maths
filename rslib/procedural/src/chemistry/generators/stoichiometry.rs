use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde_json::json;

use crate::chemistry::models::{ChemicalProblemMetadata, ChemicalRegimeKind};
use crate::chemistry::reaction::ReactionTemplates;
use crate::chemistry::species::SpeciesCatalog;
use crate::chemistry::units::ChemistryUnit;
use crate::core::{ProblemFamilyId, Result};
use crate::diagnostics::ErrorCategory;
use crate::problems::catalog::{FAMILY_CHEMISTRY_STOICHIOMETRY, TEMPLATE_CHEMISTRY_STOICHIOMETRY_V1};
use crate::problems::generator::ProblemGenerator;
use crate::problems::steps::{SolutionGraph, StepHint, StepNode, StepType};
use crate::problems::validator::{AnswerEvaluation, NumericAnswerParser, ProblemValidator};
use crate::problems::ProblemInstance;

pub struct StoichiometryGenerator;

impl StoichiometryGenerator {
    pub fn generate_problem(seed: u64, difficulty_level: u32, variant: Option<&str>) -> ProblemInstance {
        match difficulty_level {
            1 => Self::generate_level_1(seed, variant),
            2 => Self::generate_level_2(seed, variant),
            3 => Self::generate_level_3(seed, variant),
            4 => Self::generate_level_4(seed, variant),
            5 => Self::generate_level_5(seed, variant),
            _ => Self::generate_level_2(seed, variant),
        }
    }
    /// Level 1: Direct Mass <-> Moles Conversion via Molar Mass: n = m / M
    pub fn generate_level_1(seed: u64, variant: Option<&str>) -> ProblemInstance {
        let mut rng = StdRng::seed_from_u64(seed);

        let species_choices = [
            SpeciesCatalog::water(),
            SpeciesCatalog::carbon_dioxide(),
            SpeciesCatalog::methane(),
            SpeciesCatalog::ammonia(),
            SpeciesCatalog::calcium_carbonate(),
            SpeciesCatalog::sodium_hydroxide(),
        ];
        let spec_idx = rng.random_range(0..species_choices.len());
        let species = &species_choices[spec_idx];

        let mode = variant.unwrap_or(if rng.random_bool(0.5) { "mass_to_moles" } else { "moles_to_mass" });

        let (prompt, correct_val, unit, step1_expr, step1_title, _step2_expr, step2_title) = if mode == "mass_to_moles" {
            let moles = rng.random_range(2..25) as f64 * 0.25; // e.g. 0.5 to 6.0 mol
            let mass = moles * species.molar_mass;
            let p = format!(
                "Calculate the amount of substance in moles contained in **{:.2} g** of {} ({}, molar mass = **{:.2} g/mol**).",
                mass, species.name, species.formatted_formula(), species.molar_mass
            );
            (
                p,
                moles,
                ChemistryUnit::Mole,
                "n = m / M".to_string(),
                "Identify Mole Formula",
                format!("{:.2} / {:.2} = {:.4} mol", mass, species.molar_mass, moles),
                "Calculate Moles",
            )
        } else {
            let moles = rng.random_range(1..20) as f64 * 0.25;
            let mass = moles * species.molar_mass;
            let p = format!(
                "Calculate the mass in grams of **{:.2} mol** of {} ({}, molar mass = **{:.2} g/mol**).",
                moles, species.name, species.formatted_formula(), species.molar_mass
            );
            (
                p,
                mass,
                ChemistryUnit::Gram,
                "m = n * M".to_string(),
                "Identify Mass Formula",
                format!("{:.2} * {:.2} = {:.2} g", moles, species.molar_mass, mass),
                "Calculate Mass",
            )
        };

        let mut meta = ChemicalProblemMetadata::new(ChemicalRegimeKind::StoichiometryMoleConversion, unit);
        meta = meta.with_target_species(&species.formula);

        let step1 = StepNode::new(
            "formula",
            StepType::SelectEquation,
            step1_title,
            "Recall that moles n = mass m / molar mass M",
            step1_expr.clone(),
        )
        .with_hints(vec![
            StepHint::principle("Molar mass is the mass of 1 mole of a chemical substance."),
            StepHint::operation("Relate mass, moles, and molar mass using n = m / M."),
            StepHint::intermediate_relation(format!("Formula: {}", step1_expr)),
        ]);

        let step2 = StepNode::new(
            "calc",
            StepType::FinalAnswer,
            step2_title,
            "Substitute and solve",
            format!("{:.2}", correct_val),
        )
        .with_expected_value(correct_val)
        .with_dependencies(vec!["formula".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Divide the mass in grams by the molar mass in g/mol."),
            StepHint::operation(format!("Compute the numerical value for {}", species.formula)),
            StepHint::intermediate_relation(format!("{:.4} {}", correct_val, unit.symbol())),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2], "calc");

        let parameters = json!({
            "difficulty": 1,
            "variant": mode,
            "species": species.formula,
            "molar_mass": species.molar_mass,
            "target_value": correct_val,
            "unit": unit.symbol(),
            "physics_metadata": meta,
        });

        let correct_answer = json!({
            "value": correct_val,
            "unit": unit.symbol(),
            "formatted": format!("{:.2} {}", correct_val, unit.symbol()),
            "solution": format!("Using n = m/M: {:.2} {}", correct_val, unit.symbol()),
        });

        ProblemInstance::new(
            format!("inst-chem-stk-l1-{}", seed),
            FAMILY_CHEMISTRY_STOICHIOMETRY,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(json!({
            "difficulty_level": 1,
            "target_time_ms": 25_000,
            "domain": "chemistry",
            "unit": unit.symbol(),
        }))
    }

    /// Level 2: Reaction Mole Ratios via Balanced Coefficients: n_B = n_A * (b / a)
    pub fn generate_level_2(seed: u64, _variant: Option<&str>) -> ProblemInstance {
        let mut rng = StdRng::seed_from_u64(seed);

        let reactions = [
            ReactionTemplates::haber_bosch(),                  // 1 N2 + 3 H2 -> 2 NH3 (ratio 2/1)
            ReactionTemplates::aluminum_oxidation(),           // 4 Al + 3 O2 -> 2 Al2O3 (ratio 2/4 = 1/2)
            ReactionTemplates::thermite_reaction(),            // 2 Al + Fe2O3 -> Al2O3 + 2 Fe (ratio 1/2 for Al2O3)
        ];
        let rxn_idx = rng.random_range(0..reactions.len());
        let rxn = &reactions[rxn_idx];

        let reactant = &rxn.reactants[0].species;
        let product = &rxn.products[0].species;
        let a = rxn.reactants[0].coefficient;
        let b = rxn.products[0].coefficient;

        let n_reactant = (rng.random_range(2..16) as f64) * 0.5 * (a as f64);
        let n_product = n_reactant * (b as f64) / (a as f64);

        let prompt = format!(
            "Consider the balanced reaction:\n$$\\text{{{}}}$$\nIf **{:.1} mol** of {} reacts completely, how many moles of {} will be produced?",
            rxn.formatted_equation(),
            n_reactant,
            reactant.formatted_formula(),
            product.formatted_formula()
        );

        let mut meta = ChemicalProblemMetadata::new(ChemicalRegimeKind::StoichiometryReactionRatio, ChemistryUnit::Mole);
        meta = meta.with_reaction(rxn.clone()).with_target_species(&product.formula);

        let step1 = StepNode::new(
            "ratio",
            StepType::ApplyStoichiometricRatio,
            "Determine Stoichiometric Mole Ratio",
            format!("The stoichiometric ratio of {} to {} is {} / {}", product.formula, reactant.formula, b, a),
            format!("ratio = {}/{}", b, a),
        )
        .with_hints(vec![
            StepHint::principle("Coefficients in the balanced equation give the mole-to-mole ratio."),
            StepHint::operation(format!("Find the ratio of {} to {}: {} / {}", product.formula, reactant.formula, b, a)),
            StepHint::intermediate_relation(format!("n({}) = n({}) * ({}/{})", product.formula, reactant.formula, b, a)),
        ]);

        let step2 = StepNode::new(
            "calc_moles",
            StepType::FinalAnswer,
            "Calculate Produced Moles",
            format!("{:.1} * ({}/{}) = {:.2} mol", n_reactant, b, a, n_product),
            format!("{:.2}", n_product),
        )
        .with_expected_value(n_product)
        .with_dependencies(vec!["ratio".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Multiply reactant moles by the stoichiometric ratio."),
            StepHint::operation(format!("Compute {:.1} * ({:.0} / {:.0})", n_reactant, b, a)),
            StepHint::intermediate_relation(format!("{:.2} mol {}", n_product, product.formula)),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2], "calc_moles");

        let parameters = json!({
            "difficulty": 2,
            "reaction": rxn.name,
            "equation": rxn.formatted_equation(),
            "n_reactant": n_reactant,
            "n_product": n_product,
            "ratio_coeff_target": b,
            "ratio_coeff_source": a,
            "target_species": product.formula,
            "unit": "mol",
            "physics_metadata": meta,
        });

        let correct_answer = json!({
            "value": n_product,
            "unit": "mol",
            "formatted": format!("{:.2} mol", n_product),
            "solution": format!("From balanced equation, ratio = {}/{}. Moles = {:.1} * ({}/{}) = {:.2} mol", b, a, n_reactant, b, a, n_product),
        });

        ProblemInstance::new(
            format!("inst-chem-stk-l2-{}", seed),
            FAMILY_CHEMISTRY_STOICHIOMETRY,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(json!({
            "difficulty_level": 2,
            "target_time_ms": 35_000,
            "domain": "chemistry",
            "unit": "mol",
        }))
    }

    /// Level 3: Mass-to-Mass Stoichiometry: m_A -> n_A -> n_B -> m_B
    pub fn generate_level_3(seed: u64, _variant: Option<&str>) -> ProblemInstance {
        let mut rng = StdRng::seed_from_u64(seed);

        let rxn = ReactionTemplates::calcium_carbonate_calcination(); // CaCO3 -> CaO + CO2
        let reactant = &rxn.reactants[0].species; // CaCO3, M = 100.086
        let product = &rxn.products[0].species;   // CaO, M = 56.077

        let mass_a = (rng.random_range(2..12) as f64) * 25.0; // 50g, 75g, 100g, etc.
        let moles_a = mass_a / reactant.molar_mass;
        let moles_b = moles_a * 1.0; // 1:1 ratio
        let mass_b = moles_b * product.molar_mass;

        let prompt = format!(
            "For the decomposition reaction:\n$$\\text{{{}}}$$\nIf **{:.1} g** of {} ({}, molar mass = **{:.2} g/mol**) decomposes completely, what mass of {} ({}, molar mass = **{:.2} g/mol**) is produced in **grams**?",
            rxn.formatted_equation(),
            mass_a,
            reactant.name,
            reactant.formatted_formula(),
            reactant.molar_mass,
            product.name,
            product.formatted_formula(),
            product.molar_mass
        );

        let mut meta = ChemicalProblemMetadata::new(ChemicalRegimeKind::StoichiometryMassMass, ChemistryUnit::Gram);
        meta = meta.with_reaction(rxn.clone()).with_target_species(&product.formula);

        let step1 = StepNode::new(
            "calc_moles_a",
            StepType::ConvertMassToMoles,
            "Convert Reactant Mass to Moles",
            format!("{:.1} / {:.2} = {:.4} mol", mass_a, reactant.molar_mass, moles_a),
            format!("{:.4}", moles_a),
        )
        .with_expected_value(moles_a)
        .with_hints(vec![
            StepHint::principle("Divide the given reactant mass by its molar mass to find moles: n = m / M."),
            StepHint::operation(format!("Compute {:.1} / {:.2}", mass_a, reactant.molar_mass)),
            StepHint::intermediate_relation(format!("{:.4} mol {}", moles_a, reactant.formula)),
        ]);

        let step2 = StepNode::new(
            "calc_mass_b",
            StepType::FinalAnswer,
            "Convert Product Moles to Product Mass",
            format!("{:.4} * {:.2} = {:.2} g", moles_b, product.molar_mass, mass_b),
            format!("{:.2}", mass_b),
        )
        .with_expected_value(mass_b)
        .with_dependencies(vec!["calc_moles_a".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Multiply product moles by product molar mass: m = n * M."),
            StepHint::operation(format!("Compute {:.4} * {:.2}", moles_b, product.molar_mass)),
            StepHint::intermediate_relation(format!("{:.2} g {}", mass_b, product.formula)),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2], "calc_mass_b");

        let parameters = json!({
            "difficulty": 3,
            "reaction": rxn.name,
            "equation": rxn.formatted_equation(),
            "mass_a": mass_a,
            "molar_mass_a": reactant.molar_mass,
            "molar_mass_b": product.molar_mass,
            "moles_a": moles_a,
            "mass_b": mass_b,
            "unit": "g",
            "physics_metadata": meta,
        });

        let correct_answer = json!({
            "value": mass_b,
            "unit": "g",
            "formatted": format!("{:.2} g", mass_b),
            "solution": format!("n(CaCO3) = {:.1} / {:.2} = {:.4} mol. 1:1 ratio => n(CaO) = {:.4} mol. m(CaO) = {:.4} * {:.2} = {:.2} g", mass_a, reactant.molar_mass, moles_a, moles_b, moles_b, product.molar_mass, mass_b),
        });

        ProblemInstance::new(
            format!("inst-chem-stk-l3-{}", seed),
            FAMILY_CHEMISTRY_STOICHIOMETRY,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(json!({
            "difficulty_level": 3,
            "target_time_ms": 45_000,
            "domain": "chemistry",
            "unit": "g",
        }))
    }

    /// Level 4: Limiting Reagent Determination & Theoretical Yield
    pub fn generate_level_4(seed: u64, _variant: Option<&str>) -> ProblemInstance {
        let mut rng = StdRng::seed_from_u64(seed);

        let rxn = ReactionTemplates::haber_bosch(); // N2 + 3 H2 -> 2 NH3

        // Create an asymmetric initial mixture where H2 or N2 is strictly limiting
        let n_n2 = rng.random_range(3..10) as f64; // e.g. 4.0 mol
        let n_h2 = rng.random_range(6..20) as f64; // e.g. 9.0 mol

        let ratio_n2 = n_n2 / 1.0;
        let ratio_h2 = n_h2 / 3.0;

        let (limiting_name, limiting_moles, coeff_limiting, theoretical_nh3) = if ratio_n2 < ratio_h2 {
            ("N2", n_n2, 1, n_n2 * 2.0)
        } else {
            ("H2", n_h2, 3, (n_h2 / 3.0) * 2.0)
        };

        let prompt = format!(
            "In the Haber process reaction:\n$$\\text{{{}}}$$\nA reactor is loaded with **{:.1} mol** of N₂(g) and **{:.1} mol** of H₂(g).\nDetermine the limiting reagent, and calculate the maximum theoretical amount of NH₃(g) produced in **moles**.",
            rxn.formatted_equation(),
            n_n2,
            n_h2
        );

        let mut meta = ChemicalProblemMetadata::new(ChemicalRegimeKind::StoichiometryLimitingReagent, ChemistryUnit::Mole);
        meta = meta
            .with_reaction(rxn.clone())
            .with_limiting_reagent(limiting_name)
            .with_target_species("NH3");

        let step1 = StepNode::new(
            "find_limiting",
            StepType::IdentifyLimitingReagent,
            "Identify Limiting Reagent",
            format!("Compare n/coeff: N2 = {:.1}/1 = {:.2}, H2 = {:.1}/3 = {:.2}. Limiting reagent is {}", n_n2, ratio_n2, n_h2, ratio_h2, limiting_name),
            limiting_name.to_string(),
        )
        .with_hints(vec![
            StepHint::principle("The limiting reagent is the reactant that gives the smallest (moles / coefficient) ratio."),
            StepHint::operation(format!("Compute N2 ratio ({:.1}/1) and H2 ratio ({:.1}/3).", n_n2, n_h2)),
            StepHint::intermediate_relation(format!("Limiting reagent is {}", limiting_name)),
        ]);

        let step2 = StepNode::new(
            "calc_yield",
            StepType::FinalAnswer,
            "Calculate Theoretical NH3 Moles",
            format!("Based on limiting {}, NH3 produced = ({:.1}/{}) * 2 = {:.2} mol", limiting_name, limiting_moles, coeff_limiting, theoretical_nh3),
            format!("{:.2}", theoretical_nh3),
        )
        .with_expected_value(theoretical_nh3)
        .with_dependencies(vec!["find_limiting".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Calculate product yield strictly from the limiting reagent amount."),
            StepHint::operation(format!("Multiply moles of {} by the ratio (2 / {})", limiting_name, coeff_limiting)),
            StepHint::intermediate_relation(format!("{:.2} mol NH3", theoretical_nh3)),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2], "calc_yield");

        let parameters = json!({
            "difficulty": 4,
            "n_n2": n_n2,
            "n_h2": n_h2,
            "limiting_reagent": limiting_name,
            "theoretical_nh3": theoretical_nh3,
            "unit": "mol",
            "physics_metadata": meta,
        });

        let correct_answer = json!({
            "value": theoretical_nh3,
            "unit": "mol",
            "formatted": format!("{:.2} mol", theoretical_nh3),
            "solution": format!("N2 ratio = {:.2}, H2 ratio = {:.2} -> {} is limiting. NH3 = ({:.1}/{}) * 2 = {:.2} mol", ratio_n2, ratio_h2, limiting_name, limiting_moles, coeff_limiting, theoretical_nh3),
        });

        ProblemInstance::new(
            format!("inst-chem-stk-l4-{}", seed),
            FAMILY_CHEMISTRY_STOICHIOMETRY,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(json!({
            "difficulty_level": 4,
            "target_time_ms": 50_000,
            "domain": "chemistry",
            "unit": "mol",
        }))
    }

    /// Level 5: Percentage Yield & Multi-Step Transfer
    pub fn generate_level_5(seed: u64, _variant: Option<&str>) -> ProblemInstance {
        let mut rng = StdRng::seed_from_u64(seed);

        let rxn = ReactionTemplates::aluminum_oxidation(); // 4 Al + 3 O2 -> 2 Al2O3
        let al = &rxn.reactants[0].species;   // Al, M = 26.982
        let al2o3 = &rxn.products[0].species; // Al2O3, M = 101.96

        let mass_al = (rng.random_range(4..15) as f64) * 27.0; // e.g. 108g Al
        let moles_al = mass_al / al.molar_mass;
        let theoretical_moles_al2o3 = moles_al * (2.0 / 4.0); // 2 Al2O3 per 4 Al = 0.5
        let theoretical_mass_al2o3 = theoretical_moles_al2o3 * al2o3.molar_mass;

        let yield_pct = rng.random_range(75..95) as f64; // e.g. 85%
        let actual_mass_al2o3 = theoretical_mass_al2o3 * (yield_pct / 100.0);

        let prompt = format!(
            "Aluminum reacts with excess oxygen to produce aluminum oxide according to:\n$$\\text{{{}}}$$\nIf **{:.1} g** of Al (molar mass = **{:.2} g/mol**) is reacted with excess oxygen and the reaction proceeds with a **{:.0}% yield**, what is the **actual mass of Al₂O₃** (molar mass = **{:.2} g/mol**) isolated in grams?",
            rxn.formatted_equation(),
            mass_al,
            al.molar_mass,
            yield_pct,
            al2o3.molar_mass
        );

        let mut meta = ChemicalProblemMetadata::new(ChemicalRegimeKind::StoichiometryPercentageYield, ChemistryUnit::Gram);
        meta = meta.with_reaction(rxn.clone()).with_target_species("Al2O3");

        let step1 = StepNode::new(
            "theo_mass",
            StepType::ApplyStoichiometricRatio,
            "Calculate Theoretical Al2O3 Mass",
            format!("n(Al) = {:.1}/{:.2} = {:.3} mol. Theo n(Al2O3) = {:.3} * (2/4) = {:.3} mol. Theo mass = {:.2} g", mass_al, al.molar_mass, moles_al, moles_al, theoretical_moles_al2o3, theoretical_mass_al2o3),
            format!("{:.2}", theoretical_mass_al2o3),
        )
        .with_expected_value(theoretical_mass_al2o3)
        .with_hints(vec![
            StepHint::principle("Find the 100% theoretical yield before applying the percentage yield factor."),
            StepHint::operation(format!("Convert {:.1} g Al to moles, multiply by (2/4), then multiply by {:.2} g/mol.", mass_al, al2o3.molar_mass)),
            StepHint::intermediate_relation(format!("Theoretical yield = {:.2} g", theoretical_mass_al2o3)),
        ]);

        let step2 = StepNode::new(
            "actual_mass",
            StepType::FinalAnswer,
            "Calculate Actual Mass Isolated",
            format!("Actual mass = {:.2} * ({:.0}/100) = {:.2} g", theoretical_mass_al2o3, yield_pct, actual_mass_al2o3),
            format!("{:.2}", actual_mass_al2o3),
        )
        .with_expected_value(actual_mass_al2o3)
        .with_dependencies(vec!["theo_mass".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Actual yield = Theoretical yield * (Percentage yield / 100)."),
            StepHint::operation(format!("Multiply {:.2} g by {:.2}.", theoretical_mass_al2o3, yield_pct / 100.0)),
            StepHint::intermediate_relation(format!("{:.2} g Al2O3", actual_mass_al2o3)),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2], "actual_mass");

        let parameters = json!({
            "difficulty": 5,
            "mass_al": mass_al,
            "yield_percent": yield_pct,
            "theoretical_mass_al2o3": theoretical_mass_al2o3,
            "actual_mass_al2o3": actual_mass_al2o3,
            "unit": "g",
            "physics_metadata": meta,
        });

        let correct_answer = json!({
            "value": actual_mass_al2o3,
            "unit": "g",
            "formatted": format!("{:.2} g", actual_mass_al2o3),
            "solution": format!("Theo yield = {:.2} g. Actual yield ({:.0}%) = {:.2} g", theoretical_mass_al2o3, yield_pct, actual_mass_al2o3),
        });

        ProblemInstance::new(
            format!("inst-chem-stk-l5-{}", seed),
            FAMILY_CHEMISTRY_STOICHIOMETRY,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(json!({
            "difficulty_level": 5,
            "target_time_ms": 60_000,
            "domain": "chemistry",
            "unit": "g",
        }))
    }
}

impl ProblemGenerator for StoichiometryGenerator {
    fn family_id(&self) -> &str {
        FAMILY_CHEMISTRY_STOICHIOMETRY
    }

    fn template_ref(&self) -> &str {
        TEMPLATE_CHEMISTRY_STOICHIOMETRY_V1
    }

    fn supported_variants(&self) -> Vec<String> {
        vec![
            "mass_to_moles".to_string(),
            "moles_to_mass".to_string(),
            "reaction_mole_ratio".to_string(),
            "mass_to_mass".to_string(),
            "limiting_reagent".to_string(),
            "percentage_yield".to_string(),
        ]
    }

    fn target_latency_ms(&self, difficulty_level: u32) -> u64 {
        match difficulty_level {
            1 => 25_000,
            2 => 35_000,
            3 => 45_000,
            4 => 50_000,
            _ => 60_000,
        }
    }

    fn generate(
        &self,
        _family_id: &ProblemFamilyId,
        seed: u64,
        difficulty_level: u32,
        variant: Option<&str>,
    ) -> Result<ProblemInstance> {
        Ok(Self::generate_problem(seed, difficulty_level, variant))
    }
}

pub struct StoichiometryValidator;

impl ProblemValidator for StoichiometryValidator {
    fn family_id(&self) -> &str {
        FAMILY_CHEMISTRY_STOICHIOMETRY
    }

    fn evaluate(
        &self,
        instance: &ProblemInstance,
        student_answer: &serde_json::Value,
        time_taken_ms: u64,
        target_time_ms: u64,
    ) -> AnswerEvaluation {
        let expected_val = instance
            .correct_answer
            .get("value")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let parsed_val = match student_answer {
            serde_json::Value::Number(n) => n.as_f64(),
            serde_json::Value::String(s) => NumericAnswerParser::parse_string(s),
            serde_json::Value::Object(map) => map.get("value").and_then(|v| v.as_f64()),
            _ => None,
        };

        let Some(val) = parsed_val else {
            return AnswerEvaluation::incorrect(
                ErrorCategory::Syntax,
                "Unable to parse numerical chemistry answer.".to_string(),
            );
        };

        // Invariant check: mass/moles must be non-negative
        if val < -1e-4 {
            return AnswerEvaluation::incorrect(
                ErrorCategory::Concept,
                "Chemical quantity violates physical non-negativity constraint.".to_string(),
            )
            .with_parsed_values(val, expected_val);
        }

        let is_correct = (val - expected_val).abs() <= 0.05 * expected_val.abs().max(1e-4);

        if is_correct {
            let score = if time_taken_ms <= target_time_ms {
                1.0
            } else {
                (1.0 - ((time_taken_ms - target_time_ms) as f64 / target_time_ms as f64) * 0.5).max(0.5)
            };
            AnswerEvaluation::correct(score, time_taken_ms, target_time_ms)
                .with_parsed_values(val, expected_val)
        } else {
            // Misconception diagnostic checks
            let difficulty = instance.parameters.get("difficulty").and_then(|v| v.as_u64()).unwrap_or(1);

            let (cat, msg) = if difficulty == 2 {
                // Inverted stoichiometric ratio check
                let b = instance.parameters.get("ratio_coeff_target").and_then(|v| v.as_f64()).unwrap_or(1.0);
                let a = instance.parameters.get("ratio_coeff_source").and_then(|v| v.as_f64()).unwrap_or(1.0);
                let n_reactant = instance.parameters.get("n_reactant").and_then(|v| v.as_f64()).unwrap_or(1.0);
                let inverted_val = n_reactant * (a / b);
                if (val - inverted_val).abs() <= 0.05 * inverted_val.abs() {
                    (
                        ErrorCategory::Strategy,
                        format!("Stoichiometric Ratio Error: Inverted the mole ratio (used {}/{} instead of {}/{}).", a, b, b, a),
                    )
                } else {
                    (ErrorCategory::Calculation, "Incorrect stoichiometric mole calculation.".to_string())
                }
            } else if difficulty == 4 {
                // Limiting reagent error: calculated from excess instead of limiting
                (ErrorCategory::Strategy, "Limiting Reagent Error: Check which reactant is completely consumed first.".to_string())
            } else {
                (ErrorCategory::Calculation, "Incorrect stoichiometric value calculation.".to_string())
            };

            AnswerEvaluation::incorrect(cat, msg.to_string())
                .with_parsed_values(val, expected_val)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stoichiometry_generation_all_levels() {
        for diff in 1..=5 {
            let inst = StoichiometryGenerator::generate_problem(12345 + diff as u64, diff, None);
            assert!(!inst.rendered_prompt.is_empty());
            assert!(inst.solution_graph().is_some());
            assert!(inst.correct_answer.get("value").is_some());
        }
    }

    #[test]
    fn test_stoichiometry_validator_correct_and_inverted_ratio() {
        let validator = StoichiometryValidator;

        let inst = StoichiometryGenerator::generate_problem(999, 2, None);
        let correct_val = inst.correct_answer.get("value").unwrap().as_f64().unwrap();

        // Correct evaluation
        let eval = validator.evaluate(&inst, &serde_json::json!(correct_val), 20000, 35000);
        assert!(eval.is_correct);

        // Inverted ratio misconception
        let a = inst.parameters.get("ratio_coeff_source").unwrap().as_f64().unwrap();
        let b = inst.parameters.get("ratio_coeff_target").unwrap().as_f64().unwrap();
        let n_r = inst.parameters.get("n_reactant").unwrap().as_f64().unwrap();
        let bad_inverted = n_r * (a / b);

        let eval_bad = validator.evaluate(&inst, &serde_json::json!(bad_inverted), 20000, 35000);
        assert!(!eval_bad.is_correct);
        assert_eq!(eval_bad.error_category, Some(ErrorCategory::Strategy));
    }
}
