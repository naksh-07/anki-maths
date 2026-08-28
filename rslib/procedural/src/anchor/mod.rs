// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use serde::{Deserialize, Serialize};

use crate::core::{ProceduralError, Result, SchemaId};
use crate::problems::contract::DeclarativeFamilyContract;

pub mod source;

/// Strategy for generating random seeds for ephemeral problem instances.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SeedMode {
    #[default]
    Random,
    Fixed(u64),
    Daily,
}

impl SeedMode {
    pub fn is_default(&self) -> bool {
        matches!(self, SeedMode::Random)
    }
}

/// Minimal reference connecting an Anki Card to a procedural learning object schema.
/// This acts as the safe bridge metadata anchor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProceduralCardAnchor {
    /// Schema ID of the target practice object (e.g., "math.algebra.monic_quadratic")
    pub proc_schema: SchemaId,

    /// Optional reference to the static source content (e.g., StudyLab PracticeItem ID)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_ref: Option<String>,

    /// Optional fixed difficulty override
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub difficulty_override: Option<f64>,

    /// Strategy for problem instance generation seed
    #[serde(default, skip_serializing_if = "SeedMode::is_default")]
    pub seed_mode: SeedMode,

    /// Optional configuration overrides specific to this card
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub custom_params: serde_json::Value,

    /// Modern rich-content path: optional inline declarative contract bundled directly in the anchor
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inline_contract: Option<DeclarativeFamilyContract>,
}

impl ProceduralCardAnchor {
    pub fn new(proc_schema: impl Into<SchemaId>) -> Self {
        Self {
            proc_schema: proc_schema.into(),
            content_ref: None,
            difficulty_override: None,
            seed_mode: SeedMode::Random,
            custom_params: serde_json::Value::Null,
            inline_contract: None,
        }
    }

    pub fn with_difficulty_override(mut self, diff: f64) -> Self {
        self.difficulty_override = Some(diff);
        self
    }

    pub fn with_seed_mode(mut self, mode: SeedMode) -> Self {
        self.seed_mode = mode;
        self
    }

    pub fn with_inline_contract(mut self, contract: DeclarativeFamilyContract) -> Self {
        self.inline_contract = Some(contract);
        self
    }

    pub fn to_json_string(&self) -> Result<String> {
        serde_json::to_string(self).map_err(ProceduralError::from)
    }

    /// Parse a JSON string that may represent a procedural anchor.
    /// Strictly parses valid JSON anchors; malformed anchors log diagnostics and return Ok(None)
    /// to guarantee safe fallback to standard Anki card review without crashing the reviewer.
    pub fn from_json_str(s: &str) -> Result<Option<Self>> {
        let trimmed = s.trim();
        if !trimmed.starts_with('{')
            || (!trimmed.contains("proc_schema")
                && !trimmed.contains("content_ref")
                && !trimmed.contains("inline_contract"))
        {
            return Ok(None);
        }

        match serde_json::from_str::<ProceduralCardAnchor>(trimmed) {
            Ok(anchor) => Ok(Some(anchor)),
            Err(e) => {
                eprintln!("[procedural] Malformed procedural card anchor metadata: {e}");
                Ok(None)
            }
        }
    }

    /// Strict parser for programmatic validation where explicit error reporting is required.
    pub fn from_json_str_strict(s: &str) -> Result<Option<Self>> {
        let trimmed = s.trim();
        if !trimmed.starts_with('{')
            || (!trimmed.contains("proc_schema")
                && !trimmed.contains("content_ref")
                && !trimmed.contains("inline_contract"))
        {
            return Ok(None);
        }

        match serde_json::from_str::<ProceduralCardAnchor>(trimmed) {
            Ok(anchor) => Ok(Some(anchor)),
            Err(e) => Err(ProceduralError::InvalidAnchor(e.to_string())),
        }
    }

    /// Attempt to extract a procedural anchor from a list of note/card field contents.
    pub fn extract_from_card_fields(fields: &[String]) -> Result<Option<Self>> {
        for field in fields {
            if let Some(anchor) = Self::from_json_str(field)? {
                return Ok(Some(anchor));
            }
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anchor_json_roundtrip() {
        let anchor = ProceduralCardAnchor::new("math.calc.derivatives.power_rule")
            .with_difficulty_override(2.5);

        let json = anchor.to_json_string().unwrap();
        assert!(json.contains("math.calc.derivatives.power_rule"));

        let parsed = ProceduralCardAnchor::from_json_str(&json).unwrap().unwrap();
        assert_eq!(parsed.proc_schema.as_str(), "math.calc.derivatives.power_rule");
        assert_eq!(parsed.difficulty_override, Some(2.5));
    }

    #[test]
    fn test_extract_from_universe_175_note() {
        let fld = r#"{"proc_schema": "schema.math.number_system.lcm_hcf.v1", "seed_mode": {"fixed": 42}, "difficulty_override": 1.0, "inline_contract": {"contract": {"family_id": "family.math.number_system.lcm_hcf", "skill_id": "math.number_system.lcm_hcf", "domain": "mathematics", "default_schema": "schema.math.number_system.lcm_hcf.v1", "capability": "declarative", "min_difficulty": 1.0, "max_difficulty": 5.0, "supported_variants": ["lcm_two_numbers", "hcf_two_numbers"], "variant_categories": ["parameter", "structural"], "target_latency_model": {"1": 25000, "2": 35000, "3": 45000, "4": 60000, "5": 75000}, "structural_tags": ["number_system", "arithmetic", "factors"], "decision_points": ["prime_factorization", "division_method"], "error_categories": ["common_factor_omission", "arithmetic_slip"], "prerequisites": [], "provenance": {"source": "PYQ Corpus", "exam": "RRB ALP", "year": 2024, "shift": 1}, "metadata": {"title": "LCM and HCF", "category": "Number System"}}, "archetypes": [{"archetype_id": "math.ns.lcm_two_num", "difficulty_level": 1, "variant_category": "parameter", "variant_name": "lcm_two_numbers", "object_type": "problem", "parameters": [{"name": "num1", "domain": {"type": "integer_range", "min": 6, "max": 24, "step": null, "non_zero": null}}, {"name": "num2", "domain": {"type": "integer_range", "min": 8, "max": 36, "step": null, "non_zero": null}}], "constraints": [], "prompt_template": "Find the Least Common Multiple (LCM) of \\({num1}\\) and \\({num2}\\).", "answer_derivation": {"type": "lcm_array", "params": ["num1", "num2"]}, "answer_formatted_template": "{answer}", "solution_template": "Prime factorize both numbers: {num1} and {num2}. Take highest power of each prime factor. LCM = {answer}.", "step_nodes": [{"id": "step_factorize", "step_type": "arithmetic", "label": "Prime Factorization", "description_template": "Factorize {num1} and {num2}", "expected_expression_template": "LCM({num1}, {num2}) = {answer}", "alternate_templates": [], "hint_principle": "Prime factorization reveals the base components of both numbers.", "hint_operation": "Write each number as a product of prime powers.", "hint_intermediate": "Examine the common and distinct prime factors."}], "target_time_ms": 25000}]}}"#;
        let res = ProceduralCardAnchor::from_json_str(fld);
        println!("Result: {:?}", res);
        assert!(res.is_ok());
        let opt = res.unwrap();
        assert!(opt.is_some(), "Anchor should be parsed successfully!");
    }
}

