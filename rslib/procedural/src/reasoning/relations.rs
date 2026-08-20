// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use serde::{Deserialize, Serialize};

/// Standard kinship relations in formal blood relation problems.
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
    Husband,
    Wife,
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
            KinshipRelation::PaternalUncle => "Paternal Uncle",
            KinshipRelation::MaternalUncle => "Maternal Uncle",
            KinshipRelation::PaternalAunt => "Paternal Aunt",
            KinshipRelation::MaternalAunt => "Maternal Aunt",
            KinshipRelation::Grandfather => "Grandfather",
            KinshipRelation::Grandmother => "Grandmother",
            KinshipRelation::Nephew => "Nephew",
            KinshipRelation::Niece => "Niece",
            KinshipRelation::Cousin => "Cousin",
            KinshipRelation::Husband => "Husband",
            KinshipRelation::Wife => "Wife",
        }
    }
}

/// A directed relational statement between two named individuals.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KinshipStatement {
    pub person_a: String,
    pub relation: KinshipRelation,
    pub person_b: String,
}

impl KinshipStatement {
    pub fn new(person_a: &str, relation: KinshipRelation, person_b: &str) -> Self {
        Self {
            person_a: person_a.to_string(),
            relation,
            person_b: person_b.to_string(),
        }
    }

    pub fn text(&self) -> String {
        format!("{} is the {} of {}.", self.person_a, self.relation.as_str().to_lowercase(), self.person_b)
    }
}

/// 2D Compass Heading directions.
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

    pub fn create_grandfather_chain(person_a: &str, person_b: &str, person_c: &str) -> Self {
        let s1 = KinshipStatement::new(person_a, KinshipRelation::Father, person_b);
        let s2 = KinshipStatement::new(person_b, KinshipRelation::Father, person_c);

        let explanation = format!(
            "{} is the father of {}. Since {} is the father of {}, {} is the grandfather of {}.",
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

    pub fn create_cousin_chain(person_a: &str, person_b: &str, person_c: &str, person_d: &str) -> Self {
        let s1 = KinshipStatement::new(person_a, KinshipRelation::Brother, person_b);
        let s2 = KinshipStatement::new(person_a, KinshipRelation::Father, person_c);
        let s3 = KinshipStatement::new(person_b, KinshipRelation::Father, person_d);

        let explanation = format!(
            "{} and {} are brothers. {} is child of {} and {} is child of {}. Therefore, {} is the cousin of {}.",
            person_a, person_b, person_c, person_a, person_d, person_b, person_c, person_d
        );

        Self {
            statements: vec![s1, s2, s3],
            query_from: person_c.to_string(),
            query_to: person_d.to_string(),
            target_relation: KinshipRelation::Cousin,
            explanation,
        }
    }

    pub fn create_nephew_chain(person_a: &str, person_b: &str, person_c: &str) -> Self {
        let s1 = KinshipStatement::new(person_b, KinshipRelation::Brother, person_a);
        let s2 = KinshipStatement::new(person_b, KinshipRelation::Father, person_c);

        let explanation = format!(
            "{} is the brother of {}. Since {} is the son of {}, {} is the nephew of {}.",
            person_b, person_a, person_c, person_b, person_c, person_a
        );

        Self {
            statements: vec![s1, s2],
            query_from: person_c.to_string(),
            query_to: person_a.to_string(),
            target_relation: KinshipRelation::Nephew,
            explanation,
        }
    }

    pub fn create_aunt_chain(person_a: &str, person_b: &str, person_c: &str) -> Self {
        let s1 = KinshipStatement::new(person_a, KinshipRelation::Sister, person_b);
        let s2 = KinshipStatement::new(person_b, KinshipRelation::Father, person_c);

        let explanation = format!(
            "{} is the sister of {}. Since {} is the father of {}, {} is the paternal aunt of {}.",
            person_a, person_b, person_b, person_c, person_a, person_c
        );

        Self {
            statements: vec![s1, s2],
            query_from: person_a.to_string(),
            query_to: person_c.to_string(),
            target_relation: KinshipRelation::PaternalAunt,
            explanation,
        }
    }

    pub fn is_correct(&self, submission: &str) -> bool {
        let clean_sub = submission.trim().to_lowercase();
        let clean_exp = self.target_relation.as_str().to_lowercase();
        clean_sub == clean_exp
            || clean_sub.contains(&clean_exp)
            || (clean_exp.contains("uncle") && clean_sub.contains("uncle"))
            || (clean_exp.contains("aunt") && clean_sub.contains("aunt"))
            || (clean_exp.contains("grandfather") && clean_sub.contains("grandfather"))
            || (clean_exp.contains("cousin") && clean_sub.contains("cousin"))
            || (clean_exp.contains("nephew") && clean_sub.contains("nephew"))
    }
}

/// 2D Direction sense problem instance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DirectionPuzzle {
    pub steps_text: Vec<String>,
    pub displacement_x: i32,
    pub displacement_y: i32,
    pub straight_distance_m: f64,
    pub shortest_distance_meters: i64,
    pub target_heading: Heading,
    pub final_direction_from_start: Heading,
    pub explanation: String,
}

impl DirectionPuzzle {
    pub fn create_path(d1: i32, d2: i32, d3: i32) -> Self {
        let steps = vec![
            format!("Walks {} meters North.", d1),
            format!("Turns right and walks {} meters East.", d2),
            format!("Turns right and walks {} meters South.", d3),
        ];

        let dx = d2;
        let dy = d1 - d3;

        let dist = ((dx * dx + dy * dy) as f64).sqrt();
        let dist_int = dist.round() as i64;
        let heading = match (dx.cmp(&0), dy.cmp(&0)) {
            (std::cmp::Ordering::Greater, std::cmp::Ordering::Greater) => Heading::NorthEast,
            (std::cmp::Ordering::Greater, std::cmp::Ordering::Less) => Heading::SouthEast,
            (std::cmp::Ordering::Less, std::cmp::Ordering::Greater) => Heading::NorthWest,
            (std::cmp::Ordering::Less, std::cmp::Ordering::Less) => Heading::SouthWest,
            (std::cmp::Ordering::Equal, std::cmp::Ordering::Greater) => Heading::North,
            (std::cmp::Ordering::Equal, std::cmp::Ordering::Less) => Heading::South,
            (std::cmp::Ordering::Greater, std::cmp::Ordering::Equal) => Heading::East,
            _ => Heading::West,
        };

        let explanation = format!(
            "Net displacement: \\(\\Delta x = {} \\text{{ m East}}, \\Delta y = {} - {} = {} \\text{{ m North}}\\).\n\
             Straight-line distance = \\(\\sqrt{{({})^2 + ({})^2}} = **{:.1} meters** in the **{}** direction.",
            dx, d1, d3, dy, dx, dy, dist, heading.as_str()
        );

        Self {
            steps_text: steps,
            displacement_x: dx,
            displacement_y: dy,
            straight_distance_m: dist,
            shortest_distance_meters: dist_int,
            target_heading: heading,
            final_direction_from_start: heading,
            explanation,
        }
    }

    pub fn is_correct(&self, submission: &str) -> bool {
        let clean_sub = submission.trim().to_lowercase().replace('-', " ");
        let clean_exp = self.target_heading.as_str().to_lowercase().replace('-', " ");
        clean_sub == clean_exp
            || clean_sub == format!("{}", self.shortest_distance_meters)
            || clean_sub.contains(&clean_exp)
    }
}
