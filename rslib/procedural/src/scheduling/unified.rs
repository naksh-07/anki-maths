// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::core::{Domain, SchemaId, SkillId};
use crate::diagnostics::ErrorCategory;
use crate::exam::{ExamProfile, ExamRelevanceScorer, PYQSource};
use crate::practice::{
    PracticeObjective, PracticeRequest, PracticeScope, RemediationPrecedence, SchemaPracticeObject,
};
use crate::problems::registry::ProblemRegistry;
use crate::problems::ProblemInstance;
use crate::remediation::{
    RemediationActionKind, RemediationIntervention, RemediationQueue, RemediationSelector,
    RemediationUrgency,
};
use crate::scheduling::difficulty::AdaptiveDifficultyEngine;
use crate::scheduling::selector::MultiSchemaSelectionDecision;
use crate::scheduling::{PracticeSessionObject, SessionReadiness};
use crate::skills::prerequisites::{PrerequisiteEvaluation, PrerequisiteGraphService, PrerequisiteReadiness};
use crate::skills::{PracticeProgressionState, SkillState};
use crate::storage::ProceduralStore;

/// Explicit named priority tiers establishing explainable selection precedence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PriorityTier {
    /// Tier 1: Explicit user-focused single-skill or single-schema constraint.
    ExplicitScope,
    /// Tier 2: Explicit difficulty or latency constraint overrides.
    ExplicitConstraint,
    /// Tier 3: Critical remediation for urgent concept or strategy breakdowns.
    CriticalRemediation,
    /// Tier 4: Normal queued remediation intervention.
    NormalRemediation,
    /// Tier 5: Exam blueprint high-yield topic and PYQ relevance.
    ExamRelevance,
    /// Tier 6: Weak skill reinforcement / low accuracy / diagnostic sweeps.
    WeaknessAndDiagnostics,
    /// Tier 7: Fluency and speed reinforcement for slow attempts.
    FluencyAndSpeed,
    /// Tier 8: Controlled progression difficulty advancement.
    ControlledAdvancement,
    /// Tier 9: Cross-schema interleaving anti-priming adjustments.
    AntiPrimingInterleaving,
    /// Tier 10: Standard baseline rotation.
    StableRotation,
}

/// Concrete learning object produced by the unified selection engine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "learning_object_type", rename_all = "snake_case")]
pub enum LearningObjectKind {
    /// Standard procedural problem instance.
    ProceduralProblem(ProblemInstance),
    /// Authentic PYQ or derived validated PYQ variant.
    PyqVariant(ProblemInstance),
    /// Concrete executable remediation intervention (ConceptCheck, StrategyDrill, WorkedExample, DeclarativeRecall, PrerequisiteReview).
    Remediation(RemediationIntervention),
}

/// Structured decision from the unified selection pipeline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnifiedSelectionDecision {
    pub schema: SchemaPracticeObject,
    pub skill_id: SkillId,
    pub domain: Domain,
    pub learning_object: LearningObjectKind,
    pub difficulty_level: u32,
    pub target_time_ms: u64,
    pub selected_variant: Option<String>,
    pub selection_reason: String,
    pub priority_score: f64,
    pub priority_tier: PriorityTier,
    pub readiness: PrerequisiteEvaluation,
    pub advisory_warning: Option<String>,
}

impl UnifiedSelectionDecision {
    /// Converts this decision into a backward-compatible `PracticeSessionObject`.
    pub fn into_practice_session(self, card_id: Option<i64>, skill_state: Option<SkillState>) -> PracticeSessionObject {
        let instance = match self.learning_object {
            LearningObjectKind::ProceduralProblem(p) => p,
            LearningObjectKind::PyqVariant(mut p) => {
                // Ensure provenance metadata exists
                if !p.metadata.get("provenance").is_some() {
                    let mut meta_map = p.metadata.as_object().cloned().unwrap_or_default();
                    meta_map.insert(
                        "provenance".to_string(),
                        serde_json::json!({
                            "variant_type": self.selected_variant.as_deref().unwrap_or("practice_variant")
                        }),
                    );
                    p.metadata = serde_json::Value::Object(meta_map);
                }
                p
            }
            LearningObjectKind::Remediation(ref rem) => match rem {
                RemediationIntervention::ProceduralProblem(p)
                | RemediationIntervention::TransferRetry(p) => p.clone(),
                RemediationIntervention::ConceptCheck(c) => ProblemInstance::new(
                    crate::core::ProblemInstanceId::new(format!("inst-cc-{}", c.id)),
                    self.schema.problem_family_id.clone(),
                    0,
                    serde_json::json!({
                        "object_type": "concept_check",
                        "concept_check": c,
                        "remediation_message": "💡 Conceptual Check: Let's verify the core principle before proceeding."
                    }),
                    c.prompt.clone(),
                    serde_json::json!({
                        "expected_option_id": c.expected_option_id,
                        "explanation": c.explanation
                    }),
                ),
                RemediationIntervention::StrategyDrill(s) => ProblemInstance::new(
                    crate::core::ProblemInstanceId::new(format!("inst-sd-{}", s.id)),
                    self.schema.problem_family_id.clone(),
                    0,
                    serde_json::json!({
                        "object_type": "strategy_drill",
                        "strategy_drill": s,
                        "remediation_message": "🧭 Strategy Drill: Identify the optimal model/method before calculating."
                    }),
                    s.prompt.clone(),
                    serde_json::json!({
                        "preferred_option_id": s.preferred_option_id,
                        "explanation": s.explanation
                    }),
                ),
                RemediationIntervention::WorkedExample(w) => ProblemInstance::new(
                    crate::core::ProblemInstanceId::new(format!("inst-we-{}", w.id)),
                    self.schema.problem_family_id.clone(),
                    0,
                    serde_json::json!({
                        "object_type": "worked_example",
                        "worked_example": w,
                        "remediation_message": "📖 Step-by-Step Worked Example: Review canonical solution method."
                    }),
                    w.prompt.clone(),
                    serde_json::json!({
                        "canonical_steps": w.canonical_steps,
                        "method_rationale": w.method_rationale
                    }),
                ),
                RemediationIntervention::DeclarativeRecall(d) => ProblemInstance::new(
                    crate::core::ProblemInstanceId::new(format!("inst-dr-{}", d.id)),
                    self.schema.problem_family_id.clone(),
                    0,
                    serde_json::json!({
                        "object_type": "declarative_recall",
                        "declarative_recall": d,
                        "remediation_message": "🧠 Declarative Bridge: Recall prerequisite formula or concept."
                    }),
                    format!("Declarative Recall: {}", d.prompt_summary),
                    serde_json::json!({
                        "formula_or_fact": d.formula_or_fact,
                        "target_anki_card_id": d.target_anki_card_id,
                        "target_anki_tag": d.target_anki_tag
                    }),
                ),
                RemediationIntervention::PrerequisiteReview(prereq_obj) => {
                    if let Some(ref p) = prereq_obj.executable_problem {
                        p.clone()
                    } else {
                        ProblemInstance::new(
                            crate::core::ProblemInstanceId::new(format!("inst-pr-{}", prereq_obj.id)),
                            self.schema.problem_family_id.clone(),
                            0,
                            serde_json::json!({
                                "object_type": "prerequisite_review",
                                "prerequisite_review": prereq_obj,
                                "remediation_message": "⚠️ Prerequisite Foundation: Foundational skill reinforcement recommended."
                            }),
                            format!(
                                "Prerequisite Recommendation: {}\n{}",
                                prereq_obj.recommendation_summary, prereq_obj.advisory_message
                            ),
                            serde_json::json!({"ready": false}),
                        )
                    }
                }
                RemediationIntervention::RepresentationDrill(r) => ProblemInstance::new(
                    crate::core::ProblemInstanceId::new(format!("inst-rd-{}", r.id)),
                    self.schema.problem_family_id.clone(),
                    0,
                    serde_json::json!({
                        "object_type": "concept_check",
                        "remediation_message": "📊 Representation Check: Select appropriate structured diagram or form."
                    }),
                    r.prompt.clone(),
                    serde_json::json!({
                        "expected_option_id": r.expected_option_id,
                        "explanation": r.explanation
                    }),
                ),
                RemediationIntervention::CircuitBreaker(cb) => ProblemInstance::new(
                    crate::core::ProblemInstanceId::new(format!("inst-cb-{}", cb.id)),
                    self.schema.problem_family_id.clone(),
                    0,
                    serde_json::json!({
                        "object_type": "circuit_breaker",
                        "circuit_breaker": cb,
                        "remediation_message": "⏸️ Circuit Breaker Cooldown: Multiple repeated failures detected. Let's take a pause on this specific template."
                    }),
                    format!(
                        "Learning Cooldown: {}\nAction: {}",
                        cb.advisory_message, cb.suggested_action
                    ),
                    serde_json::json!({"circuit_breaker_active": true, "recurrence": cb.recurrence_count}),
                ),
            },
        };

        let session_readiness = match self.readiness.readiness {
            PrerequisiteReadiness::Ready => SessionReadiness::Ready,
            PrerequisiteReadiness::ReadyWithWarnings { .. } => SessionReadiness::Ready,
            PrerequisiteReadiness::PrerequisitesNeeded { ref missing_skills } => {
                SessionReadiness::PrerequisitesNeeded {
                    missing_skills: missing_skills.clone(),
                }
            }
            PrerequisiteReadiness::Unknown => SessionReadiness::Ready,
        };

        let mut session = PracticeSessionObject::new(self.schema, instance, card_id, skill_state);
        session.readiness = session_readiness;
        session.selected_variant = self.selected_variant;
        session.target_latency_ms = Some(self.target_time_ms);
        session.selection_reason = Some(self.selection_reason);
        session.difficulty_level = Some(self.difficulty_level);
        session
    }

    /// Converts this decision into a backward-compatible `MultiSchemaSelectionDecision`.
    pub fn to_multi_schema_decision(&self) -> MultiSchemaSelectionDecision {
        MultiSchemaSelectionDecision {
            schema: self.schema.clone(),
            difficulty_level: self.difficulty_level,
            target_time_ms: self.target_time_ms,
            selected_variant: self.selected_variant.clone(),
            selection_reason: self.selection_reason.clone(),
            priority_score: self.priority_score,
        }
    }
}

/// Unified next-learning-object selection engine orchestrating prerequisites,
/// remediation, exam relevance, anti-priming, and strict user-intent priority.
pub struct UnifiedPracticeEngine;

impl UnifiedPracticeEngine {
    /// Single unified decision pipeline selecting the next optimal learning object.
    pub fn select_next(
        request: &PracticeRequest,
        candidate_schemas: &[SchemaPracticeObject],
        schema_domains: &HashMap<SchemaId, Domain>,
        skill_states: &HashMap<SkillId, SkillState>,
        prerequisite_service: &PrerequisiteGraphService,
        remediation_queue: Option<&mut RemediationQueue>,
        exam_profile: Option<&ExamProfile>,
        eligible_pyqs: &HashMap<SchemaId, Vec<PYQSource>>,
        last_schema_id: Option<&SchemaId>,
        registry: &ProblemRegistry,
        store: &ProceduralStore,
        seed: u64,
    ) -> Option<UnifiedSelectionDecision> {
        if candidate_schemas.is_empty() {
            return None;
        }

        // ---------------------------------------------------------------------
        // STAGE 1: Scope Filtering (Strict User-Intent Precedence)
        // ---------------------------------------------------------------------
        let scoped_candidates: Vec<&SchemaPracticeObject> = candidate_schemas
            .iter()
            .filter(|s| {
                let domain = schema_domains
                    .get(&s.id)
                    .cloned()
                    .unwrap_or(Domain::Mathematics);
                request.scope.matches_schema(&s.id, &s.skill_id, &domain)
            })
            .collect();

        // In focused mode, NEVER escape the user's requested scope
        let pool = if request.scope.is_focused() {
            if scoped_candidates.is_empty() {
                return None;
            }
            scoped_candidates
        } else if scoped_candidates.is_empty() {
            candidate_schemas.iter().collect::<Vec<_>>()
        } else {
            scoped_candidates
        };

        // ---------------------------------------------------------------------
        // STAGE 2: Critical Remediation Check (Precedence Tier 3)
        // ---------------------------------------------------------------------
        if let Some(queue) = remediation_queue {
            if request.remediation_policy != RemediationPrecedence::Disabled {
                // Find pending actions matching scope
                let mut chosen_action = None;
                for action in queue.iter_pending() {
                    let domain = action.domain.clone();
                    let matches_scope = match &request.scope {
                        PracticeScope::AllDomains => true,
                        PracticeScope::SingleDomain(d) => d == &domain,
                        PracticeScope::SingleSkill(s) => s == &action.skill_id,
                        PracticeScope::SingleSchema(sch) => sch == &action.schema_id,
                        PracticeScope::MultipleSkills(skills) => skills.contains(&action.skill_id),
                        PracticeScope::MultipleSchemas(schemas) => schemas.contains(&action.schema_id),
                    };

                    if matches_scope {
                        let is_critical = action.urgency == RemediationUrgency::Critical
                            || action.recurrence_count >= 2
                            || matches!(action.kind, RemediationActionKind::ConceptCheck | RemediationActionKind::PrerequisiteReview);

                        if is_critical || request.remediation_policy == RemediationPrecedence::AllEligible {
                            chosen_action = Some(action.clone());
                            break;
                        }
                    }
                }

                if let Some(action) = chosen_action {
                    // Check if advisory only
                    if request.remediation_policy != RemediationPrecedence::AdvisoryOnly {
                        if let Ok(intervention) = RemediationSelector::select_intervention(&action, store, registry, seed) {
                            if let Ok(Some(schema)) = store.get_schema(&action.schema_id) {
                                let domain = schema_domains.get(&schema.id).cloned().unwrap_or(Domain::Mathematics);
                                let readiness = prerequisite_service.evaluate_readiness(&schema.skill_id, skill_states);
                                let target_time = 30_000;
                                let diff = action.preferred_difficulty;

                                return Some(UnifiedSelectionDecision {
                                    schema: schema.clone(),
                                    skill_id: schema.skill_id.clone(),
                                    domain,
                                    learning_object: LearningObjectKind::Remediation(intervention),
                                    difficulty_level: diff,
                                    target_time_ms: target_time,
                                    selected_variant: action.preferred_variant,
                                    selection_reason: format!("remediation_{}", action.kind.as_str()),
                                    priority_score: 1500.0 + (action.recurrence_count as f64 * 50.0),
                                    priority_tier: if action.urgency == RemediationUrgency::Critical {
                                        PriorityTier::CriticalRemediation
                                    } else {
                                        PriorityTier::NormalRemediation
                                    },
                                    readiness,
                                    advisory_warning: None,
                                });
                            }
                        }
                    }
                }
            }
        }

        // ---------------------------------------------------------------------
        // STAGE 3: Score and Rank Candidates across Objectives & Prerequisites
        // ---------------------------------------------------------------------
        let mut scored_candidates: Vec<(&SchemaPracticeObject, f64, PriorityTier, String, PrerequisiteEvaluation)> = pool
            .iter()
            .map(|&schema| {
                let domain = schema_domains.get(&schema.id).cloned().unwrap_or(Domain::Mathematics);
                let state = skill_states.get(&schema.skill_id);
                let readiness = prerequisite_service.evaluate_readiness(&schema.skill_id, skill_states);

                let (mut score, tier, reason) = Self::compute_candidate_score(
                    schema,
                    &domain,
                    state,
                    request,
                    &readiness,
                    exam_profile,
                    eligible_pyqs,
                );

                // Stage-aware anti-priming interleaving penalty across different schemas
                let stage = state.map_or(PracticeProgressionState::New, |s| s.practice_state);
                let interleaving_policy = crate::scheduling::interleaving::InterleavingPolicy::for_stage(stage);
                let penalty = interleaving_policy.compute_penalty(&schema.id, last_schema_id, &request.scope);
                score += penalty;

                (schema, score, tier, reason, readiness)
            })
            .collect();

        // Sort descending by priority score
        scored_candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let (chosen_schema, score, tier, reason, readiness) = scored_candidates[0].clone();
        let domain = schema_domains
            .get(&chosen_schema.id)
            .cloned()
            .unwrap_or(Domain::Mathematics);
        let state = skill_states.get(&chosen_schema.skill_id);

        // ---------------------------------------------------------------------
        // STAGE 4: Difficulty and Latency Evaluation
        // ---------------------------------------------------------------------
        let (mode_forced_level, mode_latency) = Self::resolve_objective_overrides(request.objective, state);
        let diff_decision = AdaptiveDifficultyEngine::evaluate_difficulty(state, mode_forced_level, mode_latency);

        // Apply explicit request difficulty constraint if present
        let final_difficulty = if let Some(ref constraint) = request.difficulty_constraint {
            constraint.clamp_level(diff_decision.level)
        } else {
            diff_decision.level
        };

        // Apply explicit request time constraint if present
        let final_target_time = if let Some(ref tc) = request.time_constraint {
            tc.target_latency_ms.unwrap_or(diff_decision.target_time_ms)
        } else {
            diff_decision.target_time_ms
        };

        // ---------------------------------------------------------------------
        // STAGE 5: Generate Problem Instance or Select PYQ Variant
        // ---------------------------------------------------------------------
        let mut learning_object = None;

        // Check if authentic PYQ or variant is available and requested for Exam objective
        if request.objective == PracticeObjective::Exam {
            if let Some(pyq_list) = eligible_pyqs.get(&chosen_schema.id) {
                if let Some(_pyq) = pyq_list.first() {
                    if let Ok(Some(family)) = store.get_problem_family(&chosen_schema.problem_family_id) {
                        if let Ok(instance) = registry.generate(
                            &chosen_schema.problem_family_id,
                            &family.template_ref,
                            seed,
                            final_difficulty,
                            None,
                        ) {
                            learning_object = Some(LearningObjectKind::PyqVariant(instance));
                        }
                    }
                }
            }
        }

        if learning_object.is_none() {
            let family = store
                .get_problem_family(&chosen_schema.problem_family_id)
                .ok()
                .flatten()
                .unwrap_or_else(|| {
                    crate::problems::ProblemFamily::new(
                        chosen_schema.problem_family_id.clone(),
                        chosen_schema.skill_id.clone(),
                        domain.clone(),
                        "Problem Family",
                        "default.v1",
                    )
                });

            let instance = registry
                .generate(
                    &chosen_schema.problem_family_id,
                    &family.template_ref,
                    seed,
                    final_difficulty,
                    None,
                )
                .unwrap_or_else(|_| {
                    ProblemInstance::new(
                        crate::core::ProblemInstanceId::new("inst-fallback"),
                        chosen_schema.problem_family_id.clone(),
                        seed,
                        serde_json::json!({}),
                        "Practice problem instance.",
                        serde_json::json!({}),
                    )
                });

            learning_object = Some(LearningObjectKind::ProceduralProblem(instance));
        }

        let advisory_warning = readiness.advisory_message.clone();

        // Structural coverage recommendation
        let recommended_variant = state.map(|s| {
            let profile = crate::scheduling::coverage::StructuralCoverageProfile::from_skill_state(s);
            let cat = profile.recommend_next_category(s.practice_state);
            cat.as_str().to_string()
        });

        Some(UnifiedSelectionDecision {
            schema: chosen_schema.clone(),
            skill_id: chosen_schema.skill_id.clone(),
            domain,
            learning_object: learning_object.unwrap(),
            difficulty_level: final_difficulty,
            target_time_ms: final_target_time,
            selected_variant: recommended_variant,
            selection_reason: reason,
            priority_score: score,
            priority_tier: tier,
            readiness,
            advisory_warning,
        })
    }

    /// Unified practice selection coordinating under an active multi-domain macro session plan.
    pub fn select_next_with_macro_plan(
        request: &PracticeRequest,
        macro_plan: &crate::scheduling::macro_allocator::MacroSessionPlan,
        budget_tracker: &crate::scheduling::workload::SessionBudgetTracker,
        candidate_schemas: &[SchemaPracticeObject],
        schema_domains: &HashMap<SchemaId, Domain>,
        skill_states: &HashMap<SkillId, SkillState>,
        prerequisite_service: &PrerequisiteGraphService,
        remediation_queue: Option<&mut RemediationQueue>,
        exam_profile: Option<&ExamProfile>,
        eligible_pyqs: &HashMap<SchemaId, Vec<PYQSource>>,
        last_schema_id: Option<&SchemaId>,
        registry: &ProblemRegistry,
        store: &ProceduralStore,
        seed: u64,
    ) -> Option<UnifiedSelectionDecision> {
        // If user scope is focused (Tier 1), user intent is 100% authoritative and overrides macro block
        if request.scope.is_focused() {
            return Self::select_next(
                request,
                candidate_schemas,
                schema_domains,
                skill_states,
                prerequisite_service,
                remediation_queue,
                exam_profile,
                eligible_pyqs,
                last_schema_id,
                registry,
                store,
                seed,
            );
        }

        // Determine current active domain from macro plan and elapsed session time
        let current_domain = macro_plan
            .active_domain_at_elapsed(budget_tracker.elapsed_ms)
            .unwrap_or(Domain::Mathematics);

        // Check if current domain budget is exhausted; if so, switch to next available non-exhausted domain
        let target_domain = if macro_plan.is_domain_exhausted(&current_domain, budget_tracker.domain_elapsed_ms.get(&current_domain).copied().unwrap_or(0)) {
            macro_plan
                .active_domains
                .iter()
                .find(|d| !macro_plan.is_domain_exhausted(d, budget_tracker.domain_elapsed_ms.get(d).copied().unwrap_or(0)))
                .cloned()
                .unwrap_or(current_domain)
        } else {
            current_domain
        };

        // Filter candidate schemas to the target domain
        let domain_filtered_candidates: Vec<SchemaPracticeObject> = candidate_schemas
            .iter()
            .filter(|s| {
                schema_domains
                    .get(&s.id)
                    .map_or(true, |d| d == &target_domain)
            })
            .cloned()
            .collect();

        let pool = if domain_filtered_candidates.is_empty() {
            candidate_schemas
        } else {
            &domain_filtered_candidates
        };

        Self::select_next(
            request,
            pool,
            schema_domains,
            skill_states,
            prerequisite_service,
            remediation_queue,
            exam_profile,
            eligible_pyqs,
            last_schema_id,
            registry,
            store,
            seed,
        )
    }

    /// Compute candidate priority score, tier, and explainable rationale.
    fn compute_candidate_score(
        schema: &SchemaPracticeObject,
        domain: &Domain,
        state: Option<&SkillState>,
        request: &PracticeRequest,
        readiness: &PrerequisiteEvaluation,
        exam_profile: Option<&ExamProfile>,
        eligible_pyqs: &HashMap<SchemaId, Vec<PYQSource>>,
    ) -> (f64, PriorityTier, String) {
        // 1. Explicit Scope Priority (Tier 1)
        if request.scope.is_focused() {
            let reason = if readiness.requires_intervention() {
                format!("focused_practice_with_prerequisite_warnings: {}", readiness.advisory_message.as_deref().unwrap_or(""))
            } else {
                "focused_user_scope_authoritative".to_string()
            };
            return (2000.0, PriorityTier::ExplicitScope, reason);
        }

        // 2. Exam Objective & Blueprint Weighting (Tier 5)
        if request.objective == PracticeObjective::Exam {
            if let Some(profile) = exam_profile {
                let pyq_count = eligible_pyqs.get(&schema.id).map_or(0, |l| l.len());
                let has_pyqs = pyq_count > 0;
                let relevance = ExamRelevanceScorer::calculate_score(
                    profile,
                    schema,
                    domain,
                    state,
                    has_pyqs,
                    false,
                    &crate::exam::ExamPracticeMode::ExamPreparation,
                );
                let score = 900.0 + (relevance.total_score * 4.0);
                return (score, PriorityTier::ExamRelevance, format!("exam_relevance_weight_{:.2}", relevance.total_score));
            }
        }

        // 3. Transfer Practice Gating (Tier 8)
        if request.objective == PracticeObjective::Transfer {
            let elig = if let Some(s) = state {
                crate::scheduling::transfer::TransferEngine::evaluate_eligibility(s, crate::scheduling::transfer::TransferLevel::NearTransfer, true)
            } else {
                crate::scheduling::transfer::TransferEligibilityEvaluation {
                    target_level: crate::scheduling::transfer::TransferLevel::NearTransfer,
                    is_eligible: false,
                    max_eligible_level: None,
                    reasons: vec!["No practice history for skill".into()],
                }
            };
            if elig.is_eligible {
                return (1500.0, PriorityTier::ControlledAdvancement, "transfer_eligible_mastery_met".to_string());
            } else {
                return (300.0, PriorityTier::ControlledAdvancement, format!("transfer_ineligible: {}", elig.reasons.join(", ")));
            }
        }

        // 4. Diagnostic Sweep Objective (Tier 6)
        if request.objective == PracticeObjective::Diagnose {
            let attempts = state.map_or(0, |s| s.total_attempts);
            let score = 1000.0 - (attempts as f64 * 100.0).min(800.0);
            return (score, PriorityTier::WeaknessAndDiagnostics, "diagnostic_topic_coverage_sweep".to_string());
        }

        // 5. Speed / Fluency Objective (Tier 7)
        if request.objective == PracticeObjective::Speed {
            let score = 750.0 + state.map_or(0.0, |s| s.recent_accuracy() * 200.0);
            return (score, PriorityTier::FluencyAndSpeed, "speed_fluency_drill".to_string());
        }

        // 6. Learn / Foundational Objective
        if request.objective == PracticeObjective::Learn {
            let score = 800.0 - state.map_or(0.0, |s| (s.total_attempts as f64 * 50.0).min(400.0));
            return (score, PriorityTier::WeaknessAndDiagnostics, "foundational_learning_scaffold".to_string());
        }

        // 7. Standard Adaptive Practice Evaluation
        let Some(s) = state else {
            // Cold start: Introduce new skill
            let base_score = if readiness.requires_intervention() { 350.0 } else { 500.0 };
            return (base_score, PriorityTier::StableRotation, "new_unseen_skill".to_string());
        };

        // Retired skills participate in low-frequency maintenance
        if s.practice_state == PracticeProgressionState::Retired || s.practice_state == PracticeProgressionState::Hibernating {
            return (250.0, PriorityTier::StableRotation, "retired_skill_low_frequency_maintenance".to_string());
        }

        let last_attempt = s.recent_attempts.last();
        let last_failed = last_attempt.map_or(false, |a| !a.is_correct);
        let recent_acc = s.recent_accuracy();

        // Critical failure / concept breakdown
        if last_failed {
            if let Some(err_cat) = last_attempt.and_then(|a| a.error_category.as_ref()) {
                if matches!(err_cat, ErrorCategory::Concept | ErrorCategory::Conceptual | ErrorCategory::Strategy) {
                    return (1200.0, PriorityTier::CriticalRemediation, "critical_remediation_concept_breakdown".to_string());
                }
            }
            return (1000.0 + (s.consecutive_failures as f64 * 50.0), PriorityTier::NormalRemediation, "remediation_recent_failure".to_string());
        }

        // Weak skill / low accuracy
        if s.practice_state == PracticeProgressionState::Learning || (s.recent_attempts.len() >= 3 && recent_acc < 0.5) {
            return (800.0 - (recent_acc * 200.0), PriorityTier::WeaknessAndDiagnostics, "weak_skill_reinforcement".to_string());
        }

        // Slow execution latency
        let last_latency = last_attempt.map_or(0, |a| a.latency_ms);
        let last_target = last_attempt.map_or(35_000, |a| a.target_latency_ms);
        if last_latency > (last_target as f64 * 1.25) as u64 {
            return (650.0, PriorityTier::FluencyAndSpeed, "fluency_reinforcement_slow_latency".to_string());
        }

        // Controlled difficulty advancement
        if s.consecutive_successes >= 2 && recent_acc >= 0.8 {
            return (450.0, PriorityTier::ControlledAdvancement, "controlled_difficulty_advancement".to_string());
        }

        // Stable normal rotation (soft penalty if prerequisites are missing)
        let prereq_penalty = if readiness.requires_intervention() { 100.0 } else { 0.0 };
        (400.0 - prereq_penalty, PriorityTier::StableRotation, "normal_rotation".to_string())
    }

    /// Resolves forced difficulty level and latency override from objective.
    pub fn resolve_objective_overrides(
        objective: PracticeObjective,
        state: Option<&SkillState>,
    ) -> (Option<u32>, Option<u64>) {
        match objective {
            PracticeObjective::Learn => (Some(1), None),
            PracticeObjective::Speed => (Some(1), Some(20_000)),
            PracticeObjective::Transfer => {
                let elig = if let Some(s) = state {
                    crate::scheduling::transfer::TransferEngine::evaluate_eligibility(s, crate::scheduling::transfer::TransferLevel::NearTransfer, true).is_eligible
                } else {
                    false
                };
                if elig {
                    (Some(5), None)
                } else {
                    (None, None)
                }
            }
            PracticeObjective::Exam => (Some(3), Some(35_000)),
            PracticeObjective::Mock => (Some(3), Some(30_000)),
            _ => (None, None),
        }
    }
}
