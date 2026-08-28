# StudyLab APKG Contract Alignment — Migration Status & Agent Handoff

**Initiative:** StudyLab APKG Contract Alignment  
**Status:** COMPLETE / FROZEN  
**Document Version:** 4.0.0 (Definitive Freeze)  
**Canonical Contract Reference:** `StudyLab-Source-APKG-Contract(1).txt` (Authoritative)  
**Last Updated:** 2026-08-28  

---

## 1. Documentation Authority Hierarchy & Agent Directive

```text
Level 1 — Canonical Contract
StudyLab-Source-APKG-Contract(1).txt (Authoritative Source of Truth — FROZEN)
        ↓
Level 2 — Project Architecture
PROJECT.md
        ↓
Level 3 — APKG / Content Documentation
docs/APKG_CONTENT_CONTRACT.md
        ↓
Level 4 — Implementation / Runtime Docs
docs/SYSTEM_ARCHITECTURE.md, docs/ARCHITECTURE_INVARIANTS.md
        ↓
Level 5 — Agent Handoff / Status
docs/APKG_CONTRACT_ALIGNMENT_STATUS.md (This Document)
```

> [!IMPORTANT]
> **Directive for Future Agents:**
> ```text
> Canonical Source APKG
>         ↓
> StudyLab Source
>         ↓
> Canonical SourceQuestion
>         ↓
> Runtime
> 
> Procedural content is a separate compatible architecture.
> It must not redefine the Source APKG contract.
> ```

---

## 2. Executive Summary & Status

All phases of the StudyLab APKG Contract Alignment initiative are **COMPLETE and FROZEN**:

- **Phase 1 (Discovery & Contract Audit):** COMPLETE
- **Phase 2 (Core Canonical Contract Implementation):** COMPLETE
- **Phase 3 (End-to-End Runtime Integration):** COMPLETE & VERIFIED
- **Phase 4 (Final Adversarial Audit, Hardening & Freeze):** COMPLETE & FROZEN

The complete canonical source-first pipeline is fully operational, hardened against adversarial inputs, verified by 100% passing test suites, and architecturally frozen:

```text
Canonical StudyLab Source APKG (.apkg)
  │ (collection.anki2 + media assets conforming to StudyLab-Source-APKG-Contract(1).txt)
  ▼
Anki Import Pipeline (Collection::import_apkg)
  │ (Automatic Hook: col.reconcile_source_questions() in import transaction)
  ▼
Canonical Source Extraction & Hardened Validation (SourceQuestion)
  │ (Content, Modality, Finite Difficulty, Semantics, Provenance, No Heuristics)
  ▼
Deterministic Storage Reconciliation (collection.procedural / practice_items)
  │ (New / Updated / Unchanged / Archived deterministic lifecycle)
  ▼
Runtime Translation (PracticeItem -> ProblemInstance / PracticeSessionObject)
  │ (Deterministic seed, correct_answer, solution graph, learning support)
  ▼
Reviewer SSR & TypeScript State Machine (ProceduralReviewer / Svelte Webview)
  │ (MCQ Radio Cards / Numerical Input / Modality Purity / No Textbox Fallbacks)
  ▼
User Answer & Mistake Classification (1-4 Reflection / Calibrated Rating)
  │ (Withholds solution during reflection; reveals derivation post-classification)
  ▼
Learner-State Firewall & Telemetry (practice_attempts, skill_states, error_events)
    (Source question content remains 100% static and immutable)
```

---

## 2. Canonical Architecture & Subsystems

### 2.1 Canonical Fields & Model
The canonical APKG contract represents immutable, curated source questions:
- **Content Fields:** `Prompt` (required), `Options` (required for MCQ, >= 2 non-empty items), `CorrectAnswer` (required; resolves to option for MCQ, finite float for Numerical), `Hint` (optional), `Solution` (optional), `Steps` (optional), `Explanation` (optional).
- **Semantic Fields:** `QuestionType` (required: `"mcq"`, `"numerical"`; never inferred), `Subject` (optional), `Chapter` (optional), `Topic` (optional), `Skill` (optional), `ProblemType` (optional), `Difficulty` (optional: finite float in `[1.0, 5.0]`).
- **Provenance Fields:** `Source` (optional), `Exam` (optional), `Year` (optional: 4-digit integer string), `Shift` (optional), `Paper` (optional), `SourceQuestionID` (optional).

### 2.2 Ingestion & Interception
- Notes with notetype name starting with `"StudyLab Source"` are recognized as canonical source questions.
- Rust rendering hook [`rslib/src/notetype/render.rs`](file:///c:/Users/Suraj/Documents/Antigravity/Anki-maths/rslib/src/notetype/render.rs) intercepts source notes via `render_source_anchor`, bypassing dynamic variant generators.
- Python Qt bridge [`qt/aqt/reviewer.py`](file:///c:/Users/Suraj/Documents/Antigravity/Anki-maths/qt/aqt/reviewer.py) `_is_procedural_card` recognizes `"StudyLab Source"` notes to control ease button suppression and mistake reflection.

### 2.3 Storage & Reconciliation
- Stored in separate SQLite database (`collection.procedural`) inside the `practice_items` table.
- Deterministic hashing tracks `New`, `Updated`, `Unchanged`, and `Archived` lifecycle states.
- Multi-identity separation guarantees that Anki Note GUID (`guid`), StudyLab Runtime ID (`pi_src_<guid>`), and authored `SourceQuestionID` remain distinct without namespace collision.

### 2.4 Modality Purity & Reviewer
- **MCQ:** Renders discrete option radio cards (keys A-D / 1-4) with zero numerical input textboxes.
- **Numerical:** Renders numeric answer input box (`#proc-answer-input`) and submit button (`#proc-submit-btn`) with zero MCQ option groups.
- **Learning Support:** When `Hint`, `Solution`, `Steps`, or `Explanation` are present, they populate learning support containers; when absent, UI renders cleanly with zero `None`, `undefined`, `TODO`, or unresolved template placeholders.

### 2.5 Learner-State Firewall
- User attempts, latency, mistakes, mastery scores, and FSRS intervals mutate exclusively runtime-owned tables (`practice_attempts`, `skill_states`, `error_events`).
- Authored source content (`Prompt`, `Options`, `CorrectAnswer`, `Difficulty`, `Provenance`, `QuestionType`) remains 100% immutable in `practice_items`.

### 2.6 Media Integration
- Media assets referenced in HTML (e.g. `<img src="studylab_diagram.png">`) are extracted by standard Anki APKG unpacker into collection media store and rendered directly in the webview.

### 2.7 Standard Anki & Legacy Compatibility
- Standard Anki note types (`Basic`, `Cloze`, custom note types) pass through standard review untouched with zero overhead or DOM leakage.
- Legacy procedural blueprints (`StudyLab Procedural Anchor`) remain isolated and functional via `DeclarativeProblemGenerator`.

---

## 3. Adversarial Hardening Summary

During Phase 4 adversarial auditing, the following boundary protections were verified and hardened:

| Attack Vector | Boundary Tested | Verified Protection & Behavior | Status |
|---|---|---|---|
| **QuestionType Inference** | Attempted to omit or pass `garbage`, `essay`, `freeform` with `Options` present | QuestionType is **never** inferred from Options. Rejects unsupported strings with `SourceContractError::InvalidQuestionType`. | **VERIFIED** |
| **MCQ Options Edge Cases** | Multiline strings, 0/1 option, duplicate options, index answers (`1..4`), letter answers (`A..D`), prefix matches (`A) text`) | Correctly trims whitespace, filters empty strings, asserts >= 2 options, resolves letter/index/exact matches, rejects out-of-bounds keys. | **VERIFIED** |
| **Numerical Answer Attacks** | Integers, decimals, negative floats, scientific notation (`1.5e4`), `NaN`, `inf`, text, units | Enforces finite float parsing (`is_finite()`). Rejects `NaN`, `inf`, `-inf`, and non-numeric strings with `InvalidCorrectAnswer`. | **VERIFIED** |
| **Difficulty Boundary Attacks** | `0.999`, `1.0`, `2.5`, `5.0`, `5.001`, `-1.0`, `100.0`, `"NaN"`, `"hard"` | Enforces strict finite range `[1.0, 5.0]`. Rejects out-of-bounds values with `InvalidDifficulty`. | **VERIFIED** |
| **Provenance Attacks** | Malformed years (`"twenty twenty four"`, `"2024.5"`, `"abc"`), special characters in `SourceQuestionID` | Validates 4-digit integer years; preserves raw `SourceQuestionID` without dummy substitutions. | **VERIFIED** |
| **Identity Separation** | Distinct notes sharing the same `SourceQuestionID` | Notes keyed by Anki Note GUID (`pi_src_<guid>`); both notes inserted and resolved independently without collision. | **VERIFIED** |
| **Learner-State Mutation** | Multiple correct/incorrect user attempts and mistake reflections | `PracticeItem` difficulty, prompt, options, correct answer, provenance remain 100% static in storage. | **VERIFIED** |
| **Malformed Note Render** | Corrupted note field in Anki collection at render time | Emits user-friendly `<div class='proc-error'>Source Engine Error</div>` without crashing Anki process. | **VERIFIED** |
| **Standard Anki Isolation** | Rendering Basic, Cloze, Custom cards alongside Source cards | Standard cards completely bypass StudyLab hooks with zero procedural DOM/event interception. | **VERIFIED** |

---

## 4. Frozen Architectural Invariants

The following invariants are **FROZEN** and must not be altered:

1. **Contract Invariant:** `StudyLab-Source-APKG-Contract(1).txt` is the authoritative source of truth.
2. **Source Immutability Invariant:** Imported source question fields are strictly immutable from learner interaction.
3. **Learner-State Firewall Invariant:** Learner telemetry and scheduling state belong exclusively to StudyLab runtime storage.
4. **QuestionType Explicitness Invariant:** QuestionType is never inferred from Options presence; only runtime-supported types (`mcq`, `numerical`) are playable.
5. **Difficulty Integrity Invariant:** Authored source `Difficulty` is a static property in `[1.0, 5.0]` and is never overwritten by learner performance.
6. **Multi-Identity Separation Invariant:** Anki Note GUID, `PracticeItemId`, and `SourceQuestionID` remain distinct entities.
7. **Modality Purity Invariant:** MCQ renders radio cards with zero textboxes; Numerical renders numeric input with zero option groups.
8. **Media Invariant:** APKG media uses the standard Anki media store without secondary media subsystems.
9. **Compatibility Invariant:** Legacy procedural generation remains isolated and does not redefine the canonical source contract.

---

## 5. Verification Evidence & Test Results

All verification suites pass 100% cleanly:

| Test Suite | Command | Result | Verification Scope |
|---|---|---|---|
| **Runtime E2E Integration** | `cargo test -p anki --test canonical_source_apkg_runtime_e2e_tests` | **10/10 PASS** (0.38s) | End-to-end import, reconciliation, MCQ/Numerical rendering, media, firewall, isolation, error handling |
| **Contract & Adversarial** | `cargo test -p procedural --test canonical_source_contract_tests` | **18/18 PASS** (0.01s) | Field validation, QuestionType attacks, MCQ/Numerical attacks, Difficulty bounds, Provenance, Identity |
| **Procedural Engine Lib** | `cargo test -p procedural --lib` | **146/146 PASS** (0.13s) | Problem generators, CAS validators, unit registries, Bayesian scheduling, SQLite migrations |
| **TypeScript Reviewer** | `npm run vitest:once` | **165/165 PASS** (8.29s) | 19 test files: state machine, MCQContainer, NumericalContainer, StepwiseContainer, keyboard trap |
| **Python APKG Validator** | `python3.11 artifacts_qa/validate_canonical_source_apkg.py artifacts_qa/canonical_source_test_fixture.apkg` | **PASS** (0.05s) | Schema conformance, 5/5 valid notes, notetype inspection |
| **Workspace Build** | `cargo check --workspace` | **PASS** (7.72s) | Clean compilation across all crates with zero warnings |

---

## 6. Known Limitations & Intentional Non-Goals

1. **Non-Goal: Dynamic Variant Generation for Source PYQs:** Canonical source questions are static by definition. Procedural parameter mutation is intentionally excluded from the source-first path.
2. **Non-Goal: Freeform Essay / Subjective Card Modalities:** Only discrete choice (`mcq`) and numerical input (`numerical`) are supported by the canonical contract.
3. **Limitation: Offline Tolerance Specification:** APKG source notes do not declare numerical tolerance; numerical comparisons evaluate exact numeric float equivalence with runtime standard floating point epsilon.
