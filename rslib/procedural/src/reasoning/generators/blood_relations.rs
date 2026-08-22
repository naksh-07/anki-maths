// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use serde_json::json;

use crate::core::{ProblemFamilyId, Result};
use crate::core::decision::{CognitiveDecisionPoint, DecisionOption};
use crate::diagnostics::ErrorCategory;
use crate::problems::generator::ProblemGenerator;
use crate::problems::steps::{HintLevel, SolutionGraph, StepHint, StepNode, StepType};
use crate::problems::validator::{AnswerEvaluation, ProblemValidator};
use crate::problems::ProblemInstance;
use crate::reasoning::models::{ReasoningProblemMetadata, SchemaKind, StrategyKind};
use crate::reasoning::relations::BloodRelationPuzzle;

pub const FAMILY_REASONING_BLOOD_RELATIONS: &str = "family.reasoning.blood_relations";
pub const TEMPLATE_REASONING_BLOOD_RELATIONS_V1: &str = "reasoning.blood_relations.v1";

/// Generator for Blood Relations kinship graph inference problems.
pub struct BloodRelationsGenerator;

const NAME_POOL: &[&str] = &[
    "Rohan", "Priya", "Amit", "Kavita", "Vikram", "Sunita", "Rahul", "Ananya",
    "Arjun", "Dev", "Rajesh", "Meena", "Karan", "Pooja", "Suresh", "Lakshmi",
    "Vijay", "Deepa", "Aditya", "Neha", "Rishi", "Tanvi", "Manoj", "Shweta",
    "Gaurav", "Nisha", "Kunal", "Divya", "Siddharth", "Rhea", "Alok", "Simran",
];

impl BloodRelationsGenerator {
    pub fn target_latency(difficulty_level: u32) -> u64 {
        match difficulty_level {
            1 => 20_000,
            2 => 25_000,
            3 => 30_000,
            4 => 35_000,
            _ => 40_000,
        }
    }

    pub fn generate_problem(seed: u64, difficulty_level: u32, variant: Option<&str>) -> ProblemInstance {
        let mut rng = StdRng::seed_from_u64(seed);

        let mut names = NAME_POOL.to_vec();
        names.shuffle(&mut rng);
        let p_a = names[0];
        let p_b = names[1];
        let p_c = names[2];
        let p_d = names[3];

        let is_strategy_drill = variant == Some("strategy_drill") || variant == Some("decision_point");

        let puzzle = if let Some(v) = variant {
            match v {
                "uncle_chain" => BloodRelationPuzzle::create_uncle_chain(p_a, p_b, p_c),
                "aunt_chain" => BloodRelationPuzzle::create_aunt_chain(p_a, p_b, p_c),
                "nephew_chain" => BloodRelationPuzzle::create_nephew_chain(p_a, p_b, p_c),
                "niece_chain" => BloodRelationPuzzle::create_niece_chain(p_a, p_b, p_c),
                "grandfather_chain" => BloodRelationPuzzle::create_grandfather_chain(p_a, p_b, p_c),
                "grandmother_chain" => BloodRelationPuzzle::create_grandmother_chain(p_a, p_b, p_c),
                "cousin_chain" => BloodRelationPuzzle::create_cousin_chain(p_a, p_b, p_c, p_d),
                "multi_hop_chain" => BloodRelationPuzzle::create_multi_hop_chain(p_a, p_b, p_c, p_d),
                _ => match difficulty_level {
                    1 => {
                        if rng.random_bool(0.5) {
                            BloodRelationPuzzle::create_uncle_chain(p_a, p_b, p_c)
                        } else {
                            BloodRelationPuzzle::create_aunt_chain(p_a, p_b, p_c)
                        }
                    }
                    2 => {
                        if rng.random_bool(0.5) {
                            BloodRelationPuzzle::create_nephew_chain(p_a, p_b, p_c)
                        } else {
                            BloodRelationPuzzle::create_niece_chain(p_a, p_b, p_c)
                        }
                    }
                    3 => {
                        if rng.random_bool(0.5) {
                            BloodRelationPuzzle::create_grandfather_chain(p_a, p_b, p_c)
                        } else {
                            BloodRelationPuzzle::create_grandmother_chain(p_a, p_b, p_c)
                        }
                    }
                    4 => BloodRelationPuzzle::create_cousin_chain(p_a, p_b, p_c, p_d),
                    _ => BloodRelationPuzzle::create_multi_hop_chain(p_a, p_b, p_c, p_d),
                },
            }
        } else {
            match difficulty_level {
                1 => {
                    if rng.random_bool(0.5) {
                        BloodRelationPuzzle::create_uncle_chain(p_a, p_b, p_c)
                    } else {
                        BloodRelationPuzzle::create_aunt_chain(p_a, p_b, p_c)
                    }
                }
                2 => {
                    if rng.random_bool(0.5) {
                        BloodRelationPuzzle::create_nephew_chain(p_a, p_b, p_c)
                    } else {
                        BloodRelationPuzzle::create_niece_chain(p_a, p_b, p_c)
                    }
                }
                3 => {
                    if rng.random_bool(0.5) {
                        BloodRelationPuzzle::create_grandfather_chain(p_a, p_b, p_c)
                    } else {
                        BloodRelationPuzzle::create_grandmother_chain(p_a, p_b, p_c)
                    }
                }
                4 => BloodRelationPuzzle::create_cousin_chain(p_a, p_b, p_c, p_d),
                _ => BloodRelationPuzzle::create_multi_hop_chain(p_a, p_b, p_c, p_d),
            }
        };

        let target_time_ms = Self::target_latency(difficulty_level);
        let stmts_text: Vec<String> = puzzle.statements.iter().map(|s| s.text()).collect();

        let scaffold = match difficulty_level {
            1 => "\n\n**Kinship Graph (Explicit Scaffold):**\n- Generation +1 (Parents/Aunts/Uncles): [___]\n- Generation 0 (Siblings/Reference): [___]\n- Generation -1 (Children/Nephews/Nieces): [___]".to_string(),
            2 => format!("\n\n**Relational Bridge (Partial Scaffold):**\n[ {} -> Intermediate Link -> {} ]", puzzle.query_from, puzzle.query_to),
            _ => String::new(),
        };

        let prompt = format!(
            "{}{}\n\n**Question:** How is **{}** related to **{}**?",
            stmts_text.join(" "),
            scaffold,
            puzzle.query_from,
            puzzle.query_to
        );

        let dp = CognitiveDecisionPoint::new(
            "dp_kinship_graph",
            "Which strategy should you use to map out these family relations?",
            vec![
                DecisionOption::new(
                    "opt_kinship",
                    "Draw a family tree graph starting from a reference person",
                    StrategyKind::ConstructKinshipGraph.as_str(),
                    true,
                    "Correct: A kinship graph is the most robust way to resolve indirect relations.",
                ),
                DecisionOption::new(
                    "opt_branch",
                    "Try to mentally guess relations without a tree",
                    StrategyKind::BranchCases.as_str(),
                    false,
                    "Sub-optimal: Mental guessing is highly error-prone across generational steps.",
                ),
            ],
            "opt_kinship",
            StrategyKind::ConstructKinshipGraph.as_str(),
            "Blood relations are best solved by systematically constructing a family tree graph.",
        );

        let mut meta = ReasoningProblemMetadata::new(SchemaKind::BloodRelations, StrategyKind::ConstructKinshipGraph)
            .with_decision_point(dp)
            .with_constraint_count(puzzle.statements.len())
            .with_scaffolding_level(if difficulty_level <= 2 { 3 - difficulty_level } else { 0 })
            .with_constraint_density(puzzle.statements.len() as f64 / 3.0)
            .with_branching_factor(if difficulty_level >= 4 { 2 } else { 1 })
            .with_search_depth(difficulty_level as usize);

        if is_strategy_drill {
            meta = meta.as_strategy_drill();
        }

        let rel_str = puzzle.target_relation.as_str().to_string();

        let step1 = StepNode::new(
            "build_kinship_tree",
            StepType::BuildRepresentation,
            "Build Family Graph",
            "Connect nodes through parent-child and sibling edges.",
            puzzle.statements[0].text(),
        )
        .with_hints(vec![
            StepHint::new(HintLevel::Principle, "Kinship Principle", "Trace each relationship step-by-step from the reference person."),
            StepHint::new(HintLevel::Operation, "Strategy Operation", format!("Identify relation between {} and {}.", puzzle.statements[0].person_a, puzzle.statements[0].person_b)),
            StepHint::new(HintLevel::IntermediateRelation, "Relation Setup", &rel_str),
        ]);

        let step2 = StepNode::new(
            "deduce_relation",
            StepType::FinalAnswer,
            "Identify Relationship",
            format!("Deduce relation of {} to {}.", puzzle.query_from, puzzle.query_to),
            rel_str.clone(),
        )
        .with_alternates(vec![
            rel_str.to_lowercase(),
            rel_str.to_uppercase(),
            rel_str.replace("Maternal ", "").to_lowercase(),
            rel_str.replace("Paternal ", "").to_lowercase(),
        ])
        .with_dependencies(vec!["build_kinship_tree".to_string()])
        .as_final();

        let graph = SolutionGraph::new(vec![step1, step2], "deduce_relation");

        let parameters = json!({
            "difficulty": difficulty_level,
            "target_relation": rel_str,
            "query_from": puzzle.query_from,
            "query_to": puzzle.query_to,
            "reasoning_metadata": meta,
        });

        let correct_answer = json!({
            "value": rel_str,
            "formatted": rel_str,
            "solution": puzzle.explanation,
        });

        let instance_id = format!("inst-blood-rel-l{}-{}", difficulty_level, seed);

        ProblemInstance::new(
            instance_id,
            FAMILY_REASONING_BLOOD_RELATIONS,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(json!({
            "difficulty_level": difficulty_level,
            "target_time_ms": target_time_ms,
            "domain": "reasoning",
            "generator": TEMPLATE_REASONING_BLOOD_RELATIONS_V1,
        }))
    }
}

impl ProblemGenerator for BloodRelationsGenerator {
    fn family_id(&self) -> &str {
        FAMILY_REASONING_BLOOD_RELATIONS
    }

    fn template_ref(&self) -> &str {
        TEMPLATE_REASONING_BLOOD_RELATIONS_V1
    }

    fn supported_variants(&self) -> Vec<String> {
        vec![
            "uncle_chain".to_string(),
            "aunt_chain".to_string(),
            "nephew_chain".to_string(),
            "niece_chain".to_string(),
            "grandfather_chain".to_string(),
            "grandmother_chain".to_string(),
            "cousin_chain".to_string(),
            "multi_hop_chain".to_string(),
            "strategy_drill".to_string(),
        ]
    }

    fn target_latency_ms(&self, difficulty_level: u32) -> u64 {
        Self::target_latency(difficulty_level)
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

/// Validator for Blood Relations problems.
pub struct BloodRelationsValidator;

impl ProblemValidator for BloodRelationsValidator {
    fn family_id(&self) -> &str {
        FAMILY_REASONING_BLOOD_RELATIONS
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
                    .or_else(|| map.get("answer"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .trim()
                    .to_string()
            }
            _ => "".to_string(),
        };

        let clean_sub = student_str.to_lowercase().replace('-', " ").replace('_', " ");
        let clean_exp = expected_str.to_lowercase().replace('-', " ").replace('_', " ");

        let is_correct = clean_sub == clean_exp
            || (clean_exp.contains("uncle") && clean_sub.contains("uncle"))
            || (clean_exp.contains("aunt") && clean_sub.contains("aunt"))
            || (clean_exp.contains("grandfather") && clean_sub.contains("grandfather"))
            || (clean_exp.contains("grandmother") && clean_sub.contains("grandmother"))
            || (clean_exp.contains("cousin") && clean_sub.contains("cousin"))
            || (clean_exp.contains("nephew") && clean_sub.contains("nephew"))
            || (clean_exp.contains("niece") && clean_sub.contains("niece"));

        if is_correct {
            AnswerEvaluation::correct(1.0, time_taken_ms, target_time_ms)
        } else {
            AnswerEvaluation::incorrect(
                ErrorCategory::Concept,
                format!(
                    "Incorrect kinship relationship. Submitted '{}', expected '{}'.",
                    student_str, expected_str
                ),
            )
        }
    }
}
