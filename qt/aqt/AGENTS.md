# Directory-Scoped Rules: PyQt Desktop Bridge (`qt/aqt/`)

**Authority:** Level 1 Subsystem Architectural Invariant  
**Target:** `qt/aqt/reviewer.py`, `qt/aqt/main.py`  
**Governing Subagents:** `visual-qa-inspector`, `implementer`, `reviewer-verifier`

---

## 1. Two-P0 Button Suppression Invariant

1. **Suppression of Standard Buttons**: When a procedural note is loaded in the reviewer (`is_procedural == True`):
   - The "Show Answer" button in `mw.bottomWeb` must be suppressed / hidden.
   - Standard SRS rating ease buttons ([1] Again, [2] Hard, [3] Good, [4] Easy) must NOT be shown during solving.
2. **Post-Answer Transition**: The bottom bar remains hidden while the student engages with procedural solving and classification. Rating calculation is handled automatically by the rating policy bridge.

---

## 2. Keyboard Interception & Event Trapping

1. **Trap `onEnterKey`**: When the student presses `Enter` or `Return` while focus is inside the procedural webview:
   - Forward the keystroke to the Svelte solving canvas to submit the step or selected option.
   - Prevent PyQt from calling the default Anki `mw.reviewer.onEnterKey()` handler which flips the card.
2. **Spacebar Anti-Bypass**: Pressing `Space` must NEVER bypass problem solving or reveal answers prematurely. Keystrokes must be scoped exclusively to interactive input.

---

## 3. IPC & Bridge Integrity

1. **Bridge Message Handling**: Custom procedural IPC messages (e.g., `procedural_submit`, `procedural_complete`) received via `pycmd()` must be handled cleanly:
   - Validate payload types before invoking backend methods.
   - Run long-running storage or scoring computations asynchronously or ensure they execute within < 5ms to prevent Qt GUI hitching.
2. **Clean Fallback for Standard Cards**: For non-procedural cards (`Basic`, `Cloze`), the bridge must immediately restore default Anki button behavior and key bindings without latency.

---

## 4. Verification Requirements

Before submitting changes in this directory:
```bash
just test-py
python artifacts_qa/inspect_live_window.py
```
