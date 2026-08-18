// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use serde::{Deserialize, Serialize};

/// Standard kinship relations in genealogical graphs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KinshipRelation {
    Father,
    Mother,
    Son,
    Daughter,
    Brother,
    Sister,
    PaternalUncle,
    MaternalUncle,
    PaternalAunt,
    MaternalAunt,
    Grandfather,
    Grandmother,
    Nephew,
    Niece,
    Cousin,
}

impl KinshipRelation {
    pub fn as_str(&self) -> &'static str {
        match self {
            KinshipRelation::Father => "Father",
            KinshipRelation::Mother => "Mother",
            KinshipRelation::Son => "Son",
            KinshipRelation::Daughter => "Daughter",
            KinshipRelation::Brother => "Brother",
            KinshipRelation::Sister => "Sister",
            KinshipRelation::PaternalUncle => "Uncle",
            KinshipRelation::MaternalUncle => "Maternal Uncle",
            KinshipRelation::PaternalAunt => "Aunt",
            KinshipRelation::MaternalAunt => "Maternal Aunt",
            KinshipRelation::Grandfather => "Grandfather",
            KinshipRelation::Grandmother => "Grandmother",
            KinshipRelation::Nephew => "Nephew",
            KinshipRelation::Niece => "Niece",
            KinshipRelation::Cousin => "Cousin",
        }
    }
}

/// A statement establishing a directed kinship link: Person A is the [relation] of Person B.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KinshipStatement {
    pub person_a: String,
    pub relation: KinshipRelation,
    pub person_b: String,
}

impl KinshipStatement {
    pub fn new(person_a: impl Into<String>, relation: KinshipRelation, person_b: impl Into<String>) -> Self {
        Self {
            person_a: person_a.into(),
            relation,
            person_b: person_b.into(),
        }
    }

    pub fn text(&self) -> String {
        format!("{} is the {} of {}.", self.person_a, self.relation.as_str().to_lowercase(), self.person_b)
    }
}

/// Cardinal and ordinal 2D directional orientations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Heading {
    North,
    South,
    East,
    West,
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
}

impl Heading {
    pub fn as_str(&self) -> &'static str {
        match self {
            Heading::North => "North",
            Heading::South => "South",
            Heading::East => "East",
            Heading::West => "West",
            Heading::NorthEast => "North-East",
            Heading::NorthWest => "North-West",
            Heading::SouthEast => "South-East",
            Heading::SouthWest => "South-West",
        }
    }

    pub fn turn_left(&self) -> Self {
        match self {
            Heading::North => Heading::West,
            Heading::West => Heading::South,
            Heading::South => Heading::East,
            Heading::East => Heading::North,
            _ => *self,
        }
    }

    pub fn turn_right(&self) -> Self {
        match self {
            Heading::North => Heading::East,
            Heading::East => Heading::South,
            Heading::South => Heading::West,
            Heading::West => Heading::North,
            _ => *self,
        }
    }
}

/// A movement displacement step in 2D space.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MovementStep {
    pub distance_meters: i32,
    pub description: String,
}

/// Blood relation puzzle instance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BloodRelationPuzzle {
    pub statements: Vec<KinshipStatement>,
    pub query_from: String,
    pub query_to: String,
    pub target_relation: KinshipRelation,
    pub explanation: String,
}

impl BloodRelationPuzzle {
    /// Create a standard maternal uncle chain:
    /// "A is the brother of B. B is the mother of C. How is A related to C?" -> "Maternal Uncle"
    pub fn create_uncle_chain(person_a: &str, person_b: &str, person_c: &str) -> Self {
        let s1 = KinshipStatement::new(person_a, KinshipRelation::Brother, person_b);
        let s2 = KinshipStatement::new(person_b, KinshipRelation::Mother, person_c);

        let explanation = format!(
            "{} is the brother of {}. Since {} is the mother of {}, {} is the maternal uncle of {}.",
            person_a, person_b, person_b, person_c, person_a, person_c
        );

        Self {
            statements: vec![s1, s2],
            query_from: person_a.to_string(),
            query_to: person_c.to_string(),
            target_relation: KinshipRelation::MaternalUncle,
            explanation,
        }
    }

    /// Create a grandfather chain:
    /// "A is the father of B. B is the mother/father of C. How is A related to C?" -> "Grandfather"
    pub fn create_grandfather_chain(person_a: &str, person_b: &str, person_c: &str) -> Self {
        let s1 = KinshipStatement::new(person_a, KinshipRelation::Father, person_b);
        let s2 = KinshipStatement::new(person_b, KinshipRelation::Father, person_c);

        let explanation = format!(
            "{} is the father of {}. {} is the father of {}. Therefore, {} is the grandfather of {}.",
            person_a, person_b, person_b, person_c, person_a, person_c
        );

        Self {
            statements: vec![s1, s2],
            query_from: person_a.to_string(),
            query_to: person_c.to_string(),
            target_relation: KinshipRelation::Grandfather,
            explanation,
        }
    }

    pub fn is_correct(&self, submission: &str) -> bool {
        let clean = submission.trim().to_lowercase().replace('-', " ");
        let exp = self.target_relation.as_str().to_lowercase();
        clean == exp
            || (exp.contains("uncle") && (clean == "uncle" || clean == "maternal uncle"))
            || (exp.contains("grandfather") && (clean == "grandfather" || clean == "grand father"))
    }
}

/// Direction & displacement path puzzle instance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DirectionPuzzle {
    pub narrative: Vec<String>,
    pub final_x: i32,
    pub final_y: i32,
    pub shortest_distance_meters: i32,
    pub final_direction_from_start: Heading,
    pub explanation: String,
}

impl DirectionPuzzle {
    /// Create a standard 3-step rectangular path puzzle:
    /// Walks North d1 meters, turns right (East) and walks d2 meters, turns right (South) and walks d3 meters.
    pub fn create_path(d1_north: i32, d2_east: i32, d3_south: i32) -> Self {
        let net_x = d2_east;
        let net_y = d1_north - d3_south;

        let heading = match (net_x.signum(), net_y.signum()) {
            (0, 1) => Heading::North,
            (0, -1) => Heading::South,
            (1, 0) => Heading::East,
            (-1, 0) => Heading::West,
            (1, 1) => Heading::NorthEast,
            (-1, 1) => Heading::NorthWest,
            (1, -1) => Heading::SouthEast,
            (-1, -1) => Heading::SouthWest,
            _ => Heading::North,
        };

        let shortest_dist = ((net_x * net_x + net_y * net_y) as f64).sqrt().round() as i32;

        let narrative = vec![
            format!("A person walks {}m towards North.", d1_north),
            format!("They turn right and walk {}m.", d2_east),
            format!("They turn right again and walk {}m.", d3_south),
        ];

        let explanation = format!(
            "Net displacement: X = +{}m (East), Y = {} - {} = {:+}m ({}). Shortest distance = {}m in direction {}.",
            net_x,
            d1_north,
            d3_south,
            net_y,
            if net_y >= 0 { "North" } else { "South" },
            shortest_dist,
            heading.as_str()
        );

        Self {
            narrative,
            final_x: net_x,
            final_y: net_y,
            shortest_distance_meters: shortest_dist,
            final_direction_from_start: heading,
            explanation,
        }
    }

    pub fn is_correct(&self, submission: &str) -> bool {
        let clean = submission.trim().to_lowercase().replace('-', " ");
        let exp_dir = self.final_direction_from_start.as_str().to_lowercase().replace('-', " ");
        let exp_dist = self.shortest_distance_meters.to_string();

        clean == exp_dir
            || clean == exp_dist
            || clean.contains(&exp_dir)
            || clean.contains(&exp_dist)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blood_relation_uncle_chain() {
        let puzzle = BloodRelationPuzzle::create_uncle_chain("Rohan", "Priya", "Amit");
        assert_eq!(puzzle.target_relation, KinshipRelation::MaternalUncle);
        assert!(puzzle.is_correct("Maternal Uncle"));
        assert!(puzzle.is_correct("Uncle"));
        assert!(!puzzle.is_correct("Father"));
    }

    #[test]
    fn test_direction_vector_displacement() {
        // Walks 10m North, 4m East, 7m South -> Net X = 4, Net Y = +3 -> Distance = 5m North-East
        let puzzle = DirectionPuzzle::create_path(10, 4, 7);
        assert_eq!(puzzle.final_x, 4);
        assert_eq!(puzzle.final_y, 3);
        assert_eq!(puzzle.shortest_distance_meters, 5);
        assert_eq!(puzzle.final_direction_from_start, Heading::NorthEast);
        assert!(puzzle.is_correct("North-East"));
        assert!(puzzle.is_correct("5"));
    }
}
