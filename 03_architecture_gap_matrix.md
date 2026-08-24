# StudyLab Architecture Gap Matrix (Phase 03)

**Author**: StudyLab Architecture Auditor  
**Date**: 2026-08-24  
**Status**: Authoritative Architectural Audit  
**Target Repository**: `Anki-maths` (`rslib/procedural`, `qt/aqt`, `pylib`, `ts/reviewer`)

---

## Executive Summary

This document provides a comprehensive, evidence-based architectural audit comparing the original design principles and specifications (`ORIGINAL_REQUEST.md`, `PROJECT.md`, `docs/procedural_architecture.md`) against the actual implementation across the repository.

### Subsystem Audit Summary

| Subsystem | Audit Focus | Alignment Status | Primary Finding / Gaps |
|---|---|---|---|
| **1. Reviewer UI & Webview Integration** | Interception hooks, DOM isolation, MathJax, Memory leaks | 🟡 PARTIAL | Core interception in `render.rs` is solid; potential keydown event listener leak on transition to standard cards. |
| **2. State Machines** | Card states, Review states, Diagnostic session states | 🟡 PARTIAL | UI solving/feedback state machine functions well; Diagnostic session engine in Rust lacks frontend state coordinator. |
| **3. Native Bridge & FFI Contracts** | Telemetry serialization, `bridgeCommand`, `custom_data` | 🔴 GAP IDENTIFIED | `_linkHandler` in `reviewer.py` drops `procedural_*` commands as no-ops; telemetry relies entirely on `mutateNextCardStates`. |
| **4. Answer Controls** | MCQ, Numerical, Stepwise modalities | 🔴 GAP IDENTIFIED | MCQ & Numerical UI parsing are operational; Stepwise validation in TS completely bypasses Rust `StepValidator`. |
| **5. Footer & Review Lifecycle** | Compact mistake classification (`[1 Silly]..[4 Unknown]`), Ease | 🟡 PARTIAL | Mistake panel is styled inside card body webview rather than native `bottom.web`; automatic ease calculation bypasses user rating. |
| **6. Learner State & Evidence Sync** | `MasteryEvidence`, `DomainEvidence`, `SkillState`, `procedural.db` | 🟡 PARTIAL | Review attempt telemetry syncs cleanly to `procedural.db`; diagnostic mock report results do not batch-update `SkillState`. |

---

## Formal Gap Matrix

| Gap ID | Subsystem | Original Principle / Requirement | Current Implementation State | Severity | Recommended Fix Strategy |
|---|---|---|---|---|---|
| **`GAP-MOD-01`** | **Answer Controls / Stepwise Modality** | **R2 / Feature 5**: Stepwise semantic step evaluation wired directly to the canonical Rust `StepValidator` without duplicate TS reasoning engines. | In `ts/reviewer/procedural.ts:746-760` (`handleStepwiseSubmit`), only the final step string is taken (`lastAnswer = steps[steps.length - 1]`) and evaluated via local scalar check `evaluateLocally(lastAnswer)`. The rich multi-step graph validation in `rslib/procedural/src/problems/steps/step_validator.rs:13-120` is completely bypassed. | **HIGH** | Wire stepwise submissions via a dedicated bridge command (`procedural_validate_steps`) to Rust `StepValidator::validate_submission()`, returning step-by-step validation status (`Valid`, `Invalid`, `PartiallyValid`) and taxonomic error flags to the UI. |
| **`GAP-BRG-01`** | **Native Bridge & FFI Contracts** | **R1, R3 / PROJECT.md Interface Contracts**: Robust bidirectional bridge between TS Webview, Python Add-on, and Rust Core for telemetry, remediation actions, and hints. | In `qt/aqt/reviewer.py:711-713`, `elif url.startswith("procedural_"): pass`. Bridge commands sent by TS (`procedural_hint:`, `procedural_try_similar:`, `procedural_declarative_recall:`, `procedural_practice_prerequisite:`, `procedural_attempt:`) are ignored as no-ops. If `mutateNextCardStates` fails, all telemetry is lost. "Try Similar" and "Practice Prerequisite" buttons have no backend effect. | **HIGH** | Implement Python dispatcher in `qt/aqt/reviewer.py` for `procedural_*` URLs to call `ProceduralService` methods, trigger remediation workflows, and dispatch prerequisite card reviews. |
| **`GAP-DIAG-01`** | **Diagnostic Session Engine & UI** | **R4 / Feature 9, 10**: Bounded Diagnostic Session (10-20 questions across Math, Reasoning, Physics, Chemistry) in measuring mode with hierarchical report (Subject -> Chapter -> Topic -> Family). | `rslib/procedural/src/exam/mock.rs:18-550` implements `MockSession`, `MockBlueprint`, and `ComprehensiveDiagnosticReport`, but `ProceduralService` (`service/mod.rs`) does not expose mock methods, and there is no UI view or route in `ts/` or `qt/` to start, navigate, submit, or render a diagnostic report. | **HIGH** | Expose `start_diagnostic_session()`, `record_diagnostic_answer()`, and `finalize_diagnostic()` on `ProceduralService`, and build a dedicated Diagnostic webview container in `ts/` to render questions and the hierarchical report. |
| **`GAP-EV-01`** | **Learner State & Evidence Sync** | **R4 / Feature 11**: Diagnostic session results feed directly into `SkillState`, `MasteryEvidence`, and `DomainEvidence` without parallel state models. | In `rslib/procedural/src/exam/mock.rs:444-550`, `generate_comprehensive_report()` calculates 4-dimension errors and weak skills in memory, but there is no mechanism writing these aggregate diagnostic findings back into `skill_states` or `domain_evidence` tables in `procedural.db`. | **HIGH** | Implement `ProceduralService::record_diagnostic_report_evidence(report)` to iterate over `report.hierarchy` and update `SkillState` mastery scores, speed statistics, and domain evidence records in `procedural.db`. |
| **`GAP-FTR-01`** | **Mistake Footer & Review Lifecycle** | **R3 / Feature 7**: Compact mistake classification footer (`[1 Silly]`, `[2 Pattern]`, `[3 Concept]`, `[4 Unknown]`) in native Anki answer/footer interaction zone without scrolling. | In `rslib/procedural/src/reviewer/template.rs:555-571` and `ts/reviewer/procedural.ts:780-800`, the mistake panel (`#proc-mistake-panel`) is inside the card body webview (`self.web`). Anki's native bottom bar (`self.bottom.web`) remains in "Show Answer" state until mistake classification completes. | **MEDIUM** | Harmonize the mistake classification lifecycle with `self.bottom.web` or seamlessly anchor the classification strip at the bottom viewport boundary, ensuring single-keystroke 1-4 classification immediately transitions to solution reveal and ease rating. |
| **`GAP-MOD-02`** | **Numerical Modality Unit Conversion** | **R2 / Feature 4**: Dedicated numeric input accepting units (`m/s`, `kg`, `mol`, `mol/L`), tolerances, fractions, and scientific notation with dimensional correctness. | In `ts/reviewer/procedural.ts:615-663`, `parseNumericValue` strips units using regex float extraction (e.g. `12 m/s` -> `12`). It does not perform unit conversion (e.g., converting `72 km/h` to `20 m/s`). Rust's `DimensionalValidator` (`physics/units.rs`) and `ChemicalDimensionalValidator` (`chemistry/units.rs`) are not invoked client-side. | **MEDIUM** | Augment numerical evaluation to handle standard unit equivalencies or delegate multi-unit checks to backend validators for Physics/Chemistry problem families. |
| **`GAP-STA-01`** | **Webview Lifecycle & Event Listener Cleanup** | **R5 / Feature 13**: Zero unhandled console errors, no memory leaks or stale event listeners on card transitions. Non-procedural cards remain 100% unaffected. | In `ts/reviewer/procedural.ts:297`, `this.addListener(window, "keydown", ...)` attaches a global window listener. `destroy()` is called only when another procedural card calls `setup()`. If the user navigates to a standard Anki card (Basic, Cloze), `setup()` is not called, leaving the global keydown listener active on `window`. | **MEDIUM** | Ensure `ProceduralReviewer.destroy()` is registered with Anki's card transition lifecycle or teardown signals so event listeners are removed immediately upon leaving the card. |
| **`GAP-SCH-01`** | **Automatic Ease Rating vs Manual Override** | **R1, R3**: Seamlessly update learner evidence without breaking standard Anki scheduling or overriding user ease selection. | In `ts/reviewer/procedural.ts:1057-1059`, `handleNext()` hardcodes `ease = isCorrect ? (isFast ? 4 : 3) : 1` and issues `bridgeCommand("procedural_answer:${ease}")`, which directly answers the card in Anki without giving the user the option to select "Hard" (ease 2) or adjust ease. | **LOW** | Present the calculated ease as a recommendation while allowing full keyboard/click access to native ease buttons (1-4) during the feedback state. |
| **`GAP-MOD-03`** | **MCQ Answering Mode in Mocks vs Practice** | **R2, R4**: Authentic selectable options for both instant practice drills and mock exam conditions. | In `ts/reviewer/procedural.ts:522-613`, selecting an option immediately triggers submission and reveals feedback. In a 10-20 question diagnostic mock test, users must be able to select, change options, and mark questions for review before submitting. | **LOW** | Support a `mode: "practice" | "mock"` configuration flag in `procedural.ts`: practice mode allows instant grading; mock mode records the choice without revealing answers until the session is submitted. |
| **`GAP-DOC-01`** | **Architectural Directory Nomenclature** | **Layout Compliance**: Accurate structural mapping across documentation and codebase. | `PROJECT.md` references `crates/anki_maths_core`, `addon/anki_maths`, `web/`, whereas the workspace integrates the engine in-tree at `rslib/procedural/`, `qt/aqt/`, and `ts/reviewer/procedural.ts`. | **LOW** | Reconcile architectural documentation to explicitly map logical component names to their physical in-tree paths. |

---

## Detailed Architectural Analysis by Subsystem

### 1. Reviewer UI and Webview Integration
- **Interception Mechanism**:
  - `rslib/src/notetype/render.rs:123-126`: Checks `nt.name.as_str().starts_with("StudyLab Procedural Anchor") && !browser`.
  - When true, executes `render_procedural_anchor()`, extracting payload via `ProceduralCardAnchor::extract_from_card_fields(&note.fields())` (`render.rs:222`), acquiring `ProceduralService` (`render.rs:228`), resolving the practice session, and calling `procedural::reviewer::render_reviewer_html(&session)` (`render.rs:238`).
  - **Verdict**: Clean, isolated, and highly performant. Upstream Anki card rendering for standard cards remains untouched.
- **Frontend Container & API**:
  - `ts/reviewer/index.ts:19` and `ts/reviewer/reviewer_extras.ts:22` expose `globalThis.anki.procedural = proceduralAPI`.
  - `proceduralAPI.setup()` (`ts/reviewer/procedural.ts:1095`) binds to `#procedural-card` or `.procedural-card-container`.
  - Memory leak protection exists within `procedural.ts` (disposables list, interval cancellation), but requires cross-card teardown hooking (`GAP-STA-01`).

### 2. State Machines & Lifecycle
- **Procedural UI State Machine (`ts/reviewer/procedural.ts:12-23`)**:
  - States: `loading` -> `ready` -> `solving` -> `hint` / `submitting` -> `mistake_classification` -> `feedback` -> `worked_example` / `next` -> `teardown`.
  - **Review State Transition**:
    - Submission triggers `bridgeCommand("ans")` (`procedural.ts:1019`), moving Anki reviewer to `self.state = "answer"`.
    - Next triggers `bridgeCommand("procedural_answer:${ease}")` (`procedural.ts:1058`), answering the card.
- **Diagnostic State Machine**:
  - Implemented in Rust `MockSession` (`rslib/procedural/src/exam/mock.rs:202-281`), tracking question list, current index, answers map, marked-for-review set, and time tracking.
  - Currently missing TS/Python glue (`GAP-DIAG-01`).

### 3. Native Bridge & FFI Contracts
- **Telemetry Flow**:
  - Webview mutates Anki card states: `globalThis.anki.mutateNextCardStates(..., customData[state].studylab = telemetry)` (`ts/reviewer/procedural.ts:995-1003`).
  - When card is answered, Rust scheduler (`rslib/src/scheduler/answering/mod.rs:350-510`) extracts `studylab`, records `PracticeAttempt` and `ErrorEvent` into `procedural.db`, enqueues remediation in `service.remediation_queue()`, and strips `studylab` from `custom_data` to comply with the 100-byte DB limit.
- **Link Handler Bridge**:
  - `qt/aqt/reviewer.py:684-716`: Handles `ans`, `ease<N>`, `procedural_answer:<N>`.
  - Line 711 (`elif url.startswith("procedural_"): pass`) drops auxiliary signals (`GAP-BRG-01`).

### 4. Answer Controls & Modalities
- **MCQ**: Authentic buttons, radio ARIA attributes, 1-4 and A-D shortcuts, canonical comparison (`procedural.ts:557-604`).
- **Numerical**: `parseNumericValue` handles prefixes (`v =`), scientific notation (`1.2e-3`), fractions (`3/4`), and unit extraction (`12m/s`) (`procedural.ts:615-663`).
- **Stepwise**: TS UI allows adding steps and requesting hints, but submission only checks the final step string locally, ignoring Rust's `StepValidator` (`GAP-MOD-01`).

### 5. Footer & Mistake Classification
- Compact mistake strip (`[1 Silly]`, `[2 Pattern]`, `[3 Concept]`, `[4 Unknown]`) in `template.rs:555-571`.
- Keyboard 1-4 triggers instant classification (`procedural.ts:338-358`).
- Rendered inside the card DOM `#proc-result-panel` rather than `reviewer_bottom` (`GAP-FTR-01`).

### 6. Learner State & Evidence Synchronization
- `MasteryEvidence`, `DomainEvidence`, and `SkillState` structures are well-defined (`rslib/procedural/src/skills/domain_evidence.rs`, `signals.rs`).
- Telemetry from review cards properly updates `SkillState` in `procedural.db` (`answering/mod.rs:430`).
- Mock/diagnostic report ingestion into `SkillState` is pending implementation (`GAP-EV-01`).

---

## Prioritized Remediation Roadmap

1. **Phase A (Critical Modality & Bridge Wiring)**:
   - Wire `handleStepwiseSubmit` in `procedural.ts` to call Rust `StepValidator` (`GAP-MOD-01`).
   - Implement Python `procedural_*` command handlers in `reviewer.py` (`GAP-BRG-01`).
   - Add teardown hook on card exit to prevent event listener leaks (`GAP-STA-01`).

2. **Phase B (Diagnostic Session Engine & UI)**:
   - Expose `MockSession` methods on `ProceduralService` (`GAP-DIAG-01`).
   - Create Diagnostic test runner & Hierarchical Report UI in `ts/` (`GAP-DIAG-01`).
   - Wire diagnostic report results to batch-update `SkillState` in `procedural.db` (`GAP-EV-01`).

3. **Phase C (UX & Modality Polish)**:
   - Refine footer mistake classification synchronization (`GAP-FTR-01`).
   - Enhance numerical unit conversion handling for Physics/Chemistry (`GAP-MOD-02`).
   - Provide manual ease rating confirmation in review feedback (`GAP-SCH-01`).
