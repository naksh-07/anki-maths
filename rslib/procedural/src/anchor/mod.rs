// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use serde::{Deserialize, Serialize};

use crate::core::{ProceduralError, Result, SchemaId};

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

    /// Optional fixed difficulty override
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub difficulty_override: Option<f64>,

    /// Strategy for problem instance generation seed
    #[serde(default, skip_serializing_if = "SeedMode::is_default")]
    pub seed_mode: SeedMode,

    /// Optional configuration overrides specific to this card
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub custom_params: serde_json::Value,
}

impl ProceduralCardAnchor {
    pub fn new(proc_schema: impl Into<SchemaId>) -> Self {
        Self {
            proc_schema: proc_schema.into(),
            difficulty_override: None,
            seed_mode: SeedMode::Random,
            custom_params: serde_json::Value::Null,
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

    pub fn to_json_string(&self) -> Result<String> {
        serde_json::to_string(self).map_err(ProceduralError::from)
    }

    /// Parse a JSON string that may represent a procedural anchor.
    /// Strictly parses valid JSON anchors; malformed anchors log diagnostics and return Ok(None)
    /// to guarantee safe fallback to standard Anki card review without crashing the reviewer.
    pub fn from_json_str(s: &str) -> Result<Option<Self>> {
        let trimmed = s.trim();
        if !trimmed.starts_with('{') || !trimmed.contains("proc_schema") {
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
        if !trimmed.starts_with('{') || !trimmed.contains("proc_schema") {
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
    fn test_extract_from_card_fields() {
        let fields = vec![
            "Front Question".to_string(),
            r#"{"proc_schema":"physics.kinematics.freefall"}"#.to_string(),
            "Back Answer".to_string(),
        ];

        let extracted = ProceduralCardAnchor::extract_from_card_fields(&fields).unwrap();
        assert!(extracted.is_some());
        assert_eq!(
            extracted.unwrap().proc_schema.as_str(),
            "physics.kinematics.freefall"
        );
    }
}
