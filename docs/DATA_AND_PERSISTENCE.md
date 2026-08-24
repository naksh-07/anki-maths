# StudyLab Data & Persistence Architecture Specification

**Document Version:** 1.0.0 (Canonical)  
**Target Subsystem:** Rust Storage Layer (`rslib/procedural/src/storage/`), SQLite Engine (`procedural.db`)  
**Status:** AUTHORITATIVE CANONICAL SPECIFICATION  
**Integrity Mode:** 100% Grounded in Executable Source Code & Test Evidence  

---

## 1. Architectural Overview & Database Separation

StudyLab enforces strict architectural separation between Anki's native collection database (`collection.anki2`) and StudyLab's procedural learning store (`<collection_name>.procedural` or `procedural.db`).

```text
┌────────────────────────────────────────┐       ┌────────────────────────────────────────┐
│        ANKI COLLECTION DATABASE        │       │       STUDYLAB PROCEDURAL STORE        │
│          (<collection>.anki2)          │       │         (<collection>.procedural)      │
├────────────────────────────────────────┤       ├────────────────────────────────────────┤
│ • Native Notes, Cards, Decks, Configs  │       │ • Discrete Skill Graph & Mastery (EMA) │
│ • FSRS / SM-2 Spaced Repetition Revlog │       │ • Problem Families & Declarative Schemas│
│ • Ephemeral ProceduralCardAnchor JSON  │       │ • Concrete Problem Instances & Graphs  │
│ • Unmodified Standard Schema           │       │ • Practice Attempts & Diagnostic Errors│
│ • 100-byte cards.data column limit     │       │ • PYQ Source Mappings & Exam Profiles  │
│ • Synchronized via AnkiWeb Media/Sync  │       │ • Canonical Practice Items & Chapters  │
│                                        │       │ • Durable Remediation Queue & Tracker  │
└────────────────────────────────────────┘       └────────────────────────────────────────┘
```

### Why Database Separation Is an Architectural Invariant:
1. **Zero Core Schema Pollution:** StudyLab does not alter or add tables to Anki's upstream `collection.anki2` schema, eliminating risk of corruption during standard Anki upgrades.
2. **AnkiWeb Sync Boundary Protection:** Standard AnkiWeb syncs `collection.anki2` and media files. Decoupling high-frequency procedural telemetry avoids bloating the sync channel or violating Anki's **100-byte custom data limit** on cards.
3. **Independent Schema Evolution:** Procedural practice requires rich relational modeling (11 tables, 17 indexes, JSON signals, directed acyclic graphs). An independent SQLite store allows seamless migration (v1 through v5) without affecting Anki collection integrity.
4. **Targeted ACID Transactions:** Attempt outcomes, mastery updates, error logs, and remediation queues are committed in dedicated atomic SQLite transactions without holding locks on the main Anki collection.

---

## 2. Storage Engine & SQLite Pragmas

The procedural store is managed by `ProceduralStore` (`rslib/procedural/src/storage/store.rs:28-53`), encapsulating a thread-safe connection `Arc<Mutex<Connection>>`.

Upon opening `<collection_name>.procedural` (or `:memory:` during automated test execution), `ProceduralStore::apply_pragmas` configures high-performance SQLite operational pragmas:

```sql
PRAGMA busy_timeout = 5000;
PRAGMA foreign_keys = ON;
PRAGMA synchronous = NORMAL;
PRAGMA temp_store = MEMORY;
PRAGMA journal_mode = WAL;
```

### Pragma Rationale:
- **`busy_timeout = 5000`:** Sets a 5-second lock acquisition timeout to prevent immediate busy errors during concurrent background task execution.
- **`foreign_keys = ON`:** Strictly enforces relational integrity and cascading deletes across skills, families, schemas, instances, and attempts.
- **`synchronous = NORMAL`:** Optimizes disk I/O in WAL mode while maintaining full ACID crash resilience.
- **`temp_store = MEMORY`:** Executes temporary tables, sorting buffers, and index builds in RAM for sub-millisecond query execution.
- **`journal_mode = WAL`:** Write-Ahead Logging allows non-blocking concurrent reads while background writes occur.

---

## 3. Migration Runner & Versioning History

Database schema evolution is managed by `MigrationRunner` (`rslib/procedural/src/storage/migration.rs`). Migrations are tracked in the `schema_migrations` table:

```sql
CREATE TABLE IF NOT EXISTS schema_migrations (
    version INTEGER PRIMARY KEY,
    description TEXT NOT NULL,
    applied_at INTEGER NOT NULL
);
```

### Complete Migration Catalog (v1 to v5)

| Version | Description | Target Subsystems & Features | Tables Created | Indexes Created |
|---|---|---|---|---|
| **v1** | Initial procedural database schema | Skills, Skill States, Problem Families, Schemas, Problem Instances, Practice Attempts, Error Events | `skills`<br>`skill_states`<br>`problem_families`<br>`schemas`<br>`problem_instances`<br>`practice_attempts`<br>`error_events` | `idx_skills_domain`<br>`idx_families_skill`<br>`idx_schemas_skill`<br>`idx_instances_family`<br>`idx_attempts_schema`<br>`idx_attempts_skill`<br>`idx_attempts_card`<br>`idx_error_events_attempt` |
| **v2** | Catalog tracking & query optimization | Catalog versioning metadata and temporal query performance | `catalog_metadata` | `idx_attempts_time`<br>`idx_skill_states_updated` |
| **v3** | Exam Content & Personalization Engine | Authentic PYQ ingestion, schema mappings, rejected variants audit, exam profiles | `pyq_sources`<br>`pyq_mappings`<br>`rejected_variants`<br>`exam_profiles` | `idx_pyq_sources_exam`<br>`idx_pyq_sources_domain`<br>`idx_pyq_mappings_schema`<br>`idx_pyq_mappings_status`<br>`idx_pyq_mappings_confidence`<br>`idx_rejected_variants_pyq`<br>`idx_rejected_variants_family` |
| **v4** | Practice Content Layer & Chapter Capability Model | Canonical practice item definitions, chapter profile configurations | `practice_items`<br>`chapter_practice_profiles` | `idx_practice_items_schema`<br>`idx_practice_items_family`<br>`idx_practice_items_chapter` |
| **v5** | Durable Remediation Queue & Recurrence Tracker | Persistent remediation queue, error recurrence tracking, circuit breaker states | `remediation_queue_items`<br>`remediation_recurrence` | `idx_remediation_queue_skill`<br>`idx_remediation_queue_urgency` |

---

## 4. Comprehensive Table DDL Specifications

### 4.1 Core Skills & Mastery Tracking

#### 1. `skills`
Represents discrete atomic cognitive skill nodes in the curriculum graph.
```sql
CREATE TABLE IF NOT EXISTS skills (
    id TEXT PRIMARY KEY,            -- Strong SkillId (e.g. "math.percentage.successive")
    domain TEXT NOT NULL,           -- Domain enum: "mathematics", "physics", "chemistry", "reasoning"
    name TEXT NOT NULL,             -- Human-readable skill name
    description TEXT NOT NULL,      -- Pedagogical description
    prerequisites TEXT NOT NULL,    -- JSON Array of prerequisite SkillIds
    metadata TEXT NOT NULL,         -- JSON object (tags, cognitive level, curriculum refs)
    created_at INTEGER NOT NULL     -- UTC epoch milliseconds
);
```

#### 2. `skill_states`
Maintains longitudinal learner mastery, progression states, and cognitive diagnostic signals.
```sql
CREATE TABLE IF NOT EXISTS skill_states (
    skill_id TEXT PRIMARY KEY,      -- Foreign key referencing skills(id)
    mastery REAL NOT NULL,          -- EMA Mastery (0.0 to 1.0; Mastery_t = 0.8*M_{t-1} + 0.2*Outcome)
    confidence REAL NOT NULL,       -- Confidence metric: min(total_attempts / 10.0, 1.0)
    total_attempts INTEGER NOT NULL,-- Lifetime attempt counter
    successful_attempts INTEGER NOT NULL, -- Lifetime successful solve counter
    last_practiced_at INTEGER,      -- UTC epoch timestamp of most recent attempt
    custom_state TEXT NOT NULL,     -- JSON object storing rich signals (see schema below)
    updated_at INTEGER NOT NULL,    -- UTC epoch timestamp of last update
    FOREIGN KEY(skill_id) REFERENCES skills(id) ON DELETE CASCADE
);
```

##### Structure of `skill_states.custom_state` JSON:
```json
{
  "progression_state": "Transfer", // "New" | "Learning" | "Fluent" | "Variation" | "Transfer" | "Mastered"
  "recent_outcomes": [1.0, 1.0, 0.0, 1.0, 1.0], // Sliding window of last 10 attempts
  "consecutive_successes": 4,
  "moving_latency": {
    "count": 12,
    "mean_ms": 21500.0,
    "variance": 4200000.0,
    "min_ms": 14200,
    "max_ms": 38000
  },
  "error_counts": {
    "concept": 1,
    "strategy": 0,
    "calculation": 2,
    "careless": 1,
    "time": 0,
    "domain_specific": {}
  },
  "variant_performance": {
    "Isomorphic": { "attempts": 6, "successes": 5, "avg_latency_ms": 20100 },
    "Structural": { "attempts": 4, "successes": 4, "avg_latency_ms": 23400 },
    "Transfer": { "attempts": 2, "successes": 2, "avg_latency_ms": 28000 }
  },
  "delayed_retention_successes": 2,
  "last_delayed_retention_at": 1724500000,
  "domain_evidence": {
    "math": {
      "pattern_recognition": 0.92,
      "method_selection": 0.88,
      "execution": 0.81,
      "verification": 0.75,
      "structural_transfer": 0.85
    }
  }
}
```

---

### 4.2 Problem Families, Schemas & Instances

#### 3. `problem_families`
Canonical catalog of parametric problem generators and contracts.
```sql
CREATE TABLE IF NOT EXISTS problem_families (
    id TEXT PRIMARY KEY,            -- Strong ProblemFamilyId (e.g. "family.math.percentage.successive")
    skill_id TEXT NOT NULL,         -- Foreign key referencing skills(id)
    domain TEXT NOT NULL,           -- Domain enum: "mathematics", "physics", etc.
    name TEXT NOT NULL,             -- Family display title
    template_ref TEXT NOT NULL,     -- Archetype template reference or generator key
    min_difficulty REAL NOT NULL,   -- Lower difficulty bound (typically 1.0)
    max_difficulty REAL NOT NULL,   -- Upper difficulty bound (typically 5.0)
    parameters_schema TEXT NOT NULL,-- JSON Schema declaring parameter domain contracts
    metadata TEXT NOT NULL,         -- JSON object (decision points, target latencies, tags)
    created_at INTEGER NOT NULL,    -- UTC epoch milliseconds
    FOREIGN KEY(skill_id) REFERENCES skills(id) ON DELETE CASCADE
);
```

#### 4. `schemas`
Executable practice schemas binding skills to problem families with concrete pedagogical configurations.
```sql
CREATE TABLE IF NOT EXISTS schemas (
    id TEXT PRIMARY KEY,            -- Strong SchemaId (e.g. "schema.percentage.successive.v1")
    skill_id TEXT NOT NULL,         -- Foreign key referencing skills(id)
    problem_family_id TEXT NOT NULL,-- Foreign key referencing problem_families(id)
    title TEXT NOT NULL,            -- Schema title
    description TEXT NOT NULL,      -- Pedagogical objective
    target_mastery REAL NOT NULL,   -- Mastery threshold for schema completion (e.g. 0.85)
    config TEXT NOT NULL,           -- JSON configuration (scaffolding rules, mode defaults)
    created_at INTEGER NOT NULL,    -- UTC epoch milliseconds
    FOREIGN KEY(skill_id) REFERENCES skills(id) ON DELETE CASCADE,
    FOREIGN KEY(problem_family_id) REFERENCES problem_families(id) ON DELETE CASCADE
);
```

#### 5. `problem_instances`
Concrete ephemeral or seeded problem instances generated during practice.
```sql
CREATE TABLE IF NOT EXISTS problem_instances (
    id TEXT PRIMARY KEY,            -- Strong ProblemInstanceId (e.g. "inst-99214")
    family_id TEXT NOT NULL,        -- Foreign key referencing problem_families(id)
    seed INTEGER NOT NULL,          -- Deterministic RNG seed (u64)
    parameters TEXT NOT NULL,       -- JSON key-value map of sampled numerical parameters
    rendered_prompt TEXT NOT NULL,  -- Rendered prompt with substituted parameters and LaTeX
    correct_answer TEXT NOT NULL,   -- JSON object storing canonical value, format, and solution_graph
    metadata TEXT NOT NULL,         -- JSON object (difficulty, provenance, tags)
    created_at INTEGER NOT NULL,    -- UTC epoch milliseconds
    FOREIGN KEY(family_id) REFERENCES problem_families(id) ON DELETE CASCADE
);
```

---

### 4.3 Practice Attempts & Diagnostic Error Events

#### 6. `practice_attempts`
Immutable log of every problem attempt performed by the learner.
```sql
CREATE TABLE IF NOT EXISTS practice_attempts (
    id TEXT PRIMARY KEY,            -- Strong AttemptId (e.g. "rev-1787580897448-1787603702257")
    instance_id TEXT NOT NULL,      -- Foreign key referencing problem_instances(id)
    schema_id TEXT NOT NULL,        -- Foreign key referencing schemas(id)
    skill_id TEXT NOT NULL,         -- Foreign key referencing skills(id)
    card_id INTEGER,                -- Optional Anki card ID (cards.id in collection.anki2)
    user_answer TEXT NOT NULL,      -- JSON representation of submitted answer or step array
    is_correct INTEGER NOT NULL,    -- Boolean flag (1 = correct, 0 = incorrect)
    score REAL NOT NULL,            -- Continuous score (0.0 to 1.0)
    time_taken_ms INTEGER NOT NULL, -- Active solving latency in milliseconds
    attempted_at INTEGER NOT NULL,  -- UTC epoch timestamp
    metadata TEXT NOT NULL,         -- JSON object (hints_used, speed_quadrant, mistake_type, error_category)
    FOREIGN KEY(instance_id) REFERENCES problem_instances(id) ON DELETE CASCADE,
    FOREIGN KEY(schema_id) REFERENCES schemas(id) ON DELETE CASCADE,
    FOREIGN KEY(skill_id) REFERENCES skills(id) ON DELETE CASCADE
);
```

#### 7. `error_events`
Fine-grained diagnostic error records attached to specific practice attempts.
```sql
CREATE TABLE IF NOT EXISTS error_events (
    id TEXT PRIMARY KEY,            -- Strong ErrorEventId (e.g. "err-1787580897448-01")
    attempt_id TEXT NOT NULL,       -- Foreign key referencing practice_attempts(id)
    error_category TEXT NOT NULL,   -- Error taxonomy category (e.g. "silly_mistake", "pattern_not_recognized")
    details TEXT NOT NULL,          -- JSON diagnostic details (step_index, expected vs actual, malrule)
    occurred_at INTEGER NOT NULL,   -- UTC epoch timestamp
    FOREIGN KEY(attempt_id) REFERENCES practice_attempts(id) ON DELETE CASCADE
);
```

---

### 4.4 Catalog & Exam Content Engine (v2 & v3)

#### 8. `catalog_metadata` (v2)
Stores catalog schema versioning and bootstrap synchronization timestamps.
```sql
CREATE TABLE IF NOT EXISTS catalog_metadata (
    key TEXT PRIMARY KEY,           -- Metadata key (e.g. "catalog_version", "last_bootstrap_at")
    value TEXT NOT NULL,            -- Metadata string value
    updated_at INTEGER NOT NULL     -- UTC epoch milliseconds
);
```

#### 9. `pyq_sources` (v3)
Stores authentic Previous Year Questions (PYQs) from benchmark competitive examinations (JEE, CAT, etc.).
```sql
CREATE TABLE IF NOT EXISTS pyq_sources (
    id TEXT PRIMARY KEY,            -- Strong PyqId (e.g. "pyq-jee-main-2023-s1-q42")
    exam TEXT NOT NULL,             -- Exam name (e.g. "JEE Main", "CAT Quant")
    year INTEGER NOT NULL,          -- Examination year
    paper TEXT,                     -- Paper / Session identifier
    shift TEXT,                     -- Shift name
    session TEXT,                   -- Examination session
    domain TEXT NOT NULL,           -- Domain enum: "mathematics", "physics", etc.
    original_question TEXT NOT NULL,-- Authentic raw question text with LaTeX
    original_options TEXT,          -- JSON array of options (for MCQ questions)
    original_answer TEXT NOT NULL,  -- Canonical answer string
    source_reference TEXT NOT NULL, -- Official citation / key reference
    provenance TEXT NOT NULL,       -- JSON ContentProvenance object
    source_version INTEGER NOT NULL,-- Ingestion format version
    import_timestamp INTEGER NOT NULL, -- UTC epoch timestamp
    metadata TEXT NOT NULL          -- JSON object
);
```

#### 10. `pyq_mappings` (v3)
Maps authentic PYQs to StudyLab skill nodes, schemas, and parameter domains.
```sql
CREATE TABLE IF NOT EXISTS pyq_mappings (
    pyq_id TEXT PRIMARY KEY,        -- Foreign key referencing pyq_sources(id)
    domain TEXT NOT NULL,           -- Domain identifier
    skill_id TEXT NOT NULL,         -- Target SkillId
    schema_id TEXT NOT NULL,        -- Target SchemaId
    problem_family_id TEXT NOT NULL,-- Target ProblemFamilyId
    variant_structure TEXT,         -- Identified structural template pattern
    difficulty_level INTEGER NOT NULL, -- Calibrated difficulty (1 to 5)
    target_latency_ms INTEGER NOT NULL,-- Exam target time budget
    diagnostic_metadata TEXT NOT NULL, -- JSON diagnostic classification rules
    status TEXT NOT NULL,           -- Mapping status: "Verified", "Candidate", "Deprecated"
    confidence TEXT NOT NULL,       -- Confidence tier: "High", "Medium", "Low"
    reviewer_notes TEXT,            -- Human expert curation notes
    updated_at INTEGER NOT NULL,    -- UTC epoch timestamp
    FOREIGN KEY(pyq_id) REFERENCES pyq_sources(id) ON DELETE CASCADE
);
```

#### 11. `rejected_variants` (v3)
Audit trail recording generative variants that failed validation checks during catalog generation.
```sql
CREATE TABLE IF NOT EXISTS rejected_variants (
    id TEXT PRIMARY KEY,            -- Strong RejectedVariantId
    source_pyq_id TEXT,             -- Optional parent PyqId
    schema_id TEXT NOT NULL,        -- Target SchemaId
    family_id TEXT NOT NULL,        -- Target ProblemFamilyId
    seed INTEGER NOT NULL,          -- RNG seed producing the defect
    variant_type TEXT NOT NULL,     -- Variant category ("Isomorphic", "Transfer", etc.)
    failure_reason TEXT NOT NULL,   -- Reason: "UnphysicalQuantity", "NonIntegerRoot", "ConstraintViolation"
    generated_instance_json TEXT NOT NULL, -- Full JSON dump of defective instance
    rejected_at INTEGER NOT NULL    -- UTC epoch timestamp
);
```

#### 12. `exam_profiles` (v3)
Defines target examination configurations, domain weightings, and pacing targets.
```sql
CREATE TABLE IF NOT EXISTS exam_profiles (
    id TEXT PRIMARY KEY,            -- Strong ExamProfileId (e.g. "exam-jee-main")
    name TEXT NOT NULL,             -- Display name
    description TEXT NOT NULL,      -- Exam overview
    subjects TEXT NOT NULL,         -- JSON array of included subjects
    domain_weights TEXT NOT NULL,   -- JSON map of domain sampling weights
    topic_weights TEXT NOT NULL,    -- JSON map of high-yield topic weights
    preferred_formats TEXT NOT NULL,-- JSON array of question formats
    target_latencies_ms TEXT NOT NULL, -- JSON map of target latencies per difficulty tier
    difficulty_distribution TEXT NOT NULL, -- JSON distribution (L1..L5 target percentages)
    pyq_weight REAL NOT NULL,       -- PYQ sampling probability (0.0 to 1.0)
    objective TEXT NOT NULL,        -- JSON ExamObjective definition
    metadata TEXT NOT NULL,         -- JSON object
    created_at INTEGER NOT NULL     -- UTC epoch milliseconds
);
```

---

### 4.5 Practice Content Layer & Chapter Capabilities (v4)

#### 13. `practice_items` (v4)
Canonical database of source-backed and synthesized practice items.
```sql
CREATE TABLE IF NOT EXISTS practice_items (
    id TEXT PRIMARY KEY,            -- Strong PracticeItemId (e.g. "pi-lcm-001")
    origin TEXT NOT NULL,           -- JSON tagged Origin enum (AuthenticPyq, CuratedSource, DerivedVariant, SyntheticSchema)
    domain TEXT NOT NULL,           -- Academic domain
    chapter TEXT NOT NULL,          -- Chapter name (e.g. "Percentages", "Kinematics")
    skill_id TEXT NOT NULL,         -- Target SkillId
    schema_id TEXT NOT NULL,        -- Target SchemaId
    problem_family_id TEXT NOT NULL,-- Target ProblemFamilyId
    question_type TEXT NOT NULL,    -- JSON tagged QuestionType (Mcq, Numerical, Structured, ReferenceOnly)
    prompt TEXT NOT NULL,           -- Formatted question prompt
    difficulty REAL NOT NULL,       -- Calibrated continuous difficulty (1.0 to 5.0)
    structural_tags TEXT NOT NULL,  -- JSON array of structural concept tags
    decision_points TEXT NOT NULL,  -- JSON array of CognitiveDecisionPoints
    error_categories TEXT NOT NULL, -- JSON array of likely error classifications
    prerequisites TEXT NOT NULL,    -- JSON array of prerequisite SkillIds
    provenance TEXT NOT NULL,       -- JSON ContentProvenance
    created_at INTEGER NOT NULL,    -- UTC epoch milliseconds
    metadata TEXT NOT NULL          -- JSON object
);
```

#### 14. `chapter_practice_profiles` (v4)
Defines chapter-level capabilities, generator support levels, and recognition signals.
```sql
CREATE TABLE IF NOT EXISTS chapter_practice_profiles (
    chapter_name TEXT PRIMARY KEY,  -- Chapter identifier (e.g. "Percentages")
    domain TEXT NOT NULL,           -- Academic domain
    supported_schemas TEXT NOT NULL,-- JSON array of supported SchemaIds
    supported_problem_families TEXT NOT NULL, -- JSON array of supported ProblemFamilyIds
    generator_capabilities TEXT NOT NULL, -- JSON map (Full, Partial, SourceOnly)
    recognition_signals TEXT NOT NULL, -- JSON array of problem recognition signals
    decision_points TEXT NOT NULL,  -- JSON array of chapter decision points
    variation_dimensions TEXT NOT NULL, -- JSON array of active variation dimensions
    prerequisites TEXT NOT NULL,    -- JSON array of prerequisite skills
    error_categories TEXT NOT NULL, -- JSON array of domain error categories
    exam_relevance TEXT NOT NULL,   -- JSON map of relevance scores per exam
    created_at INTEGER NOT NULL,    -- UTC epoch milliseconds
    metadata TEXT NOT NULL          -- JSON object
);
```

---

### 4.6 Durable Remediation Queue & Recurrence Tracker (v5)

#### 15. `remediation_queue_items` (v5)
Durable priority queue for scheduled just-in-time pedagogical interventions.
```sql
CREATE TABLE IF NOT EXISTS remediation_queue_items (
    id TEXT PRIMARY KEY,            -- Strong RemediationItemId UUID
    kind TEXT NOT NULL,             -- Action kind: "ConceptCheck", "StrategyDrill", "WorkedExample", "PrerequisiteReview", "CircuitBreaker", etc.
    skill_id TEXT NOT NULL,         -- Target SkillId
    schema_id TEXT NOT NULL,        -- Target SchemaId
    domain TEXT NOT NULL,           -- Academic domain
    primary_error TEXT NOT NULL,    -- Error category triggering remediation
    step_error TEXT,                -- Optional step-level error category
    preferred_difficulty INTEGER NOT NULL, -- Difficulty level (1 to 5)
    preferred_variant TEXT,         -- Target variant category
    source_attempt_id TEXT NOT NULL,-- AttemptId triggering the item
    urgency TEXT NOT NULL,          -- Urgency tier: "Critical", "High", "Normal", "Low"
    requires_acknowledgement INTEGER NOT NULL, -- Boolean flag (1 = mandatory confirmation)
    recurrence_count INTEGER NOT NULL, -- Current consecutive failure recurrence
    rationale TEXT NOT NULL,        -- Pedagogical explanation for intervention
    created_at INTEGER NOT NULL,    -- UTC epoch milliseconds
    FOREIGN KEY(skill_id) REFERENCES skills(id) ON DELETE CASCADE,
    FOREIGN KEY(schema_id) REFERENCES schemas(id) ON DELETE CASCADE
);
```

#### 16. `remediation_recurrence` (v5)
Tracks error category recurrence per skill to drive escalating interventions and circuit breaker loops.
```sql
CREATE TABLE IF NOT EXISTS remediation_recurrence (
    skill_id TEXT NOT NULL,         -- Target SkillId
    error_category TEXT NOT NULL,   -- Error category string
    count INTEGER NOT NULL,         -- Recurrence counter
    updated_at INTEGER NOT NULL,    -- UTC epoch timestamp
    PRIMARY KEY(skill_id, error_category)
);
```

---

## 5. Comprehensive Index Catalog

The database maintains 17 dedicated indexes optimizing relationship traversal, temporal ordering, and queue prioritization:

| Index Name | Target Table | Indexed Columns | Query Optimization Purpose |
|---|---|---|---|
| `idx_skills_domain` | `skills` | `(domain)` | Fast domain filtering during diagnostic sampling. |
| `idx_families_skill` | `problem_families` | `(skill_id)` | Foreign key joins from skills to problem families. |
| `idx_schemas_skill` | `schemas` | `(skill_id)` | Resolves active schemas for a specific skill. |
| `idx_instances_family` | `problem_instances` | `(family_id)` | Traverses generated instances per family. |
| `idx_attempts_schema` | `practice_attempts` | `(schema_id)` | Calculates schema-level accuracy metrics. |
| `idx_attempts_skill` | `practice_attempts` | `(skill_id)` | Computes skill-level mastery and sliding windows. |
| `idx_attempts_card` | `practice_attempts` | `(card_id)` | Links Anki card review requests to procedural history. |
| `idx_attempts_time` | `practice_attempts` | `(attempted_at)` | Temporal ordering for retention and latency tracking. |
| `idx_error_events_attempt` | `error_events` | `(attempt_id)` | Cascading fetch of error diagnostics per attempt. |
| `idx_skill_states_updated` | `skill_states` | `(updated_at)` | Identifies stale skills for spaced review sweeps. |
| `idx_pyq_sources_exam` | `pyq_sources` | `(exam)` | Filters authentic questions by target exam profile. |
| `idx_pyq_sources_domain` | `pyq_sources` | `(domain)` | Filters PYQ sources across academic domains. |
| `idx_pyq_mappings_schema` | `pyq_mappings` | `(schema_id)` | Resolves PYQ templates for a practice schema. |
| `idx_pyq_mappings_status` | `pyq_mappings` | `(status)` | Selects verified PYQ mappings for production decks. |
| `idx_pyq_mappings_confidence` | `pyq_mappings` | `(confidence)` | Prioritizes high-confidence mappings. |
| `idx_rejected_variants_pyq` | `rejected_variants` | `(source_pyq_id)` | Audits defect generation per source PYQ. |
| `idx_rejected_variants_family` | `rejected_variants` | `(family_id)` | Identifies problem families with generator defects. |
| `idx_practice_items_schema` | `practice_items` | `(schema_id)` | Ingestion retrieval of practice items per schema. |
| `idx_practice_items_family` | `practice_items` | `(problem_family_id)` | Ingestion retrieval of practice items per family. |
| `idx_practice_items_chapter` | `practice_items` | `(chapter)` | Groups practice items by chapter for batch export. |
| `idx_remediation_queue_skill` | `remediation_queue_items` | `(skill_id)` | Enables same-skill queue compaction. |
| `idx_remediation_queue_urgency`| `remediation_queue_items` | `(urgency)` | Priority queue extraction of critical remediation items. |

---

## 6. Atomic Transaction Lifecycles

### 6.1 Atomic Practice Attempt Ingestion (`record_practice_attempt_atomic`)
Implemented in `ProceduralStore::record_practice_attempt_atomic` (`store.rs:743-890`), this method executes in a single SQLite transaction boundary:

```text
 1. BEGIN TRANSACTION (tx = conn.transaction()?)
 2. SELECT FROM skill_states WHERE skill_id = ?1
 3. In-Memory EMA Mastery & Signal Processing:
    - Mastery_t = 0.8 * Mastery_{t-1} + 0.2 * Outcome
    - Update MovingLatencyStats & ErrorFrequencyCounts
    - Evaluate 6-Gate Progression State Promotion/Demotion
 4. INSERT INTO practice_attempts (attempt_id, instance_id, schema_id, skill_id, ...)
 5. INSERT INTO error_events (for each diagnostic error event)
 6. UPSERT INTO skill_states (updated mastery, confidence, custom_state JSON, updated_at)
 7. COMMIT TRANSACTION (tx.commit()?)
```

If any step fails (e.g. disk full or constraint violation), the entire transaction rolls back cleanly, leaving `skill_states` and `practice_attempts` in an uncorrupted state.

### 6.2 Diagnostic Session Batch Synchronization
`ProceduralStore::record_diagnostic_report_evidence` (`mock.rs:855-900`) batch-synchronizes diagnostic outcomes across 10–20 questions in a single transaction, updating `skill_states` across all 4 tested domains simultaneously.

---

## 7. SQL Query Parameterization & Injection Defense

To completely eliminate SQL injection vulnerabilities:
- **100% of SQL statements** in `rslib/procedural/src/storage/` use positional parameter placeholders (`?1, ?2, ...`) or `rusqlite::params![...]`.
- No raw user strings or JSON payloads are ever concatenated into SQL query strings.
- All JSON objects are strictly serialized via `serde_json::to_string()` prior to parameter binding.
