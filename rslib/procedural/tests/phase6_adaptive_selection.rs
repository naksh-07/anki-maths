// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use procedural::core::{Domain, PracticeItemId, ProblemFamilyId, PyqId, SchemaId, SkillId};
use procedural::content::item::{Origin, PracticeItem, QuestionType};
use procedural::exam::pyq::ContentProvenance;
use procedural::practice::{PracticeObjective, PracticeRequest, PracticeScope, RemediationPrecedence, SchemaPracticeObject};
use procedural::problems::registry::ProblemRegistry;
use procedural::scheduling::unified::{LearningObjectKind, UnifiedPracticeEngine};
use procedural::skills::{PrerequisiteGraphService, SkillState};
use procedural::skills::signals::{IndependenceLevel, MasteryEvidence};
use procedural::storage::ProceduralStore;
use std::collections::HashMap;
use tempfile::tempdir;

fn create_in_memory_store() -> ProceduralStore {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let store = ProceduralStore::open_in_memory().unwrap();
    store
}

#[test]
fn test_adaptive_selection_longitudinal_simulations() {
    let store = create_in_memory_store();
    let registry = ProblemRegistry::new(); // Dummy registry will work for STAGE 5 fallback tests since we only need metadata
    
    // Setup schemas
    let skill_id = SkillId::new("math.arithmetic.test_skill");
    let schema_id = SchemaId::new("schema.test_skill.standard");
    let problem_family_id = ProblemFamilyId::new("fam.test_skill.v1");
    let domain = Domain::Mathematics;

    let schema = SchemaPracticeObject {
        id: schema_id.clone(),
        skill_id: skill_id.clone(),
        problem_family_id: problem_family_id.clone(),
        title: "Test Schema".to_string(),
        description: "Test Schema".to_string(),
        target_mastery: 1.0,
        config: serde_json::json!({}),
        created_at: 0,
    };

    let mut schema_domains = HashMap::new();
    schema_domains.insert(schema_id.clone(), domain.clone());
    
    // Seed database with PracticeItems
    let pyq_item = PracticeItem::new(
        PracticeItemId::new("item-pyq-1"),
        Origin::AuthenticPyq { pyq_id: PyqId::new("pyq-1"), exam: "JEE".to_string(), year: 2023, shift: None },
        domain.clone(),
        "test_chapter",
        skill_id.clone(),
        schema_id.clone(),
        problem_family_id.clone(),
        QuestionType::Numerical { answer: 42.0, tolerance: None },
        "Authentic PYQ prompt",
        ContentProvenance::default()
    );
    
    let variant_item = PracticeItem::new(
        PracticeItemId::new("item-var-1"),
        Origin::DerivedVariant { parent_id: PracticeItemId::new("item-pyq-1"), generator_version: 1, seed: 101, variant_type: "parameter".to_string() },
        domain.clone(),
        "test_chapter",
        skill_id.clone(),
        schema_id.clone(),
        problem_family_id.clone(),
        QuestionType::Numerical { answer: 24.0, tolerance: None },
        "Variant prompt",
        ContentProvenance::default()
    );

    store.insert_practice_item(&pyq_item).unwrap();
    store.insert_practice_item(&variant_item).unwrap();

    let mut state = SkillState::new(skill_id.clone());
    
    let request = PracticeRequest {
        scope: PracticeScope::SingleSchema(schema_id.clone()),
        objective: PracticeObjective::Practice,
        remediation_policy: RemediationPrecedence::Disabled,
        difficulty_constraint: None,
        time_constraint: None,
        exam_profile: None,
        session_budget: None,
    };
    
    let prereq_service = PrerequisiteGraphService::new();
    
    let get_next = |s: &HashMap<SkillId, SkillState>| {
        UnifiedPracticeEngine::select_next(
            &request,
            &[schema.clone()],
            &schema_domains,
            &s,
            &prereq_service,
            None,
            None,
            &HashMap::new(),
            None,
            &registry,
            &store,
            42,
        ).unwrap()
    };
    
    // --- Learner A (Cold Start) ---
    // Expected to get Authentic (Level 0) item
    let mut states = HashMap::new();
    states.insert(skill_id.clone(), state.clone());
    
    let decision1 = get_next(&states);
    match decision1.learning_object {
        LearningObjectKind::PracticeItem(item) => {
            assert!(matches!(item.origin, Origin::AuthenticPyq { .. }), "Learner A should get Authentic Pyq");
        }
        _ => panic!("Expected PracticeItem"),
    }
    
    // Simulate successful attempt on Level 0
    let ev_success = MasteryEvidence {
        final_correctness: true,
        latency_evidence: 15000,
        independence: IndependenceLevel::Independent,
        variant_exposure: Some("item-pyq-1".to_string()),
        ..Default::default()
    };
    state.record_attempt_outcome(&ev_success, 1.0, 30000, 1000);
    
    // Verify level advanced
    assert_eq!(state.variant_progression_level, 1, "Successful solve should advance variant progression level to 1");
    
    // --- Learner B (Level 1) ---
    // Expected to get Variant (Level 1) item
    states.insert(skill_id.clone(), state.clone());
    let decision2 = get_next(&states);
    match decision2.learning_object {
        LearningObjectKind::PracticeItem(item) => {
            assert!(matches!(item.origin, Origin::DerivedVariant { .. }), "Learner B should get Derived Variant");
        }
        _ => panic!("Expected PracticeItem variant"),
    }
    
    // Simulate conceptual failure
    let ev_failure = MasteryEvidence {
        final_correctness: false,
        latency_evidence: 20000,
        independence: IndependenceLevel::Independent,
        diagnostic_errors: vec![procedural::diagnostics::ErrorCategory::Concept],
        ..Default::default()
    };
    state.record_attempt_outcome(&ev_failure, 0.0, 30000, 2000);
    
    // Verify level regressed
    assert_eq!(state.variant_progression_level, 0, "Conceptual error should reset variant progression level to 0");
    
    // --- Learner C (Exact replay test) ---
    // At Level 0, but item-pyq-1 was already solved independently.
    states.insert(skill_id.clone(), state.clone());
    let decision3 = get_next(&states);
    // Since item-pyq-1 is the only Level 0 item and it's already solved, the system should either 
    // find no canonical level 0 items and fallback to dynamic generation, OR it falls back to whatever STAGE 5 produces.
    // Let's check what it produces.
    match decision3.learning_object {
        LearningObjectKind::ProceduralProblem(_) => {
            // Success: it fell back to procedural generation because exact replay policy blocked item-pyq-1
        }
        _ => panic!("Expected ProceduralProblem fallback because Level 0 canonical item was exhausted"),
    }
}
