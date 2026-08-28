// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::borrow::Cow;
use std::collections::HashMap;

use procedural::anchor::source::{CanonicalQuestionType, SourceContractError, SourceQuestion};
use procedural::content::item::QuestionType;
use procedural::core::{Domain, PyqId};
use procedural::service::ProceduralService;
use procedural::storage::store::ProceduralStore;

#[test]
fn test_canonical_mcq_exact_match() {
    let mut fields = HashMap::new();
    fields.insert("Prompt", Cow::Borrowed("Which planet is closest to the Sun?"));
    fields.insert("QuestionType", Cow::Borrowed("MCQ"));
    fields.insert("Options", Cow::Borrowed("[\"Venus\", \"Mercury\", \"Earth\", \"Mars\"]"));
    fields.insert("CorrectAnswer", Cow::Borrowed("Mercury"));
    fields.insert("Subject", Cow::Borrowed("physics"));
    fields.insert("Chapter", Cow::Borrowed("Astronomy"));
    fields.insert("Topic", Cow::Borrowed("Solar System"));
    fields.insert("Difficulty", Cow::Borrowed("1.5"));
    fields.insert("Explanation", Cow::Borrowed("Mercury is the innermost planet."));

    let q = SourceQuestion::extract_from_card_fields(&fields).expect("Failed to parse valid canonical MCQ");
    assert_eq!(q.prompt, "Which planet is closest to the Sun?");
    assert_eq!(q.question_type, CanonicalQuestionType::Mcq);
    assert_eq!(q.correct_answer, "Mercury");
    assert_eq!(q.options.as_ref().unwrap().len(), 4);
    assert_eq!(q.difficulty, Some(1.5));
    assert_eq!(q.explanation, Some("Mercury is the innermost planet.".to_string()));

    let item = q.into_practice_item("note_guid_001");
    assert_eq!(item.id.as_str(), "pi_src_note_guid_001");
    assert_eq!(item.domain, Domain::Physics);
    assert_eq!(item.chapter, "Astronomy");
    match &item.question_type {
        QuestionType::Mcq { options, correct_option, explanation } => {
            assert_eq!(options.len(), 4);
            assert_eq!(correct_option, "Mercury");
            assert_eq!(explanation.as_deref(), Some("Mercury is the innermost planet."));
        }
        _ => panic!("Expected MCQ question type"),
    }
}

#[test]
fn test_canonical_mcq_letter_index_resolution() {
    let mut fields = HashMap::new();
    fields.insert("Prompt", Cow::Borrowed("What is 2^3?"));
    fields.insert("QuestionType", Cow::Borrowed("multiple_choice"));
    fields.insert("Options", Cow::Borrowed("[\"6\", \"8\", \"9\", \"12\"]"));
    fields.insert("CorrectAnswer", Cow::Borrowed("B")); // 2nd option -> "8"

    let q = SourceQuestion::extract_from_card_fields(&fields).expect("Failed to parse MCQ with letter answer");
    assert_eq!(q.correct_answer, "B");

    let item = q.into_practice_item("note_guid_002");
    match &item.question_type {
        QuestionType::Mcq { correct_option, .. } => {
            assert_eq!(correct_option, "8", "Letter 'B' should resolve to second option '8'");
        }
        _ => panic!("Expected MCQ question type"),
    }
}

#[test]
fn test_canonical_numerical_parsing() {
    let mut fields = HashMap::new();
    fields.insert("Prompt", Cow::Borrowed("Calculate the square of 25."));
    fields.insert("QuestionType", Cow::Borrowed("numerical"));
    fields.insert("CorrectAnswer", Cow::Borrowed("625.0"));
    fields.insert("Subject", Cow::Borrowed("mathematics"));
    fields.insert("Difficulty", Cow::Borrowed("2.0"));

    let q = SourceQuestion::extract_from_card_fields(&fields).expect("Failed to parse canonical numerical question");
    assert_eq!(q.question_type, CanonicalQuestionType::Numerical);
    assert_eq!(q.options, None);
    assert_eq!(q.correct_answer, "625.0");

    let item = q.into_practice_item("note_guid_num_01");
    match item.question_type {
        QuestionType::Numerical { answer, tolerance } => {
            assert_eq!(answer, 625.0);
            assert_eq!(tolerance, None);
        }
        _ => panic!("Expected Numerical question type"),
    }
}

#[test]
fn test_canonical_optional_fields_absence() {
    let mut fields = HashMap::new();
    fields.insert("Prompt", Cow::Borrowed("Minimal Prompt"));
    fields.insert("QuestionType", Cow::Borrowed("numerical"));
    fields.insert("CorrectAnswer", Cow::Borrowed("42"));

    let q = SourceQuestion::extract_from_card_fields(&fields).expect("Failed to parse minimal canonical question");
    assert_eq!(q.prompt, "Minimal Prompt");
    assert_eq!(q.question_type, CanonicalQuestionType::Numerical);
    assert_eq!(q.correct_answer, "42");
    assert_eq!(q.hint, None);
    assert_eq!(q.solution, None);
    assert_eq!(q.steps, None);
    assert_eq!(q.explanation, None);
    assert_eq!(q.subject, None);
    assert_eq!(q.chapter, None);
    assert_eq!(q.topic, None);
    assert_eq!(q.skill, None);
    assert_eq!(q.problem_type, None);
    assert_eq!(q.difficulty, None);
    assert_eq!(q.source, None);
    assert_eq!(q.exam, None);
    assert_eq!(q.year, None);
    assert_eq!(q.shift, None);
    assert_eq!(q.paper, None);
    assert_eq!(q.source_question_id, None);

    let item = q.into_practice_item("note_guid_min");
    assert_eq!(item.domain, Domain::Mathematics); // Default fallback
    assert_eq!(item.chapter, "General");
    assert_eq!(item.difficulty, 3.0);
}

#[test]
fn test_canonical_complete_provenance() {
    let mut fields = HashMap::new();
    fields.insert("Prompt", Cow::Borrowed("Official Exam Question"));
    fields.insert("QuestionType", Cow::Borrowed("mcq"));
    fields.insert("Options", Cow::Borrowed("[\"Option A\", \"Option B\"]"));
    fields.insert("CorrectAnswer", Cow::Borrowed("Option A"));
    fields.insert("Source", Cow::Borrowed("RRB Official Papers"));
    fields.insert("Exam", Cow::Borrowed("RRB ALP"));
    fields.insert("Year", Cow::Borrowed("2024"));
    fields.insert("Shift", Cow::Borrowed("Shift 2"));
    fields.insert("Paper", Cow::Borrowed("Paper 1 (CBT-1)"));
    fields.insert("SourceQuestionID", Cow::Borrowed("ALP_2024_S2_Q15"));

    let q = SourceQuestion::extract_from_card_fields(&fields).expect("Failed to parse full provenance");
    assert_eq!(q.source.as_deref(), Some("RRB Official Papers"));
    assert_eq!(q.exam.as_deref(), Some("RRB ALP"));
    assert_eq!(q.year, Some(2024));
    assert_eq!(q.shift.as_deref(), Some("Shift 2"));
    assert_eq!(q.paper.as_deref(), Some("Paper 1 (CBT-1)"));
    assert_eq!(q.source_question_id.as_deref(), Some("ALP_2024_S2_Q15"));

    let item = q.into_practice_item("note_guid_prov");
    assert_eq!(item.provenance.source_pyq_id, Some(PyqId::new("ALP_2024_S2_Q15")));
    assert_eq!(item.metadata.get("exam").and_then(|v| v.as_str()), Some("RRB ALP"));
    assert_eq!(item.metadata.get("year").and_then(|v| v.as_i64()), Some(2024));
    assert_eq!(item.metadata.get("shift").and_then(|v| v.as_str()), Some("Shift 2"));
    assert_eq!(item.metadata.get("paper").and_then(|v| v.as_str()), Some("Paper 1 (CBT-1)"));
    assert_eq!(item.metadata.get("source_question_id").and_then(|v| v.as_str()), Some("ALP_2024_S2_Q15"));
}

#[test]
fn test_canonical_difficulty_validation() {
    let mut fields = HashMap::new();
    fields.insert("Prompt", Cow::Borrowed("Diff Question"));
    fields.insert("QuestionType", Cow::Borrowed("numerical"));
    fields.insert("CorrectAnswer", Cow::Borrowed("10"));

    // Valid difficulties: 1.0, 3.5, 5.0
    for valid_diff in ["1.0", "3.5", "5.0"] {
        fields.insert("Difficulty", Cow::Borrowed(valid_diff));
        let q = SourceQuestion::extract_from_card_fields(&fields).unwrap();
        assert_eq!(q.difficulty, Some(valid_diff.parse::<f64>().unwrap()));
    }

    // Invalid difficulties: 0.9, 5.1, "hard", "-1"
    for invalid_diff in ["0.9", "5.1", "hard", "-1.0"] {
        fields.insert("Difficulty", Cow::Borrowed(invalid_diff));
        let err = SourceQuestion::extract_from_card_fields(&fields).unwrap_err();
        assert!(matches!(err, SourceContractError::InvalidDifficulty { .. }));
    }
}

#[test]
fn test_canonical_missing_required_fields() {
    // 1. Missing Prompt
    let mut fields = HashMap::new();
    fields.insert("QuestionType", Cow::Borrowed("numerical"));
    fields.insert("CorrectAnswer", Cow::Borrowed("42"));
    let err = SourceQuestion::extract_from_card_fields(&fields).unwrap_err();
    assert!(matches!(err, SourceContractError::MissingRequiredField { field_name: "Prompt", .. }));

    // 2. Missing QuestionType
    let mut fields = HashMap::new();
    fields.insert("Prompt", Cow::Borrowed("Some prompt"));
    fields.insert("CorrectAnswer", Cow::Borrowed("42"));
    let err = SourceQuestion::extract_from_card_fields(&fields).unwrap_err();
    assert!(matches!(err, SourceContractError::MissingRequiredField { field_name: "QuestionType", .. }));

    // 3. Missing CorrectAnswer
    let mut fields = HashMap::new();
    fields.insert("Prompt", Cow::Borrowed("Some prompt"));
    fields.insert("QuestionType", Cow::Borrowed("numerical"));
    let err = SourceQuestion::extract_from_card_fields(&fields).unwrap_err();
    assert!(matches!(err, SourceContractError::MissingRequiredField { field_name: "CorrectAnswer", .. }));
}

#[test]
fn test_canonical_unsupported_question_type() {
    let mut fields = HashMap::new();
    fields.insert("Prompt", Cow::Borrowed("Write an essay on photosynthesis."));
    fields.insert("QuestionType", Cow::Borrowed("essay"));
    fields.insert("CorrectAnswer", Cow::Borrowed("Photosynthesis is..."));

    let err = SourceQuestion::extract_from_card_fields(&fields).unwrap_err();
    assert!(matches!(err, SourceContractError::InvalidQuestionType { raw_value, .. } if raw_value == "essay"));
}

#[test]
fn test_canonical_mcq_options_validation() {
    // 1. Missing Options for MCQ
    let mut fields = HashMap::new();
    fields.insert("Prompt", Cow::Borrowed("MCQ prompt"));
    fields.insert("QuestionType", Cow::Borrowed("mcq"));
    fields.insert("CorrectAnswer", Cow::Borrowed("A"));
    let err = SourceQuestion::extract_from_card_fields(&fields).unwrap_err();
    assert!(matches!(err, SourceContractError::MissingMcqOptions { .. }));

    // 2. Only 1 option provided
    fields.insert("Options", Cow::Borrowed("[\"Single Option\"]"));
    let err = SourceQuestion::extract_from_card_fields(&fields).unwrap_err();
    assert!(matches!(err, SourceContractError::MissingMcqOptions { .. }));

    // 3. CorrectAnswer does not match options
    fields.insert("Options", Cow::Borrowed("[\"Option 1\", \"Option 2\"]"));
    fields.insert("CorrectAnswer", Cow::Borrowed("Option 3"));
    let err = SourceQuestion::extract_from_card_fields(&fields).unwrap_err();
    assert!(matches!(err, SourceContractError::InvalidCorrectAnswer { .. }));
}

#[test]
fn test_canonical_numerical_invalid_number() {
    let mut fields = HashMap::new();
    fields.insert("Prompt", Cow::Borrowed("Numerical prompt"));
    fields.insert("QuestionType", Cow::Borrowed("numerical"));
    fields.insert("CorrectAnswer", Cow::Borrowed("NotANumber"));

    let err = SourceQuestion::extract_from_card_fields(&fields).unwrap_err();
    assert!(matches!(err, SourceContractError::InvalidCorrectAnswer { .. }));
}

#[test]
fn test_canonical_deterministic_reconciliation() {
    let store = ProceduralStore::open_in_memory().unwrap();
    let service = ProceduralService::new(store);

    let guid = "anki_guid_det_100".to_string();

    let mut fields = HashMap::new();
    fields.insert("Prompt", Cow::Borrowed("Initial question prompt"));
    fields.insert("QuestionType", Cow::Borrowed("mcq"));
    fields.insert("Options", Cow::Borrowed("[\"A\", \"B\", \"C\"]"));
    fields.insert("CorrectAnswer", Cow::Borrowed("A"));
    fields.insert("Subject", Cow::Borrowed("reasoning"));
    fields.insert("Topic", Cow::Borrowed("Logic"));

    let q1 = SourceQuestion::extract_from_card_fields(&fields).unwrap();

    // 1. Initial import -> 1 NEW
    let rep1 = service.reconcile_source_questions(vec![(guid.clone(), q1.clone())]).unwrap();
    assert_eq!(rep1.new_count, 1);
    assert_eq!(rep1.unchanged_count, 0);

    // 2. Re-import identical -> 1 UNCHANGED
    let rep2 = service.reconcile_source_questions(vec![(guid.clone(), q1.clone())]).unwrap();
    assert_eq!(rep2.new_count, 0);
    assert_eq!(rep2.unchanged_count, 1);

    // 3. Update content -> 1 UPDATED
    fields.insert("Prompt", Cow::Borrowed("Modified question prompt text"));
    let q2 = SourceQuestion::extract_from_card_fields(&fields).unwrap();
    let rep3 = service.reconcile_source_questions(vec![(guid.clone(), q2)]).unwrap();
    assert_eq!(rep3.updated_count, 1);
    assert_eq!(rep3.unchanged_count, 0);

    // 4. Resolve target after reconciliation
    let session = service.resolve_source_target(&guid, Some(101)).unwrap();
    assert_eq!(session.instance.rendered_prompt, "Modified question prompt text");
    assert_eq!(session.card_id, Some(101));
}

#[test]
fn test_adversarial_question_type_never_inferred_from_options() {
    let mut fields = HashMap::new();
    fields.insert("Prompt", Cow::Borrowed("Adversarial Question"));
    fields.insert("Options", Cow::Borrowed("[\"A\", \"B\", \"C\", \"D\"]"));
    fields.insert("CorrectAnswer", Cow::Borrowed("A"));

    // 1. QuestionType is unsupported string (must NOT become MCQ even though Options are present)
    for bad_type in ["essay", "freeform", "unknown", "garbage", "text", "checkbox", "123"] {
        fields.insert("QuestionType", Cow::Borrowed(bad_type));
        let err = SourceQuestion::extract_from_card_fields(&fields).unwrap_err();
        assert!(
            matches!(err, SourceContractError::InvalidQuestionType { ref raw_value, .. } if raw_value == bad_type),
            "Expected InvalidQuestionType for '{}', got {:?}", bad_type, err
        );
    }

    // 2. QuestionType is empty string (must NOT become MCQ)
    fields.insert("QuestionType", Cow::Borrowed("   "));
    let err = SourceQuestion::extract_from_card_fields(&fields).unwrap_err();
    assert!(matches!(err, SourceContractError::MissingRequiredField { field_name: "QuestionType", .. }));
}

#[test]
fn test_adversarial_mcq_multiline_options_and_resolutions() {
    let mut fields = HashMap::new();
    fields.insert("Prompt", Cow::Borrowed("Choose the correct color"));
    fields.insert("QuestionType", Cow::Borrowed("MCQ"));
    // Newline-separated options with surrounding whitespace
    fields.insert("Options", Cow::Borrowed("  Red  \n  Green  \n  Blue  \n  Yellow  "));

    // 1. Letter index resolution ('A'/'a', 'B'/'b', 'C'/'c', 'D'/'d')
    fields.insert("CorrectAnswer", Cow::Borrowed("c"));
    let q = SourceQuestion::extract_from_card_fields(&fields).unwrap();
    assert_eq!(q.options.as_ref().unwrap(), &vec!["Red", "Green", "Blue", "Yellow"]);
    let item = q.into_practice_item("guid_mcq_adv_1");
    match &item.question_type {
        QuestionType::Mcq { correct_option, .. } => assert_eq!(correct_option, "Blue"),
        _ => panic!("Expected MCQ"),
    }

    // 2. Numeric 1-based index resolution ("1", "2", "3", "4")
    fields.insert("CorrectAnswer", Cow::Borrowed("2"));
    let q2 = SourceQuestion::extract_from_card_fields(&fields).unwrap();
    let item2 = q2.into_practice_item("guid_mcq_adv_2");
    match &item2.question_type {
        QuestionType::Mcq { correct_option, .. } => assert_eq!(correct_option, "Green"),
        _ => panic!("Expected MCQ"),
    }

    // 3. Exact text match
    fields.insert("CorrectAnswer", Cow::Borrowed("Yellow"));
    let q3 = SourceQuestion::extract_from_card_fields(&fields).unwrap();
    let item3 = q3.into_practice_item("guid_mcq_adv_3");
    match &item3.question_type {
        QuestionType::Mcq { correct_option, .. } => assert_eq!(correct_option, "Yellow"),
        _ => panic!("Expected MCQ"),
    }

    // 4. Invalid index ('E', '5', '0', 'Z') -> Must fail validation
    for invalid_ans in ["E", "e", "5", "0", "Z", "Orange", "99"] {
        fields.insert("CorrectAnswer", Cow::Borrowed(invalid_ans));
        let err = SourceQuestion::extract_from_card_fields(&fields).unwrap_err();
        assert!(matches!(err, SourceContractError::InvalidCorrectAnswer { .. }));
    }

    // 5. Options with empty/whitespace elements filtered out
    fields.insert("Options", Cow::Borrowed("[\"   \", \"Valid Option 1\", \" \", \"Valid Option 2\"]"));
    fields.insert("CorrectAnswer", Cow::Borrowed("Valid Option 1"));
    let q_clean = SourceQuestion::extract_from_card_fields(&fields).unwrap();
    assert_eq!(q_clean.options.as_ref().unwrap().len(), 2);

    // 6. Fewer than 2 non-empty options after filtering -> Fails
    fields.insert("Options", Cow::Borrowed("[\"   \", \"Only One Valid Option\"]"));
    let err_too_few = SourceQuestion::extract_from_card_fields(&fields).unwrap_err();
    assert!(matches!(err_too_few, SourceContractError::MissingMcqOptions { .. }));
}

#[test]
fn test_adversarial_numerical_scientific_negative_and_invalid_tokens() {
    let mut fields = HashMap::new();
    fields.insert("Prompt", Cow::Borrowed("Calculate scientific quantity"));
    fields.insert("QuestionType", Cow::Borrowed("numerical"));

    // 1. Valid numbers (integers, decimals, negatives, scientific notation, leading plus)
    for (valid_raw, expected_val) in [
        ("42", 42.0),
        ("3.14159265", 3.14159265),
        ("-273.15", -273.15),
        ("1.5e4", 15000.0),
        ("-3.2e-2", -0.032),
        ("+100.5", 100.5),
        ("0.0", 0.0),
        ("  -0.005  ", -0.005),
    ] {
        fields.insert("CorrectAnswer", Cow::Borrowed(valid_raw));
        let q = SourceQuestion::extract_from_card_fields(&fields).unwrap();
        assert_eq!(q.question_type, CanonicalQuestionType::Numerical);
        assert_eq!(q.options, None);
        let item = q.into_practice_item("guid_num_adv");
        match item.question_type {
            QuestionType::Numerical { answer, .. } => assert!((answer - expected_val).abs() < 1e-6),
            _ => panic!("Expected Numerical"),
        }
    }

    // 2. Invalid numbers (non-finite strings, malformed syntax, units attached, text)
    for invalid_raw in [
        "NaN",
        "nan",
        "inf",
        "Infinity",
        "-inf",
        "-Infinity",
        "12.34.56",
        "--5",
        "42 Newtons",
        "42kg",
        "five",
        "1e",
        "e5",
    ] {
        fields.insert("CorrectAnswer", Cow::Borrowed(invalid_raw));
        let err = SourceQuestion::extract_from_card_fields(&fields).unwrap_err();
        assert!(
            matches!(err, SourceContractError::InvalidCorrectAnswer { .. }),
            "Expected InvalidCorrectAnswer for '{}', got {:?}", invalid_raw, err
        );
    }

    // 3. Empty or whitespace-only CorrectAnswer must fail as MissingRequiredField
    for empty_ans in ["", "   "] {
        fields.insert("CorrectAnswer", Cow::Borrowed(empty_ans));
        let err = SourceQuestion::extract_from_card_fields(&fields).unwrap_err();
        assert!(
            matches!(err, SourceContractError::MissingRequiredField { field_name: "CorrectAnswer", .. }),
            "Expected MissingRequiredField for '{}', got {:?}", empty_ans, err
        );
    }
}

#[test]
fn test_adversarial_difficulty_strict_bounds() {
    let mut fields = HashMap::new();
    fields.insert("Prompt", Cow::Borrowed("Difficulty Boundary Test"));
    fields.insert("QuestionType", Cow::Borrowed("numerical"));
    fields.insert("CorrectAnswer", Cow::Borrowed("10"));

    // 1. Valid Difficulty values [1.0, 5.0]
    for valid_val in ["1.0", "1.000", "2.5", "3.14159", "4.999", "5.0", "  3.0  "] {
        fields.insert("Difficulty", Cow::Borrowed(valid_val));
        let q = SourceQuestion::extract_from_card_fields(&fields).unwrap();
        assert!(q.difficulty.is_some());
        let d = q.difficulty.unwrap();
        assert!(d >= 1.0 && d <= 5.0);
    }

    // 2. Out of bounds (below 1.0 or above 5.0) and non-finite / strings
    for invalid_val in [
        "0.9999",
        "0.0",
        "-1.0",
        "-5.0",
        "5.0001",
        "5.1",
        "10.0",
        "100.0",
        "NaN",
        "inf",
        "Infinity",
        "easy",
        "hard",
        "medium",
        "level 1",
    ] {
        fields.insert("Difficulty", Cow::Borrowed(invalid_val));
        let err = SourceQuestion::extract_from_card_fields(&fields).unwrap_err();
        assert!(
            matches!(err, SourceContractError::InvalidDifficulty { .. }),
            "Expected InvalidDifficulty for '{}', got {:?}", invalid_val, err
        );
    }
}

#[test]
fn test_adversarial_provenance_year_and_optional_fields() {
    let mut fields = HashMap::new();
    fields.insert("Prompt", Cow::Borrowed("Provenance Test"));
    fields.insert("QuestionType", Cow::Borrowed("numerical"));
    fields.insert("CorrectAnswer", Cow::Borrowed("100"));

    // 1. Valid integer years
    for yr in ["2024", "1999", "2018", "2025", "  2023  "] {
        fields.insert("Year", Cow::Borrowed(yr));
        let q = SourceQuestion::extract_from_card_fields(&fields).unwrap();
        assert_eq!(q.year, Some(yr.trim().parse::<i32>().unwrap()));
    }

    // 2. Invalid years
    for bad_yr in ["twenty-twenty-four", "2024.5", "abc", "2024-01-01", ""] {
        if bad_yr.is_empty() {
            fields.insert("Year", Cow::Borrowed(bad_yr));
            let q = SourceQuestion::extract_from_card_fields(&fields).unwrap();
            assert_eq!(q.year, None);
        } else {
            fields.insert("Year", Cow::Borrowed(bad_yr));
            let err = SourceQuestion::extract_from_card_fields(&fields).unwrap_err();
            assert!(matches!(err, SourceContractError::InvalidProvenance { field_name: "Year", .. }));
        }
    }
}

#[test]
fn test_adversarial_duplicate_source_question_id_distinct_guids() {
    let store = ProceduralStore::open_in_memory().unwrap();
    let service = ProceduralService::new(store);

    let mut fields1 = HashMap::new();
    fields1.insert("Prompt", Cow::Borrowed("Deck A Question"));
    fields1.insert("QuestionType", Cow::Borrowed("mcq"));
    fields1.insert("Options", Cow::Borrowed("[\"A\", \"B\"]"));
    fields1.insert("CorrectAnswer", Cow::Borrowed("A"));
    fields1.insert("SourceQuestionID", Cow::Borrowed("SHARED_PYQ_001"));

    let mut fields2 = HashMap::new();
    fields2.insert("Prompt", Cow::Borrowed("Deck B Question"));
    fields2.insert("QuestionType", Cow::Borrowed("mcq"));
    fields2.insert("Options", Cow::Borrowed("[\"A\", \"B\"]"));
    fields2.insert("CorrectAnswer", Cow::Borrowed("B"));
    fields2.insert("SourceQuestionID", Cow::Borrowed("SHARED_PYQ_001")); // Same SourceQuestionID

    let q1 = SourceQuestion::extract_from_card_fields(&fields1).unwrap();
    let q2 = SourceQuestion::extract_from_card_fields(&fields2).unwrap();

    let guid1 = "anki_guid_deck_a_001".to_string();
    let guid2 = "anki_guid_deck_b_002".to_string();

    let report = service.reconcile_source_questions(vec![
        (guid1.clone(), q1),
        (guid2.clone(), q2),
    ]).unwrap();

    assert_eq!(report.new_count, 2, "Both distinct notes must be inserted without collision");

    // Both should be resolvable independently by their Note GUIDs
    let sess1 = service.resolve_source_target(&guid1, Some(1)).unwrap();
    let sess2 = service.resolve_source_target(&guid2, Some(2)).unwrap();

    assert_eq!(sess1.instance.rendered_prompt, "Deck A Question");
    assert_eq!(sess2.instance.rendered_prompt, "Deck B Question");
}

#[test]
fn test_adversarial_learner_state_firewall_source_immutability() {
    use serde_json::json;

    let store = ProceduralStore::open_in_memory().unwrap();
    let service = ProceduralService::new(store);

    let guid = "anki_guid_firewall_test".to_string();
    let mut fields = HashMap::new();
    fields.insert("Prompt", Cow::Borrowed("Immutable Source Prompt"));
    fields.insert("QuestionType", Cow::Borrowed("mcq"));
    fields.insert("Options", Cow::Borrowed("[\"Alpha\", \"Beta\", \"Gamma\"]"));
    fields.insert("CorrectAnswer", Cow::Borrowed("Alpha"));
    fields.insert("Difficulty", Cow::Borrowed("2.5"));
    fields.insert("Subject", Cow::Borrowed("physics"));
    fields.insert("Chapter", Cow::Borrowed("Mechanics"));
    fields.insert("Topic", Cow::Borrowed("Kinematics"));
    fields.insert("Exam", Cow::Borrowed("JEE Main"));
    fields.insert("Year", Cow::Borrowed("2024"));
    fields.insert("SourceQuestionID", Cow::Borrowed("JEE_2024_PHY_01"));

    let q = SourceQuestion::extract_from_card_fields(&fields).unwrap();
    service.reconcile_source_questions(vec![(guid.clone(), q)]).unwrap();

    // Resolve target and get instance
    let session = service.resolve_source_target(&guid, Some(42)).unwrap();
    let instance_id = &session.instance.id;

    // Simulate 3 user attempts (wrong, then right, then wrong)
    let _ = service.evaluate_and_record_attempt(
        instance_id,
        Some(42),
        json!({"value": "Beta"}), // Wrong
        18000,
        0,
        1,
    ).unwrap();

    let _ = service.evaluate_and_record_attempt(
        instance_id,
        Some(42),
        json!({"value": "Alpha"}), // Correct
        12000,
        0,
        1,
    ).unwrap();

    // Verify: PracticeItem stored in SQLite database is 100% UNCHANGED
    let item_id = SourceQuestion::stable_id_from_guid(&guid);
    let stored_item = service.store().get_practice_item(&item_id).unwrap().unwrap();

    assert_eq!(stored_item.prompt, "Immutable Source Prompt");
    assert_eq!(stored_item.difficulty, 2.5, "Difficulty must remain 2.5 despite learner failures and successes");
    match stored_item.question_type {
        QuestionType::Mcq { ref options, ref correct_option, .. } => {
            assert_eq!(options, &vec!["Alpha", "Beta", "Gamma"]);
            assert_eq!(correct_option, "Alpha");
        }
        _ => panic!("Expected MCQ"),
    }
    assert_eq!(stored_item.chapter, "Mechanics");
    assert_eq!(stored_item.domain, Domain::Physics);
    assert_eq!(stored_item.metadata.get("exam").and_then(|v| v.as_str()), Some("JEE Main"));
    assert_eq!(stored_item.metadata.get("year").and_then(|v| v.as_i64()), Some(2024));
    assert_eq!(stored_item.metadata.get("source_question_id").and_then(|v| v.as_str()), Some("JEE_2024_PHY_01"));
}

