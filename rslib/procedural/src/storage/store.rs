// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

use rusqlite::{params, Connection, OptionalExtension};

use super::migration::MigrationRunner;
use crate::core::{
    AttemptId, Domain, ErrorEventId, ExamProfileId, PracticeItemId, ProblemFamilyId,
    ProblemInstanceId, PyqId, RejectedVariantId, Result, SchemaId, SkillId,
};
use crate::exam::{
    ContentProvenance, ExamObjective, ExamProfile, MappingConfidence, MappingStatus, PYQSource,
    PyqMapping, RejectedVariantRecord,
};
use crate::practice::{ErrorEvent, PracticeAttempt, SchemaPracticeObject};
use crate::problems::{ProblemFamily, ProblemInstance};
use crate::skills::{Skill, SkillState};
use crate::content::{ChapterPracticeProfile, PracticeItem};
use crate::remediation::{RemediationAction, RemediationActionKind, RemediationQueue, RemediationUrgency};
use crate::diagnostics::ErrorCategory;
use crate::problems::steps::StepErrorType;

#[derive(Clone)]
pub struct ProceduralStore {
    conn: Arc<Mutex<Connection>>,
}

impl ProceduralStore {
    fn apply_pragmas(conn: &Connection) -> Result<()> {
        conn.execute_batch(
            r#"
            PRAGMA busy_timeout = 5000;
            PRAGMA foreign_keys = ON;
            PRAGMA synchronous = NORMAL;
            PRAGMA temp_store = MEMORY;
            "#,
        )?;
        let _ = conn.query_row("PRAGMA journal_mode = WAL;", [], |_| Ok(()));
        Ok(())
    }

    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let mut conn = Connection::open(path)?;
        Self::apply_pragmas(&conn)?;
        MigrationRunner::run(&mut conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn open_in_memory() -> Result<Self> {
        let mut conn = Connection::open_in_memory()?;
        Self::apply_pragmas(&conn)?;
        MigrationRunner::run(&mut conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn insert_skill(&self, skill: &Skill) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let prereqs_json = serde_json::to_string(&skill.prerequisites)?;
        let metadata_json = serde_json::to_string(&skill.metadata)?;

        conn.execute(
            r#"
            INSERT INTO skills (id, domain, name, description, prerequisites, metadata, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(id) DO UPDATE SET
                domain = excluded.domain,
                name = excluded.name,
                description = excluded.description,
                prerequisites = excluded.prerequisites,
                metadata = excluded.metadata;
            "#,
            params![
                skill.id.as_str(),
                skill.domain.as_str(),
                skill.name,
                skill.description,
                prereqs_json,
                metadata_json,
                skill.created_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_skill(&self, id: &SkillId) -> Result<Option<Skill>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, domain, name, description, prerequisites, metadata, created_at FROM skills WHERE id = ?1",
        )?;

        let skill = stmt
            .query_row(params![id.as_str()], |row| {
                let id_str: String = row.get(0)?;
                let domain_str: String = row.get(1)?;
                let name: String = row.get(2)?;
                let description: String = row.get(3)?;
                let prereqs_str: String = row.get(4)?;
                let metadata_str: String = row.get(5)?;
                let created_at: i64 = row.get(6)?;

                let domain: Domain = domain_str.parse().unwrap_or(Domain::Custom(domain_str));
                let prerequisites: Vec<SkillId> = serde_json::from_str(&prereqs_str).unwrap_or_default();
                let metadata: serde_json::Value = serde_json::from_str(&metadata_str).unwrap_or_default();

                Ok(Skill {
                    id: SkillId::new(id_str),
                    domain,
                    name,
                    description,
                    prerequisites,
                    metadata,
                    created_at,
                })
            })
            .optional()?;

        Ok(skill)
    }

    pub fn list_skills_by_domain(&self, domain: &Domain) -> Result<Vec<Skill>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, domain, name, description, prerequisites, metadata, created_at FROM skills WHERE domain = ?1",
        )?;

        let rows = stmt.query_map(params![domain.as_str()], |row| {
            let id_str: String = row.get(0)?;
            let domain_str: String = row.get(1)?;
            let name: String = row.get(2)?;
            let description: String = row.get(3)?;
            let prereqs_str: String = row.get(4)?;
            let metadata_str: String = row.get(5)?;
            let created_at: i64 = row.get(6)?;

            let domain: Domain = domain_str.parse().unwrap_or(Domain::Custom(domain_str));
            let prerequisites: Vec<SkillId> = serde_json::from_str(&prereqs_str).unwrap_or_default();
            let metadata: serde_json::Value = serde_json::from_str(&metadata_str).unwrap_or_default();

            Ok(Skill {
                id: SkillId::new(id_str),
                domain,
                name,
                description,
                prerequisites,
                metadata,
                created_at,
            })
        })?;

        let mut skills = Vec::new();
        for s in rows {
            skills.push(s?);
        }
        Ok(skills)
    }

    pub fn list_all_skills(&self) -> Result<Vec<Skill>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, domain, name, description, prerequisites, metadata, created_at FROM skills ORDER BY id ASC",
        )?;

        let rows = stmt.query_map([], |row| {
            let id_str: String = row.get(0)?;
            let domain_str: String = row.get(1)?;
            let name: String = row.get(2)?;
            let description: String = row.get(3)?;
            let prereqs_str: String = row.get(4)?;
            let metadata_str: String = row.get(5)?;
            let created_at: i64 = row.get(6)?;

            let domain: Domain = domain_str.parse().unwrap_or(Domain::Custom(domain_str));
            let prerequisites: Vec<SkillId> = serde_json::from_str(&prereqs_str).unwrap_or_default();
            let metadata: serde_json::Value = serde_json::from_str(&metadata_str).unwrap_or_default();

            Ok(Skill {
                id: SkillId::new(id_str),
                domain,
                name,
                description,
                prerequisites,
                metadata,
                created_at,
            })
        })?;

        let mut skills = Vec::new();
        for s in rows {
            skills.push(s?);
        }
        Ok(skills)
    }

    pub fn list_all_schemas(&self) -> Result<Vec<SchemaPracticeObject>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, skill_id, problem_family_id, title, description, target_mastery, config, created_at FROM schemas ORDER BY id ASC",
        )?;

        let rows = stmt.query_map([], |row| {
            let id_str: String = row.get(0)?;
            let skill_id_str: String = row.get(1)?;
            let family_id_str: String = row.get(2)?;
            let title: String = row.get(3)?;
            let description: String = row.get(4)?;
            let target_mastery: f64 = row.get(5)?;
            let config_str: String = row.get(6)?;
            let created_at: i64 = row.get(7)?;

            let config: serde_json::Value = serde_json::from_str(&config_str).unwrap_or_default();

            Ok(SchemaPracticeObject {
                id: SchemaId::new(id_str),
                skill_id: SkillId::new(skill_id_str),
                problem_family_id: ProblemFamilyId::new(family_id_str),
                title,
                description,
                target_mastery,
                config,
                created_at,
            })
        })?;

        let mut schemas = Vec::new();
        for s in rows {
            schemas.push(s?);
        }
        Ok(schemas)
    }

    pub fn set_catalog_metadata(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"
            INSERT INTO catalog_metadata (key, value, updated_at)
            VALUES (?1, ?2, ?3)
            ON CONFLICT(key) DO UPDATE SET
                value = excluded.value,
                updated_at = excluded.updated_at;
            "#,
            params![key, value, chrono::Utc::now().timestamp()],
        )?;
        Ok(())
    }

    pub fn get_catalog_metadata(&self, key: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT value FROM catalog_metadata WHERE key = ?1")?;
        let res = stmt.query_row(params![key], |row| row.get(0)).optional()?;
        Ok(res)
    }

    pub fn upsert_skill_state(&self, state: &SkillState) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let mut state_clone = state.clone();
        state_clone.sync_custom_state();
        let custom_state_json = serde_json::to_string(&state_clone.custom_state)?;

        conn.execute(
            r#"
            INSERT INTO skill_states (
                skill_id, mastery, confidence, total_attempts,
                successful_attempts, last_practiced_at, custom_state, updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT(skill_id) DO UPDATE SET
                mastery = excluded.mastery,
                confidence = excluded.confidence,
                total_attempts = excluded.total_attempts,
                successful_attempts = excluded.successful_attempts,
                last_practiced_at = excluded.last_practiced_at,
                custom_state = excluded.custom_state,
                updated_at = excluded.updated_at;
            "#,
            params![
                state.skill_id.as_str(),
                state.mastery,
                state.confidence,
                state.total_attempts,
                state.successful_attempts,
                state.last_practiced_at,
                custom_state_json,
                state.updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_skill_state(&self, skill_id: &SkillId) -> Result<Option<SkillState>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT skill_id, mastery, confidence, total_attempts,
                   successful_attempts, last_practiced_at, custom_state, updated_at
            FROM skill_states WHERE skill_id = ?1
            "#,
        )?;

        let state = stmt
            .query_row(params![skill_id.as_str()], |row| {
                let id_str: String = row.get(0)?;
                let mastery: f64 = row.get(1)?;
                let confidence: f64 = row.get(2)?;
                let total_attempts: u32 = row.get(3)?;
                let successful_attempts: u32 = row.get(4)?;
                let last_practiced_at: Option<i64> = row.get(5)?;
                let custom_str: String = row.get(6)?;
                let updated_at: i64 = row.get(7)?;

                let custom_state: serde_json::Value =
                    serde_json::from_str(&custom_str).unwrap_or_default();

                let mut state = SkillState::new(SkillId::new(id_str));
                state.mastery = mastery;
                state.confidence = confidence;
                state.total_attempts = total_attempts;
                state.successful_attempts = successful_attempts;
                state.last_practiced_at = last_practiced_at;
                state.custom_state = custom_state;
                state.updated_at = updated_at;
                state.restore_from_custom_state();

                Ok(state)
            })
            .optional()?;

        Ok(state)
    }

    pub fn get_skill_states(&self, skill_ids: &[SkillId]) -> Result<HashMap<SkillId, SkillState>> {
        if skill_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let conn = self.conn.lock().unwrap();
        let placeholders: Vec<String> = (1..=skill_ids.len()).map(|i| format!("?{}", i)).collect();
        let sql = format!(
            r#"
            SELECT skill_id, mastery, confidence, total_attempts,
                   successful_attempts, last_practiced_at, custom_state, updated_at
            FROM skill_states WHERE skill_id IN ({})
            "#,
            placeholders.join(", ")
        );

        let mut stmt = conn.prepare(&sql)?;
        let params: Vec<&str> = skill_ids.iter().map(|s| s.as_str()).collect();
        let rows = stmt.query_map(rusqlite::params_from_iter(params), |row| {
            let id_str: String = row.get(0)?;
            let mastery: f64 = row.get(1)?;
            let confidence: f64 = row.get(2)?;
            let total_attempts: u32 = row.get(3)?;
            let successful_attempts: u32 = row.get(4)?;
            let last_practiced_at: Option<i64> = row.get(5)?;
            let custom_str: String = row.get(6)?;
            let updated_at: i64 = row.get(7)?;

            let custom_state: serde_json::Value =
                serde_json::from_str(&custom_str).unwrap_or_default();

            let mut state = SkillState::new(SkillId::new(id_str.clone()));
            state.mastery = mastery;
            state.confidence = confidence;
            state.total_attempts = total_attempts;
            state.successful_attempts = successful_attempts;
            state.last_practiced_at = last_practiced_at;
            state.custom_state = custom_state;
            state.updated_at = updated_at;
            state.restore_from_custom_state();

            Ok((SkillId::new(id_str), state))
        })?;

        let mut states = HashMap::new();
        for r in rows {
            let (id, s) = r?;
            states.insert(id, s);
        }
        Ok(states)
    }

    pub fn get_all_skill_states(&self) -> Result<HashMap<SkillId, SkillState>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT skill_id, mastery, confidence, total_attempts,
                   successful_attempts, last_practiced_at, custom_state, updated_at
            FROM skill_states
            "#,
        )?;

        let rows = stmt.query_map([], |row| {
            let id_str: String = row.get(0)?;
            let mastery: f64 = row.get(1)?;
            let confidence: f64 = row.get(2)?;
            let total_attempts: u32 = row.get(3)?;
            let successful_attempts: u32 = row.get(4)?;
            let last_practiced_at: Option<i64> = row.get(5)?;
            let custom_str: String = row.get(6)?;
            let updated_at: i64 = row.get(7)?;

            let custom_state: serde_json::Value =
                serde_json::from_str(&custom_str).unwrap_or_default();

            let mut state = SkillState::new(SkillId::new(id_str.clone()));
            state.mastery = mastery;
            state.confidence = confidence;
            state.total_attempts = total_attempts;
            state.successful_attempts = successful_attempts;
            state.last_practiced_at = last_practiced_at;
            state.custom_state = custom_state;
            state.updated_at = updated_at;
            state.restore_from_custom_state();

            Ok((SkillId::new(id_str), state))
        })?;

        let mut states = HashMap::new();
        for r in rows {
            let (id, s) = r?;
            states.insert(id, s);
        }
        Ok(states)
    }

    pub fn insert_problem_family(&self, family: &ProblemFamily) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let schema_json = serde_json::to_string(&family.parameters_schema)?;
        let metadata_json = serde_json::to_string(&family.metadata)?;

        conn.execute(
            r#"
            INSERT INTO problem_families (
                id, skill_id, domain, name, template_ref,
                min_difficulty, max_difficulty, parameters_schema, metadata, created_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            ON CONFLICT(id) DO UPDATE SET
                skill_id = excluded.skill_id,
                domain = excluded.domain,
                name = excluded.name,
                template_ref = excluded.template_ref,
                min_difficulty = excluded.min_difficulty,
                max_difficulty = excluded.max_difficulty,
                parameters_schema = excluded.parameters_schema,
                metadata = excluded.metadata;
            "#,
            params![
                family.id.as_str(),
                family.skill_id.as_str(),
                family.domain.as_str(),
                family.name,
                family.template_ref,
                family.min_difficulty,
                family.max_difficulty,
                schema_json,
                metadata_json,
                family.created_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_problem_family(&self, id: &ProblemFamilyId) -> Result<Option<ProblemFamily>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT id, skill_id, domain, name, template_ref,
                   min_difficulty, max_difficulty, parameters_schema, metadata, created_at
            FROM problem_families WHERE id = ?1
            "#,
        )?;

        let family = stmt
            .query_row(params![id.as_str()], |row| {
                let id_str: String = row.get(0)?;
                let skill_id_str: String = row.get(1)?;
                let domain_str: String = row.get(2)?;
                let name: String = row.get(3)?;
                let template_ref: String = row.get(4)?;
                let min_difficulty: f64 = row.get(5)?;
                let max_difficulty: f64 = row.get(6)?;
                let schema_str: String = row.get(7)?;
                let metadata_str: String = row.get(8)?;
                let created_at: i64 = row.get(9)?;

                let domain: Domain = domain_str.parse().unwrap_or(Domain::Custom(domain_str));
                let parameters_schema: serde_json::Value =
                    serde_json::from_str(&schema_str).unwrap_or_default();
                let metadata: serde_json::Value =
                    serde_json::from_str(&metadata_str).unwrap_or_default();

                Ok(ProblemFamily {
                    id: ProblemFamilyId::new(id_str),
                    skill_id: SkillId::new(skill_id_str),
                    domain,
                    name,
                    template_ref,
                    min_difficulty,
                    max_difficulty,
                    parameters_schema,
                    metadata,
                    created_at,
                })
            })
            .optional()?;

        Ok(family)
    }

    pub fn insert_schema(&self, schema: &SchemaPracticeObject) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let config_json = serde_json::to_string(&schema.config)?;

        conn.execute(
            r#"
            INSERT INTO schemas (
                id, skill_id, problem_family_id, title, description,
                target_mastery, config, created_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT(id) DO UPDATE SET
                skill_id = excluded.skill_id,
                problem_family_id = excluded.problem_family_id,
                title = excluded.title,
                description = excluded.description,
                target_mastery = excluded.target_mastery,
                config = excluded.config;
            "#,
            params![
                schema.id.as_str(),
                schema.skill_id.as_str(),
                schema.problem_family_id.as_str(),
                schema.title,
                schema.description,
                schema.target_mastery,
                config_json,
                schema.created_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_schema(&self, id: &SchemaId) -> Result<Option<SchemaPracticeObject>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT id, skill_id, problem_family_id, title, description,
                   target_mastery, config, created_at
            FROM schemas WHERE id = ?1
            "#,
        )?;

        let schema = stmt
            .query_row(params![id.as_str()], |row| {
                let id_str: String = row.get(0)?;
                let skill_id_str: String = row.get(1)?;
                let family_id_str: String = row.get(2)?;
                let title: String = row.get(3)?;
                let description: String = row.get(4)?;
                let target_mastery: f64 = row.get(5)?;
                let config_str: String = row.get(6)?;
                let created_at: i64 = row.get(7)?;

                let config: serde_json::Value =
                    serde_json::from_str(&config_str).unwrap_or_default();

                Ok(SchemaPracticeObject {
                    id: SchemaId::new(id_str),
                    skill_id: SkillId::new(skill_id_str),
                    problem_family_id: ProblemFamilyId::new(family_id_str),
                    title,
                    description,
                    target_mastery,
                    config,
                    created_at,
                })
            })
            .optional()?;

        Ok(schema)
    }

    pub fn get_schema_by_family(&self, family_id: &ProblemFamilyId) -> Result<Option<SchemaPracticeObject>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT id, skill_id, problem_family_id, title, description,
                   target_mastery, config, created_at
            FROM schemas WHERE problem_family_id = ?1 LIMIT 1
            "#,
        )?;

        let schema = stmt
            .query_row(params![family_id.as_str()], |row| {
                let id_str: String = row.get(0)?;
                let skill_id_str: String = row.get(1)?;
                let family_id_str: String = row.get(2)?;
                let title: String = row.get(3)?;
                let description: String = row.get(4)?;
                let target_mastery: f64 = row.get(5)?;
                let config_str: String = row.get(6)?;
                let created_at: i64 = row.get(7)?;

                let config: serde_json::Value =
                    serde_json::from_str(&config_str).unwrap_or_default();

                Ok(SchemaPracticeObject {
                    id: SchemaId::new(id_str),
                    skill_id: SkillId::new(skill_id_str),
                    problem_family_id: ProblemFamilyId::new(family_id_str),
                    title,
                    description,
                    target_mastery,
                    config,
                    created_at,
                })
            })
            .optional()?;

        Ok(schema)
    }

    pub fn insert_problem_instance(&self, instance: &ProblemInstance) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let params_json = serde_json::to_string(&instance.parameters)?;
        let ans_json = serde_json::to_string(&instance.correct_answer)?;
        let metadata_json = serde_json::to_string(&instance.metadata)?;

        conn.execute(
            r#"
            INSERT INTO problem_instances (
                id, family_id, seed, parameters, rendered_prompt,
                correct_answer, metadata, created_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT(id) DO UPDATE SET
                family_id = excluded.family_id,
                seed = excluded.seed,
                parameters = excluded.parameters,
                rendered_prompt = excluded.rendered_prompt,
                correct_answer = excluded.correct_answer,
                metadata = excluded.metadata;
            "#,
            params![
                instance.id.as_str(),
                instance.family_id.as_str(),
                instance.seed as i64,
                params_json,
                instance.rendered_prompt,
                ans_json,
                metadata_json,
                instance.created_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_problem_instance(&self, id: &ProblemInstanceId) -> Result<Option<ProblemInstance>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT id, family_id, seed, parameters, rendered_prompt,
                   correct_answer, metadata, created_at
            FROM problem_instances WHERE id = ?1
            "#,
        )?;

        let instance = stmt
            .query_row(params![id.as_str()], |row| {
                let id_str: String = row.get(0)?;
                let family_id_str: String = row.get(1)?;
                let seed_i64: i64 = row.get(2)?;
                let params_str: String = row.get(3)?;
                let rendered_prompt: String = row.get(4)?;
                let ans_str: String = row.get(5)?;
                let metadata_str: String = row.get(6)?;
                let created_at: i64 = row.get(7)?;

                let parameters: serde_json::Value =
                    serde_json::from_str(&params_str).unwrap_or_default();
                let correct_answer: serde_json::Value =
                    serde_json::from_str(&ans_str).unwrap_or_default();
                let metadata: serde_json::Value =
                    serde_json::from_str(&metadata_str).unwrap_or_default();

                Ok(ProblemInstance {
                    id: ProblemInstanceId::new(id_str),
                    family_id: ProblemFamilyId::new(family_id_str),
                    seed: seed_i64 as u64,
                    parameters,
                    rendered_prompt,
                    correct_answer,
                    metadata,
                    created_at,
                })
            })
            .optional()?;

        Ok(instance)
    }

    pub fn insert_practice_attempt(&self, attempt: &PracticeAttempt) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let user_ans_json = serde_json::to_string(&attempt.user_answer)?;
        let metadata_json = serde_json::to_string(&attempt.metadata)?;

        conn.execute(
            r#"
            INSERT INTO practice_attempts (
                id, instance_id, schema_id, skill_id, card_id,
                user_answer, is_correct, score, time_taken_ms, attempted_at, metadata
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11);
            "#,
            params![
                attempt.id.as_str(),
                attempt.instance_id.as_str(),
                attempt.schema_id.as_str(),
                attempt.skill_id.as_str(),
                attempt.card_id,
                user_ans_json,
                if attempt.is_correct { 1 } else { 0 },
                attempt.score,
                attempt.time_taken_ms as i64,
                attempt.attempted_at,
                metadata_json,
            ],
        )?;
        Ok(())
    }

    /// Atomically reads existing SkillState, applies attempt outcome, inserts practice attempt,
    /// inserts error events, and updates SkillState in a single SQLite transaction boundary.
    pub fn record_practice_attempt_atomic(
        &self,
        attempt: &PracticeAttempt,
        errors: &[ErrorEvent],
        variant: Option<&str>,
        target_latency_ms: u64,
    ) -> Result<SkillState> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;

        // 1. Transactional read of existing skill state
        let mut state = {
            let mut stmt = tx.prepare(
                r#"
                SELECT skill_id, mastery, confidence, total_attempts,
                       successful_attempts, last_practiced_at, custom_state, updated_at
                FROM skill_states WHERE skill_id = ?1
                "#,
            )?;

            let state_opt = stmt
                .query_row(params![attempt.skill_id.as_str()], |row| {
                    let id_str: String = row.get(0)?;
                    let mastery: f64 = row.get(1)?;
                    let confidence: f64 = row.get(2)?;
                    let total_attempts: u32 = row.get(3)?;
                    let successful_attempts: u32 = row.get(4)?;
                    let last_practiced_at: Option<i64> = row.get(5)?;
                    let custom_str: String = row.get(6)?;
                    let updated_at: i64 = row.get(7)?;

                    let custom_state: serde_json::Value =
                        serde_json::from_str(&custom_str).unwrap_or_default();

                    let mut s = SkillState::new(SkillId::new(id_str));
                    s.mastery = mastery;
                    s.confidence = confidence;
                    s.total_attempts = total_attempts;
                    s.successful_attempts = successful_attempts;
                    s.last_practiced_at = last_practiced_at;
                    s.custom_state = custom_state;
                    s.updated_at = updated_at;
                    s.restore_from_custom_state();

                    Ok(s)
                })
                .optional()?;

            state_opt.unwrap_or_else(|| SkillState::new(attempt.skill_id.clone()))
        };

        // 2. Build MasteryEvidence including domain_evidence
        let err_cat = attempt
            .metadata
            .get("error_category")
            .and_then(|v| serde_json::from_value(v.clone()).ok());

        let mut diagnostic_errors = Vec::new();
        if let Some(cat) = err_cat {
            diagnostic_errors.push(cat);
        }
        for err in errors {
            let cat_parsed = match err.error_category.as_str() {
                "concept" | "conceptual" => ErrorCategory::Concept,
                "strategy" => ErrorCategory::Strategy,
                "calculation" => ErrorCategory::Calculation,
                "careless" => ErrorCategory::Careless,
                "time" => ErrorCategory::Time,
                "unknown" => ErrorCategory::Unknown,
                other => ErrorCategory::DomainSpecific(other.to_string()),
            };
            if !diagnostic_errors.contains(&cat_parsed) {
                diagnostic_errors.push(cat_parsed);
            }
        }

        let hints_used = attempt
            .metadata
            .get("hints_used")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        let attempt_count = attempt
            .metadata
            .get("attempt_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(1) as u32;

        let independence = if hints_used == 0 && attempt_count <= 1 {
            crate::skills::IndependenceLevel::Independent
        } else if hints_used <= 1 && attempt_count <= 1 {
            crate::skills::IndependenceLevel::LightSupport
        } else if hints_used <= 2 || attempt_count <= 2 {
            crate::skills::IndependenceLevel::SignificantSupport
        } else {
            crate::skills::IndependenceLevel::NonIndependent
        };

        let variant_category = attempt
            .metadata
            .get("variant_category")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();

        let domain_evidence = attempt
            .metadata
            .get("domain_evidence")
            .and_then(|v| serde_json::from_value(v.clone()).ok());

        let evidence = crate::skills::signals::MasteryEvidence {
            final_correctness: attempt.is_correct,
            latency_evidence: attempt.time_taken_ms,
            independence,
            hint_dependence: hints_used,
            retry_dependence: attempt_count.saturating_sub(1),
            variant_exposure: variant.map(|s| s.to_string()),
            variant_category,
            diagnostic_errors,
            domain_evidence,
            ..Default::default()
        };

        // 3. Update in-memory state
        state.record_attempt_outcome(
            &evidence,
            attempt.score,
            target_latency_ms,
            attempt.attempted_at,
        );

        // 4. Insert practice attempt
        let user_ans_json = serde_json::to_string(&attempt.user_answer)?;
        let metadata_json = serde_json::to_string(&attempt.metadata)?;

        tx.execute(
            r#"
            INSERT INTO practice_attempts (
                id, instance_id, schema_id, skill_id, card_id,
                user_answer, is_correct, score, time_taken_ms, attempted_at, metadata
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11);
            "#,
            params![
                attempt.id.as_str(),
                attempt.instance_id.as_str(),
                attempt.schema_id.as_str(),
                attempt.skill_id.as_str(),
                attempt.card_id,
                user_ans_json,
                if attempt.is_correct { 1 } else { 0 },
                attempt.score,
                attempt.time_taken_ms as i64,
                attempt.attempted_at,
                metadata_json,
            ],
        )?;

        // 5. Insert error events
        for error in errors {
            let details_json = serde_json::to_string(&error.details)?;
            tx.execute(
                r#"
                INSERT INTO error_events (id, attempt_id, error_category, details, occurred_at)
                VALUES (?1, ?2, ?3, ?4, ?5);
                "#,
                params![
                    error.id.as_str(),
                    error.attempt_id.as_str(),
                    error.error_category,
                    details_json,
                    error.occurred_at,
                ],
            )?;
        }

        // 6. Upsert updated skill state
        state.sync_custom_state();
        let custom_state_json = serde_json::to_string(&state.custom_state)?;

        tx.execute(
            r#"
            INSERT INTO skill_states (
                skill_id, mastery, confidence, total_attempts,
                successful_attempts, last_practiced_at, custom_state, updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT(skill_id) DO UPDATE SET
                mastery = excluded.mastery,
                confidence = excluded.confidence,
                total_attempts = excluded.total_attempts,
                successful_attempts = excluded.successful_attempts,
                last_practiced_at = excluded.last_practiced_at,
                custom_state = excluded.custom_state,
                updated_at = excluded.updated_at;
            "#,
            params![
                state.skill_id.as_str(),
                state.mastery,
                state.confidence,
                state.total_attempts,
                state.successful_attempts,
                state.last_practiced_at,
                custom_state_json,
                state.updated_at,
            ],
        )?;

        tx.commit()?;
        Ok(state)
    }

    /// Atomically records a practice attempt, associated error events, and updated skill state.
    /// Uses a single SQLite transaction so any failure causes a complete rollback.
    pub fn record_attempt_atomic(
        &self,
        attempt: &PracticeAttempt,
        errors: &[ErrorEvent],
        state: &SkillState,
    ) -> Result<()> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;

        let user_ans_json = serde_json::to_string(&attempt.user_answer)?;
        let metadata_json = serde_json::to_string(&attempt.metadata)?;

        tx.execute(
            r#"
            INSERT INTO practice_attempts (
                id, instance_id, schema_id, skill_id, card_id,
                user_answer, is_correct, score, time_taken_ms, attempted_at, metadata
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11);
            "#,
            params![
                attempt.id.as_str(),
                attempt.instance_id.as_str(),
                attempt.schema_id.as_str(),
                attempt.skill_id.as_str(),
                attempt.card_id,
                user_ans_json,
                if attempt.is_correct { 1 } else { 0 },
                attempt.score,
                attempt.time_taken_ms as i64,
                attempt.attempted_at,
                metadata_json,
            ],
        )?;

        for error in errors {
            let details_json = serde_json::to_string(&error.details)?;
            tx.execute(
                r#"
                INSERT INTO error_events (id, attempt_id, error_category, details, occurred_at)
                VALUES (?1, ?2, ?3, ?4, ?5);
                "#,
                params![
                    error.id.as_str(),
                    error.attempt_id.as_str(),
                    error.error_category,
                    details_json,
                    error.occurred_at,
                ],
            )?;
        }

        let mut state_clone = state.clone();
        state_clone.sync_custom_state();
        let custom_state_json = serde_json::to_string(&state_clone.custom_state)?;

        tx.execute(
            r#"
            INSERT INTO skill_states (
                skill_id, mastery, confidence, total_attempts,
                successful_attempts, last_practiced_at, custom_state, updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT(skill_id) DO UPDATE SET
                mastery = excluded.mastery,
                confidence = excluded.confidence,
                total_attempts = excluded.total_attempts,
                successful_attempts = excluded.successful_attempts,
                last_practiced_at = excluded.last_practiced_at,
                custom_state = excluded.custom_state,
                updated_at = excluded.updated_at;
            "#,
            params![
                state.skill_id.as_str(),
                state.mastery,
                state.confidence,
                state.total_attempts,
                state.successful_attempts,
                state.last_practiced_at,
                custom_state_json,
                state.updated_at,
            ],
        )?;

        tx.commit()?;
        Ok(())
    }

    pub fn get_practice_attempts_by_schema(
        &self,
        schema_id: &SchemaId,
        limit: usize,
    ) -> Result<Vec<PracticeAttempt>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT id, instance_id, schema_id, skill_id, card_id,
                   user_answer, is_correct, score, time_taken_ms, attempted_at, metadata
            FROM practice_attempts
            WHERE schema_id = ?1
            ORDER BY attempted_at DESC, rowid DESC
            LIMIT ?2
            "#,
        )?;

        let rows = stmt.query_map(params![schema_id.as_str(), limit as i64], |row| {
            let id_str: String = row.get(0)?;
            let inst_str: String = row.get(1)?;
            let sch_str: String = row.get(2)?;
            let sk_str: String = row.get(3)?;
            let card_id: Option<i64> = row.get(4)?;
            let ans_str: String = row.get(5)?;
            let is_correct_int: i32 = row.get(6)?;
            let score: f64 = row.get(7)?;
            let time_ms: i64 = row.get(8)?;
            let attempted_at: i64 = row.get(9)?;
            let meta_str: String = row.get(10)?;

            let user_answer: serde_json::Value =
                serde_json::from_str(&ans_str).unwrap_or_default();
            let metadata: serde_json::Value =
                serde_json::from_str(&meta_str).unwrap_or_default();

            Ok(PracticeAttempt {
                id: AttemptId::new(id_str),
                instance_id: ProblemInstanceId::new(inst_str),
                schema_id: SchemaId::new(sch_str),
                skill_id: SkillId::new(sk_str),
                card_id,
                user_answer,
                is_correct: is_correct_int != 0,
                score,
                time_taken_ms: time_ms as u64,
                attempted_at,
                metadata,
            })
        })?;

        let mut attempts = Vec::new();
        for a in rows {
            attempts.push(a?);
        }
        Ok(attempts)
    }

    pub fn get_practice_attempts_by_card(&self, card_id: i64) -> Result<Vec<PracticeAttempt>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT id, instance_id, schema_id, skill_id, card_id,
                   user_answer, is_correct, score, time_taken_ms, attempted_at, metadata
            FROM practice_attempts
            WHERE card_id = ?1
            ORDER BY attempted_at DESC, rowid DESC
            "#,
        )?;

        let rows = stmt.query_map(params![card_id], |row| {
            let id_str: String = row.get(0)?;
            let inst_str: String = row.get(1)?;
            let sch_str: String = row.get(2)?;
            let sk_str: String = row.get(3)?;
            let card_id: Option<i64> = row.get(4)?;
            let ans_str: String = row.get(5)?;
            let is_correct_int: i32 = row.get(6)?;
            let score: f64 = row.get(7)?;
            let time_ms: i64 = row.get(8)?;
            let attempted_at: i64 = row.get(9)?;
            let meta_str: String = row.get(10)?;

            let user_answer: serde_json::Value =
                serde_json::from_str(&ans_str).unwrap_or_default();
            let metadata: serde_json::Value =
                serde_json::from_str(&meta_str).unwrap_or_default();

            Ok(PracticeAttempt {
                id: AttemptId::new(id_str),
                instance_id: ProblemInstanceId::new(inst_str),
                schema_id: SchemaId::new(sch_str),
                skill_id: SkillId::new(sk_str),
                card_id,
                user_answer,
                is_correct: is_correct_int != 0,
                score,
                time_taken_ms: time_ms as u64,
                attempted_at,
                metadata,
            })
        })?;

        let mut attempts = Vec::new();
        for a in rows {
            attempts.push(a?);
        }
        Ok(attempts)
    }

    pub fn insert_error_event(&self, error: &ErrorEvent) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let details_json = serde_json::to_string(&error.details)?;

        conn.execute(
            r#"
            INSERT INTO error_events (id, attempt_id, error_category, details, occurred_at)
            VALUES (?1, ?2, ?3, ?4, ?5);
            "#,
            params![
                error.id.as_str(),
                error.attempt_id.as_str(),
                error.error_category,
                details_json,
                error.occurred_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_errors_for_attempt(&self, attempt_id: &AttemptId) -> Result<Vec<ErrorEvent>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT id, attempt_id, error_category, details, occurred_at
            FROM error_events
            WHERE attempt_id = ?1
            ORDER BY occurred_at ASC
            "#,
        )?;

        let rows = stmt.query_map(params![attempt_id.as_str()], |row| {
            let id_str: String = row.get(0)?;
            let att_str: String = row.get(1)?;
            let cat_str: String = row.get(2)?;
            let details_str: String = row.get(3)?;
            let occurred_at: i64 = row.get(4)?;

            let details: serde_json::Value =
                serde_json::from_str(&details_str).unwrap_or_default();

            Ok(ErrorEvent {
                id: ErrorEventId::new(id_str),
                attempt_id: AttemptId::new(att_str),
                error_category: cat_str,
                details,
                occurred_at,
            })
        })?;

        let mut errors = Vec::new();
        for e in rows {
            errors.push(e?);
        }
        Ok(errors)
    }

    // =========================================================================
    // PYQ SOURCE & MAPPING PERSISTENCE
    // =========================================================================

    pub fn insert_pyq_source(&self, pyq: &PYQSource) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let options_json = serde_json::to_string(&pyq.original_options)?;
        let answer_json = serde_json::to_string(&pyq.original_answer)?;
        let provenance_json = serde_json::to_string(&pyq.provenance)?;
        let metadata_json = serde_json::to_string(&pyq.metadata)?;

        conn.execute(
            r#"
            INSERT INTO pyq_sources (
                id, exam, year, paper, shift, session, domain,
                original_question, original_options, original_answer,
                source_reference, provenance, source_version,
                import_timestamp, metadata
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
            ON CONFLICT(id) DO UPDATE SET
                exam = excluded.exam,
                year = excluded.year,
                paper = excluded.paper,
                shift = excluded.shift,
                session = excluded.session,
                domain = excluded.domain,
                original_question = excluded.original_question,
                original_options = excluded.original_options,
                original_answer = excluded.original_answer,
                source_reference = excluded.source_reference,
                provenance = excluded.provenance,
                source_version = excluded.source_version,
                metadata = excluded.metadata;
            "#,
            params![
                pyq.id.as_str(),
                pyq.exam,
                pyq.year,
                pyq.paper,
                pyq.shift,
                pyq.session,
                pyq.domain.as_str(),
                pyq.original_question,
                options_json,
                answer_json,
                pyq.source_reference,
                provenance_json,
                pyq.source_version,
                pyq.import_timestamp,
                metadata_json,
            ],
        )?;
        Ok(())
    }

    pub fn get_pyq_source(&self, id: &PyqId) -> Result<Option<PYQSource>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT id, exam, year, paper, shift, session, domain,
                   original_question, original_options, original_answer,
                   source_reference, provenance, source_version,
                   import_timestamp, metadata
            FROM pyq_sources
            WHERE id = ?1
            "#,
        )?;

        let source = stmt
            .query_row(params![id.as_str()], |row| {
                let id_str: String = row.get(0)?;
                let exam: String = row.get(1)?;
                let year: u32 = row.get(2)?;
                let paper: Option<String> = row.get(3)?;
                let shift: Option<String> = row.get(4)?;
                let session: Option<String> = row.get(5)?;
                let domain_str: String = row.get(6)?;
                let question: String = row.get(7)?;
                let options_str: Option<String> = row.get(8)?;
                let answer_str: String = row.get(9)?;
                let source_ref: String = row.get(10)?;
                let prov_str: String = row.get(11)?;
                let source_version: u32 = row.get(12)?;
                let import_ts: i64 = row.get(13)?;
                let metadata_str: String = row.get(14)?;

                let domain: Domain = domain_str.parse().unwrap_or(Domain::Custom(domain_str));
                let original_options: Option<Vec<String>> = options_str
                    .and_then(|s| serde_json::from_str(&s).ok());
                let original_answer: serde_json::Value =
                    serde_json::from_str(&answer_str).unwrap_or_default();
                let provenance: ContentProvenance =
                    serde_json::from_str(&prov_str).unwrap_or_default();
                let metadata: serde_json::Value =
                    serde_json::from_str(&metadata_str).unwrap_or_default();

                Ok(PYQSource {
                    id: PyqId::new(id_str),
                    exam,
                    year,
                    paper,
                    shift,
                    session,
                    domain,
                    original_question: question,
                    original_options,
                    original_answer,
                    source_reference: source_ref,
                    provenance,
                    source_version,
                    import_timestamp: import_ts,
                    metadata,
                })
            })
            .optional()?;

        Ok(source)
    }

    pub fn list_pyq_sources_by_exam(&self, exam_filter: &str) -> Result<Vec<PYQSource>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT id, exam, year, paper, shift, session, domain,
                   original_question, original_options, original_answer,
                   source_reference, provenance, source_version,
                   import_timestamp, metadata
            FROM pyq_sources
            WHERE exam = ?1
            ORDER BY year DESC, import_timestamp DESC
            "#,
        )?;

        let rows = stmt.query_map(params![exam_filter], |row| {
            let id_str: String = row.get(0)?;
            let exam: String = row.get(1)?;
            let year: u32 = row.get(2)?;
            let paper: Option<String> = row.get(3)?;
            let shift: Option<String> = row.get(4)?;
            let session: Option<String> = row.get(5)?;
            let domain_str: String = row.get(6)?;
            let question: String = row.get(7)?;
            let options_str: Option<String> = row.get(8)?;
            let answer_str: String = row.get(9)?;
            let source_ref: String = row.get(10)?;
            let prov_str: String = row.get(11)?;
            let source_version: u32 = row.get(12)?;
            let import_ts: i64 = row.get(13)?;
            let metadata_str: String = row.get(14)?;

            let domain: Domain = domain_str.parse().unwrap_or(Domain::Custom(domain_str));
            let original_options: Option<Vec<String>> = options_str
                .and_then(|s| serde_json::from_str(&s).ok());
            let original_answer: serde_json::Value =
                serde_json::from_str(&answer_str).unwrap_or_default();
            let provenance: ContentProvenance =
                serde_json::from_str(&prov_str).unwrap_or_default();
            let metadata: serde_json::Value =
                serde_json::from_str(&metadata_str).unwrap_or_default();

            Ok(PYQSource {
                id: PyqId::new(id_str),
                exam,
                year,
                paper,
                shift,
                session,
                domain,
                original_question: question,
                original_options,
                original_answer,
                source_reference: source_ref,
                provenance,
                source_version,
                import_timestamp: import_ts,
                metadata,
            })
        })?;

        let mut sources = Vec::new();
        for s in rows {
            sources.push(s?);
        }
        Ok(sources)
    }

    pub fn insert_pyq_mapping(&self, mapping: &PyqMapping) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let diag_json = serde_json::to_string(&mapping.diagnostic_metadata)?;

        conn.execute(
            r#"
            INSERT INTO pyq_mappings (
                pyq_id, domain, skill_id, schema_id, problem_family_id,
                variant_structure, difficulty_level, target_latency_ms,
                diagnostic_metadata, status, confidence, reviewer_notes,
                updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
            ON CONFLICT(pyq_id) DO UPDATE SET
                domain = excluded.domain,
                skill_id = excluded.skill_id,
                schema_id = excluded.schema_id,
                problem_family_id = excluded.problem_family_id,
                variant_structure = excluded.variant_structure,
                difficulty_level = excluded.difficulty_level,
                target_latency_ms = excluded.target_latency_ms,
                diagnostic_metadata = excluded.diagnostic_metadata,
                status = excluded.status,
                confidence = excluded.confidence,
                reviewer_notes = excluded.reviewer_notes,
                updated_at = excluded.updated_at;
            "#,
            params![
                mapping.pyq_id.as_str(),
                mapping.domain.as_str(),
                mapping.skill_id.as_str(),
                mapping.schema_id.as_str(),
                mapping.problem_family_id.as_str(),
                mapping.variant_structure,
                mapping.difficulty_level,
                mapping.target_latency_ms,
                diag_json,
                mapping.status.as_str(),
                mapping.confidence.as_str(),
                mapping.reviewer_notes,
                mapping.updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_pyq_mapping(&self, pyq_id: &PyqId) -> Result<Option<PyqMapping>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT pyq_id, domain, skill_id, schema_id, problem_family_id,
                   variant_structure, difficulty_level, target_latency_ms,
                   diagnostic_metadata, status, confidence, reviewer_notes,
                   updated_at
            FROM pyq_mappings
            WHERE pyq_id = ?1
            "#,
        )?;

        let mapping = stmt
            .query_row(params![pyq_id.as_str()], |row| {
                let id_str: String = row.get(0)?;
                let domain_str: String = row.get(1)?;
                let skill_str: String = row.get(2)?;
                let schema_str: String = row.get(3)?;
                let family_str: String = row.get(4)?;
                let variant_structure: Option<String> = row.get(5)?;
                let diff: u32 = row.get(6)?;
                let latency: u64 = row.get(7)?;
                let diag_str: String = row.get(8)?;
                let status_str: String = row.get(9)?;
                let conf_str: String = row.get(10)?;
                let notes: Option<String> = row.get(11)?;
                let updated_at: i64 = row.get(12)?;

                let domain: Domain = domain_str.parse().unwrap_or(Domain::Custom(domain_str));
                let status: MappingStatus = match status_str.as_str() {
                    "verified" => MappingStatus::Verified,
                    "rejected" => MappingStatus::Rejected,
                    "unreviewed" => MappingStatus::Unreviewed,
                    _ => MappingStatus::Mapped,
                };
                let confidence: MappingConfidence = match conf_str.as_str() {
                    "deterministic" => MappingConfidence::Deterministic,
                    "needs_review" => MappingConfidence::NeedsReview,
                    _ => MappingConfidence::HighConfidence,
                };
                let diagnostic_metadata: serde_json::Value =
                    serde_json::from_str(&diag_str).unwrap_or_default();

                Ok(PyqMapping {
                    pyq_id: PyqId::new(id_str),
                    domain,
                    skill_id: SkillId::new(skill_str),
                    schema_id: SchemaId::new(schema_str),
                    problem_family_id: ProblemFamilyId::new(family_str),
                    variant_structure,
                    difficulty_level: diff,
                    target_latency_ms: latency,
                    diagnostic_metadata,
                    status,
                    confidence,
                    reviewer_notes: notes,
                    updated_at,
                })
            })
            .optional()?;

        Ok(mapping)
    }

    pub fn list_eligible_pyqs_for_schema(&self, schema_id: &SchemaId) -> Result<Vec<(PYQSource, PyqMapping)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT s.id, s.exam, s.year, s.paper, s.shift, s.session, s.domain,
                   s.original_question, s.original_options, s.original_answer,
                   s.source_reference, s.provenance, s.source_version,
                   s.import_timestamp, s.metadata,
                   m.domain, m.skill_id, m.schema_id, m.problem_family_id,
                   m.variant_structure, m.difficulty_level, m.target_latency_ms,
                   m.diagnostic_metadata, m.status, m.confidence, m.reviewer_notes,
                   m.updated_at
            FROM pyq_sources s
            JOIN pyq_mappings m ON s.id = m.pyq_id
            WHERE m.schema_id = ?1
              AND (m.status = 'verified' OR (m.status = 'mapped' AND m.confidence IN ('deterministic', 'high_confidence')))
            ORDER BY s.year DESC
            "#,
        )?;

        let rows = stmt.query_map(params![schema_id.as_str()], |row| {
            let id_str: String = row.get(0)?;
            let exam: String = row.get(1)?;
            let year: u32 = row.get(2)?;
            let paper: Option<String> = row.get(3)?;
            let shift: Option<String> = row.get(4)?;
            let session: Option<String> = row.get(5)?;
            let domain_str: String = row.get(6)?;
            let question: String = row.get(7)?;
            let options_str: Option<String> = row.get(8)?;
            let answer_str: String = row.get(9)?;
            let source_ref: String = row.get(10)?;
            let prov_str: String = row.get(11)?;
            let source_version: u32 = row.get(12)?;
            let import_ts: i64 = row.get(13)?;
            let metadata_str: String = row.get(14)?;

            let m_domain_str: String = row.get(15)?;
            let m_skill_str: String = row.get(16)?;
            let m_schema_str: String = row.get(17)?;
            let m_family_str: String = row.get(18)?;
            let m_variant: Option<String> = row.get(19)?;
            let m_diff: u32 = row.get(20)?;
            let m_latency: u64 = row.get(21)?;
            let m_diag_str: String = row.get(22)?;
            let m_status_str: String = row.get(23)?;
            let m_conf_str: String = row.get(24)?;
            let m_notes: Option<String> = row.get(25)?;
            let m_updated_at: i64 = row.get(26)?;

            let domain: Domain = domain_str.parse().unwrap_or(Domain::Custom(domain_str));
            let original_options: Option<Vec<String>> = options_str
                .and_then(|s| serde_json::from_str(&s).ok());
            let original_answer: serde_json::Value =
                serde_json::from_str(&answer_str).unwrap_or_default();
            let provenance: ContentProvenance =
                serde_json::from_str(&prov_str).unwrap_or_default();
            let metadata: serde_json::Value =
                serde_json::from_str(&metadata_str).unwrap_or_default();

            let pyq = PYQSource {
                id: PyqId::new(id_str.clone()),
                exam,
                year,
                paper,
                shift,
                session,
                domain,
                original_question: question,
                original_options,
                original_answer,
                source_reference: source_ref,
                provenance,
                source_version,
                import_timestamp: import_ts,
                metadata,
            };

            let m_domain: Domain = m_domain_str.parse().unwrap_or(Domain::Custom(m_domain_str));
            let m_status: MappingStatus = match m_status_str.as_str() {
                "verified" => MappingStatus::Verified,
                "rejected" => MappingStatus::Rejected,
                "unreviewed" => MappingStatus::Unreviewed,
                _ => MappingStatus::Mapped,
            };
            let m_confidence: MappingConfidence = match m_conf_str.as_str() {
                "deterministic" => MappingConfidence::Deterministic,
                "needs_review" => MappingConfidence::NeedsReview,
                _ => MappingConfidence::HighConfidence,
            };
            let diagnostic_metadata: serde_json::Value =
                serde_json::from_str(&m_diag_str).unwrap_or_default();

            let mapping = PyqMapping {
                pyq_id: PyqId::new(id_str),
                domain: m_domain,
                skill_id: SkillId::new(m_skill_str),
                schema_id: SchemaId::new(m_schema_str),
                problem_family_id: ProblemFamilyId::new(m_family_str),
                variant_structure: m_variant,
                difficulty_level: m_diff,
                target_latency_ms: m_latency,
                diagnostic_metadata,
                status: m_status,
                confidence: m_confidence,
                reviewer_notes: m_notes,
                updated_at: m_updated_at,
            };

            Ok((pyq, mapping))
        })?;

        let mut list = Vec::new();
        for item in rows {
            list.push(item?);
        }
        Ok(list)
    }

    /// Retrieve all registered schemas as a HashMap keyed by SchemaId in a single query.
    pub fn list_all_schemas_map(&self) -> Result<HashMap<SchemaId, SchemaPracticeObject>> {
        let schemas = self.list_all_schemas()?;
        let mut map = HashMap::with_capacity(schemas.len());
        for s in schemas {
            map.insert(s.id.clone(), s);
        }
        Ok(map)
    }

    /// Retrieve all registered skill states as a HashMap keyed by SkillId in a single query.
    pub fn list_all_skill_states_map(&self) -> Result<HashMap<SkillId, SkillState>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT skill_id, mastery, confidence, total_attempts,
                   successful_attempts, last_practiced_at, custom_state, updated_at
            FROM skill_states
            "#,
        )?;

        let rows = stmt.query_map([], |row| {
            let id_str: String = row.get(0)?;
            let mastery: f64 = row.get(1)?;
            let confidence: f64 = row.get(2)?;
            let total_attempts: u32 = row.get(3)?;
            let successful_attempts: u32 = row.get(4)?;
            let last_practiced_at: Option<i64> = row.get(5)?;
            let custom_str: String = row.get(6)?;
            let updated_at: i64 = row.get(7)?;

            let custom_state: serde_json::Value =
                serde_json::from_str(&custom_str).unwrap_or_default();

            let mut state = SkillState::new(SkillId::new(id_str.clone()));
            state.mastery = mastery;
            state.confidence = confidence;
            state.total_attempts = total_attempts;
            state.successful_attempts = successful_attempts;
            state.last_practiced_at = last_practiced_at;
            state.custom_state = custom_state;
            state.updated_at = updated_at;
            state.restore_from_custom_state();

            Ok((SkillId::new(id_str), state))
        })?;

        let mut map = HashMap::new();
        for r in rows {
            let (id, s) = r?;
            map.insert(id, s);
        }
        Ok(map)
    }

    /// Retrieve all eligible PYQs grouped by SchemaId in a single batch query.
    pub fn list_all_eligible_pyqs_map(&self) -> Result<HashMap<SchemaId, Vec<(PYQSource, PyqMapping)>>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT s.id, s.exam, s.year, s.paper, s.shift, s.session, s.domain,
                   s.original_question, s.original_options, s.original_answer,
                   s.source_reference, s.provenance, s.source_version, s.import_timestamp,
                   s.metadata,
                   m.domain, m.skill_id, m.schema_id, m.problem_family_id,
                   m.variant_structure, m.difficulty_level, m.target_latency_ms,
                   m.diagnostic_metadata, m.status, m.confidence, m.reviewer_notes,
                   m.updated_at
            FROM pyq_sources s
            JOIN pyq_mappings m ON s.id = m.pyq_id
            WHERE (m.status = 'verified' OR (m.status = 'mapped' AND m.confidence IN ('deterministic', 'high_confidence')))
            ORDER BY s.year DESC
            "#,
        )?;

        let rows = stmt.query_map([], |row| {
            let id_str: String = row.get(0)?;
            let exam: String = row.get(1)?;
            let year: u32 = row.get(2)?;
            let paper: Option<String> = row.get(3)?;
            let shift: Option<String> = row.get(4)?;
            let session: Option<String> = row.get(5)?;
            let domain_str: String = row.get(6)?;
            let question: String = row.get(7)?;
            let options_str: Option<String> = row.get(8)?;
            let answer_str: String = row.get(9)?;
            let source_ref: String = row.get(10)?;
            let prov_str: String = row.get(11)?;
            let source_version: u32 = row.get(12)?;
            let import_ts: i64 = row.get(13)?;
            let metadata_str: String = row.get(14)?;

            let m_domain_str: String = row.get(15)?;
            let m_skill_str: String = row.get(16)?;
            let m_schema_str: String = row.get(17)?;
            let m_family_str: String = row.get(18)?;
            let m_variant: Option<String> = row.get(19)?;
            let m_diff: u32 = row.get(20)?;
            let m_latency: u64 = row.get(21)?;
            let m_diag_str: String = row.get(22)?;
            let m_status_str: String = row.get(23)?;
            let m_conf_str: String = row.get(24)?;
            let m_notes: Option<String> = row.get(25)?;
            let m_updated_at: i64 = row.get(26)?;

            let domain: Domain = domain_str.parse().unwrap_or(Domain::Custom(domain_str));
            let original_options: Option<Vec<String>> = options_str
                .and_then(|s| serde_json::from_str(&s).ok());
            let original_answer: serde_json::Value =
                serde_json::from_str(&answer_str).unwrap_or_default();
            let provenance: ContentProvenance =
                serde_json::from_str(&prov_str).unwrap_or_default();
            let metadata: serde_json::Value =
                serde_json::from_str(&metadata_str).unwrap_or_default();

            let pyq = PYQSource {
                id: PyqId::new(id_str.clone()),
                exam,
                year,
                paper,
                shift,
                session,
                domain,
                original_question: question,
                original_options,
                original_answer,
                source_reference: source_ref,
                provenance,
                source_version,
                import_timestamp: import_ts,
                metadata,
            };

            let m_domain: Domain = m_domain_str.parse().unwrap_or(Domain::Custom(m_domain_str));
            let m_status: MappingStatus = match m_status_str.as_str() {
                "verified" => MappingStatus::Verified,
                "rejected" => MappingStatus::Rejected,
                "unreviewed" => MappingStatus::Unreviewed,
                _ => MappingStatus::Mapped,
            };
            let m_confidence: MappingConfidence = match m_conf_str.as_str() {
                "deterministic" => MappingConfidence::Deterministic,
                "needs_review" => MappingConfidence::NeedsReview,
                _ => MappingConfidence::HighConfidence,
            };
            let diagnostic_metadata: serde_json::Value =
                serde_json::from_str(&m_diag_str).unwrap_or_default();

            let schema_id = SchemaId::new(m_schema_str.clone());
            let mapping = PyqMapping {
                pyq_id: PyqId::new(id_str),
                domain: m_domain,
                skill_id: SkillId::new(m_skill_str),
                schema_id: schema_id.clone(),
                problem_family_id: ProblemFamilyId::new(m_family_str),
                variant_structure: m_variant,
                difficulty_level: m_diff,
                target_latency_ms: m_latency,
                diagnostic_metadata,
                status: m_status,
                confidence: m_confidence,
                reviewer_notes: m_notes,
                updated_at: m_updated_at,
            };

            Ok((schema_id, pyq, mapping))
        })?;

        let mut map: HashMap<SchemaId, Vec<(PYQSource, PyqMapping)>> = HashMap::new();
        for item in rows {
            let (sch_id, pyq, mapping) = item?;
            map.entry(sch_id).or_default().push((pyq, mapping));
        }
        Ok(map)
    }

    // =========================================================================
    // REJECTED VARIANT AUDIT STORE
    // =========================================================================

    pub fn insert_rejected_variant(&self, record: &RejectedVariantRecord) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let instance_json = serde_json::to_string(&record.generated_instance_json)?;

        conn.execute(
            r#"
            INSERT INTO rejected_variants (
                id, source_pyq_id, schema_id, family_id, seed,
                variant_type, failure_reason, generated_instance_json,
                rejected_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
            params![
                record.id.as_str(),
                record.source_pyq_id.as_ref().map(|id| id.as_str()),
                record.schema_id.as_str(),
                record.family_id.as_str(),
                record.seed as i64,
                record.variant_type,
                record.failure_reason,
                instance_json,
                record.rejected_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_rejected_variants(&self, limit: usize) -> Result<Vec<RejectedVariantRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT id, source_pyq_id, schema_id, family_id, seed,
                   variant_type, failure_reason, generated_instance_json,
                   rejected_at
            FROM rejected_variants
            ORDER BY rejected_at DESC
            LIMIT ?1
            "#,
        )?;

        let rows = stmt.query_map(params![limit as i64], |row| {
            let id_str: String = row.get(0)?;
            let source_pyq: Option<String> = row.get(1)?;
            let schema_str: String = row.get(2)?;
            let family_str: String = row.get(3)?;
            let seed_i: i64 = row.get(4)?;
            let variant_type: String = row.get(5)?;
            let failure_reason: String = row.get(6)?;
            let inst_str: String = row.get(7)?;
            let rejected_at: i64 = row.get(8)?;

            let generated_instance_json: serde_json::Value =
                serde_json::from_str(&inst_str).unwrap_or_default();

            Ok(RejectedVariantRecord {
                id: RejectedVariantId::new(id_str),
                source_pyq_id: source_pyq.map(PyqId::new),
                schema_id: SchemaId::new(schema_str),
                family_id: ProblemFamilyId::new(family_str),
                seed: seed_i as u64,
                variant_type,
                failure_reason,
                generated_instance_json,
                rejected_at,
            })
        })?;

        let mut list = Vec::new();
        for item in rows {
            list.push(item?);
        }
        Ok(list)
    }

    // =========================================================================
    // EXAM PROFILE PERSISTENCE
    // =========================================================================

    pub fn insert_exam_profile(&self, profile: &ExamProfile) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let subjects_json = serde_json::to_string(&profile.subjects)?;
        let domain_w_json = serde_json::to_string(&profile.domain_weights)?;
        let topic_w_json = serde_json::to_string(&profile.topic_weights)?;
        let formats_json = serde_json::to_string(&profile.preferred_formats)?;
        let latencies_json = serde_json::to_string(&profile.target_latencies_ms)?;
        let diff_json = serde_json::to_string(&profile.difficulty_distribution)?;
        let metadata_json = serde_json::to_string(&profile.metadata)?;

        conn.execute(
            r#"
            INSERT INTO exam_profiles (
                id, name, description, subjects, domain_weights,
                topic_weights, preferred_formats, target_latencies_ms,
                difficulty_distribution, pyq_weight, objective,
                metadata, created_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                description = excluded.description,
                subjects = excluded.subjects,
                domain_weights = excluded.domain_weights,
                topic_weights = excluded.topic_weights,
                preferred_formats = excluded.preferred_formats,
                target_latencies_ms = excluded.target_latencies_ms,
                difficulty_distribution = excluded.difficulty_distribution,
                pyq_weight = excluded.pyq_weight,
                objective = excluded.objective,
                metadata = excluded.metadata;
            "#,
            params![
                profile.id.as_str(),
                profile.name,
                profile.description,
                subjects_json,
                domain_w_json,
                topic_w_json,
                formats_json,
                latencies_json,
                diff_json,
                profile.pyq_weight,
                profile.objective.as_str(),
                metadata_json,
                profile.created_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_exam_profile(&self, id: &ExamProfileId) -> Result<Option<ExamProfile>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT id, name, description, subjects, domain_weights,
                   topic_weights, preferred_formats, target_latencies_ms,
                   difficulty_distribution, pyq_weight, objective,
                   metadata, created_at
            FROM exam_profiles
            WHERE id = ?1
            "#,
        )?;

        let profile = stmt
            .query_row(params![id.as_str()], |row| {
                let id_str: String = row.get(0)?;
                let name: String = row.get(1)?;
                let description: String = row.get(2)?;
                let subjects_str: String = row.get(3)?;
                let domain_w_str: String = row.get(4)?;
                let topic_w_str: String = row.get(5)?;
                let formats_str: String = row.get(6)?;
                let latencies_str: String = row.get(7)?;
                let diff_str: String = row.get(8)?;
                let pyq_w: f64 = row.get(9)?;
                let obj_str: String = row.get(10)?;
                let meta_str: String = row.get(11)?;
                let created_at: i64 = row.get(12)?;

                let subjects: Vec<Domain> = serde_json::from_str(&subjects_str).unwrap_or_default();
                let domain_weights: std::collections::HashMap<Domain, f64> =
                    serde_json::from_str(&domain_w_str).unwrap_or_default();
                let topic_weights: std::collections::HashMap<String, f64> =
                    serde_json::from_str(&topic_w_str).unwrap_or_default();
                let preferred_formats: Vec<String> =
                    serde_json::from_str(&formats_str).unwrap_or_default();
                let target_latencies_ms: std::collections::HashMap<String, u64> =
                    serde_json::from_str(&latencies_str).unwrap_or_default();
                let difficulty_distribution: std::collections::HashMap<u32, f64> =
                    serde_json::from_str(&diff_str).unwrap_or_default();
                let metadata: serde_json::Value =
                    serde_json::from_str(&meta_str).unwrap_or_default();

                let objective: ExamObjective = match obj_str.as_str() {
                    "concept_mastery" => ExamObjective::ConceptMastery,
                    "comprehensive_mock" => ExamObjective::ComprehensiveMock,
                    "weak_area_remediation" => ExamObjective::WeakAreaRemediation,
                    "balanced_preparation" => ExamObjective::BalancedPreparation,
                    _ => ExamObjective::SpeedAndAccuracy,
                };

                Ok(ExamProfile {
                    id: ExamProfileId::new(id_str),
                    name,
                    description,
                    subjects,
                    domain_weights,
                    topic_weights,
                    preferred_formats,
                    target_latencies_ms,
                    difficulty_distribution,
                    pyq_weight: pyq_w,
                    objective,
                    metadata,
                    created_at,
                })
            })
            .optional()?;

        Ok(profile)
    }

    pub fn list_exam_profiles(&self) -> Result<Vec<ExamProfile>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT id, name, description, subjects, domain_weights,
                   topic_weights, preferred_formats, target_latencies_ms,
                   difficulty_distribution, pyq_weight, objective,
                   metadata, created_at
            FROM exam_profiles
            ORDER BY name ASC
            "#,
        )?;

        let rows = stmt.query_map([], |row| {
            let id_str: String = row.get(0)?;
            let name: String = row.get(1)?;
            let description: String = row.get(2)?;
            let subjects_str: String = row.get(3)?;
            let domain_w_str: String = row.get(4)?;
            let topic_w_str: String = row.get(5)?;
            let formats_str: String = row.get(6)?;
            let latencies_str: String = row.get(7)?;
            let diff_str: String = row.get(8)?;
            let pyq_w: f64 = row.get(9)?;
            let obj_str: String = row.get(10)?;
            let meta_str: String = row.get(11)?;
            let created_at: i64 = row.get(12)?;

            let subjects: Vec<Domain> = serde_json::from_str(&subjects_str).unwrap_or_default();
            let domain_weights: std::collections::HashMap<Domain, f64> =
                serde_json::from_str(&domain_w_str).unwrap_or_default();
            let topic_weights: std::collections::HashMap<String, f64> =
                serde_json::from_str(&topic_w_str).unwrap_or_default();
            let preferred_formats: Vec<String> =
                serde_json::from_str(&formats_str).unwrap_or_default();
            let target_latencies_ms: std::collections::HashMap<String, u64> =
                serde_json::from_str(&latencies_str).unwrap_or_default();
            let difficulty_distribution: std::collections::HashMap<u32, f64> =
                serde_json::from_str(&diff_str).unwrap_or_default();
            let metadata: serde_json::Value =
                serde_json::from_str(&meta_str).unwrap_or_default();

            let objective: ExamObjective = match obj_str.as_str() {
                "concept_mastery" => ExamObjective::ConceptMastery,
                "comprehensive_mock" => ExamObjective::ComprehensiveMock,
                "weak_area_remediation" => ExamObjective::WeakAreaRemediation,
                "balanced_preparation" => ExamObjective::BalancedPreparation,
                _ => ExamObjective::SpeedAndAccuracy,
            };

            Ok(ExamProfile {
                id: ExamProfileId::new(id_str),
                name,
                description,
                subjects,
                domain_weights,
                topic_weights,
                preferred_formats,
                target_latencies_ms,
                difficulty_distribution,
                pyq_weight: pyq_w,
                objective,
                metadata,
                created_at,
            })
        })?;

        let mut list = Vec::new();
        for item in rows {
            list.push(item?);
        }
        Ok(list)
    }

    /// Retrieve repeatedly failing schemas for a specific exam profile based on recent practice attempts.
    pub fn get_failing_schemas_for_exam(&self, exam_id: &str) -> Result<Vec<(SchemaId, f64, usize)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT a.schema_id,
                   AVG(CASE WHEN a.is_correct = 0 THEN 1.0 ELSE 0.0 END) as fail_rate,
                   COUNT(a.id) as total_attempts
            FROM practice_attempts a
            JOIN pyq_mappings m ON a.schema_id = m.schema_id
            JOIN pyq_sources s ON m.pyq_id = s.id
            WHERE s.exam = ?1
            GROUP BY a.schema_id
            HAVING total_attempts >= 1 AND fail_rate > 0.0
            ORDER BY fail_rate DESC, total_attempts DESC
            "#,
        )?;

        let rows = stmt.query_map(params![exam_id], |row| {
            let schema_str: String = row.get(0)?;
            let fail_rate: f64 = row.get(1)?;
            let count: usize = row.get(2)?;
            Ok((SchemaId::new(schema_str), fail_rate, count))
        })?;

        let mut list = Vec::new();
        for item in rows {
            list.push(item?);
        }
        Ok(list)
    }

    pub fn insert_practice_item(&self, item: &PracticeItem) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let origin_json = serde_json::to_string(&item.origin)?;
        let q_type_json = serde_json::to_string(&item.question_type)?;
        let tags_json = serde_json::to_string(&item.structural_tags)?;
        let decisions_json = serde_json::to_string(&item.decision_points)?;
        let errors_json = serde_json::to_string(&item.error_categories)?;
        let prereqs_json = serde_json::to_string(&item.prerequisites)?;
        let prov_json = serde_json::to_string(&item.provenance)?;
        let meta_json = serde_json::to_string(&item.metadata)?;

        conn.execute(
            r#"
            INSERT INTO practice_items (
                id, origin, domain, chapter, skill_id, schema_id, problem_family_id,
                question_type, prompt, difficulty, structural_tags, decision_points,
                error_categories, prerequisites, provenance, created_at, metadata
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)
            ON CONFLICT(id) DO UPDATE SET
                origin = excluded.origin,
                domain = excluded.domain,
                chapter = excluded.chapter,
                skill_id = excluded.skill_id,
                schema_id = excluded.schema_id,
                problem_family_id = excluded.problem_family_id,
                question_type = excluded.question_type,
                prompt = excluded.prompt,
                difficulty = excluded.difficulty,
                structural_tags = excluded.structural_tags,
                decision_points = excluded.decision_points,
                error_categories = excluded.error_categories,
                prerequisites = excluded.prerequisites,
                provenance = excluded.provenance,
                metadata = excluded.metadata;
            "#,
            params![
                item.id.as_str(),
                origin_json,
                item.domain.as_str(),
                item.chapter,
                item.skill_id.as_str(),
                item.schema_id.as_str(),
                item.problem_family_id.as_str(),
                q_type_json,
                item.prompt,
                item.difficulty,
                tags_json,
                decisions_json,
                errors_json,
                prereqs_json,
                prov_json,
                item.created_at,
                meta_json,
            ],
        )?;
        Ok(())
    }

    pub fn get_practice_item(&self, id: &PracticeItemId) -> Result<Option<PracticeItem>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT origin, domain, chapter, skill_id, schema_id, problem_family_id,
                   question_type, prompt, difficulty, structural_tags, decision_points,
                   error_categories, prerequisites, provenance, created_at, metadata
            FROM practice_items WHERE id = ?1
            "#,
        )?;

        let item = stmt
            .query_row(params![id.as_str()], |row| {
                let origin_str: String = row.get(0)?;
                let domain_str: String = row.get(1)?;
                let chapter: String = row.get(2)?;
                let skill_id_str: String = row.get(3)?;
                let schema_id_str: String = row.get(4)?;
                let family_id_str: String = row.get(5)?;
                let q_type_str: String = row.get(6)?;
                let prompt: String = row.get(7)?;
                let difficulty: f64 = row.get(8)?;
                let tags_str: String = row.get(9)?;
                let dec_str: String = row.get(10)?;
                let err_str: String = row.get(11)?;
                let pre_str: String = row.get(12)?;
                let prov_str: String = row.get(13)?;
                let created_at: i64 = row.get(14)?;
                let meta_str: String = row.get(15)?;

                Ok(PracticeItem {
                    id: id.clone(),
                    origin: serde_json::from_str(&origin_str).unwrap(),
                    domain: domain_str.parse().unwrap_or(Domain::Custom(domain_str)),
                    chapter,
                    skill_id: SkillId::new(skill_id_str),
                    schema_id: SchemaId::new(schema_id_str),
                    problem_family_id: ProblemFamilyId::new(family_id_str),
                    question_type: serde_json::from_str(&q_type_str).unwrap(),
                    prompt,
                    difficulty,
                    structural_tags: serde_json::from_str(&tags_str).unwrap_or_default(),
                    decision_points: serde_json::from_str(&dec_str).unwrap_or_default(),
                    error_categories: serde_json::from_str(&err_str).unwrap_or_default(),
                    prerequisites: serde_json::from_str(&pre_str).unwrap_or_default(),
                    provenance: serde_json::from_str(&prov_str).unwrap_or_default(),
                    created_at,
                    metadata: serde_json::from_str(&meta_str).unwrap_or_default(),
                })
            })
            .optional()?;
        Ok(item)
    }

    pub fn get_practice_items_by_schema(&self, schema_id: &SchemaId) -> Result<Vec<PracticeItem>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT id, origin, domain, chapter, skill_id, problem_family_id,
                   question_type, prompt, difficulty, structural_tags, decision_points,
                   error_categories, prerequisites, provenance, created_at, metadata
            FROM practice_items WHERE schema_id = ?1
            "#,
        )?;

        let rows = stmt.query_map(params![schema_id.as_str()], |row| {
                let id_str: String = row.get(0)?;
                let origin_str: String = row.get(1)?;
                let domain_str: String = row.get(2)?;
                let chapter: String = row.get(3)?;
                let skill_id_str: String = row.get(4)?;
                let family_id_str: String = row.get(5)?;
                let q_type_str: String = row.get(6)?;
                let prompt: String = row.get(7)?;
                let difficulty: f64 = row.get(8)?;
                let tags_str: String = row.get(9)?;
                let dec_str: String = row.get(10)?;
                let err_str: String = row.get(11)?;
                let pre_str: String = row.get(12)?;
                let prov_str: String = row.get(13)?;
                let created_at: i64 = row.get(14)?;
                let meta_str: String = row.get(15)?;

                Ok(PracticeItem {
                    id: PracticeItemId::new(id_str),
                    origin: serde_json::from_str(&origin_str).unwrap(),
                    domain: domain_str.parse().unwrap_or(Domain::Custom(domain_str)),
                    chapter,
                    skill_id: SkillId::new(skill_id_str),
                    schema_id: schema_id.clone(),
                    problem_family_id: ProblemFamilyId::new(family_id_str),
                    question_type: serde_json::from_str(&q_type_str).unwrap(),
                    prompt,
                    difficulty,
                    structural_tags: serde_json::from_str(&tags_str).unwrap_or_default(),
                    decision_points: serde_json::from_str(&dec_str).unwrap_or_default(),
                    error_categories: serde_json::from_str(&err_str).unwrap_or_default(),
                    prerequisites: serde_json::from_str(&pre_str).unwrap_or_default(),
                    provenance: serde_json::from_str(&prov_str).unwrap_or_default(),
                    created_at,
                    metadata: serde_json::from_str(&meta_str).unwrap_or_default(),
                })
        })?;

        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    }

    pub fn insert_chapter_profile(&self, profile: &ChapterPracticeProfile) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let schemas_json = serde_json::to_string(&profile.supported_schemas)?;
        let fams_json = serde_json::to_string(&profile.supported_problem_families)?;
        let caps_json = serde_json::to_string(&profile.generator_capabilities)?;
        let rec_json = serde_json::to_string(&profile.recognition_signals)?;
        let dec_json = serde_json::to_string(&profile.decision_points)?;
        let var_json = serde_json::to_string(&profile.variation_dimensions)?;
        let pre_json = serde_json::to_string(&profile.prerequisites)?;
        let err_json = serde_json::to_string(&profile.error_categories)?;
        let ex_json = serde_json::to_string(&profile.exam_relevance)?;
        let meta_json = serde_json::to_string(&profile.metadata)?;

        conn.execute(
            r#"
            INSERT INTO chapter_practice_profiles (
                chapter_name, domain, supported_schemas, supported_problem_families,
                generator_capabilities, recognition_signals, decision_points,
                variation_dimensions, prerequisites, error_categories, exam_relevance,
                created_at, metadata
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
            ON CONFLICT(chapter_name) DO UPDATE SET
                domain = excluded.domain,
                supported_schemas = excluded.supported_schemas,
                supported_problem_families = excluded.supported_problem_families,
                generator_capabilities = excluded.generator_capabilities,
                recognition_signals = excluded.recognition_signals,
                decision_points = excluded.decision_points,
                variation_dimensions = excluded.variation_dimensions,
                prerequisites = excluded.prerequisites,
                error_categories = excluded.error_categories,
                exam_relevance = excluded.exam_relevance,
                metadata = excluded.metadata;
            "#,
            params![
                profile.chapter_name,
                profile.domain.as_str(),
                schemas_json,
                fams_json,
                caps_json,
                rec_json,
                dec_json,
                var_json,
                pre_json,
                err_json,
                ex_json,
                profile.created_at,
                meta_json,
            ],
        )?;
        Ok(())
    }

    pub fn get_chapter_profile(&self, chapter_name: &str) -> Result<Option<ChapterPracticeProfile>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT domain, supported_schemas, supported_problem_families,
                   generator_capabilities, recognition_signals, decision_points,
                   variation_dimensions, prerequisites, error_categories, exam_relevance,
                   created_at, metadata
            FROM chapter_practice_profiles WHERE chapter_name = ?1
            "#,
        )?;

        let prof = stmt
            .query_row(params![chapter_name], |row| {
                let domain_str: String = row.get(0)?;
                let schemas_str: String = row.get(1)?;
                let fams_str: String = row.get(2)?;
                let caps_str: String = row.get(3)?;
                let rec_str: String = row.get(4)?;
                let dec_str: String = row.get(5)?;
                let var_str: String = row.get(6)?;
                let pre_str: String = row.get(7)?;
                let err_str: String = row.get(8)?;
                let ex_str: String = row.get(9)?;
                let created_at: i64 = row.get(10)?;
                let meta_str: String = row.get(11)?;

                Ok(ChapterPracticeProfile {
                    chapter_name: chapter_name.to_string(),
                    domain: domain_str.parse().unwrap_or(Domain::Custom(domain_str)),
                    supported_schemas: serde_json::from_str(&schemas_str).unwrap_or_default(),
                    supported_problem_families: serde_json::from_str(&fams_str).unwrap_or_default(),
                    generator_capabilities: serde_json::from_str(&caps_str).unwrap_or_default(),
                    recognition_signals: serde_json::from_str(&rec_str).unwrap_or_default(),
                    decision_points: serde_json::from_str(&dec_str).unwrap_or_default(),
                    variation_dimensions: serde_json::from_str(&var_str).unwrap_or_default(),
                    prerequisites: serde_json::from_str(&pre_str).unwrap_or_default(),
                    error_categories: serde_json::from_str(&err_str).unwrap_or_default(),
                    exam_relevance: serde_json::from_str(&ex_str).unwrap_or_default(),
                    created_at,
                    metadata: serde_json::from_str(&meta_str).unwrap_or_default(),
                })
            })
            .optional()?;
        Ok(prof)
    }

    pub fn save_remediation_queue(&self, queue: &RemediationQueue) -> Result<()> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;

        tx.execute("DELETE FROM remediation_queue_items", [])?;
        for action in &queue.pending_actions {
            let step_err_str = action.step_error.as_ref().map(|e| e.as_str());
            tx.execute(
                r#"
                INSERT INTO remediation_queue_items (
                    id, kind, skill_id, schema_id, domain, primary_error,
                    step_error, preferred_difficulty, preferred_variant,
                    source_attempt_id, urgency, requires_acknowledgement,
                    recurrence_count, rationale, created_at
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15);
                "#,
                params![
                    action.id,
                    action.kind.as_str(),
                    action.skill_id.as_str(),
                    action.schema_id.as_str(),
                    action.domain.as_str(),
                    action.primary_error.as_str(),
                    step_err_str,
                    action.preferred_difficulty as i64,
                    action.preferred_variant,
                    action.source_attempt_id.as_str(),
                    action.urgency.as_str(),
                    if action.requires_acknowledgement { 1 } else { 0 },
                    action.recurrence_count as i64,
                    action.rationale,
                    action.created_at,
                ],
            )?;
        }

        tx.execute("DELETE FROM remediation_recurrence", [])?;
        for ((skill_id, error_cat), &count) in &queue.recurrence_tracker {
            tx.execute(
                r#"
                INSERT INTO remediation_recurrence (skill_id, error_category, count, updated_at)
                VALUES (?1, ?2, ?3, ?4);
                "#,
                params![
                    skill_id.as_str(),
                    error_cat.as_str(),
                    count as i64,
                    chrono::Utc::now().timestamp(),
                ],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    pub fn load_remediation_queue(&self) -> Result<RemediationQueue> {
        let conn = self.conn.lock().unwrap();

        let table_exists: bool = conn.query_row(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='remediation_queue_items'",
            [],
            |row| row.get(0),
        ).unwrap_or(false);

        if !table_exists {
            return Ok(RemediationQueue::new());
        }

        let mut stmt = conn.prepare(
            r#"
            SELECT id, kind, skill_id, schema_id, domain, primary_error,
                   step_error, preferred_difficulty, preferred_variant,
                   source_attempt_id, urgency, requires_acknowledgement,
                   recurrence_count, rationale, created_at
            FROM remediation_queue_items
            ORDER BY created_at ASC
            "#,
        )?;

        let rows = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let kind_str: String = row.get(1)?;
            let skill_id_str: String = row.get(2)?;
            let schema_id_str: String = row.get(3)?;
            let domain_str: String = row.get(4)?;
            let primary_error_str: String = row.get(5)?;
            let step_error_str: Option<String> = row.get(6)?;
            let preferred_diff: i64 = row.get(7)?;
            let preferred_variant: Option<String> = row.get(8)?;
            let source_attempt_id_str: String = row.get(9)?;
            let urgency_str: String = row.get(10)?;
            let req_ack_int: i32 = row.get(11)?;
            let recurrence_count: i64 = row.get(12)?;
            let rationale: String = row.get(13)?;
            let created_at: i64 = row.get(14)?;

            let kind = match kind_str.as_str() {
                "procedural_variant" => RemediationActionKind::ProceduralVariant,
                "strategy_drill" => RemediationActionKind::StrategyDrill,
                "concept_check" => RemediationActionKind::ConceptCheck,
                "worked_example" => RemediationActionKind::WorkedExample,
                "prerequisite_review" => RemediationActionKind::PrerequisiteReview,
                "transfer_retry" => RemediationActionKind::TransferRetry,
                "circuit_breaker" => RemediationActionKind::CircuitBreaker,
                _ => RemediationActionKind::ProceduralVariant,
            };

            let domain: Domain = domain_str.parse().unwrap_or(Domain::Mathematics);
            let primary_error = match primary_error_str.as_str() {
                "concept" | "conceptual" => ErrorCategory::Concept,
                "strategy" => ErrorCategory::Strategy,
                "calculation" => ErrorCategory::Calculation,
                "careless" => ErrorCategory::Careless,
                "time" => ErrorCategory::Time,
                "unknown" => ErrorCategory::Unknown,
                other => ErrorCategory::DomainSpecific(other.to_string()),
            };

            let step_error: Option<StepErrorType> = step_error_str.and_then(|s| {
                serde_json::from_value(serde_json::Value::String(s)).ok()
            });

            let urgency = match urgency_str.as_str() {
                "critical" => RemediationUrgency::Critical,
                "normal" => RemediationUrgency::Normal,
                "advisory" => RemediationUrgency::Advisory,
                _ => RemediationUrgency::Normal,
            };

            Ok(RemediationAction {
                id,
                kind,
                skill_id: SkillId::new(skill_id_str),
                schema_id: SchemaId::new(schema_id_str),
                domain,
                primary_error,
                step_error,
                preferred_difficulty: preferred_diff as u32,
                preferred_variant,
                source_attempt_id: AttemptId::new(source_attempt_id_str),
                urgency,
                requires_acknowledgement: req_ack_int != 0,
                recurrence_count: recurrence_count as u32,
                rationale,
                created_at,
            })
        })?;

        let mut pending_actions = Vec::new();
        for a in rows {
            pending_actions.push(a?);
        }

        let mut recurrence_stmt = conn.prepare(
            "SELECT skill_id, error_category, count FROM remediation_recurrence",
        )?;

        let rec_rows = recurrence_stmt.query_map([], |row| {
            let skill_id_str: String = row.get(0)?;
            let cat_str: String = row.get(1)?;
            let count: i64 = row.get(2)?;

            let cat = match cat_str.as_str() {
                "concept" | "conceptual" => ErrorCategory::Concept,
                "strategy" => ErrorCategory::Strategy,
                "calculation" => ErrorCategory::Calculation,
                "careless" => ErrorCategory::Careless,
                "time" => ErrorCategory::Time,
                "unknown" => ErrorCategory::Unknown,
                other => ErrorCategory::DomainSpecific(other.to_string()),
            };

            Ok(((SkillId::new(skill_id_str), cat), count as u32))
        })?;

        let mut recurrence_tracker = HashMap::new();
        for r in rec_rows {
            let (k, v) = r?;
            recurrence_tracker.insert(k, v);
        }

        let mut queue = RemediationQueue {
            pending_actions,
            recurrence_tracker,
            max_loop_limit: 4,
        };
        queue.compact();

        Ok(queue)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_crud_flow() {
        let store = ProceduralStore::open_in_memory().unwrap();

        let skill = Skill::new(
            "math.algebra.monic_quad",
            Domain::Mathematics,
            "Monic Quadratics",
            "Factoring monic quadratic equations",
        );
        store.insert_skill(&skill).unwrap();

        let loaded_skill = store.get_skill(&skill.id).unwrap().unwrap();
        assert_eq!(loaded_skill.name, "Monic Quadratics");

        let family = ProblemFamily::new(
            "family.math.quad.monic",
            &skill.id,
            Domain::Mathematics,
            "Monic Factor Family",
            "template.monic.v1",
        );
        store.insert_problem_family(&family).unwrap();

        let schema = SchemaPracticeObject::new(
            "schema.math.quad.monic",
            &skill.id,
            &family.id,
            "Practice Monic Quadratic Factoring",
            "Factoring x^2 + bx + c",
        );
        store.insert_schema(&schema).unwrap();

        let instance = ProblemInstance::new(
            "inst.1",
            &family.id,
            12345,
            serde_json::json!({ "b": 5, "c": 6 }),
            "Factor x^2 + 5x + 6",
            serde_json::json!({ "r1": -2, "r2": -3 }),
        );
        store.insert_problem_instance(&instance).unwrap();

        let attempt = PracticeAttempt::new(
            "att.1",
            &instance.id,
            &schema.id,
            &skill.id,
            serde_json::json!({ "r1": -2, "r2": -3 }),
            true,
            1.0,
            3500,
        )
        .with_card_id(1001);
        store.insert_practice_attempt(&attempt).unwrap();

        let error = ErrorEvent::new(
            "err.1",
            &attempt.id,
            "none",
            serde_json::json!({}),
        );
        store.insert_error_event(&error).unwrap();

        let attempts = store.get_practice_attempts_by_card(1001).unwrap();
        assert_eq!(attempts.len(), 1);
        assert_eq!(attempts[0].id.as_str(), "att.1");
    }

    #[test]
    fn test_exam_and_pyq_store_flow() {
        let store = ProceduralStore::open_in_memory().unwrap();

        // 1. PYQ Source and Mapping
        let pyq = PYQSource::new(
            "pyq.rrb.2018.q1",
            "RRB ALP",
            2018,
            Domain::Mathematics,
            "Find time to cross platform",
            serde_json::json!({ "ans": 25 }),
            "RRB ALP 2018",
        );
        store.insert_pyq_source(&pyq).unwrap();

        let loaded_pyq = store.get_pyq_source(&pyq.id).unwrap().unwrap();
        assert_eq!(loaded_pyq.exam, "RRB ALP");

        let mapping = PyqMapping::new(
            &pyq.id,
            Domain::Mathematics,
            "arithmetic.time_speed_distance",
            "schema.math.arithmetic.time_speed_distance",
            "family.math.arithmetic.time_speed_distance",
            2,
            35_000,
        )
        .with_status(MappingStatus::Verified);
        store.insert_pyq_mapping(&mapping).unwrap();

        let eligible = store
            .list_eligible_pyqs_for_schema(&mapping.schema_id)
            .unwrap();
        assert_eq!(eligible.len(), 1);
        assert_eq!(eligible[0].0.id.as_str(), "pyq.rrb.2018.q1");

        // 2. Exam Profile
        let profile = ExamProfile::rrb_alp();
        store.insert_exam_profile(&profile).unwrap();

        let loaded_prof = store.get_exam_profile(&profile.id).unwrap().unwrap();
        assert_eq!(loaded_prof.name, "RRB Assistant Loco Pilot (ALP)");

        let profiles = store.list_exam_profiles().unwrap();
        assert_eq!(profiles.len(), 1);

        // 3. Rejected Variant Record
        let rej = RejectedVariantRecord {
            id: RejectedVariantId::new("rej_1"),
            source_pyq_id: Some(pyq.id.clone()),
            schema_id: mapping.schema_id.clone(),
            family_id: mapping.problem_family_id.clone(),
            seed: 999,
            variant_type: "trap".into(),
            failure_reason: "Division by zero constraint violation".into(),
            generated_instance_json: serde_json::json!({ "raw": "invalid" }),
            rejected_at: 1000,
        };
        store.insert_rejected_variant(&rej).unwrap();

        let rejs = store.get_rejected_variants(10).unwrap();
        assert_eq!(rejs.len(), 1);
        assert_eq!(rejs[0].id.as_str(), "rej_1");
    }
}
