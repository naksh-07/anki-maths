// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde_json::json;

use crate::core::{ProblemFamilyId, Result};
use crate::diagnostics::ErrorCategory;
use crate::problems::generator::ProblemGenerator;
use crate::problems::steps::{HintLevel, SolutionGraph, StepHint, StepNode, StepType};
use crate::problems::validator::{AnswerEvaluation, ProblemValidator};
use crate::problems::ProblemInstance;
use crate::reasoning::generators::{FAMILY_REASONING_SYLLOGISM, TEMPLATE_REASONING_SYLLOGISM_V1};
use crate::reasoning::models::{CognitiveDecisionPoint, DecisionOption, ReasoningProblemMetadata, SchemaKind, StrategyKind};
use crate::reasoning::syllogism::SyllogismProblem;

/// Generator for Categorical Syllogism formal logic problems.
pub struct SyllogismGenerator;

const TERM_TRIPLETS: &[(&str, &str, &str)] = &[
    ("cats", "mammals", "animals"),
    ("roses", "flowers", "plants"),
    ("poets", "writers", "artists"),
    ("sparrows", "birds", "vertebrates"),
    ("rectangles", "quadrilaterals", "polygons"),
    ("apples", "fruits", "foods"),
    ("engineers", "professionals", "graduates"),
];

impl SyllogismGenerator {
    pub fn generate_problem(seed: u64, difficulty_level: u32, variant: Option<&str>) -> ProblemInstance {
        let mut rng = StdRng::seed_from_u64(seed);
        let triplet_idx = rng.random_range(0..TERM_TRIPLETS.len());
        let (term_a, term_b, term_c) = TERM_TRIPLETS[triplet_idx];

        let is_strategy_drill = variant == Some("strategy_drill") || variant == Some("decision_point");

        let prob = match difficulty_level {
            1 => SyllogismProblem::create_barbara(term_a, term_b, term_c),
            2 => SyllogismProblem::create_celarent(term_a, term_b, term_c),
            3 => SyllogismProblem::create_darii(term_a, term_b, term_c),
            4 => SyllogismProblem::create_disjoint_some(term_a, term_b, term_c),
            _ => SyllogismProblem::create_celarent(term_a, term_b, term_c),
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
                    StrategyKind::DirectSyllogisticDeduction,
                    true,
                    "Euler diagrams provide deterministic verification of necessary truth vs contingent overlap.",
                ),
                DecisionOption::new(
                    "opt_assume",
                    "Assume 'Some' implies 'All'",
                    StrategyKind::EliminateInvalid,
                    false,
                    "Fallacy: 'Some' means at least one, not all.",
                ),
            ],
            "opt_euler",
            StrategyKind::DirectSyllogisticDeduction,
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
            "reasoning_metadata": meta,
        });

        let correct_answer = json!({
            "value": prob.canonical_answer,
            "formatted": prob.canonical_answer.clone(),
            "solution": prob.explanation,
        });

        ProblemInstance::new(
            format!("inst-reas-syl-{}", seed),
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
            "barbara_all_all".to_string(),
            "celarent_all_no".to_string(),
            "darii_some_all".to_string(),
            "disjoint_some_some".to_string(),
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
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let student_str = match student_answer {
            serde_json::Value::String(s) => s.trim().to_string(),
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
