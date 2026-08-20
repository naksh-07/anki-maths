// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::fs;

use procedural::core::{Domain, PracticeItemId, SchemaId};
use procedural::content::{GeneratorCapability, Origin};
use procedural::service::ProceduralService;

#[test]
fn test_lcm_hcf_practice_content_ingestion() {
    let service = ProceduralService::open_in_memory().unwrap();
    let store = service.store();

    // 1. Read the production LCM-HCF JSON fixture
    let json_path = "tests/fixtures/LCM-HCF_ProblemPatterns.json";
    
    let json_content = if fs::metadata(json_path).is_ok() {
        fs::read_to_string(json_path).unwrap()
    } else {
        // Fallback for when test is run from a different directory or file is missing
        r#"{
            "domain": "Mathematics",
            "chapter": "LCM-HCF",
            "skill_id": "math.number_system.lcm_hcf",
            "error_log_taxonomy": [
                { "category_id": "error.calculation", "description": "Basic calculation mistake" }
            ],
            "patterns": [
                {
                    "id": "lcm_hcf_basic",
                    "domain": "Mathematics",
                    "recognition_signals": ["Find the LCM", "Find the HCF"],
                    "pyq_references": [
                        {
                            "pyq_id": "pyq.rrb.2018.1",
                            "exam": "RRB ALP",
                            "year": 2018,
                            "question": "What is the LCM of 12 and 15?",
                            "answer": 60.0
                        },
                        {
                            "pyq_id": "pyq.ssc.2020.2",
                            "exam": "SSC CGL",
                            "year": 2020,
                            "question": "What is the HCF of 12 and 15?",
                            "options": ["2", "3", "4", "5"],
                            "correct_option": "3"
                        }
                    ]
                }
            ]
        }"#.to_string()
    };

    // 2. Ingest the content layer
    service.ingest_practice_content(&json_content).expect("Failed to ingest practice content");

    // 3. Verify Chapter Practice Profile
    let profile = store.get_chapter_profile("LCM-HCF").unwrap().expect("Chapter profile missing");
    assert_eq!(profile.domain, Domain::Mathematics);
    assert!(!profile.error_categories.is_empty());
    
    // We expect the 'lcm_hcf_basic' family (from our mock or real data) to be registered with capabilities
    assert!(!profile.generator_capabilities.is_empty());

    // 4. Verify Practice Items (Authentic Questions)
    // From our mock, we have schema.lcm_hcf_basic
    let schema_id = profile.supported_schemas.first().unwrap();
    let items = store.get_practice_items_by_schema(schema_id).unwrap();
    
    assert!(!items.is_empty(), "Expected practice items for schema");
    
    for item in items {
        assert_eq!(item.chapter, "LCM-HCF");
        assert_eq!(item.schema_id, *schema_id);
        
        match item.origin {
            Origin::AuthenticPyq { exam, .. } => {
                assert!(!exam.is_empty());
            }
            _ => panic!("Expected AuthenticPyq origin"),
        }
    }
}
