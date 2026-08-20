// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::collections::HashSet;
use procedural::core::ProblemFamilyId;
use procedural::problems::catalog::*;
use procedural::problems::generators::*;
use procedural::problems::registry::ProblemRegistry;

#[derive(Debug, Clone, Default)]
pub struct FamilyBaselineMetrics {
    pub family_id: String,
    pub total_generated: usize,
    pub valid_count: usize,
    pub rejected_count: usize,
    pub unique_prompts: usize,
    pub exact_dup_rate: f64,
    pub unique_params: usize,
    pub param_dup_rate: f64,
    pub unique_solution_graphs: usize,
    pub unique_decision_paths: usize,
    pub difficulty_distribution: [usize; 5],
    pub first_repeat_idx: Option<usize>,
    pub novelty_half_life: usize,
}

#[test]
fn test_f1_measure_all_22_families_baseline() {
    let registry = ProblemRegistry::default_registry();

    let families = vec![
        // Mathematics (14)
        (FAMILY_PERCENTAGE_SUCCESSIVE, TEMPLATE_PERCENTAGE_SUCCESSIVE_V1, "math.percentage.successive"),
        (FAMILY_LINEAR_EQUATIONS, TEMPLATE_LINEAR_EQUATIONS_V1, "algebra.linear_equations"),
        (FAMILY_PROFIT_LOSS, TEMPLATE_PROFIT_LOSS_V1, "arithmetic.profit_loss"),
        (FAMILY_RATIO, TEMPLATE_RATIO_V1, "arithmetic.ratio"),
        (FAMILY_AVERAGE, TEMPLATE_AVERAGE_V1, "arithmetic.average"),
        (FAMILY_DIVISIBILITY, TEMPLATE_DIVISIBILITY_V1, "number_system.divisibility"),
        (FAMILY_TIME_WORK, TEMPLATE_TIME_WORK_V1, "time_work.basic"),
        (FAMILY_TIME_SPEED_DISTANCE, TEMPLATE_TIME_SPEED_DISTANCE_V1, "arithmetic.time_speed_distance"),
        (FAMILY_MIXTURES_ALLIGATION, TEMPLATE_MIXTURES_ALLIGATION_V1, "arithmetic.mixtures_alligation"),
        (FAMILY_REMAINDERS_MODULAR, TEMPLATE_REMAINDERS_MODULAR_V1, "number_system.remainders_modular"),
        (FAMILY_LINEAR_INEQUALITIES, TEMPLATE_LINEAR_INEQUALITIES_V1, "algebra.linear_inequalities"),
        (FAMILY_ALGEBRAIC_IDENTITIES, TEMPLATE_ALGEBRAIC_IDENTITIES_V1, "algebra.algebraic_identities"),
        (FAMILY_GEOMETRY_TRIANGLES, TEMPLATE_GEOMETRY_TRIANGLES_V1, "geometry.triangles"),
        (FAMILY_COMBINED_MULTI_CONCEPT, TEMPLATE_COMBINED_MULTI_CONCEPT_V1, "combined.multi_concept"),
        // Physics (2)
        ("family.physics.kinematics.1d", "physics.kinematics.1d.v1", "physics.kinematics.1d"),
        ("family.physics.work_energy.mechanics", "physics.work_energy.mechanics.v1", "physics.work_energy.mechanics"),
        // Chemistry (2)
        (FAMILY_CHEMISTRY_STOICHIOMETRY, TEMPLATE_CHEMISTRY_STOICHIOMETRY_V1, "chemistry.stoichiometry.moles"),
        (FAMILY_CHEMISTRY_EQUILIBRIUM, TEMPLATE_CHEMISTRY_EQUILIBRIUM_V1, "chemistry.equilibrium.concentration"),
        // Reasoning (4)
        (FAMILY_REASONING_SERIES, TEMPLATE_REASONING_SERIES_V1, "reasoning.series.pattern_recognition"),
        (FAMILY_REASONING_SYLLOGISM, TEMPLATE_REASONING_SYLLOGISM_V1, "reasoning.syllogism.formal_inference"),
        (FAMILY_REASONING_SEATING, TEMPLATE_REASONING_SEATING_V1, "reasoning.seating.constraint_satisfaction"),
        (FAMILY_REASONING_RELATIONS, TEMPLATE_REASONING_RELATIONS_V1, "reasoning.relations.graph_inference"),
    ];

    let n_samples = 500;
    println!("\n========================================================================================================================");
    println!("F1 FULL GENERATOR CATALOG BASELINE AUDIT (N = {} per family across L1..L5)", n_samples);
    println!("========================================================================================================================");
    println!("{:<35} | {:<5} | {:<5} | {:<7} | {:<7} | {:<7} | {:<6} | {:<6} | {:<8}", 
             "Family", "Valid", "Rej", "ExDup%", "ParDup%", "UniqSG", "UniqDP", "1stRep", "NovHalfL");
    println!("------------------------------------------------------------------------------------------------------------------------");

    let mut results = Vec::new();

    for (fam_id_str, template_ref, _skill_id) in &families {
        let fam_id = ProblemFamilyId::new(*fam_id_str);
        let validator = registry.get_validator(fam_id_str);

        let mut seen_prompts: HashSet<String> = HashSet::new();
        let mut seen_params: HashSet<String> = HashSet::new();
        let mut seen_graphs: HashSet<String> = HashSet::new();
        let mut seen_decision_paths: HashSet<String> = HashSet::new();

        let mut valid_count = 0;
        let mut rej_count = 0;
        let mut first_repeat_idx = None;
        let mut diff_dist = [0usize; 5];

        let mut duplicate_count = 0;
        let mut param_dup_count = 0;
        let mut half_life = n_samples;

        for seed in 1..=n_samples {
            let difficulty = ((seed % 5) + 1) as u32;
            diff_dist[(difficulty - 1) as usize] += 1;

            match registry.generate(&fam_id, template_ref, seed as u64, difficulty, None) {
                Ok(instance) => {
                    // Check validator with generator's own correct answer
                    let is_valid = if let Some(ref val) = validator {
                        let eval_obj = val.evaluate(&instance, &instance.correct_answer, 5000, 30000);
                        let eval_fmt = if let Some(fmt) = instance.correct_answer.get("formatted") {
                            val.evaluate(&instance, fmt, 5000, 30000)
                        } else {
                            eval_obj.clone()
                        };
                        let eval_val = if let Some(v) = instance.correct_answer.get("value") {
                            val.evaluate(&instance, v, 5000, 30000)
                        } else {
                            eval_obj.clone()
                        };
                        eval_obj.is_correct || eval_fmt.is_correct || eval_val.is_correct
                    } else {
                        true
                    };

                    if is_valid {
                        valid_count += 1;
                    } else {
                        rej_count += 1;
                    }

                    // Prompt duplicate
                    let prompt_clean = instance.rendered_prompt.trim().to_string();
                    if !seen_prompts.insert(prompt_clean) {
                        duplicate_count += 1;
                        if first_repeat_idx.is_none() {
                            first_repeat_idx = Some(seed);
                        }
                    }

                    // Param duplicate
                    let param_str = instance.parameters.to_string();
                    if !seen_params.insert(param_str) {
                        param_dup_count += 1;
                    }

                    // Solution graph fingerprint
                    if let Some(sg) = instance.solution_graph() {
                        let sg_fp = format!("steps:{}-{}", sg.steps.len(), sg.steps.iter().map(|n| format!("{:?}:{}", n.step_type, n.dependencies.len())).collect::<Vec<_>>().join("|"));
                        seen_graphs.insert(sg_fp);
                    }

                    // Decision path / metadata fingerprint
                    let meta_fp = instance.metadata.to_string();
                    seen_decision_paths.insert(meta_fp);

                    // Track rolling duplicate rate for half-life
                    if seed > 20 && half_life == n_samples {
                        if duplicate_count * 2 >= seed {
                            half_life = seed;
                        }
                    }
                }
                Err(_) => {
                    rej_count += 1;
                }
            }
        }

        let exact_dup_pct = (duplicate_count as f64 / n_samples as f64) * 100.0;
        let param_dup_pct = (param_dup_count as f64 / n_samples as f64) * 100.0;

        let metrics = FamilyBaselineMetrics {
            family_id: fam_id_str.to_string(),
            total_generated: n_samples,
            valid_count,
            rejected_count: rej_count,
            unique_prompts: seen_prompts.len(),
            exact_dup_rate: exact_dup_pct,
            unique_params: seen_params.len(),
            param_dup_rate: param_dup_pct,
            unique_solution_graphs: seen_graphs.len(),
            unique_decision_paths: seen_decision_paths.len(),
            difficulty_distribution: diff_dist,
            first_repeat_idx,
            novelty_half_life: half_life,
        };

        println!("{:<35} | {:<5} | {:<5} | {:<6.1}% | {:<6.1}% | {:<7} | {:<6} | {:<6} | {:<8}", 
                 fam_id_str.replace("family.", "").replace("math.", "").replace("arithmetic.", "").replace("number_system.", "").replace("reasoning.", "").replace("physics.", "").replace("chemistry.", ""),
                 valid_count, rej_count, exact_dup_pct, param_dup_pct, seen_graphs.len(), seen_decision_paths.len(), 
                 first_repeat_idx.map(|i| format!("{}", i)).unwrap_or_else(|| "none".to_string()),
                 if half_life < n_samples { format!("{}", half_life) } else { ">500".to_string() });

        results.push(metrics);
    }
    println!("========================================================================================================================\n");
}
