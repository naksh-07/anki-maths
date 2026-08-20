// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use rand::rngs::StdRng;
use rand::SeedableRng;
use serde_json::json;

use crate::chemistry::electrochemistry::ElectrochemistryPuzzle;
use crate::chemistry::models::{ChemicalProblemMetadata, ChemicalRegimeKind};
use crate::chemistry::units::ChemistryUnit;
use crate::core::{ProblemFamilyId, Result};
use crate::core::decision::{CognitiveDecisionPoint, DecisionOption};
use crate::diagnostics::ErrorCategory;
use crate::problems::generator::ProblemGenerator;
use crate::problems::steps::{HintLevel, SolutionGraph, StepHint, StepNode, StepType};
use crate::problems::validator::{AnswerEvaluation, NumericAnswerParser, ProblemValidator};
use crate::problems::ProblemInstance;

pub const FAMILY_CHEMISTRY_ELECTROCHEMISTRY: &str = "family.chemistry.electrochemistry.cells";
pub const TEMPLATE_CHEMISTRY_ELECTROCHEMISTRY_V1: &str = "chemistry.electrochemistry.cells.v1";

/// Generator for Electrochemistry problems (Nernst equation & Faraday electrolysis).
pub struct ElectrochemistryGenerator;

impl ElectrochemistryGenerator {
    pub fn generate_problem(seed: u64, difficulty_level: u32, _variant: Option<&str>) -> ProblemInstance {
        let mut rng = StdRng::seed_from_u64(seed);
        let puzzle = ElectrochemistryPuzzle::generate_dynamic(&mut rng, difficulty_level);

        let dp = CognitiveDecisionPoint::new(
            "dp_electrochem_model",
            "Which governing equation applies to this electrochemical scenario?",
            vec![
                DecisionOption::new(
                    "opt_nernst",
                    "Nernst Equation (E = E° - (RT/nF) ln Q)",
                    "model_nernst",
                    difficulty_level == 2 || difficulty_level == 3,
                    "Use Nernst equation for non-standard concentration cell potentials in galvanic cells.",
                ),
                DecisionOption::new(
                    "opt_standard_pot",
                    "Standard Cell Potential (E° = E°_cathode - E°_anode)",
                    "model_standard",
                    difficulty_level == 1,
                    "Use standard reduction potentials difference when standard conditions (1 M, 1 atm, 298 K) are present.",
                ),
                DecisionOption::new(
                    "opt_faraday",
                    "Faraday's Laws of Electrolysis (m = M*I*t / z*F)",
                    "model_faraday",
                    difficulty_level >= 4,
                    "Use Faraday electrolysis equations when calculating electric current, time, and mass deposited.",
                ),
            ],
            match difficulty_level {
                1 => "opt_standard_pot",
                2 | 3 => "opt_nernst",
                _ => "opt_faraday",
            },
            match difficulty_level {
                1 => "model_standard",
                2 | 3 => "model_nernst",
                _ => "model_faraday",
            },
            "Determine whether the problem concerns cell voltage (Nernst) or electrolytic mass/charge transfer (Faraday).",
        );

        let meta = ChemicalProblemMetadata::new(
            ChemicalRegimeKind::EquilibriumConstantExpression,
            ChemistryUnit::Dimensionless,
        );

        let step1 = StepNode::new(
            "identify_half_reactions",
            StepType::ConstructEquilibriumExpression,
            "Identify Half-Reactions and Parameters",
            format!("Electrons transferred n = {}, Target: {}", puzzle.n_electrons, puzzle.target_quantity),
            format!("n = {}", puzzle.n_electrons),
        )
        .with_hints(vec![
            StepHint::new(HintLevel::Principle, "Governing Principle", "Identify cathode vs anode and the number of transferred electrons n."),
            StepHint::new(HintLevel::Operation, "Parameter Extraction", format!("Determine n = {} for the redox half-reaction.", puzzle.n_electrons)),
            StepHint::new(HintLevel::IntermediateRelation, "Reaction Stoichiometry", format!("Redox valence factor z = {}", puzzle.n_electrons)),
        ]);

        let step2 = StepNode::new(
            "apply_electrochemical_formula",
            StepType::ConvertMassToMoles,
            "Apply Governing Formula",
            "Calculate intermediate potential or charge transfer Q = I × t.",
            format!("Target = {:.3} {}", puzzle.correct_value, puzzle.unit_symbol),
        )
        .with_dependencies(vec!["identify_half_reactions".to_string()]);

        let step3 = StepNode::new(
            "compute_final_value",
            StepType::FinalAnswer,
            "Compute Final Value",
            format!("{:.3} {}", puzzle.correct_value, puzzle.unit_symbol),
            format!("{:.3}", puzzle.correct_value),
        )
        .with_dependencies(vec!["apply_electrochemical_formula".to_string()])
        .as_final();

        let solution_graph = SolutionGraph::new(vec![step1, step2, step3], "compute_final_value");

        let parameters = json!({
            "difficulty": difficulty_level,
            "target_quantity": puzzle.target_quantity,
            "n_electrons": puzzle.n_electrons,
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

        let instance_id = format!("inst-ec-l{}-{}", difficulty_level, seed);

        ProblemInstance::new(
            instance_id,
            FAMILY_CHEMISTRY_ELECTROCHEMISTRY,
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
            "generator": TEMPLATE_CHEMISTRY_ELECTROCHEMISTRY_V1,
        }))
    }
}

impl ProblemGenerator for ElectrochemistryGenerator {
    fn family_id(&self) -> &str {
        FAMILY_CHEMISTRY_ELECTROCHEMISTRY
    }

    fn template_ref(&self) -> &str {
        TEMPLATE_CHEMISTRY_ELECTROCHEMISTRY_V1
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

/// Validator for Electrochemistry problems.
pub struct ElectrochemistryValidator;

impl ProblemValidator for ElectrochemistryValidator {
    fn family_id(&self) -> &str {
        FAMILY_CHEMISTRY_ELECTROCHEMISTRY
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

            let is_correct = rel_diff <= 0.04 || (val - expected_val).abs() <= 0.03;

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
                    format!("Incorrect electrochemical value. Submitted {:.3}, expected {:.3} {}.", val, expected_val, unit),
                )
                .with_parsed_values(val, expected_val)
            }
        } else {
            AnswerEvaluation::incorrect(
                ErrorCategory::Syntax,
                "Could not parse numeric answer from submission.".to_string(),
            )
        }
    }
}
