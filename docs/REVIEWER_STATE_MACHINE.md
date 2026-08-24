# Reviewer State Machine

The StudyLab Reviewer intercepts the standard Anki render pipeline, injecting a TS/Vite frontend that manages a problem-solving workspace. This is NOT a flashcard flip workflow.

## TS Frontend `ProceduralUIState`

The literal states mapped in the TypeScript frontend (`ts/reviewer/procedural.ts`) are strictly:

1. **`loading`**
   - **UI:** Loading spinner.
   - **Actions:** None.
   - **Next:** `ready`.
2. **`ready`**
   - **UI:** Problem rendered, input fields focused.
   - **Actions:** Begin typing, select mode.
   - **Next:** `solving`.
3. **`solving`**
   - **UI:** Active problem-solving workspace. Native Anki answer buttons are hidden.
   - **Actions:** Type answer, request hint, submit step, submit final.
   - **Next:** `submitting`, `hint`.
4. **`hint`**
   - **UI:** Hint principle displayed.
   - **Actions:** Return to solving.
   - **Backend Effect:** Lowers `independence` score in `MasteryEvidence`.
   - **Next:** `solving`.
5. **`submitting`**
   - **UI:** Disabled inputs, evaluation spinner.
   - **Actions:** None.
   - **Next:** `feedback`, `mistake_classification`, `error`.
6. **`mistake_classification`**
   - **UI:** Prompt asking the user to self-diagnose their error type (e.g., "Math slip" vs "Didn't know formula").
   - **Actions:** Select classification.
   - **Next:** `feedback`.
7. **`feedback`**
   - **UI:** Correct/Incorrect indicators shown, solution derived from contract shown. Native Anki answer buttons may be forced or simulated depending on integration.
   - **Actions:** Proceed to next.
   - **Backend Effect:** Generates `PracticeAttempt` and updates `SkillState`.
   - **Next:** `next`.
8. **`worked_example`**
   - **UI:** Step-by-step derivation presented without inputs.
   - **Actions:** Acknowledge.
   - **Next:** `next`.
9. **`next`**
   - **UI:** Transitioning to next problem.
   - **Next:** `loading` (or Anki's native next card).
10. **`error` / `teardown`**
    - **UI:** Crash boundary or unmount sequence.

*Note: "stepwise" is an `activeMode`, guiding users node-by-node during the `solving` state. Correct/Wrong are evaluation properties assigned during `submitting`, not states themselves.*

## Cross-Layer Invariants
- **Local Evaluation:** Procedural evaluation is handled locally in the TS frontend.
- **Syncing Qt:** The frontend calls `bridgeCommand("procedural_attempt")` to record telemetry and `bridgeCommand("ans")` to force the native Anki Qt state into `"answer"`, aligning the dual state machines.
- **Wrong-Answer Integration:** Mistake classification directly feeds into `DomainEvidence`, allowing the Rust backend to decide if the next state should be a `concept_check` or a standard reschedule.
