# FINAL FRONTEND REPAIR PLAN

## Status: BLOCKED BY HARD GATE (UNVERIFIED)

Because the forensic audit failed at the Hard Gate (unable to attach to the visible HWND due to agent sandbox limitations), a comprehensive, state-by-state repair plan across all modalities cannot be generated based on authenticated native desktop evidence.

### Immediate Action Required
To proceed with the full repair plan, the native OS HWND must be accessible to the reviewer process, or the Hard Gate restriction requiring native desktop screenshot correlation must be lifted.

### Preliminary Findings (Based on CDP Evidence)
A partial inspection of the webview revealed a **P0 Defect**:
- The webview renders the problem text but **fails to render any input modality** (no MCQ buttons, no text inputs).
- The native bottom toolbar **fails to display the "Show Answer" button**.
- This creates a dead-end state where the learner cannot proceed.

### Anticipated Repair Order (Once Unblocked)
1. **P0 Modality Restore**: Ensure `template.rs` and the TS renderer inject the correct input fields (MCQ/Stepwise/Numerical) based on the `object_type`.
2. **P0 Native Footer Restore**: Ensure the native Anki `Show Answer` button is visible when appropriate, or properly delegated to the procedural UI without creating a dead-end.
3. **P1 Visual Density & Duplication**: Remove repeated expected answers or redundant timers.
4. **P1 Wrong-Answer Flow**: Validate the 1-4 classification mistake UI.
5. **P2 Polish**: Responsive alignment and whitespace fixes.
