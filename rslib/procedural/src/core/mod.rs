// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// Identifiers for learning domains supported by the engine.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Domain {
    Mathematics,
    Physics,
    Chemistry,
    Reasoning,
    #[serde(untagged)]
    Custom(String),
}

impl Domain {
    pub fn as_str(&self) -> &str {
        match self {
            Domain::Mathematics => "mathematics",
            Domain::Physics => "physics",
            Domain::Chemistry => "chemistry",
            Domain::Reasoning => "reasoning",
            Domain::Custom(s) => s.as_str(),
        }
    }
}

impl fmt::Display for Domain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for Domain {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(match s {
            "mathematics" | "math" | "maths" => Domain::Mathematics,
            "physics" => Domain::Physics,
            "chemistry" | "chem" => Domain::Chemistry,
            "reasoning" | "logic" => Domain::Reasoning,
            other => Domain::Custom(other.to_string()),
        })
    }
}

macro_rules! define_id {
    ($name:ident, $doc:expr) => {
        #[doc = $doc]
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub String);

        impl $name {
            pub fn new(id: impl Into<String>) -> Self {
                Self(id.into())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl From<&str> for $name {
            fn from(s: &str) -> Self {
                Self(s.to_string())
            }
        }

        impl From<String> for $name {
            fn from(s: String) -> Self {
                Self(s)
            }
        }

        impl From<&$name> for $name {
            fn from(id: &$name) -> Self {
                id.clone()
            }
        }
    };
}

define_id!(SkillId, "Unique identifier for a discrete skill node.");
define_id!(
    ProblemFamilyId,
    "Unique identifier for a problem generator family."
);
define_id!(
    ProblemInstanceId,
    "Unique identifier for a concrete generated problem instance."
);
define_id!(
    SchemaId,
    "Unique identifier for a procedural learning practice schema."
);
define_id!(
    AttemptId,
    "Unique identifier for a learner's practice attempt."
);
define_id!(
    ErrorEventId,
    "Unique identifier for a diagnostic error event."
);
define_id!(
    PyqId,
    "Unique identifier for an authentic Previous Year Question (PYQ) source."
);
define_id!(
    ExamProfileId,
    "Unique identifier for a target competitive exam profile."
);
define_id!(
    RejectedVariantId,
    "Unique identifier for an audit record of a rejected problem variant."
);

/// Procedural engine error representation.
#[derive(Debug)]
pub enum ProceduralError {
    Database(String),
    NotFound(String),
    Validation(String),
    Serialization(String),
    Migration(String),
    InvalidAnchor(String),
}

impl From<rusqlite::Error> for ProceduralError {
    fn from(err: rusqlite::Error) -> Self {
        ProceduralError::Database(err.to_string())
    }
}

impl From<serde_json::Error> for ProceduralError {
    fn from(err: serde_json::Error) -> Self {
        ProceduralError::Serialization(err.to_string())
    }
}

impl fmt::Display for ProceduralError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProceduralError::Database(s) => write!(f, "Database error: {s}"),
            ProceduralError::NotFound(s) => write!(f, "Entity not found: {s}"),
            ProceduralError::Validation(s) => write!(f, "Validation error: {s}"),
            ProceduralError::Serialization(s) => write!(f, "Serialization error: {s}"),
            ProceduralError::Migration(s) => write!(f, "Migration error: {s}"),
            ProceduralError::InvalidAnchor(s) => write!(f, "Invalid anchor format: {s}"),
        }
    }
}

impl std::error::Error for ProceduralError {}

pub type Result<T> = std::result::Result<T, ProceduralError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_parsing() {
        assert_eq!(
            "mathematics".parse::<Domain>().unwrap(),
            Domain::Mathematics
        );
        assert_eq!("physics".parse::<Domain>().unwrap(), Domain::Physics);
        assert_eq!("chem".parse::<Domain>().unwrap(), Domain::Chemistry);
        assert_eq!("reasoning".parse::<Domain>().unwrap(), Domain::Reasoning);
        assert_eq!(
            "quantum_mech".parse::<Domain>().unwrap(),
            Domain::Custom("quantum_mech".to_string())
        );
    }

    #[test]
    fn test_id_conversions() {
        let skill_id = SkillId::from("algebra.quadratics");
        assert_eq!(skill_id.as_str(), "algebra.quadratics");
        assert_eq!(skill_id.to_string(), "algebra.quadratics");
    }
}
