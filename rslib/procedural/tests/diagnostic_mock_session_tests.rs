// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::collections::HashSet;
use chrono::Utc;

use procedural::{
    Domain, ProceduralService,
    DiagnosticHierarchyLevel, render_diagnostic_report_html, render_diagnostic_session_html,
};

#[test]
fn test_diagnostic_mock_session_multi_domain_sampling_and_measuring_mode() {
    let service = ProceduralService::open_in_memory().unwrap();

    for total_q in [10, 16, 20] {
        let session = service.create_diagnostic_session(total_q, 600_000, 4242).unwrap();

        // 1. Validate question count and blueprint
        assert_eq!(session.questions.len(), total_q);
        assert_eq!(session.blueprint.total_questions, total_q);
        assert_eq!(session.blueprint.time_limit_ms, 600_000);
        assert_eq!(session.blueprint.positive_mark_per_question, 1.0);
        // Non-disruptive fixed measuring mode: 0 negative penalty
        assert_eq!(session.blueprint.negative_mark_per_incorrect, 0.0);

        // 2. Validate multi-domain representation across Math, Reasoning, Physics, Chemistry
        let mut domains_found = HashSet::new();
        for q in &session.questions {
            domains_found.insert(q.domain.clone());
            assert!(q.target_time_ms > 0, "Each question must have target time budget");
            assert!(q.difficulty_level >= 1 && q.difficulty_level <= 5, "Difficulty level must be 1-5");
            assert!(!q.schema_title.is_empty(), "Schema title must be populated");
            assert!(!q.instance.rendered_prompt.is_empty(), "Prompt must be rendered");
            assert!(q.instance.metadata.get("chapter").is_some(), "Chapter metadata required for 4-tier hierarchy");
            assert!(q.instance.metadata.get("topic").is_some(), "Topic metadata required for 4-tier hierarchy");
        }

        assert!(domains_found.contains(&Domain::Mathematics), "Must contain Math items");
        assert!(domains_found.contains(&Domain::Reasoning), "Must contain Reasoning items");
        assert!(domains_found.contains(&Domain::Physics), "Must contain Physics items");
        assert!(domains_found.contains(&Domain::Chemistry), "Must contain Chemistry items");
    }
}

#[test]
fn test_diagnostic_session_navigation_answering_and_marking_lifecycle() {
    let service = ProceduralService::open_in_memory().unwrap();
    let mut session = service.create_diagnostic_session(12, 360_000, 9999).unwrap();

    // Initial state
    assert_eq!(session.current_question_index, 0);
    assert!(!session.is_submitted);
    assert_eq!(session.progress_stats(), (0, 0, 12));

    // Answering question 0
    let _q0 = session.get_question(0).unwrap();
    assert!(!session.is_question_answered(0));
    session.record_answer(0, "42", 15_000);
    assert!(session.is_question_answered(0));
    assert_eq!(session.progress_stats(), (1, 0, 12));

    // Marking for review
    assert!(!session.is_question_marked(0));
    assert!(session.toggle_mark_for_review(0));
    assert!(session.is_question_marked(0));
    assert_eq!(session.progress_stats(), (1, 1, 12));
    assert!(!session.toggle_mark_for_review(0));
    assert!(!session.is_question_marked(0));

    // Navigation
    assert!(session.navigate_to(5));
    assert_eq!(session.current_question_index, 5);
    assert_eq!(session.get_current_question().unwrap().question_index, 5);
    assert!(!session.navigate_to(99)); // Out of bounds
}

#[test]
fn test_diagnostic_session_4tier_hierarchy_and_4dimension_error_report() {
    let service = ProceduralService::open_in_memory().unwrap();
    let mut session = service.create_diagnostic_session(16, 480_000, 7777).unwrap();

    // Answer questions:
    // Items 0..10: correct answers (fast)
    // Items 10..12: correct answers (slow -> speed deficit)
    // Items 12..14: calculation error with '-' sign
    // Items 14..16: incorrect answers (concept / transfer)
    for i in 0..16 {
        let q = &session.questions[i];
        let ans_str = if let Some(v) = q.instance.correct_answer.get("formatted").and_then(|f| f.as_str()) {
            v.to_string()
        } else if let Some(v) = q.instance.correct_answer.get("value").and_then(|v| v.as_f64()) {
            v.to_string()
        } else if let Some(s) = q.instance.correct_answer.as_str() {
            s.to_string()
        } else {
            "1.0".to_string()
        };

        if i < 10 {
            session.record_answer(i, ans_str, 20_000); // Correct, fast
        } else if i < 12 {
            session.record_answer(i, ans_str, 75_000); // Correct, slow (> 1.25 * 45s) -> speed deficit
        } else if i < 14 {
            session.record_answer(i, "-9999", 25_000); // Incorrect calculation slip
        } else {
            session.record_answer(i, "wrong_concept_answer", 35_000); // Incorrect concept/transfer
        }
    }

    let report = session.generate_comprehensive_report(Utc::now().timestamp_millis());

    // 1. Scoring & summary integrity
    assert_eq!(report.total_questions, 16);
    assert_eq!(report.answered_count, 16);
    assert_eq!(report.correct_count, 12);
    assert_eq!(report.incorrect_count, 4);
    assert_eq!(report.accuracy, 75.0);

    // 2. 4-Dimension Diagnostic Error Breakdown
    assert!(report.error_distribution.speed_deficit_count >= 2, "Must detect speed deficits");
    assert!(report.error_distribution.calculation_count >= 1, "Must detect calculation errors");
    assert!(
        report.error_distribution.concept_count + report.error_distribution.transfer_count >= 1,
        "Must detect concept/transfer errors"
    );

    // 3. 4-Tier Hierarchy Structure
    assert!(!report.hierarchy.is_empty(), "Hierarchy root must contain Subject nodes");
    for subject_node in &report.hierarchy {
        assert_eq!(subject_node.level, DiagnosticHierarchyLevel::Subject);
        assert!(!subject_node.name.is_empty());
        assert!(subject_node.total_questions > 0);

        for chapter_node in &subject_node.children {
            assert_eq!(chapter_node.level, DiagnosticHierarchyLevel::Chapter);
            assert!(!chapter_node.name.is_empty());

            for topic_node in &chapter_node.children {
                assert_eq!(topic_node.level, DiagnosticHierarchyLevel::Topic);
                assert!(!topic_node.name.is_empty());

                for family_node in &topic_node.children {
                    assert_eq!(family_node.level, DiagnosticHierarchyLevel::ProblemFamily);
                    assert!(!family_node.name.is_empty());
                }
            }
        }
    }

    // 4. Follow-up recommendation
    assert!(matches!(report.recommended_follow_up.objective, procedural::PracticeObjective::Practice | procedural::PracticeObjective::Speed | procedural::PracticeObjective::Transfer));
}

#[test]
fn test_diagnostic_evidence_sync_to_procedural_store() {
    let service = ProceduralService::open_in_memory().unwrap();
    let mut session = service.create_diagnostic_session(12, 360_000, 5555).unwrap();

    for i in 0..12 {
        let q = &session.questions[i];
        let ans_str = if let Some(v) = q.instance.correct_answer.get("value").and_then(|v| v.as_f64()) {
            v.to_string()
        } else {
            "0".to_string()
        };

        if i < 8 {
            session.record_answer(i, ans_str, 20_000);
        } else {
            session.record_answer(i, "wrong_attempt", 30_000);
        }
    }

    let report = session.generate_comprehensive_report(Utc::now().timestamp_millis());

    // Batch-sync evidence to ProceduralStore without parallel duplicate models
    let updated_states = service.record_diagnostic_report_evidence(&session, &report).unwrap();
    assert_eq!(updated_states.len(), 12, "Must update state for all 12 questions");

    // Verify persisted skill states in SQLite store
    for q in &session.questions {
        let state = service.store().get_skill_state(&q.skill_id).unwrap();
        assert!(state.is_some(), "SkillState must be persisted in skill_states table for {}", q.skill_id);
        let s = state.unwrap();
        assert!(s.total_attempts >= 1, "Must increment total_attempts");
    }
}

#[test]
fn test_diagnostic_html_rendering_session_and_report() {
    let service = ProceduralService::open_in_memory().unwrap();
    let mut session = service.create_diagnostic_session(10, 300_000, 1111).unwrap();
    session.record_answer(0, "42", 12_000);

    // 1. Session HTML
    let session_html = render_diagnostic_session_html(&session);
    assert!(session_html.contains("Diagnostic Mode"));
    assert!(session_html.contains("diagTimer"));
    assert!(session_html.contains("diagPaletteGrid"));
    assert!(session_html.contains("diagQuestionCard"));
    assert!(session_html.contains("diagnostic-session-data"));

    // 2. Report HTML
    let report = session.generate_comprehensive_report(Utc::now().timestamp_millis());
    let report_html = render_diagnostic_report_html(&report);
    assert!(report_html.contains("Diagnostic Assessment Report"));
    assert!(report_html.contains("Concept Errors"));
    assert!(report_html.contains("Execution / Calc"));
    assert!(report_html.contains("Transfer Deficits"));
    assert!(report_html.contains("Speed Deficits"));
    assert!(report_html.contains("hierarchyContainer"));
    assert!(report_html.contains("diagWeakSkillsList"));
    assert!(report_html.contains("startRemediationBtn"));
    assert!(report_html.contains("diagnostic-report-data"));
}
