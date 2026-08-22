// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use serde::{Deserialize, Serialize};

/// Diagnostic evidence for Mathematics problems.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct MathEvidence {
    pub pattern_recognition: Option<bool>,
    pub method_selection: Option<bool>,
    pub execution: Option<bool>,
    pub verification: Option<bool>,
    pub structural_transfer: Option<bool>,
}

/// Diagnostic evidence for Reasoning problems.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ReasoningEvidence {
    pub pattern_recognition: Option<bool>,
    pub representation: Option<bool>,
    pub constraint_extraction: Option<bool>,
    pub decision_path: Option<bool>,
    pub deduction: Option<bool>,
    pub trap_checking: Option<bool>,
    pub structural_transfer: Option<bool>,
}

/// Diagnostic evidence for Physics problems.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct PhysicsEvidence {
    pub physical_model_selection: Option<bool>,
    pub representation: Option<bool>,
    pub governing_principle: Option<bool>,
    pub equation_setup: Option<bool>,
    pub calculation: Option<bool>,
    pub unit_validity: Option<bool>,
    pub boundary_validity: Option<bool>,
    pub verification: Option<bool>,
    pub transfer: Option<bool>,
}

/// Diagnostic evidence for Chemistry problems, branched by sub-discipline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "branch")]
pub enum ChemistryEvidence {
    Physical {
        model_setup: Option<bool>,
        equation_selection: Option<bool>,
        #[serde(default)]
        intermediate_quantity: Option<bool>,
        calculation: Option<bool>,
        conservation: Option<bool>,
        verification: Option<bool>,
        transfer: Option<bool>,
    },
    Organic {
        substrate_recognition: Option<bool>,
        mechanism_pathway: Option<bool>,
        reagent_interpretation: Option<bool>,
        product_prediction: Option<bool>,
        exception_handling: Option<bool>,
        transfer: Option<bool>,
    },
    Inorganic {
        trend_reasoning: Option<bool>,
        exception_handling: Option<bool>,
        qualitative_reasoning: Option<bool>,
        transfer: Option<bool>,
    },
}

/// The typed payload storing specific domain evidence.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "domain", content = "evidence")]
pub enum DomainEvidencePayload {
    Math(MathEvidence),
    Reasoning(ReasoningEvidence),
    Physics(PhysicsEvidence),
    Chemistry(ChemistryEvidence),
}

/// A versioned wrapper for domain-specific evidence ensuring backward compatibility.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VersionedDomainEvidence {
    pub version: u32,
    #[serde(flatten)]
    pub payload: DomainEvidencePayload,
}

impl VersionedDomainEvidence {
    pub fn new_math(evidence: MathEvidence) -> Self {
        Self {
            version: 1,
            payload: DomainEvidencePayload::Math(evidence),
        }
    }

    pub fn new_reasoning(evidence: ReasoningEvidence) -> Self {
        Self {
            version: 1,
            payload: DomainEvidencePayload::Reasoning(evidence),
        }
    }

    pub fn new_physics(evidence: PhysicsEvidence) -> Self {
        Self {
            version: 1,
            payload: DomainEvidencePayload::Physics(evidence),
        }
    }

    pub fn new_chemistry(evidence: ChemistryEvidence) -> Self {
        Self {
            version: 1,
            payload: DomainEvidencePayload::Chemistry(evidence),
        }
    }

    /// Whether this evidence payload indicates a surface-level calculation, execution, or unit slip,
    /// rather than a deep conceptual breakdown.
    pub fn is_execution_error(&self) -> bool {
        match &self.payload {
            DomainEvidencePayload::Math(m) => m.execution == Some(false),
            DomainEvidencePayload::Reasoning(_) => false,
            DomainEvidencePayload::Physics(p) => {
                p.calculation == Some(false)
                    || p.unit_validity == Some(false)
                    || p.boundary_validity == Some(false)
            }
            DomainEvidencePayload::Chemistry(c) => match c {
                ChemistryEvidence::Physical { calculation, .. } => *calculation == Some(false),
                _ => false,
            },
        }
    }

    /// Whether this evidence payload explicitly indicates a failure in initial model selection,
    /// pattern recognition, or governing principles.
    pub fn is_conceptual_error(&self) -> bool {
        match &self.payload {
            DomainEvidencePayload::Math(m) => {
                m.pattern_recognition == Some(false) || m.method_selection == Some(false)
            }
            DomainEvidencePayload::Reasoning(r) => {
                r.pattern_recognition == Some(false) || r.decision_path == Some(false)
            }
            DomainEvidencePayload::Physics(p) => {
                p.physical_model_selection == Some(false) || p.governing_principle == Some(false)
            }
            DomainEvidencePayload::Chemistry(c) => match c {
                ChemistryEvidence::Physical {
                    model_setup,
                    equation_selection,
                    ..
                } => *model_setup == Some(false) || *equation_selection == Some(false),
                ChemistryEvidence::Organic {
                    substrate_recognition,
                    mechanism_pathway,
                    ..
                } => *substrate_recognition == Some(false) || *mechanism_pathway == Some(false),
                ChemistryEvidence::Inorganic {
                    trend_reasoning,
                    qualitative_reasoning,
                    ..
                } => *trend_reasoning == Some(false) || *qualitative_reasoning == Some(false),
            },
        }
    }

    /// Whether this evidence payload explicitly indicates a failure at an intermediate procedural step
    /// (e.g. mole ratio, limiting reagent, ICE table change, cell half-reaction) rather than initial setup.
    pub fn is_intermediate_error(&self) -> bool {
        match &self.payload {
            DomainEvidencePayload::Chemistry(ChemistryEvidence::Physical {
                intermediate_quantity,
                model_setup,
                equation_selection,
                conservation,
                ..
            }) => {
                *intermediate_quantity == Some(false)
                    && *model_setup != Some(false)
                    && *equation_selection != Some(false)
                    && *conservation != Some(false)
            }
            _ => false,
        }
    }
}
