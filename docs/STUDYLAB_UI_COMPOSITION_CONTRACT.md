# STUDYLAB UI COMPOSITION CONTRACT
## Canonical Master Specification for Screen Compositions, Core States, Modalities, Controls, and Runtime Boundaries

**Document Version:** 1.0.0 (Canonical Master Contract)  
**Target Repository:** `Anki-maths`  
**Subsystems Governed:** 
- TypeScript Reviewer Subsystem (`ts/reviewer/`, `ts/reviewer/components/`, `ts/reviewer/reviewer.scss`)
- Rust Procedural Template Core (`rslib/procedural/src/reviewer/template.rs`, `rslib/src/notetype/render.rs`)
- Python Desktop Host Bridge (`qt/aqt/reviewer.py`)
- Procedural Anchor Data Pipeline (`APKG` $\rightarrow$ `rslib` $\rightarrow$ `SQLite` $\rightarrow$ `Webview DOM`)

**Status:** CANONICAL CONTRACT & SINGLE SOURCE OF TRUTH (Milestone 1)  
**Integrity Mode:** Benchmark Mode (100% Grounded in Executable Architecture, Forensic Survey, and Educational Invariants)

---

## Table of Contents
1. [North Star, Core Principles & Visual Hero Invariant](#1-north-star-core-principles--visual-hero-invariant)
   - 1.1 Canonical Product Definition
   - 1.2 The Visual Hero Invariant
   - 1.3 The Core Learner Loop
   - 1.4 Open Canvas Layout Philosophy
2. [Lifecycle State Machine & The 11 Core UI States](#2-lifecycle-state-machine--the-11-core-ui-states)
   - 2.1 State Lifecycle Flowchart
   - 2.2 Comprehensive State-by-State Specification
     - 2.2.1 `loading`
     - 2.2.2 `ready`
     - 2.2.3 `solving`
     - 2.2.4 `submitting`
     - 2.2.5 `wrong_answer`
     - 2.2.6 `mistake_classification`
     - 2.2.7 `feedback`
     - 2.2.8 `next`
     - 2.2.9 `stepwise`
     - 2.2.10 `concept_check`
     - 2.2.11 `strategy_drill`
     - 2.2.12 `worked_example`
3. [Modality Composition Rules & Zero-Textbox Fallback](#3-modality-composition-rules--zero-textbox-fallback)
   - 3.1 Semantic Modality Purity Invariant
   - 3.2 Modality 1: Numerical / Quick Solve
   - 3.3 Modality 2: Multiple Choice Question (MCQ)
   - 3.4 Modality 3: Concept Check
   - 3.5 Modality 4: Strategy Drill
   - 3.6 Modality 5: Stepwise Derivation Workspace
   - 3.7 Modality 6: Worked Example
   - 3.8 Zero-Textbox Fallback Enforcement Invariant
4. [Master 23-Button Interaction Matrix & Mutually Exclusive Control Sets](#4-master-23-button-interaction-matrix--mutually-exclusive-control-sets)
   - 4.1 Master 23-Control Matrix
   - 4.2 Mutually Exclusive Control Sets
   - 4.3 Coexistence and Forbidden Combination Matrix
5. [Metacognitive Mistake Classification Reflection System](#5-metacognitive-mistake-classification-reflection-system)
   - 5.1 Cognitive Science Grounding
   - 5.2 The 4-Category Mistake Taxonomy
   - 5.3 Space/Enter Anti-Bypass Lock Architecture
   - 5.4 Deferred Solution Reveal Invariant
   - 5.5 FSRS Ease & Remediation Calibration Matrix
6. [Native Anki Runtime Boundary & Footer Ownership](#6-native-anki-runtime-boundary--footer-ownership)
   - 6.1 Host (Anki) vs Procedural (StudyLab) Responsibility Boundary
   - 6.2 Note Model Interception & Pure Non-Procedural Isolation
   - 6.3 Bottom Action Bar & Footer State-by-State Ownership
   - 6.4 Bridge Command Dispatch Table
7. [Visual Anti-Patterns & Prohibitions Ledger](#7-visual-anti-patterns--prohibitions-ledger)
8. [Acceptance Criteria & Quality Benchmarks](#8-acceptance-criteria--quality-benchmarks)
   - 8.1 Quantitative Performance Benchmarks ("Perfect Window")
   - 8.2 Qualitative Screen Composition Verification Matrix (14 Target States)

---

## 1. North Star, Core Principles & Visual Hero Invariant

### 1.1 Canonical Product Definition
StudyLab is a procedural learning, problem-solving, and diagnostic engine hosted inside the Anki desktop runtime.

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                            CANONICAL PRODUCT TRUTH                               │
├──────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│   StudyLab is NOT:                                                               │
│   • A flashcard application or card-flip/reveal deck                             │
│   • A generic quiz website embedded inside a webview                             │
│   • A replacement for Anki's spaced repetition scheduler (FSRS)                  │
│   • A telemetry dashboard exposing raw backend analytics                         │
│                                                                                  │
│   StudyLab IS:                                                                   │
│   • An interactive problem-solving workspace embedded in Anki                   │
│   • A procedural mathematical and scientific problem generator                   │
│   • A real-time Computer Algebra System (CAS) and dimensional unit evaluator     │
│   • A metacognitive error classification and cognitive remediation layer        │
│   • An adaptive next-problem selector calibrated with FSRS memory stability      │
│                                                                                  │
└──────────────────────────────────────────────────────────────────────────────────┘
```

### 1.2 The Visual Hero Invariant
In every screen composition and state in StudyLab, the **mathematical, physical, or logical problem statement is the primary visual hero**.

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                           THE VISUAL HERO INVARIANT                              │
├──────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│   1. The learner's cognitive problem-solving task occupies the primary optical   │
│      focus of the screen at all times.                                           │
│   2. The user interface chrome, badges, borders, and controls are completely     │
│      subordinate to the problem statement.                                       │
│   3. High-contrast typography and MathJax/LaTeX typesetting take precedence      │
│      over background surfaces and decorative elements.                           │
│   4. Spacing, padding, and layout are designed for deep reasoning, preventing    │
│      visual crowding, horizontal scrolling, and cognitive fatigue.              │
│                                                                                  │
└──────────────────────────────────────────────────────────────────────────────────┘
```

### 1.3 The Core Learner Loop
Every interaction in StudyLab strictly follows a 5-step learner priority:

$$\text{1. PROBLEM} \longrightarrow \text{2. ONE INTERACTION} \longrightarrow \text{3. MINIMAL FEEDBACK} \longrightarrow \text{4. DIAGNOSIS (ON ERROR)} \longrightarrow \text{5. ONE NEXT ACTION}$$

Behind the scenes, the engine executes the 6th layer silently without permanent UI clutter:

$$\text{6. ATTEMPT EVIDENCE} \longrightarrow \text{SKILL UPDATE} \longrightarrow \text{WEAKNESS DETECTION} \longrightarrow \text{FSRS CALIBRATION} \longrightarrow \text{ADAPTIVE QUEUE}$$

### 1.4 Open Canvas Layout Philosophy
1. **Open Canvas Typography:** Flat, unconfined layout using subtle 1px dividers (`--proc-border`) and generous vertical rhythm (4px/8px incremental grid, `max-width: 720px`) rather than heavy bounding boxes.
2. **Zero Nested Cards:** Card-in-a-card nesting (outer container $\rightarrow$ inner card $\rightarrow$ solution card $\rightarrow$ pitfall box) is **strictly forbidden**. Sections flow sequentially down the canvas.
3. **Subtle Accent Boundaries:** Callouts (e.g. key decisions, pitfalls, hints) utilize a 3px solid left accent border with a transparent or ultra-subtle tint background (`--proc-surface-subtle`), eliminating heavy enclosing rectangles.
4. **Muted Monochrome Chrome:** Headers use clean, single-row breadcrumbs (`Physics › Kinematics › Relative Velocity`). Generic badges (e.g. `VARIANT: PRACTICE`) are eliminated; only authentic competitive exam provenance tags (e.g. `[ JEE Main 2024 ]`) are permitted.

---

## 2. Lifecycle State Machine & The 11 Core UI States

### 2.1 State Lifecycle Flowchart

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

### 2.2 Comprehensive State-by-State Specification

#### 2.2.1 `loading`
- **State Identifier:** `loading`
- **Lifecycle Phase:** Initialization / Ingress
- **Learner Goal:** Smooth, instantaneous card transition without layout shift or un-typeset raw LaTeX flash.
- **Visual Hero:** Loading placeholder with neutral background matching native Anki canvas.
- **Primary Action:** None (system automated).
- **Secondary Actions:** None.
- **Visible Elements:** `#procedural-card` wrapper with subtle opacity fade-in.
- **Hidden / Forbidden Elements:** Solving inputs, submit buttons, hint boxes, result panels, mistake strips, native Anki `#ansbut`, native ease buttons.
- **Keyboard Invariants:** All hotkeys disabled/suppressed during DOM parsing and MathJax typesetting.
- **Transition Triggers:** DOM load event + MathJax typesetting resolution $\rightarrow$ `ready` (Target: $<80\text{ms}$).
- **Dispatched IPC Events:** None.
- **DOM Selectors:** `#procedural-card.proc-loading`.

#### 2.2.2 `ready`
- **State Identifier:** `ready`
- **Lifecycle Phase:** Problem Presentation / Armed
- **Learner Goal:** Read the problem stem, inspect given variables and parameters, and formulate solving strategy.
- **Visual Hero:** High-contrast problem statement (`#proc-prompt`) rendered cleanly in MathJax LaTeX.
- **Primary Action:** Focus input field (Numerical) or inspect option choices (MCQ/Concept/Strategy).
- **Secondary Actions:** `💡 Request Hint` (`#proc-hint-btn`, shortcut `H`/`?`), mode toggle (`Quick` vs `Stepwise` on dual-mode problems).
- **Visible Elements:** 
  - Header breadcrumbs (`.proc-header`: Subject $\rightarrow$ Topic $\rightarrow$ Skill)
  - Authentic exam provenance tag (if applicable, e.g. `[ JEE Main 2024 ]`)
  - Problem prompt (`#proc-prompt`)
  - Armed modality interaction surface (`#proc-quick-container`, `#proc-mcq-container`, or `#proc-stepwise-container`)
  - Primary CTA: `Submit Answer` (`#proc-submit-btn`) or `Check Solution` (`#proc-check-steps-btn`)
  - Hint button: `#proc-hint-btn`
- **Hidden / Forbidden Elements:** 
  - Result panel (`#proc-result-panel`) **strictly hidden**
  - Mistake classification strip (`#proc-mistake-panel`) **strictly hidden**
  - Solution derivation (`#proc-solution-container`) **strictly hidden**
  - Native Anki `#ansbut` **strictly suppressed**
  - Native Anki bottom ease buttons (`Again`, `Hard`, `Good`, `Easy`) **strictly suppressed**
  - Generic text input boxes on MCQs/Concept/Strategy/Worked Example **strictly forbidden**
  - Ticking stopwatch millisecond displays **strictly forbidden** (`ANTI-03`)
- **Keyboard Invariants:** Hotkeys armed (`1..4` / `A..D` for MCQ selection, numbers for numerical input, `H` / `?` for hint request, `Alt+Q` / `Alt+S` for mode tabs). `Space` and `Enter` do not trigger native card flip.
- **Transition Triggers:** User keystroke/focus $\rightarrow$ `solving`.

#### 2.2.3 `solving`
- **State Identifier:** `solving`
- **Lifecycle Phase:** Active Problem-Solving Inner Loop
- **Learner Goal:** Enter numerical answer with physical units, select candidate option card, or construct step derivation.
- **Visual Hero:** Problem prompt (`#proc-prompt`) and active interaction surface.
- **Primary Action:** Submit completed solution (`Enter` or click `#proc-submit-btn` / `#proc-check-steps-btn`).
- **Secondary Actions:** Request progressive hint (`H` / `?`), adjust selection.
- **Visible Elements:** 
  - Problem prompt
  - Active input field (`#proc-answer-input`) with live dynamic unit preview pill (`.proc-num-preview-pill`) OR active radio option card (`.proc-option-item.selected` with 2px accent outline)
  - Primary submission CTA
  - Hint button (`#proc-hint-btn`)
  - Inline progressive hint drawer (`#proc-hint-container`, if requested)
- **Hidden / Forbidden Elements:** 
  - Live ticking seconds timer (runs silently in telemetry background; no visual stopwatch, `ANTI-03`)
  - Duplicate static unit labels
  - Mistake classification strip
  - Canonical solution derivation
  - Native Anki ease rating bar
- **Keyboard Invariants:** 
  - Numerical mode: `Enter` submits; typing alphanumeric characters updates input and live preview pill in $<16\text{ms}$.
  - MCQ / Choice mode: `1..4` or `A..D` selects option immediately; `Enter` or `Space` on focused option confirms selection.
  - Stepwise mode: `Enter` validates current row and focuses next row; `Ctrl+Enter` triggers `Check Solution`.
  - Hint: `H` or `?` triggers progressive hint tier (Level 1 Principle $\rightarrow$ Level 2 Operation $\rightarrow$ Level 3 Setup) and dispatches `procedural_hint`.
- **Transition Triggers:** 
  - User submits answer $\rightarrow$ `submitting`
  - User requests hint $\rightarrow$ `hint` (inline disclosure within solving)

#### 2.2.4 `submitting`
- **State Identifier:** `submitting`
- **Lifecycle Phase:** Local AST Validation & Unit Normalization
- **Learner Goal:** Instantaneous validation of submitted response against mathematical constraints.
- **Visual Hero:** Problem prompt with inputs locked.
- **Primary Action:** None (automated transition, duration $<50\text{ms}$).
- **Secondary Actions:** None.
- **Visible Elements:** Problem prompt, input elements with `.proc-input-locked` class, disabled submit button (`disabled = true`).
- **Hidden / Forbidden Elements:** Interactive editing, double-submit click triggers.
- **Keyboard Invariants:** All keystrokes ignored/debounced to prevent duplicate submission events.
- **Transition Triggers:** 
  - Response is mathematically correct $\rightarrow$ `feedback`
  - Response is mathematically incorrect $\rightarrow$ `wrong_answer` $\rightarrow$ `mistake_classification`
- **Dispatched IPC Events:** Local AST evaluation and dimension normalization executed client-side; prepares telemetry payload.

#### 2.2.5 `wrong_answer`
- **State Identifier:** `wrong_answer`
- **Lifecycle Phase:** Intermediate Error Outcome
- **Learner Goal:** Immediate, calm recognition that the submitted answer did not satisfy constraints.
- **Visual Hero:** Problem statement with subtle inline status indicator.
- **Primary Action:** Focus mistake classification reflection strip.
- **Secondary Actions:** None.
- **Visible Elements:** 
  - Problem prompt
  - Subtle inline status indicator: `✗ Incorrect` (`.proc-status-incorrect`)
  - Concise deduplicated comparison: `Your answer: 24 m/s` (Expected answer withheld until reflection)
- **Hidden / Forbidden Elements:** 
  - Giant red full-bleed background banners (`ANTI-01`) **strictly forbidden**
  - Canonical step-by-step derivation (`ANTI-08`) **strictly hidden**
  - Primary CTA `#proc-next-btn` **strictly hidden**
  - Native Anki ease rating bar **strictly suppressed**
- **Keyboard Invariants:** `Space` and `Enter` keys are trapped (`e.preventDefault()`, `e.stopPropagation()`).
- **Transition Triggers:** Immediate synchronous transition $\rightarrow$ `mistake_classification`.

#### 2.2.6 `mistake_classification`
- **State Identifier:** `mistake_classification`
- **Lifecycle Phase:** Metacognitive Reflection Gate
- **Learner Goal:** Diagnose and record the root cause of the error (calculation slip vs pattern blindness vs concept gap vs missing prerequisite).
- **Visual Hero:** Compact 4-category reflection strip (`#proc-mistake-panel`).
- **Visible Content:** 
  - Prompt header: *"Classify error to reflect and optimize spaced repetition:"*
  - 4 Category Buttons (`.proc-mistake-btn`):
    1. `[1 Silly Slip]` (`silly_mistake`: Arithmetic, sign, or unit slip)
    2. `[2 Pattern Missed]` (`pattern_not_recognized`: Failed to identify structure/symmetry)
    3. `[3 Concept Gap]` (`formula_or_concept_misapplied`: Wrong formula or theorem)
    4. `[4 Prereq Unknown]` (`concept_not_known`: Missing foundational prerequisite knowledge)
- **Visible Controls:** Exactly the 4 mistake category buttons (`data-key="1..4"`).
- **Hidden / Forbidden Elements:** 
  - `Next Problem` button (`#proc-next-btn`) **strictly hidden**
  - Canonical solution derivation (`#proc-solution-container`) **strictly hidden until selection** (`ANTI-08`)
  - Native Anki ease buttons and `#ansbut` **strictly suppressed**
  - Skip / bypass controls **strictly forbidden**
- **Keyboard Invariants:** 
  - **`Space` and `Enter` keys are strictly trapped and blocked** via global capture-phase listeners.
  - Keys `1`, `2`, `3`, and `4` select the respective mistake category.
- **Transition Triggers:** Learner presses `1`, `2`, `3`, or `4` (or clicks button) $\rightarrow$ button receives 150ms visual confirmation highlight $\rightarrow$ reveals canonical solution $\rightarrow$ transitions to `feedback`.
- **Dispatched IPC Events:** `bridgeCommand("procedural_mistake:<json>")` transmitting `{ instance_id, family_id, mistake_type }`.

#### 2.2.7 `feedback`
- **State Identifier:** `feedback`
- **Lifecycle Phase:** Canonical Derivation Review & Outcome Consolidation
- **Learner Goal:** Review expert step-by-step derivation, verify mental model, inspect performance quadrant, and advance.
- **Visual Hero:** Canonical step-by-step LaTeX derivation trace (`.proc-derivation-trace`).
- **Visible Content:** 
  - Concise outcome header: `✓ Correct` or `✗ Incorrect (Categorized: Concept Gap)`
  - Deduplicated answer comparison row: `Your answer: 30 m/s · Correct answer: 30 m/s` (`ANTI-02`)
  - Canonical LaTeX derivation steps with clear mathematical progression on open canvas
  - Performance speed pill: compact muted badge (e.g. `⚡ Fast & Accurate · 8.4s` or `🎯 Accurate · 24.1s (Target: 20s)`) (`ANTI-04`)
- **Visible Controls:** 
  - Primary CTA: `#proc-next-btn` (`Next Problem ➔ (Space / Enter)`)
  - Optional Secondary CTA: `#proc-try-similar-btn` (`Try Similar Problem (Alt+T)`)
  - Optional Remedial CTA: `#proc-practice-prereq-btn` (`Practice Prerequisite (Alt+P)`)
- **Hidden / Forbidden Elements:** 
  - Interactive solving inputs (textboxes, radio inputs)
  - Duplicate expected answer labels (`ANTI-02`)
  - Duplicate time metrics and telemetry dumps (`ANTI-03`)
  - Giant red/green container boxes (`ANTI-01`)
  - Native Anki bottom ease bar (suppressed; in-card Next owns advance)
- **Keyboard Invariants:** 
  - `Space` or `Enter` advances card via `#proc-next-btn`.
  - Keys `1`, `2`, `3`, `4` optionally override the automatic calibrated FSRS ease rating.
  - `Alt+T` triggers `Try Similar Problem`.
- **Transition Triggers:** `Space` / `Enter` on `#proc-next-btn` $\rightarrow$ `next`.
- **Dispatched IPC Events:** `bridgeCommand("procedural_attempt:<json>")` transmitting comprehensive attempt telemetry and FSRS card state updates.

#### 2.2.8 `next`
- **State Identifier:** `next`
- **Lifecycle Phase:** Card Advancement & Handover
- **Learner Goal:** Seamless transition to the next card in the Anki queue.
- **Visual Hero:** Smooth fade transition container.
- **Primary Action:** Automated execution of Anki reviewer scheduling.
- **Visible Controls:** None (controls unmounting).
- **Hidden Elements:** All interactive buttons.
- **Transition Triggers:** Direct invocation of Python host handler $\rightarrow$ `teardown` $\rightarrow$ `loading` of next card.
- **Dispatched IPC Events:** `bridgeCommand("procedural_answer:<ease>")` where `<ease>` is the calibrated rating ($1$ Again, $2$ Hard, $3$ Good, $4$ Easy).

#### 2.2.9 `stepwise` (Specialized Workspace State)
- **State Identifier:** `stepwise` (or `step_answering`)
- **Lifecycle Phase:** Multi-Step Algebraic Derivation
- **Learner Goal:** Construct row-by-row mathematical derivation with intermediate validation and error localization.
- **Visual Hero:** Sequential step derivation stack (`.proc-step-row`).
- **Visible Content:** Numbered step rows, sub-goal prompts, step LaTeX inputs, inline validation status badges (`✔ Valid`, `❌ Invalid`, `⚠️ Consistent with Prior Error`).
- **Visible Controls:** 
  - `[ Check Solution ]` (`#proc-check-steps-btn`, Primary CTA)
  - `[ + Add Step ]` (`#proc-add-step-btn`)
  - `[ 💡 Request Hint ]` (`#proc-hint-btn`)
  - `[ ↺ Reset Workspace ]` (`#proc-reset-btn` / `#proc-reset-steps-btn`)
- **Hidden / Forbidden Elements:** Single-line quick solve input field, MCQ radio cards.
- **Keyboard Invariants:** `Enter` in step row adds/focuses next step; `Ctrl+Enter` validates full solution; `Alt+A` adds step; `Alt+R` resets.
- **Transition Triggers:** `Ctrl+Enter` $\rightarrow$ `submitting` $\rightarrow$ evaluates all steps against Rust `StepValidator`.
- **Dispatched IPC Events:** `bridgeCommand("procedural_validate_steps:<json>")`.

#### 2.2.10 `concept_check` (Specialized Workspace State)
- **State Identifier:** `concept_check`
- **Lifecycle Phase:** Qualitative Conceptual Diagnosis
- **Learner Goal:** Select governing physical/mathematical law or identify common conceptual fallacy without calculation burden.
- **Visual Hero:** Conceptual prompt and qualitative statement cards (`.proc-option-item`).
- **Visible Content:** 3–4 conceptual options; upon selection of a distractor, immediately reveals targeted misconception explanation callout (`.proc-option-feedback`).
- **Visible Controls:** Conceptual radio cards, `#proc-next-btn` (post-selection).
- **Hidden / Forbidden Elements:** Free-text input boxes, numerical keypads, dimensional unit pickers.
- **Keyboard Invariants:** `1..4` / `A..D` / Arrow keys select options.
- **Transition Triggers:** Selection $\rightarrow$ `submitting` $\rightarrow$ `feedback`.

#### 2.2.11 `strategy_drill` (Specialized Workspace State)
- **State Identifier:** `strategy_drill`
- **Lifecycle Phase:** Strategic Method Comparison
- **Learner Goal:** Compare candidate solving methods (e.g. Alligation vs System of Equations) and select the most efficient route.
- **Visual Hero:** Problem context box and strategy candidate cards.
- **Visible Content:** 2–4 strategy cards displaying method name, estimated step count, and complexity rating; post-selection optimality analysis box.
- **Visible Controls:** Strategy option cards, `Next Problem` CTA.
- **Hidden / Forbidden Elements:** Numerical calculation workbenches, free-text inputs.
- **Keyboard Invariants:** `1..4` / `A..D` select strategy cards.
- **Transition Triggers:** Selection $\rightarrow$ `feedback`.

#### 2.2.12 `worked_example` (Specialized Workspace State)
- **State Identifier:** `worked_example` (or `worked_step_reveal`)
- **Lifecycle Phase:** Expert Modeling & Novice Remediation
- **Learner Goal:** Read and internalize the expert solution trace to break failure loops.
- **Visual Hero:** Key Decision Point card (`.proc-decision-box` with 3px left accent) and canonical step walkthrough.
- **Visible Content:** 
  - Problem statement
  - Key Decision Point callout (highlighting critical conceptual pivot)
  - Numbered sequential solution steps
  - Method Rationale (why this strategy works)
  - Common Pitfalls to Avoid box (`.proc-pitfall-box`)
- **Visible Controls:** Exactly one primary acknowledgment gate button:
  `[ ✔ I Have Reviewed and Understood This Solution — Try Similar Problem ]` (`#proc-try-similar-btn` / `#proc-worked-ack-btn`).
- **Hidden / Forbidden Elements:** All interactive solving inputs (zero textboxes, zero radio inputs, zero stepwise fields, `ANTI-07`).
- **Keyboard Invariants:** `Space` or `Enter` activates the acknowledgment gate.
- **Transition Triggers:** Learner activates gate $\rightarrow$ dispatches `procedural_try_similar` $\rightarrow$ generates fresh parameter seed $\rightarrow$ transitions to `ready` as an active practice problem.

---

## 3. Modality Composition Rules & Zero-Textbox Fallback

### 3.1 Semantic Modality Purity Invariant

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                       SEMANTIC MODALITY PURITY INVARIANT                         │
├──────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│   Semantic Modality MUST ALWAYS match UI Modality.                               │
│   Generic textboxes are STRICTLY FORBIDDEN as fallbacks for structured choice,   │
│   conceptual reasoning, strategy comparison, or worked example objects.          │
│                                                                                  │
└──────────────────────────────────────────────────────────────────────────────────┘
```

| # | Modality Name | Learning Object Types | Designated Interaction Surface | Strictly Forbidden UI Elements |
|---|---|---|---|---|
| **1** | **Numerical / Quick** | `problem`, `quick`, Physics, Chemistry | Single `<input id="proc-answer-input">` + live preview pill (`.proc-num-preview-pill`) | MCQ option cards, mode switch toggles on single-mode cards |
| **2** | **Multiple Choice (MCQ)** | `mcq` (Reasoning, Math, Physics) | Exactly 4 discrete radio cards (`.proc-option-item`, `role="radio"`) in `.proc-option-group` | Free-text input field (`#proc-answer-input`), quick container |
| **3** | **Concept Check** | `concept_check` | 3–4 Conceptual statement cards (`role="radio"`) + targeted misconception callout | Free-text input field, numeric keypads, stepwise derivation forms |
| **4** | **Strategy Drill** | `strategy_drill` | 2–4 Strategy candidate cards with complexity/step badges + optimality rationale | Free-text input field, CAS derivation tools |
| **5** | **Stepwise Derivation** | `stepwise` | Dynamic multi-row derivation stack (`.proc-step-row`) + per-step CAS validation | Single-line quick solve textbox, MCQ radio options |
| **6** | **Worked Example** | `worked_example` | Non-evaluative expert trace + Key Decision Point + single `Try Similar` CTA gate | ALL solving input boxes, radio options, stepwise input rows |

---

### 3.2 Modality 1: Numerical / Quick Solve
- **DOM Container:** `#proc-quick-container` (inside `#procedural-card`).
- **Primary Input:** `<input id="proc-answer-input" type="text" autocomplete="off" autocorrect="off" spellcheck="false">`.
- **Live Preview Pill:** `#proc-num-preview` (`.proc-num-preview-pill`) dynamically updating on keystrokes ($<16\text{ms}$).
  - Parses scalar magnitude, fractions ($a/b$), scientific notation ($1.5\text{e-}3$), and physical units ($30\text{ m/s}$, $1.2\text{ kg}\cdot\text{m/s}^2$).
  - Validates 5D Physical Dimensions: $[M]^m [L]^l [T]^t [N]^n [K]^k$.
- **Allowed Controls:** `#proc-answer-input`, `#proc-submit-btn`, `#proc-hint-btn`, optional `#proc-tab-stepwise` (if dual-mode).
- **Forbidden Elements:** MCQ option cards, duplicate static unit hints (`.proc-unit-hint`), raw AST debug strings.

### 3.3 Modality 2: Multiple Choice Question (MCQ)
- **DOM Container:** `#proc-mcq-container` (inside `#procedural-card`).
- **Structure:** `.proc-option-group` with `role="radiogroup"` containing exactly 4 `.proc-option-item` elements labeled `A`, `B`, `C`, `D`.
- **Keyboard Navigation:** Roving `tabindex` with `ArrowUp`/`ArrowDown`/`ArrowLeft`/`ArrowRight`, direct keys `1..4` and `A..D`.
- **Visual Feedback:** Selected option receives a crisp 2px accent border (`--proc-border-focus`); on evaluation, correct option displays emerald green (`--proc-success`), distractor displays crimson (`--proc-error`).
- **Zero-Textbox Guarantee:** `#proc-quick-container` is completely absent from DOM or removed via `enforceZeroTextInputFallback()`.

### 3.4 Modality 3: Concept Check
- **DOM Container:** `#proc-mcq-container` configured with `ConceptCheckData`.
- **Structure:** 3–4 conceptual statement cards focusing on mental models, physical laws, and fallacy diagnosis.
- **Diagnostic Behavior:** Selecting a distractor immediately reveals an inline callout explaining the specific conceptual fallacy (e.g. *Additive Fallacy: Percentages with different bases cannot be directly added*) without exposing the complete numerical solution.
- **Allowed Controls:** Concept radio cards, `#proc-next-btn` (post-selection).
- **Forbidden Elements:** Free-text inputs, dimensional unit pickers, stepwise derivation rows.

### 3.5 Modality 4: Strategy Drill
- **DOM Container:** `#proc-mcq-container` configured with `StrategyDrillData`.
- **Structure:** 2–4 strategy candidate cards displaying Strategy Name, Speed Rating, and Mental Step Count.
- **Diagnostic Behavior:** Evaluated against `preferred_option_id`. Selection reveals an optimality comparison explaining why alternative valid methods are slower or more error-prone.
- **Allowed Controls:** Strategy option cards, `Next Problem` CTA.
- **Forbidden Elements:** Numerical calculation workbenches, free-text inputs.

### 3.6 Modality 5: Stepwise Derivation Workspace
- **DOM Container:** `#proc-stepwise-container`.
- **Structure:** Dynamic vertical stack of step rows (`.proc-step-row`), each with a sub-goal label, LaTeX input field, and inline validation badge.
- **Inner Loop Validation:** Intermediate steps evaluated against Rust `StepValidator` graph.
  - Flags first failing step as `❌ Invalid`.
  - Grants downstream consistency credit (`⚠️ Consistent with prior error`, `is_downstream_consistent = true`) to prevent double-penalizing mathematical carry-through.
- **Progressive Scaffolding:** 3-tier hint disclosure (Level 1: Principle $\rightarrow$ Level 2: Next Operation $\rightarrow$ Level 3: Intermediate Setup).
- **Allowed Controls:** `#proc-add-step-btn`, `#proc-check-steps-btn`, `#proc-hint-btn`, `#proc-reset-btn` / `#proc-reset-steps-btn`.
- **Forbidden Elements:** Quick solve single textbox, MCQ option cards.

### 3.7 Modality 6: Worked Example
- **DOM Container:** `#proc-worked-box` / `#proc-result-panel`.
- **Structure:** Pedagogical reading layout:
  1. Setup Context & Problem Formulation
  2. Highlighted Key Decision Point (`.proc-decision-box`) with 3px left accent
  3. Sequentially Numbered Solution Derivation Steps
  4. Method Rationale
  5. Common Pitfalls to Avoid box (`.proc-pitfall-box`)
- **Action Gate:** Single prominent button:
  `[ ✔ I Have Reviewed and Understood This Solution — Try Similar Problem ]` (`#proc-try-similar-btn` / `#proc-worked-ack-btn`).
- **Allowed Controls:** `#proc-try-similar-btn` / `#proc-worked-ack-btn`.
- **Forbidden Elements:** All interactive solving input fields (**100% absent**).

### 3.8 Zero-Textbox Fallback Enforcement Invariant
When rendering any non-numerical object (`mcq`, `concept_check`, `strategy_drill`, `worked_example`), the TypeScript reviewer executes `enforceZeroTextInputFallback()`:
1. Programmatically removes `#proc-quick-container` and `#proc-answer-input` from the DOM (or sets `display: none !important` and `disabled = true`).
2. Suppresses the mode switcher `.proc-mode-tabs`.
3. Asserts that `#proc-answer-input` is null or inert prior to arming keyboard listeners.

---

## 4. Master 23-Button Interaction Matrix & Mutually Exclusive Control Sets

### 4.1 Master 23-Control Matrix

| # | Exact Label / Icon | DOM Selector | Ownership | Priority | Applicable Objects | Active States | Hotkey | State Transition | Dispatched IPC Event |
|---|---|---|---|---|---|---|---|---|---|
| **1** | `Quick Solve` | `#proc-tab-quick` | StudyLab | Subdued Tab | `problem` (with stepwise) | `ready`, `solving` | `Alt+Q` | Switches to quick input | None |
| **2** | `Step-by-Step` | `#proc-tab-stepwise` | StudyLab | Subdued Tab | `problem` (with stepwise) | `ready`, `solving` | `Alt+S` | Switches to stepwise | None |
| **3** | `Submit Answer` | `#proc-submit-btn` | StudyLab | **Primary CTA** | `problem`, `quick`, numerical | `ready`, `solving` | `Enter` | `solving` $\rightarrow$ `submitting` | Telemetry package prep |
| **4** | `Check Solution` | `#proc-check-steps-btn` | StudyLab | **Primary CTA** | `stepwise`, `problem` | `stepwise` | `Ctrl+Enter` | `stepwise` $\rightarrow$ `submitting` | `procedural_validate_steps` |
| **5** | `+ Add Step` | `#proc-add-step-btn` | StudyLab | Secondary | `stepwise`, `problem` | `stepwise` | `Alt+A` / `Enter` | Adds step row | None |
| **6** | `💡 Request Hint` | `#proc-hint-btn` | StudyLab | Scaffolding | `problem`, `quick`, `stepwise`, `mcq` | `ready`, `solving`, `stepwise` | `H`, `?` | $\rightarrow$ `hint` (inline tier) | `procedural_hint:<json>` |
| **7** | `↺ Reset Workspace` | `#proc-reset-btn` / `#proc-reset-steps-btn` | StudyLab | Ghost / Utility | `stepwise`, `problem` | `stepwise` | `Alt+R` | Clears step inputs | None |
| **8** | `Try Similar Problem` | `#proc-try-similar-btn` | StudyLab | Secondary Remedial | All procedural objects | `feedback`, `worked_example` | `Alt+T` | $\rightarrow$ `ready` (seeded variant) | `procedural_try_similar:<json>` |
| **9** | `Next Problem ➔` | `#proc-next-btn` | StudyLab | **Primary CTA** | All procedural objects | `feedback`, `concept_check` | `Space`, `Enter` | `feedback` $\rightarrow$ `next` | `procedural_answer:<ease>` |
| **10** | `1 Silly Slip` | `.proc-mistake-btn[data-key="1"]` | StudyLab | **Primary Reflection** | All procedural (on error) | `mistake_classification` | `1` | $\rightarrow$ `feedback` | `procedural_mistake:<json>` |
| **11** | `2 Pattern Missed` | `.proc-mistake-btn[data-key="2"]` | StudyLab | **Primary Reflection** | All procedural (on error) | `mistake_classification` | `2` | $\rightarrow$ `feedback` | `procedural_mistake:<json>` |
| **12** | `3 Concept Gap` | `.proc-mistake-btn[data-key="3"]` | StudyLab | **Primary Reflection** | All procedural (on error) | `mistake_classification` | `3` | $\rightarrow$ `feedback` | `procedural_mistake:<json>` |
| **13** | `4 Prereq Unknown` | `.proc-mistake-btn[data-key="4"]` | StudyLab | **Primary Reflection** | All procedural (on error) | `mistake_classification` | `4` | $\rightarrow$ `feedback` | `procedural_mistake:<json>` |
| **14** | `Again` (Ease 1) | Native bottom ease bar | Native Anki | Native Ease Rating | Standard Anki cards ONLY | Native answer review | `1` | Advances with Ease 1 | Native Anki rating |
| **15** | `Hard` (Ease 2) | Native bottom ease bar | Native Anki | Native Ease Rating | Standard Anki cards ONLY | Native answer review | `2` | Advances with Ease 2 | Native Anki rating |
| **16** | `Good` (Ease 3) | Native bottom ease bar | Native Anki | Native Ease Rating | Standard Anki cards ONLY | Native answer review | `3` | Advances with Ease 3 | Native Anki rating |
| **17** | `Easy` (Ease 4) | Native bottom ease bar | Native Anki | Native Ease Rating | Standard Anki cards ONLY | Native answer review | `4` | Advances with Ease 4 | Native Anki rating |
| **18** | `Show Answer` | `#ansbut` (Bottom bar) | Native Anki | Native Action | Standard Anki cards ONLY | Standard question state | `Space` | Shows standard answer | Native Anki flip |
| **19** | `More` | Bottom bar menu | Native Anki | Utility Menu | All cards (Anki shell) | All states | `M` | Context popup menu | `pycmd("more")` |
| **20** | `Practice Prerequisite` | `#proc-practice-prereq-btn`| StudyLab | Remedial CTA | Objects with prereq links | `feedback` | `Alt+P` | Navigates to prereq card | `procedural_practice_prerequisite` |
| **21** | `Review in Anki` | `#proc-declarative-recall-btn`| StudyLab | Remedial CTA | `declarative_recall`, `feedback` | `feedback` | `Alt+R` | Resolves target note | `procedural_declarative_recall` |
| **22** | `[ ✔ I Have Understood ]`| `#proc-worked-ack-btn` | StudyLab | **Primary Gate** | `worked_example` | `worked_example` | `Enter`, `Space` | $\rightarrow$ `ready` (seeded variant) | `procedural_try_similar:<json>` |
| **23** | Option Card (`A..D`) | `.proc-option-item` | StudyLab | Interactive Choice | `mcq`, `concept_check`, `strategy_drill` | `ready`, `solving` | `1..4`, `A..D` | Selects / submits | Local selection |

---

### 4.2 Mutually Exclusive Control Sets

1. **Input Surfaces (Set A):** Exactly one of `#proc-quick-container`, `#proc-stepwise-container`, `#proc-mcq-container`, or `#proc-worked-box` is mounted and visible in DOM.
2. **Submission Triggers (Set B):** Exactly one of `Submit Answer` (Numerical), `Check Solution` (Stepwise), Option Selection (MCQ), or `Acknowledge Solution` (Worked Example) is active.
3. **Review Lifecycle & Footer (Set C):** On procedural cards, all native bottom ease buttons and `#ansbut` are **SUPPRESSED**. In-card `#proc-next-btn` is the sole card progression control. On standard Basic/Cloze cards, native Anki controls remain 100% active.
4. **Error Reflection vs Progression (Set D):** During `mistake_classification`, the 4 reflection buttons are mandatory; `#proc-next-btn`, `#ansbut`, and native ease buttons are strictly blocked.

### 4.3 Coexistence and Forbidden Combination Matrix

```
┌───────────────────────────────┬───────────────────────────────┬───────────────────────────────┐
│ Control                       │ Allowed Coexisting Controls   │ Strictly Forbidden Controls   │
├───────────────────────────────┼───────────────────────────────┼───────────────────────────────┤
│ `#proc-answer-input`          │ `#proc-submit-btn`, Hint, Tabs│ MCQ options, Stepwise rows    │
│ `.proc-option-item` (MCQ)     │ `#proc-hint-btn`              │ Free textboxes, Mode switch   │
│ `#proc-check-steps-btn`       │ Add Step, Hint, Reset         │ Quick solve textbox, MCQ      │
│ `.proc-mistake-btn` (1..4)    │ None (Focused reflection)     │ Next CTA, Solution, Ease bar  │
│ `#proc-next-btn`              │ Try Similar, Remedial links   │ Solving inputs, Mistake strip │
│ Native `#ansbut` / Ease Bar   │ Native Anki bottom menu       │ Procedural card UI elements   │
└───────────────────────────────┴───────────────────────────────┴───────────────────────────────┘
```

---

## 5. Metacognitive Mistake Classification Reflection System

### 5.1 Cognitive Science Grounding
The StudyLab reflection gate is grounded in two cognitive principles:
1. **The Hypercorrection Effect (Metcalfe, 2017):** High-confidence errors produce the greatest learning gains when explicitly diagnosed and corrected immediately.
2. **Self-Explanation Effect (Chi et al., 1989):** Forcing active metacognitive attribution (identifying *why* an error occurred) converts passive failure into structured schema repair.

### 5.2 The 4-Category Mistake Taxonomy

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                      4-CATEGORY MISTAKE REFLECTION TAXONOMY                      │
├──────┬──────────────────────┬────────────────────────┬───────────────────────────┤
│ Key  │ Category Identifier  │ Cognitive Failure Mode │ Engine Diagnostic Action  │
├──────┼──────────────────────┼────────────────────────┼───────────────────────────┤
│ `1`  │ `silly_mistake`      │ Calculation / Sign Slip│ Fluency drill, minor FSRS │
│ `2`  │ `pattern_not_recog`  │ Schema Blindspot       │ StrategyDrill, isomorphic │
│ `3`  │ `formula_misapplied` │ Conceptual Confusion   │ ConceptCheck, FSRS demote │
│ `4`  │ `concept_not_known`  │ Missing Prerequisite   │ PrerequisiteReview, queue │
└──────┴──────────────────────┴────────────────────────┴───────────────────────────┘
```

1. **`1 Silly Slip` (`silly_mistake`):**
   - *Definition:* The learner understood the governing physical/mathematical principle and selected the correct schema, but made an arithmetic, sign, or unit conversion slip.
   - *Diagnostic Effect:* Flags `is_execution_error = true`. Spaced repetition schedule retains stability; flags calculation check.
2. **`2 Pattern Missed` (`pattern_not_recognized`):**
   - *Definition:* The learner failed to recognize the underlying problem schema due to surface feature variation or rotation.
   - *Diagnostic Effect:* Flags transfer failure; queues `StrategyDrill` or structural isomorphic variants.
3. **`3 Concept Gap` (`formula_or_concept_misapplied`):**
   - *Definition:* The learner applied the wrong governing formula, physical law, or mathematical theorem.
   - *Diagnostic Effect:* Flags `is_conceptual_error = true`; applies FSRS stability penalty; queues `ConceptCheck` or `WorkedExample`.
4. **`4 Prereq Unknown` (`concept_not_known`):**
   - *Definition:* The learner lacked the foundational prerequisite knowledge required to begin the problem.
   - *Diagnostic Effect:* Logs missing prerequisite node in `collection.procedural`; queues `PrerequisiteReview` or declarative concept anchor card.

### 5.3 Space/Enter Anti-Bypass Lock Architecture
- **Interception Mechanism:** When `mistake_classification` state is entered, the global capture-phase keyboard listener intercepts `Space`, `Enter`, `NumpadEnter`, and `Tab`.
- **Programmatic Guarantee:** Calls `e.preventDefault()` and `e.stopPropagation()` unconditionally.
- **Bypass Prevention:** Advancing without classifying is physically and programmatically impossible.

### 5.4 Deferred Solution Reveal Invariant
- **Strict Concealment:** `#proc-solution-container` and `#proc-next-btn` remain strictly hidden with `.hidden` class (`display: none !important`) throughout the entire duration of `mistake_classification` (`ANTI-08`).
- **Reveal Trigger:** The canonical solution derivation is unhidden **only after** a valid mistake category (`1..4`) is selected.

### 5.5 FSRS Ease & Remediation Calibration Matrix

$$\text{Calibrated Ease} = f(\text{Accuracy}, \text{Mistake Category}, \text{Latency}, \text{Hints Used})$$

| Condition | Calibrated FSRS Ease | Resulting Anki Rating | Telemetry Queue Action |
|---|---|---|---|
| Incorrect (`concept_not_known` / `formula_misapplied`) | `Ease 1` | `Again` | Queue Prerequisite / ConceptCheck |
| Incorrect (`silly_mistake` / `pattern_not_recog`) | `Ease 1` | `Again` | Queue Isomorphic Variant |
| Correct with $\ge 3$ hints or Slow ($> 1.25\times\text{target}$) | `Ease 2` | `Hard` | Retain in short-term learning queue |
| Correct with 0 hints, Standard Time ($0.75\dots1.25\times\text{target}$) | `Ease 3` | `Good` | Standard FSRS stability increment |
| Correct with 0 hints, Fast Fluency ($\le 0.75\times\text{target}$) | `Ease 4` | `Easy` | Accelerated FSRS interval bonus |

---

## 6. Native Anki Runtime Boundary & Footer Ownership

### 6.1 Host (Anki) vs Procedural (StudyLab) Responsibility Boundary

```
┌──────────────────────────────────────────────┬──────────────────────────────────────────────┐
│             HOST RUNTIME (ANKI)              │         PROCEDURAL SUBSYSTEM (STUDYLAB)      │
├──────────────────────────────────────────────┼──────────────────────────────────────────────┤
│ • SQLite Collection (`collection.anki21`)    │ • Procedural Store (`collection.procedural`) │
│ • FSRS / SM-2 Scheduling Algorithm           │ • Parameter Generation & Schema Instances    │
│ • Deck Lifecycle & Card Synchronization      │ • Real-Time Computer Algebra System (CAS)    │
│ • Standard `Basic` and `Cloze` Flashcards    │ • Physical Unit Dimensional Validation (5D)  │
│ • Window Shell, Preferences, Top Menus       │ • Metacognitive Mistake Reflection Gate      │
│ • Native Bottom Action Bar (on Basic cards)  │ • In-Card `Next Problem ➔` Action Flow       │
└──────────────────────────────────────────────┴──────────────────────────────────────────────┘
```

### 6.2 Note Model Interception & Pure Non-Procedural Isolation
- **Interception Rule:** The procedural engine strictly activates **only** when `note_type.name.startsWith("StudyLab Procedural Anchor")`.
- **Non-Procedural Isolation:** Standard declarative cards (`Basic`, `Cloze`, `Image Occlusion`) bypass all procedural logic in `rslib/src/notetype/render.rs` and `qt/aqt/reviewer.py`:
  - Zero DOM manipulation.
  - Zero procedural JavaScript initialization.
  - Native Anki `#ansbut` and ease rating buttons function 100% intact.

### 6.3 Bottom Action Bar & Footer State-by-State Ownership

| Review State | Host (Anki) Bottom Toolbar | StudyLab Webview Surface | Primary Action | Keyboard Shortcuts |
|---|---|---|---|---|
| **Standard Card (Question)** | `<button id="ansbut">Show Answer</button>` + Remaining Counts | Card Front (Mustache) | Native Show Answer | `Space`, `Enter` |
| **Standard Card (Answer)** | Ease Buttons (`Again 1`, `Hard 2`, `Good 3`, `Easy 4`) | Card Back (Mustache) | Native Rating | `1`, `2`, `3`, `4`, `Space` |
| **Procedural `ready` / `solving`** | Progress counts only (`span.stattxt`); `#ansbut` **suppressed** | Problem Prompt + Modality Input Surface | Submit Answer | `Enter`, `A..D` / `1..4`, `H` |
| **Procedural `mistake_classification`** | Progress counts only | Error Status + 4 Reflection Buttons | Select Category (1..4) | `1`, `2`, `3`, `4` (Space/Enter trapped) |
| **Procedural `feedback`** | Progress counts only (Ease buttons suppressed) | Deduplicated Solution + `#proc-next-btn` | Next Problem | `Space`, `Enter` (or `1..4` override) |

### 6.4 Bridge Command Dispatch Table

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│                            PyQt Reviewer Host                               │
│                            (qt/aqt/reviewer.py)                             │
│                                      ▲                                      │
│                                      │ pycmd(endpoint)                      │
│                                      ▼                                      │
│                            Reviewer Webview DOM                             │
│                           (ts/reviewer/procedural.ts)                       │
└─────────────────────────────────────────────────────────────────────────────┘
```

- `procedural_answer:<ease>`: Directly invokes `reviewer._answerCard(ease)` committing the calibrated rating to FSRS.
- `procedural_attempt:<json>`: Records attempt telemetry (accuracy, time taken, mode, selected option, steps).
- `procedural_mistake:<json>`: Records mistake category selection (`silly_mistake`, `pattern_not_recognized`, etc.).
- `procedural_hint:<json>`: Records hint tier consumption telemetry.
- `procedural_validate_steps:<json>`: Dispatches multi-step derivation verification to Rust CAS.
- `procedural_try_similar:<json>`: Regenerates problem variant from same family and mounts in reviewer.
- `procedural_practice_prerequisite:<json>`: Navigates to linked foundational prerequisite card.

---

## 7. Visual Anti-Patterns & Prohibitions Ledger

```
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                VISUAL ANTI-PATTERN LEDGER                                        │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
```

| ID | Anti-Pattern | Visual Violation | Code Origin | Mandated Canonical Rule |
|---|---|---|---|---|
| **ANTI-01** | **Giant Red/Green Feedback Containers** | Saturated full-bleed red/green background wrapper boxes wrapping feedback. | `reviewer.scss:706-735`<br>`procedural.ts:982` | Replace with calm inline status text (`✓ Correct` or `✗ Incorrect`) on open canvas with subtle 3px left accent border. |
| **ANTI-02** | **Duplicate Expected Answer Labels** | Repeating "You answered: X", "Correct: Y", "Expected Answer: Y" 3–4 times. | `procedural.ts:993`<br>`template.rs:594` | Consolidate into a single concise comparison row: `Your answer: X · Correct answer: Y`. |
| **ANTI-03** | **Ticking Stopwatch & Meta Dumps** | Dense meta row with live ticking timer updating every 200ms during solving. | `template.rs:594`<br>`procedural.ts:546` | Suppress stopwatch during solving (runs silently in memory); display elapsed time calmly in feedback alongside speed pill. |
| **ANTI-04** | **Speed Quadrant Badges Competing with Results** | Long, heavily styled badges (e.g. `⚡ Fluency Strength (Accurate & Fast)`). | `procedural.ts:863`<br>`reviewer.scss:655` | Streamline to a subtle, compact pill: `⚡ Fast & Accurate · 8.4s` using muted tokens. |
| **ANTI-05** | **"VARIANT: PRACTICE" Chrome** | Generic variant tags (e.g. `Variant: practice`) cluttering the header. | `template.rs:109, 182` | Suppress generic practice tags completely; display badges only for verified competitive exam provenance (e.g. `[ JEE Main 2024 ]`). |
| **ANTI-06** | **Raw Internal Metadata Leakage** | Internal schema IDs (e.g. `schema.math.linear.v1`) visible in DOM. | `template.rs:527`<br>`procedural.ts:816` | Normalize all titles to human-readable strings; retain schema/family IDs strictly in HTML data attributes. |
| **ANTI-07** | **Nested Cards (Card-in-a-Card)** | Multi-layered card nesting in worked examples and solution panels. | `template.rs:362`<br>`reviewer.scss:613` | Flatten nested cards into open canvas sections with subtle 1px dividers or 3px left accent borders. |
| **ANTI-08** | **Premature Solution Reveal in Reflection** | Unhiding `#proc-solution-container` during `mistake_classification`. | `template.rs:618`<br>`procedural.ts:982` | Keep `#proc-solution-container` strictly hidden during reflection; reveal solution ONLY after 1–4 is chosen. |

---

## 8. Acceptance Criteria & Quality Benchmarks

### 8.1 Quantitative Performance Benchmarks ("Perfect Window")

| Benchmark Dimension | Target Benchmark | Hard Invalidation Threshold | Verification Method |
|---|---|---|---|
| **Input Keystroke Latency** | $< 16\text{ms}$ (60 fps frame budget) | $> 50\text{ms}$ | Keystroke profiling |
| **Live Unit Preview Pill Update** | $< 30\text{ms}$ | $> 75\text{ms}$ | Input event timing |
| **Client-Side AST / Unit Eval** | $< 50\text{ms}$ | $> 200\text{ms}$ | AST evaluation benchmark |
| **MathJax Typesetting Duration** | $< 100\text{ms}$ per card | $> 250\text{ms}$ | MathJax promise resolution |
| **Card Mount to Interactive Ready** | $< 80\text{ms}$ | $> 300\text{ms}$ | DOM ready hook |
| **IPC Bridge Dispatch Duration** | $< 10\text{ms}$ | $> 50\text{ms}$ | `pycmd` execution timing |
| **Visual Duplication Count** | **Exactly 0** duplications | $\ge 1$ duplicate element | Visual element query |
| **Telemetry Leak Count** | **Exactly 0** schema leaks | $\ge 1$ raw ID in UI | DOM text regex assertion |
| **Accessibility & Keyboard Nav** | **100% Operable** without mouse | Any required mouse click | Mouse-free test run |

### 8.2 Qualitative Screen Composition Verification Matrix (14 Target States)

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                     14 TARGET STATES VERIFICATION MATRIX                         │
├────┬─────────────────────────────┬──────────────────────────────────────────────┤
│ #  │ Target Review State         │ Primary Verification Assertions              │
├────┼─────────────────────────────┼──────────────────────────────────────────────┤
│ 1  │ Numerical Solving           │ Problem stem hero, live unit pill, no MCQ    │
│ 2  │ Numerical Correct           │ Subtle ✓ status, single expected row, Next   │
│ 3  │ Numerical Wrong             │ Subtle ✗ status, answer comparison row       │
│ 4  │ Mistake Classification      │ 4 buttons visible, Space/Enter trapped       │
│ 5  │ Numerical Feedback          │ LaTeX derivation trace, speed pill, Next CTA │
│ 6  │ MCQ                         │ 4 radio cards, zero textboxes, 1-4 hotkeys   │
│ 7  │ Concept Check               │ Conceptual options, targeted distractor text │
│ 8  │ Strategy Drill              │ Strategy cards with step counts, comparison  │
│ 9  │ Stepwise Workspace          │ Multi-row derivation, step validation badges │
│ 10 │ Worked Example              │ Key Decision box, pitfall box, Try Similar   │
│ 11 │ Physics Numerical           │ 5D unit normalization (e.g. 30 m/s, kg)      │
│ 12 │ Chemistry Numerical         │ Mole/molar mass parsing, AST balance check   │
│ 13 │ Normal Basic Flashcard      │ 100% untouched native Anki reviewer & #ansbut│
│ 14 │ Normal Cloze Flashcard      │ 100% untouched native Anki cloze rendering   │
└────┴─────────────────────────────┴──────────────────────────────────────────────┘
```

---
*Authored and Certified by StudyLab Specification & Contract Authoring Worker M1.*
