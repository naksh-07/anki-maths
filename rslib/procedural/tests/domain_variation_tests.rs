// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use procedural::core::Domain;
use procedural::problems::variation::{
    ChemistryVariationDimension, DomainVariationDimension, MathsVariationDimension,
    PhysicsVariationDimension, ReasoningVariationDimension, VariationDistance,
};

#[test]
fn test_domain_variation_dimensions_and_domains() {
    let maths_var = DomainVariationDimension::Mathematics(MathsVariationDimension::Reverse);
    assert_eq!(maths_var.domain(), Domain::Mathematics);
    assert_eq!(maths_var.as_str(), "reverse");

    let physics_var = DomainVariationDimension::Physics(PhysicsVariationDimension::ModelSelection);
    assert_eq!(physics_var.domain(), Domain::Physics);
    assert_eq!(physics_var.as_str(), "model_selection");

    let chem_var = DomainVariationDimension::Chemistry(ChemistryVariationDimension::Regime);
    assert_eq!(chem_var.domain(), Domain::Chemistry);
    assert_eq!(chem_var.as_str(), "regime");

    let reasoning_var = DomainVariationDimension::Reasoning(ReasoningVariationDimension::Strategy);
    assert_eq!(reasoning_var.domain(), Domain::Reasoning);
    assert_eq!(reasoning_var.as_str(), "strategy");
}

#[test]
fn test_variation_distance_categorization() {
    // 1. Maths Parameter with same node count -> Near
    let math_param = DomainVariationDimension::Mathematics(MathsVariationDimension::Parameter);
    assert_eq!(VariationDistance::from_dimension(&math_param, 0), VariationDistance::Near);

    // 2. Maths Reverse target -> Structural
    let math_rev = DomainVariationDimension::Mathematics(MathsVariationDimension::Reverse);
    assert_eq!(VariationDistance::from_dimension(&math_rev, 1), VariationDistance::Structural);

    // 3. Maths MultiConcept -> MultiConcept
    let math_mc = DomainVariationDimension::Mathematics(MathsVariationDimension::MultiConcept);
    assert_eq!(VariationDistance::from_dimension(&math_mc, 2), VariationDistance::MultiConcept);

    // 4. Physics Transfer -> Far
    let phys_transfer = DomainVariationDimension::Physics(PhysicsVariationDimension::Transfer);
    assert_eq!(VariationDistance::from_dimension(&phys_transfer, 3), VariationDistance::Far);

    // 5. Reasoning Structure change -> Contextual
    let r_struct = DomainVariationDimension::Reasoning(ReasoningVariationDimension::Structure);
    assert_eq!(VariationDistance::from_dimension(&r_struct, 1), VariationDistance::Contextual);
}