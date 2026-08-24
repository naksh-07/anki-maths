## 2026-08-24T13:37:33Z
You are Specialist 10 (Security, Performance & Test Automation Specialist).
Working directory: c:/Users/Suraj/Documents/Antigravity/Anki-maths/.agents/specialist10_qa_security

Read ORIGINAL_REQUEST.md at `c:/Users/Suraj/Documents/Antigravity/Anki-maths/ORIGINAL_REQUEST.md`, `PROJECT.md`, `03_architecture_gap_matrix.md`, and all specialist handoffs in `.agents/`.

Mission & Scope:
1. Execute the entire automated testing suite across Rust, Python, and TypeScript:
   - Rust: `cargo test --workspace` or `cargo test -p procedural --lib` and integration tests (`cargo test -p procedural --test diagnostic_mock_session_tests`, `cargo test -p procedural --test step_interaction_tests`, `cargo test -p procedural --test exam_engine_tests`, `cargo test -p procedural --test reviewer_tests`)
   - TypeScript: `npm run vitest:once` (or `cd ts && npx vitest run`) and typecheck / lint
   - Python: `pytest` in `qt/` or python tests
   - Workspace check: `cargo check --workspace`
2. Audit Security:
   - Verify HTML escaping and XSS safety in problem card renderers, dynamic strings, mistake footer, and diagnostic reports.
   - Verify SQL injection safety in `procedural.db` database queries (parameterized queries).
3. Audit Performance & Memory Leaks:
   - Verify MutationObserver cleanup and event listener destruction in `ts/reviewer/procedural.ts`, `mcq_container.ts`, `stepwise_container.ts`, `numerical_container.ts`, `mistake_footer.ts`, `diagnostic_session.ts`.
   - Ensure card transitions are fluid with zero console errors or lingering intervals.
4. MANDATORY INTEGRITY WARNING: DO NOT CHEAT. All test runs must be authentic and verified directly.
5. Write your comprehensive handoff report to `c:/Users/Suraj/Documents/Antigravity/Anki-maths/.agents/specialist10_qa_security/handoff.md` with: MISSION, SCOPE, SOURCES, FILES INSPECTED, FINDINGS, EVIDENCE, RISKS, RECOMMENDATION, UNKNOWN / UNVERIFIED.
6. Send a message to parent when complete.
