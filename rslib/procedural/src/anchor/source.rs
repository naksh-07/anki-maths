// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::content::item::{Origin, PracticeItem, QuestionType};
use crate::core::{Domain, PracticeItemId, ProblemFamilyId, ProceduralError, PyqId, SchemaId, SkillId};
use crate::exam::pyq::ContentProvenance;

/// Canonical question types supported by the StudyLab Source APKG contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CanonicalQuestionType {
    Mcq,
    Numerical,
}

impl CanonicalQuestionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            CanonicalQuestionType::Mcq => "mcq",
            CanonicalQuestionType::Numerical => "numerical",
        }
    }

    pub fn parse_canonical(s: &str) -> std::result::Result<Self, SourceContractError> {
        let normalized = s.trim().to_lowercase();
        match normalized.as_str() {
            "mcq" | "multiple_choice" | "multiplechoice" => Ok(CanonicalQuestionType::Mcq),
            "numerical" | "numeric" => Ok(CanonicalQuestionType::Numerical),
            _ => Err(SourceContractError::InvalidQuestionType {
                raw_value: s.to_string(),
                context: format!("Unsupported question type '{}'. Supported types: 'mcq', 'numerical'", s),
            }),
        }
    }
}

impl fmt::Display for CanonicalQuestionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Structured validation errors for the Canonical Source APKG contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceContractError {
    UnsupportedNoteType {
        note_type: String,
        context: String,
    },
    MissingRequiredField {
        field_name: &'static str,
        context: String,
    },
    InvalidQuestionType {
        raw_value: String,
        context: String,
    },
    InvalidDifficulty {
        raw_value: String,
        reason: String,
        context: String,
    },
    MissingMcqOptions {
        context: String,
    },
    InvalidCorrectAnswer {
        reason: String,
        answer: String,
        context: String,
    },
    InvalidProvenance {
        field_name: &'static str,
        raw_value: String,
        reason: String,
        context: String,
    },
    DuplicateOrAmbiguousSourceIdentity {
        identity: String,
        context: String,
    },
    SchemaMismatch {
        expected: String,
        actual: String,
        context: String,
    },
}

impl fmt::Display for SourceContractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SourceContractError::UnsupportedNoteType { note_type, context } => {
                write!(f, "Unsupported note type '{}': {}", note_type, context)
            }
            SourceContractError::MissingRequiredField { field_name, context } => {
                write!(f, "Missing required canonical field '{}': {}", field_name, context)
            }
            SourceContractError::InvalidQuestionType { raw_value, context } => {
                write!(f, "Invalid QuestionType '{}': {}", raw_value, context)
            }
            SourceContractError::InvalidDifficulty { raw_value, reason, context } => {
                write!(f, "Invalid Difficulty '{}' ({}): {}", raw_value, reason, context)
            }
            SourceContractError::MissingMcqOptions { context } => {
                write!(f, "Missing MCQ options: {}", context)
            }
            SourceContractError::InvalidCorrectAnswer { reason, answer, context } => {
                write!(f, "Invalid CorrectAnswer '{}' ({}): {}", answer, reason, context)
            }
            SourceContractError::InvalidProvenance { field_name, raw_value, reason, context } => {
                write!(f, "Invalid provenance field '{}'='{}' ({}): {}", field_name, raw_value, reason, context)
            }
            SourceContractError::DuplicateOrAmbiguousSourceIdentity { identity, context } => {
                write!(f, "Duplicate or ambiguous source identity '{}': {}", identity, context)
            }
            SourceContractError::SchemaMismatch { expected, actual, context } => {
                write!(f, "Schema mismatch (expected '{}', got '{}'): {}", expected, actual, context)
            }
        }
    }
}

impl std::error::Error for SourceContractError {}

impl From<SourceContractError> for ProceduralError {
    fn from(err: SourceContractError) -> Self {
        ProceduralError::Validation(err.to_string())
    }
}

/// Canonical Source Question data model.
/// Represents the pure, immutable source question data strictly decoupled from runtime learner state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceQuestion {
    // --- Content Fields ---
    pub prompt: String,
    pub options: Option<Vec<String>>,
    pub correct_answer: String,
    pub hint: Option<String>,
    pub solution: Option<String>,
    pub steps: Option<Vec<String>>,
    pub explanation: Option<String>,

    // --- Semantic Fields ---
    pub subject: Option<String>,
    pub chapter: Option<String>,
    pub topic: Option<String>,
    pub skill: Option<String>,
    pub problem_type: Option<String>,
    pub question_type: CanonicalQuestionType,
    pub difficulty: Option<f64>,

    // --- Provenance Fields ---
    pub source: Option<String>,
    pub exam: Option<String>,
    pub year: Option<i32>,
    pub shift: Option<String>,
    pub paper: Option<String>,
    pub source_question_id: Option<String>,
}

impl SourceQuestion {
    /// Extracts and strictly validates a canonical `SourceQuestion` from standard Anki Note fields.
    pub fn extract_from_card_fields(fields: &HashMap<&str, Cow<str>>) -> std::result::Result<Self, SourceContractError> {
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

        // 1. Required Field: Prompt
        let prompt = get_field(&["Prompt", "Question", "Front"])
            .ok_or_else(|| SourceContractError::MissingRequiredField {
                field_name: "Prompt",
                context: "Note does not contain a non-empty 'Prompt' or 'Question' field".to_string(),
            })?;

        // 2. Required Field: QuestionType (Explicitly extracted, never inferred from Options)
        let raw_qtype = get_field(&["QuestionType", "Type", "LearningObjectType"])
            .ok_or_else(|| SourceContractError::MissingRequiredField {
                field_name: "QuestionType",
                context: "Note must explicitly declare 'QuestionType' ('mcq' or 'numerical')".to_string(),
            })?;
        let question_type = CanonicalQuestionType::parse_canonical(&raw_qtype)?;

        // 3. Required Field: CorrectAnswer
        let correct_answer = get_field(&["CorrectAnswer", "Answer", "Back"])
            .ok_or_else(|| SourceContractError::MissingRequiredField {
                field_name: "CorrectAnswer",
                context: "Note does not contain a non-empty 'CorrectAnswer' or 'Answer' field".to_string(),
            })?;

        // 4. Modality-specific Options Extraction & Validation
        let options_raw = get_field(&["Options"]);
        let options = match question_type {
            CanonicalQuestionType::Mcq => {
                let opts = match options_raw {
                    Some(raw) => {
                        let trimmed = raw.trim();
                        let parsed: Vec<String> = if trimmed.starts_with('[') {
                            serde_json::from_str::<Vec<String>>(trimmed)
                                .unwrap_or_else(|_| {
                                    trimmed.lines()
                                        .map(|s| s.trim().to_string())
                                        .filter(|s| !s.is_empty())
                                        .collect()
                                })
                        } else {
                            trimmed.lines()
                                .map(|s| s.trim().to_string())
                                .filter(|s| !s.is_empty())
                                .collect()
                        };
                        parsed
                    }
                    None => {
                        return Err(SourceContractError::MissingMcqOptions {
                            context: "MCQ question must provide an 'Options' field".to_string(),
                        });
                    }
                };

                let opts: Vec<String> = opts.into_iter()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();

                if opts.len() < 2 {
                    return Err(SourceContractError::MissingMcqOptions {
                        context: format!("MCQ question requires at least 2 non-empty options, found {}", opts.len()),
                    });
                }

                // Validate that CorrectAnswer resolves to one of the supplied options
                let resolves = Self::resolve_mcq_answer(&correct_answer, &opts);
                if resolves.is_none() {
                    return Err(SourceContractError::InvalidCorrectAnswer {
                        reason: "MCQ CorrectAnswer does not match any provided Option".to_string(),
                        answer: correct_answer.clone(),
                        context: format!("Provided options: {:?}", opts),
                    });
                }

                Some(opts)
            }
            CanonicalQuestionType::Numerical => {
                // For Numerical, validate that correct_answer parses as a valid finite floating-point number
                match correct_answer.trim().parse::<f64>() {
                    Ok(val) if val.is_finite() => {}
                    _ => {
                        return Err(SourceContractError::InvalidCorrectAnswer {
                            reason: "Numerical CorrectAnswer cannot be parsed as a valid finite numeric floating point value".to_string(),
                            answer: correct_answer.clone(),
                            context: "Expected valid finite numeric string (e.g. '42', '3.14', '-10.5')".to_string(),
                        });
                    }
                }
                None
            }
        };

        // 5. Difficulty Validation (Source metadata in [1.0, 5.0])
        let difficulty = if let Some(raw_diff) = get_field(&["Difficulty"]) {
            match raw_diff.trim().parse::<f64>() {
                Ok(val) if val.is_finite() && (1.0..=5.0).contains(&val) => Some(val),
                Ok(val) => {
                    return Err(SourceContractError::InvalidDifficulty {
                        raw_value: raw_diff,
                        reason: "Difficulty rating must be within range [1.0, 5.0]".to_string(),
                        context: format!("Value {} is out of bounds", val),
                    });
                }
                Err(_) => {
                    return Err(SourceContractError::InvalidDifficulty {
                        raw_value: raw_diff,
                        reason: "Difficulty rating must be a valid float".to_string(),
                        context: "Failed to parse difficulty as f64".to_string(),
                    });
                }
            }
        } else {
            None
        };

        // 6. Optional Content Fields
        let hint = get_field(&["Hint"]);
        let solution = get_field(&["Solution"]);
        let steps = get_field(&["Steps"]).map(|s| {
            let trimmed = s.trim();
            if trimmed.starts_with('[') {
                serde_json::from_str::<Vec<String>>(trimmed).unwrap_or_else(|_| {
                    trimmed.lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect()
                })
            } else {
                trimmed.lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect()
            }
        });
        let explanation = get_field(&["Explanation"]);

        // 7. Optional Semantic Fields
        let subject = get_field(&["Subject", "Domain"]);
        let chapter = get_field(&["Chapter"]);
        let topic = get_field(&["Topic", "TopicTitle"]);
        let skill = get_field(&["Skill"]);
        let problem_type = get_field(&["ProblemType"]);

        // 8. Optional Provenance Fields
        let source = get_field(&["Source"]);
        let exam = get_field(&["Exam"]);
        let year = if let Some(raw_year) = get_field(&["Year"]) {
            match raw_year.parse::<i32>() {
                Ok(y) => Some(y),
                Err(_) => {
                    return Err(SourceContractError::InvalidProvenance {
                        field_name: "Year",
                        raw_value: raw_year,
                        reason: "Year must be a valid integer".to_string(),
                        context: "Failed to parse year".to_string(),
                    });
                }
            }
        } else {
            None
        };
        let shift = get_field(&["Shift"]);
        let paper = get_field(&["Paper"]);
        let source_question_id = get_field(&["SourceQuestionID", "SourceQuestionId", "QuestionID"]);

        Ok(Self {
            prompt,
            options,
            correct_answer,
            hint,
            solution,
            steps,
            explanation,
            subject,
            chapter,
            topic,
            skill,
            problem_type,
            question_type,
            difficulty,
            source,
            exam,
            year,
            shift,
            paper,
            source_question_id,
        })
    }

    /// Resolves an MCQ correct_answer string against a list of options.
    /// Supports exact matching, or option letter/number index resolution (e.g. 'A', '1', etc.).
    pub fn resolve_mcq_answer<'a>(answer: &str, options: &'a [String]) -> Option<&'a str> {
        let trimmed = answer.trim();

        // 1. Exact match with an option string
        for opt in options {
            if opt.trim() == trimmed {
                return Some(opt.as_str());
            }
        }

        // 2. Single letter index matching (A/a -> 0, B/b -> 1, ...)
        if trimmed.len() == 1 {
            let ch = trimmed.chars().next().unwrap();
            let idx = match ch {
                'A'..='Z' => (ch as usize) - ('A' as usize),
                'a'..='z' => (ch as usize) - ('a' as usize),
                '1'..='9' => (ch as usize) - ('1' as usize),
                _ => usize::MAX,
            };
            if idx < options.len() {
                return Some(options[idx].as_str());
            }
        }

        // 3. Prefix matching e.g. "A) Option text" or "A. Option text"
        for opt in options {
            let opt_trimmed = opt.trim();
            if opt_trimmed.starts_with(trimmed) || trimmed.starts_with(opt_trimmed) {
                return Some(opt.as_str());
            }
        }

        None
    }

    pub fn stable_id_from_guid(guid: &str) -> PracticeItemId {
        PracticeItemId::new(format!("pi_src_{}", guid))
    }

    /// Translates the canonical SourceQuestion into an internal PracticeItem for persistence and runtime execution.
    pub fn into_practice_item(self, guid: &str) -> PracticeItem {
        let domain_enum = match self.subject.as_deref().map(|s| s.to_lowercase()).as_deref() {
            Some("physics") => Domain::Physics,
            Some("chemistry") | Some("chem") => Domain::Chemistry,
            Some("reasoning") | Some("logic") => Domain::Reasoning,
            _ => Domain::Mathematics,
        };

        let chapter_str = self.chapter.clone().unwrap_or_else(|| "General".to_string());
        let topic_str = self.topic.clone().unwrap_or_else(|| "General".to_string());

        let sanitize_slug = |s: &str| -> String {
            s.to_lowercase()
                .chars()
                .map(|c| if c.is_alphanumeric() { c } else { '_' })
                .collect::<String>()
                .trim_matches('_')
                .to_string()
        };

        let subject_slug = sanitize_slug(domain_enum.as_str());
        let topic_slug = sanitize_slug(&topic_str);

        // Derive internal IDs cleanly without artificial dummy schemas
        let skill_id = if let Some(ref sk) = self.skill {
            SkillId::new(sk.clone())
        } else {
            SkillId::new(format!("skill.source.{}.{}", subject_slug, topic_slug))
        };

        let schema_id = SchemaId::new(format!("schema.source.{}.{}", subject_slug, topic_slug));
        let family_id = ProblemFamilyId::new(format!("family.source.{}.{}", subject_slug, topic_slug));

        let q_type = match self.question_type {
            CanonicalQuestionType::Mcq => {
                let opts = self.options.clone().unwrap_or_default();
                let resolved_ans = Self::resolve_mcq_answer(&self.correct_answer, &opts)
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| self.correct_answer.clone());

                QuestionType::Mcq {
                    options: opts,
                    correct_option: resolved_ans,
                    explanation: self.explanation.clone().or_else(|| self.solution.clone()),
                }
            }
            CanonicalQuestionType::Numerical => {
                let ans = self.correct_answer.trim().parse::<f64>().unwrap_or(0.0);
                QuestionType::Numerical {
                    answer: ans,
                    tolerance: None,
                }
            }
        };

        let source_ref = if let Some(ref sq_id) = self.source_question_id {
            format!("anki-note:{}:{}", guid, sq_id)
        } else {
            format!("anki-note:{}", guid)
        };

        let origin = if self.exam.is_some() && self.year.is_some() {
            Origin::AuthenticPyq {
                pyq_id: PyqId::new(self.source_question_id.clone().unwrap_or_else(|| guid.to_string())),
                exam: self.exam.clone().unwrap_or_default(),
                year: self.year.unwrap_or(0) as u32,
                shift: self.shift.clone(),
            }
        } else {
            Origin::CuratedSource { source_reference: source_ref }
        };

        let provenance = ContentProvenance {
            source_pyq_id: self.source_question_id.as_ref().map(|id| PyqId::new(id.clone())),
            source_version: 1,
            generator_version: 1,
            schema_version: 1,
            catalog_version: 1,
            variant_type: "canonical_source".into(),
            seed: None,
        };

        let item_id = Self::stable_id_from_guid(guid);

        let mut item = PracticeItem::new(
            item_id,
            origin,
            domain_enum,
            chapter_str,
            skill_id,
            schema_id,
            family_id,
            q_type,
            self.prompt.clone(),
            provenance,
        );

        if let Some(diff) = self.difficulty {
            item.difficulty = diff;
        }

        // Build comprehensive metadata JSON object
        let mut meta = serde_json::Map::new();
        meta.insert("canonical_contract".to_string(), serde_json::Value::Bool(true));
        meta.insert("question_type".to_string(), serde_json::Value::String(self.question_type.to_string()));
        if let Some(ref sub) = self.subject {
            meta.insert("subject".to_string(), serde_json::Value::String(sub.clone()));
        }
        if let Some(ref chap) = self.chapter {
            meta.insert("chapter".to_string(), serde_json::Value::String(chap.clone()));
        }
        if let Some(ref top) = self.topic {
            meta.insert("topic".to_string(), serde_json::Value::String(top.clone()));
        }
        if let Some(ref sk) = self.skill {
            meta.insert("skill".to_string(), serde_json::Value::String(sk.clone()));
        }
        if let Some(ref pt) = self.problem_type {
            meta.insert("problem_type".to_string(), serde_json::Value::String(pt.clone()));
        }
        if let Some(ref h) = self.hint {
            meta.insert("hint".to_string(), serde_json::Value::String(h.clone()));
        }
        if let Some(ref s) = self.solution {
            meta.insert("solution".to_string(), serde_json::Value::String(s.clone()));
        }
        if let Some(ref st) = self.steps {
            meta.insert("steps".to_string(), serde_json::to_value(st).unwrap_or(serde_json::Value::Null));
        }
        if let Some(ref exp) = self.explanation {
            meta.insert("explanation".to_string(), serde_json::Value::String(exp.clone()));
        }
        if let Some(ref src) = self.source {
            meta.insert("source".to_string(), serde_json::Value::String(src.clone()));
        }
        if let Some(ref ex) = self.exam {
            meta.insert("exam".to_string(), serde_json::Value::String(ex.clone()));
        }
        if let Some(yr) = self.year {
            meta.insert("year".to_string(), serde_json::Value::Number(serde_json::Number::from(yr)));
        }
        if let Some(ref sh) = self.shift {
            meta.insert("shift".to_string(), serde_json::Value::String(sh.clone()));
        }
        if let Some(ref p) = self.paper {
            meta.insert("paper".to_string(), serde_json::Value::String(p.clone()));
        }
        if let Some(ref sqid) = self.source_question_id {
            meta.insert("source_question_id".to_string(), serde_json::Value::String(sqid.clone()));
        }

        item.metadata = serde_json::Value::Object(meta);
        item
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_canonical_mcq_success() {
        let mut fields = HashMap::new();
        fields.insert("Prompt", Cow::Borrowed("What is the capital of France?"));
        fields.insert("QuestionType", Cow::Borrowed("MCQ"));
        fields.insert("Options", Cow::Borrowed("[\"Berlin\", \"Madrid\", \"Paris\", \"Rome\"]"));
        fields.insert("CorrectAnswer", Cow::Borrowed("Paris"));
        fields.insert("Difficulty", Cow::Borrowed("2.0"));
        fields.insert("Subject", Cow::Borrowed("reasoning"));
        fields.insert("Chapter", Cow::Borrowed("General Knowledge"));
        fields.insert("Topic", Cow::Borrowed("Capitals"));
        fields.insert("Exam", Cow::Borrowed("SSC CGL"));
        fields.insert("Year", Cow::Borrowed("2024"));
        fields.insert("SourceQuestionID", Cow::Borrowed("SSC_2024_01"));

        let q = SourceQuestion::extract_from_card_fields(&fields).unwrap();
        assert_eq!(q.prompt, "What is the capital of France?");
        assert_eq!(q.question_type, CanonicalQuestionType::Mcq);
        assert_eq!(q.correct_answer, "Paris");
        assert_eq!(q.difficulty, Some(2.0));
        assert_eq!(q.exam, Some("SSC CGL".to_string()));
        assert_eq!(q.year, Some(2024));
        assert_eq!(q.source_question_id, Some("SSC_2024_01".to_string()));
    }

    #[test]
    fn test_extract_canonical_numerical_success() {
        let mut fields = HashMap::new();
        fields.insert("Prompt", Cow::Borrowed("Find the value of 15 * 8."));
        fields.insert("QuestionType", Cow::Borrowed("numerical"));
        fields.insert("CorrectAnswer", Cow::Borrowed("120"));
        fields.insert("Difficulty", Cow::Borrowed("1.5"));
        fields.insert("Subject", Cow::Borrowed("mathematics"));

        let q = SourceQuestion::extract_from_card_fields(&fields).unwrap();
        assert_eq!(q.prompt, "Find the value of 15 * 8.");
        assert_eq!(q.question_type, CanonicalQuestionType::Numerical);
        assert_eq!(q.correct_answer, "120");
        assert_eq!(q.options, None);
        assert_eq!(q.difficulty, Some(1.5));
    }

    #[test]
    fn test_extract_missing_prompt() {
        let mut fields = HashMap::new();
        fields.insert("QuestionType", Cow::Borrowed("MCQ"));
        fields.insert("CorrectAnswer", Cow::Borrowed("A"));
        fields.insert("Options", Cow::Borrowed("[\"A\", \"B\"]"));

        let err = SourceQuestion::extract_from_card_fields(&fields).unwrap_err();
        assert!(matches!(err, SourceContractError::MissingRequiredField { field_name: "Prompt", .. }));
    }

    #[test]
    fn test_extract_missing_question_type() {
        let mut fields = HashMap::new();
        fields.insert("Prompt", Cow::Borrowed("Question?"));
        fields.insert("CorrectAnswer", Cow::Borrowed("42"));

        let err = SourceQuestion::extract_from_card_fields(&fields).unwrap_err();
        assert!(matches!(err, SourceContractError::MissingRequiredField { field_name: "QuestionType", .. }));
    }

    #[test]
    fn test_extract_invalid_question_type() {
        let mut fields = HashMap::new();
        fields.insert("Prompt", Cow::Borrowed("Question?"));
        fields.insert("QuestionType", Cow::Borrowed("essay"));
        fields.insert("CorrectAnswer", Cow::Borrowed("42"));

        let err = SourceQuestion::extract_from_card_fields(&fields).unwrap_err();
        assert!(matches!(err, SourceContractError::InvalidQuestionType { .. }));
    }

    #[test]
    fn test_extract_mcq_missing_options() {
        let mut fields = HashMap::new();
        fields.insert("Prompt", Cow::Borrowed("Question?"));
        fields.insert("QuestionType", Cow::Borrowed("mcq"));
        fields.insert("CorrectAnswer", Cow::Borrowed("Paris"));

        let err = SourceQuestion::extract_from_card_fields(&fields).unwrap_err();
        assert!(matches!(err, SourceContractError::MissingMcqOptions { .. }));
    }

    #[test]
    fn test_extract_mcq_correct_answer_mismatch() {
        let mut fields = HashMap::new();
        fields.insert("Prompt", Cow::Borrowed("Question?"));
        fields.insert("QuestionType", Cow::Borrowed("mcq"));
        fields.insert("Options", Cow::Borrowed("[\"Berlin\", \"Madrid\", \"Rome\"]"));
        fields.insert("CorrectAnswer", Cow::Borrowed("London"));

        let err = SourceQuestion::extract_from_card_fields(&fields).unwrap_err();
        assert!(matches!(err, SourceContractError::InvalidCorrectAnswer { .. }));
    }

    #[test]
    fn test_extract_invalid_difficulty() {
        let mut fields = HashMap::new();
        fields.insert("Prompt", Cow::Borrowed("Question?"));
        fields.insert("QuestionType", Cow::Borrowed("numerical"));
        fields.insert("CorrectAnswer", Cow::Borrowed("42"));
        fields.insert("Difficulty", Cow::Borrowed("9.9")); // Out of bounds

        let err = SourceQuestion::extract_from_card_fields(&fields).unwrap_err();
        assert!(matches!(err, SourceContractError::InvalidDifficulty { .. }));
    }

    #[test]
    fn test_into_practice_item_translation() {
        let mut fields = HashMap::new();
        fields.insert("Prompt", Cow::Borrowed("What is 10 + 20?"));
        fields.insert("QuestionType", Cow::Borrowed("MCQ"));
        fields.insert("Options", Cow::Borrowed("[\"20\", \"30\", \"40\"]"));
        fields.insert("CorrectAnswer", Cow::Borrowed("30"));
        fields.insert("Subject", Cow::Borrowed("mathematics"));
        fields.insert("Topic", Cow::Borrowed("Arithmetic"));
        fields.insert("Exam", Cow::Borrowed("RRB ALP"));
        fields.insert("Year", Cow::Borrowed("2024"));
        fields.insert("SourceQuestionID", Cow::Borrowed("RRB_ALP_Q10"));

        let q = SourceQuestion::extract_from_card_fields(&fields).unwrap();
        let item = q.into_practice_item("test_guid_123");

        assert_eq!(item.id.as_str(), "pi_src_test_guid_123");
        assert_eq!(item.domain, Domain::Mathematics);
        assert_eq!(item.chapter, "General");
        assert_eq!(item.prompt, "What is 10 + 20?");
        assert!(matches!(item.question_type, QuestionType::Mcq { .. }));
        assert_eq!(item.provenance.source_pyq_id, Some(PyqId::new("RRB_ALP_Q10")));
        assert_eq!(item.metadata.get("exam").and_then(|v| v.as_str()), Some("RRB ALP"));
    }
}
