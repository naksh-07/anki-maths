## 2026-08-24T13:24:07Z

You are Specialist 7 (Physics + Chemistry Numerical UX Specialist).
Working directory: c:/Users/Suraj/Documents/Antigravity/Anki-maths/.agents/specialist7_phys_chem_numerical_gen2

Read ORIGINAL_REQUEST.md at `c:/Users/Suraj/Documents/Antigravity/Anki-maths/ORIGINAL_REQUEST.md`, `PROJECT.md`, `03_architecture_gap_matrix.md`, `01_research_findings.md`, `02_product_reconciliation.md`.

Mission & Scope:
1. Complete Physics + Chemistry Numerical input and units parser across TS frontend and Rust backend:
   - Dedicated numeric input with units/tolerances/fractions/scientific notation (e.g. `12 m/s`, `5 kg`, `2.5 mol`, `1.2e-3 mol/L`, `6.022e23`, `3/4`), avoiding artificial choices or NaN errors.
   - Dimensional correctness checks, unit conversions, physical/chemical constant scaling, and absolute/relative tolerance evaluation.
   - Ensure frontend `ts/reviewer/components/numerical_container.ts` and backend `rslib/procedural/src/units/` (or equivalent) handle complex units cleanly without crash or NaN.
2. Run and verify unit tests in Rust and TypeScript.
3. MANDATORY INTEGRITY WARNING: DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A forensic auditor will independently verify your work.
4. Write your comprehensive handoff report to `c:/Users/Suraj/Documents/Antigravity/Anki-maths/.agents/specialist7_phys_chem_numerical_gen2/handoff.md` with: MISSION, SCOPE, SOURCES, FILES INSPECTED, FINDINGS, EVIDENCE, RISKS, RECOMMENDATION, UNKNOWN / UNVERIFIED. Include passing test commands and output.
5. Send a message to parent when complete with a summary.
