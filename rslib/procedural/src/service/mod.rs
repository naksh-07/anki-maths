// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

use chrono::Utc;
use rand::Rng;

use crate::anchor::{ProceduralCardAnchor, SeedMode};
use crate::anchor::source::SourceQuestion;
use crate::core::{
    AttemptId, Domain, ErrorEventId, ExamProfileId, ProblemFamilyId, ProblemInstanceId,
    ProceduralError, PyqId, Result, SchemaId, SkillId,
};
use crate::diagnostics::{ErrorCategory, ProceduralReviewOutcome};
use crate::exam::{
    apply_diagnostic_report_to_store, ComprehensiveDiagnosticReport, ExamPracticeMode,
    ExamProfile, ExamSessionSelector, HumanReviewWorkflow, MockBlueprint, MockQuestionItem,
    MockSession, PyqMasteryAction, PyqMasteryBridge, PyqVariantPipeline, PYQSource, PyqMapping,
    ReviewAction, ReviewInspection,
};
use crate::practice::{ErrorEvent, PracticeAttempt, PracticeRequest, SchemaPracticeObject};
use crate::problems::catalog::{
    MathsCatalog, MATHS_CATALOG_VERSION, SCHEMA_ALGEBRAIC_IDENTITIES, SCHEMA_AVERAGE,
    SCHEMA_CHEMISTRY_EQUILIBRIUM, SCHEMA_CHEMISTRY_STOICHIOMETRY,
    SCHEMA_COMBINED_MULTI_CONCEPT, SCHEMA_DIVISIBILITY, SCHEMA_GEOMETRY_TRIANGLES,
    SCHEMA_LINEAR_EQUATIONS, SCHEMA_LINEAR_INEQUALITIES, SCHEMA_MIXTURES_ALLIGATION,
    SCHEMA_PHYSICS_KINEMATICS, SCHEMA_PHYSICS_WORK_ENERGY, SCHEMA_PROFIT_LOSS, SCHEMA_RATIO,
    SCHEMA_REASONING_BLOOD_RELATIONS, SCHEMA_REASONING_CODED_EXPRESSIONS,
    SCHEMA_REASONING_DATA_SUFFICIENCY, SCHEMA_REASONING_DIRECTION_SENSE,
    SCHEMA_REASONING_FLOOR_GRID, SCHEMA_REASONING_LOGIC_DAG, SCHEMA_REASONING_RELATIONS,
    SCHEMA_REASONING_SEATING, SCHEMA_REASONING_SERIES, SCHEMA_REASONING_SYLLOGISM,
    SCHEMA_REMAINDERS_MODULAR, SCHEMA_SUCCESSIVE_PERCENTAGE, SCHEMA_TIME_SPEED_DISTANCE,
    SCHEMA_TIME_WORK,
};
use crate::problems::registry::ProblemRegistry;
use crate::problems::validator::PercentageSuccessiveValidator;
use crate::problems::{ProblemFamily, ProblemInstance};
use crate::remediation::{
    RemediationAction, RemediationAuditLog, RemediationAuditRecord, RemediationContext,
    RemediationIntervention, RemediationOutcomeStatus, RemediationPolicy, RemediationQueue,
    RemediationSelector,
};
use crate::scheduling::unified::{LearningObjectKind, UnifiedPracticeEngine};
use crate::scheduling::{
    derive_fsrs_rating, PracticeMode, PracticeSessionObject, Rating, SelectionDecision,
    SessionReadiness, VariantSelector,
};
use crate::skills::prerequisites::{PrerequisiteEvaluation, PrerequisiteGraphService};
use crate::skills::signals::MasteryEvidence;
use crate::skills::{Skill, SkillState};
use crate::storage::ProceduralStore;

pub(crate) fn format_family_title(family_id: &str) -> String {
    let last_segment = family_id.split('.').last().unwrap_or(family_id);
    let words: Vec<String> = last_segment
        .split('_')
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect();
    words.join(" ")
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ReconciliationReport {
    pub new_count: usize,
    pub updated_count: usize,
    pub unchanged_count: usize,
    pub archived_count: usize,
    pub invalid_count: usize,
    pub diagnostics: Vec<String>,
}

/// High-level service facade providing the narrow integration boundary
/// between Anki and the procedural practice engine subsystem.
#[derive(Clone)]
pub struct ProceduralService {
    store: ProceduralStore,
    registry: ProblemRegistry,
    remediation_queue: Arc<Mutex<RemediationQueue>>,
    audit_log: Arc<Mutex<RemediationAuditLog>>,
    prerequisite_service: Arc<PrerequisiteGraphService>,
}

impl std::fmt::Debug for ProceduralService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ProceduralService")
    }
}

impl ProceduralService {
    pub fn new(store: ProceduralStore) -> Self {
        let prereq_service = Arc::new(PrerequisiteGraphService::new());
        let _ = prereq_service.sync_from_store(&store);
        let queue = store.load_remediation_queue().unwrap_or_default();
        Self {
            store,
            registry: ProblemRegistry::default_maths_registry(),
            remediation_queue: Arc::new(Mutex::new(queue)),
            audit_log: Arc::new(Mutex::new(RemediationAuditLog::new())),
            prerequisite_service: prereq_service,
        }
    }

    pub fn with_registry(store: ProceduralStore, registry: ProblemRegistry) -> Self {
        let prereq_service = Arc::new(PrerequisiteGraphService::new());
        let _ = prereq_service.sync_from_store(&store);
        let queue = store.load_remediation_queue().unwrap_or_default();
        Self {
            store,
            registry,
            remediation_queue: Arc::new(Mutex::new(queue)),
            audit_log: Arc::new(Mutex::new(RemediationAuditLog::new())),
            prerequisite_service: prereq_service,
        }
    }

    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let store = ProceduralStore::open(path)?;
        let service = Self::new(store);
        service.init_default_maths_catalog()?;
        Ok(service)
    }

    pub fn open_in_memory() -> Result<Self> {
        let store = ProceduralStore::open_in_memory()?;
        let service = Self::new(store);
        service.init_default_maths_catalog()?;
        Ok(service)
    }

    pub fn store(&self) -> &ProceduralStore {
        &self.store
    }

    pub fn registry(&self) -> &ProblemRegistry {
        &self.registry
    }

    /// Seed the store with built-in canonical Maths catalog definitions.
    pub fn init_default_maths_catalog(&self) -> Result<()> {
        MathsCatalog::init_all(&self.store)?;
        let _ = self.prerequisite_service.sync_from_store(&self.store);
        Ok(())
    }

    /// Ingests study material JSON (like LCM-HCF_ProblemPatterns.json) into the Practice Content Layer.
    pub fn ingest_practice_content(&self, json_content: &str) -> Result<()> {
        crate::content::ingestion::PracticeContentIngester::ingest_study_material_json(&self.store, json_content)
    }

    /// Ingests practice questions JSON (like LCM-HCF_PracticeQuestions.json) into the Practice Content Layer.
    pub fn ingest_practice_questions(&self, json_content: &str) -> Result<()> {
        crate::content::ingestion::PracticeContentIngester::ingest_practice_questions_json(&self.store, json_content)
    }

    /// Reconciles Anki static SourceNotes against the persistent PracticeItem database.
    pub fn reconcile_source_questions(
        &self,
        source_items: Vec<(String, SourceQuestion)>,
    ) -> Result<ReconciliationReport> {
        let mut report = ReconciliationReport::default();
        let mut active_ids = Vec::new();

        for (guid, source_question) in source_items {
            let item = source_question.clone().into_practice_item(&guid);
            active_ids.push(item.id.clone());

            let current_hash = self.store.get_practice_item_hash(&item.id)?;

            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            let origin_str = serde_json::to_string(&item.origin).unwrap_or_default();
            let qtype_str = serde_json::to_string(&item.question_type).unwrap_or_default();
            let meta_str = serde_json::to_string(&item.metadata).unwrap_or_default();
            let prov_str = serde_json::to_string(&item.provenance).unwrap_or_default();
            origin_str.hash(&mut hasher);
            qtype_str.hash(&mut hasher);
            item.prompt.hash(&mut hasher);
            item.difficulty.to_bits().hash(&mut hasher);
            prov_str.hash(&mut hasher);
            meta_str.hash(&mut hasher);
            let new_hash = hasher.finish();

            if let Some(existing) = current_hash {
                if existing == new_hash {
                    report.unchanged_count += 1;
                    continue; // Skip UPSERT if identical
                } else {
                    report.updated_count += 1;
                }
            } else {
                report.new_count += 1;
            }

            self.store.upsert_practice_item(&item)?;
        }

        report.archived_count = self.store.archive_missing_practice_items(&active_ids)?;
        Ok(report)
    }

    /// Access the prerequisite graph service.
    pub fn prerequisite_service(&self) -> &PrerequisiteGraphService {
        &self.prerequisite_service
    }

    /// Evaluate prerequisite readiness and dependency chains for a given skill.
    /// Performs localized batched lookups for required dependencies without N+1 full-catalog scans.
    pub fn evaluate_prerequisites(&self, skill_id: &SkillId) -> Result<PrerequisiteEvaluation> {
        let direct = self.prerequisite_service.get_direct_prerequisites(skill_id);
        let (transitive, _) = self.prerequisite_service.get_transitive_prerequisites(skill_id);
        if direct.is_empty() && transitive.is_empty() {
            return Ok(self.prerequisite_service.evaluate_readiness(skill_id, &HashMap::new()));
        }

        let mut needed = direct;
        for t in transitive {
            if !needed.contains(&t) {
                needed.push(t);
            }
        }
        let states = self.store.get_skill_states(&needed)?;
        Ok(self.prerequisite_service.evaluate_readiness(skill_id, &states))
    }

    /// Enqueue a structured remediation action into the priority queue and persist to store.
    pub fn enqueue_remediation_action(&self, action: RemediationAction) -> Result<()> {
        let mut q = self.remediation_queue.lock().unwrap();
        q.enqueue(action);
        self.store.save_remediation_queue(&q)?;
        Ok(())
    }

    /// Persist current remediation queue state to SQLite storage.
    pub fn persist_remediation_queue(&self) -> Result<()> {
        let q = self.remediation_queue.lock().unwrap();
        self.store.save_remediation_queue(&q)?;
        Ok(())
    }

    /// Select concrete executable remediation intervention from action.
    pub fn select_remediation_intervention(&self, action: &RemediationAction, seed: u64) -> Result<RemediationIntervention> {
        RemediationSelector::select_intervention(action, &self.store, &self.registry, seed)
    }

    /// Access the remediation queue Arc mutex.
    pub fn remediation_queue(&self) -> Arc<Mutex<RemediationQueue>> {
        Arc::clone(&self.remediation_queue)
    }

    /// Resolve a schema reference by its ID. Supports canonical ID, dynamic registry contracts, and standard aliases.
    pub fn resolve_schema(&self, schema_id: &SchemaId) -> Result<Option<SchemaPracticeObject>> {
        if let Some(schema) = self.store.get_schema(schema_id)? {
            return Ok(Some(schema));
        }

        // Check if dynamically registered in registry contracts
        if let Some(contract) = self.registry.get_family_contract(schema_id.as_str()) {
            let clean_title = format_family_title(contract.family_id.as_str());
            let schema_obj = SchemaPracticeObject::new(
                contract.default_schema.clone(),
                contract.skill_id.clone(),
                contract.family_id.clone(),
                clean_title.clone(),
                format!("Declarative schema for {}", clean_title),
            );
            return Ok(Some(schema_obj));
        }

        let alias = match schema_id.as_str() {
            "percentage.successive" | "math.percentage.successive" | "schema.math.percentage.successive" => {
                SCHEMA_SUCCESSIVE_PERCENTAGE
            }
            "algebra.linear_equations" | "math.algebra.linear_equations" | "schema.math.algebra.linear_equations" => {
                SCHEMA_LINEAR_EQUATIONS
            }
            "arithmetic.profit_loss" | "math.arithmetic.profit_loss" | "schema.math.arithmetic.profit_loss" => {
                SCHEMA_PROFIT_LOSS
            }
            "arithmetic.ratio" | "math.arithmetic.ratio" | "schema.math.arithmetic.ratio" => {
                SCHEMA_RATIO
            }
            "arithmetic.average" | "math.arithmetic.average" | "schema.math.arithmetic.average" => {
                SCHEMA_AVERAGE
            }
            "number_system.divisibility" | "math.number_system.divisibility" | "schema.math.number_system.divisibility" => {
                SCHEMA_DIVISIBILITY
            }
            "time_work.basic" | "math.time_work.basic" | "schema.math.time_work.basic" => {
                SCHEMA_TIME_WORK
            }
            "arithmetic.time_speed_distance" | "math.arithmetic.time_speed_distance" | "time_speed_distance" => {
                SCHEMA_TIME_SPEED_DISTANCE
            }
            "arithmetic.mixtures_alligation" | "math.arithmetic.mixtures_alligation" | "mixtures_alligation" => {
                SCHEMA_MIXTURES_ALLIGATION
            }
            "number_system.remainders_modular" | "math.number_system.remainders_modular" | "remainders_modular" => {
                SCHEMA_REMAINDERS_MODULAR
            }
            "algebra.linear_inequalities" | "math.algebra.linear_inequalities" | "linear_inequalities" => {
                SCHEMA_LINEAR_INEQUALITIES
            }
            "algebra.algebraic_identities" | "math.algebra.algebraic_identities" | "algebraic_identities" => {
                SCHEMA_ALGEBRAIC_IDENTITIES
            }
            "geometry.triangles" | "math.geometry.triangles" | "geometry_triangles" => {
                SCHEMA_GEOMETRY_TRIANGLES
            }
            "combined.multi_concept" | "math.combined.multi_concept" | "combined_multi_concept" => {
                SCHEMA_COMBINED_MULTI_CONCEPT
            }
            "physics.kinematics.1d" | "physics.kinematics" | "kinematics_1d" | "kinematics" => {
                SCHEMA_PHYSICS_KINEMATICS
            }
            "physics.work_energy.mechanics" | "physics.work_energy" | "work_energy_mechanics" | "work_energy" => {
                SCHEMA_PHYSICS_WORK_ENERGY
            }
            "chemistry.stoichiometry.moles" | "chemistry.stoichiometry" | "stoichiometry" | "stoichiometry_moles" => {
                SCHEMA_CHEMISTRY_STOICHIOMETRY
            }
            "chemistry.equilibrium.concentration" | "chemistry.equilibrium" | "equilibrium" | "chemical_equilibrium" => {
                SCHEMA_CHEMISTRY_EQUILIBRIUM
            }
            "reasoning.series.pattern_recognition" | "reasoning.series" | "series_patterns" | "series" => {
                SCHEMA_REASONING_SERIES
            }
            "reasoning.syllogism.formal_inference" | "reasoning.syllogism" | "syllogism_categorical" | "syllogism" => {
                SCHEMA_REASONING_SYLLOGISM
            }
            "reasoning.seating.constraint_satisfaction" | "reasoning.seating" | "seating_linear" | "seating" => {
                SCHEMA_REASONING_SEATING
            }
            "reasoning.relations.graph_inference" | "reasoning.relations" | "relations_graph" | "relations" => {
                SCHEMA_REASONING_RELATIONS
            }
            "reasoning.blood_relations.kinship_graph" | "reasoning.blood_relations" | "blood_relations" => {
                SCHEMA_REASONING_BLOOD_RELATIONS
            }
            "reasoning.direction_sense.spatial_orientation" | "reasoning.direction_sense" | "direction_sense" | "direction" => {
                SCHEMA_REASONING_DIRECTION_SENSE
            }
            "reasoning.floor_grid.spatial_csp" | "reasoning.floor_grid" | "floor_grid" => {
                SCHEMA_REASONING_FLOOR_GRID
            }
            "reasoning.logic_dag.multi_step_inference" | "reasoning.logic_dag" | "logic_dag" => {
                SCHEMA_REASONING_LOGIC_DAG
            }
            "reasoning.data_sufficiency.constraint_sufficiency" | "reasoning.data_sufficiency" | "data_sufficiency" => {
                SCHEMA_REASONING_DATA_SUFFICIENCY
            }
            "reasoning.coded_expressions.symbolic_operators" | "reasoning.coded_expressions" | "coded_expressions" => {
                SCHEMA_REASONING_CODED_EXPRESSIONS
            }
            _ => return Ok(None),
        };

        self.store.get_schema(&SchemaId::from(alias))
    }

    /// Load the learner's skill state for a given skill.
    pub fn load_skill_state(&self, skill_id: &SkillId) -> Result<Option<SkillState>> {
        self.store.get_skill_state(skill_id)
    }

    /// Register or update a discrete skill node.
    pub fn register_skill(&self, skill: Skill) -> Result<()> {
        self.store.insert_skill(&skill)?;
        self.prerequisite_service
            .register_skill_prerequisites(skill.id.clone(), skill.prerequisites);
        Ok(())
    }

    /// Register or update a problem generator family.
    pub fn register_problem_family(&self, family: ProblemFamily) -> Result<()> {
        self.store.insert_problem_family(&family)
    }

    /// Register or update a practice schema.
    pub fn register_schema(&self, schema: SchemaPracticeObject) -> Result<()> {
        self.store.insert_schema(&schema)
    }

    /// Save a generated problem instance to persistence.
    pub fn save_problem_instance(&self, instance: ProblemInstance) -> Result<()> {
        self.store.insert_problem_instance(&instance)
    }

    /// Retrieve a problem instance by ID.
    pub fn get_problem_instance(
        &self,
        instance_id: &ProblemInstanceId,
    ) -> Result<Option<ProblemInstance>> {
        self.store.get_problem_instance(instance_id)
    }

    /// Record a practice attempt and associated error events.
    pub fn record_practice_attempt(
        &self,
        attempt: PracticeAttempt,
        errors: Vec<ErrorEvent>,
    ) -> Result<()> {
        let variant = attempt
            .metadata
            .get("variant")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let target_latency_ms = attempt
            .metadata
            .get("target_time_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(45_000);

        self.record_practice_attempt_with_variant(
            attempt,
            errors,
            variant.as_deref(),
            target_latency_ms,
        )
    }

    /// Record a practice attempt with explicit variant and target latency overrides.
    /// Uses a single SQLite transaction boundary to read, compute, and persist state atomically.
    pub fn record_practice_attempt_with_variant(
        &self,
        attempt: PracticeAttempt,
        errors: Vec<ErrorEvent>,
        variant: Option<&str>,
        target_latency_ms: u64,
    ) -> Result<()> {
        let _state = self.store.record_practice_attempt_atomic(
            &attempt,
            &errors,
            variant,
            target_latency_ms,
        )?;
        Ok(())
    }

    /// Query attempts linked to a specific Anki Card.
    pub fn get_attempts_for_card(&self, card_id: i64) -> Result<Vec<PracticeAttempt>> {
        self.store.get_practice_attempts_by_card(card_id)
    }

    /// Query recent attempts for a specific practice schema.
    pub fn get_recent_attempts_for_schema(
        &self,
        schema_id: &SchemaId,
        limit: usize,
    ) -> Result<Vec<PracticeAttempt>> {
        self.store.get_practice_attempts_by_schema(schema_id, limit)
    }

    /// Generate a deterministic problem instance using the registered generator registry.
    pub fn generate_problem(
        &self,
        family_id: &ProblemFamilyId,
        seed: u64,
        custom_params: &serde_json::Value,
    ) -> Result<ProblemInstance> {
        let family = self
            .store
            .get_problem_family(family_id)?
            .ok_or_else(|| ProceduralError::NotFound(format!("Problem family not found: {}", family_id)))?;

        let difficulty = custom_params
            .get("difficulty")
            .or_else(|| custom_params.get("difficulty_level"))
            .and_then(|v| v.as_u64())
            .unwrap_or(2) as u32;

        let variant = custom_params
            .get("variant")
            .and_then(|v| v.as_str());

        self.registry.generate(family_id, &family.template_ref, seed, difficulty, variant)
    }

    /// Prepare a practice session object for a card anchor reference.
    pub fn prepare_practice_session(
        &self,
        anchor: &ProceduralCardAnchor,
        card_id: Option<i64>,
    ) -> Result<PracticeSessionObject> {
        let schema = self
            .resolve_schema(&anchor.proc_schema)?
            .ok_or_else(|| ProceduralError::NotFound(format!("Schema not found: {}", anchor.proc_schema)))?;

        let family = self
            .store
            .get_problem_family(&schema.problem_family_id)?
            .ok_or_else(|| {
                ProceduralError::NotFound(format!("Problem family not found: {}", schema.problem_family_id))
            })?;

        let skill_state = self.load_skill_state(&schema.skill_id)?;

        let seed = match anchor.seed_mode {
            SeedMode::Random => rand::rng().random::<u64>(),
            SeedMode::Fixed(s) => s,
            SeedMode::Daily => {
                let today_days = Utc::now().timestamp() / 86400;
                today_days as u64
            }
        };

        // Determine difficulty and variant
        let selection_decision = VariantSelector::select_variant(skill_state.as_ref(), None, seed);

        // Generate problem instance via registry
        let instance = self.registry.generate(
            &schema.problem_family_id,
            &family.template_ref,
            seed,
            2,
            Some(selection_decision.variant.as_str()),
        )?;

        self.store.insert_problem_instance(&instance)?;

        let mut session = PracticeSessionObject::new(schema, instance, card_id, skill_state);
        session.readiness = SessionReadiness::Ready;
        session = session.with_selection_decision(&selection_decision);
        Ok(session)
    }

    /// Resolves a source target directly from persistent practice_items.
    /// This is the primary Phase 4 source-first playback path.
    pub fn resolve_source_target(
        &self,
        guid: &str,
        card_id: Option<i64>,
    ) -> Result<PracticeSessionObject> {
        let item_id = SourceQuestion::stable_id_from_guid(guid);
        let item = self.store.get_practice_item(&item_id)?
            .ok_or_else(|| ProceduralError::NotFound(format!("Source question not found in database: {}. Please reconcile first.", guid)))?;

        let instance = item.clone().into_problem_instance();
        
        let topic = item.metadata.get("topic").and_then(|v| v.as_str()).unwrap_or("General").to_string();
        let schema_title = format!("Static Source Schema for {}", topic);

        let schema = SchemaPracticeObject::new(
            item.schema_id.clone(),
            item.skill_id.clone(),
            item.problem_family_id.clone(),
            topic,
            schema_title,
        );
        
        let mut session = PracticeSessionObject::new(schema, instance, card_id, None);
        session.readiness = SessionReadiness::Ready;
        session.selection_reason = Some("StudyLab Static Source Rendering from DB".to_string());
        session.difficulty_level = Some(item.difficulty as u32);
        
        Ok(session)
    }

    /// Resolves a procedural anchor directly into a hydrated PracticeSessionObject.
    /// In Phase 35, this supports the complete contract source precedence:
    /// 1. Modern Rich APKG Path: `inline_contract` carries complete declarative blueprint
    /// 2. Hydrated Content Ref Path: `content_ref` loads static StudyLab PracticeItem
    /// 3. Legacy Catalog Fallback Path: `proc_schema` dispatches unified practice engine
    pub fn resolve_procedural_target(
        &self,
        anchor: &ProceduralCardAnchor,
        card_id: Option<i64>,
    ) -> Result<PracticeSessionObject> {
        // 1. Top Precedence: Modern Rich APKG Inline Contract
        if let Some(inline_contract) = &anchor.inline_contract {
            inline_contract.validate()?;

            let family_id = inline_contract.contract.family_id.clone();
            let template_ref = format!("{}.declarative.v1", family_id.as_str());

            let mut reg = self.registry.clone();
            reg.register_declarative_family(inline_contract.clone());

            let seed = match anchor.seed_mode {
                SeedMode::Random => rand::rng().random::<u64>(),
                SeedMode::Fixed(s) => s,
                SeedMode::Daily => (Utc::now().timestamp() / 86400) as u64,
            };

            let difficulty_level = anchor.difficulty_override.unwrap_or(2.0).clamp(1.0, 5.0) as u32;

            let instance = reg.generate(
                &family_id,
                &template_ref,
                seed,
                difficulty_level,
                None,
            )?;

            let clean_title = format_family_title(family_id.as_str());
            let schema = SchemaPracticeObject::new(
                inline_contract.contract.default_schema.clone(),
                inline_contract.contract.skill_id.clone(),
                inline_contract.contract.family_id.clone(),
                clean_title.clone(),
                format!("Declarative schema for {}", clean_title),
            );

            let skill_state = self.load_skill_state(&schema.skill_id)?;
            let target_time = inline_contract.contract.target_latency(difficulty_level);

            let mut instance_clone = instance.clone();
            if let Some(meta_obj) = instance_clone.metadata.as_object_mut() {
                if !meta_obj.contains_key("provenance") {
                    if let Ok(prov_val) = serde_json::to_value(&inline_contract.contract.provenance) {
                        meta_obj.insert("provenance".to_string(), prov_val);
                    }
                }
                if !meta_obj.contains_key("domain") {
                    meta_obj.insert("domain".to_string(), serde_json::Value::String(inline_contract.contract.domain.as_str().to_string()));
                }
            }

            let mut session = PracticeSessionObject::new(schema, instance_clone.clone(), card_id, skill_state);
            session.readiness = SessionReadiness::Ready;
            session.target_latency_ms = Some(target_time);
            session.selection_reason = Some("Inline Declarative Contract".to_string());
            session.difficulty_level = Some(difficulty_level);
            if let Some(v) = instance_clone.metadata.get("variant").and_then(|v| v.as_str()) {
                session.selected_variant = Some(v.to_string());
            }

            let _ = self.store.insert_problem_instance(&instance_clone);

            return Ok(session);
        }

        // 2. Second Precedence: Hydrated Content Ref Path
        if let Some(content_ref) = &anchor.content_ref {
            let item_id = crate::core::PracticeItemId::new(content_ref);
            let item = self.store.get_practice_item(&item_id)?
                .ok_or_else(|| ProceduralError::NotFound(format!("Content Reference '{}' not found. Please sync/import the associated StudyLab content.", content_ref)))?;
            
            let family = self.store.get_problem_family(&item.problem_family_id)?
                .ok_or_else(|| ProceduralError::NotFound(format!("Problem family not found: {}", item.problem_family_id)))?;
                
            let seed = item.provenance.seed.unwrap_or(0);
            let difficulty_level = anchor.difficulty_override.unwrap_or(2.0).clamp(1.0, 5.0) as u32;
            
            let instance = self.registry.generate(
                &family.id,
                &family.template_ref,
                seed,
                difficulty_level,
                Some(&item.provenance.variant_type),
            )?;
            
            self.store.insert_problem_instance(&instance)?;
            
            let schema = self.resolve_schema(&item.schema_id)?
                .ok_or_else(|| ProceduralError::NotFound(format!("Schema not found: {}", item.schema_id)))?;
                
            let skill_state = self.load_skill_state(&schema.skill_id)?;
            let target_time = self.registry.get_family_contract(family.id.as_str())
                .map(|c| c.target_latency(difficulty_level))
                .unwrap_or(45_000);
            
            let mut session = PracticeSessionObject::new(schema, instance.clone(), card_id, skill_state);
            session.readiness = SessionReadiness::Ready;
            session.selected_variant = Some(item.provenance.variant_type.clone());
            session.target_latency_ms = Some(target_time);
            session.selection_reason = Some("StudyLab Hydration".to_string());
            session.difficulty_level = Some(difficulty_level);
            
            return Ok(session);
        }

        // 3. Third Precedence: Legacy Fallback Branch
        let resolved_schema_id = self.resolve_schema(&anchor.proc_schema)?
            .map(|s| s.id)
            .unwrap_or_else(|| anchor.proc_schema.clone());

        let request = PracticeRequest::new(
            crate::practice::PracticeScope::SingleSchema(resolved_schema_id),
            crate::practice::PracticeObjective::Practice,
        ).with_remediation_policy(crate::practice::RemediationPrecedence::AllEligible);

        let seed = match anchor.seed_mode {
            SeedMode::Random => rand::rng().random::<u64>(),
            SeedMode::Fixed(s) => s,
            SeedMode::Daily => (Utc::now().timestamp() / 86400) as u64,
        };

        let mut session = self.prepare_unified_practice_session(&request, None, None, Some(seed))?;
        session.card_id = card_id;
        Ok(session)
    }

    /// Prepare a unified practice session driven by a composable `PracticeRequest`.
    pub fn prepare_unified_practice_session(
        &self,
        request: &PracticeRequest,
        candidate_schema_ids: Option<&[SchemaId]>,
        last_schema_id: Option<&SchemaId>,
        seed_override: Option<u64>,
    ) -> Result<PracticeSessionObject> {
        let all_schemas = if let Some(ids) = candidate_schema_ids {
            let mut list = Vec::new();
            for id in ids {
                if let Some(s) = self.resolve_schema(id)? {
                    list.push(s);
                }
            }
            list
        } else {
            self.store.list_all_schemas()?
        };

        if all_schemas.is_empty() {
            return Err(ProceduralError::NotFound("No practice schemas available in catalog".to_string()));
        }

        let mut schema_domains = HashMap::new();
        let mut eligible_pyqs = HashMap::new();
        let all_eligible_map = self.store.list_all_eligible_pyqs_map()?;

        for s in &all_schemas {
            let domain = if s.id.as_str().contains("physics") {
                Domain::Physics
            } else if s.id.as_str().contains("chem") {
                Domain::Chemistry
            } else if s.id.as_str().contains("reasoning") {
                Domain::Reasoning
            } else {
                Domain::Mathematics
            };
            schema_domains.insert(s.id.clone(), domain);

            if let Some(pyq_list) = all_eligible_map.get(&s.id) {
                if !pyq_list.is_empty() {
                    let pyq_sources = pyq_list.iter().map(|(source, _mapping)| source.clone()).collect();
                    eligible_pyqs.insert(s.id.clone(), pyq_sources);
                }
            }
        }

        let skill_states = self.store.get_all_skill_states()?;

        let exam_profile = if let Some(ref profile_id) = request.exam_profile {
            self.store.get_exam_profile(profile_id)?.or_else(|| {
                match profile_id.as_str() {
                    "rrb_alp" => Some(ExamProfile::rrb_alp()),
                    "ssc_cgl" => Some(ExamProfile::ssc_cgl()),
                    "banking_po" => Some(ExamProfile::banking_po()),
                    "jee_main_foundation" => Some(ExamProfile::jee_main_foundation()),
                    _ => None,
                }
            })
        } else {
            None
        };

        let seed = seed_override.unwrap_or_else(|| rand::rng().random::<u64>());

        let mut queue_lock = self.remediation_queue.lock().unwrap();
        let decision = UnifiedPracticeEngine::select_next(
            request,
            &all_schemas,
            &schema_domains,
            &skill_states,
            &self.prerequisite_service,
            Some(&mut *queue_lock),
            exam_profile.as_ref(),
            &eligible_pyqs,
            last_schema_id,
            &self.registry,
            &self.store,
            seed,
        )
        .ok_or_else(|| ProceduralError::NotFound("No eligible learning object selected for practice request".to_string()))?;

        let _ = self.store.save_remediation_queue(&queue_lock);
        drop(queue_lock);

        // If a generated problem was produced, persist it to the store
        match &decision.learning_object {
            LearningObjectKind::ProceduralProblem(p) | LearningObjectKind::PyqVariant(p) => {
                let _ = self.store.insert_problem_instance(p);
            }
            LearningObjectKind::Remediation(RemediationIntervention::PrerequisiteReview(prereq_obj)) => {
                if let Some(ref p) = prereq_obj.executable_problem {
                    let _ = self.store.insert_problem_instance(p);
                }
            }
            _ => {}
        }

        let state = skill_states.get(&decision.skill_id).cloned();
        Ok(decision.into_practice_session(None, state))
    }

    /// Prepare a multi-schema practice session from practice mode and candidate schemas (backward compatible).
    pub fn prepare_multi_schema_session(
        &self,
        mode: &PracticeMode,
        candidate_schema_ids: Option<&[SchemaId]>,
        last_schema_id: Option<&SchemaId>,
        seed_override: Option<u64>,
    ) -> Result<PracticeSessionObject> {
        let request = PracticeRequest::from_legacy_mode(mode);
        self.prepare_unified_practice_session(&request, candidate_schema_ids, last_schema_id, seed_override)
    }

    /// Select next problem variant based on current skill state and seed.
    pub fn select_next_variant(
        &self,
        skill_id: &SkillId,
        seed: u64,
    ) -> Result<SelectionDecision> {
        let skill_state = self.load_skill_state(skill_id)?;
        Ok(VariantSelector::select_variant(skill_state.as_ref(), None, seed))
    }

    /// Deterministically evaluate a student's answer, persist attempt telemetry and error events,
    /// update learner SkillState with moving-window learning signals, and produce a `ProceduralReviewOutcome`.
    pub fn evaluate_and_record_attempt(
        &self,
        instance_id: &ProblemInstanceId,
        card_id: Option<i64>,
        student_answer: serde_json::Value,
        time_taken_ms: u64,
        hints_used: u32,
        attempt_count: u32,
    ) -> Result<ProceduralReviewOutcome> {
        let (instance, family_id, skill_id, schema_id) = if instance_id.as_str().starts_with("inst-pi-") {
            let practice_item_id = crate::core::PracticeItemId::new(instance_id.as_str().replace("inst-pi-", ""));
            let item = self
                .store
                .get_practice_item(&practice_item_id)?
                .ok_or_else(|| ProceduralError::NotFound(format!("Practice item not found: {}", practice_item_id)))?;
            let f_id = item.problem_family_id.clone();
            let s_id = item.skill_id.clone();
            let sch_id = item.schema_id.clone();
            
            let _ = self.store.insert_skill(&crate::skills::Skill {
                id: s_id.clone(),
                domain: item.domain.clone(),
                name: item.chapter.clone(),
                description: format!("Synthetic skill for {}", item.chapter),
                prerequisites: vec![],
                metadata: serde_json::json!({ "synthetic": true }),
                created_at: chrono::Utc::now().timestamp(),
            });

            let _ = self.store.insert_problem_family(&crate::problems::ProblemFamily {
                id: f_id.clone(),
                skill_id: s_id.clone(),
                domain: item.domain.clone(),
                name: item.chapter.clone(),
                template_ref: "static_source".to_string(),
                min_difficulty: item.difficulty,
                max_difficulty: item.difficulty,
                parameters_schema: serde_json::json!({}),
                metadata: serde_json::json!({ "synthetic": true }),
                created_at: chrono::Utc::now().timestamp(),
            });

            let _ = self.store.insert_schema(&crate::practice::SchemaPracticeObject::new(
                sch_id.clone(),
                s_id.clone(),
                f_id.clone(),
                item.chapter.clone(),
                "Static Source Schema".to_string(),
            ));

            let instance = item.clone().into_problem_instance();
            let _ = self.store.insert_problem_instance(&instance);

            (instance, f_id, s_id, sch_id)
        } else {
            let instance = self
                .store
                .get_problem_instance(instance_id)?
                .ok_or_else(|| ProceduralError::NotFound(format!("Problem instance not found: {}", instance_id)))?;
            let family = self
                .store
                .get_problem_family(&instance.family_id)?
                .ok_or_else(|| ProceduralError::NotFound(format!("Problem family not found: {}", instance.family_id)))?;
            let schema_id = self
                .store
                .get_schema_by_family(&family.id)?
                .map(|s| s.id)
                .unwrap_or_else(|| SchemaId::new(format!("schema.{}", family.id.as_str())));
            (instance, family.id, family.skill_id, schema_id)
        };

        let target_time_ms = instance
            .metadata
            .get("target_time_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(45_000);

        let variant_str = instance
            .parameters
            .get("variant")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Dispatch to registered validator or fallback
        let eval = if let Some(validator) = self.registry.get_validator(family_id.as_str()) {
            validator.evaluate(&instance, &student_answer, time_taken_ms, target_time_ms)
        } else {
            // Support simple string-based matching for static source items (MCQ/Reference)
            if let Some(correct_opt) = instance.correct_answer.get("correct_option").and_then(|v| v.as_str()) {
                let student_val_str = student_answer.get("value").and_then(|v| v.as_str()).unwrap_or("");
                let is_correct = student_val_str == correct_opt;
                crate::problems::validator::AnswerEvaluation {
                    is_correct,
                    score: if is_correct { 1.0 } else { 0.0 },
                    parsed_student_value: None,
                    canonical_value: 0.0,
                    error_category: if is_correct { None } else { Some(crate::diagnostics::ErrorCategory::Unknown) },
                    diagnostic_message: None,
                }
            } else if let Some(correct_ans) = instance.correct_answer.get("answer").and_then(|v| v.as_str()) {
                let student_val_str = student_answer.get("value").and_then(|v| v.as_str()).unwrap_or("");
                let is_correct = student_val_str == correct_ans;
                crate::problems::validator::AnswerEvaluation {
                    is_correct,
                    score: if is_correct { 1.0 } else { 0.0 },
                    parsed_student_value: None,
                    canonical_value: 0.0,
                    error_category: if is_correct { None } else { Some(crate::diagnostics::ErrorCategory::Unknown) },
                    diagnostic_message: None,
                }
            } else {
                PercentageSuccessiveValidator::evaluate(
                    &instance.correct_answer,
                    &instance.parameters,
                    &student_answer,
                    time_taken_ms,
                    target_time_ms,
                )
            }
        };

        let attempt_id = AttemptId::new(format!(
            "att-{}",
            Utc::now().timestamp_nanos_opt().unwrap_or(Utc::now().timestamp())
        ));

        let metadata = serde_json::json!({
            "hints_used": hints_used,
            "attempt_count": attempt_count,
            "target_time_ms": target_time_ms,
            "parsed_student_value": eval.parsed_student_value,
            "canonical_value": eval.canonical_value,
            "diagnostic_message": eval.diagnostic_message,
            "variant": variant_str,
            "error_category": eval.error_category,
            "catalog_version": MATHS_CATALOG_VERSION,
        });

        let mut attempt = PracticeAttempt::new(
            &attempt_id,
            &instance.id,
            &schema_id,
            &skill_id,
            student_answer,
            eval.is_correct,
            eval.score,
            time_taken_ms,
        )
        .with_metadata(metadata);

        if let Some(cid) = card_id {
            attempt = attempt.with_card_id(cid);
        }

        let mut error_events = Vec::new();
        if let Some(ref cat) = eval.error_category {
            let error_id = ErrorEventId::new(format!(
                "err-{}",
                Utc::now().timestamp_nanos_opt().unwrap_or(Utc::now().timestamp())
            ));
            let details = serde_json::json!({
                "diagnostic_message": eval.diagnostic_message,
                "parsed_student_value": eval.parsed_student_value,
                "canonical_value": eval.canonical_value,
            });
            error_events.push(ErrorEvent::new(error_id, &attempt.id, cat.as_str(), details));
        }

        self.record_practice_attempt(attempt, error_events)?;

        let mut outcome = ProceduralReviewOutcome::new(
            attempt_id,
            schema_id,
            skill_id,
            instance.family_id,
            instance.seed,
            eval.is_correct,
            eval.score,
            time_taken_ms,
            target_time_ms,
            hints_used,
            attempt_count,
            eval.error_category,
        );
        outcome.diagnostic_message = eval.diagnostic_message;
        Ok(outcome)
    }

    /// Step-aware evaluation of a student's attempt across a problem instance's solution graph.
    /// Localizes first error, persists step telemetry and error events, updates SkillState,
    /// and derives diagnostic and remediation recommendations.
    pub fn evaluate_stepwise_attempt(
        &self,
        instance_id: &ProblemInstanceId,
        card_id: Option<i64>,
        submission: &crate::problems::steps::StepwiseSubmission,
    ) -> Result<ProceduralReviewOutcome> {
        let (instance, family_id, skill_id, schema_id) = if instance_id.as_str().starts_with("inst-pi-") {
            let practice_item_id = crate::core::PracticeItemId::new(instance_id.as_str().replace("inst-pi-", ""));
            let item = self
                .store
                .get_practice_item(&practice_item_id)?
                .ok_or_else(|| ProceduralError::NotFound(format!("Practice item not found: {}", practice_item_id)))?;
            let f_id = item.problem_family_id.clone();
            let s_id = item.skill_id.clone();
            let sch_id = item.schema_id.clone();

            let _ = self.store.insert_skill(&crate::skills::Skill {
                id: s_id.clone(),
                domain: item.domain.clone(),
                name: item.chapter.clone(),
                description: format!("Synthetic skill for {}", item.chapter),
                prerequisites: vec![],
                metadata: serde_json::json!({ "synthetic": true }),
                created_at: chrono::Utc::now().timestamp(),
            });

            let _ = self.store.insert_problem_family(&crate::problems::ProblemFamily {
                id: f_id.clone(),
                skill_id: s_id.clone(),
                domain: item.domain.clone(),
                name: item.chapter.clone(),
                template_ref: "static_source".to_string(),
                min_difficulty: item.difficulty,
                max_difficulty: item.difficulty,
                parameters_schema: serde_json::json!({}),
                metadata: serde_json::json!({ "synthetic": true }),
                created_at: chrono::Utc::now().timestamp(),
            });

            let _ = self.store.insert_schema(&crate::practice::SchemaPracticeObject::new(
                sch_id.clone(),
                s_id.clone(),
                f_id.clone(),
                item.chapter.clone(),
                "Static Source Schema".to_string(),
            ));

            let instance = item.clone().into_problem_instance();
            let _ = self.store.insert_problem_instance(&instance);

            (instance, f_id, s_id, sch_id)
        } else {
            let instance = self
                .store
                .get_problem_instance(instance_id)?
                .ok_or_else(|| ProceduralError::NotFound(format!("Problem instance not found: {}", instance_id)))?;
            let family = self
                .store
                .get_problem_family(&instance.family_id)?
                .ok_or_else(|| ProceduralError::NotFound(format!("Problem family not found: {}", instance.family_id)))?;
            let schema_id = self
                .store
                .get_schema_by_family(&family.id)?
                .map(|s| s.id)
                .unwrap_or_else(|| SchemaId::new(format!("schema.{}", family.id.as_str())));
            (instance, family.id, family.skill_id, schema_id)
        };

        let target_time_ms = instance
            .metadata
            .get("target_time_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(45_000);

        let variant_str = instance
            .parameters
            .get("variant")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let step_eval = if let Some(validator) = self.registry.get_validator(family_id.as_str()) {
            validator.evaluate_stepwise(&instance, submission, target_time_ms)
        } else if let Some(graph) = instance.solution_graph() {
            crate::problems::steps::StepValidator::evaluate_submission(&graph, submission, target_time_ms)
        } else {
            let ans_json = submission
                .final_answer
                .as_ref()
                .map(|s| serde_json::json!(s))
                .unwrap_or(serde_json::Value::Null);

            let ans_eval = if let Some(correct_opt) = instance.correct_answer.get("correct_option").and_then(|v| v.as_str()) {
                let student_val_str = ans_json.get("value").or_else(|| ans_json.get("answer")).and_then(|v| v.as_str()).unwrap_or("");
                let is_correct = student_val_str == correct_opt;
                crate::problems::validator::AnswerEvaluation {
                    is_correct,
                    score: if is_correct { 1.0 } else { 0.0 },
                    parsed_student_value: None,
                    canonical_value: 0.0,
                    error_category: if is_correct { None } else { Some(crate::diagnostics::ErrorCategory::Unknown) },
                    diagnostic_message: None,
                }
            } else if let Some(correct_ans) = instance.correct_answer.get("answer").and_then(|v| v.as_str()) {
                let student_val_str = ans_json.get("value").or_else(|| ans_json.get("answer")).and_then(|v| v.as_str()).unwrap_or("");
                let is_correct = student_val_str == correct_ans;
                crate::problems::validator::AnswerEvaluation {
                    is_correct,
                    score: if is_correct { 1.0 } else { 0.0 },
                    parsed_student_value: None,
                    canonical_value: 0.0,
                    error_category: if is_correct { None } else { Some(crate::diagnostics::ErrorCategory::Unknown) },
                    diagnostic_message: None,
                }
            } else {
                PercentageSuccessiveValidator::evaluate(
                    &instance.correct_answer,
                    &instance.parameters,
                    &ans_json,
                    submission.total_time_ms,
                    target_time_ms,
                )
            };
            crate::problems::steps::StepGraphEvaluation {
                is_correct: ans_eval.is_correct,
                score: ans_eval.score,
                first_error_step: None,
                first_error_type: None,
                confidence: crate::problems::steps::DiagnosticConfidence::Deterministic,
                steps_completed: submission.steps.len(),
                steps_correct: if ans_eval.is_correct { 1 } else { 0 },
                step_evaluations: Vec::new(),
                overall_feedback: ans_eval.diagnostic_message.unwrap_or_default(),
                remediation_recommendation: None,
                first_action_latency_ms: submission.first_action_latency_ms,
                step_latencies_ms: submission.steps.iter().map(|s| s.time_taken_ms).collect(),
            }
        };

        let attempt_id = AttemptId::new(format!(
            "att-{}",
            Utc::now().timestamp_nanos_opt().unwrap_or(Utc::now().timestamp())
        ));

        let error_category = step_eval.first_error_type.map(|e| match e {
            crate::problems::steps::StepErrorType::FormulaSelectionError => ErrorCategory::Concept,
            crate::problems::steps::StepErrorType::SetupError => ErrorCategory::Strategy,
            crate::problems::steps::StepErrorType::TransformationError => ErrorCategory::Strategy,
            crate::problems::steps::StepErrorType::ArithmeticError => ErrorCategory::Calculation,
            crate::problems::steps::StepErrorType::SignError => ErrorCategory::Sign,
            crate::problems::steps::StepErrorType::PrematureCompletion => ErrorCategory::Careless,
            crate::problems::steps::StepErrorType::UnitError => ErrorCategory::Unit,
            crate::problems::steps::StepErrorType::FinalAnswerFormattingError => ErrorCategory::Syntax,
            crate::problems::steps::StepErrorType::RatioInversionError => ErrorCategory::Concept,
            crate::problems::steps::StepErrorType::AlligationSwapError => ErrorCategory::Strategy,
            crate::problems::steps::StepErrorType::RateInversionError => ErrorCategory::Concept,
            crate::problems::steps::StepErrorType::InequalitySignFlipError => ErrorCategory::Sign,
            crate::problems::steps::StepErrorType::IdentityCrossTermError => ErrorCategory::Concept,
            crate::problems::steps::StepErrorType::PythagoreanLegConfusion => ErrorCategory::Concept,
            crate::problems::steps::StepErrorType::ModularReductionError => ErrorCategory::Calculation,
            crate::problems::steps::StepErrorType::ModelSelectionError => ErrorCategory::Strategy,
            crate::problems::steps::StepErrorType::RepresentationError => ErrorCategory::Concept,
            crate::problems::steps::StepErrorType::EquationSetupError => ErrorCategory::Concept,
            crate::problems::steps::StepErrorType::SignConventionError => ErrorCategory::Sign,
            crate::problems::steps::StepErrorType::AlgebraExecutionError => ErrorCategory::Calculation,
            crate::problems::steps::StepErrorType::PhysicalPlausibilityError => ErrorCategory::Concept,
            crate::problems::steps::StepErrorType::ChemicalRepresentationError => ErrorCategory::Concept,
            crate::problems::steps::StepErrorType::EquationBalanceError => ErrorCategory::Strategy,
            crate::problems::steps::StepErrorType::StoichiometricRatioError => ErrorCategory::Strategy,
            crate::problems::steps::StepErrorType::LimitingReagentError => ErrorCategory::Strategy,
            crate::problems::steps::StepErrorType::RegimeSelectionError => ErrorCategory::Concept,
            crate::problems::steps::StepErrorType::ConservationViolationError => ErrorCategory::Concept,
            crate::problems::steps::StepErrorType::SchemaRecognitionError => ErrorCategory::Concept,
            crate::problems::steps::StepErrorType::StrategySelectionError => ErrorCategory::Strategy,
            crate::problems::steps::StepErrorType::ConstraintApplicationError => ErrorCategory::Strategy,
            crate::problems::steps::StepErrorType::InferenceError => ErrorCategory::Concept,
            crate::problems::steps::StepErrorType::SearchCaseError => ErrorCategory::Strategy,
            crate::problems::steps::StepErrorType::ContradictionHandlingError => ErrorCategory::Strategy,
            crate::problems::steps::StepErrorType::ReadingTrapError => ErrorCategory::Careless,
            crate::problems::steps::StepErrorType::ExecutionSlipError => ErrorCategory::Calculation,
            crate::problems::steps::StepErrorType::Unknown => ErrorCategory::Unknown,
        });

        let domain_ev_opt = step_eval.to_domain_evidence(family_id.as_str());

        let mut metadata = serde_json::json!({
            "hints_used": submission.hints_used,
            "attempt_count": 1,
            "target_time_ms": target_time_ms,
            "diagnostic_message": step_eval.overall_feedback,
            "variant": variant_str,
            "error_category": error_category,
            "catalog_version": MATHS_CATALOG_VERSION,
            "first_error_step": step_eval.first_error_step,
            "first_error_type": step_eval.first_error_type,
            "diagnostic_confidence": step_eval.confidence,
            "remediation_recommendation": step_eval.remediation_recommendation,
            "step_evaluations": step_eval.step_evaluations,
            "first_action_latency_ms": step_eval.first_action_latency_ms,
            "step_latencies_ms": step_eval.step_latencies_ms,
        });

        if let Some(dev) = domain_ev_opt {
            if let Ok(dev_json) = serde_json::to_value(dev) {
                metadata["domain_evidence"] = dev_json;
            }
        }

        let student_val = serde_json::json!({
            "mode": submission.mode,
            "steps": submission.steps,
            "final_answer": submission.final_answer,
        });

        let mut attempt = PracticeAttempt::new(
            &attempt_id,
            &instance.id,
            &schema_id,
            &skill_id,
            student_val,
            step_eval.is_correct,
            step_eval.score,
            submission.total_time_ms,
        )
        .with_metadata(metadata);

        if let Some(cid) = card_id {
            attempt = attempt.with_card_id(cid);
        }

        let mut error_events = Vec::new();
        if let Some(ref cat) = error_category {
            let error_id = ErrorEventId::new(format!(
                "err-{}",
                Utc::now().timestamp_nanos_opt().unwrap_or(Utc::now().timestamp())
            ));
            let details = serde_json::json!({
                "first_error_step": step_eval.first_error_step,
                "first_error_type": step_eval.first_error_type,
                "diagnostic_confidence": step_eval.confidence,
                "feedback": step_eval.overall_feedback,
            });
            error_events.push(ErrorEvent::new(error_id, &attempt.id, cat.as_str(), details));
        }

        self.record_practice_attempt(attempt, error_events)?;

        let mut outcome = ProceduralReviewOutcome::new(
            attempt_id,
            schema_id,
            skill_id,
            instance.family_id,
            instance.seed,
            step_eval.is_correct,
            step_eval.score,
            submission.total_time_ms,
            target_time_ms,
            submission.hints_used,
            1,
            error_category,
        );
        outcome.diagnostic_message = Some(step_eval.overall_feedback);
        outcome = outcome.with_step_diagnostics(
            step_eval.first_error_step,
            step_eval.steps_completed,
            step_eval.steps_correct,
            step_eval.step_latencies_ms,
            step_eval.first_action_latency_ms,
            Some(step_eval.confidence.as_str().to_string()),
            step_eval.remediation_recommendation,
        );

        Ok(outcome)
    }

    /// Derive calibrated FSRS rating from a procedural review outcome and current skill state.
    pub fn derive_fsrs_rating(&self, outcome: &ProceduralReviewOutcome) -> Result<Rating> {
        let state = self.load_skill_state(&outcome.skill_id)?;
        Ok(derive_fsrs_rating(outcome, state.as_ref()))
    }

    // =========================================================================
    // EXECUTABLE REMEDIATION ENGINE (R2)
    // =========================================================================

    /// Evaluates an attempt, records telemetry, and if failed, computes a typed RemediationAction,
    /// enqueues it in the RemediationQueue, and logs an audit trail event.
    pub fn evaluate_and_remediate_attempt(
        &self,
        instance_id: &ProblemInstanceId,
        card_id: Option<i64>,
        student_answer: serde_json::Value,
        time_taken_ms: u64,
        hints_used: u32,
        attempt_count: u32,
    ) -> Result<(ProceduralReviewOutcome, Option<RemediationAction>)> {
        let outcome = self.evaluate_and_record_attempt(
            instance_id,
            card_id,
            student_answer,
            time_taken_ms,
            hints_used,
            attempt_count,
        )?;

        if !outcome.is_correct {
            let skill_state = self
                .load_skill_state(&outcome.skill_id)?
                .unwrap_or_else(|| SkillState::new(outcome.skill_id.clone()));

            let err_cat = outcome
                .error_category
                .clone()
                .unwrap_or(ErrorCategory::Unknown);

            let queue_lock = self.remediation_queue.lock().unwrap();
            let recurrence = queue_lock.get_recurrence_count(&outcome.skill_id, &err_cat) + 1;
            drop(queue_lock);

            let schema = self.store.get_schema(&outcome.schema_id)?;
            let domain = schema.as_ref().map(|s| {
                if s.id.as_str().contains("physics") {
                    Domain::Physics
                } else if s.id.as_str().contains("chem") {
                    Domain::Chemistry
                } else if s.id.as_str().contains("reasoning") {
                    Domain::Reasoning
                } else {
                    Domain::Mathematics
                }
            }).unwrap_or(Domain::Mathematics);

            let ctx = RemediationContext {
                skill_id: &outcome.skill_id,
                schema_id: &outcome.schema_id,
                domain,
                primary_error: err_cat.clone(),
                step_error: None,
                decision_point_correct: if outcome.decision_points_presented > 0 {
                    Some(outcome.decision_points_correct == outcome.decision_points_presented)
                } else {
                    None
                },
                independence: outcome.independence_level,
                progression_state: skill_state.practice_state,
                recent_attempts: &skill_state.recent_attempts,
                source_attempt_id: &outcome.attempt_id,
                recurrence_count: recurrence,
                is_transfer_attempt: false,
            };

            let action = RemediationPolicy::evaluate(&ctx);

            let mut queue_lock = self.remediation_queue.lock().unwrap();
            queue_lock.enqueue(action.clone());
            let _ = self.store.save_remediation_queue(&queue_lock);

            let mut audit_lock = self.audit_log.lock().unwrap();
            let audit_rec = RemediationAuditRecord::new(
                format!("audit-{}", action.id),
                &outcome.attempt_id,
                &outcome.skill_id,
                &outcome.schema_id,
                err_cat,
                action.kind,
                action.kind.as_str(),
                action.recurrence_count,
            );
            audit_lock.record_event(audit_rec);

            Ok((outcome, Some(action)))
        } else {
            Ok((outcome, None))
        }
    }

    /// Stepwise evaluation and remediation deriving precise step-localized remediation actions.
    pub fn evaluate_and_remediate_stepwise_attempt(
        &self,
        instance_id: &ProblemInstanceId,
        card_id: Option<i64>,
        submission: &crate::problems::steps::StepwiseSubmission,
    ) -> Result<(ProceduralReviewOutcome, Option<RemediationAction>)> {
        let outcome = self.evaluate_stepwise_attempt(instance_id, card_id, submission)?;

        if !outcome.is_correct {
            let skill_state = self
                .load_skill_state(&outcome.skill_id)?
                .unwrap_or_else(|| SkillState::new(outcome.skill_id.clone()));

            let err_cat = outcome
                .error_category
                .clone()
                .unwrap_or(ErrorCategory::Unknown);

            let queue_lock = self.remediation_queue.lock().unwrap();
            let recurrence = queue_lock.get_recurrence_count(&outcome.skill_id, &err_cat) + 1;
            drop(queue_lock);

            let schema = self.store.get_schema(&outcome.schema_id)?;
            let domain = schema.as_ref().map(|s| {
                if s.id.as_str().contains("physics") {
                    Domain::Physics
                } else if s.id.as_str().contains("chem") {
                    Domain::Chemistry
                } else if s.id.as_str().contains("reasoning") {
                    Domain::Reasoning
                } else {
                    Domain::Mathematics
                }
            }).unwrap_or(Domain::Mathematics);

            let ctx = RemediationContext {
                skill_id: &outcome.skill_id,
                schema_id: &outcome.schema_id,
                domain,
                primary_error: err_cat.clone(),
                step_error: None,
                decision_point_correct: if outcome.decision_points_presented > 0 {
                    Some(outcome.decision_points_correct == outcome.decision_points_presented)
                } else {
                    None
                },
                independence: outcome.independence_level,
                progression_state: skill_state.practice_state,
                recent_attempts: &skill_state.recent_attempts,
                source_attempt_id: &outcome.attempt_id,
                recurrence_count: recurrence,
                is_transfer_attempt: false,
            };

            let action = RemediationPolicy::evaluate(&ctx);

            let mut queue_lock = self.remediation_queue.lock().unwrap();
            queue_lock.enqueue(action.clone());
            let _ = self.store.save_remediation_queue(&queue_lock);

            let mut audit_lock = self.audit_log.lock().unwrap();
            let audit_rec = RemediationAuditRecord::new(
                format!("audit-{}", action.id),
                &outcome.attempt_id,
                &outcome.skill_id,
                &outcome.schema_id,
                err_cat,
                action.kind,
                action.kind.as_str(),
                action.recurrence_count,
            );
            audit_lock.record_event(audit_rec);

            Ok((outcome, Some(action)))
        } else {
            Ok((outcome, None))
        }
    }

    /// Select the next applicable concrete remediation intervention respecting practice mode and priority.
    pub fn get_next_remediation_intervention(
        &self,
        mode: &PracticeMode,
        seed: u64,
    ) -> Result<Option<(RemediationAction, RemediationIntervention)>> {
        let mut queue_lock = self.remediation_queue.lock().unwrap();
        if let Some(action) = queue_lock.select_next_remediation(mode) {
            let _ = self.store.save_remediation_queue(&queue_lock);
            drop(queue_lock);
            let intervention = RemediationSelector::select_intervention(
                &action,
                &self.store,
                &self.registry,
                seed,
            )?;
            Ok(Some((action, intervention)))
        } else {
            Ok(None)
        }
    }

    /// Record a learner's response to a remediation intervention, updating SkillState and audit logs.
    pub fn record_remediation_response(
        &self,
        action: &RemediationAction,
        is_correct: bool,
        evidence: &MasteryEvidence,
        score: f64,
        target_latency_ms: u64,
    ) -> Result<()> {
        let mut state = self
            .store
            .get_skill_state(&action.skill_id)?
            .unwrap_or_else(|| SkillState::new(action.skill_id.clone()));

        state.record_attempt_outcome(
            evidence,
            score,
            target_latency_ms,
            Utc::now().timestamp(),
        );

        self.store.upsert_skill_state(&state)?;

        let mut queue_lock = self.remediation_queue.lock().unwrap();
        if is_correct {
            queue_lock.record_resolution(&action.skill_id, &action.primary_error);
        }
        let _ = self.store.save_remediation_queue(&queue_lock);

        let mut audit_lock = self.audit_log.lock().unwrap();
        let audit_id = format!("audit-{}", action.id);
        if let Some(record) = audit_lock.get_record_mut(&audit_id) {
            record.mark_completed(
                is_correct,
                evidence.clone(),
                if is_correct {
                    RemediationOutcomeStatus::Resolved
                } else {
                    RemediationOutcomeStatus::Escalated
                },
            );
        }

        Ok(())
    }

    /// Number of currently pending remediation actions in queue.
    pub fn remediation_queue_len(&self) -> usize {
        let queue_lock = self.remediation_queue.lock().unwrap();
        queue_lock.len()
    }

    /// List recent remediation audit records.
    pub fn list_remediation_audit_records(&self, limit: usize) -> Vec<RemediationAuditRecord> {
        let audit_lock = self.audit_log.lock().unwrap();
        audit_lock.recent_records(limit).into_iter().cloned().collect()
    }

    /// Clear all pending remediation actions and recurrence counters.
    pub fn clear_remediation_queue(&self) {
        let mut queue_lock = self.remediation_queue.lock().unwrap();
        queue_lock.clear();
        let _ = self.store.save_remediation_queue(&queue_lock);
    }

    // =========================================================================
    // EXAM & PYQ PLATFORM ENGINE
    // =========================================================================

    /// Seed the store with built-in canonical ExamProfiles (RRB ALP, SSC CGL, Banking PO, JEE Main).
    pub fn init_default_exam_profiles(&self) -> Result<()> {
        self.store.insert_exam_profile(&ExamProfile::rrb_alp())?;
        self.store.insert_exam_profile(&ExamProfile::ssc_cgl())?;
        self.store.insert_exam_profile(&ExamProfile::banking_po())?;
        self.store.insert_exam_profile(&ExamProfile::jee_main_foundation())?;
        Ok(())
    }

    /// Ingest an authentic PYQ as immutable source content and optionally attach initial mapping.
    pub fn ingest_pyq(&self, pyq: PYQSource, mapping: Option<PyqMapping>) -> Result<()> {
        self.store.insert_pyq_source(&pyq)?;
        if let Some(m) = mapping {
            self.store.insert_pyq_mapping(&m)?;
        }
        Ok(())
    }

    /// Retrieve an authentic PYQ source document by ID.
    pub fn get_pyq(&self, id: &PyqId) -> Result<Option<PYQSource>> {
        self.store.get_pyq_source(id)
    }

    /// List all imported PYQs for a given exam title.
    pub fn list_pyqs_by_exam(&self, exam: &str) -> Result<Vec<PYQSource>> {
        self.store.list_pyq_sources_by_exam(exam)
    }

    /// Attach or update schema mapping for an authentic PYQ.
    pub fn map_pyq(&self, mapping: PyqMapping) -> Result<()> {
        self.store.insert_pyq_mapping(&mapping)
    }

    /// Retrieve the current schema mapping for a PYQ.
    pub fn get_pyq_mapping(&self, pyq_id: &PyqId) -> Result<Option<PyqMapping>> {
        self.store.get_pyq_mapping(pyq_id)
    }

    /// Execute a human review action on a PYQ mapping (Approve, Reject, Remap, Regenerate).
    pub fn review_pyq_mapping(&self, pyq_id: &PyqId, action: ReviewAction) -> Result<()> {
        let mut mapping = self
            .store
            .get_pyq_mapping(pyq_id)?
            .ok_or_else(|| ProceduralError::NotFound(format!("Mapping for PYQ '{}' not found", pyq_id)))?;

        HumanReviewWorkflow::apply_review_action(&mut mapping, action)?;
        self.store.insert_pyq_mapping(&mapping)
    }

    /// Inspect a PYQ source, its current mapping, generated test variant, and validation status.
    pub fn inspect_pyq_for_review(&self, pyq_id: &PyqId, sample_seed: u64) -> Result<Option<ReviewInspection>> {
        let pyq_opt = self.store.get_pyq_source(pyq_id)?;
        let Some(pyq) = pyq_opt else {
            return Ok(None);
        };
        let mapping = self.store.get_pyq_mapping(pyq_id)?;
        Ok(Some(HumanReviewWorkflow::inspect_pyq(
            &self.registry,
            &pyq,
            mapping.as_ref(),
            sample_seed,
        )))
    }

    /// Generate a domain-validated procedural variant derived from a mapped PYQ.
    pub fn generate_validated_pyq_variant(
        &self,
        pyq_id: &PyqId,
        seed: u64,
        variant: Option<&str>,
    ) -> Result<ProblemInstance> {
        let pyq = self
            .store
            .get_pyq_source(pyq_id)?
            .ok_or_else(|| ProceduralError::NotFound(format!("PYQ '{}' not found", pyq_id)))?;

        let mapping = self
            .store
            .get_pyq_mapping(pyq_id)?
            .ok_or_else(|| ProceduralError::NotFound(format!("Mapping for PYQ '{}' not found", pyq_id)))?;

        if !mapping.is_eligible_for_practice() {
            return Err(ProceduralError::Validation(format!(
                "PYQ '{}' mapping is not eligible for active practice (status: {:?}, confidence: {:?})",
                pyq_id, mapping.status, mapping.confidence
            )));
        }

        match PyqVariantPipeline::generate_and_validate_variant(
            &self.registry,
            Some(&pyq),
            &mapping,
            seed,
            variant,
        ) {
            Ok(instance) => {
                self.store.insert_problem_instance(&instance)?;
                Ok(instance)
            }
            Err((err, rejected_record_opt)) => {
                if let Some(ref record) = rejected_record_opt {
                    let _ = self.store.insert_rejected_variant(record);
                }
                Err(err)
            }
        }
    }

    /// Save an ExamProfile to persistence.
    pub fn save_exam_profile(&self, profile: ExamProfile) -> Result<()> {
        self.store.insert_exam_profile(&profile)
    }

    /// Retrieve an ExamProfile by ID.
    pub fn get_exam_profile(&self, id: &ExamProfileId) -> Result<Option<ExamProfile>> {
        self.store.get_exam_profile(id)
    }

    /// List all registered ExamProfiles.
    pub fn list_exam_profiles(&self) -> Result<Vec<ExamProfile>> {
        self.store.list_exam_profiles()
    }

    /// Prepare an adaptive exam practice session guided by an ExamProfile and ExamPracticeMode.
    pub fn prepare_exam_practice_session(
        &self,
        profile_id: &ExamProfileId,
        mode: &ExamPracticeMode,
        last_schema_id: Option<&SchemaId>,
        seed: u64,
    ) -> Result<Option<PracticeSessionObject>> {
        let profile = self
            .store
            .get_exam_profile(profile_id)?
            .or_else(|| {
                // Fall back to built-in canonical profiles
                match profile_id.as_str() {
                    "rrb_alp" => Some(ExamProfile::rrb_alp()),
                    "ssc_cgl" => Some(ExamProfile::ssc_cgl()),
                    "banking_po" => Some(ExamProfile::banking_po()),
                    "jee_main_foundation" => Some(ExamProfile::jee_main_foundation()),
                    _ => None,
                }
            })
            .ok_or_else(|| ProceduralError::NotFound(format!("ExamProfile '{}' not found", profile_id)))?;

        // Gather all 22 catalog schemas
        let mut candidate_schemas = Vec::new();
        let mut schema_domains = HashMap::new();

        let all_canonical_schemas = [
            (SCHEMA_SUCCESSIVE_PERCENTAGE, Domain::Mathematics),
            (SCHEMA_LINEAR_EQUATIONS, Domain::Mathematics),
            (SCHEMA_PROFIT_LOSS, Domain::Mathematics),
            (SCHEMA_RATIO, Domain::Mathematics),
            (SCHEMA_AVERAGE, Domain::Mathematics),
            (SCHEMA_DIVISIBILITY, Domain::Mathematics),
            (SCHEMA_TIME_WORK, Domain::Mathematics),
            (SCHEMA_TIME_SPEED_DISTANCE, Domain::Mathematics),
            (SCHEMA_MIXTURES_ALLIGATION, Domain::Mathematics),
            (SCHEMA_REMAINDERS_MODULAR, Domain::Mathematics),
            (SCHEMA_LINEAR_INEQUALITIES, Domain::Mathematics),
            (SCHEMA_ALGEBRAIC_IDENTITIES, Domain::Mathematics),
            (SCHEMA_GEOMETRY_TRIANGLES, Domain::Mathematics),
            (SCHEMA_COMBINED_MULTI_CONCEPT, Domain::Mathematics),
            (SCHEMA_PHYSICS_KINEMATICS, Domain::Physics),
            (SCHEMA_PHYSICS_WORK_ENERGY, Domain::Physics),
            (SCHEMA_CHEMISTRY_STOICHIOMETRY, Domain::Chemistry),
            (SCHEMA_CHEMISTRY_EQUILIBRIUM, Domain::Chemistry),
            (SCHEMA_REASONING_SERIES, Domain::Reasoning),
            (SCHEMA_REASONING_SYLLOGISM, Domain::Reasoning),
            (SCHEMA_REASONING_SEATING, Domain::Reasoning),
            (SCHEMA_REASONING_RELATIONS, Domain::Reasoning),
        ];

        let all_schemas_map = self.store.list_all_schemas_map()?;
        let all_skill_states = self.store.list_all_skill_states_map()?;
        let all_eligible_pyqs = self.store.list_all_eligible_pyqs_map()?;

        for (sid, dom) in all_canonical_schemas {
            let schema_id = SchemaId::from(sid);
            if let Some(schema) = all_schemas_map.get(&schema_id) {
                schema_domains.insert(schema.id.clone(), dom);
                candidate_schemas.push(schema.clone());
            }
        }

        // Load skill states and eligible PYQs for candidate schemas from batch maps
        let mut skill_states = HashMap::new();
        let mut eligible_pyqs = HashMap::new();

        for s in &candidate_schemas {
            if let Some(state) = all_skill_states.get(&s.skill_id) {
                skill_states.insert(s.skill_id.clone(), state.clone());
            }
            if let Some(pyq_list) = all_eligible_pyqs.get(&s.id) {
                if !pyq_list.is_empty() {
                    eligible_pyqs.insert(s.id.clone(), pyq_list.clone());
                }
            }
        }

        Ok(ExamSessionSelector::select_exam_practice(
            &profile,
            mode,
            &candidate_schemas,
            &schema_domains,
            &skill_states,
            &eligible_pyqs,
            last_schema_id,
            &self.registry,
            seed,
        ))
    }

    /// Record a PYQ practice attempt, derive FSRS rating, update SkillState, and evaluate
    /// required pedagogical progression (Variant Confirmation vs Targeted Remediation).
    pub fn record_pyq_practice_attempt(
        &self,
        mut attempt: PracticeAttempt,
        errors: Vec<ErrorEvent>,
        pyq_id: Option<PyqId>,
    ) -> Result<PyqMasteryAction> {
        if let Some(ref pid) = pyq_id {
            if let Some(obj) = attempt.metadata.as_object_mut() {
                obj.insert("source_pyq_id".to_string(), serde_json::Value::String(pid.to_string()));
            }
        }

        self.record_practice_attempt(attempt.clone(), errors)?;

        let state = self
            .store
            .get_skill_state(&attempt.skill_id)?
            .unwrap_or_else(|| SkillState::new(attempt.skill_id.clone()));

        let err_cat = attempt
            .metadata
            .get("error_category")
            .and_then(|v| serde_json::from_value(v.clone()).ok());

        let target_time_ms = attempt
            .metadata
            .get("target_time_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(45_000);

        let outcome = ProceduralReviewOutcome::new(
            attempt.id.clone(),
            attempt.schema_id.clone(),
            attempt.skill_id.clone(),
            ProblemFamilyId::new(attempt.schema_id.as_str()),
            0,
            attempt.is_correct,
            attempt.score,
            attempt.time_taken_ms,
            target_time_ms,
            0,
            1,
            err_cat.clone(),
        );

        let rating = derive_fsrs_rating(&outcome, Some(&state));
        Ok(PyqMasteryBridge::evaluate_pyq_attempt(
            &attempt,
            &state,
            err_cat.as_ref(),
            rating,
            1,
        ))
    }

    /// Retrieve schemas experiencing high failure rates for a specific target exam.
    pub fn get_exam_failing_schemas(&self, exam_id: &str) -> Result<Vec<(SchemaId, f64, usize)>> {
        self.store.get_failing_schemas_for_exam(exam_id)
    }

    // =========================================================================
    // R4 LEARNING DYNAMICS HARDENING HELPERS
    // =========================================================================

    /// Evaluates transfer eligibility for a skill given a desired transfer level.
    pub fn evaluate_transfer_eligibility(
        &self,
        skill_id: &SkillId,
        level: crate::scheduling::transfer::TransferLevel,
        supporting_schemas_stable: bool,
    ) -> Result<crate::scheduling::transfer::TransferEligibilityEvaluation> {
        let state = self
            .store
            .get_skill_state(skill_id)?
            .unwrap_or_else(|| SkillState::new(skill_id.clone()));
        Ok(crate::scheduling::transfer::TransferEngine::evaluate_eligibility(
            &state,
            level,
            supporting_schemas_stable,
        ))
    }

    /// Evaluates if a skill qualifies for retirement into low-frequency maintenance.
    pub fn evaluate_skill_retirement(
        &self,
        skill_id: &SkillId,
    ) -> Result<crate::skills::lifecycle::RetirementEvaluation> {
        let state = self
            .store
            .get_skill_state(skill_id)?
            .unwrap_or_else(|| SkillState::new(skill_id.clone()));
        let policy = crate::skills::lifecycle::RetirementPolicy::default();
        Ok(policy.evaluate_retirement_eligibility(&state))
    }

    /// Evaluates latency performance for an attempt against domain and stage policy.
    pub fn evaluate_stage_speed(
        &self,
        skill_id: &SkillId,
        actual_latency_ms: u64,
        target_override_ms: Option<u64>,
    ) -> Result<crate::scheduling::speed::SpeedEvaluation> {
        let state = self
            .store
            .get_skill_state(skill_id)?
            .unwrap_or_else(|| SkillState::new(skill_id.clone()));
        let skill = self
            .store
            .get_skill(skill_id)?
            .ok_or_else(|| ProceduralError::NotFound(format!("Skill '{}' not found", skill_id)))?;

        Ok(crate::scheduling::speed::StageSpeedPolicy::evaluate(
            state.practice_state,
            skill.domain,
            actual_latency_ms,
            target_override_ms,
        ))
    }

    // =========================================================================
    // DIAGNOSTIC ASSESSMENT / MOCK SESSION ENGINE (GAP-DIAG-01, GAP-EV-01)
    // =========================================================================

    /// Create a fixed measuring mode Diagnostic Mock Session across domains.
    pub fn create_diagnostic_session(
        &self,
        total_questions: usize,
        time_limit_ms: u64,
        seed: u64,
    ) -> Result<MockSession> {
        let blueprint = MockBlueprint::diagnostic_balanced("Diagnostic Assessment", total_questions, time_limit_ms);
        
        let math_schemas = [
            (SCHEMA_SUCCESSIVE_PERCENTAGE, Domain::Mathematics, "Arithmetic", "Percentages"),
            (SCHEMA_LINEAR_EQUATIONS, Domain::Mathematics, "Algebra", "Linear Equations"),
            (SCHEMA_PROFIT_LOSS, Domain::Mathematics, "Arithmetic", "Profit & Loss"),
            (SCHEMA_RATIO, Domain::Mathematics, "Arithmetic", "Ratio & Proportion"),
            (SCHEMA_AVERAGE, Domain::Mathematics, "Arithmetic", "Averages"),
            (SCHEMA_DIVISIBILITY, Domain::Mathematics, "Number System", "Divisibility Rules"),
            (SCHEMA_TIME_WORK, Domain::Mathematics, "Arithmetic", "Time & Work"),
            (SCHEMA_TIME_SPEED_DISTANCE, Domain::Mathematics, "Arithmetic", "Speed & Distance"),
            (SCHEMA_MIXTURES_ALLIGATION, Domain::Mathematics, "Arithmetic", "Mixtures"),
            (SCHEMA_REMAINDERS_MODULAR, Domain::Mathematics, "Number System", "Modular Arithmetic"),
            (SCHEMA_LINEAR_INEQUALITIES, Domain::Mathematics, "Algebra", "Inequalities"),
            (SCHEMA_ALGEBRAIC_IDENTITIES, Domain::Mathematics, "Algebra", "Identities"),
            (SCHEMA_GEOMETRY_TRIANGLES, Domain::Mathematics, "Geometry", "Triangles"),
            (SCHEMA_COMBINED_MULTI_CONCEPT, Domain::Mathematics, "Integrated", "Multi-Concept"),
        ];

        let reasoning_schemas = [
            (SCHEMA_REASONING_SERIES, Domain::Reasoning, "Analytical Reasoning", "Number & Letter Series"),
            (SCHEMA_REASONING_SYLLOGISM, Domain::Reasoning, "Logical Deduction", "Syllogisms"),
            (SCHEMA_REASONING_SEATING, Domain::Reasoning, "Arrangement", "Linear & Circular Seating"),
            (SCHEMA_REASONING_RELATIONS, Domain::Reasoning, "Deduction", "Blood Relations"),
        ];

        let physics_schemas = [
            (SCHEMA_PHYSICS_KINEMATICS, Domain::Physics, "Mechanics", "1D/2D Kinematics"),
            (SCHEMA_PHYSICS_WORK_ENERGY, Domain::Physics, "Mechanics", "Work, Energy & Power"),
        ];

        let chemistry_schemas = [
            (SCHEMA_CHEMISTRY_STOICHIOMETRY, Domain::Chemistry, "Physical Chemistry", "Mole & Stoichiometry"),
            (SCHEMA_CHEMISTRY_EQUILIBRIUM, Domain::Chemistry, "Physical Chemistry", "Chemical Equilibrium"),
        ];

        let mut questions = Vec::new();
        let q_count = total_questions.clamp(4, 50);

        for idx in 0..q_count {
            let domain_order = idx % 4;
            let round = idx / 4;
            let (sch_name, domain, chapter, topic) = match domain_order {
                0 => &math_schemas[round % math_schemas.len()],
                1 => &reasoning_schemas[round % reasoning_schemas.len()],
                2 => &physics_schemas[round % physics_schemas.len()],
                _ => &chemistry_schemas[round % chemistry_schemas.len()],
            };

            let schema_id = SchemaId::new(*sch_name);
            let schema = self.resolve_schema(&schema_id)?
                .ok_or_else(|| ProceduralError::NotFound(format!("Diagnostic schema '{}' not found", sch_name)))?;

            let diff_level = ((idx % 4) + 2) as u32;
            let q_seed = seed.wrapping_add((idx as u64) * 1000 + 42);
            let mut inst = self.registry.generate(&schema.problem_family_id, schema.problem_family_id.as_str(), q_seed, diff_level, None)?;
            
            inst.metadata["chapter"] = serde_json::json!(chapter);
            inst.metadata["topic"] = serde_json::json!(topic);
            inst.metadata["domain"] = serde_json::json!(domain);

            questions.push(MockQuestionItem {
                question_index: idx,
                schema_id: schema.id.clone(),
                skill_id: schema.skill_id.clone(),
                domain: domain.clone(),
                schema_title: schema.title.clone(),
                instance: inst,
                difficulty_level: diff_level,
                target_time_ms: 45_000,
                is_pyq: false,
                provenance: None,
            });
        }

        let session_id = format!("diag_sess_{}_{}", Utc::now().timestamp_millis(), seed % 10000);
        Ok(MockSession::new(session_id, blueprint, questions))
    }

    /// Batch-updates existing SkillState and DomainEvidence in store from a completed Diagnostic Mock Session.
    pub fn record_diagnostic_report_evidence(
        &self,
        session: &MockSession,
        report: &ComprehensiveDiagnosticReport,
    ) -> Result<Vec<SkillState>> {
        apply_diagnostic_report_to_store(&self.store, session, report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_maths_vertical_slice_end_to_end() {
        let service = ProceduralService::open_in_memory().unwrap();

        // 1. Anchor resolution
        let anchor = ProceduralCardAnchor::new("percentage.successive");
        let session = service.prepare_practice_session(&anchor, Some(12345)).unwrap();

        assert_eq!(session.schema.id.as_str(), "successive_percentage");
        assert_eq!(session.card_id, Some(12345));
        assert!(session.selected_variant.is_some());
        assert!(session.target_latency_ms.is_some());
        assert!(session.selection_reason.is_some());

        // 2. Extract answer value
        let correct_val = session
            .instance
            .correct_answer
            .get("value")
            .unwrap()
            .as_f64()
            .unwrap();

        // 3. Evaluate correct attempt
        let outcome = service
            .evaluate_and_record_attempt(
                &session.instance.id,
                session.card_id,
                serde_json::json!(correct_val),
                25_000,
                0,
                1,
            )
            .unwrap();

        assert!(outcome.is_correct);
        assert_eq!(outcome.score, 1.0);
        assert_eq!(outcome.error_category, None);

        // 4. Derive FSRS rating
        let rating = service.derive_fsrs_rating(&outcome).unwrap();
        assert!(matches!(rating, Rating::Good | Rating::Easy));

        // 5. Verify skill state updated with rich signals
        let state = service.load_skill_state(&outcome.skill_id).unwrap().unwrap();
        assert_eq!(state.total_attempts, 1);
        assert_eq!(state.successful_attempts, 1);
        assert_eq!(state.consecutive_successes, 1);
        assert_eq!(state.recent_accuracy(), 1.0);
        assert_eq!(state.moving_average_latency_ms(), 25_000.0);

        // 6. Query card attempts
        let attempts = service.get_attempts_for_card(12345).unwrap();
        assert_eq!(attempts.len(), 1);
        assert!(attempts[0].is_correct);
    }

    #[test]
    fn test_service_multi_schema_session_mixed_and_focused() {
        let service = ProceduralService::open_in_memory().unwrap();

        // Mixed mode session
        let session = service
            .prepare_multi_schema_session(&PracticeMode::MixedMaths, None, None, Some(42))
            .unwrap();

        assert!(!session.instance.rendered_prompt.is_empty());
        assert!(session.difficulty_level.is_some());
        assert!(session.target_latency_ms.is_some());

        // Focused mode session on Linear Equations
        let linear_skill = SkillId::new(crate::problems::catalog::SKILL_LINEAR_EQUATIONS);
        let focused_session = service
            .prepare_multi_schema_session(
                &PracticeMode::FocusedSkill {
                    skill_id: linear_skill,
                },
                None,
                None,
                Some(999),
            )
            .unwrap();

        assert_eq!(
            focused_session.schema.skill_id.as_str(),
            crate::problems::catalog::SKILL_LINEAR_EQUATIONS
        );
    }

    #[test]
    fn test_service_all_fourteen_schemas_resolution_and_generation() {
        let service = ProceduralService::open_in_memory().unwrap();

        let all_schemas = [
            "successive_percentage",
            "algebra_linear_equations",
            "arithmetic_profit_loss",
            "arithmetic_ratio",
            "arithmetic_average",
            "number_system_divisibility",
            "time_work_basic",
            "arithmetic_time_speed_distance",
            "arithmetic_mixtures_alligation",
            "number_system_remainders_modular",
            "algebra_linear_inequalities",
            "algebra_algebraic_identities",
            "geometry_triangles",
            "combined_multi_concept",
        ];

        for schema_str in all_schemas {
            let schema_id = SchemaId::new(schema_str);
            let schema = service.resolve_schema(&schema_id).unwrap();
            assert!(schema.is_some(), "Schema {} should resolve", schema_str);

            let s = schema.unwrap();
            let anchor = ProceduralCardAnchor::new(s.id.as_str());
            let session = service.prepare_practice_session(&anchor, None).unwrap();
            assert!(!session.instance.rendered_prompt.is_empty(), "Prompt should render for {}", schema_str);

            // Verify solution graph is present
            assert!(session.instance.solution_graph().is_some(), "Solution graph must exist for {}", schema_str);
        }
    }

    #[test]
    fn test_service_transfer_practice_end_to_end() {
        let service = ProceduralService::open_in_memory().unwrap();
        let sch_id = SchemaId::new(SCHEMA_TIME_SPEED_DISTANCE);
        let schema = service.resolve_schema(&sch_id).unwrap().unwrap();

        // 1. Build initial mastery (2 fast successes)
        let inst1 = service.generate_problem(&schema.problem_family_id, 101, &serde_json::Value::Null).unwrap();
        service.save_problem_instance(inst1.clone()).unwrap();
        let val1 = inst1.correct_answer.get("value").unwrap().as_f64().unwrap();
        service.evaluate_and_record_attempt(&inst1.id, None, serde_json::json!(val1), 15000, 0, 1).unwrap();

        let inst2 = service.generate_problem(&schema.problem_family_id, 102, &serde_json::Value::Null).unwrap();
        service.save_problem_instance(inst2.clone()).unwrap();
        let val2 = inst2.correct_answer.get("value").unwrap().as_f64().unwrap();
        service.evaluate_and_record_attempt(&inst2.id, None, serde_json::json!(val2), 18000, 0, 1).unwrap();

        // 2. Launch Transfer Practice mode
        let session = service
            .prepare_multi_schema_session(
                &PracticeMode::FocusedSkill { skill_id: schema.skill_id.clone() },
                None,
                None,
                Some(200),
            )
            .unwrap();

        assert!(!session.instance.rendered_prompt.is_empty());
    }

    #[test]
    fn test_service_catalog_v2_version_integrity() {
        let service = ProceduralService::open_in_memory().unwrap();
        let sch_id = SchemaId::new(SCHEMA_COMBINED_MULTI_CONCEPT);
        let schema = service.resolve_schema(&sch_id).unwrap().unwrap();

        let inst = service.generate_problem(&schema.problem_family_id, 300, &serde_json::Value::Null).unwrap();
        service.save_problem_instance(inst.clone()).unwrap();

        let val = inst.correct_answer.get("value").unwrap().as_f64().unwrap();
        let outcome = service.evaluate_and_record_attempt(&inst.id, Some(777), serde_json::json!(val), 20000, 0, 1).unwrap();
        assert!(outcome.is_correct);

        let attempts = service.get_attempts_for_card(777).unwrap();
        assert_eq!(attempts.len(), 1);
        assert_eq!(attempts[0].metadata.get("catalog_version").unwrap().as_str().unwrap(), MATHS_CATALOG_VERSION);
    }

    #[test]
    fn test_service_diagnostic_mock_session_and_evidence_sync() {
        let service = ProceduralService::open_in_memory().unwrap();
        
        // 1. Generate a 12-question diagnostic session
        let mut session = service.create_diagnostic_session(12, 600_000, 12345).unwrap();
        assert_eq!(session.questions.len(), 12);
        assert_eq!(session.blueprint.negative_mark_per_incorrect, 0.0);
        assert_eq!(session.blueprint.total_questions, 12);

        // 2. Answer questions across domains
        for i in 0..12 {
            let q = &session.questions[i];
            let ans_str = if let Some(v) = q.instance.correct_answer.get("formatted").and_then(|f| f.as_str()) {
                v.to_string()
            } else if let Some(v) = q.instance.correct_answer.get("value").and_then(|v| v.as_f64()) {
                v.to_string()
            } else if let Some(s) = q.instance.correct_answer.as_str() {
                s.to_string()
            } else {
                "0".to_string()
            };

            // Answer first 8 correctly, rest with errors
            if i < 8 {
                session.record_answer(i, ans_str, 20_000);
            } else {
                session.record_answer(i, "wrong_ans", 30_000);
            }
        }

        // 3. Generate comprehensive report
        let report = session.generate_comprehensive_report(Utc::now().timestamp_millis());
        assert_eq!(report.total_questions, 12);
        assert_eq!(report.answered_count, 12);
        assert_eq!(report.correct_count, 8);
        assert_eq!(report.incorrect_count, 4);

        // 4. Batch-sync evidence to ProceduralStore without parallel state models
        let updated_states = service.record_diagnostic_report_evidence(&session, &report).unwrap();
        assert_eq!(updated_states.len(), 12);
        
        // Verify store has updated skill states
        let math_state = service.store.get_skill_state(&session.questions[0].skill_id).unwrap();
        assert!(math_state.is_some());
    }

    #[test]
    fn test_reconciliation_lifecycle() {
        use crate::anchor::source::{CanonicalQuestionType, SourceQuestion};

        let store = crate::storage::store::ProceduralStore::open_in_memory().unwrap();
        let service = ProceduralService::new(store);

        let guid = "abc123_test".to_string();
        let source_q = SourceQuestion {
            prompt: "What is 2 + 2?".to_string(),
            options: Some(vec!["1".to_string(), "2".to_string(), "3".to_string(), "4".to_string()]),
            correct_answer: "4".to_string(),
            hint: None,
            solution: None,
            steps: None,
            explanation: Some("Basic math".to_string()),
            subject: Some("Mathematics".to_string()),
            chapter: Some("Algebra".to_string()),
            topic: Some("Math".to_string()),
            skill: Some("math.arithmetic".to_string()),
            problem_type: Some("mcq".to_string()),
            question_type: CanonicalQuestionType::Mcq,
            difficulty: Some(1.0),
            source: Some("Sample Deck".to_string()),
            exam: None,
            year: None,
            shift: None,
            paper: None,
            source_question_id: None,
        };

        // 1. Initial Insert (NEW)
        let report1 = service.reconcile_source_questions(vec![(guid.clone(), source_q.clone())]).unwrap();
        assert_eq!(report1.new_count, 1);
        assert_eq!(report1.updated_count, 0);
        assert_eq!(report1.unchanged_count, 0);
        assert_eq!(report1.archived_count, 0);

        // 2. No Changes (UNCHANGED)
        let report2 = service.reconcile_source_questions(vec![(guid.clone(), source_q.clone())]).unwrap();
        assert_eq!(report2.new_count, 0);
        assert_eq!(report2.updated_count, 0);
        assert_eq!(report2.unchanged_count, 1);
        assert_eq!(report2.archived_count, 0);

        // 3. Update existing (UPDATED)
        let mut source_q_updated = source_q.clone();
        source_q_updated.prompt = "What is 3 + 3?".to_string();
        let report3 = service.reconcile_source_questions(vec![(guid.clone(), source_q_updated.clone())]).unwrap();
        assert_eq!(report3.new_count, 0);
        assert_eq!(report3.updated_count, 1);
        assert_eq!(report3.unchanged_count, 0);
        assert_eq!(report3.archived_count, 0);

        // 4. Archive missing (ARCHIVED)
        let report4 = service.reconcile_source_questions(vec![]).unwrap();
        assert_eq!(report4.new_count, 0);
        assert_eq!(report4.updated_count, 0);
        assert_eq!(report4.unchanged_count, 0);
        assert_eq!(report4.archived_count, 1);
    }

    #[test]
    fn test_resolve_source_target() {
        use crate::anchor::source::{CanonicalQuestionType, SourceQuestion};

        let store = crate::storage::store::ProceduralStore::open_in_memory().unwrap();
        let service = ProceduralService::new(store);

        let guid = "abc123_resolve_test".to_string();
        let source_q = SourceQuestion {
            prompt: "What is 2 + 2?".to_string(),
            options: Some(vec!["1".to_string(), "2".to_string(), "3".to_string(), "4".to_string()]),
            correct_answer: "4".to_string(),
            hint: None,
            solution: None,
            steps: None,
            explanation: Some("Basic math".to_string()),
            subject: Some("Mathematics".to_string()),
            chapter: Some("Algebra".to_string()),
            topic: Some("Math".to_string()),
            skill: Some("math.arithmetic".to_string()),
            problem_type: Some("mcq".to_string()),
            question_type: CanonicalQuestionType::Mcq,
            difficulty: Some(1.0),
            source: None,
            exam: None,
            year: None,
            shift: None,
            paper: None,
            source_question_id: None,
        };

        // Initially it shouldn't exist
        let err = service.resolve_source_target(&guid, Some(42)).unwrap_err();
        assert!(matches!(err, crate::core::ProceduralError::NotFound(_)));

        // Reconcile to insert it
        service.reconcile_source_questions(vec![(guid.clone(), source_q)]).unwrap();

        // Now we can resolve it
        let session = service.resolve_source_target(&guid, Some(42)).unwrap();
        assert_eq!(session.card_id, Some(42));
        assert_eq!(session.schema.id.as_str(), "schema.source.mathematics.math");
        assert_eq!(session.instance.correct_answer["correct_option"].as_str().unwrap(), "4");
    }

    #[test]
    fn test_evaluate_source_first_attempt() {
        use crate::anchor::source::{CanonicalQuestionType, SourceQuestion};
        use serde_json::json;

        let store = crate::storage::store::ProceduralStore::open_in_memory().unwrap();
        let service = ProceduralService::new(store);

        let guid = "abc123_eval_test".to_string();
        let source_q = SourceQuestion {
            prompt: "What is 2 + 3?".to_string(),
            options: Some(vec!["3".to_string(), "4".to_string(), "5".to_string(), "6".to_string()]),
            correct_answer: "5".to_string(),
            hint: None,
            solution: None,
            steps: None,
            explanation: Some("Basic math".to_string()),
            subject: Some("Mathematics".to_string()),
            chapter: Some("Algebra".to_string()),
            topic: Some("Math".to_string()),
            skill: Some("math.arithmetic".to_string()),
            problem_type: Some("mcq".to_string()),
            question_type: CanonicalQuestionType::Mcq,
            difficulty: Some(1.0),
            source: None,
            exam: None,
            year: None,
            shift: None,
            paper: None,
            source_question_id: None,
        };

        // Reconcile to insert it
        service.reconcile_source_questions(vec![(guid.clone(), source_q)]).unwrap();

        // Resolve to get the instance ID
        let session = service.resolve_source_target(&guid, Some(42)).unwrap();
        let instance_id = &session.instance.id;
        
        assert!(instance_id.as_str().starts_with("inst-pi-"));

        // Evaluate and record attempt
        let outcome = service.evaluate_and_record_attempt(
            instance_id,
            Some(42),
            json!({"value": "5"}),
            15000,
            0,
            1,
        ).unwrap();

        assert!(outcome.is_correct);
        assert_eq!(outcome.score, 1.0);
        assert_eq!(outcome.schema_id.as_str(), "schema.source.mathematics.math");

        // Now evaluate an incorrect attempt to trigger error event logging
        let outcome2 = service.evaluate_and_record_attempt(
            instance_id,
            Some(42),
            json!({"value": "3"}),
            20000,
            0,
            2,
        ).unwrap();

        assert!(!outcome2.is_correct);
        assert_eq!(outcome2.score, 0.0);
    }
}
