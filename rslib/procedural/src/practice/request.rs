// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use serde::{Deserialize, Serialize};

use crate::core::{Domain, ExamProfileId, SchemaId, SkillId};
use crate::exam::ExamPracticeMode;
use crate::scheduling::PracticeMode;

/// Selection scope defining candidate boundary constraints for practice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "scope_type", content = "target", rename_all = "snake_case")]
pub enum PracticeScope {
    /// Practice across all registered domains without domain restriction.
    AllDomains,
    /// Practice constrained strictly to a specific subject domain.
    SingleDomain(Domain),
    /// Practice focused strictly on a single discrete skill.
    SingleSkill(SkillId),
    /// Practice focused strictly on a single practice schema.
    SingleSchema(SchemaId),
    /// Practice restricted to a bounded set of skills.
    MultipleSkills(Vec<SkillId>),
    /// Practice restricted to a bounded set of schemas.
    MultipleSchemas(Vec<SchemaId>),
}

impl Default for PracticeScope {
    fn default() -> Self {
        PracticeScope::AllDomains
    }
}

impl PracticeScope {
    /// Returns true if this scope is an explicit single-target focused mode.
    pub fn is_focused(&self) -> bool {
        matches!(
            self,
            PracticeScope::SingleSkill(_) | PracticeScope::SingleSchema(_)
        )
    }

    /// Check if a skill and its domain fall within this scope.
    pub fn matches_skill(&self, skill_id: &SkillId, domain: &Domain) -> bool {
        match self {
            PracticeScope::AllDomains => true,
            PracticeScope::SingleDomain(d) => d == domain,
            PracticeScope::SingleSkill(s) => s == skill_id,
            PracticeScope::SingleSchema(_) => true, // Evaluated at schema level
            PracticeScope::MultipleSkills(skills) => skills.contains(skill_id),
            PracticeScope::MultipleSchemas(_) => true,
        }
    }

    /// Check if a schema falls within this scope.
    pub fn matches_schema(&self, schema_id: &SchemaId, skill_id: &SkillId, domain: &Domain) -> bool {
        match self {
            PracticeScope::AllDomains => true,
            PracticeScope::SingleDomain(d) => d == domain,
            PracticeScope::SingleSkill(s) => s == skill_id,
            PracticeScope::SingleSchema(sch) => sch == schema_id,
            PracticeScope::MultipleSkills(skills) => skills.contains(skill_id),
            PracticeScope::MultipleSchemas(schemas) => schemas.contains(schema_id),
        }
    }
}

/// Pedagogical objective governing candidate ranking, difficulty, and pacing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PracticeObjective {
    /// Foundational learning with lower difficulty (L1/L2) and introductory variations.
    Learn,
    /// Standard adaptive procedural practice with fluency reinforcement.
    Practice,
    /// Speed and fluency practice with familiar topics and strict latency thresholds.
    Speed,
    /// Rapid diagnostic sweep across topics to detect weaknesses and isolate gaps.
    Diagnose,
    /// Transfer challenge presenting non-obvious, disguised, or cross-domain structures.
    Transfer,
    /// Exam blueprint practice prioritizing high-yield exam weightings and authentic PYQs.
    Exam,
    /// Timed, multi-topic exam section mock simulation.
    Mock,
}

impl Default for PracticeObjective {
    fn default() -> Self {
        PracticeObjective::Practice
    }
}

/// Explicit difficulty constraint for problem instance generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "constraint_type", rename_all = "snake_case")]
pub enum DifficultyConstraint {
    /// Force exact difficulty level (e.g. 1 to 5).
    Exact { level: u32 },
    /// Bound difficulty within an inclusive range.
    Range { min: u32, max: u32 },
    /// Minimum difficulty floor.
    Min { min: u32 },
    /// Maximum difficulty ceiling.
    Max { max: u32 },
}

impl DifficultyConstraint {
    /// Clamps an adaptive level into the constrained bounds.
    pub fn clamp_level(&self, evaluated_level: u32) -> u32 {
        match *self {
            DifficultyConstraint::Exact { level } => level.clamp(1, 5),
            DifficultyConstraint::Range { min, max } => {
                evaluated_level.clamp(min.max(1), max.min(5))
            }
            DifficultyConstraint::Min { min } => evaluated_level.max(min.max(1)).min(5),
            DifficultyConstraint::Max { max } => evaluated_level.min(max.min(5)).max(1),
        }
    }
}

/// Optional execution time constraints for latency targets or session pacing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeConstraint {
    pub target_latency_ms: Option<u64>,
    pub max_session_duration_ms: Option<u64>,
}

impl TimeConstraint {
    pub fn new() -> Self {
        Self {
            target_latency_ms: None,
            max_session_duration_ms: None,
        }
    }

    pub fn with_target_latency_ms(mut self, target_ms: u64) -> Self {
        self.target_latency_ms = Some(target_ms);
        self
    }
}

impl Default for TimeConstraint {
    fn default() -> Self {
        Self::new()
    }
}

/// Bounded practice budget constraining total items or elapsed time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "budget_type", rename_all = "snake_case")]
pub enum SessionBudget {
    ItemCount { max_items: usize },
    TimeLimitMs { max_time_ms: u64 },
    Bounded { max_items: usize, max_time_ms: u64 },
}

/// Policy governing how and when queued remediation interventions may be served.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RemediationPrecedence {
    /// Only critical failures (concept/strategy breakdowns) can trigger remediation.
    CriticalOnly,
    /// All eligible queued remediation actions can be selected according to priority.
    AllEligible,
    /// Remediation actions produce advisory warnings only, without overriding problem selection.
    AdvisoryOnly,
    /// Remediation queue evaluation is bypassed completely.
    Disabled,
}

impl Default for RemediationPrecedence {
    fn default() -> Self {
        RemediationPrecedence::AllEligible
    }
}

/// Unified, composable practice request specifying scope, objective, constraints, and policies.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PracticeRequest {
    pub scope: PracticeScope,
    pub objective: PracticeObjective,
    pub difficulty_constraint: Option<DifficultyConstraint>,
    pub time_constraint: Option<TimeConstraint>,
    pub exam_profile: Option<ExamProfileId>,
    pub remediation_policy: RemediationPrecedence,
    pub session_budget: Option<SessionBudget>,
}

impl Default for PracticeRequest {
    fn default() -> Self {
        Self {
            scope: PracticeScope::AllDomains,
            objective: PracticeObjective::Practice,
            difficulty_constraint: None,
            time_constraint: None,
            exam_profile: None,
            remediation_policy: RemediationPrecedence::AllEligible,
            session_budget: None,
        }
    }
}

impl PracticeRequest {
    pub fn new(scope: PracticeScope, objective: PracticeObjective) -> Self {
        Self {
            scope,
            objective,
            difficulty_constraint: None,
            time_constraint: None,
            exam_profile: None,
            remediation_policy: RemediationPrecedence::AllEligible,
            session_budget: None,
        }
    }

    pub fn with_scope(mut self, scope: PracticeScope) -> Self {
        self.scope = scope;
        self
    }

    pub fn with_objective(mut self, objective: PracticeObjective) -> Self {
        self.objective = objective;
        self
    }

    pub fn with_difficulty_constraint(mut self, constraint: DifficultyConstraint) -> Self {
        self.difficulty_constraint = Some(constraint);
        self
    }

    pub fn with_exact_difficulty(mut self, level: u32) -> Self {
        self.difficulty_constraint = Some(DifficultyConstraint::Exact { level });
        self
    }

    pub fn with_difficulty_range(mut self, min: u32, max: u32) -> Self {
        self.difficulty_constraint = Some(DifficultyConstraint::Range { min, max });
        self
    }

    pub fn with_time_constraint(mut self, constraint: TimeConstraint) -> Self {
        self.time_constraint = Some(constraint);
        self
    }

    pub fn with_target_latency_ms(mut self, target_ms: u64) -> Self {
        let mut tc = self.time_constraint.unwrap_or_default();
        tc.target_latency_ms = Some(target_ms);
        self.time_constraint = Some(tc);
        self
    }

    pub fn with_exam_profile(mut self, profile_id: impl Into<ExamProfileId>) -> Self {
        self.exam_profile = Some(profile_id.into());
        self
    }

    pub fn with_remediation_policy(mut self, policy: RemediationPrecedence) -> Self {
        self.remediation_policy = policy;
        self
    }

    pub fn with_session_budget(mut self, budget: SessionBudget) -> Self {
        self.session_budget = Some(budget);
        self
    }

    /// Map legacy `PracticeMode` into canonical `PracticeRequest`.
    pub fn from_legacy_mode(mode: &PracticeMode) -> Self {
        match mode {
            PracticeMode::MixedMaths => Self::new(
                PracticeScope::SingleDomain(Domain::Mathematics),
                PracticeObjective::Practice,
            ),
            PracticeMode::MixedPhysics => Self::new(
                PracticeScope::SingleDomain(Domain::Physics),
                PracticeObjective::Practice,
            ),
            PracticeMode::MixedChemistry => Self::new(
                PracticeScope::SingleDomain(Domain::Chemistry),
                PracticeObjective::Practice,
            ),
            PracticeMode::MixedReasoning => Self::new(
                PracticeScope::SingleDomain(Domain::Reasoning),
                PracticeObjective::Practice,
            ),
            PracticeMode::MixedInterleaved => {
                Self::new(PracticeScope::AllDomains, PracticeObjective::Practice)
            }
            PracticeMode::FocusedSkill { skill_id } => {
                Self::new(PracticeScope::SingleSkill(skill_id.clone()), PracticeObjective::Practice)
            }
            PracticeMode::FocusedReasoningSkill { skill_id } => {
                Self::new(PracticeScope::SingleSkill(skill_id.clone()), PracticeObjective::Practice)
            }
            PracticeMode::FocusedSchema { schema_id } => {
                Self::new(PracticeScope::SingleSchema(schema_id.clone()), PracticeObjective::Practice)
            }
            PracticeMode::StrategyDrill => Self::new(PracticeScope::AllDomains, PracticeObjective::Diagnose)
                .with_exact_difficulty(2)
                .with_target_latency_ms(15_000),
            PracticeMode::WeakSkills => {
                Self::new(PracticeScope::AllDomains, PracticeObjective::Diagnose)
            }
            PracticeMode::SpeedPractice => Self::new(
                PracticeScope::SingleDomain(Domain::Mathematics),
                PracticeObjective::Speed,
            )
            .with_exact_difficulty(1)
            .with_target_latency_ms(20_000),
            PracticeMode::SpeedReasoning => Self::new(
                PracticeScope::SingleDomain(Domain::Reasoning),
                PracticeObjective::Speed,
            )
            .with_exact_difficulty(1)
            .with_target_latency_ms(18_000),
            PracticeMode::Learning => Self::new(PracticeScope::AllDomains, PracticeObjective::Learn)
                .with_exact_difficulty(1),
            PracticeMode::TransferPractice => {
                Self::new(PracticeScope::AllDomains, PracticeObjective::Transfer)
                    .with_exact_difficulty(5)
            }
            PracticeMode::TransferReasoning => Self::new(
                PracticeScope::SingleDomain(Domain::Reasoning),
                PracticeObjective::Transfer,
            )
            .with_exact_difficulty(5),
            PracticeMode::Diagnostic => {
                Self::new(PracticeScope::AllDomains, PracticeObjective::Diagnose)
            }
            PracticeMode::DiagnosticReasoning => Self::new(
                PracticeScope::SingleDomain(Domain::Reasoning),
                PracticeObjective::Diagnose,
            ),
            PracticeMode::SkillBuilder => {
                Self::new(PracticeScope::AllDomains, PracticeObjective::Practice)
            }
            PracticeMode::ExamLike => Self::new(PracticeScope::AllDomains, PracticeObjective::Exam)
                .with_exact_difficulty(3)
                .with_target_latency_ms(35_000),
        }
    }

    /// Map legacy `ExamPracticeMode` into canonical `PracticeRequest`.
    pub fn from_exam_mode(profile_id: &ExamProfileId, mode: &ExamPracticeMode) -> Self {
        let mut req = Self::new(PracticeScope::AllDomains, PracticeObjective::Exam)
            .with_exam_profile(profile_id.clone())
            .with_target_latency_ms(35_000);

        match mode {
            ExamPracticeMode::ExamPreparation | ExamPracticeMode::MixedExam => req,
            ExamPracticeMode::WeakAreas => {
                req.objective = PracticeObjective::Diagnose;
                req
            }
            ExamPracticeMode::PyqPractice | ExamPracticeMode::PyqAndVariants => req,
            ExamPracticeMode::SpeedTraining => {
                req.objective = PracticeObjective::Speed;
                req.with_target_latency_ms(20_000)
            }
            ExamPracticeMode::Mock => {
                req.objective = PracticeObjective::Mock;
                req.with_target_latency_ms(30_000)
            }
        }
    }
}
