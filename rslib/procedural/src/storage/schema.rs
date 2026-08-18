// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

pub struct Migration {
    pub version: u32,
    pub description: &'static str,
    pub sql: &'static str,
}

pub const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        description: "Initial procedural database schema",
        sql: r#"
CREATE TABLE IF NOT EXISTS skills (
    id TEXT PRIMARY KEY,
    domain TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    prerequisites TEXT NOT NULL,
    metadata TEXT NOT NULL,
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS skill_states (
    skill_id TEXT PRIMARY KEY,
    mastery REAL NOT NULL,
    confidence REAL NOT NULL,
    total_attempts INTEGER NOT NULL,
    successful_attempts INTEGER NOT NULL,
    last_practiced_at INTEGER,
    custom_state TEXT NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY(skill_id) REFERENCES skills(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS problem_families (
    id TEXT PRIMARY KEY,
    skill_id TEXT NOT NULL,
    domain TEXT NOT NULL,
    name TEXT NOT NULL,
    template_ref TEXT NOT NULL,
    min_difficulty REAL NOT NULL,
    max_difficulty REAL NOT NULL,
    parameters_schema TEXT NOT NULL,
    metadata TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    FOREIGN KEY(skill_id) REFERENCES skills(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS schemas (
    id TEXT PRIMARY KEY,
    skill_id TEXT NOT NULL,
    problem_family_id TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    target_mastery REAL NOT NULL,
    config TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    FOREIGN KEY(skill_id) REFERENCES skills(id) ON DELETE CASCADE,
    FOREIGN KEY(problem_family_id) REFERENCES problem_families(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS problem_instances (
    id TEXT PRIMARY KEY,
    family_id TEXT NOT NULL,
    seed INTEGER NOT NULL,
    parameters TEXT NOT NULL,
    rendered_prompt TEXT NOT NULL,
    correct_answer TEXT NOT NULL,
    metadata TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    FOREIGN KEY(family_id) REFERENCES problem_families(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS practice_attempts (
    id TEXT PRIMARY KEY,
    instance_id TEXT NOT NULL,
    schema_id TEXT NOT NULL,
    skill_id TEXT NOT NULL,
    card_id INTEGER,
    user_answer TEXT NOT NULL,
    is_correct INTEGER NOT NULL,
    score REAL NOT NULL,
    time_taken_ms INTEGER NOT NULL,
    attempted_at INTEGER NOT NULL,
    metadata TEXT NOT NULL,
    FOREIGN KEY(instance_id) REFERENCES problem_instances(id) ON DELETE CASCADE,
    FOREIGN KEY(schema_id) REFERENCES schemas(id) ON DELETE CASCADE,
    FOREIGN KEY(skill_id) REFERENCES skills(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS error_events (
    id TEXT PRIMARY KEY,
    attempt_id TEXT NOT NULL,
    error_category TEXT NOT NULL,
    details TEXT NOT NULL,
    occurred_at INTEGER NOT NULL,
    FOREIGN KEY(attempt_id) REFERENCES practice_attempts(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_skills_domain ON skills(domain);
CREATE INDEX IF NOT EXISTS idx_families_skill ON problem_families(skill_id);
CREATE INDEX IF NOT EXISTS idx_schemas_skill ON schemas(skill_id);
CREATE INDEX IF NOT EXISTS idx_instances_family ON problem_instances(family_id);
CREATE INDEX IF NOT EXISTS idx_attempts_schema ON practice_attempts(schema_id);
CREATE INDEX IF NOT EXISTS idx_attempts_skill ON practice_attempts(skill_id);
CREATE INDEX IF NOT EXISTS idx_attempts_card ON practice_attempts(card_id);
CREATE INDEX IF NOT EXISTS idx_error_events_attempt ON error_events(attempt_id);
"#,
    },
    Migration {
        version: 2,
        description: "Maths Engine v1 catalog tracking and query optimization indexes",
        sql: r#"
CREATE TABLE IF NOT EXISTS catalog_metadata (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_attempts_time ON practice_attempts(attempted_at);
CREATE INDEX IF NOT EXISTS idx_skill_states_updated ON skill_states(updated_at);
"#,
    },
    Migration {
        version: 3,
        description: "Exam Content & Personalization Engine schema (PYQ sources, mappings, variants, and profiles)",
        sql: r#"
CREATE TABLE IF NOT EXISTS pyq_sources (
    id TEXT PRIMARY KEY,
    exam TEXT NOT NULL,
    year INTEGER NOT NULL,
    paper TEXT,
    shift TEXT,
    session TEXT,
    domain TEXT NOT NULL,
    original_question TEXT NOT NULL,
    original_options TEXT,
    original_answer TEXT NOT NULL,
    source_reference TEXT NOT NULL,
    provenance TEXT NOT NULL,
    source_version INTEGER NOT NULL,
    import_timestamp INTEGER NOT NULL,
    metadata TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS pyq_mappings (
    pyq_id TEXT PRIMARY KEY,
    domain TEXT NOT NULL,
    skill_id TEXT NOT NULL,
    schema_id TEXT NOT NULL,
    problem_family_id TEXT NOT NULL,
    variant_structure TEXT,
    difficulty_level INTEGER NOT NULL,
    target_latency_ms INTEGER NOT NULL,
    diagnostic_metadata TEXT NOT NULL,
    status TEXT NOT NULL,
    confidence TEXT NOT NULL,
    reviewer_notes TEXT,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY(pyq_id) REFERENCES pyq_sources(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS rejected_variants (
    id TEXT PRIMARY KEY,
    source_pyq_id TEXT,
    schema_id TEXT NOT NULL,
    family_id TEXT NOT NULL,
    seed INTEGER NOT NULL,
    variant_type TEXT NOT NULL,
    failure_reason TEXT NOT NULL,
    generated_instance_json TEXT NOT NULL,
    rejected_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS exam_profiles (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    subjects TEXT NOT NULL,
    domain_weights TEXT NOT NULL,
    topic_weights TEXT NOT NULL,
    preferred_formats TEXT NOT NULL,
    target_latencies_ms TEXT NOT NULL,
    difficulty_distribution TEXT NOT NULL,
    pyq_weight REAL NOT NULL,
    objective TEXT NOT NULL,
    metadata TEXT NOT NULL,
    created_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_pyq_sources_exam ON pyq_sources(exam);
CREATE INDEX IF NOT EXISTS idx_pyq_sources_domain ON pyq_sources(domain);
CREATE INDEX IF NOT EXISTS idx_pyq_mappings_schema ON pyq_mappings(schema_id);
CREATE INDEX IF NOT EXISTS idx_pyq_mappings_status ON pyq_mappings(status);
CREATE INDEX IF NOT EXISTS idx_pyq_mappings_confidence ON pyq_mappings(confidence);
CREATE INDEX IF NOT EXISTS idx_rejected_variants_pyq ON rejected_variants(source_pyq_id);
CREATE INDEX IF NOT EXISTS idx_rejected_variants_family ON rejected_variants(family_id);
"#,
    },
];
