# StudyLab Reviewer State Machine & Lifecycle Specification

**Document Version:** 1.0.0 (Canonical)  
**Target Subsystem:** TypeScript Reviewer Frontend (`ts/reviewer/`) & Desktop Bridge (`qt/aqt/reviewer.py`)  
**Status:** AUTHORITATIVE CANONICAL SPECIFICATION  
**Integrity Mode:** 100% Grounded in Executable Source Code & Test Evidence  

---

## 1. Executive Summary & Core Architectural Invariant

The StudyLab Reviewer intercepts Anki's standard card rendering pipeline to provide a rich, interactive, multi-domain cognitive problem-solving environment inside Anki's webview surface.

> **Core Architectural Invariant:**  
> **"StudyLab is not a flashcard system; it is a procedural problem-solving engine hosted inside Anki."**

Standard Anki cards operate on a binary declarative recall cycle: Question flip $\rightarrow$ Answer reveal $\rightarrow$ Ease rating (1–4). In contrast, the StudyLab Reviewer implements an **11-state interactive problem-solving state machine** (`ProceduralUIState` in `ts/reviewer/procedural.ts:25-36`). It manages:
- Active stopwatch latency tracking.
- Modality-specific inputs (MCQ with zero text fallback, Numerical with 5D dimensional vectors, Stepwise algebraic derivations).
- Progressive 3-tier scaffolding hints.
- Anti-bypass mistake reflection trapping (Hypercorrection effect, Metcalfe 2017).
- Pedagogical speed quadrant classification.
- Seamless bridge synchronization with Anki's desktop host and FSRS scheduler.
- Clean teardown lifecycle guaranteeing zero memory or shortcut listener leaks.

---

## 2. Complete 11-State Lifecycle Transition Diagram

```text
 ┌───────────────┐
 │    loading    │ ◄── Card initialization / DOM mount
 └───────┬───────┘
         │ (DOM bound, components initialized)
         ▼
 ┌───────────────┐
 │     ready     │ ◄── Inputs mounted & focused, timer armed
 └───────┬───────┘
         │ (First user focus / interaction)
         ▼
 ┌───────────────┐        (Hint Requested: 'H' / '?')        ┌───────────────┐
 │    solving    ├──────────────────────────────────────────►│     hint      │
 └───┬───────▲───┘                                           └───┬───────▲───┘
     │       │                                                   │       │
     │       └────────────────── (Resume Solving) ───────────────┘       │
     │                                                                   │
     │ (Submit: Enter / Ctrl+Enter / Option Click)                       │
     ▼                                                                   │
 ┌───────────────┐                                                       │
 │  submitting   │                                                       │
 └───┬───────┬───┘                                                       │
     │       │                                                           │
     │       │ [Correct Attempt]                                         │
     │       └──────────────────────────────────┐                        │
     │                                          │                        │
     │ [Incorrect Attempt]                      │                        │
     ▼                                          │                        │
 ┌───────────────────────────┐                  │                        │
 │  mistake_classification   │                  │                        │
 │  (Traps Space & Enter)    │                  │                        │
 └───────────┬───────────────┘                  │                        │
             │ (Classify: 1..4)                 │                        │
             ▼                                  ▼                        │
     ┌──────────────────────────────────────────────────┐                │
     │                     feedback                     │                │
     │  (Correctness banner, Speed Quadrant, Telemetry) │                │
     └───────────┬──────────────────────────────┬───────┘                │
                 │                              │                        │
                 │ (Try Similar Variant)        │ (Next: Space / Enter)  │
                 ▼                              ▼                        │
         ┌───────────────┐              ┌───────────────┐                │
         │worked_example │              │     next      │                │
         └───────┬───────┘              └───────┬───────┘                │
                 │                              │                        │
                 └─(Reload Variant)─────────────┴─► [Teardown / Anki]    │
                                                          │              │
                                                          ▼              │
                                                  ┌───────────────┐      │
                                                  │   teardown    │      │
                                                  └───────────────┘      │
                                                          ▲              │
 ┌───────────────┐                                        │              │
 │     error     ├────────────────────────────────────────┘              │
 └───────────────┘ (Initialization / Malformed Contract Exception)
```

---

## 3. Comprehensive State-by-State Specification

### 3.1 `loading`
- **Definition:** Initial constructor state during container instantiation, template parsing, and DOM element binding (`ts/reviewer/procedural.ts:258-290`).
- **Visible UI:** Loading spinner and card container `#procedural-card` placeholder.
- **Primary Action:** Automated component bootstrap, parsing `inline_contract`, and checking MathJax availability.
- **Secondary Actions:** None.
- **Forbidden Actions:** Any student input, keyboard interaction, or bridge dispatch.
- **Keyboard Behavior:** Completely suppressed.
- **Bridge Events:** None.
- **Persistence Effect:** None.
- **Transition Guard:** Advances to `ready` upon successful element queries and child component mounting; transitions to `error` if contract parsing fails.

### 3.2 `ready`
- **Definition:** Problem statement rendered; answer container mounted, focused, and armed (`ts/reviewer/procedural.ts:312-360`).
- **Visible UI:** Problem statement rendered; appropriate input container active (`#proc-quick-container`, `#proc-stepwise-container`, or `#proc-mcq-container`); mode switcher visible (if applicable). Native Anki ease buttons and answer bar remain hidden.
- **Primary Action:** Student reads prompt and focuses active input field or hovers over MCQ options.
- **Secondary Actions:** Mode toggle (`quick` $\leftrightarrow$ `stepwise`).
- **Forbidden Actions:** Submitting an empty or unselected response.
- **Keyboard Behavior:** Hotkeys armed (`1`–`4` / `A`–`D` for MCQ; numeric keys for Numerical container; arrow keys for radiogroup navigation).
- **Bridge Events:** None.
- **Persistence Effect:** Initializes active solving stopwatch (`this.startTime = Date.now()`).
- **Transition Guard:** Advances to `solving` upon first input interaction or timer tick.

### 3.3 `solving`
- **Definition:** Active problem-solving state with running stopwatch and live input evaluation (`ts/reviewer/procedural.ts:400-600`).
- **Visible UI:** Problem prompt, active input fields, live typing preview pill (Numerical modality), dynamic step rows (Stepwise modality), or ARIA radiogroup options (MCQ modality); "Hint" button (`#proc-hint-btn`) active.
- **Primary Action:** Entering numerical value with units, selecting MCQ option, or typing intermediate algebraic deductions.
- **Secondary Actions:** Clicking "Hint" button or pressing `H` / `?`; switching input modes.
- **Forbidden Actions:** Allowing Space or Enter to propagate to native Anki card flip handlers.
- **Keyboard Behavior:**
  - `Enter` / `Ctrl+Enter`: Triggers `handleQuickSubmit()` or `handleStepwiseSubmit()`.
  - `H` / `?`: Triggers `requestHint()`.
  - `1`–`4` / `A`–`D`: Directly selects options in MCQ container.
  - `ArrowUp` / `ArrowDown` / `ArrowLeft` / `ArrowRight`: Navigates MCQ radiogroup with roving tabindex.
- **Bridge Events:** None during active typing.
- **Persistence Effect:** `this.timerInterval` accumulates active solving time in milliseconds (`timeTakenMs`).
- **Transition Guard:**
  - Advances to `submitting` on submission action.
  - Advances to `hint` on hint request.

### 3.4 `hint`
- **Definition:** Scaffolded hint display state providing pedagogical assistance without revealing the full answer (`ts/reviewer/procedural.ts:630-680`).
- **Visible UI:** Expandable hint card `#proc-hint-box` rendered below prompt with 3-tier progressive content (Principle $\rightarrow$ Operation $\rightarrow$ Intermediate Relation); "Resume Solving" button displayed.
- **Primary Action:** Student reads pedagogical principle or equation setup and returns to solving.
- **Secondary Actions:** Request next progressive hint level (if multi-level hints exist).
- **Forbidden Actions:** Direct answer auto-population.
- **Keyboard Behavior:** `Esc` or `Enter` closes hint panel and refocuses the active answer input.
- **Bridge Events:** Emits `bridgeCommand("procedural_hint:<json>")` recording `instance_id`, `hint_level`, and exposure latency.
- **Persistence Effect:** Increments `hintsUsed`; penalizes `independence` level in `MasteryEvidence` (`Independent` $\rightarrow$ `LightSupport` $\rightarrow$ `SignificantSupport`).
- **Transition Guard:** Returns to `solving` when dismissed or when student resumes typing.

### 3.5 `submitting`
- **Definition:** Transient evaluation state executing local semantic checks and dispatching validation telemetry (`ts/reviewer/procedural.ts:788-842`).
- **Visible UI:** Inputs temporarily disabled; inline evaluation feedback active.
- **Primary Action:** Client-side AST normalization, dimensional unit conversion, linear root equivalence, or MCQ option ID comparison.
- **Secondary Actions:** None.
- **Forbidden Actions:** Modifying inputs during active evaluation.
- **Keyboard Behavior:** All input events temporarily suppressed.
- **Bridge Events:** None during synchronous local check.
- **Persistence Effect:** Evaluates correctness score (`1.0` or `0.0`), captures `timeTakenMs`, and computes `speedQuadrant`.
- **Transition Guard:**
  - Advances directly to `feedback` if `outcome.isCorrect === true`.
  - Advances to `mistake_classification` if `outcome.isCorrect === false`.

### 3.6 `mistake_classification`
- **Definition:** Metacognitive reflection state activated immediately upon an incorrect attempt (`ts/reviewer/procedural.ts:940-1010`, `ts/reviewer/components/mistake_footer.ts`).
- **Visible UI:** Input containers hidden; result panel `#proc-result-panel` displays submitted vs correct answers; compact `MistakeFooter` strip mounted with 4 classification buttons:
  - `[1 Silly]` (`silly_mistake`: Calculation or sign slip)
  - `[2 Pattern]` (`pattern_not_recognized`: Unfamiliar problem structure)
  - `[3 Concept]` (`formula_or_concept_misapplied`: Wrong formula or misapplied theorem)
  - `[4 Unknown]` (`concept_not_known`: Fundamental missing prerequisite)
- **Primary Action:** Student selects the category matching their error cause.
- **Secondary Actions:** None.
- **Forbidden Actions:** Bypassing mistake classification by fast-skipping.
- **Keyboard Behavior (Anti-Bypass Protection):**
  - **Space and Enter keys are strictly trapped:** `e.preventDefault()` and `e.stopPropagation()` prevent skipping to the next card until classified.
  - Number keys `1`, `2`, `3`, `4`: Immediately select the corresponding category and advance.
- **Bridge Events:** Emits `bridgeCommand("procedural_mistake:<json>")` recording `instance_id`, `family_id`, and `mistake_type`.
- **Persistence Effect:** Ingested into `DomainEvidence` to differentiate mechanical calculation errors from conceptual deficits (`is_execution_error()` vs `is_conceptual_error()`).
- **Transition Guard:** Advances to `feedback` after 150ms button selection delay.

### 3.7 `feedback`
- **Definition:** Comprehensive outcome review state displaying full solution derivations, performance telemetry, and remediation routes (`ts/reviewer/procedural.ts:1015-1190`).
- **Visible UI:**
  - Green (Correct) or Red (Incorrect) result banner.
  - Formatted canonical solution with MathJax derivation.
  - Time elapsed pill and **Speed Quadrant Badge** (`.proc-speed-quadrant`).
  - Native Anki bottom ease buttons revealed via `bridgeCommand("ans")`.
  - Action buttons: "Try Similar Problem", "Practice Prerequisite", and "Next Problem".
- **Primary Action:** Reviewing solution derivation and pressing `Space` / `Enter` or clicking "Next".
- **Secondary Actions:** Clicking "Try Similar Problem" or "Practice Prerequisite".
- **Forbidden Actions:** Re-editing submitted answer.
- **Keyboard Behavior:** `Space` or `Enter` executes `handleNext()`; numbers `1`–`4` trigger native Anki ease ratings.
- **Bridge Events:**
  - Injects telemetry into Anki custom data via `globalThis.anki.mutateNextCardStates(...)`.
  - Emits `bridgeCommand("procedural_attempt:<json>")`.
  - Emits `bridgeCommand("ans")` to synchronize Qt reviewer state.
- **Persistence Effect:** Queues telemetry for atomic write to `procedural.db`.
- **Transition Guard:**
  - Advances to `worked_example` if "Try Similar" is clicked.
  - Advances to `next` when "Next" or `Space`/`Enter` is pressed.

### 3.8 `worked_example`
- **Definition:** Guided review state activated during worked example review or when clicking "Try Similar Problem" (`ts/reviewer/procedural.ts:1192-1200`).
- **Visible UI:** Step-by-step canonical solution derivation with highlighted decision points; input fields suppressed; "I Understand / Generate New Variant" CTA button.
- **Primary Action:** Studying expert derivation and clicking to generate a new problem variant.
- **Secondary Actions:** Expanding intermediate reasoning rationale.
- **Forbidden Actions:** Direct rating without reviewing.
- **Keyboard Behavior:** `Enter` or `Space` triggers new variant generation.
- **Bridge Events:** Emits `bridgeCommand("procedural_try_similar:<json>")`.
- **Persistence Effect:** Records worked example exposure in `remediation_recurrence`.
- **Transition Guard:** Transitions to `ready` with a newly seeded problem instance.

### 3.9 `next`
- **Definition:** Lifecycle completion state handing control back to Anki's FSRS scheduler (`ts/reviewer/procedural.ts:1221-1230`).
- **Visible UI:** Smooth transition to next scheduled card.
- **Primary Action:** Automated cleanup and handover.
- **Secondary Actions:** None.
- **Forbidden Actions:** None.
- **Keyboard Behavior:** Native Anki review hotkeys resume.
- **Bridge Events:** Emits `bridgeCommand("procedural_answer:<ease>")` where `ease = 1` (Again on error), `3` (Good on on-target correct), or `4` (Easy on fast correct).
- **Persistence Effect:** Rust backend answering pipeline commits attempt to `procedural.db`, updates FSRS memory states, and pulls next card.
- **Transition Guard:** Hands over to `loading` for next procedural card or standard Anki template for standard cards.

### 3.10 `error`
- **Definition:** Fault-tolerant error boundary state (`ts/reviewer/procedural.ts:285-290`).
- **Visible UI:** Structured red warning banner displaying diagnostic error information.
- **Primary Action:** User clicks "Skip Card" or reports template issue.
- **Secondary Actions:** Copy error diagnostics to clipboard.
- **Forbidden Actions:** Crashing Anki webview or silently corrupting database.
- **Keyboard Behavior:** Native Anki shortcuts restored.
- **Bridge Events:** Logs error to console and Python bridge.
- **Persistence Effect:** None.
- **Transition Guard:** Transitions to `teardown` on card advance.

### 3.11 `teardown`
- **Definition:** Terminal cleanup state destroying all active event listeners, intervals, observers, and DOM bindings (`ts/reviewer/procedural.ts:1239-1278`).
- **Visible UI:** Container unmounted from webview.
- **Primary Action:** Complete garbage collection and listener detachment.
- **Secondary Actions:** None.
- **Forbidden Actions:** Executing delayed asynchronous callbacks after unmount.
- **Keyboard Behavior:** 100% restored to native Anki.
- **Bridge Events:** None.
- **Persistence Effect:** Nullifies global active reviewer references.
- **Transition Guard:** Terminal state.

---

## 4. Anti-Bypass Guardrails & Keyboard Isolation

### 4.1 Reflection Gate Trapping Mechanics
To prevent learners from mindlessly skipping incorrect answers without processing the error (Cognitive Load Theory & Hypercorrection Effect, Metcalfe 2017):

```typescript
// ts/reviewer/procedural.ts:310-360
this.disposables.push(
    addListener(window, "keydown", (e: KeyboardEvent) => {
        if (this.state === "mistake_classification") {
            // Strictly trap Space and Enter
            if (e.code === "Space" || e.key === " " || e.key === "Enter" || e.code === "Enter") {
                e.preventDefault();
                e.stopPropagation();
                return;
            }
            // Route single-keystroke classification
            if (["1", "2", "3", "4"].includes(e.key)) {
                e.preventDefault();
                e.stopPropagation();
                const categoryMap: Record<string, string> = {
                    "1": "silly_mistake",
                    "2": "pattern_not_recognized",
                    "3": "formula_or_concept_misapplied",
                    "4": "concept_not_known",
                };
                this.selectMistakeCategory(categoryMap[e.key]);
                return;
            }
        }
    }, true) // Capture phase interception
);
```

### 4.2 Standard Card Non-Interference Guarantee
1. **Container Scoping:** All StudyLab DOM elements are strictly isolated inside `#procedural-card` (`.procedural-card-container`).
2. **Deterministic Unmount:** In `qt/aqt/reviewer.py:207, 410`, before rendering any card (standard or procedural), Python evaluates:
   ```javascript
   if (globalThis.anki && globalThis.anki.procedural && typeof globalThis.anki.procedural.destroyActive === 'function') {
       globalThis.anki.procedural.destroyActive();
   }
   ```
3. **`MutationObserver` Safety Net:** A `MutationObserver` on `document.body` monitors `#procedural-card`. If the container is removed from the DOM (e.g., Anki transitions to a standard flashcard), `reviewer.destroy()` is automatically invoked.
4. **Zero Listener Leakage:** Tested across 1,000 continuous card transitions with 0 residual event listeners or memory growth (`desktop_validation_master_suite.rs`, Section 7).

---

## 5. Speed Quadrant Telemetry Matrix

Performance is categorized on submission completion via `computeSpeedQuadrant(isCorrect, timeTakenMs, targetTimeMs)` (`ts/reviewer/procedural.ts:844-875`):

```
                       LATENCY ≤ TARGET TIME          LATENCY > TARGET TIME
                   ┌─────────────────────────────┬─────────────────────────────┐
                   │     ⚡ FLUENCY STRENGTH      │    ⏱ SPEED OPPORTUNITY      │
   CORRECT ANSWER  │      (Accurate & Fast)      │   (Accurate but Slow)       │
                   │  Badge: .proc-speed-fast-   │  Badge: .proc-speed-slow-   │
                   │         correct             │         correct             │
                   ├─────────────────────────────┼─────────────────────────────┤
                   │   ⚠️ CHECK STRATEGY / TRAP  │ 💡 REVIEW CONCEPT / SETUP   │
  INCORRECT ANSWER │    (Fast but Incorrect)     │    (Slow & Incorrect)       │
                   │  Badge: .proc-speed-fast-   │  Badge: .proc-speed-slow-   │
                   │         wrong               │         wrong               │
                   └─────────────────────────────┴─────────────────────────────┘
```

### Quadrant Telemetry Definitions:
1. **`fluency_strength` (`isCorrect && isFast`):**
   - **Label:** `⚡ Fluency Strength (Accurate & Fast)`
   - **Styling:** Emerald green badge (`#10b981`), white text.
   - **Pedagogical Meaning:** High procedural fluency; candidate for difficulty advancement ($L1 \to L5$) or Ease 4 promotion.
2. **`speed_opportunity` (`isCorrect && !isFast`):**
   - **Label:** `⏱ Speed Opportunity (Accurate but Slow)`
   - **Styling:** Amber badge (`#f59e0b`), white text.
   - **Pedagogical Meaning:** Correct schema formulation but high cognitive friction; requires fluency drill repetitions.
3. **`strategy_trap` (`!isCorrect && isFast`):**
   - **Label:** `⚠️ Check Strategy / Trap (Fast but Incorrect)`
   - **Styling:** Rose red badge (`#ef4444`), white text.
   - **Pedagogical Meaning:** Impulsive execution, reading trap, or calculation slip; recommends decision-point strategy drills.
4. **`concept_setup` (`!isCorrect && !isFast`):**
   - **Label:** `💡 Review Concept / Setup (Slow & Incorrect)`
   - **Styling:** Purple badge (`#8b5cf6`), white text.
   - **Pedagogical Meaning:** Severe conceptual blockage; triggers immediate Concept Check or Worked Example remediation.

---

## 6. Modality-Specific Reviewer Behaviors

### 6.1 Multiple Choice Question (`MCQContainer`)
- **Zero Free-Text Input Enforcement (`GAP-MOD-01`):** `enforceZeroTextInputFallback()` explicitly hides `#proc-quick-container`, `#proc-stepwise-container`, `.proc-mode-switch` and sets `#proc-answer-input` to `disabled = true` with `aria-hidden = true`.
- **ARIA Accessibility:** Container assigned `role="radiogroup"`; each option assigned `role="radio"`, `aria-checked="true|false"`, and roving `tabindex="0|-1"`.
- **Keyboard Shortcuts:** `1`–`4` or `A`–`D` (alphabetic, case-insensitive) immediately select option; Arrow keys navigate; Space/Enter confirms.
- **Mock Exam Mode (`GAP-MOD-03`):** When `mode: "mock"`, option selection applies `.selected` styling but strictly suppresses `.correct`, `.incorrect`, and spoiler feedback until entire mock session submission.

### 6.2 Numerical Modality (`NumericalContainer`)
- **5D Dimensional Vector Algebra:** Analyzes physical and chemical dimensions $[M]^m[L]^l[T]^t[N]^n[K]^k$.
- **50+ Unit Conversions:** Automatically verifies compatible dimensions and applies linear multipliers / offsets (e.g. `72 km/h` $\leftrightarrow$ `20 m/s`, `1.2 mM` $\leftrightarrow$ `0.0012 M`, Celsius $\leftrightarrow$ Kelvin).
- **Live Preview Pill:** Renders real-time parsed value and recognized unit underneath the input in `.proc-num-preview-pill`.
- **Tolerance Engine:** Default $0.5\%$ relative tolerance (`tol.relative * expected`) or absolute tolerance bands.

### 6.3 Stepwise Multi-Step Derivation (`StepwiseContainer`)
- **Solution Graph Ingestion:** Pre-populates rows with step descriptions and placeholders from `options.solutionGraph.steps`.
- **Semantic Algebraic Equivalence:** Evaluates linear roots (e.g. `2x = 10` $\equiv$ `x = 5`), commutative addition (`a + b` $\equiv$ `b + a`), and percentage multipliers (`0.2` $\equiv$ `20%`).
- **Downstream Consistency (`PartiallyValid`):** If a student makes an error in step $k$, but step $k+1$ is derived algebraically correctly from the erroneous intermediate value, step $k+1$ is marked `PartiallyValid` with `isDownstreamConsistent = true`, localizing the penalty to step $k$.

---

## 7. Teardown & Garbage Collection Lifecycle

`ProceduralReviewer.destroy()` (`ts/reviewer/procedural.ts:1239-1278`) executes comprehensive teardown in 7 discrete steps:

1. **State Invalidation:** Sets `this.state = "teardown"` and `this.hasSubmitted = true`.
2. **Interval Cancellation:** Clears and nullifies `this.timerInterval`.
3. **Timeout Cancellation:** Clears and nullifies `this.focusTimeout`.
4. **Child Component Teardown:** Invokes `.destroy()` on `mcqContainer`, `numericalContainer`, `mistakeFooter`, and `stepwiseContainerComponent`.
5. **Event Listener Disposal:** Iterates through `this.disposables`, invoking unbind closures for all `window`, `document`, and element event listeners.
6. **Observer Disconnection:** Disconnects and releases the `MutationObserver`.
7. **Global Reference Nullification:** Clears `(globalThis as any).__activeProceduralReviewer = null`.

---

## 8. Verification & Test Evidence Matrix

| Test Suite File | Test Count | Verified State Machine Behaviors |
|---|:---:|---|
| `ts/reviewer/procedural.test.ts` | 27 | 11-state transitions, stopwatch, mode switching, hints, ConceptCheck, StrategyDrill, WorkedExample, Space/Enter trapping in `mistake_classification`, speed quadrant computation, telemetry persistence, `MutationObserver` teardown. |
| `ts/reviewer/components/mcq_container.test.ts` | 12 | Zero text input fallback enforcement, ARIA radiogroup, keyboard shortcuts (`1-4`, `A-D`), arrow navigation, mock exam mode spoiler suppression. |
| `ts/reviewer/components/numerical_container.test.ts` | 28 | 5D dimensional vectors, 50+ unit conversions, scientific notation, equation prefixes, physical sanity constraints, tolerance checks, preview pill. |
| `ts/reviewer/components/stepwise_container.test.ts` | 7 | Solution graph dynamic rows, algebraic root equivalence, downstream consistency (`PartiallyValid`), 3-tier progressive hints. |
| `rslib/procedural/tests/desktop_validation_master_suite.rs` | 1000 iter | 1,000 continuous card transitions in 3.09s with zero memory leaks and zero event listener leaks. |
