// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::collections::{HashMap, HashSet};

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::core::{Domain, ExamProfileId, SchemaId, SkillId};
use crate::exam::pyq::ContentProvenance;
use crate::exam::profile::ExamProfile;
use crate::practice::{
    DifficultyConstraint, PracticeObjective, PracticeRequest, PracticeScope,
    RemediationPrecedence, TimeConstraint,
};
use crate::problems::ProblemInstance;

/// Single immutable item within a Mock question set.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MockQuestionItem {
    pub question_index: usize,
    pub schema_id: SchemaId,
    pub skill_id: SkillId,
    pub domain: Domain,
    pub schema_title: String,
    pub instance: ProblemInstance,
    pub difficulty_level: u32,
    pub target_time_ms: u64,
    pub is_pyq: bool,
    pub provenance: Option<ContentProvenance>,
}

/// Specifications and constraints for generating an authentic exam simulation mock.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MockBlueprint {
    pub exam_profile_id: ExamProfileId,
    pub title: String,
    pub domain_distribution: HashMap<Domain, usize>,
    pub difficulty_distribution: HashMap<u32, f64>,
    pub total_questions: usize,
    pub time_limit_ms: u64,
    pub positive_mark_per_question: f64,
    pub negative_mark_per_incorrect: f64,
}

impl MockBlueprint {
    /// Creates a standard balanced blueprint from an ExamProfile and question count.
    pub fn from_exam_profile(profile: &ExamProfile, total_questions: usize, time_limit_ms: u64) -> Self {
        let mut domain_dist = HashMap::new();
        let num_domains = profile.subjects.len().max(1);

        for domain in &profile.subjects {
            let weight = profile.domain_weights.get(domain).copied().unwrap_or(1.0 / num_domains as f64);
            let count = (total_questions as f64 * weight).round() as usize;
            domain_dist.insert(domain.clone(), count.max(1));
        }

        // Adjust sum to match total_questions exactly
        let current_sum: usize = domain_dist.values().sum();
        if current_sum != total_questions && !profile.subjects.is_empty() {
            let first_domain = profile.subjects[0].clone();
            if let Some(c) = domain_dist.get_mut(&first_domain) {
                if current_sum < total_questions {
                    *c += total_questions - current_sum;
                } else if *c > (current_sum - total_questions) {
                    *c -= current_sum - total_questions;
                }
            }
        }

        Self {
            exam_profile_id: profile.id.clone(),
            title: format!("{} Full Mock Simulation", profile.name),
            domain_distribution: domain_dist,
            difficulty_distribution: profile.difficulty_distribution.clone(),
            total_questions,
            time_limit_ms,
            positive_mark_per_question: 1.0,
            negative_mark_per_incorrect: -0.25,
        }
    }
}

/// Recorded answer submission for an individual mock question.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MockAnswerSubmission {
    pub question_index: usize,
    pub answer: String,
    pub time_taken_ms: u64,
    pub is_marked_for_review: bool,
    pub submitted_at: i64,
}

/// Domain-level performance breakdown in a completed mock.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DomainMockMetric {
    pub domain: Domain,
    pub total: usize,
    pub answered: usize,
    pub correct: usize,
    pub accuracy: f64,
    pub mean_time_ms: f64,
    pub score: f64,
}

/// Schema-level performance breakdown in a completed mock.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SchemaMockMetric {
    pub schema_id: SchemaId,
    pub domain: Domain,
    pub total: usize,
    pub correct: usize,
    pub accuracy: f64,
    pub mean_time_ms: f64,
}

/// Actionable scoring and diagnostic report generated upon mock finalization.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MockScoringResult {
    pub mock_id: String,
    pub exam_profile_id: ExamProfileId,
    pub total_questions: usize,
    pub answered_count: usize,
    pub unanswered_count: usize,
    pub correct_count: usize,
    pub incorrect_count: usize,
    pub raw_score: f64,
    pub max_score: f64,
    pub percentage: f64,
    pub accuracy: f64,
    pub total_time_spent_ms: u64,
    pub domain_performance: HashMap<Domain, DomainMockMetric>,
    pub schema_performance: HashMap<SchemaId, SchemaMockMetric>,
    pub weak_schemas: Vec<SchemaId>,
    pub slow_schemas: Vec<SchemaId>,
    pub pyq_failures: Vec<String>,
    pub transfer_failures: Vec<String>,
}

/// Active bounded mock session encapsulating immutable questions, navigation state,
/// and independent grading isolated from standard FSRS card scheduling.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MockSession {
    pub session_id: String,
    pub blueprint: MockBlueprint,
    pub questions: Vec<MockQuestionItem>,
    pub answers: HashMap<usize, MockAnswerSubmission>,
    pub marked_for_review: HashSet<usize>,
    pub current_question_index: usize,
    pub start_time_ms: i64,
    pub end_time_ms: Option<i64>,
    pub is_submitted: bool,
    pub scoring_result: Option<MockScoringResult>,
}

impl MockSession {
    /// Initialize a new mock session with an immutable question set.
    pub fn new(session_id: impl Into<String>, blueprint: MockBlueprint, questions: Vec<MockQuestionItem>) -> Self {
        Self {
            session_id: session_id.into(),
            blueprint,
            questions,
            answers: HashMap::new(),
            marked_for_review: HashSet::new(),
            current_question_index: 0,
            start_time_ms: Utc::now().timestamp_millis(),
            end_time_ms: None,
            is_submitted: false,
            scoring_result: None,
        }
    }

    /// Record a learner's answer for a question.
    pub fn record_answer(&mut self, question_index: usize, answer: impl Into<String>, time_taken_ms: u64) {
        if self.is_submitted || question_index >= self.questions.len() {
            return;
        }

        let is_marked = self.marked_for_review.contains(&question_index);
        self.answers.insert(
            question_index,
            MockAnswerSubmission {
                question_index,
                answer: answer.into(),
                time_taken_ms,
                is_marked_for_review: is_marked,
                submitted_at: Utc::now().timestamp_millis(),
            },
        );
    }

    /// Toggle question bookmark / mark for review.
    pub fn toggle_mark_for_review(&mut self, question_index: usize) -> bool {
        if self.is_submitted || question_index >= self.questions.len() {
            return false;
        }

        if self.marked_for_review.contains(&question_index) {
            self.marked_for_review.remove(&question_index);
            if let Some(ans) = self.answers.get_mut(&question_index) {
                ans.is_marked_for_review = false;
            }
            false
        } else {
            self.marked_for_review.insert(question_index);
            if let Some(ans) = self.answers.get_mut(&question_index) {
                ans.is_marked_for_review = true;
            }
            true
        }
    }

    /// Navigate to a specific question in the mock.
    pub fn navigate_to(&mut self, question_index: usize) -> bool {
        if question_index < self.questions.len() {
            self.current_question_index = question_index;
            true
        } else {
            false
        }
    }

    /// Evaluate correctness of an answer text against the question's expected answer.
    fn is_answer_correct(expected: &serde_json::Value, submitted: &str) -> bool {
        let submitted_clean = submitted.trim().to_lowercase();
        if submitted_clean.is_empty() {
            return false;
        }

        // Check numeric comparison if value is present
        if let Some(expected_val) = expected.get("value").and_then(|v| v.as_f64()) {
            if let Ok(num) = submitted_clean.parse::<f64>() {
                let diff = (num - expected_val).abs();
                let tol = 0.01_f64.max(expected_val.abs() * 0.01);
                return diff <= tol;
            }
        }

        // Check string formatted comparison
        if let Some(fmt) = expected.get("formatted").and_then(|v| v.as_str()) {
            if fmt.trim().to_lowercase() == submitted_clean {
                return true;
            }
        }

        // Check direct key match
        if let Some(key) = expected.get("answer").and_then(|v| v.as_str()) {
            if key.trim().to_lowercase() == submitted_clean {
                return true;
            }
        }

        false
    }

    /// Finalize and score the entire mock simulation session.
    pub fn submit(&mut self, timestamp_ms: i64) -> MockScoringResult {
        self.is_submitted = true;
        self.end_time_ms = Some(timestamp_ms);

        let total_questions = self.questions.len();
        let mut correct_count = 0;
        let mut incorrect_count = 0;
        let mut total_time_ms = 0u64;

        let mut domain_stats: HashMap<Domain, (usize, usize, usize, u64)> = HashMap::new(); // (total, answered, correct, time)
        let mut schema_stats: HashMap<SchemaId, (Domain, usize, usize, u64)> = HashMap::new(); // (domain, total, correct, time)
        let mut pyq_failures = Vec::new();
        let mut transfer_failures = Vec::new();

        for q in &self.questions {
            let d_entry = domain_stats.entry(q.domain.clone()).or_insert((0, 0, 0, 0));
            d_entry.0 += 1;

            let s_entry = schema_stats.entry(q.schema_id.clone()).or_insert((q.domain.clone(), 0, 0, 0));
            s_entry.1 += 1;

            if let Some(ans) = self.answers.get(&q.question_index) {
                if !ans.answer.trim().is_empty() {
                    d_entry.1 += 1;
                    d_entry.3 += ans.time_taken_ms;
                    s_entry.3 += ans.time_taken_ms;
                    total_time_ms += ans.time_taken_ms;

                    let is_correct = Self::is_answer_correct(&q.instance.correct_answer, &ans.answer);
                    if is_correct {
                        correct_count += 1;
                        d_entry.2 += 1;
                        s_entry.2 += 1;
                    } else {
                        incorrect_count += 1;
                        if q.is_pyq {
                            pyq_failures.push(format!("{}: {}", q.schema_id, q.instance.rendered_prompt));
                        }
                        if q.difficulty_level >= 4 {
                            transfer_failures.push(format!("{}: {}", q.schema_id, q.instance.rendered_prompt));
                        }
                    }
                }
            }
        }

        let answered_count = self.answers.values().filter(|a| !a.answer.trim().is_empty()).count();
        let unanswered_count = total_questions.saturating_sub(answered_count);

        let pos_mark = self.blueprint.positive_mark_per_question;
        let neg_mark = self.blueprint.negative_mark_per_incorrect;
        let raw_score = (correct_count as f64 * pos_mark) + (incorrect_count as f64 * neg_mark);
        let max_score = total_questions as f64 * pos_mark;
        let percentage = if max_score > 0.0 { (raw_score / max_score) * 100.0 } else { 0.0 };
        let accuracy = if answered_count > 0 { (correct_count as f64 / answered_count as f64) * 100.0 } else { 0.0 };

        let mut domain_performance = HashMap::new();
        for (domain, (tot, ans, corr, t_ms)) in domain_stats {
            let acc = if ans > 0 { (corr as f64 / ans as f64) * 100.0 } else { 0.0 };
            let mean_t = if ans > 0 { t_ms as f64 / ans as f64 } else { 0.0 };
            let dom_score = (corr as f64 * pos_mark) + ((ans - corr) as f64 * neg_mark);
            domain_performance.insert(
                domain.clone(),
                DomainMockMetric {
                    domain,
                    total: tot,
                    answered: ans,
                    correct: corr,
                    accuracy: acc,
                    mean_time_ms: mean_t,
                    score: dom_score,
                },
            );
        }

        let mut schema_performance = HashMap::new();
        let mut weak_schemas = Vec::new();
        let mut slow_schemas = Vec::new();

        for (schema_id, (domain, tot, corr, t_ms)) in schema_stats {
            let acc = if tot > 0 { (corr as f64 / tot as f64) * 100.0 } else { 0.0 };
            let mean_t = if tot > 0 { t_ms as f64 / tot as f64 } else { 0.0 };

            if acc < 60.0 {
                weak_schemas.push(schema_id.clone());
            } else if mean_t > 45_000.0 {
                slow_schemas.push(schema_id.clone());
            }

            schema_performance.insert(
                schema_id.clone(),
                SchemaMockMetric {
                    schema_id,
                    domain,
                    total: tot,
                    correct: corr,
                    accuracy: acc,
                    mean_time_ms: mean_t,
                },
            );
        }

        let result = MockScoringResult {
            mock_id: self.session_id.clone(),
            exam_profile_id: self.blueprint.exam_profile_id.clone(),
            total_questions,
            answered_count,
            unanswered_count,
            correct_count,
            incorrect_count,
            raw_score,
            max_score,
            percentage,
            accuracy,
            total_time_spent_ms: total_time_ms,
            domain_performance,
            schema_performance,
            weak_schemas,
            slow_schemas,
            pyq_failures,
            transfer_failures,
        };

        self.scoring_result = Some(result.clone());
        result
    }
}

/// Follow-up recommendation engine synthesizing targeted PracticeRequests from mock diagnostics.
pub struct MockFollowUpEngine;

impl MockFollowUpEngine {
    /// Constructs a follow-up PracticeRequest addressing weaknesses diagnosed during a mock.
    pub fn generate_follow_up_request(scoring: &MockScoringResult) -> PracticeRequest {
        if !scoring.weak_schemas.is_empty() {
            // Target weak schemas with remediation enabled
            PracticeRequest::new(
                PracticeScope::MultipleSchemas(scoring.weak_schemas.clone()),
                PracticeObjective::Practice,
            )
            .with_remediation_policy(RemediationPrecedence::AllEligible)
            .with_difficulty_constraint(DifficultyConstraint::Range { min: 1, max: 3 })
        } else if !scoring.slow_schemas.is_empty() {
            // Target speed practice on familiar schemas
            PracticeRequest::new(
                PracticeScope::MultipleSchemas(scoring.slow_schemas.clone()),
                PracticeObjective::Speed,
            )
            .with_time_constraint(TimeConstraint::new().with_target_latency_ms(30_000))
            .with_difficulty_constraint(DifficultyConstraint::Exact { level: 2 })
        } else if !scoring.transfer_failures.is_empty() {
            // Target transfer variation
            PracticeRequest::new(
                PracticeScope::AllDomains,
                PracticeObjective::Transfer,
            )
            .with_difficulty_constraint(DifficultyConstraint::Range { min: 3, max: 5 })
        } else {
            // General balanced preparation
            PracticeRequest::new(
                PracticeScope::AllDomains,
                PracticeObjective::Practice,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::ExamProfileId;
    use crate::exam::profile::ExamObjective;
    use crate::problems::catalog::MathsCatalog;
    use crate::problems::generators::{PercentageSuccessiveConfig, PercentageSuccessiveGenerator};

    #[test]
    fn test_mock_session_lifecycle_scoring_and_negative_marking() {
        let profile = ExamProfile::new(
            ExamProfileId::new("exam-jee"),
            "JEE Main",
            "Engineering Entrance Exam",
            vec![Domain::Mathematics, Domain::Physics],
            ExamObjective::ComprehensiveMock,
        );

        let blueprint = MockBlueprint::from_exam_profile(&profile, 4, 300_000);
        let schema = MathsCatalog::successive_percentage_schema();

        let mut questions = Vec::new();
        for i in 0..4 {
            let inst = PercentageSuccessiveGenerator::generate_instance(
                &schema.problem_family_id,
                100 + i as u64,
                &PercentageSuccessiveConfig::default(),
            );
            questions.push(MockQuestionItem {
                question_index: i,
                schema_id: schema.id.clone(),
                skill_id: schema.skill_id.clone(),
                domain: Domain::Mathematics,
                schema_title: schema.title.clone(),
                instance: inst,
                difficulty_level: 2,
                target_time_ms: 45_000,
                is_pyq: i % 2 == 0,
                provenance: None,
            });
        }

        let mut mock = MockSession::new("mock-001", blueprint, questions);
        assert_eq!(mock.questions.len(), 4);
        assert_eq!(mock.current_question_index, 0);

        // Bookmark question 1
        let marked = mock.toggle_mark_for_review(1);
        assert!(marked);
        assert!(mock.marked_for_review.contains(&1));

        // Answer question 0 correctly
        let ans0_val = mock.questions[0].instance.correct_answer["formatted"].as_str().unwrap_or("25%").to_string();
        mock.record_answer(0, ans0_val, 20_000);

        // Answer question 1 incorrectly
        mock.record_answer(1, "999.99", 35_000);

        // Question 2 and 3 remain unanswered

        // Submit mock
        let scoring = mock.submit(Utc::now().timestamp_millis());

        assert_eq!(scoring.total_questions, 4);
        assert_eq!(scoring.answered_count, 2);
        assert_eq!(scoring.unanswered_count, 2);
        assert_eq!(scoring.correct_count, 1);
        assert_eq!(scoring.incorrect_count, 1);

        // 1 correct (+1.0) - 1 incorrect (-0.25) = 0.75
        assert!((scoring.raw_score - 0.75).abs() < 1e-6);
        assert!((scoring.max_score - 4.0).abs() < 1e-6);
        assert!((scoring.percentage - 18.75).abs() < 1e-6);
        assert!((scoring.accuracy - 50.0).abs() < 1e-6);

        // Generate follow-up
        let follow_up = MockFollowUpEngine::generate_follow_up_request(&scoring);
        assert_eq!(follow_up.objective, PracticeObjective::Practice);
        match follow_up.scope {
            PracticeScope::MultipleSchemas(schemas) => {
                assert!(schemas.contains(&schema.id));
            }
            _ => panic!("Expected MultipleSchemas scope for weak schema remediation"),
        }
    }
}
