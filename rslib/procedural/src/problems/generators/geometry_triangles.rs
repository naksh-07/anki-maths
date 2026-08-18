// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

use crate::core::{ProblemFamilyId, Result};
use crate::diagnostics::ErrorCategory;
use crate::problems::generator::ProblemGenerator;
use crate::problems::steps::{
    DiagnosticConfidence, SolutionGraph, StepGraphEvaluation, StepHint, StepNode, StepType,
    StepValidator, StepwiseSubmission,
};
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

    /// Level 1: Pythagorean triplets: a^2 + b^2 = c^2, find hypotenuse or missing leg
    fn generate_level_1(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let triplets = [
            (3, 4, 5),
            (5, 12, 13),
            (8, 15, 17),
            (7, 24, 25),
            (9, 40, 41),
            (6, 8, 10),
            (12, 16, 20),
            (10, 24, 26),
        ];
        let (a, b, c) = triplets[rng.random_range(0..triplets.len())];
        let find_hypotenuse = rng.random_bool(0.5);

        let (prompt, solution, ans_val, step1, step2) = if find_hypotenuse {
            let p = format!(
                "In a right-angled triangle, the two perpendicular legs have lengths **{} cm** and **{} cm**.\n\nFind the length of the hypotenuse in centimeters.",
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
                StepHint::principle("Pythagorean theorem for hypotenuse: c^2 = a^2 + b^2."),
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
                "In a right-angled triangle, the hypotenuse is **{} cm** and one leg is **{} cm**.\n\nFind the length of the other leg in centimeters.",
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
                StepHint::principle("To find a missing leg: b^2 = c^2 - a^2."),
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
            "learning_object_level": "procedural_execution",
        }))
    }

    /// Level 2: Area and perimeter: Area = 1/2 * base * height, find missing height
    fn generate_level_2(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let base = rng.random_range(4..=16) * 2; // even number e.g. 12
        let height = rng.random_range(6..=25);
        let area = (base * height) / 2;

        let prompt = format!(
            "The area of a triangle is **{} cm²** and its base is **{} cm**.\n\nFind the corresponding height (altitude) in centimeters.",
            area, base
        );

        let solution = format!(
            "**Step 1:** Formula for triangle area:\n\
             \\[ \\text{{Area}} = \\frac{{1}}{{2}} \\times \\text{{Base}} \\times \\text{{Height}} \\]\n\n\
             **Step 2:** Rearrange to isolate height:\n\
             \\[ \\text{{Height}} = \\frac{{2 \\times \\text{{Area}}}}{{\\text{{Base}}}} \\]\n\n\
             **Step 3:** Substitute known values:\n\
             \\[ \\text{{Height}} = \\frac{{2 \\times {}}}{{{}}} = \\frac{{{}}}{{{}}} = **{}** \\text{{ cm}} \\]",
            area, base, 2 * area, base, height
        );

        let parameters = serde_json::json!({
            "variant": "area_perimeter",
            "area": area,
            "base": base,
            "height": height,
        });

        let correct_answer = serde_json::json!({
            "value": height as f64,
            "formatted": format!("{}", height),
            "unit": "cm",
            "solution": solution,
        });

        let step1 = StepNode::new(
            "double_area",
            StepType::Transformation,
            "Multiply area by 2",
            format!("2 * {} = {}", area, 2 * area),
            format!("{}", 2 * area),
        )
        .with_expected_value((2 * area) as f64)
        .with_hints(vec![
            StepHint::principle("From Area = 1/2 * b * h, we have 2 * Area = base * height."),
            StepHint::operation(format!("Multiply 2 * {}.", area)),
            StepHint::intermediate_relation(format!("2 * Area = {}", 2 * area)),
        ]);

        let step2 = StepNode::new(
            "calc_height",
            StepType::FinalAnswer,
            "Divide by base to find height",
            format!("{} / {} = {}", 2 * area, base, height),
            format!("{}", height),
        )
        .with_expected_value(height as f64)
        .with_dependencies(vec!["double_area".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Height = (2 * Area) / Base."),
            StepHint::operation(format!("Divide {} by {}.", 2 * area, base)),
            StepHint::intermediate_relation(format!("Height = {} cm", height)),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2], "calc_height");

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
            "learning_object_level": "procedural_execution",
        }))
    }

    /// Level 3: Special triangles: Equilateral triangle perimeter to altitude / area
    fn generate_level_3(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let side = rng.random_range(2..=8) * 2; // even side e.g. 6, 8, 10, 12 cm
        let perimeter = 3 * side;
        // Altitude = (sqrt(3) / 2) * side. Let's ask for the exact multiplier of sqrt(3) or numeric value
        let alt_coeff = side / 2; // since side is even, alt = alt_coeff * sqrt(3)
        let alt_approx = (alt_coeff as f64 * 3.0_f64.sqrt() * 10.0).round() / 10.0;

        let prompt = format!(
            "An equilateral triangle has a perimeter of **{} cm** (side = **{} cm**).\n\n\
             Find its altitude in centimeters (use \\(\\sqrt{{3}} \\approx 1.732\\), round to 1 decimal place).",
            perimeter, side
        );

        let solution = format!(
            "**Step 1:** The formula for the altitude \\(h\\) of an equilateral triangle of side \\(s\\) is:\n\
             \\[ h = \\frac{{\\sqrt{{3}}}}{{2}} s \\]\n\n\
             **Step 2:** Substitute side \\(s = {}\\):\n\
             \\[ h = \\frac{{\\sqrt{{3}}}}{{2}} \\times {} = {} \\sqrt{{3}} \\]\n\n\
             **Step 3:** Evaluate numerically:\n\
             \\[ h \\approx {} \\times 1.732 = **{:.1}** \\text{{ cm}} \\]",
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
            "formatted": format!("{:.1}", alt_approx),
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
        .with_expected_value(alt_approx)
        .with_hints(vec![
            StepHint::principle("Altitude of equilateral triangle = (sqrt(3) / 2) * side."),
            StepHint::operation(format!("Compute ({} / 2) * sqrt(3) = {} * sqrt(3).", side, alt_coeff)),
            StepHint::intermediate_relation(format!("Altitude = {} * sqrt(3)", alt_coeff)),
        ]);

        let step2 = StepNode::new(
            "calc_altitude_num",
            StepType::FinalAnswer,
            "Multiply by 1.732",
            format!("{} * 1.732 = {:.1}", alt_coeff, alt_approx),
            format!("{:.1}", alt_approx),
        )
        .with_expected_value(alt_approx)
        .with_dependencies(vec!["altitude_formula".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Evaluate using sqrt(3) ≈ 1.732."),
            StepHint::operation(format!("Multiply {} * 1.732.", alt_coeff)),
            StepHint::intermediate_relation(format!("Altitude = {:.1} cm", alt_approx)),
        ]);

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
            "learning_object_level": "variation",
        }))
    }

    /// Level 4: Angle relationships: Interior angles ratio / exterior angle theorem
    fn generate_level_4(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        // Ratio of 3 angles: r1 : r2 : r3 adding to 180
        // e.g. 2:3:4 -> sum = 9 -> 1 unit = 20 deg -> angles = 40, 60, 80
        let ratios = [
            (2, 3, 4, 9, 20),
            (1, 2, 3, 6, 30),
            (3, 4, 5, 12, 15),
            (2, 3, 5, 10, 18),
            (1, 3, 5, 9, 20),
        ];
        let (r1, r2, r3, sum_r, unit_deg) = ratios[rng.random_range(0..ratios.len())];
        let largest_angle = r3 * unit_deg;

        let prompt = format!(
            "The three interior angles of a triangle are in the ratio **{} : {} : {}**.\n\nFind the measure of the **largest angle** in degrees.",
            r1, r2, r3
        );

        let solution = format!(
            "**Step 1:** The sum of angles in any triangle is always \\(180^\\circ\\).\n\n\
             **Step 2:** Calculate total ratio parts:\n\
             \\[ \\text{{Total Parts}} = {} + {} + {} = {} \\]\n\n\
             **Step 3:** Find the value of one ratio part:\n\
             \\[ 1 \\text{{ part}} = \\frac{{180^\\circ}}{{{}}} = {}^\\circ \\]\n\n\
             **Step 4:** Calculate the largest angle ({} parts):\n\
             \\[ \\text{{Largest Angle}} = {} \\times {}^\\circ = **{}^\\circ** \\]",
            r1, r2, r3, sum_r, sum_r, unit_deg, r3, r3, unit_deg, largest_angle
        );

        let parameters = serde_json::json!({
            "variant": "angle_relationships",
            "ratios": [r1, r2, r3],
            "sum_parts": sum_r,
            "unit_degree": unit_deg,
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
            format!("180 / ({} + {} + {}) = 180 / {} = {}", r1, r2, r3, sum_r, unit_deg),
            format!("{}", unit_deg),
        )
        .with_expected_value(unit_deg as f64)
        .with_hints(vec![
            StepHint::principle("The sum of angles in a triangle is 180°. Divide 180 by the sum of ratio terms."),
            StepHint::operation(format!("Divide 180 by {}.", sum_r)),
            StepHint::intermediate_relation(format!("1 ratio part = {}°", unit_deg)),
        ]);

        let step2 = StepNode::new(
            "calc_largest_angle",
            StepType::FinalAnswer,
            "Multiply largest part by unit degrees",
            format!("{} * {} = {}", r3, unit_deg, largest_angle),
            format!("{}", largest_angle),
        )
        .with_expected_value(largest_angle as f64)
        .with_dependencies(vec!["calc_unit_part".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Multiply the largest ratio coefficient by the degree per part."),
            StepHint::operation(format!("Multiply {} * {}.", r3, unit_deg)),
            StepHint::intermediate_relation(format!("Largest angle = {}°", largest_angle)),
        ]);

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
            "learning_object_level": "variation",
        }))
    }

    /// Level 5: Transfer spatial word problem (Ladder leaning against vertical wall)
    fn generate_level_5(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        // Ladder of length L rests against wall with foot at distance d from wall, reaches height h
        // (5, 12, 13) or (8, 15, 17) or (7, 24, 25)
        let ladder_cases = [
            (13, 5, 12),
            (17, 8, 15),
            (25, 7, 24),
            (15, 9, 12),
            (10, 6, 8),
        ];
        let (ladder_len, base_dist, wall_height) = ladder_cases[rng.random_range(0..ladder_cases.len())];

        let prompt = format!(
            "A **{} meter long ladder** is placed against a vertical wall such that the foot of the ladder is **{} meters** away from the base of the wall.\n\n\
             How high up the wall does the top of the ladder reach in meters?",
            ladder_len, base_dist
        );

        let solution = format!(
            "**Step 1:** Model the configuration as a right-angled triangle where:\n\
             - Hypotenuse \\(c = {} \\text{{ m}}\\) (ladder length)\n\
             - Base \\(a = {} \\text{{ m}}\\) (distance from wall)\n\
             - Height \\(h = b\\) (reach on vertical wall)\n\n\
             **Step 2:** Apply Pythagorean theorem:\n\
             \\[ h^2 = c^2 - a^2 = ({})^2 - ({})^2 = {} - {} = {} \\]\n\n\
             **Step 3:** Solve for \\(h\\):\n\
             \\[ h = \\sqrt{{{}}} = **{}** \\text{{ meters}} \\]",
            ladder_len, base_dist, ladder_len, base_dist, ladder_len * ladder_len, base_dist * base_dist, wall_height * wall_height,
            wall_height * wall_height, wall_height
        );

        let parameters = serde_json::json!({
            "variant": "transfer_spatial",
            "ladder_length": ladder_len,
            "base_distance": base_dist,
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
            "Set up h^2 = L^2 - d^2",
            format!("{}^2 - {}^2 = {} - {} = {}", ladder_len, base_dist, ladder_len * ladder_len, base_dist * base_dist, wall_height * wall_height),
            format!("{}", wall_height * wall_height),
        )
        .with_expected_value((wall_height * wall_height) as f64)
        .with_hints(vec![
            StepHint::principle("The ladder forms the hypotenuse of a right triangle with the ground and wall: h^2 = L^2 - d^2."),
            StepHint::operation(format!("Compute {}^2 - {}^2.", ladder_len, base_dist)),
            StepHint::intermediate_relation(format!("h^2 = {}", wall_height * wall_height)),
        ]);

        let step2 = StepNode::new(
            "calc_height_reach",
            StepType::FinalAnswer,
            "Take square root to find reached height",
            format!("sqrt({}) = {}", wall_height * wall_height, wall_height),
            format!("{}", wall_height),
        )
        .with_expected_value(wall_height as f64)
        .with_dependencies(vec!["model_pythagoras".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Take the square root of the height squared."),
            StepHint::operation(format!("Compute sqrt({}).", wall_height * wall_height)),
            StepHint::intermediate_relation(format!("Height reach = {} meters", wall_height)),
        ]);

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
            "target_time_ms": 35_000,
            "difficulty_level": 5,
            "variant": "transfer_spatial",
            "learning_object_level": "transfer",
        }))
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
            _ => 35_000,
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

#[derive(Debug, Clone, Default)]
pub struct GeometryTrianglesValidator;

impl ProblemValidator for GeometryTrianglesValidator {
    fn family_id(&self) -> &str {
        FAMILY_GEOMETRY_TRIANGLES
    }

    fn evaluate(
        &self,
        instance: &ProblemInstance,
        student_answer: &serde_json::Value,
        time_taken_ms: u64,
        target_time_ms: u64,
    ) -> AnswerEvaluation {
        let expected_val = instance
            .correct_answer
            .get("value")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let parsed_val = NumericAnswerParser::parse_student_answer(student_answer);

        if let Some(student_num) = parsed_val {
            let diff = (student_num - expected_val).abs();
            let is_correct = diff <= 0.15;

            if is_correct {
                let score = if target_time_ms > 0 && time_taken_ms > target_time_ms {
                    0.85
                } else {
                    1.0
                };
                AnswerEvaluation::correct(score, time_taken_ms, target_time_ms)
                    .with_parsed_values(student_num, expected_val)
                    .with_diagnostic("✓ Correct geometric triangle calculation.")
            } else {
                // Check if student added instead of subtracted in Pythagoras: sqrt(c^2 + a^2) instead of sqrt(c^2 - a^2)
                let c = instance.parameters.get("c").or_else(|| instance.parameters.get("ladder_length")).and_then(|v| v.as_f64()).unwrap_or(0.0);
                let a = instance.parameters.get("a").or_else(|| instance.parameters.get("base_distance")).and_then(|v| v.as_f64()).unwrap_or(0.0);
                let find_hyp = instance.parameters.get("find_hypotenuse").and_then(|v| v.as_bool()).unwrap_or(false);

                if !find_hyp && c > 0.0 && a > 0.0 {
                    let wrong_hyp_add = (c * c + a * a).sqrt();
                    if (student_num - wrong_hyp_add).abs() <= 0.5 {
                        return AnswerEvaluation::incorrect(
                            ErrorCategory::Concept,
                            "Pythagorean confusion: To find a missing leg, subtract: b^2 = c^2 - a^2 (you added squares instead of subtracting from the hypotenuse).",
                        )
                        .with_parsed_values(student_num, expected_val);
                    }
                }

                AnswerEvaluation::incorrect(
                    ErrorCategory::Calculation,
                    format!("Calculation error: Expected {:.1}, but received {:.1}.", expected_val, student_num),
                )
                .with_parsed_values(student_num, expected_val)
            }
        } else {
            AnswerEvaluation::incorrect(
                ErrorCategory::Careless,
                "Unable to parse response. Please submit a valid number.",
            )
        }
    }

    fn evaluate_stepwise(
        &self,
        instance: &ProblemInstance,
        submission: &StepwiseSubmission,
        target_time_ms: u64,
    ) -> StepGraphEvaluation {
        if let Some(graph) = instance.solution_graph() {
            StepValidator::evaluate_submission(&graph, submission, target_time_ms)
        } else {
            StepGraphEvaluation {
                is_correct: false,
                score: 0.0,
                first_error_step: None,
                first_error_type: None,
                confidence: DiagnosticConfidence::Uncertain,
                steps_completed: submission.steps.len(),
                steps_correct: 0,
                step_evaluations: Vec::new(),
                overall_feedback: "Solution graph missing for stepwise evaluation.".to_string(),
                remediation_recommendation: None,
                first_action_latency_ms: submission.first_action_latency_ms,
                step_latencies_ms: submission.steps.iter().map(|s| s.time_taken_ms).collect(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geometry_triangles_generation_all_levels() {
        let gen = GeometryTrianglesGenerator;
        let validator = GeometryTrianglesValidator;

        for level in 1..=5 {
            let inst = gen.generate(&ProblemFamilyId::new(FAMILY_GEOMETRY_TRIANGLES), 42 + level as u64, level, None).unwrap();
            assert!(!inst.rendered_prompt.is_empty(), "Prompt non-empty for L{}", level);

            let graph = inst.solution_graph();
            assert!(graph.is_some(), "SolutionGraph exists for L{}", level);
            assert!(graph.unwrap().validate_topology(), "Topology valid for L{}", level);

            let correct_ans = inst.correct_answer.get("value").unwrap();
            let eval = validator.evaluate(&inst, correct_ans, 15000, 30000);
            assert!(eval.is_correct, "Self-eval succeeds for L{}", level);
        }
    }

    #[test]
    fn test_geometry_triangles_pythagoras_leg_confusion_diagnostic() {
        let gen = GeometryTrianglesGenerator;
        let validator = GeometryTrianglesValidator;

        let inst = gen.generate(&ProblemFamilyId::new(FAMILY_GEOMETRY_TRIANGLES), 100, 5, Some("transfer_spatial")).unwrap();
        let l = inst.parameters.get("ladder_length").unwrap().as_f64().unwrap();
        let d = inst.parameters.get("base_distance").unwrap().as_f64().unwrap();

        // Submit wrong sqrt(L^2 + d^2)
        let wrong_add = (l * l + d * d).sqrt();
        let eval = validator.evaluate(&inst, &serde_json::json!(wrong_add), 20000, 40000);
        assert!(!eval.is_correct);
        assert_eq!(eval.error_category, Some(ErrorCategory::Concept));
        assert!(eval.diagnostic_message.unwrap().contains("Pythagorean confusion"));
    }
}
