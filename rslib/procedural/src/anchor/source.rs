// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::borrow::Cow;
use std::collections::HashMap;

use crate::core::{Domain, PracticeItemId, ProceduralError, Result};
use crate::content::item::{Origin, PracticeItem, QuestionType};
use crate::exam::pyq::ContentProvenance;
use crate::practice::SchemaPracticeObject;
use crate::scheduling::{PracticeSessionObject, SessionReadiness};

/// A static source question extracted directly from Anki note fields.
#[derive(Debug, Clone, PartialEq)]
pub struct SourceQuestion {
    pub prompt: String,
    pub options: Option<Vec<String>>,
    pub correct_answer: String,
    pub explanation: Option<String>,
    pub domain: String,
    pub chapter: String,
    pub topic: String,
    pub difficulty: f64,
}

impl SourceQuestion {
    /// Extracts a SourceQuestion from standard Anki Note fields.
    pub fn extract_from_card_fields(fields: &HashMap<&str, Cow<str>>) -> Result<Self> {
        let get_field = |keys: &[&str]| -> Option<String> {
            for &key in keys {
                if let Some(val) = fields.get(key) {
                    let trimmed = val.trim();
                    if !trimmed.is_empty() {
                        return Some(trimmed.to_string());
                    }
                }
            }
            None
        };

        let prompt = get_field(&["Prompt", "Question", "Front"])
            .ok_or_else(|| ProceduralError::InvalidAnchor("Missing Prompt/Question field in Source Note".into()))?;
        
        let correct_answer = get_field(&["CorrectAnswer", "Answer", "Back"])
            .ok_or_else(|| ProceduralError::InvalidAnchor("Missing CorrectAnswer/Answer field in Source Note".into()))?;

        let options_raw = get_field(&["Options"]);
        let options = options_raw.map(|opts| {
            let trimmed = opts.trim();
            // Assume options are newline separated or JSON array.
            if trimmed.starts_with('[') {
                if let Ok(arr) = serde_json::from_str::<Vec<String>>(trimmed) {
                    return arr;
                }
            }
            trimmed.lines().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
        });

        let explanation = get_field(&["Explanation", "Solution", "Steps"]);
        let domain = get_field(&["Domain", "Subject"]).unwrap_or_else(|| "mathematics".to_string());
        let chapter = get_field(&["Chapter"]).unwrap_or_else(|| "General".to_string());
        let topic = get_field(&["Topic", "TopicTitle"]).unwrap_or_else(|| "General".to_string());
        let difficulty = get_field(&["Difficulty"]).and_then(|d| d.parse::<f64>().ok()).unwrap_or(2.0);

        Ok(Self {
            prompt,
            options,
            correct_answer,
            explanation,
            domain,
            chapter,
            topic,
            difficulty,
        })
    }

    pub fn stable_id_from_guid(guid: &str) -> PracticeItemId {
        PracticeItemId::new(format!("pi_src_{}", guid))
    }

    /// Translates the extracted SourceQuestion into a canonical PracticeItem for persistence.
    pub fn into_practice_item(self, guid: &str) -> PracticeItem {
        let domain_enum = match self.domain.to_lowercase().as_str() {
            "physics" => Domain::Physics,
            "chemistry" => Domain::Chemistry,
            "reasoning" => Domain::Reasoning,
            _ => Domain::Mathematics,
        };

        let q_type = if let Some(opts) = self.options {
            QuestionType::Mcq {
                options: opts,
                correct_option: self.correct_answer.clone(),
                explanation: self.explanation.clone(),
            }
        } else {
            QuestionType::Numerical {
                answer: self.correct_answer.parse().unwrap_or(0.0),
                tolerance: None,
            }
        };

        let provenance = ContentProvenance {
            source_pyq_id: None,
            source_version: 1,
            generator_version: 1,
            schema_version: 1,
            catalog_version: 1,
            variant_type: "static".into(),
            seed: None,
        };

        let schema_id = "schema.static.source";
        let family_id = "family.static.source";
        let skill_id = "skill.static.source";
        
        let item_id = Self::stable_id_from_guid(guid);

        let mut item = PracticeItem::new(
            item_id,
            Origin::CuratedSource { source_reference: format!("anki-note:{}", guid) },
            domain_enum,
            self.chapter.clone(),
            skill_id,
            schema_id,
            family_id,
            q_type,
            self.prompt.clone(),
            provenance,
        );
        item.difficulty = self.difficulty;
        
        // Ensure topic is stored for unified rendering if needed
        let mut meta = serde_json::Map::new();
        meta.insert("topic".to_string(), serde_json::Value::String(self.topic.clone()));
        item.metadata = serde_json::Value::Object(meta);
        
        item
    }

    /// Translates the extracted SourceQuestion into a unified PracticeSessionObject
    /// capable of rendering in the existing procedural StudyLab UI.
    pub fn into_practice_session(self, card_id: Option<i64>) -> Result<PracticeSessionObject> {
        let _domain_enum = match self.domain.to_lowercase().as_str() {
            "physics" => Domain::Physics,
            "chemistry" => Domain::Chemistry,
            "reasoning" => Domain::Reasoning,
            _ => Domain::Mathematics,
        };

        let _q_type = if let Some(ref opts) = self.options {
            QuestionType::Mcq {
                options: opts.clone(),
                correct_option: self.correct_answer.clone(),
                explanation: self.explanation.clone(),
            }
        } else {
            QuestionType::Numerical {
                answer: self.correct_answer.parse().unwrap_or(0.0),
                tolerance: None,
            }
        };

        let _provenance = ContentProvenance {
            source_pyq_id: None,
            source_version: 1,
            generator_version: 1,
            schema_version: 1,
            catalog_version: 1,
            variant_type: "static".into(),
            seed: None,
        };

        let schema_id = "schema.static.source";
        let family_id = "family.static.source";
        let skill_id = "skill.static.source";

        // This remains for backward-compatible on-the-fly rendering without reconciliation.
        // During Phase 3, this is technically a fallback if the DB is missing the record,
        // but we'll leave it standalone.
        let guid = "render_fallback";
        let item = self.clone().into_practice_item(guid);
        
        let instance = item.into_problem_instance();
        
        let schema = SchemaPracticeObject::new(
            schema_id.to_string(),
            skill_id.to_string(),
            family_id.to_string(),
            self.topic.clone(),
            format!("Static Source Schema for {}", self.topic),
        );
        
        let mut session = PracticeSessionObject::new(schema, instance, card_id, None);
        session.readiness = SessionReadiness::Ready;
        session.selection_reason = Some("StudyLab Static Source Rendering".to_string());
        session.difficulty_level = Some(self.difficulty as u32);
        
        Ok(session)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_source_question_success() {
        let mut fields = HashMap::new();
        fields.insert("Prompt", Cow::Borrowed("What is 2+2?"));
        fields.insert("CorrectAnswer", Cow::Borrowed("4"));
        fields.insert("Options", Cow::Borrowed("[\"3\", \"4\", \"5\"]"));
        fields.insert("Domain", Cow::Borrowed("mathematics"));

        let q = SourceQuestion::extract_from_card_fields(&fields).unwrap();
        assert_eq!(q.prompt, "What is 2+2?");
        assert_eq!(q.correct_answer, "4");
        assert_eq!(q.options, Some(vec!["3".to_string(), "4".to_string(), "5".to_string()]));
        assert_eq!(q.domain, "mathematics");
        assert_eq!(q.difficulty, 2.0); // default
    }

    #[test]
    fn test_extract_source_question_missing_fields() {
        let mut fields = HashMap::new();
        fields.insert("Prompt", Cow::Borrowed("What is 2+2?"));
        // Missing CorrectAnswer

        let err = SourceQuestion::extract_from_card_fields(&fields).unwrap_err();
        assert!(matches!(err, ProceduralError::InvalidAnchor(_)));
    }

    #[test]
    fn test_into_practice_session() {
        let q = SourceQuestion {
            prompt: "What is 2+2?".into(),
            options: Some(vec!["3".into(), "4".into()]),
            correct_answer: "4".into(),
            explanation: None,
            domain: "mathematics".into(),
            chapter: "Algebra".into(),
            topic: "Addition".into(),
            difficulty: 1.5,
        };

        let session = q.into_practice_session(Some(123)).unwrap();
        assert_eq!(session.card_id, Some(123));
        assert_eq!(session.schema.id.as_str(), "schema.static.source");
        assert_eq!(session.instance.correct_answer["correct_option"].as_str().unwrap(), "4");
    }
}
