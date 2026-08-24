# 01. Authoritative Research Findings: Native Anki Reviewer Architecture, Answer Modalities & Assessment Engineering

**Project**: StudyLab Final Reconciliation Mission  
**Document**: `01_research_findings.md`  
**Specialist Role**: Native Anki Reviewer Researcher  
**Status**: Authoritative Reference Artifact  
**Date**: 2026-08-24  

---

## Executive Summary & Product North Star

The **Product North Star** of StudyLab is defined by a singular guiding principle:
> *"Anki is the familiar, distraction-free spaced repetition shell; StudyLab provides the procedural intelligence layer inside it — Anki, but it understands how I solve problems."*

To achieve this vision without compromising Anki's battle-tested stability or alienating experienced learners, StudyLab must integrate natively with Anki's existing Qt6/QtWebEngine reviewer architecture, review lifecycle hooks, footer controls, and keyboard navigation semantics. Non-procedural flashcards (Standard Basic, Cloze, Image Occlusion) must remain 100% untouched and operate with native speed and zero regressions.

This document compiles the exhaustive, authoritative research across five core dimensions:
1. **Native Anki Reviewer Interaction Models & Architecture**: Webview division, answer reveal pipeline, review lifecycle hooks, bottom bar contracts, rating buttons, and keyboard shortcut event dispatch.
2. **Exam-Style Multiple Choice Question (MCQ) UX**: Authentic presentation, direct option selection, distractor feedback, 1–4 and A–D accessibility, position bias mitigation, and elimination of synthetic text inputs.
3. **Numerical & Dimensional Answering UX**: Dedicated numeric inputs, robust normalization (scientific notation, fractions, negative signs, equation prefixes), physical/chemical unit parsing, dimensional vector validation, and adaptive tolerances.
4. **Diagnostic Assessment & Mock-Test Engine**: Multi-dimensional diagnostic taxonomy (Concept, Execution, Transfer, Speed), speed-accuracy quadrant modeling, 4-tier hierarchical reporting (Subject $\to$ Chapter $\to$ Topic $\to$ Family), mixed-domain sampling, and native learner evidence synchronization (`SkillState`, `MasteryEvidence`, `DomainEvidence`).
5. **Reasoning Assessment & Logic Failure Diagnostics**: Taxonomy of reasoning failures (schema recognition, representation, constraint application, logical inference, search case branching, contradiction resolution, trap distractor susceptibility).

---

## 1. Native Anki Reviewer Interaction Models & Architecture

### 1.1 Dual Webview Architecture

Anki's desktop GUI (`aqt`) employs a decoupled dual-webview architecture embedded via PyQt6 / QtWebEngine (`qt/aqt/webview.py`, `qt/aqt/reviewer.py`):

```
+-------------------------------------------------------------------------+
| Anki Desktop Window (AnkiQt)                                            |
|                                                                         |
|  +-------------------------------------------------------------------+  |
|  | Top Toolbar Webview (`mw.toolbarWeb` / AnkiWebViewKind.TOP_TOOLBAR)  |
|  | Deck Name | Card Counts (New / Learn / Review) | Sync / Deck Browser |
|  +-------------------------------------------------------------------+  |
|                                                                         |
|  +-------------------------------------------------------------------+  |
|  | Main Reviewer Webview (`mw.web` / AnkiWebViewKind.MAIN)           |  |
|  |                                                                   |  |
|  |  #qa container (dir="auto")                                       |  |
|  |    +-----------------------------------------------------------+  |  |
|  |    | Question / Problem Prompt HTML                            |  |  |
|  |    | Interactive Solve Area (MCQ / Numerical / Stepwise)       |  |  |
|  |    | Solution / Feedback Panel / Hints                         |  |  |
|  |    +-----------------------------------------------------------+  |  |
|  +-------------------------------------------------------------------+  |
|                                                                         |
|  +-------------------------------------------------------------------+  |
|  | Bottom Bar Webview (`mw.bottomWeb` / AnkiWebViewKind.BOTTOM_TOOLBAR) |
|  |  [Edit (E)]   |   [Show Answer (Space)] / [1 Again | 2 Hard | 3 Good | 4 Easy]   |   [More (M) v] |
|  +-------------------------------------------------------------------+  |
+-------------------------------------------------------------------------+
```

#### Main Review Webview (`self.web` / `AnkiWebViewKind.MAIN`)
- **Shell HTML** (`Reviewer.revHtml()` in `qt/aqt/reviewer.py:329-345`):
  ```html
  <script>
    window.anki = window.anki || {};
    window.anki._state_mutation_key = "...";
  </script>
  <div id="_mark" hidden>&#x2605;</div>
  <div id="_flag" hidden>&#x2691;</div>
  <div id="qa" dir="auto"></div>
  ```
- **Assets Loaded**: `css/reviewer.css`, `js/reviewer.js` (compiled from `ts/reviewer/index.ts`).
- **Bridge Dispatch**: Operates via `QWebChannel` transport exposing `pycmd(arg, callback)` and `bridgeCommand(arg)` (`qt/aqt/webview.py:84-118`).
- **Card Context**: Modified dynamically by `_showQuestion()` and `_showAnswer()` evaluating `_showQuestion(q, a, bodyclass)` and `_showAnswer(a)`.

#### Bottom Bar Webview (`self.bottom.web` / `mw.bottomWeb` / `AnkiWebViewKind.BOTTOM_TOOLBAR`)
- **Shell HTML** (`Reviewer._bottomHTML()` in `qt/aqt/reviewer.py:833-862`):
  - Left slot: `Edit` button (`onclick="pycmd('edit');"` / shortcut `E`).
  - Center slot: `#middle` dynamic table containing either `#ansbut` (`Show Answer` / shortcut `Space`) or ease rating buttons (`Again`, `Hard`, `Good`, `Easy` via `pycmd('ease1')` through `pycmd('ease4')`).
  - Right slot: `More` context menu button (`onclick="pycmd('more');"` / shortcut `M`) and elapsed timer.
- **Assets Loaded**: `css/toolbar-bottom.css`, `css/reviewer-bottom.css`, `js/vendor/jquery.min.js`, `js/reviewer-bottom.js`.

---

### 1.2 Native Answer Reveal Flow & Lifecycle Hooks

The complete review lifecycle operates as a state machine transitioning across `question`, `answer`, and `transition` states:

```
[ Card Selected in Queue ]
           │
           ▼
[ Reviewer.nextCard() ] ──► [ _get_next_v3_card() ]
           │
           ▼
[ Reviewer._showQuestion() ]
   ├── gui_hooks.reviewer_will_play_question_sounds
   ├── q = gui_hooks.card_will_show(q, card, "reviewQuestion")
   ├── _run_state_mutation_hook() (custom FSRS card scheduling)
   ├── self.web.eval("_showQuestion(q, a, bodyclass)")
   ├── self._showAnswerButton() in bottomWeb ("Show Answer" / Space)
   └── gui_hooks.reviewer_did_show_question(card)
           │
           │  (Learner interacts / solves / submits / presses Space)
           ▼
[ Reviewer._showAnswer() ]  (triggered via pycmd("ans") or onEnterKey)
   ├── gui_hooks.reviewer_will_play_answer_sounds
   ├── a = gui_hooks.card_will_show(a, card, "reviewAnswer")
   ├── self.web.eval("_showAnswer(a)")
   ├── self._showEaseButtons() in bottomWeb ([1 Again] [2 Hard] [3 Good] [4 Easy])
   └── gui_hooks.reviewer_did_show_answer(card)
           │
           │  (Learner rates card via 1-4, Space, or mistake classification)
           ▼
[ Reviewer._answerCard(ease) ]
   ├── (proceed, ease) = gui_hooks.reviewer_will_answer_card((True, ease), reviewer, card)
   ├── sched.build_answer(card, states, rating)
   ├── answer_card(answer).run_in_background()
   ├── gui_hooks.reviewer_did_answer_card(reviewer, card, ease)
   └── self.nextCard()
```

#### Key Reviewer Hooks (`qt/tools/genhooks_gui.py` & `pylib/anki/hooks.py`)
1. `card_will_show(text: str, card: Card, kind: str) -> str`:
   - Intercepts card HTML before rendering in `self.web`. `kind` is `"reviewQuestion"`, `"reviewAnswer"`, `"clayoutQuestion"`, etc.
   - Used by StudyLab to inject procedural styles, MathJax scripts, and container bindings.
2. `reviewer_did_show_question(card: Card)`:
   - Notifies extensions when the question has been rendered and focused.
3. `reviewer_did_show_answer(card: Card)`:
   - Notifies extensions when the back/answer side is shown.
4. `reviewer_will_answer_card(ease_tuple: tuple[bool, Literal[1, 2, 3, 4]], reviewer: Reviewer, card: Card) -> tuple[bool, Literal[1, 2, 3, 4]]`:
   - Allows intercepting or overriding the ease rating before it is written to the scheduler database.
5. `reviewer_did_answer_card(reviewer: Reviewer, card: Card, ease: Literal[1, 2, 3, 4])`:
   - Primary hook for telemetry recording and learner mastery evidence synchronization.
6. `reviewer_will_end()`:
   - Lifecycle cleanup hook triggered when exiting review mode to avoid dangling timers or listeners.

---

### 1.3 Rating Buttons & Keyboard Shortcut Handling

#### Native Keyboard Bindings (`Reviewer._shortcutKeys()` in `qt/aqt/reviewer.py:601-645`):
- `Space`, `Return`, `Enter` $\to$ `self.onEnterKey`:
  - In `state == "question"`: Calls `self._getTypedAnswer()` $\to$ evaluates `getTypedAnswer()` in JS $\to$ calls `self._showAnswer()`.
  - In `state == "answer"`: Calls `self.bottom.web.evalWithCallback("selectedAnswerButton()", self._onAnswerButton)` $\to$ rates with `_defaultEase()` (Ease 3 = `Good`).
- `1`, `2`, `3`, `4` $\to$ `self._answerCard(1..4)`:
  - Bound via `aqt.mw.pm.default_answer_keys`.
  - In `state == "question"`: Ignored because `_answerCard()` guards with `if self.state != "answer": return`.
  - In `state == "answer"`: Rates card immediately as 1 (`Again`), 2 (`Hard`), 3 (`Good`), or 4 (`Easy`).
- Utility shortcuts: `e` (Edit current card), `r`/`F5` (Replay audio), `m` (More context menu), `Ctrl+1..7` (Flags), `*` (Toggle mark), `-`/`=` (Bury card/note), `@`/`!` (Suspend card/note), `u` (Undo).

#### Procedural Interaction & Keyboard Event Trapping:
To prevent standard Anki shortcuts from prematurely advancing procedural cards while the student is actively solving:
1. **Solving State**:
   - Keystrokes `1`–`4` and `A`–`D` pressed outside text inputs are captured by `ts/reviewer/procedural.ts` to select MCQ options.
   - `Space` and `Enter` outside input trigger local validation/submission rather than jumping straight to native answer flip.
2. **Mistake Classification State**:
   - When a student answers incorrectly, `Space` and `Enter` are trapped (`e.preventDefault()`, `e.stopPropagation()`) to prevent skipping the reflection step.
   - Keys `1`, `2`, `3`, `4` trigger mistake classification (`[1 Silly]`, `[2 Pattern]`, `[3 Concept]`, `[4 Unknown]`).
3. **Feedback State**:
   - Once classified and reviewed, pressing `Space` or `Enter` executes `handleNext()`, transmitting `bridgeCommand("procedural_answer:1")` for incorrect attempts, cleanly synchronizing with native scheduling.

---

## 2. Exam-Style Multiple Choice Question (MCQ) UX

### 2.1 Architectural Contract & Anti-Patterns to Eliminate

Standard competitive examinations (JEE, NEET, SAT, CAT, GMAT) present multiple-choice questions as structured, distinct options with immediate visual selection affordances. 

```
                                  [ MCQ Problem Prompt ]
        "What is the net single discount equivalent to two successive discounts of 20% and 10%?"
                                             │
      ┌──────────────────┬───────────────────┴───────────────────┬──────────────────┐
      ▼                  ▼                                       ▼                  ▼
[ A: 30% ]          [ B: 28% ]                              [ C: 25% ]         [ D: 32% ]
(Key '1' / 'A')     (Key '2' / 'B')                         (Key '3' / 'C')    (Key '4' / 'D')
```

#### Critical Anti-Pattern: Synthetic Text Input Fallback
In legacy procedural implementations, MCQ items were frequently degraded into generic text boxes asking students to type `"B"` or `"28%"`. This creates major UX defects:
- **Artificial Typing Friction**: Forces physical keyboard typing where single-key or tap selection is standard.
- **Parsing Ambiguity**: Fails if the student types `"b"`, `"Option B"`, `"28"`, `"28%"` or `"B - 28%"`.
- **Loss of Exam Authenticity**: Destroys the cognitive fidelity of authentic exam practice.

**Contract Rule**: MCQ items MUST render real interactive option containers (`.proc-option-item`) and NEVER render a text input field (`#proc-answer-input`).

---

### 2.2 Option Selection & Keyboard Accessibility Matrix

| Interaction Method | Action / Keystroke | Behavior / Visual State |
|:---|:---|:---|
| **Mouse / Touch** | Single click on `.proc-option-item` | Focuses and immediately selects the target option. |
| **Numeric Keys** | Press `1`, `2`, `3`, or `4` | Selects option at index 0, 1, 2, or 3 respectively. |
| **Alpha Keys** | Press `A`, `B`, `C`, or `D` (case-insensitive) | Selects option A, B, C, or D respectively. |
| **Arrow Navigation**| Press `Up` / `Down` or `Left` / `Right` | Moves focus between adjacent option elements. |
| **Confirmation** | Press `Space` or `Enter` on focused option | Confirms selection and triggers evaluation. |

#### Accessibility & ARIA Specifications:
- Parent container: `role="radiogroup"`, `aria-label="Multiple choice options"`.
- Each option element: `role="radio"`, `aria-checked="true|false"`, `tabindex="0"`.
- Keyboard event listener: Attached to `window` with active state checking (`this.state === "solving"`).

---

### 2.3 Position Bias Mitigation & Canonical Identity

In randomized generation, fixed option placement creates severe psychological biases (e.g. students guessing `"C"` or choosing top options).

#### Mitigation Protocol:
1. **Deterministic Permutation Shuffling**:
   - Problem instances generate 4 candidate options: 1 canonical correct option and 3 calibrated distractors.
   - Options are shuffled using a deterministic pseudo-random generator seeded by `(problem_seed ^ 0x5DEECE66D)`.
2. **Canonical Identity Tracking**:
   - Evaluation NEVER relies on positional index (0, 1, 2, 3) alone.
   - Each option maintains a semantic identifier (`data-opt-id="opt_correct_28"`, `data-opt-id="opt_distractor_sum_30"`).
   - The correctness contract validates:
     $$\text{is\_correct} = (\text{selected\_option\_id} == \text{canonical\_option\_id})$$

---

### 2.4 Explanatory Feedback & Distractor Rationales

Upon option selection:
- The selected option is locked and marked with `.selected`.
- The correct option is highlighted in emerald green (`.correct`).
- If an incorrect option was chosen, it is highlighted in ruby red (`.incorrect`) and other options are dimmed (`.disabled`).
- **Distractor Diagnosis**: Displays why the chosen distractor represents a specific conceptual trap (e.g. *"Choosing 30% indicates simple linear addition $20\% + 10\%$, ignoring that the second discount applies to the discounted price."*).
- Canonical step-by-step derivation is revealed in `#proc-solution-container`.

---

## 3. Numerical & Dimensional Answering UX

### 3.1 Dedicated Numeric Input Architecture

For calculation-intensive disciplines (Mathematics, Physics, Chemistry), problems require direct numeric entry rather than multiple-choice guessing.

```
+-----------------------------------------------------------------------------------+
| Numerical Problem Prompt                                                          |
| A 2.5 kg projectile is launched at 12 m/s. Find its kinetic energy in Joules.      |
|                                                                                   |
|  [ Quick Solve ]   [ Step-by-Step Solve ]                                         |
|                                                                                   |
|  Enter final answer:                                                              |
|  [ 180 J                                      ]   [ Submit ]                      |
|                                                                                   |
|  Parsed: 180.0 J  |  Dimension: [M]^1 [L]^2 [T]^-2 (ENERGY)  |  Status: ✓ MATCH   |
+-----------------------------------------------------------------------------------+
```

---

### 3.2 Robust Normalization & Parsing Engine

Learners type numeric responses in diverse natural formats. The parser (`ProceduralReviewer.parseNumericValue` in `ts/reviewer/procedural.ts:615-663` and Rust `DimensionalValidator` in `rslib/procedural/src/physics/units.rs`) must handle all variations deterministically:

#### Parsing Capabilities Matrix:
1. **Equation Prefix Stripping**:
   - Inputs: `"x = 6"`, `"v = 12 m/s"`, `"KE = 180"`, `"ans: 42"`, `"Result = -3.5"`.
   - Regex: `^[a-zA-Z_\s]+[:=]\s*` $\to$ stripped cleanly.
2. **Negative Numbers & Unicode Symbols**:
   - Handles standard hyphen `-`, ASCII minus, and Unicode minus `−` (`\u2212`).
3. **Arithmetic Fractions**:
   - Inputs: `"3/4"`, `"-7/8"`, `"22/7"`, `"1 1/2"`.
   - Resolves: $\frac{\text{numerator}}{\text{denominator}}$ with divide-by-zero protection.
4. **Scientific & Engineering Notation**:
   - Standard E-notation: `"1.2e-3"`, `"4.5E6"`, `"-3.2e+4"`.
   - LaTeX / Multiplier style: `"1.2 x 10^-3"`, `"1.2 * 10^3"`, `"1.2 × 10^-3"`.
   - Evaluation: $m \times 10^e$.
5. **Currency & Extraneous Symbols**:
   - Strips `$`, `€`, `£`, `₹`, `%`, and formatting commas (`1,000` $\to$ `1000`).

---

### 3.3 Physical & Chemical Units & Dimensional Correctness

Physical and chemical problem solving requires dimensional integrity. A bare number without units (or with incorrect units) is physically incomplete.

#### 1. Dimensional Vectors

##### Physics Vector ($[M]^m [L]^l [T]^t$):
```rust
pub struct PhysicalDimension {
    pub mass: i8,    // Mass exponent (kg)
    pub length: i8,  // Length exponent (m)
    pub time: i8,    // Time exponent (s)
}
```
- Dimensionless: $[0, 0, 0]$
- Length (`m`, `km`): $[0, 1, 0]$
- Velocity (`m/s`, `km/h`): $[0, 1, -1]$
- Acceleration (`m/s²`): $[0, 1, -2]$
- Force (`N`): $[1, 1, -2]$
- Energy (`J`): $[1, 2, -2]$
- Power (`W`): $[1, 2, -3]$

##### Chemistry Vector ($[M]^m [L]^l [T]^t [N]^n [\Theta]^k$):
```rust
pub struct ChemicalDimension {
    pub mass: i8,        // Mass (g, kg)
    pub length: i8,      // Volume (L, mL, m^3)
    pub time: i8,        // Time (s, min, h)
    pub amount: i8,      // Amount of substance (mol, mmol)
    pub temperature: i8, // Temperature (K, °C)
}
```
- Molar Mass (`g/mol`): $[1, 0, 0, -1, 0]$
- Concentration (`M`, `mol/L`): $[0, -3, 0, 1, 0]$
- Molar Energy (`kJ/mol`): $[1, 2, -2, -1, 0]$

#### 2. Unit Scale Normalization & Compatibility Verification

```
User Input: "72 km/h"
  1. Extract number: 72.0
  2. Extract unit string: "km/h"
  3. Resolve unit: PhysicsUnit::KilometerPerHour
  4. Check dimension: PhysicalDimension::VELOCITY ([0, 1, -1]) == Expected ([0, 1, -1]) -> ✓ COMPATIBLE
  5. Convert to Base SI: 72.0 * (5.0 / 18.0) = 20.0 m/s
  6. Compare against Expected Value (20.0 m/s) with tolerance -> ✓ CORRECT
```

If the student enters `"72 m/s"` (dimensionally correct, but wrong magnitude) or `"72 N"` (dimensionally incompatible), the validator flags the exact error category: `ErrorCategory::Unit` or `ErrorCategory::Calculation`.

---

### 3.4 Adaptive Tolerances

Numerical comparisons must distinguish precision rounding from actual mathematical error:

$$\Delta = |x_{\text{submitted}} - x_{\text{expected}}|$$
$$\text{Tolerance}_{\text{effective}} = \max\left(\text{tol}_{\text{absolute}}, |x_{\text{expected}}| \times \text{tol}_{\text{relative}}\right)$$
$$\text{Correct} \iff \Delta \le \text{Tolerance}_{\text{effective}}$$

- **Default Relative Tolerance**: $1\%$ ($0.01$) for continuous physical quantities.
- **Default Absolute Tolerance**: $0.01$ for values near zero.
- **Exact Evaluation**: Used for integer counting, combinatorial values, and balancing coefficients ($\text{tol} = 0$).

---

## 4. Diagnostic Assessment & Mock-Test Design

### 4.1 Four-Dimensional Assessment Taxonomy

A modern diagnostic assessment engine must evaluate more than binary correctness. StudyLab measures learner capability across four orthogonal dimensions:

```
                          ┌────────────────────────┐
                          │ DIAGNOSTIC PERFORMANCE │
                          └───────────┬────────────┘
                                      │
       ┌──────────────────┬───────────┴───────────┬──────────────────┐
       ▼                  ▼                       ▼                  ▼
  [ CONCEPT ]       [ EXECUTION ]           [ TRANSFER ]          [ SPEED ]
  - Theoretical     - Calculation           - Non-standard        - Latency vs
    soundness         accuracy                representation        Target Time
  - Schema          - Sign / transformation - Multi-concept       - Fluency &
    recognition       integrity               coupling              automaticity
```

1. **Concept**: Ability to identify governing principles, recall relevant definitions/theorems, and select the correct solution strategy.
2. **Execution**: Ability to execute arithmetic, algebraic transformations, balancing, and unit substitutions without calculation slips.
3. **Transfer**: Ability to solve isomorphic, structural, and contextual variants where surface features differ from standard textbook templates.
4. **Speed**: Time efficiency relative to target benchmarks ($T_{\text{actual}} \le T_{\text{target}}$).

---

### 4.2 Speed-Accuracy Quadrant Model

Plotting response time against correctness yields the **Speed-Accuracy Quadrant Model** implemented in `ts/reviewer/procedural.ts:704-735` and `rslib/procedural/src/skills/signals.rs`:

```
                 Accuracy (High)
                       ▲
                       │
       QUADRANT II     │      QUADRANT I
    Speed Opportunity  │   Fluency Strength
    (Accurate, Slow)   │   (Accurate, Fast)
                       │
 ──────────────────────┼──────────────────────► Latency (Fast)
                       │
       QUADRANT IV     │     QUADRANT III
     Concept / Setup   │    Strategy / Trap
    (Incorrect, Slow)  │   (Incorrect, Fast)
                       │
                 Accuracy (Low)
```

| Quadrant | Accuracy | Speed | Diagnostic Classification | Prescribed Remediation |
|:---|:---|:---|:---|:---|
| **Q1: Fluency Strength** | Correct | $\le T_{\text{target}}$ | High automaticity; robust schema mastery. | Advance to higher difficulty / transfer variants. |
| **Q2: Speed Opportunity** | Correct | $> T_{\text{target}}$ | Method sound, but high cognitive load / friction. | Fluency drills, short-cut strategies, timed practice. |
| **Q3: Strategy / Trap** | Incorrect| $\le T_{\text{target}}$ | Impulsive slip, calculation error, or distractor trap. | Error analysis, trap checking, sign verification. |
| **Q4: Concept / Setup Gap**| Incorrect| $> T_{\text{target}}$ | Fundamental gap in theoretical understanding. | Stepwise remediation, worked examples, prerequisite review. |

---

### 4.3 Four-Tier Hierarchical Diagnostic Reporting

A diagnostic test must produce actionable, structured insights that pinpoint exact learner deficits rather than an uninformative total percentage score:

```
Subject (e.g., Mathematics)
  └── Chapter (e.g., Algebra)
        └── Topic (e.g., Linear Equations)
              └── Problem Family (e.g., Two-Step Linear Equations)
```

#### Diagnostic Report Structure (`rslib/procedural/src/exam/mock.rs:117-175`):
```json
{
  "session_id": "diag-20260824-001",
  "total_questions": 16,
  "accuracy": 0.8125,
  "total_time_spent_ms": 324000,
  "hierarchy": [
    {
      "id": "domain.math",
      "name": "Mathematics",
      "level": "subject",
      "total_questions": 4,
      "correct_count": 3,
      "accuracy": 0.75,
      "concept_errors": 0,
      "calculation_errors": 1,
      "transfer_errors": 0,
      "speed_deficits": 1,
      "children": [
        {
          "id": "chapter.math.algebra",
          "name": "Algebra",
          "level": "chapter",
          "children": [
            {
              "id": "topic.math.algebra.linear_eq",
              "name": "Linear Equations",
              "level": "topic",
              "children": [
                {
                  "id": "family.math.algebra.linear_equations",
                  "name": "Two-Step Linear Equations",
                  "level": "problem_family",
                  "accuracy": 1.0,
                  "mean_time_ms": 18200.0
                }
              ]
            }
          ]
        }
      ]
    }
  ],
  "error_distribution": {
    "concept_count": 1,
    "calculation_count": 1,
    "transfer_count": 1,
    "speed_deficit_count": 2
  },
  "weak_skills": ["chemistry.thermodynamics.entropy"],
  "slow_skills": ["reasoning.seating_circular"],
  "recommended_follow_up": {
    "scope": "weak_skills_targeted",
    "target_count": 5
  }
}
```

---

### 4.4 Mixed-Domain Sampling & Bounded Session Engine

#### "Measure Mode" Design Principles:
1. **No Mid-Test Adaptive Disruptions**:
   - Standard practice sessions adapt difficulty dynamically after every card.
   - Diagnostic mock sessions MUST maintain an immutable, pre-generated question blueprint (10–20 items) across Mathematics, Reasoning, Physics, and Chemistry.
   - Preserves psychometric validity and standardized scoring.
2. **Balanced Domain Allocation**:
   - 4 Domain Distribution: $25\%$ Mathematics, $25\%$ Reasoning, $25\%$ Physics, $25\%$ Chemistry.
   - Bounded Time Budget: e.g. 15–20 minutes total with per-item target latency pacing indicators.
3. **Direct Learner Evidence Synchronization**:
   - Diagnostic outcomes update existing `SkillState`, `MasteryEvidence`, and `DomainEvidence` structures (`rslib/procedural/src/skills/`).
   - Does NOT create parallel or disconnected learner state databases.

---

## 5. Reasoning Assessment & Logic Failure Diagnostics

### 5.1 Taxonomy of Reasoning Failures

Reasoning problems (linear/circular seating arrangements, floor/grid puzzles, syllogisms, logic DAGs, series completions) fail differently than calculation problems. The taxonomy in `rslib/procedural/src/reasoning/diagnostics.rs` establishes 11 discrete failure modes:

```
                               ┌────────────────────────┐
                               │ REASONING ERROR MODES  │
                               └───────────┬────────────┘
                                           │
  ┌───────────────────────┬────────────────┼───────────────────────┬───────────────────────┐
  ▼                       ▼                ▼                       ▼                       ▼
[ Schema Recognition ]  [ Representation ] [ Constraint Violation ] [ Inference Leap ]   [ Search Case Error ]
- Confused circular     - Inverted grid   - Violated negative     - Affirmed consequent   - Missed branching
  vs linear arrangement   matrix mapping    condition ("not next")  without justification   possibility
```

1. **Schema Recognition Error**: Failed to classify structural archetype (e.g. treated a bidirectional circular arrangement as a fixed unilateral line).
2. **Strategy Selection Error**: Chose an inefficient entry point or weak initial anchor variable.
3. **Representation / Structural Error**: Constructed a flawed diagram, incorrect slot numbering, or invalid logical DAG representation.
4. **Constraint Application Error**: Ignored or violated an explicit condition (e.g. placed $A$ adjacent to $B$ when prompt specified *"A does not sit next to B"*).
5. **Inference Error**: Made an invalid logical deduction or unjustified relational leap not supported by premises.
6. **Search Case Error**: Failed to evaluate parallel branching cases in constraint satisfaction or terminated search prematurely.
7. **Contradiction Handling Error**: Failed to recognize an impossible assignment or resolve proof-by-contradiction.
8. **Reading Trap Error**: Fell for subtle linguistic phrasing (*"only if"*, *"neither...nor"*, inverse qualifiers).
9. **Execution Slip**: Logical deduction sound, but clerical counting or transcription slip occurred.
10. **Time Error**: Logical solution correct, but latency exceeded target pacing.
11. **Unknown**: Unclassified failure.

---

### 5.2 Structured Reasoning UI & Immediate Feedback

When a reasoning failure occurs:
- The system pinpoints the exact premise or constraint that was violated.
- Renders the valid constraint graph / slot allocation beside the student's erroneous assignment.
- Offers an immediate "Try Similar Structural Variant" bridge to reinforce correct diagrammatic representation before memory decay occurs.

---

## 6. Synthesis & Architectural Recommendations for Reconciliation

| Component | Current State | Target Reconciled Architecture | Rationale & Impact |
|:---|:---|:---|:---|
| **MCQ Modality** | Mixed representation, text input legacy risk | Authentic `.proc-option-group` with 1–4 / A–D keyboard selection, canonical identity evaluation, no text input | Eliminates artificial typing friction, ensures 100% exam fidelity |
| **Numerical Modality** | Basic float parsing | Dedicated input supporting negative numbers, fractions, scientific notation, equation prefixes, and dimensional vectors | Prevents NaN errors, verifies physical/chemical unit integrity |
| **Stepwise Modality** | Standalone TS logic | Semantic step evaluation wired directly to Rust `StepValidator` via FFI bridge | Single source of truth, eliminates duplicate TS math engine sprawl |
| **Reviewer Footer** | Standard Anki footer or detached container | Compact mistake footer `[1 Silly]`, `[2 Pattern]`, `[3 Concept]`, `[4 Unknown]` in native footer interaction zone | Natural review ergonomics, no disruptive screen scrolling |
| **Standard Cards** | Shared webview environment | Strict isolation for Basic, Cloze, and non-procedural cards | Zero regressions for standard Anki flashcards and core shortcuts |
| **Diagnostic Engine** | Ad-hoc practice sessions | 10–20 item fixed blueprint across 4 domains with hierarchical reporting (Subject $\to$ Chapter $\to$ Topic $\to$ Family) | Authentic mock simulation and granular skill deficit diagnostics |
| **Learner State** | Multiple prototype models | Unified sync to `SkillState`, `MasteryEvidence`, and `DomainEvidence` | Clean architectural coherence without parallel redundant data stores |

---

## 7. Verification & Audit Evidence Sources

1. **Anki Core Reviewer Architecture**:
   - `qt/aqt/reviewer.py`: Reviewer lifecycle, dual webview management, `_showQuestion()`, `_showAnswer()`, `_answerCard()`, `_shortcutKeys()`.
   - `qt/aqt/webview.py`: `AnkiWebView`, `AnkiWebViewKind`, `QWebChannel` transport script.
   - `pylib/anki/hooks.py` & `qt/tools/genhooks_gui.py`: Reviewer lifecycle hooks.
   - `ts/reviewer/index.ts` & `ts/reviewer/procedural.ts`: Webview DOM renderer, keyboard event interceptor, procedural state machine.
2. **Procedural Rust Core (`rslib/procedural/src/`)**:
   - `reviewer/template.rs`: Safe HTML/JS template rendering, XSS sanitization.
   - `physics/units.rs` & `chemistry/units.rs`: Dimensional vectors, unit parsing, scale conversions.
   - `reasoning/diagnostics.rs`: 11-category reasoning failure taxonomy.
   - `exam/mock.rs`: Bounded diagnostic mock engine, 4-tier hierarchical report generator.
   - `skills/domain_evidence.rs` & `skills/signals.rs`: `DomainEvidencePayload`, speed-accuracy quadrant models.
3. **Live Desktop QA Harness**:
   - `tools/forensic_reviewer.py` & `tools/test_live_reviewer.py`: QtWebEngine remote debugging (port 9222) CDP verification suite.
