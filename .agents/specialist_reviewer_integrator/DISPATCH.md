## 2026-08-24T12:22:37Z
You are the NATIVE REVIEWER & BRIDGE INTEGRATOR (Worker) for the STUDYLAB FINAL RECONCILIATION MISSION.
Working directory: c:/Users/Suraj/Documents/Antigravity/Anki-maths
Your metadata folder: c:/Users/Suraj/Documents/Antigravity/Anki-maths/.agents/specialist_reviewer_integrator

Read the authoritative user request at `c:/Users/Suraj/Documents/Antigravity/Anki-maths/ORIGINAL_REQUEST.md`.
Read `c:/Users/Suraj/Documents/Antigravity/Anki-maths/PROJECT.md`.
Read `c:/Users/Suraj/Documents/Antigravity/Anki-maths/03_architecture_gap_matrix.md`, `01_research_findings.md`, and `02_product_reconciliation.md`.

Your Mission:
1. Resolve `GAP-BRG-01`, `GAP-FTR-01`, and `GAP-STA-01`:
   - In `qt/aqt/reviewer.py`, implement proper dispatching for `procedural_*` bridge commands (`procedural_hint:`, `procedural_try_similar:`, `procedural_practice_prerequisite:`, `procedural_mistake:`) instead of dropping them as no-ops.
   - Implement the compact mistake classification footer (`[1 Silly]`, `[2 Pattern]`, `[3 Concept]`, `[4 Unknown]`) in the primary interaction zone seamlessly integrated with Anki's review lifecycle (wrong answer -> compact mistake footer -> classification -> solution/hint -> rating buttons).
   - Ensure `ProceduralReviewer` registers proper cleanup (`destroy()`) when cards transition, removing global keydown listeners so non-procedural standard Anki cards (Basic, Cloze) experience 0% regression and 100% native shortcut fidelity.
2. Run Anki reviewer integration tests and verify clean card lifecycle transitions.

Exclusive Write Ownership:
- `qt/aqt/reviewer.py`
- `qt/aqt/webview.py`
- `ts/reviewer/procedural.ts`
- `ts/reviewer/components/mistake_footer.ts`
