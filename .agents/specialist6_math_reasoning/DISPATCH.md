## 2026-08-24T12:22:37Z
You are the MATH + REASONING PEDAGOGY SPECIALIST (Worker) for the STUDYLAB FINAL RECONCILIATION MISSION.
Working directory: c:/Users/Suraj/Documents/Antigravity/Anki-maths
Your metadata folder: c:/Users/Suraj/Documents/Antigravity/Anki-maths/.agents/specialist6_math_reasoning

Read the authoritative user request at `c:/Users/Suraj/Documents/Antigravity/Anki-maths/ORIGINAL_REQUEST.md`.
Read `c:/Users/Suraj/Documents/Antigravity/Anki-maths/PROJECT.md`.
Read `c:/Users/Suraj/Documents/Antigravity/Anki-maths/03_architecture_gap_matrix.md`, `01_research_findings.md`, and `02_product_reconciliation.md`.

Your Mission:
1. Resolve `GAP-MOD-01`: Connect the TypeScript Stepwise card UI directly to the canonical Rust `StepValidator` (`rslib/procedural/src/problems/steps/step_validator.rs`).
2. Ensure intermediate algebraic, calculus, and reasoning step submissions are evaluated semantically through the Rust engine rather than relying on a scalar final-step string check in TypeScript.
3. Validate reasoning pedagogical structures: discrete logic, constraint satisfaction, and multi-step deduction.
4. Run Rust and TypeScript test suites verifying stepwise evaluation (`cargo test --package anki_procedural` or `just test-rust` and `just test-ts`).

Exclusive Write Ownership:
- `rslib/procedural/src/problems/steps/`
- `ts/reviewer/components/stepwise_container.ts` / related stepwise TS components
