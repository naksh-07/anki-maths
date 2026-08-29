# StudyLab Runtime Interaction Contract

**Status:** Canonical Master Contract

## 1. Canonical State Machine

The interaction model strictly adheres to the following flow:

```text
SOLVING
  |
  +-- correct --> FEEDBACK/SOLUTION --> automatic advance
  |
  +-- incorrect --> MISTAKE CLASSIFICATION
                         |
                         +-- 1 Silly Slip ------+
                         +-- 2 Pattern Missed --+
                         +-- 3 Concept Gap -----+--> record classification --> advance
                         +-- 4 Prereq Unknown --+
```

There is **NO** user-facing `Next Card` state.

## 2. Event Ownership & Host Boundary
- **TypeScript (StudyLab)**: Owns the interaction lifecycle, hotkeys, submission logic, mistake classification state, and the transient feedback rendering.
- **Python (Anki Host)**: Owns the transition to the next card via IPC bridge commands once the TypeScript state machine completes its interaction cycle.
- **Forbidden Paths**: A procedural card MUST NOT be graded or advanced via native Anki keyboard shortcuts or fallback mechanisms while in the `SOLVING` or `MISTAKE CLASSIFICATION` states.

## 3. Correct Flow & Automatic Advancement
- When the learner submits a correct answer, the UI transitions to a transient `FEEDBACK/SOLUTION` state.
- The state automatically advances to the next card after rendering the feedback, or immediately if transient feedback is skipped.
- At no point does the user click a "Next Card" button.

## 4. Incorrect Flow & Classification
- When the learner submits an incorrect answer, the UI transitions to `MISTAKE CLASSIFICATION`.
- The user MUST select one of the four classification buttons.
- Upon classification selection, the attempt is immediately graded/recorded, and the UI advances to the next card automatically.

## 5. Keyboard Contract
- **Answer Selection (MCQ)**: Keys `A`, `B`, `C`, `D` or `1`, `2`, `3`, `4`.
- **Answer Submission**: `Enter` (from text input) or `Space/Enter` (if focus allows).
- **During Mistake Classification**:
  - Keys `1`, `2`, `3`, `4` map directly to the four mistake categories.
  - `Space` and `Enter` MUST NOT bypass the mandatory classification. They are strictly trapped to prevent rote spamming.
- **During Transient Correct Feedback**: If the system pauses on correct feedback, pressing `Space` or `Enter` advances immediately to the next card (acting as an early skip of the auto-advance).
