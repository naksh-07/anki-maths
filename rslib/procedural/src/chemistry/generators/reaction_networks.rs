// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use rand::rngs::StdRng;
use rand::SeedableRng;
use serde_json::json;

use crate::chemistry::models::{ChemicalProblemMetadata, ChemicalRegimeKind};
use crate::chemistry::reaction_networks::ReactionNetworkPuzzle;
use crate::chemistry::units::ChemistryUnit;
use crate::core::{ProblemFamilyId, Result};
use crate::core::decision::{CognitiveDecisionPoint, DecisionOption};
use crate::diagnostics::ErrorCategory;
use crate::problems::generator::ProblemGenerator;
use crate::problems::steps::{HintLevel, SolutionGraph, StepHint, StepNode, StepType};
use crate::problems::validator::{AnswerEvaluation, NumericAnswerParser, ProblemValidator};
use crate::problems::ProblemInstance;

pub const FAMILY_CHEMISTRY_REACTION_NETWORKS: &str = "family.chemistry.reaction_networks.multistage";
pub const TEMPLATE_CHEMISTRY_REACTION_NETWORKS_V1: &str = "chemistry.reaction_networks.multistage.v1";

/// Generator for Multi-Stage Reaction Networks and Synthesis Pathways.
pub struct ReactionNetworksGenerator;

impl ReactionNetworksGenerator {
    pub fn generate_problem(seed: u64, difficulty_level: u32, _variant: Option<&str>) -> ProblemInstance {
        let mut rng = StdRng::seed_from_u64(seed);
        let puzzle = ReactionNetworkPuzzle::generate_dynamic(&mut rng, difficulty_level);

        let dp = CognitiveDecisionPoint::new(
            "dp_reaction_network_path",
            "What is the multi-stage conversion strategy for this reaction network?",
            vec![
                DecisionOption::new(
                    "opt_sequential_yield",
                    "Sequential Molar Transfer with Intermediate Yield Multiplication (eta_net = eta1 * eta2)",
                    "path_sequential_yield",
                    difficulty_level <= 3,
                    "Multiply fractional yields along the linear synthesis chain from initial precursor to target.",
                ),
                DecisionOption::new(
                    "opt_multistage_stoich",
                    "3-Stage Net Reaction Stoichiometry (Cancel intermediate link species)",
                    "path_multistage_net",
                    difficulty_level == 4,
                    "Sum intermediate reaction equations to find overall mole conversion factor.",
                ),
                DecisionOption::new(
                    "opt_mixture_system",
                    "Simultaneous Linear Mass/Mole Balance for Mixture Components",
                    "path_mixture_system",
                    difficulty_level >= 5,
                    "Set up simultaneous equations (x + y = mass_total, mol1 + mol2 = mol_gas) for mixture components.",
                ),
            ],
            match difficulty_level {
                1 | 2 | 3 => "opt_sequential_yield",
                4 => "opt_multistage_stoich",
                _ => "opt_mixture_system",
            },
            match difficulty_level {
                1 | 2 | 3 => "path_sequential_yield",
                4 => "path_multistage_net",
                _ => "path_mixture_system",
            },
            "Always link intermediate species and account for stage percentage yields before calculating final product mass.",
        );

        let meta = ChemicalProblemMetadata::new(
            ChemicalRegimeKind::StoichiometryMassMass,
            ChemistryUnit::Gram,
        );

        let step1 = StepNode::new(
            "trace_network_stages",
            StepType::ApplyStoichiometricRatio,
            "Trace Reaction Stages and Net Yield",
            format!("Stages: {}, Net Yield = {:.1}%", puzzle.reaction_stages.len(), puzzle.overall_yield * 100.0),
            format!("Net Yield = {:.1}%", puzzle.overall_yield * 100.0),
        )
        .with_hints(vec![
            StepHint::new(HintLevel::Principle, "Multi-Stage Principle", "Identify intermediate connecting species and multiply stage yields."),
            StepHint::new(HintLevel::Operation, "Molar Ratio Tracing", "Trace mole ratio from starting reactant to final product."),
            StepHint::new(HintLevel::IntermediateRelation, "Network Chain", puzzle.reaction_stages.join(" -> ")),
        ]);

        let step2 = StepNode::new(
            "propagate_intermediate_flux",
            StepType::ConvertMassToMoles,
            "Propagate Intermediate Molar Flux",
            "Calculate moles of target product produced from initial precursor.",
            format!("Target = {:.1} {}", puzzle.correct_mass_g, puzzle.unit_symbol),
        )
        .with_dependencies(vec!["trace_network_stages".to_string()]);

        let step3 = StepNode::new(
            "compute_final_mass",
            StepType::FinalAnswer,
            "Compute Final Target Mass / Percentage",
            format!("{:.1} {}", puzzle.correct_mass_g, puzzle.unit_symbol),
            format!("{:.1}", puzzle.correct_mass_g),
        )
        .with_dependencies(vec!["propagate_intermediate_flux".to_string()])
        .as_final();

        let solution_graph = SolutionGraph::new(vec![step1, step2, step3], "compute_final_mass");

        let parameters = json!({
            "difficulty": difficulty_level,
            "stages": puzzle.reaction_stages,
            "target_product": puzzle.target_product_name,
            "unit": puzzle.unit_symbol,
            "chemical_metadata": meta,
            "decision_point": dp,
        });

        let correct_answer = json!({
            "value": puzzle.correct_mass_g,
            "formatted": format!("{:.1} {}", puzzle.correct_mass_g, puzzle.unit_symbol),
            "unit": puzzle.unit_symbol,
            "explanation": puzzle.step_by_step_explanation,
        });

        let instance_id = format!("inst-rn-l{}-{}", difficulty_level, seed);

        ProblemInstance::new(
            instance_id,
            FAMILY_CHEMISTRY_REACTION_NETWORKS,
            seed,
            parameters,
            puzzle.question_prompt,
            correct_answer,
        )
        .with_solution_graph(solution_graph)
        .with_metadata(json!({
            "difficulty_level": difficulty_level,
            "target_time_ms": 45_000,
            "domain": "chemistry",
            "generator": TEMPLATE_CHEMISTRY_REACTION_NETWORKS_V1,
        }))
    }
}

impl ProblemGenerator for ReactionNetworksGenerator {
    fn family_id(&self) -> &str {
        FAMILY_CHEMISTRY_REACTION_NETWORKS
    }

    fn template_ref(&self) -> &str {
        TEMPLATE_CHEMISTRY_REACTION_NETWORKS_V1
    }

    fn supported_variants(&self) -> Vec<String> {
        vec!["default".to_string(), "strategy_drill".to_string()]
    }

    fn target_latency_ms(&self, _difficulty_level: u32) -> u64 {
        45_000
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

/// Validator for Reaction Networks problems.
pub struct ReactionNetworksValidator;

impl ProblemValidator for ReactionNetworksValidator {
    fn family_id(&self) -> &str {
        FAMILY_CHEMISTRY_REACTION_NETWORKS
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

            let is_correct = rel_diff <= 0.04 || (val - expected_val).abs() <= 0.2;

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
                    format!("Incorrect reaction network calculation. Submitted {:.1}, expected {:.1} {}.", val, expected_val, unit),
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
