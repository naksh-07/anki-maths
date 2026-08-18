use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde_json::json;

use crate::chemistry::models::{ChemicalProblemMetadata, ChemicalRegimeKind};
use crate::chemistry::reaction::ReactionTemplates;
use crate::chemistry::species::SpeciesCatalog;
use crate::chemistry::units::ChemistryUnit;
use crate::core::{ProblemFamilyId, Result};
use crate::diagnostics::ErrorCategory;
use crate::problems::catalog::{FAMILY_CHEMISTRY_EQUILIBRIUM, TEMPLATE_CHEMISTRY_EQUILIBRIUM_V1};
use crate::problems::generator::ProblemGenerator;
use crate::problems::steps::{SolutionGraph, StepHint, StepNode, StepType};
use crate::problems::validator::{AnswerEvaluation, NumericAnswerParser, ProblemValidator};
use crate::problems::ProblemInstance;

pub struct EquilibriumGenerator;

impl EquilibriumGenerator {
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
    /// Level 1: Solution Molarity & Unit Conversion: M = n / V = m / (M_mol * V)
    pub fn generate_level_1(seed: u64, _variant: Option<&str>) -> ProblemInstance {
        let mut rng = StdRng::seed_from_u64(seed);

        let species_choices = [
            SpeciesCatalog::sodium_hydroxide(), // NaOH, M = 39.997
            SpeciesCatalog::sodium_chloride(),   // NaCl, M = 58.44
            SpeciesCatalog::hydrochloric_acid(), // HCl, M = 36.46
        ];
        let spec_idx = rng.random_range(0..species_choices.len());
        let species = &species_choices[spec_idx];

        let vol_ml = (rng.random_range(2..10) as f64) * 250.0; // 500 mL, 750 mL, 1000 mL, etc.
        let vol_l = vol_ml / 1000.0;
        let molarity = (rng.random_range(2..12) as f64) * 0.1; // 0.2 M to 1.2 M
        let moles = molarity * vol_l;
        let mass = moles * species.molar_mass;

        let prompt = format!(
            "A chemist dissolves **{:.2} g** of {} ({}, molar mass = **{:.2} g/mol**) in distilled water to make exactly **{:.0} mL** of solution.\nCalculate the molar concentration (**Molarity, M**) of the resulting solution.",
            mass, species.name, species.formatted_formula(), species.molar_mass, vol_ml
        );

        let mut meta = ChemicalProblemMetadata::new(ChemicalRegimeKind::ConcentrationMolarity, ChemistryUnit::Molar);
        meta = meta.with_target_species(&species.formula);

        let step1 = StepNode::new(
            "moles",
            StepType::ConvertMassToMoles,
            "Convert Solute Mass to Moles",
            format!("n = {:.2} / {:.2} = {:.4} mol", mass, species.molar_mass, moles),
            format!("{:.4}", moles),
        )
        .with_expected_value(moles)
        .with_hints(vec![
            StepHint::principle("Molarity requires the amount of solute in moles divided by volume in liters."),
            StepHint::operation(format!("Divide mass ({:.2} g) by molar mass ({:.2} g/mol).", mass, species.molar_mass)),
            StepHint::intermediate_relation(format!("n = {:.4} mol", moles)),
        ]);

        let step2 = StepNode::new(
            "molarity",
            StepType::FinalAnswer,
            "Calculate Molarity",
            format!("M = {:.4} mol / {:.3} L = {:.3} M", moles, vol_l, molarity),
            format!("{:.3}", molarity),
        )
        .with_expected_value(molarity)
        .with_dependencies(vec!["moles".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Convert solution volume from mL to L (divide by 1000), then divide moles by liters."),
            StepHint::operation(format!("Compute {:.4} / {:.3}", moles, vol_l)),
            StepHint::intermediate_relation(format!("{:.3} M", molarity)),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2], "molarity");

        let parameters = json!({
            "difficulty": 1,
            "solute": species.formula,
            "mass": mass,
            "molar_mass": species.molar_mass,
            "vol_ml": vol_ml,
            "vol_l": vol_l,
            "molarity": molarity,
            "unit": "M",
            "physics_metadata": meta,
        });

        let correct_answer = json!({
            "value": molarity,
            "unit": "M",
            "formatted": format!("{:.3} M", molarity),
            "solution": format!("Moles = {:.2}/{:.2} = {:.4} mol. Volume = {:.3} L. Molarity = {:.4}/{:.3} = {:.3} M", mass, species.molar_mass, moles, vol_l, moles, vol_l, molarity),
        });

        ProblemInstance::new(
            format!("inst-chem-eq-l1-{}", seed),
            FAMILY_CHEMISTRY_EQUILIBRIUM,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(json!({
            "difficulty_level": 1,
            "target_time_ms": 30_000,
            "domain": "chemistry",
            "unit": "M",
        }))
    }

    /// Level 2: Equilibrium Constant Expression Kc = [Products]^p / [Reactants]^r
    pub fn generate_level_2(seed: u64, _variant: Option<&str>) -> ProblemInstance {
        let mut rng = StdRng::seed_from_u64(seed);

        let rxn = ReactionTemplates::pcl5_decomposition(); // PCl5(g) ⇌ PCl3(g) + Cl2(g)
        let conc_pcl5 = (rng.random_range(1..6) as f64) * 0.1; // 0.1 to 0.5 M
        let conc_pcl3 = (rng.random_range(2..8) as f64) * 0.05; // 0.10 to 0.35 M
        let conc_cl2 = conc_pcl3; // Equal in simple dissociation

        let kc = (conc_pcl3 * conc_cl2) / conc_pcl5;

        let prompt = format!(
            "At a specific temperature, the gas-phase equilibrium reaction:\n$$\\text{{{}}}$$\nreaches equilibrium with measured concentrations:\n- $[\\text{{PCl}}_5] = {:.2}\\text{{ M}}$\n- $[\\text{{PCl}}_3] = {:.3}\\text{{ M}}$\n- $[\\text{{Cl}}_2] = {:.3}\\text{{ M}}$\n\nCalculate the equilibrium constant $K_c$ for this reaction.",
            rxn.formatted_equation(),
            conc_pcl5,
            conc_pcl3,
            conc_cl2
        );

        let mut meta = ChemicalProblemMetadata::new(ChemicalRegimeKind::EquilibriumConstantExpression, ChemistryUnit::Dimensionless);
        meta = meta.with_reaction(rxn.clone()).with_equilibrium_constant(kc);

        let step1 = StepNode::new(
            "kc_expression",
            StepType::ConstructEquilibriumExpression,
            "Formulate Kc Expression",
            "Kc = ([PCl3] * [Cl2]) / [PCl5]".to_string(),
            "Kc = ([PCl3]*[Cl2])/[PCl5]".to_string(),
        )
        .with_hints(vec![
            StepHint::principle("The equilibrium constant Kc is the ratio of product concentrations to reactant concentrations, each raised to their stoichiometric powers."),
            StepHint::operation("Write Kc = [PCl3][Cl2] / [PCl5]."),
            StepHint::intermediate_relation("Kc = ([PCl3]*[Cl2]) / [PCl5]"),
        ]);

        let step2 = StepNode::new(
            "calc_kc",
            StepType::FinalAnswer,
            "Calculate Kc Value",
            format!("({:.3} * {:.3}) / {:.2} = {:.4}", conc_pcl3, conc_cl2, conc_pcl5, kc),
            format!("{:.4}", kc),
        )
        .with_expected_value(kc)
        .with_dependencies(vec!["kc_expression".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Substitute equilibrium concentrations directly into the expression."),
            StepHint::operation(format!("Compute ({:.3} * {:.3}) / {:.2}", conc_pcl3, conc_cl2, conc_pcl5)),
            StepHint::intermediate_relation(format!("Kc = {:.4}", kc)),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2], "calc_kc");

        let parameters = json!({
            "difficulty": 2,
            "equation": rxn.formatted_equation(),
            "conc_pcl5": conc_pcl5,
            "conc_pcl3": conc_pcl3,
            "conc_cl2": conc_cl2,
            "kc": kc,
            "unit": "",
            "physics_metadata": meta,
        });

        let correct_answer = json!({
            "value": kc,
            "unit": "",
            "formatted": format!("{:.4}", kc),
            "solution": format!("Kc = ([PCl3]*[Cl2])/[PCl5] = ({:.3} * {:.3}) / {:.2} = {:.4}", conc_pcl3, conc_cl2, conc_pcl5, kc),
        });

        ProblemInstance::new(
            format!("inst-chem-eq-l2-{}", seed),
            FAMILY_CHEMISTRY_EQUILIBRIUM,
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
            "unit": "",
        }))
    }

    /// Level 3: ICE Table (Initial, Change, Equilibrium) Analysis
    pub fn generate_level_3(seed: u64, _variant: Option<&str>) -> ProblemInstance {
        let mut rng = StdRng::seed_from_u64(seed);

        let rxn = ReactionTemplates::hydrogen_iodide_equilibrium(); // H2 + I2 ⇌ 2 HI
        let init_h2 = (rng.random_range(5..12) as f64) * 0.1; // e.g. 0.8 M
        let init_i2 = init_h2;
        let x = (rng.random_range(2..6) as f64) * 0.1; // extent of reaction: 0.2 to 0.5 M

        let eq_h2 = init_h2 - x;
        let eq_i2 = init_i2 - x;
        let eq_hi = 2.0 * x;

        let kc = (eq_hi * eq_hi) / (eq_h2 * eq_i2);

        let prompt = format!(
            "In a closed container at $450^\\circ\\text{{C}}$, an initial mixture contains **{:.2} M** of H₂(g) and **{:.2} M** of I₂(g) with no initial HI.\nAt equilibrium according to:\n$$\\text{{{}}}$$\nthe equilibrium concentration of HI(g) is measured to be **{:.2} M**.\nCalculate the equilibrium constant $K_c$.",
            init_h2,
            init_i2,
            rxn.formatted_equation(),
            eq_hi
        );

        let mut meta = ChemicalProblemMetadata::new(ChemicalRegimeKind::EquilibriumIceTable, ChemistryUnit::Dimensionless);
        meta = meta.with_reaction(rxn.clone()).with_equilibrium_constant(kc);

        let step1 = StepNode::new(
            "ice_change",
            StepType::ConstructEquilibriumExpression,
            "Determine Equilibrium Concentrations via ICE",
            format!("[HI]eq = 2x = {:.2} -> x = {:.2}. [H2]eq = [I2]eq = {:.2} - {:.2} = {:.2} M", eq_hi, x, init_h2, x, eq_h2),
            format!("{:.2}", eq_h2),
        )
        .with_expected_value(eq_h2)
        .with_hints(vec![
            StepHint::principle("Use an ICE table: Change in H2 and I2 is -x, change in HI is +2x."),
            StepHint::operation(format!("Since [HI] = 2x = {:.2} M, x = {:.2} M. [H2]eq = {:.2} - {:.2} M.", eq_hi, x, init_h2, x)),
            StepHint::intermediate_relation(format!("[H2]eq = {:.2} M", eq_h2)),
        ]);

        let step2 = StepNode::new(
            "calc_kc",
            StepType::FinalAnswer,
            "Calculate Kc",
            format!("Kc = [HI]^2 / ([H2]*[I2]) = ({:.2})^2 / ({:.2} * {:.2}) = {:.2}", eq_hi, eq_h2, eq_i2, kc),
            format!("{:.2}", kc),
        )
        .with_expected_value(kc)
        .with_dependencies(vec!["ice_change".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Substitute equilibrium concentrations: Kc = [HI]^2 / ([H2][I2])."),
            StepHint::operation(format!("Compute ({:.2})^2 / ({:.2})^2", eq_hi, eq_h2)),
            StepHint::intermediate_relation(format!("Kc = {:.2}", kc)),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2], "calc_kc");

        let parameters = json!({
            "difficulty": 3,
            "init_h2": init_h2,
            "eq_hi": eq_hi,
            "eq_h2": eq_h2,
            "kc": kc,
            "unit": "",
            "physics_metadata": meta,
        });

        let correct_answer = json!({
            "value": kc,
            "unit": "",
            "formatted": format!("{:.2}", kc),
            "solution": format!("x = {:.2}/2 = {:.2} M. [H2]eq = {:.2} M. Kc = ({:.2})^2 / ({:.2})^2 = {:.2}", eq_hi, x, eq_h2, eq_hi, eq_h2, kc),
        });

        ProblemInstance::new(
            format!("inst-chem-eq-l3-{}", seed),
            FAMILY_CHEMISTRY_EQUILIBRIUM,
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
            "unit": "",
        }))
    }

    /// Level 4: Quadratic Equilibrium Calculation from Kc and Initial Concentration
    pub fn generate_level_4(seed: u64, _variant: Option<&str>) -> ProblemInstance {
        let mut rng = StdRng::seed_from_u64(seed);

        // Reverse representation: N2O4(g) ⇌ 2 NO2(g) for convenient single dissociation
        // N2O4 ⇌ 2 NO2. Initial [N2O4] = c0, Change = -x, +2x -> Kc = (2x)^2 / (c0 - x) = 4x^2 / (c0 - x)
        let c0 = (rng.random_range(4..10) as f64) * 0.1; // 0.4 to 0.9 M
        let x = (rng.random_range(1..4) as f64) * 0.05;  // 0.05 to 0.15 M

        let eq_n2o4 = c0 - x;
        let eq_no2 = 2.0 * x;
        let kc = (eq_no2 * eq_no2) / eq_n2o4; // e.g. (2x)^2 / (c0 - x)

        let prompt = format!(
            "Dinitrogen tetroxide dissociates into nitrogen dioxide:\n$$\\text{{N}}_2\\text{{O}}_4(g) \\rightleftharpoons 2 \\text{{NO}}_2(g)$$\nAt a given temperature, $K_c = {:.3}$. If initial $[\\text{{N}}_2\\text{{O}}_4] = {:.2}\\text{{ M}}$ with no initial NO₂, calculate the **equilibrium concentration of NO₂(g)** in **M**.",
            kc,
            c0
        );

        let mut meta = ChemicalProblemMetadata::new(ChemicalRegimeKind::EquilibriumQuadraticCalculation, ChemistryUnit::Molar);
        meta = meta.with_equilibrium_constant(kc).with_target_species("NO2");

        let step1 = StepNode::new(
            "setup_eq",
            StepType::ConstructEquilibriumExpression,
            "Set Up Equilibrium Algebraic Equation",
            format!("Kc = (2x)^2 / ({:.2} - x) = {:.3} -> 4x^2 + {:.3}x - {:.3} = 0", c0, kc, kc, kc * c0),
            format!("{:.3}", kc),
        )
        .with_hints(vec![
            StepHint::principle("Set up the ICE table: [N2O4]eq = c0 - x, [NO2]eq = 2x. Substitute into Kc = [NO2]^2 / [N2O4]."),
            StepHint::operation(format!("Substitute into Kc = 4x^2 / ({:.2} - x) = {:.3}", c0, kc)),
            StepHint::intermediate_relation(format!("4x^2 = {:.3} * ({:.2} - x)", kc, c0)),
        ]);

        let step2 = StepNode::new(
            "calc_no2",
            StepType::FinalAnswer,
            "Solve for [NO2]eq = 2x",
            format!("x = {:.4} M -> [NO2]eq = 2 * {:.4} = {:.3} M", x, x, eq_no2),
            format!("{:.3}", eq_no2),
        )
        .with_expected_value(eq_no2)
        .with_dependencies(vec!["setup_eq".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Solve the quadratic for positive root x, then compute [NO2]eq = 2x."),
            StepHint::operation(format!("Compute 2 * {:.4}", x)),
            StepHint::intermediate_relation(format!("[NO2]eq = {:.3} M", eq_no2)),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2], "calc_no2");

        let parameters = json!({
            "difficulty": 4,
            "c0": c0,
            "kc": kc,
            "x": x,
            "eq_no2": eq_no2,
            "unit": "M",
            "physics_metadata": meta,
        });

        let correct_answer = json!({
            "value": eq_no2,
            "unit": "M",
            "formatted": format!("{:.3} M", eq_no2),
            "solution": format!("Kc = 4x^2 / ({:.2} - x) = {:.3} => x = {:.4} M. [NO2]eq = 2x = {:.3} M", c0, kc, x, eq_no2),
        });

        ProblemInstance::new(
            format!("inst-chem-eq-l4-{}", seed),
            FAMILY_CHEMISTRY_EQUILIBRIUM,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(json!({
            "difficulty_level": 4,
            "target_time_ms": 60_000,
            "domain": "chemistry",
            "unit": "M",
        }))
    }

    /// Level 5: Le Chatelier Principle Response & Reaction Quotient Qc Transfer
    pub fn generate_level_5(seed: u64, _variant: Option<&str>) -> ProblemInstance {
        let mut rng = StdRng::seed_from_u64(seed);

        let rxn = ReactionTemplates::sulfur_trioxide_equilibrium(); // 2 SO2 + O2 ⇌ 2 SO3
        // Given equilibrium state with Kc:
        let eq_so2 = 0.20;
        let eq_o2 = 0.10;
        let eq_so3 = 0.40;
        let kc = (eq_so3 * eq_so3) / (eq_so2 * eq_so2 * eq_o2); // (0.16) / (0.04 * 0.1) = 0.16 / 0.004 = 40.0

        // Disturbance: Add additional SO2
        let added_so2 = (rng.random_range(1..5) as f64) * 0.1; // e.g. +0.2 M -> [SO2]_init = 0.40 M
        let inst_so2 = eq_so2 + added_so2;

        // Calculate Qc at the instant of disturbance
        let qc = (eq_so3 * eq_so3) / (inst_so2 * inst_so2 * eq_o2);

        let prompt = format!(
            "For the equilibrium reaction:\n$$\\text{{{}}}$$\nwith $K_c = {:.1}$ at $700\\text{{ K}}$, a system currently has concentrations:\n- $[\\text{{SO}}_2] = {:.2}\\text{{ M}}$\n- $[\\text{{O}}_2] = {:.2}\\text{{ M}}$\n- $[\\text{{SO}}_3] = {:.2}\\text{{ M}}$\n\nCalculate the **Reaction Quotient $Q_c$** immediately after injecting additional SO₂ to raise $[\\text{{SO}}_2]$ to **{:.2} M**.",
            rxn.formatted_equation(),
            kc,
            eq_so2,
            eq_o2,
            eq_so3,
            inst_so2
        );

        let mut meta = ChemicalProblemMetadata::new(ChemicalRegimeKind::EquilibriumLeChatelier, ChemistryUnit::Dimensionless);
        meta = meta.with_reaction(rxn.clone()).with_equilibrium_constant(kc);

        let step1 = StepNode::new(
            "qc_expr",
            StepType::ConstructEquilibriumExpression,
            "Formulate Qc Expression",
            "Qc = [SO3]^2 / ([SO2]^2 * [O2])".to_string(),
            "Qc = ([SO3]^2)/(([SO2]^2)*[O2])".to_string(),
        )
        .with_hints(vec![
            StepHint::principle("The reaction quotient Qc has the identical mathematical form as Kc, but uses instantaneous non-equilibrium concentrations."),
            StepHint::operation("Write Qc = [SO3]^2 / ([SO2]^2 * [O2])."),
            StepHint::intermediate_relation("Qc = [SO3]^2 / ([SO2]^2 * [O2])"),
        ]);

        let step2 = StepNode::new(
            "calc_qc",
            StepType::FinalAnswer,
            "Calculate Instantaneous Qc",
            format!("Qc = ({:.2})^2 / (({:.2})^2 * {:.2}) = {:.3}", eq_so3, inst_so2, eq_o2, qc),
            format!("{:.3}", qc),
        )
        .with_expected_value(qc)
        .with_dependencies(vec!["qc_expr".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Substitute the new instantaneous concentration of SO2 along with existing O2 and SO3."),
            StepHint::operation(format!("Compute ({:.2})^2 / (({:.2})^2 * {:.2})", eq_so3, inst_so2, eq_o2)),
            StepHint::intermediate_relation(format!("Qc = {:.3} (Note: Qc < Kc -> shifts forward)", qc)),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2], "calc_qc");

        let parameters = json!({
            "difficulty": 5,
            "kc": kc,
            "inst_so2": inst_so2,
            "eq_o2": eq_o2,
            "eq_so3": eq_so3,
            "qc": qc,
            "unit": "",
            "physics_metadata": meta,
        });

        let correct_answer = json!({
            "value": qc,
            "unit": "",
            "formatted": format!("{:.3}", qc),
            "solution": format!("Qc = ({:.2})^2 / (({:.2})^2 * {:.2}) = {:.3} (Since Qc < Kc, reaction shifts towards products)", eq_so3, inst_so2, eq_o2, qc),
        });

        ProblemInstance::new(
            format!("inst-chem-eq-l5-{}", seed),
            FAMILY_CHEMISTRY_EQUILIBRIUM,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(json!({
            "difficulty_level": 5,
            "target_time_ms": 50_000,
            "domain": "chemistry",
            "unit": "",
        }))
    }
}

impl ProblemGenerator for EquilibriumGenerator {
    fn family_id(&self) -> &str {
        FAMILY_CHEMISTRY_EQUILIBRIUM
    }

    fn template_ref(&self) -> &str {
        TEMPLATE_CHEMISTRY_EQUILIBRIUM_V1
    }

    fn supported_variants(&self) -> Vec<String> {
        vec![
            "solution_molarity".to_string(),
            "equilibrium_expression".to_string(),
            "ice_table".to_string(),
            "quadratic_equilibrium".to_string(),
            "le_chatelier_qc".to_string(),
        ]
    }

    fn target_latency_ms(&self, difficulty_level: u32) -> u64 {
        match difficulty_level {
            1 => 30_000,
            2 => 35_000,
            3 => 45_000,
            4 => 60_000,
            _ => 50_000,
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

pub struct EquilibriumValidator;

impl ProblemValidator for EquilibriumValidator {
    fn family_id(&self) -> &str {
        FAMILY_CHEMISTRY_EQUILIBRIUM
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
                "Unable to parse numerical equilibrium answer.".to_string(),
            );
        };

        // Invariant check: concentration or Kc must be strictly positive
        if val <= 0.0 {
            return AnswerEvaluation::incorrect(
                ErrorCategory::Concept,
                "Chemical equilibrium constant and concentration must be strictly positive.".to_string(),
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
            // Check for inverted Kc expression misconception
            if expected_val > 0.0 {
                let inverted_val = 1.0 / expected_val;
                if (val - inverted_val).abs() <= 0.05 * inverted_val.abs() {
                    return AnswerEvaluation::incorrect(
                        ErrorCategory::Strategy,
                        "Equilibrium Expression Error: Inverted the Kc expression (Reactants / Products instead of Products / Reactants).".to_string(),
                    )
                    .with_parsed_values(val, expected_val);
                }
            }

            AnswerEvaluation::incorrect(
                ErrorCategory::Calculation,
                "Incorrect equilibrium value calculation.".to_string(),
            )
            .with_parsed_values(val, expected_val)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equilibrium_generation_all_levels() {
        for diff in 1..=5 {
            let inst = EquilibriumGenerator::generate_problem(54321 + diff as u64, diff, None);
            assert!(!inst.rendered_prompt.is_empty());
            assert!(inst.solution_graph().is_some());
            assert!(inst.correct_answer.get("value").is_some());
        }
    }

    #[test]
    fn test_equilibrium_validator_correct_and_inverted_kc() {
        let validator = EquilibriumValidator;

        let inst = EquilibriumGenerator::generate_problem(777, 2, None);
        let correct_val = inst.correct_answer.get("value").unwrap().as_f64().unwrap();

        // Correct evaluation
        let eval = validator.evaluate(&inst, &serde_json::json!(correct_val), 25000, 35000);
        assert!(eval.is_correct);

        // Inverted Kc expression misconception
        let inverted_kc = 1.0 / correct_val;
        let eval_inv = validator.evaluate(&inst, &serde_json::json!(inverted_kc), 25000, 35000);
        assert!(!eval_inv.is_correct);
        assert_eq!(eval_inv.error_category, Some(ErrorCategory::Strategy));
    }
}
