## 2026-08-24T13:37:33Z
You are Specialist 9 (QtWebEngine / Desktop-Webview Verification Specialist).
Working directory: c:/Users/Suraj/Documents/Antigravity/Anki-maths/.agents/specialist9_desktop_verifier

Read `c:/Users/Suraj/Documents/Antigravity/Anki-maths/ORIGINAL_REQUEST.md`, `PROJECT.md`, `03_architecture_gap_matrix.md`, and the skill instructions at `C:\Users\Suraj\.gemini\config\skills\desktop-webview-reviewer\SKILL.md` and `C:\Users\Suraj\.gemini\config\skills\qt-desktop-reviewer\SKILL.md`.

Mission & Scope:
1. Conduct Live QtWebEngine Desktop Verification via `desktop-webview-reviewer` / CDP attach mode against the running Anki desktop instance (e.g. debugging port 8000 / 8765 / CDP endpoint or launch with remote debugging if needed).
2. Execute the complete Live UI Test Matrix:
   - Mathematics: Numerical input, MCQ option click & 1-4 keyboard navigation, Wrong Answer flow & compact mistake footer `[1 Silly]`, `[2 Pattern]`, `[3 Concept]`, `[4 Unknown]`, Stepwise semantic step validation.
   - Reasoning: MCQ option click, 1-4 keys, structured reasoning cards.
   - Physics: Numerical with units (`12 m/s`, `72 km/h`, `5 kg`), tolerances, dimensional check.
   - Chemistry: Numerical with units (`1.2e-3 mol/L`, `6.022e23`), scientific notation, MCQ.
   - Native Anki: Standard Basic & Cloze cards, Show Answer, Again/Hard/Good/Easy rating buttons, shortcuts (1-4, Space), zero regressions on standard cards.
   - Diagnostic Mock Test: Launch diagnostic session, navigate palette, answer questions across 4 domains, submit, render 4-tier hierarchical report (Subject -> Chapter -> Topic -> Family) and 4-dimension skill breakdown.
3. Capture visual evidence:
   - Save screenshots in `c:/Users/Suraj/Documents/Antigravity/Anki-maths/05_live_ui_screenshots/` with descriptive filenames (e.g. `01_math_mcq.png`, `02_math_stepwise.png`, `03_mistake_footer.png`, `04_physics_units.png`, `05_chem_scinotation.png`, `06_native_cloze.png`, `07_diagnostic_session.png`, `08_diagnostic_report.png`).
4. Generate structured JSON evidence files:
   - `c:/Users/Suraj/Documents/Antigravity/Anki-maths/04_live_ui_evidence.json`
   - `c:/Users/Suraj/Documents/Antigravity/Anki-maths/06_diagnostic_live_evidence.json`
5. MANDATORY INTEGRITY WARNING: DO NOT CHEAT. All evidence, screenshots, and JSON telemetry must be authentic and reflect actual live QtWebEngine rendering.
6. Write your comprehensive handoff report to `c:/Users/Suraj/Documents/Antigravity/Anki-maths/.agents/specialist9_desktop_verifier/handoff.md` with: MISSION, SCOPE, SOURCES, FILES INSPECTED, FINDINGS, EVIDENCE, RISKS, RECOMMENDATION, UNKNOWN / UNVERIFIED.
7. Send a message to parent when complete.
