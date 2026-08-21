// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::core::{Domain, ProblemFamilyId, SchemaId, SkillId};
use crate::exam::pyq::ContentProvenance;
use crate::problems::steps::StepType;
use crate::skills::signals::VariantCategory;

/// High-level generator capability classification for problem families.
/// Explicitly classifies how a given family produces problem instances.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProblemFamilyCapability {
    /// Level 1: Parameter domains, algebraic relations, template rendering, and direct answer derivations.
    Declarative,
    /// Level 2: Constraint satisfaction problems (e.g. seating arrangements, floor/grid spatial layouts).
    ConstraintSolver,
    /// Level 2: Formal logic, syllogisms, logic DAGs, coded expressions.
    SymbolicLogic,
    /// Level 2: Physical laws, equations of motion, dimensional analysis, unit conversions.
    DomainPhysics,
    /// Level 2: Chemical reactions, stoichiometry, equilibrium, ionic titration.
    DomainChemistry,
    /// Level 2: Coordinate geometry, triangle inequalities, geometric properties.
    DomainGeometry,
    /// Level 3: Multi-concept composite generators or highly specialized custom logic.
    Specialized,
}

impl ProblemFamilyCapability {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProblemFamilyCapability::Declarative => "declarative",
            ProblemFamilyCapability::ConstraintSolver => "constraint_solver",
            ProblemFamilyCapability::SymbolicLogic => "symbolic_logic",
            ProblemFamilyCapability::DomainPhysics => "domain_physics",
            ProblemFamilyCapability::DomainChemistry => "domain_chemistry",
            ProblemFamilyCapability::DomainGeometry => "domain_geometry",
            ProblemFamilyCapability::Specialized => "specialized",
        }
    }

    pub fn is_declarative(&self) -> bool {
        matches!(self, ProblemFamilyCapability::Declarative)
    }

    pub fn is_domain_capability(&self) -> bool {
        matches!(
            self,
            ProblemFamilyCapability::ConstraintSolver
                | ProblemFamilyCapability::SymbolicLogic
                | ProblemFamilyCapability::DomainPhysics
                | ProblemFamilyCapability::DomainChemistry
                | ProblemFamilyCapability::DomainGeometry
        )
    }
}

impl std::fmt::Display for ProblemFamilyCapability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Unified procedural contract exposing complete conceptual and operational metadata
/// for a problem family.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProblemFamilyContract {
    pub family_id: ProblemFamilyId,
    pub skill_id: SkillId,
    pub domain: Domain,
    pub default_schema: SchemaId,
    pub capability: ProblemFamilyCapability,
    pub min_difficulty: f64,
    pub max_difficulty: f64,
    pub supported_variants: Vec<String>,
    pub variant_categories: Vec<VariantCategory>,
    pub target_latency_model: HashMap<u32, u64>,
    pub structural_tags: Vec<String>,
    pub decision_points: Vec<String>,
    pub error_categories: Vec<String>,
    pub prerequisites: Vec<String>,
    pub provenance: Option<ContentProvenance>,
    pub metadata: serde_json::Value,
}

impl ProblemFamilyContract {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        family_id: impl Into<ProblemFamilyId>,
        skill_id: impl Into<SkillId>,
        domain: Domain,
        default_schema: impl Into<SchemaId>,
        capability: ProblemFamilyCapability,
    ) -> Self {
        let mut target_latency_model = HashMap::new();
        target_latency_model.insert(1, 25_000);
        target_latency_model.insert(2, 35_000);
        target_latency_model.insert(3, 50_000);
        target_latency_model.insert(4, 65_000);
        target_latency_model.insert(5, 80_000);

        Self {
            family_id: family_id.into(),
            skill_id: skill_id.into(),
            domain,
            default_schema: default_schema.into(),
            capability,
            min_difficulty: 1.0,
            max_difficulty: 5.0,
            supported_variants: Vec::new(),
            variant_categories: vec![VariantCategory::Parameter, VariantCategory::Isomorphic],
            target_latency_model,
            structural_tags: Vec::new(),
            decision_points: Vec::new(),
            error_categories: Vec::new(),
            prerequisites: Vec::new(),
            provenance: None,
            metadata: serde_json::Value::Object(Default::default()),
        }
    }

    pub fn with_difficulty_range(mut self, min: f64, max: f64) -> Self {
        self.min_difficulty = min;
        self.max_difficulty = max;
        self
    }

    pub fn with_variants(mut self, variants: Vec<impl Into<String>>) -> Self {
        self.supported_variants = variants.into_iter().map(|v| v.into()).collect();
        self
    }

    pub fn with_variant_categories(mut self, categories: Vec<VariantCategory>) -> Self {
        self.variant_categories = categories;
        self
    }

    pub fn with_target_latency(mut self, level: u32, latency_ms: u64) -> Self {
        self.target_latency_model.insert(level, latency_ms);
        self
    }

    pub fn with_structural_tags(mut self, tags: Vec<impl Into<String>>) -> Self {
        self.structural_tags = tags.into_iter().map(|t| t.into()).collect();
        self
    }

    pub fn with_decision_points(mut self, points: Vec<impl Into<String>>) -> Self {
        self.decision_points = points.into_iter().map(|p| p.into()).collect();
        self
    }

    pub fn with_error_categories(mut self, errors: Vec<impl Into<String>>) -> Self {
        self.error_categories = errors.into_iter().map(|e| e.into()).collect();
        self
    }

    pub fn with_prerequisites(mut self, prereqs: Vec<impl Into<String>>) -> Self {
        self.prerequisites = prereqs.into_iter().map(|p| p.into()).collect();
        self
    }

    pub fn with_provenance(mut self, provenance: ContentProvenance) -> Self {
        self.provenance = Some(provenance);
        self
    }

    pub fn target_latency(&self, level: u32) -> u64 {
        self.target_latency_model
            .get(&level)
            .copied()
            .unwrap_or(45_000)
    }
}

/// Sampling domain specification for a generated parameter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ParameterDomain {
    /// Uniform integer in [min, max] with optional step and non-zero constraint.
    IntegerRange {
        min: i64,
        max: i64,
        step: Option<i64>,
        non_zero: Option<bool>,
    },
    /// Uniform floating point in [min, max] with specified precision decimals.
    FloatRange {
        min: f64,
        max: f64,
        precision: usize,
    },
    /// Uniform random choice from discrete static options.
    DiscreteChoice {
        values: Vec<serde_json::Value>,
    },
    /// Derived linear expression `target = a * x + b` from already-sampled parameters.
    DerivedLinear {
        a_param: String,
        x_param: String,
        b_param: String,
    },
    /// Derived product `target = a * b`.
    DerivedProduct {
        a_param: String,
        b_param: String,
    },
    /// Derived sum `target = a + b`.
    DerivedSum {
        a_param: String,
        b_param: String,
    },
    /// Derived difference `target = a - b`.
    DerivedDifference {
        a_param: String,
        b_param: String,
    },
    /// Derived signed string formatting: "+ b" or "- |b|"
    DerivedSignedString {
        param: String,
    },
}

/// Parameter definition with name and domain.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterSpec {
    pub name: String,
    pub domain: ParameterDomain,
}

impl ParameterSpec {
    pub fn new(name: impl Into<String>, domain: ParameterDomain) -> Self {
        Self {
            name: name.into(),
            domain,
        }
    }

    pub fn integer_range(name: impl Into<String>, min: i64, max: i64) -> Self {
        Self::new(
            name,
            ParameterDomain::IntegerRange {
                min,
                max,
                step: None,
                non_zero: None,
            },
        )
    }

    pub fn non_zero_integer_range(name: impl Into<String>, min: i64, max: i64) -> Self {
        Self::new(
            name,
            ParameterDomain::IntegerRange {
                min,
                max,
                step: None,
                non_zero: Some(true),
            },
        )
    }

    pub fn discrete_choice(name: impl Into<String>, values: Vec<serde_json::Value>) -> Self {
        Self::new(name, ParameterDomain::DiscreteChoice { values })
    }

    pub fn derived_linear(
        name: impl Into<String>,
        a_param: impl Into<String>,
        x_param: impl Into<String>,
        b_param: impl Into<String>,
    ) -> Self {
        Self::new(
            name,
            ParameterDomain::DerivedLinear {
                a_param: a_param.into(),
                x_param: x_param.into(),
                b_param: b_param.into(),
            },
        )
    }

    pub fn derived_product(
        name: impl Into<String>,
        a_param: impl Into<String>,
        b_param: impl Into<String>,
    ) -> Self {
        Self::new(
            name,
            ParameterDomain::DerivedProduct {
                a_param: a_param.into(),
                b_param: b_param.into(),
            },
        )
    }

    pub fn derived_sum(
        name: impl Into<String>,
        a_param: impl Into<String>,
        b_param: impl Into<String>,
    ) -> Self {
        Self::new(
            name,
            ParameterDomain::DerivedSum {
                a_param: a_param.into(),
                b_param: b_param.into(),
            },
        )
    }

    pub fn derived_difference(
        name: impl Into<String>,
        a_param: impl Into<String>,
        b_param: impl Into<String>,
    ) -> Self {
        Self::new(
            name,
            ParameterDomain::DerivedDifference {
                a_param: a_param.into(),
                b_param: b_param.into(),
            },
        )
    }

    pub fn derived_signed(name: impl Into<String>, param: impl Into<String>) -> Self {
        Self::new(
            name,
            ParameterDomain::DerivedSignedString {
                param: param.into(),
            },
        )
    }
}

/// Constraints that sampled parameters must satisfy.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ConstraintSpec {
    /// param_a != param_b
    NotEqual { param_a: String, param_b: String },
    /// param != 0
    NonZero { param: String },
    /// numerator % denominator == 0
    Divisible {
        numerator: String,
        denominator: String,
    },
    /// param_a > param_b
    GreaterThan { param_a: String, param_b: String },
    /// custom predicate name
    Predicate { name: String },
}

/// Answer derivation rules to compute canonical answer value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AnswerDerivation {
    /// Direct parameter lookup.
    DirectParam { param_name: String },
    /// Solve ax + b = c -> x = (c - b) / a
    LinearTwoStep {
        c_param: String,
        b_param: String,
        a_param: String,
    },
    /// Solve ax + b = cx + d -> x = (d - b) / (a - c)
    LinearVariablesBothSides {
        d_param: String,
        b_param: String,
        a_param: String,
        c_param: String,
    },
    /// Solve a(bx + c) = d -> x = (d/a - c) / b
    LinearDistributive {
        d_param: String,
        a_param: String,
        c_param: String,
        b_param: String,
    },
    /// Solve x/a + b = c -> x = a * (c - b)
    LinearFractional {
        c_param: String,
        b_param: String,
        a_param: String,
    },
    /// Direct quotient a / b
    Quotient {
        numerator_param: String,
        denominator_param: String,
    },
    /// Direct product a * b
    Product {
        a_param: String,
        b_param: String,
    },
}

/// Specification for a structured solution step in the solution graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StepNodeSpec {
    pub id: String,
    pub step_type: StepType,
    pub label: String,
    pub description_template: String,
    pub expected_expression_template: String,
    pub alternate_templates: Vec<String>,
    pub hint_principle: String,
    pub hint_operation: String,
    pub hint_intermediate: String,
}

impl StepNodeSpec {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: impl Into<String>,
        step_type: StepType,
        label: impl Into<String>,
        description: impl Into<String>,
        expected: impl Into<String>,
        alternates: Vec<String>,
        hint_principle: impl Into<String>,
        hint_operation: impl Into<String>,
        hint_intermediate: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            step_type,
            label: label.into(),
            description_template: description.into(),
            expected_expression_template: expected.into(),
            alternate_templates: alternates,
            hint_principle: hint_principle.into(),
            hint_operation: hint_operation.into(),
            hint_intermediate: hint_intermediate.into(),
        }
    }
}

/// Declarative archetype definition for a specific structural pattern & difficulty.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeclarativeArchetype {
    pub archetype_id: String,
    pub difficulty_level: u32,
    pub variant_category: VariantCategory,
    pub variant_name: String,
    pub parameters: Vec<ParameterSpec>,
    pub constraints: Vec<ConstraintSpec>,
    pub prompt_template: String,
    pub answer_derivation: AnswerDerivation,
    pub answer_formatted_template: String,
    pub solution_template: String,
    pub step_nodes: Vec<StepNodeSpec>,
    pub target_time_ms: u64,
}

impl DeclarativeArchetype {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        archetype_id: impl Into<String>,
        difficulty_level: u32,
        variant_category: VariantCategory,
        variant_name: impl Into<String>,
        parameters: Vec<ParameterSpec>,
        prompt_template: impl Into<String>,
        answer_derivation: AnswerDerivation,
        answer_formatted_template: impl Into<String>,
        solution_template: impl Into<String>,
        target_time_ms: u64,
    ) -> Self {
        Self {
            archetype_id: archetype_id.into(),
            difficulty_level,
            variant_category,
            variant_name: variant_name.into(),
            parameters,
            constraints: Vec::new(),
            prompt_template: prompt_template.into(),
            answer_derivation,
            answer_formatted_template: answer_formatted_template.into(),
            solution_template: solution_template.into(),
            step_nodes: Vec::new(),
            target_time_ms,
        }
    }

    pub fn with_constraints(mut self, constraints: Vec<ConstraintSpec>) -> Self {
        self.constraints = constraints;
        self
    }

    pub fn with_step_nodes(mut self, nodes: Vec<StepNodeSpec>) -> Self {
        self.step_nodes = nodes;
        self
    }
}

/// Complete declarative contract for a problem family consisting of structural
/// metadata and declarative archetypes for all difficulty levels.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeclarativeFamilyContract {
    pub contract: ProblemFamilyContract,
    pub archetypes: Vec<DeclarativeArchetype>,
}

impl DeclarativeFamilyContract {
    pub fn new(contract: ProblemFamilyContract, archetypes: Vec<DeclarativeArchetype>) -> Self {
        Self {
            contract,
            archetypes,
        }
    }

    pub fn find_archetype(&self, difficulty_level: u32, variant: Option<&str>) -> Option<&DeclarativeArchetype> {
        if let Some(v_name) = variant {
            if let Some(arch) = self.archetypes.iter().find(|a| a.variant_name == v_name) {
                return Some(arch);
            }
        }
        self.archetypes
            .iter()
            .find(|a| a.difficulty_level == difficulty_level)
            .or_else(|| self.archetypes.first())
    }
}
