// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::core::{Domain, ProblemFamilyId, PyqId, SchemaId, SkillId};

/// Current catalog and schema version constants for provenance tracking.
pub const DEFAULT_PYQ_SOURCE_VERSION: u32 = 1;
pub const DEFAULT_GENERATOR_VERSION: u32 = 1;
pub const DEFAULT_SCHEMA_VERSION: u32 = 1;
pub const DEFAULT_CATALOG_VERSION: u32 = 1;

/// Complete, immutable provenance record retained on every problem instance,
/// practice attempt, or derivative practice object originated from a PYQ or schema.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentProvenance {
    /// Associated source PYQ identifier if this content derived from an authentic PYQ
    pub source_pyq_id: Option<PyqId>,
    /// Incremental version of the underlying PYQ source document
    pub source_version: u32,
    /// Procedural engine generator implementation version
    pub generator_version: u32,
    /// Schema definition version
    pub schema_version: u32,
    /// Catalog registry release version
    pub catalog_version: u32,
    /// Specific variant class / transformation applied
    pub variant_type: String,
    /// Deterministic RNG seed used to generate parameters and content
    pub seed: Option<u64>,
}

impl ContentProvenance {
    pub fn new_direct_procedural(
        generator_version: u32,
        schema_version: u32,
        catalog_version: u32,
        variant_type: impl Into<String>,
        seed: u64,
    ) -> Self {
        Self {
            source_pyq_id: None,
            source_version: 0,
            generator_version,
            schema_version,
            catalog_version,
            variant_type: variant_type.into(),
            seed: Some(seed),
        }
    }

    pub fn new_pyq_derived(
        pyq_id: PyqId,
        source_version: u32,
        generator_version: u32,
        schema_version: u32,
        catalog_version: u32,
        variant_type: impl Into<String>,
        seed: Option<u64>,
    ) -> Self {
        Self {
            source_pyq_id: Some(pyq_id),
            source_version,
            generator_version,
            schema_version,
            catalog_version,
            variant_type: variant_type.into(),
            seed,
        }
    }
}

impl Default for ContentProvenance {
    fn default() -> Self {
        Self {
            source_pyq_id: None,
            source_version: DEFAULT_PYQ_SOURCE_VERSION,
            generator_version: DEFAULT_GENERATOR_VERSION,
            schema_version: DEFAULT_SCHEMA_VERSION,
            catalog_version: DEFAULT_CATALOG_VERSION,
            variant_type: "standard".to_string(),
            seed: None,
        }
    }
}

/// Immutable, first-class source Previous Year Question (PYQ) representation.
/// The original PYQ is an immutable reference object and is never silently modified
/// by procedural generators.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PYQSource {
    pub id: PyqId,
    pub exam: String,
    pub year: u32,
    pub paper: Option<String>,
    pub shift: Option<String>,
    pub session: Option<String>,
    pub domain: Domain,
    /// Verbatim original question prompt
    pub original_question: String,
    /// Original multiple-choice options if applicable
    pub original_options: Option<Vec<String>>,
    /// Original official answer key
    pub original_answer: serde_json::Value,
    /// Source citation or reference (e.g. "RRB ALP 2018 Stage 1 Shift 2")
    pub source_reference: String,
    /// Full provenance tracking
    pub provenance: ContentProvenance,
    pub source_version: u32,
    pub import_timestamp: i64,
    pub metadata: serde_json::Value,
}

impl PYQSource {
    pub fn new(
        id: impl Into<PyqId>,
        exam: impl Into<String>,
        year: u32,
        domain: Domain,
        original_question: impl Into<String>,
        original_answer: serde_json::Value,
        source_reference: impl Into<String>,
    ) -> Self {
        let pyq_id = id.into();
        let prov = ContentProvenance::new_pyq_derived(
            pyq_id.clone(),
            DEFAULT_PYQ_SOURCE_VERSION,
            DEFAULT_GENERATOR_VERSION,
            DEFAULT_SCHEMA_VERSION,
            DEFAULT_CATALOG_VERSION,
            "authentic_pyq",
            None,
        );

        Self {
            id: pyq_id,
            exam: exam.into(),
            year,
            paper: None,
            shift: None,
            session: None,
            domain,
            original_question: original_question.into(),
            original_options: None,
            original_answer,
            source_reference: source_reference.into(),
            provenance: prov,
            source_version: DEFAULT_PYQ_SOURCE_VERSION,
            import_timestamp: Utc::now().timestamp(),
            metadata: serde_json::Value::Object(Default::default()),
        }
    }

    pub fn with_shift_info(
        mut self,
        paper: Option<impl Into<String>>,
        shift: Option<impl Into<String>>,
        session: Option<impl Into<String>>,
    ) -> Self {
        self.paper = paper.map(|s| s.into());
        self.shift = shift.map(|s| s.into());
        self.session = session.map(|s| s.into());
        self
    }

    pub fn with_options(mut self, options: Vec<String>) -> Self {
        self.original_options = Some(options);
        self
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn with_source_version(mut self, version: u32) -> Self {
        self.source_version = version;
        self.provenance.source_version = version;
        self
    }
}

/// Review lifecycle status of a PYQ-to-Schema mapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MappingStatus {
    /// Newly imported, awaiting schema alignment
    Unreviewed,
    /// Schema mapping proposed or deterministically assigned
    Mapped,
    /// Human-verified or benchmark-validated mapping
    Verified,
    /// Incompatible, flawed, or rejected mapping
    Rejected,
}

impl MappingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            MappingStatus::Unreviewed => "unreviewed",
            MappingStatus::Mapped => "mapped",
            MappingStatus::Verified => "verified",
            MappingStatus::Rejected => "rejected",
        }
    }
}

impl std::fmt::Display for MappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Confidence level of the schema mapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MappingConfidence {
    /// Fully deterministic rule-based mapping (exact schema match)
    Deterministic,
    /// High-confidence programmatic match
    HighConfidence,
    /// Low-confidence or ambiguous mapping requiring human review
    NeedsReview,
}

impl MappingConfidence {
    pub fn as_str(&self) -> &'static str {
        match self {
            MappingConfidence::Deterministic => "deterministic",
            MappingConfidence::HighConfidence => "high_confidence",
            MappingConfidence::NeedsReview => "needs_review",
        }
    }
}

impl std::fmt::Display for MappingConfidence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Structured mapping bridge linking an authentic PYQ to the procedural engine's
/// Skill, Schema, and ProblemFamily taxonomy.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PyqMapping {
    pub pyq_id: PyqId,
    pub domain: Domain,
    pub skill_id: SkillId,
    pub schema_id: SchemaId,
    pub problem_family_id: ProblemFamilyId,
    /// Suggested variant structural template (e.g. "numerical", "reverse", "trap")
    pub variant_structure: Option<String>,
    /// Calibrated difficulty level (1..=5)
    pub difficulty_level: u32,
    /// Benchmark target solving time in milliseconds
    pub target_latency_ms: u64,
    /// Diagnostic tags and error categories expected for this question
    pub diagnostic_metadata: serde_json::Value,
    /// Lifecycle review status
    pub status: MappingStatus,
    /// Classification confidence
    pub confidence: MappingConfidence,
    /// Reviewer annotations or rationale
    pub reviewer_notes: Option<String>,
    /// Last update timestamp
    pub updated_at: i64,
}

impl PyqMapping {
    pub fn new(
        pyq_id: impl Into<PyqId>,
        domain: Domain,
        skill_id: impl Into<SkillId>,
        schema_id: impl Into<SchemaId>,
        problem_family_id: impl Into<ProblemFamilyId>,
        difficulty_level: u32,
        target_latency_ms: u64,
    ) -> Self {
        Self {
            pyq_id: pyq_id.into(),
            domain,
            skill_id: skill_id.into(),
            schema_id: schema_id.into(),
            problem_family_id: problem_family_id.into(),
            variant_structure: None,
            difficulty_level: difficulty_level.clamp(1, 5),
            target_latency_ms,
            diagnostic_metadata: serde_json::Value::Object(Default::default()),
            status: MappingStatus::Mapped,
            confidence: MappingConfidence::HighConfidence,
            reviewer_notes: None,
            updated_at: Utc::now().timestamp(),
        }
    }

    pub fn with_status(mut self, status: MappingStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_confidence(mut self, confidence: MappingConfidence) -> Self {
        self.confidence = confidence;
        self
    }

    pub fn with_variant_structure(mut self, structure: impl Into<String>) -> Self {
        self.variant_structure = Some(structure.into());
        self
    }

    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.reviewer_notes = Some(notes.into());
        self
    }

    pub fn with_diagnostics(mut self, diagnostics: serde_json::Value) -> Self {
        self.diagnostic_metadata = diagnostics;
        self
    }

    /// Whether this mapping is eligible for autonomous practice selection.
    /// Unreviewed, NeedsReview, and Rejected mappings are strictly gated out.
    pub fn is_eligible_for_practice(&self) -> bool {
        match self.status {
            MappingStatus::Verified => true,
            MappingStatus::Mapped => match self.confidence {
                MappingConfidence::Deterministic | MappingConfidence::HighConfidence => true,
                MappingConfidence::NeedsReview => false,
            },
            MappingStatus::Unreviewed | MappingStatus::Rejected => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pyq_source_immutability_and_creation() {
        let pyq = PYQSource::new(
            "pyq.rrb_alp.2018.shift1.q14",
            "RRB ALP",
            2018,
            Domain::Mathematics,
            "A train 150m long is running at 54 km/h. How much time will it take to cross a platform 250m long?",
            serde_json::json!({ "answer_seconds": 26.67, "correct_option": "A" }),
            "RRB ALP Stage 1 - 09 Aug 2018 Shift 1 Q.14",
        )
        .with_shift_info(Some("Stage 1"), Some("Shift 1"), Some("Session 1"))
        .with_options(vec![
            "26.67 seconds".into(),
            "24.00 seconds".into(),
            "30.00 seconds".into(),
            "20.50 seconds".into(),
        ]);

        assert_eq!(pyq.exam, "RRB ALP");
        assert_eq!(pyq.year, 2018);
        assert_eq!(pyq.shift.as_deref(), Some("Shift 1"));
        assert_eq!(pyq.provenance.source_pyq_id, Some(pyq.id.clone()));
        assert_eq!(pyq.provenance.variant_type, "authentic_pyq");
    }

    #[test]
    fn test_pyq_mapping_eligibility_gating() {
        let mut mapping = PyqMapping::new(
            "pyq.rrb_alp.2018.shift1.q14",
            Domain::Mathematics,
            "arithmetic.time_speed_distance",
            "schema.math.arithmetic.time_speed_distance",
            "family.math.arithmetic.time_speed_distance",
            2,
            40_000,
        );

        // High confidence mapped -> eligible
        assert!(mapping.is_eligible_for_practice());

        // Needs review -> not eligible
        mapping.confidence = MappingConfidence::NeedsReview;
        assert!(!mapping.is_eligible_for_practice());

        // Verified -> always eligible
        mapping.status = MappingStatus::Verified;
        assert!(mapping.is_eligible_for_practice());

        // Rejected -> not eligible
        mapping.status = MappingStatus::Rejected;
        assert!(!mapping.is_eligible_for_practice());
    }
}
