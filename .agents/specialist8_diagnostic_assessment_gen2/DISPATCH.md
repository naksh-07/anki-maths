## 2026-08-24T13:24:07Z
You are Specialist 8 (Diagnostic / Assessment Specialist).
Working directory: c:/Users/Suraj/Documents/Antigravity/Anki-maths/.agents/specialist8_diagnostic_assessment_gen2

Read ORIGINAL_REQUEST.md at `c:/Users/Suraj/Documents/Antigravity/Anki-maths/ORIGINAL_REQUEST.md`, `PROJECT.md`, `03_architecture_gap_matrix.md`, `01_research_findings.md`, `02_product_reconciliation.md`.

Mission & Scope:
1. Build a lightweight Diagnostic Mock Test Session layer over the existing learning engine:
   - Support 10-20 questions sampled across 4 domains (Math, Reasoning, Physics, Chemistry) with time budgets.
   - Non-disruptive measurement during test (measuring rather than aggressively adapting mid-test).
   - Hierarchical diagnostic reports: 4-tier hierarchy (Subject -> Chapter -> Topic -> Problem Family) and skill dimensions (Concept, Calculation/Execution, Transfer, Speed).
   - Feed diagnostic results into existing `MasteryEvidence`, `DomainEvidence`, and `SkillState` in `procedural.db` without duplicate parallel state models.
   - TS report UI rendering the hierarchical breakdown cleanly.
2. Implement and execute unit tests in Rust and TypeScript.
3. MANDATORY INTEGRITY WARNING: DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A forensic auditor will independently verify your work.
4. Write your comprehensive handoff report to `c:/Users/Suraj/Documents/Antigravity/Anki-maths/.agents/specialist8_diagnostic_assessment_gen2/handoff.md` with: MISSION, SCOPE, SOURCES, FILES INSPECTED, FINDINGS, EVIDENCE, RISKS, RECOMMENDATION, UNKNOWN / UNVERIFIED. Include passing test commands and output.
5. Send a message to parent when complete with a summary.
