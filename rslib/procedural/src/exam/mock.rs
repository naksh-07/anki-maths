use std::collections::{HashMap, HashSet};

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::core::{Domain, ErrorEventId, ExamProfileId, Result, SchemaId, SkillId};
use crate::diagnostics::ErrorCategory;
use crate::exam::pyq::ContentProvenance;
use crate::exam::profile::ExamProfile;
use crate::practice::{
    DifficultyConstraint, ErrorEvent, PracticeAttempt, PracticeObjective, PracticeRequest,
    PracticeScope, RemediationPrecedence, TimeConstraint,
};
use crate::problems::ProblemInstance;
use crate::skills::domain_evidence::{
    ChemistryEvidence, MathEvidence, PhysicsEvidence, ReasoningEvidence, VersionedDomainEvidence,
};
use crate::skills::SkillState;
use crate::storage::ProceduralStore;

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

    /// Creates a balanced 4-domain diagnostic assessment blueprint across Math, Reasoning, Physics, Chemistry.
    pub fn diagnostic_default(total_questions: usize, time_limit_ms: u64) -> Self {
        Self::diagnostic_balanced("Comprehensive Multi-Domain Diagnostic Assessment", total_questions, time_limit_ms)
    }

    /// Creates a balanced diagnostic blueprint with custom title.
    pub fn diagnostic_balanced(title: impl Into<String>, total_questions: usize, time_limit_ms: u64) -> Self {
        let base_count = total_questions / 4;
        let remainder = total_questions % 4;

        let mut domain_dist = HashMap::new();
        domain_dist.insert(Domain::Mathematics, (base_count + if remainder > 0 { 1 } else { 0 }).max(1));
        domain_dist.insert(Domain::Reasoning, (base_count + if remainder > 1 { 1 } else { 0 }).max(1));
        domain_dist.insert(Domain::Physics, (base_count + if remainder > 2 { 1 } else { 0 }).max(1));
        domain_dist.insert(Domain::Chemistry, base_count.max(1));

        let mut diff_dist = HashMap::new();
        diff_dist.insert(2, 0.40); // 40% Standard
        diff_dist.insert(3, 0.40); // 40% Multi-step
        diff_dist.insert(4, 0.20); // 20% Advanced / Transfer

        Self {
            exam_profile_id: ExamProfileId::new("diagnostic-multi-domain"),
            title: title.into(),
            domain_distribution: domain_dist,
            difficulty_distribution: diff_dist,
            total_questions,
            time_limit_ms,
            positive_mark_per_question: 1.0,
            negative_mark_per_incorrect: 0.0, // Fixed measuring mode: no negative penalty by default
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

/// Diagnostic node in the Subject -> Chapter -> Topic -> Problem Family hierarchy.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiagnosticHierarchyNode {
    pub id: String,
    pub name: String,
    pub level: DiagnosticHierarchyLevel,
    pub total_questions: usize,
    pub answered_count: usize,
    pub correct_count: usize,
    pub accuracy: f64,
    pub mean_time_ms: f64,
    pub concept_errors: usize,
    pub calculation_errors: usize,
    pub transfer_errors: usize,
    pub speed_deficits: usize,
    pub children: Vec<DiagnosticHierarchyNode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticHierarchyLevel {
    Subject,
    Chapter,
    Topic,
    ProblemFamily,
}

/// 4-dimension diagnostic error distribution.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DiagnosticErrorDistribution {
    pub concept_count: usize,
    pub calculation_count: usize,
    pub transfer_count: usize,
    pub speed_deficit_count: usize,
}

/// Comprehensive Diagnostic Report returned upon finalization.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComprehensiveDiagnosticReport {
    pub session_id: String,
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
    pub hierarchy: Vec<DiagnosticHierarchyNode>,
    pub error_distribution: DiagnosticErrorDistribution,
    pub weak_skills: Vec<SkillId>,
    pub slow_skills: Vec<SkillId>,
    pub transfer_gaps: Vec<SkillId>,
    pub recommended_follow_up: PracticeRequest,
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

    /// Retrieve a question item by index.
    pub fn get_question(&self, index: usize) -> Option<&MockQuestionItem> {
        self.questions.get(index)
    }

    /// Retrieve all question items.
    pub fn get_all_questions(&self) -> &[MockQuestionItem] {
        &self.questions
    }

    /// Retrieve the active question item.
    pub fn get_current_question(&self) -> Option<&MockQuestionItem> {
        self.questions.get(self.current_question_index)
    }

    /// Check if a question at the given index has been answered.
    pub fn is_question_answered(&self, index: usize) -> bool {
        self.answers.get(&index).map_or(false, |a| !a.answer.trim().is_empty())
    }

    /// Check if a question at the given index is marked for review.
    pub fn is_question_marked(&self, index: usize) -> bool {
        self.marked_for_review.contains(&index)
    }

    /// Retrieve progress statistics: (answered_count, marked_count, total_count).
    pub fn progress_stats(&self) -> (usize, usize, usize) {
        let answered = self.answers.values().filter(|a| !a.answer.trim().is_empty()).count();
        let marked = self.marked_for_review.len();
        (answered, marked, self.questions.len())
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
    pub fn is_answer_correct(expected: &serde_json::Value, submitted: &str) -> bool {
        let submitted_clean = submitted.trim().to_lowercase();
        if submitted_clean.is_empty() {
            return false;
        }

        // 1. Direct string comparison
        if let Some(expected_str) = expected.as_str() {
            if expected_str.trim().to_lowercase() == submitted_clean {
                return true;
            }
        }

        // 2. Direct numeric comparison if expected is a number
        if let Some(expected_num) = expected.as_f64() {
            if let Ok(num) = submitted_clean.parse::<f64>() {
                let diff = (num - expected_num).abs();
                let tol = 0.01_f64.max(expected_num.abs() * 0.01);
                if diff <= tol {
                    return true;
                }
            }
        }

        // 3. Check common structured fields in expected answer JSON
        let numeric_keys = ["value", "ans", "result", "effective", "length_m", "answer_days", "numeric_value"];
        for key in &numeric_keys {
            if let Some(val) = expected.get(*key).and_then(|v| v.as_f64()) {
                // Try parsing submitted directly as float
                if let Ok(num) = submitted_clean.parse::<f64>() {
                    let diff = (num - val).abs();
                    let tol = 0.01_f64.max(val.abs() * 0.01);
                    if diff <= tol {
                        return true;
                    }
                }
                // Try extracting first number from string with units (e.g. "20 m/s" -> 20.0)
                let num_str: String = submitted_clean
                    .chars()
                    .take_while(|c| c.is_ascii_digit() || *c == '.' || *c == '-' || *c == '+')
                    .collect();
                if let Ok(num) = num_str.parse::<f64>() {
                    let diff = (num - val).abs();
                    let tol = 0.01_f64.max(val.abs() * 0.01);
                    if diff <= tol {
                        return true;
                    }
                }
            }
        }

        // 4. Check option and choice IDs for MCQ
        let option_keys = [
            "option_id",
            "correct_option_id",
            "selected_option_id",
            "canonical_option_id",
            "choice",
            "expected_option_id",
        ];
        for key in &option_keys {
            if let Some(opt) = expected.get(*key).and_then(|v| v.as_str()) {
                if opt.trim().to_lowercase() == submitted_clean {
                    return true;
                }
            }
        }

        // 5. Check string formatted comparison
        let string_keys = ["formatted", "answer", "text", "label", "formula_or_fact"];
        for key in &string_keys {
            if let Some(fmt) = expected.get(*key).and_then(|v| v.as_str()) {
                if fmt.trim().to_lowercase() == submitted_clean {
                    return true;
                }
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

    /// Generate a 4-tier hierarchical diagnostic report with 4-quadrant error analysis.
    pub fn generate_comprehensive_report(&mut self, timestamp_ms: i64) -> ComprehensiveDiagnosticReport {
        let scoring = if let Some(ref res) = self.scoring_result {
            res.clone()
        } else {
            self.submit(timestamp_ms)
        };

        let mut error_distribution = DiagnosticErrorDistribution::default();
        let mut weak_skills_set = HashSet::new();
        let mut slow_skills_set = HashSet::new();
        let mut transfer_gaps_set = HashSet::new();

        // 1. Group questions by Domain -> Chapter -> Topic (Skill) -> Problem Family (Schema)
        let mut domain_map: HashMap<Domain, HashMap<String, HashMap<SkillId, Vec<&MockQuestionItem>>>> = HashMap::new();

        for q in &self.questions {
            let chapter = q.instance.metadata.get("chapter")
                .and_then(|v| v.as_str())
                .unwrap_or("General")
                .to_string();

            domain_map
                .entry(q.domain.clone())
                .or_default()
                .entry(chapter)
                .or_default()
                .entry(q.skill_id.clone())
                .or_default()
                .push(q);
        }

        let mut hierarchy = Vec::new();

        for (domain, chapters) in domain_map {
            let mut dom_total = 0;
            let mut dom_answered = 0;
            let mut dom_correct = 0;
            let mut dom_time_ms = 0u64;
            let mut dom_concept = 0;
            let mut dom_calc = 0;
            let mut dom_transfer = 0;
            let mut dom_speed = 0;

            let mut chapter_nodes = Vec::new();

            for (chapter_name, skills) in chapters {
                let mut chap_total = 0;
                let mut chap_answered = 0;
                let mut chap_correct = 0;
                let mut chap_time_ms = 0u64;
                let mut chap_concept = 0;
                let mut chap_calc = 0;
                let mut chap_transfer = 0;
                let mut chap_speed = 0;

                let mut skill_nodes = Vec::new();

                for (skill_id, questions) in skills {
                    let mut skill_total = 0;
                    let mut skill_answered = 0;
                    let mut skill_correct = 0;
                    let mut skill_time_ms = 0u64;
                    let mut skill_concept = 0;
                    let mut skill_calc = 0;
                    let mut skill_transfer = 0;
                    let mut skill_speed = 0;

                    let mut family_map: HashMap<SchemaId, Vec<&MockQuestionItem>> = HashMap::new();
                    for q in &questions {
                        family_map.entry(q.schema_id.clone()).or_default().push(q);
                    }

                    let mut family_nodes = Vec::new();

                    for (schema_id, fam_questions) in family_map {
                        let mut fam_total = 0;
                        let mut fam_answered = 0;
                        let mut fam_correct = 0;
                        let mut fam_time_ms = 0u64;
                        let mut fam_concept = 0;
                        let mut fam_calc = 0;
                        let mut fam_transfer = 0;
                        let mut fam_speed = 0;

                        for q in &fam_questions {
                            fam_total += 1;
                            if let Some(ans) = self.answers.get(&q.question_index) {
                                if !ans.answer.trim().is_empty() {
                                    fam_answered += 1;
                                    fam_time_ms += ans.time_taken_ms;
                                    let is_corr = Self::is_answer_correct(&q.instance.correct_answer, &ans.answer);
                                    if is_corr {
                                        fam_correct += 1;
                                        if ans.time_taken_ms > (q.target_time_ms * 125 / 100) {
                                            fam_speed += 1;
                                            error_distribution.speed_deficit_count += 1;
                                        }
                                    } else {
                                        if q.difficulty_level >= 4 {
                                            fam_transfer += 1;
                                            error_distribution.transfer_count += 1;
                                            transfer_gaps_set.insert(q.skill_id.clone());
                                        } else if ans.answer.contains('-') && !q.instance.rendered_prompt.contains("negative") {
                                            fam_calc += 1;
                                            error_distribution.calculation_count += 1;
                                        } else {
                                            fam_concept += 1;
                                            error_distribution.concept_count += 1;
                                        }
                                    }
                                }
                            }
                        }

                        let fam_acc = if fam_answered > 0 { (fam_correct as f64 / fam_answered as f64) * 100.0 } else { 0.0 };
                        let fam_mean_t = if fam_answered > 0 { fam_time_ms as f64 / fam_answered as f64 } else { 0.0 };

                        family_nodes.push(DiagnosticHierarchyNode {
                            id: schema_id.to_string(),
                            name: schema_id.to_string(),
                            level: DiagnosticHierarchyLevel::ProblemFamily,
                            total_questions: fam_total,
                            answered_count: fam_answered,
                            correct_count: fam_correct,
                            accuracy: fam_acc,
                            mean_time_ms: fam_mean_t,
                            concept_errors: fam_concept,
                            calculation_errors: fam_calc,
                            transfer_errors: fam_transfer,
                            speed_deficits: fam_speed,
                            children: Vec::new(),
                        });

                        skill_total += fam_total;
                        skill_answered += fam_answered;
                        skill_correct += fam_correct;
                        skill_time_ms += fam_time_ms;
                        skill_concept += fam_concept;
                        skill_calc += fam_calc;
                        skill_transfer += fam_transfer;
                        skill_speed += fam_speed;
                    }

                    let skill_acc = if skill_answered > 0 { (skill_correct as f64 / skill_answered as f64) * 100.0 } else { 0.0 };
                    let skill_mean_t = if skill_answered > 0 { skill_time_ms as f64 / skill_answered as f64 } else { 0.0 };

                    if skill_answered > 0 && skill_acc < 60.0 {
                        weak_skills_set.insert(skill_id.clone());
                    }
                    if skill_answered > 0 && skill_mean_t > 45_000.0 {
                        slow_skills_set.insert(skill_id.clone());
                    }

                    skill_nodes.push(DiagnosticHierarchyNode {
                        id: skill_id.to_string(),
                        name: skill_id.to_string(),
                        level: DiagnosticHierarchyLevel::Topic,
                        total_questions: skill_total,
                        answered_count: skill_answered,
                        correct_count: skill_correct,
                        accuracy: skill_acc,
                        mean_time_ms: skill_mean_t,
                        concept_errors: skill_concept,
                        calculation_errors: skill_calc,
                        transfer_errors: skill_transfer,
                        speed_deficits: skill_speed,
                        children: family_nodes,
                    });

                    chap_total += skill_total;
                    chap_answered += skill_answered;
                    chap_correct += skill_correct;
                    chap_time_ms += skill_time_ms;
                    chap_concept += skill_concept;
                    chap_calc += skill_calc;
                    chap_transfer += skill_transfer;
                    chap_speed += skill_speed;
                }

                let chap_acc = if chap_answered > 0 { (chap_correct as f64 / chap_answered as f64) * 100.0 } else { 0.0 };
                let chap_mean_t = if chap_answered > 0 { chap_time_ms as f64 / chap_answered as f64 } else { 0.0 };

                chapter_nodes.push(DiagnosticHierarchyNode {
                    id: format!("{}:{}", domain.as_str(), chapter_name),
                    name: chapter_name,
                    level: DiagnosticHierarchyLevel::Chapter,
                    total_questions: chap_total,
                    answered_count: chap_answered,
                    correct_count: chap_correct,
                    accuracy: chap_acc,
                    mean_time_ms: chap_mean_t,
                    concept_errors: chap_concept,
                    calculation_errors: chap_calc,
                    transfer_errors: chap_transfer,
                    speed_deficits: chap_speed,
                    children: skill_nodes,
                });

                dom_total += chap_total;
                dom_answered += chap_answered;
                dom_correct += chap_correct;
                dom_time_ms += chap_time_ms;
                dom_concept += chap_concept;
                dom_calc += chap_calc;
                dom_transfer += chap_transfer;
                dom_speed += chap_speed;
            }

            let dom_acc = if dom_answered > 0 { (dom_correct as f64 / dom_answered as f64) * 100.0 } else { 0.0 };
            let dom_mean_t = if dom_answered > 0 { dom_time_ms as f64 / dom_answered as f64 } else { 0.0 };

            hierarchy.push(DiagnosticHierarchyNode {
                id: domain.as_str().to_string(),
                name: domain.as_str().to_string(),
                level: DiagnosticHierarchyLevel::Subject,
                total_questions: dom_total,
                answered_count: dom_answered,
                correct_count: dom_correct,
                accuracy: dom_acc,
                mean_time_ms: dom_mean_t,
                concept_errors: dom_concept,
                calculation_errors: dom_calc,
                transfer_errors: dom_transfer,
                speed_deficits: dom_speed,
                children: chapter_nodes,
            });
        }

        let recommended_follow_up = MockFollowUpEngine::generate_follow_up_request(&scoring);

        ComprehensiveDiagnosticReport {
            session_id: self.session_id.clone(),
            exam_profile_id: self.blueprint.exam_profile_id.clone(),
            total_questions: scoring.total_questions,
            answered_count: scoring.answered_count,
            unanswered_count: scoring.unanswered_count,
            correct_count: scoring.correct_count,
            incorrect_count: scoring.incorrect_count,
            raw_score: scoring.raw_score,
            max_score: scoring.max_score,
            percentage: scoring.percentage,
            accuracy: scoring.accuracy,
            total_time_spent_ms: scoring.total_time_spent_ms,
            hierarchy,
            error_distribution,
            weak_skills: weak_skills_set.into_iter().collect(),
            slow_skills: slow_skills_set.into_iter().collect(),
            transfer_gaps: transfer_gaps_set.into_iter().collect(),
            recommended_follow_up,
        }
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

/// Batch-updates existing `SkillState` and `DomainEvidence` structures in `procedural.db`
/// based on the finalized diagnostic mock session results without duplicate parallel state models.
pub fn apply_diagnostic_report_to_store(
    store: &ProceduralStore,
    session: &MockSession,
    _report: &ComprehensiveDiagnosticReport,
) -> Result<Vec<SkillState>> {
    let mut updated_states = Vec::new();

    for q in &session.questions {
        if let Some(ans) = session.answers.get(&q.question_index) {
            let submitted_clean = ans.answer.trim();
            if submitted_clean.is_empty() {
                continue;
            }

            let is_correct = MockSession::is_answer_correct(&q.instance.correct_answer, submitted_clean);
            let time_taken_ms = ans.time_taken_ms;
            let target_time_ms = q.target_time_ms;

            // 1. Determine error category and diagnostic flag
            let (error_cat, is_speed_deficit) = if !is_correct {
                if q.difficulty_level >= 4 {
                    (Some(ErrorCategory::DomainSpecific("transfer".to_string())), false)
                } else if submitted_clean.contains('-') && !q.instance.rendered_prompt.contains("negative") {
                    (Some(ErrorCategory::Calculation), false)
                } else {
                    (Some(ErrorCategory::Concept), false)
                }
            } else if time_taken_ms > (target_time_ms * 125 / 100) {
                (Some(ErrorCategory::Time), true)
            } else {
                (None, false)
            };

            let is_calc_err = error_cat == Some(ErrorCategory::Calculation);
            let is_trans_err = matches!(error_cat, Some(ErrorCategory::DomainSpecific(ref s)) if s == "transfer");

            // 2. Synthesize domain-specific diagnostic evidence
            let domain_evidence = match &q.domain {
                Domain::Mathematics => {
                    VersionedDomainEvidence::new_math(MathEvidence {
                        pattern_recognition: Some(is_correct),
                        method_selection: Some(is_correct),
                        execution: Some(is_correct || !is_calc_err),
                        verification: Some(is_correct),
                        structural_transfer: Some(is_correct || !is_trans_err),
                    })
                }
                Domain::Reasoning => {
                    VersionedDomainEvidence::new_reasoning(ReasoningEvidence {
                        pattern_recognition: Some(is_correct),
                        representation: Some(is_correct),
                        constraint_extraction: Some(is_correct),
                        decision_path: Some(is_correct),
                        deduction: Some(is_correct),
                        trap_checking: Some(is_correct),
                        structural_transfer: Some(is_correct || !is_trans_err),
                    })
                }
                Domain::Physics => {
                    VersionedDomainEvidence::new_physics(PhysicsEvidence {
                        physical_model_selection: Some(is_correct),
                        representation: Some(is_correct),
                        governing_principle: Some(is_correct),
                        equation_setup: Some(is_correct),
                        calculation: Some(is_correct || !is_calc_err),
                        unit_validity: Some(is_correct),
                        boundary_validity: Some(is_correct),
                        verification: Some(is_correct),
                        transfer: Some(is_correct || !is_trans_err),
                    })
                }
                Domain::Chemistry => {
                    VersionedDomainEvidence::new_chemistry(ChemistryEvidence::Physical {
                        model_setup: Some(is_correct),
                        equation_selection: Some(is_correct),
                        intermediate_quantity: Some(is_correct),
                        calculation: Some(is_correct || !is_calc_err),
                        conservation: Some(is_correct),
                        verification: Some(is_correct),
                        transfer: Some(is_correct || !is_trans_err),
                    })
                }
                _ => {
                    VersionedDomainEvidence::new_math(MathEvidence {
                        pattern_recognition: Some(is_correct),
                        method_selection: Some(is_correct),
                        execution: Some(is_correct || !is_calc_err),
                        verification: Some(is_correct),
                        structural_transfer: Some(is_correct || !is_trans_err),
                    })
                }
            };

            // 3. Ensure ProblemInstance is persisted to satisfy foreign keys
            let _ = store.insert_problem_instance(&q.instance);

            // 4. Construct typed PracticeAttempt
            let attempt_id = format!("diag_att_{}_{}", session.session_id, q.question_index);
            let mut attempt = PracticeAttempt::new(
                attempt_id.clone(),
                &q.instance.id,
                q.schema_id.clone(),
                q.skill_id.clone(),
                serde_json::json!({ "submitted": ans.answer }),
                is_correct,
                if is_correct { 1.0 } else { 0.0 },
                time_taken_ms,
            );

            attempt.metadata["is_diagnostic"] = serde_json::json!(true);
            attempt.metadata["session_id"] = serde_json::json!(session.session_id);
            attempt.metadata["target_latency_ms"] = serde_json::json!(target_time_ms);
            attempt.metadata["difficulty_level"] = serde_json::json!(q.difficulty_level);
            attempt.metadata["domain_evidence"] = serde_json::to_value(&domain_evidence).unwrap_or_default();
            if let Some(ref cat) = error_cat {
                attempt.metadata["error_category"] = serde_json::to_value(cat).unwrap_or_default();
            }

            // 5. Construct ErrorEvent if failed or speed deficit
            let mut error_events = Vec::new();
            if let Some(ref cat) = error_cat {
                if !is_correct || is_speed_deficit {
                    let err_id = ErrorEventId::new(format!("err_diag_{}_{}", session.session_id, q.question_index));
                    let attempt_id_typed = crate::core::AttemptId::new(attempt_id);
                    error_events.push(ErrorEvent::new(
                        err_id,
                        &attempt_id_typed,
                        cat.as_str(),
                        serde_json::json!({
                            "schema_id": q.schema_id,
                            "skill_id": q.skill_id,
                            "difficulty": q.difficulty_level,
                            "time_taken_ms": time_taken_ms,
                        }),
                    ));
                }
            }

            // 6. Record attempt into store atomically (updates SkillState in skill_states table)
            let updated_state = store.record_practice_attempt_atomic(
                &attempt,
                &error_events,
                None,
                target_time_ms,
            )?;

            updated_states.push(updated_state);
        }
    }

    Ok(updated_states)
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

    #[test]
    fn test_mock_session_comprehensive_diagnostic_hierarchy() {
        let profile = ExamProfile::new(
            ExamProfileId::new("exam-cat"),
            "CAT Prep",
            "Management Entrance",
            vec![Domain::Mathematics, Domain::Reasoning],
            ExamObjective::ComprehensiveMock,
        );

        let blueprint = MockBlueprint::from_exam_profile(&profile, 4, 300_000);
        let schema = MathsCatalog::successive_percentage_schema();

        let mut questions = Vec::new();
        for i in 0..4 {
            let mut inst = PercentageSuccessiveGenerator::generate_instance(
                &schema.problem_family_id,
                200 + i as u64,
                &PercentageSuccessiveConfig::default(),
            );
            inst.metadata["chapter"] = serde_json::json!("Arithmetic");

            questions.push(MockQuestionItem {
                question_index: i,
                schema_id: schema.id.clone(),
                skill_id: schema.skill_id.clone(),
                domain: if i < 2 { Domain::Mathematics } else { Domain::Reasoning },
                schema_title: schema.title.clone(),
                instance: inst,
                difficulty_level: if i == 3 { 4 } else { 2 },
                target_time_ms: 30_000,
                is_pyq: false,
                provenance: None,
            });
        }

        let mut mock = MockSession::new("mock-diag-001", blueprint, questions);

        // Q0: Correct, fast (15s)
        let ans0_val = mock.questions[0].instance.correct_answer["formatted"].as_str().unwrap_or("25%").to_string();
        mock.record_answer(0, ans0_val, 15_000);

        // Q1: Incorrect, conceptual slip
        mock.record_answer(1, "999.0", 25_000);

        // Q2: Correct, speed deficit (45s > 30s * 1.25)
        let ans2_val = mock.questions[2].instance.correct_answer["formatted"].as_str().unwrap_or("25%").to_string();
        mock.record_answer(2, ans2_val, 45_000);

        // Q3: Incorrect, transfer failure (level 4)
        mock.record_answer(3, "0.0", 30_000);

        let report = mock.generate_comprehensive_report(Utc::now().timestamp_millis());

        assert_eq!(report.total_questions, 4);
        assert_eq!(report.answered_count, 4);
        assert_eq!(report.correct_count, 2);
        assert_eq!(report.incorrect_count, 2);

        // Check error distribution: 1 speed deficit, 1 concept error, 1 transfer error
        assert_eq!(report.error_distribution.speed_deficit_count, 1);
        assert_eq!(report.error_distribution.concept_count, 1);
        assert_eq!(report.error_distribution.transfer_count, 1);

        // Check 4-tier hierarchy
        assert!(!report.hierarchy.is_empty());
        let math_node = report.hierarchy.iter().find(|n| n.name == "mathematics");
        assert!(math_node.is_some());
        let math_node = math_node.unwrap();
        assert_eq!(math_node.total_questions, 2);
        assert_eq!(math_node.correct_count, 1);
        assert_eq!(math_node.children[0].name, "Arithmetic");
        assert_eq!(math_node.children[0].children[0].level, DiagnosticHierarchyLevel::Topic);
    }

    #[test]
    fn test_diagnostic_evidence_store_sync_and_domain_evidence_updates() {
        let store = ProceduralStore::open_in_memory().unwrap();
        MathsCatalog::init_all(&store).unwrap();

        let blueprint = MockBlueprint::diagnostic_default(4, 300_000);
        let schema = MathsCatalog::successive_percentage_schema();

        let mut questions = Vec::new();
        for i in 0..4 {
            let mut inst = PercentageSuccessiveGenerator::generate_instance(
                &schema.problem_family_id,
                300 + i as u64,
                &PercentageSuccessiveConfig::default(),
            );
            inst.metadata["chapter"] = serde_json::json!("Arithmetic");

            questions.push(MockQuestionItem {
                question_index: i,
                schema_id: schema.id.clone(),
                skill_id: schema.skill_id.clone(),
                domain: Domain::Mathematics,
                schema_title: schema.title.clone(),
                instance: inst,
                difficulty_level: 2,
                target_time_ms: 30_000,
                is_pyq: false,
                provenance: None,
            });
        }

        let mut mock = MockSession::new("mock-sync-001", blueprint, questions);

        // Answer Q0 correctly and fast
        let ans0 = mock.questions[0].instance.correct_answer["formatted"].as_str().unwrap().to_string();
        mock.record_answer(0, ans0, 20_000);

        // Answer Q1 with sign error
        mock.record_answer(1, "-28%", 25_000);

        let report = mock.generate_comprehensive_report(Utc::now().timestamp_millis());
        let updated_states = apply_diagnostic_report_to_store(&store, &mock, &report).unwrap();

        assert_eq!(updated_states.len(), 2);
        let state = store.get_skill_state(&schema.skill_id).unwrap().unwrap();
        assert_eq!(state.total_attempts, 2);
        assert_eq!(state.successful_attempts, 1);
        assert!(state.mastery > 0.0);
    }
}
