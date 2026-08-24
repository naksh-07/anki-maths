// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::str::FromStr;
use super::quantity::Quantity;
use super::unit_def::Unit;

/// Result of parsing a numerical input with optional unit.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedQuantity {
    pub value: f64,
    pub unit: Option<Unit>,
    pub raw_unit_str: Option<String>,
    pub has_explicit_unit: bool,
}

impl ParsedQuantity {
    pub fn to_quantity(&self, fallback_unit: Unit) -> Quantity {
        Quantity::new(self.value, self.unit.unwrap_or(fallback_unit))
    }
}

/// Robust parser for Physics, Chemistry, and Mathematics numerical responses.
pub struct UnitParser;

impl UnitParser {
    pub fn parse_json(input: &serde_json::Value) -> Option<ParsedQuantity> {
        match input {
            serde_json::Value::Number(n) => {
                n.as_f64().map(|v| ParsedQuantity {
                    value: v,
                    unit: None,
                    raw_unit_str: None,
                    has_explicit_unit: false,
                })
            }
            serde_json::Value::String(s) => Self::parse_string(s),
            serde_json::Value::Object(map) => {
                if let Some(val_obj) = map.get("value").or_else(|| map.get("answer")) {
                    let mut parsed = Self::parse_json(val_obj)?;
                    if let Some(unit_val) = map.get("unit").and_then(|u| u.as_str()) {
                        if let Ok(u) = Unit::from_str(unit_val) {
                            parsed.unit = Some(u);
                            parsed.has_explicit_unit = true;
                            parsed.raw_unit_str = Some(unit_val.to_string());
                        }
                    }
                    Some(parsed)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn parse_string(s: &str) -> Option<ParsedQuantity> {
        let mut trimmed = s.trim();
        if trimmed.is_empty() {
            return None;
        }

        // 1. Strip equation prefixes
        if let Some(pos) = trimmed.find(|c| c == '=' || c == ':' ) {
            let prefix = trimmed[..pos].trim();
            let is_valid_prefix = !prefix.is_empty()
                && prefix.chars().all(|c| c.is_alphabetic() || c == '_' || c == '[' || c == ']' || c == '+' || c == '-' || c.is_whitespace());
            if is_valid_prefix {
                trimmed = trimmed[pos + 1..].trim();
            } else {
                return None;
            }
        }

        // 2. Remove currency symbols and commas
        let cleaned = trimmed
            .replace('$', "")
            .replace('€', "")
            .replace('£', "")
            .replace('₹', "")
            .replace(',', "");

        let cleaned_trim = cleaned.trim();
        if cleaned_trim.is_empty() {
            return None;
        }

        // 3. Percent handling
        if cleaned_trim.ends_with('%') {
            let num_str = cleaned_trim[..cleaned_trim.len() - 1].trim();
            if let Ok(val) = num_str.parse::<f64>() {
                if !val.is_nan() && !val.is_infinite() {
                    return Some(ParsedQuantity {
                        value: val,
                        unit: Some(Unit::Percent),
                        raw_unit_str: Some("%".to_string()),
                        has_explicit_unit: true,
                    });
                }
            }
        }

        // 4. Scientific notation with x10^ or *10^ or ×10^
        let normalized = cleaned_trim
            .replace('⁰', "0")
            .replace('¹', "1")
            .replace('²', "2")
            .replace('³', "3")
            .replace('⁴', "4")
            .replace('⁵', "5")
            .replace('⁶', "6")
            .replace('⁷', "7")
            .replace('⁸', "8")
            .replace('⁹', "9")
            .replace('⁻', "-")
            .replace('⁺', "+")
            .replace('×', "x")
            .replace('·', "*")
            .replace('•', "*");

        let lower = normalized.to_lowercase();
        for marker in &[
            "x 10^", "* 10^", "x 10", "* 10",
            "x10^", "*10^", "x10", "*10",
        ] {
            if let Some(pos) = lower.find(marker) {
                let mantissa_part = lower[..pos].trim();
                let after_marker = &lower[pos + marker.len()..];
                let exp_digits: String = after_marker
                    .chars()
                    .take_while(|c| c.is_ascii_digit() || *c == '-' || *c == '+')
                    .collect();
                if let (Ok(m), Ok(e)) = (mantissa_part.parse::<f64>(), exp_digits.parse::<i32>()) {
                    let num_val = m * 10f64.powi(e);
                    if !num_val.is_nan() && !num_val.is_infinite() {
                        let rem = after_marker[exp_digits.len()..].trim();
                        let (unit, raw_unit) = if rem.is_empty() {
                            (Some(Unit::Dimensionless), None)
                        } else if let Ok(u) = Unit::from_str(rem) {
                            (Some(u), Some(rem.to_string()))
                        } else {
                            (None, Some(rem.to_string()))
                        };
                        return Some(ParsedQuantity {
                            value: num_val,
                            unit,
                            raw_unit_str: raw_unit,
                            has_explicit_unit: !rem.is_empty(),
                        });
                    }
                }
            }
        }

        // 5. Fraction format
        if let Some(slash_idx) = normalized.find('/') {
            let left = normalized[..slash_idx].trim();
            let right = normalized[slash_idx + 1..].trim();
            let is_unit_slash = left.chars().any(|c| c.is_alphabetic()) && !left.chars().any(|c| c.is_ascii_digit());
            if !is_unit_slash {
                let num_res = left.parse::<f64>();
                let den_digits: String = right
                    .chars()
                    .take_while(|c| c.is_ascii_digit() || *c == '.' || *c == '-' || *c == '+')
                    .collect();
                if let (Ok(num), Ok(den)) = (num_res, den_digits.parse::<f64>()) {
                    if den.abs() > f64::EPSILON {
                        let frac_val = num / den;
                        if !frac_val.is_nan() && !frac_val.is_infinite() {
                            let rem = right[den_digits.len()..].trim();
                            let (unit, raw_unit) = if rem.is_empty() {
                                (Some(Unit::Dimensionless), None)
                            } else if let Ok(u) = Unit::from_str(rem) {
                                (Some(u), Some(rem.to_string()))
                            } else {
                                (None, Some(rem.to_string()))
                            };
                            return Some(ParsedQuantity {
                                value: frac_val,
                                unit,
                                raw_unit_str: raw_unit,
                                has_explicit_unit: !rem.is_empty(),
                            });
                        }
                    }
                }
            }
        }

        // 6. Leading float extraction
        let mut chars = normalized.chars().peekable();
        let mut num_str = String::new();
        if let Some(&c) = chars.peek() {
            if c == '+' || c == '-' {
                num_str.push(chars.next().unwrap());
            }
        }
        let mut has_dot = false;
        let mut has_exp = false;
        while let Some(&c) = chars.peek() {
            if c.is_ascii_digit() {
                num_str.push(chars.next().unwrap());
            } else if c == '.' && !has_dot && !has_exp {
                has_dot = true;
                num_str.push(chars.next().unwrap());
            } else if (c == 'e' || c == 'E') && !has_exp && !num_str.is_empty() {
                has_exp = true;
                num_str.push(chars.next().unwrap());
                if let Some(&sign) = chars.peek() {
                    if sign == '+' || sign == '-' {
                        num_str.push(chars.next().unwrap());
                    }
                }
            } else {
                break;
            }
        }

        if num_str.chars().any(|c| c.is_ascii_digit()) {
            if let Ok(num_val) = num_str.parse::<f64>() {
                if !num_val.is_nan() && !num_val.is_infinite() {
                    let remainder: String = chars.collect();
                    let rem_trimmed = remainder.trim();
                    if rem_trimmed.is_empty() {
                        return Some(ParsedQuantity {
                            value: num_val,
                            unit: Some(Unit::Dimensionless),
                            raw_unit_str: None,
                            has_explicit_unit: false,
                        });
                    }
                    if let Ok(unit) = Unit::from_str(rem_trimmed) {
                        return Some(ParsedQuantity {
                            value: num_val,
                            unit: Some(unit),
                            raw_unit_str: Some(rem_trimmed.to_string()),
                            has_explicit_unit: true,
                        });
                    } else {
                        return Some(ParsedQuantity {
                            value: num_val,
                            unit: None,
                            raw_unit_str: Some(rem_trimmed.to_string()),
                            has_explicit_unit: true,
                        });
                    }
                }
            }
        }
        None
    }

    pub fn parse_scalar(s: &str) -> Option<f64> {
        Self::parse_string(s).map(|p| p.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unit_parser_cases() {
        let p1 = UnitParser::parse_string("12 m/s").unwrap();
        assert_eq!(p1.value, 12.0);
        assert_eq!(p1.unit, Some(Unit::MeterPerSecond));
        let p2 = UnitParser::parse_string("v = 72 km/h").unwrap();
        assert_eq!(p2.value, 72.0);
        assert_eq!(p2.unit, Some(Unit::KilometerPerHour));
        let p3 = UnitParser::parse_string("5 kg").unwrap();
        assert_eq!(p3.value, 5.0);
        assert_eq!(p3.unit, Some(Unit::Kilogram));
        let p4 = UnitParser::parse_string("2.5 mol").unwrap();
        assert_eq!(p4.value, 2.5);
        assert_eq!(p4.unit, Some(Unit::Mole));
        let p5 = UnitParser::parse_string("1.2e-3 mol/L").unwrap();
        assert!((p5.value - 0.0012).abs() < 1e-6);
        assert_eq!(p5.unit, Some(Unit::Molar));
        let p6 = UnitParser::parse_string("0.0012 M").unwrap();
        assert!((p6.value - 0.0012).abs() < 1e-6);
        assert_eq!(p6.unit, Some(Unit::Molar));
        let p7 = UnitParser::parse_string("3/4 m/s").unwrap();
        assert_eq!(p7.value, 0.75);
        assert_eq!(p7.unit, Some(Unit::MeterPerSecond));
        let p8 = UnitParser::parse_string("-9.8 m/s^2").unwrap();
        assert_eq!(p8.value, -9.8);
        assert_eq!(p8.unit, Some(Unit::MeterPerSecondSquared));
        let p9 = UnitParser::parse_string("1.2 x 10^-3 mol/L").unwrap();
        assert!((p9.value - 0.0012).abs() < 1e-6);
        assert_eq!(p9.unit, Some(Unit::Molar));
        let p10 = UnitParser::parse_string("3x10^4").unwrap();
        assert_eq!(p10.value, 30000.0);
        assert_eq!(p10.unit, Some(Unit::Dimensionless));
        let p11 = UnitParser::parse_string("+32%").unwrap();
        assert_eq!(p11.value, 32.0);
        assert_eq!(p11.unit, Some(Unit::Percent));
        let p12 = UnitParser::parse_string("  1,200.50  ").unwrap();
        assert_eq!(p12.value, 1200.5);
        assert_eq!(p12.unit, Some(Unit::Dimensionless));

        // Additional physics and chemistry cases
        let p13 = UnitParser::parse_string("6.022e23").unwrap();
        assert!((p13.value - 6.022e23).abs() / 6.022e23 < 1e-6);
        assert_eq!(p13.unit, Some(Unit::Dimensionless));

        let p14 = UnitParser::parse_string("6.022 x 10^23").unwrap();
        assert!((p14.value - 6.022e23).abs() / 6.022e23 < 1e-6);

        let p15 = UnitParser::parse_string("6.022 x 10²³").unwrap();
        assert!((p15.value - 6.022e23).abs() / 6.022e23 < 1e-6);

        let p16 = UnitParser::parse_string("1.2 × 10⁻³ mol/L").unwrap();
        assert!((p16.value - 0.0012).abs() < 1e-6);
        assert_eq!(p16.unit, Some(Unit::Molar));

        let p17 = UnitParser::parse_string("50.5 kJ/mol").unwrap();
        assert_eq!(p17.value, 50.5);
        assert_eq!(p17.unit, Some(Unit::KilojoulePerMole));

        let p18 = UnitParser::parse_string("25 °C").unwrap();
        assert_eq!(p18.value, 25.0);
        assert_eq!(p18.unit, Some(Unit::Celsius));

        let p19 = UnitParser::parse_string("101.325 kPa").unwrap();
        assert_eq!(p19.value, 101.325);
        assert_eq!(p19.unit, Some(Unit::Kilopascal));

        let p20 = UnitParser::parse_string("1.03 g/cm^3").unwrap();
        assert_eq!(p20.value, 1.03);
        assert_eq!(p20.unit, Some(Unit::GramPerCubicCentimeter));

        assert!(UnitParser::parse_string("invalid_input_text").is_none());
    }
}
