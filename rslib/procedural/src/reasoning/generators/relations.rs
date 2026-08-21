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
use crate::reasoning::generators::{FAMILY_REASONING_RELATIONS, TEMPLATE_REASONING_RELATIONS_V1};
use crate::core::decision::{CognitiveDecisionPoint, DecisionOption};
use crate::reasoning::models::{ReasoningProblemMetadata, SchemaKind, StrategyKind};
use crate::reasoning::relations::{BloodRelationPuzzle, DirectionPuzzle};

/// Generator for Blood Relations and Direction Sense relational graph problems.
pub struct RelationsGenerator;

const NAME_POOL: &[&str] = &[
    "Rohan", "Priya", "Amit", "Kavita", "Vikram", "Sunita", "Rahul", "Ananya",
    "Arjun", "Dev", "Rajesh", "Meena", "Karan", "Pooja", "Suresh", "Lakshmi",
    "Vijay", "Deepa", "Aditya", "Neha", "Rishi", "Tanvi", "Manoj", "Shweta",
    "Gaurav", "Nisha", "Kunal", "Divya", "Siddharth", "Rhea", "Alok", "Simran",
];

impl RelationsGenerator {
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

        // Alternate between Blood Relations and Direction Sense
        if difficulty_level == 2 || difficulty_level == 4 {
            let d1 = rng.random_range(6..=50);
            let d2 = rng.random_range(4..=40);
            let d3 = rng.random_range(2..=(d1 + 20));
            let dir_prob = DirectionPuzzle::create_path(d1, d2, d3);
            Self::build_direction_instance(seed, dir_prob, difficulty_level, is_strategy_drill)
        } else {
            let blood_mode = rng.random_range(0..5);
            let blood_prob = match blood_mode {
                0 => BloodRelationPuzzle::create_uncle_chain(p_a, p_b, p_c),
                1 => BloodRelationPuzzle::create_grandfather_chain(p_a, p_b, p_c),
                2 => BloodRelationPuzzle::create_cousin_chain(p_a, p_b, p_c, p_d),
                3 => BloodRelationPuzzle::create_nephew_chain(p_a, p_b, p_c),
                _ => BloodRelationPuzzle::create_aunt_chain(p_a, p_b, p_c),
            };
            Self::build_blood_relation_instance(seed, blood_prob, difficulty_level, is_strategy_drill)
        }
    }

    fn build_blood_relation_instance(
        seed: u64,
        prob: BloodRelationPuzzle,
        difficulty: u32,
        is_strategy_drill: bool,
    ) -> ProblemInstance {
        let target_time_ms = Self::target_latency(difficulty);
        let stmts_text: Vec<String> = prob.statements.iter().map(|s| s.text()).collect();
        let prompt = format!(
            "{}\n\n**Question:** How is **{}** related to **{}**?",
            stmts_text.join(" "),
            prob.query_from,
            prob.query_to
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
            .with_decision_point(dp);

        if is_strategy_drill {
            meta = meta.as_strategy_drill();
        }

        let rel_str = prob.target_relation.as_str().to_string();

        let step1 = StepNode::new(
            "build_kinship_tree",
            StepType::BuildRepresentation,
            "Build Family Graph",
            "Connect nodes through parent-child and sibling edges.",
            prob.statements[0].text(),
        )
        .with_hints(vec![
            StepHint::new(HintLevel::Principle, "Kinship Principle", "Trace each relationship step-by-step from the reference person."),
            StepHint::new(HintLevel::Operation, "Strategy Operation", format!("Identify relation between {} and {}.", prob.statements[0].person_a, prob.statements[0].person_b)),
            StepHint::new(HintLevel::IntermediateRelation, "Relation Setup", &rel_str),
        ]);

        let step2 = StepNode::new(
            "deduce_relation",
            StepType::FinalAnswer,
            "Identify Relationship",
            format!("Deduce relation of {} to {}.", prob.query_from, prob.query_to),
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
            "difficulty": difficulty,
            "target_relation": rel_str,
            "query_from": prob.query_from,
            "query_to": prob.query_to,
            "reasoning_metadata": meta,
        });

        let correct_answer = json!({
            "value": rel_str,
            "formatted": rel_str,
            "solution": prob.explanation,
        });

        let instance_id = format!("inst-relations-l{}-{}", difficulty, seed);

        ProblemInstance::new(
            instance_id,
            FAMILY_REASONING_RELATIONS,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(json!({
            "difficulty_level": difficulty,
            "target_time_ms": target_time_ms,
            "domain": "reasoning",
            "generator": TEMPLATE_REASONING_RELATIONS_V1,
        }))
    }

    fn build_direction_instance(
        seed: u64,
        prob: DirectionPuzzle,
        difficulty: u32,
        is_strategy_drill: bool,
    ) -> ProblemInstance {
        let target_time_ms = Self::target_latency(difficulty);
        let prompt = format!(
            "A person starts from a fixed point O and performs the following walk:\n{}\n\n\
             **Question:** In which **direction** is the person located relative to the starting point?",
            prob.steps_text.iter().map(|s| format!("- {}", s)).collect::<Vec<_>>().join("\n")
        );

        let heading_str = prob.target_heading.as_str().to_string();

        let dp = CognitiveDecisionPoint::new(
            "dp_direction_vector",
            "Which method should you use to calculate the net displacement?",
            vec![
                DecisionOption::new(
                    "opt_vector",
                    "Sum horizontal (East/West) and vertical (North/South) displacement components separately",
                    StrategyKind::TraceDirectionVectors.as_str(),
                    true,
                    "Component summation (Δx, Δy) simplifies 2D trajectories into a single coordinate point.",
                ),
                DecisionOption::new(
                    "opt_sketch",
                    "Estimate the direction visually from a rough hand sketch without numbers",
                    StrategyKind::BranchCases.as_str(),
                    false,
                    "Sub-optimal: Qualitative sketches fail when distances in opposite directions are close.",
                ),
            ],
            "opt_vector",
            StrategyKind::TraceDirectionVectors.as_str(),
            "Decompose each path segment into East/West (X) and North/South (Y) components.",
        );

        let mut meta = ReasoningProblemMetadata::new(SchemaKind::DirectionSense, StrategyKind::TraceDirectionVectors)
            .with_decision_point(dp);

        if is_strategy_drill {
            meta = meta.as_strategy_drill();
        }

        let step1 = StepNode::new(
            "calc_components",
            StepType::Transformation,
            "Calculate Net Coordinate Displacement",
            format!("Δx = {} m, Δy = {} m", prob.displacement_x, prob.displacement_y),
            format!("({}, {})", prob.displacement_x, prob.displacement_y),
        )
        .with_hints(vec![
            StepHint::new(HintLevel::Principle, "2D Displacement", "Track net East (+x) / West (-x) and net North (+y) / South (-y)."),
            StepHint::new(HintLevel::Operation, "Sum Components", format!("Net Δx = {} m, Δy = {} m.", prob.displacement_x, prob.displacement_y)),
            StepHint::new(HintLevel::IntermediateRelation, "Coordinates", format!("({}, {})", prob.displacement_x, prob.displacement_y)),
        ]);

        let step2 = StepNode::new(
            "determine_direction",
            StepType::FinalAnswer,
            "Determine Cardinal/Ordinal Direction",
            format!("Vector ({}, {}) corresponds to direction {}.", prob.displacement_x, prob.displacement_y, heading_str),
            heading_str.clone(),
        )
        .with_alternates(vec![
            heading_str.to_lowercase(),
            heading_str.replace('-', "").to_lowercase(),
            heading_str.replace('-', " ").to_lowercase(),
        ])
        .with_dependencies(vec!["calc_components".to_string()])
        .as_final();

        let graph = SolutionGraph::new(vec![step1, step2], "determine_direction");

        let parameters = json!({
            "difficulty": difficulty,
            "dx": prob.displacement_x,
            "dy": prob.displacement_y,
            "straight_distance": prob.straight_distance_m,
            "target_heading": heading_str,
            "reasoning_metadata": meta,
        });

        let correct_answer = json!({
            "value": heading_str,
            "formatted": heading_str,
            "solution": prob.explanation,
        });

        let instance_id = format!("inst-direction-l{}-{}", difficulty, seed);

        ProblemInstance::new(
            instance_id,
            FAMILY_REASONING_RELATIONS,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(json!({
            "difficulty_level": difficulty,
            "target_time_ms": target_time_ms,
            "domain": "reasoning",
            "generator": TEMPLATE_REASONING_RELATIONS_V1,
        }))
    }
}

impl ProblemGenerator for RelationsGenerator {
    fn family_id(&self) -> &str {
        FAMILY_REASONING_RELATIONS
    }

    fn template_ref(&self) -> &str {
        TEMPLATE_REASONING_RELATIONS_V1
    }

    fn supported_variants(&self) -> Vec<String> {
        vec![
            "uncle_chain".to_string(),
            "grandfather_chain".to_string(),
            "cousin_chain".to_string(),
            "direction_vector_3step".to_string(),
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

/// Validator for Relational Graph and Direction Sense problems.
pub struct RelationsValidator;

impl ProblemValidator for RelationsValidator {
    fn family_id(&self) -> &str {
        FAMILY_REASONING_RELATIONS
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
            || clean_sub.contains(&clean_exp)
            || (clean_exp.contains("uncle") && clean_sub.contains("uncle"))
            || (clean_exp.contains("aunt") && clean_sub.contains("aunt"))
            || (clean_exp.contains("grandfather") && clean_sub.contains("grandfather"))
            || (clean_exp.contains("grandmother") && clean_sub.contains("grandmother"))
            || (clean_exp.contains("cousin") && clean_sub.contains("cousin"))
            || (clean_exp.contains("nephew") && clean_sub.contains("nephew"))
            || (clean_exp.contains("niece") && clean_sub.contains("niece"))
            || (clean_exp.contains("north") && clean_sub == "north")
            || (clean_exp.contains("south") && clean_sub == "south")
            || (clean_exp.contains("east") && clean_sub == "east")
            || (clean_exp.contains("west") && clean_sub == "west");

        if is_correct {
            AnswerEvaluation::correct(1.0, time_taken_ms, target_time_ms)
        } else {
            AnswerEvaluation::incorrect(
                ErrorCategory::Concept,
                format!(
                    "Incorrect relation/direction. Submitted '{}', expected '{}'.",
                    student_str, expected_str
                ),
            )
        }
    }
}
