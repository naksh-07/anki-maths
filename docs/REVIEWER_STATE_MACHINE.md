# Reviewer State Machine

The StudyLab Reviewer intercepts the standard Anki render pipeline, injecting a TypeScript webview workspace (`ts/reviewer/procedural.ts`). This is **not** a flashcard flip/reveal workflow; it is an active problem-solving environment.

---

## State Machine Architecture

The frontend lifecycle is governed by the `ProceduralUIState` union type (`ts/reviewer/procedural.ts:25-36`):

```text
loading ──> ready ──> solving ──(submit)──> submitting
                         │                        │
                    (hint event)           (eval outcome)
                         │                        │
                         ▼                  ┌─────┴──────────────────┐
                       hint                 ▼                        ▼
                         │              [Correct]               [Incorrect]
                         └─(resume)─>       │                        │
                                            │                        ▼
                                            │              mistake_classification
                                            │                        │
                                            │                  (categorized)
                                            │                        │
                                            ▼                        ▼
                                         feedback <──────────────────┘
                                            │
                                          (next)
                                            │
                                            ▼
                                          next ──> teardown / Anki lifecycle
```

---

## Comprehensive State Table

### 1. `loading`
- **Visible UI:** Loading spinner with initial DOM initialization.
- **Primary Action:** Automated container bootstrap and asset verification.
- **Secondary Actions:** None.
- **Forbidden Actions:** Any user input or bridge dispatch.
- **Keyboard Behavior:** Suppressed.
- **Bridge Event:** None.
- **Persistence Effect:** None.
- **Next State:** `ready`.

### 2. `ready`
- **Visible UI:** Problem statement rendered, active answer container (`mcq`, `numerical`, or `stepwise`) mounted and focused.
- **Primary Action:** User focuses input field or views MCQ options.
- **Secondary Actions:** Mode toggle (`quick` $\leftrightarrow$ `stepwise`).
- **Forbidden Actions:** Answer submission before typing.
- **Keyboard Behavior:** Hotkeys enabled (`1`–`4` / `A`–`D` for MCQ; numeric keys for numbers).
- **Bridge Event:** None.
- **Persistence Effect:** Records initial presentation timestamp for latency tracking.
- **Next State:** `solving`.

### 3. `solving`
- **Visible UI:** Active problem-solving workspace. Native Anki rating and show-answer buttons are hidden.
- **Primary Action:** Typing mathematical/numerical answer, selecting MCQ option, or entering step deduction.
- **Secondary Actions:** Click "Hint" button (`?` or `H`), toggle stepwise mode.
- **Forbidden Actions:** Space/Enter passing to native Anki card flip.
- **Keyboard Behavior:**
  - `Enter` / `Ctrl+Enter`: Triggers submit.
  - `H` / `?`: Triggers hint request.
  - `1`–`4` / `A`–`D`: Toggles option selection in MCQ container.
- **Bridge Event:** None during editing.
- **Persistence Effect:** Telemetry timer accumulates active solving time.
- **Next State:** `submitting` (on submit) or `hint` (on hint request).

### 4. `hint`
- **Visible UI:** Problem container dimmed or expandable hint accordion opened showing pedagogical principle without revealing the final answer.
- **Primary Action:** Read hint and click "Resume Solving" or continue typing.
- **Secondary Actions:** Request secondary hint if available in `StepNodeSpec`.
- **Forbidden Actions:** Direct answer auto-fill.
- **Keyboard Behavior:** `Esc` or `Enter` closes hint panel and refocuses answer container.
- **Bridge Event:** Dispatches `bridgeCommand("procedural_hint:{...}")` to record hint exposure in Python.
- **Persistence Effect:** Lowers `independence` score in `MasteryEvidence`.
- **Next State:** `solving`.

### 5. `submitting`
- **Visible UI:** Disabled answer fields with quick inline validation spinner.
- **Primary Action:** Local semantic evaluation against `inline_contract` derivations (`ts/reviewer/procedural.ts:788-842`).
- **Secondary Actions:** None.
- **Forbidden Actions:** Modifying inputs.
- **Keyboard Behavior:** All input suppressed.
- **Bridge Event:** Dispatches `bridgeCommand("procedural_attempt:{...JSON...}")` followed by `bridgeCommand("ans")` to advance Qt reviewer state to `"answer"`.
- **Persistence Effect:** Telemetry snapshot created.
- **Next State:** `feedback` (if correct) or `mistake_classification` (if incorrect).

### 6. `mistake_classification`
- **Visible UI:** Compact mistake categorization footer mounted in the native answer interaction zone:
  - `[1] Silly Mistake` (arithmetic or sign slip)
  - `[2] Pattern Not Recognized` (unfamiliar form or variant)
  - `[3] Formula/Concept Misapplied` (wrong theorem or setup)
  - `[4] Concept Not Known` (missing prerequisite knowledge)
- **Primary Action:** Self-diagnose and classify the cognitive error.
- **Secondary Actions:** None.
- **Forbidden Actions:** Bypassing categorization without selecting a category.
- **Keyboard Behavior (Anti-Bypass Guardrail):**
  - Space and Enter keys are **strictly trapped and prevented** (`kbEvent.preventDefault(); kbEvent.stopPropagation();`) unless focused on a classification button.
  - Number keys `1`–`4` immediately classify the mistake and trigger transition.
- **Bridge Event:** Dispatches `bridgeCommand("procedural_mistake:{...}")`.
- **Persistence Effect:** Maps directly into `DomainEvidence` (`is_execution_error()` vs `is_conceptual_error()`).
- **Next State:** `feedback`.

### 7. `feedback`
- **Visible UI:** Correct (green) or Incorrect (red) banner, complete derived solution explanation, dimensional unit breakdown, and remediation action buttons (`Try Similar`, `Practice Prerequisite`).
- **Primary Action:** Review explanation and press `Space` / `Enter` or click "Continue".
- **Secondary Actions:** Click "Try Similar" or "Practice Prerequisite" to branch into immediate practice.
- **Forbidden Actions:** Re-editing submitted answer.
- **Keyboard Behavior:** `Space` / `Enter` invokes `handleNext()`.
- **Bridge Event:** Dispatches `bridgeCommand("procedural_answer:<ease>")`.
- **Persistence Effect:** Injects telemetry into Anki custom data via `globalThis.anki.mutateNextCardStates` and triggers FSRS review via Python.
- **Next State:** `next`.

### 8. `worked_example`
- **Visible UI:** Complete step-by-step worked solution highlighting critical decision nodes, rendered without input fields.
- **Primary Action:** Read derivation and click "I Understand / Continue" button.
- **Secondary Actions:** Expand sub-step rationale.
- **Forbidden Actions:** Direct rating.
- **Keyboard Behavior:** `Space` / `Enter` acknowledges the worked example.
- **Bridge Event:** Dispatches acknowledgement telemetry.
- **Persistence Effect:** Records worked example exposure and queues immediate transfer retry.
- **Next State:** `next`.

### 9. `next`
- **Visible UI:** Smooth transition to next scheduled item or next Anki review card.
- **Primary Action:** Automated cleanup and handover.
- **Secondary Actions:** None.
- **Forbidden Actions:** None.
- **Keyboard Behavior:** Native Anki reviewer shortcuts resume.
- **Bridge Event:** FSRS reschedule command executed in Qt/Rust backend.
- **Persistence Effect:** State flushed to `procedural.db`.
- **Next State:** `loading` (for next procedural anchor) or native Anki card view.

### 10. `error` / `teardown`
- **Visible UI:** Error boundary with diagnostic traceback if initialization fails; otherwise graceful teardown on container unmount (`ts/reviewer/procedural.ts:1239-1278`).
- **Primary Action:** Safe fallback to standard Anki answer view if a contract is corrupted.
- **Secondary Actions:** Copy error details.
- **Forbidden Actions:** Silent state corruption.
- **Keyboard Behavior:** Native shortcuts restored.
- **Bridge Event:** Logs initialization error.
- **Persistence Effect:** None.
- **Next State:** Terminal unmount.

---

## Active Modes: `quick` vs `stepwise`

- **`quick` Mode (Default):**
  - Single consolidated input for final numeric, formula, or MCQ answer.
  - Optimal for high-speed fluency training.
- **`stepwise` Mode:**
  - Dynamic multi-step accordion (`StepwiseContainer`).
  - Evaluates intermediate sub-goals sequentially.
  - Evaluated against `StepValidator` (`rslib/procedural/src/problems/steps/step_validator.rs`).

---

## Lifecycle Boundary Handover

1. **StudyLab Ownership Phase:**
   - From `loading` through `solving`, `submitting`, `mistake_classification`, and `feedback`.
   - StudyLab controls the entire DOM, traps keyboard shortcuts, evaluates math/logic, and captures cognitive diagnostics.
2. **Anki / FSRS Ownership Phase:**
   - When the learner finishes feedback and presses Next, StudyLab calculates the programmatic ease score (`1` for incorrect, `3` for slow correct, `4` for fast correct) and dispatches `bridgeCommand("procedural_answer:<ease>")`.
   - Anki's native review engine (`qt/aqt/reviewer.py:_answerCard`) executes FSRS scheduling, updates `collection.anki2` intervals, and pulls the next card from the queue.
