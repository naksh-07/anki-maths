// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use crate::content::{ChapterPracticeProfile, GeneratorCapability, Origin, PracticeItem, QuestionType};
use crate::core::{Domain, PracticeItemId, ProblemFamilyId, PyqId, SchemaId, SkillId};
use crate::exam::pyq::ContentProvenance;
use crate::storage::ProceduralStore;

/// Parses StudyLab source material (like LCM-HCF_ProblemPatterns.json and LCM-HCF_PracticeQuestions.json)
/// and ingests canonical `PracticeItem`s and a `ChapterPracticeProfile` into the store.
pub struct PracticeContentIngester;

impl PracticeContentIngester {
    /// Ingests study material JSON. Handles either ProblemPatterns.json, PracticeQuestions.json,
    /// or a combined bundle containing both patterns and questions.
    pub fn ingest_study_material_json(store: &ProceduralStore, json_content: &str) -> crate::core::Result<()> {
        let clean_json = json_content.trim_start_matches('\u{feff}');
        let root: serde_json::Value = serde_json::from_str(clean_json)?;
        
        let has_patterns = root.get("patterns").and_then(|p| p.as_array()).is_some();
        let has_questions = root.get("questions").and_then(|q| q.as_array()).is_some();

        if has_patterns {
            Self::ingest_patterns_json(store, &root)?;
        }
        
        if has_questions {
            Self::ingest_questions_json_value(store, &root)?;
        }

        if !has_patterns && !has_questions {
            // Fallback for minimalist or profile-only JSON
            Self::ingest_patterns_json(store, &root)?;
        }

        Ok(())
    }

    /// Dedicated ingester for canonical [Chapter]_PracticeQuestions.json artifacts
    pub fn ingest_practice_questions_json(store: &ProceduralStore, json_content: &str) -> crate::core::Result<()> {
        let clean_json = json_content.trim_start_matches('\u{feff}');
        let root: serde_json::Value = serde_json::from_str(clean_json)?;
        Self::ingest_questions_json_value(store, &root)
    }

    fn ingest_patterns_json(store: &ProceduralStore, root: &serde_json::Value) -> crate::core::Result<()> {
        let domain_str = root["domain"].as_str().unwrap_or("Unknown");
        let chapter = root["chapter"].as_str().unwrap_or("Unknown");
        let skill_id_str = root["skill_id"].as_str().unwrap_or("unknown");
        
        let domain: Domain = domain_str.to_lowercase().parse().unwrap_or(Domain::Custom(domain_str.to_string()));
        let skill_id = SkillId::new(skill_id_str);
        
        let mut profile = store.get_chapter_profile(chapter)?.unwrap_or_else(|| {
            ChapterPracticeProfile::new(chapter, domain.clone())
        });
        
        if let Some(errors) = root["error_log_taxonomy"].as_array() {
            for err in errors {
                if let Some(err_id) = err["category_id"].as_str() {
                    if !profile.error_categories.contains(&err_id.to_string()) {
                        profile.error_categories.push(err_id.to_string());
                    }
                } else if let Some(err_str) = err.as_str() {
                    if !profile.error_categories.contains(&err_str.to_string()) {
                        profile.error_categories.push(err_str.to_string());
                    }
                }
            }
        }

        if let Some(patterns) = root["patterns"].as_array() {
            for pat in patterns {
                let pat_id = pat["id"].as_str().unwrap_or("unknown_pat");
                // Create a schema_id and family_id from the pattern id if none provided
                let schema_id = if let Some(sid) = pat["schema_id"].as_str() {
                    if sid.starts_with("schema.") { SchemaId::new(sid) } else { SchemaId::new(format!("schema.{}", sid)) }
                } else {
                    SchemaId::new(format!("schema.{}", pat_id))
                };
                let family_id = if let Some(fid) = pat["problem_family"].as_str() {
                    if fid.starts_with("family.") { ProblemFamilyId::new(fid) } else { ProblemFamilyId::new(format!("family.{}", fid)) }
                } else {
                    ProblemFamilyId::new(format!("family.{}", pat_id))
                };
                
                if !profile.supported_schemas.contains(&schema_id) {
                    profile.supported_schemas.push(schema_id.clone());
                }
                // Also ensure schema.<pat_id> is registered if different
                let alt_schema = SchemaId::new(format!("schema.{}", pat_id));
                if !profile.supported_schemas.contains(&alt_schema) {
                    profile.supported_schemas.push(alt_schema.clone());
                }

                if !profile.supported_problem_families.contains(&family_id) {
                    profile.supported_problem_families.push(family_id.clone());
                }
                profile.generator_capabilities.insert(family_id.as_str().to_string(), GeneratorCapability::SourceOnly);
                
                if let Some(signals) = pat["recognition_signals"].as_array() {
                    for sig in signals {
                        if let Some(s) = sig.as_str() {
                            profile.recognition_signals.push(s.to_string());
                        }
                    }
                }
                
                if let Some(prereqs) = pat["prerequisites"].as_array() {
                    for pr in prereqs {
                        if let Some(p) = pr.as_str() {
                            profile.prerequisites.push(p.to_string());
                        }
                    }
                }
                
                // Parse PYQs / source questions as canonical PracticeItems if present
                if let Some(pyqs) = pat["pyq_references"].as_array() {
                    for pyq in pyqs {
                        let exam = pyq["exam"].as_str().unwrap_or("Unknown");
                        let year = pyq["year"].as_u64().unwrap_or(0) as u32;
                        let shift = pyq["shift"].as_str().map(|s| s.to_string());
                        let question_number = pyq["question_number"].as_str().unwrap_or("");
                        let source_book = pyq["source"].as_str().unwrap_or("");
                        
                        let derived_id = if let Some(id) = pyq["pyq_id"].as_str() {
                            id.replace(' ', "_").to_lowercase()
                        } else if !question_number.is_empty() {
                            format!("{}_{}_{}",
                                exam.to_lowercase().replace(' ', "_"),
                                year,
                                question_number.to_lowercase().replace(' ', "_"))
                        } else if !source_book.is_empty() {
                            format!("{}_{}_ref",
                                exam.to_lowercase().replace(' ', "_"),
                                year)
                        } else {
                            format!("pyq_{}_{}", exam.to_lowercase().replace(' ', "_"), year)
                        };
                        
                        let p_id = PyqId::new(&derived_id);
                        let item_id = PracticeItemId::new(format!("item_{}", derived_id));
                        
                        let origin = Origin::AuthenticPyq {
                            pyq_id: p_id.clone(),
                            exam: exam.to_string(),
                            year,
                            shift,
                        };
                        
                        let raw_prompt = pyq["question"].as_str().or_else(|| pyq["prompt"].as_str());
                        let has_prompt = raw_prompt.map(|s| !s.trim().is_empty()).unwrap_or(false);
                        let prompt = raw_prompt.unwrap_or("").trim().to_string();
                        
                        let ref_description = if !source_book.is_empty() && !question_number.is_empty() {
                            format!("{}, {} {}, {}", source_book, exam, year, question_number)
                        } else if !source_book.is_empty() {
                            format!("{}, {} {}", source_book, exam, year)
                        } else {
                            format!("{} {} {}", exam, year, question_number)
                        };

                        let question_type = if !has_prompt {
                            QuestionType::ReferenceOnly {
                                source_reference: ref_description,
                            }
                        } else if let Some(options) = pyq["options"].as_array() {
                            if options.is_empty() {
                                QuestionType::ReferenceOnly {
                                    source_reference: ref_description.clone(),
                                }
                            } else {
                                QuestionType::Mcq {
                                    options: options.iter().filter_map(|o| o.as_str().map(|s| s.to_string())).collect(),
                                    correct_option: pyq["correct_option"].as_str().unwrap_or("").to_string(),
                                    explanation: pyq["explanation"].as_str().map(|s| s.to_string()),
                                }
                            }
                        } else if let Some(ans) = pyq["answer"].as_f64() {
                            QuestionType::Numerical {
                                answer: ans,
                                tolerance: pyq["tolerance"].as_f64(),
                            }
                        } else if pyq.get("steps").is_some() && !pyq["steps"].is_null() {
                            QuestionType::Structured {
                                steps: pyq["steps"].clone(),
                            }
                        } else {
                            QuestionType::ReferenceOnly {
                                source_reference: ref_description,
                            }
                        };
                        
                        let prov = ContentProvenance::new_pyq_derived(
                            p_id.clone(), 1, 1, 1, 1, "authentic_pyq", None
                        );
                        
                        let mut item = PracticeItem::new(
                            item_id,
                            origin,
                            domain.clone(),
                            chapter,
                            skill_id.clone(),
                            schema_id.clone(),
                            family_id.clone(),
                            question_type,
                            prompt,
                            prov
                        );
                        
                        item.metadata = pyq.clone();
                        
                        store.insert_practice_item(&item)?;
                    }
                }
            }
        }

        store.insert_chapter_profile(&profile)?;
        Ok(())
    }

    fn ingest_questions_json_value(store: &ProceduralStore, root: &serde_json::Value) -> crate::core::Result<()> {
        let domain_str = root["domain"].as_str().unwrap_or("Unknown");
        let chapter = root["chapter"].as_str().unwrap_or("Unknown");
        let skill_id_str = root["skill_id"].as_str().unwrap_or("unknown");
        
        let domain: Domain = domain_str.to_lowercase().parse().unwrap_or(Domain::Custom(domain_str.to_string()));
        let skill_id = SkillId::new(skill_id_str);
        
        let mut profile = store.get_chapter_profile(chapter)?.unwrap_or_else(|| {
            ChapterPracticeProfile::new(chapter, domain.clone())
        });

        if let Some(questions) = root["questions"].as_array() {
            for q in questions {
                let q_id_str = q["id"].as_str().unwrap_or("unknown_q");
                let item_id = if q_id_str.starts_with("item_") || q_id_str.starts_with("item-") {
                    PracticeItemId::new(q_id_str)
                } else {
                    PracticeItemId::new(format!("item_{}", q_id_str))
                };

                let origin_type = q["origin_type"].as_str().unwrap_or("AUTHENTIC_PYQ");
                let origin = match origin_type {
                    "AUTHENTIC_PYQ" => {
                        let exam_meta = &q["exam_metadata"];
                        let exam = exam_meta["exam"].as_str().or_else(|| q["exam"].as_str()).unwrap_or("Unknown");
                        let year = exam_meta["year"].as_u64().or_else(|| q["year"].as_u64()).unwrap_or(0) as u32;
                        let shift = exam_meta["shift"].as_str().or_else(|| q["shift"].as_str()).map(|s| s.to_string());
                        let pyq_id = PyqId::new(format!("pyq_{}", q_id_str));
                        Origin::AuthenticPyq {
                            pyq_id,
                            exam: exam.to_string(),
                            year,
                            shift,
                        }
                    }
                    "CURATED_SOURCE" => {
                        let src_ref = q["source_provenance"]["source_book"].as_str()
                            .or_else(|| q["source_reference"].as_str())
                            .or_else(|| q["source"].as_str())
                            .unwrap_or(chapter);
                        Origin::CuratedSource {
                            source_reference: src_ref.to_string(),
                        }
                    }
                    "DERIVED_VARIANT" => {
                        let parent_id = q["parent_id"].as_str().map(PracticeItemId::new)
                            .unwrap_or_else(|| PracticeItemId::new(format!("{}_parent", q_id_str)));
                        let gen_v = q["generator_version"].as_u64().unwrap_or(1) as u32;
                        let seed = q["seed"].as_u64().unwrap_or(0);
                        let v_type = q["variant_type"].as_str().unwrap_or("parameter").to_string();
                        Origin::DerivedVariant {
                            parent_id,
                            generator_version: gen_v,
                            seed,
                            variant_type: v_type,
                        }
                    }
                    "SYNTHETIC_SCHEMA" => {
                        let gen_v = q["generator_version"].as_u64().unwrap_or(1) as u32;
                        let seed = q["seed"].as_u64().unwrap_or(0);
                        Origin::SyntheticSchema {
                            generator_version: gen_v,
                            seed,
                        }
                    }
                    _ => Origin::CuratedSource {
                        source_reference: chapter.to_string(),
                    },
                };

                let prompt_raw = q["prompt"].as_str().or_else(|| q["question"].as_str()).unwrap_or("");
                let prompt = prompt_raw.trim().to_string();
                let has_prompt = !prompt.is_empty();

                let q_type_str = q["question_type"].as_str().unwrap_or("");
                let question_type = if !has_prompt || q_type_str == "reference_only" {
                    let ref_desc = q["source_reference"].as_str()
                        .or_else(|| q["source_provenance"]["source_book"].as_str())
                        .unwrap_or(chapter)
                        .to_string();
                    QuestionType::ReferenceOnly { source_reference: ref_desc }
                } else if q_type_str == "mcq" || q.get("options").and_then(|o| o.as_array()).is_some() {
                    let options: Vec<String> = q["options"].as_array()
                        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                        .unwrap_or_default();
                    let correct_opt = q["correct_option"].as_str()
                        .or_else(|| q["correct_answer"].as_str())
                        .or_else(|| q["answer"].as_str())
                        .unwrap_or("")
                        .to_string();
                    let explanation = q["explanation"].as_str().map(|s| s.to_string());
                    QuestionType::Mcq {
                        options,
                        correct_option: correct_opt,
                        explanation,
                    }
                } else if q_type_str == "numerical" || q.get("answer").and_then(|a| a.as_f64()).is_some() || q.get("correct_answer").and_then(|a| a.as_f64()).is_some() {
                    let ans = q["answer"].as_f64()
                        .or_else(|| q["correct_answer"].as_f64())
                        .unwrap_or(0.0);
                    let tol = q["tolerance"].as_f64();
                    QuestionType::Numerical {
                        answer: ans,
                        tolerance: tol,
                    }
                } else if q.get("steps").is_some() && !q["steps"].is_null() {
                    QuestionType::Structured {
                        steps: q["steps"].clone(),
                    }
                } else {
                    let ref_desc = q["source_reference"].as_str().unwrap_or(chapter).to_string();
                    QuestionType::ReferenceOnly { source_reference: ref_desc }
                };

                let schema_id = if let Some(sid) = q["schema_id"].as_str() {
                    if sid.starts_with("schema.") { SchemaId::new(sid) } else { SchemaId::new(format!("schema.{}", sid)) }
                } else if let Some(pid) = q["pattern_id"].as_str() {
                    SchemaId::new(format!("schema.{}", pid))
                } else {
                    SchemaId::new(format!("schema.{}_default", chapter.to_lowercase().replace(' ', "_")))
                };

                let family_id = if let Some(fid) = q["problem_family"].as_str().or_else(|| q["problem_family_id"].as_str()) {
                    if fid.starts_with("family.") { ProblemFamilyId::new(fid) } else { ProblemFamilyId::new(format!("family.{}", fid)) }
                } else if let Some(pid) = q["pattern_id"].as_str() {
                    ProblemFamilyId::new(format!("family.{}", pid))
                } else {
                    ProblemFamilyId::new(format!("family.{}_default", chapter.to_lowercase().replace(' ', "_")))
                };

                if !profile.supported_schemas.contains(&schema_id) {
                    profile.supported_schemas.push(schema_id.clone());
                }
                if !profile.supported_problem_families.contains(&family_id) {
                    profile.supported_problem_families.push(family_id.clone());
                }

                let prov = ContentProvenance::new_pyq_derived(
                    PyqId::new(format!("pyq_{}", q_id_str)), 1, 1, 1, 1, "authentic_pyq", None
                );

                let mut item = PracticeItem::new(
                    item_id,
                    origin,
                    domain.clone(),
                    chapter,
                    skill_id.clone(),
                    schema_id.clone(),
                    family_id.clone(),
                    question_type,
                    prompt,
                    prov
                );

                if let Some(diff) = q["difficulty"].as_f64() {
                    item.difficulty = diff;
                }
                if let Some(tags) = q["structural_tags"].as_array() {
                    item.structural_tags = tags.iter().filter_map(|t| t.as_str().map(|s| s.to_string())).collect();
                }
                if let Some(dps) = q["decision_points"].as_array() {
                    item.decision_points = dps.iter().filter_map(|t| t.as_str().map(|s| s.to_string())).collect();
                }
                if let Some(errs) = q["error_categories"].as_array() {
                    item.error_categories = errs.iter().filter_map(|t| t.as_str().map(|s| s.to_string())).collect();
                }
                if let Some(prereqs) = q["prerequisites"].as_array() {
                    item.prerequisites = prereqs.iter().filter_map(|t| t.as_str().map(|s| s.to_string())).collect();
                }

                item.metadata = q.clone();
                store.insert_practice_item(&item)?;
            }
        }

        store.insert_chapter_profile(&profile)?;
        Ok(())
    }
}
