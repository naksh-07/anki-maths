// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use rand::rngs::StdRng;
use rand::SeedableRng;
use serde_json::json;

use crate::chemistry::kinetics::KineticsPuzzle;
use crate::chemistry::models::{ChemicalProblemMetadata, ChemicalRegimeKind};
use crate::chemistry::units::ChemistryUnit;
use crate::core::{ProblemFamilyId, Result};
use crate::core::decision::{CognitiveDecisionPoint, DecisionOption};
use crate::diagnostics::ErrorCategory;
use crate::problems::generator::ProblemGenerator;
use crate::problems::steps::{HintLevel, SolutionGraph, StepHint, StepNode, StepType};
use crate::problems::validator::{AnswerEvaluation, NumericAnswerParser, ProblemValidator};
use crate::problems::ProblemInstance;

pub const FAMILY_CHEMISTRY_KINETICS: &str = "family.chemistry.kinetics.integrated_rates";
pub const TEMPLATE_CHEMISTRY_KINETICS_V1: &str = "chemistry.kinetics.integrated_rates.v1";

/// Generator for Chemical Kinetics problems (Integrated rate laws, half-life, Arrhenius).
pub struct ChemicalKineticsGenerator;

impl ChemicalKineticsGenerator {
    pub fn generate_problem(seed: u64, difficulty_level: u32, _variant: Option<&str>) -> ProblemInstance {
        let mut rng = StdRng::seed_from_u64(seed);
        let puzzle = KineticsPuzzle::generate_dynamic(&mut rng, difficulty_level);

        let dp = CognitiveDecisionPoint::new(
            "dp_kinetics_order",
            "Which integrated rate law formula or kinetic relationship applies?",
            vec![
                DecisionOption::new(
                    "opt_first_order",
                    "1st Order Kinetics (t_1/2 = 0.693/k or ln([A]0/[A]t) = kt)",
                    "law_first_order",
                    difficulty_level <= 2,
                    "Use first-order integrated rate law when rate depends linearly on single concentration.",
                ),
                DecisionOption::new(
                    "opt_second_order",
                    "2nd Order Kinetics (1/[A]t = 1/[A]0 + kt)",
                    "law_second_order",
                    difficulty_level == 3,
                    "Use second-order reciprocal equation when rate is proportional to [A]^2.",
                ),
                DecisionOption::new(
                    "opt_initial_rate",
                    "Method of Initial Rates (Compare isolated concentration changes)",
                    "law_initial_rate",
                    difficulty_level == 4,
                    "Use initial rate comparison table to deduce individual reaction orders.",
                ),
                DecisionOption::new(
                    "opt_arrhenius",
                    "Arrhenius Temperature Dependence (ln(k2/k1) = (Ea/R)(1/T1 - 1/T2))",
                    "law_arrhenius",
                    difficulty_level >= 5,
                    "Use Arrhenius equation when temperature changes alter rate constants.",
                ),
            ],
            match difficulty_level {
                1 | 2 => "opt_first_order",
                3 => "opt_second_order",
                4 => "opt_initial_rate",
                _ => "opt_arrhenius",
            },
            match difficulty_level {
                1 | 2 => "law_first_order",
                3 => "law_second_order",
                4 => "law_initial_rate",
                _ => "law_arrhenius",
            },
            "Always determine the reaction order or temperature dependence before selecting the rate law equation.",
        );

        let meta = ChemicalProblemMetadata::new(
            ChemicalRegimeKind::EquilibriumConstantExpression,
            ChemistryUnit::Dimensionless,
        );

        let step1 = StepNode::new(
            "identify_kinetic_order",
            StepType::ConstructEquilibriumExpression,
            "Identify Kinetic Order and Rate Constants",
            format!("Target: {}, Order: {}", puzzle.target_quantity, puzzle.reaction_order),
            format!("Order = {}", puzzle.reaction_order),
        )
        .with_hints(vec![
            StepHint::new(HintLevel::Principle, "Kinetics Principle", "Check the units of k or experimental ratios to deduce reaction order."),
            StepHint::new(HintLevel::Operation, "Equation Formulation", "Formulate the integrated rate law."),
            StepHint::new(HintLevel::IntermediateRelation, "Reaction Order", format!("Order = {}", puzzle.reaction_order)),
        ]);

        let step2 = StepNode::new(
            "apply_integrated_rate_law",
            StepType::ConvertMassToMoles,
            "Apply Integrated Rate Law",
            "Substitute parameters into the integrated rate or Arrhenius equation.",
            format!("Target = {:.3} {}", puzzle.correct_value, puzzle.unit_symbol),
        )
        .with_dependencies(vec!["identify_kinetic_order".to_string()]);

        let step3 = StepNode::new(
            "calculate_target_quantity",
            StepType::FinalAnswer,
            "Calculate Target Quantity",
            format!("{:.3} {}", puzzle.correct_value, puzzle.unit_symbol),
            format!("{:.3}", puzzle.correct_value),
        )
        .with_dependencies(vec!["apply_integrated_rate_law".to_string()])
        .as_final();

        let solution_graph = SolutionGraph::new(vec![step1, step2, step3], "calculate_target_quantity");

        let parameters = json!({
            "difficulty": difficulty_level,
            "reaction_order": puzzle.reaction_order,
            "target_quantity": puzzle.target_quantity,
            "unit": puzzle.unit_symbol,
            "chemical_metadata": meta,
            "decision_point": dp,
        });

        let correct_answer = json!({
            "value": puzzle.correct_value,
            "formatted": format!("{:.3} {}", puzzle.correct_value, puzzle.unit_symbol),
            "unit": puzzle.unit_symbol,
            "explanation": puzzle.step_by_step_explanation,
        });

        let instance_id = format!("inst-ck-l{}-{}", difficulty_level, seed);

        ProblemInstance::new(
            instance_id,
            FAMILY_CHEMISTRY_KINETICS,
            seed,
            parameters,
            puzzle.question_prompt,
            correct_answer,
        )
        .with_solution_graph(solution_graph)
        .with_metadata(json!({
            "difficulty_level": difficulty_level,
            "target_time_ms": 35_000,
            "domain": "chemistry",
            "generator": TEMPLATE_CHEMISTRY_KINETICS_V1,
        }))
    }
}

impl ProblemGenerator for ChemicalKineticsGenerator {
    fn family_id(&self) -> &str {
        FAMILY_CHEMISTRY_KINETICS
    }

    fn template_ref(&self) -> &str {
        TEMPLATE_CHEMISTRY_KINETICS_V1
    }

    fn supported_variants(&self) -> Vec<String> {
        vec!["default".to_string(), "strategy_drill".to_string()]
    }

    fn target_latency_ms(&self, _difficulty_level: u32) -> u64 {
        35_000
    }

    fn generate(
        &self,
        _family_id: &ProblemFamilyId,
        seed: u64,
        difficulty_level: u32,
        variant: Option<&str>,
    ) -> Result<ProblemInstance> {
        Ok(Self::generate_problem(seed, difficulty_level, variant))
    }
}

/// Validator for Chemical Kinetics problems.
pub struct ChemicalKineticsValidator;

impl ProblemValidator for ChemicalKineticsValidator {
    fn family_id(&self) -> &str {
        FAMILY_CHEMISTRY_KINETICS
    }

    fn evaluate(
        &self,
        instance: &ProblemInstance,
        student_answer: &serde_json::Value,
        time_taken_ms: u64,
        target_time_ms: u64,
    ) -> AnswerEvaluation {
        let expected_val = instance
            .correct_answer
            .get("value")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let unit = instance
            .correct_answer
            .get("unit")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let parsed = NumericAnswerParser::parse_value(student_answer);

        if let Some(val) = parsed {
            let rel_diff = if expected_val.abs() > 1e-4 {
                (val - expected_val).abs() / expected_val.abs()
            } else {
                (val - expected_val).abs()
            };

            let is_correct = rel_diff <= 0.04 || (val - expected_val).abs() <= 0.05;

            if is_correct {
                let score = if time_taken_ms <= target_time_ms {
                    1.0
                } else {
                    (1.0 - ((time_taken_ms - target_time_ms) as f64 / target_time_ms as f64) * 0.5).max(0.5)
                };
                AnswerEvaluation::correct(score, time_taken_ms, target_time_ms)
                    .with_parsed_values(val, expected_val)
            } else {
                AnswerEvaluation::incorrect(
                    ErrorCategory::Calculation,
                    format!("Incorrect kinetics calculation. Submitted {:.3}, expected {:.3} {}.", val, expected_val, unit),
                )
                .with_parsed_values(val, expected_val)
            }
        } else {
            AnswerEvaluation::incorrect(
                ErrorCategory::Syntax,
                "Could not parse numeric kinetics answer from submission.".to_string(),
            )
        }
    }
}
