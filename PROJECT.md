# Project: StudyLab Final Reconciliation

## Architecture
- **Host**: Native Anki desktop application (Python backend + Qt6 / QtWebEngine UI).
- **Core Engine**: Rust extension module (`anki_maths_core`) providing high-performance parsing, CAS, math verification, Rust semantic `StepValidator`, and learner mastery state models.
- **Bridge**: Python add-on layer bridging Anki reviewer hooks, webview injection, command handling, and Rust core FFI.
- **Web Frontend**: TypeScript/Vite webview UI running inside Anki's review webview and bottom bar webview, rendering math/reasoning/physics/chemistry cards, answer controls, compact mistake classification footer, and diagnostic test interface.

## Feature Inventory
| # | Feature | Description | Milestone | Source |
|---|---------|-------------|-----------|--------|
| 1 | Product & Native Anki Research | UX archaeology, native Anki review lifecycle, answer reveal, footer contracts, keyboard shortcuts | M1 | R1 |
| 2 | Architecture Gap Matrix | Formal comparison of original principles vs implementation across reviewer UI, bridge, controls, footer, state | M1 | R1 |
| 3 | MCQ Modality Contract | Authentic selectable options, 1-4/A-D keyboard shortcuts, canonical identity evaluation, no text input fallback | M2 | R2 |
| 4 | Numerical Modality Contract | Dedicated numeric inputs accepting units (m/s, kg, mol, etc.), tolerances, fractions, scientific notation, dimensional correctness | M2 | R2 |
| 5 | Stepwise Modality & Rust StepValidator | Semantic step evaluation wired directly to Rust StepValidator without duplicate TS logic | M2 | R2 |
| 6 | Content Mold Scalability | Declarative content + universal runtime mold without per-topic generator sprawl | M2 | R2 |
| 7 | Native Mistake Footer Lifecycle | Compact mistake classification footer `[1 Silly]`, `[2 Pattern]`, `[3 Concept]`, `[4 Unknown]` in native footer zone | M3 | R3 |
| 8 | Standard Anki Card Non-Regression | Zero regressions on Basic, Cloze, and other standard card types and shortcuts | M3 | R3 |
| 9 | Diagnostic Session Engine | 10-20 questions across Math, Reasoning, Physics, Chemistry with fixed measuring mode | M4 | R4 |
| 10 | Hierarchical Diagnostic Reporting | Performance breakdown: Subject -> Chapter -> Topic -> Family and Concept/Execution/Transfer/Speed | M4 | R4 |
| 11 | Mastery & Domain Evidence Sync | Diagnostic results feed directly into `SkillState`, `MasteryEvidence`, and `DomainEvidence` without parallel state models | M4 | R4 |
| 12 | Automated Suite Verification | `just check`, `just test-rust`, `just test-py`, `just test-ts` all clean and passing | M5 | R6 |
| 13 | Security & Performance Hardening | Zero console errors, no memory leaks or dangling event listeners, XSS/HTML sanitized | M5 | R5 |
| 14 | Live QtWebEngine Desktop Verification | Remote CDP testing against running dev Anki instance across all modalities and native cards | M6 | R5 |
| 15 | Live Evidence Package (04, 05, 06) | `04_live_ui_evidence.json`, `05_live_ui_screenshots/`, `06_diagnostic_live_evidence.json` | M6 | R5 |
| 16 | Independent 15-Point Release Gate | Comprehensive audit, `07_test_summary.md`, and `08_release_decision.md` | M7 | R6 |

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| M1 | Research, Product Vision & Gap Matrix | 01_research_findings.md, 02_product_reconciliation.md, 03_architecture_gap_matrix.md | None | IN_PROGRESS |
| M2 | Modality Contracts & Content Molds | MCQ options, Numerical with units/tolerances, Stepwise Rust StepValidator | M1 | PLANNED |
| M3 | Native Reviewer & Mistake Footer | Compact mistake footer [1 Silly]..[4 Unknown], standard card protection | M2 | PLANNED |
| M4 | Diagnostic Mock-Test Engine & Reports | 10-20 questions (4 domains), hierarchical reports, learner evidence sync | M2 | PLANNED |
| M5 | Automated Tests, Security & Performance | Test suite execution, memory leak prevention, XSS safety | M2, M3, M4 | PLANNED |
| M6 | Live QtWebEngine Desktop Verification | Live CDP attach against running Anki, screenshot evidence (04, 05, 06) | M3, M4 | PLANNED |
| M7 | Release Gating & Evidence Deliverables | 07_test_summary.md, 08_release_decision.md, 15-point release rule | M1-M6 | PLANNED |

## Interface Contracts
### TS Webview ↔ Python Add-on Bridge
- `pycmd("anki_maths:action?payload=...")` or `window.ankiBridge`
- Answer submission payload: `{ type: "mcq" | "numerical" | "stepwise", answer: string | object, metadata: object }`
- Mistake classification payload: `{ mistake_type: "silly" | "pattern" | "concept" | "unknown", card_id: number }`
- Diagnostic session commands: `{ action: "start_diagnostic" | "submit_item" | "finish_diagnostic", ... }`

### Python Add-on ↔ Rust Core FFI
- Native FFI bindings via PyO3 / C ABI (`anki_maths_core`)
- Step validation: `validate_step(problem_state, user_step) -> StepValidationResult`
- Learner state: `record_evidence(learner_id, evidence) -> UpdatedSkillState`

## Code Layout
- `crates/anki_maths_core/`: Rust core algorithms, CAS, step validation, domain models, learning state.
- `addon/anki_maths/`: Python Anki add-on, hooks, webview managers, bridge handlers, review lifecycle.
- `web/`: TypeScript/Vite frontend components, problem templates, answer controls, diagnostic UI.
- `tests/`: Automated tests across Rust, Python, TypeScript, and APKG generators.
- `evidence/`: Verification reports and live desktop artifacts (01 through 08).
