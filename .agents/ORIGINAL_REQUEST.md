# Original User Request

## 2026-08-26T06:42:07Z

# Teamwork Project Prompt — Draft

> Status: Launched
> Goal: Craft prompt → get user approval → delegate to teamwork_preview
> Requested team: Full team (implied by the broad scope and audit requirements)

Perform ONE FINAL FULL-SYSTEM AUDIT for StudyLab, a procedural problem-solving and adaptive learning engine hosted inside the Anki desktop runtime, treating the repository as a RELEASE CANDIDATE. 

Working directory: c:\Users\Suraj\Documents\Antigravity\Anki-maths
Integrity mode: benchmark

## Requirements

### R1. Documentation ↔ Code Reconciliation
Trace the system end-to-end (APKG → Anki collection → ProceduralPayload → Rust → DB → Qt Reviewer → Webview → TS state machine). Read all canonical docs in docs/ and compare them against the actual implementation. Fix any contradictions (e.g., duplicate controls, wrong answer entry modalities).

### R2. Frontend & Modality Audit
Audit frontend state transitions, render lifecycle, keyboard handling, and UI/UX for all modalities (mcq, concept_check, strategy_drill, stepwise, etc.). Ensure there are no UI leakages between cards, no duplicate Next buttons, and no textboxes where they shouldn't exist. Fix all P0 and P1 bugs.

### R3. Desktop Layout & Visual Forensic Review
Ensure the UI is designed for a laptop (test 1366×768 to 1920×1080). It must not feel like a web app or giant dashboard. Primary problems must be visible immediately without awkward scrolling.

### R4. APKG & Database Verification
Verify generated canonical APKG and Database persistence (attempt persistence, skill state, orphan cleanup, migration safety). Ensure Anki's normal functionality (Basic, Cloze) remains completely untouched by StudyLab CSS/events.

### R5. Bug Register and Fix Policy
Produce docs/FINAL_RELEASE_AUDIT.md. Fix all P0 (release blockers) and P1 (important before release) bugs. Do NOT redesign the backend, change DB schemas casually, or change FSRS behavior. Perform the smallest correct fix.

### R6. Second Mandatory Audit
After fixing issues, rebuild, rerun automated tests, rerun APKG validation, rerun the full live desktop matrix, and compare BEFORE vs AFTER. No defect may be marked resolved without verification.

## Verification Resources
- Use the desktop-webview-reviewer for live, visible desktop verifications.
- Automated tests: 
pm run vitest:once and cargo test -p procedural.
- Reviewer tests: Python/Qt tests and build verification.

## Acceptance Criteria

### Live Desktop Verification
- [ ] The audit MUST use the REAL visible Windows Anki DEV GUI via desktop-webview-reviewer. Headless runs or fake viewports are strictly prohibited.
- [ ] Captured dual screenshots (native desktop + webview) proving real execution for all tested user flows.

### Product Quality & Regressions
- [ ] Normal Anki (Basic, Cloze) is clean and unaffected.
- [ ] No P0/P1 bugs remain in the bug register.
- [ ] No raw telemetry/debug UI is visible.
- [ ] Canonical APKG is valid and DB persistence is valid.

### Release Artifacts
- [ ] docs/FINAL_RELEASE_AUDIT.md is complete with Bug ID, Root Cause, Fix, and Live Evidence.
- [ ] docs/FINAL_RELEASE_NOTES.md is generated.
- [ ] rtifacts_qa/final_release_audit/evidence.json is produced containing all screenshots, hashes, and validation results.
- [ ] Final release verdict (🟢 RELEASE READY, 🟡 RELEASE CANDIDATE, or 🔴 RELEASE BLOCKED) is declared based on the objective criteria above.
