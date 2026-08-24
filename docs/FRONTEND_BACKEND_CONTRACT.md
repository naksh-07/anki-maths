# Frontend / Backend Contract

StudyLab operates a synchronized dual-layer architecture where the TypeScript/Vite webview provides the interactive problem-solving workspace (`ts/reviewer/procedural.ts`), and the Python/Qt/Rust backend maintains canonical evaluation, scheduling, and database persistence.

---

## Communication Bridge Architecture

Communication between the webview and native Anki runs through Anki's standard IPC channel via `bridgeCommand("<command>")`, routed directly in Python by `_handle_procedural_command` (`qt/aqt/reviewer.py:724-774`).

```text
Webview (TS) ──[bridgeCommand]──> Qt Reviewer (Python) ──[PyO3 IPC]──> Procedural Subsystem (Rust)
     │                                    │                                      │
     ▼                                    ▼                                      ▼
Local Eval / UI State           Reviewer Lifecycle Sync                procedural.db Persistence
```

---

## Comprehensive Event Trace Matrix

| Frontend Event | Payload Schema | Bridge Command | Python Handler (`qt/aqt/reviewer.py`) | Evaluator Layer | Persistence Target | Response / UI Effect |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Answer Submission** | `{ latency_ms, is_correct, answer_raw, mode, steps }` | `procedural_attempt:{...}` + `ans` | `_handle_procedural_command`<br>(Lines 758, 783) | TS (inline contract derivations) + Rust `StepValidator` | Stored in `self._last_procedural_attempt` | Sets `self.state = "answer"`, triggers native typed answer pipeline, transitions UI to `feedback` or `mistake_classification`. |
| **Reschedule / Next** | Integer ease: `1` (Again), `3` (Good), `4` (Easy) | `procedural_answer:<ease>` | `_handle_procedural_command`<br>(Lines 703, 735) | Python/Rust FSRS Engine | Flushes telemetry to `procedural.db` via Rust scheduler hook | Calls `self._answerCard(val)` to formally reschedule card and advance to next item. |
| **Hint Requested** | `{ hint_index, step_id, timestamp }` | `procedural_hint:{...}` | `_handle_procedural_command`<br>(Lines 756, 779) | TS (`StepNodeSpec`) | Stored in `self._last_procedural_hint` | Renders pedagogical principle in UI; decrements attempt `independence` score. |
| **Mistake Classified** | `{ category, reason, error_type }` | `procedural_mistake:{...}` | `_handle_procedural_command`<br>(Lines 762, 790) | Learner self-diagnosis | Stored in `self._last_procedural_mistake` | Unlocks `feedback` state; feeds diagnostic classifier in `domain_evidence.rs`. |
| **Step Validation** | `{ step_id, input_expr, current_step }` | `procedural_validate_steps:{...}` | `_handle_procedural_command`<br>(Lines 760, 775) | Rust `StepValidator` (`rslib/procedural/src/problems/steps/step_validator.rs`) | Stored in `self._last_procedural_stepwise_validation` | Returns step correctness, updates step status indicator, enables next step input. |
| **Try Similar Variant** | `{ family_id, seed_mode: "Random" }` | `procedural_try_similar:{...}` | `_handle_procedural_command`<br>(Lines 764, 794) | `DeclarativeProblemGenerator` | Emits `ProblemInstance` to DB | Calls `self._showQuestion()` to re-render fresh numerical variant without leaving review. |
| **Practice Prerequisite** | `{ prerequisite_family_id, topic_id }` | `procedural_practice_prerequisite:{...}` | `_handle_procedural_command`<br>(Lines 766, 803) | `RemediationPolicy` | Queues JIT item in `remediation_queue_items` | Injects prerequisite drill into immediate queue. |
| **Declarative Recall** | `{ fact_id, target_symbol }` | `procedural_declarative_recall:{...}` | `_handle_procedural_command`<br>(Lines 768, 810) | Direct string/value comparator | Updates `SkillState` recall latency | Injects formula recall prompt. |

---

## Telemetry Persistence Pipeline

Telemetry injection bypasses Anki collection pollution using a multi-stage transport pipeline:

1. **Webview State Mutation:**
   ```typescript
   globalThis.anki.mutateNextCardStates((globalThis.anki as any)._state_mutation_key, async (states, customData) => {
       for (const state of ["again", "hard", "good", "easy"]) {
           if (customData[state]) {
               customData[state].studylab = { ...customData[state].studylab, ...telemetry };
           }
       }
   });
   ```
2. **Scheduler Hook & Database Flush:**
   - When `_answerCard()` executes, the Rust answering pipeline (`rslib/src/scheduler/answering/mod.rs:350-435`) extracts the `studylab` custom data dictionary.
   - It records `PracticeAttempt`, `ErrorEvent`, and updates `SkillState` directly in `procedural.db` (`rslib/procedural/src/storage/store.rs`).
   - The custom data payload is consumed and cleared, keeping `collection.anki2` completely free of procedural telemetry schema clutter.

---

## Evaluation Boundaries: Convenience vs Authority

- **UI-Level Evaluation (Zero Latency):**
  - Executed client-side in TypeScript (`ts/reviewer/procedural.ts:788-842` and `ts/reviewer/components/numerical_container.ts`).
  - Provides instantaneous UI state transitions, input error highlights, and dimensional unit checks based on values supplied in `inline_contract`.
- **Canonical Semantic Authority (Rust Backend):**
  - Executed in Rust (`rslib/procedural/src/problems/steps/step_validator.rs` and `rslib/procedural/src/problems/validator.rs`).
  - Authoritatively computes mathematical equivalence, multi-step symbolic inferences, and domain diagnostic classifications (`DomainEvidence`).
  - Frontend evaluation logic must never diverge from or replace backend contract verification.
