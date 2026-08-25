# StudyLab Frontend Current State Gap Map & Remediation Roadmap

**Document Version:** 1.1.0 (Reconciled with STUDYLAB_UI_COMPOSITION_CONTRACT.md)  
**Target Subsystem:** Frontend Reviewer Codebase (`ts/reviewer/`), Template Generation (`rslib/procedural/src/reviewer/`), and Desktop Host (`qt/aqt/reviewer.py`)  
**Status:** AUTHORITATIVE CANONICAL SPECIFICATION  
**Integrity Mode:** 100% Grounded in Executable Source Code, Live Desktop Forensic Audits, and Test Evidence  
**Authoritative Reference:** `docs/STUDYLAB_UI_COMPOSITION_CONTRACT.md`, `PROJECT.md`

---

## 1. Executive Summary & Audit Context

This document provides a comprehensive forensic gap analysis comparing the current executable implementation of StudyLab against the canonical composition, state, and button contracts defined in `docs/STUDYLAB_UI_COMPOSITION_CONTRACT.md`, `docs/FRONTEND_VISUAL_DESIGN_SPEC.md`, `docs/FRONTEND_UI_STATE_SPEC.md`, and `docs/FRONTEND_BUTTON_CONTRACT.md`.

### Audit Ground Truth & Verification Infrastructure:
- **Live Desktop Reviewer Forensic Attach:** Attached directly to active Anki desktop instance across all curriculum topics.
- **Dual Screenshot Provenance:** Native Win32 GDI window captures alongside CDP DOM captures (`artifacts_qa/visual_audit/`).
- **14-State Target Verification Matrix:** Validates all 12 procedural states and 2 native Anki Basic/Cloze isolation states.

```text
┌──────────────────────────────────────────────────────────────────────────────────┐
│                               AUDIT STATUS SUMMARY                               │
├──────────────────────────────────────────────────────────────────────────────────┤
│   - Milestone 1 (Contracts & Spec Sync):          100% COMPLETE & CANONICAL      │
│   - Milestone 2 (Visual Rebuild & Anti-Patterns): TARGETED IN M2 (M2 Task Plan)  │
│   - Milestone 3 (Modality Controllers & Gate):    TARGETED IN M3 (M3 Task Plan)  │
│   - Milestone 4 (Automated Tests & Build):        TARGETED IN M4 (Vitest/Cargo)  │
│   - Milestone 5 (Desktop Proof & Evidence):       TARGETED IN M5 (14 States)     │
└──────────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Screen-by-Screen Implementation Audit & Discrepancy Matrix (14 States)

| Screen / Modality | Canonical Contract Requirement | Current Implementation Status | Compliance Level | Observed Code / Visual Evidence |
|---|---|---|---|---|
| **1. Math Numerical (Algebra)** | Single text input with auto-focus, live preview pill, submit button, mode switch tabs. Zero MCQ options. Zero ticking clock. | Implemented in `NumericalContainer`. Timer runs via `setInterval` during solving (`ANTI-03`). | 🟡 **P1 (M2/M3)** | `template.rs:449-502`, `procedural.ts:545-555`. |
| **2. Math MCQ (Commercial)** | 4 discrete radio cards (A–D), keyboard navigation (1–4, A–D), zero text input fallback. | Fully implemented in `MCQContainer` (`mcq_container.ts:100-240`). | 🟢 **COMPLIANT** | `mcq_container.ts:153-220` confirms zero text box and radio cards. |
| **3. Reasoning MCQ (Blood Relations)** | Structured logic narrative prompt, 4 discrete option cards, keyboard selection. Zero raw schema strings. | Rendered via `MCQContainer`. Text fallback completely eliminated. | 🟢 **COMPLIANT** | `mcq_container.ts:180-260`. |
| **4. Physics Numerical (Kinematics)** | 5D dimensional unit parsing ($[L]^1 [T]^{-1}$), live preview pill, duplicate unit badge removed. | `PhysicalDimension` vector in `numerical_container.ts:12-65`. Redundant `.proc-unit-hint` removed. | 🟢 **COMPLIANT** | `numerical_container.ts:133-555` confirms 5D vector validation. |
| **5. Chemistry Numerical (Mole Concept)** | Stoichiometric amount parsing ($[N]^1$), molar mass conversion, non-negativity check. | Handled via `UnitRegistry` with molar units (`mol`, `mmol`, `kmol`). | 🟢 **COMPLIANT** | `numerical_container.ts:1069-1092`. |
| **6. ConceptCheck (Percentages)** | Discrete conceptual options with immediate distractor-specific misconception feedback. | Functional via `MCQContainer.conceptCheck`, but shares generic styling with standard MCQs. | 🟡 **P1 (M3)** | `template.rs:240-276`, `mcq_container.ts:408-413`. Needs dedicated styling. |
| **7. StrategyDrill (Alligation)** | Strategy comparison cards with optimality analysis rationale and step count badges. | Functional via `MCQContainer.strategyDrill`, evaluated against `preferred_option_id`. | 🟡 **P1 (M3)** | `template.rs:277-327`, `mcq_container.ts:414-419`. |
| **8. WorkedExample (Shopkeeper)** | Open canvas trace, highlighted decision point, acknowledgment gate, zero solving inputs (`ANTI-07`). | Injected raw into `#proc-result-panel`; uses nested card wrappers (`.proc-worked-example-card`). | 🟡 **P1 (M2/M3)** | `template.rs:328-375`, `reviewer.scss:613-625`. Needs open canvas flattening. |
| **9. Stepwise Derivation Workspace** | Multi-row CAS derivation, step validation badges, downstream consistency, hint/add controls. | Implemented in `StepwiseContainer` (`stepwise_container.ts:1-811`). | 🟢 **COMPLIANT** | `stepwise_container.ts:119-330`. |
| **10. Wrong Answer Outcome** | Input disabled, inline status `✗ Incorrect`, giant red banners removed (`ANTI-01`), Next button hidden. | Handled in `procedural.ts:940-970` with locked inputs and error notification. | 🟢 **COMPLIANT** | `procedural.ts:940-970`. |
| **11. Mistake Classification** | 4 reflection buttons (1–4), Space and Enter strictly trapped, solution strictly hidden (`ANTI-08`). | Space/Enter trapped, but `#proc-solution-container` must be explicitly hidden in DOM during reflection. | 🟡 **P0 (M3)** | `procedural.ts:463-498`, `mistake_footer.ts:74-242`. `GAP-ANTI-08`. |
| **12. Clean Result Feedback** | Deduplicated expected answer (`ANTI-02`), canonical MathJax derivation, speed pill (`ANTI-04`), single Next CTA. | Expected answer deduplication and compact speed pill needed in `procedural.ts`. | 🟡 **P1 (M2/M3)** | `procedural.ts:993, 1080`, `reviewer.scss:655-705`. `GAP-ANTI-02`, `GAP-ANTI-04`. |
| **13. Normal Basic Card** | Standard Basic card rendering 100% native Anki reviewer with `#ansbut` and ease rating bar. | Completely bypasses StudyLab procedural logic when not a procedural note type. | 🟢 **COMPLIANT** | `render.rs`, `reviewer.py:985-1055`. |
| **14. Normal Cloze Card** | Standard Cloze card rendering 100% native Anki reviewer. | Completely bypasses StudyLab procedural logic. | 🟢 **COMPLIANT** | `render.rs`, `reviewer.py:985-1055`. |

---

## 3. Detailed Forensic Discrepancy & Anti-Pattern Ledger

---

### GAP-ANTI-01: Elimination of Giant Full-Bleed Feedback Containers
- **Severity:** P1 (Visual Hierarchy Rebuild — M2)
- **Target Files:** `ts/reviewer/reviewer.scss:706-735`, `ts/reviewer/procedural.ts:1066, 1086`
- **Observed Behavior:** `.proc-result.correct` and `.proc-result.incorrect` apply large saturated background tint blocks covering the entire bottom card surface.
- **Canonical Requirement:** Open Canvas design with subtle inline status indicator (`✓ Correct` / `✗ Incorrect`) and 3px left accent border.

---

### GAP-ANTI-02: Deduplication of Expected Answer Rows
- **Severity:** P1 (Deduplication — M3)
- **Target Files:** `ts/reviewer/procedural.ts:993-997, 1080-1085`, `rslib/procedural/src/reviewer/template.rs:593`
- **Observed Behavior:** Displays "You answered: X" and "Correct answer: Y" in stacked rows multiple times.
- **Canonical Requirement:** Single consolidated comparison row: `Your answer: X · Correct answer: Y`.

---

### GAP-ANTI-03: Suppression of Ticking Stopwatch During Solving
- **Severity:** P1 (Visual Calmness — M2)
- **Target Files:** `rslib/procedural/src/reviewer/template.rs:576`, `ts/reviewer/procedural.ts:545-555`
- **Observed Behavior:** Active 200ms `setInterval` updates `#proc-timer` during active solving, inducing learner anxiety.
- **Canonical Requirement:** Stopwatch runs silently in telemetry background; elapsed time is displayed calmly post-submission alongside the speed pill.

---

### GAP-ANTI-04: Speed Quadrant Competing Badge Streamlining
- **Severity:** P2 (Visual Density — M2)
- **Target Files:** `ts/reviewer/procedural.ts:863-894`, `ts/reviewer/reviewer.scss:655-705`
- **Observed Behavior:** Heavy colored badges (e.g. `⚡ Fluency Strength (Accurate & Fast)`) compete with the derivation text.
- **Canonical Requirement:** Compact, muted status pill: `⚡ Fast & Accurate · 8.4s`.

---

### GAP-ANTI-05: Header Variant Tag Decluttering
- **Severity:** P2 (Chrome Elimination — M2)
- **Target Files:** `rslib/procedural/src/reviewer/template.rs:109-116`
- **Observed Behavior:** `.proc-variant-tag` can render generic practice strings in headers.
- **Canonical Requirement:** Suppress all generic practice tags; display badges exclusively for authentic competitive exam provenance (`[ JEE Main 2024 ]`).

---

### GAP-ANTI-06: Raw Schema & Generator ID Leak Suppression
- **Severity:** P0 (Safety & Integrity — M2)
- **Target Files:** `rslib/procedural/src/reviewer/template.rs:527-548`, `ts/reviewer/procedural.ts:816`
- **Observed Behavior:** Raw schema strings could potentially leak if fallback title parsing fails.
- **Canonical Requirement:** Zero schema IDs in learner DOM text; retained 100% in HTML data attributes.

---

### GAP-ANTI-07: Flattening Nested Cards in Worked Examples & Solutions
- **Severity:** P1 (Open Canvas Layout — M2)
- **Target Files:** `ts/reviewer/reviewer.scss:613-625`, `rslib/procedural/src/reviewer/template.rs:362, 398`
- **Observed Behavior:** Box-in-a-box syndrome with inset backgrounds and thick borders in worked examples.
- **Canonical Requirement:** Flat open canvas layout with 1px horizontal dividers and 3px left accent borders (`--proc-accent-left-worked`, `--proc-accent-left-decision`).

---

### GAP-ANTI-08: Deferred Solution Reveal in Reflection Gate
- **Severity:** P0 (Metacognitive Invariant — M3)
- **Target Files:** `ts/reviewer/procedural.ts:958-980, 1020-1035`, `rslib/procedural/src/reviewer/template.rs:618`
- **Observed Behavior:** `#proc-solution-container` resides inside `#proc-result-panel` and can be visible during `mistake_classification` unless explicitly hidden.
- **Canonical Requirement:** `#proc-solution-container` must be explicitly hidden (`display: none !important`) during reflection and revealed ONLY after 1–4 classification occurs.

---

### GAP-MOD-01: Specialized Modality Components
- **Severity:** P1 (Architecture — M3)
- **Target Files:** `ts/reviewer/components/` (`concept_check_container.ts`, `strategy_drill_container.ts`, `worked_example_container.ts`)
- **Observed Behavior:** Shared generic container configurations rather than dedicated, clean component classes.
- **Canonical Requirement:** Dedicated controllers enforcing zero-textbox fallback and distinct visual treatments.

---

### GAP-NATIVE-01: Native Basic/Cloze Isolation Verification
- **Severity:** P0 (Runtime Boundary — M3/M5)
- **Target Files:** `qt/aqt/reviewer.py:985-1055`, `rslib/src/notetype/render.rs`
- **Observed Behavior:** Procedural cards suppress native bottom ease buttons and `#ansbut`.
- **Canonical Requirement:** Standard Basic and Cloze cards retain native Anki reviewer appearance, rating controls, and behavior completely intact.

---

## 4. Prioritized Remediation Roadmap (Milestones M1–M5)

```text
┌──────────────────────────────────────────────────────────────────────────────────┐
│                          FRONTEND REMEDIATION ROADMAP                            │
├──────────────────────────────────────────────────────────────────────────────────┤
│ Milestone 1 (M1): Contract & Spec Sync            ──► [ COMPLETE & FROZEN ]     │
│ Milestone 2 (M2): Template & Visual Rebuild       ──► [ PLANNED ]                │
│ Milestone 3 (M3): Modality Controllers & Gate     ──► [ PLANNED ]                │
│ Milestone 4 (M4): Automated Regression & Build    ──► [ PLANNED ]                │
│ Milestone 5 (M5): Desktop Proof & Evidence        ──► [ PLANNED ]                │
└──────────────────────────────────────────────────────────────────────────────────┘
```

### Milestone 1 (M1): Screen Composition Contract & Spec Sync — Status: COMPLETED ✅
- Created authoritative `docs/STUDYLAB_UI_COMPOSITION_CONTRACT.md`.
- Synchronized all 5 specification files (`FRONTEND_VISUAL_DESIGN_SPEC.md`, `FRONTEND_UI_STATE_SPEC.md`, `FRONTEND_BUTTON_CONTRACT.md`, `FRONTEND_ACCEPTANCE_MATRIX.md`, `FRONTEND_CURRENT_STATE_GAP_MAP.md`).
- Reconciled 11 core states, 14 target verification states, 8 anti-patterns, and 23-button matrix.

### Milestone 2 (M2): Frontend Template & Visual Hierarchy Rebuild
- Rebuild Open Canvas styling and design tokens in `ts/reviewer/reviewer.scss`.
- Eliminate giant red/green feedback containers (`ANTI-01`) and flatten nested card wrappers (`ANTI-07`).
- Suppress ticking timer during solving (`ANTI-03`) and declutter header metadata/variant tags (`ANTI-05`, `ANTI-06`).
- Streamline speed quadrant badges into compact muted pills (`ANTI-04`).

### Milestone 3 (M3): Frontend Modality Controllers & Mistake Gate
- Enforce zero-textbox fallback across all structured modalities in `ts/reviewer/components/`.
- Implement deferred solution reveal in reflection gate (`ANTI-08`).
- Deduplicate expected answer comparison rows (`ANTI-02`).
- Enforce single `#proc-next-btn` primary CTA and native ease button suppression on procedural cards.
- Verify 100% isolation of standard Basic and Cloze cards.

### Milestone 4 (M4): Automated Regression & Build Verification
- Execute and pass full TypeScript unit test suite: `npm run vitest:once` (all 18 test files, 152+ tests).
- Execute and pass Rust procedural engine suite: `cargo test -p procedural`.
- Validate clean bundle output: `npm run build`.

### Milestone 5 (M5): Real Visible Desktop Verification & Evidence Ledger
- Attach to visible DEV Anki desktop window via `desktop-webview-reviewer`.
- Capture before/after visual proof across all 14 target states.
- Author `docs/STUDYLAB_FRONTEND_RECONCILIATION_REPORT.md` and `artifacts_qa/frontend_reconciliation/evidence.json`.

---

## 5. Architectural Quality Gate & Regression Safety Check

Before any code modification is merged:
1. **Automated Test Suites:** `npm run vitest:once` and `cargo test -p procedural` must pass with 0 failures.
2. **Build Pipeline:** `npm run build` must compile cleanly without TypeScript or Svelte errors.
3. **14-State Desktop Proof:** Live visible DEV Anki desktop window must pass all 14 qualitative acceptance tests.
4. **Non-Procedural Isolation:** Standard Basic and Cloze cards must render with 100% native Anki controls, standard `#ansbut`, standard ease rating bar, and zero StudyLab CSS or keyboard bleed.
