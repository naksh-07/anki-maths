// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use procedural::core::{Domain, SchemaId, SkillId};
use procedural::practice::{PracticeObjective, PracticeRequest, PracticeScope};
use procedural::scheduling::interleaving::InterleavingPolicy;
use procedural::scheduling::unified::{PriorityTier, UnifiedPracticeEngine};
use procedural::skills::prerequisites::PrerequisiteGraphService;
use procedural::skills::signals::PracticeProgressionState;
use procedural::skills::SkillState;
use procedural::storage::ProceduralStore;
use std::collections::HashMap;
use std::sync::Arc;

#[test]
fn test_stage_aware_anti_priming_penalties() {
    let schema_a = SchemaId::new("schema_a");
    let schema_b = SchemaId::new("schema_b");
    let open_scope = PracticeScope::AllDomains;

    // 1. New stage -> mild penalty (-20.0) in open mixed pools
    let pol_new = InterleavingPolicy::for_stage(PracticeProgressionState::New);
    assert_eq!(pol_new.compute_penalty(&schema_a, Some(&schema_a), &open_scope), -20.0);
    assert_eq!(pol_new.compute_penalty(&schema_b, Some(&schema_a), &open_scope), 0.0);

    // 2. Learning stage -> mild penalty (-120.0)
    let pol_learning = InterleavingPolicy::for_stage(PracticeProgressionState::Learning);
    assert_eq!(pol_learning.compute_penalty(&schema_a, Some(&schema_a), &open_scope), -120.0);
    assert_eq!(pol_learning.compute_penalty(&schema_b, Some(&schema_a), &open_scope), 0.0);

    // 3. Fluent stage -> moderate penalty (-150.0)
    let pol_fluent = InterleavingPolicy::for_stage(PracticeProgressionState::Fluent);
    assert_eq!(pol_fluent.compute_penalty(&schema_a, Some(&schema_a), &open_scope), -150.0);

    // 4. Variation stage -> strong penalty (-300.0)
    let pol_var = InterleavingPolicy::for_stage(PracticeProgressionState::Variation);
    assert_eq!(pol_var.compute_penalty(&schema_a, Some(&schema_a), &open_scope), -300.0);

    // 5. Transfer stage -> high penalty (-350.0)
    let pol_transfer = InterleavingPolicy::for_stage(PracticeProgressionState::Transfer);
    assert_eq!(pol_transfer.compute_penalty(&schema_a, Some(&schema_a), &open_scope), -350.0);

    // 6. Mastered & Retired stages -> maintenance rotation penalty (-200.0)
    let pol_mastered = InterleavingPolicy::for_stage(PracticeProgressionState::Mastered);
    assert_eq!(pol_mastered.compute_penalty(&schema_a, Some(&schema_a), &open_scope), -200.0);

    let pol_retired = InterleavingPolicy::for_stage(PracticeProgressionState::Retired);
    assert_eq!(pol_retired.compute_penalty(&schema_a, Some(&schema_a), &open_scope), -200.0);
}

#[test]
fn test_focused_scope_strictly_bypasses_anti_priming_penalty() {
    let schema_target = SchemaId::new("linear_equations");
    let focused_scope = PracticeScope::SingleSchema(schema_target.clone());

    // Even in high-interleaving stages like Variation or Transfer, focused requests have 0 penalty
    let pol_var = InterleavingPolicy::for_stage(PracticeProgressionState::Variation);
    assert_eq!(
        pol_var.compute_penalty(&schema_target, Some(&schema_target), &focused_scope),
        0.0
    );

    let pol_transfer = InterleavingPolicy::for_stage(PracticeProgressionState::Transfer);
    assert_eq!(
        pol_transfer.compute_penalty(&schema_target, Some(&schema_target), &focused_scope),
        0.0
    );
}

#[test]
fn test_unified_engine_stage_interleaving_selects_different_schema_in_variation_stage() {
    let store = Arc::new(ProceduralStore::open_in_memory().unwrap());
    let prereq_service = Arc::new(PrerequisiteGraphService::new());

    let schema1_id = SchemaId::new("successive_percentage");
    let schema2_id = SchemaId::new("linear_equations");

    let skill1 = SkillId::new("percentage.successive");
    let skill2 = SkillId::new("algebra.linear");

    let mut schema_domains = HashMap::new();
    schema_domains.insert(schema1_id.clone(), Domain::Mathematics);
    schema_domains.insert(schema2_id.clone(), Domain::Mathematics);

    let schema1 = procedural::practice::SchemaPracticeObject::new(
        schema1_id.clone(),
        skill1.clone(),
        procedural::core::ProblemFamilyId::new("percentage_successive"),
        "Successive Percentage",
        "Successive percentage problems",
    );

    let schema2 = procedural::practice::SchemaPracticeObject::new(
        schema2_id.clone(),
        skill2.clone(),
        procedural::core::ProblemFamilyId::new("linear_equations"),
        "Linear Equations",
        "Linear equation problems",
    );

    let mut state1 = SkillState::new(skill1);
    state1.practice_state = PracticeProgressionState::Variation;
    state1.total_attempts = 10;

    let mut state2 = SkillState::new(skill2);
    state2.practice_state = PracticeProgressionState::Variation;
    state2.total_attempts = 10;

    let mut skill_states = HashMap::new();
    skill_states.insert(schema1.skill_id.clone(), state1);
    skill_states.insert(schema2.skill_id.clone(), state2);

    let schemas = vec![schema1, schema2];
    let request = PracticeRequest::new(PracticeScope::AllDomains, PracticeObjective::Practice);

    let registry = procedural::problems::registry::ProblemRegistry::new();
    let mut remediation_queue = procedural::remediation::RemediationQueue::new();

    // Since last schema was schema1, variation-stage interleaving penalty (-300) causes schema2 to be chosen
    let decision = UnifiedPracticeEngine::select_next(
        &request,
        &schemas,
        &schema_domains,
        &skill_states,
        &prereq_service,
        Some(&mut remediation_queue),
        None,
        &HashMap::new(),
        Some(&schema1_id),
        &registry,
        &store,
        42,
    )
    .unwrap();

    assert_eq!(decision.schema.id, schema2_id);
}