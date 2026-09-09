# StudyLab Polyglot Project Navigation Map

**Authority:** Authoritative Polyglot Navigation Reference  
**Target Repository:** `naksh-07/anki-maths`  
**Status:** CANONICAL REPOSITORY MAP  
**Integrity Mode:** 100% Grounded in Verified Repository Paths and Executable Source Code

---

## 1. High-Level Polyglot Architecture

StudyLab is integrated into Anki as a multi-tier polyglot subsystem:

```text
┌──────────────────────────────────────────────────────────────────────────────────┐
│                         POLYGLOT SUBSYSTEM TAXONOMY                              │
├───────────────────┬────────────────────────────┬─────────────────────────────────┤
│ Tier / Language   │ Physical Repository Path   │ Primary Responsibilities        │
├───────────────────┼────────────────────────────┼─────────────────────────────────┤
│ **Rust Core**     │ `rslib/procedural/`        │ Procedural engine, solvers,     │
│                   │                            │ 5D units, storage, diagnostics  │
│                   │ `rslib/src/`               │ Anki hooks: storage, render,    │
│                   │                            │ telemetry stripping, apkg import│
├───────────────────┼────────────────────────────┼─────────────────────────────────┤
│ **TypeScript UI** │ `ts/reviewer/`             │ Open Canvas solving UI, state   │
│                   │                            │ machine, modalities, keyboard   │
├───────────────────┼────────────────────────────┼─────────────────────────────────┤
│ **Python / Qt**   │ `qt/aqt/reviewer.py`       │ PyQt6 Reviewer bridge, button   │
│                   │                            │ suppression, IPC command router │
│                   │ `pylib/`                   │ Python bindings for Rust core   │
├───────────────────┼────────────────────────────┼─────────────────────────────────┤
│ **Protobuf IPC**  │ `proto/`                   │ Cross-language RPC definitions  │
├───────────────────┼────────────────────────────┼─────────────────────────────────┤
│ **Content Tools** │ `tools/`                   │ 175-topic content factory       │
│                   │ Root generator scripts     │ APKG fixture generators         │
├───────────────────┼────────────────────────────┼─────────────────────────────────┤
│ **QA & Artifacts**│ `artifacts_qa/`            │ APKG validators, test fixtures  │
├───────────────────┼────────────────────────────┼─────────────────────────────────┤
│ **Documentation** │ `docs/`                    │ Canonical specifications        │
│                   │ `docs/contracts/`          │ Frozen Level 1 master contracts │
└───────────────────┴────────────────────────────┴─────────────────────────────────┘
```

---

## 2. Exhaustive Directory & File Index

Every path listed below has been verified against the physical filesystem:

### 2.1 Rust Subsystem (`rslib/`)

#### Core Procedural Engine (`rslib/procedural/src/`)
- **`anchor/`**: Note model anchor extraction and ingestion.
  - `mod.rs`: `ProceduralCardAnchor` definition and 3-tier blueprint resolver.
  - `source.rs`: `SourceQuestion` definition for curated static items conforming to `StudyLab-Source-APKG-Contract(1).txt`.
- **`chemistry/`**: Chemistry domain solver, reaction stoichiometry, and chemical equilibrium.
- **`content/`**: Declarative content resolution and local schema cache.
- **`core/`**: Fundamental academic domain enums (`mod.rs`), error definitions, and cognitive `decision.rs`.
- **`diagnostics/`**: Diagnostic modeling and multi-dimensional error metrics.
- **`exam/`**: Mock examination engine (`mock.rs`) with diagnostic batch synchronization into `SkillState`.
- **`physics/`**: Physics domain formulas, kinematics, and physical sanity bounds.
- **`practice/`**: Practice problem instances, attempt records, and error events.
- **`problems/`**: Problem generation and validation.
  - `contract.rs`: `DeclarativeFamilyContract`, 15 parameter domains, 24 derivation variants.
  - `declarative.rs`: Universal declarative problem generator.
  - `registry.rs`: Domain problem family registry and dispatch.
  - `steps/`:
    - `step_validator.rs`: CAS step semantic validator and root solver.
    - `step_graph.rs`: Derivation graph nodes and solution paths.
- **`reasoning/`**: Non-verbal reasoning, CSP solver, and seating arrangement algorithms.
- **`remediation/`**: JIT remediation engine.
  - `policy.rs`: 9-tier pedagogical remediation action selector.
  - `queue.rs`: Remediation priority queue with recurrence circuit breakers ($\ge 5$).
- **`reviewer/`**: Reviewer webview HTML template synthesis.
  - `template.rs`: HTML generation, LaTeX preservation, and mandatory `escape_html()` XSS sanitization.
- **`scheduling/`**: Adaptive scheduling and FSRS integration.
  - `rating_policy.rs`: `derive_fsrs_rating()` converting procedural telemetry to Anki ease ratings (`1..4`).
  - `unified.rs`: Unified 10-tier scheduling engine.
- **`service/`**: `ProceduralService` façade (`mod.rs`) coordinating storage, generation, and reconciliation.
- **`skills/`**: Cognitive mastery engine.
  - `mod.rs`: `SkillState` tracking and attempt recording.
  - `progression.rs`: Exponential Moving Average ($\alpha=0.20$) and 6-gate promotion policy ($New \to Mastered$).
  - `domain_evidence.rs`: Multi-domain diagnostic evidence accumulators (`DomainEvidencePayload`).
- **`storage/`**: Dedicated SQLite store (`collection.procedural`).
  - `store.rs`: `ProceduralStore` with 100% parameterized queries and ACID atomic transactions.
  - `schema.rs`: DDL defining 16 tables and 22 indexes.
  - `migration.rs`: Schema migrations v1 through v5.
- **`units/`**: Physical dimensional analysis.
  - `mod.rs`: 5D dimensional vectors ($[M][L][T][N][K]$) and 40+ unit registry.

#### Rust Upstream Integration Touchpoints (`rslib/src/`)
- **`rslib/src/collection/mod.rs` (Lines 141, 173–183):**
  `Collection::procedural_service()` lazily initializes `<col_path>.procedural` SQLite database.
- **`rslib/src/notetype/render.rs` (Lines 122–131):**
  `CardRenderContext::render_card()` intercepts notes starting with `"StudyLab Procedural Anchor"` (`render_procedural_anchor()`) and `"StudyLab Source"` (`render_source_anchor()`). Standard notes bypass with zero overhead.
- **`rslib/src/scheduler/answering/mod.rs` (Lines 353–505):**
  Ingests `studylab` telemetry, records attempts in `<col>.procedural`, and strips `studylab` metadata from `card.custom_data` to satisfy Anki's 100-byte database limit.
- **`rslib/src/import_export/package/apkg/import/mod.rs` (Line 69):**
  Automatically calls `col.reconcile_source_questions()` upon importing `.apkg` packages.

---

### 2.2 Frontend TypeScript Subsystem (`ts/reviewer/`)

- **`ts/reviewer/procedural.ts`:**
  Core `ProceduralReviewer` state machine. Manages solving lifecycle, hotkey bindings, submission debouncing, and host teardown (`destroyActive()`).
- **`ts/reviewer/answering.ts`:**
  `mutateNextCardStates` bridge helper for telemetry packaging.
- **`ts/reviewer/components/mcq_container.ts`:**
  `MCQContainer`: Renders discrete option cards with keyboard hotkeys (`1`–`4`, `A`–`D`). Enforces zero text-box fallback.
- **`ts/reviewer/components/numerical_container.ts`:**
  `NumericalContainer`: Dedicated numeric input with 5D dimensional vector algebra and 40+ unit registry parsing.
- **`ts/reviewer/components/stepwise_container.ts`:**
  `StepwiseContainer`: Interactive derivation graph for multi-step reasoning.
- **`ts/reviewer/diagnostic/`:**
  - `diagnostic_session.ts`: Controller for diagnostic mock-test assessments.
  - `diagnostic_report.ts`: Renders 4-tier diagnostic reports (Subject $\to$ Chapter $\to$ Topic $\to$ Family).
- **`ts/reviewer/reviewer.scss`:**
  Design token stylesheet (`--proc-*`) providing Open Canvas layout with 3px left accent borders.

---

### 2.3 Python / PyQt6 Subsystem (`qt/aqt/`, `pylib/`)

- **`qt/aqt/reviewer.py`:**
  - `_is_procedural_card()`: Detects StudyLab cards by note type prefix.
  - `_showAnswerButton()` & `_showEaseButtons()`: Suppresses native Anki rating buttons on procedural cards.
  - `onEnterKey()`: Intercepts Space/Enter to delegate to `globalThis.anki.procedural.handleNativeShowAnswer()`, blocking bypasses during solving and mistake classification.
  - `_handle_procedural_command()`: Routes IPC commands (`procedural_attempt`, `procedural_mistake`, `procedural_answer`, `procedural_hint`, `procedural_validate_steps`, `procedural_try_similar`, `procedural_practice_prerequisite`, `procedural_declarative_recall`).
  - `_showQuestion()` (Line 416) & `cleanup()` (Line 208): Invokes `globalThis.anki.procedural.destroyActive()` to purge event listeners.
- **`qt/aqt/webview.py`:**
  PyQt6 `QWebEngineView` wrapper hosting `mw.web` (main reviewer) and `mw.bottomWeb` (bottom bar).

---

### 2.4 Content Tools & Packaging

- **`tools/studylab_content_factory.py`:**
  Universal Phase 36C content factory generating release-quality declarative contracts across all 175 target STEM topics (59 Math, 30 Reasoning, 40 Physics, 46 Chemistry).
- **`generate_canonical_source_apkg.py`:**
  Deterministic canonical Source APKG generator adhering strictly to `StudyLab-Source-APKG-Contract(1).txt`.
- **`generate_procedural_apkg.py`:**
  Procedural blueprint APKG generator packaging declarative problem blueprints.
- **`artifacts_qa/validate_canonical_source_apkg.py`:**
  QA validator verifying that an APKG complies with the canonical Source note model and field contract.
- **`artifacts_qa/validate_canonical_apkg.py`:**
  QA validator verifying that generated Universe APKGs comply with the 175-topic declarative blueprint contract.
- **`artifacts_qa/challenger_2_master_runner.py`:**
  Release candidate master runner executing 4 adversarial test dimensions (Math CAS, SQLite Persistence, 100-Byte Telemetry, Cold-Start APKG).
- **`artifacts_qa/canonical_source_test_fixture.apkg`:**
  Verified canonical Source APKG test fixture.

---

### 2.5 Canonical Documentation Index (`docs/`)

#### Master Contracts & Invariants
- **`StudyLab-Source-APKG-Contract(1).txt`**: Level 1 FROZEN contract for canonical static Source APKGs.
- **`docs/contracts/StudyLab-Runtime-Interaction-Contract.md`**: Level 1 FROZEN runtime interaction contract (state machine, keyboard contract, no "Next Card" button).
- **`docs/contracts/StudyLab-Runtime-UI-Contract.md`**: Level 1 FROZEN runtime UI contract (layout, suppressed controls, mistake footer).
- **`docs/ARCHITECTURE_INVARIANTS.md`**: The 16 frozen non-negotiable architectural invariants.
- **`docs/DOCUMENTATION_MAP.md`**: Sitemaps, reading paths, and source-of-truth hierarchy.
- **`docs/DOCUMENTATION_TRUTH_MATRIX.md`**: 18-area truth matrix resolving historical gap reports against executable source code.
- **`docs/FINAL_LIVE_UI_FORENSIC_REPORT.md`**: Physical desktop verification, window HWND audit, and Two-P0 reconciliation.

#### Architecture & Subsystem Deep Dives
- **`docs/SYSTEM_ARCHITECTURE.md`**: End-to-end multi-layer pipeline and crate architecture.
- **`docs/PRODUCT_BOUNDARIES.md`**: Host-guest decoupling architecture and responsibility matrix.
- **`docs/STUDYLAB_PRODUCT_CONTRACT.md`**: Product North Star and cognitive models.
- **`docs/DATABASE_DATA_CONTRACT.md`** & **`docs/DATA_AND_PERSISTENCE.md`**: `collection.procedural` SQLite schema, WAL mode, migrations.
- **`docs/APKG_CONTENT_CONTRACT.md`**: Dual content architecture specification.
- **`docs/FRONTEND_PRODUCT_SPEC.md`**: 9 learning object modalities and UI specifications.
- **`docs/FRONTEND_BACKEND_CONTRACT.md`**: IPC bridge commands and hook lifecycle.
- **`docs/LEARNING_MODEL.md`**: EMA mastery model, BKT comparison, 6 progression gates.
- **`docs/LEARNING_OBJECTS.md`**: Modality containers and stepwise derivation graphs.
- **`docs/REVIEWER_STATE_MACHINE.md`**: 11-state machine, speed quadrants, keyboard isolation.
- **`docs/DIAGNOSTIC_AND_REMEDIATION.md`**: Diagnostic mock tests, error analysis, JIT remediation.

---

## 3. Key Symbol & Integration Contract Ledger

| Symbol / Type | Defined In | Language | Role / Contract |
|---|---|---|---|
| `ProceduralService` | `rslib/procedural/src/service/mod.rs` | Rust | Central service facade coordinating engine, storage, and reconciliation |
| `SourceQuestion` | `rslib/procedural/src/anchor/source.rs` | Rust | Canonical model for curated static source questions |
| `ProceduralCardAnchor` | `rslib/procedural/src/anchor/mod.rs` | Rust | Model for declarative problem blueprints |
| `ProceduralStore` | `rslib/procedural/src/storage/store.rs` | Rust | SQLite database manager for `<col>.procedural` |
| `StepValidator` | `rslib/procedural/src/problems/steps/step_validator.rs` | Rust | CAS step validator performing root solving & tree equivalence |
| `DeclarativeFamilyContract` | `rslib/procedural/src/problems/contract.rs` | Rust | Declarative problem blueprint specification |
| `SkillState` | `rslib/procedural/src/skills/mod.rs` | Rust | Per-skill learner mastery state |
| `derive_fsrs_rating` | `rslib/procedural/src/scheduling/rating_policy.rs` | Rust | Translates procedural attempt telemetry to Anki ease ratings (`1..4`) |
| `escape_html` | `rslib/procedural/src/reviewer/template.rs` | Rust | Mandatory XSS sanitization for webview templates |
| `ProceduralReviewer` | `ts/reviewer/procedural.ts` | TypeScript | Frontend state machine controller |
| `MCQContainer` | `ts/reviewer/components/mcq_container.ts` | TypeScript | Modality container for discrete choice with hotkeys |
| `NumericalContainer` | `ts/reviewer/components/numerical_container.ts` | TypeScript | Modality container for 5D physical units and numeric inputs |
| `StepwiseContainer` | `ts/reviewer/components/stepwise_container.ts` | TypeScript | Modality container for multi-step derivations |
| `destroyActive()` | `ts/reviewer/procedural.ts:1453` | TypeScript | Global unmount cleanup hook called by Python host |
| `_handle_procedural_command` | `qt/aqt/reviewer.py:758` | Python | IPC bridge dispatcher for `procedural_*` commands |
| `_is_procedural_card` | `qt/aqt/reviewer.py:696` | Python | Card discriminator checking note type name prefixes |

---

## 4. Verified Command Reference

All commands must be run from the repository root (`naksh-07/anki-maths`):

```powershell
# 1. Rust engine compilation check
cargo check -p procedural

# 2. Rust procedural unit tests (146 lib unit tests, 0.12s)
cargo test -p procedural --lib

# 3. 175-topic factory validation test (5 tests, 0.08s)
cargo test -p procedural --test phase36c_all_175_topics_factory_tests

# 4. Canonical Source APKG contract test (18 tests, 0.01s)
cargo test -p procedural --test canonical_source_contract_tests

# 5. Desktop validation master suite (10 tests, 2.75s)
cargo test -p procedural --test desktop_validation_master_suite

# 6. Production hardening concurrency & resilience test (7 tests, 0.08s)
cargo test -p procedural --test phase40_production_hardening_tests

# 7. Generate canonical source APKG fixture
python generate_canonical_source_apkg.py artifacts_qa/canonical_source_test_fixture.apkg

# 8. Validate canonical source APKG fixture against Level 1 contract
python artifacts_qa/validate_canonical_source_apkg.py artifacts_qa/canonical_source_test_fixture.apkg

# 9. Validate all 175 topic declarative blueprints
python tools/studylab_content_factory.py --validate

# 10. Generate full 175-topic Universe APKGs
python tools/studylab_content_factory.py --generate-apkgs dist/apkgs

# 11. Validate generated Universe APKG
python artifacts_qa/validate_canonical_apkg.py

# 12. Run Challenger 2 master adversarial audit (4 dimensions)
python artifacts_qa/challenger_2_master_runner.py

# 13. Python test suite (PyQt + pylib)
just test-py

# 14. Linting & type checking
just lint

# 15. Format checking & fixing
just fmt
just fix-fmt

# 16. Comprehensive system build & check
just check
```
