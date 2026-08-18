// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

/// Deterministic chemical invariant boundary and consistency validator.
pub struct ChemicalInvariantValidator;

impl ChemicalInvariantValidator {
    pub const TOLERANCE: f64 = 1e-6;

    /// Validate that mass is strictly non-negative.
    pub fn validate_mass_non_negative(mass_g: f64) -> Result<(), String> {
        if mass_g < -Self::TOLERANCE {
            Err(format!("Chemical mass cannot be negative: {:.4} g", mass_g))
        } else {
            Ok(())
        }
    }

    /// Validate that amount of substance in moles is strictly non-negative.
    pub fn validate_moles_non_negative(moles: f64) -> Result<(), String> {
        if moles < -Self::TOLERANCE {
            Err(format!("Amount of substance (moles) cannot be negative: {:.6} mol", moles))
        } else {
            Ok(())
        }
    }

    /// Validate that solution volume is strictly positive.
    pub fn validate_volume_positive(volume_l: f64) -> Result<(), String> {
        if volume_l <= Self::TOLERANCE {
            Err(format!("Solution volume must be strictly positive: {:.4} L", volume_l))
        } else {
            Ok(())
        }
    }

    /// Validate that chemical concentration (Molarity) is strictly non-negative.
    pub fn validate_concentration_non_negative(molarity: f64) -> Result<(), String> {
        if molarity < -Self::TOLERANCE {
            Err(format!("Concentration cannot be negative: {:.4} M", molarity))
        } else {
            Ok(())
        }
    }

    /// Validate that chemical equilibrium constant is strictly positive and finite.
    pub fn validate_equilibrium_constant_positive(kc: f64) -> Result<(), String> {
        if kc <= Self::TOLERANCE || !kc.is_finite() {
            Err(format!("Equilibrium constant Kc must be strictly positive and finite: {:.4e}", kc))
        } else {
            Ok(())
        }
    }

    /// Validate that percentage yield is bounded between 0% and 100%.
    pub fn validate_percentage_yield(yield_pct: f64) -> Result<(), String> {
        if yield_pct < -Self::TOLERANCE || yield_pct > 100.0 + Self::TOLERANCE {
            Err(format!("Percentage yield must be between 0% and 100%: {:.2}%", yield_pct))
        } else {
            Ok(())
        }
    }

    /// Validate limiting reagent identification consistency:
    /// Compares (n_A / a) and (n_B / b). The smaller ratio is the true limiting reagent.
    pub fn validate_limiting_reagent(
        moles_a: f64,
        coeff_a: u32,
        name_a: &str,
        moles_b: f64,
        coeff_b: u32,
        name_b: &str,
        claimed_limiting: &str,
    ) -> Result<(), String> {
        let ratio_a = moles_a / coeff_a as f64;
        let ratio_b = moles_b / coeff_b as f64;

        let expected_limiting = if ratio_a < ratio_b { name_a } else { name_b };
        if claimed_limiting.to_uppercase() != expected_limiting.to_uppercase() {
            Err(format!(
                "Limiting reagent error: Claimed '{}', but {} has lower stoichiometric ratio ({:.4} vs {:.4})",
                claimed_limiting, expected_limiting, ratio_a, ratio_b
            ))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chemical_invariants_validations() {
        assert!(ChemicalInvariantValidator::validate_mass_non_negative(15.2).is_ok());
        assert!(ChemicalInvariantValidator::validate_mass_non_negative(-0.5).is_err());

        assert!(ChemicalInvariantValidator::validate_volume_positive(0.5).is_ok());
        assert!(ChemicalInvariantValidator::validate_volume_positive(0.0).is_err());

        assert!(ChemicalInvariantValidator::validate_equilibrium_constant_positive(4.5e-3).is_ok());
        assert!(ChemicalInvariantValidator::validate_equilibrium_constant_positive(0.0).is_err());

        assert!(ChemicalInvariantValidator::validate_percentage_yield(88.5).is_ok());
        assert!(ChemicalInvariantValidator::validate_percentage_yield(105.0).is_err());

        // 2 mol N2 (coeff 1) vs 3 mol H2 (coeff 3):
        // N2 ratio = 2/1 = 2.0; H2 ratio = 3/3 = 1.0 -> H2 is limiting
        assert!(ChemicalInvariantValidator::validate_limiting_reagent(
            2.0, 1, "N2", 3.0, 3, "H2", "H2"
        ).is_ok());
        assert!(ChemicalInvariantValidator::validate_limiting_reagent(
            2.0, 1, "N2", 3.0, 3, "H2", "N2"
        ).is_err());
    }
}
