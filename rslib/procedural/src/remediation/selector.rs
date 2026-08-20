// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use crate::core::{Domain, ProceduralError, Result};
use crate::problems::registry::ProblemRegistry;
use crate::remediation::actions::{RemediationAction, RemediationActionKind};
use crate::remediation::objects::{
    CircuitBreakerObject, ConceptCheckObject, ConceptCheckOption, DeclarativeRecallBridge, PrerequisiteReviewObject,
    RemediationIntervention, RepresentationDrillObject, RepresentationOption, StrategyDrillObject,
    StrategyOption, WorkedExampleObject,
};
use crate::storage::ProceduralStore;

/// Transforms a typed RemediationAction into a concrete, executable RemediationIntervention.
pub struct RemediationSelector;

impl RemediationSelector {
    /// Select or construct the concrete learning intervention for a given action.
    pub fn select_intervention(
        action: &RemediationAction,
        store: &ProceduralStore,
        registry: &ProblemRegistry,
        seed: u64,
    ) -> Result<RemediationIntervention> {
        match action.kind {
            RemediationActionKind::CircuitBreaker => {
                Ok(RemediationIntervention::CircuitBreaker(
                    CircuitBreakerObject::new(
                        format!("cb-{}-{}", action.schema_id, seed),
                        &action.skill_id,
                        &action.schema_id,
                        action.domain.clone(),
                        action.recurrence_count,
                        "Repeated isomorphic failure limit reached. Take a brief break or switch topics to reset working memory.",
                        "Switch to a related foundational topic or return after spacing interval.",
                    ),
                ))
            }
            RemediationActionKind::ConceptCheck => {
                Ok(RemediationIntervention::ConceptCheck(
                    Self::build_concept_check(action, seed),
                ))
            }
            RemediationActionKind::StrategyDrill => {
                Ok(RemediationIntervention::StrategyDrill(
                    Self::build_strategy_drill(action, seed),
                ))
            }
            RemediationActionKind::RepresentationDrill => {
                Ok(RemediationIntervention::RepresentationDrill(
                    Self::build_representation_drill(action, seed),
                ))
            }
            RemediationActionKind::WorkedExample => {
                Ok(RemediationIntervention::WorkedExample(
                    Self::build_worked_example(action, seed),
                ))
            }
            RemediationActionKind::DeclarativeRecall => {
                Ok(RemediationIntervention::DeclarativeRecall(
                    Self::build_declarative_recall(action),
                ))
            }
            RemediationActionKind::PrerequisiteReview => {
                Ok(RemediationIntervention::PrerequisiteReview(
                    Self::build_executable_prerequisite_review(action, store, Some(registry), seed)?,
                ))
            }
            RemediationActionKind::ProceduralVariant | RemediationActionKind::TransferRetry => {
                // Generate a procedural problem instance
                let schema = store.get_schema(&action.schema_id)?
                    .ok_or_else(|| ProceduralError::NotFound(format!("Schema not found: {}", action.schema_id)))?;
                
                let family = store.get_problem_family(&schema.problem_family_id)?
                    .ok_or_else(|| ProceduralError::NotFound(format!("Problem family not found: {}", schema.problem_family_id)))?;

                let instance = registry.generate(
                    &schema.problem_family_id,
                    &family.template_ref,
                    seed,
                    action.preferred_difficulty,
                    action.preferred_variant.as_deref(),
                )?;

                if action.kind == RemediationActionKind::TransferRetry {
                    Ok(RemediationIntervention::TransferRetry(instance))
                } else {
                    Ok(RemediationIntervention::ProceduralProblem(instance))
                }
            }
        }
    }

    // =========================================================================
    // DOMAIN CONTENT BUILDERS (Deterministic & Evidence-Supported)
    // =========================================================================

    /// Build a domain-appropriate ConceptCheck object.
    pub fn build_concept_check(action: &RemediationAction, seed: u64) -> ConceptCheckObject {
        let sid = action.schema_id.as_str();

        match action.domain {
            Domain::Mathematics => {
                if sid.contains("percentage") {
                    ConceptCheckObject::new(
                        format!("cc-math-pct-{}", seed),
                        &action.skill_id,
                        &action.schema_id,
                        Domain::Mathematics,
                        "A store increases an item's price by 20%, and later decreases the new price by 20%.\n\nHow does the final price compare to the original?",
                        vec![
                            ConceptCheckOption::new(
                                "opt_dec",
                                "The final price is 4% less than the original price.",
                                true,
                                "compounding_multiplier",
                                "Correct: Multipliers compound multiplicatively (1.20 × 0.80 = 0.96, which is 4% less).",
                            ),
                            ConceptCheckOption::new(
                                "opt_same",
                                "The final price is exactly the same as the original price.",
                                false,
                                "additive_fallacy",
                                "Misconception: Successive percentages operate on different base amounts; they do not cancel out.",
                            ),
                            ConceptCheckOption::new(
                                "opt_inc",
                                "The final price is 4% higher than the original price.",
                                false,
                                "inverted_compound",
                                "Incorrect: 1.20 × 0.80 equals 0.96 (a decrease, not an increase).",
                            ),
                        ],
                        "opt_dec",
                        "Successive percentages compound multiplicatively. Always multiply the intermediate decimal multipliers (1 ± r₁) × (1 ± r₂).",
                    )
                } else if sid.contains("ratio") {
                    ConceptCheckObject::new(
                        format!("cc-math-ratio-{}", seed),
                        &action.skill_id,
                        &action.schema_id,
                        Domain::Mathematics,
                        "If the ratio of boys to girls in a class is 3:5 and there are 24 boys, what represents the value of one ratio unit (part)?",
                        vec![
                            ConceptCheckOption::new(
                                "opt_unit",
                                "24 / 3 = 8 students per ratio unit",
                                true,
                                "unitary_ratio_method",
                                "Correct: The given count (24) corresponds to the boys' ratio term (3 parts).",
                            ),
                            ConceptCheckOption::new(
                                "opt_total",
                                "24 / 8 = 3 students per ratio unit",
                                false,
                                "ratio_total_confusion",
                                "Incorrect: 24 is the number of boys alone, not the total number of students in the class.",
                            ),
                        ],
                        "opt_unit",
                        "Always equate the given concrete quantity to its corresponding ratio part to determine the unitary multiplier.",
                    )
                } else if sid.contains("linear_equations") {
                    ConceptCheckObject::new(
                        format!("cc-math-linear-{}", seed),
                        &action.skill_id,
                        &action.schema_id,
                        Domain::Mathematics,
                        "In solving the equation 3x - 7 = 14, which operation preserves equality and isolates the variable term?",
                        vec![
                            ConceptCheckOption::new(
                                "opt_add",
                                "Add 7 to both sides of the equation",
                                true,
                                "additive_inverse_transposition",
                                "Correct: Adding 7 cancels -7 on the LHS and isolates the 3x term.",
                            ),
                            ConceptCheckOption::new(
                                "opt_sub",
                                "Subtract 7 from both sides",
                                false,
                                "sign_reversal_slip",
                                "Incorrect: Subtracting 7 gives 3x - 14, failing to isolate 3x.",
                            ),
                        ],
                        "opt_add",
                        "To eliminate a subtracted constant term, apply its additive inverse (+7) to both sides.",
                    )
                } else {
                    // Generic math concept check
                    ConceptCheckObject::new(
                        format!("cc-math-gen-{}", seed),
                        &action.skill_id,
                        &action.schema_id,
                        Domain::Mathematics,
                        "When combining individual work rates or speeds, what principle applies?",
                        vec![
                            ConceptCheckOption::new(
                                "opt_rate",
                                "Work rates per unit time add directly: Rate_total = Rate_1 + Rate_2",
                                true,
                                "rate_superposition",
                                "Correct: Rates (work/time) add linearly, while total times do not.",
                            ),
                            ConceptCheckOption::new(
                                "opt_time",
                                "Total time taken is the sum of individual times: Time_total = Time_1 + Time_2",
                                false,
                                "time_additive_fallacy",
                                "Fallacy: People working together take less time, never more time.",
                            ),
                        ],
                        "opt_rate",
                        "Always convert to unitary work rates (1/T) before adding.",
                    )
                }
            }

            Domain::Physics => {
                ConceptCheckObject::new(
                    format!("cc-phys-{}", seed),
                    &action.skill_id,
                    &action.schema_id,
                    Domain::Physics,
                    "An object is dropped from rest under constant gravitational acceleration g. Which physical model governs its displacement over time?",
                    vec![
                        ConceptCheckOption::new(
                            "opt_uam",
                            "Uniformly Accelerated Motion: s = (1/2)gt²",
                            true,
                            "constant_acceleration_model",
                            "Correct: Gravitational acceleration is constant near Earth's surface.",
                        ),
                        ConceptCheckOption::new(
                            "opt_uvm",
                            "Uniform Velocity Motion: s = vt",
                            false,
                            "zero_acceleration_fallacy",
                            "Incorrect: Velocity increases continuously under gravity; acceleration is non-zero.",
                        ),
                    ],
                    "opt_uam",
                    "Identify the governing regime first: If acceleration is constant and non-zero, use kinematics equations with a = g.",
                )
            }

            Domain::Chemistry => {
                ConceptCheckObject::new(
                    format!("cc-chem-{}", seed),
                    &action.skill_id,
                    &action.schema_id,
                    Domain::Chemistry,
                    "In a chemical reaction where one reactant is completely consumed, which regime determines theoretical product yield?",
                    vec![
                        ConceptCheckOption::new(
                            "opt_limiting",
                            "Limiting reagent stoichiometric mole-ratio regime",
                            true,
                            "limiting_reagent_regime",
                            "Correct: The limiting reactant limits the maximum moles of product formed.",
                        ),
                        ConceptCheckOption::new(
                            "opt_excess",
                            "Excess reagent quantity",
                            false,
                            "excess_reagent_fallacy",
                            "Incorrect: Excess reagent remains unreacted and does not dictate maximum yield.",
                        ),
                    ],
                    "opt_limiting",
                    "Identify the limiting reactant by comparing (available moles / stoichiometric coefficient) for each reactant.",
                )
            }

            Domain::Reasoning => {
                ConceptCheckObject::new(
                    format!("cc-reason-{}", seed),
                    &action.skill_id,
                    &action.schema_id,
                    Domain::Reasoning,
                    "In a deductive puzzle with both fixed position clues and relative adjacency clues, what is the sound deduction order?",
                    vec![
                        ConceptCheckOption::new(
                            "opt_fixed",
                            "Anchor definite fixed positions first, then propagate relative constraints",
                            true,
                            "anchor_invariants_first",
                            "Correct: Fixed positions establish coordinate bounds and immediately eliminate branching.",
                        ),
                        ConceptCheckOption::new(
                            "opt_guess",
                            "Branch hypotheses for relative clues before placing fixed positions",
                            false,
                            "unconstrained_guessing",
                            "Sub-optimal: Guessing before placing invariant anchors increases tree search complexity.",
                        ),
                    ],
                    "opt_fixed",
                    "Always anchor invariant fixed positions first to drastically reduce problem search space.",
                )
            }

            Domain::Custom(_) => {
                ConceptCheckObject::new(
                    format!("cc-custom-{}", seed),
                    &action.skill_id,
                    &action.schema_id,
                    Domain::Mathematics,
                    "Which principle correctly applies to this problem context?",
                    vec![
                        ConceptCheckOption::new("opt_1", "Primary canonical principle", true, "canonical", "Correct."),
                        ConceptCheckOption::new("opt_2", "Alternative distractor", false, "distractor", "Incorrect."),
                    ],
                    "opt_1",
                    "Apply core domain definitions.",
                )
            }
        }
    }

    /// Build a domain-appropriate StrategyDrill object.
    pub fn build_strategy_drill(action: &RemediationAction, seed: u64) -> StrategyDrillObject {
        let sid = action.schema_id.as_str();

        match action.domain {
            Domain::Mathematics => {
                if sid.contains("percentage") {
                    StrategyDrillObject::new(
                        format!("sd-math-pct-{}", seed),
                        &action.skill_id,
                        &action.schema_id,
                        Domain::Mathematics,
                        "Which method is most direct and least error-prone for calculating the original price before two successive discounts of 10% and 20% resulting in $72?",
                        "Problem: Final Price = $72 after successive discounts of 10% and 20%. Find original price.",
                        vec![
                            StrategyOption::new(
                                "opt_mult",
                                "Compute combined multiplier (0.90 × 0.80 = 0.72), then divide: 72 / 0.72",
                                "decimal_multiplier_inversion",
                                true,
                                "Optimal: One simple division solves for the initial value cleanly ($100).",
                            ),
                            StrategyOption::new(
                                "opt_add",
                                "Add 10% + 20% = 30%, then calculate 72 × 1.30",
                                "additive_heuristic_error",
                                false,
                                "Invalid: Percentage discounts compound multiplicatively, not additively.",
                            ),
                        ],
                        "opt_mult",
                        "For reverse percentage problems, divide the final value by the product of individual multipliers.",
                    )
                } else if sid.contains("linear_equations") {
                    StrategyDrillObject::new(
                        format!("sd-math-lin-{}", seed),
                        &action.skill_id,
                        &action.schema_id,
                        Domain::Mathematics,
                        "To solve 2(x - 3) = 14, what is the cleanest first step?",
                        "Equation: 2(x - 3) = 14",
                        vec![
                            StrategyOption::new(
                                "opt_div",
                                "Divide both sides by 2 to get x - 3 = 7",
                                "divide_factor_first",
                                true,
                                "Optimal: Eliminates parentheses immediately without expanding terms.",
                            ),
                            StrategyOption::new(
                                "opt_expand",
                                "Expand the LHS to 2x - 6 = 14",
                                "distribute_multiplication",
                                true,
                                "Valid: Expansion works correctly, though dividing by 2 is often faster.",
                            ),
                        ],
                        "opt_div",
                        "Dividing by common factors first simplifies mental arithmetic.",
                    )
                } else {
                    StrategyDrillObject::new(
                        format!("sd-math-gen-{}", seed),
                        &action.skill_id,
                        &action.schema_id,
                        Domain::Mathematics,
                        "What is the most robust strategy to begin solving this multi-step problem?",
                        "Problem requires setting up relations between multiple given quantities.",
                        vec![
                            StrategyOption::new(
                                "opt_var",
                                "Define clear algebraic variables and translate word statements into equations",
                                "algebraic_modeling",
                                true,
                                "Optimal: Systematic algebraic modeling avoids ambiguous arithmetic jumps.",
                            ),
                            StrategyOption::new(
                                "opt_guess",
                                "Trial and error by plugging in answer choices",
                                "trial_and_error",
                                false,
                                "Sub-optimal: Guessing is slow and unreliable for complex structural variants.",
                            ),
                        ],
                        "opt_var",
                        "Formulate algebraic equations explicitly before calculating.",
                    )
                }
            }

            Domain::Physics => {
                StrategyDrillObject::new(
                    format!("sd-phys-{}", seed),
                    &action.skill_id,
                    &action.schema_id,
                    Domain::Physics,
                    "A projectile is launched with initial velocity v₀ at angle θ. To find the maximum height reached, which kinematic strategy is most direct?",
                    "Knowns: v₀, θ, g. Unknown: H_max.",
                    vec![
                        StrategyOption::new(
                            "opt_vy",
                            "Set vertical velocity component v_y = 0 at peak and use v_y² = (v₀ sin θ)² - 2gH",
                            "vertical_peak_zero_velocity",
                            true,
                            "Optimal: Direct algebraic relationship without needing total time of flight.",
                        ),
                        StrategyOption::new(
                            "opt_time",
                            "Find total flight time, halve it, then substitute into displacement equation",
                            "two_step_time_substitution",
                            true,
                            "Valid: Accurate but requires two separate calculation steps.",
                        ),
                    ],
                    "opt_vy",
                    "At the peak of trajectory, vertical velocity v_y is instantaneously zero.",
                )
            }

            Domain::Chemistry => {
                StrategyDrillObject::new(
                    format!("sd-chem-{}", seed),
                    &action.skill_id,
                    &action.schema_id,
                    Domain::Chemistry,
                    "Given mass of Reactant A in grams, what is the canonical 3-step strategy to find mass of Product B?",
                    "Reaction: aA -> bB. Given: mass_A. Find: mass_B.",
                    vec![
                        StrategyOption::new(
                            "opt_mole_bridge",
                            "Mass A -> Moles A (÷ Molar Mass A) -> Moles B (× b/a) -> Mass B (× Molar Mass B)",
                            "stoichiometric_mole_bridge",
                            true,
                            "Optimal: The mole bridge is the only chemically sound path between different substances.",
                        ),
                        StrategyOption::new(
                            "opt_mass_ratio",
                            "Multiply mass of A directly by the stoichiometric coefficient ratio (b/a)",
                            "mass_ratio_fallacy",
                            false,
                            "Fallacy: Stoichiometric coefficients represent mole ratios, NOT mass ratios.",
                        ),
                    ],
                    "opt_mole_bridge",
                    "Never convert directly from mass of A to mass of B without going through moles.",
                )
            }

            Domain::Reasoning => {
                StrategyDrillObject::new(
                    format!("sd-reason-{}", seed),
                    &action.skill_id,
                    &action.schema_id,
                    Domain::Reasoning,
                    "In a circular seating arrangement where people face both inward and outward, which element should you place first?",
                    "Clues contain 1 definite orientation and several relative left/right clues.",
                    vec![
                        StrategyOption::new(
                            "opt_anchor",
                            "The person whose exact orientation (facing inside/outside) and position is explicitly known",
                            "anchor_known_orientation",
                            true,
                            "Optimal: Direction of left/right depends entirely on orientation.",
                        ),
                        StrategyOption::new(
                            "opt_relative",
                            "Any relative pair regardless of unknown facing direction",
                            "unoriented_branching",
                            false,
                            "Sub-optimal: Unoriented pairs generate 2x branching states unnecessarily.",
                        ),
                    ],
                    "opt_anchor",
                    "Anchor people with known facing directions first to fix left/right frame of reference.",
                )
            }

            Domain::Custom(_) => {
                StrategyDrillObject::new(
                    format!("sd-custom-{}", seed),
                    &action.skill_id,
                    &action.schema_id,
                    Domain::Mathematics,
                    "Which initial strategy should be selected?",
                    "Context",
                    vec![
                        StrategyOption::new("opt_1", "Optimal direct strategy", "optimal", true, "Correct."),
                        StrategyOption::new("opt_2", "Sub-optimal branch", "suboptimal", false, "Incorrect."),
                    ],
                    "opt_1",
                    "Select optimal method.",
                )
            }
        }
    }

    /// Build a RepresentationDrill object.
    pub fn build_representation_drill(action: &RemediationAction, seed: u64) -> RepresentationDrillObject {
        RepresentationDrillObject::new(
            format!("rd-{}", seed),
            &action.skill_id,
            &action.schema_id,
            action.domain.clone(),
            "Which diagrammatic / symbolic representation correctly aligns with the given physical or structural constraints?",
            vec![
                RepresentationOption::new(
                    "opt_rep_correct",
                    "Standard Cartesian frame with upward/forward as positive (+y, +x)",
                    true,
                    "standard_cartesian",
                    "Correct: Standard coordinate frames maintain consistent sign conventions.",
                ),
                RepresentationOption::new(
                    "opt_rep_wrong",
                    "Inverted sign convention without explicit origin declaration",
                    false,
                    "inconsistent_frame",
                    "Incorrect: Inconsistent coordinate frames lead to sign reversal errors.",
                ),
            ],
            "opt_rep_correct",
            "Always fix and maintain a clear coordinate and directional convention before solving.",
        )
    }

    /// Build a canonical WorkedExample object.
    pub fn build_worked_example(action: &RemediationAction, seed: u64) -> WorkedExampleObject {
        let sid = action.schema_id.as_str();

        if sid.contains("percentage") {
            WorkedExampleObject::new(
                format!("we-math-pct-{}", seed),
                &action.skill_id,
                &action.schema_id,
                Domain::Mathematics,
                "Worked Example: Successive Percentage Compounding",
                "A product costing $500 is marked up by 20% and later discounted by 10%. Find the final price.",
                vec![
                    "1. Express percentage changes as decimal multipliers: +20% -> 1.20; -10% -> 0.90.".to_string(),
                    "2. Compute net combined multiplier: Multiplier = 1.20 × 0.90 = 1.08.".to_string(),
                    "3. Apply to base price: Final Price = $500 × 1.08 = $540.00.".to_string(),
                ],
                "Decision Point: Use multiplicative factors (1.20 × 0.90) rather than adding (+20% - 10% = +10%).",
                "Successive percentage changes compound on intermediate values. Multiplicative chaining is mathematically exact.",
                vec![
                    "Common Mistake: Adding percentages directly (e.g. 500 × 1.10 = 550) - WRONG!".to_string(),
                    "Common Mistake: Applying second percentage to original base rather than intermediate value.".to_string(),
                ],
            )
        } else if action.domain == Domain::Physics {
            WorkedExampleObject::new(
                format!("we-phys-{}", seed),
                &action.skill_id,
                &action.schema_id,
                Domain::Physics,
                "Worked Example: 1D Kinematics with Uniform Acceleration",
                "A train starting from rest accelerates at 1.5 m/s² over a distance of 300 m. Find its final velocity.",
                vec![
                    "1. Identify knowns & unknown: u = 0 m/s, a = 1.5 m/s², s = 300 m, find v.".to_string(),
                    "2. Select equation without time parameter t: v² = u² + 2as.".to_string(),
                    "3. Substitute values: v² = 0 + 2(1.5)(300) = 900.".to_string(),
                    "4. Solve for v: v = √900 = 30 m/s.".to_string(),
                ],
                "Decision Point: Choose v² = u² + 2as to eliminate unnecessary time calculation.",
                "Matching known and unknown variables to kinematic relations minimizes algebra steps.",
                vec![
                    "Common Mistake: Using s = vt (assumes zero acceleration) - WRONG!".to_string(),
                    "Common Mistake: Forgetting to take square root of v² at final step.".to_string(),
                ],
            )
        } else if action.domain == Domain::Chemistry {
            WorkedExampleObject::new(
                format!("we-chem-{}", seed),
                &action.skill_id,
                &action.schema_id,
                Domain::Chemistry,
                "Worked Example: Stoichiometric Mole Bridge",
                "How many grams of CO₂ are produced from the complete combustion of 32 g of CH₄ (Molar Mass = 16 g/mol)?",
                vec![
                    "1. Balance chemical reaction: CH₄ + 2O₂ -> CO₂ + 2H₂O (1 mole CH₄ : 1 mole CO₂).".to_string(),
                    "2. Convert Mass CH₄ to Moles: Moles CH₄ = 32 g / (16 g/mol) = 2.0 moles.".to_string(),
                    "3. Use mole ratio: Moles CO₂ = 2.0 × (1/1) = 2.0 moles CO₂.".to_string(),
                    "4. Convert Moles CO₂ to Mass: Mass CO₂ = 2.0 moles × (44 g/mol) = 88 g.".to_string(),
                ],
                "Decision Point: Always bridge through moles; never multiply grams directly by coefficients.",
                "Chemical coefficients establish molar equivalence, which is converted to mass via substance molar mass.",
                vec![
                    "Common Mistake: Assuming grams ratio equals coefficient ratio - WRONG!".to_string(),
                ],
            )
        } else {
            WorkedExampleObject::new(
                format!("we-gen-{}", seed),
                &action.skill_id,
                &action.schema_id,
                action.domain.clone(),
                "Worked Example: Step-by-Step Solution",
                "Canonical demonstration problem.",
                vec![
                    "1. Identify structure and given constraints.".to_string(),
                    "2. Formulate governing equations / relations.".to_string(),
                    "3. Compute final answer with appropriate units.".to_string(),
                ],
                "Decision Point: Establish structural model before calculation.",
                "Systematic problem solving reduces cognitive slip errors.",
                vec!["Mistake: Skipping initial constraint verification.".to_string()],
            )
        }
    }

    /// Build a DeclarativeRecallBridge object.
    pub fn build_declarative_recall(action: &RemediationAction) -> DeclarativeRecallBridge {
        DeclarativeRecallBridge::new(
            format!("dec-{}", action.skill_id),
            &action.skill_id,
            action.domain.clone(),
            format!("Declarative Recall: {}", action.skill_id),
            "Recall the core definition, formula, or unit conversion factor for this concept.",
            "Review associated flashcard and test active recall of definitions.",
        )
        .with_tag(format!("procedural::{}", action.skill_id))
    }

    /// Build a backward-compatible advisory PrerequisiteReviewObject.
    pub fn build_prerequisite_review(
        action: &RemediationAction,
        store: &ProceduralStore,
    ) -> Result<PrerequisiteReviewObject> {
        Self::build_executable_prerequisite_review(action, store, None, 0)
    }

    /// Build an executable PrerequisiteReviewObject that resolves the primary missing prerequisite and generates foundational practice.
    pub fn build_executable_prerequisite_review(
        action: &RemediationAction,
        store: &ProceduralStore,
        registry: Option<&ProblemRegistry>,
        seed: u64,
    ) -> Result<PrerequisiteReviewObject> {
        let prereqs = store.get_skill(&action.skill_id)?
            .map(|s| s.prerequisites)
            .unwrap_or_default();

        let mut review_obj = PrerequisiteReviewObject::new(
            format!("prereq-{}", action.skill_id),
            &action.skill_id,
            prereqs.clone(),
            action.domain.clone(),
            format!("Foundational prerequisite review recommended for skill '{}'", action.skill_id),
            "Persistent conceptual breakdowns suggest gaps in foundational prerequisites. Review earlier skills before re-attempting.",
        );

        if let Some(primary_prereq) = prereqs.first() {
            let all_schemas = store.list_all_schemas().unwrap_or_default();
            let matching_schema = all_schemas.into_iter().find(|s| &s.skill_id == primary_prereq);

            let mut executable_problem = None;
            let schema_id_opt = matching_schema.as_ref().map(|s| s.id.clone());

            if let (Some(schema), Some(reg)) = (matching_schema.as_ref(), registry) {
                if let Ok(Some(family)) = store.get_problem_family(&schema.problem_family_id) {
                    if let Ok(instance) = reg.generate(
                        &schema.problem_family_id,
                        &family.template_ref,
                        seed,
                        1, // Foundational L1 difficulty
                        None,
                    ) {
                        executable_problem = Some(instance);
                    }
                }
            }

            review_obj = review_obj.with_executable_prerequisite(
                primary_prereq.clone(),
                schema_id_opt,
                executable_problem,
            );
        }

        Ok(review_obj)
    }
}
