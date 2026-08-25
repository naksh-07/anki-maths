# StudyLab Frontend UI State Specification

**Document Version:** 1.0.0 (Canonical)  
**Target Subsystem:** TypeScript Reviewer Frontend (`ts/reviewer/`), Reviewer Webview Templates (`rslib/procedural/src/reviewer/`), and Desktop Host Bridge (`qt/aqt/reviewer.py`)  
**Status:** AUTHORITATIVE CANONICAL SPECIFICATION (Mission Section 6)  
**Integrity Mode:** 100% Grounded in Executable Source Code, Live UI Audits, and Educational Invariants  

---

## 1. Executive Summary & Core Architectural Invariants

The StudyLab Reviewer provides an interactive, multi-domain cognitive problem-solving environment embedded within Anki. It replaces Anki's standard binary declarative recall flip cycle with an **explicit, state-driven procedural learning machine**.

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                             PRIMARY PRODUCT TRUTH                                │
│                                                                                  │
│   "StudyLab is a procedural problem-solving engine hosted inside Anki.           │
│    The learner sees the minimum UI necessary to perform the intended             │
│    cognitive interaction. The learning object is primary; UI is subordinate."    │
└──────────────────────────────────────────────────────────────────────────────────┘
```

### Core Invariants:
1. **Semantic Modality Identity:** If a learning object is an MCQ, ConceptCheck, StrategyDrill, or WorkedExample, it **MUST NEVER** render a free-text input field. Semantic modality matches UI interaction surface unconditionally.
2. **One-Interaction-Surface:** At any given state, there is exactly **one** primary call-to-action. Duplicate controls (e.g., in-card submit plus native Anki ease buttons) are strictly forbidden.
3. **Anti-Bypass Metacognitive Reflection Gate:** Upon submitting an incorrect answer, advancing to the next card or revealing full solutions without classifying the error category (`1 Silly Slip`, `2 Pattern Missed`, `3 Concept Gap`, `4 Prereq Unknown`) is **physically and programmatically impossible**. `Space` and `Enter` keys are trapped until classification occurs.
4. **Calm Visual Hierarchy:** The problem statement and interactive elements are the visual heroes. Diagnostic metrics, target times, and raw schema identifiers remain in the backend telemetry layer unless pedagogically required.

---

## 2. Complete State Machine Lifecycle Graph

The StudyLab frontend implements 14 distinct states spanning general lifecycle, interaction modalities, metacognitive reflection, and cleanup:

```text
 ┌───────────────┐
 │    loading    │ ◄── Card initialization & DOM template parse
 └───────┬───────┘
         │ (DOM bound, components mounted, MathJax ready)
         ▼
 ┌───────────────┐
 │     ready     │ ◄── Inputs armed, focus placed, active stopwatch started
 └───────┬───────┘
         │ (First user focus / interaction)
         ▼
 ┌─────────────────────────────────────────────────────────────────────────────────┐
 │                                   SOLVING                                       │
 │                                                                                 │
 │   ┌───────────────────────┐  (H / ?)  ┌───────────────────────┐                 │
 │   │        solving        ├──────────►│         hint          │                 │
 │   │ (Active stopwatch)    │◄──────────┤ (3-tier progressive)  │                 │
 │   └───────────┬───────────┘ (Resume)  └───────────────────────┘                 │
 │               │                                                                 │
 │   ┌───────────┴───────────────────────────────────────────────┐                 │
 │   │                Object-Specific Sub-States                 │                 │
 │   │                                                           │                 │
 │   │  [Stepwise Workspace]         [MCQ / Choice]              │                 │
 │   │  ┌──────────────────────┐     ┌────────────────────────┐  │                 │
 │   │  │   step_answering     │     │     mcq_selected       │  │                 │
 │   │  │ (Row edit / CAS eval)│     │ (Radio active border)  │  │                 │
 │   │  └──────────────────────┘     └────────────────────────┘  │                 │
 │   │                                                           │                 │
 │   │  [Worked Example]             [Concept / Strategy]        │                 │
 │   │  ┌──────────────────────┐     ┌────────────────────────┐  │                 │
 │   │  │  worked_step_reveal  │     │   diagnostic_summary   │  │                 │
 │   │  │ (Trace step review)  │     │ (Distractor feedback)  │  │                 │
 │   │  └──────────────────────┘     └────────────────────────┘  │                 │
 │   └───────────────────────────┬───────────────────────────────┘                 │
 └───────────────────────────────┼─────────────────────────────────────────────────┘
                                 │
                                 │ (Submit: Enter / Ctrl+Enter / Option Selection)
                                 ▼
                         ┌───────────────┐
                         │  submitting   │ ◄── AST check, dimension normalization, CAS eval
                         └───┬───────┬───┘
                             │       │
            [Correct Attempt]│       │[Incorrect Attempt]
                             │       ▼
                             │   ┌───────────────────────────┐
                             │   │  mistake_classification   │ ◄── Space & Enter trapped
                             │   │  (4 Category Reflection)  │
                             │   └───────────┬───────────────┘
                             │               │ (Classify: 1..4)
                             ▼               ▼
                     ┌───────────────────────────────┐
                     │           feedback            │ ◄── Deduplicated canonical solution,
                     │  (Deduplicated Result Screen) │     speed quadrant, single Next CTA
                     └───────────────┬───────────────┘
                                     │
                     ┌───────────────┴───────────────┐
                     │ (Next Problem: Space / Enter) │ (Try Similar Variant)
                     ▼                               ▼
             ┌───────────────┐               ┌───────────────┐
             │     next      │               │worked_example │
             └───────┬───────┘               └───────┬───────┘
                     │ (Teardown / FSRS Answer)      │ (Generate & Seed Variant)
                     ▼                               ▼
             ┌───────────────┐               ┌───────────────┐
             │   teardown    │               │     ready     │
             └───────────────┘               └───────────────┘
                     ▲
                     │ (Fatal exception / Malformed contract)
             ┌───────┴───────┐
             │     error     │
             └───────────────┘
```

---

## 3. Comprehensive State-by-State Specification

---

### 3.1 `loading`

- **State Identification:** `ProceduralUIState = "loading"` (`ts/reviewer/procedural.ts:25-36, 258-290`)
- **Cognitive Purpose & Learner Goal:** Instantaneous initialization screen. Ensures all DOM containers, LaTeX/MathJax renderers, and contract parsers are initialized before exposing inputs to prevent layout shift or dropped keystrokes.
- **Visible Content:**
  - Container wrapper `#procedural-card.proc-card` with subdued placeholder outline.
  - Centered CSS loading spinner or subtle skeleton placeholder (`.proc-loading-indicator`).
- **Visible Controls:** None. All interactive buttons and inputs are either unmounted or set to `display: none`.
- **Hidden Controls:**
  - `#proc-quick-container`, `#proc-stepwise-container`, `#proc-mcq-container` (Hidden).
  - `#proc-submit-btn`, `#proc-check-steps-btn`, `#proc-hint-btn`, `#proc-reset-btn` (Hidden).
  - `#proc-mistake-panel`, `#proc-result-panel` (Hidden).
  - Native Anki bottom bar buttons (`#ansbut`, ease buttons `1`..`4`) (Suppressed by `reviewer.py:_showAnswerButton`).
- **Primary CTA:** None (automated transition).
- **Secondary CTA:** None.
- **Keyboard Behavior:** Completely suppressed. All keydown events return early with `e.preventDefault()`.
- **Native Anki Controls:** Top toolbar visible (`Deck`, `Add`, `Browse`, `Stats`); bottom bar `#ansbut` replaced with remaining card count via `_showAnswerButton()`.
- **State Transitions:**
  - $\rightarrow$ `ready`: Triggered automatically when DOM elements are queried, `inline_contract` is parsed, and MathJax typeset promise resolves.
  - $\rightarrow$ `error`: Triggered if `ProceduralCardAnchor` JSON is missing, corrupted, or incompatible.
- **Backend IPC Events:** None.
- **Learner-Visible Data vs Hidden Engine Data:**
  - *Learner-Visible:* Minimal loading state indicator.
  - *Hidden Engine Data:* Raw payload parsing (`inline_contract`), MathJax version check, memory leak cleanup hooks.

---

### 3.2 `ready`

- **State Identification:** `ProceduralUIState = "ready"` (`ts/reviewer/procedural.ts:312-360`)
- **Cognitive Purpose & Learner Goal:** Clear, unobstructed presentation of the problem statement. The learner reads the problem prompt, reviews context/givens, and prepares their cognitive solving strategy.
- **Visible Content:**
  - Header `.proc-header`: Subject / Domain badge (`.proc-badge-domain`), Topic / Skill title (`.proc-title`), Difficulty rating (optional, `.proc-diff-tag`).
  - Problem Prompt `#proc-prompt`: Rendered statement with embedded MathJax formulas, physics units, chemical equations, or logic puzzles.
  - Active Modality Workspace:
    - *Numerical:* `#proc-quick-container` with `#proc-answer-input` (`placeholder="Type final answer..."`).
    - *MCQ / ConceptCheck / StrategyDrill:* `#proc-mcq-container` with `.proc-option-group` (`role="radiogroup"`).
    - *Stepwise:* `#proc-stepwise-container` with initial empty step row `#proc-step-input-0`.
    - *WorkedExample:* `#proc-worked-box` with canonical derivation trace.
- **Visible Controls:**
  - Modality-specific inputs: `#proc-answer-input` (focused), option cards `.proc-option-item`, or stepwise input `.proc-step-input-0`.
  - Mode Switcher `.proc-mode-tabs` (only for `problem` modality with stepwise support: `Quick Solve` / `Step-by-Step`).
  - Scaffolding CTA: `#proc-hint-btn` (`💡 Request Hint`).
  - Submission CTA: `#proc-submit-btn` (`Submit Answer`) or `#proc-check-steps-btn` (`Check Solution`).
- **Hidden Controls:**
  - `#proc-mistake-panel` (Hidden).
  - `#proc-result-panel` (Hidden).
  - `#proc-hint-box` (Hidden).
  - Native Anki bottom ease buttons (`1`..`4`) and `#ansbut` (Hidden).
- **Primary CTA:** Focus input field or hover over first option card.
- **Secondary CTA:** Toggle between Quick Solve and Stepwise mode (`#proc-tab-stepwise`).
- **Keyboard Behavior:**
  - Focus automatically set to active input or first option item.
  - Hotkeys armed: `1`..`4` / `A`..`D` for MCQ selection; numeric and decimal keys for numerical inputs; `H` / `?` for hint request.
  - Spacebar and Enter do not trigger card flip.
- **Native Anki Controls:** Bottom bar shows only deck progress count (`self._remaining()`). Ease buttons hidden.
- **State Transitions:**
  - $\rightarrow$ `solving`: Triggered on first input keystroke, option click, or after 100ms timer tick.
  - $\rightarrow$ `hint`: Triggered if user clicks `#proc-hint-btn` or presses `H` / `?`.
- **Backend IPC Events:** None.
- **Learner-Visible Data vs Hidden Engine Data:**
  - *Learner-Visible:* Problem text, Givens/Variables, Units required, Input field/Options.
  - *Hidden Engine Data:* Stopwatch initialized (`this.startTime = Date.now()`), target response time `targetTimeMs`, parameter seed, expected canonical expression.

---

### 3.3 `solving`

- **State Identification:** `ProceduralUIState = "solving"` (`ts/reviewer/procedural.ts:400-600`)
- **Cognitive Purpose & Learner Goal:** Active cognitive execution. The student formulates their derivation, calculates numerical values with dimensional units, or reasons through MCQ options.
- **Visible Content:**
  - Problem prompt `#proc-prompt`.
  - Live interaction workspace with active user input:
    - *Numerical Modality:* Active typing with dynamic unit preview pill `.proc-num-preview-pill` (e.g., `Parsed: 30 m/s`).
    - *MCQ Modality:* Active option focus with visual focus indicator outline.
    - *Stepwise Modality:* Multi-row algebraic derivation workspace with active cursor in current step row.
- **Visible Controls:**
  - Active input field `#proc-answer-input` or stepwise row inputs.
  - Submit CTA `#proc-submit-btn` (enabled once input is non-empty).
  - Hint CTA `#proc-hint-btn` (`💡 Request Hint (H)`).
  - Mode Switcher (if applicable).
- **Hidden Controls:**
  - Result panel, mistake classification panel, Anki ease rating buttons.
- **Primary CTA:** Submit completed answer (`Enter` / `#proc-submit-btn`).
- **Secondary CTA:** Request progressive hint (`H` / `#proc-hint-btn`).
- **Keyboard Behavior:**
  - `Enter` / `Ctrl+Enter`: Submits answer via `handleQuickSubmit()` or `handleStepwiseSubmit()`.
  - `H` / `?`: Triggers `requestHint()`.
  - `1`..`4` / `A`..`D`: In MCQ mode, selects option index 0..3.
  - `ArrowUp` / `ArrowDown` / `ArrowLeft` / `ArrowRight`: Moves focus across options with roving tabindex.
  - `Space` outside text input: Submits response; **strictly intercepted** (`e.preventDefault()`, `e.stopPropagation()`) to prevent Anki card flip.
- **Native Anki Controls:** Completely decoupled from review navigation. Native `_getTypedAnswer()` intercepted to delegate to `globalThis.anki.procedural.handleNativeShowAnswer()`.
- **State Transitions:**
  - $\rightarrow$ `submitting`: Triggered when student presses `Enter` or clicks `#proc-submit-btn`.
  - $\rightarrow$ `hint`: Triggered when student clicks `#proc-hint-btn` or presses `H` / `?`.
  - $\rightarrow$ `mcq_selected`: In MCQ mode, triggered when an option card is clicked or selected via hotkey.
- **Backend IPC Events:** None during active typing (minimizes IPC thrashing).
- **Learner-Visible Data vs Hidden Engine Data:**
  - *Learner-Visible:* Typed characters, parsed unit preview pill, selected option state.
  - *Hidden Engine Data:* Millisecond stopwatch accumulation (`timeTakenMs`), keystroke latency histogram, intermediate AST validation tokens.

---

### 3.4 `hint`

- **State Identification:** `ProceduralUIState = "hint"` (`ts/reviewer/procedural.ts:630-680`, `ts/reviewer/components/stepwise_container.ts:600-660`)
- **Cognitive Purpose & Learner Goal:** Scaffolded metacognitive support. Provides progressive guidance to help the student overcome an impasse without giving away the final solution.
- **Visible Content:**
  - Expandable hint card `#proc-hint-box.proc-hint-box` mounted directly below the problem prompt.
  - 3-Tier Progressive Content:
    - **Tier 1 (Principle):** Governing physical law, mathematical theorem, or reasoning strategy (e.g., *"Apply Work-Energy Theorem: $W_{\text{net}} = \Delta K$."*).
    - **Tier 2 (Operation):** Specific algebraic setup or formula substitution (e.g., *"Substitute $F \cdot d = \frac{1}{2} m v_f^2 - \frac{1}{2} m v_i^2$."*).
    - **Tier 3 (Intermediate Relation):** Partially simplified numerical or symbolic relation (e.g., *"Solve for $v_f = \sqrt{2 \cdot 50 \cdot 4 / 2} = 14.14\,\text{m/s}$."*).
- **Visible Controls:**
  - `[ Next Hint Level ]` button (if higher hint tier exists).
  - `[ Resume Solving ]` button (`#proc-resume-btn`).
  - Active input field (remains visible below hint card).
- **Hidden Controls:**
  - Solution derivation, mistake classification panel, Anki ease buttons.
- **Primary CTA:** Read hint tier and return to active solving (`Resume Solving` / `Esc` / `Enter`).
- **Secondary CTA:** Request next progressive hint level (Tier 1 $\rightarrow$ Tier 2 $\rightarrow$ Tier 3).
- **Keyboard Behavior:**
  - `Esc` or `Enter`: Closes hint card and refocuses active answer input.
  - `H` / `?`: Advances to next available hint tier.
- **Native Anki Controls:** Unchanged (Ease buttons hidden).
- **State Transitions:**
  - $\rightarrow$ `solving`: Triggered when hint is dismissed or student types in the answer input.
- **Backend IPC Events:**
  - Emits `bridgeCommand("procedural_hint:<json>")` with payload:
    ```json
    {
      "instance_id": "inst-phys-work-101",
      "hint_level": 1,
      "step_id": "step_1_work_energy",
      "elapsed_ms": 18450
    }
    ```
- **Learner-Visible Data vs Hidden Engine Data:**
  - *Learner-Visible:* Tier badge (`💡 Hint Level 1 of 3: Core Principle`), pedagogical text.
  - *Hidden Engine Data:* Increments `hintsUsed` counter; marks attempt independence level as `LightSupport` or `SignificantSupport` in `MasteryEvidence`.

---

### 3.5 `submitting`

- **State Identification:** `ProceduralUIState = "submitting"` (`ts/reviewer/procedural.ts:788-842`)
- **Cognitive Purpose & Learner Goal:** Evaluation state. Performs instant local AST normalization, dimensional unit conversion, linear root equivalence, or MCQ option comparison.
- **Visible Content:**
  - Inputs temporarily locked with `.proc-input-locked` visual style.
  - Subtle inline evaluation status indicator (e.g., spinner or checking indicator).
- **Visible Controls:** Inputs and buttons are disabled (`disabled = true`) to prevent double submission.
- **Hidden Controls:** Result panel, mistake classification panel.
- **Primary CTA:** None (instantaneous evaluation < 50ms).
- **Secondary CTA:** None.
- **Keyboard Behavior:** All input keystrokes temporarily ignored.
- **Native Anki Controls:** Unchanged.
- **State Transitions:**
  - $\rightarrow$ `feedback`: If `outcome.isCorrect === true`.
  - $\rightarrow$ `mistake_classification`: If `outcome.isCorrect === false`.
- **Backend IPC Events:** Evaluated locally on frontend; packages telemetry payload for subsequent bridge dispatch.
- **Learner-Visible Data vs Hidden Engine Data:**
  - *Learner-Visible:* Input locked state.
  - *Hidden Engine Data:* Calculates exact score (`1.0` or `0.0`), computes `timeTakenMs`, derives `speedQuadrant` (`fluency_strength`, `speed_opportunity`, `strategy_trap`, `concept_setup`).

---

### 3.6 `mistake_classification`

- **State Identification:** `ProceduralUIState = "mistake_classification"` (`ts/reviewer/procedural.ts:940-1010`, `ts/reviewer/components/mistake_footer.ts:80-160`)
- **Cognitive Purpose & Learner Goal:** Metacognitive error reflection. Grounded in Cognitive Load Theory and the Hypercorrection Effect (Metcalfe 2017), this state prompts the learner to diagnose the root cause of their error before receiving the solution.
- **Visible Content:**
  - Error Notification Banner `#proc-error-banner`: Concise outcome message (e.g., *"❌ Incorrect: Result does not satisfy equation (Expected: 7, Submitted: 9)"*).
  - Compact Reflection Strip `#proc-mistake-panel.proc-mistake-panel`:
    - Heading: *"Classify error (1-4) to reflect and optimize spaced repetition:"*
    - 4 Discrete Categorization Buttons (`.proc-mistake-btn` / `.proc-mistake-card`):
      1. `[1 Silly]` (`silly_mistake`: Arithmetic, sign slip, or misread values).
      2. `[2 Pattern]` (`pattern_not_recognized`: Failed to identify schema/structure).
      3. `[3 Concept]` (`formula_or_concept_misapplied`: Applied wrong formula/law).
      4. `[4 Unknown]` (`concept_not_known`: Missing fundamental prerequisite).
- **Visible Controls:** 4 mistake classification buttons (`[1 Silly]`, `[2 Pattern]`, `[3 Concept]`, `[4 Unknown]`).
- **Hidden Controls:**
  - `#proc-next-btn` / `Next Problem` (Strictly hidden until classified).
  - Solution derivation trace (Partially collapsed or hidden until classified).
  - Native Anki ease rating buttons (`1`..`4`) (Suppressed).
- **Primary CTA:** Click one of the 4 classification buttons or press keys `1`, `2`, `3`, or `4`.
- **Secondary CTA:** None.
- **Keyboard Behavior (Anti-Bypass Lock):**
  - **Space and Enter are strictly trapped:** Handlers execute `e.preventDefault()` and `e.stopPropagation()`. The user **cannot** skip past this reflection gate.
  - Number keys `1`, `2`, `3`, `4`: Select the respective mistake category immediately and trigger transition.
- **Native Anki Controls:** Completely suppressed; no card flip permitted.
- **State Transitions:**
  - $\rightarrow$ `feedback`: Triggered 150ms after a category is selected (allows visual button highlight feedback).
- **Backend IPC Events:**
  - Emits `bridgeCommand("procedural_mistake:<json>")` with payload:
    ```json
    {
      "instance_id": "inst-math-quad-502",
      "family_id": "family.math.algebra.quadratic",
      "mistake_type": "formula_or_concept_misapplied"
    }
    ```
- **Learner-Visible Data vs Hidden Engine Data:**
  - *Learner-Visible:* Submitted vs Expected value summary, 4 mistake buttons with active selection highlight.
  - *Hidden Engine Data:* Updates `DomainEvidence` error classification weights (`is_execution_error()` vs `is_conceptual_error()`), calibrates FSRS ease penalty, and queues targeted remediation.

---

### 3.7 `feedback`

- **State Identification:** `ProceduralUIState = "feedback"` (`ts/reviewer/procedural.ts:1015-1190`)
- **Cognitive Purpose & Learner Goal:** Comprehensive outcome review and consolidation. The student reviews the step-by-step canonical derivation, verifies their cognitive model, and transitions to the next problem or remedial practice.
- **Visible Content:**
  - Clean Result Banner `.proc-result-header`:
    - Correct: `✓ Correct Solution` (Green banner).
    - Incorrect: `❌ Canonical Derivation & Solution` (Red/Amber banner).
  - Deduplicated Expected Answer Row `.proc-expected-row`: Formatted MathJax final value (rendered exactly once).
  - Step-by-Step Derivation Trace `.proc-derivation-trace`: Structured mathematical / logical derivation steps.
  - Performance & Speed Quadrant Badge `.proc-speed-quadrant`:
    - `⚡ Fluency Strength` (Green: Fast & Correct).
    - `🎯 Conceptual Accuracy` (Blue: Methodical & Correct).
    - `⚠️ Speed Trap / Calculation Slip` (Orange: Fast & Incorrect).
    - `📚 Concept Review Recommended` (Red: Slow & Incorrect).
- **Visible Controls:**
  - Primary Action: `#proc-next-btn` (`Next Problem (Space / Enter)`).
  - Remediation Action (if incorrect): `#proc-try-similar-btn` (`Try Similar Problem`) or `#proc-practice-prereq-btn` (`Practice Prerequisite`).
- **Hidden Controls:**
  - Input fields and submission buttons (Removed/hidden).
  - Mistake classification strip (Hidden after selection).
  - Native Anki bottom ease buttons (Hidden/suppressed on procedural cards to enforce the One-Interaction-Surface invariant).
- **Primary CTA:** Click `#proc-next-btn` or press `Space` / `Enter`.
- **Secondary CTA:** Click `#proc-try-similar-btn` to practice an immediate seeded variant.
- **Keyboard Behavior:**
  - `Space` or `Enter`: Advances to next scheduled card via `handleNext()`.
  - Keys `1`, `2`, `3`, `4`: Optional direct ease rating override (`1: Again`, `2: Hard`, `3: Good`, `4: Easy`), dispatching `procedural_answer:<ease>`.
- **Native Anki Controls:**
  - Synchronizes Python host state via `self.state = "answer"`.
  - Bottom bar displays single `#proc-next-btn` inside the webview; duplicate bottom ease buttons remain hidden.
- **State Transitions:**
  - $\rightarrow$ `next`: Triggered when student presses `Space`, `Enter`, or clicks `#proc-next-btn`.
  - $\rightarrow$ `worked_example`: Triggered if student clicks `#proc-try-similar-btn`.
- **Backend IPC Events:**
  - Updates next card state via `globalThis.anki.mutateNextCardStates(...)`.
  - Emits `bridgeCommand("procedural_attempt:<json>")` with complete `AttemptResultPayload`:
    ```json
    {
      "instanceId": "inst-phys-kinematics-102",
      "familyId": "family.phys.kinematics.1d_fall",
      "schemaId": "schema.phys.kinematics.v_squared",
      "skillId": "skill.phys.kinematics.free_fall",
      "domain": "physics",
      "answer": "30 m/s",
      "mode": "quick",
      "steps": [],
      "hintsUsed": 0,
      "timeTakenMs": 8420,
      "targetTimeMs": 15000,
      "isCorrect": true,
      "score": 1.0,
      "speedQuadrant": "fluency_strength",
      "mistakeType": null
    }
    ```
- **Learner-Visible Data vs Hidden Engine Data:**
  - *Learner-Visible:* Correctness banner, canonical solution derivation, elapsed time, speed quadrant badge, Next button.
  - *Hidden Engine Data:* Full SQLite transaction queue for `procedural.db`, Bayesian Knowledge Tracing / Elo skill rating updates, FSRS memory stability parameters ($S, D, R$).

---

### 3.8 `next`

- **State Identification:** `ProceduralUIState = "next"` (`ts/reviewer/procedural.ts:1221-1230`)
- **Cognitive Purpose & Learner Goal:** Lifecycle handover state. Safely completes current card lifecycle, triggers FSRS review logging in Rust backend, and loads the next queued learning object.
- **Visible Content:** Smooth transition screen / container fade out.
- **Visible Controls:** None.
- **Hidden Controls:** All procedural controls unmounted.
- **Primary CTA:** None (automated handover).
- **Secondary CTA:** None.
- **Keyboard Behavior:** Restored to native Anki reviewer handlers.
- **Native Anki Controls:** Invokes `Reviewer._answerCard(ease)` with calibrated ease (`1: Again` for incorrect attempts, `3: Good` for on-target correct, `4: Easy` for fast fluency strength).
- **State Transitions:**
  - $\rightarrow$ `teardown`: Automatic cleanup.
  - $\rightarrow$ `loading`: For the subsequent scheduled procedural card.
- **Backend IPC Events:**
  - Emits `bridgeCommand("procedural_answer:<ease>")` where `<ease>` $\in \{1, 2, 3, 4\}$.
- **Learner-Visible Data vs Hidden Engine Data:**
  - *Learner-Visible:* Loading of next card.
  - *Hidden Engine Data:* Atomic database commit in `procedural.db`, `revlog` entry write, spaced repetition interval scheduling.

---

### 3.9 Object-Specific State: `step_answering`

- **State Identification:** Active Stepwise Derivation Mode (`ts/reviewer/components/stepwise_container.ts:250-450`)
- **Applicable Learning Object:** `stepwise`, `problem` (in stepwise mode)
- **Cognitive Purpose & Learner Goal:** Cognitive Tutor Inner Loop. Enables the student to enter multi-step mathematical/physical derivations row by row, receiving validation on each intermediate step before proceeding.
- **Visible Content:**
  - Dynamic Step Rows `.proc-step-row`:
    - Step index badge (e.g., `Step 1:`, `Step 2:`).
    - Step description / sub-goal prompt (e.g., *"Isolate variable term: Subtract 15 from both sides"*).
    - Step input field `.proc-step-input` containing LaTeX / algebraic formula.
    - Inline step validation badge:
      - `✓ Valid` (Green badge).
      - `⚠️ Partially Valid` (Amber badge: Valid reasoning derived from prior incorrect step).
      - `❌ Invalid` (Red badge: Mathematical syntax error or incorrect derivation).
- **Visible Controls:**
  - `[ + Add Step ]` button (`#proc-add-step-btn`).
  - `[ 💡 Request Hint ]` button (`#proc-hint-btn`).
  - `[ ↺ Reset Workspace ]` button (`#proc-reset-btn`).
  - `[ Check Solution ]` primary CTA button (`#proc-check-steps-btn`).
- **Hidden Controls:** Quick solve single input field `#proc-answer-input`.
- **Primary CTA:** Click `[ Check Solution ]` or press `Ctrl+Enter`.
- **Secondary CTA:** Click `[ + Add Step ]` to insert an additional intermediate derivation row.
- **Keyboard Behavior:**
  - `Enter` in step input: Validates current step and focuses next step row.
  - `Ctrl+Enter`: Triggers full derivation evaluation (`handleStepwiseSubmit()`).
  - `Tab` / `Shift+Tab`: Navigates between step rows and controls.
- **Native Anki Controls:** Ease buttons suppressed.
- **State Transitions:**
  - $\rightarrow$ `submitting`: On `Check Solution` trigger.
  - $\rightarrow$ `hint`: On `Request Hint` trigger.
- **Backend IPC Events:**
  - Emits `bridgeCommand("procedural_validate_steps:<json>")` on intermediate step check:
    ```json
    {
      "instance_id": "inst-math-linear-301",
      "step_index": 0,
      "step_id": "step_1_subtract",
      "expression": "5x = 30",
      "is_valid": true,
      "is_downstream_consistent": false,
      "first_error_step": null
    }
    ```
- **Learner-Visible Data vs Hidden Engine Data:**
  - *Learner-Visible:* Step derivation rows, validation badges, error diagnostic hints.
  - *Hidden Engine Data:* Rust `StepValidator` AST equivalence tree, intermediate expression canonical normalization.

---

### 3.10 Object-Specific State: `mcq_selected`

- **State Identification:** Active Option Selection State (`ts/reviewer/components/mcq_container.ts:180-260`)
- **Applicable Learning Object:** `mcq`, `concept_check`, `strategy_drill`
- **Cognitive Purpose & Learner Goal:** Immediate visual confirmation of selected choice before final submission, providing visual tactile feedback.
- **Visible Content:**
  - 4 Discrete Option Cards `.proc-option-item`:
    - Option Key Badge `.proc-option-key` (`A`, `B`, `C`, `D` or `1`, `2`, `3`, `4`).
    - Option Label `.proc-option-label` with formatted MathJax / text.
    - Selected Option styling: Highlighted background, 2px solid primary accent border (`var(--proc-primary-color)`), elevated box shadow.
- **Visible Controls:**
  - 4 Option Cards `.proc-option-item` (clickable, ARIA `role="radio"`, `aria-checked="true"` for selected item).
  - Submit CTA `#proc-submit-btn` (in "mock" mode) or automatic evaluation trigger (in "practice" mode).
- **Hidden Controls:** Free text input fields (Strictly forbidden).
- **Primary CTA:** Press `Enter` or click selected option to confirm and submit.
- **Secondary CTA:** Click a different option or use arrow keys to change selection.
- **Keyboard Behavior:**
  - Keys `1`..`4` / `A`..`D`: Instantly updates selection to corresponding option.
  - `ArrowUp` / `ArrowDown`: Moves selection up/down.
  - `Enter` / `Space`: Confirms selection and triggers evaluation.
- **Native Anki Controls:** Suppressed.
- **State Transitions:**
  - $\rightarrow$ `submitting`: Triggered when selection is confirmed.
- **Backend IPC Events:** None until submission.
- **Learner-Visible Data vs Hidden Engine Data:**
  - *Learner-Visible:* Highlighted card border and checked radio status.
  - *Hidden Engine Data:* Selected option ID, selection change timestamp, hesitation latency.

---

### 3.11 Object-Specific State: `worked_step_reveal`

- **State Identification:** Worked Example Trace Review (`ts/reviewer/procedural.ts:1192-1200`)
- **Applicable Learning Object:** `worked_example`
- **Cognitive Purpose & Learner Goal:** Low-cognitive-load expert modeling. Guides the student through a canonical expert solution after repeated failure loops, focusing on key decision points and problem schema.
- **Visible Content:**
  - Context & Setup Card `.proc-worked-context`: Problem statement and initial givens.
  - Highlighted Decision Point Card `.proc-decision-box`: Crucial conceptual turning point (e.g., *"⭐ Key Decision: Base of percentage calculation must be the actual weight dispensed (900g), not the marked 1kg."*).
  - Step-by-Step Expert Trace `.proc-worked-steps`: Sequentially numbered derivation steps.
  - Method Rationale & Common Pitfalls `.proc-rationale-box`: Why this method was chosen and common traps to avoid.
- **Visible Controls:**
  - Primary Action Gate: `[ ✔ I Have Reviewed and Understood This Solution ]` (`#proc-try-similar-btn`).
- **Hidden Controls:**
  - Solving input fields, MCQ options, submission buttons (Completely hidden).
- **Primary CTA:** Click `[ ✔ I Have Reviewed and Understood This Solution ]` or press `Enter` / `Space`.
- **Secondary CTA:** None.
- **Keyboard Behavior:** `Enter` or `Space` triggers acknowledgement and generates a fresh practice variant.
- **Native Anki Controls:** Suppressed.
- **State Transitions:**
  - $\rightarrow$ `ready`: Loads newly seeded practice variant (`TransferRetry`).
- **Backend IPC Events:**
  - Emits `bridgeCommand("procedural_try_similar:<json>")` with payload:
    ```json
    {
      "instance_id": "inst-math-shopkeeper-801",
      "family_id": "family.math.commercial.dishonest_shopkeeper"
    }
    ```
- **Learner-Visible Data vs Hidden Engine Data:**
  - *Learner-Visible:* Expert solution trace, key decision point, acknowledgement CTA.
  - *Hidden Engine Data:* Logs worked example exposure in `remediation_recurrence`; zero mastery score credited for passive viewing.

---

### 3.12 Object-Specific State: `diagnostic_summary`

- **State Identification:** ConceptCheck / StrategyDrill Diagnostic Feedback (`ts/reviewer/components/mcq_container.ts:320-390`)
- **Applicable Learning Object:** `concept_check`, `strategy_drill`
- **Cognitive Purpose & Learner Goal:** Targeted diagnostic feedback on conceptual misconceptions or sub-optimal problem-solving strategies.
- **Visible Content:**
  - Option cards annotated with individual correctness:
    - Selected Option: Highlighted Green (if correct) or Red (if distractor).
    - Canonical Correct Option: Highlighted Green with `✓ Correct` badge.
  - Inline Diagnostic Callout Box `.proc-option-feedback`:
    - *ConceptCheck Example:* *"⚠️ Additive Fallacy: The second 10% increase acts on the already-increased base (1.10), resulting in $1.10 \times 1.10 = 1.21$ (+21%), not +20%."*
    - *StrategyDrill Example:* *"⭐ Optimal Strategy: Direct alligation cross rule produces the 3 : 2 ratio in 1 mental step vs 4 steps in algebraic system."*
- **Visible Controls:**
  - `#proc-next-btn` (`Next Problem (Space / Enter)`).
- **Hidden Controls:** Option selection is locked; inputs disabled.
- **Primary CTA:** Press `Space` / `Enter` to advance.
- **Secondary CTA:** None.
- **Keyboard Behavior:** `Space` or `Enter` advances to next card.
- **Native Anki Controls:** Suppressed.
- **State Transitions:**
  - $\rightarrow$ `next`: On `Space` / `Enter`.
- **Backend IPC Events:** Emits `procedural_attempt` with misconception tag.
- **Learner-Visible Data vs Hidden Engine Data:**
  - *Learner-Visible:* Diagnostic explanation specific to chosen distractor.
  - *Hidden Engine Data:* Updates misconception taxonomy ledger in `procedural.db`.

---

### 3.13 `error`

- **State Identification:** `ProceduralUIState = "error"` (`ts/reviewer/procedural.ts:285-290`)
- **Cognitive Purpose & Learner Goal:** Fault-tolerant error boundary. Prevents blank white screens or Anki reviewer crashes if an anchor payload is malformed or unparseable.
- **Visible Content:**
  - Structured Error Banner `.proc-error-boundary`:
    - Title: *"⚠️ StudyLab Template Error"*
    - Message: Human-readable error description (e.g., *"Unable to parse ProceduralPayload. Field 'options' was missing or invalid."*).
- **Visible Controls:**
  - `[ Skip Card (Space) ]` button.
  - `[ Copy Diagnostics ]` button.
- **Hidden Controls:** All problem-solving containers.
- **Primary CTA:** Click `[ Skip Card ]` or press `Space` to advance safely.
- **Secondary CTA:** Click `[ Copy Diagnostics ]` to paste debug info.
- **Keyboard Behavior:** `Space` or `Enter` bypasses card and logs error.
- **Native Anki Controls:** Standard Anki shortcuts restored.
- **State Transitions:**
  - $\rightarrow$ `teardown`: On skip.
- **Backend IPC Events:** Logs error message to Python console and stderr.
- **Learner-Visible Data vs Hidden Engine Data:**
  - *Learner-Visible:* Safe error explanation, Skip CTA.
  - *Hidden Engine Data:* Stack trace, malformed JSON snippet.

---

### 3.14 `teardown`

- **State Identification:** `ProceduralUIState = "teardown"` (`ts/reviewer/procedural.ts:1239-1278`)
- **Cognitive Purpose & Learner Goal:** Terminal garbage collection and listener detachment state. Guarantees 0% memory leaks, 0% event listener leaks, and clean DOM restoration when transitioning to standard Anki cards.
- **Visible Content:** DOM container detached or reset.
- **Visible Controls:** None.
- **Hidden Controls:** All.
- **Primary CTA:** None.
- **Secondary CTA:** None.
- **Keyboard Behavior:** 100% restored to native Anki reviewer handlers.
- **Native Anki Controls:** Fully restored.
- **State Transitions:** Terminal state.
- **Backend IPC Events:** Python invokes `Reviewer._destroyActive()`.
- **Learner-Visible Data vs Hidden Engine Data:**
  - *Learner-Visible:* Clean transition.
  - *Hidden Engine Data:* All `disposables` executed: `window.removeEventListener`, `MutationObserver.disconnect()`, timer intervals cleared, global reviewer references nullified.

---

## 4. State Matrix Summary Table

| State Name | Primary Visual Content | Visible Controls | Primary CTA | Keyboard Primary | Space/Enter Trapped? | Native Anki State | Backend IPC Event |
|---|---|---|---|---|---|---|---|
| **`loading`** | Spinner / skeleton outline | None | None (Auto) | Suppressed | Yes (Suppressed) | Count only | None |
| **`ready`** | Prompt, armed inputs, header | Input field, Submit, Hint | Read & Focus | Hotkeys armed | Yes (No flip) | Count only | None |
| **`solving`** | Prompt, active input, preview pill | Input field, Submit, Hint | Submit Answer | `Enter` (Submit) | Yes (Submits) | Count only | None |
| **`hint`** | 3-tier progressive hint card | Next Hint, Resume Solving | Resume Solving | `Esc` / `Enter` (Resume) | Yes (Closes hint) | Count only | `procedural_hint` |
| **`submitting`** | Locked inputs, eval indicator | None (Disabled) | None (Auto) | Suppressed | Yes (Suppressed) | Count only | None |
| **`mistake_classification`** | Error banner, 4 reflection buttons | 4 Category Buttons | Classify Error | `1`..`4` (Select) | **STRICTLY TRAPPED** | Count only | `procedural_mistake` |
| **`feedback`** | Result banner, canonical derivation | Next Problem, Try Similar | Next Problem | `Space` / `Enter` (Next) | No (Advances) | `ans` sync | `procedural_attempt` |
| **`next`** | Transition fade | None | None (Auto) | Native resumed | No | Anki answering | `procedural_answer:<ease>` |
| **`step_answering`** | Derivation rows, step badges | Add Step, Reset, Check | Check Solution | `Ctrl+Enter` (Check) | Yes (Submits steps)| Count only | `procedural_validate_steps` |
| **`mcq_selected`** | 4 radio cards, active border | Option cards, Submit | Confirm Option | `1`..`4` / `Enter` | Yes (Submits) | Count only | None |
| **`worked_step_reveal`** | Canonical trace, decision box | Understand & Try Similar | Try Similar | `Enter` / `Space` | Yes (Triggers variant) | Count only | `procedural_try_similar` |
| **`diagnostic_summary`** | Distractor feedback callout | Next Problem | Next Problem | `Space` / `Enter` (Next) | No (Advances) | Count only | `procedural_attempt` |
| **`error`** | Error boundary warning banner | Skip Card, Copy Debug | Skip Card | `Space` (Skip) | No | Restored | Stderr log |
| **`teardown`** | Container unmounted | None | None | 100% Native | No | 100% Native | Python GC hook |

---

## 5. Learner-Visible Data vs Hidden Engine Data Ledger

| Data Attribute | Learning Purpose / Semantic Meaning | Learner Visibility Level | Reason for Display / Suppression |
|---|---|---|---|
| **Problem Prompt & Formulas** | Core problem statement and givens | **ALWAYS VISIBLE** | Primary cognitive hero of the workspace. |
| **Domain / Chapter / Topic** | High-level curricular orientation | **VISIBLE (Header)** | Provides context without cluttering solving space. |
| **Live Unit Preview Pill** | Instant feedback on parsed magnitude & unit | **VISIBLE (Typing)** | Prevents syntax misunderstandings; confirms unit interpretation. |
| **Progressive Hint Text** | Scaffolding for impasse resolution | **VISIBLE (On Request)** | Scaffolded 3-tier disclosure aids metacognition. |
| **Mistake Classification Buttons** | Metacognitive error attribution | **VISIBLE (On Error)** | Required for reflection and FSRS calibration. |
| **Canonical Solution Derivation** | Expert mathematical & logical trace | **VISIBLE (Feedback)** | Enables error correction and cognitive modeling. |
| **Speed Quadrant Badge** | Performance classification (Speed vs Accuracy) | **VISIBLE (Feedback)** | Positive pedagogical framing of speed and fluency. |
| **Millisecond Stopwatch (`timeTakenMs`)** | Precise latency tracking | **HIDDEN (Solving) / SUBDUED (Feedback)** | Constant ticking causes anxiety; displayed calmly post-submission. |
| **Target Response Time (`targetTimeMs`)** | Backend benchmark for fluency quadrant | **HIDDEN ENGINE DATA** | Used for computation; displaying it induces unnecessary time pressure. |
| **Schema ID / Family ID** | Internal procedural generator identifiers | **HIDDEN ENGINE DATA** | Zero pedagogical value; raw developer leak if shown. |
| **Bayesian Knowledge Tracing ($P(L)$)** | Mastered skill probability vector | **HIDDEN ENGINE DATA** | Stored in `procedural.db`; aggregated in background analytics. |
| **FSRS Parameters ($S, D, R$)** | Memory stability, difficulty, retrievability | **HIDDEN ENGINE DATA** | Handled natively by Anki's backend scheduler. |
| **Step AST Equivalence Tokens** | Normalized CAS representation of equations | **HIDDEN ENGINE DATA** | Used by `StepValidator`; learner only sees rendered LaTeX. |

---

## 6. Zero-Fallback & Anti-Bypass Invariant Verification

1. **Zero Text Input Fallback:** When `object_type` is `"mcq" | "concept_check" | "strategy_drill" | "worked_example"`, the DOM selector `#proc-answer-input` is strictly null and `#proc-quick-container` is completely absent. Any attempt to mount a generic text box on these modalities is a contract violation.
2. **Anti-Bypass Reflection Lock:** In `mistake_classification`, the event listener on `window` intercepts `keydown` for `Space`, `Enter`, `NumpadEnter`, `Tab`, and `Arrow` keys. If no mistake category is active, `e.preventDefault()` and `e.stopPropagation()` are called. Skipping to the next card or revealing answers without classifying is impossible.
3. **One-Interaction-Surface:** Native ease buttons in `qt/aqt/reviewer.py` are hidden for procedural cards via `_showAnswerButton()` and suppressed ease bar. The single `#proc-next-btn` within the card controls card advancement.
