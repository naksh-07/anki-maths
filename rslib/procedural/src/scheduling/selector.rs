// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::collections::HashMap;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

use crate::core::{SchemaId, SkillId};
use crate::diagnostics::ErrorCategory;
use crate::practice::SchemaPracticeObject;
use crate::problems::generators::percentage_successive::PercentageVariant;
use crate::scheduling::difficulty::AdaptiveDifficultyEngine;
use crate::skills::{PracticeProgressionState, SkillState};

/// User-controlled procedural practice modes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum PracticeMode {
    /// Mixed practice interleaving across multiple eligible mathematics schemas.
    MixedMaths,
    /// Mixed practice interleaving across multiple eligible physics schemas.
    MixedPhysics,
    /// Mixed practice interleaving across multiple eligible chemistry schemas.
    MixedChemistry,
    /// Mixed practice interleaving across multiple eligible reasoning schemas.
    MixedReasoning,
    /// Global cross-domain interleaving (Maths + Physics + Chemistry + Reasoning).
    MixedInterleaved,
    /// Focused practice dedicated exclusively to a specific skill without forced interleaving.
    FocusedSkill { skill_id: SkillId },
    /// Focused practice dedicated exclusively to a specific reasoning skill.
    FocusedReasoningSkill { skill_id: SkillId },
    /// Focused practice on a single specific schema.
    FocusedSchema { schema_id: SchemaId },
    /// Strategic reasoning drill focusing on cognitive decision points and first strategy selection.
    StrategyDrill,
    /// Practice prioritized around weak skills / recent failures.
    WeakSkills,
    /// Speed and fluency practice with familiar topics and emphasized latency.
    SpeedPractice,
    /// Speed practice for reasoning topics under strict time threshold.
    SpeedReasoning,
    /// Foundational learning mode with lower difficulty (L1/L2) and foundational variants.
    Learning,
    /// Transfer practice presenting non-obvious, disguised, or combined cross-context problems.
    TransferPractice,
    /// Transfer reasoning challenge with novel structures and gated eligibility.
    TransferReasoning,
    /// Rapid diagnostic sweep across topics to baseline proficiency and isolate gaps.
    Diagnostic,
    /// Diagnostic sweep focusing on reasoning strategies and representations.
    DiagnosticReasoning,
    /// Progressive mastery builder scaffolding from simple variations to complex structures.
    SkillBuilder,
    /// Timed, high-stakes exam simulation mode with realistic pacing and mixed topics.
    ExamLike,
}

impl Default for PracticeMode {
    fn default() -> Self {
        PracticeMode::MixedMaths
    }
}

/// Evaluation of whether a learner is eligible for Level 5 Transfer practice on a skill.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransferEligibility {
    pub is_eligible: bool,
    pub consecutive_successes: u32,
    pub recent_accuracy: f64,
    pub distinct_variants_mastered: usize,
    pub has_recent_concept_failure: bool,
    pub latency_acceptable: bool,
    pub ineligible_reasons: Vec<String>,
}

/// Evaluator enforcing strict pedagogical gating rules for Transfer practice.
pub struct TransferEligibilityEngine;

impl TransferEligibilityEngine {
    /// Evaluate whether a skill state meets all criteria for Transfer Practice:
    /// 1. Repeated success on standard variants (>= 2 consecutive successes or >= 80% accuracy)
    /// 2. Acceptable execution latency (<= 1.25x target latency)
    /// 3. No recent concept or strategy breakdowns
    /// 4. Demonstration of performance across at least 1-2 distinct structural variants
    pub fn evaluate_eligibility(state: Option<&SkillState>) -> TransferEligibility {
        let mut reasons = Vec::new();

        let Some(s) = state else {
            reasons.push("Skill has not been practiced yet (cold start).".to_string());
            return TransferEligibility {
                is_eligible: false,
                consecutive_successes: 0,
                recent_accuracy: 0.0,
                distinct_variants_mastered: 0,
                has_recent_concept_failure: false,
                latency_acceptable: false,
                ineligible_reasons: reasons,
            };
        };

        let consecutive_successes = s.consecutive_successes;
        let recent_accuracy = s.recent_accuracy();

        // 1. Success rate requirement
        let success_met = consecutive_successes >= 2 || (s.recent_attempts.len() >= 3 && recent_accuracy >= 0.8);
        if !success_met {
            reasons.push(format!(
                "Requires >= 2 consecutive successes or >= 80% accuracy (current: {} successes, {:.0}% acc).",
                consecutive_successes, recent_accuracy * 100.0
            ));
        }

        // 2. Recent concept / strategy breakdown check (last 3 attempts)
        let mut has_concept_failure = false;
        for attempt in s.recent_attempts.iter().rev().take(3) {
            if !attempt.is_correct {
                if let Some(ref cat) = attempt.error_category {
                    if matches!(cat, ErrorCategory::Concept | ErrorCategory::Conceptual | ErrorCategory::Strategy) {
                        has_concept_failure = true;
                        reasons.push(format!("Recent conceptual breakdown detected ({:?}).", cat));
                        break;
                    }
                }
            }
        }

        // 3. Distinct variants mastered
        let mut variants_seen = std::collections::HashSet::new();
        for attempt in s.recent_attempts.iter().filter(|a| a.is_correct) {
            if let Some(ref v) = attempt.variant {
                variants_seen.insert(v.clone());
            }
        }
        let distinct_variants = variants_seen.len();
        if s.recent_attempts.len() >= 4 && distinct_variants < 1 {
            reasons.push("Must demonstrate mastery across standard structural variants first.".to_string());
        }

        // 4. Latency acceptable check
        let last_attempt = s.recent_attempts.last();
        let latency_acceptable = if let Some(last) = last_attempt {
            let target = if last.target_latency_ms > 0 { last.target_latency_ms } else { 35_000 };
            last.latency_ms <= (target as f64 * 1.35) as u64
        } else {
            true
        };

        if !latency_acceptable {
            reasons.push("Execution latency exceeds target thresholds; build procedural fluency first.".to_string());
        }

        let is_eligible = success_met && !has_concept_failure && latency_acceptable;

        TransferEligibility {
            is_eligible,
            consecutive_successes,
            recent_accuracy,
            distinct_variants_mastered: distinct_variants,
            has_recent_concept_failure: has_concept_failure,
            latency_acceptable,
            ineligible_reasons: reasons,
        }
    }
}

/// Structured decision from multi-schema session selection.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultiSchemaSelectionDecision {
    pub schema: SchemaPracticeObject,
    pub difficulty_level: u32,
    pub target_time_ms: u64,
    pub selected_variant: Option<String>,
    pub selection_reason: String,
    pub priority_score: f64,
}

/// Selector responsible for multi-schema prioritization, adaptive difficulty, and cross-schema interleaving.
pub struct MultiSchemaSelector;

impl MultiSchemaSelector {
    /// Select the next optimal schema to practice from candidate schemas based on mode,
    /// skill states, error diagnostics, and anti-priming policies.
    pub fn select_next_schema(
        mode: &PracticeMode,
        candidate_schemas: &[SchemaPracticeObject],
        skill_states: &HashMap<SkillId, SkillState>,
        last_schema_id: Option<&SchemaId>,
        _seed: u64,
    ) -> Option<MultiSchemaSelectionDecision> {
        if candidate_schemas.is_empty() {
            return None;
        }

        // 1. Filter candidates by practice mode
        let filtered_candidates: Vec<&SchemaPracticeObject> = match mode {
            PracticeMode::FocusedSkill { skill_id }
            | PracticeMode::FocusedReasoningSkill { skill_id } => candidate_schemas
                .iter()
                .filter(|s| &s.skill_id == skill_id)
                .collect(),
            PracticeMode::FocusedSchema { schema_id } => candidate_schemas
                .iter()
                .filter(|s| &s.id == schema_id)
                .collect(),
            _ => candidate_schemas.iter().collect(),
        };

        let pool = if filtered_candidates.is_empty() {
            candidate_schemas.iter().collect::<Vec<_>>()
        } else {
            filtered_candidates
        };

        if pool.len() == 1 {
            let schema = pool[0];
            let state = skill_states.get(&schema.skill_id);
            let (forced_level, latency_override) = Self::resolve_mode_overrides(mode, state);
            let (score, reason) = Self::compute_priority_score(schema, state, mode);
            let diff_decision = AdaptiveDifficultyEngine::evaluate_difficulty(state, forced_level, latency_override);
            return Some(MultiSchemaSelectionDecision {
                schema: schema.clone(),
                difficulty_level: diff_decision.level,
                target_time_ms: diff_decision.target_time_ms,
                selected_variant: None,
                selection_reason: reason,
                priority_score: score,
            });
        }

        // 2. Score each candidate based on priority heuristics
        let mut scored_candidates: Vec<(&SchemaPracticeObject, f64, String)> = pool
            .iter()
            .map(|&schema| {
                let state = skill_states.get(&schema.skill_id);
                let (score, reason) = Self::compute_priority_score(schema, state, mode);
                (schema, score, reason)
            })
            .collect();

        // 3. Apply Cross-Schema Anti-Priming Interleaving
        let apply_interleaving = matches!(
            mode,
            PracticeMode::MixedMaths
                | PracticeMode::MixedPhysics
                | PracticeMode::MixedChemistry
                | PracticeMode::MixedReasoning
                | PracticeMode::WeakSkills
                | PracticeMode::SpeedPractice
                | PracticeMode::SpeedReasoning
                | PracticeMode::ExamLike
                | PracticeMode::Diagnostic
                | PracticeMode::DiagnosticReasoning
                | PracticeMode::StrategyDrill
        );
        if apply_interleaving {
            if let Some(last_id) = last_schema_id {
                for (schema, score, _) in &mut scored_candidates {
                    if &schema.id == last_id {
                        // Penalty reduces score so alternative schemas are preferred unless critical remediation is needed
                        *score -= 300.0;
                    }
                }
            }
        }

        // Sort descending by priority score
        scored_candidates.sort_by(|a, b| {
            b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Top candidate is chosen
        let (chosen_schema, score, ref reason) = scored_candidates[0];
        let state = skill_states.get(&chosen_schema.skill_id);
        let (forced_level, latency_override) = Self::resolve_mode_overrides(mode, state);

        let diff_decision = AdaptiveDifficultyEngine::evaluate_difficulty(state, forced_level, latency_override);

        Some(MultiSchemaSelectionDecision {
            schema: (*chosen_schema).clone(),
            difficulty_level: diff_decision.level,
            target_time_ms: diff_decision.target_time_ms,
            selected_variant: None,
            selection_reason: reason.clone(),
            priority_score: score,
        })
    }

    /// Resolve forced difficulty level and latency override based on practice mode and state.
    pub fn resolve_mode_overrides(
        mode: &PracticeMode,
        state: Option<&SkillState>,
    ) -> (Option<u32>, Option<u64>) {
        let forced_level = match mode {
            PracticeMode::Learning => Some(1),
            PracticeMode::SpeedPractice | PracticeMode::SpeedReasoning => Some(1), // Speed practice on easier levels
            PracticeMode::TransferPractice | PracticeMode::TransferReasoning => {
                let elig = TransferEligibilityEngine::evaluate_eligibility(state);
                if elig.is_eligible {
                    Some(5) // Level 5 Transfer variant
                } else {
                    None // Fall back to adaptive scaffolding
                }
            }
            PracticeMode::StrategyDrill => Some(2), // Strategy drill baseline
            PracticeMode::ExamLike => Some(3),     // Real exam baseline difficulty
            _ => None,
        };

        let latency_override = match mode {
            PracticeMode::SpeedPractice => Some(20_000), // Speed drill 20s target
            PracticeMode::SpeedReasoning => Some(18_000), // Reasoning speed drill 18s target
            PracticeMode::StrategyDrill => Some(15_000),  // Strategy selection drill 15s target
            PracticeMode::ExamLike => Some(35_000),       // Strict exam pacing
            _ => None,
        };

        (forced_level, latency_override)
    }

    /// Compute priority score for a schema candidate based on learning state and mode.
    fn compute_priority_score(
        _schema: &SchemaPracticeObject,
        state: Option<&SkillState>,
        mode: &PracticeMode,
    ) -> (f64, String) {
        // Special practice modes handling
        match mode {
            PracticeMode::TransferPractice | PracticeMode::TransferReasoning => {
                let elig = TransferEligibilityEngine::evaluate_eligibility(state);
                if elig.is_eligible {
                    return (1500.0, "transfer_practice_eligible_mastery_met".to_string());
                } else {
                    return (300.0, format!("transfer_ineligible: {}", elig.ineligible_reasons.join(", ")));
                }
            }
            PracticeMode::Diagnostic | PracticeMode::DiagnosticReasoning => {
                let attempts_count = state.map_or(0, |s| s.total_attempts);
                let score = 1000.0 - (attempts_count as f64 * 100.0).min(800.0);
                return (score, "diagnostic_topic_coverage_sweep".to_string());
            }
            PracticeMode::StrategyDrill => {
                let s_opt = state;
                let score = if let Some(s) = s_opt {
                    if let Some(last) = s.recent_attempts.last() {
                        if !last.is_correct && last.error_category.as_ref().map_or(false, |c| matches!(c, ErrorCategory::Strategy | ErrorCategory::Concept)) {
                            1200.0
                        } else {
                            700.0
                        }
                    } else {
                        700.0
                    }
                } else {
                    600.0
                };
                return (score, "strategy_drill_cognitive_focus".to_string());
            }
            PracticeMode::SkillBuilder => {
                let s_opt = state;
                let score = if let Some(s) = s_opt {
                    500.0 + (s.consecutive_successes as f64 * 50.0)
                } else {
                    600.0
                };
                return (score, "skill_builder_progressive_scaffolding".to_string());
            }
            _ => {}
        }

        let Some(s) = state else {
            // Cold start: High priority to introduce new skills
            return (500.0, "new_unseen_skill".to_string());
        };

        let last_attempt = s.recent_attempts.last();
        let last_failed = last_attempt.map_or(false, |a| !a.is_correct);
        let recent_acc = s.recent_accuracy();

        // 1. CRITICAL REMEDIATION (Score: 1000+)
        // If the learner just failed or had a concept breakdown, remediate immediately
        if last_failed {
            if let Some(err_cat) = last_attempt.and_then(|a| a.error_category.as_ref()) {
                if matches!(err_cat, ErrorCategory::Concept | ErrorCategory::Conceptual | ErrorCategory::Strategy) {
                    return (1200.0, "critical_remediation_concept_breakdown".to_string());
                }
            }
            return (1000.0 + (s.consecutive_failures as f64 * 50.0), "remediation_recent_failure".to_string());
        }

        // 2. WEAK SKILL / LOW ACCURACY (< 50% recent accuracy or New/Learning state)
        if s.practice_state == PracticeProgressionState::Learning || (s.recent_attempts.len() >= 3 && recent_acc < 0.5) {
            let bonus = if matches!(mode, PracticeMode::WeakSkills) { 300.0 } else { 0.0 };
            return (800.0 - (recent_acc * 200.0) + bonus, "weak_skill_reinforcement".to_string());
        }

        // 3. FLUENCY REINFORCEMENT (Correct but slow)
        let last_latency = last_attempt.map_or(0, |a| a.latency_ms);
        let last_target = last_attempt.map_or(35_000, |a| a.target_latency_ms);
        let was_slow = last_latency > (last_target as f64 * 1.25) as u64;
        if was_slow {
            let bonus = if matches!(mode, PracticeMode::SpeedPractice) { 250.0 } else { 0.0 };
            return (650.0 + bonus, "fluency_reinforcement_slow_latency".to_string());
        }

        // 4. CONTROLLED ADVANCEMENT (Consecutive successes, high accuracy)
        if s.consecutive_successes >= 2 && recent_acc >= 0.8 {
            return (450.0, "controlled_difficulty_advancement".to_string());
        }

        // 5. STABLE NORMAL PRACTICE
        (400.0, "normal_rotation".to_string())
    }
}

/// Structured output from variant selection indicating the chosen variant and heuristic rationale.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectionDecision {
    pub variant: PercentageVariant,
    pub target_time_ms: u64,
    pub selection_reason: String,
}

impl SelectionDecision {
    pub fn new(variant: PercentageVariant, target_time_ms: u64, reason: impl Into<String>) -> Self {
        Self {
            variant,
            target_time_ms,
            selection_reason: reason.into(),
        }
    }
}

/// Selector responsible for variant-aware problem selection and anti-priming interleaving (backward compatible).
#[derive(Debug, Clone, Default)]
pub struct VariantSelector;

impl VariantSelector {
    const ALL_VARIANTS: &'static [PercentageVariant] = &[
        PercentageVariant::ForwardTwoStep,
        PercentageVariant::ReverseInitial,
        PercentageVariant::NetEquivalentChange,
        PercentageVariant::ForwardThreeStep,
    ];

    /// Default target latency per variant.
    pub fn default_target_latency_ms(variant: PercentageVariant) -> u64 {
        match variant {
            PercentageVariant::ForwardTwoStep => 35_000,
            PercentageVariant::ReverseInitial => 45_000,
            PercentageVariant::NetEquivalentChange => 40_000,
            PercentageVariant::ForwardThreeStep => 60_000,
        }
    }

    /// Select an optimal problem variant deterministically based on learner state, error diagnostics,
    /// and interleaving policies.
    pub fn select_variant(
        skill_state: Option<&SkillState>,
        allowed_variants: Option<&[PercentageVariant]>,
        seed: u64,
    ) -> SelectionDecision {
        let pool: Vec<PercentageVariant> = match allowed_variants {
            Some(list) if !list.is_empty() => list.to_vec(),
            _ => Self::ALL_VARIANTS.to_vec(),
        };

        if pool.len() == 1 {
            let variant = pool[0];
            return SelectionDecision::new(
                variant,
                Self::default_target_latency_ms(variant),
                "single_variant_configured",
            );
        }

        let mut rng = StdRng::seed_from_u64(seed);

        let state = match skill_state {
            Some(s) => s,
            None => {
                let variant = PercentageVariant::ForwardTwoStep;
                return SelectionDecision::new(
                    variant,
                    Self::default_target_latency_ms(variant),
                    "cold_start_standard_variant",
                );
            }
        };

        let last_attempt = state.recent_attempts.last();
        let last_variant_str = last_attempt.and_then(|a| a.variant.as_deref());
        let last_failed = last_attempt.map_or(false, |a| !a.is_correct);
        let recent_acc = state.recent_accuracy();

        // 1. REPEAT FAILED VARIANT REMEDIATION ACCORDING TO ERROR TAXONOMY
        if last_failed {
            if let Some(err_cat) = last_attempt.and_then(|a| a.error_category.as_ref()) {
                match err_cat {
                    ErrorCategory::Concept | ErrorCategory::Conceptual => {
                        // Concept breakdown -> drop to foundational forward variant
                        let variant = PercentageVariant::ForwardTwoStep;
                        if pool.contains(&variant) {
                            return SelectionDecision::new(
                                variant,
                                Self::default_target_latency_ms(variant),
                                "remediate_concept_error_standard_variant",
                            );
                        }
                    }
                    ErrorCategory::Sign => {
                        // Sign error -> target directional/sign-focused variant
                        let variant = PercentageVariant::NetEquivalentChange;
                        if pool.contains(&variant) {
                            return SelectionDecision::new(
                                variant,
                                Self::default_target_latency_ms(variant),
                                "remediate_sign_error_directional_variant",
                            );
                        }
                    }
                    ErrorCategory::Strategy => {
                        // Strategy error -> lower complexity standard variant
                        let variant = PercentageVariant::ForwardTwoStep;
                        if pool.contains(&variant) {
                            return SelectionDecision::new(
                                variant,
                                Self::default_target_latency_ms(variant),
                                "remediate_strategy_error_guided_variant",
                            );
                        }
                    }
                    _ => {}
                }
            }

            if let Some(last_var_name) = last_variant_str {
                let parsed_variant = match last_var_name {
                    "forward_two_step" => Some(PercentageVariant::ForwardTwoStep),
                    "reverse_initial" => Some(PercentageVariant::ReverseInitial),
                    "net_equivalent_change" => Some(PercentageVariant::NetEquivalentChange),
                    "forward_three_step" => Some(PercentageVariant::ForwardThreeStep),
                    _ => None,
                };

                if let Some(v) = parsed_variant {
                    if pool.contains(&v) {
                        return SelectionDecision::new(
                            v,
                            Self::default_target_latency_ms(v),
                            format!("remediate_failed_variant:{}", last_var_name),
                        );
                    }
                }
            }
        }

        // 2. FOUNDATIONAL / WEAK PERFORMANCE
        if state.practice_state == PracticeProgressionState::New
            || state.practice_state == PracticeProgressionState::Learning
            || (state.recent_attempts.len() >= 3 && recent_acc < 0.5)
        {
            let variant = PercentageVariant::ForwardTwoStep;
            if pool.contains(&variant) {
                return SelectionDecision::new(
                    variant,
                    Self::default_target_latency_ms(variant),
                    "baseline_proficiency_building",
                );
            }
        }

        // 3. CORRECT BUT SLOW
        let last_latency = last_attempt.map_or(0, |a| a.latency_ms);
        let last_target = last_attempt.map_or(35_000, |a| a.target_latency_ms);
        let was_slow = last_latency > (last_target as f64 * 1.25) as u64;

        if was_slow && !last_failed {
            let preferred = if pool.contains(&PercentageVariant::ForwardTwoStep) {
                PercentageVariant::ForwardTwoStep
            } else if pool.contains(&PercentageVariant::NetEquivalentChange) {
                PercentageVariant::NetEquivalentChange
            } else {
                pool[0]
            };

            return SelectionDecision::new(
                preferred,
                Self::default_target_latency_ms(preferred),
                "fluency_reinforcement_slow_latency",
            );
        }

        // 4. STRONG AND FAST PERFORMANCE
        let was_fast = last_latency <= (last_target as f64 * 0.75) as u64;
        let strong_performance = state.consecutive_successes >= 2 || (recent_acc >= 0.8 && was_fast);

        if strong_performance {
            let advanced_candidates: Vec<PercentageVariant> = pool
                .iter()
                .copied()
                .filter(|v| match v {
                    PercentageVariant::ReverseInitial
                    | PercentageVariant::NetEquivalentChange
                    | PercentageVariant::ForwardThreeStep => true,
                    _ => false,
                })
                .collect();

            if !advanced_candidates.is_empty() {
                let filtered = Self::apply_anti_priming(&advanced_candidates, last_variant_str);
                let choice_idx = rng.random_range(0..filtered.len());
                let chosen = filtered[choice_idx];
                return SelectionDecision::new(
                    chosen,
                    Self::default_target_latency_ms(chosen),
                    "introduce_structural_variation",
                );
            }
        }

        // 5. STABLE PERFORMANCE
        let eligible = Self::apply_anti_priming(&pool, last_variant_str);
        let choice_idx = rng.random_range(0..eligible.len());
        let chosen = eligible[choice_idx];

        SelectionDecision::new(
            chosen,
            Self::default_target_latency_ms(chosen),
            "anti_priming_variant_rotation",
        )
    }

    fn apply_anti_priming(
        candidates: &[PercentageVariant],
        last_variant_str: Option<&str>,
    ) -> Vec<PercentageVariant> {
        if candidates.len() <= 1 || last_variant_str.is_none() {
            return candidates.to_vec();
        }

        let last_str = last_variant_str.unwrap();
        let non_repeated: Vec<PercentageVariant> = candidates
            .iter()
            .copied()
            .filter(|v| match v {
                PercentageVariant::ForwardTwoStep => last_str != "forward_two_step",
                PercentageVariant::ReverseInitial => last_str != "reverse_initial",
                PercentageVariant::NetEquivalentChange => last_str != "net_equivalent_change",
                PercentageVariant::ForwardThreeStep => last_str != "forward_three_step",
            })
            .collect();

        if non_repeated.is_empty() {
            candidates.to_vec()
        } else {
            non_repeated
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::problems::catalog::MathsCatalog;

    #[test]
    fn test_multi_schema_selector_remediation_priority() {
        let sch_pct = MathsCatalog::successive_percentage_schema();
        let sch_ratio = MathsCatalog::ratio_schema();
        let candidates = vec![sch_pct.clone(), sch_ratio.clone()];

        let mut states = HashMap::new();
        let mut pct_state = SkillState::new(&sch_pct.skill_id);
        // Record a failure on Percentage
        pct_state.record_attempt_outcome(
            false,
            0.0,
            25000,
            35000,
            Some("standard"),
            Some(&ErrorCategory::Concept),
            1000,
        );
        states.insert(sch_pct.skill_id.clone(), pct_state);

        let decision = MultiSchemaSelector::select_next_schema(
            &PracticeMode::MixedMaths,
            &candidates,
            &states,
            None,
            42,
        )
        .unwrap();

        assert_eq!(decision.schema.id, sch_pct.id);
        assert!(decision.selection_reason.contains("critical_remediation"));
    }

    #[test]
    fn test_multi_schema_cross_schema_anti_priming() {
        let sch_pct = MathsCatalog::successive_percentage_schema();
        let sch_ratio = MathsCatalog::ratio_schema();
        let sch_linear = MathsCatalog::linear_equations_schema();
        let candidates = vec![sch_pct.clone(), sch_ratio.clone(), sch_linear.clone()];

        let states = HashMap::new(); // All cold start equal priority

        // If last practiced was Percentage, selector in MixedMaths should select a DIFFERENT schema
        let decision = MultiSchemaSelector::select_next_schema(
            &PracticeMode::MixedMaths,
            &candidates,
            &states,
            Some(&sch_pct.id),
            12345,
        )
        .unwrap();

        assert_ne!(decision.schema.id, sch_pct.id);
    }

    #[test]
    fn test_focused_mode_overrides_interleaving() {
        let sch_pct = MathsCatalog::successive_percentage_schema();
        let sch_ratio = MathsCatalog::ratio_schema();
        let candidates = vec![sch_pct.clone(), sch_ratio.clone()];
        let states = HashMap::new();

        let mode = PracticeMode::FocusedSkill {
            skill_id: sch_pct.skill_id.clone(),
        };
        let decision = MultiSchemaSelector::select_next_schema(
            &mode,
            &candidates,
            &states,
            Some(&sch_pct.id),
            12345,
        )
        .unwrap();

        assert_eq!(decision.schema.id, sch_pct.id);
    }

    #[test]
    fn test_transfer_eligibility_gating() {
        let skill_id = SkillId::new("math.arithmetic.mixtures_alligation");
        let mut state = SkillState::new(&skill_id);

        // 1. Cold start -> not eligible
        let elig_cold = TransferEligibilityEngine::evaluate_eligibility(Some(&state));
        assert!(!elig_cold.is_eligible);

        // 2. Add 2 fast, correct attempts on standard variants -> eligible!
        state.record_attempt_outcome(true, 1.0, 15000, 35000, Some("alligation_ratio"), None, 1000);
        state.record_attempt_outcome(true, 1.0, 18000, 35000, Some("two_component_blend"), None, 2000);

        let elig_ready = TransferEligibilityEngine::evaluate_eligibility(Some(&state));
        assert!(elig_ready.is_eligible, "Should be eligible with 2 consecutive fast successes");
        assert_eq!(elig_ready.consecutive_successes, 2);

        // 3. Add a concept error -> immediately ineligible
        state.record_attempt_outcome(
            false,
            0.0,
            25000,
            35000,
            Some("two_component_blend"),
            Some(&ErrorCategory::Concept),
            3000,
        );

        let elig_failed = TransferEligibilityEngine::evaluate_eligibility(Some(&state));
        assert!(!elig_failed.is_eligible, "Concept failure should disqualify transfer mode");
        assert!(elig_failed.has_recent_concept_failure);
    }

    #[test]
    fn test_transfer_practice_selection_difficulty_level_5() {
        let sch_tsd = MathsCatalog::time_speed_distance_schema();
        let candidates = vec![sch_tsd.clone()];

        let mut states = HashMap::new();
        let mut tsd_state = SkillState::new(&sch_tsd.skill_id);
        tsd_state.record_attempt_outcome(true, 1.0, 15000, 40000, Some("direct_formula"), None, 1000);
        tsd_state.record_attempt_outcome(true, 1.0, 18000, 40000, Some("average_speed"), None, 2000);
        states.insert(sch_tsd.skill_id.clone(), tsd_state);

        let decision = MultiSchemaSelector::select_next_schema(
            &PracticeMode::TransferPractice,
            &candidates,
            &states,
            None,
            42,
        )
        .unwrap();

        assert_eq!(decision.difficulty_level, 5, "Transfer mode must force difficulty level 5");
        assert!(decision.selection_reason.contains("transfer_practice_eligible"));
    }
}
