# StudyLab Frontend Acceptance Matrix & "Perfect Window" Contract

**Document Version:** 1.1.0 (Reconciled with STUDYLAB_UI_COMPOSITION_CONTRACT.md)  
**Target Subsystem:** Frontend Reviewer (`ts/reviewer/`), Desktop Host (`qt/aqt/reviewer.py`), and QA Verification Harness (`tools/live_modality_verifier.py`)  
**Status:** AUTHORITATIVE CANONICAL SPECIFICATION  
**Integrity Mode:** 100% Grounded in Live Desktop Forensic Evidence, WCAG 2.1 AA, and Performance Benchmarks  
**Authoritative Reference:** `docs/STUDYLAB_UI_COMPOSITION_CONTRACT.md`, `PROJECT.md`

---

## 1. Executive Summary & Quality Principles

This document establishes the definitive, testable acceptance criteria for every screen, state, interaction modality, and responsive viewport in StudyLab. It defines what constitutes a **"Perfect Window"** — the standard of performance, visual calmness, accessibility, and pedagogical rigor that every StudyLab interaction must achieve.

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                           THE "PERFECT WINDOW" STANDARD                          │
│                                                                                  │
│   "A StudyLab review window is perfect when the problem statement is the         │
│    uncluttered visual hero, micro-interactions respond in under 50ms,            │
│    the interface is 100% operable by keyboard alone, error reflection is         │
│    metacognitively unskippable, all 8 visual anti-patterns are eliminated,       │
│    and zero raw engine telemetry leaks to the learner surface."                  │
└──────────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Section 21: Screen-by-Screen Acceptance Contract (14 Canonical Verification States)

Below is the exhaustive acceptance contract across all 14 canonical reviewer screens and modalities:

---

### Screen 1: Mathematics Numerical (Algebra / Linear Equations)
- **Modality:** `NumericalContainer` (Quick Solve with optional Stepwise toggle).
- **Target Skill:** Solving algebraic equations (e.g., $4x - 7 = 21$).
- **Visual Assertions:**
  - `#proc-prompt` displays rendered LaTeX formula without layout shift.
  - `#proc-answer-input` is automatically focused on mount with placeholder `"Type final answer..."`.
  - `#proc-submit-btn` (`Submit Answer`) is visible and active.
  - `.proc-mode-tabs` displays `Quick Solve` (active) and `Step-by-Step` (inactive) pills.
  - `#proc-mcq-container` is **STRICTLY ABSENT** from the DOM.
  - Zero ticking stopwatch counter (`ANTI-03`).
- **Interaction Assertions:**
  - Typing `7` updates input value in < 16ms.
  - Pressing `Enter` triggers `handleQuickSubmit()` immediately.
  - Pressing `Alt+S` switches seamlessly to Stepwise workspace without losing typed input.
- **Evaluation Assertions:**
  - Evaluates algebraic equivalence ($x = 7$, `7`, `+7` are all valid).
  - Submitting `7` advances to `feedback` with subtle inline `✓ Correct` status on open canvas.
  - Submitting `9` transitions to `mistake_classification` with inline status and deferred solution.
- **Pass / Fail Verdict Criteria:** **PASS** if zero MCQ options appear, input receives auto-focus, and evaluation responds in < 200ms.

---

### Screen 2: Mathematics MCQ (Commercial / Profit & Loss)
- **Modality:** `MCQContainer` (4 Discrete Option Cards A–D).
- **Target Skill:** Calculating Cost Price from Selling Price and profit margin.
- **Visual Assertions:**
  - Rendered problem stem with currency symbols (e.g., `₹540`, `20%`).
  - Exactly 4 discrete option cards (`.proc-option-item`) inside `.proc-option-group` (`role="radiogroup"`).
  - Each card displays key badge (`A`, `B`, `C`, `D`) and formatted label (e.g., `A: ₹450`).
  - `#proc-answer-input`, `#proc-quick-container`, and `.proc-mode-tabs` are **STRICTLY ABSENT**.
- **Interaction Assertions:**
  - Pressing `A` or `1` immediately selects Option A with active 2px highlight border and `aria-checked="true"`.
  - Arrow keys (`Up` / `Down`) cycle focus smoothly with roving tabindex.
  - Clicking an option or pressing `Enter` confirms choice and evaluates instantly.
- **Evaluation Assertions:**
  - Evaluates against canonical `expected_option_id`.
  - Generates `AttemptResultPayload` containing selected option ID and distractor metadata.
- **Pass / Fail Verdict Criteria:** **PASS** if zero text input fallback exists, options are keyboard navigable, and selected option highlights instantly.

---

### Screen 3: Logical Reasoning MCQ (Blood Relations / Seating Arrangements)
- **Modality:** `MCQContainer` with Structured Logic Stem.
- **Target Skill:** Deductive relationship reasoning.
- **Visual Assertions:**
  - Clear narrative problem prompt formatted for readability.
  - 4 discrete relationship option cards (e.g., `A: Sister`, `B: Mother`, `C: Aunt`, `D: Daughter`).
  - Active selection highlighted with 2px primary accent border.
  - Zero raw schema names (`family.reasoning.*`) or internal generator IDs visible (`ANTI-06`).
- **Interaction Assertions:**
  - Hotkeys `1`..`4` / `A`..`D` operate without mouse dependency.
  - Double clicking does not trigger duplicate submission.
- **Pass / Fail Verdict Criteria:** **PASS** if narrative text wraps cleanly without horizontal scroll and hotkeys function reliably.

---

### Screen 4: Physics Numerical (Kinematics / Physical 5D Vectors)
- **Modality:** `NumericalContainer` with Live 5D Dimensional Vector Parsing.
- **Target Skill:** 1D Free Fall calculation requiring magnitude and dimensional unit ($v = \sqrt{2gh}$).
- **Visual Assertions:**
  - Problem prompt specifying initial conditions ($h = 45\,\text{m}$, $g = 10\,\text{m/s}^2$).
  - `#proc-answer-input` field with live preview pill `.proc-num-preview-pill` below it.
  - Typing `30 m/s` renders live preview pill: `Parsed: 30 m/s` ($[L]^1 [T]^{-1}$).
  - Redundant static unit badges (`.proc-unit-hint`) are **STRICTLY REMOVED** (prevents visual duplication).
- **Interaction Assertions:**
  - Accepts compound unit aliases: `m/s`, `m s^-1`, `m/sec`, `meters per second`.
  - Handles unit prefixes (e.g., `0.03 km/s` $\rightarrow$ converted to `30 m/s`).
  - Tolerance check: Accepts $30.0 \pm 0.5\,\text{m/s}$ (relative tolerance 1.5%).
- **Pass / Fail Verdict Criteria:** **PASS** if single preview pill appears, dimensional units are validated, and unit conversions execute correctly.

---

### Screen 5: Chemistry Numerical (Physical Chemistry / Mole Concept)
- **Modality:** `NumericalContainer` with Stoichiometric Amount Dimensions.
- **Target Skill:** Molar mass and stoichiometry calculation ($n = m / M$).
- **Visual Assertions:**
  - Problem prompt with formatted chemical formulas (e.g., $\text{CO}_2$, $\text{H}_2\text{SO}_4$).
  - Single input field accepting numeric magnitude and molar unit (e.g., `1.0 mol`).
  - Dynamic preview pill displaying parsed chemical amount: `Parsed: 1 mol` ($[N]^1$).
- **Interaction Assertions:**
  - Accepts `mol`, `moles`, `mmol` ($10^{-3}\,\text{mol}$), `kmol` ($10^3\,\text{mol}$).
  - Enforces non-negativity constraint (negative moles rejected immediately).
- **Pass / Fail Verdict Criteria:** **PASS** if chemical sub-indices render via MathJax and molar conversions compute accurately.

---

### Screen 6: ConceptCheck (Commercial / Successive Percentage Distractor Diagnostics)
- **Modality:** `ConceptCheck` (Targeted Distractor Feedback).
- **Target Skill:** Diagnosing the additive fallacy in successive percentage changes.
- **Visual Assertions:**
  - Prompt: *"A price increases by 10% and then increases again by 10%. What is the net percentage increase?"*
  - 4 Conceptual Choices (`role="radio"`).
  - Free text input field is **STRICTLY ABSENT**.
- **Interaction & Diagnostic Assertions:**
  - Selecting Distractor 2 (`+20%`) reveals immediate targeted diagnostic callout with 3px left accent:
    *"⚠️ Additive Fallacy: The second 10% increase acts on the already-increased base (1.10), giving $1.10 \times 1.10 = 1.21$ (+21%), not +20%."*
  - Misconception tag `misconception.percentage.additive_fallacy` recorded in telemetry.
- **Pass / Fail Verdict Criteria:** **PASS** if distractor-specific feedback appears immediately without revealing generic solution text.

---

### Screen 7: StrategyDrill (Arithmetic Rates / Mixtures & Alligation)
- **Modality:** `StrategyDrill` (Method Selection & Optimality Analysis).
- **Target Skill:** Selecting optimal problem-solving strategy (Alligation Cross vs Algebraic Equations).
- **Visual Assertions:**
  - Context box detailing mixture pricing scenario.
  - Option cards representing distinct solving strategies with compact step counts and speed ratings.
- **Interaction & Feedback Assertions:**
  - Selecting Option 1 reveals optimality rationale with 3px left accent rule.
  - Evaluated against `preferred_option_id`.
- **Pass / Fail Verdict Criteria:** **PASS** if strategy comparison feedback renders clearly and encourages optimal cognitive path.

---

### Screen 8: WorkedExample (Commercial / Dishonest Shopkeeper)
- **Modality:** `WorkedExample` (Expert Modeling & Solution Trace).
- **Target Skill:** Expert solution trace modeling for high-recurrence failure loops.
- **Visual Assertions (`ANTI-07`):**
  - Open canvas layout with subtle horizontal dividers (zero nested boxed containers).
  - Highlighted Key Decision Point card with 3px left accent rule:
    *"⭐ Key Decision: The base of the percentage calculation is the actual weight dispensed (900g), not the advertised 1000g."*
  - Sequentially numbered canonical derivation steps with MathJax formatting.
  - Method rationale and common pitfalls callout.
  - Primary Action Gate button: `[ ✔ I Have Reviewed and Understood This Solution — Try Similar Problem ]` (`#proc-try-similar-btn`).
  - Solving input boxes and MCQ options are **STRICTLY ABSENT**.
- **Interaction Assertions:**
  - Pressing `Enter`, `Space`, or clicking the gate button dispatches `procedural_try_similar` and loads a fresh practice variant (`TransferRetry`).
- **Pass / Fail Verdict Criteria:** **PASS** if solving inputs are completely absent and the acknowledgment gate triggers a new variant.

---

### Screen 9: Stepwise Solving Workspace (Algebra / Multi-Step Derivations)
- **Modality:** `StepwiseContainer` (Cognitive Tutor Inner Loop).
- **Target Skill:** Multi-step algebraic equation derivation.
- **Visual Assertions:**
  - Multi-row derivation workspace (`#proc-stepwise-container`).
  - Step 1 row contains sub-goal prompt and LaTeX input field.
  - Stepwise control toolbar: `[ + Add Step ]`, `[ 💡 Request Hint ]`, `[ ↺ Reset ]`, `[ Check Solution ]`.
  - Quick solve single input box is **STRICTLY ABSENT**.
- **Interaction & CAS Validation Assertions:**
  - Typing `5x = 30` in Step 1 and pressing `Enter` validates the step as `✓ Valid` (green badge) and automatically appends/focuses Step 2.
  - Downstream consistency: If a student makes an intermediate calculation error in Step 1 but logically deduces Step 2 from that error, Step 2 is marked `⚠️ Consistent with Prior Error` (`is_downstream_consistent = true`), preventing double-penalty.
  - Pressing `Ctrl+Enter` or clicking `[ Check Solution ]` evaluates all steps against the Rust `StepValidator`.
- **Pass / Fail Verdict Criteria:** **PASS** if individual step validation badges render properly and downstream consistency logic prevents cascading failure.

---

### Screen 10: Wrong Answer Outcome State
- **Modality:** Evaluation Failure & Metacognitive Trigger.
- **Visual Assertions:**
  - Inputs disabled (`.proc-input-locked`).
  - Concise inline outcome indicator: `✗ Incorrect` (`.proc-status-incorrect`).
  - Giant red full-bleed banners (`ANTI-01`) are **STRICTLY FORBIDDEN**.
  - Primary `Next Problem` button is **STRICTLY HIDDEN**.
  - Mistake classification strip `#proc-mistake-panel` is displayed immediately.
- **Interaction & Anti-Bypass Assertions:**
  - Pressing `Space` or `Enter` is **strictly trapped** (`e.preventDefault()`, `e.stopPropagation()`).
  - The student **cannot** skip to the next card until error classification is performed.
- **Pass / Fail Verdict Criteria:** **PASS** if Next button is hidden and Space/Enter cannot skip the reflection gate.

---

### Screen 11: Mistake Classification Reflection Gate
- **Modality:** `MistakeFooter` Metacognitive Reflection Strip.
- **Visual Assertions:**
  - Compact horizontal strip with 4 classification buttons (`data-key="1..4"`):
    1. `[1 Silly Slip]` (`silly_mistake`: Calculation or sign slip)
    2. `[2 Pattern Missed]` (`pattern_not_recognized`: Unfamiliar problem structure)
    3. `[3 Concept Gap]` (`formula_or_concept_misapplied`: Wrong formula applied)
    4. `[4 Prereq Unknown]` (`concept_not_known`: Missing prerequisite knowledge)
  - Full solution derivation (`#proc-solution-container`) is **STRICTLY HIDDEN** during reflection (`ANTI-08`).
- **Interaction Assertions:**
  - Pressing numeric key `1`, `2`, `3`, or `4` highlights the selected button and triggers transition after 150ms delay.
  - Emits `bridgeCommand("procedural_mistake:<json>")`.
  - Reveals canonical solution only upon selection.
- **Pass / Fail Verdict Criteria:** **PASS** if all 4 categories respond to keyboard shortcuts 1–4, Space/Enter remain trapped, and solution is deferred.

---

### Screen 12: Clean Result Feedback & Next Action State
- **Modality:** Consolidated Outcome & Derivation Review.
- **Visual Assertions:**
  - Open canvas layout without full-bleed red/green containers (`ANTI-01`).
  - Single, deduplicated expected answer row (`Your answer: 30 m/s · Correct answer: 30 m/s`) rendered exactly once (`ANTI-02`).
  - Complete canonical MathJax derivation trace `.proc-derivation-trace` (`ANTI-07`).
  - Performance speed pill: compact muted badge (e.g. `⚡ Fast & Accurate · 8.4s`) (`ANTI-04`).
  - Single primary action button: `[ Next Problem ➔ (Space / Enter) ]` (`#proc-next-btn`).
  - Native Anki bottom ease buttons remain hidden/suppressed.
  - Raw engine telemetry (`attempt_id`, `loss_score`, `raw_seed`, `schema_id`) is **STRICTLY HIDDEN** (`ANTI-06`).
- **Interaction Assertions:**
  - Pressing `Space` or `Enter` executes `handleNext()`, dispatching `procedural_answer:<ease>` with calibrated FSRS ease.
  - Pressing keys `1`..`4` provides explicit ease override (`1: Again`, `2: Hard`, `3: Good`, `4: Easy`).
- **Pass / Fail Verdict Criteria:** **PASS** if zero duplicate text exists, speed pill renders, and Space/Enter smoothly advances the deck.

---

### Screen 13: Normal Basic Card (Native Reviewer Isolation)
- **Modality:** Native Anki Reviewer (non-procedural Basic flashcard).
- **Target Card:** Standard Basic flashcard (Front / Back).
- **Visual Assertions:**
  - Rendered via standard Anki template without `#procedural-card` or `#proc-root` wrappers.
  - Native `#ansbut` (`Show Answer`) displayed in question state.
  - Native `Again (1)`, `Hard (2)`, `Good (3)`, `Easy (4)` rating buttons displayed in answer state.
  - Zero StudyLab UI chrome, zero procedural styles, zero procedural DOM injection.
- **Interaction Assertions:**
  - `Space` flips card to answer side natively.
  - `1`, `2`, `3`, `4` keys rate card natively via standard Anki scheduler.
  - Zero procedural interception, zero procedural IPC bridge dispatch.
- **Pass / Fail Verdict Criteria:** **PASS** if standard Basic flashcards are 100% untouched native Anki reviews.

---

### Screen 14: Normal Cloze Card (Native Reviewer Isolation)
- **Modality:** Native Anki Reviewer (non-procedural Cloze card).
- **Target Card:** Standard Cloze flashcard (`{{c1::term}}`).
- **Visual Assertions:**
  - Standard Cloze formatting with blue highlight on `[...]`.
  - Standard Anki question/answer lifecycle.
  - Native `#ansbut` and ease rating buttons operating normally.
- **Interaction Assertions:**
  - Spacebar flips card; 1–4 keys rate card natively.
- **Pass / Fail Verdict Criteria:** **PASS** if native Cloze review is 100% untouched.

---

## 3. Responsive Layout & Edge-Case Acceptance Criteria

| Scenario / Edge Case | Acceptance Standard | Failure Invalidation Condition |
|---|---|---|
| **Compact Mobile Viewport (320px – 480px)** | All inputs, buttons, and text wrap vertically without horizontal overflow. Option cards stack in a single column with min tap target 44x44px. | Horizontal scrollbar appears on card body; buttons clip outside viewport. |
| **Standard Desktop Viewport (600px – 1200px)** | Card centered with max-width 720px (`#proc-root`). Clear 16px padding between panels. | Excessive whitespace; elements stretched unreadably across entire screen. |
| **Ultra-Wide Viewport (1440px – 4K)** | Content remains contained within 720px max-width container with subtle margin. Font sizes scale harmoniously. | Problem text spans 3000px horizontally; reading lines exceed 120 characters. |
| **Complex MathJax Formula Overflow** | Long mathematical equations wrap smoothly or enable localized horizontal scroll on formula container only. | Equation breaks card boundary or forces global webview horizontal scroll. |
| **High-Precision Decimals & Scientific Notation** | Accepts `3.14`, `3.14159`, and `1.5e5`. Trailing zeros handled accurately based on significant figures. | Floating point precision errors ($0.1 + 0.2 = 0.30000000000000004$) rejected as incorrect. |
| **Physical Unit Whitespace Resilience** | Accepts `30m/s`, `30 m/s`, `30  m/s`, `30 m / s`, `30 meter/second`. | Missing space between number and unit causes parsing crash. |
| **Rapid Double Click / Keypress Debounce** | Submitting answer or clicking options twice in < 100ms triggers exactly one evaluation and one IPC event. | Duplicate attempts logged in `procedural.db`; race condition crashes state machine. |
| **Webview Navigation / Card Teardown** | Navigating to a standard Anki card unmounts container, disconnects all listeners, and restores native Anki shortcuts. | Memory leak; orphaned keydown listeners intercepting spacebar on standard flashcards. |

---

## 4. Section 23: "Perfect Window" Definition & Quantitative Thresholds

```text
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                           "PERFECT WINDOW" BENCHMARK LEDGER                             │
├───────────────────────────────┬───────────────────────────┬─────────────────────────────┤
│ Metric Category               │ Target Benchmark          │ Hard Invalidation Limit     │
├───────────────────────────────┼───────────────────────────┼─────────────────────────────┤
│ Keystroke & Input Latency     │ < 16ms (1 Frame at 60fps) │ > 50ms                      │
│ Preview Pill Update Latency   │ < 30ms                    │ > 75ms                      │
│ Local AST Evaluation Latency  │ < 50ms                    │ > 200ms                     │
│ MathJax Typesetting Latency   │ < 100ms per card          │ > 250ms                     │
│ Card Mount to Interactive     │ < 80ms                    │ > 300ms                     │
│ IPC Bridge Dispatch Latency   │ < 10ms                    │ > 50ms                      │
│ WCAG Accessibility Rating     │ WCAG 2.1 AA Compliant     │ Non-compliant color/focus   │
│ Keyboard Operability          │ 100% Mouse-Free Operable  │ Any required mouse click    │
│ Visual Duplication Count      │ Exactly 0 Duplications    │ $\ge 1$ Duplicate Element   │
│ Engine Telemetry Leak Count   │ Exactly 0 Leaks           │ $\ge 1$ Raw Schema Leak     │
│ Anti-Pattern Count (01..08)   │ Exactly 0 Violations      │ $\ge 1$ Anti-Pattern Active │
└───────────────────────────────┴───────────────────────────┴─────────────────────────────┘
```

### 4.1 Latency & Performance Thresholds
1. **Interactive Input Response (< 50ms):** Typing in `#proc-answer-input` or navigating MCQ options with arrow keys must reflect visually in under 50ms without perceptible input lag.
2. **Instant Local Evaluation (< 200ms):** Client-side AST validation, unit algebra parsing, and linear root equivalence checking must complete and render feedback in under 200ms.
3. **Smooth MathJax Typesetting (< 150ms):** Mathematical formulas and chemical symbols must typeset within 150ms of card load without visible font flicker or reflow jumping.

### 4.2 Accessibility & Usability Standards (WCAG 2.1 AA)
1. **Contrast Ratio ($\ge 4.5:1$):** All text elements, option labels, and button text must maintain a minimum contrast ratio of $4.5:1$ against their background ($3:1$ for large headings) in both Light and Dark themes.
2. **Visible Focus Rings:** Every interactive input, option card, and button must display a high-visibility focus outline ($2\text{px solid } \text{var(--proc-primary-color)}$) when focused via keyboard navigation.
3. **Semantic ARIA Structure:**
   - MCQ option container: `role="radiogroup"`, `aria-label="Multiple choice options"`.
   - MCQ option items: `role="radio"`, `aria-checked="true|false"`, `tabindex="0|-1"`.
   - Stepwise workspace: `role="region"`, `aria-label="Step-by-step derivation workspace"`.
   - Error notification banner: `role="alert"`, `aria-live="assertive"`.

### 4.3 Keyboard Navigation Standards (100% Mouse-Free)
A student must be able to complete a 50-problem review session using **only the keyboard**:
- Enter numbers/units $\rightarrow$ Press `Enter` to submit.
- Select options $\rightarrow$ Press `1`..`4` or `A`..`D` $\rightarrow$ Press `Enter`.
- Classify errors $\rightarrow$ Press `1`..`4` in `mistake_classification`.
- Advance to next card $\rightarrow$ Press `Space` or `Enter` in `feedback`.
- Open hint $\rightarrow$ Press `H` $\rightarrow$ Close hint with `Esc`.
