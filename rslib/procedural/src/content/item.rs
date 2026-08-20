// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::core::{Domain, PracticeItemId, ProblemFamilyId, PyqId, SchemaId, SkillId};
use crate::exam::pyq::ContentProvenance;

/// Origin of a practice item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Origin {
    AuthenticPyq {
        pyq_id: PyqId,
        exam: String,
        year: u32,
        shift: Option<String>,
    },
    CuratedSource {
        source_reference: String,
    },
    DerivedVariant {
        parent_id: PracticeItemId,
        generator_version: u32,
        seed: u64,
        variant_type: String,
    },
    SyntheticSchema {
        generator_version: u32,
        seed: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum QuestionType {
    Mcq {
        options: Vec<String>,
        correct_option: String,
        explanation: Option<String>,
    },
    Numerical {
        answer: f64,
        tolerance: Option<f64>,
    },
    Structured {
        steps: serde_json::Value,
    },
    ReferenceOnly {
        source_reference: String,
    },
}

/// The smallest canonical persistent representation for a source-backed practice question.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PracticeItem {
    pub id: PracticeItemId,
    pub origin: Origin,
    pub domain: Domain,
    pub chapter: String,
    pub skill_id: SkillId,
    pub schema_id: SchemaId,
    pub problem_family_id: ProblemFamilyId,
    
    pub question_type: QuestionType,
    pub prompt: String,
    
    pub difficulty: f64,
    pub structural_tags: Vec<String>,
    pub decision_points: Vec<String>,
    pub error_categories: Vec<String>,
    pub prerequisites: Vec<String>,
    
    pub provenance: ContentProvenance,
    
    pub created_at: i64,
    pub metadata: serde_json::Value,
}

impl PracticeItem {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: impl Into<PracticeItemId>,
        origin: Origin,
        domain: Domain,
        chapter: impl Into<String>,
        skill_id: impl Into<SkillId>,
        schema_id: impl Into<SchemaId>,
        problem_family_id: impl Into<ProblemFamilyId>,
        question_type: QuestionType,
        prompt: impl Into<String>,
        provenance: ContentProvenance,
    ) -> Self {
        Self {
            id: id.into(),
            origin,
            domain,
            chapter: chapter.into(),
            skill_id: skill_id.into(),
            schema_id: schema_id.into(),
            problem_family_id: problem_family_id.into(),
            question_type,
            prompt: prompt.into(),
            difficulty: 3.0,
            structural_tags: vec![],
            decision_points: vec![],
            error_categories: vec![],
            prerequisites: vec![],
            provenance,
            created_at: Utc::now().timestamp(),
            metadata: serde_json::Value::Object(Default::default()),
        }
    }

    /// Converts a static source practice item into an executable `ProblemInstance`
    pub fn into_problem_instance(self) -> crate::problems::ProblemInstance {
        let mut parameters = serde_json::json!({});
        let mut correct_answer = serde_json::json!({});
        match self.question_type {
            QuestionType::Mcq { options, correct_option, explanation } => {
                parameters["options"] = serde_json::to_value(options).unwrap();
                correct_answer["correct_option"] = serde_json::to_value(correct_option).unwrap();
                if let Some(exp) = explanation {
                    correct_answer["explanation"] = serde_json::to_value(exp).unwrap();
                }
            }
            QuestionType::Numerical { answer, tolerance } => {
                correct_answer["answer"] = serde_json::to_value(answer).unwrap();
                if let Some(t) = tolerance {
                    correct_answer["tolerance"] = serde_json::to_value(t).unwrap();
                }
            }
            QuestionType::Structured { steps } => {
                correct_answer["steps"] = steps;
            }
            QuestionType::ReferenceOnly { source_reference } => {
                correct_answer["reference_only"] = serde_json::to_value(source_reference).unwrap();
            }
        }
        
        let mut metadata = self.metadata;
        if let Some(obj) = metadata.as_object_mut() {
            obj.insert("practice_item_id".to_string(), serde_json::to_value(&self.id).unwrap());
            obj.insert("origin".to_string(), serde_json::to_value(&self.origin).unwrap());
            obj.insert("chapter".to_string(), serde_json::to_value(&self.chapter).unwrap());
            obj.insert("difficulty".to_string(), serde_json::to_value(self.difficulty).unwrap());
            obj.insert("structural_tags".to_string(), serde_json::to_value(&self.structural_tags).unwrap());
            obj.insert("provenance".to_string(), serde_json::to_value(&self.provenance).unwrap());
            obj.insert("is_source_item".to_string(), serde_json::json!(true));
        }

        let mut instance = crate::problems::ProblemInstance::new(
            crate::core::ProblemInstanceId::new(format!("inst-pi-{}", self.id)),
            self.problem_family_id,
            0, // deterministic seed for static items
            parameters,
            self.prompt,
            correct_answer,
        );
        instance.metadata = metadata;
        instance
    }
}
