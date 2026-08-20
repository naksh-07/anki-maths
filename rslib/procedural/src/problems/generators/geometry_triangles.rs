// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

use crate::core::{ProblemFamilyId, Result};
use crate::diagnostics::ErrorCategory;
use crate::problems::generator::ProblemGenerator;
use crate::problems::steps::{SolutionGraph, StepHint, StepNode, StepType};
use crate::problems::validator::{AnswerEvaluation, NumericAnswerParser, ProblemValidator};
use crate::problems::ProblemInstance;

pub const FAMILY_GEOMETRY_TRIANGLES: &str = "family.math.geometry.triangles";
pub const TEMPLATE_GEOMETRY_TRIANGLES_V1: &str = "math.geometry.triangles.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GeometryTrianglesVariant {
    PythagoreanTriplets,
    AreaPerimeter,
    SpecialTriangles,
    AngleRelationships,
    TransferSpatial,
}

impl GeometryTrianglesVariant {
    pub fn as_str(&self) -> &'static str {
        match self {
            GeometryTrianglesVariant::PythagoreanTriplets => "pythagorean_triplets",
            GeometryTrianglesVariant::AreaPerimeter => "area_perimeter",
            GeometryTrianglesVariant::SpecialTriangles => "special_triangles",
            GeometryTrianglesVariant::AngleRelationships => "angle_relationships",
            GeometryTrianglesVariant::TransferSpatial => "transfer_spatial",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct GeometryTrianglesGenerator;

fn gcd(mut a: i64, mut b: i64) -> i64 {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    if a == 0 { 1 } else { a.abs() }
}

impl GeometryTrianglesGenerator {
    pub fn generate_problem(seed: u64, difficulty_level: u32, variant: Option<&str>) -> ProblemInstance {
        let mut rng = StdRng::seed_from_u64(seed);
        let chosen_variant = if let Some(v) = variant {
            match v {
                "pythagorean_triplets" => GeometryTrianglesVariant::PythagoreanTriplets,
                "area_perimeter" => GeometryTrianglesVariant::AreaPerimeter,
                "special_triangles" => GeometryTrianglesVariant::SpecialTriangles,
                "angle_relationships" => GeometryTrianglesVariant::AngleRelationships,
                "transfer_spatial" => GeometryTrianglesVariant::TransferSpatial,
                _ => GeometryTrianglesVariant::PythagoreanTriplets,
            }
        } else {
            match difficulty_level {
                1 => GeometryTrianglesVariant::PythagoreanTriplets,
                2 => GeometryTrianglesVariant::AreaPerimeter,
                3 => GeometryTrianglesVariant::SpecialTriangles,
                4 => GeometryTrianglesVariant::AngleRelationships,
                _ => GeometryTrianglesVariant::TransferSpatial,
            }
        };

        match chosen_variant {
            GeometryTrianglesVariant::PythagoreanTriplets => Self::generate_level_1(&mut rng, seed),
            GeometryTrianglesVariant::AreaPerimeter => Self::generate_level_2(&mut rng, seed),
            GeometryTrianglesVariant::SpecialTriangles => Self::generate_level_3(&mut rng, seed),
            GeometryTrianglesVariant::AngleRelationships => Self::generate_level_4(&mut rng, seed),
            GeometryTrianglesVariant::TransferSpatial => Self::generate_level_5(&mut rng, seed),
        }
    }

    /// Level 1: Dynamic Pythagorean triplets: a^2 + b^2 = c^2 generated via Euclid's formula.
    fn generate_level_1(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        // Generate (m, n) with m > n >= 1
        let (m, n) = loop {
            let m = rng.random_range(2..=20) as i64;
            let n = rng.random_range(1..m) as i64;
            if (m - n) % 2 == 1 && gcd(m, n) == 1 {
                break (m, n);
            }
        };
        let k = rng.random_range(1..=15) as i64;

        let a = k * (m * m - n * n);
        let b = k * (2 * m * n);
        let c = k * (m * m + n * n);

        let find_hypotenuse = rng.random_bool(0.5);

        let (prompt, solution, ans_val, step1, step2) = if find_hypotenuse {
            let p = format!(
                "In a right-angled triangle, the two perpendicular legs measure **{} cm** and **{} cm**.\n\n\
                 Find the length of the **hypotenuse** in centimeters.",
                a, b
            );
            let s = format!(
                "**Step 1:** Apply the Pythagorean theorem:\n\
                 \\[ c^2 = a^2 + b^2 = ({})^2 + ({})^2 = {} + {} = {} \\]\n\n\
                 **Step 2:** Take the square root:\n\
                 \\[ c = \\sqrt{{{}}} = **{}** \\text{{ cm}} \\]",
                a, b, a * a, b * b, c * c, c * c, c
            );
            let s1 = StepNode::new(
                "calc_sum_squares",
                StepType::Transformation,
                "Compute sum of squares of legs",
                format!("{}^2 + {}^2 = {} + {} = {}", a, b, a * a, b * b, c * c),
                format!("{}", c * c),
            )
            .with_expected_value((c * c) as f64)
            .with_hints(vec![
                StepHint::principle("Pythagorean theorem: c^2 = a^2 + b^2."),
                StepHint::operation(format!("Compute {}^2 + {}^2.", a, b)),
                StepHint::intermediate_relation(format!("c^2 = {}", c * c)),
            ]);

            let s2 = StepNode::new(
                "calc_hypotenuse",
                StepType::FinalAnswer,
                "Take square root to find hypotenuse",
                format!("sqrt({}) = {}", c * c, c),
                format!("{}", c),
            )
            .with_expected_value(c as f64)
            .with_dependencies(vec!["calc_sum_squares".to_string()])
            .as_final()
            .with_hints(vec![
                StepHint::principle("Take the principal square root of the sum of squares."),
                StepHint::operation(format!("Calculate sqrt({}).", c * c)),
                StepHint::intermediate_relation(format!("Hypotenuse c = {} cm", c)),
            ]);

            (p, s, c as f64, s1, s2)
        } else {
            let p = format!(
                "In a right-angled triangle, the hypotenuse measures **{} cm** and one leg measures **{} cm**.\n\n\
                 Find the length of the other leg in centimeters.",
                c, a
            );
            let s = format!(
                "**Step 1:** Apply the Pythagorean theorem for missing leg:\n\
                 \\[ b^2 = c^2 - a^2 = ({})^2 - ({})^2 = {} - {} = {} \\]\n\n\
                 **Step 2:** Take the square root:\n\
                 \\[ b = \\sqrt{{{}}} = **{}** \\text{{ cm}} \\]",
                c, a, c * c, a * a, b * b, b * b, b
            );
            let s1 = StepNode::new(
                "calc_diff_squares",
                StepType::Transformation,
                "Compute difference of squares",
                format!("{}^2 - {}^2 = {} - {} = {}", c, a, c * c, a * a, b * b),
                format!("{}", b * b),
            )
            .with_expected_value((b * b) as f64)
            .with_hints(vec![
                StepHint::principle("To find missing leg: b^2 = c^2 - a^2."),
                StepHint::operation(format!("Compute {}^2 - {}^2.", c, a)),
                StepHint::intermediate_relation(format!("b^2 = {}", b * b)),
            ]);

            let s2 = StepNode::new(
                "calc_leg",
                StepType::FinalAnswer,
                "Take square root to find leg",
                format!("sqrt({}) = {}", b * b, b),
                format!("{}", b),
            )
            .with_expected_value(b as f64)
            .with_dependencies(vec!["calc_diff_squares".to_string()])
            .as_final()
            .with_hints(vec![
                StepHint::principle("Take the square root of the difference of squares."),
                StepHint::operation(format!("Calculate sqrt({}).", b * b)),
                StepHint::intermediate_relation(format!("Leg b = {} cm", b)),
            ]);

            (p, s, b as f64, s1, s2)
        };

        let parameters = serde_json::json!({
            "variant": "pythagorean_triplets",
            "a": a,
            "b": b,
            "c": c,
            "find_hypotenuse": find_hypotenuse,
            "result": ans_val,
        });

        let correct_answer = serde_json::json!({
            "value": ans_val,
            "formatted": format!("{}", ans_val),
            "unit": "cm",
            "solution": solution,
        });

        let graph = SolutionGraph::new(vec![step1, step2], if find_hypotenuse { "calc_hypotenuse" } else { "calc_leg" });

        ProblemInstance::new(
            format!("inst-geom-l1-{}", seed),
            FAMILY_GEOMETRY_TRIANGLES,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(serde_json::json!({
            "target_time_ms": 25_000,
            "difficulty_level": 1,
            "variant": "pythagorean_triplets",
        }))
    }

    /// Level 2: Area and perimeter: Area = 1/2 * base * height, find missing height or base
    fn generate_level_2(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let base = (rng.random_range(2..=50) * 2) as i64; // even base e.g. 4 to 100 cm
        let height = rng.random_range(4..=100) as i64;
        let area = (base * height) / 2;

        let find_height = rng.random_bool(0.6);

        let (prompt, solution, ans_val, step1, step2) = if find_height {
            let p = format!(
                "The area of a triangle is **{} cm²** and its base length is **{} cm**.\n\n\
                 Find the corresponding height (altitude) in centimeters.",
                area, base
            );
            let s = format!(
                "**Step 1:** Formula for triangle area:\n\
                 \\[ \\text{{Area}} = \\frac{{1}}{{2}} \\times \\text{{Base}} \\times \\text{{Height}} \\]\n\n\
                 **Step 2:** Rearrange to isolate height:\n\
                 \\[ \\text{{Height}} = \\frac{{2 \\times \\text{{Area}}}}{{\\text{{Base}}}} = \\frac{{2 \\times {}}}{{{}}} = \\frac{{{}}}{{{}}} = **{}** \\text{{ cm}} \\]",
                area, base, 2 * area, base, height
            );
            let s1 = StepNode::new(
                "double_area",
                StepType::Transformation,
                "Multiply area by 2",
                format!("2 * {} = {}", area, 2 * area),
                format!("{}", 2 * area),
            )
            .with_expected_value((2 * area) as f64);

            let s2 = StepNode::new(
                "calc_height",
                StepType::FinalAnswer,
                "Divide doubled area by base",
                format!("{} / {} = {}", 2 * area, base, height),
                format!("{}", height),
            )
            .with_expected_value(height as f64)
            .with_dependencies(vec!["double_area".to_string()])
            .as_final();

            (p, s, height as f64, s1, s2)
        } else {
            let p = format!(
                "The area of a triangle is **{} cm²** and its height (altitude) is **{} cm**.\n\n\
                 Find the length of the corresponding base in centimeters.",
                area, height
            );
            let s = format!(
                "**Step 1:** Formula for triangle area:\n\
                 \\[ \\text{{Base}} = \\frac{{2 \\times \\text{{Area}}}}{{\\text{{Height}}}} = \\frac{{2 \\times {}}}{{{}}} = \\frac{{{}}}{{{}}} = **{}** \\text{{ cm}} \\]",
                area, height, 2 * area, height, base
            );
            let s1 = StepNode::new(
                "double_area",
                StepType::Transformation,
                "Multiply area by 2",
                format!("2 * {} = {}", area, 2 * area),
                format!("{}", 2 * area),
            )
            .with_expected_value((2 * area) as f64);

            let s2 = StepNode::new(
                "calc_base",
                StepType::FinalAnswer,
                "Divide doubled area by height",
                format!("{} / {} = {}", 2 * area, height, base),
                format!("{}", base),
            )
            .with_expected_value(base as f64)
            .with_dependencies(vec!["double_area".to_string()])
            .as_final();

            (p, s, base as f64, s1, s2)
        };

        let parameters = serde_json::json!({
            "variant": "area_perimeter",
            "area": area,
            "base": base,
            "height": height,
            "result": ans_val,
        });

        let correct_answer = serde_json::json!({
            "value": ans_val,
            "formatted": format!("{}", ans_val),
            "unit": "cm",
            "solution": solution,
        });

        let graph = SolutionGraph::new(vec![step1, step2], if find_height { "calc_height" } else { "calc_base" });

        ProblemInstance::new(
            format!("inst-geom-l2-{}", seed),
            FAMILY_GEOMETRY_TRIANGLES,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(serde_json::json!({
            "target_time_ms": 30_000,
            "difficulty_level": 2,
            "variant": "area_perimeter",
        }))
    }

    /// Level 3: Special triangles: Equilateral altitude/area or Inradius r = A/s
    fn generate_level_3(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let is_inradius = rng.random_bool(0.35);

        if is_inradius {
            // Right-angled triangle inradius: r = (a + b - c) / 2
            let (m, n) = loop {
                let m = rng.random_range(2..=15) as i64;
                let n = rng.random_range(1..m) as i64;
                if (m - n) % 2 == 1 && gcd(m, n) == 1 {
                    break (m, n);
                }
            };
            let k = rng.random_range(1..=10) as i64;
            let a = k * (m * m - n * n);
            let b = k * (2 * m * n);
            let c = k * (m * m + n * n);
            let r = (a + b - c) / 2;

            let prompt = format!(
                "A right-angled triangle has sides of length **{} cm**, **{} cm**, and hypotenuse **{} cm**.\n\n\
                 Find the radius of its inscribed circle (**inradius**) in centimeters.",
                a, b, c
            );

            let solution = format!(
                "**Step 1:** In a right triangle, the inradius \\(r\\) is given by:\n\
                 \\[ r = \\frac{{a + b - c}}{{2}} \\]\n\n\
                 **Step 2:** Substitute sides:\n\
                 \\[ r = \\frac{{{a} + {b} - {c}}}{{2}} = \\frac{{{}}}{{2}} = **{}** \\text{{ cm}} \\]",
                a + b - c, r
            );

            let parameters = serde_json::json!({
                "variant": "inradius_special",
                "a": a, "b": b, "c": c, "inradius": r,
            });

            let correct_answer = serde_json::json!({
                "value": r as f64,
                "formatted": format!("{}", r),
                "unit": "cm",
                "solution": solution,
            });

            let step1 = StepNode::new(
                "calc_numerator",
                StepType::Transformation,
                "Compute a + b - c",
                format!("{} + {} - {} = {}", a, b, c, a + b - c),
                format!("{}", a + b - c),
            )
            .with_expected_value((a + b - c) as f64);

            let step2 = StepNode::new(
                "calc_inradius",
                StepType::FinalAnswer,
                "Divide by 2 to find inradius",
                format!("{} / 2 = {}", a + b - c, r),
                format!("{}", r),
            )
            .with_expected_value(r as f64)
            .with_dependencies(vec!["calc_numerator".to_string()])
            .as_final();

            let graph = SolutionGraph::new(vec![step1, step2], "calc_inradius");

            ProblemInstance::new(
                format!("inst-geom-l3-{}", seed),
                FAMILY_GEOMETRY_TRIANGLES,
                seed,
                parameters,
                prompt,
                correct_answer,
            )
            .with_solution_graph(graph)
            .with_metadata(serde_json::json!({
                "target_time_ms": 35_000,
                "difficulty_level": 3,
                "variant": "special_triangles",
            }))
        } else {
            let side = (rng.random_range(2..=50) * 2) as i64; // even side e.g. 4 to 100 cm
            let perimeter = 3 * side;
            let alt_coeff = side / 2;
            let alt_approx = (alt_coeff as f64 * 3.0_f64.sqrt() * 100.0).round() / 100.0;

            let prompt = format!(
                "An equilateral triangle has a perimeter of **{} cm** (side = **{} cm**).\n\n\
                 Find its altitude in centimeters (use \\(\\sqrt{{3}} \\approx 1.732\\), round to 2 decimal places).",
                perimeter, side
            );

            let solution = format!(
                "**Step 1:** Formula for the altitude \\(h\\) of an equilateral triangle of side \\(s\\):\n\
                 \\[ h = \\frac{{\\sqrt{{3}}}}{{2}} s \\]\n\n\
                 **Step 2:** Substitute \\(s = {}\\):\n\
                 \\[ h = \\frac{{\\sqrt{{3}}}}{{2}} \\times {} = {} \\sqrt{{3}} \\]\n\n\
                 **Step 3:** Evaluate numerically:\n\
                 \\[ h \\approx {} \\times 1.732 = **{:.2}** \\text{{ cm}} \\]",
                side, side, alt_coeff, alt_coeff, alt_approx
            );

            let parameters = serde_json::json!({
                "variant": "special_triangles",
                "side": side,
                "perimeter": perimeter,
                "alt_coeff": alt_coeff,
                "altitude": alt_approx,
            });

            let correct_answer = serde_json::json!({
                "value": alt_approx,
                "formatted": format!("{:.2}", alt_approx),
                "unit": "cm",
                "solution": solution,
            });

            let step1 = StepNode::new(
                "altitude_formula",
                StepType::Transformation,
                "Apply altitude formula s * sqrt(3) / 2",
                format!("{} * sqrt(3) / 2 = {} * sqrt(3)", side, alt_coeff),
                format!("{} * sqrt(3)", alt_coeff),
            )
            .with_expected_value(alt_approx);

            let step2 = StepNode::new(
                "calc_altitude_num",
                StepType::FinalAnswer,
                "Multiply by 1.732",
                format!("{} * 1.732 = {:.2}", alt_coeff, alt_approx),
                format!("{:.2}", alt_approx),
            )
            .with_expected_value(alt_approx)
            .with_dependencies(vec!["altitude_formula".to_string()])
            .as_final();

            let graph = SolutionGraph::new(vec![step1, step2], "calc_altitude_num");

            ProblemInstance::new(
                format!("inst-geom-l3-{}", seed),
                FAMILY_GEOMETRY_TRIANGLES,
                seed,
                parameters,
                prompt,
                correct_answer,
            )
            .with_solution_graph(graph)
            .with_metadata(serde_json::json!({
                "target_time_ms": 35_000,
                "difficulty_level": 3,
                "variant": "special_triangles",
            }))
        }
    }

    /// Level 4: Angle relationships: Ratio of angles, exterior angle theorem, or isosceles vertex angle
    fn generate_level_4(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let mode = rng.random_range(0..3);

        if mode == 0 {
            // Ratio of 3 angles: r1 : r2 : r3 adding to 180
            let possible_sums = [5, 6, 9, 10, 12, 15, 18, 20, 30, 36, 45, 60, 90];
            let sum_r = possible_sums[rng.random_range(0..possible_sums.len())];
            let unit_deg = 180 / sum_r;
            
            let r1 = rng.random_range(1..sum_r - 1);
            let r2 = rng.random_range(1..sum_r - r1);
            let r3 = sum_r - r1 - r2;
            
            let mut ratios = [r1, r2, r3];
            ratios.sort_unstable(); // to easily pick the largest
            let (r1, r2, r3) = (ratios[0], ratios[1], ratios[2]);

            let largest_angle = r3 * unit_deg;

            let prompt = format!(
                "The three interior angles of a triangle are in the ratio **{} : {} : {}**.\n\n\
                 Find the measure of the **largest angle** in degrees.",
                r1, r2, r3
            );

            let solution = format!(
                "**Step 1:** Angle sum of triangle = \\(180^\\circ\\).\n\
                 Total ratio parts = \\({} + {} + {} = {}\\).\n\n\
                 **Step 2:** One ratio unit = \\(180^\\circ / {} = {}^\\circ\\).\n\n\
                 **Step 3:** Largest angle = \\({} \\times {}^\\circ = **{}^\\circ** \\]",
                r1, r2, r3, sum_r, sum_r, unit_deg, r3, unit_deg, largest_angle
            );

            let parameters = serde_json::json!({
                "variant": "angle_ratio",
                "ratios": [r1, r2, r3],
                "largest_angle": largest_angle,
            });

            let correct_answer = serde_json::json!({
                "value": largest_angle as f64,
                "formatted": format!("{}", largest_angle),
                "unit": "degrees",
                "solution": solution,
            });

            let step1 = StepNode::new(
                "calc_unit_part",
                StepType::Transformation,
                "Find degree per ratio part",
                format!("180 / {} = {}", sum_r, unit_deg),
                format!("{}", unit_deg),
            )
            .with_expected_value(unit_deg as f64);

            let step2 = StepNode::new(
                "calc_largest_angle",
                StepType::FinalAnswer,
                "Multiply largest part by unit degrees",
                format!("{} * {} = {}", r3, unit_deg, largest_angle),
                format!("{}", largest_angle),
            )
            .with_expected_value(largest_angle as f64)
            .with_dependencies(vec!["calc_unit_part".to_string()])
            .as_final();

            let graph = SolutionGraph::new(vec![step1, step2], "calc_largest_angle");

            ProblemInstance::new(
                format!("inst-geom-l4-{}", seed),
                FAMILY_GEOMETRY_TRIANGLES,
                seed,
                parameters,
                prompt,
                correct_answer,
            )
            .with_solution_graph(graph)
            .with_metadata(serde_json::json!({
                "target_time_ms": 30_000,
                "difficulty_level": 4,
                "variant": "angle_relationships",
            }))
        } else if mode == 1 {
            // Exterior angle theorem: ext_angle = int_a + int_b
            let int_a = rng.random_range(15..=90) as i64;
            let int_b = rng.random_range(15..=(175 - int_a)) as i64;
            let ext_angle = int_a + int_b;

            let prompt = format!(
                "An exterior angle of a triangle measures **{}^\\circ**, and one of its interior opposite angles measures **{}^\\circ**.\n\n\
                 Find the measure of the other interior opposite angle in degrees.",
                ext_angle, int_a
            );

            let solution = format!(
                "**Step 1:** By the Exterior Angle Theorem:\n\
                 \\[ \\text{{Exterior Angle}} = \\text{{Sum of Two Interior Opposite Angles}} \\]\n\
                 \\[ {}^\\circ = {}^\\circ + x \\]\n\n\
                 **Step 2:** Solve for \\(x\\):\n\
                 \\[ x = {}^\\circ - {}^\\circ = **{}^\\circ** \\]",
                ext_angle, int_a, ext_angle, int_a, int_b
            );

            let parameters = serde_json::json!({
                "variant": "exterior_angle",
                "ext_angle": ext_angle,
                "int_a": int_a,
                "int_b": int_b,
            });

            let correct_answer = serde_json::json!({
                "value": int_b as f64,
                "formatted": format!("{}", int_b),
                "unit": "degrees",
                "solution": solution,
            });

            let step1 = StepNode::new(
                "calc_interior_angle",
                StepType::FinalAnswer,
                "Subtract known interior angle from exterior angle",
                format!("{} - {} = {}", ext_angle, int_a, int_b),
                format!("{}", int_b),
            )
            .with_expected_value(int_b as f64)
            .as_final();

            let graph = SolutionGraph::new(vec![step1], "calc_interior_angle");

            ProblemInstance::new(
                format!("inst-geom-l4-{}", seed),
                FAMILY_GEOMETRY_TRIANGLES,
                seed,
                parameters,
                prompt,
                correct_answer,
            )
            .with_solution_graph(graph)
            .with_metadata(serde_json::json!({
                "target_time_ms": 25_000,
                "difficulty_level": 4,
                "variant": "angle_relationships",
            }))
        } else {
            // Isosceles triangle: vertex angle V, base angles B = (180 - V)/2
            let base_angle = rng.random_range(5..=85) as i64;
            let vertex_angle = 180 - 2 * base_angle;

            let find_base = rng.random_bool(0.5);

            if find_base {
                let prompt = format!(
                    "In an isosceles triangle \\(ABC\\) with \\(AB = AC\\), the vertex angle \\(\\angle A\\) measures **{}^\\circ**.\n\n\
                     Find the measure of each base angle \\(\\angle B\\) in degrees.",
                    vertex_angle
                );

                let solution = format!(
                    "**Step 1:** In an isosceles triangle, angles opposite to equal sides are equal (\\(\\angle B = \\angle C\\)).\n\n\
                     **Step 2:** Angle sum equation:\n\
                     \\[ \\angle A + 2\\angle B = 180^\\circ \\implies {}^\\circ + 2\\angle B = 180^\\circ \\]\n\n\
                     **Step 3:** Solve for \\(\\angle B\\):\n\
                     \\[ \\angle B = \\frac{{180^\\circ - {}^\\circ}}{{2}} = \\frac{{{}^\\circ}}{{2}} = **{}^\\circ** \\]",
                    vertex_angle, vertex_angle, 180 - vertex_angle, base_angle
                );

                let parameters = serde_json::json!({
                    "variant": "isosceles_angles",
                    "find_base": true,
                    "vertex_angle": vertex_angle,
                    "base_angle": base_angle,
                });

                let correct_answer = serde_json::json!({
                    "value": base_angle as f64,
                    "formatted": format!("{}", base_angle),
                    "unit": "degrees",
                    "solution": solution,
                });

                let step1 = StepNode::new(
                    "calc_base_angle",
                    StepType::FinalAnswer,
                    "Compute (180 - vertex) / 2",
                    format!("(180 - {}) / 2 = {}", vertex_angle, base_angle),
                    format!("{}", base_angle),
                )
                .with_expected_value(base_angle as f64)
                .as_final();

                let graph = SolutionGraph::new(vec![step1], "calc_base_angle");

                ProblemInstance::new(
                    format!("inst-geom-l4-{}", seed),
                    FAMILY_GEOMETRY_TRIANGLES,
                    seed,
                    parameters,
                    prompt,
                    correct_answer,
                )
                .with_solution_graph(graph)
                .with_metadata(serde_json::json!({
                    "target_time_ms": 25_000,
                    "difficulty_level": 4,
                    "variant": "angle_relationships",
                }))
            } else {
                let prompt = format!(
                    "In an isosceles triangle \\(ABC\\) with \\(AB = AC\\), one of the base angles \\(\\angle B\\) measures **{}^\\circ**.\n\n\
                     Find the measure of the vertex angle \\(\\angle A\\) in degrees.",
                    base_angle
                );

                let solution = format!(
                    "**Step 1:** In an isosceles triangle, angles opposite to equal sides are equal, so \\(\\angle C = {}^\\circ\\).\n\n\
                     **Step 2:** Angle sum equation:\n\
                     \\[ \\angle A + \\angle B + \\angle C = 180^\\circ \\implies \\angle A + {}^\\circ + {}^\\circ = 180^\\circ \\]\n\n\
                     **Step 3:** Solve for \\(\\angle A\\):\n\
                     \\[ \\angle A = 180^\\circ - {}^\\circ = **{}^\\circ** \\]",
                    base_angle, base_angle, base_angle, 2 * base_angle, vertex_angle
                );

                let parameters = serde_json::json!({
                    "variant": "isosceles_angles",
                    "find_base": false,
                    "vertex_angle": vertex_angle,
                    "base_angle": base_angle,
                });

                let correct_answer = serde_json::json!({
                    "value": vertex_angle as f64,
                    "formatted": format!("{}", vertex_angle),
                    "unit": "degrees",
                    "solution": solution,
                });

                let step1 = StepNode::new(
                    "calc_vertex_angle",
                    StepType::FinalAnswer,
                    "Compute 180 - 2 * base",
                    format!("180 - 2 * {} = {}", base_angle, vertex_angle),
                    format!("{}", vertex_angle),
                )
                .with_expected_value(vertex_angle as f64)
                .as_final();

                let graph = SolutionGraph::new(vec![step1], "calc_vertex_angle");

                ProblemInstance::new(
                    format!("inst-geom-l4-{}", seed),
                    FAMILY_GEOMETRY_TRIANGLES,
                    seed,
                    parameters,
                    prompt,
                    correct_answer,
                )
                .with_solution_graph(graph)
                .with_metadata(serde_json::json!({
                    "target_time_ms": 25_000,
                    "difficulty_level": 4,
                    "variant": "angle_relationships",
                }))
            }
        }
    }

    /// Level 5: Transfer spatial word problems: Ladders, Shadows / Similar Triangles, River Distance
    fn generate_level_5(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let is_shadow_similarity = rng.random_bool(0.5);

        if is_shadow_similarity {
            // Similar triangle shadow problem: pole of height h1 casts shadow s1, building casts shadow s2. Find h2.
            let h1 = rng.random_range(2..=20) as i64; // e.g. 3m pole
            let s1 = rng.random_range(2..=15) as i64; // casts 4m shadow
            let mult = rng.random_range(3..=30) as i64;
            let s2 = s1 * mult; // building shadow e.g. 40m
            let h2 = h1 * mult; // building height e.g. 30m

            let prompt = format!(
                "At a particular time of day, a vertical flagpole of height **{} meters** casts a shadow of length **{} meters** on level ground.\n\n\
                 At the same instant, a nearby observation tower casts a shadow of length **{} meters**.\n\
                 What is the **height of the tower** in meters?",
                h1, s1, s2
            );

            let solution = format!(
                "**Step 1:** Model the situation using similar right triangles (same sun elevation angle):\n\
                 \\[ \\frac{{\\text{{Height of Tower}}}}{{\\text{{Shadow of Tower}}}} = \\frac{{\\text{{Height of Pole}}}}{{\\text{{Shadow of Pole}}}} \\]\n\n\
                 **Step 2:** Set up the proportion:\n\
                 \\[ \\frac{{H}}{{{}}} = \\frac{{{}}}{{{}}} \\]\n\n\
                 **Step 3:** Solve for \\(H\\):\n\
                 \\[ H = {} \\times \\frac{{{}}}{{{}}} = **{}** \\text{{ meters}} \\]",
                s2, h1, s1, s2, h1, s1, h2
            );

            let parameters = serde_json::json!({
                "variant": "similar_triangles_shadow",
                "h1": h1, "s1": s1, "s2": s2, "h2": h2,
            });

            let correct_answer = serde_json::json!({
                "value": h2 as f64,
                "formatted": format!("{}", h2),
                "unit": "meters",
                "solution": solution,
            });

            let step1 = StepNode::new(
                "calc_similarity_scale",
                StepType::Transformation,
                "Find scale factor of shadows s2 / s1",
                format!("{} / {} = {}", s2, s1, mult),
                format!("{}", mult),
            )
            .with_expected_value(mult as f64);

            let step2 = StepNode::new(
                "calc_tower_height",
                StepType::FinalAnswer,
                "Multiply pole height by scale factor",
                format!("{} * {} = {}", h1, mult, h2),
                format!("{}", h2),
            )
            .with_expected_value(h2 as f64)
            .with_dependencies(vec!["calc_similarity_scale".to_string()])
            .as_final();

            let graph = SolutionGraph::new(vec![step1, step2], "calc_tower_height");

            ProblemInstance::new(
                format!("inst-geom-l5-{}", seed),
                FAMILY_GEOMETRY_TRIANGLES,
                seed,
                parameters,
                prompt,
                correct_answer,
            )
            .with_solution_graph(graph)
            .with_metadata(serde_json::json!({
                "target_time_ms": 40_000,
                "difficulty_level": 5,
                "variant": "transfer_spatial",
            }))
        } else {
            // Dynamic ladder problem with arbitrary triplet
            let (m, n) = loop {
                let m = rng.random_range(2..=15) as i64;
                let n = rng.random_range(1..m) as i64;
                if (m - n) % 2 == 1 && gcd(m, n) == 1 {
                    break (m, n);
                }
            };
            let k = rng.random_range(1..=10) as i64;
            let base_dist = k * (m * m - n * n);
            let wall_height = k * (2 * m * n);
            let ladder_len = k * (m * m + n * n);

            let prompt = format!(
                "A **{} meter long ladder** is leaned against a vertical wall such that the foot of the ladder is **{} meters** away from the base of the wall.\n\n\
                 How high up the wall does the top of the ladder reach in meters?",
                ladder_len, base_dist
            );

            let solution = format!(
                "**Step 1:** Model as a right triangle with hypotenuse \\(c = {} \\text{{ m}}\\) and base \\(a = {} \\text{{ m}}\\).\n\n\
                 **Step 2:** Apply Pythagorean theorem:\n\
                 \\[ h^2 = c^2 - a^2 = ({})^2 - ({})^2 = {} - {} = {} \\]\n\n\
                 **Step 3:** Solve for \\(h\\):\n\
                 \\[ h = \\sqrt{{{}}} = **{}** \\text{{ meters}} \\]",
                ladder_len, base_dist,
                ladder_len, base_dist, ladder_len * ladder_len, base_dist * base_dist, wall_height * wall_height,
                wall_height * wall_height, wall_height
            );

            let parameters = serde_json::json!({
                "variant": "transfer_spatial_ladder",
                "ladder_len": ladder_len,
                "base_dist": base_dist,
                "wall_height": wall_height,
            });

            let correct_answer = serde_json::json!({
                "value": wall_height as f64,
                "formatted": format!("{}", wall_height),
                "unit": "meters",
                "solution": solution,
            });

            let step1 = StepNode::new(
                "model_pythagoras",
                StepType::Transformation,
                "Compute h^2 = L^2 - d^2",
                format!("{}^2 - {}^2 = {}", ladder_len, base_dist, wall_height * wall_height),
                format!("{}", wall_height * wall_height),
            )
            .with_expected_value((wall_height * wall_height) as f64);

            let step2 = StepNode::new(
                "calc_height_reach",
                StepType::FinalAnswer,
                "Take square root to find wall reach height",
                format!("sqrt({}) = {}", wall_height * wall_height, wall_height),
                format!("{}", wall_height),
            )
            .with_expected_value(wall_height as f64)
            .with_dependencies(vec!["model_pythagoras".to_string()])
            .as_final();

            let graph = SolutionGraph::new(vec![step1, step2], "calc_height_reach");

            ProblemInstance::new(
                format!("inst-geom-l5-{}", seed),
                FAMILY_GEOMETRY_TRIANGLES,
                seed,
                parameters,
                prompt,
                correct_answer,
            )
            .with_solution_graph(graph)
            .with_metadata(serde_json::json!({
                "target_time_ms": 40_000,
                "difficulty_level": 5,
                "variant": "transfer_spatial",
            }))
        }
    }
}

impl ProblemGenerator for GeometryTrianglesGenerator {
    fn family_id(&self) -> &str {
        FAMILY_GEOMETRY_TRIANGLES
    }

    fn template_ref(&self) -> &str {
        TEMPLATE_GEOMETRY_TRIANGLES_V1
    }

    fn supported_variants(&self) -> Vec<String> {
        vec![
            "pythagorean_triplets".to_string(),
            "area_perimeter".to_string(),
            "special_triangles".to_string(),
            "angle_relationships".to_string(),
            "transfer_spatial".to_string(),
        ]
    }

    fn target_latency_ms(&self, difficulty_level: u32) -> u64 {
        match difficulty_level {
            1 => 25_000,
            2 => 30_000,
            3 => 35_000,
            4 => 30_000,
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

pub struct GeometryTrianglesValidator;

impl ProblemValidator for GeometryTrianglesValidator {
    fn family_id(&self) -> &str {
        FAMILY_GEOMETRY_TRIANGLES
    }

    fn evaluate(
        &self,
        instance: &ProblemInstance,
        student_input: &serde_json::Value,
        time_taken_ms: u64,
        target_time_ms: u64,
    ) -> AnswerEvaluation {
        let expected_val = instance
            .correct_answer
            .get("value")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let parsed_val = NumericAnswerParser::parse_value(student_input);
        let Some(student_num) = parsed_val else {
            return AnswerEvaluation {
                is_correct: false,
                score: 0.0,
                parsed_student_value: None,
                canonical_value: expected_val,
                error_category: Some(ErrorCategory::Calculation),
                diagnostic_message: Some("Could not parse answer as a number.".to_string()),
            };
        };

        let diff = (student_num - expected_val).abs();
        let is_correct = diff <= 0.1 || (expected_val > 0.0 && diff / expected_val <= 0.01);

        if is_correct {
            let score = if target_time_ms > 0 && time_taken_ms > target_time_ms * 2 {
                0.8
            } else {
                1.0
            };
            AnswerEvaluation::correct(score, time_taken_ms, target_time_ms)
                .with_parsed_values(student_num, expected_val)
        } else {
            AnswerEvaluation::incorrect(
                ErrorCategory::Calculation,
                format!("Incorrect answer. Submitted {:.2}, expected {:.2}.", student_num, expected_val),
            )
            .with_parsed_values(student_num, expected_val)
        }
    }
}
