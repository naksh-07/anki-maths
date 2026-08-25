# StudyLab Frontend Button & Control Contract

**Document Version:** 1.1.0 (Reconciled with STUDYLAB_UI_COMPOSITION_CONTRACT.md)  
**Target Subsystem:** Reviewer UI Controls (`ts/reviewer/`), Template Elements (`rslib/procedural/src/reviewer/template.rs`), and Desktop Reviewer Shell (`qt/aqt/reviewer.py`)  
**Status:** AUTHORITATIVE CANONICAL SPECIFICATION  
**Integrity Mode:** 100% Grounded in Executable Source Code, Live UI Audits, and Operational Invariants  
**Authoritative Reference:** `docs/STUDYLAB_UI_COMPOSITION_CONTRACT.md`, `PROJECT.md`

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
2. **Anti-Bypass Reflection Lock:** Error reflection controls (`[1 Silly Slip]`, `[2 Pattern Missed]`, `[3 Concept Gap]`, `[4 Prereq Unknown]`) block card progression until one category is selected. Space/Enter keys are trapped.
3. **Deferred Solution Reveal (`ANTI-08`):** Full solution derivation remains hidden during mistake classification.
4. **Coordinated Anki Ownership:** Standard Anki ease buttons (`Again`, `Hard`, `Good`, `Easy`) and `#ansbut` are suppressed on procedural cards to prevent duplicate rating bars and state desynchronization. Card advance is driven by StudyLab's `#proc-next-btn` (`procedural_answer:<ease>`), with hotkey overrides (`1`..`4`) preserved for power users. Standard Basic and Cloze cards remain 100% untouched native reviews.

---

## 2. Canonical Master 23-Control Matrix

Below is the comprehensive matrix specifying every single control in the StudyLab universe:

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

## 3. Exhaustive Individual Control Specifications

### 3.1 `Quick Solve` Tab (`#proc-tab-quick`)
- **Applicable Learning Objects:** `problem` (where procedural contract specifies both quick and stepwise modes).
- **Active States:** `ready`, `solving`.
- **Keyboard Shortcut:** `Alt+Q`.
- **Forbidden Combinations:** **STRICTLY PROHIBITED** on single-mode objects (`mcq`, `concept_check`, `strategy_drill`, `worked_example`, pure numerical, pure stepwise).

### 3.2 `Step-by-Step` Tab (`#proc-tab-stepwise`)
- **Applicable Learning Objects:** `problem` (where procedural contract specifies stepwise support).
- **Active States:** `ready`, `solving`.
- **Keyboard Shortcut:** `Alt+S`.
- **Forbidden Combinations:** **STRICTLY PROHIBITED** on single-mode objects (`mcq`, `concept_check`, `strategy_drill`, `worked_example`).

### 3.3 `Submit Answer` Button (`#proc-submit-btn`)
- **Applicable Learning Objects:** `problem`, `quick`, numerical calculations.
- **Active States:** `ready`, `solving`.
- **Visual Priority:** **Primary CTA** in Numerical mode.
- **Keyboard Shortcut:** `Enter` (when input is focused), `Ctrl+Enter`.
- **Forbidden Combinations:** Must **NOT** coexist with `#proc-check-steps-btn`, `.proc-option-group`, `#proc-next-btn`, `#ansbut`.

### 3.4 `Check Solution` Button (`#proc-check-steps-btn`)
- **Applicable Learning Objects:** `stepwise`, `problem` (in stepwise mode).
- **Active States:** `stepwise`.
- **Visual Priority:** **Primary CTA** for Stepwise workspace.
- **Keyboard Shortcut:** `Ctrl+Enter`.
- **Dispatched IPC Event:** `procedural_validate_steps:<json>`.

### 3.5 `+ Add Step` Button (`#proc-add-step-btn`)
- **Applicable Learning Objects:** `stepwise`, `problem` (stepwise mode).
- **Active States:** `stepwise`.
- **Keyboard Shortcut:** `Alt+A`, or pressing `Enter` in the last step input row.

### 3.6 `💡 Request Hint` Button (`#proc-hint-btn`)
- **Applicable Learning Objects:** `problem`, `quick`, `stepwise`, `mcq`.
- **Active States:** `ready`, `solving`, `stepwise`.
- **Keyboard Shortcut:** `H`, `?`.
- **Dispatched IPC Event:** `procedural_hint:<json>`.
- **Forbidden Combinations:** Must **NOT** appear during `mistake_classification` or `feedback`.

### 3.7 `↺ Reset Workspace` Button (`#proc-reset-btn` / `#proc-reset-steps-btn`)
- **Applicable Learning Objects:** `stepwise`, `problem` (stepwise mode).
- **Active States:** `stepwise`.
- **Keyboard Shortcut:** `Alt+R`.

### 3.8 `Try Similar Problem` Button (`#proc-try-similar-btn`)
- **Applicable Learning Objects:** All procedural objects.
- **Active States:** `feedback`, `worked_example`.
- **Keyboard Shortcut:** `Alt+T`.
- **Dispatched IPC Event:** `procedural_try_similar:<json>`.

### 3.9 `Next Problem ➔` Button (`#proc-next-btn`)
- **Applicable Learning Objects:** All procedural objects.
- **Active States:** `feedback`, `concept_check`.
- **Visual Priority:** **Primary CTA** on feedback screens.
- **Keyboard Shortcut:** `Space`, `Enter`.
- **Dispatched IPC Event:** `procedural_answer:<ease>`.
- **Forbidden Combinations:** Must **NOT** appear during `solving`, `submitting`, or `mistake_classification`. Native bottom ease buttons must remain suppressed.

### 3.10 Mistake Classification Buttons (`.proc-mistake-btn[data-key="1..4"]`)
- **Labels:** `[1 Silly Slip]`, `[2 Pattern Missed]`, `[3 Concept Gap]`, `[4 Prereq Unknown]`.
- **Active States:** `mistake_classification`.
- **Visual Priority:** **Exclusive Primary Reflection Grid**.
- **Keyboard Shortcuts:** `1`, `2`, `3`, `4`.
- **Dispatched IPC Event:** `procedural_mistake:<json>`.
- **Anti-Bypass Guarantee:** `#proc-next-btn`, `#ansbut`, and native ease buttons are strictly blocked. Solution container remains hidden (`ANTI-08`).

### 3.11 Native Anki Controls (`Again`, `Hard`, `Good`, `Easy`, `#ansbut`)
- **Applicable Objects:** Standard Anki Basic/Cloze cards **ONLY**.
- **Procedural Cards Rule:** Strictly suppressed via host bridge to enforce the One-Interaction-Surface invariant.

---

## 4. Mutually Exclusive Control Sets

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
- **`mistake_classification` State:** The 4 mistake buttons are **MANDATORY**. `#proc-next-btn`, `#ansbut`, and native ease buttons are **STRICTLY BLOCKED**. Solution container is hidden (`ANTI-08`).
- **`feedback` State:** `#proc-next-btn` is **ACTIVE**. Mistake classification buttons are **REMOVED**. Solution container is **REVEALED**.

---

## 5. Keyboard Navigation & Hotkey Mapping

| Key / Shortcut | Scope / State | Behavior & Routing | Trapping / Propagation |
|---|---|---|---|
| `Enter` | `solving` (Input focused) | Submits quick answer via `handleQuickSubmit()` | `e.preventDefault()` |
| `Ctrl+Enter` | `stepwise` | Submits stepwise derivation via `handleStepwiseSubmit()` | `e.preventDefault()` |
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
| `Alt+A` | `stepwise` | Appends a new step row | `e.preventDefault()` |
| `Alt+R` | `stepwise` | Resets step workspace | `e.preventDefault()` |
| `Alt+T` | `feedback` | Generates and loads similar problem variant | `e.preventDefault()` |
| `M` | Global Anki Shell | Opens native Anki `More` context menu | Native Anki handler |
