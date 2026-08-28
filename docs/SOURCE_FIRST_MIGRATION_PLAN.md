# StudyLab Source-First Migration Plan

## A. Executive Summary
The existing StudyLab architecture relies on a **generator-first** model: Anki notes (`StudyLab Procedural Anchor`) act as declarative blueprints (`ProceduralCardAnchor`), which are intercepted at render time by a Rust procedural engine to generate dynamic variants on the fly. 

The new target state is a **source-first** architecture: an external authoring system produces finished `.apkg` packages containing concrete source questions. StudyLab's responsibility shifts strictly to consuming these finished packages, interpreting eligible source questions, and presenting them through the existing procedural UI (without generating artificial variants). The boundary firmly establishes that StudyLab is an interpretation, evaluation, and diagnostic engine, not an authoring/generation system.

## B. Current-State Audit
- **Integration**: The pipeline intercepts rendering inside `rslib/src/notetype/render.rs` for notes starting with `StudyLab Procedural Anchor`.
- **Anchor Parsing**: `rslib/procedural/src/anchor/mod.rs` parses `ProceduralCardAnchor` JSON payloads from the note's fields, extracting `proc_schema` and `inline_contract`.
- **Storage**: An isolated SQLite DB (`collection.procedural`) manages 16 tables, capturing attempts, errors, schemas, and skill states (defined in `rslib/procedural/src/storage/schema.rs`).
- **Frontend**: The Qt reviewer bridge (`qt/aqt/reviewer.py`) dispatches events to a robust TypeScript 11-state machine (`ts/reviewer/`).

## C. Existing Assets (PRESERVE)
The following infrastructure is robust, verified, and **MUST** be preserved:
- **Procedural UI & Components**: `MCQContainer`, `NumericalContainer`, `StepwiseContainer`, `MistakeFooter`.
- **State Machine**: The 11-state interaction lifecycle in the TypeScript frontend.
- **Persistence Layer**: The `collection.procedural` SQLite schema, including its transaction boundaries and WAL mode.
- **Host-Guest Bridge**: The IPC communication established in `qt/aqt/reviewer.py`.
- **Diagnostic Foundation**: Existing error categorization models located in `rslib/procedural/src/diagnostics/`.

## D. UI Preservation Map
- **Layout**: The Open Canvas 720px max-width layout remains untouched.
- **Components**: Do NOT replace the working modality containers (`MCQ`, `Numerical`, `Stepwise`). They will simply receive static, extracted parameters rather than generated ones.
- **Styling**: Existing CSS/SCSS and visual forensics are preserved. Normal Anki Basic/Cloze cards remain completely isolated.

## E. Anki/APKG Integration Map
- **Current Path**: `tools/studylab_content_factory.py` -> `.apkg` (Anchors) -> Anki Import -> `render.rs` Interception -> Generation -> UI.
- **Target Path**: External `.apkg` (Finished Source Notes) -> Anki Import -> Standard Anki `collection.anki2` -> StudyLab Eligibility Hook -> Extract to `practice_items` / `pyq_sources` -> Procedural UI.
- **Rationale**: StudyLab will hook into the standard Anki note flow, relying on Anki's native syncing and `.apkg` importing logic to manage the payload, only intervening when a note is explicitly flagged as a StudyLab eligible source.

## F. Data Model Map
Based on `rslib/procedural/src/storage/schema.rs`:
- **Source Questions**: The schema already possesses structures for source-first data introduced in Migrations 3 and 4: `pyq_sources`, `pyq_mappings`, and `practice_items`.
- **Learner Evidence**: `practice_attempts` and `error_events` will continue to function natively.
- **Metadata**: Attributes like Difficulty, Question Type, Hint, Solution, and Strategy are mapped directly into the `practice_items` table.

## G. Diagnostic Map
- The diagnostic engine (`rslib/procedural/src/diagnostics/`) currently evaluates dynamically generated attempts. 
- In the source-first model, the diagnostic engine will evaluate responses against the static, imported `correct_answer` or `solution_template` stored in `practice_items` or `pyq_sources`. The mistake classification taxonomy (`Silly Slip`, `Concept Gap`, etc.) remains identical.

## H. Generation Map
- **Current**: `tools/studylab_content_factory.py` contains the hardcoded definitions for 175 topics across Maths, Reasoning, Physics, and Chemistry.
- **Target**: DEFERRED. The generation infrastructure should be preserved as optional future work but completely decoupled from the critical path of the runtime. No generated variants will be enforced.

## I. Identity/Reconciliation Proposal
- **Stable Identity**: Source question identity MUST rely on the Anki `Note GUID` combined with a designated repository-specific source identifier (if available in the note fields) to ensure determinism. Anki `Card ID` is insufficient alone as it mutates on collection rebuilds.
- **Re-import Model**: 
  - **NEW**: GUID not present in `practice_items`; insert.
  - **UPDATED**: GUID matches, hash of content differs; update `practice_items` while preserving `skill_states` and `practice_attempts`.
  - **REMOVED**: GUID missing from collection; mark as archived in `practice_items` (do not cascade delete learner history).

## J. Four-Subject Architecture
- **Shared Core**: All subjects (Mathematics, Reasoning, Physics, Chemistry) will use the same `practice_items` table and UI state machine.
- **Subject Semantics**: 
  - *Maths/Physics/Chemistry*: Retains capabilities for formulas, stepwise execution, and units.
  - *Reasoning*: Retains discrete logic options (100% MCQ capability as seen in the factory).

## K. Validation Strategy
The future source-first validator must perform a tripartite reconciliation:
1. **Source Count**: Number of Anki notes matching the StudyLab eligibility criteria.
2. **Discovered Count**: Number of valid rows instantiated in `practice_items` / `pyq_sources`.
3. **Playable Count**: Number of items successfully rendered in the UI without crashing.
Any intentional exclusion (e.g., malformed note) must be explicitly logged. Silent loss is unacceptable.

## L. Documentation Map
| Document | Status | Action | Reason |
| -------- | ------ | ------ | ------ |
| `PROJECT.md` | ACTIVE | MODIFY | Update architecture diagram to remove dynamic generation from the critical path. |
| `APKG_CONTENT_CONTRACT.md` | ACTIVE | MODIFY | Needs to reflect static source ingestion rather than procedural blueprints. |
| `DATABASE_DATA_CONTRACT.md` | ACTIVE | KEEP | Schema already supports `practice_items` and `pyq_sources`. |
| `STUDYLAB_PRODUCT_CONTRACT.md` | ACTIVE | MODIFY | Update product boundaries to remove authoring/generation responsibilities. |
| `SYSTEM_ARCHITECTURE.md` | ACTIVE | MODIFY | Align pipeline trace with source-first ingestion. |

## M. KEEP / MODIFY / ADD / DEFER Matrix
- **KEEP**: SQLite Database schema, Procedural Frontend UI, Diagnostic engine, Host-Guest IPC bridge.
- **MODIFY**: `render.rs` to intercept based on a static eligibility hook rather than a dynamic `ProceduralCardAnchor`.
- **ADD**: A deterministic reconciliation module that syncs Anki Note GUIDs into `practice_items` on `.apkg` import or collection load.
- **DEFER**: `studylab_content_factory.py` (Topic factory and variant generation).

## N. Implementation Phases
1. **Phase 1 — Discovery & Planning**: COMPLETE.
2. **Phase 2 — Eligibility & Interception (Read-Only)**: COMPLETE. Implemented `StudyLab Source` prefix interception hook in `render.rs`. Extracted fields (`Prompt`, `Options`, `CorrectAnswer`, etc.) into unified `SourceQuestion` struct in `rslib/procedural/src/anchor/source.rs` and translated directly into a `PracticeSessionObject` for UI rendering without generation.
3. **Phase 3 — Identity & Persistence (Write)**: COMPLETE. Implemented reconciliation logic in `ProceduralService::reconcile_source_questions`. Mapped the extracted struct to canonical `PracticeItem` utilizing Note GUIDs. Added deterministic hashing to avoid unnecessary updates. Implemented SQL `UPSERT` in `ProceduralStore` and handles New/Updated/Archived states safely.
4. **Phase 4 — Runtime Wiring**: Connect the procedural rendering engine to feed off `practice_items` instead of `ProceduralCardAnchor` templates.
5. **Phase 5 — Diagnostics & Telemetry Validation**: Ensure that learner evidence (`error_events`, `practice_attempts`) routes correctly to the new stable identities.

## O. Definition of Done
The migration is complete when StudyLab can ingest an external `.apkg` of static source questions, securely identify and register them in `collection.procedural`, and render them natively in the existing Open Canvas UI without executing any dynamic generation logic, whilst preserving all learner evidence.

---

## Documentation Synchronization Protocol
For every future implementation phase:
1. Inspect affected documentation before coding.
2. Update contracts when behaviour changes.
3. Update architecture documentation when architecture changes.
4. Update data contracts when schema changes.
5. Update APKG/Anki integration docs when integration changes.
6. Update diagnostic docs when diagnostic behaviour changes.
7. Update validation docs when validation rules change.
8. Update tests/documented evidence when behaviour is verified.
9. Remove obsolete claims.
10. Record migration decisions and rationale.
11. Run documentation consistency checks where possible.
12. Mark the phase incomplete if critical documentation is stale.
