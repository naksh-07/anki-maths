# Project: StudyLab Frontend Reconciliation and UI Composition Rebuild

## Architecture

StudyLab operates as an interactive procedural learning engine embedded within Anki Desktop's QtWebEngine review webview:

$$\text{Anki SQLite Store} \longrightarrow \text{Rust Core (rslib/procedural)} \longrightarrow \text{Python Host (qt/aqt/reviewer.py)} \longrightarrow \text{TypeScript UI (ts/reviewer)}$$

### Key Invariants:
1. **Visual Hero Invariant**: The problem statement is the primary visual hero of the interface. The UI is completely subordinate to the learner's cognitive problem-solving task.
2. **Semantic Modality Purity**: Semantic modality must ALWAYS match UI modality. Zero-textbox fallback for MCQ, ConceptCheck, StrategyDrill, and WorkedExample.
3. **Open Canvas Layout**: Open canvas typography, subtle 1px dividers, flat surfaces, zero rainbow badges, zero giant colored feedback boxes, zero nested cards.
4. **Anti-Bypass Reflection Gate**: On incorrect answers, Space/Enter keys are trapped until the learner selects a mistake category (1 Silly, 2 Pattern, 3 Concept, 4 Prereq). Full solution derivation remains hidden until reflection is completed.
5. **Runtime Boundary & Single CTA**: StudyLab owns problem interaction and the post-answer `Next Problem ➔` CTA (`#proc-next-btn`). Card advancement computes calibrated FSRS ease and dispatches `procedural_answer:<ease>`. Standard Basic and Cloze cards remain 100% untouched native Anki reviews.

---

## Feature Inventory

| # | Feature | Description | Milestone | Source |
|---|---|---|---|---|
| 1 | **UI Composition Contract** | Canonical single source of truth defining learner goal, hero, primary/secondary actions, visible/hidden/forbidden content for all 11 states | M1 | `docs/STUDYLAB_UI_COMPOSITION_CONTRACT.md` |
| 2 | **Spec Documentation Sync** | Synchronize `FRONTEND_VISUAL_DESIGN_SPEC.md`, `FRONTEND_UI_STATE_SPEC.md`, `FRONTEND_BUTTON_CONTRACT.md`, `FRONTEND_ACCEPTANCE_MATRIX.md`, `FRONTEND_CURRENT_STATE_GAP_MAP.md` | M1 | `docs/FRONTEND_*.md` |
| 3 | **Open Canvas Styling & Token System** | Implement clean typography, 4px/8px spacing grid, theme tokens (`:root`, `body.nightMode`), subtle borders, zero heavy drop-shadows | M2 | `ts/reviewer/reviewer.scss` |
| 4 | **Eliminate Giant Feedback Containers** | Replace full-bleed red/green background wrappers (`ANTI-01`) with subtle inline status indicators (`✓ Correct`, `✗ Incorrect`) | M2 | `ts/reviewer/reviewer.scss`, `ts/reviewer/procedural.ts` |
| 5 | **Flatten Nested Cards & Solutions** | Remove card-in-a-card syndrome (`ANTI-07`) in worked examples, solution panels, and pitfall boxes using open layout and left accent rules | M2 | `rslib/.../template.rs`, `ts/reviewer/reviewer.scss` |
| 6 | **Header Decluttering & Variant Suppression** | Suppress generic practice/variant badges (`ANTI-05`) and raw internal schema IDs (`ANTI-06`); preserve only verified exam provenance badges | M2 | `rslib/.../template.rs` |
| 7 | **Timer Presentation Optimization** | Suppress ticking stopwatch during active solving (`ANTI-03`); display elapsed time cleanly in post-submission feedback alongside speed pill | M2 | `rslib/.../template.rs`, `ts/reviewer/procedural.ts` |
| 8 | **Streamline Speed Quadrant Badges** | Replace long competing badges (`ANTI-04`) with compact muted status pills (`⚡ Fast & Accurate · 8.4s`) | M2 | `ts/reviewer/procedural.ts`, `ts/reviewer/reviewer.scss` |
| 9 | **Deduplicate Answer Comparison Labels** | Consolidate duplicate "Expected Answer" / "You answered" strings (`ANTI-02`) into a single concise comparison row | M3 | `ts/reviewer/procedural.ts` |
| 10 | **Deferred Solution Reveal in Reflection Gate** | Keep `#proc-solution-container` strictly hidden during `mistake_classification` (`ANTI-08`); reveal solution only after selecting 1–4 | M3 | `ts/reviewer/procedural.ts` |
| 11 | **Modality Interface Enforcement** | Enforce dedicated zero-textbox interaction surfaces for Numerical, MCQ, Concept Check, Strategy Drill, Stepwise, Worked Example | M3 | `ts/reviewer/components/*.ts` |
| 12 | **Mistake Classification Reflection UI** | Metacognitive error classification (1 Silly Slip, 2 Pattern Missed, 3 Concept Gap, 4 Prereq Unknown) with Space/Enter anti-bypass lock | M3 | `ts/reviewer/components/mistake_footer.ts` |
| 13 | **Single Next CTA & Footer Boundary** | StudyLab controls in-card `Next Problem ➔` CTA; dispatches calibrated FSRS ease to Python host; suppresses duplicate bottom ease buttons | M3 | `ts/reviewer/procedural.ts`, `qt/aqt/reviewer.py` |
| 14 | **Non-Procedural Card Isolation** | Verify Basic and Cloze cards bypass StudyLab entirely with standard Anki `#ansbut` and ease ratings | M3 | `qt/aqt/reviewer.py`, `rslib/.../render.rs` |
| 15 | **TypeScript Unit Test Suite** | Execute and verify 100% pass rate on `npm run vitest:once` (all 18 test files, 152+ unit tests) | M4 | `ts/reviewer/__tests__/*.test.ts` |
| 16 | **Rust Procedural Test Suite** | Execute and verify 100% pass rate on `cargo test -p procedural` | M4 | `rslib/procedural/` |
| 17 | **Clean Web Build Pipeline** | Execute and verify `npm run build` generates error-free bundles in `out/sveltekit` | M4 | `package.json`, Vite build |
| 18 | **Desktop DEV Webview Verification (14 States)** | Connect to running DEV Anki instance via `desktop-webview-reviewer` and capture before/after visual proof for all 14 target states | M5 | Real DEV Anki desktop window |
| 19 | **Evidence Ledger & QA Report** | Author `docs/STUDYLAB_FRONTEND_RECONCILIATION_REPORT.md` and structured `artifacts_qa/frontend_reconciliation/evidence.json` | M5 | `artifacts_qa/frontend_reconciliation/` |

---

## Milestones

| # | Name | Scope | Dependencies | Status |
|---|---|---|---|---|
| **M1** | **Screen Composition Contract & Spec Sync** | Create `STUDYLAB_UI_COMPOSITION_CONTRACT.md` and update `FRONTEND_*.md` specs | None | DONE |
| **M2** | **Frontend Template & Visual Hierarchy Rebuild** | Open canvas styling, CSS tokens, anti-pattern removal, flattening cards, decluttering header & timers | M1 | DONE |
| **M3** | **Frontend Modality Controllers & Mistake Gate** | Modality enforcement, answer deduplication, deferred solution reveal, anti-bypass mistake reflection, Next CTA | M1, M2 | DONE |
| **M4** | **Automated Regression & Build Verification** | `npm run vitest:once`, `cargo test -p procedural`, `npm run build` | M2, M3 | DONE |
| **M5** | **Real Desktop Verification & Evidence Ledger** | Live desktop DEV Anki inspection across 14 states, `evidence.json`, final audit report | M4 | DONE |

---

## Interface Contracts

### 11 Core UI States & Semantic Hierarchy
1. `loading`: Blank card frame during initial DOM setup & MathJax compilation.
2. `ready`: Clean problem hero displayed; primary input/option surface focused; hotkeys armed.
3. `solving`: Active problem solving; live preview pill for numeric input; radio option selection.
4. `hint`: Inline progressive hint disclosure (Principle $\to$ Operation $\to$ Intermediate Setup).
5. `submitting`: Input evaluation and local AST/unit validation.
6. `wrong_answer` / `mistake_classification`: Metacognitive pause; mistake options (1–4) visible; Space/Enter trapped; solution hidden.
7. `feedback`: Concise outcome statement (`✓ Correct` / `✗ Incorrect`); step-by-step LaTeX derivation; streamlined speed pill; single `Next Problem ➔` CTA.
8. `next`: Calibrated FSRS ease bridge call (`procedural_answer:<ease>`); card transition.
9. `stepwise`: Multi-step algebraic derivation workspace with individual step validation.
10. `concept_check`: Problem context with conceptual options and distractor misconception feedback.
11. `strategy_drill` / `worked_example`: Context with strategy comparison options / canonical derivation trace + `Try Similar` CTA.

### 4 Mistake Categories (Metacognitive Reflection)
- `1 Silly Slip` (`silly_mistake`): Calculation/arithmetic typo or unit mistake.
- `2 Pattern Missed` (`pattern_not_recognized`): Failed to spot problem structure/symmetry.
- `3 Concept Gap` (`formula_or_concept_misapplied`): Wrong formula or conceptual confusion.
- `4 Prereq Unknown` (`concept_not_known`): Missing fundamental prerequisite knowledge.

### Master IPC Bridge Endpoints (`qt/aqt/reviewer.py` $\leftrightarrow$ `ts/reviewer/procedural.ts`)
- `procedural_answer:<ease>`: Advances card with calibrated FSRS ease rating (1=Again, 2=Hard, 3=Good, 4=Easy).
- `procedural_attempt:<json>`: Records attempt telemetry (accuracy, timeTakenMs, targetTimeMs, speedQuadrant).
- `procedural_mistake:<json>`: Records mistake category selection and remediation triggers.
- `procedural_hint:<json>`: Increments hint tier request counter.
- `procedural_validate_steps:<json>`: Dispatches multi-step derivation validation.
- `procedural_try_similar:<json>`: Requests dynamic parameter regeneration for worked examples.

---

## Code Layout

- **Specifications & Documentation**: `docs/STUDYLAB_UI_COMPOSITION_CONTRACT.md`, `docs/FRONTEND_*.md`
- **TypeScript Reviewer Subsystem**: `ts/reviewer/procedural.ts`, `ts/reviewer/reviewer.scss`, `ts/reviewer/components/`
- **Rust Reviewer Template**: `rslib/procedural/src/reviewer/template.rs`
- **Python Desktop Host**: `qt/aqt/reviewer.py`
- **Test Suites**: `ts/reviewer/__tests__/`, `rslib/procedural/tests/`
- **Forensic QA Artifacts**: `docs/STUDYLAB_FRONTEND_RECONCILIATION_REPORT.md`, `artifacts_qa/frontend_reconciliation/evidence.json`
