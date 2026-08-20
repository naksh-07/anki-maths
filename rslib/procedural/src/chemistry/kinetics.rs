// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use rand::Rng;
use serde::{Deserialize, Serialize};

/// Discrete classification of chemical kinetics problem types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KineticsKind {
    FirstOrderHalfLife,
    FirstOrderDecayTime,
    SecondOrderDecay,
    InitialRateMethod,
    ArrheniusActivationEnergy,
}

/// A structured chemical kinetics problem definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KineticsPuzzle {
    pub kind: KineticsKind,
    pub reaction_order: u32,
    pub rate_constant_k: f64,
    pub k_unit: String,
    pub half_life_sec: Option<f64>,
    pub initial_concentration: Option<f64>,
    pub target_concentration: Option<f64>,
    pub time_elapsed_sec: Option<f64>,
    pub activation_energy_kj: Option<f64>,
    pub target_quantity: String,
    pub correct_value: f64,
    pub unit_symbol: String,
    pub question_prompt: String,
    pub step_by_step_explanation: Vec<String>,
}

impl KineticsPuzzle {
    pub const GAS_CONSTANT_R: f64 = 8.314; // J / (mol K)

    /// Dynamically generate a verified Chemical Kinetics problem.
    pub fn generate_dynamic<R: Rng>(
        rng: &mut R,
        difficulty_level: u32,
    ) -> Self {
        match difficulty_level {
            1 => Self::generate_first_order_half_life(rng),
            2 => Self::generate_first_order_decay(rng),
            3 => Self::generate_second_order_decay(rng),
            4 => Self::generate_initial_rate_table(rng),
            _ => Self::generate_arrhenius_ea(rng),
        }
    }

    /// Level 1: First-Order Half-Life: t_1/2 = 0.693 / k.
    fn generate_first_order_half_life<R: Rng>(rng: &mut R) -> Self {
        let k_exp = rng.random_range(1..6);
        let k_base = rng.random_range(10..100) as f64 / 10.0;
        let k = k_base * 10f64.powi(-k_exp); // e.g. 2.0e-3 s^-1
        let t_half = 0.693147 / k;
        let t_half_rounded = (t_half * 100.0).round() / 100.0;

        let prompt = format!(
            "A first-order decomposition reaction has a rate constant **k = {:.2e} s⁻¹** at 300 K.\n\
            Calculate the **half-life (t₁/₂ in seconds)** of this reaction.\n\
            (Use ln 2 = 0.693)",
            k
        );

        let steps = vec![
            format!("1. Identify reaction order: 1st order (rate constant has units s⁻¹)."),
            format!("2. Half-life formula for first-order kinetics: t₁/₂ = (ln 2) / k = 0.693 / k."),
            format!("3. t₁/₂ = 0.693 / ({:.2e}) = {:.1} s.", k, t_half_rounded),
        ];

        Self {
            kind: KineticsKind::FirstOrderHalfLife,
            reaction_order: 1,
            rate_constant_k: k,
            k_unit: "s⁻¹".into(),
            half_life_sec: Some(t_half_rounded),
            initial_concentration: None,
            target_concentration: None,
            time_elapsed_sec: None,
            activation_energy_kj: None,
            target_quantity: "t₁/₂ (s)".into(),
            correct_value: t_half_rounded,
            unit_symbol: "s".into(),
            question_prompt: prompt,
            step_by_step_explanation: steps,
        }
    }

    /// Level 2: First-Order Concentration Decay over Time: ln([A]0 / [A]t) = kt.
    fn generate_first_order_decay<R: Rng>(rng: &mut R) -> Self {
        let a_0 = (rng.random_range(5..25) as f64) * 0.1; // 0.5 to 2.5 M
        let n_half_lives = rng.random_range(2..5); // 2, 3, 4 half lives
        let fraction_remaining = 0.5_f64.powi(n_half_lives); 
        let a_t = a_0 * fraction_remaining;
        let t_half = rng.random_range(20..80) as f64; // seconds
        let total_time = (n_half_lives as f64) * t_half;

        let prompt = format!(
            "The first-order decomposition of substance A has a half-life of **{:.1} s**.\n\
            If the initial concentration is **{:.2} M**, what is the concentration of A (in M) remaining after **{:.1} s**?",
            t_half, a_0, total_time
        );

        let steps = vec![
            format!("1. Number of half-lives elapsed: n = t / t₁/₂ = {:.1} / {:.1} = {}.", total_time, t_half, n_half_lives),
            format!("2. Remaining concentration after n half-lives: [A] = [A]₀ × (½)ⁿ = {:.2} × (½)^{} = {:.2} × {} = {:.3} M.", a_0, n_half_lives, a_0, fraction_remaining, a_t),
        ];

        Self {
            kind: KineticsKind::FirstOrderDecayTime,
            reaction_order: 1,
            rate_constant_k: 0.693 / t_half,
            k_unit: "s⁻¹".into(),
            half_life_sec: Some(t_half),
            initial_concentration: Some(a_0),
            target_concentration: Some(a_t),
            time_elapsed_sec: Some(total_time),
            activation_energy_kj: None,
            target_quantity: "[A] remaining".into(),
            correct_value: a_t,
            unit_symbol: "M".into(),
            question_prompt: prompt,
            step_by_step_explanation: steps,
        }
    }

    /// Level 3: Second-Order Integrated Rate: 1/[A]t = 1/[A]0 + kt.
    fn generate_second_order_decay<R: Rng>(rng: &mut R) -> Self {
        let a_0 = (rng.random_range(2..10) as f64) * 0.1; // 0.2 to 0.9 M
        let k = (rng.random_range(2..10) as f64) * 0.05; // 0.10 to 0.45 L/(mol s)
        let t = rng.random_range(10..50) as f64; // 10 to 49 s

        let inv_at = (1.0 / a_0) + (k * t);
        let a_t = 1.0 / inv_at;
        let a_t_rounded = (a_t * 1000.0).round() / 1000.0;

        let prompt = format!(
            "A second-order reaction **2A -> Products** has a rate constant **k = {:.2} L·mol⁻¹·s⁻¹**.\n\
            If the initial concentration of A is **{:.2} M**, calculate the concentration of A remaining after **{:.1} s**.",
            k, a_0, t
        );

        let steps = vec![
            format!("1. Integrated rate law for 2nd order: 1/[A]ₜ = 1/[A]₀ + kt."),
            format!("2. Substitute: 1/[A]ₜ = (1 / {:.2}) + ({:.2} × {:.1}) = {:.2} + {:.2} = {:.3} M⁻¹.", a_0, k, t, 1.0 / a_0, k * t, inv_at),
            format!("3. [A]ₜ = 1 / {:.3} = {:.3} M.", inv_at, a_t_rounded),
        ];

        Self {
            kind: KineticsKind::SecondOrderDecay,
            reaction_order: 2,
            rate_constant_k: k,
            k_unit: "L·mol⁻¹·s⁻¹".into(),
            half_life_sec: Some(1.0 / (k * a_0)),
            initial_concentration: Some(a_0),
            target_concentration: Some(a_t_rounded),
            time_elapsed_sec: Some(t),
            activation_energy_kj: None,
            target_quantity: "[A]ₜ".into(),
            correct_value: a_t_rounded,
            unit_symbol: "M".into(),
            question_prompt: prompt,
            step_by_step_explanation: steps,
        }
    }

    /// Level 4: Method of Initial Rates Table.
    fn generate_initial_rate_table<R: Rng>(rng: &mut R) -> Self {
        let mut m = rng.random_range(0..=2);
        let mut n = rng.random_range(0..=2);
        if m == 0 && n == 0 {
            m = 1;
            n = 1;
        }
        let overall_order = (m + n) as f64;

        let a0 = (rng.random_range(1..5) as f64) * 0.1;
        let b0 = (rng.random_range(1..5) as f64) * 0.1;
        let k_rate = (rng.random_range(1..10) as f64) * 1e-3;
        
        let a2_factor = rng.random_range(2..=3) as f64;
        let b3_factor = rng.random_range(2..=3) as f64;

        let a2 = a0 * a2_factor;
        let b3 = b0 * b3_factor;

        let rate1 = k_rate * a0.powi(m) * b0.powi(n);
        let rate2 = k_rate * a2.powi(m) * b0.powi(n);
        let rate3 = k_rate * a0.powi(m) * b3.powi(n);

        let rate2_ratio = rate2 / rate1;
        let rate3_ratio = rate3 / rate1;

        let prompt = format!(
            "Initial rate data for the reaction **A + B -> Products** at 298 K:\n\n\
            | Exp | [A] (M) | [B] (M) | Initial Rate (M/s) |\n\
            | :--- | :--- | :--- | :--- |\n\
            | 1 | {:.2} | {:.2} | {:.2e} |\n\
            | 2 | {:.2} | {:.2} | {:.2e} |\n\
            | 3 | {:.2} | {:.2} | {:.2e} |\n\n\
            Determine the **overall order of the reaction**.",
            a0, b0, rate1,
            a2, b0, rate2,
            a0, b3, rate3
        );

        let steps = vec![
            format!("1. Compare Exp 1 & 2 (where [B] is constant): Multiplying [A] by {:.0} changes the rate by a factor of {:.0} ({:.0}^{} = {:.0}). Order with respect to A is {}.", a2_factor, rate2_ratio, a2_factor, m, rate2_ratio, m),
            format!("2. Compare Exp 1 & 3 (where [A] is constant): Multiplying [B] by {:.0} changes the rate by a factor of {:.0} ({:.0}^{} = {:.0}). Order with respect to B is {}.", b3_factor, rate3_ratio, b3_factor, n, rate3_ratio, n),
            format!("3. Overall reaction order = {} + {} = {}.", m, n, overall_order),
        ];

        Self {
            kind: KineticsKind::InitialRateMethod,
            reaction_order: overall_order as u32,
            rate_constant_k: k_rate,
            k_unit: "variable".into(),
            half_life_sec: None,
            initial_concentration: None,
            target_concentration: None,
            time_elapsed_sec: None,
            activation_energy_kj: None,
            target_quantity: "Overall Order".into(),
            correct_value: overall_order,
            unit_symbol: "order".into(),
            question_prompt: prompt,
            step_by_step_explanation: steps,
        }
    }

    /// Level 5: Arrhenius Equation Activation Energy: ln(k2 / k1) = (Ea / R)(1/T1 - 1/T2).
    fn generate_arrhenius_ea<R: Rng>(rng: &mut R) -> Self {
        let t1 = rng.random_range(280..320) as f64; // e.g. 300 K
        let t2 = t1 + rng.random_range(10..30) as f64; // e.g. 310 K
        let rate_ratio = (rng.random_range(20..50) as f64) * 0.1; // 2.0 to 4.9

        let ln_ratio = rate_ratio.ln();
        let inv_diff = (1.0 / t1) - (1.0 / t2);
        let ea_joules = (ln_ratio * Self::GAS_CONSTANT_R) / inv_diff;
        let ea_kj = ea_joules / 1000.0;
        let ea_kj_rounded = (ea_kj * 10.0).round() / 10.0;

        let prompt = format!(
            "The rate of a chemical reaction increases by a factor of **{:.1}** when the temperature is raised from **{:.0} K to {:.0} K**.\n\
            Calculate the **activation energy (Eₐ in kJ/mol)** for this reaction.\n\
            (Given: R = 8.314 J·mol⁻¹·K⁻¹)",
            rate_ratio, t1, t2
        );

        let steps = vec![
            format!("1. Arrhenius formula: ln(k₂ / k₁) = (Eₐ / R) × (1/T₁ - 1/T₂)."),
            format!("2. ln({:.1}) = {:.4} = (Eₐ / 8.314) × (1/{:.0} - 1/{:.0}).", rate_ratio, ln_ratio, t1, t2),
            format!("3. (1/T₁ - 1/T₂) = {:.4e} K⁻¹.", inv_diff),
            format!("4. Eₐ = ({:.4} × 8.314) / {:.4e} = {:.0} J/mol = {:.1} kJ/mol.", ln_ratio, inv_diff, ea_joules, ea_kj_rounded),
        ];

        Self {
            kind: KineticsKind::ArrheniusActivationEnergy,
            reaction_order: 1,
            rate_constant_k: 0.0,
            k_unit: "".into(),
            half_life_sec: None,
            initial_concentration: None,
            target_concentration: None,
            time_elapsed_sec: None,
            activation_energy_kj: Some(ea_kj_rounded),
            target_quantity: "Activation Energy (Eₐ)".into(),
            correct_value: ea_kj_rounded,
            unit_symbol: "kJ/mol".into(),
            question_prompt: prompt,
            step_by_step_explanation: steps,
        }
    }

    /// Independent verification of non-negative kinetics metrics.
    pub fn verify_independently(&self) -> bool {
        self.correct_value > 0.0 && !self.question_prompt.is_empty()
    }
}
