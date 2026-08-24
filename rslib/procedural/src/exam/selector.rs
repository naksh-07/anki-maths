// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::core::{Domain, ProblemInstanceId, SchemaId, SkillId};
use crate::diagnostics::ErrorCategory;
use crate::exam::pipeline::PyqVariantPipeline;
use crate::exam::profile::ExamProfile;
use crate::exam::pyq::{PYQSource, PyqMapping};
use crate::practice::SchemaPracticeObject;
use crate::problems::registry::ProblemRegistry;
use crate::problems::ProblemInstance;
use crate::scheduling::difficulty::AdaptiveDifficultyEngine;
use crate::scheduling::{PracticeSessionObject, SessionReadiness};
use crate::skills::SkillState;

/// Dedicated practice modes tailored for competitive exam preparation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum ExamPracticeMode {
    /// Comprehensive weighted practice guided by exam blueprint and learner mastery
    ExamPreparation,
    /// Authentic Previous Year Questions (PYQs) only
    PyqPractice,
    /// Authentic PYQ followed by confirmed procedural variants
    PyqAndVariants,
    /// Aggressively prioritize weak and high-error exam topics
    WeakAreas,
    /// Speed drills under exam target latencies on familiar topics
    SpeedTraining,
    /// Balanced interleaving across all subjects/domains in the exam profile
    MixedExam,
    /// Timed, strict simulation replicating exact exam blueprint distribution
    Mock,
}

impl Default for ExamPracticeMode {
    fn default() -> Self {
        ExamPracticeMode::ExamPreparation
    }
}

/// Explainable scoring breakdown for exam candidate selection.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExamRelevanceScore {
    pub total_score: f64,
    pub domain_weight_component: f64,
    pub topic_weight_component: f64,
    pub mastery_urgency_component: f64,
    pub error_urgency_component: f64,
    pub latency_gap_component: f64,
    pub pyq_presence_bonus: f64,
    pub anti_priming_penalty: f64,
    pub mode_modifier: f64,
    pub rationale: String,
}

/// Scorer calculating deterministic, explainable priority scores for schemas under an ExamProfile.
pub struct ExamRelevanceScorer;

impl ExamRelevanceScorer {
    pub fn calculate_score(
        profile: &ExamProfile,
        schema: &SchemaPracticeObject,
        domain: &Domain,
        state: Option<&SkillState>,
        has_eligible_pyqs: bool,
        is_last_practiced: bool,
        mode: &ExamPracticeMode,
    ) -> ExamRelevanceScore {
        let domain_w = profile.get_domain_weight(domain);
        let topic_w = profile.get_topic_weight(schema.id.as_str());

        let domain_component = domain_w * 100.0;
        let topic_component = (topic_w - 1.0) * 50.0;

        let mastery = state.map_or(0.0, |s| s.mastery);
        let recent_acc = state.map_or(0.0, |s| s.recent_accuracy());
        let attempts = state.map_or(0, |s| s.total_attempts);

        // 1. Mastery urgency: lower mastery = higher priority
        let mastery_urgency = if attempts == 0 {
            150.0 // Cold start unpracticed topic
        } else {
            (1.0 - mastery.clamp(0.0, 1.0)) * 200.0 + (1.0 - recent_acc.clamp(0.0, 1.0)) * 100.0
        };

        // 2. Error urgency: recent errors increase urgency
        let mut error_urgency = 0.0;
        if let Some(s) = state {
            let recent_errors = s.recent_attempts.iter().filter(|a| !a.is_correct).count();
            error_urgency = recent_errors as f64 * 60.0;

            // Concept breakdown amplifier
            for a in s.recent_attempts.iter().rev().take(3) {
                if !a.is_correct && a.error_category.as_ref().map_or(false, |c| matches!(c, ErrorCategory::Concept | ErrorCategory::Conceptual)) {
                    error_urgency += 100.0;
                    break;
                }
            }
        }

        // 3. Latency gap: learner taking longer than exam target latency
        let target_lat = profile.get_target_latency_ms(&schema.id, domain);
        let avg_lat = state.map_or(target_lat as f64, |s| s.latency_stats.moving_average_ms);
        let latency_gap = if avg_lat > (target_lat as f64 * 1.2) {
            75.0
        } else {
            0.0
        };

        // 4. PYQ presence bonus
        let pyq_bonus = if has_eligible_pyqs {
            profile.pyq_weight * 40.0
        } else {
            0.0
        };

        // 5. Anti-priming penalty for immediately repeated schema
        let anti_priming = if is_last_practiced { -250.0 } else { 0.0 };

        // 6. Mode-specific modifier
        let mut mode_modifier = 0.0;
        match mode {
            ExamPracticeMode::WeakAreas => {
                if mastery < 0.6 || error_urgency > 50.0 {
                    mode_modifier = 300.0;
                }
            }
            ExamPracticeMode::SpeedTraining => {
                if mastery >= 0.7 {
                    mode_modifier = 250.0 + latency_gap;
                } else {
                    mode_modifier = -100.0; // Avoid speed training on unfamiliar skills
                }
            }
            ExamPracticeMode::PyqPractice | ExamPracticeMode::PyqAndVariants => {
                if has_eligible_pyqs {
                    mode_modifier = 200.0;
                } else {
                    mode_modifier = -500.0;
                }
            }
            ExamPracticeMode::MixedExam => {
                // Interleaving balance across domains
                mode_modifier = 50.0;
            }
            ExamPracticeMode::ExamPreparation | ExamPracticeMode::Mock => {
                mode_modifier = 0.0;
            }
        }

        let total_score = domain_component
            + topic_component
            + mastery_urgency
            + error_urgency
            + latency_gap
            + pyq_bonus
            + anti_priming
            + mode_modifier;

        let rationale = format!(
            "Exam Relevance (dom={:.2}, top={:.2}) + Urgency (mast={:.0}, err={:.0}) + PYQ Bonus ({:.0})",
            domain_component, topic_component, mastery_urgency, error_urgency, pyq_bonus
        );

        ExamRelevanceScore {
            total_score,
            domain_weight_component: domain_component,
            topic_weight_component: topic_component,
            mastery_urgency_component: mastery_urgency,
            error_urgency_component: error_urgency,
            latency_gap_component: latency_gap,
            pyq_presence_bonus: pyq_bonus,
            anti_priming_penalty: anti_priming,
            mode_modifier,
            rationale,
        }
    }
}

/// Domain-neutral exam adaptation layer and session selector.
pub struct ExamSessionSelector;

impl ExamSessionSelector {
    /// Select the next optimal schema, practice mode, difficulty level, and problem instance
    /// for a target ExamProfile.
    pub fn select_exam_practice(
        profile: &ExamProfile,
        mode: &ExamPracticeMode,
        candidate_schemas: &[SchemaPracticeObject],
        schema_domains: &HashMap<SchemaId, Domain>,
        skill_states: &HashMap<SkillId, SkillState>,
        eligible_pyqs_by_schema: &HashMap<SchemaId, Vec<(PYQSource, PyqMapping)>>,
        last_schema_id: Option<&SchemaId>,
        registry: &ProblemRegistry,
        seed: u64,
    ) -> Option<PracticeSessionObject> {
        if candidate_schemas.is_empty() {
            return None;
        }

        // 1. Filter schemas belonging to subjects in the ExamProfile
        let eligible_schemas: Vec<&SchemaPracticeObject> = candidate_schemas
            .iter()
            .filter(|s| {
                let domain = schema_domains
                    .get(&s.id)
                    .cloned()
                    .unwrap_or(Domain::Mathematics);
                profile.subjects.contains(&domain)
            })
            .collect();

        if eligible_schemas.is_empty() {
            return None;
        }

        // 2. Score all candidate schemas using ExamRelevanceScorer
        let mut scored_schemas: Vec<(&SchemaPracticeObject, ExamRelevanceScore)> = eligible_schemas
            .iter()
            .map(|&schema| {
                let domain = schema_domains
                    .get(&schema.id)
                    .cloned()
                    .unwrap_or(Domain::Mathematics);
                let state = skill_states.get(&schema.skill_id);
                let has_pyqs = eligible_pyqs_by_schema
                    .get(&schema.id)
                    .map_or(false, |list| !list.is_empty());
                let is_last = last_schema_id.map_or(false, |id| id == &schema.id);

                let score = ExamRelevanceScorer::calculate_score(
                    profile,
                    schema,
                    &domain,
                    state,
                    has_pyqs,
                    is_last,
                    mode,
                );
                (schema, score)
            })
            .collect();

        // Sort descending by score
        scored_schemas.sort_by(|a, b| {
            b.1.total_score
                .partial_cmp(&a.1.total_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let (chosen_schema, ref score) = scored_schemas[0];
        let domain = schema_domains
            .get(&chosen_schema.id)
            .cloned()
            .unwrap_or(Domain::Mathematics);
        let state = skill_states.get(&chosen_schema.skill_id);

        // 3. Calibrate Difficulty Level
        let diff_decision = AdaptiveDifficultyEngine::evaluate_difficulty(state, None, None);
        let calibrated_level = Self::calibrate_difficulty_to_profile(profile, diff_decision.level);

        let target_latency_ms = profile.get_target_latency_ms(&chosen_schema.id, &domain);

        // 4. Determine whether to present an authentic PYQ or a procedural variant
        let pyqs = eligible_pyqs_by_schema.get(&chosen_schema.id);
        let should_use_pyq = match mode {
            ExamPracticeMode::PyqPractice => true,
            ExamPracticeMode::PyqAndVariants => {
                // If learner has high mastery or recently succeeded, show variant; otherwise PYQ
                state.map_or(true, |s| s.consecutive_successes == 0)
            }
            _ => pyqs.map_or(false, |list| !list.is_empty()) && (seed % 3 == 0),
        };

        let instance = if should_use_pyq && pyqs.map_or(false, |list| !list.is_empty()) {
            let pyq_list = pyqs.unwrap();
            let idx = (seed as usize) % pyq_list.len();
            let (pyq, mapping) = &pyq_list[idx];
            Self::create_instance_from_pyq(pyq, mapping)
        } else {
            // Generate validated procedural variant
            let mut variant_mapping = PyqMapping::new(
                format!("gen_{}", chosen_schema.id.as_str()),
                domain,
                chosen_schema.skill_id.clone(),
                chosen_schema.id.clone(),
                chosen_schema.problem_family_id.clone(),
                calibrated_level,
                target_latency_ms,
            );
            if let Some(s) = state {
                if s.consecutive_successes >= 2 {
                    variant_mapping.variant_structure = Some("structural".into());
                }
            }

            match PyqVariantPipeline::generate_and_validate_variant(
                registry,
                None,
                &variant_mapping,
                seed,
                None,
            ) {
                Ok(inst) => inst,
                Err((_, _)) => {
                    // Fallback to generator direct instance if validator rejected test seed
                    let gen = registry.get_generator(chosen_schema.problem_family_id.as_str())?;
                    gen.generate(
                        &chosen_schema.problem_family_id,
                        seed,
                        calibrated_level,
                        Some("standard"),
                    )
                    .ok()?
                }
            }
        };

        let mut session = PracticeSessionObject::new(
            (*chosen_schema).clone(),
            instance,
            None,
            state.cloned(),
        )
        .with_readiness(SessionReadiness::Ready);

        session.difficulty_level = Some(calibrated_level);
        session.target_latency_ms = Some(target_latency_ms);
        session.selection_reason = Some(format!(
            "Exam [{}] Mode [{:?}]: {}",
            profile.name, mode, score.rationale
        ));

        Some(session)
    }

    /// Calibrate difficulty based on the target exam profile's difficulty distribution.
    fn calibrate_difficulty_to_profile(profile: &ExamProfile, adaptive_level: u32) -> u32 {
        // If profile concentrates heavily on low difficulty (e.g. RRB ALP), clamp max difficulty
        let high_diff_weight = profile.difficulty_distribution.get(&4).copied().unwrap_or(0.0)
            + profile.difficulty_distribution.get(&5).copied().unwrap_or(0.0);

        if high_diff_weight < 0.05 && adaptive_level > 3 {
            return 3;
        }

        // If profile concentrates heavily on high difficulty (e.g. JEE Main), boost baseline
        let low_diff_weight = profile.difficulty_distribution.get(&1).copied().unwrap_or(0.0);
        if low_diff_weight < 0.05 && adaptive_level < 2 {
            return 2;
        }

        adaptive_level
    }

    /// Convert an authentic PYQ into a playable ProblemInstance.
    pub fn create_instance_from_pyq(pyq: &PYQSource, mapping: &PyqMapping) -> ProblemInstance {
        let prompt = pyq.original_question.clone();
        let is_mcq = pyq.original_options.is_some();

        let mut parameters = serde_json::json!({ "pyq_source_id": pyq.id.as_str() });
        if let Some(ref options) = pyq.original_options {
            parameters["options"] = serde_json::to_value(options).unwrap_or_default();
        }

        let mut metadata = serde_json::json!({
            "is_authentic_pyq": true,
            "source_pyq_id": pyq.id.as_str(),
            "exam": pyq.exam,
            "year": pyq.year,
            "source_reference": pyq.source_reference,
            "provenance": pyq.provenance,
        });

        if is_mcq {
            metadata["object_type"] = serde_json::json!("mcq");
        }

        ProblemInstance::new(
            ProblemInstanceId::new(format!("inst_pyq_{}", pyq.id.as_str())),
            mapping.problem_family_id.clone(),
            0,
            parameters,
            prompt,
            pyq.original_answer.clone(),
        )
        .with_metadata(metadata)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::problems::catalog::SCHEMA_SUCCESSIVE_PERCENTAGE;

    #[test]
    fn test_exam_relevance_scorer_and_selection() {
        let profile = ExamProfile::rrb_alp();
        let schema = SchemaPracticeObject::new(
            SCHEMA_SUCCESSIVE_PERCENTAGE,
            "percentage.successive",
            "family.math.percentage.successive",
            "Successive Percentage",
            "Desc",
        );

        let mut skill_states = HashMap::new();
        let mut state = SkillState::new("percentage.successive");
        state.mastery = 0.3; // Low mastery should boost urgency score
        skill_states.insert(state.skill_id.clone(), state);

        let mut domains = HashMap::new();
        domains.insert(SchemaId::from(SCHEMA_SUCCESSIVE_PERCENTAGE), Domain::Mathematics);

        let pyqs = HashMap::new();
        let registry = ProblemRegistry::default_maths_registry();

        let session = ExamSessionSelector::select_exam_practice(
            &profile,
            &ExamPracticeMode::ExamPreparation,
            &[schema],
            &domains,
            &skill_states,
            &pyqs,
            None,
            &registry,
            42,
        );

        assert!(session.is_some());
        let s = session.unwrap();
        assert_eq!(s.schema.id.as_str(), SCHEMA_SUCCESSIVE_PERCENTAGE);
        assert!(s.selection_reason.unwrap().contains("RRB Assistant Loco Pilot"));
    }
}
