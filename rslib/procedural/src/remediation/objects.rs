// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use serde::{Deserialize, Serialize};

use crate::core::{Domain, SchemaId, SkillId};
use crate::diagnostics::ErrorCategory;
use crate::problems::ProblemInstance;
use crate::skills::signals::{IndependenceLevel, MasteryEvidence};

/// An option in a discrete conceptual check.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConceptCheckOption {
    pub id: String,
    pub label: String,
    pub is_correct: bool,
    pub concept_tag: String,
    pub feedback: String,
}

impl ConceptCheckOption {
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        is_correct: bool,
        concept_tag: impl Into<String>,
        feedback: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            is_correct,
            concept_tag: concept_tag.into(),
            feedback: feedback.into(),
        }
    }
}

/// Evaluation result from a learner answering a ConceptCheck.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConceptCheckEvaluation {
    pub is_correct: bool,
    pub chosen_option_id: String,
    pub expected_option_id: String,
    pub concept_tag: Option<String>,
    pub feedback: String,
    pub evidence: MasteryEvidence,
}

/// Micro learning object for evaluating conceptual and schema understanding.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConceptCheckObject {
    pub id: String,
    pub skill_id: SkillId,
    pub schema_id: SchemaId,
    pub domain: Domain,
    pub prompt: String,
    pub context: Option<String>,
    pub options: Vec<ConceptCheckOption>,
    pub expected_option_id: String,
    pub explanation: String,
}

impl ConceptCheckObject {
    pub fn new(
        id: impl Into<String>,
        skill_id: impl Into<SkillId>,
        schema_id: impl Into<SchemaId>,
        domain: Domain,
        prompt: impl Into<String>,
        options: Vec<ConceptCheckOption>,
        expected_option_id: impl Into<String>,
        explanation: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            skill_id: skill_id.into(),
            schema_id: schema_id.into(),
            domain,
            prompt: prompt.into(),
            context: None,
            options,
            expected_option_id: expected_option_id.into(),
            explanation: explanation.into(),
        }
    }

    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    /// Evaluate learner's selection and construct R1 MasteryEvidence.
    pub fn evaluate_choice(&self, chosen_id: &str, latency_ms: u64) -> ConceptCheckEvaluation {
        if let Some(opt) = self.options.iter().find(|o| o.id == chosen_id) {
            let is_correct = opt.is_correct;
            let mut diagnostic_errors = Vec::new();
            if !is_correct {
                diagnostic_errors.push(ErrorCategory::Concept);
            }

            let evidence = MasteryEvidence {
                final_correctness: is_correct,
                decision_quality: Some(if is_correct { 1.0 } else { 0.0 }),
                step_quality: None,
                independence: IndependenceLevel::Independent,
                max_hint_level: None,
                hint_dependence: 0,
                retry_dependence: 0,
                variant_exposure: Some("concept_check".to_string()),
                transfer_evidence: false,
                latency_evidence: latency_ms,
                diagnostic_errors,
            };

            ConceptCheckEvaluation {
                is_correct,
                chosen_option_id: chosen_id.to_string(),
                expected_option_id: self.expected_option_id.clone(),
                concept_tag: Some(opt.concept_tag.clone()),
                feedback: opt.feedback.clone(),
                evidence,
            }
        } else {
            let evidence = MasteryEvidence {
                final_correctness: false,
                decision_quality: Some(0.0),
                step_quality: None,
                independence: IndependenceLevel::Independent,
                max_hint_level: None,
                hint_dependence: 0,
                retry_dependence: 0,
                variant_exposure: Some("concept_check".to_string()),
                transfer_evidence: false,
                latency_evidence: latency_ms,
                diagnostic_errors: vec![ErrorCategory::Concept],
            };

            ConceptCheckEvaluation {
                is_correct: false,
                chosen_option_id: chosen_id.to_string(),
                expected_option_id: self.expected_option_id.clone(),
                concept_tag: None,
                feedback: "Invalid option selected.".to_string(),
                evidence,
            }
        }
    }
}

/// An option in a strategy selection drill.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StrategyOption {
    pub id: String,
    pub label: String,
    pub strategy_tag: String,
    pub is_optimal: bool,
    pub feedback: String,
}

impl StrategyOption {
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        strategy_tag: impl Into<String>,
        is_optimal: bool,
        feedback: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            strategy_tag: strategy_tag.into(),
            is_optimal,
            feedback: feedback.into(),
        }
    }
}

/// Evaluation result from a learner answering a StrategyDrill.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StrategyDrillEvaluation {
    pub is_correct: bool,
    pub chosen_option_id: String,
    pub preferred_option_id: String,
    pub strategy_tag: Option<String>,
    pub feedback: String,
    pub evidence: MasteryEvidence,
}

/// Micro learning object for evaluating strategy and approach selection before solving.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StrategyDrillObject {
    pub id: String,
    pub skill_id: SkillId,
    pub schema_id: SchemaId,
    pub domain: Domain,
    pub prompt: String,
    pub problem_context: String,
    pub options: Vec<StrategyOption>,
    pub preferred_option_id: String,
    pub explanation: String,
}

impl StrategyDrillObject {
    pub fn new(
        id: impl Into<String>,
        skill_id: impl Into<SkillId>,
        schema_id: impl Into<SchemaId>,
        domain: Domain,
        prompt: impl Into<String>,
        problem_context: impl Into<String>,
        options: Vec<StrategyOption>,
        preferred_option_id: impl Into<String>,
        explanation: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            skill_id: skill_id.into(),
            schema_id: schema_id.into(),
            domain,
            prompt: prompt.into(),
            problem_context: problem_context.into(),
            options,
            preferred_option_id: preferred_option_id.into(),
            explanation: explanation.into(),
        }
    }

    /// Evaluate learner's strategy choice and construct R1 MasteryEvidence.
    pub fn evaluate_choice(&self, chosen_id: &str, latency_ms: u64) -> StrategyDrillEvaluation {
        if let Some(opt) = self.options.iter().find(|o| o.id == chosen_id) {
            let is_correct = opt.is_optimal;
            let mut diagnostic_errors = Vec::new();
            if !is_correct {
                diagnostic_errors.push(ErrorCategory::Strategy);
            }

            let evidence = MasteryEvidence {
                final_correctness: is_correct,
                decision_quality: Some(if is_correct { 1.0 } else { 0.0 }),
                step_quality: None,
                independence: IndependenceLevel::Independent,
                max_hint_level: None,
                hint_dependence: 0,
                retry_dependence: 0,
                variant_exposure: Some("strategy_drill".to_string()),
                transfer_evidence: false,
                latency_evidence: latency_ms,
                diagnostic_errors,
            };

            StrategyDrillEvaluation {
                is_correct,
                chosen_option_id: chosen_id.to_string(),
                preferred_option_id: self.preferred_option_id.clone(),
                strategy_tag: Some(opt.strategy_tag.clone()),
                feedback: opt.feedback.clone(),
                evidence,
            }
        } else {
            let evidence = MasteryEvidence {
                final_correctness: false,
                decision_quality: Some(0.0),
                step_quality: None,
                independence: IndependenceLevel::Independent,
                max_hint_level: None,
                hint_dependence: 0,
                retry_dependence: 0,
                variant_exposure: Some("strategy_drill".to_string()),
                transfer_evidence: false,
                latency_evidence: latency_ms,
                diagnostic_errors: vec![ErrorCategory::Strategy],
            };

            StrategyDrillEvaluation {
                is_correct: false,
                chosen_option_id: chosen_id.to_string(),
                preferred_option_id: self.preferred_option_id.clone(),
                strategy_tag: None,
                feedback: "Invalid strategy option selected.".to_string(),
                evidence,
            }
        }
    }
}

/// An option in a representation drill.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RepresentationOption {
    pub id: String,
    pub label: String,
    pub is_correct: bool,
    pub representation_kind: String,
    pub feedback: String,
}

impl RepresentationOption {
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        is_correct: bool,
        representation_kind: impl Into<String>,
        feedback: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            is_correct,
            representation_kind: representation_kind.into(),
            feedback: feedback.into(),
        }
    }
}

/// Evaluation result from a learner answering a RepresentationDrill.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RepresentationDrillEvaluation {
    pub is_correct: bool,
    pub chosen_option_id: String,
    pub expected_option_id: String,
    pub representation_kind: Option<String>,
    pub feedback: String,
    pub evidence: MasteryEvidence,
}

/// Micro learning object for structured diagrammatic or symbolic representation choices.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RepresentationDrillObject {
    pub id: String,
    pub skill_id: SkillId,
    pub schema_id: SchemaId,
    pub domain: Domain,
    pub prompt: String,
    pub options: Vec<RepresentationOption>,
    pub expected_option_id: String,
    pub explanation: String,
}

impl RepresentationDrillObject {
    pub fn new(
        id: impl Into<String>,
        skill_id: impl Into<SkillId>,
        schema_id: impl Into<SchemaId>,
        domain: Domain,
        prompt: impl Into<String>,
        options: Vec<RepresentationOption>,
        expected_option_id: impl Into<String>,
        explanation: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            skill_id: skill_id.into(),
            schema_id: schema_id.into(),
            domain,
            prompt: prompt.into(),
            options,
            expected_option_id: expected_option_id.into(),
            explanation: explanation.into(),
        }
    }

    /// Evaluate learner's representation choice.
    pub fn evaluate_choice(&self, chosen_id: &str, latency_ms: u64) -> RepresentationDrillEvaluation {
        if let Some(opt) = self.options.iter().find(|o| o.id == chosen_id) {
            let is_correct = opt.is_correct;
            let mut diagnostic_errors = Vec::new();
            if !is_correct {
                diagnostic_errors.push(ErrorCategory::Concept);
            }

            let evidence = MasteryEvidence {
                final_correctness: is_correct,
                decision_quality: Some(if is_correct { 1.0 } else { 0.0 }),
                step_quality: None,
                independence: IndependenceLevel::Independent,
                max_hint_level: None,
                hint_dependence: 0,
                retry_dependence: 0,
                variant_exposure: Some("representation_drill".to_string()),
                transfer_evidence: false,
                latency_evidence: latency_ms,
                diagnostic_errors,
            };

            RepresentationDrillEvaluation {
                is_correct,
                chosen_option_id: chosen_id.to_string(),
                expected_option_id: self.expected_option_id.clone(),
                representation_kind: Some(opt.representation_kind.clone()),
                feedback: opt.feedback.clone(),
                evidence,
            }
        } else {
            let evidence = MasteryEvidence {
                final_correctness: false,
                decision_quality: Some(0.0),
                step_quality: None,
                independence: IndependenceLevel::Independent,
                max_hint_level: None,
                hint_dependence: 0,
                retry_dependence: 0,
                variant_exposure: Some("representation_drill".to_string()),
                transfer_evidence: false,
                latency_evidence: latency_ms,
                diagnostic_errors: vec![ErrorCategory::Concept],
            };

            RepresentationDrillEvaluation {
                is_correct: false,
                chosen_option_id: chosen_id.to_string(),
                expected_option_id: self.expected_option_id.clone(),
                representation_kind: None,
                feedback: "Invalid representation option selected.".to_string(),
                evidence,
            }
        }
    }
}

/// Deterministic, structured worked example for conceptual repair.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkedExampleObject {
    pub id: String,
    pub skill_id: SkillId,
    pub schema_id: SchemaId,
    pub domain: Domain,
    pub prompt: String,
    pub problem_context: String,
    pub canonical_steps: Vec<String>,
    pub highlighted_decision_point: String,
    pub method_rationale: String,
    pub common_mistakes_to_avoid: Vec<String>,
}

impl WorkedExampleObject {
    pub fn new(
        id: impl Into<String>,
        skill_id: impl Into<SkillId>,
        schema_id: impl Into<SchemaId>,
        domain: Domain,
        prompt: impl Into<String>,
        problem_context: impl Into<String>,
        canonical_steps: Vec<String>,
        highlighted_decision_point: impl Into<String>,
        method_rationale: impl Into<String>,
        common_mistakes_to_avoid: Vec<String>,
    ) -> Self {
        Self {
            id: id.into(),
            skill_id: skill_id.into(),
            schema_id: schema_id.into(),
            domain,
            prompt: prompt.into(),
            problem_context: problem_context.into(),
            canonical_steps,
            highlighted_decision_point: highlighted_decision_point.into(),
            method_rationale: method_rationale.into(),
            common_mistakes_to_avoid,
        }
    }

    /// Generate viewing evidence. Viewing a worked example provides exposure,
    /// but NEVER falsely awards mastery or correctness.
    pub fn generate_viewing_evidence(&self, view_time_ms: u64) -> MasteryEvidence {
        MasteryEvidence {
            final_correctness: false,
            decision_quality: None,
            step_quality: None,
            independence: IndependenceLevel::NonIndependent,
            max_hint_level: None,
            hint_dependence: 0,
            retry_dependence: 0,
            variant_exposure: Some("worked_example_view".to_string()),
            transfer_evidence: false,
            latency_evidence: view_time_ms,
            diagnostic_errors: Vec::new(),
        }
    }
}

/// Bridge connecting declarative memory gaps to native Anki cards or tags.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeclarativeRecallBridge {
    pub id: String,
    pub skill_id: SkillId,
    pub domain: Domain,
    pub concept_name: String,
    pub prompt_summary: String,
    pub formula_or_fact: String,
    pub target_anki_card_id: Option<i64>,
    pub target_anki_tag: Option<String>,
}

impl DeclarativeRecallBridge {
    pub fn new(
        id: impl Into<String>,
        skill_id: impl Into<SkillId>,
        domain: Domain,
        concept_name: impl Into<String>,
        prompt_summary: impl Into<String>,
        formula_or_fact: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            skill_id: skill_id.into(),
            domain,
            concept_name: concept_name.into(),
            prompt_summary: prompt_summary.into(),
            formula_or_fact: formula_or_fact.into(),
            target_anki_card_id: None,
            target_anki_tag: None,
        }
    }

    pub fn with_card_id(mut self, card_id: i64) -> Self {
        self.target_anki_card_id = Some(card_id);
        self
    }

    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.target_anki_tag = Some(tag.into());
        self
    }
}

/// Executable remediation object recommending and providing foundational prerequisite review.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PrerequisiteReviewObject {
    pub id: String,
    pub target_skill_id: SkillId,
    pub prerequisite_skill_ids: Vec<SkillId>,
    pub domain: Domain,
    pub recommendation_summary: String,
    pub advisory_message: String,
    pub primary_missing_prerequisite: Option<SkillId>,
    pub executable_schema_id: Option<SchemaId>,
    pub executable_problem: Option<ProblemInstance>,
}

impl PrerequisiteReviewObject {
    pub fn new(
        id: impl Into<String>,
        target_skill_id: impl Into<SkillId>,
        prerequisite_skill_ids: Vec<SkillId>,
        domain: Domain,
        recommendation_summary: impl Into<String>,
        advisory_message: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            target_skill_id: target_skill_id.into(),
            prerequisite_skill_ids,
            domain,
            recommendation_summary: recommendation_summary.into(),
            advisory_message: advisory_message.into(),
            primary_missing_prerequisite: None,
            executable_schema_id: None,
            executable_problem: None,
        }
    }

    pub fn with_executable_prerequisite(
        mut self,
        prereq_skill_id: SkillId,
        schema_id: Option<SchemaId>,
        problem: Option<ProblemInstance>,
    ) -> Self {
        self.primary_missing_prerequisite = Some(prereq_skill_id);
        self.executable_schema_id = schema_id;
        self.executable_problem = problem;
        self
    }
}

/// Concrete learning intervention presented to the learner during practice.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "intervention_type", rename_all = "snake_case")]
pub enum RemediationIntervention {
    ConceptCheck(ConceptCheckObject),
    StrategyDrill(StrategyDrillObject),
    RepresentationDrill(RepresentationDrillObject),
    WorkedExample(WorkedExampleObject),
    ProceduralProblem(ProblemInstance),
    DeclarativeRecall(DeclarativeRecallBridge),
    PrerequisiteReview(PrerequisiteReviewObject),
    TransferRetry(ProblemInstance),
}

impl RemediationIntervention {
    pub fn skill_id(&self) -> &SkillId {
        match self {
            RemediationIntervention::ConceptCheck(c) => &c.skill_id,
            RemediationIntervention::StrategyDrill(s) => &s.skill_id,
            RemediationIntervention::RepresentationDrill(r) => &r.skill_id,
            RemediationIntervention::WorkedExample(w) => &w.skill_id,
            RemediationIntervention::ProceduralProblem(_p) => {
                // Return dummy reference or we can store skill_id in metadata if needed
                static DUMMY_SKILL: std::sync::OnceLock<SkillId> = std::sync::OnceLock::new();
                DUMMY_SKILL.get_or_init(|| SkillId::new("unknown"))
            }
            RemediationIntervention::DeclarativeRecall(d) => &d.skill_id,
            RemediationIntervention::PrerequisiteReview(p) => &p.target_skill_id,
            RemediationIntervention::TransferRetry(_) => {
                static DUMMY_SKILL: std::sync::OnceLock<SkillId> = std::sync::OnceLock::new();
                DUMMY_SKILL.get_or_init(|| SkillId::new("unknown"))
            }
        }
    }
}
