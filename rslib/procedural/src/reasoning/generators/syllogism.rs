// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use serde_json::json;

use crate::core::{ProblemFamilyId, Result};
use crate::diagnostics::ErrorCategory;
use crate::problems::generator::ProblemGenerator;
use crate::problems::steps::{HintLevel, SolutionGraph, StepHint, StepNode, StepType};
use crate::problems::validator::{AnswerEvaluation, ProblemValidator};
use crate::problems::ProblemInstance;
use crate::reasoning::generators::{FAMILY_REASONING_SYLLOGISM, TEMPLATE_REASONING_SYLLOGISM_V1};
use crate::core::decision::{CognitiveDecisionPoint, DecisionOption};
use crate::reasoning::models::{ReasoningProblemMetadata, SchemaKind, StrategyKind};
use crate::reasoning::syllogism::SyllogismProblem;

/// Generator for Categorical Syllogism formal logic problems.
pub struct SyllogismGenerator;

const NOUN_POOL: &[&str] = &[
    "cats", "mammals", "animals", "sparrows", "birds", "vertebrates",
    "dolphins", "cetaceans", "creatures", "lions", "carnivores", "predators",
    "frogs", "amphibians", "reptiles", "eagles", "raptors", "fish",
    "bees", "insects", "arthropods", "snakes", "roses", "flowers",
    "plants", "oaks", "trees", "flora", "apples", "fruits",
    "foods", "tulips", "perennials", "carrots", "vegetables", "roots",
    "poets", "writers", "artists", "engineers", "professionals", "graduates",
    "surgeons", "doctors", "specialists", "pianists", "musicians", "performers",
    "architects", "designers", "builders", "chemists", "scientists", "researchers",
    "judges", "lawyers", "authorities", "rectangles", "quadrilaterals", "polygons",
    "cubes", "polyhedra", "solids", "circles", "curves", "metals",
    "conductors", "elements", "acids", "electrolytes", "compounds", "planets",
    "stars", "novels", "books", "publications", "routers", "devices",
];

impl SyllogismGenerator {
    pub fn generate_problem(seed: u64, difficulty_level: u32, variant: Option<&str>) -> ProblemInstance {
        let mut rng = StdRng::seed_from_u64(seed);

        let mut sample_pool = NOUN_POOL.to_vec();
        sample_pool.shuffle(&mut rng);
        let term_a = sample_pool[0];
        let term_b = sample_pool[1];
        let term_c = sample_pool[2];

        let is_strategy_drill = variant == Some("strategy_drill") || variant == Some("decision_point");

        // Mode determines which classical form to generate to balance conclusions
        let mode = rng.random_range(0..6);

        let prob = match mode {
            0 => SyllogismProblem::create_barbara(term_a, term_b, term_c), // Both follow
            1 => SyllogismProblem::create_celarent(term_a, term_b, term_c), // Only I follows
            2 => SyllogismProblem::create_darii(term_a, term_b, term_c), // Only I follows
            3 => SyllogismProblem::create_only_two_follows(term_a, term_b, term_c), // Only II follows
            4 => SyllogismProblem::create_ferio(term_a, term_b, term_c), // Only I follows
            _ => SyllogismProblem::create_disjoint_some(term_a, term_b, term_c), // Neither follows
        };

        let premises_formatted: Vec<String> = prob
            .premises
            .iter()
            .enumerate()
            .map(|(i, p)| format!("{}. {}", i + 1, p.statement()))
            .collect();

        let conclusions_formatted: Vec<String> = prob
            .conclusions
            .iter()
            .map(|c| format!("**Conclusion {}:** {}", if c.id == 1 { "I" } else { "II" }, c.proposition.statement()))
            .collect();

        let prompt = format!(
            "Given the following statements, determine which conclusion(s) follow logically from the premises:\n\n\
            **Statements:**\n{}\n\n\
            **Conclusions:**\n{}\n\n\
            **Options:**\n\
            A. Only I follows\n\
            B. Only II follows\n\
            C. Both I and II follow\n\
            D. Neither follows",
            premises_formatted.join("\n"),
            conclusions_formatted.join("\n")
        );

        let dp = CognitiveDecisionPoint::new(
            "dp_syllogism_strategy",
            "What is the most reliable strategy for evaluating categorical syllogisms?",
            vec![
                DecisionOption::new(
                    "opt_euler",
                    "Draw or conceptualize Venn/Euler set containment and disjointness diagrams",
                    StrategyKind::DirectSyllogisticDeduction.as_str(),
                    true,
                    "Euler diagrams provide deterministic verification of necessary truth vs contingent overlap.",
                ),
                DecisionOption::new(
                    "opt_assume",
                    "Assume 'Some' implies 'All'",
                    StrategyKind::EliminateInvalid.as_str(),
                    false,
                    "Fallacy: 'Some' means at least one, not all.",
                ),
            ],
            "opt_euler",
            StrategyKind::DirectSyllogisticDeduction.as_str(),
            "Use set containment (A ⊆ B) and disjointness (A ∩ B = ∅) rules.",
        );

        let mut meta = ReasoningProblemMetadata::new(
            SchemaKind::CategoricalSyllogism,
            StrategyKind::DirectSyllogisticDeduction,
        )
        .with_decision_point(dp)
        .with_constraint_count(prob.premises.len());

        if is_strategy_drill {
            meta = meta.as_strategy_drill();
        }

        let step1 = StepNode::new(
            "model_premises",
            StepType::BuildRepresentation,
            "Model Set Relations",
            "Represent premises as set inclusions, overlaps, and disjoint regions.",
            prob.premises[0].statement(),
        )
        .with_hints(vec![
            StepHint::new(HintLevel::Principle, "Formal Logic Principle", "A conclusion is valid ONLY if it holds in EVERY possible Venn/Euler configuration."),
            StepHint::new(HintLevel::Operation, "Strategy Operation", "Test whether the premises force the conclusion or permit counter-examples."),
            StepHint::new(HintLevel::IntermediateRelation, "Truth Check", &prob.canonical_answer),
        ]);

        let step2 = StepNode::new(
            "evaluate_conclusions",
            StepType::FinalAnswer,
            "Evaluate Conclusions",
            "Check truth of conclusions against set model.",
            prob.canonical_answer.clone(),
        )
        .with_alternates(vec![
            prob.canonical_answer.to_lowercase(),
            match prob.canonical_answer.as_str() {
                "Only I follows" => "A".to_string(),
                "Only II follows" => "B".to_string(),
                "Both I and II follow" => "C".to_string(),
                _ => "D".to_string(),
            },
        ])
        .with_dependencies(vec!["model_premises".to_string()])
        .as_final();

        let graph = SolutionGraph::new(vec![step1, step2], "evaluate_conclusions");

        let parameters = json!({
            "difficulty": difficulty_level,
            "canonical_answer": prob.canonical_answer,
            "terms": [term_a, term_b, term_c],
            "reasoning_metadata": meta,
        });

        let correct_answer = json!({
            "value": prob.canonical_answer,
            "formatted": prob.canonical_answer,
            "option": match prob.canonical_answer.as_str() {
                "Only I follows" => "A",
                "Only II follows" => "B",
                "Both I and II follow" => "C",
                _ => "D",
            },
            "solution": prob.explanation,
        });

        let instance_id = format!("inst-syllogism-l{}-{}", difficulty_level, seed);

        ProblemInstance::new(
            instance_id,
            FAMILY_REASONING_SYLLOGISM,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(json!({
            "difficulty_level": difficulty_level,
            "target_time_ms": 30_000,
            "domain": "reasoning",
            "generator": TEMPLATE_REASONING_SYLLOGISM_V1,
        }))
    }
}

impl ProblemGenerator for SyllogismGenerator {
    fn family_id(&self) -> &str {
        FAMILY_REASONING_SYLLOGISM
    }

    fn template_ref(&self) -> &str {
        TEMPLATE_REASONING_SYLLOGISM_V1
    }

    fn supported_variants(&self) -> Vec<String> {
        vec![
            "barbara_aaa".to_string(),
            "celarent_eae".to_string(),
            "darii_aii".to_string(),
            "disjoint_invalid".to_string(),
            "strategy_drill".to_string(),
        ]
    }

    fn target_latency_ms(&self, difficulty_level: u32) -> u64 {
        match difficulty_level {
            1 => 20_000,
            2 => 25_000,
            3 => 30_000,
            4 => 35_000,
            _ => 40_000,
        }
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

/// Validator for Categorical Syllogism problems.
pub struct SyllogismValidator;

impl ProblemValidator for SyllogismValidator {
    fn family_id(&self) -> &str {
        FAMILY_REASONING_SYLLOGISM
    }

    fn evaluate(
        &self,
        instance: &ProblemInstance,
        student_answer: &serde_json::Value,
        time_taken_ms: u64,
        target_time_ms: u64,
    ) -> AnswerEvaluation {
        let expected_str = instance
            .correct_answer
            .get("formatted")
            .or_else(|| instance.correct_answer.get("value"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let student_str = match student_answer {
            serde_json::Value::String(s) => s.trim().to_string(),
            serde_json::Value::Object(map) => {
                map.get("formatted")
                    .or_else(|| map.get("value"))
                    .or_else(|| map.get("option"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .trim()
                    .to_string()
            }
            _ => "".to_string(),
        };

        let clean_sub = student_str.to_lowercase().replace('_', " ");
        let clean_exp = expected_str.to_lowercase().replace('_', " ");

        let is_correct = clean_sub == clean_exp
            || (clean_exp.contains("only i follows") && (clean_sub == "a" || clean_sub == "only i" || clean_sub == "1"))
            || (clean_exp.contains("only ii follows") && (clean_sub == "b" || clean_sub == "only ii" || clean_sub == "2"))
            || (clean_exp.contains("both i and ii follow") && (clean_sub == "c" || clean_sub == "both" || clean_sub == "both follow"))
            || (clean_exp.contains("neither follows") && (clean_sub == "d" || clean_sub == "neither" || clean_sub == "none"));

        if is_correct {
            AnswerEvaluation::correct(1.0, time_taken_ms, target_time_ms)
        } else {
            AnswerEvaluation::incorrect(
                ErrorCategory::Concept,
                format!(
                    "Incorrect deduction. Submitted '{}', expected '{}'.",
                    student_str, expected_str
                ),
            )
        }
    }
}
