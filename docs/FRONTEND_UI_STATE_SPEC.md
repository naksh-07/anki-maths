# StudyLab Frontend UI State Specification

**Document Version:** 1.1.0 (Reconciled with STUDYLAB_UI_COMPOSITION_CONTRACT.md)  
**Target Subsystem:** TypeScript Reviewer Frontend (`ts/reviewer/`), Reviewer Webview Templates (`rslib/procedural/src/reviewer/`), and Desktop Host Bridge (`qt/aqt/reviewer.py`)  
**Status:** AUTHORITATIVE CANONICAL SPECIFICATION  
**Integrity Mode:** 100% Grounded in Executable Source Code, Live UI Audits, and Educational Invariants  
**Authoritative Reference:** `docs/STUDYLAB_UI_COMPOSITION_CONTRACT.md`, `PROJECT.md`

---

## 1. Executive Summary & Core Architectural Invariants

The StudyLab Reviewer provides an interactive, multi-domain cognitive problem-solving environment embedded within Anki. It replaces Anki's standard binary declarative recall flip cycle with an **explicit, state-driven procedural learning machine**.

```text
ANKI STANDARD REVIEW:
    Question → Flip / Reveal → Grade

STUDYLAB PROCEDURAL REVIEW:
    Problem → Interactive Work → Evaluate → Diagnose (on error) → Next
```

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
1. **Visual Hero Invariant:** The problem statement (`#proc-prompt`) is the primary visual hero of the workspace at all times.
2. **Semantic Modality Identity:** If a learning object is an MCQ, ConceptCheck, StrategyDrill, or WorkedExample, it **MUST NEVER** render a free-text input field. Semantic modality matches UI interaction surface unconditionally.
3. **One-Interaction-Surface:** At any given state, there is exactly **one** primary call-to-action. Duplicate controls (e.g., in-card submit plus native Anki ease buttons) are strictly forbidden.
4. **Anti-Bypass Metacognitive Reflection Gate:** Upon submitting an incorrect answer, advancing to the next card without classifying the error category (`1 Silly Slip`, `2 Pattern Missed`, `3 Concept Gap`, `4 Prereq Unknown`) is **physically and programmatically impossible**. `Space` and `Enter` keys are trapped until classification occurs.
5. **Deferred Solution Reveal (`ANTI-08`):** `#proc-solution-container` remains strictly hidden throughout `mistake_classification` and is revealed only after 1–4 classification is selected.
6. **Open Canvas Subordination:** Eliminating heavy card wrappers, giant feedback tint boxes (`ANTI-01`), nested panels (`ANTI-07`), ticking stopwatches during solving (`ANTI-03`), and duplicate expected answers (`ANTI-02`).

---

## 2. Complete State Machine Lifecycle Graph

StudyLab models 11 core UI states and associated lifecycle phases:

```text
 ┌─────────────────┐
 │     loading     │ ◄── DOM Mount, Data Attribute Extraction & MathJax Queue
 └────────┬────────┘
          │ (DOM Ready & Typeset Complete)
          ▼
 ┌─────────────────┐
 │      ready      │ ◄── Clean Problem Hero, Modality Input Armed, Hotkeys Active
 └────────┬────────┘
          │ (First Focus / Keystroke)
          ▼
 ┌─────────────────────────────────────────────────────────────────────────────────┐
 │                                   SOLVING                                       │
 │                                                                                 │
 │   ┌───────────────────────┐  (H / ?)  ┌───────────────────────┐                 │
 │   │        solving        ├──────────►│         hint          │                 │
 │   │ (Input + Preview Pill)│◄──────────┤ (3-Tier Scaffolding)  │                 │
 │   └───────────┬───────────┘ (Resume)  └───────────────────────┘                 │
 │               │                                                                 │
 │   ┌───────────┴───────────────────────────────────────────────┐                 │
 │   │               Modality Workspace Specializations          │                 │
 │   │                                                           │                 │
 │   │  [Stepwise Workspace]         [MCQ / Choice]              │                 │
 │   │  ┌──────────────────────┐     ┌────────────────────────┐  │                 │
 │   │  │       stepwise       │     │      mcq_selected      │  │                 │
 │   │  │ (Row edit / CAS eval)│     │ (Radio 2px Accent Ring)│  │                 │
 │   │  └──────────────────────┘     └────────────────────────┘  │                 │
 │   │                                                           │                 │
 │   │  [Worked Example]             [Concept / Strategy]        │                 │
 │   │  ┌──────────────────────┐     ┌────────────────────────┐  │                 │
 │   │  │    worked_example    │     │concept_check / strategy│  │                 │
 │   │  │ (Trace + Ack Gate)   │     │ (Diagnostic Choices)   │  │                 │
 │   │  └──────────────────────┘     └────────────────────────┘  │                 │
 │   └───────────────────────────┬───────────────────────────────┘                 │
 └───────────────────────────────┼─────────────────────────────────────────────────┘
                                 │
                                 │ (Submit: Enter / Option Click / Ctrl+Enter)
                                 ▼
                         ┌───────────────┐
                         │  submitting   │ ◄── Inputs Locked (.proc-input-locked), Local AST Eval
                         └───┬───────┬───┘
                             │       │
            [Correct Attempt]│       │ [Incorrect Attempt]
                             │       ▼
                             │   ┌───────────────────────────┐
                             │   │       wrong_answer        │ ◄── Inline Failure Outcome
                             │   └───────────┬───────────────┘
                             │               │
                             │               ▼
                             │   ┌───────────────────────────┐
                             │   │  mistake_classification   │ ◄── Space/Enter TRAPPED
                             │   │  (1-4 Metacognitive Gate) │     Solution STRICTLY HIDDEN
                             │   └───────────┬───────────────┘
                             │               │ (Select Category: 1..4)
                             ▼               ▼
                     ┌───────────────────────────────┐
                     │           feedback            │ ◄── Deduplicated Derivation, Speed Pill,
                     │  (Canonical Outcome Screen)   │     Single "Next Problem ➔" CTA
                     └───────────────┬───────────────┘
                                     │
                     ┌───────────────┴───────────────┐
                     │ (Next Problem: Space / Enter) │ (Try Similar Variant: Alt+T)
                     ▼                               ▼
             ┌───────────────┐               ┌───────────────┐
             │     next      │               │worked_example │
             └───────┬───────┘               └───────┬───────┘
                     │ (FSRS Bridge Dispatch)        │ (Seed New Parameters)
                     ▼                               ▼
             ┌───────────────┐               ┌───────────────┐
             │   teardown    │               │     ready     │
             └───────────────┘               └───────────────┘
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
  - Native Anki bottom bar buttons (`#ansbut`, ease buttons `1`..`4`) (Suppressed).
- **Primary CTA:** None (automated transition).
- **Secondary CTA:** None.
- **Keyboard Behavior:** Completely suppressed. All keydown events return early with `e.preventDefault()`.
- **Native Anki Controls:** Top toolbar visible; bottom bar `#ansbut` replaced with remaining card count.
- **State Transitions:**
  - $\rightarrow$ `ready`: Triggered automatically when DOM elements are queried, `inline_contract` is parsed, and MathJax typeset promise resolves (Target: $<80\text{ms}$).
- **Backend IPC Events:** None.

---

### 3.2 `ready`

- **State Identification:** `ProceduralUIState = "ready"` (`ts/reviewer/procedural.ts:312-360`)
- **Cognitive Purpose & Learner Goal:** Clear, unobstructed presentation of the problem statement. The learner reads the problem prompt, reviews context/givens, and prepares their cognitive solving strategy.
- **Visible Content:**
  - Header `.proc-header`: Subject / Topic / Skill breadcrumb, authentic exam badge (e.g. `[ JEE Main 2024 ]`).
  - Problem Prompt `#proc-prompt`: High-contrast statement rendered cleanly in MathJax LaTeX.
  - Active Modality Workspace:
    - *Numerical:* `#proc-quick-container` with `#proc-answer-input` (`placeholder="Type final answer..."`).
    - *MCQ / ConceptCheck / StrategyDrill:* `#proc-mcq-container` with `.proc-option-group` (`role="radiogroup"`).
    - *Stepwise:* `#proc-stepwise-container` with initial empty step row.
    - *WorkedExample:* `#proc-worked-box` with canonical derivation trace.
- **Visible Controls:**
  - Modality-specific inputs: `#proc-answer-input` (focused), option cards `.proc-option-item`, or stepwise input.
  - Mode Switcher `.proc-mode-tabs` (only for `problem` modality with stepwise support: `Quick Solve` / `Step-by-Step`).
  - Scaffolding CTA: `#proc-hint-btn` (`💡 Request Hint`).
  - Submission CTA: `#proc-submit-btn` (`Submit Answer`) or `#proc-check-steps-btn` (`Check Solution`).
- **Hidden Controls:**
  - `#proc-mistake-panel` (Hidden).
  - `#proc-result-panel` (Hidden).
  - `#proc-hint-container` (Hidden).
  - Native Anki bottom ease buttons (`1`..`4`) and `#ansbut` (Hidden).
  - Ticking stopwatch displays (`ANTI-03`) (Strictly forbidden).
- **Primary CTA:** Focus input field or hover over first option card.
- **Secondary CTA:** Toggle between Quick Solve and Stepwise mode (`#proc-tab-stepwise`).
- **Keyboard Behavior:**
  - Focus automatically set to active input or first option item.
  - Hotkeys armed: `1`..`4` / `A`..`D` for MCQ selection; numeric and decimal keys for numerical inputs; `H` / `?` for hint request.
  - Spacebar and Enter do not trigger native card flip.
- **Native Anki Controls:** Bottom bar shows only deck progress count. Ease buttons hidden.
- **State Transitions:**
  - $\rightarrow$ `solving`: Triggered on first input keystroke, option click, or focus.
  - $\rightarrow$ `hint`: Triggered if user clicks `#proc-hint-btn` or presses `H` / `?`.
- **Backend IPC Events:** None.

---

### 3.3 `solving`

- **State Identification:** `ProceduralUIState = "solving"` (`ts/reviewer/procedural.ts:400-600`)
- **Cognitive Purpose & Learner Goal:** Active cognitive execution. The student formulates their derivation, calculates numerical values with dimensional units, or reasons through MCQ options.
- **Visible Content:**
  - Problem prompt `#proc-prompt`.
  - Live interaction workspace with active user input:
    - *Numerical Modality:* Active typing with dynamic unit preview pill `.proc-num-preview-pill` (e.g., `Parsed: 30 m/s`).
    - *MCQ Modality:* Active option focus with visual 2px accent outline.
    - *Stepwise Modality:* Multi-row algebraic derivation workspace with active cursor in current step row.
- **Visible Controls:**
  - Active input field `#proc-answer-input` or stepwise row inputs.
  - Submit CTA `#proc-submit-btn` (enabled once input is non-empty).
  - Hint CTA `#proc-hint-btn` (`💡 Request Hint (H)`).
  - Mode Switcher (if applicable).
- **Hidden Controls (`ANTI-03`):**
  - **No visible ticking stopwatch:** Timer runs silently in memory accumulator (`this.startTime`), preventing anxiety.
  - Result panel, mistake classification panel, Anki ease rating buttons.
- **Primary CTA:** Submit completed answer (`Enter` / `#proc-submit-btn`).
- **Secondary CTA:** Request progressive hint (`H` / `#proc-hint-btn`).
- **Keyboard Behavior:**
  - `Enter` / `Ctrl+Enter`: Submits answer via `handleQuickSubmit()` or `handleStepwiseSubmit()`.
  - `H` / `?`: Triggers `requestHint()`.
  - `1`..`4` / `A`..`D`: In MCQ mode, selects option index 0..3.
  - `ArrowUp` / `ArrowDown` / `ArrowLeft` / `ArrowRight`: Moves focus across options with roving tabindex.
  - `Space` outside text input: Submits response; **strictly intercepted** (`e.preventDefault()`, `e.stopPropagation()`) to prevent Anki card flip.
- **State Transitions:**
  - $\rightarrow$ `submitting`: Triggered when student presses `Enter` or clicks `#proc-submit-btn`.
  - $\rightarrow$ `hint`: Triggered when student clicks `#proc-hint-btn` or presses `H` / `?`.

---

### 3.4 `hint`

- **State Identification:** `ProceduralUIState = "hint"` (`ts/reviewer/procedural.ts:630-680`)
- **Cognitive Purpose & Learner Goal:** Scaffolded metacognitive support. Provides progressive guidance to help the student overcome an impasse without giving away the final solution.
- **Visible Content:**
  - Expandable hint card `#proc-hint-container.proc-hint-box` mounted directly below the problem prompt.
  - 3-Tier Progressive Content:
    - **Tier 1 (Principle):** Governing physical law, mathematical theorem, or reasoning strategy.
    - **Tier 2 (Operation):** Specific algebraic setup or formula substitution.
    - **Tier 3 (Intermediate Relation):** Partially simplified numerical or symbolic relation.
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
- **Backend IPC Events:** Emits `bridgeCommand("procedural_hint:<json>")`.

---

### 3.5 `submitting`

- **State Identification:** `ProceduralUIState = "submitting"` (`ts/reviewer/procedural.ts:788-842`)
- **Cognitive Purpose & Learner Goal:** Evaluation state. Performs instant local AST normalization, dimensional unit conversion, linear root equivalence, or MCQ option comparison.
- **Visible Content:**
  - Inputs temporarily locked with `.proc-input-locked` visual style.
  - Disabled submit button (`disabled = true`).
- **Visible Controls:** None (inputs locked).
- **Hidden Controls:** Result panel, mistake classification panel.
- **Primary CTA:** None (instantaneous evaluation < 50ms).
- **Keyboard Behavior:** All input keystrokes temporarily ignored/debounced.
- **State Transitions:**
  - $\rightarrow$ `feedback`: If `outcome.isCorrect === true`.
  - $\rightarrow$ `wrong_answer` $\rightarrow$ `mistake_classification`: If `outcome.isCorrect === false`.

---

### 3.6 `wrong_answer` / `mistake_classification`

- **State Identification:** `ProceduralUIState = "mistake_classification"` (`ts/reviewer/procedural.ts:940-1010`, `ts/reviewer/components/mistake_footer.ts:80-160`)
- **Cognitive Purpose & Learner Goal:** Metacognitive error reflection. Grounded in Cognitive Load Theory and the Hypercorrection Effect (Metcalfe 2017), this state prompts the learner to diagnose the root cause of their error before receiving the solution.
- **Visible Content:**
  - Inline error status row: `✗ Incorrect` with deduplicated comparison (`Your answer: 24 m/s`).
  - Compact Reflection Strip `#proc-mistake-panel.proc-mistake-panel`:
    - Heading: *"Classify error (1-4) to reflect and optimize spaced repetition:"*
    - 4 Discrete Categorization Buttons (`.proc-mistake-btn[data-key="1..4"]`):
      1. `[1 Silly Slip]` (`silly_mistake`: Arithmetic, sign, or unit slip).
      2. `[2 Pattern Missed]` (`pattern_not_recognized`: Failed to identify schema/structure).
      3. `[3 Concept Gap]` (`formula_or_concept_misapplied`: Applied wrong formula/law).
      4. `[4 Prereq Unknown]` (`concept_not_known`: Missing fundamental prerequisite).
- **Visible Controls:** Exactly the 4 mistake classification buttons.
- **Hidden Controls (`ANTI-08`):**
  - **`#proc-solution-container` is strictly hidden (`display: none !important`) until 1–4 classification occurs.**
  - `#proc-next-btn` / `Next Problem` (Strictly hidden until classified).
  - Native Anki ease rating buttons (`1`..`4`) and `#ansbut` (Suppressed).
  - Giant red full-bleed background banners (`ANTI-01`) (Strictly forbidden).
- **Primary CTA:** Click one of the 4 classification buttons or press keys `1`, `2`, `3`, or `4`.
- **Keyboard Behavior (Anti-Bypass Lock):**
  - **Space and Enter are strictly trapped:** Handlers execute `e.preventDefault()` and `e.stopPropagation()`. The user **cannot** skip past this reflection gate.
  - Number keys `1`, `2`, `3`, `4`: Select the respective mistake category immediately and trigger transition.
- **State Transitions:**
  - $\rightarrow$ `feedback`: Triggered 150ms after a category is selected (reveals canonical solution).
- **Backend IPC Events:** Emits `bridgeCommand("procedural_mistake:<json>")`.

---

### 3.7 `feedback`

- **State Identification:** `ProceduralUIState = "feedback"` (`ts/reviewer/procedural.ts:1015-1190`)
- **Cognitive Purpose & Learner Goal:** Comprehensive outcome review and consolidation. The student reviews the step-by-step canonical derivation, verifies their cognitive model, and transitions to the next problem or remedial practice.
- **Visible Content (Open Canvas):**
  - Outcome Header: `✓ Correct` or `✗ Incorrect (Categorized: Concept Gap)`.
  - Deduplicated Expected Answer Row (`ANTI-02`): `Your answer: 30 m/s · Correct answer: 30 m/s` rendered exactly once.
  - Step-by-Step LaTeX Derivation Trace on open canvas (`ANTI-07`).
  - Performance Speed Pill (`ANTI-04`): `⚡ Fast & Accurate · 8.4s` or `🎯 Accurate · 24.1s (Target: 20s)`.
- **Visible Controls:**
  - Primary Action: `#proc-next-btn` (`Next Problem ➔ (Space / Enter)`).
  - Remediation Action: `#proc-try-similar-btn` (`Try Similar Problem (Alt+T)`).
- **Hidden Controls:**
  - Input fields and submission buttons (Removed/hidden).
  - Mistake classification strip (Hidden after selection).
  - Native Anki bottom ease buttons (Hidden/suppressed on procedural cards).
  - Giant red/green container boxes (`ANTI-01`) (Strictly forbidden).
- **Primary CTA:** Click `#proc-next-btn` or press `Space` / `Enter`.
- **Secondary CTA:** Click `#proc-try-similar-btn` to practice an immediate seeded variant.
- **Keyboard Behavior:**
  - `Space` or `Enter`: Advances to next scheduled card via `handleNext()`.
  - Keys `1`, `2`, `3`, `4`: Optional direct ease rating override (`1: Again`, `2: Hard`, `3: Good`, `4: Easy`), dispatching `procedural_answer:<ease>`.
  - `Alt+T`: Triggers `Try Similar Problem`.
- **State Transitions:**
  - $\rightarrow$ `next`: Triggered when student presses `Space`, `Enter`, or clicks `#proc-next-btn`.
- **Backend IPC Events:** Emits `bridgeCommand("procedural_attempt:<json>")`.

---

### 3.8 `next`

- **State Identification:** `ProceduralUIState = "next"` (`ts/reviewer/procedural.ts:1221-1230`)
- **Cognitive Purpose & Learner Goal:** Lifecycle handover state. Safely completes current card lifecycle, triggers FSRS review logging in Rust backend, and loads the next queued learning object.
- **Visible Content:** Smooth transition container.
- **Visible Controls:** None (controls unmounting).
- **Primary CTA:** None (automated handover).
- **Dispatched IPC Events:** `bridgeCommand("procedural_answer:<ease>")` where `<ease>` $\in \{1, 2, 3, 4\}$.

---

### 3.9 Object-Specific States

#### `stepwise` (`step_answering`)
- **Modality:** `stepwise`, `problem` (with stepwise support).
- **Visible Content:** Numbered step rows, sub-goal prompts, step LaTeX inputs, inline validation status badges (`✔ Valid`, `❌ Invalid`, `⚠️ Consistent with Prior Error`).
- **Visible Controls:** `[ Check Solution ]` (`#proc-check-steps-btn`), `[ + Add Step ]`, `[ 💡 Request Hint ]`, `[ ↺ Reset ]`.
- **Keyboard:** `Enter` advances step row; `Ctrl+Enter` checks solution.

#### `concept_check` (`diagnostic_summary`)
- **Modality:** `concept_check`.
- **Visible Content:** Conceptual choices (role="radio") + immediate distractor misconception callout. Zero textboxes.

#### `strategy_drill`
- **Modality:** `strategy_drill`.
- **Visible Content:** Strategy candidate cards with step count/speed rating + optimality rationale. Zero textboxes.

#### `worked_example` (`worked_step_reveal`)
- **Modality:** `worked_example`.
- **Visible Content:** Open canvas expert trace + Key Decision Point + single `Try Similar` acknowledgment gate. Zero textboxes.

---

## 4. State Matrix Summary Table

| State Name | Primary Visual Content | Visible Controls | Primary CTA | Keyboard Primary | Space/Enter Trapped? | Native Anki State | Backend IPC Event |
|---|---|---|---|---|---|---|---|
| **`loading`** | Skeleton placeholder | None | None (Auto) | Suppressed | Yes (Suppressed) | Count only | None |
| **`ready`** | Prompt, armed inputs, header | Input field, Submit, Hint | Read & Focus | Hotkeys armed | Yes (No flip) | Count only | None |
| **`solving`** | Prompt, active input, preview pill | Input field, Submit, Hint | Submit Answer | `Enter` (Submit) | Yes (Submits) | Count only | None |
| **`hint`** | 3-tier progressive hint card | Next Hint, Resume Solving | Resume Solving | `Esc` / `Enter` (Resume) | Yes (Closes hint) | Count only | `procedural_hint` |
| **`submitting`** | Locked inputs, eval indicator | None (Disabled) | None (Auto) | Suppressed | Yes (Suppressed) | Count only | None |
| **`wrong_answer`** | Inline status `✗ Incorrect` | None (Transitioning) | Focus Reflection | Suppressed | Yes (Trapped) | Count only | None |
| **`mistake_classification`** | Error banner, 4 reflection buttons | 4 Category Buttons | Classify Error | `1`..`4` (Select) | **STRICTLY TRAPPED** | Count only | `procedural_mistake` |
| **`feedback`** | Result header, canonical derivation | Next Problem, Try Similar | Next Problem | `Space` / `Enter` (Next) | No (Advances) | `ans` sync | `procedural_attempt` |
| **`next`** | Transition fade | None | None (Auto) | Native resumed | No | Anki answering | `procedural_answer:<ease>` |
| **`stepwise`** | Derivation rows, step badges | Add Step, Reset, Check | Check Solution | `Ctrl+Enter` (Check) | Yes (Submits steps)| Count only | `procedural_validate_steps` |
| **`concept_check`** | Conceptual choices + feedback | Option cards, Next | Next Problem | `1`..`4` / `Space` | No | Count only | `procedural_attempt` |
| **`strategy_drill`** | Strategy cards + optimality | Strategy cards, Next | Next Problem | `1`..`4` / `Space` | No | Count only | `procedural_attempt` |
| **`worked_example`** | Canonical trace, decision box | Understand & Try Similar | Try Similar | `Enter` / `Space` | Yes (Triggers variant) | Count only | `procedural_try_similar` |
| **`teardown`** | Container unmounted | None | None | 100% Native | No | 100% Native | None |

---

## 5. Learner-Visible Data vs Hidden Engine Data Ledger

| Data Attribute | Learning Purpose / Semantic Meaning | Learner Visibility Level | Reason for Display / Suppression |
|---|---|---|---|
| **Problem Prompt & Formulas** | Core problem statement and givens | **ALWAYS VISIBLE** | Primary cognitive hero of the workspace. |
| **Domain / Topic / Skill** | Curricular orientation | **VISIBLE (Header)** | Provides context without cluttering solving space. |
| **Live Unit Preview Pill** | Instant feedback on parsed magnitude & unit | **VISIBLE (Typing)** | Prevents syntax misunderstandings; confirms unit interpretation. |
| **Progressive Hint Text** | Scaffolding for impasse resolution | **VISIBLE (On Request)** | Scaffolded 3-tier disclosure aids metacognition. |
| **Mistake Classification Buttons** | Metacognitive error attribution | **VISIBLE (On Error)** | Required for reflection and FSRS calibration. |
| **Canonical Solution Derivation** | Expert mathematical & logical trace | **VISIBLE (Feedback)** | Revealed post-reflection for cognitive modeling (`ANTI-08`). |
| **Speed Pill (`ANTI-04`)** | Performance classification (Speed vs Accuracy) | **VISIBLE (Feedback)** | Compact, muted pill format (`⚡ Fast & Accurate · 8.4s`). |
| **Millisecond Stopwatch (`ANTI-03`)** | Latency tracking | **HIDDEN (Solving) / SUBDUED (Feedback)** | Ticking stopwatch suppressed during solving; elapsed time shown post-submission. |
| **Target Response Time (`targetTimeMs`)** | Backend benchmark for speed pill | **HIDDEN ENGINE DATA** | Used for computation; displaying it induces unnecessary time pressure. |
| **Schema ID / Family ID (`ANTI-06`)** | Internal procedural generator identifiers | **HIDDEN ENGINE DATA** | Zero pedagogical value; raw developer leak if shown. |
| **FSRS Parameters ($S, D, R$)** | Memory stability, difficulty, retrievability | **HIDDEN ENGINE DATA** | Handled natively by Anki's backend scheduler. |
| **Step AST Equivalence Tokens** | Normalized CAS representation of equations | **HIDDEN ENGINE DATA** | Used by `StepValidator`; learner only sees rendered LaTeX. |

---

## 6. Zero-Fallback & Anti-Bypass Invariant Verification

1. **Zero Text Input Fallback:** When `object_type` is `"mcq" | "concept_check" | "strategy_drill" | "worked_example"`, `#proc-answer-input` is strictly absent.
2. **Anti-Bypass Reflection Lock:** In `mistake_classification`, `Space` and `Enter` are trapped with `e.preventDefault()` and `e.stopPropagation()`.
3. **Deferred Solution Reveal (`ANTI-08`):** `#proc-solution-container` is strictly hidden during `mistake_classification` and unhidden only after category selection.
4. **One-Interaction-Surface:** Native ease buttons are suppressed on procedural cards; in-card `#proc-next-btn` drives progression. Standard Basic and Cloze cards remain 100% untouched native reviews.
