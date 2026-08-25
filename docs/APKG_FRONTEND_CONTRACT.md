# StudyLab Cross-Layer Data Flow & Pipeline Contract
## APKG ➔ Backend (Rust) ➔ Database (SQLite) ➔ Desktop Host (Python/Qt) ➔ Reviewer UI (TypeScript)

**Document Version:** 1.0.0 (Canonical)  
**Target Subsystems:** Note Storage (`collection.anki2`), Procedural Core (`rslib/procedural/`), SQLite Store (`procedural.db`), Python Desktop Host (`qt/aqt/reviewer.py`), TypeScript Reviewer Surface (`ts/reviewer/`)  
**Status:** AUTHORITATIVE CANONICAL SPECIFICATION  
**Integrity Mode:** 100% Grounded in Executable Source Code, Tests, and Bridge Protocol Handlers  

---

## 1. End-to-End Cross-Layer Architecture

StudyLab operates a 4-tier pipeline that spans from offline Anki package definitions (`.apkg`), through the compiled native Rust engine, across thread-safe SQLite persistence, through the Python/Qt desktop bridge, and into the interactive TypeScript webview surface:

```text
┌────────────────────────────────────────────────────────────────────────────────────────┐
│ 1. APKG NOTE LAYER (collection.anki2)                                                  │
│    - Note Type: "StudyLab Procedural Anchor"                                           │
│    - Field 0: ProceduralPayload JSON (inline_contract, seed_mode, difficulty_override) │
│    - Fields 1..5: TopicTitle, Domain, Provenance, Difficulty, LearningObjectType       │
└──────────────────────────────────────────┬─────────────────────────────────────────────┘
                                           │
                                           │ Note fields extracted via render.rs
                                           ▼
┌────────────────────────────────────────────────────────────────────────────────────────┐
│ 2. RUST BACKEND ENGINE (rslib/procedural)                                              │
│    - Deserialization: ProceduralCardAnchor::extract_from_card_fields()                │
│    - 3-Tier Target Resolution: inline_contract -> content_ref -> proc_schema           │
│    - Problem Generation: DeclarativeProblemGenerator::generate(seed)                   │
│    - Instance Sampling & Derivation Evaluation: ProblemInstance + SolutionGraph        │
│    - HTML/JS Template Mounting: render_reviewer_html(&session) with Anti-Cheat Filter │
└────────────────────┬──────────────────────────────────────────────┬────────────────────┘
                     │                                              │
      Write Attempt  │                                Render HTML   │ Webview Mount
      & Error State  │                                Context        │
                     ▼                                              ▼
┌────────────────────────────────────────┐     ┌─────────────────────────────────────────┐
│ 3. SQLITE PERSISTENCE (procedural.db)  │     │ 4. DESKTOP HOST & WEBVIEW UI            │
│    - practice_attempts (Immutable Log) │     │    - Desktop Host (qt/aqt/reviewer.py)  │
│    - skill_states (EMA Mastery Updates)│     │      Bridge command routing (_linkHand) │
│    - error_events (Diagnostic Records) │     │    - Webview UI (ts/reviewer/procedural)│
│    - remediation_queue_items (Priority)│     │      Interactive state machine & widgets│
└────────────────────────────────────────┘     └────────────────────┬────────────────────┘
                                                                    │
                                     Bridge Commands via IPC        │
                                     (procedural_attempt, hint, ...)│
                                                                    ▼
                                               [Return to Layer 2 & 3 for Ingestion]
```

---

## 2. Cross-Layer Field Mapping Matrix

The following matrix provides an exhaustive, field-by-field trace across all 4 architectural layers. It documents how properties originate in the APKG note, deserialize in Rust, persist to SQLite, bridge across Python, and render in TypeScript:

| Canonical Property | Layer 1: APKG Note (`collection.anki2`) | Layer 2: Rust Engine (`rslib/procedural`) | Layer 3: SQLite Store (`procedural.db`) | Layer 4a: Python Host (`reviewer.py`) | Layer 4b: TypeScript UI (`ts/reviewer`) |
|---|---|---|---|---|---|
| **Problem Family ID** | `ProceduralPayload.inline_contract.contract.family_id` | `session.instance.family_id` (`ProblemFamilyId`) | `problem_families.id`<br>`practice_attempts.metadata.family_id` | `_last_procedural_attempt.family_id` | `options.familyId`<br>`attempt.familyId` |
| **Skill ID** | `ProceduralPayload.inline_contract.contract.skill_id` | `session.schema.skill_id` (`SkillId`) | `skills.id`<br>`skill_states.skill_id`<br>`practice_attempts.skill_id` | `_last_procedural_attempt.skill_id` | `options.skillId`<br>`attempt.skillId` |
| **Schema ID** | `ProceduralPayload.proc_schema` | `session.schema.id` (`SchemaId`) | `schemas.id`<br>`practice_attempts.schema_id` | `_last_procedural_attempt.schema_id` | `options.schemaId`<br>`attempt.schemaId` |
| **Instance ID** | Generated at runtime (Ephemeral) | `session.instance.id` (`ProblemInstanceId`) | `problem_instances.id`<br>`practice_attempts.instance_id` | `_last_procedural_attempt.instance_id` | `options.instanceId`<br>`attempt.instanceId` |
| **Domain / Discipline**| `Domain` field (ord 2) or `contract.domain` | `session.schema.domain` (`Domain`) | `skills.domain`<br>`practice_items.domain` | `_last_procedural_attempt.domain` | `options.domain`<br>Domain badge |
| **RNG Seed Mode** | `ProceduralPayload.seed_mode` (`"random"`, `{"fixed": N}`) | `anchor.seed_mode` (`SeedMode`) | `problem_instances.seed` | N/A (Internal engine state) | N/A (Embedded in instance ID) |
| **Difficulty Level** | `Difficulty` field (ord 4) or `contract.min_difficulty` | `session.difficulty_level` (1..5) | `practice_items.difficulty`<br>`practice_attempts.metadata.difficulty`| Displayed in HUD tooltip | `options.difficultyLevel`<br>Difficulty pill badge |
| **Learning Modality** | `LearningObjectType` field (ord 5) or `archetype.object_type` | `object_type` (`"problem"`, `"mcq"`, `"stepwise"`, `"worked_example"`) | `practice_items.question_type` | Routes interaction mode | `options.objectType`<br>Mounts active container |
| **Question Prompt** | `archetype.prompt_template` | `session.instance.rendered_prompt` (Interpolated + LaTeX) | `problem_instances.rendered_prompt`<br>`practice_items.prompt` | Rendered into webview HTML | `#proc-prompt` HTML element |
| **Options / Distractors**| `archetype.parameters.options` (for MCQ) | `session.instance.parameters["options"]` | `practice_items.decision_points` (JSON) | Rendered into option buttons | `.proc-option-btn` group |
| **Target Time (ms)** | `archetype.target_time_ms` or `contract.target_latency_model` | `session.target_latency_ms` (u64) | `pyq_mappings.target_latency_ms`<br>`exam_profiles.target_latencies_ms` | N/A | `options.targetTimeMs`<br>Pacing indicator |
| **Progressive Hints** | `archetype.step_nodes[i].hint_*` | `session.instance.metadata["hints"]` (Sanitized tiers) | `practice_attempts.metadata.hints_used` | `_on_procedural_hint` | `options.hints`<br>`#proc-hint-container` |
| **Canonical Solution** | `archetype.solution_template` | `session.solution_text` (Sanitized / Gated) | `problem_instances.correct_answer` | Revealed post-submission | `#proc-solution-container` (Hidden during solve) |
| **Step Validation Graph**| `archetype.step_nodes` | `session.solution_graph` (`StepValidationGraph`) | `practice_items.decision_points` | `_on_procedural_validate_steps` | `StepwiseContainer.evaluateSteps()` |
| **Provenance Citation**| `Provenance` field (ord 3) or `contract.provenance` | `session.provenance` (`ContentProvenance`) | `pyq_sources`<br>`practice_items.provenance` | Displayed in provenance modal | `options.provenance`<br>Provenance footer link |
| **User Answer** | Captured at runtime (Frontend) | Deserialized in `AttemptResultPayload` | `practice_attempts.user_answer` (JSON) | `_last_procedural_attempt.answer` | User input string / selection |
| **Time Taken (ms)** | Measured at runtime (Frontend) | Deserialized in `AttemptResultPayload` | `practice_attempts.time_taken_ms` | `_last_procedural_attempt.time_taken_ms` | Stopwatch timer (`timeTakenMs`) |
| **Correctness & Score**| Evaluated at runtime (Frontend/Engine) | `is_correct` (bool), `score` (0.0..1.0) | `practice_attempts.is_correct`<br>`practice_attempts.score` | `_last_procedural_attempt.is_correct` | Result banner state (`score`) |
| **Speed Quadrant** | Classified at runtime (Frontend) | Deserialized in `AttemptResultPayload` | `practice_attempts.metadata.speed_quadrant` | `_last_procedural_attempt.speed_quadrant` | Evaluated against target time |
| **Mistake Category** | Reflected at runtime (Learner 1-4) | Deserialized in `MistakeSelectionPayload`| `practice_attempts.metadata.mistake_type`<br>`error_events.error_category` | `_last_procedural_mistake.mistake_type` | Mistake reflection buttons (1-4) |
| **FSRS Ease Rating** | Derived at runtime (calibrated 1..4) | Calibrated by `ProceduralReviewer` | `revlog.ease` (in `collection.anki2`) | `_answerCard(val)` execution | Ease button mutation / Next trigger |

---

## 3. Field Ownership & Lifecycle Across All 4 Layers

To prevent synchronization drift, each field has a strictly defined single source of truth and clear immutability rules:

```text
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                                FIELD OWNERSHIP & LIFECYCLE                             │
├────────────────────┬──────────────────┬─────────────────┬──────────────────────────────┤
│ Subsystem / Layer  │ Fields Owned     │ Mutability      │ Lifecycle State              │
├────────────────────┼──────────────────┼─────────────────┼──────────────────────────────┤
│ **APKG Note**      │ `proc_schema`,   │ Read-Only       │ Immutable on disk; created   │
│                    │ `inline_contract`│ Blueprint       │ at authoring time.           │
├────────────────────┼──────────────────┼─────────────────┼──────────────────────────────┤
│ **Rust Engine**    │ `instance_id`,   │ Ephemeral       │ Instantiated per review;     │
│                    │ `rendered_prompt`│ Generation      │ discarded after render.      │
├────────────────────┼──────────────────┼─────────────────┼──────────────────────────────┤
│ **TypeScript UI**  │ `user_answer`,   │ Active Session  │ Collected during interaction;│
│                    │ `time_taken_ms`  │ State           │ dispatched via bridge.       │
├────────────────────┼──────────────────┼─────────────────┼──────────────────────────────┤
│ **SQLite DB**      │ `mastery`,       │ Durable Append/ │ Committed in atomic ACID     │
│                    │ `attempts`, `err`│ Update          │ transactions upon solve.     │
└────────────────────┴──────────────────┴─────────────────┴──────────────────────────────┘
```

### 3.1 Ownership Invariants
1. **APKG Notes are Read-Only Templates:** Notes stored in `collection.anki2` are never mutated during review. They serve purely as declarative blueprints.
2. **Problem Instances are Deterministically Reproducible:** Concrete instances sampled in memory can be reproduced exactly from `(archetype, seed)`.
3. **Telemetry is Append-Only:** Rows in `practice_attempts` and `error_events` are strictly immutable once written.
4. **Mastery Updates are Transactional:** Updates to `skill_states.mastery` and `custom_state` occur exclusively through atomic SQLite transactions executing EMA equations.

---

## 4. Ephemeral Backend Sanitization & Anti-Cheating Boundary

To prevent learners from inspecting DOM developer tools to reveal answers or circumvent the diagnostic engine, the Rust backend enforces strict **Ephemeral Backend Sanitization** prior to mounting the webview:

```text
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                         EPHEMERAL BACKEND SANITIZATION BOUNDARY                        │
├────────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                        │
│   [Full Procedural Session Object (Rust Memory)]                                       │
│   ├── Instance ID, Family ID, Skill ID, Schema ID                                      │
│   ├── Parameters Map & Interpolated Prompt (Safe)                                      │
│   ├── Raw Solution Graph & Multi-Step Derivation (Sensitive)                           │
│   ├── Correct Answer String / Numerical Float (Sensitive)                             │
│   ├── Step Validation Rules & Malrules (Sensitive)                                     │
│   └── Pedagogical Diagnostic Scoring Keys (Sensitive)                                  │
│                                                                                        │
│                                           │                                            │
│                                           ▼                                            │
│                          [Sanitization & Stripping Filter]                             │
│                                           │                                            │
│         ┌─────────────────────────────────┴─────────────────────────────────┐          │
│         ▼                                                                   ▼          │
│   [Injected to Client Webview]                                [Retained in Backend]    │
│   • Sanitized Prompt & MathJax                                • Canonical Answer Key   │
│   • Parameter Map & Distractor Array                          • AST Equivalence Engine │
│   • Tier 1 & Tier 2 Hint Strings                              • Detailed Error Malrules│
│   • Target Latency & Difficulty Tier                          • Raw Mastery Equations  │
│   • Solution Container (CSS Display: None)                    • Multi-Step Graph Rules │
│                                                                                        │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

### 4.1 Client-Visible vs Backend-Restricted Data

| Data Element | Injected in `options` JSON? | DOM Visibility During Solve | Post-Submission Visibility |
|---|---|---|---|
| **`rendered_prompt`** | Yes | Visible (`#proc-prompt`) | Visible |
| **`options` (MCQ)** | Yes | Visible (`.proc-option-btn`) | Highlighted (Green/Red) |
| **`target_time_ms`** | Yes | Hidden (Pacing timer active) | Revealed in Pacing Pill |
| **`hint_principle`** | Yes | Hidden until requested | Revealed upon request |
| **`hint_operation`** | Yes | Hidden until requested | Revealed upon request |
| **`hint_intermediate`** | Yes | Hidden until requested | Revealed upon request |
| **`solution_text`** | Sanitized in HTML | Hidden (`display: none`) | Revealed (`#proc-solution-container`) |
| **`correct_answer`** | Gated / Sanitized | Hidden | Revealed in Solution Body |
| **`solution_graph`** | Validated on Backend | Hidden | Step-by-step review |
| **`malrules`** | No (Backend only) | Never injected | Error category tag |
| **`mastery_delta`** | No (DB transaction) | Never injected | Updated in profile |

---

## 5. Complete Bridge Command Protocol Catalog

Communication between the TypeScript webview and Anki's Python desktop host flows exclusively via the `bridgeCommand("<command>")` protocol (`qt/aqt/reviewer.py:697-825`).

### 5.1 Protocol Command Catalog

| Command Protocol Signature | Sender Subsystem | Python Link Receiver | Payload Schema | Operational Side Effect |
|---|---|---|---|---|
| **`procedural_attempt:<json>`** | `ProceduralReviewer.finishAttempt()` | `_on_procedural_attempt` (`reviewer.py:783`) | `AttemptResultPayload` | Stores attempt snapshot in `self._last_procedural_attempt`; triggers atomic persistence; transitions review state to `"answer"`. |
| **`procedural_hint:<json>`** | `ProceduralReviewer.requestHint()` | `_on_procedural_hint` (`reviewer.py:779`) | `HintRequestPayload` | Stores hint telemetry in `self._last_procedural_hint`; tracks hint exposure level and elapsed time. |
| **`procedural_validate_steps:<json>`** | `StepwiseContainer.evaluateSteps()` | `_on_procedural_validate_steps` (`reviewer.py:775`) | `StepwiseValidationPayload` | Passes intermediate algebraic steps to Rust `StepValidator` for AST equivalence and malrule classification. |
| **`procedural_mistake:<json>`** | `ProceduralReviewer.selectMistakeCategory()` | `_on_procedural_mistake` (`reviewer.py:790`) | `MistakeSelectionPayload` | Captures learner reflection (1-4) and logs categorized record into `error_events` in `procedural.db`. |
| **`procedural_try_similar:<json>`** | `ProceduralReviewer.handleTrySimilar()` | `_on_procedural_try_similar` (`reviewer.py:794`) | `TrySimilarPayload` | Displays tooltip `"Generating similar variant..."`; re-samples seed and triggers `self._showQuestion()`. |
| **`procedural_practice_prerequisite:<json>`** | `ProceduralReviewer.handlePracticePrerequisite()` | `_on_procedural_practice_prerequisite` (`reviewer.py:803`) | `PrerequisitePracticePayload` | Displays tooltip `"Practice Prerequisite: {skill}"`; queues remedial item in `remediation_queue_items`. |
| **`procedural_declarative_recall:<json>`** | `ProceduralReviewer.handleDeclarativeRecallAction()` | `_on_procedural_declarative_recall` (`reviewer.py:810`) | `DeclarativeRecallPayload` | Resolves associated standard Anki card in `collection.anki2` and triggers focused recall card review. |
| **`statesMutated`** | State Mutation Closure | `_linkHandler` (`reviewer.py:722`) | None | Sets `self._states_mutated = True`, unblocking deferred ease button rendering. |
| **`procedural_answer:<ease>`** | `ProceduralReviewer.handleNext()` | `_linkHandler` (`reviewer.py:703`) | None (Integer `<ease>` in URL: 1..4) | Invokes `self._answerCard(val)` to execute standard Anki FSRS card scheduling and advance to next card. |

---

## 6. Telemetry Ingestion & In-Memory to DB Flow

When the learner solves a problem, telemetry flows across a structured 7-step atomic pipeline:

```text
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                         7-STEP ATOMIC TELEMETRY INGESTION TRACE                        │
├────────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                        │
│  1. [TypeScript Interaction]                                                           │
│     • Student submits answer; timer stops: `timeTakenMs = 21450`.                      │
│     • Speed quadrant evaluated: `"fluency_strength"`.                                 │
│     • `AttemptResultPayload` dispatched via `bridgeCommand("procedural_attempt:...")`. │
│                                                                                        │
│  2. [Python Desktop Bridge]                                                            │
│     • `reviewer.py:_handle_procedural_command` intercepts payload.                     │
│     • Passes payload across PyO3 FFI boundary to Rust `ProceduralService`.             │
│                                                                                        │
│  3. [Rust Atomic Transaction Ingestion (`record_practice_attempt_atomic`)]             │
│     • BEGIN TRANSACTION (`tx = conn.transaction()?`).                                  │
│     • Ingests into `practice_attempts` table.                                          │
│                                                                                        │
│  4. [EMA Mastery Calculation]                                                          │
│     • Loads existing state from `skill_states`.                                        │
│     • Computes Exponential Moving Average:                                             │
│       $$\text{Mastery}_t = 0.8 \cdot \text{Mastery}_{t-1} + 0.2 \cdot \text{Outcome}$$ │
│     • Updates moving latency statistics and error frequency counters.                  │
│                                                                                        │
│  5. [Diagnostic Error Logging]                                                         │
│     • If incorrect or mistake classified, inserts row into `error_events`.             │
│     • If consecutive failures occur, inserts item into `remediation_queue_items`.      │
│                                                                                        │
│  6. [Commit Transaction]                                                               │
│     • `tx.commit()?` guarantees ACID durability.                                       │
│                                                                                        │
│  7. [FSRS Card Rescheduling]                                                           │
│     • Bridge triggers `self._answerCard(ease)` to update Anki's native FSRS revlog.    │
│                                                                                        │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 7. FSRS & Native Anki Scheduling Integration

StudyLab seamlessly maps procedural performance metrics into Anki's native FSRS (Free Spaced Repetition Scheduler) / SM-2 rating tiers:

```text
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                         PROCEDURAL TO FSRS RATING CALIBRATION                          │
├─────────┬──────────────┬───────────────────────────────────────────────────────────────┤
│ FSRS    │ Button Label │ Procedural Calibration Logic                                  │
├─────────┼──────────────┼───────────────────────────────────────────────────────────────┤
│ **1**   │ **Again**    │ • Incorrect answer (`score == 0.0`) OR                        │
│         │              │ • Concept Gap / Prereq Unknown mistake reflection OR          │
│         │              │ • $\ge 3$ hints used during solving.                          │
├─────────┼──────────────┼───────────────────────────────────────────────────────────────┤
│ **2**   │ **Hard**     │ • Correct solve but latency $> 1.5 \times$ target time OR     │
│         │              │ • Correct solve with 1–2 hints used OR                        │
│         │              │ • Silly slip / pattern missed reflection.                     │
├─────────┼──────────────┼───────────────────────────────────────────────────────────────┤
│ **3**   │ **Good**     │ • Correct solve within normal time budget ($0.8\text{x}$ to   │
│         │              │   $1.5\text{x}$ target) with zero hints used.                 │
├─────────┼──────────────┼───────────────────────────────────────────────────────────────┤
│ **4**   │ **Easy**     │ • Flawless rapid solve ($< 0.8\text{x}$ target time) with      │
│         │              │   zero hints used and high mastery progression state.         │
└─────────┴──────────────┴───────────────────────────────────────────────────────────────┘
```

---

## 8. Summary & Acceptance Checklist

| Pipeline Stage | Operational Requirement | Conformance Proof |
|---|---|---|
| **Layer 1 ➔ 2** | `ProceduralPayload` extracted via `render_procedural_anchor` | `rslib/src/notetype/render.rs:122` |
| **Layer 2 ➔ 3** | Atomic persistence in `procedural.db` via WAL transactions | `rslib/procedural/src/storage/store.rs:743` |
| **Layer 2 ➔ 4** | HTML/JS mounting with sanitized payloads | `rslib/procedural/src/reviewer/template.rs:35` |
| **Layer 4 ➔ 4a** | Bridge command dispatch via `bridgeCommand` IPC | `qt/aqt/reviewer.py:697` |
| **Layer 4a ➔ 2** | PyO3 telemetry forwarding to Rust core engine | `rslib/procedural/src/service/mod.rs` |
| **Layer 4a ➔ 1** | Native FSRS card answering via `_answerCard` | `qt/aqt/reviewer.py:703` |
| **Anti-Cheat Gate** | Ephemeral stripping of solutions and diagnostic scoring keys | Verified in `render_reviewer_html` |
