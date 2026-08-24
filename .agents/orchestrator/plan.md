# StudyLab Final Reconciliation Execution Plan

## Objective
Reconcile, audit, implement, and verify the full StudyLab product architecture within Anki, ensuring native reviewer harmony, flawless answer modalities (MCQ, Numerical with units/tolerances/sci notation, Stepwise), diagnostic mock testing across 4 domains, live QtWebEngine desktop verification via CDP, and 15-point release gate certification.

## Phase Breakdown & Subagent Mapping

### Phase 1: Research, Product Archaeology & Gap Matrix (R1)
- **Specialist 1: PRODUCT VISION / UX ARCHAEOLOGIST** (`teamwork_preview_explorer`)
  - Target: Reconcile North Star, study prior commits, walkthroughs, artifacts (`02_product_reconciliation.md`).
- **Specialist 2: NATIVE ANKI REVIEWER RESEARCHER** (`teamwork_preview_explorer`)
  - Target: Research native Anki reviewer interaction models, reviewer webview DOM, bottom bar / footer lifecycle, keyboard shortcuts, custom constraints (`01_research_findings.md` Part 1).
- **Specialist 3: STUDYLAB ARCHITECTURE AUDITOR** (`teamwork_preview_explorer`)
  - Target: Deep codebase audit against original principles across reviewer UI, state machines, native bridge, answer controls, footer, and learner state -> generate formal gap matrix (`03_architecture_gap_matrix.md`).
- **Specialist 4: CONTENT CONTRACT / APKG SPECIALIST** (`teamwork_preview_explorer`)
  - Target: Verify APKG -> PracticeItem -> ProblemInstance -> Template -> UI pipeline, ensure declarative content + universal mold scalability.

### Phase 2: Core Modality, Reviewer & Diagnostic Implementation (R2, R3, R4)
- **Specialist 5: MCQ / ANSWER-MODALITY SPECIALIST** (`teamwork_preview_worker`)
  - Target: Ensure MCQ real options (A-D / 1-4 keys, canonical identity evaluation, no text input).
- **Specialist 6: MATH + REASONING PEDAGOGY SPECIALIST** (`teamwork_preview_worker`)
  - Target: Stepwise semantic evaluation wired to canonical Rust StepValidator, structural/constraint reasoning.
- **Specialist 7: PHYSICS + CHEMISTRY NUMERICAL UX SPECIALIST** (`teamwork_preview_worker`)
  - Target: Dedicated numeric input with units/tolerances/fractions/scientific notation (e.g. `12 m/s`, `5 kg`, `2.5 mol`, `1.2e-3 mol/L`), dimensional correctness.
- **Specialist 8: DIAGNOSTIC / ASSESSMENT SPECIALIST** (`teamwork_preview_worker`)
  - Target: Diagnostic session engine (10-20 questions across 4 domains, hierarchical reporting Subject->Chapter->Topic->Family + Concept/Execution/Transfer/Speed, mastery/domain evidence integration).
- **Specialist: NATIVE REVIEWER INTEGRATOR** (`teamwork_preview_worker`)
  - Target: Compact mistake footer `[1 Silly]`, `[2 Pattern]`, `[3 Concept]`, `[4 Unknown]` in native footer interaction zone, standard card non-regression.

### Phase 3: Automated Quality, Security & Performance Hardening
- **Specialist 10: SECURITY / PERFORMANCE / REGRESSION SPECIALIST** (`teamwork_preview_worker` & `teamwork_preview_critic`)
  - Target: Execute test suites (`just check`, `just test-rust`, `just test-py`, `just test-ts`), verify card render performance, transition fluidity, absence of console errors, memory leaks, and XSS/HTML safety.

### Phase 4: Live QtWebEngine Desktop Verification (R5)
- **Specialist 9: QTWEBENGINE / DESKTOP-WEBVIEW SPECIALIST** (`teamwork_preview_worker` with `qt-desktop-reviewer` / `desktop-webview-reviewer`)
  - Target: Connect via CDP to running Anki dev instance, run full UI test matrix across Math, Reasoning, Physics, Chemistry, Native Anki, Diagnostic, record screenshots and output `04_live_ui_evidence.json`, `05_live_ui_screenshots/`, `06_diagnostic_live_evidence.json`.

### Phase 5: Independent Release Gating & Artifact Consolidation (R6)
- **INDEPENDENT VERIFIER & FORENSIC AUDITOR** (`teamwork_preview_auditor` / `teamwork_preview_reviewer`)
  - Target: Validate all 15 release rules, review artifacts 01 to 06, compile `07_test_summary.md`, issue formal verdict in `08_release_decision.md`.
