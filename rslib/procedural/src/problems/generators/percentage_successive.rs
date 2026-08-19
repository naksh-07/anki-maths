// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

use crate::core::{ProblemFamilyId, Result};
use crate::core::decision::{CognitiveDecisionPoint, DecisionOption};
use crate::problems::generator::ProblemGenerator;
use crate::problems::steps::{SolutionGraph, StepHint, StepNode, StepType};
use crate::problems::ProblemInstance;

/// Direction of percentage change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeDirection {
    Increase,
    Decrease,
}

impl ChangeDirection {
    pub fn sign(&self) -> f64 {
        match self {
            ChangeDirection::Increase => 1.0,
            ChangeDirection::Decrease => -1.0,
        }
    }

    pub fn verb(&self) -> &'static str {
        match self {
            ChangeDirection::Increase => "increased",
            ChangeDirection::Decrease => "decreased",
        }
    }

    pub fn noun(&self) -> &'static str {
        match self {
            ChangeDirection::Increase => "increase",
            ChangeDirection::Decrease => "decrease",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            ChangeDirection::Increase => "+",
            ChangeDirection::Decrease => "-",
        }
    }
}

/// A single percentage change step.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PercentageStep {
    pub percent: f64,
    pub direction: ChangeDirection,
}

impl PercentageStep {
    pub fn multiplier(&self) -> f64 {
        1.0 + (self.direction.sign() * self.percent / 100.0)
    }
}

/// Variants of successive percentage problems.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PercentageVariant {
    /// Given initial value and 2 changes, find final value.
    ForwardTwoStep,
    /// Given final value and 2 changes, recover initial value.
    ReverseInitial,
    /// Given 2 changes, find the single equivalent net percentage change.
    NetEquivalentChange,
    /// Given initial value and 3 changes, find final value (higher difficulty).
    ForwardThreeStep,
}

impl PercentageVariant {
    pub fn as_str(&self) -> &'static str {
        match self {
            PercentageVariant::ForwardTwoStep => "forward_two_step",
            PercentageVariant::ReverseInitial => "reverse_initial",
            PercentageVariant::NetEquivalentChange => "net_equivalent_change",
            PercentageVariant::ForwardThreeStep => "forward_three_step",
        }
    }
}

impl std::fmt::Display for PercentageVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Generated problem payload containing parameters, prompts, answers, and solutions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeneratedPercentageProblem {
    pub variant: PercentageVariant,
    pub initial_value: f64,
    pub steps: Vec<PercentageStep>,
    pub final_value: f64,
    pub net_percentage_change: f64,
    pub target_answer_value: f64,
    pub target_unit: String,
    pub rendered_prompt: String,
    pub canonical_answer_text: String,
    pub worked_solution: String,
    pub difficulty: f64,
    pub target_time_ms: u64,
}

/// Configuration options for the successive percentage generator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PercentageSuccessiveConfig {
    pub allowed_variants: Option<Vec<PercentageVariant>>,
    pub min_difficulty: Option<f64>,
    pub max_difficulty: Option<f64>,
}

impl Default for PercentageSuccessiveConfig {
    fn default() -> Self {
        Self {
            allowed_variants: None,
            min_difficulty: Some(1.0),
            max_difficulty: Some(5.0),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PercentageSuccessiveGenerator;

impl PercentageSuccessiveGenerator {
    const CLEAN_INITIAL_VALUES: &'static [f64] = &[
        50.0, 80.0, 100.0, 120.0, 150.0, 200.0, 240.0, 250.0, 300.0, 400.0, 500.0, 600.0,
        800.0, 1000.0, 1200.0, 1500.0, 2000.0, 2400.0, 3000.0, 5000.0,
    ];

    const CLEAN_RATES_STANDARD: &'static [f64] = &[
        5.0, 10.0, 15.0, 20.0, 25.0, 30.0, 40.0, 50.0,
    ];

    /// Generate a deterministic problem instance for a specific variant from a 64-bit seed.
    pub fn generate_for_variant(
        variant: PercentageVariant,
        seed: u64,
    ) -> GeneratedPercentageProblem {
        let mut rng = StdRng::seed_from_u64(seed);
        match variant {
            PercentageVariant::ForwardTwoStep => Self::generate_forward_two_step(&mut rng),
            PercentageVariant::ReverseInitial => Self::generate_reverse_initial(&mut rng),
            PercentageVariant::NetEquivalentChange => Self::generate_net_equivalent(&mut rng),
            PercentageVariant::ForwardThreeStep => Self::generate_forward_three_step(&mut rng),
        }
    }

    /// Generate a deterministic problem instance from a 64-bit seed and config.
    pub fn generate(
        seed: u64,
        config: &PercentageSuccessiveConfig,
    ) -> GeneratedPercentageProblem {
        let mut rng = StdRng::seed_from_u64(seed);

        let variant = Self::choose_variant(&mut rng, config);
        Self::generate_for_variant(variant, seed)
    }

    /// Build a persistent ProblemInstance entity for a specific chosen variant.
    pub fn generate_instance_for_variant(
        family_id: &ProblemFamilyId,
        seed: u64,
        variant: PercentageVariant,
    ) -> ProblemInstance {
        let problem = Self::generate_for_variant(variant, seed);

        let parameters = serde_json::json!({
            "variant": problem.variant,
            "initial_value": problem.initial_value,
            "steps": problem.steps,
            "final_value": problem.final_value,
            "net_percentage_change": problem.net_percentage_change,
            "target_unit": problem.target_unit,
        });

        let correct_answer = serde_json::json!({
            "value": problem.target_answer_value,
            "formatted": problem.canonical_answer_text,
            "unit": problem.target_unit,
            "solution": problem.worked_solution,
        });

        let dp = CognitiveDecisionPoint::new(
            "dp_percentage_strategy",
            "Which strategy is most efficient for computing successive percentage changes?",
            vec![
                DecisionOption::new(
                    "opt_net",
                    "Use the net equivalent change formula (a + b + ab/100) or decimal multipliers",
                    "multiplier_or_net",
                    true,
                    "Correct: Multiplicative compounding is mathematically robust and independent of base values.",
                ),
                DecisionOption::new(
                    "opt_step",
                    "Assume a base of 100 and calculate intermediate values step by step",
                    "step_by_step",
                    true,
                    "Valid but often slower: Calculating intermediate values can lead to messy decimals.",
                ),
                DecisionOption::new(
                    "opt_add",
                    "Simply add the percentages together (a + b)",
                    "additive_fallacy",
                    false,
                    "Fallacy: Successive percentages compound, they do not simply add.",
                ),
            ],
            "opt_net",
            "multiplier_or_net",
            "Successive percentages compound multiplicatively. Always use decimal multipliers (e.g., 1.20 * 0.80) or the net change formula.",
        );

        let metadata = serde_json::json!({
            "difficulty": problem.difficulty,
            "target_time_ms": problem.target_time_ms,
            "generator": "math.percentage.successive.v1",
            "decision_point": dp,
        });

        let graph = Self::build_solution_graph(&problem);

        ProblemInstance::new(
            format!("inst-{}-{}", family_id, seed),
            family_id.clone(),
            seed,
            parameters,
            problem.rendered_prompt,
            correct_answer,
        )
        .with_metadata(metadata)
        .with_solution_graph(graph)
    }

    /// Build a persistent ProblemInstance entity from generation.
    pub fn generate_instance(
        family_id: &ProblemFamilyId,
        seed: u64,
        config: &PercentageSuccessiveConfig,
    ) -> ProblemInstance {
        let problem = Self::generate(seed, config);

        let parameters = serde_json::json!({
            "variant": problem.variant,
            "initial_value": problem.initial_value,
            "steps": problem.steps,
            "final_value": problem.final_value,
            "net_percentage_change": problem.net_percentage_change,
            "target_unit": problem.target_unit,
        });

        let correct_answer = serde_json::json!({
            "value": problem.target_answer_value,
            "formatted": problem.canonical_answer_text,
            "unit": problem.target_unit,
            "solution": problem.worked_solution,
        });

        let dp = CognitiveDecisionPoint::new(
            "dp_percentage_strategy",
            "Which strategy is most efficient for computing successive percentage changes?",
            vec![
                DecisionOption::new(
                    "opt_net",
                    "Use the net equivalent change formula (a + b + ab/100) or decimal multipliers",
                    "multiplier_or_net",
                    true,
                    "Correct: Multiplicative compounding is mathematically robust and independent of base values.",
                ),
                DecisionOption::new(
                    "opt_step",
                    "Assume a base of 100 and calculate intermediate values step by step",
                    "step_by_step",
                    true,
                    "Valid but often slower: Calculating intermediate values can lead to messy decimals.",
                ),
                DecisionOption::new(
                    "opt_add",
                    "Simply add the percentages together (a + b)",
                    "additive_fallacy",
                    false,
                    "Fallacy: Successive percentages compound, they do not simply add.",
                ),
            ],
            "opt_net",
            "multiplier_or_net",
            "Successive percentages compound multiplicatively. Always use decimal multipliers (e.g., 1.20 * 0.80) or the net change formula.",
        );

        let metadata = serde_json::json!({
            "difficulty": problem.difficulty,
            "target_time_ms": problem.target_time_ms,
            "generator": "math.percentage.successive.v1",
            "decision_point": dp,
        });

        let graph = Self::build_solution_graph(&problem);

        ProblemInstance::new(
            format!("inst-{}-{}", family_id, seed),
            family_id.clone(),
            seed,
            parameters,
            problem.rendered_prompt,
            correct_answer,
        )
        .with_metadata(metadata)
        .with_solution_graph(graph)
    }

    /// Construct a structured SolutionGraph from a generated percentage problem.
    pub fn build_solution_graph(problem: &GeneratedPercentageProblem) -> SolutionGraph {
        match problem.variant {
            PercentageVariant::ForwardTwoStep | PercentageVariant::ForwardThreeStep => {
                let mut nodes = Vec::new();
                let mut current_val = problem.initial_value;
                let mut prev_id: Option<String> = None;

                for (idx, step) in problem.steps.iter().enumerate() {
                    let next_val = (current_val * step.multiplier() * 100.0).round() / 100.0;
                    let is_last = idx == problem.steps.len() - 1;
                    let step_id = format!("step_{}_transform", idx + 1);

                    let mut node = StepNode::new(
                        &step_id,
                        if is_last { StepType::FinalAnswer } else { StepType::IntermediateResult },
                        format!("Apply {}% {}", step.percent, step.direction.noun()),
                        format!("Calculate value after {} change", step.direction.noun()),
                        format!("{:.2}", next_val),
                    )
                    .with_expected_value(next_val)
                    .with_alternates(vec![
                        format!("{:.0}", next_val),
                        format!("${:.2}", next_val),
                        format!("${:.0}", next_val),
                    ])
                    .with_hints(vec![
                        StepHint::principle("Successive changes multiply the current intermediate value, not the base."),
                        StepHint::operation(format!("Multiply current amount ({:.2}) by multiplier ({:.2}).", current_val, step.multiplier())),
                        StepHint::intermediate_relation(format!("{:.2} * (1 {} {:.2}) = {:.2}", current_val, step.direction.symbol(), step.percent / 100.0, next_val)),
                    ]);

                    if let Some(ref p_id) = prev_id {
                        node = node.with_dependencies(vec![p_id.clone()]);
                    }
                    if is_last {
                        node = node.as_final();
                    }

                    nodes.push(node);
                    prev_id = Some(step_id);
                    current_val = next_val;
                }

                let final_id = prev_id.unwrap_or_else(|| "step_1_transform".to_string());
                SolutionGraph::new(nodes, final_id)
            }
            PercentageVariant::NetEquivalentChange => {
                let step1 = StepNode::new(
                    "net_multiplier",
                    StepType::Transformation,
                    "Calculate net combined multiplier",
                    "Multiply individual growth factors together",
                    format!("{:.4}", problem.steps.iter().map(|s| s.multiplier()).product::<f64>()),
                )
                .with_hints(vec![
                    StepHint::principle("The net multiplier is the product of all individual multipliers."),
                    StepHint::operation("Multiply (1 ± r₁) * (1 ± r₂)."),
                    StepHint::intermediate_relation("Net Multiplier = m1 * m2"),
                ]);

                let step2 = StepNode::new(
                    "net_percent",
                    StepType::FinalAnswer,
                    "Convert to net percentage change",
                    "Subtract 1 and multiply by 100",
                    format!("{}", problem.canonical_answer_text),
                )
                .with_expected_value(problem.target_answer_value)
                .with_alternates(vec![
                    format!("{:+}%", problem.net_percentage_change),
                    format!("{}%", problem.canonical_answer_text),
                ])
                .with_dependencies(vec!["net_multiplier".to_string()])
                .as_final()
                .with_hints(vec![
                    StepHint::principle("Net percentage = (Net Multiplier - 1) * 100%."),
                    StepHint::operation("Subtract 1 from the combined multiplier and convert to percentage."),
                    StepHint::intermediate_relation(format!("({:.4} - 1) * 100 = {}%", problem.steps.iter().map(|s| s.multiplier()).product::<f64>(), problem.canonical_answer_text)),
                ]);

                SolutionGraph::new(vec![step1, step2], "net_percent")
            }
            PercentageVariant::ReverseInitial => {
                let net_mult = problem.steps.iter().map(|s| s.multiplier()).product::<f64>();

                let step1 = StepNode::new(
                    "combined_factor",
                    StepType::Transformation,
                    "Find combined multiplier",
                    "Compute total growth factor",
                    format!("{:.4}", net_mult),
                )
                .with_hints(vec![
                    StepHint::principle("Final Value = Initial Value * Combined Multiplier."),
                    StepHint::operation("Multiply the individual growth factors."),
                    StepHint::intermediate_relation(format!("Multiplier = {:.4}", net_mult)),
                ]);

                let step2 = StepNode::new(
                    "recover_initial",
                    StepType::FinalAnswer,
                    "Recover initial value",
                    "Divide final value by combined multiplier",
                    format!("{}", problem.canonical_answer_text),
                )
                .with_expected_value(problem.target_answer_value)
                .with_alternates(vec![
                    format!("${}", problem.canonical_answer_text),
                ])
                .with_dependencies(vec!["combined_factor".to_string()])
                .as_final()
                .with_hints(vec![
                    StepHint::principle("To find the initial amount before changes, divide the final amount by the combined multiplier."),
                    StepHint::operation(format!("Divide {} by {:.4}.", problem.final_value, net_mult)),
                    StepHint::intermediate_relation(format!("Initial = {} / {:.4} = {}", problem.final_value, net_mult, problem.canonical_answer_text)),
                ]);

                SolutionGraph::new(vec![step1, step2], "recover_initial")
            }
        }
    }

    fn choose_variant(rng: &mut StdRng, config: &PercentageSuccessiveConfig) -> PercentageVariant {
        if let Some(ref allowed) = config.allowed_variants {
            if !allowed.is_empty() {
                let idx = rng.random_range(0..allowed.len());
                return allowed[idx];
            }
        }

        let min_diff = config.min_difficulty.unwrap_or(1.0);
        let max_diff = config.max_difficulty.unwrap_or(5.0);

        if max_diff < 2.0 {
            PercentageVariant::ForwardTwoStep
        } else if max_diff >= 3.5 && min_diff >= 2.5 && rng.random_bool(0.25) {
            PercentageVariant::ForwardThreeStep
        } else {
            let roll = rng.random_range(0..10);
            if roll < 4 {
                PercentageVariant::ForwardTwoStep
            } else if roll < 7 {
                PercentageVariant::ReverseInitial
            } else {
                PercentageVariant::NetEquivalentChange
            }
        }
    }

    fn generate_forward_two_step(rng: &mut StdRng) -> GeneratedPercentageProblem {
        let (initial, step1, step2) = Self::pick_clean_two_steps(rng);
        let val1 = initial * step1.multiplier();
        let val2 = val1 * step2.multiplier();

        let net_multiplier = step1.multiplier() * step2.multiplier();
        let net_change = ((net_multiplier - 1.0) * 100.0 * 100.0).round() / 100.0;

        let prompt = format!(
            "An initial value of ${:.0} is first {} by {:.0}% and then {} by {:.0}%.\n\nWhat is the final value?",
            initial,
            step1.direction.verb(),
            step1.percent,
            step2.direction.verb(),
            step2.percent,
        );

        let ans_val = (val2 * 100.0).round() / 100.0;
        let canonical_ans = if ans_val.fract() == 0.0 {
            format!("{:.0}", ans_val)
        } else {
            format!("{:.2}", ans_val)
        };

        let worked_solution = format!(
            "**Step 1:** Calculate value after first {}:\n\
             ${:.0} × (1 {} {:.2}) = ${:.2}\n\n\
             **Step 2:** Calculate value after second {}:\n\
             ${:.2} × (1 {} {:.2}) = **${}**\n\n\
             *(Using combined formula: Final = {:.0} × {:.2} × {:.2} = ${})*",
            step1.direction.noun(),
            initial,
            step1.direction.symbol(),
            step1.percent / 100.0,
            val1,
            step2.direction.noun(),
            val1,
            step2.direction.symbol(),
            step2.percent / 100.0,
            canonical_ans,
            initial,
            step1.multiplier(),
            step2.multiplier(),
            canonical_ans
        );

        GeneratedPercentageProblem {
            variant: PercentageVariant::ForwardTwoStep,
            initial_value: initial,
            steps: vec![step1, step2],
            final_value: ans_val,
            net_percentage_change: net_change,
            target_answer_value: ans_val,
            target_unit: "$".to_string(),
            rendered_prompt: prompt,
            canonical_answer_text: canonical_ans,
            worked_solution,
            difficulty: 2.0,
            target_time_ms: 35_000,
        }
    }

    fn generate_reverse_initial(rng: &mut StdRng) -> GeneratedPercentageProblem {
        let (initial, step1, step2) = Self::pick_clean_two_steps(rng);
        let val1 = initial * step1.multiplier();
        let val2 = val1 * step2.multiplier();

        let final_val = (val2 * 100.0).round() / 100.0;
        let final_text = if final_val.fract() == 0.0 {
            format!("{:.0}", final_val)
        } else {
            format!("{:.2}", final_val)
        };

        let net_multiplier = step1.multiplier() * step2.multiplier();
        let net_change = ((net_multiplier - 1.0) * 100.0 * 100.0).round() / 100.0;

        let prompt = format!(
            "After a quantity is {} by {:.0}% and then {} by {:.0}%, the resulting final value is ${}.\n\nWhat was the original initial value?",
            step1.direction.verb(),
            step1.percent,
            step2.direction.verb(),
            step2.percent,
            final_text,
        );

        let ans_val = initial;
        let canonical_ans = format!("{:.0}", ans_val);

        let worked_solution = format!(
            "**Formula:** Final Value = Initial Value × (1 ± r₁) × (1 ± r₂)\n\n\
             **Step 1:** Compute combined multiplier:\n\
             Multiplier = (1 {} {:.2}) × (1 {} {:.2}) = {:.4}\n\n\
             **Step 2:** Solve for Initial Value:\n\
             Initial Value = ${} / {:.4} = **${}**",
            step1.direction.symbol(),
            step1.percent / 100.0,
            step2.direction.symbol(),
            step2.percent / 100.0,
            net_multiplier,
            final_text,
            net_multiplier,
            canonical_ans,
        );

        GeneratedPercentageProblem {
            variant: PercentageVariant::ReverseInitial,
            initial_value: initial,
            steps: vec![step1, step2],
            final_value: final_val,
            net_percentage_change: net_change,
            target_answer_value: ans_val,
            target_unit: "$".to_string(),
            rendered_prompt: prompt,
            canonical_answer_text: canonical_ans,
            worked_solution,
            difficulty: 3.0,
            target_time_ms: 50_000,
        }
    }

    fn generate_net_equivalent(rng: &mut StdRng) -> GeneratedPercentageProblem {
        let (_, step1, step2) = Self::pick_clean_two_steps(rng);
        let net_multiplier = step1.multiplier() * step2.multiplier();
        let net_change = ((net_multiplier - 1.0) * 100.0 * 100.0).round() / 100.0;

        let prompt = format!(
            "What single equivalent net percentage change corresponds to a successive {} of {:.0}% followed by a {} of {:.0}%?\n\n(State your answer as a percentage, e.g. +8% or -28%)",
            step1.direction.noun(),
            step1.percent,
            step2.direction.noun(),
            step2.percent,
        );

        let ans_val = net_change;
        let canonical_ans = if ans_val > 0.0 {
            format!("+{:.0}%", ans_val)
        } else if ans_val < 0.0 {
            format!("{:.0}%", ans_val)
        } else {
            "0%".to_string()
        };

        let worked_solution = format!(
            "**Formula:** Net Multiplier = (1 ± r₁) × (1 ± r₂)\n\n\
             **Step 1:** Express percentage changes as multipliers:\n\
             Step 1: (1 {} {:.2}) = {:.2}\n\
             Step 2: (1 {} {:.2}) = {:.2}\n\n\
             **Step 2:** Multiply factors:\n\
             Net Factor = {:.2} × {:.2} = {:.4}\n\n\
             **Step 3:** Convert to net percentage change:\n\
             Net Change = ({:.4} - 1) × 100% = **{}**",
            step1.direction.symbol(),
            step1.percent / 100.0,
            step1.multiplier(),
            step2.direction.symbol(),
            step2.percent / 100.0,
            step2.multiplier(),
            step1.multiplier(),
            step2.multiplier(),
            net_multiplier,
            net_multiplier,
            canonical_ans,
        );

        GeneratedPercentageProblem {
            variant: PercentageVariant::NetEquivalentChange,
            initial_value: 100.0,
            steps: vec![step1, step2],
            final_value: 100.0 * net_multiplier,
            net_percentage_change: net_change,
            target_answer_value: ans_val,
            target_unit: "%".to_string(),
            rendered_prompt: prompt,
            canonical_answer_text: canonical_ans,
            worked_solution,
            difficulty: 2.2,
            target_time_ms: 40_000,
        }
    }

    fn generate_forward_three_step(rng: &mut StdRng) -> GeneratedPercentageProblem {
        let initial_idx = rng.random_range(0..Self::CLEAN_INITIAL_VALUES.len());
        let initial = Self::CLEAN_INITIAL_VALUES[initial_idx];

        let rates = &[10.0, 20.0, 25.0, 50.0];
        let r1 = rates[rng.random_range(0..rates.len())];
        let r2 = rates[rng.random_range(0..rates.len())];
        let r3 = rates[rng.random_range(0..rates.len())];

        let d1 = if rng.random_bool(0.5) { ChangeDirection::Increase } else { ChangeDirection::Decrease };
        let d2 = if rng.random_bool(0.5) { ChangeDirection::Increase } else { ChangeDirection::Decrease };
        let d3 = if rng.random_bool(0.5) { ChangeDirection::Increase } else { ChangeDirection::Decrease };

        let step1 = PercentageStep { percent: r1, direction: d1 };
        let step2 = PercentageStep { percent: r2, direction: d2 };
        let step3 = PercentageStep { percent: r3, direction: d3 };

        let val1 = initial * step1.multiplier();
        let val2 = val1 * step2.multiplier();
        let val3 = val2 * step3.multiplier();

        let net_multiplier = step1.multiplier() * step2.multiplier() * step3.multiplier();
        let net_change = ((net_multiplier - 1.0) * 100.0 * 100.0).round() / 100.0;
        let ans_val = (val3 * 100.0).round() / 100.0;

        let canonical_ans = if ans_val.fract() == 0.0 {
            format!("{:.0}", ans_val)
        } else {
            format!("{:.2}", ans_val)
        };

        let prompt = format!(
            "An initial value of ${:.0} undergoes three successive changes: first {} by {:.0}%, then {} by {:.0}%, and finally {} by {:.0}%.\n\nWhat is the final value?",
            initial,
            step1.direction.verb(),
            step1.percent,
            step2.direction.verb(),
            step2.percent,
            step3.direction.verb(),
            step3.percent,
        );

        let worked_solution = format!(
            "**Step 1:** Value after first change: ${:.0} × {:.2} = ${:.2}\n\
             **Step 2:** Value after second change: ${:.2} × {:.2} = ${:.2}\n\
             **Step 3:** Value after third change: ${:.2} × {:.2} = **${}**\n\n\
             *(Using combined formula: Final = {:.0} × {:.2} × {:.2} × {:.2} = ${})*",
            initial,
            step1.multiplier(),
            val1,
            val1,
            step2.multiplier(),
            val2,
            val2,
            step3.multiplier(),
            canonical_ans,
            initial,
            step1.multiplier(),
            step2.multiplier(),
            step3.multiplier(),
            canonical_ans,
        );

        GeneratedPercentageProblem {
            variant: PercentageVariant::ForwardThreeStep,
            initial_value: initial,
            steps: vec![step1, step2, step3],
            final_value: ans_val,
            net_percentage_change: net_change,
            target_answer_value: ans_val,
            target_unit: "$".to_string(),
            rendered_prompt: prompt,
            canonical_answer_text: canonical_ans,
            worked_solution,
            difficulty: 4.0,
            target_time_ms: 65_000,
        }
    }

    fn pick_clean_two_steps(rng: &mut StdRng) -> (f64, PercentageStep, PercentageStep) {
        let dir_case = rng.random_range(0..4);
        let (d1, d2) = match dir_case {
            0 => (ChangeDirection::Increase, ChangeDirection::Increase),
            1 => (ChangeDirection::Increase, ChangeDirection::Decrease),
            2 => (ChangeDirection::Decrease, ChangeDirection::Increase),
            _ => (ChangeDirection::Decrease, ChangeDirection::Decrease),
        };

        let initial_idx = rng.random_range(0..Self::CLEAN_INITIAL_VALUES.len());
        let initial = Self::CLEAN_INITIAL_VALUES[initial_idx];

        let rate_idx1 = rng.random_range(0..Self::CLEAN_RATES_STANDARD.len());
        let r1 = Self::CLEAN_RATES_STANDARD[rate_idx1];

        let rate_idx2 = rng.random_range(0..Self::CLEAN_RATES_STANDARD.len());
        let r2 = Self::CLEAN_RATES_STANDARD[rate_idx2];

        let step1 = PercentageStep { percent: r1, direction: d1 };
        let step2 = PercentageStep { percent: r2, direction: d2 };

        (initial, step1, step2)
    }
}

impl ProblemGenerator for PercentageSuccessiveGenerator {
    fn family_id(&self) -> &str {
        "family.math.percentage.successive"
    }

    fn template_ref(&self) -> &str {
        "math.percentage.successive.v1"
    }

    fn supported_variants(&self) -> Vec<String> {
        vec![
            "forward_two_step".to_string(),
            "reverse_initial".to_string(),
            "net_equivalent_change".to_string(),
            "forward_three_step".to_string(),
        ]
    }

    fn target_latency_ms(&self, difficulty_level: u32) -> u64 {
        match difficulty_level {
            1 => 30_000,
            2 => 35_000,
            3 => 50_000,
            4 => 65_000,
            _ => 55_000,
        }
    }

    fn generate(
        &self,
        family_id: &ProblemFamilyId,
        seed: u64,
        difficulty_level: u32,
        variant: Option<&str>,
    ) -> Result<ProblemInstance> {
        let chosen_variant = if let Some(v_str) = variant {
            match v_str {
                "forward_two_step" => PercentageVariant::ForwardTwoStep,
                "reverse_initial" => PercentageVariant::ReverseInitial,
                "net_equivalent_change" => PercentageVariant::NetEquivalentChange,
                "forward_three_step" => PercentageVariant::ForwardThreeStep,
                _ => PercentageVariant::ForwardTwoStep,
            }
        } else {
            match difficulty_level {
                1 => PercentageVariant::ForwardTwoStep,
                2 => PercentageVariant::ForwardTwoStep,
                3 => PercentageVariant::ReverseInitial,
                4 => PercentageVariant::ForwardThreeStep,
                _ => PercentageVariant::NetEquivalentChange,
            }
        };

        Ok(Self::generate_instance_for_variant(family_id, seed, chosen_variant))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_generation_reproducibility() {
        let config = PercentageSuccessiveConfig::default();
        let seed = 424242;

        let p1 = PercentageSuccessiveGenerator::generate(seed, &config);
        let p2 = PercentageSuccessiveGenerator::generate(seed, &config);

        assert_eq!(p1, p2);
        assert_eq!(p1.rendered_prompt, p2.rendered_prompt);
        assert_eq!(p1.canonical_answer_text, p2.canonical_answer_text);
        assert_eq!(p1.target_answer_value, p2.target_answer_value);
    }

    #[test]
    fn test_valid_parameter_ranges_across_many_seeds() {
        let config = PercentageSuccessiveConfig::default();

        for seed in 1..=500 {
            let p = PercentageSuccessiveGenerator::generate(seed, &config);

            assert!(p.initial_value > 0.0, "initial value must be positive");
            assert!(p.final_value > 0.0, "final value must be positive");
            assert!(!p.rendered_prompt.is_empty(), "prompt must not be empty");
            assert!(!p.canonical_answer_text.is_empty(), "canonical answer text must not be empty");
            assert!(!p.worked_solution.is_empty(), "solution must not be empty");

            for step in &p.steps {
                assert!(step.percent > 0.0 && step.percent <= 100.0, "rate must be in (0, 100]");
            }
        }
    }

    #[test]
    fn test_all_variants_produce_valid_solutions() {
        let variants = vec![
            PercentageVariant::ForwardTwoStep,
            PercentageVariant::ReverseInitial,
            PercentageVariant::NetEquivalentChange,
            PercentageVariant::ForwardThreeStep,
        ];

        for variant in variants {
            let config = PercentageSuccessiveConfig {
                allowed_variants: Some(vec![variant]),
                min_difficulty: Some(1.0),
                max_difficulty: Some(5.0),
            };

            for seed in 100..120 {
                let p = PercentageSuccessiveGenerator::generate(seed, &config);
                assert_eq!(p.variant, variant);
                assert!(p.target_answer_value.is_finite());
            }
        }
    }

    #[test]
    fn test_problem_generator_trait_implementation() {
        let gen = PercentageSuccessiveGenerator::default();
        assert_eq!(gen.family_id(), "family.math.percentage.successive");
        assert_eq!(gen.template_ref(), "math.percentage.successive.v1");

        let fam_id = ProblemFamilyId::new("family.math.percentage.successive");
        let inst_l1 = gen.generate(&fam_id, 1234, 1, None).unwrap();
        assert!(inst_l1.rendered_prompt.contains("What is the final value?"));

        let inst_l3 = gen.generate(&fam_id, 1234, 3, None).unwrap();
        assert!(inst_l3.rendered_prompt.contains("What was the original initial value?"));
    }
}
