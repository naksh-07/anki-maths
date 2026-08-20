// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use rand::rngs::StdRng;
use rand::SeedableRng;
use serde_json::json;

use crate::chemistry::buffers_titration::BufferTitrationPuzzle;
use crate::chemistry::models::{ChemicalProblemMetadata, ChemicalRegimeKind};
use crate::chemistry::units::ChemistryUnit;
use crate::core::{ProblemFamilyId, Result};
use crate::core::decision::{CognitiveDecisionPoint, DecisionOption};
use crate::diagnostics::ErrorCategory;
use crate::problems::generator::ProblemGenerator;
use crate::problems::steps::{HintLevel, SolutionGraph, StepHint, StepNode, StepType};
use crate::problems::validator::{AnswerEvaluation, NumericAnswerParser, ProblemValidator};
use crate::problems::ProblemInstance;

pub const FAMILY_CHEMISTRY_BUFFERS_TITRATION: &str = "family.chemistry.buffers_titration.ionic";
pub const TEMPLATE_CHEMISTRY_BUFFERS_TITRATION_V1: &str = "chemistry.buffers_titration.ionic.v1";

/// Generator for Ionic Equilibrium, Buffers & Titration problems.
pub struct BuffersTitrationGenerator;

impl BuffersTitrationGenerator {
    pub fn generate_problem(seed: u64, difficulty_level: u32, _variant: Option<&str>) -> ProblemInstance {
        let mut rng = StdRng::seed_from_u64(seed);
        let puzzle = BufferTitrationPuzzle::generate_dynamic(&mut rng, difficulty_level);

        let dp = CognitiveDecisionPoint::new(
            "dp_ionic_regime",
            "Which ionic equilibrium regime is active in this solution?",
            vec![
                DecisionOption::new(
                    "opt_buffer",
                    "Acidic or Basic Buffer (Henderson-Hasselbalch)",
                    "regime_buffer",
                    difficulty_level == 2 || difficulty_level == 3 || difficulty_level == 5,
                    "Use Henderson-Hasselbalch when weak conjugate acid-base pairs co-exist in significant amounts.",
                ),
                DecisionOption::new(
                    "opt_pure_weak",
                    "Pure Weak Acid Dissociation ([H+] = sqrt(Ka * C))",
                    "regime_weak_acid",
                    difficulty_level == 1,
                    "Use pure dissociation when only the weak acid is initially dissolved in water.",
                ),
                DecisionOption::new(
                    "opt_salt_hydrolysis",
                    "Salt Hydrolysis at Equivalence Point (pH = 7 + 0.5*pKa + 0.5*logC)",
                    "regime_hydrolysis",
                    difficulty_level == 4,
                    "At the titration equivalence point, only the conjugate salt is present, undergoing anion/cation hydrolysis.",
                ),
            ],
            match difficulty_level {
                1 => "opt_pure_weak",
                4 => "opt_salt_hydrolysis",
                _ => "opt_buffer",
            },
            match difficulty_level {
                1 => "regime_weak_acid",
                4 => "regime_hydrolysis",
                _ => "regime_buffer",
            },
            "Always identify the dominant species present (Pure Acid vs Buffer vs Salt Hydrolysis) before applying an equation.",
        );

        let meta = ChemicalProblemMetadata::new(
            ChemicalRegimeKind::EquilibriumConstantExpression,
            ChemistryUnit::Dimensionless,
        );

        let step1 = StepNode::new(
            "identify_regime",
            StepType::ConstructEquilibriumExpression,
            "Identify Ionic Equilibrium Regime",
            format!("Classify solution regime: {:?}", puzzle.regime),
            format!("{:?}", puzzle.regime),
        )
        .with_hints(vec![
            StepHint::new(HintLevel::Principle, "Regime Identification", "Identify if this is a weak acid, buffer, or salt hydrolysis."),
            StepHint::new(HintLevel::Operation, "Formula Selection", "Select Henderson-Hasselbalch or Salt Hydrolysis formula."),
            StepHint::new(HintLevel::IntermediateRelation, "Equilibrium Regime", format!("{:?}", puzzle.regime)),
        ]);

        let step2 = StepNode::new(
            "calculate_equilibrium_moles",
            StepType::ConvertMassToMoles,
            "Calculate Moles / Concentrations",
            "Account for neutralization stoichiometry or dissociation balance.",
            format!("pKa = {:.2}", puzzle.pka_or_pkb),
        )
        .with_dependencies(vec!["identify_regime".to_string()]);

        let step3 = StepNode::new(
            "compute_ph",
            StepType::FinalAnswer,
            "Compute Target pH",
            format!("Target pH = {:.2}", puzzle.correct_ph),
            format!("{:.2}", puzzle.correct_ph),
        )
        .with_dependencies(vec!["calculate_equilibrium_moles".to_string()])
        .as_final();

        let solution_graph = SolutionGraph::new(vec![step1, step2, step3], "compute_ph");

        let parameters = json!({
            "difficulty": difficulty_level,
            "acid_or_base": puzzle.acid_or_base_name,
            "pka": puzzle.pka_or_pkb,
            "chemical_metadata": meta,
            "decision_point": dp,
        });

        let correct_answer = json!({
            "value": puzzle.correct_ph,
            "formatted": format!("{:.2}", puzzle.correct_ph),
            "correct_ph": puzzle.correct_ph,
            "unit": "pH",
            "explanation": puzzle.step_by_step_explanation,
        });

        let instance_id = format!("inst-bt-l{}-{}", difficulty_level, seed);

        ProblemInstance::new(
            instance_id,
            FAMILY_CHEMISTRY_BUFFERS_TITRATION,
            seed,
            parameters,
            puzzle.question_prompt,
            correct_answer,
        )
        .with_solution_graph(solution_graph)
        .with_metadata(json!({
            "difficulty_level": difficulty_level,
            "target_time_ms": 40_000,
            "domain": "chemistry",
            "generator": TEMPLATE_CHEMISTRY_BUFFERS_TITRATION_V1,
        }))
    }
}

impl ProblemGenerator for BuffersTitrationGenerator {
    fn family_id(&self) -> &str {
        FAMILY_CHEMISTRY_BUFFERS_TITRATION
    }

    fn template_ref(&self) -> &str {
        TEMPLATE_CHEMISTRY_BUFFERS_TITRATION_V1
    }

    fn supported_variants(&self) -> Vec<String> {
        vec!["default".to_string(), "strategy_drill".to_string()]
    }

    fn target_latency_ms(&self, _difficulty_level: u32) -> u64 {
        40_000
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

/// Validator for Ionic Equilibrium & Buffers problems.
pub struct BuffersTitrationValidator;

impl ProblemValidator for BuffersTitrationValidator {
    fn family_id(&self) -> &str {
        FAMILY_CHEMISTRY_BUFFERS_TITRATION
    }

    fn evaluate(
        &self,
        instance: &ProblemInstance,
        student_answer: &serde_json::Value,
        time_taken_ms: u64,
        target_time_ms: u64,
    ) -> AnswerEvaluation {
        let expected_ph = instance
            .correct_answer
            .get("value")
            .and_then(|v| v.as_f64())
            .unwrap_or(7.0);

        let parsed = NumericAnswerParser::parse_value(student_answer);

        if let Some(val) = parsed {
            let is_correct = (val - expected_ph).abs() <= 0.06;
            if is_correct {
                let score = if time_taken_ms <= target_time_ms {
                    1.0
                } else {
                    (1.0 - ((time_taken_ms - target_time_ms) as f64 / target_time_ms as f64) * 0.5).max(0.5)
                };
                AnswerEvaluation::correct(score, time_taken_ms, target_time_ms)
                    .with_parsed_values(val, expected_ph)
            } else {
                AnswerEvaluation::incorrect(
                    ErrorCategory::Calculation,
                    format!("Incorrect buffer pH calculation. Submitted {:.2}, expected {:.2}.", val, expected_ph),
                )
                .with_parsed_values(val, expected_ph)
            }
        } else {
            AnswerEvaluation::incorrect(
                ErrorCategory::Syntax,
                "Could not parse numeric pH value from answer submission.".to_string(),
            )
        }
    }
}
