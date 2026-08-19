// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use procedural::practice::SessionBudget;
use procedural::scheduling::workload::{
    SessionBudgetTracker, WorkloadSafeguards, WorkloadSnapshot, WorkloadState,
};

#[test]
fn test_workload_state_classification() {
    // 1. Clean queue -> Sustainable
    let snap_clean = WorkloadSnapshot {
        pending_remediation_count: 0,
        critical_remediation_count: 0,
        due_memory_reviews: 5,
        active_learning_skills: 2,
        transfer_pending_count: 1,
        total_composite_load: 8,
    };
    assert_eq!(snap_clean.compute_state(), WorkloadState::Sustainable);

    // 2. Moderate backlog -> Heavy
    let snap_heavy = WorkloadSnapshot {
        pending_remediation_count: 4,
        critical_remediation_count: 1,
        due_memory_reviews: 10,
        active_learning_skills: 4,
        transfer_pending_count: 2,
        total_composite_load: 20,
    };
    assert_eq!(snap_heavy.compute_state(), WorkloadState::Heavy);

    // 3. High critical remediation / large backlog -> Overloaded
    let snap_overload = WorkloadSnapshot {
        pending_remediation_count: 7,
        critical_remediation_count: 3,
        due_memory_reviews: 20,
        active_learning_skills: 6,
        transfer_pending_count: 4,
        total_composite_load: 35,
    };
    assert_eq!(snap_overload.compute_state(), WorkloadState::Overloaded);
}

#[test]
fn test_session_budget_enforcement() {
    // 1. ItemCount budget: exhausts exactly at max_items
    let mut tracker_items = SessionBudgetTracker::new(Some(SessionBudget::ItemCount { max_items: 3 }));
    assert!(!tracker_items.is_exhausted);
    tracker_items.record_item(10_000, false);
    assert!(!tracker_items.is_exhausted);
    tracker_items.record_item(15_000, false);
    assert!(!tracker_items.is_exhausted);
    tracker_items.record_item(12_000, false);
    assert!(tracker_items.is_exhausted);

    // 2. TimeLimitMs budget: exhausts when elapsed time meets/exceeds limit
    let mut tracker_time = SessionBudgetTracker::new(Some(SessionBudget::TimeLimitMs { max_time_ms: 60_000 }));
    tracker_time.record_item(30_000, false);
    assert!(!tracker_time.is_exhausted);
    tracker_time.record_item(35_000, false); // total 65s >= 60s
    assert!(tracker_time.is_exhausted);

    // 3. Bounded budget: exhausts on either condition
    let mut tracker_bounded = SessionBudgetTracker::new(Some(SessionBudget::Bounded {
        max_items: 5,
        max_time_ms: 50_000,
    }));
    tracker_bounded.record_item(55_000, false); // exceeded time on item 1
    assert!(tracker_bounded.is_exhausted);
}

#[test]
fn test_workload_safeguards_cap_remediation_interventions() {
    let safeguards = WorkloadSafeguards {
        max_remediations_per_session: 2,
        max_prerequisite_depth: 10,
        max_concurrent_new_skills: 4,
    };

    let mut tracker = SessionBudgetTracker::new(None);
    assert!(tracker.can_serve_remediation(&safeguards));

    tracker.record_item(15_000, true); // Served remediation 1
    assert!(tracker.can_serve_remediation(&safeguards));

    tracker.record_item(15_000, true); // Served remediation 2
    // Max per-session cap reached
    assert!(!tracker.can_serve_remediation(&safeguards));
}
