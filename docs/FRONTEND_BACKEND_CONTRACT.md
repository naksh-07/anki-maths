# Frontend / Backend Contract

StudyLab operates a dual-layer state machine where the TS/Vite frontend provides the problem-solving workspace, and the Python/Qt/Rust backend maintains canonical telemetry and scheduling integration.

## Communication Bridge

The TS frontend communicates with the Qt backend (`qt/aqt/reviewer.py`) using `bridgeCommand("<command>")`. The `_handle_procedural_command` function acts as the central router.

### 1. Answer Submission
- **Flow:** TS evaluates input → TS sends `bridgeCommand("procedural_attempt:{...JSON...}")` → TS sends `bridgeCommand("ans")`.
- **Payload:** JSON containing latency, final correctness, step trace, and active mode.
- **Qt Action:** `procedural_attempt` stores telemetry in instance variables (`self._last_procedural_attempt`). It forces `self.state = "answer"` and calls `self._showEaseButtons()` if native integration requires it. `ans` triggers Anki's native `_getTypedAnswer()` pipeline.
- **Persistence:** TS uses `globalThis.anki.mutateNextCardStates` to inject telemetry into the v3 scheduler states, eventually flushing to `procedural.db` via Rust.

### 2. Next Problem (Scheduling)
- **Flow:** TS sends `bridgeCommand("procedural_answer:<ease>")`.
- **Payload:** An integer ease calculation. `1` (Incorrect), `3` (Slow Correct), `4` (Fast Correct).
- **Qt Action:** Maps to `self._answerCard(val)` to formally reschedule the procedural anchor using FSRS.

### 3. Hints & Mistakes
- **Flow:** TS sends `bridgeCommand("procedural_hint:{...}")` or `bridgeCommand("procedural_mistake:{...}")`.
- **Payload:** Metadata about the requested hint or self-classified mistake.
- **Qt Action:** Modifies the pending telemetry payload. Mistake classifications are mapped into `DomainEvidence` (e.g. execution vs concept) when finalized.

### 4. Step Validation
- **Flow:** TS sends `bridgeCommand("procedural_validate_steps:{...}")`.
- **Payload:** The mathematical/logical expression of the current intermediate step.
- **Qt Action:** Used for server-side evaluation if local TS evaluation requires heavy symbolic algebra offloading.

### 5. Learning Object Injection
- **Flow:** TS sends commands like `procedural_try_similar`, `procedural_declarative_recall`, or `procedural_practice_prerequisite`.
- **Qt Action:** Intercepts standard FSRS queues to inject JIT remediation tasks before pulling the next scheduled card.

## Design Rule: Avoid Duplication
- **Evaluation Ownership:** The Rust core owns canonical correctness rules (via the `inline_contract`). The TS frontend evaluates locally based on that contract to ensure zero latency during problem-solving, but Rust is the ultimate authority. Do not recreate learning logic exclusively in TS.
