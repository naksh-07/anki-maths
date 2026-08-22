// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Phase 18: Detailed Source Extraction and Archetype Inventory Helper

use std::fs;
use std::path::Path;

use procedural::content::ingestion::PracticeContentIngester;
use procedural::content::QuestionType;
use procedural::core::ProblemFamilyId;
use procedural::storage::ProceduralStore;

const WORKSPACE_LCM_HCF_JSON: &str = r"C:\Users\Suraj\Pictures\Books\Acadmey\ALP\Prompts\AI Notes\Study Materials\Math\LCM-HCF\Optional\LCM-HCF_ProblemPatterns.json";
const WORKSPACE_LCM_HCF_QUESTIONS_JSON: &str = r"C:\Users\Suraj\Pictures\Books\Acadmey\ALP\Prompts\AI Notes\Study Materials\Math\LCM-HCF\Optional\LCM-HCF_PracticeQuestions.json";
const WORKSPACE_REASONING_JSON: &str = r"C:\Users\Suraj\Pictures\Books\Acadmey\ALP\Prompts\AI Notes\.agents\skills\study-source-core\scripts\scratch\fixtures\reasoning_problem_patterns.json";
const WORKSPACE_REASONING_QUESTIONS_JSON: &str = r"C:\Users\Suraj\Pictures\Books\Acadmey\ALP\Prompts\AI Notes\.agents\skills\study-source-core\scripts\scratch\fixtures\reasoning_practice_questions.json";

fn load_file_or_fallback(path: &str, fallback: &str) -> String {
    if Path::new(path).exists() {
        fs::read_to_string(path).unwrap_or_else(|_| fallback.to_string())
    } else {
        fallback.to_string()
    }
}

#[test]
fn test_phase18_dump_all_source_questions_and_patterns() {
    let math_pat = load_file_or_fallback(WORKSPACE_LCM_HCF_JSON, r#"{"patterns":[]}"#);
    let math_q = load_file_or_fallback(WORKSPACE_LCM_HCF_QUESTIONS_JSON, r#"{"questions":[]}"#);
    let reas_pat = load_file_or_fallback(WORKSPACE_REASONING_JSON, r#"{"patterns":[]}"#);
    let reas_q = load_file_or_fallback(WORKSPACE_REASONING_QUESTIONS_JSON, r#"{"questions":[]}"#);

    let clean_math_pat = math_pat.trim_start_matches('\u{feff}');
    let clean_math_q = math_q.trim_start_matches('\u{feff}');
    let clean_reas_pat = reas_pat.trim_start_matches('\u{feff}');
    let clean_reas_q = reas_q.trim_start_matches('\u{feff}');

    println!("=== MATH PATTERNS JSON ===");
    let parsed_math_pat: serde_json::Value = serde_json::from_str(clean_math_pat).unwrap_or_default();
    if let Some(pats) = parsed_math_pat.get("patterns").and_then(|p| p.as_array()) {
        for p in pats {
            println!("Pattern ID: {}, ProblemType: {}, Schema: {}, Family: {}, Prereqs: {:?}, Traps: {:?}",
                p["id"], p["problem_type"], p["schema_id"], p["problem_family"], p["prerequisites"], p["common_traps"]);
            if let Some(pyqs) = p.get("pyq_references").and_then(|q| q.as_array()) {
                println!("  PYQ Refs count in pattern: {}", pyqs.len());
                for pyq in pyqs {
                    println!("    PYQ Ref: Exam={}, Year={}, Shift={:?}, Question='{}'", pyq["exam"], pyq["year"], pyq.get("shift"), pyq.get("question").and_then(|q| q.as_str()).unwrap_or(""));
                }
            }
        }
    }

    println!("\n=== MATH PRACTICE QUESTIONS JSON ===");
    let parsed_math_q: serde_json::Value = serde_json::from_str(clean_math_q).unwrap_or_default();
    if let Some(qs) = parsed_math_q.get("questions").and_then(|q| q.as_array()) {
        println!("Math Questions Count: {}", qs.len());
        for q in qs {
            println!("Q ID: {}, Origin: {}, Pattern: {}, Schema: {}, Family: {}, Difficulty: {}, Type: {}, Prompt: {}",
                q["id"], q["origin_type"], q.get("pattern_id").unwrap_or(&serde_json::Value::Null),
                q.get("schema_id").unwrap_or(&serde_json::Value::Null),
                q.get("problem_family").unwrap_or(&serde_json::Value::Null),
                q.get("difficulty").unwrap_or(&serde_json::Value::Null),
                q.get("question_type").unwrap_or(&serde_json::Value::Null),
                q.get("prompt").and_then(|s| s.as_str()).unwrap_or("").lines().next().unwrap_or(""));
        }
    }

    println!("\n=== REASONING PATTERNS JSON ===");
    let parsed_reas_pat: serde_json::Value = serde_json::from_str(clean_reas_pat).unwrap_or_default();
    if let Some(pats) = parsed_reas_pat.get("patterns").and_then(|p| p.as_array()) {
        for p in pats {
            println!("Pattern ID: {}, ProblemType: {}, Schema: {}, Family: {}",
                p["id"], p["problem_type"], p["schema_id"], p["problem_family"]);
            if let Some(pyqs) = p.get("pyq_references").and_then(|q| q.as_array()) {
                println!("  PYQ Refs count in pattern: {}", pyqs.len());
                for pyq in pyqs {
                    println!("    PYQ Ref: Exam={}, Year={}, Shift={:?}, Question='{}'", pyq["exam"], pyq["year"], pyq.get("shift"), pyq.get("question").and_then(|q| q.as_str()).unwrap_or(""));
                }
            }
        }
    }

    println!("\n=== REASONING PRACTICE QUESTIONS JSON ===");
    let parsed_reas_q: serde_json::Value = serde_json::from_str(clean_reas_q).unwrap_or_default();
    if let Some(qs) = parsed_reas_q.get("questions").and_then(|q| q.as_array()) {
        println!("Reasoning Questions Count: {}", qs.len());
        for q in qs {
            println!("Q ID: {}, Origin: {}, Pattern: {}, Schema: {}, Family: {}, Difficulty: {}, Type: {}, Prompt: {}",
                q["id"], q["origin_type"], q.get("pattern_id").unwrap_or(&serde_json::Value::Null),
                q.get("schema_id").unwrap_or(&serde_json::Value::Null),
                q.get("problem_family").unwrap_or(&serde_json::Value::Null),
                q.get("difficulty").unwrap_or(&serde_json::Value::Null),
                q.get("question_type").unwrap_or(&serde_json::Value::Null),
                q.get("prompt").and_then(|s| s.as_str()).unwrap_or("").lines().next().unwrap_or(""));
        }
    }
}