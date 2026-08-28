> **NOTE**: The canonical source of truth for all frontend UI/UX, modalities, state transitions, and interaction invariants is now defined in [FRONTEND_PRODUCT_SPEC.md](./FRONTEND_PRODUCT_SPEC.md). Any frontend-specific implementation details in this document are superseded by the new specification.

# StudyLab Architecture Invariants & Frozen Non-Negotiables

**Document Version:** 1.0.0 (Canonical Master Specification)  
**Target Repository:** `Anki-maths` (StudyLab Procedural Intelligence Subsystem)  
**Status:** AUTHORITATIVE FROZEN SPECIFICATION  
**Integrity Mode:** Benchmark Mode (100% Grounded in Executable Code, Tests, and Architectural Directives)  

---

## 1. Executive Intent & Enforcement Authority

This document defines the **16 Frozen Architecture Invariants** of StudyLab. These rules are strictly non-negotiable for all software engineers, AI agents, content authors, and QA verifiers working on the codebase.

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                            BENCHMARK INTEGRITY MANDATE                           │
├──────────────────────────────────────────────────────────────────────────────────┤
│ ALL IMPLEMENTATIONS MUST BE GENUINE.                                             │
│                                                                                  │
│ • DO NOT create dummy or facade implementations that produce correct-looking     │
│   outputs without genuine underlying logic.                                      │
│ • DO NOT hardcode test results, expected outputs, or verification strings.       │
│ • DO NOT silently drift from architectural invariants or bypass validation.      │
│ • Every component must maintain real state and produce real behavior.            │
└──────────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. The 16 Non-Negotiable Architecture Invariants

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                         THE 16 ARCHITECTURE INVARIANTS                           │
├────┬─────────────────────────────────┬────┬──────────────────────────────────────┤
│ #  │ Invariant Name                  │ #  │ Invariant Name                       │
├────┼─────────────────────────────────┼────┼──────────────────────────────────────┤
│ 1  │ Not a Flashcard System          │ 9  │ Blueprint vs History Ownership       │
│ 2  │ Do Not Recreate Anki / FSRS     │ 10 │ Zero-Rust Declarative Authoring      │
│ 3  │ Problem-Solving Workspace UI    │ 11 │ No Internal Leakage to Learner       │
│ 4  │ Modality-Matched Semantics      │ 12 │ Single Canonical Evaluation Source   │
│ 5  │ Semantic Input Validation       │ 13 │ Standard Anki Zero Non-Regression    │
│ 6  │ Canonical Stepwise Validation   │ 14 │ Diagnostic Unified Learner Model     │
│ 7  │ Unified SkillState Progression  │ 15 │ Tier 1 Inline Contract Precedence    │
│ 8  │ Orthogonal Diagnostic Evidence  │ 16 │ Docs as Supreme Source of Truth      │
└────┴─────────────────────────────────┴────┴──────────────────────────────────────┘
```

---

### Invariant 1: StudyLab Is Not a Flashcard System
- **Invariant Statement:** StudyLab is an adaptive procedural problem-solving engine hosted inside Anki; it is NEVER an "enhanced flashcard reviewer", "math addon", or "interactive quiz deck".
- **Pedagogical Rationale:** Flashcards optimize declarative paired-associate memory retrieval ($Q \to A$). Problem solving requires compiling and executing cognitive production rules across variable parameter spaces. Conflating procedural practice with flashcard recall induces the *Illusion of Competence*.
- **Executable Code Evidence:** `rslib/src/notetype/render.rs:122–126` (`render_procedural_anchor`); `rslib/procedural/src/core/mod.rs` (`Domain`); `ts/reviewer/procedural.ts:1–50` (`ProceduralReviewer`).
- **Test Evidence:** `rslib/procedural/tests/desktop_validation_master_suite.rs` (Section 1 & 6); `ts/reviewer/procedural.test.ts`.
- **Violation Failure Mode:** Reduces problem solving to static text recall; prevents parametric variation and diagnostic error attribution.

---

### Invariant 2: Do Not Recreate Anki's Responsibilities
- **Invariant Statement:** Anki owns windowing, profiles, media sync, collection storage (`collection.anki2`), and temporal spaced repetition (FSRS/SM-2). StudyLab intelligence lives strictly in `procedural.db` and `rslib/procedural/`.
- **Pedagogical Rationale:** Dividing responsibilities preserves Anki's battle-tested desktop synchronization and scheduling mathematical foundations while isolating procedural intelligence.
- **Executable Code Evidence:** `rslib/src/collection/mod.rs:141, 173–183` (`ProceduralService::open`); `rslib/procedural/src/storage/store.rs` (`ProceduralStore`); `rslib/procedural/src/scheduling/rating_policy.rs` (`derive_fsrs_rating`).
- **Test Evidence:** `rslib/procedural/tests/defect_remediation_tests.rs` (`test_fsrs_rating_scenarios_regression`); `qt/tests/test_phase13.py`.
- **Violation Failure Mode:** Schema corruption in `collection.anki2`, broken AnkiWeb synchronization, or conflicting spaced repetition intervals.

---

### Invariant 3: Reviewer UI Follows Problem-Solving Workflow
- **Invariant Statement:** The reviewer webview operates as an interactive solving workspace (`loading` $\rightarrow$ `ready` $\rightarrow$ `solving` $\rightarrow$ `submitting` $\rightarrow$ `mistake_classification` $\rightarrow$ `feedback` $\rightarrow$ `next`), NOT a card flip/reveal workflow (`front` $\rightarrow$ `back`).
- **Pedagogical Rationale:** Active generation and problem solving require stopwatch tracking, live validation feedback, progressive scaffolding hints, and post-error reflection trapping.
- **Executable Code Evidence:** `ts/reviewer/procedural.ts:12–25` (`ProceduralUIState` with 11 states); `ts/reviewer/procedural.ts:310–360` (Space/Enter trapping in `mistake_classification`).
- **Test Evidence:** `ts/reviewer/procedural.test.ts` (27 tests); `05_live_ui_screenshots/03_mistake_footer.png`.
- **Violation Failure Mode:** Learners passively flip to the answer without executing intermediate steps, destroying the diagnostic feedback loop.

---

### Invariant 4: Correct Answer Modality Must Match Learning-Object Semantics
- **Invariant Statement:** Answer modalities must strictly reflect domain semantics: `mcq` requires ARIA radiogroups with keyboard hotkeys (`1`–`4`, `A`–`D`); `numerical` requires 5D dimensional vectors with unit registries; `stepwise` requires multi-node derivation graphs; `worked_example` requires faded solution steps.
- **Pedagogical Rationale:** Eliminates extraneous typing friction and matches cognitive load to the target learning construct (Sweller 1988).
- **Executable Code Evidence:** `ts/reviewer/components/mcq_container.ts` (`MCQContainer`, `enforceZeroTextInputFallback`); `ts/reviewer/components/numerical_container.ts` (`NumericalContainer`, `PhysicalDimension`, `UnitRegistry`); `ts/reviewer/components/stepwise_container.ts` (`StepwiseContainer`).
- **Test Evidence:** `ts/reviewer/components/mcq_container.test.ts` (12 tests); `ts/reviewer/components/numerical_container.test.ts` (28 tests); `ts/reviewer/components/stepwise_container.test.ts` (7 tests).
- **Violation Failure Mode:** Text input fallback on MCQ cards creates typing friction; raw float parsing fails to validate units ($72\text{ km/h} \neq 20\text{ m/s}$).

---

### Invariant 5: Generic Fill-in Input Is Not a Universal Modality
- **Invariant Statement:** Numeric and algebraic inputs must never rely on basic string matching. They must be semantically validated for dimensional consistency, unit conversions, and mathematical equivalence within defined tolerance bands.
- **Pedagogical Rationale:** Prevents false negatives from valid alternative representations (e.g. `$1,250.50`, `75%`, `1.2e-3 mol/L`, `3/4`, `1200 mV`).
- **Executable Code Evidence:** `ts/reviewer/components/numerical_container.ts` (`NumericalParser`, `UnitRegistry`); `rslib/procedural/src/units/` (`UnitParser`, `UnitAnswerValidator`).
- **Test Evidence:** `ts/reviewer/components/numerical_container.test.ts`; `rslib/procedural/src/units/` (8 unit tests).
- **Violation Failure Mode:** Rejection of valid scientific answers causing learner frustration and inaccurate skill degradation.

---

### Invariant 6: Stepwise Uses Canonical Semantic Validation
- **Invariant Statement:** Intermediate derivation steps are validated using graph-level semantic equivalence (`StepValidator`), solving linear roots and evaluating commutative equivalence rather than brittle string matching.
- **Pedagogical Rationale:** Step-based intelligent tutoring achieves $d \approx 0.76$ effect size by evaluating intermediate reasoning steps without forcing arbitrary algebraic formatting.
- **Executable Code Evidence:** `rslib/procedural/src/problems/steps/step_validator.rs` (`StepValidator`, `MathSemanticComparator`); `ts/reviewer/components/stepwise_container.ts`.
- **Test Evidence:** `rslib/procedural/tests/step_interaction_tests.rs` (8 tests); `rslib/procedural/tests/maths_vertical_slice_tests.rs`.
- **Violation Failure Mode:** Valid intermediate steps (e.g. `2x + 6 = 16` vs `2x = 10`) rejected as incorrect, or compounding penalties for downstream consistent steps.

---

### Invariant 7: Learner State Is Unified Across Content Origins
- **Invariant Statement:** `SkillState` tracking, EMA mastery updates, and composite progression gates ($New \to Mastered$) operate identically regardless of whether content originated via `inline_contract`, `content_ref`, or authentic PYQ.
- **Pedagogical Rationale:** The learner model must reflect true cognitive competence across all sources without bifurcated or siloed mastery pools.
- **Executable Code Evidence:** `rslib/procedural/src/skills/mod.rs` (`SkillState`, `record_attempt`); `rslib/procedural/src/skills/progression.rs` (`ProgressionPolicy`).
- **Test Evidence:** `rslib/procedural/tests/phase28_domain_evidence_contract.rs` (7 tests); `phase29_domain_evidence_adaptive.rs`.
- **Violation Failure Mode:** Inconsistent mastery ratings and redundant practice scheduling across different card sources.

---

### Invariant 8: Domain Evidence Is Diagnostic, Not Fake Precision
- **Invariant Statement:** Diagnostic error attribution must distinguish execution slips from conceptual misunderstandings, strategy selection failures, representation errors, and missing prerequisites (`is_execution_error()` vs `is_conceptual_error()`).
- **Pedagogical Rationale:** Grounded in the Assessment Triangle (Pellegrino 2001). An arithmetic slip with correct governing principle setup must not demote conceptual mastery.
- **Executable Code Evidence:** `rslib/procedural/src/skills/domain_evidence.rs` (`DomainEvidencePayload`, `MathEvidence`, `PhysicsEvidence`, `ChemistryEvidence`, `ReasoningEvidence`, `is_execution_error()`, `is_conceptual_error()`).
- **Test Evidence:** `rslib/procedural/tests/phase28_domain_evidence_contract.rs`; `rslib/procedural/tests/phase29_domain_evidence_adaptive.rs`.
- **Violation Failure Mode:** Resetting a student's physics progression to beginner level because of a minor rounding slip on the final arithmetic operation.

---

### Invariant 9: APKG Owns Definitions; Runtime Owns Learner History
- **Invariant Statement:** `.apkg` files are static definitions (declarative blueprints for procedural cards; immutable curated questions conforming to `StudyLab-Source-APKG-Contract(1).txt` for source cards). The runtime executes or reconciles problems and persists historical attempts exclusively in `collection.procedural`.
- **Pedagogical Rationale:** Separating static curricular definitions from dynamic attempt history enables universal deck sharing without leaking user-specific attempt data.
- **Executable Code Evidence:** `rslib/procedural/src/anchor/mod.rs` (`ProceduralCardAnchor`, `SourceQuestion`); `rslib/procedural/src/storage/store.rs` (`ProceduralStore`).
- **Test Evidence:** `rslib/procedural/tests/phase35_apkg_self_contained.rs` (2 tests); `rslib/tests/canonical_source_apkg_runtime_e2e_tests.rs`.
- **Violation Failure Mode:** Bloated deck exports containing private user attempt histories, or shared decks overwriting existing learner states.

---

### Invariant 10: Zero-Rust Declarative Authoring for Ordinary Topics
- **Invariant Statement:** Ordinary new academic topics must use the universal declarative archetype engine (`tools/studylab_content_factory.py`, `DeclarativeProblemGenerator`); compiling new Rust generator code is strictly reserved for genuinely novel execution engines.
- **Pedagogical Rationale:** Enables rapid, scalable curricular authoring across hundreds of STEM topics without binary rebuilds.
- **Executable Code Evidence:** `rslib/procedural/src/problems/contract.rs` (`DeclarativeFamilyContract`, 15 `ParameterDomain` variants, 24 `AnswerDerivation` variants); `rslib/procedural/src/problems/declarative.rs`.
- **Test Evidence:** `rslib/procedural/tests/phase36c_all_175_topics_factory_tests.rs` (175 topics rendered in 50.6ms with zero Rust generators).
- **Violation Failure Mode:** Codebase bloat with hundreds of redundant Rust files for basic mathematical formulas.

---

### Invariant 11: Internal Identifiers and Debug Metadata Never Leak to Learners
- **Invariant Statement:** Raw internal IDs (`family.math.percentage.successive`), debug strings, remediation tags, and internal error codes must never appear in the learner-facing UI.
- **Pedagogical Rationale:** Preserves a clean, professional, distraction-free educational environment.
- **Executable Code Evidence:** `rslib/procedural/src/reviewer/template.rs` (`escape_html`, clean LaTeX formatting); `ts/reviewer/components/mistake_footer.ts`.
- **Test Evidence:** `rslib/procedural/src/reviewer/tests` (`test_xss_escaping_and_latex_preservation`).
- **Violation Failure Mode:** Confusing technical jargon displayed to students during active problem solving.

---

### Invariant 12: Single Canonical Evaluation Source
- **Invariant Statement:** While the TypeScript frontend evaluates locally for zero-latency UI feedback, the Rust backend contract (`StepValidator`, `ProblemFamilyContract`, `SourceQuestion`) is the authoritative source of truth for answer evaluation and evidence compilation.
- **Pedagogical Rationale:** Prevents divergence between client-side JavaScript math approximations and backend database persistence.
- **Executable Code Evidence:** `ts/reviewer/answering.ts` (`mutateNextCardStates`); `rslib/src/scheduler/answering/mod.rs:353–505`.
- **Test Evidence:** `qt/tests/test_phase13.py`; `ts/reviewer/lib.test.ts`.
- **Violation Failure Mode:** Desynchronization where the webview marks an answer correct but the database records a failure.

---

### Invariant 13: Standard Anki Cards Remain Completely Untouched
- **Invariant Statement:** Interception in `rslib/src/notetype/render.rs` strictly targets notes whose note type name starts with `"StudyLab Source"` or `"StudyLab Procedural Anchor"`. Standard cards (`Basic`, `Cloze`) render through Anki's standard Mustache pipeline with zero overhead.
- **Pedagogical Rationale:** Guarantees 100% non-regression for existing user flashcard collections.
- **Executable Code Evidence:** `rslib/src/notetype/render.rs:122–126`; `qt/aqt/reviewer.py:674–679` (`_is_procedural_card`).
- **Test Evidence:** `qt/tests/` (84 passed in 30.50s); `pylib/tests/` (114 passed).
- **Violation Failure Mode:** Breaking standard flashcard rendering, sound playback, or cloze deletions.

---

### Invariant 14: Diagnostic Sessions Inform Standard Practice State
- **Invariant Statement:** Diagnostic mock-test sessions (`MockSession`) do not create a disconnected, parallel learner model; diagnostic outcomes are batch-synchronized into `SkillState` in `procedural.db`.
- **Pedagogical Rationale:** Baseline diagnostic measurements immediately inform daily adaptive practice and remediation scheduling.
- **Executable Code Evidence:** `rslib/procedural/src/exam/mock.rs:855–910` (`apply_diagnostic_report_to_store`); `rslib/procedural/src/service/mod.rs` (`record_diagnostic_report_evidence`).
- **Test Evidence:** `rslib/procedural/tests/diagnostic_mock_session_tests.rs` (5 passed in 0.04s).
- **Violation Failure Mode:** Redundant diagnostic re-testing and failure to adapt daily reviews to discovered weaknesses.

---

### Invariant 15: Tier 1 Inline Contract Is the Preferred Procedural Content Path
- **Invariant Statement:** For procedural decks, unless payload size is strictly prohibitive, declarative blueprints should be packaged directly into card anchors via `inline_contract`, ensuring self-contained deck portability.
- **Pedagogical Rationale:** Enables seamless deck distribution without requiring users to pre-seed external SQLite tables.
- **Executable Code Evidence:** `rslib/procedural/src/anchor/mod.rs` (`ProceduralCardAnchor::inline_contract`); `rslib/procedural/src/service/mod.rs:484–600`.
- **Test Evidence:** `rslib/procedural/tests/phase35_apkg_self_contained.rs` (2 tests).
- **Violation Failure Mode:** "Missing schema" crashes when importing decks onto fresh installations.

---

### Invariant 16: Documentation Hierarchy & Frozen Contract Primacy
- **Invariant Statement:** `StudyLab-Source-APKG-Contract(1).txt` is the frozen Level 1 authoritative contract for canonical Source APKGs. The canonical documentation suite in `docs/` is the master architectural specification for system components. Historical phase reports (`01_` through `08_`) and `.agents/` logs are archaeological context only.
- **Pedagogical Rationale:** Ensures that clean-context AI agents and human engineers have a single, unified, contradiction-free specification with explicit authority hierarchy.
- **Executable Code Evidence:** `docs/DOCUMENTATION_TRUTH_MATRIX.md` (Source-of-Truth Hierarchy); `PROJECT.md`.
- **Test Evidence:** Verified by the 15-Point Release Gate Audit (15/15 PASS) and Phase 4 Frozen Contract validation.
- **Violation Failure Mode:** Architectural confusion from stale phase reports, obsolete design assumptions, or ambiguous content pathways.

## 3. Security, Sandboxing & Memory Safety Invariants

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                         SECURITY & SAFETY INVARIANTS                             │
├────────────────────────────────┬─────────────────────────────────────────────────┤
│ Invariant Rule                 │ Enforcement Mechanism & Ground Truth             │
├────────────────────────────────┼─────────────────────────────────────────────────┤
│ **Output HTML Escaping**       │ All dynamic strings passed to webview templates │
│                                │ must pass through `escape_html()`               │
│                                │ (`rslib/procedural/src/reviewer/template.rs:18`) │
├────────────────────────────────┼─────────────────────────────────────────────────┤
│ **JSON Script Breakout**       │ JSON strings in `<script>` tags must escape     │
│                                │ `</script>` as `<\/script>`                     │
│                                │ (`reviewer/template.rs:35`)                     │
├────────────────────────────────┼─────────────────────────────────────────────────┤
│ **100% Parameterized SQL**     │ Zero SQL string interpolation; all queries must │
│                                │ use bound parameters `?1, ?2, ...` or           │
│                                │ `rusqlite::params!` (`storage/store.rs`)        │
├────────────────────────────────┼─────────────────────────────────────────────────┤
│ **Teardown & Memory Safety**   │ `destroyActive()` and `MutationObserver` must   │
│                                │ dispose all window event listeners upon card    │
│                                │ unmount (`ts/reviewer/procedural.ts:1239`)      │
└────────────────────────────────┴─────────────────────────────────────────────────┘
```

---

## 4. 15-Point Release Gate Verification Summary

The 15-point release decision checklist from `08_release_decision.md` confirms 100% compliance across all architectural invariants:

| # | Gate Requirement | Verification Command / Artifact | Status |
|---|---|---|:---:|
| 1 | Subsystem Gap Closure (10/10 Gaps) | `rslib/procedural/tests/`, `ts/reviewer/` | **PASS** |
| 2 | Multi-Domain Proofs (4 Domains) | `phase35_six_domain_proof.rs` | **PASS** |
| 3 | Zero-Rust Declarative Factory (175 Topics) | `phase36c_all_175_topics_factory_tests.rs` (50.6ms) | **PASS** |
| 4 | Memory Safety & Teardown (1,000 Transitions) | `desktop_validation_master_suite.rs` (3.09s, 0 leaks) | **PASS** |
| 5 | Database Isolation & Migrations (v1–v5) | `storage::tests`, `procedural.db` WAL | **PASS** |
| 6 | FSRS Rating Bridge & Ephemeral Stripping | `scheduler/answering/mod.rs:501` ($\le 100$ bytes) | **PASS** |
| 7 | Security Audit (HTML Escaping & SQL Params) | `reviewer::tests`, `storage/store.rs` | **PASS** |
| 8 | Python/Qt Bridge Dispatch (8 Commands) | `qt/tests/test_phase13.py`, `reviewer.py` | **PASS** |
| 9 | Vitest Reviewer Tests (150 Tests) | `npm run vitest:once` (18 test files) | **PASS** |
| 10 | Rust Unit & Integration Tests (208 Tests) | `cargo test --workspace` (100% pass) | **PASS** |
| 11 | Python GUI Tests (93 Tests) | `pytest qt/tests pylib/tests` (100% pass) | **PASS** |
| 12 | Live QtWebEngine CDP Screenshots (8 Views) | `05_live_ui_screenshots/` (SHA-256 verified) | **PASS** |
| 13 | 6-Gate Mastery Promotion Policy | `progression.rs`, 30-day simulation suite | **PASS** |
| 14 | 9-Tier Remediation Precedence & Circuit Breaker | `remediation::tests`, `queue.rs` | **PASS** |
| 15 | Diagnostic Mock Engine & 4-Tier Reports | `mock.rs`, `diagnostic_session.test.ts` | **PASS** |

---

*The 16 Architecture Invariants are hereby frozen as the canonical ground truth of StudyLab.*

