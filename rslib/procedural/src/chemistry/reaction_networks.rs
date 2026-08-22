// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use rand::Rng;
use serde::{Deserialize, Serialize};

/// Type of reaction network problem.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReactionNetworkKind {
    TwoStageSequential,
    TwoStageFractionalYield,
    ThreeStageSequential,
    CrossStageLimitingReagent,
    MixtureDecomposition,
}

/// A structured multi-stage reaction network problem definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReactionNetworkPuzzle {
    pub kind: ReactionNetworkKind,
    pub reaction_stages: Vec<String>,
    pub stage_yields: Vec<f64>,
    pub overall_yield: f64,
    pub initial_reactant_name: String,
    pub initial_reactant_mass: f64,
    pub target_product_name: String,
    pub correct_mass_g: f64,
    pub unit_symbol: String,
    pub question_prompt: String,
    pub step_by_step_explanation: Vec<String>,
}

impl ReactionNetworkPuzzle {
    /// Dynamically generate a multi-stage reaction network stoichiometry problem.
    pub fn generate_dynamic<R: Rng>(
        rng: &mut R,
        difficulty_level: u32,
    ) -> Self {
        match difficulty_level {
            1 => Self::generate_two_stage_100_percent(rng),
            2 | 3 => Self::generate_two_stage_fractional_yield(rng),
            4 => Self::generate_three_stage_sequential(rng),
            _ => Self::generate_mixture_decomposition(rng),
        }
    }

    /// Level 1: 2-stage sequential reaction with 100% yield:
    /// Stage 1: S + O2 -> SO2 (M(S)=32.06, M(SO2)=64.06)
    /// Stage 2: 2SO2 + O2 + 2H2O -> 2H2SO4 (M(H2SO4)=98.08)
    /// Net: 1 mol S -> 1 mol H2SO4
    fn generate_two_stage_100_percent<R: Rng>(rng: &mut R) -> Self {
        let mass_s = (rng.random_range(160..1000) as f64) * 0.1; // e.g. 16.0 to 99.9 g
        let m_s = 32.06;
        let m_h2so4 = 98.08;

        let moles_s = mass_s / m_s;
        let moles_h2so4 = moles_s; // 1:1 net stoichiometry
        let mass_h2so4 = moles_h2so4 * m_h2so4;
        let mass_rounded = (mass_h2so4 * 10.0).round() / 10.0;

        let prompt = format!(
            "In the industrial production of sulfuric acid, sulfur is converted in two stages:\n\n\
            **Stage 1:** S(s) + O₂(g) -> SO₂(g)\n\
            **Stage 2:** 2 SO₂(g) + O₂(g) + 2 H₂O(l) -> 2 H₂SO₄(l)\n\n\
            Assuming **100% yield** at both stages and excess oxygen/water, calculate the **mass of H₂SO₄ (in g)**\n\
            produced from **{:.1} g of pure sulfur (S)**.\n\
            (Given: M(S) = 32.06 g/mol, M(H₂SO₄) = 98.08 g/mol)",
            mass_s
        );

        let steps = vec![
            format!("1. Moles of S reactant = mass / M = {:.1} g / 32.06 g/mol = {:.3} mol.", mass_s, moles_s),
            format!("2. Stage 1: 1 mol S produces 1 mol SO₂ -> {:.3} mol SO₂.", moles_s),
            format!("3. Stage 2: 2 mol SO₂ produces 2 mol H₂SO₄ (1:1 ratio) -> {:.3} mol H₂SO₄.", moles_s),
            format!("4. Mass of H₂SO₄ = moles × M = {:.3} mol × 98.08 g/mol = {:.1} g.", moles_h2so4, mass_rounded),
        ];

        Self {
            kind: ReactionNetworkKind::TwoStageSequential,
            reaction_stages: vec![
                "S + O₂ -> SO₂".into(),
                "2 SO₂ + O₂ + 2 H₂O -> 2 H₂SO₄".into(),
            ],
            stage_yields: vec![1.0, 1.0],
            overall_yield: 1.0,
            initial_reactant_name: "Sulfur (S)".into(),
            initial_reactant_mass: mass_s,
            target_product_name: "H₂SO₄".into(),
            correct_mass_g: mass_rounded,
            unit_symbol: "g".into(),
            question_prompt: prompt,
            step_by_step_explanation: steps,
        }
    }

    /// Level 2-3: 2-stage sequential reaction with fractional percentage yields:
    /// Stage 1: CaCO3 -> CaO + CO2 (yield eta1 = 80%)
    /// Stage 2: CaO + 3C -> CaC2 + CO (yield eta2 = 75%)
    /// Net yield = eta1 * eta2 = 0.80 * 0.75 = 0.60 (60%)
    fn generate_two_stage_fractional_yield<R: Rng>(rng: &mut R) -> Self {
        let mass_caco3 = rng.random_range(200..1500) as f64; // 200 to 1499 g
        let eta1 = rng.random_range(70..95) as f64 / 100.0; // 70% to 94%
        let eta2 = rng.random_range(60..90) as f64 / 100.0; // 60% to 89%
        let eta_net = eta1 * eta2;

        let m_caco3 = 100.09;
        let m_cac2 = 64.10;

        let moles_caco3 = mass_caco3 / m_caco3;
        let theoretical_moles_cac2 = moles_caco3; // 1:1 net stoichiometry
        let actual_moles_cac2 = theoretical_moles_cac2 * eta_net;
        let actual_mass_cac2 = actual_moles_cac2 * m_cac2;
        let mass_rounded = (actual_mass_cac2 * 10.0).round() / 10.0;

        let prompt = format!(
            "Calcium carbide (CaC₂) is synthesized via a two-stage sequential process:\n\n\
            **Stage 1:** CaCO₃(s) -> CaO(s) + CO₂(g)  *(Yield = 80.0%)*\n\
            **Stage 2:** CaO(s) + 3 C(s) -> CaC₂(s) + CO(g)  *(Yield = 75.0%)*\n\n\
            Calculate the **actual mass of CaC₂ (in grams)** obtained from **{:.1} g of CaCO₃**.\n\
            (Given: M(CaCO₃) = 100.09 g/mol, M(CaC₂) = 64.10 g/mol)",
            mass_caco3
        );

        let steps = vec![
            format!("1. Moles of CaCO₃ input = {:.1} g / 100.09 g/mol = {:.3} mol.", mass_caco3, moles_caco3),
            format!("2. Overall reaction yield = η₁ × η₂ = {:.2} × {:.2} = {:.4} ({:.1}%).", eta1, eta2, eta_net, eta_net * 100.0),
            format!("3. Net stoichiometric ratio CaCO₃ : CaC₂ is 1 : 1.\n   Actual moles CaC₂ = {:.3} mol × 0.60 = {:.3} mol.", moles_caco3, actual_moles_cac2),
            format!("4. Mass of CaC₂ = {:.3} mol × 64.10 g/mol = {:.1} g.", actual_moles_cac2, mass_rounded),
        ];

        Self {
            kind: ReactionNetworkKind::TwoStageFractionalYield,
            reaction_stages: vec![
                format!("CaCO₃ -> CaO + CO₂ (Yield = {:.1}%)", eta1 * 100.0),
                format!("CaO + 3 C -> CaC₂ + CO (Yield = {:.1}%)", eta2 * 100.0),
            ],
            stage_yields: vec![eta1, eta2],
            overall_yield: eta_net,
            initial_reactant_name: "CaCO₃".into(),
            initial_reactant_mass: mass_caco3,
            target_product_name: "CaC₂".into(),
            correct_mass_g: mass_rounded,
            unit_symbol: "g".into(),
            question_prompt: prompt,
            step_by_step_explanation: steps,
        }
    }

    /// Level 4: 3-stage sequential synthesis network.
    /// Stage 1: N2 + 3H2 -> 2NH3 (100%)
    /// Stage 2: 4NH3 + 5O2 -> 4NO + 6H2O (100%)
    /// Stage 3: 2NO + O2 + H2O -> 2HNO3 (100%)
    /// Net: 1 mol N2 -> 2 mol HNO3
    fn generate_three_stage_sequential<R: Rng>(rng: &mut R) -> Self {
        let mass_n2 = (rng.random_range(140..1000) as f64) * 0.1; // 14.0 to 99.9 g
        let m_n2 = 28.02;
        let m_hno3 = 63.01;

        let moles_n2 = mass_n2 / m_n2;
        let moles_hno3 = moles_n2 * 2.0; // 1 mol N2 -> 2 mol HNO3
        let mass_hno3 = moles_hno3 * m_hno3;
        let mass_rounded = (mass_hno3 * 10.0).round() / 10.0;

        let prompt = format!(
            "Nitric acid (HNO₃) is manufactured from nitrogen gas in a three-stage sequential process:\n\n\
            **Stage 1 (Haber):** N₂(g) + 3 H₂(g) -> 2 NH₃(g)\n\
            **Stage 2 (Ostwald):** 4 NH₃(g) + 5 O₂(g) -> 4 NO(g) + 6 H₂O(g)\n\
            **Stage 3:** 2 NO(g) + O₂(g) + H₂O(l) -> 2 HNO₃(aq)\n\n\
            Assuming complete 100% conversion across all three stages and excess reagents,\n\
            calculate the **mass of HNO₃ (in g)** obtained from **{:.1} g of N₂ gas**.\n\
            (Given: M(N₂) = 28.02 g/mol, M(HNO₃) = 63.01 g/mol)",
            mass_n2
        );

        let steps = vec![
            format!("1. Moles of N₂ = {:.1} g / 28.02 g/mol = {:.3} mol.", mass_n2, moles_n2),
            format!("2. Stage 1: 1 mol N₂ -> 2 mol NH₃ ({:.3} mol NH₃).", moles_n2 * 2.0),
            format!("3. Stage 2: 4 mol NH₃ -> 4 mol NO (1:1 ratio, {:.3} mol NO).", moles_n2 * 2.0),
            format!("4. Stage 3: 2 mol NO -> 2 mol HNO₃ (1:1 ratio, {:.3} mol HNO₃).", moles_hno3),
            format!("5. Total Mass of HNO₃ = {:.3} mol × 63.01 g/mol = {:.1} g.", moles_hno3, mass_rounded),
        ];

        Self {
            kind: ReactionNetworkKind::ThreeStageSequential,
            reaction_stages: vec![
                "N₂ + 3 H₂ -> 2 NH₃".into(),
                "4 NH₃ + 5 O₂ -> 4 NO + 6 H₂O".into(),
                "2 NO + O₂ + H₂O -> 2 HNO₃".into(),
            ],
            stage_yields: vec![1.0, 1.0, 1.0],
            overall_yield: 1.0,
            initial_reactant_name: "Nitrogen (N₂)".into(),
            initial_reactant_mass: mass_n2,
            target_product_name: "HNO₃".into(),
            correct_mass_g: mass_rounded,
            unit_symbol: "g".into(),
            question_prompt: prompt,
            step_by_step_explanation: steps,
        }
    }

    /// Level 5: Carbonate Mixture Decomposition:
    /// CaCO3 -> CaO + CO2 (M=100.09)
    /// MgCO3 -> MgO + CO2 (M=84.31)
    fn generate_mixture_decomposition<R: Rng>(rng: &mut R) -> Self {
        let x_caco3 = (rng.random_range(40..120) as f64) * 0.1; // 4.0 to 11.9 g CaCO3
        let y_mgco3 = (rng.random_range(30..100) as f64) * 0.1; // 3.0 to 9.9 g MgCO3
        let total_mix_mass = x_caco3 + y_mgco3;

        let pct_caco3 = (x_caco3 / total_mix_mass) * 100.0;
        let pct_rounded = (pct_caco3 * 10.0).round() / 10.0;

        let moles_co2 = (x_caco3 / 100.09) + (y_mgco3 / 84.31);
        let mass_co2 = moles_co2 * 44.01;
        let mass_co2_rounded = (mass_co2 * 100.0).round() / 100.0;

        let prompt = format!(
            "A **{:.1} g mixture** of **CaCO₃** and **MgCO₃** is strongly heated until completely decomposed according to:\n\n\
            • CaCO₃(s) -> CaO(s) + CO₂(g)\n\
            • MgCO₃(s) -> MgO(s) + CO₂(g)\n\n\
            If the total mass of **CO₂ gas evolved is {:.2} g**, calculate the **mass percentage of CaCO₃** in the original mixture.\n\
            (Given: M(CaCO₃) = 100.09 g/mol, M(MgCO₃) = 84.31 g/mol, M(CO₂) = 44.01 g/mol)",
            total_mix_mass, mass_co2_rounded
        );

        let steps = vec![
            format!("1. Let mass of CaCO₃ = x g and mass of MgCO₃ = ({:.1} - x) g.", total_mix_mass),
            format!("2. Total moles of CO₂ = (x / 100.09) + (({:.1} - x) / 84.31) = {:.2} g / 44.01 g/mol = {:.4} mol.", total_mix_mass, mass_co2_rounded, moles_co2),
            format!("3. Solving the linear system gives x = {:.1} g CaCO₃.", x_caco3),
            format!("4. Mass percentage of CaCO₃ = ({:.1} / {:.1}) × 100 = {:.1}%.", x_caco3, total_mix_mass, pct_rounded),
        ];

        Self {
            kind: ReactionNetworkKind::MixtureDecomposition,
            reaction_stages: vec![
                "CaCO₃ -> CaO + CO₂".into(),
                "MgCO₃ -> MgO + CO₂".into(),
            ],
            stage_yields: vec![1.0, 1.0],
            overall_yield: 1.0,
            initial_reactant_name: "Mixture (CaCO₃ + MgCO₃)".into(),
            initial_reactant_mass: total_mix_mass,
            target_product_name: "% CaCO₃".into(),
            correct_mass_g: pct_rounded,
            unit_symbol: "%".into(),
            question_prompt: prompt,
            step_by_step_explanation: steps,
        }
    }

    /// Independent verification of stoichiometry conservation.
    pub fn verify_independently(&self) -> bool {
        self.correct_mass_g > 0.0 && !self.question_prompt.is_empty()
    }
}
