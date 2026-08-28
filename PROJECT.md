# Project: StudyLab Release Candidate Full-System Audit

## Architecture
StudyLab is a procedural problem-solving and adaptive learning engine hosted inside the Anki desktop runtime, providing generative problem generation, step-by-step validation, mistake classification, longitudinal skill tracking, and adaptive remediation.

```
[Canonical APKG / Static Source Exporter]
             │ (StudyLab Source Note / static questions)
             ▼
[Anki Collection: collection.anki2] ──(isolated)──> [Normal Cards: Basic, Cloze]
             │
             ▼ (rslib/src/notetype/render.rs Interception Hook)
[Rust Procedural Engine: rslib/procedural/] (Bypasses Generation)
             │
             ├──> [Isolated DB: <collection>.procedural] (practice_items, pyq_sources)
             │
             ▼ (IPC / Qt Reviewer Bridge: qt/aqt/reviewer.py)
[Qt WebEngine Reviewer Viewport]
             │
             ▼ (TypeScript Reviewer State Machine: ts/reviewer/procedural.ts)
[Component Hierarchy: MCQContainer, NumericalContainer, StepwiseContainer, MistakeFooter]
```

### Module Boundaries & Data Flow
1. **Packaging Tier (`tools/studylab_content_factory.py`, `generate_procedural_apkg.py`)**:
   - Generates canonical `.apkg` files containing `StudyLab Procedural Anchor` notes.
   - Embeds Tier-1 `inline_contract` declarative blueprints (175 curriculum topics across Math, Reasoning, Physics, Chemistry).
2. **Core Ingestion & Interception (`rslib/src/notetype/render.rs`)**:
   - Intercepts note rendering if name starts with `StudyLab Procedural Anchor`. Standard notes (`Basic`, `Cloze`) bypass with zero overhead.
   - Parses `ProceduralCardAnchor` and resolves 3-tier target hierarchy (`inline_contract` -> `content_ref` -> `proc_schema`).
3. **Backend Procedural Engine (`rslib/procedural/`)**:
   - Generates dynamic variants, evaluates math/symbolic/dimensional expressions, and computes Bayesian mastery updates.
4. **Storage Engine (`collection.procedural`)**:
   - Completely separate SQLite database from `collection.anki2`.
   - Manages 16 tables (`skills`, `skill_states`, `practice_attempts`, `error_events`, `remediation_queue_items`, etc.) with ACID single-transaction atomicity and `ON DELETE CASCADE`.
5. **Host-Guest Bridge (`qt/aqt/reviewer.py`)**:
   - Suppresses native Anki `#ansbut` and ease buttons on procedural cards.
   - Routes bridge commands (`procedural_attempt`, `procedural_hint`, `procedural_mistake`, `procedural_answer:1..4`).
   - Evaluates `destroyActive()` before each card render to prevent event/state leaks.
6. **Frontend State Machine & Open Canvas UI (`ts/reviewer/`)**:
   - 11-state deterministic state machine.
   - Modality-matched input containers (`MCQContainer`, `NumericalContainer`, `StepwiseContainer`, `MistakeFooter`).
   - Open Canvas 720px max-width layout with subtle 3px left accent borders and fixed bottom interaction footer.

---

## Feature Inventory
| # | Feature | Description | Milestone | Source |
|---|---------|-------------|-----------|--------|
| 1 | Documentation <-> Code Reconciliation | Verify end-to-end pipeline trace and synchronize all canonical docs in `docs/` | M1 | ORIGINAL_REQUEST §R1 |
| 2 | Modality Semantic Purity & Invariants | Verify discrete choice (`mcq`, `concept_check`, `strategy_drill`) and reading objects have zero unwanted textboxes (`enforceZeroTextInputFallback`) | M2 | ORIGINAL_REQUEST §R2 |
| 3 | State Machine & Card Teardown Lifecycle | Verify 11-state transitions, double-submit debouncing, error handling, and `destroyActive()` host teardown | M2 | ORIGINAL_REQUEST §R2 |
| 4 | Numerical 5D Vector & Unit Registry | Verify 5D dimensional analysis ($[M][L][T][N][K]$), 40+ unit registry, scalar/fraction/scientific parsing, single preview pill | M2 | ORIGINAL_REQUEST §R2 |
| 5 | Stepwise Multi-Step CAS Evaluation | Verify step stack, linear equation simplification, commutative addition, error categorization, 3-tier hints | M2 | ORIGINAL_REQUEST §R2 |
| 6 | Mistake Classification & Space/Enter Trap | Verify 4-category error reflection strip (`Silly Slip`, `Pattern Missed`, `Concept Gap`, `Prereq Unknown`) with keyboard trapping | M2 | ORIGINAL_REQUEST §R2 |
| 7 | Desktop Layout & Open Canvas (1366x768 to 1920x1080) | Verify 720px max-width container, 3px accent borders, deduplicated answer rows, muted speed pills, bottom footer padding | M3 | ORIGINAL_REQUEST §R3 |
| 8 | Anki Standard Card Isolation | Verify standard Basic and Cloze cards bypass StudyLab hooks with zero DOM/CSS/event leakage | M3 | ORIGINAL_REQUEST §R3, §R4 |
| 9 | Rust Procedural Engine & 175-Topic Factory | Verify `cargo test -p procedural` (100% pass) and all 175 topic contracts via `phase36c_all_175_topics_factory_tests` | M4 | ORIGINAL_REQUEST §R4 |
| 10 | SQLite Persistence & ACID Atomicity | Verify `collection.procedural` 16 tables, 22 indexes, WAL mode, single-transaction atomic attempt logging, migrations v1-v5 | M4 | ORIGINAL_REQUEST §R4 |
| 11 | Telemetry 100-Byte Stripping & AnkiWeb Safety | Verify `custom_data["studylab"]` stripped before committing to `collection.anki2` | M4 | ORIGINAL_REQUEST §R4 |
| 12 | Canonical APKG Generation & Self-Contained Import | Verify `StudyLab_Full_Universe_175.apkg` validation and zero-pre-seeding cold-start import | M4 | ORIGINAL_REQUEST §R4 |
| 13 | Release Candidate Bug Register & Policy | Compile complete bug register and fix policy in `docs/FINAL_RELEASE_AUDIT.md` | M5 | ORIGINAL_REQUEST §R5 |
| 14 | Automated Test Matrix Requalification | Rebuild and run complete automated test suite (`npm run vitest:once`, `cargo test -p procedural`, pytest) | M6 | ORIGINAL_REQUEST §R6 |
| 15 | Live Desktop Webview Forensic Verification | Execute real visible Windows Anki DEV GUI matrix via `desktop-webview-reviewer` across 14 canonical UI states with dual screenshots | M6 | ORIGINAL_REQUEST §R6 |
| 16 | Release Deliverables & Verdict Declaration | Generate `docs/FINAL_RELEASE_AUDIT.md`, `docs/FINAL_RELEASE_NOTES.md`, `artifacts_qa/final_release_audit/evidence.json`, declare release verdict | M7 | ORIGINAL_REQUEST Acceptance |

---

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| M1 | Documentation & Pipeline Trace Audit | End-to-end trace, documentation reconciliation against code in `docs/` | none | DONE |
| M2 | Frontend State Machine & Modalities Audit | 18 vitest suites, 11-state lifecycle, component invariants, zero textboxes | none | DONE |
| M3 | Desktop Layout & Visual Forensic Review | Open Canvas 720px layout, responsive viewports (1366x768 to 1920x1080), Basic/Cloze CSS isolation | none | DONE |
| M4 | Backend Engine, SQLite DB & APKG Requalification | `cargo test -p procedural`, 175 topic factory tests, SQLite ACID & migrations, APKG validator | none | DONE |
| M5 | Bug Register & Final Release Audit Compilation | Author `docs/FINAL_RELEASE_AUDIT.md` and `docs/FINAL_RELEASE_NOTES.md` | M1, M2, M3, M4 | DONE |
| M6 | Second Mandatory Audit & Live Desktop Matrix | Re-execute automated suites, validate APKG, execute `artifacts_qa/live_visual_audit_runner.py` / `desktop-webview-reviewer` evidence | M5 | DONE |
| M7 | Release Package Assembly & Final Verdict Declaration | Assemble `evidence.json`, verify acceptance criteria, declare release verdict | M6 | DONE |

---

## Interface Contracts

### Rust Procedural Engine ↔ SQLite Storage (`collection.procedural`)
- `ProceduralStore::record_practice_attempt_atomic(conn: &mut Connection, record: &PracticeAttemptRecord) -> Result<AttemptOutcome>`
- Schema migrations applied via `MigrationRunner::run(&mut conn) -> Result<usize>` inside single transactions.

### Rust Rendering Hook ↔ Anki Notetype (`rslib/src/notetype/render.rs`)
- Signature: `render_procedural_anchor(&self, note: &Note, card: &Card, nt: &Notetype) -> Result<RenderedTemplate>`
- Gated strictly by: `nt.name.starts_with("StudyLab Procedural Anchor") && !browser`.

### Qt Reviewer Bridge ↔ Webview Frontend (`qt/aqt/reviewer.py` ↔ `ts/reviewer/procedural.ts`)
- Bridge dispatch: `globalThis.pycmd("procedural_attempt:" + JSON.stringify(attempt))`
- Host teardown: `globalThis.anki.procedural.destroyActive()`
- State progression: `globalThis.pycmd("procedural_answer:" + ease)`

---

## Code Layout
- `rslib/procedural/` — Core Rust procedural problem-solving engine, domain generators, solvers, schemas, and SQLite store.
- `rslib/src/notetype/render.rs` — Anki notetype rendering hook and procedural anchor extraction.
- `rslib/src/scheduler/answering/mod.rs` — Telemetry ingestion and `custom_data` 100-byte stripping.
- `qt/aqt/reviewer.py` — Python Qt Reviewer bridge, ease button suppression, and command dispatch.
- `ts/reviewer/` — TypeScript frontend state machine, components (`MCQContainer`, `NumericalContainer`, `StepwiseContainer`, `MistakeFooter`), and `reviewer.scss`.
- `tools/studylab_content_factory.py` — Canonical 175-topic content factory and APKG generator.
- `artifacts_qa/` — QA validators, APKG schema checkers, live visual audit runner, and forensic test evidence.
- `docs/` — Canonical architecture documentation, truth matrices, and release audit reports.
