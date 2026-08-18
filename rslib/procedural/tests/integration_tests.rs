// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use procedural::{
    AttemptDiagnosticSummary, Domain, ErrorCategory, ErrorEvent, PracticeAttempt, ProblemFamily,
    ProblemInstance, ProceduralCardAnchor, ProceduralService, ProceduralStore, SchemaPracticeObject,
    SeedMode, Skill,
};
use tempfile::tempdir;

#[test]
fn test_multi_domain_models_serialization() {
    let domains = vec![
        (Domain::Mathematics, "math.calculus.integrals", "Definite Integrals"),
        (Domain::Physics, "physics.electromagnetism.coulomb", "Coulomb's Law"),
        (Domain::Chemistry, "chem.thermo.enthalpy", "Hess's Law Enthalpy"),
        (Domain::Reasoning, "reasoning.logic.syllogisms", "Categorical Syllogisms"),
    ];

    for (domain, skill_id, name) in domains {
        let skill = Skill::new(skill_id, domain.clone(), name, format!("Practice {name}"));
        let json = serde_json::to_string(&skill).expect("serialize skill");
        let deserialized: Skill = serde_json::from_str(&json).expect("deserialize skill");

        assert_eq!(skill.id, deserialized.id);
        assert_eq!(skill.domain, deserialized.domain);
        assert_eq!(skill.name, deserialized.name);
    }
}

#[test]
fn test_database_migrations_and_idempotency() {
    let dir = tempdir().expect("tempdir");
    let db_path = dir.path().join("procedural.db");

    // First open runs migrations
    {
        let store = ProceduralStore::open(&db_path).expect("open store");
        let skill = Skill::new(
            "math.algebra.monic",
            Domain::Mathematics,
            "Monic Factoring",
            "Factoring monic quadratics",
        );
        store.insert_skill(&skill).expect("insert skill");
    }

    // Re-open runs migrations idempotently and retains data
    {
        let store = ProceduralStore::open(&db_path).expect("reopen store");
        let skill = store
            .get_skill(&"math.algebra.monic".into())
            .expect("get skill")
            .expect("skill must exist");
        assert_eq!(skill.name, "Monic Factoring");
    }
}

#[test]
fn test_anki_card_anchor_extraction_and_resolution() {
    let service = ProceduralService::open_in_memory().expect("open service");

    // Setup skill, family, and schema
    let skill = Skill::new(
        "physics.optics.snell",
        Domain::Physics,
        "Snell's Law",
        "Refraction index calculations",
    );
    service.register_skill(skill.clone()).expect("register skill");

    let family = ProblemFamily::new(
        "fam.physics.optics.snell",
        &skill.id,
        Domain::Physics,
        "Snell's Law Generator",
        "math.percentage.successive.v1",
    );
    service.register_problem_family(family.clone()).expect("register family");

    let schema = SchemaPracticeObject::new(
        "schema.physics.snell",
        &skill.id,
        &family.id,
        "Practice Snell's Law",
        "Calculate refraction angle given n1, n2, theta1",
    );
    service.register_schema(schema.clone()).expect("register schema");

    // Test anchor parsing from note fields
    let card_fields = vec![
        "Calculate the angle of refraction:".to_string(),
        r#"{"proc_schema": "schema.physics.snell", "difficulty_override": 1.5, "seed_mode": "random"}"#.to_string(),
        "Solution placeholder".to_string(),
    ];

    let anchor = ProceduralCardAnchor::extract_from_card_fields(&card_fields)
        .expect("extract anchor")
        .expect("anchor must be present");

    assert_eq!(anchor.proc_schema.as_str(), "schema.physics.snell");
    assert_eq!(anchor.difficulty_override, Some(1.5));
    assert_eq!(anchor.seed_mode, SeedMode::Random);

    // Prepare practice session
    let session = service
        .prepare_practice_session(&anchor, Some(9999))
        .expect("prepare session");

    assert_eq!(session.schema.id.as_str(), "schema.physics.snell");
    assert_eq!(session.card_id, Some(9999));
    assert!(!session.instance.id.as_str().is_empty());
}

#[test]
fn test_attempts_error_events_and_diagnostics() {
    let service = ProceduralService::open_in_memory().expect("open service");

    let skill = Skill::new(
        "chem.stoich.limiting",
        Domain::Chemistry,
        "Limiting Reactant",
        "Find limiting reactant and theoretical yield",
    );
    service.register_skill(skill.clone()).expect("register skill");

    let family = ProblemFamily::new(
        "fam.chem.stoich.limiting",
        &skill.id,
        Domain::Chemistry,
        "Limiting Reactant Family",
        "chem.stoich.v1",
    );
    service.register_problem_family(family.clone()).expect("register family");

    let schema = SchemaPracticeObject::new(
        "schema.chem.stoich.limiting",
        &skill.id,
        &family.id,
        "Practice Limiting Reactant",
        "Reaction yield practice",
    );
    service.register_schema(schema.clone()).expect("register schema");

    let instance = ProblemInstance::new(
        "inst-chem-1",
        &family.id,
        777,
        serde_json::json!({ "reaction": "2H2 + O2 -> 2H2O", "moles_H2": 4.0, "moles_O2": 1.5 }),
        "Find moles of H2O formed from 4.0 mol H2 and 1.5 mol O2",
        serde_json::json!({ "limiting": "O2", "moles_H2O": 3.0 }),
    );
    service.save_problem_instance(instance.clone()).expect("save instance");

    // Attempt 1: Failed due to calculation error
    let attempt1 = PracticeAttempt::new(
        "att-chem-1",
        &instance.id,
        &schema.id,
        &skill.id,
        serde_json::json!({ "limiting": "H2", "moles_H2O": 4.0 }),
        false,
        0.0,
        6500,
    )
    .with_card_id(4242);

    let error1 = ErrorEvent::new(
        "err-chem-1",
        &attempt1.id,
        ErrorCategory::Conceptual.to_string(),
        serde_json::json!({ "reason": "Chose excess reactant instead of limiting" }),
    );

    service.record_practice_attempt(attempt1.clone(), vec![error1.clone()]).expect("record attempt 1");

    // Attempt 2: Successful
    let attempt2 = PracticeAttempt::new(
        "att-chem-2",
        &instance.id,
        &schema.id,
        &skill.id,
        serde_json::json!({ "limiting": "O2", "moles_H2O": 3.0 }),
        true,
        1.0,
        3200,
    )
    .with_card_id(4242);

    service.record_practice_attempt(attempt2.clone(), vec![]).expect("record attempt 2");

    // Verify card attempts query
    let card_attempts = service.get_attempts_for_card(4242).expect("get card attempts");
    assert_eq!(card_attempts.len(), 2);

    // Verify skill state updated
    let state = service.load_skill_state(&skill.id).expect("load state").expect("state exists");
    assert_eq!(state.total_attempts, 2);
    assert_eq!(state.successful_attempts, 1);
    assert_eq!(state.success_rate(), 0.5);

    // Verify diagnostics computation
    let all_attempts = service.get_recent_attempts_for_schema(&schema.id, 10).expect("get schema attempts");
    let summary = AttemptDiagnosticSummary::compute(&all_attempts, &[error1]);
    assert_eq!(summary.total_attempts, 2);
    assert_eq!(summary.correct_attempts, 1);
    assert_eq!(summary.accuracy, 0.5);
    assert_eq!(summary.error_breakdown.get("conceptual"), Some(&1));
}

#[test]
fn test_database_independence() {
    let dir = tempdir().expect("tempdir");
    let proc_db_path = dir.path().join("procedural.db");

    // Create procedural database
    let store = ProceduralStore::open(&proc_db_path).expect("open proc db");
    store.insert_skill(&Skill::new(
        "test.skill",
        Domain::Mathematics,
        "Test Skill",
        "Desc",
    )).expect("insert skill");

    // Ensure the file exists
    assert!(proc_db_path.exists());

    // Drop store
    drop(store);

    // Delete procedural.db completely
    std::fs::remove_file(&proc_db_path).expect("remove procedural db");
    assert!(!proc_db_path.exists());

    // Recreating it works without leaving any corrupt state or dependencies
    let new_store = ProceduralStore::open(&proc_db_path).expect("reopen clean proc db");
    let skill = new_store.get_skill(&"test.skill".into()).expect("query skill");
    assert!(skill.is_none());
}
