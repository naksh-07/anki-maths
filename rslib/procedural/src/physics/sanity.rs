// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

/// Speed of light in vacuum in m/s.
pub const SPEED_OF_LIGHT: f64 = 299_792_458.0;

/// Deterministic physical sanity and boundary constraint checker.
#[derive(Debug, Clone, Default)]
pub struct PhysicalSanityValidator;

impl PhysicalSanityValidator {
    /// Time must be non-negative in standard forward-evolution problems.
    pub fn check_time(t: f64) -> Result<(), String> {
        if t < 0.0 {
            Err(format!("Time cannot be negative in this context (got t = {:.2}s).", t))
        } else {
            Ok(())
        }
    }

    /// Mass must be strictly positive in classical mechanics.
    pub fn check_mass(m: f64) -> Result<(), String> {
        if m <= 0.0 {
            Err(format!("Mass must be strictly positive (got m = {:.2}kg).", m))
        } else {
            Ok(())
        }
    }

    /// Velocity magnitude cannot exceed the speed of light.
    pub fn check_sublight_speed(v: f64) -> Result<(), String> {
        if v.abs() >= SPEED_OF_LIGHT {
            Err(format!(
                "Speed violates relativistic limit |v| < c (got v = {:.2} m/s).",
                v
            ))
        } else {
            Ok(())
        }
    }

    /// Kinetic energy must be non-negative: KE = 1/2 m v^2 >= 0.
    pub fn check_kinetic_energy(ke: f64) -> Result<(), String> {
        if ke < -1e-6 {
            Err(format!("Kinetic energy cannot be negative (got KE = {:.2}J).", ke))
        } else {
            Ok(())
        }
    }

    /// Height above reference plane cannot be negative when floor is at h=0.
    pub fn check_height(h: f64) -> Result<(), String> {
        if h < -1e-6 {
            Err(format!("Height below datum reference plane (got h = {:.2}m).", h))
        } else {
            Ok(())
        }
    }

    /// Conservation of mechanical energy sanity: E_initial == E_final (within tolerance).
    pub fn check_energy_conservation(e_initial: f64, e_final: f64, tolerance: f64) -> Result<(), String> {
        if (e_initial - e_final).abs() > tolerance {
            Err(format!(
                "Energy conservation violated: Initial E = {:.2}J != Final E = {:.2}J (diff = {:.2}J).",
                e_initial, e_final, (e_initial - e_final).abs()
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
    fn test_time_and_mass_constraints() {
        assert!(PhysicalSanityValidator::check_time(5.0).is_ok());
        assert!(PhysicalSanityValidator::check_time(0.0).is_ok());
        assert!(PhysicalSanityValidator::check_time(-2.5).is_err());

        assert!(PhysicalSanityValidator::check_mass(2.0).is_ok());
        assert!(PhysicalSanityValidator::check_mass(0.0).is_err());
        assert!(PhysicalSanityValidator::check_mass(-10.0).is_err());
    }

    #[test]
    fn test_energy_and_speed_constraints() {
        assert!(PhysicalSanityValidator::check_kinetic_energy(150.0).is_ok());
        assert!(PhysicalSanityValidator::check_kinetic_energy(-50.0).is_err());

        assert!(PhysicalSanityValidator::check_sublight_speed(25.0).is_ok());
        assert!(PhysicalSanityValidator::check_sublight_speed(400_000_000.0).is_err());

        assert!(PhysicalSanityValidator::check_energy_conservation(100.0, 100.05, 0.1).is_ok());
        assert!(PhysicalSanityValidator::check_energy_conservation(100.0, 80.0, 0.1).is_err());
    }
}
