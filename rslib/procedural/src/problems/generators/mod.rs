// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

pub mod algebraic_identities;
pub mod average;
pub mod combined_multi_concept;
pub mod divisibility;
pub mod geometry_triangles;
pub mod linear_equations;
pub mod linear_inequalities;
pub mod mixtures_alligation;
pub mod percentage_successive;
pub mod profit_loss;
pub mod ratio;
pub mod remainders_modular;
pub mod time_speed_distance;
pub mod time_work;

pub use algebraic_identities::{
    AlgebraicIdentitiesGenerator, AlgebraicIdentitiesValidator, AlgebraicIdentitiesVariant,
    FAMILY_ALGEBRAIC_IDENTITIES, TEMPLATE_ALGEBRAIC_IDENTITIES_V1,
};
pub use average::{AverageGenerator, AverageValidator, AverageVariant, FAMILY_AVERAGE, TEMPLATE_AVERAGE_V1};
pub use combined_multi_concept::{
    CombinedMultiConceptGenerator, CombinedMultiConceptValidator, CombinedMultiConceptVariant,
    FAMILY_COMBINED_MULTI_CONCEPT, TEMPLATE_COMBINED_MULTI_CONCEPT_V1,
};
pub use divisibility::{
    DivisibilityGenerator, DivisibilityValidator, DivisibilityVariant, FAMILY_DIVISIBILITY,
    TEMPLATE_DIVISIBILITY_V1,
};
pub use geometry_triangles::{
    GeometryTrianglesGenerator, GeometryTrianglesValidator, GeometryTrianglesVariant,
    FAMILY_GEOMETRY_TRIANGLES, TEMPLATE_GEOMETRY_TRIANGLES_V1,
};
pub use linear_equations::{
    LinearEquationVariant, LinearEquationsGenerator, LinearEquationsValidator,
    FAMILY_LINEAR_EQUATIONS, TEMPLATE_LINEAR_EQUATIONS_V1,
};
pub use linear_inequalities::{
    LinearInequalitiesGenerator, LinearInequalitiesValidator, LinearInequalitiesVariant,
    FAMILY_LINEAR_INEQUALITIES, TEMPLATE_LINEAR_INEQUALITIES_V1,
};
pub use mixtures_alligation::{
    MixturesAlligationGenerator, MixturesAlligationValidator, MixturesAlligationVariant,
    FAMILY_MIXTURES_ALLIGATION, TEMPLATE_MIXTURES_ALLIGATION_V1,
};
pub use percentage_successive::{
    ChangeDirection, GeneratedPercentageProblem, PercentageStep, PercentageSuccessiveConfig,
    PercentageSuccessiveGenerator, PercentageVariant,
};
pub use profit_loss::{
    ProfitLossGenerator, ProfitLossValidator, ProfitLossVariant, FAMILY_PROFIT_LOSS,
    TEMPLATE_PROFIT_LOSS_V1,
};
pub use ratio::{RatioGenerator, RatioValidator, RatioVariant, FAMILY_RATIO, TEMPLATE_RATIO_V1};
pub use remainders_modular::{
    RemaindersModularGenerator, RemaindersModularValidator, RemaindersModularVariant,
    FAMILY_REMAINDERS_MODULAR, TEMPLATE_REMAINDERS_MODULAR_V1,
};
pub use time_speed_distance::{
    TimeSpeedDistanceGenerator, TimeSpeedDistanceValidator, TimeSpeedDistanceVariant,
    FAMILY_TIME_SPEED_DISTANCE, TEMPLATE_TIME_SPEED_DISTANCE_V1,
};
pub use time_work::{
    TimeWorkGenerator, TimeWorkValidator, TimeWorkVariant, FAMILY_TIME_WORK, TEMPLATE_TIME_WORK_V1,
};
