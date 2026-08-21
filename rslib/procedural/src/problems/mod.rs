// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

pub mod catalog;
pub mod contract;
pub mod declarative;
pub mod generator;
pub mod generators;
pub mod registry;
pub mod steps;
pub mod validator;
pub mod variation;

pub use catalog::*;
pub use contract::*;
pub use declarative::*;
pub use generator::*;
pub use generators::*;
pub use registry::*;
pub use steps::*;
pub use validator::*;
pub use variation::*;

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::core::{Domain, ProblemFamilyId, ProblemInstanceId, SkillId};

/// Group or generator family defining a class of procedurally generated problems.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProblemFamily {
    pub id: ProblemFamilyId,
    pub skill_id: SkillId,
    pub domain: Domain,
    pub name: String,
    /// Reference identifier to the generator template or engine module
    pub template_ref: String,
    pub min_difficulty: f64,
    pub max_difficulty: f64,
    /// JSON schema describing the valid generated parameters
    pub parameters_schema: serde_json::Value,
    pub metadata: serde_json::Value,
    pub created_at: i64,
}

impl ProblemFamily {
    pub fn new(
        id: impl Into<ProblemFamilyId>,
        skill_id: impl Into<SkillId>,
        domain: Domain,
        name: impl Into<String>,
        template_ref: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            skill_id: skill_id.into(),
            domain,
            name: name.into(),
            template_ref: template_ref.into(),
            min_difficulty: 1.0,
            max_difficulty: 5.0,
            parameters_schema: serde_json::Value::Object(Default::default()),
            metadata: serde_json::Value::Object(Default::default()),
            created_at: Utc::now().timestamp(),
        }
    }

    pub fn with_difficulty_range(mut self, min: f64, max: f64) -> Self {
        self.min_difficulty = min;
        self.max_difficulty = max;
        self
    }

    pub fn with_schema(mut self, schema: serde_json::Value) -> Self {
        self.parameters_schema = schema;
        self
    }
}

/// A concrete, ephemeral generated practice problem instance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProblemInstance {
    pub id: ProblemInstanceId,
    pub family_id: ProblemFamilyId,
    /// Deterministic seed that generated this problem instance
    pub seed: u64,
    /// Concrete evaluated parameter map for this instance
    pub parameters: serde_json::Value,
    /// Rendered prompt (LaTeX/Markdown/HTML/Text)
    pub rendered_prompt: String,
    /// Canonical solution / structured answer key
    pub correct_answer: serde_json::Value,
    pub metadata: serde_json::Value,
    pub created_at: i64,
}

impl ProblemInstance {
    pub fn new(
        id: impl Into<ProblemInstanceId>,
        family_id: impl Into<ProblemFamilyId>,
        seed: u64,
        parameters: serde_json::Value,
        rendered_prompt: impl Into<String>,
        correct_answer: serde_json::Value,
    ) -> Self {
        Self {
            id: id.into(),
            family_id: family_id.into(),
            seed,
            parameters,
            rendered_prompt: rendered_prompt.into(),
            correct_answer,
            metadata: serde_json::Value::Object(Default::default()),
            created_at: Utc::now().timestamp(),
        }
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }

    /// Retrieve the structured solution graph if present in correct_answer or metadata.
    pub fn solution_graph(&self) -> Option<SolutionGraph> {
        self.correct_answer
            .get("solution_graph")
            .or_else(|| self.metadata.get("solution_graph"))
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// Attach a structured solution graph into the problem instance's correct_answer.
    pub fn with_solution_graph(mut self, graph: SolutionGraph) -> Self {
        if let Some(obj) = self.correct_answer.as_object_mut() {
            if let Ok(val) = serde_json::to_value(&graph) {
                obj.insert("solution_graph".to_string(), val);
            }
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_problem_family_and_instance() {
        let family = ProblemFamily::new(
            "physics.kinematics.freefall",
            "physics.kinematics.1d",
            Domain::Physics,
            "Free Fall Calculation",
            "kinematics.freefall.v1",
        )
        .with_difficulty_range(1.0, 3.0);

        let instance = ProblemInstance::new(
            "inst-12345",
            family.id.clone(),
            42,
            serde_json::json!({ "height": 20.0, "g": 9.8 }),
            "A ball drops from 20m. Find impact velocity.",
            serde_json::json!({ "velocity": 19.799 }),
        );

        assert_eq!(instance.seed, 42);
        assert_eq!(instance.family_id, family.id);
    }
}
