// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use crate::core::{Domain, Result, SkillId};
use crate::practice::SchemaPracticeObject;
use crate::problems::generators::{
    FAMILY_ALGEBRAIC_IDENTITIES, FAMILY_AVERAGE, FAMILY_COMBINED_MULTI_CONCEPT,
    FAMILY_DIVISIBILITY, FAMILY_GEOMETRY_TRIANGLES, FAMILY_LINEAR_EQUATIONS,
    FAMILY_LINEAR_INEQUALITIES, FAMILY_MIXTURES_ALLIGATION, FAMILY_PROFIT_LOSS, FAMILY_RATIO,
    FAMILY_REMAINDERS_MODULAR, FAMILY_TIME_SPEED_DISTANCE, FAMILY_TIME_WORK,
    TEMPLATE_ALGEBRAIC_IDENTITIES_V1, TEMPLATE_AVERAGE_V1, TEMPLATE_COMBINED_MULTI_CONCEPT_V1,
    TEMPLATE_DIVISIBILITY_V1, TEMPLATE_GEOMETRY_TRIANGLES_V1, TEMPLATE_LINEAR_EQUATIONS_V1,
    TEMPLATE_LINEAR_INEQUALITIES_V1, TEMPLATE_MIXTURES_ALLIGATION_V1, TEMPLATE_PROFIT_LOSS_V1,
    TEMPLATE_RATIO_V1, TEMPLATE_REMAINDERS_MODULAR_V1, TEMPLATE_TIME_SPEED_DISTANCE_V1,
    TEMPLATE_TIME_WORK_V1,
};
use crate::problems::ProblemFamily;
use crate::skills::Skill;
use crate::storage::ProceduralStore;

// Standard Skills
pub const SKILL_PERCENTAGE_SUCCESSIVE: &str = "percentage.successive";
pub const SKILL_MATH_PERCENTAGE_SUCCESSIVE: &str = "math.percentage.successive";
pub const SKILL_LINEAR_EQUATIONS: &str = "algebra.linear_equations";
pub const SKILL_PROFIT_LOSS: &str = "arithmetic.profit_loss";
pub const SKILL_RATIO: &str = "arithmetic.ratio";
pub const SKILL_AVERAGE: &str = "arithmetic.average";
pub const SKILL_DIVISIBILITY: &str = "number_system.divisibility";
pub const SKILL_TIME_WORK: &str = "time_work.basic";
pub const SKILL_TIME_SPEED_DISTANCE: &str = "arithmetic.time_speed_distance";
pub const SKILL_MIXTURES_ALLIGATION: &str = "arithmetic.mixtures_alligation";
pub const SKILL_REMAINDERS_MODULAR: &str = "number_system.remainders_modular";
pub const SKILL_LINEAR_INEQUALITIES: &str = "algebra.linear_inequalities";
pub const SKILL_ALGEBRAIC_IDENTITIES: &str = "algebra.algebraic_identities";
pub const SKILL_GEOMETRY_TRIANGLES: &str = "geometry.triangles";
pub const SKILL_COMBINED_MULTI_CONCEPT: &str = "combined.multi_concept";

// Physics Skills
pub const SKILL_PHYSICS_KINEMATICS: &str = "physics.kinematics.1d";
pub const SKILL_PHYSICS_WORK_ENERGY: &str = "physics.work_energy.mechanics";

// Chemistry Skills
pub const SKILL_CHEMISTRY_STOICHIOMETRY: &str = "chemistry.stoichiometry.moles";
pub const SKILL_CHEMISTRY_EQUILIBRIUM: &str = "chemistry.equilibrium.concentration";
pub const SKILL_CHEMISTRY_BUFFERS_TITRATION: &str = "chemistry.ionic_equilibrium.buffers_titration";
pub const SKILL_CHEMISTRY_ELECTROCHEMISTRY: &str = "chemistry.electrochemistry.nernst_faraday";
pub const SKILL_CHEMISTRY_KINETICS: &str = "chemistry.kinetics.rate_laws";
pub const SKILL_CHEMISTRY_REACTION_NETWORKS: &str = "chemistry.reaction_networks.multistage_synthesis";

// Reasoning Skills
pub const SKILL_REASONING_SERIES: &str = "reasoning.series.pattern_recognition";
pub const SKILL_REASONING_SYLLOGISM: &str = "reasoning.syllogism.formal_inference";
pub const SKILL_REASONING_SEATING: &str = "reasoning.seating.constraint_satisfaction";
pub const SKILL_REASONING_RELATIONS: &str = "reasoning.relations.graph_inference";
pub const SKILL_REASONING_BLOOD_RELATIONS: &str = "reasoning.blood_relations.kinship_graph";
pub const SKILL_REASONING_DIRECTION_SENSE: &str = "reasoning.direction_sense.spatial_orientation";
pub const SKILL_REASONING_FLOOR_GRID: &str = "reasoning.floor_grid.spatial_csp";
pub const SKILL_REASONING_LOGIC_DAG: &str = "reasoning.logic_dag.multi_step_inference";
pub const SKILL_REASONING_DATA_SUFFICIENCY: &str = "reasoning.data_sufficiency.constraint_sufficiency";
pub const SKILL_REASONING_CODED_EXPRESSIONS: &str = "reasoning.coded_expressions.symbolic_operators";

// Standard Families
pub const FAMILY_PERCENTAGE_SUCCESSIVE: &str = "family.math.percentage.successive";
pub const TEMPLATE_PERCENTAGE_SUCCESSIVE_V1: &str = "math.percentage.successive.v1";

// Chemistry Families
pub const FAMILY_CHEMISTRY_STOICHIOMETRY: &str = "family.chemistry.stoichiometry.moles";
pub const TEMPLATE_CHEMISTRY_STOICHIOMETRY_V1: &str = "chemistry.stoichiometry.moles.v1";
pub const FAMILY_CHEMISTRY_EQUILIBRIUM: &str = "family.chemistry.equilibrium.concentration";
pub const TEMPLATE_CHEMISTRY_EQUILIBRIUM_V1: &str = "chemistry.equilibrium.concentration.v1";
pub const FAMILY_CHEMISTRY_BUFFERS_TITRATION: &str = crate::chemistry::generators::FAMILY_CHEMISTRY_BUFFERS_TITRATION;
pub const TEMPLATE_CHEMISTRY_BUFFERS_TITRATION_V1: &str = crate::chemistry::generators::TEMPLATE_CHEMISTRY_BUFFERS_TITRATION_V1;
pub const FAMILY_CHEMISTRY_ELECTROCHEMISTRY: &str = crate::chemistry::generators::FAMILY_CHEMISTRY_ELECTROCHEMISTRY;
pub const TEMPLATE_CHEMISTRY_ELECTROCHEMISTRY_V1: &str = crate::chemistry::generators::TEMPLATE_CHEMISTRY_ELECTROCHEMISTRY_V1;
pub const FAMILY_CHEMISTRY_KINETICS: &str = crate::chemistry::generators::FAMILY_CHEMISTRY_KINETICS;
pub const TEMPLATE_CHEMISTRY_KINETICS_V1: &str = crate::chemistry::generators::TEMPLATE_CHEMISTRY_KINETICS_V1;
pub const FAMILY_CHEMISTRY_REACTION_NETWORKS: &str = crate::chemistry::generators::FAMILY_CHEMISTRY_REACTION_NETWORKS;
pub const TEMPLATE_CHEMISTRY_REACTION_NETWORKS_V1: &str = crate::chemistry::generators::TEMPLATE_CHEMISTRY_REACTION_NETWORKS_V1;

// Reasoning Families
pub const FAMILY_REASONING_SERIES: &str = crate::reasoning::generators::FAMILY_REASONING_SERIES;
pub const TEMPLATE_REASONING_SERIES_V1: &str = crate::reasoning::generators::TEMPLATE_REASONING_SERIES_V1;
pub const FAMILY_REASONING_SYLLOGISM: &str = crate::reasoning::generators::FAMILY_REASONING_SYLLOGISM;
pub const TEMPLATE_REASONING_SYLLOGISM_V1: &str = crate::reasoning::generators::TEMPLATE_REASONING_SYLLOGISM_V1;
pub const FAMILY_REASONING_SEATING: &str = crate::reasoning::generators::FAMILY_REASONING_SEATING;
pub const TEMPLATE_REASONING_SEATING_V1: &str = crate::reasoning::generators::TEMPLATE_REASONING_SEATING_V1;
pub const FAMILY_REASONING_RELATIONS: &str = crate::reasoning::generators::FAMILY_REASONING_RELATIONS;
pub const TEMPLATE_REASONING_RELATIONS_V1: &str = crate::reasoning::generators::TEMPLATE_REASONING_RELATIONS_V1;
pub const FAMILY_REASONING_BLOOD_RELATIONS: &str = crate::reasoning::generators::FAMILY_REASONING_BLOOD_RELATIONS;
pub const TEMPLATE_REASONING_BLOOD_RELATIONS_V1: &str = crate::reasoning::generators::TEMPLATE_REASONING_BLOOD_RELATIONS_V1;
pub const FAMILY_REASONING_DIRECTION_SENSE: &str = crate::reasoning::generators::FAMILY_REASONING_DIRECTION_SENSE;
pub const TEMPLATE_REASONING_DIRECTION_SENSE_V1: &str = crate::reasoning::generators::TEMPLATE_REASONING_DIRECTION_SENSE_V1;
pub const FAMILY_REASONING_FLOOR_GRID: &str = crate::reasoning::generators::FAMILY_REASONING_FLOOR_GRID;
pub const TEMPLATE_REASONING_FLOOR_GRID_V1: &str = crate::reasoning::generators::TEMPLATE_REASONING_FLOOR_GRID_V1;
pub const FAMILY_REASONING_LOGIC_DAG: &str = crate::reasoning::generators::FAMILY_REASONING_LOGIC_DAG;
pub const TEMPLATE_REASONING_LOGIC_DAG_V1: &str = crate::reasoning::generators::TEMPLATE_REASONING_LOGIC_DAG_V1;
pub const FAMILY_REASONING_DATA_SUFFICIENCY: &str = crate::reasoning::generators::FAMILY_REASONING_DATA_SUFFICIENCY;
pub const TEMPLATE_REASONING_DATA_SUFFICIENCY_V1: &str = crate::reasoning::generators::TEMPLATE_REASONING_DATA_SUFFICIENCY_V1;
pub const FAMILY_REASONING_CODED_EXPRESSIONS: &str = crate::reasoning::generators::FAMILY_REASONING_CODED_EXPRESSIONS;
pub const TEMPLATE_REASONING_CODED_EXPRESSIONS_V1: &str = crate::reasoning::generators::TEMPLATE_REASONING_CODED_EXPRESSIONS_V1;

// Standard Schemas
pub const SCHEMA_SUCCESSIVE_PERCENTAGE: &str = "successive_percentage";
pub const SCHEMA_MATH_PERCENTAGE_SUCCESSIVE: &str = "schema.math.percentage.successive";
pub const SCHEMA_LINEAR_EQUATIONS: &str = "algebra_linear_equations";
pub const SCHEMA_PROFIT_LOSS: &str = "arithmetic_profit_loss";
pub const SCHEMA_RATIO: &str = "arithmetic_ratio";
pub const SCHEMA_AVERAGE: &str = "arithmetic_average";
pub const SCHEMA_DIVISIBILITY: &str = "number_system_divisibility";
pub const SCHEMA_TIME_WORK: &str = "time_work_basic";
pub const SCHEMA_TIME_SPEED_DISTANCE: &str = "arithmetic_time_speed_distance";
pub const SCHEMA_MIXTURES_ALLIGATION: &str = "arithmetic_mixtures_alligation";
pub const SCHEMA_REMAINDERS_MODULAR: &str = "number_system_remainders_modular";
pub const SCHEMA_LINEAR_INEQUALITIES: &str = "algebra_linear_inequalities";
pub const SCHEMA_ALGEBRAIC_IDENTITIES: &str = "algebra_algebraic_identities";
pub const SCHEMA_GEOMETRY_TRIANGLES: &str = "geometry_triangles";
pub const SCHEMA_COMBINED_MULTI_CONCEPT: &str = "combined_multi_concept";

// Physics Schemas
pub const SCHEMA_PHYSICS_KINEMATICS: &str = "physics_kinematics_1d";
pub const SCHEMA_PHYSICS_WORK_ENERGY: &str = "physics_work_energy_mechanics";

// Chemistry Schemas
pub const SCHEMA_CHEMISTRY_STOICHIOMETRY: &str = "chemistry_stoichiometry_moles";
pub const SCHEMA_CHEMISTRY_EQUILIBRIUM: &str = "chemistry_equilibrium_concentration";
pub const SCHEMA_CHEMISTRY_BUFFERS_TITRATION: &str = "chemistry_buffers_titration_ionic";
pub const SCHEMA_CHEMISTRY_ELECTROCHEMISTRY: &str = "chemistry_electrochemistry_nernst_faraday";
pub const SCHEMA_CHEMISTRY_KINETICS: &str = "chemistry_kinetics_integrated_rates";
pub const SCHEMA_CHEMISTRY_REACTION_NETWORKS: &str = "chemistry_reaction_networks_multistage";

// Reasoning Schemas
pub const SCHEMA_REASONING_SERIES: &str = "reasoning_series_patterns";
pub const SCHEMA_REASONING_SYLLOGISM: &str = "reasoning_syllogism_categorical";
pub const SCHEMA_REASONING_SEATING: &str = "reasoning_seating_linear";
pub const SCHEMA_REASONING_RELATIONS: &str = "reasoning_relations_graph";
pub const SCHEMA_REASONING_BLOOD_RELATIONS: &str = "reasoning_blood_relations";
pub const SCHEMA_REASONING_DIRECTION_SENSE: &str = "reasoning_direction_sense";
pub const SCHEMA_REASONING_FLOOR_GRID: &str = "reasoning_floor_grid_spatial_csp";
pub const SCHEMA_REASONING_LOGIC_DAG: &str = "reasoning_logic_dag_multi_step_inference";
pub const SCHEMA_REASONING_DATA_SUFFICIENCY: &str = "reasoning_data_sufficiency_determinacy";
pub const SCHEMA_REASONING_CODED_EXPRESSIONS: &str = "reasoning_coded_expressions_symbolic_operators";

pub const PROCEDURAL_CATALOG_VERSION: &str = "3.0.0";
pub const MATHS_CATALOG_VERSION: &str = "3.0.0";

/// Canonical Mathematics catalog definitions and bootstrap initializer.
pub struct MathsCatalog;

impl MathsCatalog {
    // 1. Successive Percentage
    pub fn successive_percentage_skill() -> Skill {
        Skill::new(
            SKILL_PERCENTAGE_SUCCESSIVE,
            Domain::Mathematics,
            "Successive Percentage Changes",
            "Calculates final values, initial values, or net percentage changes across sequential percentage changes.",
        )
        .with_prerequisites(vec![SkillId::from("percentage.basic")])
        .with_metadata(serde_json::json!({
            "target_time_ms": 45_000,
            "typical_difficulty": 2.0,
            "domain_category": "arithmetic.percentage",
            "catalog_version": MATHS_CATALOG_VERSION,
        }))
    }

    pub fn successive_percentage_family() -> ProblemFamily {
        ProblemFamily::new(
            FAMILY_PERCENTAGE_SUCCESSIVE,
            SKILL_PERCENTAGE_SUCCESSIVE,
            Domain::Mathematics,
            "Successive Percentage Problem Family",
            TEMPLATE_PERCENTAGE_SUCCESSIVE_V1,
        )
        .with_difficulty_range(1.0, 5.0)
    }

    pub fn successive_percentage_schema() -> SchemaPracticeObject {
        SchemaPracticeObject::new(
            SCHEMA_SUCCESSIVE_PERCENTAGE,
            SKILL_PERCENTAGE_SUCCESSIVE,
            FAMILY_PERCENTAGE_SUCCESSIVE,
            "Successive Percentage Practice",
            "Practice solving sequential percentage transformations: Final = Initial × ∏(1 ± rate).",
        )
        .with_target_mastery(0.85)
        .with_config(serde_json::json!({
            "target_time_ms": 45_000,
            "formula": "Final = Initial * Product(1 ± rate)",
            "difficulty": 2.0,
        }))
    }

    // 2. Linear Equations
    pub fn linear_equations_skill() -> Skill {
        Skill::new(
            SKILL_LINEAR_EQUATIONS,
            Domain::Mathematics,
            "Linear Equations in One Variable",
            "Solving one-variable linear equations across standard, distributive, fractional, and word problem structures.",
        )
        .with_metadata(serde_json::json!({
            "target_time_ms": 35_000,
            "typical_difficulty": 2.0,
            "domain_category": "algebra.linear",
            "catalog_version": MATHS_CATALOG_VERSION,
        }))
    }

    pub fn linear_equations_family() -> ProblemFamily {
        ProblemFamily::new(
            FAMILY_LINEAR_EQUATIONS,
            SKILL_LINEAR_EQUATIONS,
            Domain::Mathematics,
            "Linear Equations Problem Family",
            TEMPLATE_LINEAR_EQUATIONS_V1,
        )
        .with_difficulty_range(1.0, 5.0)
    }

    pub fn linear_equations_schema() -> SchemaPracticeObject {
        SchemaPracticeObject::new(
            SCHEMA_LINEAR_EQUATIONS,
            SKILL_LINEAR_EQUATIONS,
            FAMILY_LINEAR_EQUATIONS,
            "Linear Equations Practice",
            "Solve single-variable linear equations: isolate unknown x via algebraic balancing.",
        )
        .with_target_mastery(0.85)
        .with_config(serde_json::json!({
            "target_time_ms": 35_000,
            "difficulty": 2.0,
        }))
    }

    // 3. Profit & Loss
    pub fn profit_loss_skill() -> Skill {
        Skill::new(
            SKILL_PROFIT_LOSS,
            Domain::Mathematics,
            "Profit, Loss, and Discount",
            "Calculates CP, SP, profit/loss percentages, marked price markup, and successive discounts.",
        )
        .with_prerequisites(vec![SkillId::from(SKILL_PERCENTAGE_SUCCESSIVE)])
        .with_metadata(serde_json::json!({
            "target_time_ms": 40_000,
            "typical_difficulty": 2.0,
            "domain_category": "arithmetic.commercial",
            "catalog_version": MATHS_CATALOG_VERSION,
        }))
    }

    pub fn profit_loss_family() -> ProblemFamily {
        ProblemFamily::new(
            FAMILY_PROFIT_LOSS,
            SKILL_PROFIT_LOSS,
            Domain::Mathematics,
            "Profit and Loss Problem Family",
            TEMPLATE_PROFIT_LOSS_V1,
        )
        .with_difficulty_range(1.0, 5.0)
    }

    pub fn profit_loss_schema() -> SchemaPracticeObject {
        SchemaPracticeObject::new(
            SCHEMA_PROFIT_LOSS,
            SKILL_PROFIT_LOSS,
            FAMILY_PROFIT_LOSS,
            "Profit and Loss Practice",
            "Practice calculating profit, loss, cost price, selling price, and discounts.",
        )
        .with_target_mastery(0.85)
        .with_config(serde_json::json!({
            "target_time_ms": 40_000,
            "difficulty": 2.0,
        }))
    }

    // 4. Ratio & Proportion
    pub fn ratio_skill() -> Skill {
        Skill::new(
            SKILL_RATIO,
            Domain::Mathematics,
            "Ratio and Proportion",
            "Divides quantities by ratio, solves missing proportions, combines multi-part ratios, and mixture proportions.",
        )
        .with_metadata(serde_json::json!({
            "target_time_ms": 35_000,
            "typical_difficulty": 2.0,
            "domain_category": "arithmetic.ratio",
            "catalog_version": MATHS_CATALOG_VERSION,
        }))
    }

    pub fn ratio_family() -> ProblemFamily {
        ProblemFamily::new(
            FAMILY_RATIO,
            SKILL_RATIO,
            Domain::Mathematics,
            "Ratio and Proportion Problem Family",
            TEMPLATE_RATIO_V1,
        )
        .with_difficulty_range(1.0, 5.0)
    }

    pub fn ratio_schema() -> SchemaPracticeObject {
        SchemaPracticeObject::new(
            SCHEMA_RATIO,
            SKILL_RATIO,
            FAMILY_RATIO,
            "Ratio and Proportion Practice",
            "Practice solving proportions, ratio division, and mixture problems.",
        )
        .with_target_mastery(0.85)
        .with_config(serde_json::json!({
            "target_time_ms": 35_000,
            "difficulty": 2.0,
        }))
    }

    // 5. Average
    pub fn average_skill() -> Skill {
        Skill::new(
            SKILL_AVERAGE,
            Domain::Mathematics,
            "Averages and Mean Values",
            "Computes arithmetic mean, finds missing data points, handles group inclusions/exclusions, and weighted averages.",
        )
        .with_metadata(serde_json::json!({
            "target_time_ms": 35_000,
            "typical_difficulty": 2.0,
            "domain_category": "arithmetic.statistics",
            "catalog_version": MATHS_CATALOG_VERSION,
        }))
    }

    pub fn average_family() -> ProblemFamily {
        ProblemFamily::new(
            FAMILY_AVERAGE,
            SKILL_AVERAGE,
            Domain::Mathematics,
            "Average Problem Family",
            TEMPLATE_AVERAGE_V1,
        )
        .with_difficulty_range(1.0, 5.0)
    }

    pub fn average_schema() -> SchemaPracticeObject {
        SchemaPracticeObject::new(
            SCHEMA_AVERAGE,
            SKILL_AVERAGE,
            FAMILY_AVERAGE,
            "Average Practice",
            "Practice calculating averages, weighted sums, and inclusion/exclusion shifts.",
        )
        .with_target_mastery(0.85)
        .with_config(serde_json::json!({
            "target_time_ms": 35_000,
            "difficulty": 2.0,
        }))
    }

    // 6. Divisibility
    pub fn divisibility_skill() -> Skill {
        Skill::new(
            SKILL_DIVISIBILITY,
            Domain::Mathematics,
            "Number System Divisibility Rules",
            "Applies divisibility rules (3, 4, 8, 9, 11, composite rules like 12, 72, 88), and solves missing digit constraints.",
        )
        .with_metadata(serde_json::json!({
            "target_time_ms": 35_000,
            "typical_difficulty": 2.0,
            "domain_category": "number_system",
            "catalog_version": MATHS_CATALOG_VERSION,
        }))
    }

    pub fn divisibility_family() -> ProblemFamily {
        ProblemFamily::new(
            FAMILY_DIVISIBILITY,
            SKILL_DIVISIBILITY,
            Domain::Mathematics,
            "Divisibility Problem Family",
            TEMPLATE_DIVISIBILITY_V1,
        )
        .with_difficulty_range(1.0, 5.0)
    }

    pub fn divisibility_schema() -> SchemaPracticeObject {
        SchemaPracticeObject::new(
            SCHEMA_DIVISIBILITY,
            SKILL_DIVISIBILITY,
            FAMILY_DIVISIBILITY,
            "Divisibility Practice",
            "Practice applying divisibility rules, identifying factors, and solving digit constraints.",
        )
        .with_target_mastery(0.85)
        .with_config(serde_json::json!({
            "target_time_ms": 35_000,
            "difficulty": 2.0,
        }))
    }

    // 7. Time & Work
    pub fn time_work_skill() -> Skill {
        Skill::new(
            SKILL_TIME_WORK,
            Domain::Mathematics,
            "Time and Work",
            "Calculates single worker rates, combined worker times, worker departure problems, relative efficiencies, and pipes/cisterns.",
        )
        .with_prerequisites(vec![SkillId::from(SKILL_RATIO)])
        .with_metadata(serde_json::json!({
            "target_time_ms": 40_000,
            "typical_difficulty": 2.0,
            "domain_category": "arithmetic.work",
            "catalog_version": MATHS_CATALOG_VERSION,
        }))
    }

    pub fn time_work_family() -> ProblemFamily {
        ProblemFamily::new(
            FAMILY_TIME_WORK,
            SKILL_TIME_WORK,
            Domain::Mathematics,
            "Time and Work Problem Family",
            TEMPLATE_TIME_WORK_V1,
        )
        .with_difficulty_range(1.0, 5.0)
    }

    pub fn time_work_schema() -> SchemaPracticeObject {
        SchemaPracticeObject::new(
            SCHEMA_TIME_WORK,
            SKILL_TIME_WORK,
            FAMILY_TIME_WORK,
            "Time and Work Practice",
            "Practice solving work rates, combined work times, and pipes and cisterns problems.",
        )
        .with_target_mastery(0.85)
        .with_config(serde_json::json!({
            "target_time_ms": 40_000,
            "difficulty": 2.0,
        }))
    }

    // 8. Time, Speed & Distance (TSD)
    pub fn time_speed_distance_skill() -> Skill {
        Skill::new(
            SKILL_TIME_SPEED_DISTANCE,
            Domain::Mathematics,
            "Time, Speed and Distance",
            "Solves distance-rate-time relationships, unit conversions (km/h <-> m/s), average speeds, relative speeds of trains, and journey equations.",
        )
        .with_prerequisites(vec![SkillId::from(SKILL_RATIO)])
        .with_metadata(serde_json::json!({
            "target_time_ms": 40_000,
            "typical_difficulty": 2.5,
            "domain_category": "arithmetic.motion",
            "catalog_version": MATHS_CATALOG_VERSION,
        }))
    }

    pub fn time_speed_distance_family() -> ProblemFamily {
        ProblemFamily::new(
            FAMILY_TIME_SPEED_DISTANCE,
            SKILL_TIME_SPEED_DISTANCE,
            Domain::Mathematics,
            "Time, Speed and Distance Problem Family",
            TEMPLATE_TIME_SPEED_DISTANCE_V1,
        )
        .with_difficulty_range(1.0, 5.0)
    }

    pub fn time_speed_distance_schema() -> SchemaPracticeObject {
        SchemaPracticeObject::new(
            SCHEMA_TIME_SPEED_DISTANCE,
            SKILL_TIME_SPEED_DISTANCE,
            FAMILY_TIME_SPEED_DISTANCE,
            "Time, Speed and Distance Practice",
            "Practice solving travel distances, unit conversions, harmonic average speeds, and relative motion problems.",
        )
        .with_target_mastery(0.85)
        .with_config(serde_json::json!({
            "target_time_ms": 40_000,
            "difficulty": 2.5,
        }))
    }

    // 9. Mixtures & Alligation
    pub fn mixtures_alligation_skill() -> Skill {
        Skill::new(
            SKILL_MIXTURES_ALLIGATION,
            Domain::Mathematics,
            "Mixtures and Alligation",
            "Solves two-component blends, Rule of Alligation ratios, solvent dilutions, repeated liquid replacements, and commercial alloys.",
        )
        .with_prerequisites(vec![SkillId::from(SKILL_RATIO), SkillId::from(SKILL_AVERAGE)])
        .with_metadata(serde_json::json!({
            "target_time_ms": 35_000,
            "typical_difficulty": 2.5,
            "domain_category": "arithmetic.mixtures",
            "catalog_version": MATHS_CATALOG_VERSION,
        }))
    }

    pub fn mixtures_alligation_family() -> ProblemFamily {
        ProblemFamily::new(
            FAMILY_MIXTURES_ALLIGATION,
            SKILL_MIXTURES_ALLIGATION,
            Domain::Mathematics,
            "Mixtures and Alligation Problem Family",
            TEMPLATE_MIXTURES_ALLIGATION_V1,
        )
        .with_difficulty_range(1.0, 5.0)
    }

    pub fn mixtures_alligation_schema() -> SchemaPracticeObject {
        SchemaPracticeObject::new(
            SCHEMA_MIXTURES_ALLIGATION,
            SKILL_MIXTURES_ALLIGATION,
            FAMILY_MIXTURES_ALLIGATION,
            "Mixtures and Alligation Practice",
            "Practice solving alligation ratios, concentrations, dilutions, and repeated replacement formulas.",
        )
        .with_target_mastery(0.85)
        .with_config(serde_json::json!({
            "target_time_ms": 35_000,
            "difficulty": 2.5,
        }))
    }

    // 10. Remainders & Modular Arithmetic
    pub fn remainders_modular_skill() -> Skill {
        Skill::new(
            SKILL_REMAINDERS_MODULAR,
            Domain::Mathematics,
            "Remainders and Modular Arithmetic",
            "Solves Euclidean division identities, expression remainders (A*B mod M), power unit digit cyclicity, and cyclical calendar modular arithmetic.",
        )
        .with_prerequisites(vec![SkillId::from(SKILL_DIVISIBILITY)])
        .with_metadata(serde_json::json!({
            "target_time_ms": 30_000,
            "typical_difficulty": 2.0,
            "domain_category": "number_system.modular",
            "catalog_version": MATHS_CATALOG_VERSION,
        }))
    }

    pub fn remainders_modular_family() -> ProblemFamily {
        ProblemFamily::new(
            FAMILY_REMAINDERS_MODULAR,
            SKILL_REMAINDERS_MODULAR,
            Domain::Mathematics,
            "Remainders and Modular Problem Family",
            TEMPLATE_REMAINDERS_MODULAR_V1,
        )
        .with_difficulty_range(1.0, 5.0)
    }

    pub fn remainders_modular_schema() -> SchemaPracticeObject {
        SchemaPracticeObject::new(
            SCHEMA_REMAINDERS_MODULAR,
            SKILL_REMAINDERS_MODULAR,
            FAMILY_REMAINDERS_MODULAR,
            "Remainders and Modular Practice",
            "Practice calculating modular remainders, power cyclicity, and calendar modular arithmetic.",
        )
        .with_target_mastery(0.85)
        .with_config(serde_json::json!({
            "target_time_ms": 30_000,
            "difficulty": 2.0,
        }))
    }

    // 11. Linear Inequalities
    pub fn linear_inequalities_skill() -> Skill {
        Skill::new(
            SKILL_LINEAR_INEQUALITIES,
            Domain::Mathematics,
            "Linear Inequalities in One Variable",
            "Solves one-step, two-step, negative-coefficient sign flips, variables on both sides, and compound double inequalities.",
        )
        .with_prerequisites(vec![SkillId::from(SKILL_LINEAR_EQUATIONS)])
        .with_metadata(serde_json::json!({
            "target_time_ms": 30_000,
            "typical_difficulty": 2.0,
            "domain_category": "algebra.inequalities",
            "catalog_version": MATHS_CATALOG_VERSION,
        }))
    }

    pub fn linear_inequalities_family() -> ProblemFamily {
        ProblemFamily::new(
            FAMILY_LINEAR_INEQUALITIES,
            SKILL_LINEAR_INEQUALITIES,
            Domain::Mathematics,
            "Linear Inequalities Problem Family",
            TEMPLATE_LINEAR_INEQUALITIES_V1,
        )
        .with_difficulty_range(1.0, 5.0)
    }

    pub fn linear_inequalities_schema() -> SchemaPracticeObject {
        SchemaPracticeObject::new(
            SCHEMA_LINEAR_INEQUALITIES,
            SKILL_LINEAR_INEQUALITIES,
            FAMILY_LINEAR_INEQUALITIES,
            "Linear Inequalities Practice",
            "Practice solving one-variable inequalities with sign reversal rules and integer solution intervals.",
        )
        .with_target_mastery(0.85)
        .with_config(serde_json::json!({
            "target_time_ms": 30_000,
            "difficulty": 2.0,
        }))
    }

    // 12. Algebraic Identities
    pub fn algebraic_identities_skill() -> Skill {
        Skill::new(
            SKILL_ALGEBRAIC_IDENTITIES,
            Domain::Mathematics,
            "Algebraic Identities and Expansions",
            "Applies difference of squares, sum/product identities, reciprocal squares (x + 1/x = k), reciprocal cubes, and conditional cubic identities.",
        )
        .with_prerequisites(vec![SkillId::from(SKILL_LINEAR_EQUATIONS)])
        .with_metadata(serde_json::json!({
            "target_time_ms": 30_000,
            "typical_difficulty": 2.5,
            "domain_category": "algebra.identities",
            "catalog_version": MATHS_CATALOG_VERSION,
        }))
    }

    pub fn algebraic_identities_family() -> ProblemFamily {
        ProblemFamily::new(
            FAMILY_ALGEBRAIC_IDENTITIES,
            SKILL_ALGEBRAIC_IDENTITIES,
            Domain::Mathematics,
            "Algebraic Identities Problem Family",
            TEMPLATE_ALGEBRAIC_IDENTITIES_V1,
        )
        .with_difficulty_range(1.0, 5.0)
    }

    pub fn algebraic_identities_schema() -> SchemaPracticeObject {
        SchemaPracticeObject::new(
            SCHEMA_ALGEBRAIC_IDENTITIES,
            SKILL_ALGEBRAIC_IDENTITIES,
            FAMILY_ALGEBRAIC_IDENTITIES,
            "Algebraic Identities Practice",
            "Practice expanding, factoring, and evaluating algebraic identities and reciprocal polynomial expressions.",
        )
        .with_target_mastery(0.85)
        .with_config(serde_json::json!({
            "target_time_ms": 30_000,
            "difficulty": 2.5,
        }))
    }

    // 13. Geometry: Triangles & Pythagoras
    pub fn geometry_triangles_skill() -> Skill {
        Skill::new(
            SKILL_GEOMETRY_TRIANGLES,
            Domain::Mathematics,
            "Geometry: Triangles and Pythagorean Theorem",
            "Solves right-angled Pythagorean triplets, triangle area and altitude, equilateral/special triangles, angle ratios, and spatial transfer problems.",
        )
        .with_prerequisites(vec![SkillId::from(SKILL_LINEAR_EQUATIONS)])
        .with_metadata(serde_json::json!({
            "target_time_ms": 30_000,
            "typical_difficulty": 2.0,
            "domain_category": "geometry.triangles",
            "catalog_version": MATHS_CATALOG_VERSION,
        }))
    }

    pub fn geometry_triangles_family() -> ProblemFamily {
        ProblemFamily::new(
            FAMILY_GEOMETRY_TRIANGLES,
            SKILL_GEOMETRY_TRIANGLES,
            Domain::Mathematics,
            "Geometry Triangles Problem Family",
            TEMPLATE_GEOMETRY_TRIANGLES_V1,
        )
        .with_difficulty_range(1.0, 5.0)
    }

    pub fn geometry_triangles_schema() -> SchemaPracticeObject {
        SchemaPracticeObject::new(
            SCHEMA_GEOMETRY_TRIANGLES,
            SKILL_GEOMETRY_TRIANGLES,
            FAMILY_GEOMETRY_TRIANGLES,
            "Geometry Triangles Practice",
            "Practice solving right triangles, area formulas, angle theorems, and spatial geometry models.",
        )
        .with_target_mastery(0.85)
        .with_config(serde_json::json!({
            "target_time_ms": 30_000,
            "difficulty": 2.0,
        }))
    }

    // 14. Combined Multi-Concept Mathematics
    pub fn combined_multi_concept_skill() -> Skill {
        Skill::new(
            SKILL_COMBINED_MULTI_CONCEPT,
            Domain::Mathematics,
            "Combined Multi-Concept Mathematics",
            "Integrates cross-schema problem structures including Percentage + Ratio, Profit/Loss + Successive Discounts, Ratio + Average, and Time/Work + Efficiency.",
        )
        .with_prerequisites(vec![
            SkillId::from(SKILL_PERCENTAGE_SUCCESSIVE),
            SkillId::from(SKILL_RATIO),
        ])
        .with_metadata(serde_json::json!({
            "target_time_ms": 40_000,
            "typical_difficulty": 3.0,
            "domain_category": "combined.multi_concept",
            "catalog_version": MATHS_CATALOG_VERSION,
        }))
    }

    pub fn combined_multi_concept_family() -> ProblemFamily {
        ProblemFamily::new(
            FAMILY_COMBINED_MULTI_CONCEPT,
            SKILL_COMBINED_MULTI_CONCEPT,
            Domain::Mathematics,
            "Combined Multi-Concept Problem Family",
            TEMPLATE_COMBINED_MULTI_CONCEPT_V1,
        )
        .with_difficulty_range(1.0, 5.0)
    }

    pub fn combined_multi_concept_schema() -> SchemaPracticeObject {
        SchemaPracticeObject::new(
            SCHEMA_COMBINED_MULTI_CONCEPT,
            SKILL_COMBINED_MULTI_CONCEPT,
            FAMILY_COMBINED_MULTI_CONCEPT,
            "Combined Multi-Concept Practice",
            "Practice solving composite problems spanning percentages, ratios, weighted averages, and multi-domain efficiency.",
        )
        .with_target_mastery(0.85)
        .with_config(serde_json::json!({
            "target_time_ms": 40_000,
            "difficulty": 3.0,
        }))
    }

    /// Idempotently initialize the full Mathematics catalog into the database.
    pub fn init_all(store: &ProceduralStore) -> Result<()> {
        // 1. Successive Percentage
        store.insert_skill(&Self::successive_percentage_skill())?;
        store.insert_problem_family(&Self::successive_percentage_family())?;
        store.insert_schema(&Self::successive_percentage_schema())?;

        // 2. Linear Equations
        store.insert_skill(&Self::linear_equations_skill())?;
        store.insert_problem_family(&Self::linear_equations_family())?;
        store.insert_schema(&Self::linear_equations_schema())?;

        // 3. Profit & Loss
        store.insert_skill(&Self::profit_loss_skill())?;
        store.insert_problem_family(&Self::profit_loss_family())?;
        store.insert_schema(&Self::profit_loss_schema())?;

        // 4. Ratio
        store.insert_skill(&Self::ratio_skill())?;
        store.insert_problem_family(&Self::ratio_family())?;
        store.insert_schema(&Self::ratio_schema())?;

        // 5. Average
        store.insert_skill(&Self::average_skill())?;
        store.insert_problem_family(&Self::average_family())?;
        store.insert_schema(&Self::average_schema())?;

        // 6. Divisibility
        store.insert_skill(&Self::divisibility_skill())?;
        store.insert_problem_family(&Self::divisibility_family())?;
        store.insert_schema(&Self::divisibility_schema())?;

        // 7. Time & Work
        store.insert_skill(&Self::time_work_skill())?;
        store.insert_problem_family(&Self::time_work_family())?;
        store.insert_schema(&Self::time_work_schema())?;

        // 8. Time, Speed & Distance
        store.insert_skill(&Self::time_speed_distance_skill())?;
        store.insert_problem_family(&Self::time_speed_distance_family())?;
        store.insert_schema(&Self::time_speed_distance_schema())?;

        // 9. Mixtures & Alligation
        store.insert_skill(&Self::mixtures_alligation_skill())?;
        store.insert_problem_family(&Self::mixtures_alligation_family())?;
        store.insert_schema(&Self::mixtures_alligation_schema())?;

        // 10. Remainders & Modular
        store.insert_skill(&Self::remainders_modular_skill())?;
        store.insert_problem_family(&Self::remainders_modular_family())?;
        store.insert_schema(&Self::remainders_modular_schema())?;

        // 11. Linear Inequalities
        store.insert_skill(&Self::linear_inequalities_skill())?;
        store.insert_problem_family(&Self::linear_inequalities_family())?;
        store.insert_schema(&Self::linear_inequalities_schema())?;

        // 12. Algebraic Identities
        store.insert_skill(&Self::algebraic_identities_skill())?;
        store.insert_problem_family(&Self::algebraic_identities_family())?;
        store.insert_schema(&Self::algebraic_identities_schema())?;

        // 13. Geometry Triangles
        store.insert_skill(&Self::geometry_triangles_skill())?;
        store.insert_problem_family(&Self::geometry_triangles_family())?;
        store.insert_schema(&Self::geometry_triangles_schema())?;

        // 14. Combined Multi-Concept
        store.insert_skill(&Self::combined_multi_concept_skill())?;
        store.insert_problem_family(&Self::combined_multi_concept_family())?;
        store.insert_schema(&Self::combined_multi_concept_schema())?;

        // 15. Physics: Kinematics 1D
        store.insert_skill(&Self::kinematics_skill())?;
        store.insert_problem_family(&Self::kinematics_family())?;
        store.insert_schema(&Self::kinematics_schema())?;

        // 16. Physics: Work & Energy
        store.insert_skill(&Self::work_energy_skill())?;
        store.insert_problem_family(&Self::work_energy_family())?;
        store.insert_schema(&Self::work_energy_schema())?;

        // 17. Chemistry: Stoichiometry
        store.insert_skill(&Self::stoichiometry_skill())?;
        store.insert_problem_family(&Self::stoichiometry_family())?;
        store.insert_schema(&Self::stoichiometry_schema())?;

        // 18. Chemistry: Equilibrium & Concentration
        store.insert_skill(&Self::equilibrium_skill())?;
        store.insert_problem_family(&Self::equilibrium_family())?;
        store.insert_schema(&Self::equilibrium_schema())?;

        // 19. Reasoning: Series Patterns
        store.insert_skill(&Self::series_skill())?;
        store.insert_problem_family(&Self::series_family())?;
        store.insert_schema(&Self::series_schema())?;

        // 20. Reasoning: Categorical Syllogism
        store.insert_skill(&Self::syllogism_skill())?;
        store.insert_problem_family(&Self::syllogism_family())?;
        store.insert_schema(&Self::syllogism_schema())?;

        // 21. Reasoning: Linear Seating
        store.insert_skill(&Self::seating_skill())?;
        store.insert_problem_family(&Self::seating_family())?;
        store.insert_schema(&Self::seating_schema())?;

        // 22. Reasoning: Relational Graphs & Direction
        store.insert_skill(&Self::relations_skill())?;
        store.insert_problem_family(&Self::relations_family())?;
        store.insert_schema(&Self::relations_schema())?;

        // 23. Chemistry: Ionic Equilibrium (Buffers & Titration)
        store.insert_skill(&Self::buffers_titration_skill())?;
        store.insert_problem_family(&Self::buffers_titration_family())?;
        store.insert_schema(&Self::buffers_titration_schema())?;

        // 24. Chemistry: Electrochemistry (Nernst & Faraday)
        store.insert_skill(&Self::electrochemistry_skill())?;
        store.insert_problem_family(&Self::electrochemistry_family())?;
        store.insert_schema(&Self::electrochemistry_schema())?;

        // 25. Chemistry: Chemical Kinetics (Integrated Rates)
        store.insert_skill(&Self::kinetics_skill())?;
        store.insert_problem_family(&Self::kinetics_family())?;
        store.insert_schema(&Self::kinetics_schema())?;

        // 26. Chemistry: Stoichiometry (Reaction Networks)
        store.insert_skill(&Self::reaction_networks_skill())?;
        store.insert_problem_family(&Self::reaction_networks_family())?;
        store.insert_schema(&Self::reaction_networks_schema())?;

        // 27. Reasoning: Analytical CSP (Floor / Grid)
        store.insert_skill(&Self::floor_grid_skill())?;
        store.insert_problem_family(&Self::floor_grid_family())?;
        store.insert_schema(&Self::floor_grid_schema())?;

        // 28. Reasoning: Deductive Logic (Multi-Premise DAG)
        store.insert_skill(&Self::logic_dag_skill())?;
        store.insert_problem_family(&Self::logic_dag_family())?;
        store.insert_schema(&Self::logic_dag_schema())?;

        // 29. Reasoning: Meta-Cognitive (Data Sufficiency)
        store.insert_skill(&Self::data_sufficiency_skill())?;
        store.insert_problem_family(&Self::data_sufficiency_family())?;
        store.insert_schema(&Self::data_sufficiency_schema())?;

        // 30. Reasoning: Graph/Relational (Coded Expressions)
        store.insert_skill(&Self::coded_expressions_skill())?;
        store.insert_problem_family(&Self::coded_expressions_family())?;
        store.insert_schema(&Self::coded_expressions_schema())?;

        // 31. Reasoning: Blood Relations (Kinship Graph)
        store.insert_skill(&Self::blood_relations_skill())?;
        store.insert_problem_family(&Self::blood_relations_family())?;
        store.insert_schema(&Self::blood_relations_schema())?;

        // 32. Reasoning: Direction Sense (Spatial Orientation)
        store.insert_skill(&Self::direction_sense_skill())?;
        store.insert_problem_family(&Self::direction_sense_family())?;
        store.insert_schema(&Self::direction_sense_schema())?;

        Ok(())
    }

    // 15. Physics: Kinematics 1D
    pub fn kinematics_skill() -> Skill {
        Skill::new(
            SKILL_PHYSICS_KINEMATICS,
            Domain::Physics,
            "1D Kinematics and Motion",
            "Models 1D rectilinear motion with uniform speed, constant acceleration (v=u+at, s=ut+1/2at^2, v^2=u^2+2as), unit conversion (km/h <-> m/s), braking distance, and vertical projectile motion under gravity.",
        )
        .with_metadata(serde_json::json!({
            "target_time_ms": 35_000,
            "typical_difficulty": 2.5,
            "domain_category": "physics.kinematics",
            "catalog_version": PROCEDURAL_CATALOG_VERSION,
        }))
    }

    pub fn kinematics_family() -> ProblemFamily {
        ProblemFamily::new(
            crate::physics::generators::FAMILY_PHYSICS_KINEMATICS,
            SKILL_PHYSICS_KINEMATICS,
            Domain::Physics,
            "1D Kinematics Problem Family",
            crate::physics::generators::TEMPLATE_PHYSICS_KINEMATICS_V1,
        )
        .with_difficulty_range(1.0, 5.0)
    }

    pub fn kinematics_schema() -> SchemaPracticeObject {
        SchemaPracticeObject::new(
            SCHEMA_PHYSICS_KINEMATICS,
            SKILL_PHYSICS_KINEMATICS,
            crate::physics::generators::FAMILY_PHYSICS_KINEMATICS,
            "1D Kinematics Practice",
            "Practice solving 1D kinematics problems spanning uniform velocity, accelerated motion, braking distance, and vertical free-fall.",
        )
        .with_target_mastery(0.85)
        .with_config(serde_json::json!({
            "target_time_ms": 35_000,
            "difficulty": 2.5,
            "domain": "physics",
        }))
    }

    // 16. Physics: Work & Energy Mechanics
    pub fn work_energy_skill() -> Skill {
        Skill::new(
            SKILL_PHYSICS_WORK_ENERGY,
            Domain::Physics,
            "Work, Energy and Power Mechanics",
            "Applies Kinetic Energy, Gravitational Potential Energy, Work Done by Constant Forces, Work-Energy Theorem, Mechanical Energy Conservation, and Power calculations.",
        )
        .with_prerequisites(vec![SkillId::from(SKILL_PHYSICS_KINEMATICS)])
        .with_metadata(serde_json::json!({
            "target_time_ms": 35_000,
            "typical_difficulty": 2.5,
            "domain_category": "physics.work_energy",
            "catalog_version": PROCEDURAL_CATALOG_VERSION,
        }))
    }

    pub fn work_energy_family() -> ProblemFamily {
        ProblemFamily::new(
            crate::physics::generators::FAMILY_PHYSICS_WORK_ENERGY,
            SKILL_PHYSICS_WORK_ENERGY,
            Domain::Physics,
            "Work and Energy Problem Family",
            crate::physics::generators::TEMPLATE_PHYSICS_WORK_ENERGY_V1,
        )
        .with_difficulty_range(1.0, 5.0)
    }

    pub fn work_energy_schema() -> SchemaPracticeObject {
        SchemaPracticeObject::new(
            SCHEMA_PHYSICS_WORK_ENERGY,
            SKILL_PHYSICS_WORK_ENERGY,
            crate::physics::generators::FAMILY_PHYSICS_WORK_ENERGY,
            "Work & Energy Practice",
            "Practice solving work done by forces, kinetic and potential energy conversions, conservation of mechanical energy, and mechanical power rate.",
        )
        .with_target_mastery(0.85)
        .with_config(serde_json::json!({
            "target_time_ms": 35_000,
            "difficulty": 2.5,
            "domain": "physics",
        }))
    }

    // 17. Chemistry: Stoichiometry / Mole Concept
    pub fn stoichiometry_skill() -> Skill {
        Skill::new(
            SKILL_CHEMISTRY_STOICHIOMETRY,
            Domain::Chemistry,
            "Stoichiometry and Mole Calculations",
            "Applies molar mass conversions (n=m/M), reaction stoichiometric ratios, mass-to-mass stoichiometry, limiting reagent identification, and percentage yield.",
        )
        .with_metadata(serde_json::json!({
            "target_time_ms": 35_000,
            "typical_difficulty": 2.5,
            "domain_category": "chemistry.stoichiometry",
            "catalog_version": PROCEDURAL_CATALOG_VERSION,
        }))
    }

    pub fn stoichiometry_family() -> ProblemFamily {
        ProblemFamily::new(
            FAMILY_CHEMISTRY_STOICHIOMETRY,
            SKILL_CHEMISTRY_STOICHIOMETRY,
            Domain::Chemistry,
            "Stoichiometry Problem Family",
            TEMPLATE_CHEMISTRY_STOICHIOMETRY_V1,
        )
        .with_difficulty_range(1.0, 5.0)
    }

    pub fn stoichiometry_schema() -> SchemaPracticeObject {
        SchemaPracticeObject::new(
            SCHEMA_CHEMISTRY_STOICHIOMETRY,
            SKILL_CHEMISTRY_STOICHIOMETRY,
            FAMILY_CHEMISTRY_STOICHIOMETRY,
            "Stoichiometry & Mole Concept Practice",
            "Practice solving chemical mole conversions, stoichiometric reaction ratios, limiting reagents, and percentage yields.",
        )
        .with_target_mastery(0.85)
        .with_config(serde_json::json!({
            "target_time_ms": 35_000,
            "difficulty": 2.5,
            "domain": "chemistry",
        }))
    }

    // 18. Chemistry: Equilibrium & Concentration
    pub fn equilibrium_skill() -> Skill {
        Skill::new(
            SKILL_CHEMISTRY_EQUILIBRIUM,
            Domain::Chemistry,
            "Chemical Equilibrium and Solution Concentration",
            "Calculates solution molarity (M=n/V), equilibrium constant expressions (Kc=[C]^c/[A]^a), ICE table extent calculations, quadratic equilibrium solving, and Le Chatelier reaction quotient Qc shifts.",
        )
        .with_prerequisites(vec![SkillId::from(SKILL_CHEMISTRY_STOICHIOMETRY)])
        .with_metadata(serde_json::json!({
            "target_time_ms": 40_000,
            "typical_difficulty": 2.5,
            "domain_category": "chemistry.equilibrium",
            "catalog_version": PROCEDURAL_CATALOG_VERSION,
        }))
    }

    pub fn equilibrium_family() -> ProblemFamily {
        ProblemFamily::new(
            FAMILY_CHEMISTRY_EQUILIBRIUM,
            SKILL_CHEMISTRY_EQUILIBRIUM,
            Domain::Chemistry,
            "Chemical Equilibrium Problem Family",
            TEMPLATE_CHEMISTRY_EQUILIBRIUM_V1,
        )
        .with_difficulty_range(1.0, 5.0)
    }

    pub fn equilibrium_schema() -> SchemaPracticeObject {
        SchemaPracticeObject::new(
            SCHEMA_CHEMISTRY_EQUILIBRIUM,
            SKILL_CHEMISTRY_EQUILIBRIUM,
            FAMILY_CHEMISTRY_EQUILIBRIUM,
            "Chemical Equilibrium Practice",
            "Practice solving solution concentrations, equilibrium constant expressions, ICE tables, and Le Chatelier disturbance shifts.",
        )
        .with_target_mastery(0.85)
        .with_config(serde_json::json!({
            "target_time_ms": 40_000,
            "difficulty": 2.5,
            "domain": "chemistry",
        }))
    }

    // 19. Reasoning: Series Patterns
    pub fn series_skill() -> Skill {
        Skill::new(
            SKILL_REASONING_SERIES,
            Domain::Reasoning,
            "Series and Pattern Recognition",
            "Identifies underlying mathematical rules (arithmetic difference, increasing difference, geometric ratio, alternating operations, alphabet shifts) and determines missing terms.",
        )
        .with_metadata(serde_json::json!({
            "target_time_ms": 25_000,
            "typical_difficulty": 2.0,
            "domain_category": "reasoning.series",
            "catalog_version": PROCEDURAL_CATALOG_VERSION,
        }))
    }

    pub fn series_family() -> ProblemFamily {
        ProblemFamily::new(
            FAMILY_REASONING_SERIES,
            SKILL_REASONING_SERIES,
            Domain::Reasoning,
            "Series Problem Family",
            TEMPLATE_REASONING_SERIES_V1,
        )
        .with_difficulty_range(1.0, 5.0)
    }

    pub fn series_schema() -> SchemaPracticeObject {
        SchemaPracticeObject::new(
            SCHEMA_REASONING_SERIES,
            SKILL_REASONING_SERIES,
            FAMILY_REASONING_SERIES,
            "Series & Pattern Recognition Practice",
            "Practice identifying and completing number and alphabet sequences across arithmetic, geometric, and alternating patterns.",
        )
        .with_target_mastery(0.85)
        .with_config(serde_json::json!({
            "target_time_ms": 25_000,
            "difficulty": 2.0,
            "domain": "reasoning",
        }))
    }

    // 20. Reasoning: Categorical Syllogism
    pub fn syllogism_skill() -> Skill {
        Skill::new(
            SKILL_REASONING_SYLLOGISM,
            Domain::Reasoning,
            "Categorical Syllogism and Formal Logic",
            "Evaluates formal deductive conclusions from quantified premises (All, No, Some, Some Not) using set-theoretic and Euler model validity.",
        )
        .with_metadata(serde_json::json!({
            "target_time_ms": 30_000,
            "typical_difficulty": 2.5,
            "domain_category": "reasoning.syllogism",
            "catalog_version": PROCEDURAL_CATALOG_VERSION,
        }))
    }

    pub fn syllogism_family() -> ProblemFamily {
        ProblemFamily::new(
            FAMILY_REASONING_SYLLOGISM,
            SKILL_REASONING_SYLLOGISM,
            Domain::Reasoning,
            "Syllogism Problem Family",
            TEMPLATE_REASONING_SYLLOGISM_V1,
        )
        .with_difficulty_range(1.0, 5.0)
    }

    pub fn syllogism_schema() -> SchemaPracticeObject {
        SchemaPracticeObject::new(
            SCHEMA_REASONING_SYLLOGISM,
            SKILL_REASONING_SYLLOGISM,
            FAMILY_REASONING_SYLLOGISM,
            "Categorical Syllogism Practice",
            "Practice formal deductive reasoning and validity checking across categorical logic forms.",
        )
        .with_target_mastery(0.85)
        .with_config(serde_json::json!({
            "target_time_ms": 30_000,
            "difficulty": 2.5,
            "domain": "reasoning",
        }))
    }

    // 21. Reasoning: Linear Seating
    pub fn seating_skill() -> Skill {
        Skill::new(
            SKILL_REASONING_SEATING,
            Domain::Reasoning,
            "Linear Seating and Constraint Satisfaction",
            "Applies constraint satisfaction strategies (anchor fixed positions, propagate adjacency and relative ordering, branch cases) to solve linear seating puzzles.",
        )
        .with_prerequisites(vec![SkillId::from(SKILL_REASONING_SERIES)])
        .with_metadata(serde_json::json!({
            "target_time_ms": 35_000,
            "typical_difficulty": 2.5,
            "domain_category": "reasoning.seating",
            "catalog_version": PROCEDURAL_CATALOG_VERSION,
        }))
    }

    pub fn seating_family() -> ProblemFamily {
        ProblemFamily::new(
            FAMILY_REASONING_SEATING,
            SKILL_REASONING_SEATING,
            Domain::Reasoning,
            "Linear Seating Problem Family",
            TEMPLATE_REASONING_SEATING_V1,
        )
        .with_difficulty_range(1.0, 5.0)
    }

    pub fn seating_schema() -> SchemaPracticeObject {
        SchemaPracticeObject::new(
            SCHEMA_REASONING_SEATING,
            SKILL_REASONING_SEATING,
            FAMILY_REASONING_SEATING,
            "Linear Seating Arrangement Practice",
            "Practice solving multi-entity positional arrangement puzzles using deterministic constraint satisfaction.",
        )
        .with_target_mastery(0.85)
        .with_config(serde_json::json!({
            "target_time_ms": 35_000,
            "difficulty": 2.5,
            "domain": "reasoning",
        }))
    }

    // 22. Reasoning: Relational Graphs & Direction
    pub fn relations_skill() -> Skill {
        Skill::new(
            SKILL_REASONING_RELATIONS,
            Domain::Reasoning,
            "Relational Graph and Spatial Direction",
            "Models kinship family graphs and traces 2D coordinate displacement vectors and compass orientations.",
        )
        .with_metadata(serde_json::json!({
            "target_time_ms": 30_000,
            "typical_difficulty": 2.5,
            "domain_category": "reasoning.relations",
            "catalog_version": PROCEDURAL_CATALOG_VERSION,
        }))
    }

    pub fn relations_family() -> ProblemFamily {
        ProblemFamily::new(
            FAMILY_REASONING_RELATIONS,
            SKILL_REASONING_RELATIONS,
            Domain::Reasoning,
            "Relational Graph Problem Family",
            TEMPLATE_REASONING_RELATIONS_V1,
        )
        .with_difficulty_range(1.0, 5.0)
    }

    pub fn relations_schema() -> SchemaPracticeObject {
        SchemaPracticeObject::new(
            SCHEMA_REASONING_RELATIONS,
            SKILL_REASONING_RELATIONS,
            FAMILY_REASONING_RELATIONS,
            "Relational & Spatial Graph Practice",
            "Practice genealogical kinship path deduction and 2D spatial movement displacement calculations.",
        )
        .with_target_mastery(0.85)
        .with_config(serde_json::json!({
            "target_time_ms": 30_000,
            "difficulty": 2.5,
            "domain": "reasoning",
        }))
    }

    // 23. Chemistry: Ionic Equilibrium (Buffers & Titration)
    pub fn buffers_titration_skill() -> Skill {
        Skill::new(
            SKILL_CHEMISTRY_BUFFERS_TITRATION,
            Domain::Chemistry,
            "Ionic Equilibrium, Buffers and Titrations",
            "Calculates pH of weak acid/base solutions, buffer solutions via Henderson-Hasselbalch, buffer capacity shifts, and titration curves (equivalence point hydrolysis, half-equivalence, excess reagent).",
        )
        .with_prerequisites(vec![SkillId::from(SKILL_CHEMISTRY_EQUILIBRIUM)])
        .with_metadata(serde_json::json!({
            "target_time_ms": 40_000,
            "typical_difficulty": 3.0,
            "domain_category": "chemistry.ionic_equilibrium",
            "catalog_version": PROCEDURAL_CATALOG_VERSION,
        }))
    }

    pub fn buffers_titration_family() -> ProblemFamily {
        ProblemFamily::new(
            FAMILY_CHEMISTRY_BUFFERS_TITRATION,
            SKILL_CHEMISTRY_BUFFERS_TITRATION,
            Domain::Chemistry,
            "Buffers & Titration Problem Family",
            TEMPLATE_CHEMISTRY_BUFFERS_TITRATION_V1,
        )
        .with_difficulty_range(1.0, 5.0)
    }

    pub fn buffers_titration_schema() -> SchemaPracticeObject {
        SchemaPracticeObject::new(
            SCHEMA_CHEMISTRY_BUFFERS_TITRATION,
            SKILL_CHEMISTRY_BUFFERS_TITRATION,
            FAMILY_CHEMISTRY_BUFFERS_TITRATION,
            "Buffers & Titration Practice",
            "Practice solving aqueous ionic equilibrium, buffer Henderson-Hasselbalch calculations, and titration endpoints.",
        )
        .with_target_mastery(0.85)
        .with_config(serde_json::json!({
            "target_time_ms": 40_000,
            "difficulty": 3.0,
            "domain": "chemistry",
        }))
    }

    // 24. Chemistry: Electrochemistry (Nernst & Faraday)
    pub fn electrochemistry_skill() -> Skill {
        Skill::new(
            SKILL_CHEMISTRY_ELECTROCHEMISTRY,
            Domain::Chemistry,
            "Electrochemistry, Nernst Equation and Faraday Electrolysis",
            "Calculates galvanic standard cell potentials (E°_cell = E°_cathode - E°_anode), non-standard Nernst cell potentials (E = E° - (RT/nF) ln Q), and quantitative Faraday electrolysis (m = M*I*t / z*F).",
        )
        .with_prerequisites(vec![SkillId::from(SKILL_CHEMISTRY_STOICHIOMETRY)])
        .with_metadata(serde_json::json!({
            "target_time_ms": 40_000,
            "typical_difficulty": 3.0,
            "domain_category": "chemistry.electrochemistry",
            "catalog_version": PROCEDURAL_CATALOG_VERSION,
        }))
    }

    pub fn electrochemistry_family() -> ProblemFamily {
        ProblemFamily::new(
            FAMILY_CHEMISTRY_ELECTROCHEMISTRY,
            SKILL_CHEMISTRY_ELECTROCHEMISTRY,
            Domain::Chemistry,
            "Electrochemistry Problem Family",
            TEMPLATE_CHEMISTRY_ELECTROCHEMISTRY_V1,
        )
        .with_difficulty_range(1.0, 5.0)
    }

    pub fn electrochemistry_schema() -> SchemaPracticeObject {
        SchemaPracticeObject::new(
            SCHEMA_CHEMISTRY_ELECTROCHEMISTRY,
            SKILL_CHEMISTRY_ELECTROCHEMISTRY,
            FAMILY_CHEMISTRY_ELECTROCHEMISTRY,
            "Electrochemistry Practice",
            "Practice solving electrochemical cell voltages, Nernst non-standard potentials, and quantitative Faraday electrolysis.",
        )
        .with_target_mastery(0.85)
        .with_config(serde_json::json!({
            "target_time_ms": 40_000,
            "difficulty": 3.0,
            "domain": "chemistry",
        }))
    }

    // 25. Chemistry: Chemical Kinetics (Integrated Rates)
    pub fn kinetics_skill() -> Skill {
        Skill::new(
            SKILL_CHEMISTRY_KINETICS,
            Domain::Chemistry,
            "Chemical Kinetics, Integrated Rate Laws and Arrhenius Equation",
            "Applies 0th, 1st, and 2nd order integrated rate laws, calculates half-life (t_1/2), deduces reaction orders via initial rates method, and calculates activation energy (Ea) via Arrhenius equation.",
        )
        .with_prerequisites(vec![SkillId::from(SKILL_CHEMISTRY_STOICHIOMETRY)])
        .with_metadata(serde_json::json!({
            "target_time_ms": 35_000,
            "typical_difficulty": 3.0,
            "domain_category": "chemistry.kinetics",
            "catalog_version": PROCEDURAL_CATALOG_VERSION,
        }))
    }

    pub fn kinetics_family() -> ProblemFamily {
        ProblemFamily::new(
            FAMILY_CHEMISTRY_KINETICS,
            SKILL_CHEMISTRY_KINETICS,
            Domain::Chemistry,
            "Chemical Kinetics Problem Family",
            TEMPLATE_CHEMISTRY_KINETICS_V1,
        )
        .with_difficulty_range(1.0, 5.0)
    }

    pub fn kinetics_schema() -> SchemaPracticeObject {
        SchemaPracticeObject::new(
            SCHEMA_CHEMISTRY_KINETICS,
            SKILL_CHEMISTRY_KINETICS,
            FAMILY_CHEMISTRY_KINETICS,
            "Chemical Kinetics Practice",
            "Practice solving zero/first/second-order integrated rate laws, initial rate tables, and Arrhenius activation energy.",
        )
        .with_target_mastery(0.85)
        .with_config(serde_json::json!({
            "target_time_ms": 35_000,
            "difficulty": 3.0,
            "domain": "chemistry",
        }))
    }

    // 26. Chemistry: Stoichiometry (Reaction Networks)
    pub fn reaction_networks_skill() -> Skill {
        Skill::new(
            SKILL_CHEMISTRY_REACTION_NETWORKS,
            Domain::Chemistry,
            "Reaction Networks and Multi-Stage Synthesis",
            "Solves multi-stage sequential synthesis pathways, tracks intermediate molar flux, calculates net percentage yield across multiple stages (eta_net = eta1 * eta2), and performs mixture stoichiometric decomposition.",
        )
        .with_prerequisites(vec![SkillId::from(SKILL_CHEMISTRY_STOICHIOMETRY)])
        .with_metadata(serde_json::json!({
            "target_time_ms": 45_000,
            "typical_difficulty": 3.5,
            "domain_category": "chemistry.reaction_networks",
            "catalog_version": PROCEDURAL_CATALOG_VERSION,
        }))
    }

    pub fn reaction_networks_family() -> ProblemFamily {
        ProblemFamily::new(
            FAMILY_CHEMISTRY_REACTION_NETWORKS,
            SKILL_CHEMISTRY_REACTION_NETWORKS,
            Domain::Chemistry,
            "Reaction Networks Problem Family",
            TEMPLATE_CHEMISTRY_REACTION_NETWORKS_V1,
        )
        .with_difficulty_range(1.0, 5.0)
    }

    pub fn reaction_networks_schema() -> SchemaPracticeObject {
        SchemaPracticeObject::new(
            SCHEMA_CHEMISTRY_REACTION_NETWORKS,
            SKILL_CHEMISTRY_REACTION_NETWORKS,
            FAMILY_CHEMISTRY_REACTION_NETWORKS,
            "Reaction Networks Practice",
            "Practice solving multi-step sequential chemical syntheses, intermediate species flux balance, and mixture stoichiometry.",
        )
        .with_target_mastery(0.85)
        .with_config(serde_json::json!({
            "target_time_ms": 45_000,
            "difficulty": 3.5,
            "domain": "chemistry",
        }))
    }

    // 27. Reasoning: Analytical CSP (Floor / Grid)
    pub fn floor_grid_skill() -> Skill {
        Skill::new(
            SKILL_REASONING_FLOOR_GRID,
            Domain::Reasoning,
            "Floor and Grid Spatial Constraint Satisfaction",
            "Applies discrete constraint satisfaction to solve multi-entity vertical floor assignments and 2D grid matrix puzzles with relative distance, betweenness, and parity bounds.",
        )
        .with_prerequisites(vec![SkillId::from(SKILL_REASONING_SEATING)])
        .with_metadata(serde_json::json!({
            "target_time_ms": 45_000,
            "typical_difficulty": 3.0,
            "domain_category": "reasoning.floor_grid",
            "catalog_version": PROCEDURAL_CATALOG_VERSION,
        }))
    }

    pub fn floor_grid_family() -> ProblemFamily {
        ProblemFamily::new(
            FAMILY_REASONING_FLOOR_GRID,
            SKILL_REASONING_FLOOR_GRID,
            Domain::Reasoning,
            "Floor & Grid CSP Problem Family",
            TEMPLATE_REASONING_FLOOR_GRID_V1,
        )
        .with_difficulty_range(1.0, 5.0)
    }

    pub fn floor_grid_schema() -> SchemaPracticeObject {
        SchemaPracticeObject::new(
            SCHEMA_REASONING_FLOOR_GRID,
            SKILL_REASONING_FLOOR_GRID,
            FAMILY_REASONING_FLOOR_GRID,
            "Floor & Grid Spatial CSP Practice",
            "Practice multi-constraint vertical floor placement and 2D matrix spatial layout deduction.",
        )
        .with_target_mastery(0.85)
        .with_config(serde_json::json!({
            "target_time_ms": 45_000,
            "difficulty": 3.0,
            "domain": "reasoning",
        }))
    }

    // 28. Reasoning: Deductive Logic (Multi-Premise DAG)
    pub fn logic_dag_skill() -> Skill {
        Skill::new(
            SKILL_REASONING_LOGIC_DAG,
            Domain::Reasoning,
            "Multi-Premise Deductive Logic DAG",
            "Evaluates multi-step propositional inference chains (Modus Ponens, Modus Tollens, Disjunctive Syllogism, Hypothetical Syllogism) to derive logically necessary conclusions or detect contradictions.",
        )
        .with_prerequisites(vec![SkillId::from(SKILL_REASONING_SYLLOGISM)])
        .with_metadata(serde_json::json!({
            "target_time_ms": 40_000,
            "typical_difficulty": 3.0,
            "domain_category": "reasoning.logic_dag",
            "catalog_version": PROCEDURAL_CATALOG_VERSION,
        }))
    }

    pub fn logic_dag_family() -> ProblemFamily {
        ProblemFamily::new(
            FAMILY_REASONING_LOGIC_DAG,
            SKILL_REASONING_LOGIC_DAG,
            Domain::Reasoning,
            "Logic DAG Problem Family",
            TEMPLATE_REASONING_LOGIC_DAG_V1,
        )
        .with_difficulty_range(1.0, 5.0)
    }

    pub fn logic_dag_schema() -> SchemaPracticeObject {
        SchemaPracticeObject::new(
            SCHEMA_REASONING_LOGIC_DAG,
            SKILL_REASONING_LOGIC_DAG,
            FAMILY_REASONING_LOGIC_DAG,
            "Multi-Premise Deductive Logic Practice",
            "Practice multi-stage proposition chaining and deductive inference rule derivation.",
        )
        .with_target_mastery(0.85)
        .with_config(serde_json::json!({
            "target_time_ms": 40_000,
            "difficulty": 3.0,
            "domain": "reasoning",
        }))
    }

    // 29. Reasoning: Meta-Cognitive (Data Sufficiency)
    pub fn data_sufficiency_skill() -> Skill {
        Skill::new(
            SKILL_REASONING_DATA_SUFFICIENCY,
            Domain::Reasoning,
            "Data Sufficiency and System Determinacy",
            "Evaluates whether given statements individually or combined provide mathematically sufficient information to uniquely determine the value of a target quantity.",
        )
        .with_prerequisites(vec![SkillId::from(SKILL_LINEAR_EQUATIONS)])
        .with_metadata(serde_json::json!({
            "target_time_ms": 35_000,
            "typical_difficulty": 3.0,
            "domain_category": "reasoning.data_sufficiency",
            "catalog_version": PROCEDURAL_CATALOG_VERSION,
        }))
    }

    pub fn data_sufficiency_family() -> ProblemFamily {
        ProblemFamily::new(
            FAMILY_REASONING_DATA_SUFFICIENCY,
            SKILL_REASONING_DATA_SUFFICIENCY,
            Domain::Reasoning,
            "Data Sufficiency Problem Family",
            TEMPLATE_REASONING_DATA_SUFFICIENCY_V1,
        )
        .with_difficulty_range(1.0, 5.0)
    }

    pub fn data_sufficiency_schema() -> SchemaPracticeObject {
        SchemaPracticeObject::new(
            SCHEMA_REASONING_DATA_SUFFICIENCY,
            SKILL_REASONING_DATA_SUFFICIENCY,
            FAMILY_REASONING_DATA_SUFFICIENCY,
            "Data Sufficiency Practice",
            "Practice meta-cognitive evaluation of statement sufficiency, algebraic determinacy, and counterexample elimination.",
        )
        .with_target_mastery(0.85)
        .with_config(serde_json::json!({
            "target_time_ms": 35_000,
            "difficulty": 3.0,
            "domain": "reasoning",
        }))
    }

    // 30. Reasoning: Graph/Relational (Coded Expressions)
    pub fn coded_expressions_skill() -> Skill {
        Skill::new(
            SKILL_REASONING_CODED_EXPRESSIONS,
            Domain::Reasoning,
            "Coded Symbolic Expressions and Relational Graphs",
            "Decodes symbolic relational and directional operator expressions, constructs directed entity graph paths, and deduces composite relationships and 2D displacement vectors.",
        )
        .with_prerequisites(vec![SkillId::from(SKILL_REASONING_RELATIONS)])
        .with_metadata(serde_json::json!({
            "target_time_ms": 40_000,
            "typical_difficulty": 3.0,
            "domain_category": "reasoning.coded_expressions",
            "catalog_version": PROCEDURAL_CATALOG_VERSION,
        }))
    }

    pub fn coded_expressions_family() -> ProblemFamily {
        ProblemFamily::new(
            FAMILY_REASONING_CODED_EXPRESSIONS,
            SKILL_REASONING_CODED_EXPRESSIONS,
            Domain::Reasoning,
            "Coded Expressions Problem Family",
            TEMPLATE_REASONING_CODED_EXPRESSIONS_V1,
        )
        .with_difficulty_range(1.0, 5.0)
    }

    pub fn coded_expressions_schema() -> SchemaPracticeObject {
        SchemaPracticeObject::new(
            SCHEMA_REASONING_CODED_EXPRESSIONS,
            SKILL_REASONING_CODED_EXPRESSIONS,
            FAMILY_REASONING_CODED_EXPRESSIONS,
            "Coded Expressions Practice",
            "Practice decoding symbolic relational operator chains and directional vector expressions.",
        )
        .with_target_mastery(0.85)
        .with_config(serde_json::json!({
            "target_time_ms": 40_000,
            "difficulty": 3.0,
            "domain": "reasoning",
        }))
    }

    // 31. Reasoning: Blood Relations (Kinship Graph)
    pub fn blood_relations_skill() -> Skill {
        Skill::new(
            SKILL_REASONING_BLOOD_RELATIONS,
            Domain::Reasoning,
            "Blood Relations and Kinship Graph Inference",
            "Constructs genealogical kinship trees and deduces multi-hop family relationships across generational tiers.",
        )
        .with_metadata(serde_json::json!({
            "target_time_ms": 30_000,
            "typical_difficulty": 2.5,
            "domain_category": "reasoning.blood_relations",
            "catalog_version": PROCEDURAL_CATALOG_VERSION,
        }))
    }

    pub fn blood_relations_family() -> ProblemFamily {
        ProblemFamily::new(
            FAMILY_REASONING_BLOOD_RELATIONS,
            SKILL_REASONING_BLOOD_RELATIONS,
            Domain::Reasoning,
            "Blood Relations Problem Family",
            TEMPLATE_REASONING_BLOOD_RELATIONS_V1,
        )
        .with_difficulty_range(1.0, 5.0)
    }

    pub fn blood_relations_schema() -> SchemaPracticeObject {
        SchemaPracticeObject::new(
            SCHEMA_REASONING_BLOOD_RELATIONS,
            SKILL_REASONING_BLOOD_RELATIONS,
            FAMILY_REASONING_BLOOD_RELATIONS,
            "Blood Relations Practice",
            "Practice solving multi-generational family kinship graphs and relationship deduction.",
        )
        .with_target_mastery(0.85)
        .with_config(serde_json::json!({
            "target_time_ms": 30_000,
            "difficulty": 2.5,
            "domain": "reasoning",
        }))
    }

    // 32. Reasoning: Direction Sense (Spatial Orientation)
    pub fn direction_sense_skill() -> Skill {
        Skill::new(
            SKILL_REASONING_DIRECTION_SENSE,
            Domain::Reasoning,
            "Direction Sense and 2D Spatial Orientation",
            "Traces 2D coordinate displacement vectors, turns, and computes cardinal and ordinal directions relative to origin.",
        )
        .with_metadata(serde_json::json!({
            "target_time_ms": 30_000,
            "typical_difficulty": 2.5,
            "domain_category": "reasoning.direction_sense",
            "catalog_version": PROCEDURAL_CATALOG_VERSION,
        }))
    }

    pub fn direction_sense_family() -> ProblemFamily {
        ProblemFamily::new(
            FAMILY_REASONING_DIRECTION_SENSE,
            SKILL_REASONING_DIRECTION_SENSE,
            Domain::Reasoning,
            "Direction Sense Problem Family",
            TEMPLATE_REASONING_DIRECTION_SENSE_V1,
        )
        .with_difficulty_range(1.0, 5.0)
    }

    pub fn direction_sense_schema() -> SchemaPracticeObject {
        SchemaPracticeObject::new(
            SCHEMA_REASONING_DIRECTION_SENSE,
            SKILL_REASONING_DIRECTION_SENSE,
            FAMILY_REASONING_DIRECTION_SENSE,
            "Direction Sense Practice",
            "Practice decomposing multi-turn walks into 2D orthogonal displacement vectors and determining final compass headings.",
        )
        .with_target_mastery(0.85)
        .with_config(serde_json::json!({
            "target_time_ms": 30_000,
            "difficulty": 2.5,
            "domain": "reasoning",
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catalog_idempotent_init() {
        let store = ProceduralStore::open_in_memory().unwrap();
        MathsCatalog::init_all(&store).unwrap();

        // Check skills
        let skill_pct = store
            .get_skill(&SkillId::new(SKILL_PERCENTAGE_SUCCESSIVE))
            .unwrap();
        assert!(skill_pct.is_some());

        let skill_tsd = store
            .get_skill(&SkillId::new(SKILL_TIME_SPEED_DISTANCE))
            .unwrap();
        assert!(skill_tsd.is_some());

        let skill_comb = store
            .get_skill(&SkillId::new(SKILL_COMBINED_MULTI_CONCEPT))
            .unwrap();
        assert!(skill_comb.is_some());

        // Re-running init must succeed without duplicate key errors (idempotent)
        MathsCatalog::init_all(&store).unwrap();
    }
}
