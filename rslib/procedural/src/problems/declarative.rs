use std::collections::HashMap;
use std::sync::Arc;

use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use serde_json::{json, Value};

use crate::core::{Domain, ProblemFamilyId, ProceduralError, Result};
use crate::problems::contract::{
    AnswerDerivation, ConstraintSpec, DeclarativeArchetype, DeclarativeFamilyContract,
    ParameterDomain, ParameterSpec, ProblemFamilyCapability, ProblemFamilyContract, StepNodeSpec,
};
use crate::problems::generator::ProblemGenerator;
use crate::problems::steps::{SolutionGraph, StepHint, StepNode, StepType};
use crate::problems::ProblemInstance;
use crate::skills::signals::VariantCategory;

fn gcd_u64(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

fn lcm_u64(a: u64, b: u64) -> u64 {
    if a == 0 || b == 0 {
        0
    } else {
        let gcd = gcd_u64(a, b);
        (a / gcd).checked_mul(b).unwrap_or(u64::MAX)
    }
}

/// Canonical answer result value supporting both numeric and text forms.
#[derive(Debug, Clone, PartialEq)]
pub enum CanonicalAnswerValue {
    Numeric(f64),
    Text(String),
}

impl CanonicalAnswerValue {
    pub fn to_json_value(&self) -> Value {
        match self {
            CanonicalAnswerValue::Numeric(n) => json!(n),
            CanonicalAnswerValue::Text(t) => json!(t),
        }
    }

    pub fn format_default(&self) -> String {
        match self {
            CanonicalAnswerValue::Numeric(n) => {
                if n.fract() == 0.0 {
                    format!("{:.0}", n)
                } else {
                    format!("{:.2}", n)
                }
            }
            CanonicalAnswerValue::Text(t) => t.clone(),
        }
    }
}

/// Dynamic problem generator driven by a declarative contract and archetype specifications.
#[derive(Debug, Clone)]
pub struct DeclarativeProblemGenerator {
    contract: Arc<DeclarativeFamilyContract>,
    template_ref: String,
}

impl DeclarativeProblemGenerator {
    pub fn new(contract: Arc<DeclarativeFamilyContract>) -> Self {
        let template_ref = format!("{}.declarative.v1", contract.contract.family_id.as_str());
        Self {
            contract,
            template_ref,
        }
    }

    pub fn with_template_ref(contract: Arc<DeclarativeFamilyContract>, template_ref: impl Into<String>) -> Self {
        Self {
            contract,
            template_ref: template_ref.into(),
        }
    }

    pub fn contract(&self) -> &DeclarativeFamilyContract {
        &self.contract
    }

    /// Evaluates string template with `{placeholder}` substitution from parameter map.
    pub fn interpolate(template: &str, params: &HashMap<String, String>) -> String {
        let mut result = template.to_string();
        for (key, val) in params {
            let placeholder = format!("{{{}}}", key);
            result = result.replace(&placeholder, val);
        }
        result
    }

    /// Sample parameters and evaluate derived expressions deterministically from RNG.
    fn sample_parameters(
        archetype: &DeclarativeArchetype,
        rng: &mut StdRng,
    ) -> (HashMap<String, Value>, HashMap<String, String>) {
        let mut raw_params: HashMap<String, Value> = HashMap::new();
        let mut str_params: HashMap<String, String> = HashMap::new();

        // 1. Initial pass: sample direct parameters
        for spec in &archetype.parameters {
            match &spec.domain {
                ParameterDomain::IntegerRange { min, max, step, non_zero } => {
                    let mut val = if min > max {
                        *min
                    } else if let Some(s) = step {
                        let step_val = (*s).max(1);
                        let span = max.saturating_sub(*min);
                        let steps_count = span / step_val;
                        let step_idx = rng.random_range(0..=steps_count);
                        min.saturating_add(step_idx.saturating_mul(step_val)).min(*max)
                    } else {
                        rng.random_range(*min..=*max)
                    };

                    if non_zero.unwrap_or(false) && val == 0 {
                        val = if rng.random_bool(0.5) {
                            if *max >= 1 { 1 } else { -1 }
                        } else {
                            if *min <= -1 { -1 } else { 1 }
                        };
                    }

                    raw_params.insert(spec.name.clone(), Value::from(val));
                    str_params.insert(spec.name.clone(), format!("{}", val));
                }
                ParameterDomain::FloatRange { min, max, precision } => {
                    let val = if min.is_nan() || max.is_nan() || min.is_infinite() || max.is_infinite() || min >= max {
                        if min.is_nan() || min.is_infinite() { 0.0 } else { *min }
                    } else {
                        rng.random_range(*min..=*max)
                    };
                    let formatted = format!("{:.1$}", val, precision);
                    raw_params.insert(spec.name.clone(), Value::from(val));
                    str_params.insert(spec.name.clone(), formatted);
                }
                ParameterDomain::DiscreteChoice { values } => {
                    if !values.is_empty() {
                        let idx = rng.random_range(0..values.len());
                        let val = &values[idx];
                        raw_params.insert(spec.name.clone(), val.clone());
                        let s = match val {
                            Value::String(s) => s.clone(),
                            Value::Number(n) => n.to_string(),
                            Value::Bool(b) => b.to_string(),
                            _ => val.to_string(),
                        };
                        str_params.insert(spec.name.clone(), s);
                    }
                }
                ParameterDomain::PermutationChoice { pool, count } => {
                    let mut pool_clone = pool.clone();
                    pool_clone.shuffle(rng);
                    let selected: Vec<String> = pool_clone.into_iter().take(*count).collect();
                    for (idx, item) in selected.iter().enumerate() {
                        let sub_key = format!("{}_{}", spec.name, idx);
                        raw_params.insert(sub_key.clone(), Value::String(item.clone()));
                        str_params.insert(sub_key, item.clone());
                    }
                    raw_params.insert(
                        spec.name.clone(),
                        Value::Array(selected.iter().map(|s| Value::String(s.clone())).collect()),
                    );
                    str_params.insert(spec.name.clone(), selected.join(", "));
                }
                ParameterDomain::PrimeFactorGrid {
                    base_primes,
                    min_exponents,
                    max_exponents,
                } => {
                    let mut composite = 1u64;
                    for (i, &p) in base_primes.iter().enumerate() {
                        let min_e = min_exponents.get(i).copied().unwrap_or(1).min(63);
                        let max_e = max_exponents.get(i).copied().unwrap_or(min_e).min(63);
                        let exp = if max_e > min_e {
                            rng.random_range(min_e..=max_e)
                        } else {
                            min_e
                        };
                        let term = p.checked_pow(exp).unwrap_or(u64::MAX);
                        composite = composite.saturating_mul(term);
                    }
                    raw_params.insert(spec.name.clone(), Value::from(composite));
                    str_params.insert(spec.name.clone(), composite.to_string());
                }
                ParameterDomain::CoprimePair { min, max } => {
                    let mut a_val = *min;
                    let mut b_val = *max;
                    if min < max {
                        for _ in 0..50 {
                            let a = rng.random_range(*min..=*max);
                            let b = rng.random_range(*min..=*max);
                            if a != b && gcd_u64(a.unsigned_abs(), b.unsigned_abs()) == 1 {
                                a_val = a;
                                b_val = b;
                                break;
                            }
                        }
                    }
                    raw_params.insert(format!("{}_a", spec.name), Value::from(a_val));
                    str_params.insert(format!("{}_a", spec.name), a_val.to_string());
                    raw_params.insert(format!("{}_b", spec.name), Value::from(b_val));
                    str_params.insert(format!("{}_b", spec.name), b_val.to_string());
                    raw_params.insert(spec.name.clone(), Value::from(a_val));
                    str_params.insert(spec.name.clone(), a_val.to_string());
                }
                _ => {}
            }
        }

        // 2. Second pass: evaluate derived expressions in sequence
        for spec in &archetype.parameters {
            match &spec.domain {
                ParameterDomain::DerivedLinear { a_param, x_param, b_param } => {
                    let a = raw_params.get(a_param).and_then(|v| v.as_i64()).unwrap_or(1);
                    let x = raw_params.get(x_param).and_then(|v| v.as_i64()).unwrap_or(0);
                    let b = raw_params.get(b_param).and_then(|v| v.as_i64()).unwrap_or(0);
                    let c = a.saturating_mul(x).saturating_add(b);
                    raw_params.insert(spec.name.clone(), Value::from(c));
                    str_params.insert(spec.name.clone(), format!("{}", c));
                }
                ParameterDomain::DerivedProduct { a_param, b_param } => {
                    let a = raw_params.get(a_param).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))).unwrap_or(1.0);
                    let b = raw_params.get(b_param).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))).unwrap_or(1.0);
                    let res = a * b;
                    raw_params.insert(spec.name.clone(), Value::from(res));
                    str_params.insert(spec.name.clone(), if res.fract() == 0.0 { format!("{:.0}", res) } else { format!("{:.2}", res) });
                }
                ParameterDomain::DerivedSum { a_param, b_param } => {
                    let a = raw_params.get(a_param).and_then(|v| v.as_i64()).unwrap_or(0);
                    let b = raw_params.get(b_param).and_then(|v| v.as_i64()).unwrap_or(0);
                    let res = a.saturating_add(b);
                    raw_params.insert(spec.name.clone(), Value::from(res));
                    str_params.insert(spec.name.clone(), format!("{}", res));
                }
                ParameterDomain::DerivedDifference { a_param, b_param } => {
                    let a = raw_params.get(a_param).and_then(|v| v.as_i64()).unwrap_or(0);
                    let b = raw_params.get(b_param).and_then(|v| v.as_i64()).unwrap_or(0);
                    let res = a.saturating_sub(b);
                    raw_params.insert(spec.name.clone(), Value::from(res));
                    str_params.insert(spec.name.clone(), format!("{}", res));
                }
                ParameterDomain::DerivedQuotient { a_param, b_param, precision } => {
                    let a = raw_params.get(a_param).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))).unwrap_or(0.0);
                    let b = raw_params.get(b_param).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))).unwrap_or(1.0);
                    let res = if b.abs() < 1e-9 { 0.0 } else { a / b };
                    raw_params.insert(spec.name.clone(), Value::from(res));
                    let prec = precision.unwrap_or(2);
                    let formatted = if res.fract() == 0.0 { format!("{:.0}", res) } else { format!("{:.1$}", res, prec) };
                    str_params.insert(spec.name.clone(), formatted);
                }
                ParameterDomain::DerivedSignedString { param } => {
                    let b = raw_params.get(param).and_then(|v| v.as_i64()).unwrap_or(0);
                    let formatted = if b >= 0 {
                        format!("+ {}", b)
                    } else {
                        format!("- {}", b.unsigned_abs())
                    };
                    raw_params.insert(spec.name.clone(), Value::from(formatted.clone()));
                    str_params.insert(spec.name.clone(), formatted);
                }
                ParameterDomain::DerivedPower { base_param, exponent } => {
                    let base = raw_params.get(base_param).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))).unwrap_or(1.0);
                    let res = base.powi(*exponent as i32);
                    raw_params.insert(spec.name.clone(), Value::from(res));
                    str_params.insert(spec.name.clone(), if res.fract() == 0.0 { format!("{:.0}", res) } else { format!("{:.2}", res) });
                }
                ParameterDomain::DerivedPercentage { base_param, rate_param } => {
                    let base = raw_params.get(base_param).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))).unwrap_or(0.0);
                    let rate = raw_params.get(rate_param).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))).unwrap_or(0.0);
                    let res = (base * rate) / 100.0;
                    raw_params.insert(spec.name.clone(), Value::from(res));
                    str_params.insert(spec.name.clone(), if res.fract() == 0.0 { format!("{:.0}", res) } else { format!("{:.2}", res) });
                }
                ParameterDomain::DerivedHypotenuse { a_param, b_param } => {
                    let a = raw_params.get(a_param).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))).unwrap_or(0.0);
                    let b = raw_params.get(b_param).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))).unwrap_or(0.0);
                    let res = (a * a + b * b).sqrt();
                    raw_params.insert(spec.name.clone(), Value::from(res));
                    str_params.insert(spec.name.clone(), if res.fract() == 0.0 { format!("{:.0}", res) } else { format!("{:.2}", res) });
                }
                ParameterDomain::DerivedPythagoreanLeg { c_param, a_param } => {
                    let c = raw_params.get(c_param).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))).unwrap_or(0.0);
                    let a = raw_params.get(a_param).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))).unwrap_or(0.0);
                    let diff = (c * c - a * a).max(0.0);
                    let res = diff.sqrt();
                    raw_params.insert(spec.name.clone(), Value::from(res));
                    str_params.insert(spec.name.clone(), if res.fract() == 0.0 { format!("{:.0}", res) } else { format!("{:.2}", res) });
                }
                _ => {}
            }
        }

        (raw_params, str_params)
    }

    /// Verify whether sampled parameters satisfy all declared constraints.
    fn check_constraints(archetype: &DeclarativeArchetype, raw_params: &HashMap<String, Value>) -> bool {
        for constraint in &archetype.constraints {
            match constraint {
                ConstraintSpec::NotEqual { param_a, param_b } => {
                    let val_a = raw_params.get(param_a);
                    let val_b = raw_params.get(param_b);
                    if let (Some(a), Some(b)) = (val_a, val_b) {
                        if a == b {
                            return false;
                        }
                    }
                }
                ConstraintSpec::NonZero { param } => {
                    if let Some(val) = raw_params.get(param) {
                        if let Some(i) = val.as_i64() {
                            if i == 0 { return false; }
                        } else if let Some(f) = val.as_f64() {
                            if f.abs() < 1e-9 { return false; }
                        }
                    }
                }
                ConstraintSpec::Divisible { numerator, denominator } => {
                    let num = raw_params.get(numerator).and_then(|v| v.as_i64());
                    let den = raw_params.get(denominator).and_then(|v| v.as_i64());
                    if let (Some(n), Some(d)) = (num, den) {
                        if d == 0 || n % d != 0 {
                            return false;
                        }
                    }
                }
                ConstraintSpec::GreaterThan { param_a, param_b } => {
                    let val_a = raw_params.get(param_a).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64)));
                    let val_b = raw_params.get(param_b).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64)));
                    if let (Some(a), Some(b)) = (val_a, val_b) {
                        if a <= b {
                            return false;
                        }
                    }
                }
                ConstraintSpec::LessThan { param_a, param_b } => {
                    let val_a = raw_params.get(param_a).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64)));
                    let val_b = raw_params.get(param_b).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64)));
                    if let (Some(a), Some(b)) = (val_a, val_b) {
                        if a >= b {
                            return false;
                        }
                    }
                }
                ConstraintSpec::SumEquals { param_a, param_b, target } => {
                    let val_a = raw_params.get(param_a).and_then(|v| v.as_i64());
                    let val_b = raw_params.get(param_b).and_then(|v| v.as_i64());
                    if let (Some(a), Some(b)) = (val_a, val_b) {
                        if a + b != *target {
                            return false;
                        }
                    }
                }
                ConstraintSpec::Predicate { .. } => {}
            }
        }
        true
    }

    /// Compute canonical answer (numeric or text) according to derivation specification.
    fn compute_answer(
        derivation: &AnswerDerivation,
        raw_params: &HashMap<String, Value>,
    ) -> Result<CanonicalAnswerValue> {
        match derivation {
            AnswerDerivation::DirectParam { param_name } => {
                let val = raw_params
                    .get(param_name)
                    .and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64)))
                    .ok_or_else(|| {
                        ProceduralError::Validation(format!(
                            "Missing parameter {} for answer derivation",
                            param_name
                        ))
                    })?;
                Ok(CanonicalAnswerValue::Numeric(val))
            }
            AnswerDerivation::DirectStringParam { param_name } => {
                let s = raw_params
                    .get(param_name)
                    .map(|v| match v {
                        Value::String(s) => s.clone(),
                        _ => v.to_string(),
                    })
                    .ok_or_else(|| {
                        ProceduralError::Validation(format!(
                            "Missing parameter {} for string answer derivation",
                            param_name
                        ))
                    })?;
                Ok(CanonicalAnswerValue::Text(s))
            }
            AnswerDerivation::LinearTwoStep { c_param, b_param, a_param } => {
                let c = raw_params.get(c_param).and_then(|v| v.as_i64()).unwrap_or(0);
                let b = raw_params.get(b_param).and_then(|v| v.as_i64()).unwrap_or(0);
                let a = raw_params.get(a_param).and_then(|v| v.as_i64()).unwrap_or(1);
                if a == 0 {
                    return Err(ProceduralError::Validation("Divisor 'a' cannot be zero".to_string()));
                }
                Ok(CanonicalAnswerValue::Numeric((c - b) as f64 / a as f64))
            }
            AnswerDerivation::LinearVariablesBothSides { d_param, b_param, a_param, c_param } => {
                let d = raw_params.get(d_param).and_then(|v| v.as_i64()).unwrap_or(0);
                let b = raw_params.get(b_param).and_then(|v| v.as_i64()).unwrap_or(0);
                let a = raw_params.get(a_param).and_then(|v| v.as_i64()).unwrap_or(1);
                let c = raw_params.get(c_param).and_then(|v| v.as_i64()).unwrap_or(0);
                let diff_a = a - c;
                if diff_a == 0 {
                    return Err(ProceduralError::Validation("Denominator (a - c) cannot be zero".to_string()));
                }
                Ok(CanonicalAnswerValue::Numeric((d - b) as f64 / diff_a as f64))
            }
            AnswerDerivation::LinearDistributive { d_param, a_param, c_param, b_param } => {
                let d = raw_params.get(d_param).and_then(|v| v.as_i64()).unwrap_or(0);
                let a = raw_params.get(a_param).and_then(|v| v.as_i64()).unwrap_or(1);
                let c = raw_params.get(c_param).and_then(|v| v.as_i64()).unwrap_or(0);
                let b = raw_params.get(b_param).and_then(|v| v.as_i64()).unwrap_or(1);
                if a == 0 || b == 0 {
                    return Err(ProceduralError::Validation("Distributive divisors cannot be zero".to_string()));
                }
                let inner = (d as f64 / a as f64) - c as f64;
                Ok(CanonicalAnswerValue::Numeric(inner / b as f64))
            }
            AnswerDerivation::LinearFractional { c_param, b_param, a_param } => {
                let c = raw_params.get(c_param).and_then(|v| v.as_i64()).unwrap_or(0);
                let b = raw_params.get(b_param).and_then(|v| v.as_i64()).unwrap_or(0);
                let a = raw_params.get(a_param).and_then(|v| v.as_i64()).unwrap_or(1);
                Ok(CanonicalAnswerValue::Numeric((a * (c - b)) as f64))
            }
            AnswerDerivation::Quotient { numerator_param, denominator_param } => {
                let num = raw_params.get(numerator_param).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))).unwrap_or(0.0);
                let den = raw_params.get(denominator_param).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))).unwrap_or(1.0);
                if den.abs() < 1e-9 {
                    return Err(ProceduralError::Validation("Denominator cannot be zero".to_string()));
                }
                Ok(CanonicalAnswerValue::Numeric(num / den))
            }
            AnswerDerivation::Product { a_param, b_param } => {
                let a = raw_params.get(a_param).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))).unwrap_or(0.0);
                let b = raw_params.get(b_param).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))).unwrap_or(0.0);
                Ok(CanonicalAnswerValue::Numeric(a * b))
            }
            AnswerDerivation::PercentageAmount { base_param, percent_param } => {
                let base = raw_params.get(base_param).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))).unwrap_or(0.0);
                let pct = raw_params.get(percent_param).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))).unwrap_or(0.0);
                Ok(CanonicalAnswerValue::Numeric((base * pct) / 100.0))
            }
            AnswerDerivation::LcmArray { params } => {
                if params.is_empty() {
                    return Err(ProceduralError::Validation("LcmArray requires at least 1 parameter".to_string()));
                }
                let mut current_lcm = raw_params.get(&params[0]).and_then(|v| v.as_i64()).unwrap_or(1).unsigned_abs();
                for p in &params[1..] {
                    let next_val = raw_params.get(p).and_then(|v| v.as_i64()).unwrap_or(1).unsigned_abs();
                    current_lcm = lcm_u64(current_lcm, next_val);
                }
                Ok(CanonicalAnswerValue::Numeric(current_lcm as f64))
            }
            AnswerDerivation::GcdArray { params } => {
                if params.is_empty() {
                    return Err(ProceduralError::Validation("GcdArray requires at least 1 parameter".to_string()));
                }
                let mut current_gcd = raw_params.get(&params[0]).and_then(|v| v.as_i64()).unwrap_or(1).unsigned_abs();
                for p in &params[1..] {
                    let next_val = raw_params.get(p).and_then(|v| v.as_i64()).unwrap_or(1).unsigned_abs();
                    current_gcd = gcd_u64(current_gcd, next_val);
                }
                Ok(CanonicalAnswerValue::Numeric(current_gcd as f64))
            }
            AnswerDerivation::Remainder { dividend_param, divisor_param } => {
                let num = raw_params.get(dividend_param).and_then(|v| v.as_i64()).unwrap_or(0);
                let den = raw_params.get(divisor_param).and_then(|v| v.as_i64()).unwrap_or(1);
                if den == 0 || (num == i64::MIN && den == -1) {
                    return Err(ProceduralError::Validation("Invalid divisor or arithmetic overflow".to_string()));
                }
                Ok(CanonicalAnswerValue::Numeric((num % den) as f64))
            }
            AnswerDerivation::PythagorasHypotenuse { a_param, b_param } => {
                let a = raw_params.get(a_param).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))).unwrap_or(0.0);
                let b = raw_params.get(b_param).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))).unwrap_or(0.0);
                Ok(CanonicalAnswerValue::Numeric((a * a + b * b).sqrt()))
            }
            AnswerDerivation::PythagorasLeg { c_param, a_param } => {
                let c = raw_params.get(c_param).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))).unwrap_or(0.0);
                let a = raw_params.get(a_param).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))).unwrap_or(0.0);
                let diff = (c * c - a * a).max(0.0);
                Ok(CanonicalAnswerValue::Numeric(diff.sqrt()))
            }
            AnswerDerivation::TriangleArea { base_param, height_param } => {
                let b = raw_params.get(base_param).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))).unwrap_or(0.0);
                let h = raw_params.get(height_param).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))).unwrap_or(0.0);
                Ok(CanonicalAnswerValue::Numeric(0.5 * b * h))
            }
            AnswerDerivation::CircleArea { radius_param, pi_approx } => {
                let r = raw_params.get(radius_param).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))).unwrap_or(0.0);
                let pi = pi_approx.unwrap_or(std::f64::consts::PI);
                Ok(CanonicalAnswerValue::Numeric(pi * r * r))
            }
            AnswerDerivation::ArithmeticSeriesSum { n_param, a_param, d_param } => {
                let n = raw_params.get(n_param).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))).unwrap_or(1.0);
                let a = raw_params.get(a_param).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))).unwrap_or(0.0);
                let d = raw_params.get(d_param).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))).unwrap_or(0.0);
                let s_n = (n / 2.0) * (2.0 * a + (n - 1.0) * d);
                Ok(CanonicalAnswerValue::Numeric(s_n))
            }
            AnswerDerivation::KinematicVelocity { u_param, a_param, t_param } => {
                let u = raw_params.get(u_param).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))).unwrap_or(0.0);
                let a = raw_params.get(a_param).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))).unwrap_or(0.0);
                let t = raw_params.get(t_param).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))).unwrap_or(0.0);
                Ok(CanonicalAnswerValue::Numeric(u + a * t))
            }
            AnswerDerivation::KinematicDisplacement { u_param, a_param, t_param } => {
                let u = raw_params.get(u_param).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))).unwrap_or(0.0);
                let a = raw_params.get(a_param).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))).unwrap_or(0.0);
                let t = raw_params.get(t_param).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))).unwrap_or(0.0);
                Ok(CanonicalAnswerValue::Numeric(u * t + 0.5 * a * t * t))
            }
            AnswerDerivation::KinematicStoppingDistance { u_param, a_param } => {
                let u = raw_params.get(u_param).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))).unwrap_or(0.0);
                let a = raw_params.get(a_param).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))).unwrap_or(1.0);
                if a.abs() < 1e-9 {
                    return Err(ProceduralError::Validation("Acceleration cannot be zero for stopping distance".to_string()));
                }
                Ok(CanonicalAnswerValue::Numeric((u * u) / (2.0 * a)))
            }
            AnswerDerivation::KinematicTime { u_param, v_param, a_param } => {
                let u = raw_params.get(u_param).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))).unwrap_or(0.0);
                let v = raw_params.get(v_param).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))).unwrap_or(0.0);
                let a = raw_params.get(a_param).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))).unwrap_or(1.0);
                if a.abs() < 1e-9 {
                    return Err(ProceduralError::Validation("Acceleration cannot be zero for kinematic time".to_string()));
                }
                Ok(CanonicalAnswerValue::Numeric((v - u) / a))
            }
            AnswerDerivation::KinematicWorkEnergy { mass_param, velocity_param } => {
                let m = raw_params.get(mass_param).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))).unwrap_or(0.0);
                let v = raw_params.get(velocity_param).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))).unwrap_or(0.0);
                Ok(CanonicalAnswerValue::Numeric(0.5 * m * v * v))
            }
            AnswerDerivation::StoichiometricMolesToMass { moles_param, molar_mass_param } => {
                let n = raw_params.get(moles_param).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))).unwrap_or(0.0);
                let m_mol = raw_params.get(molar_mass_param).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))).unwrap_or(1.0);
                Ok(CanonicalAnswerValue::Numeric(n * m_mol))
            }
            AnswerDerivation::StoichiometricMassToMoles { mass_param, molar_mass_param } => {
                let m = raw_params.get(mass_param).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))).unwrap_or(0.0);
                let m_mol = raw_params.get(molar_mass_param).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))).unwrap_or(1.0);
                if m_mol.abs() < 1e-9 {
                    return Err(ProceduralError::Validation("Molar mass cannot be zero".to_string()));
                }
                Ok(CanonicalAnswerValue::Numeric(m / m_mol))
            }
            AnswerDerivation::StoichiometricMoleRatio { moles_a_param, coeff_a, coeff_b } => {
                let n_a = raw_params.get(moles_a_param).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))).unwrap_or(0.0);
                if coeff_a.abs() < 1e-9 {
                    return Err(ProceduralError::Validation("Coefficient A cannot be zero".to_string()));
                }
                Ok(CanonicalAnswerValue::Numeric(n_a * (*coeff_b / *coeff_a)))
            }
            AnswerDerivation::StoichiometricMassToMass { mass_a_param, molar_mass_a, coeff_a, coeff_b, molar_mass_b } => {
                let m_a = raw_params.get(mass_a_param).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))).unwrap_or(0.0);
                let mm_a = raw_params.get(molar_mass_a).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))).unwrap_or(1.0);
                let mm_b = raw_params.get(molar_mass_b).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))).unwrap_or(1.0);
                if mm_a.abs() < 1e-9 || coeff_a.abs() < 1e-9 {
                    return Err(ProceduralError::Validation("Divisors cannot be zero in stoichiometric mass-to-mass".to_string()));
                }
                let moles_a = m_a / mm_a;
                let moles_b = moles_a * (*coeff_b / *coeff_a);
                Ok(CanonicalAnswerValue::Numeric(moles_b * mm_b))
            }
            AnswerDerivation::EquilibriumKc { conc_products, conc_reactants } => {
                let mut prod_num = 1.0;
                for (p_name, exp) in conc_products {
                    let val = raw_params.get(p_name).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))).unwrap_or(1.0);
                    prod_num *= val.powf(*exp);
                }
                let mut react_den = 1.0;
                for (r_name, exp) in conc_reactants {
                    let val = raw_params.get(r_name).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))).unwrap_or(1.0);
                    react_den *= val.powf(*exp);
                }
                if react_den.abs() < 1e-9 {
                    return Err(ProceduralError::Validation("Reactants concentration denominator cannot be zero".to_string()));
                }
                Ok(CanonicalAnswerValue::Numeric(prod_num / react_den))
            }
            AnswerDerivation::IdealGasLawPressure { moles_param, temp_param, vol_param, r_const } => {
                let n = raw_params.get(moles_param).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))).unwrap_or(0.0);
                let t = raw_params.get(temp_param).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))).unwrap_or(273.0);
                let v = raw_params.get(vol_param).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))).unwrap_or(1.0);
                let r = r_const.unwrap_or(8.314);
                if v.abs() < 1e-9 {
                    return Err(ProceduralError::Validation("Volume cannot be zero in ideal gas law".to_string()));
                }
                Ok(CanonicalAnswerValue::Numeric((n * r * t) / v))
            }
            AnswerDerivation::IdealGasLawVolume { moles_param, temp_param, press_param, r_const } => {
                let n = raw_params.get(moles_param).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))).unwrap_or(0.0);
                let t = raw_params.get(temp_param).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))).unwrap_or(273.0);
                let p = raw_params.get(press_param).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))).unwrap_or(1.0);
                let r = r_const.unwrap_or(8.314);
                if p.abs() < 1e-9 {
                    return Err(ProceduralError::Validation("Pressure cannot be zero in ideal gas law".to_string()));
                }
                Ok(CanonicalAnswerValue::Numeric((n * r * t) / p))
            }
            AnswerDerivation::SymbolicLogicEvaluation { p_param, q_param, operator } => {
                let p = raw_params.get(p_param).and_then(|v| v.as_bool().or_else(|| v.as_i64().map(|i| i != 0))).unwrap_or(true);
                let q = raw_params.get(q_param).and_then(|v| v.as_bool().or_else(|| v.as_i64().map(|i| i != 0))).unwrap_or(true);
                let res = match operator.to_uppercase().as_str() {
                    "AND" | "&&" | "CONJUNCTION" => p && q,
                    "OR" | "||" | "DISJUNCTION" => p || q,
                    "IMPLIES" | "->" | "CONDITIONAL" => !p || q,
                    "EQUIV" | "<->" | "BICONDITIONAL" => p == q,
                    "XOR" => p ^ q,
                    "NOT_P" | "!P" => !p,
                    _ => p && q,
                };
                Ok(CanonicalAnswerValue::Text(if res { "True".to_string() } else { "False".to_string() }))
            }
        }
    }

    /// Build solution graph with interpolated descriptions and hints.
    fn build_solution_graph(
        archetype: &DeclarativeArchetype,
        str_params: &HashMap<String, String>,
    ) -> Option<SolutionGraph> {
        if archetype.step_nodes.is_empty() {
            return None;
        }

        let mut nodes: Vec<StepNode> = Vec::new();
        let mut previous_node_id: Option<String> = None;
        let mut final_id = String::new();

        let count = archetype.step_nodes.len();
        for (idx, spec) in archetype.step_nodes.iter().enumerate() {
            let description = Self::interpolate(&spec.description_template, str_params);
            let expected_expr = Self::interpolate(&spec.expected_expression_template, str_params);
            let alternates: Vec<String> = spec
                .alternate_templates
                .iter()
                .map(|alt| Self::interpolate(alt, str_params))
                .collect();

            let hint_p = Self::interpolate(&spec.hint_principle, str_params);
            let hint_o = Self::interpolate(&spec.hint_operation, str_params);
            let hint_i = Self::interpolate(&spec.hint_intermediate, str_params);

            let mut node = StepNode::new(
                &spec.id,
                spec.step_type,
                &spec.label,
                description,
                expected_expr,
            )
            .with_alternates(alternates)
            .with_hints(vec![
                StepHint::principle(hint_p),
                StepHint::operation(hint_o),
                StepHint::intermediate_relation(hint_i),
            ]);

            if let Some(prev_id) = &previous_node_id {
                node = node.with_dependencies(vec![prev_id.clone()]);
            }

            if idx == count - 1 {
                node = node.as_final();
                final_id = spec.id.clone();
            }

            previous_node_id = Some(spec.id.clone());
            nodes.push(node);
        }

        Some(SolutionGraph::new(nodes, final_id))
    }
}

impl ProblemGenerator for DeclarativeProblemGenerator {
    fn family_id(&self) -> &str {
        self.contract.contract.family_id.as_str()
    }

    fn template_ref(&self) -> &str {
        &self.template_ref
    }

    fn difficulty_range(&self) -> (u32, u32) {
        (
            self.contract.contract.min_difficulty as u32,
            self.contract.contract.max_difficulty as u32,
        )
    }

    fn supported_variants(&self) -> Vec<String> {
        self.contract
            .archetypes
            .iter()
            .map(|a| a.variant_name.clone())
            .collect()
    }

    fn target_latency_ms(&self, difficulty_level: u32) -> u64 {
        self.contract.contract.target_latency(difficulty_level)
    }

    fn generate(
        &self,
        family_id: &ProblemFamilyId,
        seed: u64,
        difficulty_level: u32,
        variant: Option<&str>,
    ) -> Result<ProblemInstance> {
        let archetype = self
            .contract
            .find_archetype(difficulty_level, variant)
            .ok_or_else(|| {
                ProceduralError::NotFound(format!(
                    "No declarative archetype found for family {} at level {} / variant {:?}",
                    family_id, difficulty_level, variant
                ))
            })?;

        let mut rng = StdRng::seed_from_u64(seed);
        let mut sample_attempt = 0;
        let mut raw_params = HashMap::new();
        let mut str_params = HashMap::new();

        // Retry loop for constraints satisfaction (deterministic with RNG advancement)
        while sample_attempt < 20 {
            let (raw, str_p) = Self::sample_parameters(archetype, &mut rng);
            if Self::check_constraints(archetype, &raw) {
                raw_params = raw;
                str_params = str_p;
                break;
            }
            sample_attempt += 1;
        }

        if raw_params.is_empty() {
            return Err(ProceduralError::Validation(format!(
                "Failed to generate parameters satisfying constraints for family {} (archetype {})",
                family_id, archetype.archetype_id
            )));
        }

        // Compute canonical answer
        let canonical_answer = Self::compute_answer(&archetype.answer_derivation, &raw_params)?;
        
        // Add answer to string parameters for template interpolation
        let formatted_val = canonical_answer.format_default();
        str_params.insert("answer".to_string(), formatted_val.clone());
        str_params.insert("target_val".to_string(), formatted_val.clone());

        // Derived extra parameters like (c - b), (a - c), (d - b) for step interpolations
        if let (Some(c), Some(b)) = (raw_params.get("c").and_then(|v| v.as_i64()), raw_params.get("b").and_then(|v| v.as_i64())) {
            str_params.insert("c_minus_b".to_string(), format!("{}", c - b));
        }
        if let (Some(a), Some(c)) = (raw_params.get("a").and_then(|v| v.as_i64()), raw_params.get("c").and_then(|v| v.as_i64())) {
            str_params.insert("a_minus_c".to_string(), format!("{}", a - c));
        }
        if let (Some(d), Some(b)) = (raw_params.get("d").and_then(|v| v.as_i64()), raw_params.get("b").and_then(|v| v.as_i64())) {
            str_params.insert("d_minus_b".to_string(), format!("{}", d - b));
        }

        // Interpolate prompt and solution
        let rendered_prompt = Self::interpolate(&archetype.prompt_template, &str_params);
        let solution_text = Self::interpolate(&archetype.solution_template, &str_params);
        let formatted_answer = Self::interpolate(&archetype.answer_formatted_template, &str_params);

        // Parameters JSON
        let mut params_obj = serde_json::Map::new();
        params_obj.insert("variant".to_string(), Value::String(archetype.variant_name.clone()));
        params_obj.insert("archetype_id".to_string(), Value::String(archetype.archetype_id.clone()));
        params_obj.insert("difficulty_level".to_string(), Value::from(archetype.difficulty_level));
        for (k, v) in &raw_params {
            params_obj.insert(k.clone(), v.clone());
        }

        // Correct answer JSON
        let correct_answer = serde_json::json!({
            "value": canonical_answer.to_json_value(),
            "formatted": if formatted_answer.is_empty() { formatted_val } else { formatted_answer },
            "solution": solution_text,
        });

        // Solution graph
        let mut instance = ProblemInstance::new(
            crate::core::ProblemInstanceId::new(format!("inst-dec-{}-{}", family_id.as_str().replace('.', "-"), seed)),
            family_id.clone(),
            seed,
            Value::Object(params_obj),
            rendered_prompt,
            correct_answer,
        );

        if let Some(graph) = Self::build_solution_graph(archetype, &str_params) {
            instance = instance.with_solution_graph(graph);
        }

        // Instance metadata
        let mut metadata_map = serde_json::Map::new();
        metadata_map.insert("target_time_ms".to_string(), Value::from(archetype.target_time_ms));
        metadata_map.insert("difficulty_level".to_string(), Value::from(archetype.difficulty_level));
        metadata_map.insert("variant".to_string(), Value::String(archetype.variant_name.clone()));
        metadata_map.insert("variant_category".to_string(), Value::String(archetype.variant_category.as_str().to_string()));
        metadata_map.insert("archetype_id".to_string(), Value::String(archetype.archetype_id.clone()));
        metadata_map.insert("is_declarative".to_string(), Value::Bool(true));
        metadata_map.insert("contract_version".to_string(), Value::from(1));

        if let Some(ot) = &archetype.object_type {
            metadata_map.insert("object_type".to_string(), Value::String(ot.clone()));
        }
        if let Some(arch_meta) = &archetype.metadata {
            if let Some(obj) = arch_meta.as_object() {
                for (k, v) in obj {
                    metadata_map.insert(k.clone(), v.clone());
                }
            }
        }

        instance = instance.with_metadata(Value::Object(metadata_map));

        Ok(instance)
    }
}

// ---------------------------------------------------------------------------
// Proof of Concept: Linear Equations Declarative Contract
// ---------------------------------------------------------------------------

pub fn linear_equations_declarative_contract() -> DeclarativeFamilyContract {
    let contract = ProblemFamilyContract::new(
        "family.math.algebra.linear_equations",
        "algebra.linear_equations",
        Domain::Mathematics,
        "schema.algebra.linear_equations.v1",
        ProblemFamilyCapability::Declarative,
    )
    .with_difficulty_range(1.0, 5.0)
    .with_variants(vec![
        "two_step_basic",
        "variables_both_sides",
        "distributive",
        "fractional_coefficients",
        "word_problem",
    ])
    .with_variant_categories(vec![
        VariantCategory::Parameter,
        VariantCategory::Isomorphic,
        VariantCategory::Structural,
        VariantCategory::Structural,
        VariantCategory::Contextual,
    ])
    .with_target_latency(1, 25_000)
    .with_target_latency(2, 35_000)
    .with_target_latency(3, 50_000)
    .with_target_latency(4, 65_000)
    .with_target_latency(5, 60_000)
    .with_structural_tags(vec!["algebra", "linear", "two_step", "balance_method"])
    .with_decision_points(vec!["isolate_variable_term", "inverse_multiplication", "collect_like_terms"])
    .with_error_categories(vec!["sign_flip_omitted", "divide_subtraction_order", "arithmetic_careless"])
    .with_prerequisites(vec!["arithmetic.basic_integers", "algebra.variables"]);

    let mut archetypes = Vec::new();

    // Archetype 1 (Level 1): Two-step Basic ax + b = c
    let arch_1 = DeclarativeArchetype::new(
        "linear_eq.two_step_basic",
        1,
        VariantCategory::Parameter,
        "two_step_basic",
        vec![
            ParameterSpec::integer_range("a", 2, 9),
            ParameterSpec::integer_range("x", -12, 12),
            ParameterSpec::integer_range("b", -20, 20),
            ParameterSpec::derived_linear("c", "a", "x", "b"),
            ParameterSpec::derived_signed("b_sign", "b"),
        ],
        "Solve for \\(x\\):\n\n\\[ {a}x {b_sign} = {c} \\]",
        AnswerDerivation::LinearTwoStep {
            c_param: "c".to_string(),
            b_param: "b".to_string(),
            a_param: "a".to_string(),
        },
        "{answer}",
        "**Step 1:** Subtract {b} from both sides:\n\\[ {a}x = {c} - ({b}) = {c_minus_b} \\]\n\n**Step 2:** Divide both sides by {a}:\n\\[ x = \\frac{{{c_minus_b}}}{{{a}}} = **{answer}** \\]",
        25_000,
    )
    .with_step_nodes(vec![
        StepNodeSpec::new(
            "isolate_var",
            StepType::EquationRearrangement,
            "Isolate variable term",
            "Subtract {b} from both sides",
            "{a}x = {c_minus_b}",
            vec!["{a}x = {c} - ({b})".to_string(), "{a}x = {c} - {b}".to_string()],
            "To isolate the variable term, perform the inverse arithmetic operation on both sides.",
            "Subtract {b} from both sides of the equation.",
            "{a}x = {c} - ({b}) = {c_minus_b}",
        ),
        StepNodeSpec::new(
            "solve_x",
            StepType::Simplification,
            "Solve for x",
            "Divide both sides by {a}",
            "x = {answer}",
            vec!["x = {c_minus_b} / {a}".to_string(), "{answer}".to_string()],
            "Divide by the coefficient of x to solve for the unknown.",
            "Divide both sides by {a}.",
            "x = {c_minus_b} / {a} = {answer}",
        ),
    ]);
    archetypes.push(arch_1);

    // Archetype 2 (Level 2): Variables on Both Sides ax + b = cx + d
    let arch_2 = DeclarativeArchetype::new(
        "linear_eq.variables_both_sides",
        2,
        VariantCategory::Isomorphic,
        "variables_both_sides",
        vec![
            ParameterSpec::integer_range("a", 3, 10),
            ParameterSpec::integer_range("c", 1, 9),
            ParameterSpec::integer_range("x", -10, 10),
            ParameterSpec::integer_range("b", -15, 15),
            ParameterSpec::derived_signed("b_sign", "b"),
            // d = (a - c)*x + b
            ParameterSpec::derived_difference("a_minus_c_raw", "a", "c"),
            ParameterSpec::derived_linear("d", "a_minus_c_raw", "x", "b"),
            ParameterSpec::derived_signed("d_sign", "d"),
        ],
        "Solve for \\(x\\):\n\n\\[ {a}x {b_sign} = {c}x {d_sign} \\]",
        AnswerDerivation::LinearVariablesBothSides {
            d_param: "d".to_string(),
            b_param: "b".to_string(),
            a_param: "a".to_string(),
            c_param: "c".to_string(),
        },
        "{answer}",
        "**Step 1:** Collect variable terms on the left: subtract {c}x from both sides:\n\\[ ({a} - {c})x {b_sign} = {d} \\implies {a_minus_c}x {b_sign} = {d} \\]\n\n**Step 2:** Subtract {b} from both sides:\n\\[ {a_minus_c}x = {d} - ({b}) = {d_minus_b} \\]\n\n**Step 3:** Divide by {a_minus_c}:\n\\[ x = \\frac{{{d_minus_b}}}{{{a_minus_c}}} = **{answer}** \\]",
        35_000,
    )
    .with_constraints(vec![
        ConstraintSpec::NotEqual {
            param_a: "a".to_string(),
            param_b: "c".to_string(),
        },
    ])
    .with_step_nodes(vec![
        StepNodeSpec::new(
            "collect_vars",
            StepType::EquationRearrangement,
            "Collect variable terms",
            "Subtract {c}x from both sides",
            "{a_minus_c}x {b_sign} = {d}",
            vec!["{a}x - {c}x {b_sign} = {d}".to_string()],
            "Subtract the smaller variable term from both sides to gather terms on one side.",
            "Subtract {c}x from both sides.",
            "{a_minus_c}x {b_sign} = {d}",
        ),
        StepNodeSpec::new(
            "isolate_term",
            StepType::EquationRearrangement,
            "Isolate variable term",
            "Subtract {b} from both sides",
            "{a_minus_c}x = {d_minus_b}",
            vec!["{a_minus_c}x = {d} - ({b})".to_string()],
            "Isolate the variable term on the left side.",
            "Subtract {b} from both sides.",
            "{a_minus_c}x = {d_minus_b}",
        ),
        StepNodeSpec::new(
            "solve_x",
            StepType::Simplification,
            "Solve for x",
            "Divide both sides by {a_minus_c}",
            "x = {answer}",
            vec!["x = {d_minus_b} / {a_minus_c}".to_string()],
            "Divide by the coefficient of x.",
            "Divide both sides by {a_minus_c}.",
            "x = {answer}",
        ),
    ]);
    archetypes.push(arch_2);

    // Archetype 3 (Level 3): Distributive Property a(bx + c) = d
    let arch_3 = DeclarativeArchetype::new(
        "linear_eq.distributive",
        3,
        VariantCategory::Structural,
        "distributive",
        vec![
            ParameterSpec::integer_range("a", 2, 6),
            ParameterSpec::integer_range("b", 1, 4),
            ParameterSpec::integer_range("x", -8, 8),
            ParameterSpec::integer_range("c", -10, 10),
            ParameterSpec::derived_signed("c_sign", "c"),
            // inner = b*x + c
            ParameterSpec::derived_linear("inner", "b", "x", "c"),
            // d = a * inner
            ParameterSpec::derived_product("d", "a", "inner"),
        ],
        "Solve for \\(x\\):\n\n\\[ {a}({b}x {c_sign}) = {d} \\]",
        AnswerDerivation::LinearDistributive {
            d_param: "d".to_string(),
            a_param: "a".to_string(),
            c_param: "c".to_string(),
            b_param: "b".to_string(),
        },
        "{answer}",
        "**Step 1:** Expand parenthesis or divide both sides by {a}:\n\\[ {b}x {c_sign} = \\frac{{{d}}}{{{a}}} = {inner} \\]\n\n**Step 2:** Subtract {c} from both sides:\n\\[ {b}x = {inner} - ({c}) \\]\n\n**Step 3:** Divide by {b}:\n\\[ x = **{answer}** \\]",
        50_000,
    )
    .with_step_nodes(vec![
        StepNodeSpec::new(
            "expand_or_divide",
            StepType::EquationRearrangement,
            "Divide by outer factor",
            "Divide both sides by {a}",
            "{b}x {c_sign} = {inner}",
            vec!["{b}x + {c} = {inner}".to_string()],
            "Divide both sides by the outer multiplier to simplify the bracketed expression.",
            "Divide both sides by {a}.",
            "{b}x {c_sign} = {inner}",
        ),
        StepNodeSpec::new(
            "solve_x",
            StepType::Simplification,
            "Solve for x",
            "Isolate and divide by {b}",
            "x = {answer}",
            vec!["{answer}".to_string()],
            "Subtract the constant term and divide by the coefficient of x.",
            "Subtract {c} and divide by {b}.",
            "x = {answer}",
        ),
    ]);
    archetypes.push(arch_3);

    // Archetype 4 (Level 4): Fractional Coefficients x/a + b = c
    let arch_4 = DeclarativeArchetype::new(
        "linear_eq.fractional_coefficients",
        4,
        VariantCategory::Structural,
        "fractional_coefficients",
        vec![
            ParameterSpec::integer_range("a", 2, 8),
            ParameterSpec::integer_range("x_quotient", -10, 10),
            ParameterSpec::integer_range("b", -15, 15),
            ParameterSpec::derived_signed("b_sign", "b"),
            // x = a * x_quotient
            ParameterSpec::derived_product("x", "a", "x_quotient"),
            // c = x_quotient + b
            ParameterSpec::derived_sum("c", "x_quotient", "b"),
        ],
        "Solve for \\(x\\):\n\n\\[ \\frac{{x}}{{{a}}} {b_sign} = {c} \\]",
        AnswerDerivation::LinearFractional {
            c_param: "c".to_string(),
            b_param: "b".to_string(),
            a_param: "a".to_string(),
        },
        "{answer}",
        "**Step 1:** Subtract {b} from both sides:\n\\[ \\frac{{x}}{{{a}}} = {c} - ({b}) = {x_quotient} \\]\n\n**Step 2:** Multiply both sides by {a}:\n\\[ x = {x_quotient} \\times {a} = **{answer}** \\]",
        65_000,
    )
    .with_step_nodes(vec![
        StepNodeSpec::new(
            "isolate_fraction",
            StepType::EquationRearrangement,
            "Isolate fraction term",
            "Subtract {b} from both sides",
            "x/{a} = {x_quotient}",
            vec!["\\frac{{x}}{{{a}}} = {x_quotient}".to_string()],
            "Isolate the fraction term first before clearing the denominator.",
            "Subtract {b} from both sides.",
            "x/{a} = {x_quotient}",
        ),
        StepNodeSpec::new(
            "clear_denominator",
            StepType::Simplification,
            "Clear denominator",
            "Multiply both sides by {a}",
            "x = {answer}",
            vec!["x = {x_quotient} * {a}".to_string()],
            "Multiply by the denominator to solve for x.",
            "Multiply both sides by {a}.",
            "x = {answer}",
        ),
    ]);
    archetypes.push(arch_4);

    // Archetype 5 (Level 5): Contextual Word Problem
    let arch_5 = DeclarativeArchetype::new(
        "linear_eq.word_problem",
        5,
        VariantCategory::Contextual,
        "word_problem",
        vec![
            ParameterSpec::integer_range("rate", 12, 45),
            ParameterSpec::integer_range("base_fee", 20, 80),
            ParameterSpec::integer_range("hours", 3, 14),
            ParameterSpec::derived_linear("total_cost", "rate", "hours", "base_fee"),
        ],
        "A technician charges a flat diagnostic fee of **${base_fee}** plus **${rate} per hour** for repairs.\n\nIf the total bill was **${total_cost}**, how many hours of labor were billed?",
        AnswerDerivation::LinearTwoStep {
            c_param: "total_cost".to_string(),
            b_param: "base_fee".to_string(),
            a_param: "rate".to_string(),
        },
        "{answer}",
        "**Step 1:** Formulate equation:\n\\[ {rate}h + {base_fee} = {total_cost} \\]\n\n**Step 2:** Subtract base fee:\n\\[ {rate}h = {total_cost} - {base_fee} = {c_minus_b} \\]\n\n**Step 3:** Divide by hourly rate:\n\\[ h = \\frac{{{c_minus_b}}}{{{rate}}} = **{answer} hours** \\]",
        60_000,
    )
    .with_step_nodes(vec![
        StepNodeSpec::new(
            "formulate_eq",
            StepType::EquationRearrangement,
            "Formulate linear equation",
            "Set up rate equation",
            "{rate}h + {base_fee} = {total_cost}",
            vec!["{rate}x + {base_fee} = {total_cost}".to_string()],
            "Translate the word problem into a standard linear equation: Total = (Rate * hours) + Base.",
            "Set up {rate}h + {base_fee} = {total_cost}.",
            "{rate}h + {base_fee} = {total_cost}",
        ),
        StepNodeSpec::new(
            "solve_unknown",
            StepType::Simplification,
            "Solve for labor hours",
            "Solve for h",
            "h = {answer}",
            vec!["h = {answer} hours".to_string(), "{answer}".to_string()],
            "Solve the linear equation for the number of hours.",
            "Subtract base fee and divide by rate.",
            "h = {answer}",
        ),
    ]);
    archetypes.push(arch_5);

    DeclarativeFamilyContract::new(contract, archetypes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_equations_declarative_generation_all_levels() {
        let contract = Arc::new(linear_equations_declarative_contract());
        let generator = DeclarativeProblemGenerator::new(contract.clone());
        let family_id = ProblemFamilyId::new("family.math.algebra.linear_equations");

        for level in 1..=5 {
            for seed in 1..=10 {
                let inst = generator.generate(&family_id, seed, level, None).unwrap();
                assert!(!inst.rendered_prompt.is_empty(), "Prompt should not be empty at level {}", level);
                assert!(inst.correct_answer.get("value").is_some(), "Value must exist at level {}", level);
                assert!(inst.solution_graph().is_some(), "Solution graph should be present at level {}", level);

                let meta = &inst.metadata;
                assert_eq!(meta["difficulty_level"].as_u64().unwrap(), level as u64);
                assert_eq!(meta["is_declarative"].as_bool().unwrap(), true);
            }
        }
    }
}
