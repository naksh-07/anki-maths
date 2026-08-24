## 2026-08-24T12:22:37Z
You are the DIAGNOSTIC / ASSESSMENT SPECIALIST (Worker) for the STUDYLAB FINAL RECONCILIATION MISSION.
Working directory: c:/Users/Suraj/Documents/Antigravity/Anki-maths
Your metadata folder: c:/Users/Suraj/Documents/Antigravity/Anki-maths/.agents/specialist8_diagnostic_assessment

Read the authoritative user request at `c:/Users/Suraj/Documents/Antigravity/Anki-maths/ORIGINAL_REQUEST.md`.
Read `c:/Users/Suraj/Documents/Antigravity/Anki-maths/PROJECT.md`.
Read `c:/Users/Suraj/Documents/Antigravity/Anki-maths/03_architecture_gap_matrix.md`, `01_research_findings.md`, and `02_product_reconciliation.md`.

Your Mission:
1. Resolve `GAP-DIAG-01` and `GAP-EV-01`:
   - Wire the Diagnostic Mock-Test Session Engine (`MockSession`, `ComprehensiveDiagnosticReport` in `rslib/procedural/src/exam/mock.rs`) through `ProceduralService` and Python/TS bridges.
   - Build/verify the lightweight Diagnostic Session webview container (10-20 questions across Math, Reasoning, Physics, Chemistry in fixed measuring mode with time budget).
   - Generate hierarchical diagnostic reports: Subject -> Chapter -> Topic -> Problem Family and Concept/Execution/Transfer/Speed dimensions.
   - Ensure completed diagnostic sessions batch-update existing `SkillState` and `DomainEvidence` structures in `procedural.db` without duplicate parallel state models.
2. Run automated tests for diagnostic session execution and evidence updates.

Exclusive Write Ownership:
- `rslib/procedural/src/exam/`
- `ts/reviewer/diagnostic/` / related diagnostic TS UI files
