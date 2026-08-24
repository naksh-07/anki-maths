# StudyLab Product Boundaries & Decoupling Architecture

**Document Version:** 1.0.0 (Canonical Master Specification)  
**Target Repository:** `Anki-maths` (StudyLab Procedural Intelligence Subsystem)  
**Status:** AUTHORITATIVE SPECIFICATION  
**Integrity Mode:** Benchmark Mode (100% Grounded in Executable Code, Tests, and Verified Integration Points)  

---

## 1. Executive Summary & Core Boundary Principle

StudyLab operates inside Anki via a strictly disciplined **host-guest integration pattern** (often termed the "Trojan-horse" architecture). 

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                            CORE BOUNDARY DIRECTIVE                               │
├──────────────────────────────────────────────────────────────────────────────────┤
│ "Anki is the host and integration environment, NOT StudyLab's product identity." │
│                                                                                  │
│ • Anki owns collection storage, user profiles, media sync, window lifecycle,    │
│   and temporal spaced-repetition scheduling (FSRS/SM-2).                         │
│ • StudyLab owns dynamic problem generation, step-level semantic validation,     │
│   multi-dimensional cognitive diagnostics, JIT remediation, and learner state.  │
│ • The two systems communicate through narrow, typed IPC bridges and ephemeral    │
│   telemetry envelopes with ZERO database cross-contamination.                    │
└──────────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Comprehensive Responsibility Matrix

The boundary between Anki and StudyLab is clean, comprehensive, and non-overlapping:

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                        SYSTEM BOUNDARY RESPONSIBILITY MAP                        │
├────────────────────────────┬──────────────────┬──────────────────┬───────────────┤
│ Functional Subsystem       │ Host SRS (Anki)  │ Procedural Engine│ Shared Bridge │
├────────────────────────────┼──────────────────┼──────────────────┼───────────────┤
│ Declarative Flashcards     │ **Primary Owner**│ —                │ —             │
│ Basic / Cloze Note Types   │ **Primary Owner**│ —                │ —             │
│ Reviewer Window & Desktop  │ **Primary Owner**│ —                │ Container DOM │
│ Spaced Interval Math (FSRS)│ **Primary Owner**│ —                │ Rating Map    │
│ `collection.anki2` SQLite  │ **Primary Owner**│ —                │ —             │
│ Sync & Media Server        │ **Primary Owner**│ —                │ —             │
│ Parametric Problem Gen     │ —                │ **Primary Owner**│ —             │
│ Stepwise Semantic Validator│ —                │ **Primary Owner**│ —             │
│ Multi-Domain Reasoning CSP │ —                │ **Primary Owner**│ —             │
│ Diagnostic Evidence Model  │ —                │ **Primary Owner**│ —             │
│ EMA Mastery & Progression  │ —                │ **Primary Owner**│ —             │
│ JIT Remediation Queue      │ —                │ **Primary Owner**│ —             │
│ `procedural.db` SQLite     │ —                │ **Primary Owner**│ —             │
│ Declarative Content Factory│ —                │ **Primary Owner**│ APKG Packager │
│ Card Scheduling Anchor     │ Note Type Record │ Payload Resolver │ Anki Note     │
│ Review Telemetry Pipeline  │ Answering Hook   │ Telemetry Parser │ Ephemeral JSON│
└────────────────────────────┴──────────────────┴──────────────────┴───────────────┘
```

---

## 3. The 3 Explicit Rust Integration Touchpoints

StudyLab connects to Anki's Rust core (`rslib/`) through exactly **three safe, explicit integration touchpoints**:

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                     THE 3 RUST BACKEND INTEGRATION TOUCHPOINTS                   │
├──────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│   1. Service Storage Initialization ────► `rslib/src/collection/mod.rs:141, 173` │
│      (Opens `<col_path>.procedural` SQLite database on demand)                   │
│                                                                                  │
│   2. Webview Card Render Hook ─────────► `rslib/src/notetype/render.rs:122, 199` │
│      (Intercepts `"StudyLab Procedural Anchor"` notes and renders webview HTML)  │
│                                                                                  │
│   3. Answer & Telemetry Pipeline ──────► `rslib/src/scheduler/answering/mod.rs`  │
│      (Extracts `studylab` JSON, updates `procedural.db`, strips custom data)     │
│                                                                                  │
└──────────────────────────────────────────────────────────────────────────────────┘
```

### Touchpoint 1: Storage Initialization (`rslib/src/collection/mod.rs`)
- **Code Ground Truth:** `rslib/src/collection/mod.rs:141, 173–183`.
- **Mechanism:** `Collection` contains an optional thread-safe reference:
  ```rust
  pub(crate) procedural_service: Option<Arc<procedural::service::ProceduralService>>,
  ```
- **Lifecycle:** When requested by the rendering or answering pipeline, `Collection::procedural_service()` lazily initializes:
  ```rust
  let path = self.col_path.with_extension("procedural");
  ProceduralService::open(path)
  ```
- **Isolation Guarantee:** The procedural store is kept in a separate file (`<collection_name>.procedural`), ensuring that Anki's core `collection.anki2` database handle is never shared or locked by procedural operations.

### Touchpoint 2: Card Rendering Interception (`rslib/src/notetype/render.rs`)
- **Code Ground Truth:** `rslib/src/notetype/render.rs:122–126, 199–240`.
- **Mechanism:** In `CardRenderContext::render()`, before compiling standard Mustache templates, the backend inspects the note type name:
  ```rust
  if nt.name.as_str().starts_with("StudyLab Procedural Anchor") && !browser {
      return self.render_procedural_anchor(note, card, nt);
  }
  ```
- **Execution:**
  1. `render_procedural_anchor` extracts `ProceduralCardAnchor` from `note.fields()` via `ProceduralCardAnchor::extract_from_card_fields()`.
  2. Resolves the target family, seed, and difficulty via `service.resolve_procedural_target(&anchor, Some(card.id.0))`.
  3. Generates complete HTML/JS/CSS webview content with MathJax scripts via `procedural::reviewer::render_reviewer_html(&session)`.
- **Non-Regression Guarantee:** Standard cards (`Basic`, `Cloze`, custom note types) evaluate `starts_with` to `false` and bypass this hook with zero overhead.

### Touchpoint 3: Answer Submission & Telemetry Pipeline (`rslib/src/scheduler/answering/mod.rs`)
- **Code Ground Truth:** `rslib/src/scheduler/answering/mod.rs:353–505`.
- **Mechanism:** When a learner submits a card in the reviewer, the TypeScript frontend merges telemetry into `custom_data`. Anki's answering engine intercepts this in `answer_card`:
  1. Detects `custom_data` containing `"proceduralRemediation"` or `"studylab"`.
  2. Deserializes the rich telemetry payload: `skill_id`, `schema_id`, `instance_id`, `time_taken_ms`, `error_category`, `domain_evidence`.
  3. Atomically commits the practice attempt and updates `SkillState` in `procedural.db` via `ProceduralStore::record_practice_attempt_atomic()`.
  4. Evaluates `RemediationPolicy` and queues follow-up interventions in `remediation_queue_items`.
  5. **Ephemeral Stripping:** Strips the `studylab` payload from `custom_data` before saving the card to `collection.anki2` to respect Anki's 100-byte database column limit.

---

## 4. Database Isolation Architecture

StudyLab enforces strict physical separation between Anki's collection database and StudyLab's procedural database:

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                         DATABASE DECOUPLING TOPOLOGY                             │
├────────────────────────────────────────┬─────────────────────────────────────────┤
│       ANKI DATABASE: `col.anki2`       │     STUDYLAB DATABASE: `col.procedural` │
├────────────────────────────────────────┼─────────────────────────────────────────┤
│ • Tables: `notes`, `cards`, `decks`,   │ • Tables: `skills`, `skill_states`,     │
│   `revlog`, `col`, `config`, `fields`  │   `problem_families`, `schemas`,        │
│ • Schema: Standard upstream Anki DDL   │   `problem_instances`, `practice_       │
│ • Sync: Synced via AnkiWeb protocol    │   attempts`, `error_events`,            │
│ • Storage: Traditional SQLite          │   `practice_items`, `remediation_queue` │
│ • Constraints: 100-byte `cards.data`   │ • Schema: v1–v5 migrations (11 tables)  │
│ • Invariant: ZERO StudyLab tables      │ • Pragmas: WAL mode, busy_timeout=5000  │
│   or columns ever added here           │ • Invariant: ZERO Anki core entities    │
└────────────────────────────────────────┴─────────────────────────────────────────┘
```

### Why Database Decoupling Is Non-Negotiable
1. **Sync Safety:** AnkiWeb uses optimized row-level hashing and binary sync protocols for `collection.anki2`. Injecting dynamic procedural attempt logs or multi-step graphs would corrupt sync performance and violate schema constraints.
2. **Schema Independence:** StudyLab can evolve its database schema (migrations v1 through v5 and beyond) without requiring coordinated Anki collection migrations or risking collection recovery failures.
3. **Zero Blast Radius:** If a user uninstalls StudyLab or opens their deck on a standard Anki desktop client, `collection.anki2` remains 100% valid and operational.

---

## 5. The 100-Byte Anki Custom Data Boundary

Anki's database schema enforces a strict size restriction on the `cards.data` column (`custom_data`), enforced by `card.validate_custom_data()`: **maximum 100 bytes**.

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                   THE EPHEMERAL TELEMETRY LIFECYCLE                              │
├──────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│   1. TS Reviewer Frontend (`ts/reviewer/answering.ts`)                           │
│      • Packages rich JSON telemetry (~800 bytes) into `customData["studylab"]`   │
│                                                                                  │
│   2. Python/Qt Reviewer Bridge (`qt/aqt/reviewer.py`)                            │
│      • Passes `CardAnswer` containing `custom_data` across IPC to Rust backend   │
│                                                                                  │
│   3. Rust Answering Engine (`rslib/src/scheduler/answering/mod.rs`)              │
│      • Extracts `studylab` JSON envelope                                         │
│      • Ingests into `procedural.db` via `record_practice_attempt_atomic()`       │
│                                                                                  │
│   4. Ephemeral Stripping Step (`mod.rs:501`)                                     │
│      • Strips `"studylab"` key from `custom_data` object                         │
│      • Leaves `{}` or non-StudyLab custom data (typically 2–30 bytes)            │
│                                                                                  │
│   5. Anki SQLite Commit (`collection.anki2`)                                     │
│      • `card.validate_custom_data()` verifies `len <= 100` bytes (PASSES)        │
│      • Commits clean card record to Anki database                                │
│                                                                                  │
└──────────────────────────────────────────────────────────────────────────────────┘
```

This lifecycle guarantees that StudyLab can transmit rich, high-fidelity cognitive evidence (solution graphs, step derivations, speed ratios) from webview to backend without ever violating Anki's storage limits.

---

## 6. FSRS Scheduling Bridge & Non-Interference

StudyLab defers all temporal spaced-repetition calculations to Anki's native **Free Spaced Repetition Scheduler (FSRS)** or SM-2 scheduler:

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                      FSRS RATING DERIVATION FLOW                                 │
├──────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│   Procedural Telemetry:                                                          │
│   • Correctness: `is_correct`                                                    │
│   • Latency: `time_taken_ms` vs `target_time_ms`                                 │
│   • Scaffolding: `hints_used`, `attempt_count`                                   │
│   • Error Taxonomy: `is_conceptual_error` vs `is_execution_error`                │
│                                                                                  │
│                                  │                                               │
│                                  ▼                                               │
│             `derive_fsrs_rating()` (`rating_policy.rs`)                          │
│                                  │                                               │
│         ┌──────────────┬─────────┴─────────┬──────────────┐                      │
│         ▼              ▼                   ▼              ▼                      │
│      [Again]        [Hard]              [Good]         [Easy]                    │
│      (Rating 1)     (Rating 2)          (Rating 3)     (Rating 4)                │
│      • Incorrect    • Slow latency      • On-target    • Fast latency            │
│      • Concept gap  • 1-2 hints used    • Correct      • 0 hints, 1 try          │
│      • >= 3 retries • Calculation slip  • Expected     • Strong history          │
│                                                                                  │
│                                  │                                               │
│                                  ▼                                               │
│                     Native Anki FSRS Scheduler                                   │
│                     (Computes next review interval)                              │
│                                                                                  │
└──────────────────────────────────────────────────────────────────────────────────┘
```

- **Separation of Concerns:** FSRS determines **WHEN** the next practice session occurs ($Interval = f(Stability, Difficulty)$); StudyLab determines **WHAT** parametric problem instance, difficulty level, and scaffolding support is presented.
- **Ease Button Visibility:** In feedback state, StudyLab displays the derived rating recommendation while revealing native Anki ease buttons (1–4) for optional manual learner adjustment.

---

## 7. UI & Window Lifecycle Isolation

StudyLab guarantees 100% non-interference with standard Anki cards through strict DOM containment and teardown hooks:

1. **DOM Scoping:** All StudyLab elements are encapsulated within `#procedural-card` (`.procedural-card-container`). Standard Anki toolbar, menus, and bottom bar operate unmodified.
2. **`destroyActive()` Teardown Hook (`qt/aqt/reviewer.py:207, 410`):**
   Before rendering any card (standard or procedural), `reviewer.py` evaluates:
   ```javascript
   if (globalThis.anki && globalThis.anki.procedural && typeof globalThis.anki.procedural.destroyActive === 'function') {
       globalThis.anki.procedural.destroyActive();
   }
   ```
3. **`MutationObserver` Container Monitor (`ts/reviewer/procedural.ts:240–270`):**
   A `MutationObserver` attached to `document.body` monitors `#procedural-card`. If the container is unmounted (e.g. user transitions to a standard flashcard), `ProceduralReviewer.destroy()` automatically unbinds all event listeners, clears stopwatch timers, and nulls references.
4. **Zero Shortcut Leakage:** Keyboard handlers (Space, Enter, `1`–`4`, `A`–`D`) are active only during the `solving` and `mistake_classification` states. Upon unmounting, standard Anki shortcuts are immediately restored.

---

## 8. APKG Package Boundary & Portability

StudyLab content is distributed using standard Anki Package (`.apkg`) files. The package boundary adheres to the **Universal Portability Invariant**:

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                     3-TIER CONTENT RESOLUTION HIERARCHY                          │
├─────────┬──────────────────┬─────────────────────────────────────────────────────┤
│ Tier    │ Resolution Target│ Mechanism & Behavior                                │
├─────────┼──────────────────┼─────────────────────────────────────────────────────┤
│ **1**   │ `inline_contract`│ **Preferred Path:** Complete self-contained JSON    │
│         │ (Self-Contained) │ blueprint embedded in the card payload. Works on    │
│         │                  │ any client with zero external database dependencies.│
├─────────┼──────────────────┼─────────────────────────────────────────────────────┤
│ **2**   │ `content_ref`    │ Resolves against pre-ingested canonical items in    │
│         │ (Local DB Store) │ `procedural.db` (`practice_items` table).           │
├─────────┼──────────────────┼─────────────────────────────────────────────────────┤
│ **3**   │ `proc_schema`    │ Legacy string ID dispatching to built-in Rust       │
│         │ (Rust Catalog)   │ generator families in `rslib/procedural/`.          │
└─────────┴──────────────────┴─────────────────────────────────────────────────────┘
```

- **Zero Binary Code inside Decks:** `.apkg` files contain declarative JSON blueprints (`ProblemFamilyContract`, `ParameterDomain`, `AnswerDerivation`), never compiled machine code or scripts.
- **Portability:** Decks can be imported, exported, shared, and synced across Anki clients without custom binary installers.

---

## 9. The 8-Tier Source-of-Truth Hierarchy

When reconciling discrepancies across documentation, phase reports, and code, all claims must be resolved strictly according to the **8-Tier Source-of-Truth Hierarchy**:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     SOURCE-OF-TRUTH HIERARCHY                           │
├───────┬──────────────────────────────────┬──────────────────────────────┤
│ Tier  │ Authority Source                 │ Resolution Role              │
├───────┼──────────────────────────────────┼──────────────────────────────┤
│ **1** │ Current Executable Source Code   │ Supreme Ground Truth         │
│ **2** │ Current Passing Test Suites      │ Behavioral Ground Truth      │
│ **3** │ Current Schemas / Migrations     │ Structural Ground Truth      │
│ **4** │ Current Verified Artifacts       │ Empirical Ground Truth       │
│ **5** │ Explicit Product Requirements    │ Intent Ground Truth          │
│ **6** │ Canonical Suite Documentation    │ Explanatory Interface        │
│ **7** │ Historical Phase Reports (01-08) │ Archaeological Context       │
│ **8** │ General / Unverified Assumptions │ Subordinate (Discard)        │
└───────┴──────────────────────────────────┴──────────────────────────────┘
```

### Governing Directives
1. **Existing docs are NOT automatically correct:** If older documentation claims a feature behaves in a certain way, but current source code behaves differently, the code and passing tests govern.
2. **Existing code is NOT automatically product intent:** If an implementation drifted from intended cognitive architecture, the drift must be explicitly documented.
3. **Research Facts vs. Product Decisions must be demarcated:** Scientific principles (Cognitive Load Theory, ACT-R) are research invariants; specific constants ($\alpha=0.20$, 4 mistake buttons, 5 difficulty tiers) are engineering heuristics.

---

*For detailed system architecture and implementation specifications, see [docs/SYSTEM_ARCHITECTURE.md](SYSTEM_ARCHITECTURE.md) and [docs/ARCHITECTURE_INVARIANTS.md](ARCHITECTURE_INVARIANTS.md).*
