// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::core::{
    Domain, ProblemFamilyId, ProceduralError, PyqId, RejectedVariantId, SchemaId,
};
use crate::exam::pyq::{
    ContentProvenance, PYQSource, PyqMapping, DEFAULT_CATALOG_VERSION, DEFAULT_GENERATOR_VERSION,
    DEFAULT_SCHEMA_VERSION,
};
use crate::problems::registry::ProblemRegistry;
use crate::problems::ProblemInstance;

/// Record of a generated problem variant that failed the domain validation gate.
/// Persisted to disk/database rather than silently discarded to enable generator debugging.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RejectedVariantRecord {
    pub id: RejectedVariantId,
    pub source_pyq_id: Option<PyqId>,
    pub schema_id: SchemaId,
    pub family_id: ProblemFamilyId,
    pub seed: u64,
    pub variant_type: String,
    pub failure_reason: String,
    pub generated_instance_json: serde_json::Value,
    pub rejected_at: i64,
}

impl RejectedVariantRecord {
    pub fn new(
        source_pyq_id: Option<PyqId>,
        schema_id: SchemaId,
        family_id: ProblemFamilyId,
        seed: u64,
        variant_type: impl Into<String>,
        failure_reason: impl Into<String>,
        generated_instance: &ProblemInstance,
    ) -> Self {
        let instance_json = serde_json::to_value(generated_instance)
            .unwrap_or(serde_json::Value::String("unserializable".into()));

        Self {
            id: RejectedVariantId::new(format!("rej_{}_{}_{}", family_id.as_str(), seed, Utc::now().timestamp_millis())),
            source_pyq_id,
            schema_id,
            family_id,
            seed,
            variant_type: variant_type.into(),
            failure_reason: failure_reason.into(),
            generated_instance_json: instance_json,
            rejected_at: Utc::now().timestamp(),
        }
    }
}

/// Domain-specific variant taxonomy and validation engine.
pub struct PyqVariantPipeline;

impl PyqVariantPipeline {
    /// Verify that the requested variant class is supported by the domain and family.
    pub fn is_variant_supported_for_domain(domain: &Domain, variant: &str) -> bool {
        let v = variant.to_lowercase();
        match domain {
            Domain::Mathematics => matches!(
                v.as_str(),
                "standard"
                    | "isomorphic"
                    | "numerical"
                    | "structural"
                    | "reverse"
                    | "trap"
                    | "boundary_trap"
                    | "transfer"
            ),
            Domain::Physics => matches!(
                v.as_str(),
                "standard"
                    | "parameter"
                    | "initial_condition"
                    | "representation"
                    | "model_selection"
                    | "transfer"
            ),
            Domain::Chemistry => matches!(
                v.as_str(),
                "standard"
                    | "quantity"
                    | "species"
                    | "regime"
                    | "constraint"
                    | "transfer"
            ),
            Domain::Reasoning => matches!(
                v.as_str(),
                "standard"
                    | "entity"
                    | "constraint"
                    | "strategy"
                    | "structural"
                    | "transfer"
            ),
            Domain::Custom(_) => true,
        }
    }

    /// Generate a deterministic, domain-validated variant instance derived from a PYQ and its mapping.
    pub fn generate_and_validate_variant(
        registry: &ProblemRegistry,
        pyq: Option<&PYQSource>,
        mapping: &PyqMapping,
        seed: u64,
        variant: Option<&str>,
    ) -> std::result::Result<ProblemInstance, (ProceduralError, Option<RejectedVariantRecord>)> {
        let requested_variant = variant
            .or_else(|| mapping.variant_structure.as_deref())
            .unwrap_or("standard");

        // 1. Enforce domain variant taxonomy
        if !Self::is_variant_supported_for_domain(&mapping.domain, requested_variant) {
            let err = ProceduralError::Validation(format!(
                "Variant '{}' is not a supported variant class for domain '{:?}'",
                requested_variant, mapping.domain
            ));
            return Err((err, None));
        }

        // 2. Resolve generator from ProblemRegistry
        let generator = registry
            .get_generator(mapping.problem_family_id.as_str())
            .ok_or_else(|| {
                (
                    ProceduralError::NotFound(format!(
                        "Generator not found for problem family '{}'",
                        mapping.problem_family_id
                    )),
                    None,
                )
            })?;

        // 3. Generate raw problem instance deterministically
        let mut instance = generator
            .generate(
                &mapping.problem_family_id,
                seed,
                mapping.difficulty_level,
                Some(requested_variant),
            )
            .map_err(|e| (e, None))?;

        // 4. Attach complete content provenance
        let provenance = if let Some(p) = pyq {
            ContentProvenance::new_pyq_derived(
                p.id.clone(),
                p.source_version,
                DEFAULT_GENERATOR_VERSION,
                DEFAULT_SCHEMA_VERSION,
                DEFAULT_CATALOG_VERSION,
                requested_variant,
                Some(seed),
            )
        } else {
            ContentProvenance::new_direct_procedural(
                DEFAULT_GENERATOR_VERSION,
                DEFAULT_SCHEMA_VERSION,
                DEFAULT_CATALOG_VERSION,
                requested_variant,
                seed,
            )
        };

        if let Ok(prov_val) = serde_json::to_value(&provenance) {
            if let Some(obj) = instance.metadata.as_object_mut() {
                obj.insert("provenance".to_string(), prov_val);
            }
        }

        // 5. Domain Validation Gate
        if let Some(validator) = registry.get_validator(mapping.problem_family_id.as_str()) {
            let eval = validator.evaluate(&instance, &instance.correct_answer, 30_000, 30_000);
            if !eval.is_correct {
                let failure_reason = format!(
                    "Validator self-consistency check failed on canonical solution: {}",
                    eval.diagnostic_message.unwrap_or_else(|| "unspecified error".into())
                );
                let rejected_record = RejectedVariantRecord::new(
                    pyq.map(|p| p.id.clone()),
                    mapping.schema_id.clone(),
                    mapping.problem_family_id.clone(),
                    seed,
                    requested_variant,
                    failure_reason.clone(),
                    &instance,
                );
                return Err((ProceduralError::Validation(failure_reason), Some(rejected_record)));
            }
        }

        Ok(instance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_variant_taxonomy_enforcement() {
        assert!(PyqVariantPipeline::is_variant_supported_for_domain(
            &Domain::Mathematics,
            "structural"
        ));
        assert!(PyqVariantPipeline::is_variant_supported_for_domain(
            &Domain::Physics,
            "initial_condition"
        ));
        assert!(PyqVariantPipeline::is_variant_supported_for_domain(
            &Domain::Chemistry,
            "species"
        ));
        assert!(PyqVariantPipeline::is_variant_supported_for_domain(
            &Domain::Reasoning,
            "strategy"
        ));

        // Invalid variant across domains
        assert!(!PyqVariantPipeline::is_variant_supported_for_domain(
            &Domain::Mathematics,
            "chemical_species"
        ));
        assert!(!PyqVariantPipeline::is_variant_supported_for_domain(
            &Domain::Reasoning,
            "orbital_quantum"
        ));
    }

    #[test]
    fn test_pipeline_generates_and_validates_maths_variant() {
        let registry = ProblemRegistry::default_maths_registry();
        let mapping = PyqMapping::new(
            "pyq.test.001",
            Domain::Mathematics,
            "percentage.successive",
            "schema.math.percentage.successive",
            "family.math.percentage.successive",
            2,
            35_000,
        )
        .with_variant_structure("reverse");

        let result = PyqVariantPipeline::generate_and_validate_variant(
            &registry,
            None,
            &mapping,
            12345,
            Some("reverse"),
        );

        assert!(result.is_ok());
        let instance = result.unwrap();
        assert_eq!(instance.seed, 12345);
        assert!(instance.rendered_prompt.len() > 10);

        let prov: ContentProvenance = serde_json::from_value(
            instance.metadata.get("provenance").unwrap().clone(),
        )
        .unwrap();
        assert_eq!(prov.seed, Some(12345));
        assert_eq!(prov.variant_type, "reverse");
    }
}
