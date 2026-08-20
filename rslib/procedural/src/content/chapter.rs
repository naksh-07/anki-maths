// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::collections::HashMap;

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::core::{Domain, ProblemFamilyId, SchemaId};

/// Declares whether the procedural engine can generate variants for a given family.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GeneratorCapability {
    /// Source questions + generator + variants + transfer
    Full,
    /// Source questions + limited generated variants
    Partial,
    /// Authentic/curated questions + procedural metadata, but no generator yet
    SourceOnly,
}

/// Chapter-level capability/configuration object.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChapterPracticeProfile {
    pub chapter_name: String,
    pub domain: Domain,
    
    pub supported_schemas: Vec<SchemaId>,
    pub supported_problem_families: Vec<ProblemFamilyId>,
    
    pub generator_capabilities: HashMap<String, GeneratorCapability>, // ProblemFamilyId as String
    
    pub recognition_signals: Vec<String>,
    pub decision_points: Vec<String>,
    pub variation_dimensions: Vec<String>,
    pub prerequisites: Vec<String>,
    pub error_categories: Vec<String>,
    pub exam_relevance: Vec<String>,
    
    pub created_at: i64,
    pub metadata: serde_json::Value,
}

impl ChapterPracticeProfile {
    pub fn new(chapter_name: impl Into<String>, domain: Domain) -> Self {
        Self {
            chapter_name: chapter_name.into(),
            domain,
            supported_schemas: vec![],
            supported_problem_families: vec![],
            generator_capabilities: HashMap::new(),
            recognition_signals: vec![],
            decision_points: vec![],
            variation_dimensions: vec![],
            prerequisites: vec![],
            error_categories: vec![],
            exam_relevance: vec![],
            created_at: Utc::now().timestamp(),
            metadata: serde_json::Value::Object(Default::default()),
        }
    }
}
