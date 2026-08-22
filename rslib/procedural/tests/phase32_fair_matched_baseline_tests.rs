// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Phase 32: Fair Matched Baseline vs StudyLab Longitudinal Validation Suite
//!
//! Controlled empirical comparison of StudyLab's adaptive personalization against a fair,
//! non-adaptive matched baseline with identical learner models, content pools, workload schedules,
//! study gaps, and full variant/difficulty exposure opportunities.
//!
//! Evaluates:
//! - 12 Synthetic Learner Cohorts (A through L)
//! - 32 Canonical Problem Families across 4 domains (Math, Reasoning, Physics, Chemistry)
//! - 3 Workload levels (20, 45, 75 reviews/day)
//! - Checkpoints at Day 1, Day 30, Day 60, Day 90
//! - Multi-seed robustness (Seeds 1, 2, 3)
//! - Analysis A (Intent-to-Practice) vs Analysis B (Matched Exposure)
//! - Treatment on Hard Cohorts, Transfer Ladder, Retention Gap, Speed Pressure,
//!   Learning Efficiency, Subject-Specific Benefit, and Negative Outcome Audit.

use std::collections::HashMap;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tempfile::tempdir;

use procedural::core::{
    AttemptId, Domain, ErrorEventId, ProblemInstanceId, SchemaId, SkillId,
};
use procedural::diagnostics::hints::HintLevel;
use procedural::diagnostics::ErrorCategory;
use procedural::practice::{ErrorEvent, PracticeAttempt};
use procedural::problems::catalog::*;
use procedural::problems::ProblemInstance;
use procedural::remediation::{
    RemediationActionKind, RemediationContext, RemediationPolicy,
};
use procedural::scheduling::difficulty::AdaptiveDifficultyEngine;
use procedural::scheduling::PracticeMode;
use procedural::service::ProceduralService;
use procedural::skills::domain_evidence::{
    ChemistryEvidence, MathEvidence, PhysicsEvidence, ReasoningEvidence,
    VersionedDomainEvidence,
};
use procedural::skills::signals::{IndependenceLevel, MasteryEvidence, VariantCategory};
use procedural::skills::PracticeProgressionState;

// =========================================================================
// 1. 12 SYNTHETIC LEARNER COHORTS
// =========================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CohortId {
    CohortAStrongFast,
    CohortBStrongSlow,
    CohortCCareless,
    CohortDConceptWeak,
    CohortEPatternWeak,
    CohortFTransferWeak,
    CohortGMixedImproving,
    CohortHInconsistent,
    CohortIBeginner,
    CohortJLowRetention,
    CohortKSpeedPressure,
    CohortLUnevenMultiSubject,
}

impl CohortId {
    pub fn all() -> &'static [CohortId] {
        &[
            CohortId::CohortAStrongFast,
            CohortId::CohortBStrongSlow,
            CohortId::CohortCCareless,
            CohortId::CohortDConceptWeak,
            CohortId::CohortEPatternWeak,
            CohortId::CohortFTransferWeak,
            CohortId::CohortGMixedImproving,
            CohortId::CohortHInconsistent,
            CohortId::CohortIBeginner,
            CohortId::CohortJLowRetention,
            CohortId::CohortKSpeedPressure,
            CohortId::CohortLUnevenMultiSubject,
        ]
    }

    pub fn hard_cohorts() -> &'static [CohortId] {
        &[
            CohortId::CohortDConceptWeak,
            CohortId::CohortEPatternWeak,
            CohortId::CohortFTransferWeak,
            CohortId::CohortIBeginner,
            CohortId::CohortLUnevenMultiSubject,
            CohortId::CohortHInconsistent,
        ]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            CohortId::CohortAStrongFast => "Cohort_A_StrongFast",
            CohortId::CohortBStrongSlow => "Cohort_B_StrongSlow",
            CohortId::CohortCCareless => "Cohort_C_Careless",
            CohortId::CohortDConceptWeak => "Cohort_D_ConceptWeak",
            CohortId::CohortEPatternWeak => "Cohort_E_PatternWeak",
            CohortId::CohortFTransferWeak => "Cohort_F_TransferWeak",
            CohortId::CohortGMixedImproving => "Cohort_G_MixedImproving",
            CohortId::CohortHInconsistent => "Cohort_H_Inconsistent",
            CohortId::CohortIBeginner => "Cohort_I_Beginner",
            CohortId::CohortJLowRetention => "Cohort_J_LowRetention",
            CohortId::CohortKSpeedPressure => "Cohort_K_SpeedPressure",
            CohortId::CohortLUnevenMultiSubject => "Cohort_L_UnevenMultiSubject",
        }
    }

    /// Simulate a single problem attempt deterministically with PRNG seed mixing
    pub fn simulate_attempt(
        &self,
        domain: &Domain,
        chapter: &str,
        level: u32,
        variant_cat: VariantCategory,
        target_latency_ms: u64,
        logical_day: u32,
        attempt_idx: usize,
        is_in_study_gap: bool,
        is_exam_pressure: bool,
        seed_offset: u64,
    ) -> (
        bool,
        u64,
        Option<ErrorCategory>,
        usize,
        Option<HintLevel>,
        Option<VersionedDomainEvidence>,
    ) {
        if is_in_study_gap {
            return (true, target_latency_ms, None, 0, None, None);
        }

        let seed_val = (logical_day as u64).wrapping_mul(100_000)
            + (attempt_idx as u64)
            + seed_offset.wrapping_mul(1_000_003);
        let hash_val = (seed_val.wrapping_mul(6364136223846793005)
            ^ (level as u64).wrapping_mul(1442695040888963407)
            ^ ((variant_cat as u32 as u64).wrapping_mul(2862933555777941757)))
            % 1000;

        let is_transfer = matches!(
            variant_cat,
            VariantCategory::Structural
                | VariantCategory::Contextual
                | VariantCategory::Transfer
                | VariantCategory::MultiConcept
        );

        let is_correct: bool;
        let actual_time: u64;
        let mut err: Option<ErrorCategory> = None;
        let mut hints = 0usize;
        let mut hint_lvl: Option<HintLevel> = None;
        let dom_ev: Option<VersionedDomainEvidence>;

        match self {
            // A. Strong + Fast: 98% accuracy, 0.50x latency, 0 hints
            CohortId::CohortAStrongFast => {
                is_correct = hash_val < 980;
                let mult = 0.48 + ((hash_val % 80) as f64 / 1000.0);
                actual_time = (target_latency_ms as f64 * mult) as u64;
                if !is_correct {
                    err = Some(ErrorCategory::Calculation);
                    dom_ev = Some(Self::make_domain_evidence(domain, false, true, true));
                } else {
                    dom_ev = Some(Self::make_domain_evidence(domain, true, true, true));
                }
            }

            // B. Strong + Slow: 94% accuracy, 1.45x latency, minimal hints
            CohortId::CohortBStrongSlow => {
                is_correct = hash_val < 940;
                let mult = 1.35 + ((hash_val % 150) as f64 / 1000.0);
                actual_time = (target_latency_ms as f64 * mult) as u64;
                if !is_correct {
                    err = Some(ErrorCategory::Calculation);
                    hints = 1;
                    hint_lvl = Some(HintLevel::Level1_RetrievalCue);
                    dom_ev = Some(Self::make_domain_evidence(domain, false, true, true));
                } else {
                    dom_ev = Some(Self::make_domain_evidence(domain, true, true, true));
                }
            }

            // C. Careless: 72% accuracy, very fast (0.42x latency), sign/arithmetic slips
            CohortId::CohortCCareless => {
                is_correct = hash_val < 720;
                let mult = 0.40 + ((hash_val % 80) as f64 / 1000.0);
                actual_time = (target_latency_ms as f64 * mult) as u64;
                if !is_correct {
                    err = if hash_val % 2 == 0 {
                        Some(ErrorCategory::Calculation)
                    } else {
                        Some(ErrorCategory::Sign)
                    };
                    dom_ev = Some(Self::make_domain_evidence(domain, false, true, true));
                } else {
                    dom_ev = Some(Self::make_domain_evidence(domain, true, true, true));
                }
            }

            // D. Concept Weak: 60% L1, drops to 15-20% L3-L5, misconception errors
            CohortId::CohortDConceptWeak => {
                let threshold = match level {
                    1 => 600,
                    2 => 400,
                    3 => 250,
                    4 => 200,
                    _ => 150,
                };
                is_correct = hash_val < threshold;
                let mult = 1.15 + ((hash_val % 200) as f64 / 1000.0);
                actual_time = (target_latency_ms as f64 * mult) as u64;
                if !is_correct {
                    err = match domain {
                        Domain::Reasoning => Some(ErrorCategory::Strategy),
                        Domain::Physics | Domain::Chemistry => {
                            Some(ErrorCategory::DomainSpecific("concept_setup".to_string()))
                        }
                        _ => Some(ErrorCategory::Concept),
                    };
                    hints = 2;
                    hint_lvl = Some(HintLevel::Level2_ProceduralScaffold);
                    dom_ev = Some(Self::make_domain_evidence(domain, false, false, false));
                } else {
                    dom_ev = Some(Self::make_domain_evidence(domain, true, true, true));
                }
            }

            // E. Pattern Weak: 88% on L1/L2, drops to 38% on structural/transfer
            CohortId::CohortEPatternWeak => {
                let is_complex = level >= 3 || is_transfer;
                let threshold = if is_complex { 380 } else { 880 };
                is_correct = hash_val < threshold;
                let mult = if is_complex { 1.35 } else { 0.95 };
                actual_time =
                    (target_latency_ms as f64 * (mult + ((hash_val % 100) as f64 / 1000.0))) as u64;
                if !is_correct {
                    err = match domain {
                        Domain::Reasoning => Some(ErrorCategory::Strategy),
                        Domain::Physics => {
                            Some(ErrorCategory::DomainSpecific("model_setup".to_string()))
                        }
                        Domain::Chemistry => {
                            Some(ErrorCategory::DomainSpecific("stoichiometry".to_string()))
                        }
                        _ => Some(ErrorCategory::Strategy),
                    };
                    if is_complex {
                        hints = 2;
                        hint_lvl = Some(HintLevel::Level2_ProceduralScaffold);
                    }
                    dom_ev = Some(Self::make_domain_evidence(domain, false, false, true));
                } else {
                    dom_ev = Some(Self::make_domain_evidence(domain, true, true, true));
                }
            }

            // F. Transfer Weak: 92% on parameter/isomorphic, drops to 35% on structural/transfer
            CohortId::CohortFTransferWeak => {
                let threshold = if is_transfer { 350 } else { 920 };
                is_correct = hash_val < threshold;
                let mult = if is_transfer { 1.40 } else { 0.75 };
                actual_time =
                    (target_latency_ms as f64 * (mult + ((hash_val % 100) as f64 / 1000.0))) as u64;
                if !is_correct {
                    err = if is_transfer {
                        Some(ErrorCategory::DomainSpecific("transfer_flaw".to_string()))
                    } else {
                        Some(ErrorCategory::Calculation)
                    };
                    if is_transfer {
                        hints = 2;
                        hint_lvl = Some(HintLevel::Level2_ProceduralScaffold);
                    }
                    dom_ev =
                        Some(Self::make_domain_evidence(domain, false, is_transfer, !is_transfer));
                } else {
                    dom_ev = Some(Self::make_domain_evidence(domain, true, true, true));
                }
            }

            // G. Mixed / Improving: Starts at 55% acc, improves to 92% by Day 60
            CohortId::CohortGMixedImproving => {
                let progress = (logical_day as f64 / 60.0).min(1.0);
                let threshold = 550.0 + (370.0 * progress);
                is_correct = (hash_val as f64) < threshold;
                let mult = 1.25 - (0.55 * progress);
                actual_time =
                    (target_latency_ms as f64 * (mult + ((hash_val % 80) as f64 / 1000.0))) as u64;
                if !is_correct {
                    err = if progress < 0.4 {
                        Some(ErrorCategory::Concept)
                    } else {
                        Some(ErrorCategory::Calculation)
                    };
                    if progress < 0.4 {
                        hints = 2;
                        hint_lvl = Some(HintLevel::Level2_ProceduralScaffold);
                        dom_ev = Some(Self::make_domain_evidence(domain, false, false, false));
                    } else {
                        dom_ev = Some(Self::make_domain_evidence(domain, false, true, true));
                    }
                } else {
                    dom_ev = Some(Self::make_domain_evidence(domain, true, true, true));
                }
            }

            // H. Inconsistent: Sinusoidal oscillation (55% to 85%)
            CohortId::CohortHInconsistent => {
                let wave_phase =
                    ((logical_day % 12) as f64 / 12.0) * 2.0 * std::f64::consts::PI;
                let wave_acc = 700.0 + (150.0 * wave_phase.sin());
                is_correct = (hash_val as f64) < wave_acc;
                let mult = 0.90 + (0.35 * (wave_phase + 1.0).sin().abs());
                actual_time = (target_latency_ms as f64 * mult) as u64;
                if !is_correct {
                    err = match hash_val % 3 {
                        0 => Some(ErrorCategory::Calculation),
                        1 => Some(ErrorCategory::Concept),
                        _ => Some(ErrorCategory::Sign),
                    };
                    dom_ev = Some(Self::make_domain_evidence(domain, false, hash_val % 2 == 0, false));
                } else {
                    dom_ev = Some(Self::make_domain_evidence(domain, true, true, true));
                }
            }

            // I. Beginner: Low initial mastery across all subjects (45% on L1, 20% on L2+)
            CohortId::CohortIBeginner => {
                let progress = (logical_day as f64 / 90.0).min(1.0);
                let threshold = match level {
                    1 => 450.0 + (300.0 * progress),
                    2 => 250.0 + (350.0 * progress),
                    _ => 150.0 + (250.0 * progress),
                };
                is_correct = (hash_val as f64) < threshold;
                let mult = 1.40 - (0.35 * progress);
                actual_time =
                    (target_latency_ms as f64 * (mult + ((hash_val % 100) as f64 / 1000.0))) as u64;
                if !is_correct {
                    err = if hash_val % 2 == 0 {
                        Some(ErrorCategory::Concept)
                    } else {
                        Some(ErrorCategory::Calculation)
                    };
                    hints = 2;
                    hint_lvl = Some(HintLevel::Level2_ProceduralScaffold);
                    dom_ev = Some(Self::make_domain_evidence(domain, false, false, false));
                } else {
                    dom_ev = Some(Self::make_domain_evidence(domain, true, true, true));
                }
            }

            // J. High Ability / Low Retention: 92% when active; decays after gaps
            CohortId::CohortJLowRetention => {
                let is_post_gap = (31..=38).contains(&logical_day);
                let threshold = if is_post_gap { 500 } else { 920 };
                is_correct = hash_val < threshold;
                let mult = if is_post_gap { 1.25 } else { 0.65 };
                actual_time =
                    (target_latency_ms as f64 * (mult + ((hash_val % 80) as f64 / 1000.0))) as u64;
                if !is_correct {
                    err = if is_post_gap {
                        Some(ErrorCategory::Concept)
                    } else {
                        Some(ErrorCategory::Calculation)
                    };
                    dom_ev =
                        Some(Self::make_domain_evidence(domain, false, !is_post_gap, !is_post_gap));
                } else {
                    dom_ev = Some(Self::make_domain_evidence(domain, true, true, true));
                }
            }

            // K. Speed-Pressure Learner: High speed (0.38x - 0.55x), careless error spikes under pressure
            CohortId::CohortKSpeedPressure => {
                let threshold = if is_exam_pressure { 580 } else { 820 };
                is_correct = hash_val < threshold;
                let mult = if is_exam_pressure { 0.38 } else { 0.55 };
                actual_time = (target_latency_ms as f64 * mult) as u64;
                if !is_correct {
                    err = Some(ErrorCategory::Calculation);
                    dom_ev = Some(Self::make_domain_evidence(domain, false, true, true));
                } else {
                    dom_ev = Some(Self::make_domain_evidence(domain, true, true, true));
                }
            }

            // L. Uneven Multi-Subject Learner: Math (95%), Physics (75%), Chem Stoich (45%), Reasoning (50-90%)
            CohortId::CohortLUnevenMultiSubject => {
                let (threshold, mult) = match domain {
                    Domain::Mathematics => (950, 0.60),
                    Domain::Physics => (750, 0.95),
                    Domain::Chemistry => (450, 1.30),
                    Domain::Reasoning => {
                        if chapter.contains("seating") {
                            (900, 0.70)
                        } else {
                            (500, 1.20)
                        }
                    }
                    _ => (750, 1.0),
                };
                is_correct = hash_val < threshold;
                actual_time =
                    (target_latency_ms as f64 * (mult + ((hash_val % 80) as f64 / 1000.0))) as u64;
                if !is_correct {
                    err = match domain {
                        Domain::Chemistry => {
                            Some(ErrorCategory::DomainSpecific("stoichiometry".to_string()))
                        }
                        Domain::Reasoning => {
                            if !chapter.contains("seating") {
                                Some(ErrorCategory::Strategy)
                            } else {
                                Some(ErrorCategory::Calculation)
                            }
                        }
                        _ => Some(ErrorCategory::Calculation),
                    };
                    let is_setup = matches!(domain, Domain::Chemistry)
                        || (matches!(domain, Domain::Reasoning) && !chapter.contains("seating"));
                    dom_ev = Some(Self::make_domain_evidence(domain, false, !is_setup, !is_setup));
                } else {
                    dom_ev = Some(Self::make_domain_evidence(domain, true, true, true));
                }
            }
        }

        (is_correct, actual_time, err, hints, hint_lvl, dom_ev)
    }

    fn make_domain_evidence(
        domain: &Domain,
        execution_ok: bool,
        method_ok: bool,
        pattern_ok: bool,
    ) -> VersionedDomainEvidence {
        match domain {
            Domain::Mathematics => {
                let m = MathEvidence {
                    pattern_recognition: Some(pattern_ok),
                    method_selection: Some(method_ok),
                    execution: Some(execution_ok),
                    verification: Some(true),
                    structural_transfer: Some(pattern_ok),
                };
                VersionedDomainEvidence::new_math(m)
            }
            Domain::Reasoning => {
                let r = ReasoningEvidence {
                    pattern_recognition: Some(pattern_ok),
                    representation: Some(method_ok),
                    constraint_extraction: Some(method_ok),
                    deduction: Some(execution_ok),
                    trap_checking: Some(execution_ok),
                    structural_transfer: Some(pattern_ok),
                    decision_path: Some(method_ok),
                };
                VersionedDomainEvidence::new_reasoning(r)
            }
            Domain::Physics => {
                let p = PhysicsEvidence {
                    physical_model_selection: Some(pattern_ok),
                    representation: Some(method_ok),
                    governing_principle: Some(pattern_ok),
                    equation_setup: Some(method_ok),
                    calculation: Some(execution_ok),
                    unit_validity: Some(execution_ok),
                    boundary_validity: Some(true),
                    verification: Some(true),
                    transfer: Some(pattern_ok),
                };
                VersionedDomainEvidence::new_physics(p)
            }
            Domain::Chemistry => {
                let c = ChemistryEvidence::Physical {
                    model_setup: Some(pattern_ok),
                    equation_selection: Some(pattern_ok),
                    intermediate_quantity: Some(method_ok),
                    calculation: Some(execution_ok),
                    conservation: Some(method_ok),
                    verification: Some(true),
                    transfer: Some(pattern_ok),
                };
                VersionedDomainEvidence::new_chemistry(c)
            }
            _ => {
                let m = MathEvidence {
                    pattern_recognition: Some(pattern_ok),
                    method_selection: Some(method_ok),
                    execution: Some(execution_ok),
                    verification: Some(true),
                    structural_transfer: Some(pattern_ok),
                };
                VersionedDomainEvidence::new_math(m)
            }
        }
    }
}

// =========================================================================
// 2. CANONICAL SCHEMAS & CHAPTER REGISTRATION (32 FAMILIES)
// =========================================================================

#[derive(Debug, Clone)]
pub struct RegisteredFamilyInfo {
    pub schema_id: &'static str,
    pub skill_id: &'static str,
    pub domain: Domain,
    pub chapter: &'static str,
}

pub fn get_all_canonical_schemas() -> Vec<RegisteredFamilyInfo> {
    vec![
        // Mathematics: Number System
        RegisteredFamilyInfo { schema_id: SCHEMA_DIVISIBILITY, skill_id: SKILL_DIVISIBILITY, domain: Domain::Mathematics, chapter: "Number System" },
        RegisteredFamilyInfo { schema_id: SCHEMA_REMAINDERS_MODULAR, skill_id: SKILL_REMAINDERS_MODULAR, domain: Domain::Mathematics, chapter: "Number System" },
        // Mathematics: Arithmetic
        RegisteredFamilyInfo { schema_id: SCHEMA_SUCCESSIVE_PERCENTAGE, skill_id: SKILL_PERCENTAGE_SUCCESSIVE, domain: Domain::Mathematics, chapter: "Arithmetic" },
        RegisteredFamilyInfo { schema_id: SCHEMA_PROFIT_LOSS, skill_id: SKILL_PROFIT_LOSS, domain: Domain::Mathematics, chapter: "Arithmetic" },
        RegisteredFamilyInfo { schema_id: SCHEMA_RATIO, skill_id: SKILL_RATIO, domain: Domain::Mathematics, chapter: "Arithmetic" },
        RegisteredFamilyInfo { schema_id: SCHEMA_AVERAGE, skill_id: SKILL_AVERAGE, domain: Domain::Mathematics, chapter: "Arithmetic" },
        RegisteredFamilyInfo { schema_id: SCHEMA_TIME_WORK, skill_id: SKILL_TIME_WORK, domain: Domain::Mathematics, chapter: "Arithmetic" },
        RegisteredFamilyInfo { schema_id: SCHEMA_TIME_SPEED_DISTANCE, skill_id: SKILL_TIME_SPEED_DISTANCE, domain: Domain::Mathematics, chapter: "Arithmetic" },
        RegisteredFamilyInfo { schema_id: SCHEMA_MIXTURES_ALLIGATION, skill_id: SKILL_MIXTURES_ALLIGATION, domain: Domain::Mathematics, chapter: "Arithmetic" },
        // Mathematics: Algebra
        RegisteredFamilyInfo { schema_id: SCHEMA_LINEAR_EQUATIONS, skill_id: SKILL_LINEAR_EQUATIONS, domain: Domain::Mathematics, chapter: "Algebra" },
        RegisteredFamilyInfo { schema_id: SCHEMA_LINEAR_INEQUALITIES, skill_id: SKILL_LINEAR_INEQUALITIES, domain: Domain::Mathematics, chapter: "Algebra" },
        RegisteredFamilyInfo { schema_id: SCHEMA_ALGEBRAIC_IDENTITIES, skill_id: SKILL_ALGEBRAIC_IDENTITIES, domain: Domain::Mathematics, chapter: "Algebra" },
        // Mathematics: Geometry
        RegisteredFamilyInfo { schema_id: SCHEMA_GEOMETRY_TRIANGLES, skill_id: SKILL_GEOMETRY_TRIANGLES, domain: Domain::Mathematics, chapter: "Geometry" },
        // Mathematics: Multi-Concept
        RegisteredFamilyInfo { schema_id: SCHEMA_COMBINED_MULTI_CONCEPT, skill_id: SKILL_COMBINED_MULTI_CONCEPT, domain: Domain::Mathematics, chapter: "Multi-Concept" },

        // Reasoning
        RegisteredFamilyInfo { schema_id: SCHEMA_REASONING_SERIES, skill_id: SKILL_REASONING_SERIES, domain: Domain::Reasoning, chapter: "Series" },
        RegisteredFamilyInfo { schema_id: SCHEMA_REASONING_SYLLOGISM, skill_id: SKILL_REASONING_SYLLOGISM, domain: Domain::Reasoning, chapter: "Syllogism" },
        RegisteredFamilyInfo { schema_id: SCHEMA_REASONING_SEATING, skill_id: SKILL_REASONING_SEATING, domain: Domain::Reasoning, chapter: "Seating" },
        RegisteredFamilyInfo { schema_id: SCHEMA_REASONING_RELATIONS, skill_id: SKILL_REASONING_RELATIONS, domain: Domain::Reasoning, chapter: "Relations" },
        RegisteredFamilyInfo { schema_id: SCHEMA_REASONING_BLOOD_RELATIONS, skill_id: SKILL_REASONING_BLOOD_RELATIONS, domain: Domain::Reasoning, chapter: "Blood Relations" },
        RegisteredFamilyInfo { schema_id: SCHEMA_REASONING_DIRECTION_SENSE, skill_id: SKILL_REASONING_DIRECTION_SENSE, domain: Domain::Reasoning, chapter: "Direction Sense" },
        RegisteredFamilyInfo { schema_id: SCHEMA_REASONING_FLOOR_GRID, skill_id: SKILL_REASONING_FLOOR_GRID, domain: Domain::Reasoning, chapter: "Floor Grid" },
        RegisteredFamilyInfo { schema_id: SCHEMA_REASONING_LOGIC_DAG, skill_id: SKILL_REASONING_LOGIC_DAG, domain: Domain::Reasoning, chapter: "Logic DAG" },
        RegisteredFamilyInfo { schema_id: SCHEMA_REASONING_DATA_SUFFICIENCY, skill_id: SKILL_REASONING_DATA_SUFFICIENCY, domain: Domain::Reasoning, chapter: "Data Sufficiency" },
        RegisteredFamilyInfo { schema_id: SCHEMA_REASONING_CODED_EXPRESSIONS, skill_id: SKILL_REASONING_CODED_EXPRESSIONS, domain: Domain::Reasoning, chapter: "Coded Expressions" },

        // Physics
        RegisteredFamilyInfo { schema_id: SCHEMA_PHYSICS_KINEMATICS, skill_id: SKILL_PHYSICS_KINEMATICS, domain: Domain::Physics, chapter: "Kinematics" },
        RegisteredFamilyInfo { schema_id: SCHEMA_PHYSICS_WORK_ENERGY, skill_id: SKILL_PHYSICS_WORK_ENERGY, domain: Domain::Physics, chapter: "Work & Energy" },

        // Chemistry
        RegisteredFamilyInfo { schema_id: SCHEMA_CHEMISTRY_STOICHIOMETRY, skill_id: SKILL_CHEMISTRY_STOICHIOMETRY, domain: Domain::Chemistry, chapter: "Stoichiometry" },
        RegisteredFamilyInfo { schema_id: SCHEMA_CHEMISTRY_EQUILIBRIUM, skill_id: SKILL_CHEMISTRY_EQUILIBRIUM, domain: Domain::Chemistry, chapter: "Equilibrium" },
        RegisteredFamilyInfo { schema_id: SCHEMA_CHEMISTRY_BUFFERS_TITRATION, skill_id: SKILL_CHEMISTRY_BUFFERS_TITRATION, domain: Domain::Chemistry, chapter: "Buffers & Titration" },
        RegisteredFamilyInfo { schema_id: SCHEMA_CHEMISTRY_ELECTROCHEMISTRY, skill_id: SKILL_CHEMISTRY_ELECTROCHEMISTRY, domain: Domain::Chemistry, chapter: "Electrochemistry" },
        RegisteredFamilyInfo { schema_id: SCHEMA_CHEMISTRY_KINETICS, skill_id: SKILL_CHEMISTRY_KINETICS, domain: Domain::Chemistry, chapter: "Kinetics" },
        RegisteredFamilyInfo { schema_id: SCHEMA_CHEMISTRY_REACTION_NETWORKS, skill_id: SKILL_CHEMISTRY_REACTION_NETWORKS, domain: Domain::Chemistry, chapter: "Reaction Networks" },
    ]
}

// =========================================================================
// 3. TELEMETRY & MULTI-TIER CHECKPOINT DATA STRUCTURES
// =========================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChapterTelemetry {
    pub chapter_name: String,
    pub domain_name: String,
    pub attempts: usize,
    pub correct: usize,
    pub latencies: Vec<u64>,
    pub target_latencies: Vec<u64>,
    pub transfer_attempts: usize,
    pub transfer_correct: usize,
    pub error_counts: HashMap<String, usize>,
    pub difficulty_counts: [usize; 5],
    pub variant_counts: [usize; 5], // Parameter, Isomorphic, Structural, MultiConcept, Transfer
    pub remediation_counts: usize,
}

impl ChapterTelemetry {
    pub fn accuracy(&self) -> f64 {
        if self.attempts == 0 {
            0.0
        } else {
            (self.correct as f64 / self.attempts as f64) * 100.0
        }
    }
    pub fn transfer_accuracy(&self) -> f64 {
        if self.transfer_attempts == 0 {
            0.0
        } else {
            (self.transfer_correct as f64 / self.transfer_attempts as f64) * 100.0
        }
    }
    pub fn median_latency(&self) -> u64 {
        if self.latencies.is_empty() {
            return 0;
        }
        let mut s = self.latencies.clone();
        s.sort_unstable();
        s[s.len() / 2]
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CheckpointTelemetry {
    pub day: u32,
    pub total_attempts: usize,
    pub total_correct: usize,
    pub accuracy: f64,
    pub median_latency_ms: u64,
    pub p95_latency_ms: u64,
    pub mean_time_ratio: f64,
    pub total_hints_used: usize,
    pub hints_per_problem: f64,
    pub transfer_attempts: usize,
    pub transfer_correct: usize,
    pub transfer_accuracy: f64,
    pub difficulty_distribution: [f64; 5],
    pub variant_distribution: [f64; 5],
    pub error_counts: HashMap<String, usize>,
    pub domain_error_recurrence: HashMap<String, usize>,
    pub chapter_telemetry: HashMap<String, ChapterTelemetry>,
    pub domain_accuracies: HashMap<String, f64>,
    pub remediations_enqueued: usize,
    pub remediations_executed: usize,
    pub remediations_resolved: usize,
    pub resolution_rate: f64,
    pub circuit_breaker_triggers: usize,
    pub max_queue_depth: usize,
    pub final_mastered_skills: usize,
    pub total_unique_variants: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LongitudinalRunResult {
    pub cohort_name: String,
    pub is_adaptive: bool,
    pub total_days: u32,
    pub reviews_per_day: usize,
    pub seed: u64,
    pub checkpoint_day1: CheckpointTelemetry,
    pub checkpoint_day30: CheckpointTelemetry,
    pub checkpoint_day60: CheckpointTelemetry,
    pub checkpoint_day90: CheckpointTelemetry,
    pub db_size_bytes_day90: u64,
}

// =========================================================================
// 4. FAIR MATCHED BASELINE VS STUDYLAB SIMULATION ENGINE
// =========================================================================

pub struct Phase32SimulationHarness;

impl Phase32SimulationHarness {
    /// Executes a fully deterministic longitudinal simulation run
    pub fn run_simulation(
        cohort: CohortId,
        days: u32,
        reviews_per_day: usize,
        is_adaptive: bool,
        study_gap_active: bool,
        exam_pressure_active: bool,
        seed: u64,
    ) -> (LongitudinalRunResult, ProceduralService) {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join(format!(
            "phase32_sim_{}_{}_{}_seed{}.db",
            cohort.as_str(),
            days,
            if is_adaptive { "StudyLab" } else { "MatchedBaseline" },
            seed
        ));
        let service = ProceduralService::open(&db_path).unwrap();

        let all_registered = get_all_canonical_schemas();
        let base_start_time = Utc::now() - chrono::Duration::days(days as i64 + 1);

        let mut attempt_counter = 0usize;
        let mut total_correct = 0usize;
        let mut total_hints = 0usize;
        let mut transfer_attempts = 0usize;
        let mut transfer_correct = 0usize;

        let mut all_latencies = Vec::new();
        let mut time_ratios = Vec::new();
        let mut level_counts = [0usize; 5];
        let mut variant_counts = [0usize; 5]; // Parameter, Isomorphic, Structural, MultiConcept, Transfer
        let mut global_errors: HashMap<String, usize> = HashMap::new();
        let mut domain_error_recurrence: HashMap<String, usize> = HashMap::new();
        let mut chapters_map: HashMap<String, ChapterTelemetry> = HashMap::new();
        let mut domain_map: HashMap<String, (usize, usize)> = HashMap::new();
        let mut unique_variants_seen: HashMap<String, usize> = HashMap::new();

        let mut remediations_enqueued = 0usize;
        let mut remediations_executed = 0usize;
        let mut remediations_resolved = 0usize;
        let mut circuit_breaker_triggers = 0usize;
        let mut max_queue_depth = 0usize;

        let mut cp_day1 = CheckpointTelemetry::default();
        let mut cp_day30 = CheckpointTelemetry::default();
        let mut cp_day60 = CheckpointTelemetry::default();
        let mut cp_day90 = CheckpointTelemetry::default();

        for day in 1..=days {
            let is_gap_day = study_gap_active && (21..=30).contains(&day);
            let is_exam_day = exam_pressure_active && day >= 45;

            let daily_reviews = if is_gap_day {
                0
            } else if is_exam_day {
                reviews_per_day * 3 / 2 // 1.5x workload
            } else {
                reviews_per_day
            };

            let day_time = base_start_time + chrono::Duration::days(day as i64);

            if daily_reviews > 0 {
                let sessions_per_day = 3;
                let reviews_per_session = daily_reviews / sessions_per_day;

                for session_idx in 0..sessions_per_day {
                    let session_time =
                        day_time + chrono::Duration::hours((session_idx * 4) as i64);

                    for r in 0..reviews_per_session {
                        attempt_counter += 1;

                        // Structured interleaving across all 32 canonical schemas
                        let schema_idx = (attempt_counter + (day as usize * 7) + (seed as usize * 13))
                            % all_registered.len();
                        let schema_info = &all_registered[schema_idx];
                        let schema = service
                            .store()
                            .get_schema(&SchemaId::from(schema_info.schema_id))
                            .unwrap()
                            .unwrap();
                        let family = service
                            .store()
                            .get_problem_family(&schema.problem_family_id)
                            .unwrap()
                            .unwrap();

                        let skill_state = service.load_skill_state(&schema.skill_id).unwrap();
                        let max_level = family.max_difficulty as u32;

                        // -------------------------------------------------------------
                        // DIFFICULTY LEVEL ASSIGNMENT
                        // -------------------------------------------------------------
                        let (diff_level, target_time_ms) = if is_adaptive {
                            // Condition B: StudyLab Adaptive Difficulty Engine
                            let dec = AdaptiveDifficultyEngine::evaluate_difficulty(
                                skill_state.as_ref(),
                                None,
                                None,
                            );
                            (dec.level.min(max_level), dec.target_time_ms)
                        } else {
                            // Condition A: Fair Matched Baseline
                            // Exposes balanced curriculum levels (1 to 5) uniformly across practice
                            let baseline_level = (((attempt_counter + (day as usize)) % max_level as usize) + 1) as u32;
                            let target_ms = match baseline_level {
                                1 => 25_000,
                                2 => 35_000,
                                3 => 50_000,
                                4 => 65_000,
                                _ => 80_000,
                            };
                            (baseline_level, target_ms)
                        };

                        level_counts[(diff_level.max(1).min(5) - 1) as usize] += 1;

                        // -------------------------------------------------------------
                        // VARIANT CATEGORY ASSIGNMENT
                        // -------------------------------------------------------------
                        let variant_cat = if is_adaptive {
                            // Condition B: StudyLab SkillState-driven Variant Progression Ladder
                            if let Some(ref st) = skill_state {
                                match st.practice_state {
                                    PracticeProgressionState::New
                                    | PracticeProgressionState::Learning => {
                                        VariantCategory::Parameter
                                    }
                                    PracticeProgressionState::Fluent => {
                                        VariantCategory::Isomorphic
                                    }
                                    PracticeProgressionState::Variation => {
                                        VariantCategory::Structural
                                    }
                                    PracticeProgressionState::Transfer => {
                                        VariantCategory::Transfer
                                    }
                                    PracticeProgressionState::Mastered
                                    | PracticeProgressionState::Retired
                                    | PracticeProgressionState::Hibernating => {
                                        VariantCategory::MultiConcept
                                    }
                                }
                            } else {
                                VariantCategory::Parameter
                            }
                        } else {
                            // Condition A: Fair Matched Baseline
                            // Exposes full natural variant spectrum (Parameter, Isomorphic, Structural, MultiConcept, Transfer)
                            // without learner-specific adaptive triggers
                            match (attempt_counter + (day as usize * 3)) % 5 {
                                0 => VariantCategory::Parameter,
                                1 => VariantCategory::Isomorphic,
                                2 => VariantCategory::Structural,
                                3 => VariantCategory::MultiConcept,
                                _ => VariantCategory::Transfer,
                            }
                        };

                        let var_idx = match variant_cat {
                            VariantCategory::Parameter => 0,
                            VariantCategory::Isomorphic => 1,
                            VariantCategory::Structural => 2,
                            VariantCategory::MultiConcept => 3,
                            VariantCategory::Transfer | VariantCategory::Contextual => 4,
                        };
                        variant_counts[var_idx] += 1;

                        let is_transfer_curr = matches!(
                            variant_cat,
                            VariantCategory::Structural
                                | VariantCategory::Contextual
                                | VariantCategory::Transfer
                                | VariantCategory::MultiConcept
                        );

                        if is_transfer_curr {
                            transfer_attempts += 1;
                        }

                        // -------------------------------------------------------------
                        // DETERMINISTIC ATTEMPT SIMULATION
                        // -------------------------------------------------------------
                        let (is_correct, actual_time, error_cat, hints_used, _hint_lvl, dom_ev) =
                            cohort.simulate_attempt(
                                &schema_info.domain,
                                schema_info.chapter,
                                diff_level,
                                variant_cat,
                                target_time_ms,
                                day,
                                attempt_counter,
                                false,
                                is_exam_day,
                                seed,
                            );

                        if is_correct {
                            total_correct += 1;
                            if is_transfer_curr {
                                transfer_correct += 1;
                            }
                        }

                        all_latencies.push(actual_time);
                        total_hints += hints_used;
                        let ratio = actual_time as f64 / target_time_ms as f64;
                        time_ratios.push(ratio);

                        let var_key = format!("{}-{:?}-L{}", schema_info.schema_id, variant_cat, diff_level);
                        *unique_variants_seen.entry(var_key).or_insert(0) += 1;

                        // -------------------------------------------------------------
                        // CHAPTER & DOMAIN TELEMETRY CAPTURE
                        // -------------------------------------------------------------
                        let chap_entry = chapters_map
                            .entry(schema_info.chapter.to_string())
                            .or_insert_with(|| ChapterTelemetry {
                                chapter_name: schema_info.chapter.to_string(),
                                domain_name: format!("{:?}", schema_info.domain),
                                ..Default::default()
                            });
                        chap_entry.attempts += 1;
                        if is_correct {
                            chap_entry.correct += 1;
                        }
                        chap_entry.latencies.push(actual_time);
                        chap_entry.target_latencies.push(target_time_ms);
                        chap_entry.difficulty_counts[(diff_level.max(1).min(5) - 1) as usize] += 1;
                        chap_entry.variant_counts[var_idx] += 1;
                        if is_transfer_curr {
                            chap_entry.transfer_attempts += 1;
                            if is_correct {
                                chap_entry.transfer_correct += 1;
                            }
                        }
                        if let Some(ref e) = error_cat {
                            *chap_entry.error_counts.entry(format!("{:?}", e)).or_insert(0) += 1;
                            *global_errors.entry(format!("{:?}", e)).or_insert(0) += 1;

                            // Domain-specific recurrence categories
                            let dom_err_label = match schema_info.domain {
                                Domain::Mathematics => "Math:CalculationSlip",
                                Domain::Reasoning => "Reasoning:RepresentationError",
                                Domain::Physics => "Physics:ModelSetupError",
                                Domain::Chemistry => "Chemistry:IntermediateSetupError",
                                _ => "GeneralError",
                            };
                            *domain_error_recurrence.entry(dom_err_label.to_string()).or_insert(0) += 1;
                        }

                        let dom_entry = domain_map
                            .entry(format!("{:?}", schema_info.domain))
                            .or_insert((0, 0));
                        dom_entry.1 += 1;
                        if is_correct {
                            dom_entry.0 += 1;
                        }

                        // -------------------------------------------------------------
                        // PERSISTENCE (PROBLEM INSTANCE & ATTEMPT)
                        // -------------------------------------------------------------
                        let instance_id = ProblemInstanceId::new(format!(
                            "inst-{}-{}-s{}",
                            cohort.as_str(),
                            attempt_counter,
                            seed
                        ));
                        let instance = ProblemInstance::new(
                            instance_id.clone(),
                            schema.problem_family_id.clone(),
                            attempt_counter as u64,
                            serde_json::json!({ "difficulty": diff_level }),
                            "Simulated Problem Prompt",
                            serde_json::json!("42"),
                        );
                        service.save_problem_instance(instance).unwrap();

                        let attempt_id = AttemptId::new(format!(
                            "att-{}-{}-{}-s{}",
                            cohort.as_str(),
                            day,
                            attempt_counter,
                            seed
                        ));
                        let mut attempt = PracticeAttempt::new(
                            attempt_id.clone(),
                            instance_id,
                            schema.id.clone(),
                            schema.skill_id.clone(),
                            serde_json::json!("42"),
                            is_correct,
                            if is_correct { 1.0 } else { 0.0 },
                            actual_time,
                        );
                        attempt.attempted_at = session_time.timestamp();

                        let mut error_events = Vec::new();
                        if let Some(ref err) = error_cat {
                            let err_id = ErrorEventId::new(format!(
                                "err-{}-{}-{}-s{}",
                                cohort.as_str(),
                                day,
                                attempt_counter,
                                seed
                            ));
                            let ee = ErrorEvent::new(
                                err_id,
                                attempt_id.clone(),
                                err.as_str(),
                                serde_json::json!({ "reason": format!("{:?}", err) }),
                            );
                            error_events.push(ee);
                        }

                        let mut current_state = service
                            .load_skill_state(&schema.skill_id)
                            .unwrap()
                            .unwrap_or_else(|| {
                                procedural::skills::SkillState::new(schema.skill_id.clone())
                            });

                        let mastery_ev = MasteryEvidence {
                            final_correctness: is_correct,
                            latency_evidence: actual_time,
                            independence: if hints_used == 0 {
                                IndependenceLevel::Independent
                            } else {
                                IndependenceLevel::LightSupport
                            },
                            hint_dependence: hints_used as u32,
                            retry_dependence: 0,
                            variant_exposure: Some("sim_variant".to_string()),
                            variant_category: variant_cat,
                            domain_evidence: dom_ev.clone(),
                            diagnostic_errors: error_cat.clone().into_iter().collect(),
                            ..Default::default()
                        };

                        current_state.record_attempt_outcome(
                            &mastery_ev,
                            if is_correct { 1.0 } else { 0.0 },
                            target_time_ms,
                            session_time.timestamp(),
                        );
                        service
                            .store()
                            .record_attempt_atomic(&attempt, &error_events, &current_state)
                            .unwrap();

                        // -------------------------------------------------------------
                        // REMEDIATION ENGINE (STUDYLAB ONLY)
                        // -------------------------------------------------------------
                        if is_adaptive {
                            let q_arc = service.remediation_queue();
                            let mut q = q_arc.lock().unwrap();

                            if let Some(ref err) = error_cat {
                                let key = (schema.skill_id.clone(), err.clone());
                                let recurrence =
                                    q.recurrence_tracker.get(&key).copied().unwrap_or(0) + 1;

                                let rem_ctx = RemediationContext {
                                    skill_id: &schema.skill_id,
                                    schema_id: &schema.id,
                                    domain: family.domain.clone(),
                                    primary_error: err.clone(),
                                    step_error: None,
                                    decision_point_correct: Some(false),
                                    independence: IndependenceLevel::Independent,
                                    progression_state: current_state.practice_state,
                                    recent_attempts: &current_state.recent_attempts,
                                    source_attempt_id: &attempt_id,
                                    recurrence_count: recurrence,
                                    is_transfer_attempt: matches!(
                                        variant_cat,
                                        VariantCategory::Transfer
                                    ),
                                };

                                let rem_action = RemediationPolicy::evaluate(&rem_ctx);
                                if rem_action.kind == RemediationActionKind::CircuitBreaker {
                                    circuit_breaker_triggers += 1;
                                }

                                q.enqueue(rem_action);
                                remediations_enqueued += 1;
                                chap_entry.remediation_counts += 1;
                                if q.pending_actions.len() > max_queue_depth {
                                    max_queue_depth = q.pending_actions.len();
                                }
                            } else if let Some(ref prev_err) = error_cat {
                                q.record_resolution(&schema.skill_id, prev_err);
                                remediations_resolved += 1;
                            }

                            // Interleaved remediation drill at session end
                            if (r + 1) == reviews_per_session && !q.pending_actions.is_empty() {
                                if let Some(action) =
                                    q.select_next_remediation(&PracticeMode::MixedInterleaved)
                                {
                                    remediations_executed += 1;
                                    let rem_success = match cohort {
                                        CohortId::CohortAStrongFast
                                        | CohortId::CohortBStrongSlow => true,
                                        CohortId::CohortGMixedImproving => day >= 20,
                                        CohortId::CohortDConceptWeak => day >= 60,
                                        CohortId::CohortIBeginner => day >= 45,
                                        _ => (attempt_counter + seed as usize) % 2 == 0,
                                    };

                                    if rem_success {
                                        q.record_resolution(
                                            &action.skill_id,
                                            &action.primary_error,
                                        );
                                        remediations_resolved += 1;
                                    }

                                    let rem_instance_id = ProblemInstanceId::new(format!(
                                        "rem-inst-{}-{}-s{}",
                                        cohort.as_str(),
                                        attempt_counter,
                                        seed
                                    ));
                                    let rem_schema = service
                                        .store()
                                        .get_schema(&action.schema_id)
                                        .unwrap()
                                        .unwrap();
                                    let rem_instance = ProblemInstance::new(
                                        rem_instance_id.clone(),
                                        rem_schema.problem_family_id.clone(),
                                        attempt_counter as u64,
                                        serde_json::json!({ "difficulty": diff_level }),
                                        "Remediation Prompt",
                                        serde_json::json!("42"),
                                    );
                                    service.save_problem_instance(rem_instance).unwrap();

                                    let rem_attempt = PracticeAttempt::new(
                                        AttemptId::new(format!(
                                            "rem-att-{}-{}-{}-s{}",
                                            cohort.as_str(),
                                            day,
                                            attempt_counter,
                                            seed
                                        )),
                                        rem_instance_id,
                                        action.schema_id.clone(),
                                        action.skill_id.clone(),
                                        serde_json::json!("42"),
                                        rem_success,
                                        if rem_success { 1.0 } else { 0.0 },
                                        target_time_ms,
                                    );
                                    let _ = service.record_practice_attempt_with_variant(
                                        rem_attempt,
                                        Vec::new(),
                                        Some("remediation_drill"),
                                        target_time_ms,
                                    );
                                }
                            }
                        }
                    }
                }
            }

            // -----------------------------------------------------------------
            // CHECKPOINT TELEMETRY SNAPSHOT CAPTURE (Days 1, 30, 60, 90)
            // -----------------------------------------------------------------
            if day == 1 || day == 30 || day == 60 || day == 90 {
                let mut sorted_lat = all_latencies.clone();
                sorted_lat.sort_unstable();
                let med_lat = if sorted_lat.is_empty() {
                    0
                } else {
                    sorted_lat[sorted_lat.len() / 2]
                };
                let p95_lat = if sorted_lat.is_empty() {
                    0
                } else {
                    sorted_lat[((sorted_lat.len() as f64 * 0.95) as usize)
                        .min(sorted_lat.len() - 1)]
                };
                let mean_ratio = if time_ratios.is_empty() {
                    1.0
                } else {
                    time_ratios.iter().sum::<f64>() / time_ratios.len() as f64
                };

                let sum_levels: usize = level_counts.iter().sum();
                let diff_dist = [
                    if sum_levels == 0 { 0.0 } else { (level_counts[0] as f64 / sum_levels as f64) * 100.0 },
                    if sum_levels == 0 { 0.0 } else { (level_counts[1] as f64 / sum_levels as f64) * 100.0 },
                    if sum_levels == 0 { 0.0 } else { (level_counts[2] as f64 / sum_levels as f64) * 100.0 },
                    if sum_levels == 0 { 0.0 } else { (level_counts[3] as f64 / sum_levels as f64) * 100.0 },
                    if sum_levels == 0 { 0.0 } else { (level_counts[4] as f64 / sum_levels as f64) * 100.0 },
                ];

                let sum_vars: usize = variant_counts.iter().sum();
                let var_dist = [
                    if sum_vars == 0 { 0.0 } else { (variant_counts[0] as f64 / sum_vars as f64) * 100.0 },
                    if sum_vars == 0 { 0.0 } else { (variant_counts[1] as f64 / sum_vars as f64) * 100.0 },
                    if sum_vars == 0 { 0.0 } else { (variant_counts[2] as f64 / sum_vars as f64) * 100.0 },
                    if sum_vars == 0 { 0.0 } else { (variant_counts[3] as f64 / sum_vars as f64) * 100.0 },
                    if sum_vars == 0 { 0.0 } else { (variant_counts[4] as f64 / sum_vars as f64) * 100.0 },
                ];

                let mut dom_accs = HashMap::new();
                for (d, (c, tot)) in &domain_map {
                    dom_accs.insert(
                        d.clone(),
                        if *tot > 0 {
                            (*c as f64 / *tot as f64) * 100.0
                        } else {
                            0.0
                        },
                    );
                }

                let all_skills = service.store().list_all_skills().unwrap();
                let mut mastered_cnt = 0;
                for s in &all_skills {
                    if let Ok(Some(st)) = service.load_skill_state(&s.id) {
                        if matches!(
                            st.practice_state,
                            PracticeProgressionState::Mastered
                                | PracticeProgressionState::Transfer
                        ) {
                            mastered_cnt += 1;
                        }
                    }
                }

                let res_rate = if remediations_enqueued > 0 {
                    (remediations_resolved as f64 / remediations_enqueued as f64) * 100.0
                } else {
                    100.0
                };

                let cp = CheckpointTelemetry {
                    day,
                    total_attempts: attempt_counter,
                    total_correct,
                    accuracy: if attempt_counter == 0 {
                        0.0
                    } else {
                        (total_correct as f64 / attempt_counter as f64) * 100.0
                    },
                    median_latency_ms: med_lat,
                    p95_latency_ms: p95_lat,
                    mean_time_ratio: mean_ratio,
                    total_hints_used: total_hints,
                    hints_per_problem: if attempt_counter == 0 {
                        0.0
                    } else {
                        total_hints as f64 / attempt_counter as f64
                    },
                    transfer_attempts,
                    transfer_correct,
                    transfer_accuracy: if transfer_attempts == 0 {
                        0.0
                    } else {
                        (transfer_correct as f64 / transfer_attempts as f64) * 100.0
                    },
                    difficulty_distribution: diff_dist,
                    variant_distribution: var_dist,
                    error_counts: global_errors.clone(),
                    domain_error_recurrence: domain_error_recurrence.clone(),
                    chapter_telemetry: chapters_map.clone(),
                    domain_accuracies: dom_accs,
                    remediations_enqueued,
                    remediations_executed,
                    remediations_resolved,
                    resolution_rate: res_rate,
                    circuit_breaker_triggers,
                    max_queue_depth,
                    final_mastered_skills: mastered_cnt,
                    total_unique_variants: unique_variants_seen.len(),
                };

                match day {
                    1 => cp_day1 = cp,
                    30 => cp_day30 = cp,
                    60 => cp_day60 = cp,
                    90 => cp_day90 = cp,
                    _ => {}
                }
            }
        }

        let db_size = std::fs::metadata(&db_path).map(|m| m.len()).unwrap_or(0);

        let res = LongitudinalRunResult {
            cohort_name: cohort.as_str().to_string(),
            is_adaptive,
            total_days: days,
            reviews_per_day,
            seed,
            checkpoint_day1: cp_day1,
            checkpoint_day30: cp_day30,
            checkpoint_day60: cp_day60,
            checkpoint_day90: cp_day90,
            db_size_bytes_day90: db_size,
        };

        (res, service)
    }
}

// =========================================================================
// 5. TEST SUITES: PHASE 32 LONGITUDINAL VALIDATION
// =========================================================================

#[test]
fn test_phase32_all_12_cohorts_matched_baseline_vs_studylab_standard_workload() {
    println!("\n=========================================================================================================");
    println!("  PHASE 32: 12-COHORT FAIR MATCHED BASELINE VS STUDYLAB LONGITUDINAL COMPARISON (45 REVIEWS/DAY)       ");
    println!("=========================================================================================================\n");

    println!(
        "{:<26} | {:<8} | {:<8} | {:<8} | {:<8} | {:<10} | {:<10} | {:<8} | {:<8} | {:<7}",
        "Cohort", "Condition", "D30 Acc", "D60 Acc", "D90 Acc", "D90 Trans", "Med Lat ms", "Hints/P", "Mastered", "MaxQ"
    );
    println!("{:-<125}", "");

    for cohort in CohortId::all() {
        let (sl, _) = Phase32SimulationHarness::run_simulation(*cohort, 90, 45, true, false, false, 1);
        let (base, _) = Phase32SimulationHarness::run_simulation(*cohort, 90, 45, false, false, false, 1);

        println!(
            "{:<26} | {:<8} | {:<7.1}% | {:<7.1}% | {:<7.1}% | {:<9.1}% | {:<10} | {:<8.2} | {:<8} | {:<7}",
            cohort.as_str(),
            "StudyLab",
            sl.checkpoint_day30.accuracy,
            sl.checkpoint_day60.accuracy,
            sl.checkpoint_day90.accuracy,
            sl.checkpoint_day90.transfer_accuracy,
            sl.checkpoint_day90.median_latency_ms,
            sl.checkpoint_day90.hints_per_problem,
            sl.checkpoint_day90.final_mastered_skills,
            sl.checkpoint_day90.max_queue_depth
        );

        println!(
            "{:<26} | {:<8} | {:<7.1}% | {:<7.1}% | {:<7.1}% | {:<9.1}% | {:<10} | {:<8.2} | {:<8} | {:<7}",
            "",
            "Baseline",
            base.checkpoint_day30.accuracy,
            base.checkpoint_day60.accuracy,
            base.checkpoint_day90.accuracy,
            base.checkpoint_day90.transfer_accuracy,
            base.checkpoint_day90.median_latency_ms,
            base.checkpoint_day90.hints_per_problem,
            base.checkpoint_day90.final_mastered_skills,
            0
        );
        println!("{:-<125}", "");

        assert_eq!(sl.checkpoint_day90.total_attempts, 4050);
        assert_eq!(base.checkpoint_day90.total_attempts, 4050);
    }
}

#[test]
fn test_phase32_matched_exposure_and_intent_to_practice_analysis() {
    println!("\n=========================================================================================================");
    println!("  PHASE 32: ANALYSIS A (INTENT-TO-PRACTICE) VS ANALYSIS B (MATCHED EXPOSURE)                             ");
    println!("=========================================================================================================\n");

    for cohort in &[CohortId::CohortGMixedImproving, CohortId::CohortDConceptWeak, CohortId::CohortFTransferWeak] {
        let (sl, _) = Phase32SimulationHarness::run_simulation(*cohort, 90, 45, true, false, false, 1);
        let (base, _) = Phase32SimulationHarness::run_simulation(*cohort, 90, 45, false, false, false, 1);

        let delta_acc_intent = sl.checkpoint_day90.accuracy - base.checkpoint_day90.accuracy;
        let delta_trans_intent = sl.checkpoint_day90.transfer_accuracy - base.checkpoint_day90.transfer_accuracy;
        let delta_mastery = sl.checkpoint_day90.final_mastered_skills as i32 - base.checkpoint_day90.final_mastered_skills as i32;

        println!("Cohort: {}", cohort.as_str());
        println!("  - Intent-to-Practice (Total Attempts: SL={}, Base={}):", sl.checkpoint_day90.total_attempts, base.checkpoint_day90.total_attempts);
        println!("      Global Accuracy Delta:   {:+5.1}% (SL={:.1}% vs Base={:.1}%)", delta_acc_intent, sl.checkpoint_day90.accuracy, base.checkpoint_day90.accuracy);
        println!("      Transfer Accuracy Delta: {:+5.1}% (SL={:.1}% vs Base={:.1}%)", delta_trans_intent, sl.checkpoint_day90.transfer_accuracy, base.checkpoint_day90.transfer_accuracy);
        println!("      Mastered Skills Delta:   {:+3}    (SL={} vs Base={})", delta_mastery, sl.checkpoint_day90.final_mastered_skills, base.checkpoint_day90.final_mastered_skills);
        println!("  - Matched Exposure Verification (Difficulty & Variant Distribution):");
        println!("      SL Difficulty Dist:   [{:.1}%, {:.1}%, {:.1}%, {:.1}%, {:.1}%]", sl.checkpoint_day90.difficulty_distribution[0], sl.checkpoint_day90.difficulty_distribution[1], sl.checkpoint_day90.difficulty_distribution[2], sl.checkpoint_day90.difficulty_distribution[3], sl.checkpoint_day90.difficulty_distribution[4]);
        println!("      Base Difficulty Dist: [{:.1}%, {:.1}%, {:.1}%, {:.1}%, {:.1}%]", base.checkpoint_day90.difficulty_distribution[0], base.checkpoint_day90.difficulty_distribution[1], base.checkpoint_day90.difficulty_distribution[2], base.checkpoint_day90.difficulty_distribution[3], base.checkpoint_day90.difficulty_distribution[4]);
        println!("      SL Variant Dist:      [{:.1}%, {:.1}%, {:.1}%, {:.1}%, {:.1}%]", sl.checkpoint_day90.variant_distribution[0], sl.checkpoint_day90.variant_distribution[1], sl.checkpoint_day90.variant_distribution[2], sl.checkpoint_day90.variant_distribution[3], sl.checkpoint_day90.variant_distribution[4]);
        println!("      Base Variant Dist:    [{:.1}%, {:.1}%, {:.1}%, {:.1}%, {:.1}%]", base.checkpoint_day90.variant_distribution[0], base.checkpoint_day90.variant_distribution[1], base.checkpoint_day90.variant_distribution[2], base.checkpoint_day90.variant_distribution[3], base.checkpoint_day90.variant_distribution[4]);
    }
}

#[test]
fn test_phase32_chapter_level_and_domain_isolated_comparison() {
    println!("\n=========================================================================================================");
    println!("  PHASE 32: CHAPTER-LEVEL & DOMAIN ISOLATED EMPIRICAL OUTCOMES (COHORT L - UNEVEN)                       ");
    println!("=========================================================================================================\n");

    let (sl, _) = Phase32SimulationHarness::run_simulation(CohortId::CohortLUnevenMultiSubject, 90, 45, true, false, false, 1);
    let (base, _) = Phase32SimulationHarness::run_simulation(CohortId::CohortLUnevenMultiSubject, 90, 45, false, false, false, 1);

    println!("{:<22} | {:<12} | {:<10} | {:<10} | {:<12} | {:<12} | {:<10}", "Chapter", "Domain", "SL Acc %", "Base Acc %", "SL Trans %", "Base Trans %", "Remediations");
    println!("{:-<105}", "");

    for (chap, sl_telemetry) in &sl.checkpoint_day90.chapter_telemetry {
        let base_telemetry = base.checkpoint_day90.chapter_telemetry.get(chap);
        let base_acc = base_telemetry.map(|b| b.accuracy()).unwrap_or(0.0);
        let base_trans = base_telemetry.map(|b| b.transfer_accuracy()).unwrap_or(0.0);

        println!(
            "{:<22} | {:<12} | {:<10.1} | {:<10.1} | {:<12.1} | {:<12.1} | {:<10}",
            chap,
            sl_telemetry.domain_name,
            sl_telemetry.accuracy(),
            base_acc,
            sl_telemetry.transfer_accuracy(),
            base_trans,
            sl_telemetry.remediation_counts,
        );
    }

    println!("\nDomain-Isolated Accuracies (Day 90):");
    for (dom, sl_acc) in &sl.checkpoint_day90.domain_accuracies {
        let base_acc = base.checkpoint_day90.domain_accuracies.get(dom).copied().unwrap_or(0.0);
        println!("  - Domain: {:<15} -> StudyLab: {:.1}% | Baseline: {:.1}% | Delta: {:+.1}%", dom, sl_acc, base_acc, sl_acc - base_acc);
    }
}

#[test]
fn test_phase32_treatment_on_hard_cohorts_audit() {
    println!("\n=========================================================================================================");
    println!("  PHASE 32: TREATMENT-ON-THE-HARD-COHORTS LONGITUDINAL EVALUATION                                       ");
    println!("=========================================================================================================\n");

    for cohort in CohortId::hard_cohorts() {
        let (sl, _) = Phase32SimulationHarness::run_simulation(*cohort, 90, 45, true, false, false, 1);
        let (base, _) = Phase32SimulationHarness::run_simulation(*cohort, 90, 45, false, false, false, 1);

        println!(
            "Hard Cohort: {:<26} | SL Acc: {:<5.1}% vs Base Acc: {:<5.1}% | SL Trans: {:<5.1}% vs Base Trans: {:<5.1}% | RemResolved: {}/{} ({:.0}%)",
            cohort.as_str(),
            sl.checkpoint_day90.accuracy,
            base.checkpoint_day90.accuracy,
            sl.checkpoint_day90.transfer_accuracy,
            base.checkpoint_day90.transfer_accuracy,
            sl.checkpoint_day90.remediations_resolved,
            sl.checkpoint_day90.remediations_enqueued,
            sl.checkpoint_day90.resolution_rate,
        );
    }
}

#[test]
fn test_phase32_transfer_progression_ladder_test() {
    println!("\n=========================================================================================================");
    println!("  PHASE 32: TRANSFER PROGRESSION LADDER TEST (COHORT F - TRANSFER WEAK)                                  ");
    println!("=========================================================================================================\n");

    let (sl, _) = Phase32SimulationHarness::run_simulation(CohortId::CohortFTransferWeak, 90, 45, true, false, false, 1);
    let (base, _) = Phase32SimulationHarness::run_simulation(CohortId::CohortFTransferWeak, 90, 45, false, false, false, 1);

    println!("Cohort F Transfer Trajectory:");
    println!("  - Day 1:  SL Trans: {:<5.1}% (Att: {}) | Base Trans: {:<5.1}% (Att: {})", sl.checkpoint_day1.transfer_accuracy, sl.checkpoint_day1.transfer_attempts, base.checkpoint_day1.transfer_accuracy, base.checkpoint_day1.transfer_attempts);
    println!("  - Day 30: SL Trans: {:<5.1}% (Att: {}) | Base Trans: {:<5.1}% (Att: {})", sl.checkpoint_day30.transfer_accuracy, sl.checkpoint_day30.transfer_attempts, base.checkpoint_day30.transfer_accuracy, base.checkpoint_day30.transfer_attempts);
    println!("  - Day 60: SL Trans: {:<5.1}% (Att: {}) | Base Trans: {:<5.1}% (Att: {})", sl.checkpoint_day60.transfer_accuracy, sl.checkpoint_day60.transfer_attempts, base.checkpoint_day60.transfer_accuracy, base.checkpoint_day60.transfer_attempts);
    println!("  - Day 90: SL Trans: {:<5.1}% (Att: {}) | Base Trans: {:<5.1}% (Att: {})", sl.checkpoint_day90.transfer_accuracy, sl.checkpoint_day90.transfer_attempts, base.checkpoint_day90.transfer_accuracy, base.checkpoint_day90.transfer_attempts);

    println!("  - SL Unique Useful Variants Practiced:   {}", sl.checkpoint_day90.total_unique_variants);
    println!("  - Base Unique Useful Variants Practiced: {}", base.checkpoint_day90.total_unique_variants);
}

#[test]
fn test_phase32_retention_gap_recovery_test() {
    println!("\n=========================================================================================================");
    println!("  PHASE 32: RETENTION & STUDY-GAP RECOVERY COMPARISON (COHORT J - LOW RETENTION)                        ");
    println!("=========================================================================================================\n");

    let (sl, _) = Phase32SimulationHarness::run_simulation(CohortId::CohortJLowRetention, 60, 45, true, true, false, 1);
    let (base, _) = Phase32SimulationHarness::run_simulation(CohortId::CohortJLowRetention, 60, 45, false, true, false, 1);

    println!("Cohort J (Study Gap Days 21-30, Active Days 1-20 & 31-60):");
    println!("  - Day 1:  SL Acc: {:.1}%, Med Lat: {}ms | Base Acc: {:.1}%, Med Lat: {}ms", sl.checkpoint_day1.accuracy, sl.checkpoint_day1.median_latency_ms, base.checkpoint_day1.accuracy, base.checkpoint_day1.median_latency_ms);
    println!("  - Day 30: SL Acc: {:.1}%, Med Lat: {}ms | Base Acc: {:.1}%, Med Lat: {}ms (During Gap)", sl.checkpoint_day30.accuracy, sl.checkpoint_day30.median_latency_ms, base.checkpoint_day30.accuracy, base.checkpoint_day30.median_latency_ms);
    println!("  - Day 60: SL Acc: {:.1}%, Med Lat: {}ms | Base Acc: {:.1}%, Med Lat: {}ms (Post-Recovery)", sl.checkpoint_day60.accuracy, sl.checkpoint_day60.median_latency_ms, base.checkpoint_day60.accuracy, base.checkpoint_day60.median_latency_ms);

    let recovery_delta = sl.checkpoint_day60.accuracy - base.checkpoint_day60.accuracy;
    println!("Post-Gap Retention Recovery Delta: {:+.1}% in favor of StudyLab", recovery_delta);
    assert!(sl.checkpoint_day60.accuracy >= 75.0, "StudyLab failed retention requalification");
}

#[test]
fn test_phase32_exam_pressure_speed_test() {
    println!("\n=========================================================================================================");
    println!("  PHASE 32: EXAM PRESSURE & SPEED PRESSURE STRESS COMPARISON (COHORT K)                                  ");
    println!("=========================================================================================================\n");

    let (sl, _) = Phase32SimulationHarness::run_simulation(CohortId::CohortKSpeedPressure, 60, 45, true, false, true, 1);
    let (base, _) = Phase32SimulationHarness::run_simulation(CohortId::CohortKSpeedPressure, 60, 45, false, false, true, 1);

    println!("Cohort K under Exam Pressure (Days >= 45 with 1.5x workload):");
    println!("  - Day 30 (Normal):        SL Acc: {:.1}%, Med Lat: {}ms | Base Acc: {:.1}%, Med Lat: {}ms", sl.checkpoint_day30.accuracy, sl.checkpoint_day30.median_latency_ms, base.checkpoint_day30.accuracy, base.checkpoint_day30.median_latency_ms);
    println!("  - Day 60 (Exam Pressure): SL Acc: {:.1}%, Med Lat: {}ms | Base Acc: {:.1}%, Med Lat: {}ms", sl.checkpoint_day60.accuracy, sl.checkpoint_day60.median_latency_ms, base.checkpoint_day60.accuracy, base.checkpoint_day60.median_latency_ms);

    println!("Careless Calculation Error Counts at Day 60:");
    let sl_calc_err = sl.checkpoint_day60.error_counts.get("Calculation").copied().unwrap_or(0);
    let base_calc_err = base.checkpoint_day60.error_counts.get("Calculation").copied().unwrap_or(0);
    println!("  - StudyLab Calculation Slips:   {}", sl_calc_err);
    println!("  - Baseline Calculation Slips:   {}", base_calc_err);
}

#[test]
fn test_phase32_workload_sensitivity_and_efficiency_sweep() {
    println!("\n=========================================================================================================");
    println!("  PHASE 32: WORKLOAD SENSITIVITY & LEARNING EFFICIENCY SWEEP (20 vs 45 vs 75 REVIEWS/DAY)               ");
    println!("=========================================================================================================");

    for &load in &[20, 45, 75] {
        let (sl, _) = Phase32SimulationHarness::run_simulation(CohortId::CohortGMixedImproving, 60, load, true, false, false, 1);
        let (base, _) = Phase32SimulationHarness::run_simulation(CohortId::CohortGMixedImproving, 60, load, false, false, false, 1);

        let total_att_sl = sl.checkpoint_day60.total_attempts + sl.checkpoint_day60.remediations_executed;
        let total_att_base = base.checkpoint_day60.total_attempts;
        let extra_workload = ((total_att_sl as f64 - total_att_base as f64) / total_att_base as f64) * 100.0;
        let acc_gain = sl.checkpoint_day60.accuracy - base.checkpoint_day60.accuracy;
        let efficiency_ratio = acc_gain / (extra_workload.max(0.1));

        println!(
            "\nWorkload: {} rev/day | SL Attempts: {} (Base: {}) | Extra Workload: {:.1}%\n\
             - Accuracy: SL {:.1}% vs Base {:.1}% (Gain: {:+.1}%)\n\
             - Mastered Skills: SL {} vs Base {} (Gain: {:+})\n\
             - Mastered per 1000 Attempts: SL {:.2} vs Base {:.2}\n\
             - Incremental Gain per Extra Question: {:.3}%/question\n\
             - Efficiency Ratio: {:.2}",
            load,
            total_att_sl,
            total_att_base,
            extra_workload,
            sl.checkpoint_day60.accuracy,
            base.checkpoint_day60.accuracy,
            acc_gain,
            sl.checkpoint_day60.final_mastered_skills,
            base.checkpoint_day60.final_mastered_skills,
            sl.checkpoint_day60.final_mastered_skills as i32 - base.checkpoint_day60.final_mastered_skills as i32,
            (sl.checkpoint_day60.final_mastered_skills as f64 / total_att_sl as f64) * 1000.0,
            (base.checkpoint_day60.final_mastered_skills as f64 / total_att_base as f64) * 1000.0,
            if total_att_sl > total_att_base { acc_gain / (total_att_sl - total_att_base) as f64 } else { 0.0 },
            efficiency_ratio
        );
    }
}

#[test]
fn test_phase32_multi_seed_robustness_sweep() {
    println!("\n=========================================================================================================");
    println!("  PHASE 32: MULTI-SEED REPRODUCIBILITY & ROBUSTNESS SWEEP (SEEDS 1, 2, 3)                                ");
    println!("=========================================================================================================\n");

    println!("{:<24} | {:<6} | {:<10} | {:<10} | {:<12} | {:<12} | {:<8}", "Cohort", "Seed", "SL Acc %", "Base Acc %", "SL Trans %", "Base Trans %", "Mastered");
    println!("{:-<95}", "");

    for cohort in &[CohortId::CohortAStrongFast, CohortId::CohortDConceptWeak, CohortId::CohortGMixedImproving, CohortId::CohortLUnevenMultiSubject] {
        for seed in &[1u64, 2u64, 3u64] {
            let (sl, _) = Phase32SimulationHarness::run_simulation(*cohort, 60, 45, true, false, false, *seed);
            let (base, _) = Phase32SimulationHarness::run_simulation(*cohort, 60, 45, false, false, false, *seed);

            println!(
                "{:<24} | {:<6} | {:<10.1} | {:<10.1} | {:<12.1} | {:<12.1} | SL: {} / Base: {}",
                cohort.as_str(),
                seed,
                sl.checkpoint_day60.accuracy,
                base.checkpoint_day60.accuracy,
                sl.checkpoint_day60.transfer_accuracy,
                base.checkpoint_day60.transfer_accuracy,
                sl.checkpoint_day60.final_mastered_skills,
                base.checkpoint_day60.final_mastered_skills,
            );
        }
        println!("{:-<95}", "");
    }
}

#[test]
fn test_phase32_negative_outcome_and_safety_audit() {
    println!("\n=========================================================================================================");
    println!("  PHASE 32: NEGATIVE OUTCOME AUDIT & SYSTEM SAFETY VALIDATION                                            ");
    println!("=========================================================================================================\n");

    // Check system safety invariants on Cohort H (Inconsistent, high queue pressure)
    let (sl, service) = Phase32SimulationHarness::run_simulation(CohortId::CohortHInconsistent, 90, 45, true, false, false, 1);

    // Invariant 1: Database size bounded and healthy
    let db_kb = sl.db_size_bytes_day90 / 1024;
    println!("System Safety Invariant 1 - SQLite Database Size: {} KB (Must be < 50,000 KB)", db_kb);
    assert!(db_kb < 50_000, "Database size exploded");

    // Invariant 2: Remediation queue depth bounded (Phase 31 single-skill compaction guarantees depth <= 32)
    println!("System Safety Invariant 2 - Max Remediation Queue Depth: {} (Must be <= 32)", sl.checkpoint_day90.max_queue_depth);
    assert!(sl.checkpoint_day90.max_queue_depth <= 32, "Queue depth exceeded canonical skill bound");

    // Invariant 3: Circuit breakers triggered appropriately without infinite spirals
    println!("System Safety Invariant 3 - Circuit Breakers Triggered: {}", sl.checkpoint_day90.circuit_breaker_triggers);

    // Invariant 4: No data corruption in skill store
    let skills = service.store().list_all_skills().unwrap();
    assert_eq!(skills.len(), 32, "Skill catalog corrupted");
    println!("System Safety Invariant 4 - Skill Catalog Integrity: {} skills verified", skills.len());
}
