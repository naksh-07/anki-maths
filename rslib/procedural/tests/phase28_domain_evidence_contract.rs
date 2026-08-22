// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use procedural::skills::{
    ChemistryEvidence, DomainEvidencePayload, MathEvidence, PhysicsEvidence, ReasoningEvidence,
    VersionedDomainEvidence,
};
use procedural::skills::{MasteryEvidence, SkillState, RecentAttemptRecord};

#[test]
fn test_math_evidence_serialization() {
    let math_ev = MathEvidence {
        pattern_recognition: Some(true),
        method_selection: Some(false),
        execution: Some(true),
        verification: None,
        structural_transfer: Some(true),
    };

    let domain_ev = VersionedDomainEvidence::new_math(math_ev.clone());

    let mut mastery_ev = MasteryEvidence::default();
    mastery_ev.domain_evidence = Some(domain_ev);

    let json = serde_json::to_string(&mastery_ev).unwrap();
    
    // Ensure "math" variant and fields exist
    assert!(json.contains(r#""domain":"Math""#));
    assert!(json.contains(r#""pattern_recognition":true"#));
    assert!(json.contains(r#""method_selection":false"#));

    let deserialized: MasteryEvidence = serde_json::from_str(&json).unwrap();
    let des_domain = deserialized.domain_evidence.unwrap();
    assert_eq!(des_domain.version, 1);
    
    match des_domain.payload {
        DomainEvidencePayload::Math(m) => assert_eq!(m, math_ev),
        _ => panic!("Expected Math variant"),
    }
}

#[test]
fn test_reasoning_evidence_serialization() {
    let reasoning_ev = ReasoningEvidence {
        pattern_recognition: Some(true),
        deduction: Some(true),
        trap_checking: Some(false),
        ..Default::default()
    };

    let domain_ev = VersionedDomainEvidence::new_reasoning(reasoning_ev.clone());
    let mut mastery_ev = MasteryEvidence::default();
    mastery_ev.domain_evidence = Some(domain_ev);

    let json = serde_json::to_string(&mastery_ev).unwrap();
    assert!(json.contains(r#""domain":"Reasoning""#));
    assert!(json.contains(r#""deduction":true"#));

    let deserialized: MasteryEvidence = serde_json::from_str(&json).unwrap();
    let des_domain = deserialized.domain_evidence.unwrap();
    
    match des_domain.payload {
        DomainEvidencePayload::Reasoning(r) => assert_eq!(r, reasoning_ev),
        _ => panic!("Expected Reasoning variant"),
    }
}

#[test]
fn test_physics_evidence_serialization() {
    let physics_ev = PhysicsEvidence {
        physical_model_selection: Some(true),
        equation_setup: Some(true),
        unit_validity: Some(false),
        ..Default::default()
    };

    let domain_ev = VersionedDomainEvidence::new_physics(physics_ev.clone());
    let mut mastery_ev = MasteryEvidence::default();
    mastery_ev.domain_evidence = Some(domain_ev);

    let json = serde_json::to_string(&mastery_ev).unwrap();
    assert!(json.contains(r#""domain":"Physics""#));

    let deserialized: MasteryEvidence = serde_json::from_str(&json).unwrap();
    let des_domain = deserialized.domain_evidence.unwrap();
    
    match des_domain.payload {
        DomainEvidencePayload::Physics(p) => assert_eq!(p, physics_ev),
        _ => panic!("Expected Physics variant"),
    }
}

#[test]
fn test_chemistry_evidence_serialization() {
    // Physical Chemistry
    let phys_chem = ChemistryEvidence::Physical {
        model_setup: Some(true),
        equation_selection: Some(true),
        intermediate_quantity: Some(true),
        calculation: Some(false),
        conservation: None,
        verification: None,
        transfer: None,
    };

    let domain_ev = VersionedDomainEvidence::new_chemistry(phys_chem.clone());
    let mut mastery_ev = MasteryEvidence::default();
    mastery_ev.domain_evidence = Some(domain_ev);

    let json = serde_json::to_string(&mastery_ev).unwrap();
    assert!(json.contains(r#""domain":"Chemistry""#));
    assert!(json.contains(r#""branch":"Physical""#));
    assert!(json.contains(r#""calculation":false"#));

    let deserialized: MasteryEvidence = serde_json::from_str(&json).unwrap();
    let des_domain = deserialized.domain_evidence.unwrap();
    
    match des_domain.payload {
        DomainEvidencePayload::Chemistry(c) => assert_eq!(c, phys_chem),
        _ => panic!("Expected Chemistry variant"),
    }

    // Organic Chemistry
    let org_chem = ChemistryEvidence::Organic {
        substrate_recognition: Some(true),
        mechanism_pathway: Some(true),
        reagent_interpretation: None,
        product_prediction: Some(false),
        exception_handling: None,
        transfer: None,
    };
    let domain_ev_org = VersionedDomainEvidence::new_chemistry(org_chem.clone());
    let mut mastery_ev_org = MasteryEvidence::default();
    mastery_ev_org.domain_evidence = Some(domain_ev_org);
    let json_org = serde_json::to_string(&mastery_ev_org).unwrap();
    assert!(json_org.contains(r#""branch":"Organic""#));

    let deserialized_org: MasteryEvidence = serde_json::from_str(&json_org).unwrap();
    match deserialized_org.domain_evidence.unwrap().payload {
        DomainEvidencePayload::Chemistry(c) => assert_eq!(c, org_chem),
        _ => panic!("Expected Chemistry variant"),
    }
}

#[test]
fn test_backward_compatibility() {
    // A JSON payload representing an old MasteryEvidence without the domain_evidence field
    let old_json = r#"{
        "final_correctness": true,
        "decision_quality": null,
        "step_quality": null,
        "independence": "independent",
        "max_hint_level": null,
        "hint_dependence": 0,
        "retry_dependence": 0,
        "variant_exposure": "v1",
        "variant_category": "parameter",
        "solution_graph_fingerprint": null,
        "cognitive_decision_correct": null,
        "transfer_evidence": false,
        "latency_evidence": 12000,
        "time_since_last_ms": null,
        "domain_competence_verified": null,
        "diagnostic_errors": []
    }"#;

    let deserialized: MasteryEvidence = serde_json::from_str(old_json).expect("Should deserialize old schema successfully");
    assert_eq!(deserialized.domain_evidence, None);
    assert_eq!(deserialized.final_correctness, true);
    assert_eq!(deserialized.latency_evidence, 12000);
}

#[test]
fn test_invalid_payload_handling() {
    // Providing an unknown domain variant should fail to deserialize properly if strictly typed
    let malformed_json = r#"{
        "final_correctness": true,
        "independence": "independent",
        "hint_dependence": 0,
        "retry_dependence": 0,
        "variant_category": "parameter",
        "transfer_evidence": false,
        "latency_evidence": 12000,
        "diagnostic_errors": [],
        "domain_evidence": {
            "version": 1,
            "domain": "UnknownDomain",
            "evidence": {}
        }
    }"#;

    let result: Result<MasteryEvidence, _> = serde_json::from_str(malformed_json);
    assert!(result.is_err(), "Should error out on unknown domain payload");
}

#[test]
fn test_skill_state_persistence_isolation() {
    let mut skill_state = SkillState::new("math.calculus.limits");
    
    let math_ev = MathEvidence {
        execution: Some(true),
        method_selection: Some(false),
        ..Default::default()
    };

    let domain_ev = VersionedDomainEvidence::new_math(math_ev.clone());
    
    let mut mastery_ev = MasteryEvidence::default();
    mastery_ev.domain_evidence = Some(domain_ev.clone());

    // Act
    skill_state.record_attempt_outcome(&mastery_ev, 0.5, 30_000, 1000);

    // Verify recent attempt record has domain evidence
    let recent_attempt = skill_state.recent_attempts.last().unwrap();
    assert!(recent_attempt.domain_evidence.is_some());
    let saved_domain_ev = recent_attempt.domain_evidence.as_ref().unwrap();
    assert_eq!(saved_domain_ev.version, 1);
    
    match &saved_domain_ev.payload {
        DomainEvidencePayload::Math(m) => assert_eq!(m, &math_ev),
        _ => panic!("Expected Math variant in persisted RecentAttemptRecord"),
    }
}