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
use crate::reasoning::generators::{FAMILY_REASONING_RELATIONS, TEMPLATE_REASONING_RELATIONS_V1};
use crate::reasoning::models::{CognitiveDecisionPoint, DecisionOption, ReasoningProblemMetadata, SchemaKind, StrategyKind};
use crate::reasoning::relations::{BloodRelationPuzzle, DirectionPuzzle};

/// Generator for Blood Relations and Direction Sense relational graph problems.
pub struct RelationsGenerator;

const FAMILY_NAMES: &[(&str, &str, &str)] = &[
    ("Rohan", "Priya", "Amit"),
    ("Vikram", "Sunita", "Rahul"),
    ("Arjun", "Kavita", "Ananya"),
    ("Rajesh", "Meena", "Karan"),
];

impl RelationsGenerator {
    pub fn generate_problem(seed: u64, difficulty_level: u32, variant: Option<&str>) -> ProblemInstance {
        let mut rng = StdRng::seed_from_u64(seed);
        let name_idx = rng.random_range(0..FAMILY_NAMES.len());
        let (p_a, p_b, p_c) = FAMILY_NAMES[name_idx];

        let is_strategy_drill = variant == Some("strategy_drill") || variant == Some("decision_point");

        // Alternate between Blood Relations and Direction Sense based on difficulty/seed
        if difficulty_level == 2 || difficulty_level == 4 {
            let d1 = rng.random_range(8..=15);
            let d2 = rng.random_range(3..=8);
            let d3 = rng.random_range(2..=d1 - 1);
            let dir_prob = DirectionPuzzle::create_path(d1, d2, d3);
            Self::build_direction_instance(seed, dir_prob, difficulty_level, is_strategy_drill)
        } else {
            let blood_prob = if difficulty_level >= 3 {
                BloodRelationPuzzle::create_grandfather_chain(p_a, p_b, p_c)
            } else {
                BloodRelationPuzzle::create_uncle_chain(p_a, p_b, p_c)
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
        let stmts_text: Vec<String> = prob.statements.iter().map(|s| s.text()).collect();
        let prompt = format!(
            "{}\n\n**Question:** How is {} related to {}?",
            stmts_text.join(" "),
            prob.query_from,
            prob.query_to
        );

        let dp = CognitiveDecisionPoint::new(
            "dp_kinship_graph",
            "What representation is best suited for multi-step kinship relations?",
            vec![
                DecisionOption::new(
                    "opt_tree",
                    "Construct a generational family tree with gender and lineage markers",
                    StrategyKind::ConstructKinshipGraph,
                    true,
                    "Generational graphs prevent confusion between maternal and paternal lines.",
                ),
                DecisionOption::new(
                    "opt_invert",
                    "Invert the relationship subject and object randomly",
                    StrategyKind::EliminateInvalid,
                    false,
                    "Inversion error: A related to C is opposite to C related to A.",
                ),
            ],
            "opt_tree",
            StrategyKind::ConstructKinshipGraph,
            "Build family tree graph tracking generational levels (+1 for parents, 0 for siblings).",
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
            StepHint::new(HintLevel::Operation, "Strategy Operation", format!("Identify the relation between {} and {}.", prob.statements[0].person_a, prob.statements[0].person_b)),
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
            rel_str.replace("Maternal ", "").to_lowercase(),
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
            "formatted": rel_str.clone(),
            "solution": prob.explanation,
        });

        ProblemInstance::new(
            format!("inst-reas-rel-{}", seed),
            FAMILY_REASONING_RELATIONS,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(json!({
            "difficulty_level": difficulty,
            "target_time_ms": 25_000,
            "domain": "reasoning",
        }))
    }

    fn build_direction_instance(
        seed: u64,
        prob: DirectionPuzzle,
        difficulty: u32,
        is_strategy_drill: bool,
    ) -> ProblemInstance {
        let narrative_text = prob.narrative.join(" ");
        let prompt = format!(
            "{}\n\n**Question:** In which direction is the person from their starting point?",
            narrative_text
        );

        let dp = CognitiveDecisionPoint::new(
            "dp_direction_vector",
            "What strategy simplifies multi-turn direction path problems?",
            vec![
                DecisionOption::new(
                    "opt_vector",
                    "Track independent horizontal (East-West) and vertical (North-South) coordinate sums",
                    StrategyKind::TraceDirectionVectors,
                    true,
                    "Orthogonal coordinate projection allows exact displacement calculation.",
                ),
            ],
            "opt_vector",
            StrategyKind::TraceDirectionVectors,
            "Sum vector components: Net X = East - West, Net Y = North - South.",
        );

        let mut meta = ReasoningProblemMetadata::new(SchemaKind::DirectionSense, StrategyKind::TraceDirectionVectors)
            .with_decision_point(dp);

        if is_strategy_drill {
            meta = meta.as_strategy_drill();
        }

        let dir_str = prob.final_direction_from_start.as_str().to_string();

        let step1 = StepNode::new(
            "track_displacements",
            StepType::BuildRepresentation,
            "Track 2D Coordinates",
            "Sum vertical (North/South) and horizontal (East/West) movements.",
            format!("X = {}m, Y = {}m", prob.final_x, prob.final_y),
        );

        let step2 = StepNode::new(
            "final_direction",
            StepType::FinalAnswer,
            "Determine Final Direction",
            "Determine quadrant orientation from origin.",
            dir_str.clone(),
        )
        .with_alternates(vec![
            dir_str.to_lowercase(),
            dir_str.replace('-', " ").to_lowercase(),
        ])
        .with_dependencies(vec!["track_displacements".to_string()])
        .as_final();

        let graph = SolutionGraph::new(vec![step1, step2], "final_direction");

        let parameters = json!({
            "difficulty": difficulty,
            "final_x": prob.final_x,
            "final_y": prob.final_y,
            "direction": dir_str,
            "reasoning_metadata": meta,
        });

        let correct_answer = json!({
            "value": dir_str,
            "formatted": dir_str.clone(),
            "solution": prob.explanation,
        });

        ProblemInstance::new(
            format!("inst-reas-dir-{}", seed),
            FAMILY_REASONING_RELATIONS,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(json!({
            "difficulty_level": difficulty,
            "target_time_ms": 30_000,
            "domain": "reasoning",
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
            "direction_displacement".to_string(),
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

/// Validator for Blood Relations and Direction Sense problems.
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
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let student_str = match student_answer {
            serde_json::Value::String(s) => s.trim().to_string(),
            _ => "".to_string(),
        };

        let clean_sub = student_str.to_lowercase().replace('-', " ");
        let clean_exp = expected_str.to_lowercase().replace('-', " ");

        let is_correct = clean_sub == clean_exp
            || (clean_exp.contains("uncle") && (clean_sub == "uncle" || clean_sub == "maternal uncle"))
            || (clean_exp.contains("grandfather") && (clean_sub == "grandfather" || clean_sub == "grand father"))
            || clean_sub.contains(&clean_exp);

        if is_correct {
            AnswerEvaluation::correct(1.0, time_taken_ms, target_time_ms)
        } else {
            AnswerEvaluation::incorrect(
                ErrorCategory::Concept,
                format!(
                    "Incorrect relational inference. Submitted '{}', expected '{}'.",
                    student_str, expected_str
                ),
            )
        }
    }
}
