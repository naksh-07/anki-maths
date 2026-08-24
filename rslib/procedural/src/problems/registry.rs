// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::collections::HashMap;
use std::sync::Arc;

use crate::chemistry::generators::{
    BuffersTitrationGenerator, BuffersTitrationValidator, ChemicalKineticsGenerator,
    ChemicalKineticsValidator, ElectrochemistryGenerator, ElectrochemistryValidator,
    EquilibriumGenerator, EquilibriumValidator, ReactionNetworksGenerator,
    ReactionNetworksValidator, StoichiometryGenerator, StoichiometryValidator,
};
use crate::core::{Domain, ProblemFamilyId, ProceduralError, Result};
use crate::physics::generators::{
    Kinematics1DGenerator, Kinematics1DValidator, WorkEnergyGenerator, WorkEnergyValidator,
};
use crate::problems::contract::{
    DeclarativeFamilyContract, ProblemFamilyCapability, ProblemFamilyContract,
};
use crate::problems::declarative::{
    linear_equations_declarative_contract, DeclarativeProblemGenerator,
};
use crate::problems::generator::ProblemGenerator;
use crate::problems::generators::{
    AlgebraicIdentitiesGenerator, AlgebraicIdentitiesValidator, AverageGenerator, AverageValidator,
    CombinedMultiConceptGenerator, CombinedMultiConceptValidator, DivisibilityGenerator,
    DivisibilityValidator, GeometryTrianglesGenerator, GeometryTrianglesValidator,
    LinearEquationsGenerator, LinearEquationsValidator, LinearInequalitiesGenerator,
    LinearInequalitiesValidator, MixturesAlligationGenerator, MixturesAlligationValidator,
    PercentageSuccessiveGenerator, ProfitLossGenerator, ProfitLossValidator, RatioGenerator,
    RatioValidator, RemaindersModularGenerator, RemaindersModularValidator,
    TimeSpeedDistanceGenerator, TimeSpeedDistanceValidator, TimeWorkGenerator, TimeWorkValidator,
};
use crate::reasoning::generators::{
    BloodRelationsGenerator, BloodRelationsValidator, CodedExpressionsGenerator,
    CodedExpressionsValidator, DataSufficiencyGenerator, DataSufficiencyValidator,
    DirectionSenseGenerator, DirectionSenseValidator, FloorGridGenerator, FloorGridValidator,
    LogicDagGenerator, LogicDagValidator, RelationsGenerator, RelationsValidator, SeatingGenerator,
    SeatingValidator, SeriesGenerator, SeriesValidator, SyllogismGenerator, SyllogismValidator,
};
use crate::problems::validator::{PercentageSuccessiveValidator, ProblemValidator};
use crate::problems::ProblemInstance;

/// Unified registry for dynamic, domain-agnostic dispatch of problem generators and validators.
#[derive(Clone)]
pub struct ProblemRegistry {
    generators_by_family: HashMap<String, Arc<dyn ProblemGenerator>>,
    generators_by_template: HashMap<String, Arc<dyn ProblemGenerator>>,
    validators_by_family: HashMap<String, Arc<dyn ProblemValidator>>,
    contracts_by_family: HashMap<String, Arc<ProblemFamilyContract>>,
    declarative_generators: HashMap<String, Arc<DeclarativeProblemGenerator>>,
}

impl Default for ProblemRegistry {
    fn default() -> Self {
        Self::default_maths_registry()
    }
}

impl ProblemRegistry {
    pub fn new() -> Self {
        Self {
            generators_by_family: HashMap::new(),
            generators_by_template: HashMap::new(),
            validators_by_family: HashMap::new(),
            contracts_by_family: HashMap::new(),
            declarative_generators: HashMap::new(),
        }
    }

    /// Register a problem generator for its family and template references.
    pub fn register_generator(&mut self, generator: Arc<dyn ProblemGenerator>) {
        self.generators_by_family
            .insert(generator.family_id().to_string(), generator.clone());
        self.generators_by_template
            .insert(generator.template_ref().to_string(), generator);
    }

    /// Register a problem validator for its family ID.
    pub fn register_validator(&mut self, validator: Arc<dyn ProblemValidator>) {
        self.validators_by_family
            .insert(validator.family_id().to_string(), validator);
    }

    /// Register a problem family contract.
    pub fn register_family_contract(&mut self, contract: Arc<ProblemFamilyContract>) {
        self.contracts_by_family
            .insert(contract.family_id.as_str().to_string(), contract);
    }

    /// Register a declarative problem family with its contract and generator.
    pub fn register_declarative_family(&mut self, contract: DeclarativeFamilyContract) {
        let contract_arc = Arc::new(contract.contract.clone());
        self.register_family_contract(contract_arc);

        let dec_gen = Arc::new(DeclarativeProblemGenerator::new(Arc::new(contract)));
        self.declarative_generators
            .insert(dec_gen.family_id().to_string(), dec_gen.clone());
        self.generators_by_template
            .insert(dec_gen.template_ref().to_string(), dec_gen);
    }

    /// Retrieve generator by family ID or template reference.
    pub fn get_generator(&self, family_or_template: &str) -> Option<Arc<dyn ProblemGenerator>> {
        self.generators_by_family
            .get(family_or_template)
            .or_else(|| self.generators_by_template.get(family_or_template))
            .cloned()
    }

    /// Retrieve validator by family ID or template reference.
    pub fn get_validator(&self, family_id: &str) -> Option<Arc<dyn ProblemValidator>> {
        self.validators_by_family.get(family_id).cloned()
    }

    /// Retrieve family contract by family ID.
    pub fn get_family_contract(&self, family_id: &str) -> Option<Arc<ProblemFamilyContract>> {
        self.contracts_by_family.get(family_id).cloned()
    }

    /// Retrieve generator capability for a given family ID.
    pub fn get_capability(&self, family_id: &str) -> Option<ProblemFamilyCapability> {
        self.contracts_by_family.get(family_id).map(|c| c.capability)
    }

    /// Retrieve declarative generator by family ID.
    pub fn get_declarative_generator(&self, family_id: &str) -> Option<Arc<DeclarativeProblemGenerator>> {
        self.declarative_generators.get(family_id).cloned()
    }

    /// Generate a problem instance deterministically through dynamic dispatch.
    /// Resilient strategy: attempts declarative generation with validation;
    /// if declarative generation is missing or validation fails, automatically
    /// falls back to the specialized generator.
    pub fn generate(
        &self,
        family_id: &ProblemFamilyId,
        template_ref: &str,
        seed: u64,
        difficulty_level: u32,
        variant: Option<&str>,
    ) -> Result<ProblemInstance> {
        // 1. Attempt declarative generation if registered
        if let Some(dec_gen) = self.get_declarative_generator(family_id.as_str()) {
            if let Ok(instance) = dec_gen.generate(family_id, seed, difficulty_level, variant) {
                // Verify generated instance against the registered validator
                if let Some(validator) = self.get_validator(family_id.as_str()) {
                    if let Some(ans_val) = instance.correct_answer.get("value") {
                        let eval = validator.evaluate(
                            &instance,
                            ans_val,
                            15_000,
                            dec_gen.target_latency_ms(difficulty_level),
                        );
                        if eval.is_correct {
                            return Ok(instance);
                        }
                    } else {
                        return Ok(instance);
                    }
                } else {
                    return Ok(instance);
                }
            }
            // If declarative generation errored or validation rejected, fall through to specialized fallback!
        }

        // 2. Dispatch to specialized generator (or fallback)
        if let Some(gen) = self
            .get_generator(family_id.as_str())
            .or_else(|| self.get_generator(template_ref))
        {
            gen.generate(family_id, seed, difficulty_level, variant)
        } else {
            Err(ProceduralError::NotFound(format!(
                "No registered problem generator for family {} / template {}",
                family_id, template_ref
            )))
        }
    }

    /// Convenience helper: generate instance with difficulty level using family_id.
    pub fn generate_with_difficulty(
        &self,
        family_id: &ProblemFamilyId,
        seed: u64,
        difficulty_level: u32,
    ) -> Result<ProblemInstance> {
        self.generate(family_id, family_id.as_str(), seed, difficulty_level, None)
    }

    /// Build canonical Mathematics registry containing all 14 topic generators and validators.
    pub fn default_maths_registry() -> Self {
        let mut registry = Self::new();

        // 1. Successive Percentage
        registry.register_generator(Arc::new(PercentageSuccessiveGenerator));
        registry.register_validator(Arc::new(PercentageSuccessiveValidator));

        // 2. Linear Equations
        registry.register_generator(Arc::new(LinearEquationsGenerator));
        registry.register_validator(Arc::new(LinearEquationsValidator));

        // 3. Profit & Loss
        registry.register_generator(Arc::new(ProfitLossGenerator));
        registry.register_validator(Arc::new(ProfitLossValidator));

        // 4. Ratio
        registry.register_generator(Arc::new(RatioGenerator));
        registry.register_validator(Arc::new(RatioValidator));

        // 5. Average
        registry.register_generator(Arc::new(AverageGenerator));
        registry.register_validator(Arc::new(AverageValidator));

        // 6. Divisibility
        registry.register_generator(Arc::new(DivisibilityGenerator));
        registry.register_validator(Arc::new(DivisibilityValidator));

        // 7. Time & Work
        registry.register_generator(Arc::new(TimeWorkGenerator));
        registry.register_validator(Arc::new(TimeWorkValidator));

        // 8. Time, Speed & Distance
        registry.register_generator(Arc::new(TimeSpeedDistanceGenerator));
        registry.register_validator(Arc::new(TimeSpeedDistanceValidator));

        // 9. Mixtures & Alligation
        registry.register_generator(Arc::new(MixturesAlligationGenerator));
        registry.register_validator(Arc::new(MixturesAlligationValidator));

        // 10. Remainders & Modular
        registry.register_generator(Arc::new(RemaindersModularGenerator));
        registry.register_validator(Arc::new(RemaindersModularValidator));

        // 11. Linear Inequalities
        registry.register_generator(Arc::new(LinearInequalitiesGenerator));
        registry.register_validator(Arc::new(LinearInequalitiesValidator));

        // 12. Algebraic Identities
        registry.register_generator(Arc::new(AlgebraicIdentitiesGenerator));
        registry.register_validator(Arc::new(AlgebraicIdentitiesValidator));

        // 13. Geometry Triangles
        registry.register_generator(Arc::new(GeometryTrianglesGenerator));
        registry.register_validator(Arc::new(GeometryTrianglesValidator));

        // 14. Combined Multi-Concept
        registry.register_generator(Arc::new(CombinedMultiConceptGenerator));
        registry.register_validator(Arc::new(CombinedMultiConceptValidator));

        // Register all family contracts across domains
        registry.register_all_contracts();

        // Register Declarative MVP generator
        registry.register_declarative_family(linear_equations_declarative_contract());

        // Physics generators & validators
        registry.register_physics();

        // Chemistry generators & validators
        registry.register_chemistry();

        // Reasoning generators & validators
        registry.register_reasoning();

        registry
    }

    /// Register structural and execution contracts for all 32 problem families across domains.
    pub fn register_all_contracts(&mut self) {
        // --- 14 Mathematics Families ---
        self.register_family_contract(Arc::new(ProblemFamilyContract::new(
            "family.math.percentage.successive",
            "percentage.successive",
            Domain::Mathematics,
            "schema.math.percentage.successive.v1",
            ProblemFamilyCapability::Declarative,
        ).with_variants(vec!["forward_two_step", "reverse_initial", "net_equivalent_change", "forward_three_step"])));

        self.register_family_contract(Arc::new(ProblemFamilyContract::new(
            "family.math.algebra.linear_equations",
            "algebra.linear_equations",
            Domain::Mathematics,
            "schema.algebra.linear_equations.v1",
            ProblemFamilyCapability::Declarative,
        ).with_variants(vec!["two_step_basic", "variables_both_sides", "distributive", "fractional_coefficients", "word_problem"])));

        self.register_family_contract(Arc::new(ProblemFamilyContract::new(
            "family.math.arithmetic.profit_loss",
            "arithmetic.profit_loss",
            Domain::Mathematics,
            "schema.arithmetic.profit_loss.v1",
            ProblemFamilyCapability::Declarative,
        ).with_variants(vec!["basic_profit_loss", "discount_markup", "successive_discounts", "false_weights", "faulty_transactions"])));

        self.register_family_contract(Arc::new(ProblemFamilyContract::new(
            "family.math.arithmetic.ratio",
            "arithmetic.ratio",
            Domain::Mathematics,
            "schema.arithmetic.ratio.v1",
            ProblemFamilyCapability::Declarative,
        ).with_variants(vec!["divide_amount", "missing_proportion", "three_part_ratio", "ratio_shift", "mixture_proportion"])));

        self.register_family_contract(Arc::new(ProblemFamilyContract::new(
            "family.math.arithmetic.average",
            "arithmetic.average",
            Domain::Mathematics,
            "schema.arithmetic.average.v1",
            ProblemFamilyCapability::Declarative,
        ).with_variants(vec!["direct_average", "missing_value", "inclusion_exclusion", "weighted_average", "average_speed"])));

        self.register_family_contract(Arc::new(ProblemFamilyContract::new(
            "family.math.number_system.divisibility",
            "number_system.divisibility",
            Domain::Mathematics,
            "schema.number_system.divisibility.v1",
            ProblemFamilyCapability::Declarative,
        ).with_variants(vec!["rule_based_2_to_11", "composite_rules", "missing_digit_divisibility", "remainder_properties", "factor_count"])));

        self.register_family_contract(Arc::new(ProblemFamilyContract::new(
            "family.math.time_work.basic",
            "time_work.basic",
            Domain::Mathematics,
            "schema.time_work.basic.v1",
            ProblemFamilyCapability::Declarative,
        ).with_variants(vec!["basic_two_person", "alternating_work", "pipes_cisterns", "man_days_equivalence", "efficiency_ratio"])));

        self.register_family_contract(Arc::new(ProblemFamilyContract::new(
            "family.math.arithmetic.time_speed_distance",
            "arithmetic.time_speed_distance",
            Domain::Mathematics,
            "schema.arithmetic.time_speed_distance.v1",
            ProblemFamilyCapability::Declarative,
        ).with_variants(vec!["direct_speed_time", "relative_speed_opposite", "relative_speed_same", "train_crossing", "boats_streams"])));

        self.register_family_contract(Arc::new(ProblemFamilyContract::new(
            "family.math.arithmetic.mixtures_alligation",
            "arithmetic.mixtures_alligation",
            Domain::Mathematics,
            "schema.arithmetic.mixtures_alligation.v1",
            ProblemFamilyCapability::Declarative,
        ).with_variants(vec!["simple_alligation_rule", "replacement_process", "three_component_mixture", "cost_price_mixture"])));

        self.register_family_contract(Arc::new(ProblemFamilyContract::new(
            "family.math.number_system.remainders_modular",
            "number_system.remainders_modular",
            Domain::Mathematics,
            "schema.number_system.remainders_modular.v1",
            ProblemFamilyCapability::Declarative,
        ).with_variants(vec!["direct_mod_arithmetic", "linear_congruence", "chinese_remainder_theorem_simple", "wilson_fermat_powers"])));

        self.register_family_contract(Arc::new(ProblemFamilyContract::new(
            "family.math.algebra.linear_inequalities",
            "algebra.linear_inequalities",
            Domain::Mathematics,
            "schema.algebra.linear_inequalities.v1",
            ProblemFamilyCapability::Declarative,
        ).with_variants(vec!["one_step_inequality", "two_step_inequality", "compound_inequality", "absolute_value_inequality", "sign_flip_trap"])));

        self.register_family_contract(Arc::new(ProblemFamilyContract::new(
            "family.math.algebra.algebraic_identities",
            "algebra.algebraic_identities",
            Domain::Mathematics,
            "schema.algebra.algebraic_identities.v1",
            ProblemFamilyCapability::Declarative,
        ).with_variants(vec!["square_sum_diff", "difference_of_squares", "cube_identities", "symmetric_functions", "conditional_identities"])));

        self.register_family_contract(Arc::new(ProblemFamilyContract::new(
            "family.math.geometry.triangles",
            "geometry.triangles",
            Domain::Mathematics,
            "schema.geometry.triangles.v1",
            ProblemFamilyCapability::DomainGeometry,
        ).with_variants(vec!["angle_sum_exterior", "pythagoras_triplets", "similarity_ratio", "area_hero_special", "congruence_criteria"])));

        self.register_family_contract(Arc::new(ProblemFamilyContract::new(
            "family.math.combined.multi_concept",
            "combined.multi_concept",
            Domain::Mathematics,
            "schema.combined.multi_concept.v1",
            ProblemFamilyCapability::Specialized,
        ).with_variants(vec!["percentage_and_ratio", "profit_loss_and_discount", "speed_distance_and_work", "algebra_and_geometry"])));

        // --- 2 Physics Families ---
        self.register_family_contract(Arc::new(ProblemFamilyContract::new(
            "family.physics.kinematics.1d",
            "physics.kinematics.1d",
            Domain::Physics,
            "schema.physics.kinematics.1d.v1",
            ProblemFamilyCapability::DomainPhysics,
        ).with_variants(vec!["constant_velocity", "uniform_acceleration", "free_fall_gravity", "multi_stage_motion", "stopping_distance"])));

        self.register_family_contract(Arc::new(ProblemFamilyContract::new(
            "family.physics.work_energy.mechanics",
            "physics.work_energy.mechanics",
            Domain::Physics,
            "schema.physics.work_energy.mechanics.v1",
            ProblemFamilyCapability::DomainPhysics,
        ).with_variants(vec!["work_constant_force", "kinetic_potential_conversion", "conservation_mechanical_energy", "power_work_rate", "non_conservative_friction"])));

        // --- 6 Chemistry Families ---
        self.register_family_contract(Arc::new(ProblemFamilyContract::new(
            "family.chemistry.stoichiometry.moles",
            "chemistry.stoichiometry.moles",
            Domain::Chemistry,
            "schema.chemistry.stoichiometry.moles.v1",
            ProblemFamilyCapability::DomainChemistry,
        ).with_variants(vec!["molar_mass_conversions", "reaction_stoichiometry", "limiting_reagent", "percentage_yield", "gas_volume_stoichiometry"])));

        self.register_family_contract(Arc::new(ProblemFamilyContract::new(
            "family.chemistry.equilibrium.concentration",
            "chemistry.equilibrium.concentration",
            Domain::Chemistry,
            "schema.chemistry.equilibrium.concentration.v1",
            ProblemFamilyCapability::DomainChemistry,
        ).with_variants(vec!["kc_expression", "equilibrium_concentrations", "kp_from_partial_pressures", "le_chatelier_shift", "ice_table_solver"])));

        self.register_family_contract(Arc::new(ProblemFamilyContract::new(
            "family.chemistry.ionic_equilibrium.buffers_titration",
            "chemistry.ionic_equilibrium.buffers_titration",
            Domain::Chemistry,
            "schema.chemistry.ionic_equilibrium.buffers_titration.v1",
            ProblemFamilyCapability::DomainChemistry,
        ).with_variants(vec!["ph_strong_acid_base", "buffer_henderson_hasselbalch", "titration_equivalence_point", "solubility_product_ksp", "common_ion_effect"])));

        self.register_family_contract(Arc::new(ProblemFamilyContract::new(
            "family.chemistry.electrochemistry.nernst_faraday",
            "chemistry.electrochemistry.nernst_faraday",
            Domain::Chemistry,
            "schema.chemistry.electrochemistry.nernst_faraday.v1",
            ProblemFamilyCapability::DomainChemistry,
        ).with_variants(vec!["standard_cell_potential", "nernst_equation", "faradays_laws_electrolysis", "gibbs_free_energy_cell", "galvanic_cell_notation"])));

        self.register_family_contract(Arc::new(ProblemFamilyContract::new(
            "family.chemistry.kinetics.rate_laws",
            "chemistry.kinetics.rate_laws",
            Domain::Chemistry,
            "schema.chemistry.kinetics.rate_laws.v1",
            ProblemFamilyCapability::DomainChemistry,
        ).with_variants(vec!["initial_rates_order", "integrated_first_order", "half_life_calculation", "arrhenius_activation_energy", "reaction_mechanisms"])));

        self.register_family_contract(Arc::new(ProblemFamilyContract::new(
            "family.chemistry.reaction_networks.multistage_synthesis",
            "chemistry.reaction_networks.multistage_synthesis",
            Domain::Chemistry,
            "schema.chemistry.reaction_networks.multistage_synthesis.v1",
            ProblemFamilyCapability::DomainChemistry,
        ).with_variants(vec!["two_step_synthesis", "functional_group_interconversion", "overall_yield_network", "branched_reaction_tree", "reagent_selection_chain"])));

        // --- 10 Reasoning Families ---
        self.register_family_contract(Arc::new(ProblemFamilyContract::new(
            "family.reasoning.series.pattern_recognition",
            "reasoning.series.pattern_recognition",
            Domain::Reasoning,
            "schema.reasoning.series.pattern_recognition.v1",
            ProblemFamilyCapability::Declarative,
        ).with_variants(vec!["arithmetic_progression", "geometric_progression", "alternating_series", "difference_series", "fibonacci_like"])));

        self.register_family_contract(Arc::new(ProblemFamilyContract::new(
            "family.reasoning.syllogism.formal_inference",
            "reasoning.syllogism.formal_inference",
            Domain::Reasoning,
            "schema.reasoning.syllogism.formal_inference.v1",
            ProblemFamilyCapability::SymbolicLogic,
        ).with_variants(vec!["two_premise_standard", "either_or_conclusion", "possibility_case", "three_premise_chain", "negative_premise_rules"])));

        self.register_family_contract(Arc::new(ProblemFamilyContract::new(
            "family.reasoning.seating.constraint_satisfaction",
            "reasoning.seating.constraint_satisfaction",
            Domain::Reasoning,
            "schema.reasoning.seating.constraint_satisfaction.v1",
            ProblemFamilyCapability::ConstraintSolver,
        ).with_variants(vec!["linear_single_row", "circular_facing_center", "linear_facing_north_south", "circular_mixed_facing", "parallel_rows"])));

        self.register_family_contract(Arc::new(ProblemFamilyContract::new(
            "family.reasoning.relations.graph_inference",
            "reasoning.relations.graph_inference",
            Domain::Reasoning,
            "schema.reasoning.relations.graph_inference.v1",
            ProblemFamilyCapability::SymbolicLogic,
        ).with_variants(vec!["order_and_ranking", "height_weight_comparison", "transitive_relations", "inequality_statements", "combined_relational_dag"])));

        self.register_family_contract(Arc::new(ProblemFamilyContract::new(
            "family.reasoning.floor_grid.spatial_csp",
            "reasoning.floor_grid.spatial_csp",
            Domain::Reasoning,
            "schema.reasoning.floor_grid.spatial_csp.v1",
            ProblemFamilyCapability::ConstraintSolver,
        ).with_variants(vec!["building_floors_simple", "flat_floor_matrix", "schedule_day_time_grid", "box_stack_ordering", "multi_variable_grid"])));

        self.register_family_contract(Arc::new(ProblemFamilyContract::new(
            "family.reasoning.logic_dag.multi_step_inference",
            "reasoning.logic_dag.multi_step_inference",
            Domain::Reasoning,
            "schema.reasoning.logic_dag.multi_step_inference.v1",
            ProblemFamilyCapability::SymbolicLogic,
        ).with_variants(vec!["cause_and_effect", "statement_assumptions", "course_of_action", "statement_arguments", "deductive_inference_dag"])));

        self.register_family_contract(Arc::new(ProblemFamilyContract::new(
            "family.reasoning.data_sufficiency.constraint_sufficiency",
            "reasoning.data_sufficiency.constraint_sufficiency",
            Domain::Reasoning,
            "schema.reasoning.data_sufficiency.constraint_sufficiency.v1",
            ProblemFamilyCapability::SymbolicLogic,
        ).with_variants(vec!["algebra_sufficiency", "arithmetic_sufficiency", "geometry_sufficiency", "relations_sufficiency", "ordering_sufficiency"])));

        self.register_family_contract(Arc::new(ProblemFamilyContract::new(
            "family.reasoning.coded_expressions.symbolic_operators",
            "reasoning.coded_expressions.symbolic_operators",
            Domain::Reasoning,
            "schema.reasoning.coded_expressions.symbolic_operators.v1",
            ProblemFamilyCapability::Declarative,
        ).with_variants(vec!["symbolic_arithmetic_substitution", "coded_inequalities", "operator_swap_validity", "binary_coded_logic", "nested_operator_trees"])));

        self.register_family_contract(Arc::new(ProblemFamilyContract::new(
            "family.reasoning.blood_relations.kinship_graph",
            "reasoning.blood_relations.kinship_graph",
            Domain::Reasoning,
            "schema.reasoning.blood_relations.kinship_graph.v1",
            ProblemFamilyCapability::SymbolicLogic,
        ).with_variants(vec!["single_person_pointing", "coded_blood_relations", "family_tree_puzzle", "generation_gap_deduction", "multi_generation_kinship"])));

        self.register_family_contract(Arc::new(ProblemFamilyContract::new(
            "family.reasoning.direction_sense.spatial_orientation",
            "reasoning.direction_sense.spatial_orientation",
            Domain::Reasoning,
            "schema.reasoning.direction_sense.spatial_orientation.v1",
            ProblemFamilyCapability::SymbolicLogic,
        ).with_variants(vec!["cardinal_turns_path", "shortest_distance_pythagoras", "shadow_sun_direction", "coded_direction_paths", "multi_person_relative_bearing"])));
    }

    /// Register all Physics problem generators and validators.
    pub fn register_physics(&mut self) {
        // 15. Kinematics 1D
        self.register_generator(Arc::new(Kinematics1DGenerator));
        self.register_validator(Arc::new(Kinematics1DValidator));

        // 16. Work & Energy
        self.register_generator(Arc::new(WorkEnergyGenerator));
        self.register_validator(Arc::new(WorkEnergyValidator));
    }

    /// Register all Chemistry problem generators and validators.
    pub fn register_chemistry(&mut self) {
        // 17. Stoichiometry / Mole Concept
        self.register_generator(Arc::new(StoichiometryGenerator));
        self.register_validator(Arc::new(StoichiometryValidator));

        // 18. Equilibrium & Concentration
        self.register_generator(Arc::new(EquilibriumGenerator));
        self.register_validator(Arc::new(EquilibriumValidator));

        // 23. Ionic Equilibrium (Buffers & Titration)
        self.register_generator(Arc::new(BuffersTitrationGenerator));
        self.register_validator(Arc::new(BuffersTitrationValidator));

        // 24. Electrochemistry (Nernst & Faraday)
        self.register_generator(Arc::new(ElectrochemistryGenerator));
        self.register_validator(Arc::new(ElectrochemistryValidator));

        // 25. Chemical Kinetics (Integrated Rates)
        self.register_generator(Arc::new(ChemicalKineticsGenerator));
        self.register_validator(Arc::new(ChemicalKineticsValidator));

        // 26. Reaction Networks (Multi-Stage Synthesis)
        self.register_generator(Arc::new(ReactionNetworksGenerator));
        self.register_validator(Arc::new(ReactionNetworksValidator));
    }

    /// Register all Reasoning problem generators and validators.
    pub fn register_reasoning(&mut self) {
        // 19. Series Patterns
        self.register_generator(Arc::new(SeriesGenerator));
        self.register_validator(Arc::new(SeriesValidator));

        // 20. Categorical Syllogism
        self.register_generator(Arc::new(SyllogismGenerator));
        self.register_validator(Arc::new(SyllogismValidator));

        // 21. Linear Seating Arrangement
        self.register_generator(Arc::new(SeatingGenerator));
        self.register_validator(Arc::new(SeatingValidator));

        // 22. Relational Graphs & Direction
        self.register_generator(Arc::new(RelationsGenerator));
        self.register_validator(Arc::new(RelationsValidator));

        // 27. Analytical CSP (Floor / Grid)
        self.register_generator(Arc::new(FloorGridGenerator));
        self.register_validator(Arc::new(FloorGridValidator));

        // 28. Deductive Logic (Multi-Premise DAG)
        self.register_generator(Arc::new(LogicDagGenerator));
        self.register_validator(Arc::new(LogicDagValidator));

        // 29. Meta-Cognitive (Data Sufficiency)
        self.register_generator(Arc::new(DataSufficiencyGenerator));
        self.register_validator(Arc::new(DataSufficiencyValidator));

        // 30. Graph/Relational (Coded Expressions)
        self.register_generator(Arc::new(CodedExpressionsGenerator));
        self.register_validator(Arc::new(CodedExpressionsValidator));

        // 31. Blood Relations (Kinship Graph)
        self.register_generator(Arc::new(BloodRelationsGenerator));
        self.register_validator(Arc::new(BloodRelationsValidator));

        // 32. Direction Sense (Spatial Orientation)
        self.register_generator(Arc::new(DirectionSenseGenerator));
        self.register_validator(Arc::new(DirectionSenseValidator));
    }

    /// Build canonical full registry containing all Mathematics, Physics, Chemistry, and Reasoning families.
    pub fn default_registry() -> Self {
        Self::default_maths_registry()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::problems::generators::{
        FAMILY_ALGEBRAIC_IDENTITIES, FAMILY_AVERAGE, FAMILY_COMBINED_MULTI_CONCEPT,
        FAMILY_DIVISIBILITY, FAMILY_GEOMETRY_TRIANGLES, FAMILY_LINEAR_EQUATIONS,
        FAMILY_LINEAR_INEQUALITIES, FAMILY_MIXTURES_ALLIGATION, FAMILY_PROFIT_LOSS, FAMILY_RATIO,
        FAMILY_REMAINDERS_MODULAR, FAMILY_TIME_SPEED_DISTANCE, FAMILY_TIME_WORK,
        TEMPLATE_ALGEBRAIC_IDENTITIES_V1, TEMPLATE_AVERAGE_V1, TEMPLATE_COMBINED_MULTI_CONCEPT_V1,
        TEMPLATE_DIVISIBILITY_V1, TEMPLATE_GEOMETRY_TRIANGLES_V1, TEMPLATE_LINEAR_EQUATIONS_V1,
        TEMPLATE_LINEAR_INEQUALITIES_V1, TEMPLATE_MIXTURES_ALLIGATION_V1, TEMPLATE_PROFIT_LOSS_V1,
        TEMPLATE_RATIO_V1, TEMPLATE_REMAINDERS_MODULAR_V1, TEMPLATE_TIME_SPEED_DISTANCE_V1,
        TEMPLATE_TIME_WORK_V1,
    };

    #[test]
    fn test_registry_dispatch_all_fourteen_maths_families() {
        let registry = ProblemRegistry::default_maths_registry();

        let families = vec![
            ("family.math.percentage.successive", "math.percentage.successive.v1"),
            (FAMILY_LINEAR_EQUATIONS, TEMPLATE_LINEAR_EQUATIONS_V1),
            (FAMILY_PROFIT_LOSS, TEMPLATE_PROFIT_LOSS_V1),
            (FAMILY_RATIO, TEMPLATE_RATIO_V1),
            (FAMILY_AVERAGE, TEMPLATE_AVERAGE_V1),
            (FAMILY_DIVISIBILITY, TEMPLATE_DIVISIBILITY_V1),
            (FAMILY_TIME_WORK, TEMPLATE_TIME_WORK_V1),
            (FAMILY_TIME_SPEED_DISTANCE, TEMPLATE_TIME_SPEED_DISTANCE_V1),
            (FAMILY_MIXTURES_ALLIGATION, TEMPLATE_MIXTURES_ALLIGATION_V1),
            (FAMILY_REMAINDERS_MODULAR, TEMPLATE_REMAINDERS_MODULAR_V1),
            (FAMILY_LINEAR_INEQUALITIES, TEMPLATE_LINEAR_INEQUALITIES_V1),
            (FAMILY_ALGEBRAIC_IDENTITIES, TEMPLATE_ALGEBRAIC_IDENTITIES_V1),
            (FAMILY_GEOMETRY_TRIANGLES, TEMPLATE_GEOMETRY_TRIANGLES_V1),
            (FAMILY_COMBINED_MULTI_CONCEPT, TEMPLATE_COMBINED_MULTI_CONCEPT_V1),
        ];

        for (fam_id_str, template_ref) in families {
            let fam_id = ProblemFamilyId::new(fam_id_str);
            let inst = registry
                .generate(&fam_id, template_ref, 42, 2, None)
                .unwrap();
            assert!(!inst.rendered_prompt.is_empty(), "Prompt should not be empty for {}", fam_id_str);

            let validator = registry.get_validator(fam_id_str);
            assert!(validator.is_some(), "Validator should exist for {}", fam_id_str);
        }
    }

    #[test]
    fn test_registry_dispatch_all_thirty_multi_domain_families() {
        let registry = ProblemRegistry::default_registry();

        let all_families = vec![
            // 14 Maths
            ("family.math.percentage.successive", "math.percentage.successive.v1"),
            (FAMILY_LINEAR_EQUATIONS, TEMPLATE_LINEAR_EQUATIONS_V1),
            (FAMILY_PROFIT_LOSS, TEMPLATE_PROFIT_LOSS_V1),
            (FAMILY_RATIO, TEMPLATE_RATIO_V1),
            (FAMILY_AVERAGE, TEMPLATE_AVERAGE_V1),
            (FAMILY_DIVISIBILITY, TEMPLATE_DIVISIBILITY_V1),
            (FAMILY_TIME_WORK, TEMPLATE_TIME_WORK_V1),
            (FAMILY_TIME_SPEED_DISTANCE, TEMPLATE_TIME_SPEED_DISTANCE_V1),
            (FAMILY_MIXTURES_ALLIGATION, TEMPLATE_MIXTURES_ALLIGATION_V1),
            (FAMILY_REMAINDERS_MODULAR, TEMPLATE_REMAINDERS_MODULAR_V1),
            (FAMILY_LINEAR_INEQUALITIES, TEMPLATE_LINEAR_INEQUALITIES_V1),
            (FAMILY_ALGEBRAIC_IDENTITIES, TEMPLATE_ALGEBRAIC_IDENTITIES_V1),
            (FAMILY_GEOMETRY_TRIANGLES, TEMPLATE_GEOMETRY_TRIANGLES_V1),
            (FAMILY_COMBINED_MULTI_CONCEPT, TEMPLATE_COMBINED_MULTI_CONCEPT_V1),
            // 2 Physics
            (crate::physics::generators::FAMILY_PHYSICS_KINEMATICS, crate::physics::generators::TEMPLATE_PHYSICS_KINEMATICS_V1),
            (crate::physics::generators::FAMILY_PHYSICS_WORK_ENERGY, crate::physics::generators::TEMPLATE_PHYSICS_WORK_ENERGY_V1),
            // 6 Chemistry
            (crate::chemistry::generators::FAMILY_CHEMISTRY_STOICHIOMETRY, crate::chemistry::generators::TEMPLATE_CHEMISTRY_STOICHIOMETRY_V1),
            (crate::chemistry::generators::FAMILY_CHEMISTRY_EQUILIBRIUM, crate::chemistry::generators::TEMPLATE_CHEMISTRY_EQUILIBRIUM_V1),
            (crate::chemistry::generators::FAMILY_CHEMISTRY_BUFFERS_TITRATION, crate::chemistry::generators::TEMPLATE_CHEMISTRY_BUFFERS_TITRATION_V1),
            (crate::chemistry::generators::FAMILY_CHEMISTRY_ELECTROCHEMISTRY, crate::chemistry::generators::TEMPLATE_CHEMISTRY_ELECTROCHEMISTRY_V1),
            (crate::chemistry::generators::FAMILY_CHEMISTRY_KINETICS, crate::chemistry::generators::TEMPLATE_CHEMISTRY_KINETICS_V1),
            (crate::chemistry::generators::FAMILY_CHEMISTRY_REACTION_NETWORKS, crate::chemistry::generators::TEMPLATE_CHEMISTRY_REACTION_NETWORKS_V1),
            // 8 Reasoning
            (crate::reasoning::generators::FAMILY_REASONING_SERIES, crate::reasoning::generators::TEMPLATE_REASONING_SERIES_V1),
            (crate::reasoning::generators::FAMILY_REASONING_SYLLOGISM, crate::reasoning::generators::TEMPLATE_REASONING_SYLLOGISM_V1),
            (crate::reasoning::generators::FAMILY_REASONING_SEATING, crate::reasoning::generators::TEMPLATE_REASONING_SEATING_V1),
            (crate::reasoning::generators::FAMILY_REASONING_RELATIONS, crate::reasoning::generators::TEMPLATE_REASONING_RELATIONS_V1),
            (crate::reasoning::generators::FAMILY_REASONING_FLOOR_GRID, crate::reasoning::generators::TEMPLATE_REASONING_FLOOR_GRID_V1),
            (crate::reasoning::generators::FAMILY_REASONING_LOGIC_DAG, crate::reasoning::generators::TEMPLATE_REASONING_LOGIC_DAG_V1),
            (crate::reasoning::generators::FAMILY_REASONING_DATA_SUFFICIENCY, crate::reasoning::generators::TEMPLATE_REASONING_DATA_SUFFICIENCY_V1),
            (crate::reasoning::generators::FAMILY_REASONING_CODED_EXPRESSIONS, crate::reasoning::generators::TEMPLATE_REASONING_CODED_EXPRESSIONS_V1),
        ];

        assert_eq!(all_families.len(), 30, "Must have exactly 30 registered multi-domain families");

        for (fam_id_str, template_ref) in all_families {
            let fam_id = ProblemFamilyId::new(fam_id_str);
            let inst = registry
                .generate(&fam_id, template_ref, 101, 2, None)
                .unwrap_or_else(|e| panic!("Failed to generate for {}: {:?}", fam_id_str, e));
            assert!(!inst.rendered_prompt.is_empty(), "Prompt should not be empty for {}", fam_id_str);

            let validator = registry.get_validator(fam_id_str);
            assert!(validator.is_some(), "Validator should exist for {}", fam_id_str);
        }
    }
}
