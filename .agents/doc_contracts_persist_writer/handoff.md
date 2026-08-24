# Handoff Report: Contracts, State Machine, Persistence, Map & Open Questions

**Agent Role:** CONTRACTS & PERSISTENCE DOC WRITER  
**Working Directory:** `C:\Users\Suraj\Documents\Antigravity\Anki-maths\.agents\doc_contracts_persist_writer\`  
**Date:** 2026-08-25  
**Target Output Documents:**
1. `docs/REVIEWER_STATE_MACHINE.md`
2. `docs/FRONTEND_BACKEND_CONTRACT.md`
3. `docs/DATA_AND_PERSISTENCE.md`
4. `docs/DOCUMENTATION_MAP.md`
5. `docs/OPEN_QUESTIONS.md`

---

## 1. Observation

Direct observations from source code and test archaeology:
1. **Frontend State Machine (`ts/reviewer/procedural.ts:25-36`):**
   - `ProceduralUIState` declares 11 states: `"loading"`, `"ready"`, `"solving"`, `"hint"`, `"submitting"`, `"mistake_classification"`, `"feedback"`, `"worked_example"`, `"next"`, `"error"`, `"teardown"`.
   - `computeSpeedQuadrant(isCorrect, timeTakenMs, targetTimeMs)` categorizes performance into 4 quadrants (`fluency_strength`, `speed_opportunity`, `strategy_trap`, `concept_setup`).
   - `MistakeFooter` captures 4 mistake categories (`silly_mistake`, `pattern_not_recognized`, `formula_or_concept_misapplied`, `concept_not_known`) and traps Space/Enter keys to prevent reflection bypass.
   - `destroy()` method cleanly disposes intervals, timeouts, child containers, DOM event listeners, MutationObservers, and global references.

2. **IPC Bridge Protocol (`qt/aqt/reviewer.py:697-825` & `ts/reviewer/answering.ts`):**
   - Python `Reviewer._linkHandler` and `_handle_procedural_command` dispatch 8 procedural bridge commands: `procedural_answer:`, `procedural_attempt:`, `procedural_hint:`, `procedural_validate_steps:`, `procedural_mistake:`, `procedural_try_similar:`, `procedural_practice_prerequisite:`, and `procedural_declarative_recall:`.
   - `globalThis.anki.mutateNextCardStates` injects rich `studylab` telemetry into card custom data.
   - Rust scheduler answering hook (`rslib/src/scheduler/answering/mod.rs:353-505`) ingests telemetry into `procedural.db` and strips the `studylab` payload before committing to `collection.anki2`, preserving the 100-byte custom data column limit.

3. **Storage & Persistence Architecture (`rslib/procedural/src/storage/`):**
   - SQLite store is located at `<collection_name>.procedural` (`procedural.db`), completely separated from `collection.anki2`.
   - Pragmas applied: `busy_timeout = 5000`, `foreign_keys = ON`, `synchronous = NORMAL`, `temp_store = MEMORY`, `journal_mode = WAL`.
   - Schema migrations (v1 to v5) define 11 tables and 17 dedicated indexes.
   - `ProceduralStore::record_practice_attempt_atomic()` executes in a single atomic SQLite transaction boundary.

4. **Documentation Map & Truth Matrix:**
   - Reconciles historical drift (Phase 01–03 gap matrix) and aligns repository documentation to the 8-tier source-of-truth hierarchy.

---

## 2. Logic Chain

1. **Source Evidence to Specification Alignment:** Every architectural detail, schema DDL, IPC signature, and state transition was cross-referenced directly with physical files in `rslib/procedural/`, `rslib/`, `qt/aqt/`, and `ts/reviewer/`.
2. **Pedagogical & Invariant Grounding:** State transitions and telemetry designs are grounded in cognitive psychology principles (Hypercorrection effect, Cognitive Load Theory, ACT-R Two-Memory Model) as synthesized in `docs/DEEPSEARCH_EVIDENCE.md` and `docs/DOCUMENTATION_TRUTH_MATRIX.md`.
3. **Strict Database Isolation:** The persistence model guarantees that StudyLab practice produces zero schema pollution, zero column overflow in Anki's `cards.data`, and zero interference with standard Anki flashcards.
4. **Pruned Open Questions:** Questions answered by code (generator dispatch, step validation, database separation, custom data stripping, keyboard trapping) were formally marked as resolved, leaving only 5 genuine product/platform explorations.

---

## 3. Caveats

- **Source Code Immutability:** In accordance with Benchmark Integrity rules, zero source code files (`.rs`, `.ts`, `.py`) were modified. All authoring was restricted to the 5 target Markdown files in `docs/` and agent metadata.
- **Future Platform Spikes:** The open questions regarding WebAssembly mobile builds and canvas OCR represent forward-looking explorations that will require dedicated implementation spikes when mobile support is prioritized.

---

## 4. Conclusion

The 5 canonical documentation files have been completely authored, formatted, and verified against the repository's executable source code:
1. `docs/REVIEWER_STATE_MACHINE.md` (Authoritative 11-state transition lifecycle, ASCII/Mermaid state diagrams, speed quadrant telemetry, anti-bypass reflection trapping, and teardown mechanics).
2. `docs/FRONTEND_BACKEND_CONTRACT.md` (Exhaustive bridge command catalog, JSON schemas, customData lifecycle, ephemeral stripping, and Python hook trace).
3. `docs/DATA_AND_PERSISTENCE.md` (New canonical SQLite persistence specification with complete DDL for 11 tables, 17 indexes, WAL pragmas, v1–v5 migrations, and atomic transaction lifecycles).
4. `docs/DOCUMENTATION_MAP.md` (Master sitemap, 6 reader personas/paths, 8-tier source hierarchy, and canonical source code traceability directory).
5. `docs/OPEN_QUESTIONS.md` (Strictly pruned register highlighting 6 verified resolutions and 5 genuinely open product/platform decisions).

---

## 5. Verification Method

To independently verify the authored documentation against the executable codebase:

1. **State Machine Verification:**
   - Inspect `ts/reviewer/procedural.ts` lines 25–36, 310–360, 704–735, 844–875, 1239–1278.
   - Run Vitest reviewer test suite: `npm run vitest:once ts/reviewer/procedural.test.ts` (27 tests pass).
2. **Bridge Contract Verification:**
   - Inspect `qt/aqt/reviewer.py` lines 697–825 and `rslib/src/scheduler/answering/mod.rs` lines 353–505.
   - Run Qt bridge integration test: `pytest qt/tests/test_phase13.py` (Passes).
3. **Database Schema Verification:**
   - Inspect `rslib/procedural/src/storage/schema.rs`, `store.rs`, and `migration.rs`.
   - Run Rust storage unit tests: `cargo test --lib -p procedural storage::tests` (4 tests pass).
4. **Desktop Master Validation Suite:**
   - Run `cargo test -p procedural --test desktop_validation_master_suite` (All 17 sections pass, including 1,000 continuous card transitions).
