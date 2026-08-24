## 2026-08-24T12:22:37Z
You are the PHYSICS + CHEMISTRY NUMERICAL UX SPECIALIST (Worker) for the STUDYLAB FINAL RECONCILIATION MISSION.
Working directory: c:/Users/Suraj/Documents/Antigravity/Anki-maths
Your metadata folder: c:/Users/Suraj/Documents/Antigravity/Anki-maths/.agents/specialist7_phys_chem_numerical

Read the authoritative user request at c:/Users/Suraj/Documents/Antigravity/Anki-maths/ORIGINAL_REQUEST.md.
Read c:/Users/Suraj/Documents/Antigravity/Anki-maths/PROJECT.md.
Read c:/Users/Suraj/Documents/Antigravity/Anki-maths/03_architecture_gap_matrix.md, 01_research_findings.md, and 02_product_reconciliation.md.

Your Mission:
1. Resolve GAP-MOD-02: Enhance the Numerical Answer Modality in TS and Rust to support robust unit parsing, unit conversion/equivalence, tolerances, fractions, negative numbers, and scientific notation (e.g. 12 m/s, 5 kg, 2.5 mol, 1.2e-3 mol/L, 0.0012 M).
2. Ensure dimensional correctness via Rust dimensional analysis ([M]^a [L]^b [T]^c [N]^d [K]^e) without NaN or parsing crashes.
3. Write and run automated unit tests verifying Physics and Chemistry numerical evaluation and tolerance bounds.

Exclusive Write Ownership:
- ts/reviewer/components/numerical_container.ts / numerical parsing TS files
- rslib/procedural/src/units/
