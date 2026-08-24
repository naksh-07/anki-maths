// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

export type Domain = "mathematics" | "reasoning" | "physics" | "chemistry" | string;

export interface ProblemInstance {
    id: string;
    family_id: string;
    seed: number;
    parameters: Record<string, any>;
    rendered_prompt: string;
    correct_answer: any;
    explanation_markdown?: string;
    solution_graph?: any;
    metadata: Record<string, any>;
}

export interface MockQuestionItem {
    question_index: number;
    schema_id: string;
    skill_id: string;
    domain: Domain;
    schema_title: string;
    instance: ProblemInstance;
    difficulty_level: number;
    target_time_ms: number;
    is_pyq: boolean;
    provenance?: any;
}

export interface MockAnswerSubmission {
    question_index: number;
    answer: string;
    time_taken_ms: number;
    timestamp_ms: number;
}

export interface MockBlueprint {
    exam_profile_id: string;
    title: string;
    domain_distribution: Record<string, number>;
    difficulty_distribution: Record<string, number>;
    total_questions: number;
    time_limit_ms: number;
    positive_mark_per_question: number;
    negative_mark_per_incorrect: number;
}

export interface MockScoringResult {
    mock_id: string;
    exam_profile_id: string;
    total_questions: number;
    answered_count: number;
    unanswered_count: number;
    correct_count: number;
    incorrect_count: number;
    raw_score: number;
    max_score: number;
    percentage: number;
    accuracy: number;
    total_time_spent_ms: number;
    domain_performance: Record<string, any>;
    schema_performance: Record<string, any>;
    weak_schemas: string[];
    slow_schemas: string[];
    pyq_failures: string[];
    transfer_failures: string[];
}

export type DiagnosticHierarchyLevel = "Subject" | "Chapter" | "Topic" | "ProblemFamily";

export interface DiagnosticHierarchyNode {
    id: string;
    name: string;
    level: DiagnosticHierarchyLevel;
    total_questions: number;
    answered_count: number;
    correct_count: number;
    accuracy: number;
    mean_time_ms: number;
    concept_errors: number;
    calculation_errors: number;
    transfer_errors: number;
    speed_deficits: number;
    children: DiagnosticHierarchyNode[];
}

export interface DiagnosticErrorDistribution {
    concept_count: number;
    calculation_count: number;
    transfer_count: number;
    speed_deficit_count: number;
}

export interface ComprehensiveDiagnosticReport {
    session_id: string;
    exam_profile_id: string;
    total_questions: number;
    answered_count: number;
    unanswered_count: number;
    correct_count: number;
    incorrect_count: number;
    raw_score: number;
    max_score: number;
    percentage: number;
    accuracy: number;
    total_time_spent_ms: number;
    hierarchy: DiagnosticHierarchyNode[];
    error_distribution: DiagnosticErrorDistribution;
    weak_skills: string[];
    slow_skills: string[];
    transfer_gaps: string[];
    recommended_follow_up: any;
}

export interface MockSession {
    session_id: string;
    blueprint: MockBlueprint;
    questions: MockQuestionItem[];
    answers: Record<number, MockAnswerSubmission>;
    marked_for_review: number[];
    start_time_ms?: number;
    end_time_ms?: number;
    is_submitted: boolean;
    scoring_result?: MockScoringResult;
}
