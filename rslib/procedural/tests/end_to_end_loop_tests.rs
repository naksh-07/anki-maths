// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use procedural::anchor::ProceduralCardAnchor;
use procedural::diagnostics::ErrorCategory;
use procedural::scheduling::{Rating, SessionReadiness};
use procedural::skills::PracticeProgressionState;
use procedural::service::ProceduralService;

#[test]
fn test_closed_learning_loop_end_to_end() {
    // 1. Initialize clean service and default catalog in memory
    let service = ProceduralService::open_in_memory().expect("open in memory");

    // 2. Anki Card references a procedural schema
    let card_id = 999999;
    let anchor = ProceduralCardAnchor::new("percentage.successive");

    // 3. Prepare first practice session (cold start -> chooses standard ForwardTwoStep)
    let session1 = service
        .prepare_practice_session(&anchor, Some(card_id))
        .expect("prepare session 1");

    assert_eq!(session1.readiness, SessionReadiness::Ready);
    assert_eq!(session1.card_id, Some(card_id));
    assert_eq!(session1.selected_variant.as_deref(), Some("forward_two_step"));
    assert_eq!(session1.selection_reason.as_deref(), Some("cold_start_standard_variant"));
    assert!(session1.target_latency_ms.is_some());

    // 4. Extract correct answer from generated instance
    let ans_val_1 = session1
        .instance
        .correct_answer
        .get("value")
        .unwrap()
        .as_f64()
        .unwrap();

    // 5. Learner submits correct answer with fast latency (18s on 35s target)
    let outcome1 = service
        .evaluate_and_record_attempt(
            &session1.instance.id,
            session1.card_id,
            serde_json::json!(ans_val_1),
            18_000,
            0,
            1,
        )
        .expect("evaluate attempt 1");

    assert!(outcome1.is_correct);
    assert_eq!(outcome1.score, 1.0);
    assert_eq!(outcome1.error_category, None);

    // 6. Derive calibrated FSRS rating
    let rating1 = service.derive_fsrs_rating(&outcome1).expect("derive rating 1");
    // Fast + correct on cold start maps to Easy or Good
    assert!(matches!(rating1, Rating::Good | Rating::Easy));

    // 7. Verify SkillState was updated with rich learning signals
    let state1 = service
        .load_skill_state(&outcome1.skill_id)
        .expect("load skill state")
        .expect("state exists");
    assert_eq!(state1.total_attempts, 1);
    assert_eq!(state1.successful_attempts, 1);
    assert_eq!(state1.consecutive_successes, 1);
    assert_eq!(state1.recent_accuracy(), 1.0);
    assert_eq!(state1.practice_state, PracticeProgressionState::Learning);

    // 8. Prepare second practice session (learner progresses)
    let session2 = service
        .prepare_practice_session(&anchor, Some(card_id))
        .expect("prepare session 2");

    assert_eq!(session2.card_id, Some(card_id));
    let ans_val_2 = session2
        .instance
        .correct_answer
        .get("value")
        .unwrap()
        .as_f64()
        .unwrap();

    // 9. Learner submits another fast correct answer (8s on 15s target)
    let outcome2 = service
        .evaluate_and_record_attempt(
            &session2.instance.id,
            session2.card_id,
            serde_json::json!(ans_val_2),
            8_000,
            0,
            1,
        )
        .expect("evaluate attempt 2");

    let rating2 = service.derive_fsrs_rating(&outcome2).expect("derive rating 2");
    assert_eq!(rating2, Rating::Easy);

    // 10. Third session: learner makes a careless calculation error on ReverseInitial
    let next_decision = service.select_next_variant(&outcome2.skill_id, 777).expect("next variant");
    assert!(next_decision.target_time_ms > 0);

    let session3 = service
        .prepare_practice_session(&anchor, Some(card_id))
        .expect("prepare session 3");

    let fail_outcome = service
        .evaluate_and_record_attempt(
            &session3.instance.id,
            session3.card_id,
            serde_json::json!("-12345"),
            30_000,
            0,
            1,
        )
        .expect("evaluate fail attempt");

    assert!(!fail_outcome.is_correct);
    assert_eq!(fail_outcome.score, 0.0);

    // FSRS rating for failure must be Again
    let fail_rating = service.derive_fsrs_rating(&fail_outcome).expect("derive fail rating");
    assert_eq!(fail_rating, Rating::Again);

    // 11. Verify state reflects the failure and updates consecutive counters
    let state3 = service
        .load_skill_state(&fail_outcome.skill_id)
        .expect("load state 3")
        .expect("state exists");
    assert_eq!(state3.total_attempts, 3);
    assert_eq!(state3.successful_attempts, 2);
    assert_eq!(state3.failed_attempts, 1);
    assert_eq!(state3.consecutive_failures, 1);
    assert_eq!(state3.consecutive_successes, 0);
    assert_eq!(state3.error_counts.get_count(&ErrorCategory::Unknown), 1);

    // 12. Next session generation responds to recent error
    let session4 = service
        .prepare_practice_session(&anchor, Some(card_id))
        .expect("prepare session 4");
    assert!(session4.selected_variant.is_some());
    assert!(session4.selection_reason.is_some());
}
