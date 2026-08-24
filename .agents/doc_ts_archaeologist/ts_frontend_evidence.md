# TypeScript Reviewer Frontend: Comprehensive Evidence & Source-Truth Report

**Auditor:** TypeScript Reviewer Archaeologist  
**Date:** 2026-08-25  
**Working Directory:** `ts/reviewer/`, `ts/reviewer/components/`, `ts/reviewer/diagnostic/`, `ts/tests/`  
**Test Status:** 18 test files passing (150 tests total; 94 reviewer unit tests + 2 Playwright E2E suites)

---

## 1. Executive Summary & Component Architecture

The StudyLab TypeScript frontend reviewer is a modular, high-fidelity procedural learning engine embedded within Anki's webview surface. It adheres strictly to the core architectural invariant: **"StudyLab is not a flashcard system; it is a procedural problem-solving engine hosted inside Anki."**

The frontend provides rich interactive problem-solving environments for Mathematics, Physics, Chemistry, and Logical Reasoning, handling multiple answer modalities, progressive scaffolding, error taxonomy classification, and native Anki scheduler synchronization.

### Source Files & Architecture Map

| File Path | Primary Class / Object | Responsibilities & Public Interface |
|---|---|---|
| `ts/reviewer/procedural.ts` | `ProceduralReviewer`, `proceduralAPI` | Main orchestrator managing lifecycle, stopwatch timer, state machine transitions, speed quadrant computation, telemetry packaging (`mutateNextCardStates`), and bridge command dispatch. |
| `ts/reviewer/components/mcq_container.ts` | `MCQContainer` | Multiple Choice Question modality: zero text input enforcement, ARIA radiogroup accessibility, keyboard shortcuts (`1-4`, `A-D`), canonical identity matching, instant practice vs mock exam mode. |
| `ts/reviewer/components/numerical_container.ts` | `NumericalContainer`, `NumericalParser`, `UnitRegistry`, `PhysicalDimension` | Numerical modality: 5D dimensional vector analysis, 50+ unit conversions (Physics & Chemistry), scientific notation (`1.2e-3`), fractions (`3/4`), live preview pill, tolerance checking, unit error diagnostics. |
| `ts/reviewer/components/stepwise_container.ts` | `StepwiseContainer` | Multi-step procedural reasoning: dynamic step rows, algebraic & linear root equivalence, downstream consistency tracking (`PartiallyValid`), taxonomic error diagnosis, 3-tier progressive hints. |
| `ts/reviewer/components/mistake_footer.ts` | `MistakeFooter` | Compact mistake classification strip (`[1 Silly]`, `[2 Pattern]`, `[3 Concept]`, `[4 Unknown]`), reflection protection (trapping Space/Enter), telemetry dispatch. |
| `ts/reviewer/answering.ts` | `mutateNextCardStates`, `applyStateTransform` | Anki customData bridge: protobuf/JSON state transformations, merging StudyLab telemetry into `again`, `hard`, `good`, and `easy` card states. |
| `ts/reviewer/diagnostic/diagnostic_session.ts` | `DiagnosticSessionController` | Diagnostic mock assessment session controller: palette grid, timer countdown, question navigation, mark-for-review, answer persistence, test submission. |
| `ts/reviewer/diagnostic/diagnostic_report.ts` | `DiagnosticReportController` | 4-tier diagnostic report controller: subject/chapter/topic/family hierarchy drill-down, 4-dimension error distribution (Concept, Calculation, Transfer, Speed), weak skills chips, remediation trigger. |
| `ts/reviewer/index.ts` & `reviewer_extras.ts` | Global Namespace Export | Attaches `globalThis.anki.procedural = proceduralAPI` and `globalThis.anki.mutateNextCardStates`. |

---

## 2. State Machine & Lifecycle Hooks

### 2.1 State Definitions (`ProceduralUIState`)

The state machine in `ts/reviewer/procedural.ts` defines 11 explicit states:
1. `loading`: Initial constructor state during DOM element binding.
2. `ready`: DOM elements bound, child components initialized, event listeners registered.
3. `solving`: Stopwatch running, interactive inputs active, keyboard shortcuts routed to modality components.
4. `hint`: Temporary state during hint request; renders progressive hint box and dispatches `procedural_hint` bridge command.
5. `submitting`: Answer submitted or MCQ clicked; local evaluation executing.
6. `mistake_classification`: Activated upon incorrect submission; hides inputs, renders `MistakeFooter` (`1-4`), traps Space and Enter to prevent accidental reflection bypass.
7. `feedback`: Displays correctness banner, speed quadrant badge, time elapsed, and canonical answer; pushes telemetry to Anki `customData`; dispatches `procedural_attempt` and `ans` to reveal ease buttons.
8. `worked_example`: Activated when clicking "Try Similar Problem" in Worked Example modality; dispatches `procedural_try_similar`.
9. `next`: Triggered by clicking "Next" or pressing Enter/Space in feedback state; dispatches `procedural_answer:<ease>`.
10. `error`: Error state for malformed parameters or container faults.
11. `teardown`: Clears timer intervals, unbinds window and DOM listeners, disconnects MutationObserver, resets global active references.

### 2.2 Speed Quadrant & Telemetry Metrics

On submission completion, `computeSpeedQuadrant(isCorrect, timeTakenMs, targetTimeMs)` categorizes performance into four pedagogical quadrants:
- **`fluency_strength`** (Accurate & Fast: `isCorrect && time <= targetTime`): Label `⚡ Fluency Strength (Accurate & Fast)`, Class `.proc-speed-fast-correct`.
- **`speed_opportunity`** (Accurate but Slow: `isCorrect && time > targetTime`): Label `⏱ Speed Opportunity (Accurate but Slow)`, Class `.proc-speed-slow-correct`.
- **`strategy_trap`** (Fast but Incorrect: `!isCorrect && time <= targetTime`): Label `⚠️ Check Strategy / Trap (Fast but Incorrect)`, Class `.proc-speed-fast-wrong`.
- **`concept_setup`** (Slow & Incorrect: `!isCorrect && time > targetTime`): Label `💡 Review Concept / Setup (Slow & Incorrect)`, Class `.proc-speed-slow-wrong`.

### 2.3 StudyLab Telemetry Contract

Telemetry is merged into Anki's next card states via `globalThis.anki.mutateNextCardStates`:
```json
{
  "v": 1,
  "actualTimeMs": 24500,
  "targetTimeMs": 30000,
  "isCorrect": false,
  "hintsUsed": 0,
  "mistakeType": "pattern_not_recognized",
  "mode": "quick",
  "proceduralPerformance": {
    "classification": "incorrect",
    "timeRatio": 0.82,
    "mistakeType": "pattern_not_recognized",
    "hintsUsed": 0
  },
  "proceduralRemediation": {
    "needed": true,
    "reason": "pattern_not_recognized",
    "skillId": "math.percentage.successive",
    "schemaId": "successive_percentage",
    "familyId": "family.math.percentage.successive",
    "topicId": "Percentages"
  },
  "attemptResult": {
    "instanceId": "inst_100",
    "answer": "30%",
    "mode": "quick",
    "steps": [],
    "hintsUsed": 0,
    "timeTakenMs": 24500,
    "isCorrect": false,
    "score": 0.0,
    "speedQuadrant": "strategy_trap"
  }
}
```

---

## 3. Answer Modalities & Component Ground Truth

### 3.1 Multiple Choice Question (`MCQContainer`)
- **Zero Free-Text Input Enforcement:** `enforceZeroTextInputFallback()` explicitly hides `#proc-quick-container`, `#proc-stepwise-container`, `.proc-mode-switch` and sets `#proc-answer-input` to `disabled = true` with `aria-hidden = true`.
- **ARIA Radiogroup Accessibility:** The group receives `role="radiogroup"`; each option receives `role="radio"`, `aria-checked="true|false"`, and roving `tabindex="0|-1"`.
- **Keyboard Shortcuts:**
  - `1`, `2`, `3`, `4`, ... (numeric selection mapped 1-to-1 to option index).
  - `A`, `B`, `C`, `D` / `a`, `b`, `c`, `d` (alphabetic selection, case-insensitive).
  - `ArrowDown` / `ArrowRight` (navigate forward with wraparound), `ArrowUp` / `ArrowLeft` (navigate backward with wraparound).
  - `Enter` / `Space` (confirm selection on focused option item).
- **Canonical Evaluation:** Matches against `canonical_id`, `correct_option_id`, `expected_option_id`, `correct_option`, `formatted`, `answer`, or `value`, tolerating semantic ID, letter, 0-based index, 1-based index, or formatted label.
- **ConceptCheck & StrategyDrill Integration:**
  - `ConceptCheckData`: Evaluates `chosen.is_correct` against `cc.expected_option_id`, revealing misconception feedback.
  - `StrategyDrillData`: Evaluates `chosen.is_optimal` against `sd.preferred_option_id`, revealing strategy optimality feedback.
- **Mock Exam Mode (GAP-MOD-03):** When `mode === "mock"`, selecting an option highlights it with `.selected` but prevents spoiler styling (`.correct`, `.incorrect`, `.disabled` are suppressed). Dispatches `procedural_mock_selection` and provides on-demand evaluation via `mcq.evaluate()`.

### 3.2 Numerical Modality (`NumericalContainer`, `NumericalParser`, `UnitRegistry`, `PhysicalDimension`)
- **5-Dimensional Physical Vector:** `PhysicalDimension` tracks `[Mass]^m * [Length]^l * [Time]^t * [AmountOfSubstance]^n * [Temperature]^k`. Supports full dimensional algebra (`multiply`, `divide`, `pow`, `isCompatibleWith`).
- **Comprehensive Unit Registry:**
  - Over 50 registered physical and chemical units.
  - Linear conversion with multiplier `toSiMultiplier` and temperature offset `offsetToSi` (Celsius $\leftrightarrow$ Kelvin: $T_K = T_C + 273.15$).
  - Physics conversions: `72 km/h` $\leftrightarrow$ `20 m/s` (multiplier 5/18), `1.03 g/cm³` $\leftrightarrow$ `1030 kg/m³`, `1 atm` $\leftrightarrow$ `101.325 kPa`, `1 eV` $\leftrightarrow$ `1.602176634e-19 J`.
  - Chemistry conversions: `1.2 mM` $\leftrightarrow$ `0.0012 M`, `18.015 g/mol` $\leftrightarrow$ `0.018015 kg/mol`, `50.5 kJ/mol`.
- **Parsing Flexibility (`NumericalParser`):**
  - Equation prefix stripping (`v = 15.5 m/s`, `[H+] = 1.0e-7 M`, `ans = 100`).
  - Currency & comma cleaning (`$1,250.50` $\to$ `1250.5`).
  - Percent parsing (`75%` $\to$ `75`, unit `PERCENT`).
  - Unicode superscript math normalization (`⁰`..`⁹`, `⁻`, `⁺`, `·`, `•`, `×`, `²`, `³`, `Å`).
  - Scientific notation: `1.2e-3 mol/L`, `6.022 x 10^23`, `3x10^4 J`, `1.2 × 10⁻³ M`.
  - Fractions: `3/4`, `3/4 m/s`, `-1/2 kg`.
- **Tolerance Engine:** Absolute (`tol.absolute`), relative (`tol.relative * expected`), or combined (`Math.max(absTol, Math.abs(expected) * relTol)`). Default: 0.5% relative tolerance.
- **Diagnostic Sanity Checks:**
  - Non-negative physical constraint check for mass, distance, and moles.
  - Missing unit requirement check (`requireUnit: true`).
  - Incompatible dimension rejection (e.g., submitting `kg` when `m/s` is expected).
  - Common mistake heuristics: reminds students of missing $5/18$ conversion for `km/h` $\to$ `m/s`, or $\div 1000$ for `g` $\to$ `kg` and `mM` $\to$ `M`.
- **Live Typing Preview Pill:** Renders `.proc-num-preview-pill` underneath the input in real time showing parsed value and unit.

### 3.3 Stepwise Multi-Step Reasoning (`StepwiseContainer`)
- **Solution Graph Population:** Pre-populates rows with step descriptions and placeholders from `options.solutionGraph.steps`.
- **Dynamic Step Manipulation:** `addStepRow()` appends intermediate step rows; `resetSteps()` restores initial graph state.
- **Semantic Algebraic Equivalence:**
  - Linear equation solving: `extractLinearRoot("2x = 10", "")` solves for root $x = 5$, enabling equivalence between `2x = 10` and `x = 5`.
  - Commutative addition matching: `a + b` $\equiv$ `b + a`.
  - Multiplier vs percentage equivalence: `0.2` $\equiv$ `20%`.
  - Relational reasoning slot matching: `Slot 3 = Charlie` $\equiv$ `Charlie`.
- **Downstream Consistency Tracking (`PartiallyValid`):**
  - If a student makes an error in step $k$ (e.g. calculates $2x = 12$ instead of $10$, root $6$), and step $k+1$ correctly follows from root $6$ ($x = 6$), step $k+1$ is marked `"partially_valid"` with `isDownstreamConsistent = true` ("Derived consistently from previous error").
- **Taxonomic Error Diagnosis:**
  - Localizes `firstErrorStep` and classifies `firstErrorType`: `sign_error`, `arithmetic_error`, `schema_recognition_error`, `strategy_selection_error`, `representation_error`, `constraint_application_error`, `inference_error`, `search_case_error`, `contradiction_handling_error`, `transformation_error`.
  - Recommends targeted remediation actions (e.g., `remediate:coordinate_system_setup`, `remediate:strategy_selection_drill`, `remediate:simpler_numbers_variant`).
- **Progressive 3-Tier Scaffolding:**
  - Level 1: Principle
  - Level 2: Operation
  - Level 3: Intermediate Relation

---

## 4. Native Anki Footer & Mistake Reflection Flow

### 4.1 Mistake Categories & Ground Truth Values
`MISTAKE_CATEGORIES` in `ts/reviewer/components/mistake_footer.ts:25-58` defines:
1. `key: 1` $\to$ `"silly_mistake"` (`Silly Slip` / Arithmetic or calculation slip)
2. `key: 2` $\to$ `"pattern_not_recognized"` (`Pattern Missed` / Failed to identify problem structure or schema)
3. `key: 3` $\to$ `"formula_or_concept_misapplied"` (`Concept Gap` / Wrong formula or misapplied theorem)
4. `key: 4` $\to$ `"concept_not_known"` (`Prereq Unknown` / Fundamental knowledge gap or missing prerequisite)

### 4.2 Reflection Gate & Space/Enter Trap
To ensure deliberate metacognitive reflection and prevent mindless fast-skipping:
- In `mistake_classification` state, Space and Enter key events are strictly trapped (`e.preventDefault()`, `e.stopPropagation()`).
- Pressing Space or Enter **cannot** bypass the mistake classification step unless a mistake button is actively selected.
- Selecting `1`, `2`, `3`, or `4` immediately classifies the mistake, dispatches `procedural_mistake`, and advances to feedback after 150ms.

### 4.3 Isolation & Non-Interference with Standard Flashcards
- All StudyLab DOM elements exist strictly within the container `#procedural-card` (`.procedural-card-container`).
- In Python/Qt (`qt/aqt/reviewer.py:207, 410`), before loading any card (standard Anki or procedural), `reviewer.py` evaluates:
  ```javascript
  if (globalThis.anki && globalThis.anki.procedural && typeof globalThis.anki.procedural.destroyActive === 'function') {
      globalThis.anki.procedural.destroyActive();
  }
  ```
- Additionally, a `MutationObserver` on `document.body` monitors the container. If `#procedural-card` is removed from the DOM (e.g. during transition to a standard flashcard), `reviewer.destroy()` is automatically invoked.
- This guarantees zero memory leaks, zero event listener residue, and 100% non-interference with standard Anki flashcards.

---

## 5. Teardown Lifecycle & Memory Safety

`ProceduralReviewer.destroy()` (`ts/reviewer/procedural.ts:1239-1278`) executes comprehensive cleanup:
1. Sets state to `"teardown"` and `hasSubmitted = true`.
2. Clears and nulls `this.timerInterval`.
3. Clears and nulls `this.focusTimeout`.
4. Calls `destroy()` on all child components: `mcqContainer`, `numericalContainer`, `mistakeFooter`, `stepwiseContainerComponent`.
5. Iterates through all registered closures in `this.disposables` (removing all event listeners from `window`, `document`, and child nodes).
6. Disconnects and releases the `MutationObserver`.
7. Resets `(globalThis as any).__activeProceduralReviewer = null`.

`proceduralAPI.setup(...)` enforces idempotency: if an active reviewer exists on `globalThis` or the target container, it is destroyed before creating a new instance.

---

## 6. Comprehensive Test Enumeration & Evidence

Vitest test execution confirms **18 test files passing (150 tests total)**, of which **7 test files and 94 tests** directly verify the reviewer and procedural engine:

### Summary Table of Reviewer Test Suites

| # | Test File | Test Count | Key Areas Covered |
|---|---|:---:|---|
| 1 | `ts/reviewer/components/numerical_container.test.ts` | 28 | 5D dimensional algebra, 50+ unit conversions (Physics & Chemistry), scientific notation (`1.2e-3`, `6.022x10^23`), fractions (`3/4`), equation prefixes, physical non-negativity, tolerance checking, unit error diagnostics, live preview pill. |
| 2 | `ts/reviewer/procedural.test.ts` | 27 | Full `ProceduralReviewer` API, stopwatch, mode switching (Quick vs Stepwise), hints, ConceptCheck, StrategyDrill, WorkedExample, DeclarativeRecall, Space/Enter trapping in mistake state, performance ratio classification (`fast_correct`, `on_target_correct`, `slow_correct`, `incorrect`), telemetry persistence via `mutateNextCardStates`, MutationObserver teardown. |
| 3 | `ts/reviewer/components/mcq_container.test.ts` | 12 | Zero text input fallback enforcement, ARIA radiogroup/radio/aria-checked, roving tabindex, click selection, keyboard shortcuts (`1-4`, `A-D`), arrow navigation, Enter/Space activation, canonical evaluation, ConceptCheck/StrategyDrill, Mock exam mode (GAP-MOD-03 spoiler suppression). |
| 4 | `ts/reviewer/diagnostic/diagnostic_session.test.ts` | 10 | `DiagnosticSessionController`: Question palette grid, countdown timer & 120s warning, numerical & MCQ question rendering, answer recording via click & keyboard (`1-4`, `A-D`), mark-for-review toggle (`m`), clear answer, jump to question, test submission bridge dispatch. |
| 5 | `ts/reviewer/components/stepwise_container.test.ts` | 7 | `StepwiseContainer`: Solution graph population, dynamic step row addition & reset, algebraic/linear root equivalence, downstream consistency tracking (`PartiallyValid`), reasoning constraint diagnosis, 3-tier progressive hints. |
| 6 | `ts/reviewer/diagnostic/diagnostic_report.test.ts` | 5 | `DiagnosticReportController`: 4-dimension summary counts (Concept, Calculation, Transfer, Speed), weak/slow/transfer chips, 4-tier hierarchy nodes (Subject, Chapter, Topic, ProblemFamily), collapsible trees, follow-up remediation bridge trigger. |
| 7 | `ts/reviewer/lib.test.ts` | 5 | Anki customData protobuf transform (`applyStateTransform`), scheduling states mutation, interval adjustments, customData packing/unpacking. |
| 8 | `ts/tests/e2e/procedural-runtime.spec.ts` | E2E | Playwright CDP connection to live Anki QtWebEngine: APKG fixture import, deck navigation, review UI rendering, answer submission, mistake classification click, ease button click. |
| 9 | `ts/tests/e2e/procedural-smoke.spec.ts` | E2E | Playwright RPC validation: APKG package import, `renderExistingCard` RPC, Math vs Reasoning card HTML verification, schema check, mistake classification script verification. |

---

## 7. Python/Qt Bridge Integration Mapping

| TypeScript Bridge Call | Python/Qt Receiver (`qt/aqt/reviewer.py`) | Action / Telemetry Effect |
|---|---|---|
| `bridgeCommand("procedural_attempt:<json>")` | `_on_procedural_attempt(data)` | Records attempt telemetry; transitions reviewer state to `"answer"`; calls `_showEaseButtons()`. |
| `bridgeCommand("procedural_mistake:<json>")` | `_on_procedural_mistake(data)` | Records mistake classification (`silly_mistake`, `pattern_not_recognized`, `formula_or_concept_misapplied`, `concept_not_known`). |
| `bridgeCommand("procedural_hint:<json>")` | `_on_procedural_hint(data)` | Records progressive hint request level and latency. |
| `bridgeCommand("procedural_validate_steps:<json>")` | `_on_procedural_validate_steps(data)` | Records multi-step reasoning validation telemetry and first error localization. |
| `bridgeCommand("procedural_try_similar:<json>")` | `_on_procedural_try_similar(data)` | Re-generates problem instance with new random seed for the same problem family. |
| `bridgeCommand("procedural_practice_prerequisite:<json>")` | `_on_procedural_practice_prerequisite(data)` | Navigates to targeted remedial practice for the identified missing prerequisite skill. |
| `bridgeCommand("procedural_declarative_recall:<json>")` | `_on_procedural_declarative_recall(data)` | Opens or schedules the associated foundational Anki flashcard for declarative recall. |
| `bridgeCommand("procedural_answer:<ease>")` | `_handle_procedural_command("procedural_answer:<ease>")` | Answers card with ease `1` (Again on error), `3` (Good on on-target correct), or `4` (Easy on fast correct). |
| `bridgeCommand("ans")` | Native Anki Reviewer Handler | Synchronizes Anki webview state to show bottom ease buttons. |
