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
use crate::core::{ProblemFamilyId, ProceduralError, Result};
use crate::physics::generators::{
    Kinematics1DGenerator, Kinematics1DValidator, WorkEnergyGenerator, WorkEnergyValidator,
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
}

impl Default for ProblemRegistry {
    fn default() -> Self {
        Self::default_maths_registry()
    }
}

impl ProblemRegistry {
    pub fn new() -> Self {
        Self::default()
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

    /// Generate a problem instance deterministically through dynamic dispatch.
    pub fn generate(
        &self,
        family_id: &ProblemFamilyId,
        template_ref: &str,
        seed: u64,
        difficulty_level: u32,
        variant: Option<&str>,
    ) -> Result<ProblemInstance> {
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

    /// Build canonical Mathematics registry containing all 14 topic generators and validators.
    pub fn default_maths_registry() -> Self {
        let mut registry = Self {
            generators_by_family: HashMap::new(),
            generators_by_template: HashMap::new(),
            validators_by_family: HashMap::new(),
        };

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

        // Physics generators & validators
        registry.register_physics();

        // Chemistry generators & validators
        registry.register_chemistry();

        // Reasoning generators & validators
        registry.register_reasoning();

        registry
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
