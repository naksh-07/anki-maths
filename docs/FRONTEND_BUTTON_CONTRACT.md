# StudyLab Frontend Button & Control Contract

**Document Version:** 1.0.0 (Canonical)  
**Target Subsystem:** Reviewer UI Controls (`ts/reviewer/`), Template Elements (`rslib/procedural/src/reviewer/template.rs`), and Desktop Reviewer Shell (`qt/aqt/reviewer.py`)  
**Status:** AUTHORITATIVE CANONICAL SPECIFICATION (Mission Section 7)  
**Integrity Mode:** 100% Grounded in Executable Source Code, Live UI Audits, and Operational Invariants  

---

## 1. Executive Summary & Control Surface Invariants

Every button, input, toggle, and interactive element in the StudyLab Reviewer is governed by strict pedagogical, ergonomic, and architectural rules.

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                           ONE-INTERACTION-SURFACE RULE                           │
│                                                                                  │
│   "At any given state, there is exactly ONE primary semantic action.             │
│    Duplicate or competing controls performing identical or conflicting          │
│    actions are strictly forbidden. Modality controls must be mutually            │
│    exclusive with fallback inputs."                                              │
└──────────────────────────────────────────────────────────────────────────────────┘
```

### Key Principles:
1. **Zero Text Input Fallback:** For structured choice modalities (`mcq`, `concept_check`, `strategy_drill`, `worked_example`), the text input field `#proc-answer-input` and mode switch tabs are strictly absent.
2. **Anti-Bypass Lock:** Error reflection controls (`[1 Silly]`, `[2 Pattern]`, `[3 Concept]`, `[4 Unknown]`) block card progression until one category is selected.
3. **Coordinated Anki Ownership:** Standard Anki ease buttons (`Again`, `Hard`, `Good`, `Easy`) and `#ansbut` are suppressed on procedural cards to prevent duplicate rating bars and state desynchronization. Card advance is driven by StudyLab's `#proc-next-btn`, with hotkey overrides (`1`..`4`) preserved for power users.

---

## 2. Canonical Master Button Matrix

Below is the comprehensive matrix specifying every single control in the StudyLab universe:

| # | Exact Label / Icon | Control Selector | Ownership | Priority | Applicable Objects | Active States | Keyboard Shortcut | State Transition | Dispatched IPC Event |
|---|---|---|---|---|---|---|---|---|---|
| 1 | `Quick Solve` | `#proc-tab-quick` | StudyLab | Subdued Tab | `problem` (with stepwise) | `ready`, `solving` | `Alt+Q` | Switches to quick input | None |
| 2 | `Step-by-Step Solve` | `#proc-tab-stepwise` | StudyLab | Subdued Tab | `problem` (with stepwise) | `ready`, `solving` | `Alt+S` | Switches to stepwise | None |
| 3 | `Submit Answer` | `#proc-submit-btn` | StudyLab | **Primary CTA** | `problem`, `quick`, `numerical` | `ready`, `solving` | `Enter`, `Ctrl+Enter` | `solving` $\rightarrow$ `submitting` | Telemetry package prep |
| 4 | `Check Solution` | `#proc-check-steps-btn` | StudyLab | **Primary CTA** | `stepwise`, `problem` (stepwise) | `step_answering` | `Ctrl+Enter` | `step_answering` $\rightarrow$ `submitting` | `procedural_validate_steps` |
| 5 | `+ Add Step` | `#proc-add-step-btn` | StudyLab | Secondary | `stepwise`, `problem` (stepwise) | `step_answering` | `Alt+A` / `Enter` in row | Adds new step row | None |
| 6 | `💡 Request Hint` | `#proc-hint-btn` | StudyLab | Secondary | `problem`, `quick`, `stepwise`, `mcq` | `ready`, `solving`, `step_answering` | `H`, `?` | `solving` $\rightarrow$ `hint` | `procedural_hint:<json>` |
| 7 | `↺ Reset Workspace` | `#proc-reset-btn` | StudyLab | Ghost / Utility | `stepwise`, `problem` (stepwise) | `step_answering` | `Alt+R` | Clears step inputs | None |
| 8 | `Try Similar Problem` | `#proc-try-similar-btn` | StudyLab | Secondary / Remedial | All procedural objects | `feedback`, `worked_example` | `Alt+T` | `feedback` $\rightarrow$ `ready` (new seed) | `procedural_try_similar:<json>` |
| 9 | `Next Problem (Space / Enter)` | `#proc-next-btn` | StudyLab | **Primary CTA** | All procedural objects | `feedback`, `diagnostic_summary` | `Space`, `Enter` | `feedback` $\rightarrow$ `next` | `procedural_answer:<ease>` |
| 10 | `[1 Silly]` / `1 Silly Slip` | `.proc-mistake-btn[data-key="1"]` | StudyLab | **Primary Reflection** | All procedural objects (on error) | `mistake_classification` | `1` | `mistake` $\rightarrow$ `feedback` | `procedural_mistake:<json>` |
| 11 | `[2 Pattern]` / `2 Pattern Missed`| `.proc-mistake-btn[data-key="2"]` | StudyLab | **Primary Reflection** | All procedural objects (on error) | `mistake_classification` | `2` | `mistake` $\rightarrow$ `feedback` | `procedural_mistake:<json>` |
| 12 | `[3 Concept]` / `3 Concept Gap` | `.proc-mistake-btn[data-key="3"]` | StudyLab | **Primary Reflection** | All procedural objects (on error) | `mistake_classification` | `3` | `mistake` $\rightarrow$ `feedback` | `procedural_mistake:<json>` |
| 13 | `[4 Unknown]` / `4 Prereq Unknown`| `.proc-mistake-btn[data-key="4"]` | StudyLab | **Primary Reflection** | All procedural objects (on error) | `mistake_classification` | `4` | `mistake` $\rightarrow$ `feedback` | `procedural_mistake:<json>` |
| 14 | `Again` (Ease 1) | Native bottom ease bar | Native Anki | Native Ease | Standard Anki cards only | Native answer review | `1` | Advances with Ease 1 | Native Anki rating |
| 15 | `Hard` (Ease 2) | Native bottom ease bar | Native Anki | Native Ease | Standard Anki cards only | Native answer review | `2` | Advances with Ease 2 | Native Anki rating |
| 16 | `Good` (Ease 3) | Native bottom ease bar | Native Anki | Native Ease | Standard Anki cards only | Native answer review | `3` | Advances with Ease 3 | Native Anki rating |
| 17 | `Easy` (Ease 4) | Native bottom ease bar | Native Anki | Native Ease | Standard Anki cards only | Native answer review | `4` | Advances with Ease 4 | Native Anki rating |
| 18 | `Show Answer` | `#ansbut` (Bottom bar) | Native Anki | Native Action | Standard Anki cards only | Standard question state | `Space` | Shows standard answer | Native Anki flip |
| 19 | `More` | Bottom bar more menu | Native Anki | Utility Menu | All cards (Anki shell) | All states | `M` | Opens context menu | Native Anki popup |
| 20 | `Practice Prerequisite` | `#proc-practice-prereq-btn` | StudyLab | Remedial CTA | Objects with prereq links | `feedback` | `Alt+P` | Navigates to prereq card | `procedural_practice_prerequisite:<json>` |
| 21 | `Review in Anki` | `#proc-declarative-recall-btn` | StudyLab | Remedial CTA | `declarative_recall`, `feedback` | `feedback` | `Alt+R` | Resolves target Anki card | `procedural_declarative_recall:<json>` |
| 22 | `[ ✔ I Have Understood ]` | `#proc-worked-ack-btn` | StudyLab | **Primary Gate** | `worked_example` | `worked_step_reveal` | `Enter`, `Space` | `worked_step_reveal` $\rightarrow$ `ready` | `procedural_try_similar:<json>` |
| 23 | Option Card (`A`..`D`) | `.proc-option-item` | StudyLab | Interactive Choice | `mcq`, `concept_check`, `strategy_drill` | `ready`, `solving`, `mcq_selected` | `1`..`4`, `A`..`D`, Arrows | Updates selection / submits | Local selection update |

---

## 3. Exhaustive Individual Control Specifications

---

### 3.1 `Quick Solve` Tab (`#proc-tab-quick`)
- **Exact Label & Visual:** `Quick Solve` (Subdued pill tab button, active state highlighted with accent border).
- **Pedagogical Purpose:** Enables the student to bypass multi-step derivation when they have high cognitive fluency and wish to provide the final numerical answer directly.
- **Applicable Learning Objects:** `problem` (where procedural contract specifies both quick and stepwise modes).
- **Active States:** `ready`, `solving`.
- **Visual Priority:** Subdued navigation tab (`.proc-tab-btn`).
- **Ownership:** StudyLab Frontend (`ts/reviewer/procedural.ts:350`).
- **DOM Selector:** `#proc-tab-quick` inside `.proc-mode-tabs`.
- **Keyboard Shortcut:** `Alt+Q`.
- **State Transition:** Activates `#proc-quick-container`, focuses `#proc-answer-input`, hides `#proc-stepwise-container`.
- **Dispatched IPC Event:** None.
- **Coexistence Rules:** May coexist with `#proc-tab-stepwise`, `#proc-hint-btn`, `#proc-submit-btn`.
- **Forbidden Combinations:** **STRICTLY FORBIDDEN** on `mcq`, `concept_check`, `strategy_drill`, `worked_example`.

---

### 3.2 `Step-by-Step Solve` Tab (`#proc-tab-stepwise`)
- **Exact Label & Visual:** `Step-by-Step` (Subdued pill tab button).
- **Pedagogical Purpose:** Opens the multi-step algebraic derivation workspace when a student wants structured intermediate validation.
- **Applicable Learning Objects:** `problem` (where procedural contract specifies stepwise support).
- **Active States:** `ready`, `solving`.
- **Visual Priority:** Subdued navigation tab (`.proc-tab-btn`).
- **Ownership:** StudyLab Frontend (`ts/reviewer/procedural.ts:351`).
- **DOM Selector:** `#proc-tab-stepwise` inside `.proc-mode-tabs`.
- **Keyboard Shortcut:** `Alt+S`.
- **State Transition:** Activates `#proc-stepwise-container`, hides `#proc-quick-container`, sets state to `step_answering`.
- **Dispatched IPC Event:** None.
- **Coexistence Rules:** May coexist with `#proc-tab-quick`, `#proc-hint-btn`, `#proc-check-steps-btn`.
- **Forbidden Combinations:** **STRICTLY FORBIDDEN** on `mcq`, `concept_check`, `strategy_drill`, `worked_example`.

---

### 3.3 `Submit Answer` Button (`#proc-submit-btn`)
- **Exact Label & Visual:** `Submit Answer` (Solid primary accent button, 14px bold, elevated on hover).
- **Pedagogical Purpose:** Confirms completion of quick numerical/symbolic entry and triggers semantic validation.
- **Applicable Learning Objects:** `problem`, `quick`, numerical calculations.
- **Active States:** `ready`, `solving`.
- **Visual Priority:** **Primary CTA** (Unique primary action on the screen).
- **Ownership:** StudyLab Frontend (`ts/reviewer/procedural.ts:354-365`).
- **DOM Selector:** `#proc-submit-btn` inside `#proc-quick-container`.
- **Keyboard Shortcut:** `Enter` (when input is focused), `Ctrl+Enter`.
- **State Transition:** `solving` $\rightarrow$ `submitting` $\rightarrow$ (`feedback` if correct, `mistake_classification` if incorrect).
- **Dispatched IPC Event:** Packages attempt telemetry for bridge dispatch.
- **Coexistence Rules:** Coexists with `#proc-answer-input`, `#proc-hint-btn`.
- **Forbidden Combinations:** Must **NOT** coexist with `#proc-check-steps-btn`, `.proc-option-group`, `#proc-next-btn`, `#ansbut`.

---

### 3.4 `Check Solution` Button (`#proc-check-steps-btn`)
- **Exact Label & Visual:** `Check Solution` (Solid primary accent button with checkmark icon).
- **Pedagogical Purpose:** Evaluates all submitted derivation steps in the Stepwise workspace against the Rust `StepValidator` graph.
- **Applicable Learning Objects:** `stepwise`, `problem` (in stepwise mode).
- **Active States:** `step_answering`.
- **Visual Priority:** **Primary CTA** for Stepwise workspace.
- **Ownership:** StudyLab Stepwise Container (`ts/reviewer/components/stepwise_container.ts:310-330`).
- **DOM Selector:** `#proc-check-steps-btn` inside `.proc-stepwise-controls`.
- **Keyboard Shortcut:** `Ctrl+Enter`.
- **State Transition:** `step_answering` $\rightarrow$ `submitting` $\rightarrow$ (`feedback` / `mistake_classification`).
- **Dispatched IPC Event:** Emits `procedural_validate_steps:<json>`.
- **Coexistence Rules:** Coexists with `#proc-add-step-btn`, `#proc-hint-btn`, `#proc-reset-btn`.
- **Forbidden Combinations:** Must **NOT** coexist with `#proc-submit-btn`, `#proc-quick-container`, `.proc-option-group`.

---

### 3.5 `+ Add Step` Button (`#proc-add-step-btn`)
- **Exact Label & Visual:** `+ Add Step` (Secondary outline button).
- **Pedagogical Purpose:** Appends a new intermediate algebraic derivation row to the Stepwise workspace.
- **Applicable Learning Objects:** `stepwise`, `problem` (stepwise mode).
- **Active States:** `step_answering`.
- **Visual Priority:** Secondary Action.
- **Ownership:** StudyLab Stepwise Container (`ts/reviewer/components/stepwise_container.ts:280-305`).
- **DOM Selector:** `#proc-add-step-btn` inside `.proc-stepwise-controls`.
- **Keyboard Shortcut:** `Alt+A`, or pressing `Enter` in the last step input row.
- **State Transition:** Appends DOM row `.proc-step-row` and focuses the new input.
- **Dispatched IPC Event:** None.
- **Coexistence Rules:** Coexists with `#proc-check-steps-btn`, `#proc-hint-btn`, `#proc-reset-btn`.
- **Forbidden Combinations:** Must **NOT** appear in non-stepwise modalities.

---

### 3.6 `💡 Request Hint` Button (`#proc-hint-btn`)
- **Exact Label & Visual:** `💡 Request Hint` (Subdued icon button with amber bulb).
- **Pedagogical Purpose:** Provides progressive 3-tier scaffolding (Principle $\rightarrow$ Operation $\rightarrow$ Intermediate Relation) when the student is stuck.
- **Applicable Learning Objects:** `problem`, `quick`, `stepwise`, `mcq`.
- **Active States:** `ready`, `solving`, `step_answering`.
- **Visual Priority:** Secondary / Scaffolding Action.
- **Ownership:** StudyLab Frontend (`ts/reviewer/procedural.ts:630-660`).
- **DOM Selector:** `#proc-hint-btn`.
- **Keyboard Shortcut:** `H`, `?`.
- **State Transition:** `solving` $\rightarrow$ `hint` (reveals `#proc-hint-box`).
- **Dispatched IPC Event:** Emits `bridgeCommand("procedural_hint:<json>")`.
- **Coexistence Rules:** Coexists with active inputs, submit buttons, mode switchers.
- **Forbidden Combinations:** Must **NOT** appear during `mistake_classification` or `feedback`.

---

### 3.7 `↺ Reset Workspace` Button (`#proc-reset-btn`)
- **Exact Label & Visual:** `↺ Reset` (Ghost utility button).
- **Pedagogical Purpose:** Clears all entered derivation steps and restores initial empty workspace.
- **Applicable Learning Objects:** `stepwise`, `problem` (stepwise mode).
- **Active States:** `step_answering`.
- **Visual Priority:** Ghost / Utility.
- **Ownership:** StudyLab Stepwise Container (`ts/reviewer/components/stepwise_container.ts:340-355`).
- **DOM Selector:** `#proc-reset-btn`.
- **Keyboard Shortcut:** `Alt+R`.
- **State Transition:** Resets step rows to step 0.
- **Dispatched IPC Event:** None.
- **Coexistence Rules:** Coexists with `#proc-add-step-btn`, `#proc-check-steps-btn`.
- **Forbidden Combinations:** Must **NOT** appear in quick or MCQ modes.

---

### 3.8 `Try Similar Problem` Button (`#proc-try-similar-btn`)
- **Exact Label & Visual:** `Try Similar Problem` (Secondary outline button with refresh icon).
- **Pedagogical Purpose:** Generates an immediate freshly parameterized problem instance of the same problem family for transfer practice.
- **Applicable Learning Objects:** All procedural objects (especially after incorrect attempt or worked example review).
- **Active States:** `feedback`, `worked_example`.
- **Visual Priority:** Secondary Remedial CTA.
- **Ownership:** StudyLab Frontend (`ts/reviewer/procedural.ts:1192-1200`).
- **DOM Selector:** `#proc-try-similar-btn` inside `#proc-result-panel`.
- **Keyboard Shortcut:** `Alt+T`.
- **State Transition:** Reloads question surface with newly seeded parameters (`feedback` $\rightarrow$ `ready`).
- **Dispatched IPC Event:** Emits `bridgeCommand("procedural_try_similar:<json>")`.
- **Coexistence Rules:** Coexists with `#proc-next-btn`, `#proc-practice-prereq-btn`.
- **Forbidden Combinations:** Must **NOT** appear during active solving.

---

### 3.9 `Next Problem` Button (`#proc-next-btn`)
- **Exact Label & Visual:** `Next Problem (Space / Enter)` (Solid primary accent button, 14px bold, prominent in result card).
- **Pedagogical Purpose:** Advances to the next scheduled card in the Anki review queue with the engine-calibrated FSRS ease rating.
- **Applicable Learning Objects:** All procedural objects.
- **Active States:** `feedback`, `diagnostic_summary`.
- **Visual Priority:** **Primary CTA** on feedback screens.
- **Ownership:** StudyLab Frontend (`ts/reviewer/procedural.ts:1221-1230`).
- **DOM Selector:** `#proc-next-btn` inside `#proc-result-panel`.
- **Keyboard Shortcut:** `Space`, `Enter`.
- **State Transition:** `feedback` $\rightarrow$ `next` $\rightarrow$ `teardown`.
- **Dispatched IPC Event:** Emits `bridgeCommand("procedural_answer:<ease>")`.
- **Coexistence Rules:** Coexists with `#proc-try-similar-btn`, `#proc-practice-prereq-btn`.
- **Forbidden Combinations:** Must **NOT** appear during `solving`, `submitting`, or `mistake_classification`. Native bottom ease buttons (`Again`, `Hard`, `Good`, `Easy`) must remain suppressed to prevent duplicate Next controls.

---

### 3.10 Mistake Classification Buttons (`[1 Silly]`, `[2 Pattern]`, `[3 Concept]`, `[4 Unknown]`)
- **Exact Labels & Visuals:**
  - `[1 Silly]` / `Silly Slip`: Calculation or sign slip (`.proc-mistake-btn[data-key="1"]`).
  - `[2 Pattern]` / `Pattern Missed`: Unrecognized problem structure (`.proc-mistake-btn[data-key="2"]`).
  - `[3 Concept]` / `Concept Gap`: Wrong formula or misapplied theorem (`.proc-mistake-btn[data-key="3"]`).
  - `[4 Unknown]` / `Prereq Unknown`: Missing fundamental prerequisite (`.proc-mistake-btn[data-key="4"]`).
- **Pedagogical Purpose:** Metacognitive reflection gate. Captures student error attribution for Bayesian Knowledge Tracing and FSRS calibration.
- **Applicable Learning Objects:** All procedural objects (upon incorrect submission).
- **Active States:** `mistake_classification`.
- **Visual Priority:** **Exclusive Primary Reflection Grid** (4 equal-weight cards).
- **Ownership:** StudyLab Mistake Footer (`ts/reviewer/components/mistake_footer.ts:25-60`).
- **DOM Selector:** `.proc-mistake-btn` / `.proc-mistake-card` inside `#proc-mistake-panel`.
- **Keyboard Shortcut:** `1`, `2`, `3`, `4`.
- **State Transition:** `mistake_classification` $\rightarrow$ `feedback` (150ms delay with visual highlight).
- **Dispatched IPC Event:** Emits `bridgeCommand("procedural_mistake:<json>")`.
- **Coexistence Rules:** Coexist exclusively with each other inside `#proc-mistake-panel`.
- **Forbidden Combinations:** `#proc-next-btn`, `#ansbut`, and native ease buttons are **STRICTLY FORBIDDEN** until classification is complete.

---

### 3.11 Native Anki Ease Buttons (`Again`, `Hard`, `Good`, `Easy`)
- **Exact Labels & Visuals:** `Again (1)`, `Hard (2)`, `Good (3)`, `Easy (4)` (Native Anki bottom toolbar buttons).
- **Pedagogical Purpose:** Standard Anki flashcard rating interface.
- **Applicable Learning Objects:** Standard Anki Basic/Cloze cards **ONLY**.
- **Active States:** Standard Anki answer state (`Reviewer.state == "answer"`).
- **Visual Priority:** Primary Anki rating bar.
- **Ownership:** Native Anki Desktop (`qt/aqt/reviewer.py:1037-1055`).
- **DOM Selector:** Bottom webview `#outer` ease buttons.
- **Keyboard Shortcut:** `1`, `2`, `3`, `4`.
- **State Transition:** Executes `Reviewer._answerCard(val)`.
- **Dispatched IPC Event:** `ease1`, `ease2`, `ease3`, `ease4`.
- **Coexistence Rules:** Coexists with `Edit`, `More`, and standard Anki answer content.
- **Forbidden Combinations:** **STRICTLY SUPPRESSED ON STUDYLAB PROCEDURAL CARDS**. On procedural cards, the bottom ease buttons are hidden via `Reviewer._showEaseButtons()` suppression to maintain the One-Interaction-Surface invariant.

---

### 3.12 Native Anki `Show Answer` Button (`#ansbut`)
- **Exact Label & Visual:** `Show Answer` (Bottom bar spacebar button).
- **Pedagogical Purpose:** Reveals declarative answer on standard Anki flashcards.
- **Applicable Learning Objects:** Standard Anki Basic/Cloze cards **ONLY**.
- **Active States:** Standard Anki question state (`Reviewer.state == "question"`).
- **Visual Priority:** Primary Anki flip button.
- **Ownership:** Native Anki Desktop (`qt/aqt/reviewer.py:985-1004`).
- **DOM Selector:** Bottom webview `#ansbut`.
- **Keyboard Shortcut:** `Space`.
- **State Transition:** Executes `Reviewer._showAnswer()`.
- **Dispatched IPC Event:** `ans`.
- **Coexistence Rules:** Coexists with deck due counts on standard cards.
- **Forbidden Combinations:** **STRICTLY SUPPRESSED ON STUDYLAB PROCEDURAL CARDS**. Replaced with deck progress counts via `Reviewer._showAnswerButton()`. If triggered via menu, intercepted by `Reviewer._getTypedAnswer()` $\rightarrow$ `handleNativeShowAnswer()` to prevent DOM destruction.

---

### 3.13 Native Anki `More` Menu (`pycmd("more")`)
- **Exact Label & Visual:** `More ▾` (Subdued text button in bottom bar).
- **Pedagogical Purpose:** Accesses Anki card management utilities (Bury Card, Suspend Card, Delete Note, Options).
- **Applicable Learning Objects:** All cards (Anki desktop shell utility).
- **Active States:** All states.
- **Visual Priority:** Utility / Context Menu.
- **Ownership:** Native Anki Desktop (`qt/aqt/reviewer.py:725-726, 964-967`).
- **DOM Selector:** Bottom webview `button[onclick*="more"]`.
- **Keyboard Shortcut:** `M`.
- **State Transition:** Opens native Qt context popup menu.
- **Dispatched IPC Event:** `pycmd("more")`.
- **Coexistence Rules:** Coexists with all review screens in the bottom bar.
- **Forbidden Combinations:** None.

---

### 3.14 `Practice Prerequisite` Button (`#proc-practice-prereq-btn`)
- **Exact Label & Visual:** `📚 Practice Prerequisite: {Skill Name}` (Subdued remedial pill button).
- **Pedagogical Purpose:** Navigates to a diagnostic or foundational card for a missing prerequisite identified during error classification.
- **Applicable Learning Objects:** Procedural objects with mapped prerequisite skills.
- **Active States:** `feedback` (when mistake category is `concept_not_known` or prerequisite evidence is triggered).
- **Visual Priority:** Secondary Remedial Action.
- **Ownership:** StudyLab Frontend (`ts/reviewer/procedural.ts:1214-1218`).
- **DOM Selector:** `#proc-practice-prereq-btn` inside `#proc-result-panel`.
- **Keyboard Shortcut:** `Alt+P`.
- **State Transition:** Triggers prerequisite card loading.
- **Dispatched IPC Event:** Emits `bridgeCommand("procedural_practice_prerequisite:<json>")`.
- **Coexistence Rules:** Coexists with `#proc-next-btn`, `#proc-try-similar-btn`.
- **Forbidden Combinations:** Forbidden during active solving.

---

### 3.15 `Worked Example Acknowledge` Button (`#proc-worked-ack-btn`)
- **Exact Label & Visual:** `[ ✔ I Have Reviewed and Understood This Solution ]` (Solid primary green button, 14px bold).
- **Pedagogical Purpose:** Mandatory metacognitive gate confirming the learner has studied the expert solution trace before receiving a fresh transfer variant.
- **Applicable Learning Objects:** `worked_example`.
- **Active States:** `worked_step_reveal`.
- **Visual Priority:** **Primary Gate CTA**.
- **Ownership:** StudyLab Frontend (`ts/reviewer/procedural.ts:1192-1200`).
- **DOM Selector:** `#proc-worked-ack-btn` / `#proc-try-similar-btn`.
- **Keyboard Shortcut:** `Enter`, `Space`.
- **State Transition:** `worked_step_reveal` $\rightarrow$ `ready` (loads newly seeded transfer retry instance).
- **Dispatched IPC Event:** Emits `bridgeCommand("procedural_try_similar:<json>")`.
- **Coexistence Rules:** Coexists exclusively with the worked example trace cards.
- **Forbidden Combinations:** Must **NOT** coexist with solving inputs or MCQ options.

---

### 3.16 Option Cards (`.proc-option-item`)
- **Exact Label & Visual:** Discrete radio cards with option key (`A`, `B`, `C`, `D`) and formatted text/MathJax label.
- **Pedagogical Purpose:** Provides zero-text input choice selection for conceptual, reasoning, and strategic problems.
- **Applicable Learning Objects:** `mcq`, `concept_check`, `strategy_drill`.
- **Active States:** `ready`, `solving`, `mcq_selected`.
- **Visual Priority:** Interactive Selection Group (`role="radiogroup"`).
- **Ownership:** StudyLab MCQ Container (`ts/reviewer/components/mcq_container.ts:100-240`).
- **DOM Selector:** `.proc-option-item` inside `.proc-option-group`.
- **Keyboard Shortcut:** `1`..`4`, `A`..`D`, `ArrowUp` / `ArrowDown` (with roving tabindex).
- **State Transition:** Focuses and selects option (`mcq_selected`); in practice mode, submits selection.
- **Dispatched IPC Event:** None until evaluation.
- **Coexistence Rules:** Coexists with `#proc-hint-btn`.
- **Forbidden Combinations:** **STRICTLY FORBIDDEN** to coexist with `#proc-answer-input`, `#proc-quick-container`, or `#proc-stepwise-container`.

---

## 4. Mutually Exclusive Control Sets

To guarantee zero UI ambiguity and prevent conflicting user actions, the following control sets are strictly mutually exclusive:

### Control Set A: Input Modality Surfaces
At any instant, exactly **one** of the following modality surfaces may exist in the DOM:
- **Set A1 (Numerical Quick Input):** `#proc-quick-container` (`#proc-answer-input`, `#proc-submit-btn`).
- **Set A2 (Stepwise Workspace):** `#proc-stepwise-container` (`.proc-step-row`, `#proc-add-step-btn`, `#proc-check-steps-btn`, `#proc-reset-btn`).
- **Set A3 (Structured Choice):** `#proc-mcq-container` (`.proc-option-group`, `.proc-option-item`).
- **Set A4 (Worked Example Trace):** `#proc-worked-box` (`.proc-worked-steps`, `#proc-worked-ack-btn`).

*Enforcement Rule:* If `Set A3` is active, `Set A1`, `Set A2`, and `Set A4` are removed from the DOM.

---

### Control Set B: Submission & Evaluation Triggers
At any instant, exactly **one** submission action is enabled:
- `Submit Answer` (`#proc-submit-btn`) — Active in Numerical Quick mode only.
- `Check Solution` (`#proc-check-steps-btn`) — Active in Stepwise mode only.
- `Option Click / Hotkey (1-4)` — Active in MCQ / Choice mode only.
- `Acknowledge Solution` (`#proc-worked-ack-btn`) — Active in Worked Example mode only.

---

### Control Set C: Review Lifecycle & Bottom Bar Controls
At any instant, exactly **one** card progression mechanism exists:
- **Standard Cards:** Native `#ansbut` (Question state) OR Native `Again/Hard/Good/Easy` (Answer state).
- **Procedural Solving State:** All native bottom ease buttons and `#ansbut` **SUPPRESSED**. In-card `#proc-submit-btn` is active.
- **Procedural Feedback State:** All native bottom ease buttons **SUPPRESSED**. In-card `#proc-next-btn` is the single active progression control.

---

### Control Set D: Error Reflection vs Progression
During error processing:
- **`mistake_classification` State:** The 4 mistake buttons (`[1 Silly]`, `[2 Pattern]`, `[3 Concept]`, `[4 Unknown]`) are **MANDATORY**. `#proc-next-btn`, `#ansbut`, and native ease buttons are **STRICTLY BLOCKED**.
- **`feedback` State:** `#proc-next-btn` is **ACTIVE**. Mistake classification buttons are **REMOVED**.

---

## 5. Keyboard Navigation & Hotkey Mapping

| Key / Shortcut | Scope / State | Behavior & Routing | Trapping / Propagation |
|---|---|---|---|
| `Enter` | `solving` (Input focused) | Submits quick answer via `handleQuickSubmit()` | `e.preventDefault()` |
| `Ctrl+Enter` | `step_answering` | Submits stepwise derivation via `handleStepwiseSubmit()` | `e.preventDefault()` |
| `Space` | `solving` | Submits response (does NOT flip card) | `e.preventDefault()`, `e.stopPropagation()` |
| `Space` / `Enter` | `mistake_classification` | **Strictly Trapped** (Cannot bypass reflection) | `e.preventDefault()`, `e.stopPropagation()` |
| `Space` / `Enter` | `feedback` | Advances to next card via `handleNext()` | `e.preventDefault()` |
| `1`, `2`, `3`, `4` | `ready`, `solving` (MCQ) | Selects option A, B, C, D respectively | `e.preventDefault()` |
| `A`, `B`, `C`, `D` | `ready`, `solving` (MCQ) | Selects option A, B, C, D respectively | `e.preventDefault()` |
| `1`, `2`, `3`, `4` | `mistake_classification` | Selects Silly, Pattern, Concept, Unknown | `e.preventDefault()`, `e.stopPropagation()` |
| `1`, `2`, `3`, `4` | `feedback` | Direct ease rating override (`Again`, `Hard`, `Good`, `Easy`)| `e.preventDefault()` |
| `H`, `?` | `ready`, `solving` | Opens progressive hint card | `e.preventDefault()` |
| `Esc` | `hint` | Closes hint card and refocuses input | `e.preventDefault()` |
| `ArrowUp` / `ArrowDown` | `ready`, `solving` (MCQ) | Navigates radio option focus with roving tabindex | `e.preventDefault()` |
| `Alt+Q` | `ready`, `solving` (`problem`) | Switches mode to Quick Solve | `e.preventDefault()` |
| `Alt+S` | `ready`, `solving` (`problem`) | Switches mode to Step-by-Step | `e.preventDefault()` |
| `Alt+A` | `step_answering` | Appends a new step row | `e.preventDefault()` |
| `Alt+R` | `step_answering` | Resets step workspace | `e.preventDefault()` |
| `Alt+T` | `feedback` | Generates and loads similar problem variant | `e.preventDefault()` |
| `M` | Global Anki Shell | Opens native Anki `More` context menu | Native Anki handler |
