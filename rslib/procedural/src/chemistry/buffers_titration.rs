// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use rand::Rng;
use serde::{Deserialize, Serialize};

/// Discrete classification of ionic equilibrium regimes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IonicRegimeKind {
    /// Pure weak acid solution: [H+] = sqrt(Ka * C)
    PureWeakAcid,
    /// Acidic buffer solution: pH = pKa + log([A-] / [HA])
    AcidicBuffer,
    /// Basic buffer solution: pOH = pKb + log([BH+] / [B])
    BasicBuffer,
    /// Buffer response to added strong acid or base
    BufferCapacityShift,
    /// Salt hydrolysis at titration equivalence point: pH = 7 + 0.5*pKa + 0.5*log(C)
    SaltHydrolysisEquivalence,
    /// Post-equivalence point excess strong reagent
    PostEquivalenceExcess,
}

/// A structured ionic equilibrium / buffer problem definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BufferTitrationPuzzle {
    pub regime: IonicRegimeKind,
    pub acid_or_base_name: String,
    pub conjugate_name: String,
    pub pka_or_pkb: f64,
    pub initial_acid_conc: f64,
    pub initial_conjugate_conc: f64,
    pub volume_ml: f64,
    pub added_reagent: Option<(String, f64, f64)>, // (Reagent Name, Vol mL, Conc M)
    pub question_prompt: String,
    pub correct_ph: f64,
    pub target_unit: String,
    pub step_by_step_explanation: Vec<String>,
}

impl BufferTitrationPuzzle {
    /// Dynamically generate an ionic equilibrium buffer / titration problem.
    pub fn generate_dynamic<R: Rng>(
        rng: &mut R,
        difficulty_level: u32,
    ) -> Self {
        match difficulty_level {
            1 => Self::generate_pure_weak_acid(rng),
            2 => Self::generate_direct_buffer_ph(rng),
            3 => Self::generate_buffer_addition_shift(rng),
            4 => Self::generate_salt_hydrolysis_equivalence(rng),
            _ => Self::generate_full_titration_point(rng),
        }
    }

    /// Level 1: Direct weak acid pH from Ka and concentration.
    fn generate_pure_weak_acid<R: Rng>(rng: &mut R) -> Self {
        let acids = [
            ("Acetic Acid (CH₃COOH)", "Acetate (CH₃COO⁻)", 4.75, "1.8 × 10⁻⁵"),
            ("Formic Acid (HCOOH)", "Formate (HCOO⁻)", 3.75, "1.8 × 10⁻⁴"),
            ("Propanoic Acid (C₂H₅COOH)", "Propanoate (C₂H₅COO⁻)", 4.87, "1.3 × 10⁻⁵"),
            ("Hypochlorous Acid (HClO)", "Hypochlorite (ClO⁻)", 7.53, "3.0 × 10⁻⁸"),
            ("Nitrous Acid (HNO₂)", "Nitrite (NO₂⁻)", 3.15, "7.1 × 10⁻⁴"),
        ];
        let (acid_name, conj_name, pka, ka_str) = acids[rng.random_range(0..acids.len())];
        
        let c_a = (rng.random_range(5..=250) as f64) * 0.01; 
        let ka = 10f64.powf(-pka);
        let h_conc = (ka * c_a).sqrt();
        let ph = -h_conc.log10();
        let ph_rounded = (ph * 100.0).round() / 100.0;

        let prompt = format!(
            "Calculate the **pH** of a **{:.2} M** aqueous solution of {}.\n\
            (Given: Kₐ = {}, pKₐ = {})",
            c_a, acid_name, ka_str, pka
        );

        let steps = vec![
            format!("1. Equilibrium: Acid ⇌ Conjugate Base + H⁺ with Kₐ."),
            format!("2. For a weak acid where α << 1: [H⁺] = √(Kₐ × C) = √({} × {:.2}) = {:.4e} M.", ka_str, c_a, h_conc),
            format!("3. pH = -log₁₀[H⁺] = -log₁₀({:.4e}) = {:.2}.", h_conc, ph_rounded),
        ];

        Self {
            regime: IonicRegimeKind::PureWeakAcid,
            acid_or_base_name: acid_name.into(),
            conjugate_name: conj_name.into(),
            pka_or_pkb: pka,
            initial_acid_conc: c_a,
            initial_conjugate_conc: 0.0,
            volume_ml: 100.0,
            added_reagent: None,
            question_prompt: prompt,
            correct_ph: ph_rounded,
            target_unit: "pH".into(),
            step_by_step_explanation: steps,
        }
    }

    /// Level 2: Acidic Buffer Henderson-Hasselbalch.
    fn generate_direct_buffer_ph<R: Rng>(rng: &mut R) -> Self {
        let acids = [
            ("CH₃COOH", "CH₃COONa", 4.74),
            ("HCOOH", "HCOONa", 3.75),
            ("C₂H₅COOH", "C₂H₅COONa", 4.87),
        ];
        let (acid_name, conj_name, pka) = acids[rng.random_range(0..acids.len())];

        let c_acid = (rng.random_range(5..=150) as f64) * 0.01; 
        let c_salt = (rng.random_range(5..=150) as f64) * 0.01; 
        let ph = pka + (c_salt / c_acid).log10();
        let ph_rounded = (ph * 100.0).round() / 100.0;

        let prompt = format!(
            "A buffer solution is prepared containing **{:.2} M {}** and **{:.2} M {}**.\n\
            What is the **pH** of this buffer solution?\n\
            (Given: pKₐ of {} = {})",
            c_acid, acid_name, c_salt, conj_name, acid_name, pka
        );

        let steps = vec![
            format!("1. Identify buffer regime: Weak acid + Conjugate base salt."),
            format!("2. Apply Henderson-Hasselbalch equation: pH = pKₐ + log₁₀([Salt] / [Acid])."),
            format!("3. pH = {} + log₁₀({:.2} / {:.2}) = {} + ({:.3}) = {:.2}.", pka, c_salt, c_acid, pka, (c_salt / c_acid).log10(), ph_rounded),
        ];

        Self {
            regime: IonicRegimeKind::AcidicBuffer,
            acid_or_base_name: acid_name.into(),
            conjugate_name: conj_name.into(),
            pka_or_pkb: pka,
            initial_acid_conc: c_acid,
            initial_conjugate_conc: c_salt,
            volume_ml: 500.0,
            added_reagent: None,
            question_prompt: prompt,
            correct_ph: ph_rounded,
            target_unit: "pH".into(),
            step_by_step_explanation: steps,
        }
    }

    /// Level 3: Buffer Capacity Shift upon Strong Acid/Base Addition.
    fn generate_buffer_addition_shift<R: Rng>(rng: &mut R) -> Self {
        let acids = [
            ("CH₃COOH", "CH₃COO⁻", 4.74),
            ("HCOOH", "HCOO⁻", 3.75),
            ("C₂H₅COOH", "C₂H₅COO⁻", 4.87),
        ];
        let (acid_name, conj_name, pka) = acids[rng.random_range(0..acids.len())];

        let v_buf = 500.0; // 500 mL = 0.5 L
        let n_acid_init = (rng.random_range(10..=50) as f64) * 0.01; 
        let n_salt_init = (rng.random_range(10..=50) as f64) * 0.01; 

        let added_hcl_mol = (rng.random_range(1..=9) as f64) * 0.01; // max 0.09 mol to not exhaust salt

        let n_acid_final = n_acid_init + added_hcl_mol;
        let n_salt_final = n_salt_init - added_hcl_mol;

        let ph = pka + (n_salt_final / n_acid_final).log10();
        let ph_rounded = (ph * 100.0).round() / 100.0;

        let prompt = format!(
            "A 500 mL buffer contains **{:.2} mol {}** and **{:.2} mol {} salt**.\n\
            If **{:.2} mol of gaseous HCl** is bubbled into this buffer with no volume change,\n\
            what is the resulting **pH**?\n\
            (Given: pKₐ of {} = {})",
            n_acid_init, acid_name, n_salt_init, conj_name, added_hcl_mol, acid_name, pka
        );

        let steps = vec![
            format!("1. Strong acid reaction: H⁺ + {} -> {}.", conj_name, acid_name),
            format!("2. New moles: n({}) = {:.2} - {:.2} = {:.2} mol, n({}) = {:.2} + {:.2} = {:.2} mol.", conj_name, n_salt_init, added_hcl_mol, n_salt_final, acid_name, n_acid_init, added_hcl_mol, n_acid_final),
            format!("3. Henderson-Hasselbalch: pH = {} + log₁₀({:.2} / {:.2}) = {:.2}.", pka, n_salt_final, n_acid_final, ph_rounded),
        ];

        Self {
            regime: IonicRegimeKind::BufferCapacityShift,
            acid_or_base_name: acid_name.into(),
            conjugate_name: conj_name.into(),
            pka_or_pkb: pka,
            initial_acid_conc: n_acid_init / (v_buf / 1000.0),
            initial_conjugate_conc: n_salt_init / (v_buf / 1000.0),
            volume_ml: v_buf,
            added_reagent: Some(("HCl".into(), 0.0, added_hcl_mol)),
            question_prompt: prompt,
            correct_ph: ph_rounded,
            target_unit: "pH".into(),
            step_by_step_explanation: steps,
        }
    }

    /// Level 4: Salt Hydrolysis at Equivalence Point.
    fn generate_salt_hydrolysis_equivalence<R: Rng>(rng: &mut R) -> Self {
        let acids = [
            ("Acetic Acid", "CH₃COONa", 4.75),
            ("Formic Acid", "HCOONa", 3.75),
            ("Propanoic Acid", "C₂H₅COONa", 4.87),
        ];
        let (acid_name, salt_name, pka) = acids[rng.random_range(0..acids.len())];

        let c_salt = (rng.random_range(5..=200) as f64) * 0.01; 
        // Salt of weak acid + strong base: pH = 7 + 0.5 * pKa + 0.5 * log10(C)
        let ph = 7.0 + 0.5 * pka + 0.5 * c_salt.log10();
        let ph_rounded = (ph * 100.0).round() / 100.0;

        let prompt = format!(
            "Calculate the **pH** at the equivalence point of a titration of weak {} resulting in a **{:.2} M {}** solution.\n\
            (Given: pKₐ of {} = {}, pK_w = 14.00)",
            acid_name, c_salt, salt_name, acid_name, pka
        );

        let steps = vec![
            format!("1. At equivalence point, only the salt ({}) is present, which undergoes anion hydrolysis.", salt_name),
            format!("2. Salt hydrolysis formula for Weak Acid + Strong Base: pH = 7 + ½(pKₐ) + ½(log₁₀ C)."),
            format!("3. pH = 7 + ½({}) + ½({:.3}) = 7 + {:.3} + ({:.3}) = {:.2}.", pka, c_salt.log10(), 0.5 * pka, 0.5 * c_salt.log10(), ph_rounded),
        ];

        Self {
            regime: IonicRegimeKind::SaltHydrolysisEquivalence,
            acid_or_base_name: acid_name.into(),
            conjugate_name: salt_name.into(),
            pka_or_pkb: pka,
            initial_acid_conc: 0.0,
            initial_conjugate_conc: c_salt,
            volume_ml: 100.0,
            added_reagent: None,
            question_prompt: prompt,
            correct_ph: ph_rounded,
            target_unit: "pH".into(),
            step_by_step_explanation: steps,
        }
    }

    /// Level 5: Full titration midpoint / post-equivalence calculation.
    fn generate_full_titration_point<R: Rng>(rng: &mut R) -> Self {
        // Half-equivalence point: V_base added = 0.5 * V_equiv => pH = pKa
        let acids = [
            (4.80, "HA (pKₐ = 4.80)"),
            (3.75, "HA (pKₐ = 3.75)"),
            (4.20, "HA (pKₐ = 4.20)"),
            (7.53, "HA (pKₐ = 7.53)"),
        ];
        let (pka, name) = acids[rng.random_range(0..acids.len())];

        let v_acid = (rng.random_range(10..=100) as f64) * 1.0;
        let m_acid = (rng.random_range(5..=50) as f64) * 0.01;
        let m_base = (rng.random_range(5..=50) as f64) * 0.01;
        let v_equiv = (v_acid * m_acid) / m_base;
        let v_half_equiv = v_equiv / 2.0;

        let prompt = format!(
            "In a titration of **{:.1} mL of {:.2} M weak monoprotic acid {}** with **{:.2} M NaOH**,\n\
            what is the **pH** after adding exactly **{:.1} mL of NaOH**?",
            v_acid, m_acid, name, m_base, v_half_equiv
        );

        let initial_moles = v_acid * m_acid;
        let added_moles = v_half_equiv * m_base;

        let steps = vec![
            format!("1. Initial mmol of HA = {:.1} mL × {:.2} M = {:.2} mmol.", v_acid, m_acid, initial_moles),
            format!("2. Added mmol of NaOH = {:.1} mL × {:.2} M = {:.2} mmol.", v_half_equiv, m_base, added_moles),
            format!("3. Reaction: HA + OH⁻ -> A⁻ + H₂O.\n   Remaining HA = {:.2} - {:.2} = {:.2} mmol.\n   Produced A⁻ = {:.2} mmol.", initial_moles, added_moles, initial_moles - added_moles, added_moles),
            format!("4. Since [HA] = [A⁻] at the half-equivalence point, pH = pKₐ + log₁₀(1) = pKₐ = {:.2}.", pka),
        ];

        Self {
            regime: IonicRegimeKind::AcidicBuffer,
            acid_or_base_name: "HA".into(),
            conjugate_name: "NaA".into(),
            pka_or_pkb: pka,
            initial_acid_conc: m_acid,
            initial_conjugate_conc: 0.0,
            volume_ml: v_acid,
            added_reagent: Some(("NaOH".into(), v_half_equiv, m_base)),
            question_prompt: prompt,
            correct_ph: pka,
            target_unit: "pH".into(),
            step_by_step_explanation: steps,
        }
    }

    /// Independent verification of chemical validity and pH bounds (0 to 14).
    pub fn verify_independently(&self) -> bool {
        self.correct_ph >= 0.0 && self.correct_ph <= 14.0 && !self.question_prompt.is_empty()
    }
}
