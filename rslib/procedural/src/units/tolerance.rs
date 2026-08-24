// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use serde::{Deserialize, Serialize};

/// Tolerance specification for numerical evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Tolerance {
    Absolute { absolute: f64 },
    Relative { relative: f64 },
    Combined { absolute: f64, relative: f64 },
}

impl Tolerance {
    pub fn absolute(tol: f64) -> Self {
        Self::Absolute { absolute: tol.abs() }
    }

    pub fn relative(fraction: f64) -> Self {
        Self::Relative { relative: fraction.abs() }
    }

    pub fn combined(absolute: f64, relative: f64) -> Self {
        Self::Combined {
            absolute: absolute.abs(),
            relative: relative.abs(),
        }
    }

    pub fn default_physics() -> Self {
        Self::combined(0.1, 0.005) // 0.5% or 0.1 absolute
    }

    pub fn default_chemistry() -> Self {
        Self::combined(1e-4, 0.01) // 1% or 1e-4 absolute
    }

    pub fn default_math() -> Self {
        Self::combined(0.01, 0.001) // 0.1% or 0.01 absolute
    }

    pub fn from_json_or_default(v: Option<&serde_json::Value>, default_tol: Tolerance) -> Self {
        let Some(val) = v else { return default_tol; };
        match val {
            serde_json::Value::Number(n) => {
                if let Some(f) = n.as_f64() {
                    Self::absolute(f)
                } else {
                    default_tol
                }
            }
            serde_json::Value::Object(map) => {
                let abs_val = map.get("absolute").and_then(|v| v.as_f64());
                let rel_val = map.get("relative").or_else(|| map.get("percent")).and_then(|v| v.as_f64());
                match (abs_val, rel_val) {
                    (Some(a), Some(r)) => Self::combined(a, r),
                    (Some(a), None) => Self::absolute(a),
                    (None, Some(r)) => Self::relative(r),
                    _ => default_tol,
                }
            }
            _ => default_tol,
        }
    }

    pub fn is_within(&self, actual: f64, expected: f64) -> bool {
        if actual.is_nan() || expected.is_nan() || actual.is_infinite() || expected.is_infinite() {
            return false;
        }
        let diff = (actual - expected).abs();
        match *self {
            Tolerance::Absolute { absolute } => diff <= absolute,
            Tolerance::Relative { relative } => diff <= expected.abs() * relative,
            Tolerance::Combined { absolute, relative } => {
                diff <= absolute.max(expected.abs() * relative)
            }
        }
    }
}

impl Default for Tolerance {
    fn default() -> Self {
        Self::combined(0.01, 0.01)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tolerance_checks() {
        let tol_abs = Tolerance::absolute(0.5);
        assert!(tol_abs.is_within(10.4, 10.0));
        assert!(!tol_abs.is_within(10.6, 10.0));
        let tol_rel = Tolerance::relative(0.05);
        assert!(tol_rel.is_within(104.0, 100.0));
        assert!(!tol_rel.is_within(106.0, 100.0));
        let tol_comb = Tolerance::combined(0.1, 0.01);
        assert!(tol_comb.is_within(0.05, 0.0));
        assert!(tol_comb.is_within(100.9, 100.0));
        assert!(!tol_comb.is_within(102.0, 100.0));
    }
}
