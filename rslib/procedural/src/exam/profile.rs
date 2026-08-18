// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::collections::HashMap;

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::core::{Domain, ExamProfileId, SchemaId};

/// Strategic preparation objective for an exam study profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExamObjective {
    /// Maximize speed, quick recognition, and time-bound fluency
    SpeedAndAccuracy,
    /// Deep concept grounding, structural variations, and transfer readiness
    ConceptMastery,
    /// Balanced coverage across syllabus topics and standard difficulty bands
    BalancedPreparation,
    /// Timed simulation replicating exact exam blueprint distribution
    ComprehensiveMock,
    /// Aggressive focus on struggling topics and high-error schemas
    WeakAreaRemediation,
}

impl ExamObjective {
    pub fn as_str(&self) -> &'static str {
        match self {
            ExamObjective::SpeedAndAccuracy => "speed_and_accuracy",
            ExamObjective::ConceptMastery => "concept_mastery",
            ExamObjective::BalancedPreparation => "balanced_preparation",
            ExamObjective::ComprehensiveMock => "comprehensive_mock",
            ExamObjective::WeakAreaRemediation => "weak_area_remediation",
        }
    }
}

impl std::fmt::Display for ExamObjective {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Comprehensive blueprint specifying syllabus constraints, domain weights,
/// topic weights, difficulty distributions, and target latencies for a target competitive exam.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExamProfile {
    pub id: ExamProfileId,
    pub name: String,
    pub description: String,
    pub subjects: Vec<Domain>,
    /// Relative weights across academic domains (e.g. Maths: 0.35, Reasoning: 0.35, Physics: 0.20, Chemistry: 0.10)
    pub domain_weights: HashMap<Domain, f64>,
    /// Relative importance multipliers for specific schemas or topics (e.g. "arithmetic.time_speed_distance": 1.5)
    pub topic_weights: HashMap<String, f64>,
    /// Preferred question formats (e.g. "multiple_choice", "numerical_input")
    pub preferred_formats: Vec<String>,
    /// Target solving time benchmarks in milliseconds for schemas under this exam
    pub target_latencies_ms: HashMap<String, u64>,
    /// Target difficulty distribution mapping discrete level (1..=5) to target percentage (0.0..=1.0)
    pub difficulty_distribution: HashMap<u32, f64>,
    /// Multiplier weighting authentic PYQ instances relative to procedural variants (e.g. 1.5)
    pub pyq_weight: f64,
    /// High-level learning objective
    pub objective: ExamObjective,
    pub metadata: serde_json::Value,
    pub created_at: i64,
}

impl ExamProfile {
    pub fn new(
        id: impl Into<ExamProfileId>,
        name: impl Into<String>,
        description: impl Into<String>,
        subjects: Vec<Domain>,
        objective: ExamObjective,
    ) -> Self {
        let mut domain_weights = HashMap::new();
        let count = subjects.len();
        if count > 0 {
            let equal_weight = 1.0 / count as f64;
            for domain in &subjects {
                domain_weights.insert(domain.clone(), equal_weight);
            }
        }

        let mut diff_dist = HashMap::new();
        diff_dist.insert(1, 0.20);
        diff_dist.insert(2, 0.40);
        diff_dist.insert(3, 0.30);
        diff_dist.insert(4, 0.10);
        diff_dist.insert(5, 0.00);

        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            subjects,
            domain_weights,
            topic_weights: HashMap::new(),
            preferred_formats: vec!["multiple_choice".to_string(), "numerical_input".to_string()],
            target_latencies_ms: HashMap::new(),
            difficulty_distribution: diff_dist,
            pyq_weight: 1.5,
            objective,
            metadata: serde_json::Value::Object(Default::default()),
            created_at: Utc::now().timestamp(),
        }
    }

    pub fn with_domain_weights(mut self, weights: HashMap<Domain, f64>) -> Self {
        self.domain_weights = weights;
        self
    }

    pub fn with_topic_weights(mut self, weights: HashMap<String, f64>) -> Self {
        self.topic_weights = weights;
        self
    }

    pub fn with_target_latencies(mut self, latencies: HashMap<String, u64>) -> Self {
        self.target_latencies_ms = latencies;
        self
    }

    pub fn with_difficulty_distribution(mut self, dist: HashMap<u32, f64>) -> Self {
        self.difficulty_distribution = dist;
        self
    }

    pub fn with_pyq_weight(mut self, weight: f64) -> Self {
        self.pyq_weight = weight;
        self
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }

    /// Retrieve the effective domain weight, defaulting to 1.0 / N if not specified.
    pub fn get_domain_weight(&self, domain: &Domain) -> f64 {
        self.domain_weights
            .get(domain)
            .copied()
            .unwrap_or_else(|| {
                if self.subjects.contains(domain) {
                    1.0 / self.subjects.len().max(1) as f64
                } else {
                    0.0
                }
            })
    }

    /// Retrieve the effective topic/schema weight multiplier, defaulting to 1.0.
    pub fn get_topic_weight(&self, schema_or_topic: &str) -> f64 {
        self.topic_weights
            .get(schema_or_topic)
            .copied()
            .unwrap_or(1.0)
    }

    /// Retrieve the target latency in milliseconds for a specific schema,
    /// falling back to a domain-level default if not overridden.
    pub fn get_target_latency_ms(&self, schema_id: &SchemaId, domain: &Domain) -> u64 {
        if let Some(&lat) = self.target_latencies_ms.get(schema_id.as_str()) {
            return lat;
        }

        // Domain-specific sensible default latencies calibrated for typical exams
        match domain {
            Domain::Mathematics => 45_000,
            Domain::Physics => 60_000,
            Domain::Chemistry => 50_000,
            Domain::Reasoning => 40_000,
            Domain::Custom(_) => 45_000,
        }
    }

    // =========================================================================
    // CANONICAL PRE-CONFIGURED EXAM BLUEPRINTS
    // =========================================================================

    /// Canonical profile for Railway Recruitment Board (RRB) Assistant Loco Pilot (ALP).
    /// Emphasizes Basic Arithmetic (Time-Speed-Distance, Work, Percentages),
    /// General Intelligence (Reasoning Series & Seating), and General Science (Kinematics & Stoichiometry).
    pub fn rrb_alp() -> Self {
        let subjects = vec![
            Domain::Mathematics,
            Domain::Reasoning,
            Domain::Physics,
            Domain::Chemistry,
        ];

        let mut domain_weights = HashMap::new();
        domain_weights.insert(Domain::Mathematics, 0.35);
        domain_weights.insert(Domain::Reasoning, 0.35);
        domain_weights.insert(Domain::Physics, 0.20);
        domain_weights.insert(Domain::Chemistry, 0.10);

        let mut topic_weights = HashMap::new();
        topic_weights.insert("schema.math.arithmetic.time_speed_distance".into(), 1.5);
        topic_weights.insert("schema.math.arithmetic.time_work".into(), 1.4);
        topic_weights.insert("schema.math.percentage.successive".into(), 1.3);
        topic_weights.insert("schema.reasoning.series.pattern_recognition".into(), 1.5);
        topic_weights.insert("schema.reasoning.seating.constraint_satisfaction".into(), 1.4);
        topic_weights.insert("schema.physics.kinematics.1d".into(), 1.3);
        topic_weights.insert("schema.chemistry.stoichiometry.moles".into(), 1.2);

        let mut diff_dist = HashMap::new();
        diff_dist.insert(1, 0.25);
        diff_dist.insert(2, 0.50);
        diff_dist.insert(3, 0.25);
        diff_dist.insert(4, 0.00);
        diff_dist.insert(5, 0.00);

        let mut target_latencies = HashMap::new();
        target_latencies.insert("schema.math.arithmetic.time_speed_distance".into(), 35_000);
        target_latencies.insert("schema.reasoning.series.pattern_recognition".into(), 25_000);
        target_latencies.insert("schema.physics.kinematics.1d".into(), 40_000);

        Self::new(
            "rrb_alp",
            "RRB Assistant Loco Pilot (ALP)",
            "Railway recruitment syllabus emphasizing arithmetic speed, reasoning patterns, and applied science fundamentals.",
            subjects,
            ExamObjective::SpeedAndAccuracy,
        )
        .with_domain_weights(domain_weights)
        .with_topic_weights(topic_weights)
        .with_difficulty_distribution(diff_dist)
        .with_target_latencies(target_latencies)
        .with_pyq_weight(1.5)
    }

    /// Canonical profile for Staff Selection Commission Combined Graduate Level (SSC CGL).
    /// Emphasizes Quantitative Aptitude (Algebra, Geometry, Arithmetic) and Logical Reasoning.
    pub fn ssc_cgl() -> Self {
        let subjects = vec![
            Domain::Mathematics,
            Domain::Reasoning,
            Domain::Physics,
            Domain::Chemistry,
        ];

        let mut domain_weights = HashMap::new();
        domain_weights.insert(Domain::Mathematics, 0.40);
        domain_weights.insert(Domain::Reasoning, 0.40);
        domain_weights.insert(Domain::Physics, 0.10);
        domain_weights.insert(Domain::Chemistry, 0.10);

        let mut topic_weights = HashMap::new();
        topic_weights.insert("schema.math.algebra.algebraic_identities".into(), 1.6);
        topic_weights.insert("schema.math.geometry.triangles".into(), 1.5);
        topic_weights.insert("schema.math.arithmetic.profit_loss".into(), 1.4);
        topic_weights.insert("schema.reasoning.syllogism.formal_inference".into(), 1.5);
        topic_weights.insert("schema.reasoning.relations.graph_inference".into(), 1.4);

        let mut diff_dist = HashMap::new();
        diff_dist.insert(1, 0.15);
        diff_dist.insert(2, 0.40);
        diff_dist.insert(3, 0.35);
        diff_dist.insert(4, 0.10);
        diff_dist.insert(5, 0.00);

        Self::new(
            "ssc_cgl",
            "SSC Combined Graduate Level (CGL)",
            "Tier 1/Tier 2 quantitative aptitude and reasoning syllabus requiring quick algebra and deduction.",
            subjects,
            ExamObjective::BalancedPreparation,
        )
        .with_domain_weights(domain_weights)
        .with_topic_weights(topic_weights)
        .with_difficulty_distribution(diff_dist)
        .with_pyq_weight(1.4)
    }

    /// Canonical profile for Banking Examination (IBPS / SBI PO & Clerk).
    /// Emphasizes high-complexity Logical Reasoning (Puzzles, Seating Arrangements, Syllogisms)
    /// and Quantitative Data Interpretation (Ratios, Mixtures, Percentages).
    pub fn banking_po() -> Self {
        let subjects = vec![Domain::Reasoning, Domain::Mathematics];

        let mut domain_weights = HashMap::new();
        domain_weights.insert(Domain::Reasoning, 0.50);
        domain_weights.insert(Domain::Mathematics, 0.50);

        let mut topic_weights = HashMap::new();
        topic_weights.insert("schema.reasoning.seating.constraint_satisfaction".into(), 1.8);
        topic_weights.insert("schema.reasoning.syllogism.formal_inference".into(), 1.6);
        topic_weights.insert("schema.math.arithmetic.mixtures_alligation".into(), 1.5);
        topic_weights.insert("schema.math.arithmetic.ratio".into(), 1.4);
        topic_weights.insert("schema.math.percentage.successive".into(), 1.4);

        let mut diff_dist = HashMap::new();
        diff_dist.insert(1, 0.05);
        diff_dist.insert(2, 0.25);
        diff_dist.insert(3, 0.45);
        diff_dist.insert(4, 0.25);
        diff_dist.insert(5, 0.00);

        let mut target_latencies = HashMap::new();
        target_latencies.insert("schema.reasoning.seating.constraint_satisfaction".into(), 75_000);
        target_latencies.insert("schema.reasoning.syllogism.formal_inference".into(), 30_000);

        Self::new(
            "banking_po",
            "Banking Probationary Officer (IBPS/SBI PO)",
            "Banking exam profile emphasizing high-complexity seating puzzles, syllogisms, and commercial arithmetic.",
            subjects,
            ExamObjective::SpeedAndAccuracy,
        )
        .with_domain_weights(domain_weights)
        .with_topic_weights(topic_weights)
        .with_difficulty_distribution(diff_dist)
        .with_target_latencies(target_latencies)
        .with_pyq_weight(1.3)
    }

    /// Canonical profile for Joint Entrance Examination (JEE) Main Foundation.
    /// Emphasizes Physics Mechanics, Chemical Equilibrium & Stoichiometry, and Advanced Mathematics.
    pub fn jee_main_foundation() -> Self {
        let subjects = vec![Domain::Physics, Domain::Chemistry, Domain::Mathematics];

        let mut domain_weights = HashMap::new();
        domain_weights.insert(Domain::Physics, 0.35);
        domain_weights.insert(Domain::Chemistry, 0.35);
        domain_weights.insert(Domain::Mathematics, 0.30);

        let mut topic_weights = HashMap::new();
        topic_weights.insert("schema.physics.kinematics.1d".into(), 1.5);
        topic_weights.insert("schema.physics.work_energy.mechanics".into(), 1.6);
        topic_weights.insert("schema.chemistry.stoichiometry.moles".into(), 1.4);
        topic_weights.insert("schema.chemistry.equilibrium.concentration".into(), 1.6);
        topic_weights.insert("schema.math.geometry.triangles".into(), 1.4);
        topic_weights.insert("schema.math.algebra.algebraic_identities".into(), 1.3);

        let mut diff_dist = HashMap::new();
        diff_dist.insert(1, 0.00);
        diff_dist.insert(2, 0.15);
        diff_dist.insert(3, 0.40);
        diff_dist.insert(4, 0.35);
        diff_dist.insert(5, 0.10);

        let mut target_latencies = HashMap::new();
        target_latencies.insert("schema.physics.kinematics.1d".into(), 75_000);
        target_latencies.insert("schema.physics.work_energy.mechanics".into(), 90_000);
        target_latencies.insert("schema.chemistry.equilibrium.concentration".into(), 80_000);

        Self::new(
            "jee_main_foundation",
            "JEE Main Foundation (PCM)",
            "Engineering entrance preparation focusing on conceptual physics mechanics, chemical equilibrium, and advanced geometry.",
            subjects,
            ExamObjective::ConceptMastery,
        )
        .with_domain_weights(domain_weights)
        .with_topic_weights(topic_weights)
        .with_difficulty_distribution(diff_dist)
        .with_target_latencies(target_latencies)
        .with_pyq_weight(1.6)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exam_profiles_and_domain_weighting() {
        let rrb = ExamProfile::rrb_alp();
        assert_eq!(rrb.id.as_str(), "rrb_alp");
        assert_eq!(rrb.get_domain_weight(&Domain::Mathematics), 0.35);
        assert_eq!(rrb.get_domain_weight(&Domain::Reasoning), 0.35);
        assert_eq!(rrb.get_domain_weight(&Domain::Physics), 0.20);
        assert_eq!(rrb.get_domain_weight(&Domain::Chemistry), 0.10);
        assert_eq!(
            rrb.get_topic_weight("schema.math.arithmetic.time_speed_distance"),
            1.5
        );

        let jee = ExamProfile::jee_main_foundation();
        assert_eq!(jee.get_domain_weight(&Domain::Physics), 0.35);
        assert_eq!(jee.get_domain_weight(&Domain::Reasoning), 0.0);
        assert_eq!(
            jee.get_target_latency_ms(
                &SchemaId::from("schema.physics.work_energy.mechanics"),
                &Domain::Physics
            ),
            90_000
        );
    }
}
