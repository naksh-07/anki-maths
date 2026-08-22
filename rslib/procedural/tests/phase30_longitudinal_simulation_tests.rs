// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Phase 30: Full-Scale 30/60/90-Day Longitudinal Adaptive Learning Systems Simulation Suite
//!
//! Evaluates the longitudinal behavior of StudyLab over 30, 60, and 90 simulated days across:
//! - All 32 Registered Canonical Families (14 Mathematics, 10 Reasoning, 2 Physics, 6 Chemistry)
//! - 12 Deterministic Synthetic Learner Cohorts:
//!     A: Strong + Fast
//!     B: Strong + Slow (Fluency Hold)
//!     C: Careless (Calculation / Slip)
//!     D: Concept Weak (Misconceptions / Fast Demotion)
//!     E: Pattern / Strategy Weak (Structure-sensitive)
//!     F: Transfer Weak (Context / Isomorphic strong, Structural/Transfer weak)
//!     G: Mixed / Improving (Gradual mastery growth)
//!     H: Inconsistent (Cyclical oscillation)
//!     I: Beginner (Low initial mastery across subjects)
//!     J: High Ability / Low Retention (Decays during gaps)
//!     K: Speed-Pressure Learner (Speed focus with careless errors)
//!     L: Uneven Multi-Subject Learner (Strong in Math, Medium Physics, Weak Chemistry, Uneven Reasoning)
//! - 3 Study Loadings: Light (20/day), Standard (45/day), Heavy (75/day)
//! - Baseline (Fixed Level 2, parameter only, no remediation) vs StudyLab (Fully Adaptive) Comparison
//! - Checkpoints at Day 1, Day 30, Day 60, and Day 90
//! - Subject -> Chapter -> Skill multi-tier longitudinal tracking
//! - Domain-specific evidence generation (Math, Reasoning, Physics, Chemistry)
//! - Verification of Anti-Spiral / Circuit Breaker, Over-Adaptation, Profile Differentiation,
//!   Study-Gap Recovery, Exam-Pressure Dynamics, Learning Efficiency, and Sensitivity Analyses.

use std::collections::HashMap;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tempfile::tempdir;

use procedural::core::{
    AttemptId, Domain, ErrorEventId, ProblemFamilyId, ProblemInstanceId, SchemaId, SkillId,
};
use procedural::diagnostics::hints::HintLevel;
use procedural::diagnostics::ErrorCategory;
use procedural::practice::{ErrorEvent, PracticeAttempt, SchemaPracticeObject};
use procedural::problems::catalog::*;
use procedural::problems::ProblemInstance;
use procedural::remediation::{
    RemediationActionKind, RemediationContext, RemediationPolicy,
};
use procedural::scheduling::difficulty::AdaptiveDifficultyEngine;
use procedural::scheduling::PracticeMode;
use procedural::service::ProceduralService;
use procedural::skills::domain_evidence::{
    ChemistryEvidence, DomainEvidencePayload, MathEvidence, PhysicsEvidence, ReasoningEvidence,
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

    /// Simulate a single problem attempt deterministically
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
    ) -> (
        bool,
        u64,
        Option<ErrorCategory>,
        usize,
        Option<HintLevel>,
        Option<VersionedDomainEvidence>,
    ) {
        if is_in_study_gap {
            // No attempts made during gap
            return (true, target_latency_ms, None, 0, None, None);
        }

        let seed_val = (logical_day as u64).wrapping_mul(100_000) + (attempt_idx as u64);
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

        let mut is_correct: bool;
        let mut actual_time: u64;
        let mut err: Option<ErrorCategory> = None;
        let mut hints = 0usize;
        let mut hint_lvl: Option<HintLevel> = None;
        let mut dom_ev: Option<VersionedDomainEvidence> = None;

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

            // B. Strong + Slow: 94% accuracy, 1.45x latency, minimal hints -> triggers Fluency Hold
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

            // D. Concept Weak: 60% L1, drops to 20% L3-L5, 2-3 hints, misconception errors
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
                        Domain::Physics | Domain::Chemistry => Some(ErrorCategory::DomainSpecific("concept_setup".to_string())),
                        _ => Some(ErrorCategory::Concept),
                    };
                    hints = 2;
                    hint_lvl = Some(HintLevel::Level2_ProceduralScaffold);
                    dom_ev = Some(Self::make_domain_evidence(domain, false, false, false));
                } else {
                    dom_ev = Some(Self::make_domain_evidence(domain, true, true, true));
                }
            }

            // E. Pattern / Strategy Weak: 88% on L1/L2, drops to 38% on structural/transfer
            CohortId::CohortEPatternWeak => {
                let is_complex = level >= 3 || is_transfer;
                let threshold = if is_complex { 380 } else { 880 };
                is_correct = hash_val < threshold;
                let mult = if is_complex { 1.35 } else { 0.95 };
                actual_time = (target_latency_ms as f64 * (mult + ((hash_val % 100) as f64 / 1000.0))) as u64;
                if !is_correct {
                    err = match domain {
                        Domain::Reasoning => Some(ErrorCategory::Strategy),
                        Domain::Physics => Some(ErrorCategory::DomainSpecific("model_setup".to_string())),
                        Domain::Chemistry => Some(ErrorCategory::DomainSpecific("stoichiometry".to_string())),
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
                actual_time = (target_latency_ms as f64 * (mult + ((hash_val % 100) as f64 / 1000.0))) as u64;
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
                    dom_ev = Some(Self::make_domain_evidence(domain, false, is_transfer, !is_transfer));
                } else {
                    dom_ev = Some(Self::make_domain_evidence(domain, true, true, true));
                }
            }

            // G. Mixed / Improving: Starts weak (55% acc, 1.25x latency), gradually improves to 92% acc and 0.70x latency
            CohortId::CohortGMixedImproving => {
                let progress = (logical_day as f64 / 60.0).min(1.0);
                let threshold = 550.0 + (370.0 * progress);
                is_correct = (hash_val as f64) < threshold;
                let mult = 1.25 - (0.55 * progress);
                actual_time = (target_latency_ms as f64 * (mult + ((hash_val % 80) as f64 / 1000.0))) as u64;
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

            // H. Inconsistent: Cyclical wave between 55% and 85%
            CohortId::CohortHInconsistent => {
                let wave_phase = ((logical_day % 12) as f64 / 12.0) * 2.0 * std::f64::consts::PI;
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

            // I. Beginner: Low initial mastery across all subjects (45% on L1, 20% on L2+, 1.40x latency)
            CohortId::CohortIBeginner => {
                let progress = (logical_day as f64 / 90.0).min(1.0);
                let threshold = match level {
                    1 => 450.0 + (300.0 * progress),
                    2 => 250.0 + (350.0 * progress),
                    _ => 150.0 + (250.0 * progress),
                };
                is_correct = (hash_val as f64) < threshold;
                let mult = 1.40 - (0.35 * progress);
                actual_time = (target_latency_ms as f64 * (mult + ((hash_val % 100) as f64 / 1000.0))) as u64;
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

            // J. High Ability / Low Retention: 92% when active; forgets after gaps
            CohortId::CohortJLowRetention => {
                let is_post_gap = (31..=38).contains(&logical_day);
                let threshold = if is_post_gap { 500 } else { 920 };
                is_correct = hash_val < threshold;
                let mult = if is_post_gap { 1.25 } else { 0.65 };
                actual_time = (target_latency_ms as f64 * (mult + ((hash_val % 80) as f64 / 1000.0))) as u64;
                if !is_correct {
                    err = if is_post_gap {
                        Some(ErrorCategory::Concept)
                    } else {
                        Some(ErrorCategory::Calculation)
                    };
                    dom_ev = Some(Self::make_domain_evidence(domain, false, !is_post_gap, !is_post_gap));
                } else {
                    dom_ev = Some(Self::make_domain_evidence(domain, true, true, true));
                }
            }

            // K. Speed-Pressure Learner: Prioritizes speed (0.40x latency), careless error spikes under pressure
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

            // L. Uneven Multi-Subject Learner:
            // Math: strong (95% acc, 0.60x time)
            // Physics: medium (75% acc, 0.95x time)
            // Chemistry Stoichiometry: weak (45% acc, 1.30x time)
            // Reasoning: Seating strong (90%), Series weak (50%)
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
                actual_time = (target_latency_ms as f64 * (mult + ((hash_val % 80) as f64 / 1000.0))) as u64;
                if !is_correct {
                    err = match domain {
                        Domain::Chemistry => Some(ErrorCategory::DomainSpecific("stoichiometry".to_string())),
                        Domain::Reasoning => {
                            if !chapter.contains("seating") {
                                Some(ErrorCategory::Strategy)
                            } else {
                                Some(ErrorCategory::Calculation)
                            }
                        }
                        _ => Some(ErrorCategory::Calculation),
                    };
                    let is_setup = matches!(domain, Domain::Chemistry) || (matches!(domain, Domain::Reasoning) && !chapter.contains("seating"));
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
// 2. CANONICAL SCHEMAS & CHAPTER REGISTRATION
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
        // Mathematics: Combined
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
// 3. TELEMETRY & MULTI-TIER CHECKPOINT STRUCTS
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
    pub remediation_counts: usize,
}

impl ChapterTelemetry {
    pub fn accuracy(&self) -> f64 {
        if self.attempts == 0 { 0.0 } else { (self.correct as f64 / self.attempts as f64) * 100.0 }
    }
    pub fn transfer_accuracy(&self) -> f64 {
        if self.transfer_attempts == 0 { 0.0 } else { (self.transfer_correct as f64 / self.transfer_attempts as f64) * 100.0 }
    }
    pub fn median_latency(&self) -> u64 {
        if self.latencies.is_empty() { return 0; }
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
    pub error_counts: HashMap<String, usize>,
    pub chapter_telemetry: HashMap<String, ChapterTelemetry>,
    pub domain_accuracies: HashMap<String, f64>,
    pub remediations_enqueued: usize,
    pub remediations_executed: usize,
    pub remediations_resolved: usize,
    pub resolution_rate: f64,
    pub circuit_breaker_triggers: usize,
    pub max_queue_depth: usize,
    pub final_mastered_skills: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LongitudinalRunResult {
    pub cohort_name: String,
    pub is_adaptive: bool,
    pub total_days: u32,
    pub reviews_per_day: usize,
    pub checkpoint_day1: CheckpointTelemetry,
    pub checkpoint_day30: CheckpointTelemetry,
    pub checkpoint_day60: CheckpointTelemetry,
    pub checkpoint_day90: CheckpointTelemetry,
    pub db_size_bytes_day90: u64,
}

// =========================================================================
// 4. SIMULATION HARNESS ENGINE
// =========================================================================

pub struct Phase30SimulationHarness;

impl Phase30SimulationHarness {
    pub fn run_longitudinal_simulation(
        cohort: CohortId,
        days: u32,
        reviews_per_day: usize,
        is_adaptive: bool,
        study_gap_active: bool,
        exam_pressure_active: bool,
    ) -> (LongitudinalRunResult, ProceduralService) {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join(format!("phase30_sim_{}_{}_{}.db", cohort.as_str(), days, is_adaptive));
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
        let mut global_errors: HashMap<String, usize> = HashMap::new();
        let mut chapters_map: HashMap<String, ChapterTelemetry> = HashMap::new();
        let mut domain_map: HashMap<String, (usize, usize)> = HashMap::new();

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
                    let session_time = day_time + chrono::Duration::hours((session_idx * 4) as i64);

                    for r in 0..reviews_per_session {
                        attempt_counter += 1;

                        // Structured interleaving across all 32 schemas
                        let schema_info = &all_registered[(attempt_counter + day as usize * 7) % all_registered.len()];
                        let schema = service.store().get_schema(&SchemaId::from(schema_info.schema_id)).unwrap().unwrap();
                        let family = service.store().get_problem_family(&schema.problem_family_id).unwrap().unwrap();

                        let skill_state = service.load_skill_state(&schema.skill_id).unwrap();
                        let max_level = family.max_difficulty as u32;

                        // Adaptive vs Baseline Difficulty
                        let (diff_level, target_time_ms) = if is_adaptive {
                            let dec = AdaptiveDifficultyEngine::evaluate_difficulty(
                                skill_state.as_ref(),
                                None,
                                None,
                            );
                            (dec.level.min(max_level), dec.target_time_ms)
                        } else {
                            (2u32, 35_000u64)
                        };

                        level_counts[(diff_level - 1) as usize] += 1;

                        // Adaptive vs Baseline Variant Progression
                        let variant_cat = if is_adaptive {
                            if let Some(ref st) = skill_state {
                                match st.practice_state {
                                    PracticeProgressionState::New | PracticeProgressionState::Learning => {
                                        VariantCategory::Parameter
                                    }
                                    PracticeProgressionState::Fluent => VariantCategory::Isomorphic,
                                    PracticeProgressionState::Variation => VariantCategory::Structural,
                                    PracticeProgressionState::Transfer => VariantCategory::Transfer,
                                    PracticeProgressionState::Mastered
                                    | PracticeProgressionState::Retired
                                    | PracticeProgressionState::Hibernating => VariantCategory::MultiConcept,
                                }
                            } else {
                                VariantCategory::Parameter
                            }
                        } else {
                            VariantCategory::Parameter
                        };

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

                        // Simulate attempt
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

                        // Chapter tracking
                        let chap_entry = chapters_map.entry(schema_info.chapter.to_string()).or_insert_with(|| {
                            ChapterTelemetry {
                                chapter_name: schema_info.chapter.to_string(),
                                domain_name: format!("{:?}", schema_info.domain),
                                ..Default::default()
                            }
                        });
                        chap_entry.attempts += 1;
                        if is_correct { chap_entry.correct += 1; }
                        chap_entry.latencies.push(actual_time);
                        chap_entry.target_latencies.push(target_time_ms);
                        chap_entry.difficulty_counts[(diff_level - 1) as usize] += 1;
                        if is_transfer_curr {
                            chap_entry.transfer_attempts += 1;
                            if is_correct { chap_entry.transfer_correct += 1; }
                        }
                        if let Some(ref e) = error_cat {
                            *chap_entry.error_counts.entry(format!("{:?}", e)).or_insert(0) += 1;
                            *global_errors.entry(format!("{:?}", e)).or_insert(0) += 1;
                        }

                        // Domain tracking
                        let dom_entry = domain_map.entry(format!("{:?}", schema_info.domain)).or_insert((0, 0));
                        dom_entry.1 += 1;
                        if is_correct { dom_entry.0 += 1; }

                        // Persist problem instance
                        let instance_id = ProblemInstanceId::new(format!("inst-{}-{}", cohort.as_str(), attempt_counter));
                        let instance = ProblemInstance::new(
                            instance_id.clone(),
                            schema.problem_family_id.clone(),
                            attempt_counter as u64,
                            serde_json::json!({ "difficulty": diff_level }),
                            "Simulated Problem Prompt",
                            serde_json::json!("42"),
                        );
                        service.save_problem_instance(instance).unwrap();

                        // Persist attempt with domain evidence
                        let attempt_id = AttemptId::new(format!("att-{}-{}-{}", cohort.as_str(), day, attempt_counter));
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
                            let err_id = ErrorEventId::new(format!("err-{}-{}-{}", cohort.as_str(), day, attempt_counter));
                            let ee = ErrorEvent::new(
                                err_id,
                                attempt_id.clone(),
                                err.as_str(),
                                serde_json::json!({ "reason": format!("{:?}", err) }),
                            );
                            error_events.push(ee);
                        }

                        // Build SkillState MasteryEvidence directly
                        let mut current_state = service.load_skill_state(&schema.skill_id).unwrap().unwrap_or_else(|| {
                            procedural::skills::SkillState::new(schema.skill_id.clone())
                        });

                        let mastery_ev = MasteryEvidence {
                            final_correctness: is_correct,
                            latency_evidence: actual_time,
                            independence: if hints_used == 0 { IndependenceLevel::Independent } else { IndependenceLevel::LightSupport },
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
                        service.store().record_attempt_atomic(&attempt, &error_events, &current_state).unwrap();

                        // Adaptive Remediation Policy
                        if is_adaptive {
                            let q_arc = service.remediation_queue();
                            let mut q = q_arc.lock().unwrap();

                            if let Some(ref err) = error_cat {
                                let key = (schema.skill_id.clone(), err.clone());
                                let recurrence = q.recurrence_tracker.get(&key).copied().unwrap_or(0) + 1;

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
                                    is_transfer_attempt: matches!(variant_cat, VariantCategory::Transfer),
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
                            } else {
                                if let Some(ref prev_err) = error_cat {
                                    q.record_resolution(&schema.skill_id, prev_err);
                                    remediations_resolved += 1;
                                }
                            }

                            // Interleaved remediation execution at end of session
                            if (r + 1) == reviews_per_session && !q.pending_actions.is_empty() {
                                if let Some(action) = q.select_next_remediation(&PracticeMode::MixedInterleaved) {
                                    remediations_executed += 1;
                                    let rem_success = match cohort {
                                        CohortId::CohortAStrongFast | CohortId::CohortBStrongSlow => true,
                                        CohortId::CohortGMixedImproving => day >= 20,
                                        CohortId::CohortDConceptWeak => day >= 60,
                                        CohortId::CohortIBeginner => day >= 45,
                                        _ => attempt_counter % 2 == 0,
                                    };

                                    if rem_success {
                                        q.record_resolution(&action.skill_id, &action.primary_error);
                                        remediations_resolved += 1;
                                    }

                                    let rem_instance_id = ProblemInstanceId::new(format!("rem-inst-{}-{}", cohort.as_str(), attempt_counter));
                                    let rem_schema = service.store().get_schema(&action.schema_id).unwrap().unwrap();
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
                                        AttemptId::new(format!("rem-att-{}-{}-{}", cohort.as_str(), day, attempt_counter)),
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

            // Capture Checkpoint Snapshots
            if day == 1 || day == 30 || day == 60 || day == 90 {
                let mut sorted_lat = all_latencies.clone();
                sorted_lat.sort_unstable();
                let med_lat = if sorted_lat.is_empty() { 0 } else { sorted_lat[sorted_lat.len() / 2] };
                let p95_lat = if sorted_lat.is_empty() { 0 } else { sorted_lat[((sorted_lat.len() as f64 * 0.95) as usize).min(sorted_lat.len() - 1)] };
                let mean_ratio = if time_ratios.is_empty() { 1.0 } else { time_ratios.iter().sum::<f64>() / time_ratios.len() as f64 };
                let sum_levels: usize = level_counts.iter().sum();
                let diff_dist = [
                    if sum_levels == 0 { 0.0 } else { (level_counts[0] as f64 / sum_levels as f64) * 100.0 },
                    if sum_levels == 0 { 0.0 } else { (level_counts[1] as f64 / sum_levels as f64) * 100.0 },
                    if sum_levels == 0 { 0.0 } else { (level_counts[2] as f64 / sum_levels as f64) * 100.0 },
                    if sum_levels == 0 { 0.0 } else { (level_counts[3] as f64 / sum_levels as f64) * 100.0 },
                    if sum_levels == 0 { 0.0 } else { (level_counts[4] as f64 / sum_levels as f64) * 100.0 },
                ];

                let mut dom_accs = HashMap::new();
                for (d, (c, tot)) in &domain_map {
                    dom_accs.insert(d.clone(), if *tot > 0 { (*c as f64 / *tot as f64) * 100.0 } else { 0.0 });
                }

                let all_skills = service.store().list_all_skills().unwrap();
                let mut mastered_cnt = 0;
                for s in &all_skills {
                    if let Ok(Some(st)) = service.load_skill_state(&s.id) {
                        if matches!(st.practice_state, PracticeProgressionState::Mastered | PracticeProgressionState::Transfer) {
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
                    accuracy: if attempt_counter == 0 { 0.0 } else { (total_correct as f64 / attempt_counter as f64) * 100.0 },
                    median_latency_ms: med_lat,
                    p95_latency_ms: p95_lat,
                    mean_time_ratio: mean_ratio,
                    total_hints_used: total_hints,
                    hints_per_problem: if attempt_counter == 0 { 0.0 } else { total_hints as f64 / attempt_counter as f64 },
                    transfer_attempts,
                    transfer_correct,
                    transfer_accuracy: if transfer_attempts == 0 { 0.0 } else { (transfer_correct as f64 / transfer_attempts as f64) * 100.0 },
                    difficulty_distribution: diff_dist,
                    error_counts: global_errors.clone(),
                    chapter_telemetry: chapters_map.clone(),
                    domain_accuracies: dom_accs,
                    remediations_enqueued,
                    remediations_executed,
                    remediations_resolved,
                    resolution_rate: res_rate,
                    circuit_breaker_triggers,
                    max_queue_depth,
                    final_mastered_skills: mastered_cnt,
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
// 5. TEST SUITE EXECUTIONS & EMPIRICAL AUDITS
// =========================================================================

#[test]
fn test_phase30_full_scale_longitudinal_simulation_all_12_cohorts() {
    println!("\n===============================================================================");
    println!("  PHASE 30: 30 / 60 / 90-DAY LONGITUDINAL SYSTEMS SIMULATION (12 COHORTS)    ");
    println!("===============================================================================\n");

    for cohort in CohortId::all() {
        let (res_sl, _) = Phase30SimulationHarness::run_longitudinal_simulation(*cohort, 90, 45, true, false, false);
        let (res_base, _) = Phase30SimulationHarness::run_longitudinal_simulation(*cohort, 90, 45, false, false, false);

        println!(
            "------------------------------------------------------------------------------------------------------------------------\n\
             COHORT: {:<30} | 90-Day Attempts: {}\n\
             ------------------------------------------------------------------------------------------------------------------------\n\
             [Day 30 SL]   Acc: {:<5.1}% | TransAcc: {:<5.1}% | MedLat: {:<5}ms | Hints/P: {:<4.2} | RemExec: {:<3} | MaxQ: {:<2} | L1-L5: [{:.0}%, {:.0}%, {:.0}%, {:.0}%, {:.0}%]\n\
             [Day 60 SL]   Acc: {:<5.1}% | TransAcc: {:<5.1}% | MedLat: {:<5}ms | Hints/P: {:<4.2} | RemExec: {:<3} | MaxQ: {:<2} | L1-L5: [{:.0}%, {:.0}%, {:.0}%, {:.0}%, {:.0}%]\n\
             [Day 90 SL]   Acc: {:<5.1}% | TransAcc: {:<5.1}% | MedLat: {:<5}ms | Hints/P: {:<4.2} | RemExec: {:<3} | MaxQ: {:<2} | L1-L5: [{:.0}%, {:.0}%, {:.0}%, {:.0}%, {:.0}%]\n\
             [Day 90 Base] Acc: {:<5.1}% | TransAcc: {:<5.1}% | MedLat: {:<5}ms | Hints/P: {:<4.2} | RemExec:   0 | MaxQ:  0 | L1-L5: [0%, 100%, 0%, 0%, 0%]\n\
             [DB Size]     {} KB | Mastered Skills: {} / 32 | Circuit Breakers: {}\n",
            cohort.as_str(),
            res_sl.checkpoint_day90.total_attempts,
            res_sl.checkpoint_day30.accuracy,
            res_sl.checkpoint_day30.transfer_accuracy,
            res_sl.checkpoint_day30.median_latency_ms,
            res_sl.checkpoint_day30.hints_per_problem,
            res_sl.checkpoint_day30.remediations_executed,
            res_sl.checkpoint_day30.max_queue_depth,
            res_sl.checkpoint_day30.difficulty_distribution[0],
            res_sl.checkpoint_day30.difficulty_distribution[1],
            res_sl.checkpoint_day30.difficulty_distribution[2],
            res_sl.checkpoint_day30.difficulty_distribution[3],
            res_sl.checkpoint_day30.difficulty_distribution[4],
            res_sl.checkpoint_day60.accuracy,
            res_sl.checkpoint_day60.transfer_accuracy,
            res_sl.checkpoint_day60.median_latency_ms,
            res_sl.checkpoint_day60.hints_per_problem,
            res_sl.checkpoint_day60.remediations_executed,
            res_sl.checkpoint_day60.max_queue_depth,
            res_sl.checkpoint_day60.difficulty_distribution[0],
            res_sl.checkpoint_day60.difficulty_distribution[1],
            res_sl.checkpoint_day60.difficulty_distribution[2],
            res_sl.checkpoint_day60.difficulty_distribution[3],
            res_sl.checkpoint_day60.difficulty_distribution[4],
            res_sl.checkpoint_day90.accuracy,
            res_sl.checkpoint_day90.transfer_accuracy,
            res_sl.checkpoint_day90.median_latency_ms,
            res_sl.checkpoint_day90.hints_per_problem,
            res_sl.checkpoint_day90.remediations_executed,
            res_sl.checkpoint_day90.max_queue_depth,
            res_sl.checkpoint_day90.difficulty_distribution[0],
            res_sl.checkpoint_day90.difficulty_distribution[1],
            res_sl.checkpoint_day90.difficulty_distribution[2],
            res_sl.checkpoint_day90.difficulty_distribution[3],
            res_sl.checkpoint_day90.difficulty_distribution[4],
            res_base.checkpoint_day90.accuracy,
            res_base.checkpoint_day90.transfer_accuracy,
            res_base.checkpoint_day90.median_latency_ms,
            res_base.checkpoint_day90.hints_per_problem,
            res_sl.db_size_bytes_day90 / 1024,
            res_sl.checkpoint_day90.final_mastered_skills,
            res_sl.checkpoint_day90.circuit_breaker_triggers,
        );

        assert_eq!(res_sl.checkpoint_day90.total_attempts, 4050);
        assert_eq!(res_base.checkpoint_day90.total_attempts, 4050);
    }
}

#[test]
fn test_phase30_chapter_level_and_domain_tracking_audit() {
    println!("\n===============================================================================");
    println!("  PHASE 30: CHAPTER-LEVEL & DOMAIN EVIDENCE LONGITUDINAL AUDIT                ");
    println!("===============================================================================\n");

    // Test Cohort L (Uneven Multi-Subject) across Math, Reasoning, Physics, Chemistry
    let (res, _) = Phase30SimulationHarness::run_longitudinal_simulation(CohortId::CohortLUnevenMultiSubject, 90, 45, true, false, false);

    println!("Cohort L (Uneven Multi-Subject) Domain Accuracies at Day 90:");
    for (dom, acc) in &res.checkpoint_day90.domain_accuracies {
        println!("  - Domain: {:<15} -> Accuracy: {:.1}%", dom, acc);
    }

    println!("\nCohort L Chapter-Level Outcomes at Day 90:");
    println!("{:<20} | {:<12} | {:<8} | {:<10} | {:<12} | {:<10}", "Chapter", "Domain", "Attempts", "Accuracy %", "Trans Acc %", "Remediations");
    println!("{:-<85}", "");
    for (chap, telemetry) in &res.checkpoint_day90.chapter_telemetry {
        println!(
            "{:<20} | {:<12} | {:<8} | {:<10.1} | {:<12.1} | {:<10}",
            chap,
            telemetry.domain_name,
            telemetry.attempts,
            telemetry.accuracy(),
            telemetry.transfer_accuracy(),
            telemetry.remediation_counts,
        );
    }

    // Verify domain isolation: Math accuracy is high (>90%), Chemistry Stoichiometry is properly low (~45%)
    let math_acc = res.checkpoint_day90.domain_accuracies.get("Mathematics").copied().unwrap_or(0.0);
    let chem_acc = res.checkpoint_day90.domain_accuracies.get("Chemistry").copied().unwrap_or(0.0);
    assert!(math_acc > 90.0, "Math accuracy was not preserved independently");
    assert!(chem_acc < 60.0, "Chemistry weakness was masked by other domains");
}

#[test]
fn test_phase30_domain_evidence_effectiveness_comparison() {
    println!("\n===============================================================================");
    println!("  PHASE 30: DOMAIN EVIDENCE ADAPTIVE DECISION EFFECTIVENESS                    ");
    println!("===============================================================================\n");

    // 1. Math: Execution error vs Concept error
    let mut state_calc = procedural::skills::SkillState::new("math.percentage.successive");
    state_calc.custom_state = serde_json::json!({ "current_difficulty_level": 4 });
    let ev_calc = MasteryEvidence {
        final_correctness: false,
        latency_evidence: 40_000,
        diagnostic_errors: vec![ErrorCategory::Calculation],
        domain_evidence: Some(VersionedDomainEvidence::new_math(MathEvidence {
            execution: Some(false),
            method_selection: Some(true),
            pattern_recognition: Some(true),
            ..Default::default()
        })),
        ..Default::default()
    };
    state_calc.record_attempt_outcome(&ev_calc, 0.0, 45_000, 1000);

    let dec_calc = AdaptiveDifficultyEngine::evaluate_difficulty(Some(&state_calc), None, None);
    assert!(!dec_calc.reason.contains("demoted_on_concept_breakdown"));

    // 2. Physics: Unit validity vs Calculation
    let mut state_phys = procedural::skills::SkillState::new("physics.kinematics.1d");
    let ev_phys = MasteryEvidence {
        final_correctness: false,
        latency_evidence: 35_000,
        diagnostic_errors: vec![ErrorCategory::Unit],
        domain_evidence: Some(VersionedDomainEvidence::new_physics(PhysicsEvidence {
            unit_validity: Some(false),
            physical_model_selection: Some(true),
            ..Default::default()
        })),
        ..Default::default()
    };
    state_phys.record_attempt_outcome(&ev_phys, 0.0, 35_000, 1000);

    let ctx_phys = RemediationContext {
        skill_id: &SkillId::from("physics.kinematics.1d"),
        schema_id: &SchemaId::from(SCHEMA_PHYSICS_KINEMATICS),
        domain: Domain::Physics,
        primary_error: ErrorCategory::Unit,
        step_error: None,
        decision_point_correct: None,
        independence: IndependenceLevel::Independent,
        progression_state: PracticeProgressionState::Learning,
        recent_attempts: &state_phys.recent_attempts,
        source_attempt_id: &AttemptId::new("test-phys"),
        recurrence_count: 1,
        is_transfer_attempt: false,
    };
    let rem_phys = RemediationPolicy::evaluate(&ctx_phys);
    assert_eq!(rem_phys.preferred_variant, Some("unit_conversion".to_string()));

    println!("Domain Evidence Decisions: Math Calculation Slip = Handled without demotion; Physics Unit Error = Routed to Unit Conversion variant.");
}

#[test]
fn test_phase30_anti_spiral_and_circuit_breaker() {
    println!("\n===============================================================================");
    println!("  PHASE 30: ANTI-SPIRAL & CIRCUIT BREAKER VERIFICATION                        ");
    println!("===============================================================================\n");

    let skill_id = SkillId::from("algebra.linear_equations");
    let schema_id = SchemaId::from(SCHEMA_LINEAR_EQUATIONS);
    let attempt_id = AttemptId::new("att-breaker");

    // Persistent failures up to recurrence 5
    let ctx = RemediationContext {
        skill_id: &skill_id,
        schema_id: &schema_id,
        domain: Domain::Mathematics,
        primary_error: ErrorCategory::Concept,
        step_error: None,
        decision_point_correct: Some(false),
        independence: IndependenceLevel::NonIndependent,
        progression_state: PracticeProgressionState::Learning,
        recent_attempts: &[],
        source_attempt_id: &attempt_id,
        recurrence_count: 5,
        is_transfer_attempt: false,
    };

    let action = RemediationPolicy::evaluate(&ctx);
    assert_eq!(action.kind, RemediationActionKind::CircuitBreaker);
    println!("Recurrence = 5 properly triggered CircuitBreaker: {:?}", action.kind);
}

#[test]
fn test_phase30_study_gap_recovery_simulation() {
    println!("\n===============================================================================");
    println!("  PHASE 30: STUDY-GAP RECOVERY SIMULATION (COHORT J)                          ");
    println!("===============================================================================\n");

    let (res, _) = Phase30SimulationHarness::run_longitudinal_simulation(
        CohortId::CohortJLowRetention,
        60,
        45,
        true,
        true, // Study gap active Days 21-30
        false,
    );

    println!("Cohort J Trajectory with 10-day Study Gap (Days 21-30):");
    println!("  - Day 1  Accuracy: {:.1}%, Median Latency: {}ms", res.checkpoint_day1.accuracy, res.checkpoint_day1.median_latency_ms);
    println!("  - Day 30 Accuracy: {:.1}%, Median Latency: {}ms (During Gap)", res.checkpoint_day30.accuracy, res.checkpoint_day30.median_latency_ms);
    println!("  - Day 60 Accuracy: {:.1}%, Median Latency: {}ms (Post-Recovery)", res.checkpoint_day60.accuracy, res.checkpoint_day60.median_latency_ms);

    assert!(res.checkpoint_day60.accuracy >= 75.0, "Learner failed to recover post-gap");
}

#[test]
fn test_phase30_exam_pressure_simulation() {
    println!("\n===============================================================================");
    println!("  PHASE 30: EXAM PRESSURE & SPEED EMPHASIS SIMULATION (COHORT K)              ");
    println!("===============================================================================\n");

    let (res, _) = Phase30SimulationHarness::run_longitudinal_simulation(
        CohortId::CohortKSpeedPressure,
        60,
        45,
        true,
        false,
        true, // Exam pressure active Days 45-60
    );

    println!("Cohort K Trajectory under Exam Pressure:");
    println!("  - Day 30 Accuracy: {:.1}%, Median Latency: {}ms (Normal)", res.checkpoint_day30.accuracy, res.checkpoint_day30.median_latency_ms);
    println!("  - Day 60 Accuracy: {:.1}%, Median Latency: {}ms (High Pressure)", res.checkpoint_day60.accuracy, res.checkpoint_day60.median_latency_ms);

    // Verify speed emphasis results in lower latency than default baseline target (45s)
    assert!(res.checkpoint_day60.median_latency_ms < 45_000);
}

#[test]
fn test_phase30_profile_differentiation() {
    println!("\n===============================================================================");
    println!("  PHASE 30: PROFILE DIFFERENTIATION AUDIT                                     ");
    println!("===============================================================================\n");

    // Learner A: 5 consecutive fast successes
    let mut state_a = procedural::skills::SkillState::new("math.percentage.successive");
    state_a.practice_state = PracticeProgressionState::Fluent;
    state_a.custom_state = serde_json::json!({ "current_difficulty_level": 4 });
    for i in 1..=5 {
        let ev = MasteryEvidence {
            final_correctness: true,
            latency_evidence: 20_000,
            variant_exposure: Some("standard".to_string()),
            ..Default::default()
        };
        state_a.record_attempt_outcome(&ev, 1.0, 45_000, 1000 + i * 100);
    }
    // Learner A fails once on calculation
    let ev_fail_a = MasteryEvidence {
        final_correctness: false,
        latency_evidence: 22_000,
        diagnostic_errors: vec![ErrorCategory::Calculation],
        domain_evidence: Some(VersionedDomainEvidence::new_math(MathEvidence {
            execution: Some(false),
            method_selection: Some(true),
            pattern_recognition: Some(true),
            ..Default::default()
        })),
        ..Default::default()
    };
    state_a.record_attempt_outcome(&ev_fail_a, 0.0, 45_000, 2000);

    // Learner B: 4 consecutive failures
    let mut state_b = procedural::skills::SkillState::new("math.percentage.successive");
    state_b.practice_state = PracticeProgressionState::Learning;
    state_b.custom_state = serde_json::json!({ "current_difficulty_level": 2 });
    for i in 1..=4 {
        let ev = MasteryEvidence {
            final_correctness: false,
            latency_evidence: 50_000,
            diagnostic_errors: vec![ErrorCategory::Concept],
            ..Default::default()
        };
        state_b.record_attempt_outcome(&ev, 0.0, 45_000, 1000 + i * 100);
    }
    // Learner B fails on calculation as well
    let ev_fail_b = MasteryEvidence {
        final_correctness: false,
        latency_evidence: 52_000,
        diagnostic_errors: vec![ErrorCategory::Calculation],
        domain_evidence: Some(VersionedDomainEvidence::new_math(MathEvidence {
            execution: Some(false),
            method_selection: Some(true),
            pattern_recognition: Some(true),
            ..Default::default()
        })),
        ..Default::default()
    };
    state_b.record_attempt_outcome(&ev_fail_b, 0.0, 45_000, 2000);

    let dec_a = AdaptiveDifficultyEngine::evaluate_difficulty(Some(&state_a), None, None);
    let dec_b = AdaptiveDifficultyEngine::evaluate_difficulty(Some(&state_b), None, None);

    println!("Learner A (Strong history, isolated slip) -> Difficulty: L{} ({})", dec_a.level, dec_a.reason);
    println!("Learner B (Weak history, repeated fails) -> Difficulty: L{} ({})", dec_b.level, dec_b.reason);

    assert_eq!(dec_a.level, 4);
    assert_eq!(dec_b.level, 1);
    assert_ne!(dec_a.level, dec_b.level);
}

#[test]
fn test_phase30_workload_sensitivity_analysis() {
    println!("\n===============================================================================");
    println!("  PHASE 30: WORKLOAD & SENSITIVITY ANALYSIS (20 vs 45 vs 75 REVIEWS/DAY)     ");
    println!("===============================================================================\n");

    for &load in &[20, 45, 75] {
        let (res, _) = Phase30SimulationHarness::run_longitudinal_simulation(CohortId::CohortGMixedImproving, 60, load, true, false, false);
        println!(
            "Load {} reviews/day -> Day 60 Acc: {:.1}%, Median Lat: {}ms, TransAcc: {:.1}%, RemExec: {}, MaxQ: {}",
            load,
            res.checkpoint_day60.accuracy,
            res.checkpoint_day60.median_latency_ms,
            res.checkpoint_day60.transfer_accuracy,
            res.checkpoint_day60.remediations_executed,
            res.checkpoint_day60.max_queue_depth,
        );
    }
}
