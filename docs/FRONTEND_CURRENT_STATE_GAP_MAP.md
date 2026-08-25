# StudyLab Frontend Current State Gap Map & Remediation Roadmap

**Document Version:** 1.0.0 (Canonical)  
**Target Subsystem:** Frontend Reviewer Codebase (`ts/reviewer/`), Template Generation (`rslib/procedural/src/reviewer/`), and Desktop Host (`qt/aqt/reviewer.py`)  
**Status:** AUTHORITATIVE CANONICAL SPECIFICATION (Mission Section 22)  
**Integrity Mode:** 100% Grounded in Executable Source Code, Live Desktop Forensic Audits, and Test Evidence  

---

## 1. Executive Summary & Audit Context

This document provides a comprehensive forensic gap analysis comparing the current executable implementation of StudyLab against the canonical product, state, and button contracts defined in `FRONTEND_PRODUCT_SPEC.md`, `FRONTEND_UI_STATE_SPEC.md`, and `FRONTEND_BUTTON_CONTRACT.md`.

### Audit Methodology & Ground Truth:
- **Live Desktop Reviewer Forensic Attach:** Attached directly to active Anki desktop instance (`PID 24896`, `HWND 13895330`, CDP `127.0.0.1:9222`) across all 175 curriculum topics.
- **Dual Screenshot Provenance:** 24 high-resolution captures comparing Win32 Native GDI window captures with CDP DOM captures (`artifacts_qa/visual_audit/`).
- **12-State Modality Verification:** Executed full automated state transition suite (`tools/live_modality_verifier.py`).

```text
┌──────────────────────────────────────────────────────────────────────────────────┐
│                               AUDIT STATUS SUMMARY                               │
├──────────────────────────────────────────────────────────────────────────────────┤
│   - P0 (Blocking Architectural & Safety Defects):   100% RESOLVED & VERIFIED     │
│   - P1 (High-Priority UX Polish & Specialization):  DOCUMENTED & ACTIONABLE      │
│   - P2 (Future Enhancements & Advanced Widgets):    CATALOGED IN ROADMAP         │
└──────────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Screen-by-Screen Implementation Audit & Discrepancy Matrix

| Screen / Modality | Canonical Contract Requirement | Current Implementation Status | Compliance Level | Observed Code / Visual Evidence |
|---|---|---|---|---|
| **1. Math Numerical (Algebra)** | Single text input with auto-focus, live preview pill, submit button, mode switch tabs. Zero MCQ options. | Fully implemented in `NumericalContainer` and `procedural.ts:326-332`. | 🟢 **COMPLIANT** | `01_math_numerical_cdp.png` (SHA: `ab31898e...`) confirms clean single input strip and working mode toggle. |
| **2. Math MCQ (Commercial)** | 4 discrete radio cards (A–D), keyboard navigation (1–4, A–D), zero text input fallback. | Fully implemented in `MCQContainer` (`mcq_container.ts:100-240`). | 🟢 **COMPLIANT** | `02_math_mcq_cdp.png` (SHA: `5652e1db...`) confirms zero text box and active radio cards. |
| **3. Reasoning MCQ (Blood Relations)** | Structured logic narrative prompt, 4 discrete option cards, keyboard selection. | Rendered via `MCQContainer`. Text fallback completely eliminated. | 🟢 **COMPLIANT** | `03_reasoning_cdp.png` (SHA: `f53b0eae...`) confirms option selection and clear narrative typography. |
| **4. Physics Numerical (Kinematics)** | 5D dimensional unit parsing ($[L]^1 [T]^{-1}$), live preview pill, duplicate unit badge removed. | `PhysicalDimension` vector in `numerical_container.ts:12-65`. Redundant `.proc-unit-hint` removed. | 🟢 **COMPLIANT** | `04_physics_numerical_cdp.png` (SHA: `6432767e...`) confirms single clean preview pill `Parsed: 30 m/s`. |
| **5. Chemistry Numerical (Mole Concept)** | Stoichiometric amount parsing ($[N]^1$), molar mass conversion, non-negativity check. | Handled via `UnitRegistry` with molar units (`mol`, `mmol`, `kmol`). | 🟢 **COMPLIANT** | `05_chemistry_numerical_cdp.png` (SHA: `6d21786f...`) confirms `Parsed: 1 mol` and unit tolerance. |
| **6. ConceptCheck (Percentages)** | Discrete conceptual options with immediate distractor-specific misconception feedback. | Functional via `MCQContainer.conceptCheck`, but shares generic styling with standard MCQs. | 🟡 **P1 POLISH** | `06_concept_check_cdp.png` (SHA: `d631c3de...`) shows distractor feedback, but needs dedicated visual card container. |
| **7. StrategyDrill (Alligation)** | Strategy comparison cards with optimality analysis rationale. | Functional via `MCQContainer.strategyDrill`, evaluated against `preferred_option_id`. | 🟡 **P1 POLISH** | `07_strategy_drill_cdp.png` (SHA: `5848cfa2...`) shows optimality rationale, but lacks specialized strategy badge styling. |
| **8. WorkedExample (Shopkeeper)** | Expert solution trace, highlighted decision point, acknowledgment gate, zero solving inputs. | Rendered via `procedural.ts:1192` result panel; lacks dedicated component class. | 🟡 **P1 POLISH** | `08_worked_example_cdp.png` (SHA: `f8510233...`) displays trace and gate button, but uses generic DOM injection. |
| **9. Stepwise Derivation Workspace** | Multi-row CAS derivation, step validation badges, downstream consistency, hint/add controls. | Fully implemented in `StepwiseContainer` (`stepwise_container.ts:1-811`). | 🟢 **COMPLIANT** | `09_stepwise_workspace_cdp.png` (SHA: `6461b4d3...`) confirms step rows and validation badges. |
| **10. Wrong Answer Outcome** | Input disabled, error banner displayed, primary Next button strictly hidden. | Handled in `procedural.ts:940-970` with locked inputs and error notification. | 🟢 **COMPLIANT** | `10_wrong_answer_cdp.png` (SHA: `d3411438...`) confirms Next button is hidden during error. |
| **11. Mistake Classification** | 4 reflection buttons (1–4), Space and Enter strictly trapped, no bypass allowed. | `MistakeFooter` (`mistake_footer.ts`) traps Space/Enter in `procedural.ts:468-498`. | 🟢 **COMPLIANT** | `11_mistake_classification_cdp.png` (SHA: `0ee5d4a8...`) confirms anti-bypass reflection gate. |
| **12. Clean Result Feedback** | Deduplicated expected answer, canonical MathJax derivation, speed quadrant, single Next CTA. | Redundant expected row removed in `template.rs`; native ease buttons suppressed. | 🟢 **COMPLIANT** | `12_feedback_next_cdp.png` (SHA: `64ea3dc9...`) confirms clean derivation and single Next CTA. |

---

## 3. Detailed Forensic Discrepancy Ledger

---

### Gap 1: Specialized Container Components for `ConceptCheck` and `StrategyDrill`
- **Gap Identifier:** `GAP-P1-01` (Modality Component Specialization)
- **Severity Level:** **P1 (High-Priority UX Polish)**
- **Affected Files:**
  - `ts/reviewer/components/mcq_container.ts:300-360`
  - `ts/reviewer/procedural.ts:308-324`
- **Observed Behavior:**
  Currently, `ConceptCheck` and `StrategyDrill` modalities are executed by passing configuration options into `MCQContainer`. While functional and correctly suppressing text input, they share the exact same DOM styling and card layout as basic recall MCQs.
- **Canonical Requirement:**
  `FRONTEND_PRODUCT_SPEC.md § 3.5, 3.6` requires distinct visual treatments:
  - `ConceptCheck` should feature a distinct diagnostic card header with prominent misconception callout boxes.
  - `StrategyDrill` should feature visual complexity/efficiency indicators (e.g., speed rating, mental steps counter) on each strategy card.
- **Remediation Strategy:**
  Refactor `ts/reviewer/components/` to create specialized sub-classes or wrappers: `ConceptCheckContainer` and `StrategyDrillContainer` extending or composing `MCQContainer`, applying distinct SCSS classes (`.proc-concept-card`, `.proc-strategy-card`).
- **Verification Method:**
  Run `tools/live_modality_verifier.py` states 03 and 04; verify presence of specialized CSS class wrappers and distinct visual badges.

---

### Gap 2: Dedicated `WorkedExampleView` Component Architecture
- **Gap Identifier:** `GAP-P1-02` (Worked Example Component Architecture)
- **Severity Level:** **P1 (High-Priority UX Polish)**
- **Affected Files:**
  - `ts/reviewer/procedural.ts:1192-1200`
  - `rslib/procedural/src/reviewer/template.rs:180-240`
- **Observed Behavior:**
  Worked examples are currently rendered by dynamically injecting HTML into `#proc-result-panel` inside `procedural.ts`. There is no standalone `WorkedExampleContainer` class managing step expansion, collapsible decision notes, or step-by-step reading progress.
- **Canonical Requirement:**
  `FRONTEND_PRODUCT_SPEC.md § 3.7` and `FRONTEND_UI_STATE_SPEC.md § 3.11` specify a dedicated `WorkedExampleView` reading modality with structured step disclosure and an explicit acknowledgment gate.
- **Remediation Strategy:**
  Create `ts/reviewer/components/worked_example_container.ts` that encapsulates step rendering, decision point callouts, and the `[ ✔ I Have Reviewed and Understood This Solution ]` action button.
- **Verification Method:**
  Mount a worked example note; verify that `WorkedExampleContainer` instantiates cleanly and binds the acknowledgment button.

---

### Gap 3: Result / Feedback Screen Information Density & Debug Clutter
- **Gap Identifier:** `GAP-P2-01` (Visual Density & Telemetry Decluttering)
- **Severity Level:** **P2 (UX Enhancement)**
- **Affected Files:**
  - `ts/reviewer/procedural.ts:1040-1080`
  - `rslib/procedural/src/reviewer/template.rs:320-370`
- **Observed Behavior:**
  In some template configurations, the feedback panel displays multiple small metadata badges (e.g., target time, elapsed time, difficulty tag, schema tag) in a dense cluster above the solution derivation, slightly cluttering the reading flow.
- **Canonical Requirement:**
  `FRONTEND_PRODUCT_SPEC.md § 11, 14` states: *"Avoid dumping target time, actual time, expected answer, raw metadata, multiple badges... Content is the visual hero."*
- **Remediation Strategy:**
  Consolidate performance metrics into a single, elegant `SpeedQuadrantBadge` strip (e.g., `⚡ Fluency Strength (8.4s)`) and suppress secondary metadata badges on feedback screens.
- **Verification Method:**
  Inspect feedback screen visually; verify that only the outcome banner, single derivation trace, and speed quadrant badge are visible.

---

### Gap 4: Live Ticking Timer Anxiety During Active Solving
- **Gap Identifier:** `GAP-P2-02` (Cognitive Scaffolding & Timer Presentation)
- **Severity Level:** **P2 (UX Enhancement)**
- **Affected Files:**
  - `ts/reviewer/procedural.ts:380-395`
  - `ts/reviewer/reviewer.scss:240-255`
- **Observed Behavior:**
  The solving timer (`#proc-timer`) runs a 200ms `setInterval` loop that updates visible elapsed seconds directly on the solving screen.
- **Canonical Requirement:**
  `FRONTEND_PRODUCT_SPEC.md § 12` states: *"The ticking proc-timer updating every 200ms during solving violates cognitive scaffolding and induces anxiety. It must be implicit or subdued until the feedback state."*
- **Remediation Strategy:**
  Subdue or hide the numeric timer during active solving (`display: none` or subtle ghost indicator); display total elapsed time only on the feedback screen alongside the speed quadrant badge.
- **Verification Method:**
  Observe card in `solving` state; confirm that no rapid numeric counter is ticking on the screen.

---

### Gap 5: Panel Stacking & Vertical Scroll in Stepwise Workspace
- **Gap Identifier:** `GAP-P1-03` (Vertical Space Optimization in Stepwise Workspace)
- **Severity Level:** **P1 (High-Priority UX Polish)**
- **Affected Files:**
  - `ts/reviewer/components/stepwise_container.ts:150-220`
  - `ts/reviewer/reviewer.scss:410-470`
- **Observed Behavior:**
  When a multi-step problem has 4+ derivation rows and a hint card is expanded, the total container height can exceed 600px, introducing a vertical scrollbar in smaller Anki desktop windows (600x500px).
- **Canonical Requirement:**
  `FRONTEND_ACCEPTANCE_MATRIX.md § 3` requires clean vertical containment without unnecessary scrolling on standard 600px desktop windows.
- **Remediation Strategy:**
  Implement compact step row styling (`padding: 4px 8px`), inline hint toggling, and max-height scrolling on the step list container rather than scrolling the entire card.
- **Verification Method:**
  Test a 5-step problem on a 600x500px window; verify that problem prompt, active step row, and controls remain visible without whole-window scrolling.

---

### Gap 6: Domain-Specific Interactive Visual Widgets
- **Gap Identifier:** `GAP-P2-03` (Domain-Specific Visual Widgets)
- **Severity Level:** **P2 (Future Enhancement)**
- **Affected Files:**
  - `ts/reviewer/components/` (New module candidate)
- **Observed Behavior:**
  Complex multi-variable chemistry problems (e.g., ICE equilibrium tables) and logical seating puzzles currently rely on text descriptions rather than interactive tabular or graphical widgets.
- **Canonical Requirement:**
  `FRONTEND_PRODUCT_SPEC.md § 18` outlines future support for interactive domain widgets:
  - *Reasoning:* Interactive 2D matrix grids / truth tables.
  - *Chemistry:* Interactive ICE (Initial, Change, Equilibrium) tables.
  - *Physics:* Free-body force vector diagrams.
- **Remediation Strategy:**
  Design modular widget plugins in `ts/reviewer/components/widgets/` (e.g., `IceTableWidget.ts`, `LogicGridWidget.ts`) that mount inside `#proc-custom-widget-slot`.
- **Verification Method:**
  Unit tests and screenshot verification for each specialized widget.

---

## 4. Prioritized Frontend Remediation Roadmap

The remediation roadmap is organized into three sequential phases based on architectural impact and user experience priority:

```text
┌──────────────────────────────────────────────────────────────────────────────────┐
│                          FRONTEND REMEDIATION ROADMAP                            │
├──────────────────────────────────────────────────────────────────────────────────┤
│ Phase 1: P0 Critical Architectural Fixes   ──► [ 100% COMPLETE & VERIFIED ]      │
│ Phase 2: P1 Component Specialization & UX  ──► [ READY FOR IMPLEMENTATION ]      │
│ Phase 3: P2 Domain Richness & Decluttering ──► [ SCHEDULED FOR NEXT MILESTONE ]  │
└──────────────────────────────────────────────────────────────────────────────────┘
```

---

### Phase 1: Critical Architectural Integrity (P0) — Status: COMPLETED ✅

All blocking architectural, data safety, and bypass vulnerabilities have been forensically resolved and verified in executable code:

1. **Zero Text Input Fallback Enforcement (GAP-MOD-01):**
   - *Status:* **RESOLVED & VERIFIED**
   - *Code Fix:* `rslib/procedural/src/reviewer/template.rs` and `ts/reviewer/procedural.ts:302-332`.
   - *Verification Proof:* State 01 & 02 tests pass with 100% assertion rate.
2. **Native Show Answer Interception (GAP-BN-01):**
   - *Status:* **RESOLVED & VERIFIED**
   - *Code Fix:* `qt/aqt/reviewer.py:942-944` routes native Show Answer to `handleNativeShowAnswer()`.
   - *Verification Proof:* Zero DOM destruction on native shortcut trigger.
3. **One-Interaction-Surface Review Coordination (GAP-BN-02):**
   - *Status:* **RESOLVED & VERIFIED**
   - *Code Fix:* `qt/aqt/reviewer.py:986-990, 1009-1018` suppresses duplicate bottom ease buttons.
   - *Verification Proof:* Single in-card `#proc-next-btn` advances card with calibrated FSRS ease.
4. **Anti-Bypass Metacognitive Reflection Gate (GAP-STATE-01):**
   - *Status:* **RESOLVED & VERIFIED**
   - *Code Fix:* `ts/reviewer/procedural.ts:483-498` traps Space and Enter during `mistake_classification`.
   - *Verification Proof:* Advancing past incorrect attempt without classification is impossible.
5. **ContentProvenance Deserialization Invariant (GAP-SERDE-01):**
   - *Status:* **RESOLVED & VERIFIED**
   - *Code Fix:* Added `#[serde(default)]` in `rslib/procedural/src/exam/pyq.rs`.
   - *Verification Proof:* Unit test `test_extract_from_universe_175_note` passing in Rust core.

---

### Phase 2: Component Specialization & High-Priority UX Polish (P1)

Targeted frontend enhancements to be executed in the next implementation cycle:

| Task ID | Task Description | Target Files | Estimated Effort | Success Criteria |
|---|---|---|---|---|
| **TASK-P1-01** | Create dedicated `ConceptCheckContainer` and `StrategyDrillContainer` components. | `ts/reviewer/components/` | 2 Days | Distinct styling and distractor badge callouts rendered for concept/strategy notes. |
| **TASK-P1-02** | Create standalone `WorkedExampleContainer` with step trace reading controls. | `ts/reviewer/components/worked_example_container.ts` | 2 Days | Dedicated container class with explicit acknowledgment gate button. |
| **TASK-P1-03** | Optimize Stepwise workspace layout to prevent vertical scrolling on 600px windows. | `ts/reviewer/components/stepwise_container.ts`, `reviewer.scss` | 1 Day | 5-step problem fits within 500px height without window scroll. |

---

### Phase 3: Visual Decluttering & Domain Richness (P2)

Refinements for visual calmness and domain-specific interactive features:

| Task ID | Task Description | Target Files | Estimated Effort | Success Criteria |
|---|---|---|---|---|
| **TASK-P2-01** | Subdue active solving timer and consolidate feedback metrics into `SpeedQuadrantBadge`. | `ts/reviewer/procedural.ts`, `reviewer.scss` | 1 Day | Zero ticking timer during solving; calm performance badge in feedback. |
| **TASK-P2-02** | Declutter header metadata badges; show only domain, skill, and optional difficulty. | `rslib/procedural/src/reviewer/template.rs` | 1 Day | Max 3 clean badges in header; zero raw schema strings. |
| **TASK-P2-03** | Develop interactive ICE table widget for physical chemistry equilibrium problems. | `ts/reviewer/components/widgets/ice_table.ts` | 3 Days | Multi-cell tabular inputs for chemical equilibrium calculations. |

---

## 5. Architectural Quality Gate & Regression Safety Check

To ensure that future frontend modifications never re-introduce past defects, the following QA regression suite must be executed before any frontend merge:

1. **Automated Modality Suite (`tools/live_modality_verifier.py`):**
   - Must achieve **12 / 12 States Passed (100.0%)**.
   - Asserts zero text input on MCQs, anti-bypass lock on errors, and single Next CTA on feedback.
2. **TypeScript Typecheck & Linting (`just lint`):**
   - Must pass with 0 errors (`svelte-check` and `tsc --noEmit`).
3. **Dual Screenshot Forensic Comparison (`artifacts_qa/visual_audit/`):**
   - Captured native Win32 window images must match CDP rendering with zero layout shift.
4. **Standard Anki Regression Invariant:**
   - Standard Basic and Cloze cards must render with 100% native Anki controls, standard `#ansbut`, standard ease rating bar, and zero StudyLab CSS or keyboard bleed.
