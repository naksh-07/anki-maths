# Original User Request

> [!NOTE]
> **Historical Artifact Notice:** This file contains historical planning and dispatch records from prior phases. The authoritative, frozen canonical documentation for the StudyLab architecture and contracts is located in [`docs/README.md`](../docs/README.md).

## Initial Request — 2026-08-24T12:00:48Z

# STUDYLAB FINAL RECONCILIATION MISSION
# Full Product Audit + Research + Architecture Reconciliation + Implementation
# + LIVE DESKTOP VERIFICATION + DIAGNOSTIC PROOF + RELEASE GATE

Working directory: c:/Users/Suraj/Documents/Antigravity/Anki-maths
Integrity mode: development

Execute the comprehensive StudyLab Final Reconciliation Mission: audit, research, reconcile architecture and answer modalities, integrate natively with Anki's reviewer and footer lifecycle, implement diagnostic mock-testing across four domains (Math, Reasoning, Physics, Chemistry), perform live QtWebEngine desktop verification, and enforce rigorous release gating.

## MISSION CONTROL & SPECIALIST ROLES
Use the full specialist workforce:
1. PRODUCT VISION / UX ARCHAEOLOGIST
2. NATIVE ANKI REVIEWER RESEARCHER
3. STUDYLAB ARCHITECTURE AUDITOR
4. CONTENT CONTRACT / APKG SPECIALIST
5. MCQ / ANSWER-MODALITY SPECIALIST
6. MATH + REASONING PEDAGOGY SPECIALIST
7. PHYSICS + CHEMISTRY NUMERICAL UX SPECIALIST
8. DIAGNOSTIC / ASSESSMENT SPECIALIST
9. QTWEBENGINE / DESKTOP-WEBVIEW SPECIALIST
10. SECURITY / PERFORMANCE / REGRESSION SPECIALIST
Plus INDEPENDENT VERIFIER for release gating.

Every specialist handoff MUST include:
- MISSION
- SCOPE
- SOURCES
- FILES / URLS INSPECTED
- FINDINGS
- EVIDENCE
- RISKS
- RECOMMENDATION
- UNKNOWN / UNVERIFIED

## Requirements

### R1. Research & Product Vision Reconciliation
- Reconcile and preserve the Product North Star: Anki is the familiar shell; StudyLab provides the procedural learning layer inside it ("Anki, but it understands how I solve problems").
- Research native Anki reviewer interaction models (answer reveal, rating footer, keyboard behavior, review lifecycle, custom constraints), exam-style MCQ UX (presentation, selection, feedback, keyboard accessibility, position bias), numerical answering (units, tolerances, scientific notation, dimensional correctness), diagnostic assessment design (concept, execution, transfer, speed; chapter/topic diagnosis; mixed-domain sampling), and reasoning assessment (structural/representation errors, logic/constraint failures).
- Inspect source of truth: `naksh-07/anki-maths`, `naksh-07/desktop-webview-reviewer`, existing Phase 40/41 walkthroughs and artifacts, Git history, official Anki docs. Independently verify all claims.
- Produce a formal gap matrix (`03_architecture_gap_matrix.md`) comparing original principles against current implementation across reviewer UI, state machines, native bridge, answer controls, footer, and learner state.

### R2. Answer Modality Contract & Content Mold Scalability
- Enforce strict modality contracts across the pipeline (APKG -> PracticeItem -> ProblemInstance -> Template -> UI):
  - MCQ: real selectable options (A-D / 1-4 keyboard navigation, canonical identity evaluation, no text input).
  - Numerical: dedicated numeric input with units/tolerances/fractions/scientific notation (e.g., Physics/Chemistry: `12 m/s`, `5 kg`, `2.5 mol`, `1.2e-3 mol/L`), avoiding artificial choices or NaN errors.
  - Stepwise: semantic step evaluation wired to the canonical Rust `StepValidator` without duplicate TS reasoning engines.
- Maintain content/mold scalability: declarative content + universal runtime mold without per-topic backend generator sprawl.

### R3. Native Anki Reviewer & Wrong-Answer Integration
- Integrate wrong-answer reflection naturally into Anki's review lifecycle: wrong result -> concise feedback -> compact Anki-like mistake footer (`[1 Silly]`, `[2 Pattern]`, `[3 Concept]`, `[4 Unknown/Prerequisite]`) -> classification -> solution/hint -> remediation/rating.
- Keep the mistake footer in the native Anki answer/footer interaction zone rather than a giant detached panel.
- Ensure non-procedural standard Anki cards (Basic, Cloze, etc.) remain 100% unaffected and native controls/shortcuts operate correctly.

### R4. Diagnostic / Mock-Test Session Engine
- Build a lightweight Diagnostic Session layer over the existing learning engine (measuring rather than aggressively adapting mid-test).
- Support 10-20 questions across mixed subjects/chapters/topics/families with time budgets.
- Generate hierarchical diagnostic reports: Subject -> Chapter -> Topic -> Problem Family and skill dimensions (Concept, Calculation/Execution, Transfer, Speed), feeding into existing `MasteryEvidence`, `DomainEvidence`, and `SkillState`.

### R5. Live QtWebEngine Desktop Verification & Gating
- Use `desktop-webview-reviewer` via QtWebEngine remote debugging (CDP attach mode against the running Anki dev instance via `tools/run.py` / `just run`).
- Execute the complete live UI test matrix:
  - Mathematics (numerical, MCQ, wrong answer, stepwise)
  - Reasoning (MCQ, structured)
  - Physics (numerical with units, MCQ)
  - Chemistry (numerical, MCQ)
  - Native Anki (Basic, Cloze, Show Answer, Again/Hard/Good/Easy, keyboard shortcuts)
  - Diagnostic (start, answer questions, finish, render hierarchical report)
- Verify card render performance, transition fluidity, absence of console errors, memory leaks, and XSS/HTML safety.

### R6. Independent Final Verification & Evidence Deliverables
- Have an independent verifier audit the complete implementation against the 15-point release rule.
- Produce the full 8-artifact evidence package:
  1. `01_research_findings.md`
  2. `02_product_reconciliation.md`
  3. `03_architecture_gap_matrix.md`
  4. `04_live_ui_evidence.json`
  5. `05_live_ui_screenshots/`
  6. `06_diagnostic_live_evidence.json`
  7. `07_test_summary.md`
  8. `08_release_decision.md`

## Acceptance Criteria

### Product & Modality Integrity
- [ ] MCQ cards render with authentic selectable options, support 1-4/A-D keyboard selection, and evaluate canonically without text inputs.
- [ ] Numerical cards accept decimal, fractional, negative, scientific notation, and physics/chemistry units without NaN or parsing crashes.
- [ ] Stepwise cards validate inputs via Rust semantic StepValidator.
- [ ] Standard Anki cards (Basic, Cloze) function natively with zero regressions.

### Reviewer & Wrong-Answer Experience
- [ ] Wrong answers trigger a compact, native-feeling mistake classification footer (`[1 Silly]`, `[2 Pattern]`, `[3 Concept]`, `[4 Unknown]`) in the primary interaction zone without scrolling.
- [ ] Rating and review lifecycle seamlessly updates learner evidence without breaking standard Anki scheduling.

### Diagnostic Session & Hierarchy
- [ ] Diagnostic session executes 10-20 items without mid-test disruption and completes with a full diagnostic breakdown.
- [ ] Diagnostic report correctly breaks down performance into Subject -> Chapter -> Topic -> Problem Family and Concept/Execution/Transfer/Speed.
- [ ] Diagnostic evidence updates existing `SkillState` and `DomainEvidence` structures without duplicate parallel state models.

### Verification & Release Gate
- [ ] Live QtWebEngine verification executed via `desktop-webview-reviewer` CDP attach mode with `RUNTIME_VERIFIED` status and screenshot evidence.
- [ ] All automated unit, integration, and APKG test suites pass (`just check`, `just test-rust`, `just test-py`, `just test-ts`).
- [ ] Zero unhandled console errors, no memory leaks or stale listeners on card transitions.
- [ ] Independent verification validates all criteria and generates the complete 8-part evidence package.
- [ ] Final release verdict is explicitly justified as 🟢 RELEASE READY, 🟡 CONDITIONAL, or 🔴 NOT RELEASE READY according to Section 25.

## Follow-up — 2026-08-24T13:21:36Z

# STUDYLAB FINAL RECONCILIATION MISSION
# Full Product Audit + Research + Architecture Reconciliation + Implementation
# + LIVE DESKTOP VERIFICATION + DIAGNOSTIC PROOF + RELEASE GATE

Working directory: c:/Users/Suraj/Documents/Antigravity/Anki-maths
Integrity mode: development

Execute the comprehensive StudyLab Final Reconciliation Mission: audit, research, reconcile architecture and answer modalities, integrate natively with Anki's reviewer and footer lifecycle, implement diagnostic mock-testing across four domains (Math, Reasoning, Physics, Chemistry), perform live QtWebEngine desktop verification, and enforce rigorous release gating.

## MISSION CONTROL & SPECIALIST ROLES
Use the full specialist workforce:
1. PRODUCT VISION / UX ARCHAEOLOGIST
2. NATIVE ANKI REVIEWER RESEARCHER
3. STUDYLAB ARCHITECTURE AUDITOR
4. CONTENT CONTRACT / APKG SPECIALIST
5. MCQ / ANSWER-MODALITY SPECIALIST
6. MATH + REASONING PEDAGOGY SPECIALIST
7. PHYSICS + CHEMISTRY NUMERICAL UX SPECIALIST
8. DIAGNOSTIC / ASSESSMENT SPECIALIST
9. QTWEBENGINE / DESKTOP-WEBVIEW SPECIALIST
10. SECURITY / PERFORMANCE / REGRESSION SPECIALIST
Plus INDEPENDENT VERIFIER for release gating.

Every specialist handoff MUST include:
- MISSION
- SCOPE
- SOURCES
- FILES / URLS INSPECTED
- FINDINGS
- EVIDENCE
- RISKS
- RECOMMENDATION
- UNKNOWN / UNVERIFIED

## Context on Current Progress
- Phase 1 research artifacts already authored and available in root and `.agents/`:
  - `01_research_findings.md`
  - `02_product_reconciliation.md`
  - `03_architecture_gap_matrix.md`
- MCQ Modality component implemented & verified in `ts/reviewer/components/mcq_container.ts` & `mcq_container.test.ts`.
- Mistake footer component implemented in `ts/reviewer/components/mistake_footer.ts`.
- Python bridge dispatcher implemented in `qt/aqt/reviewer.py`.
- Teardown MutationObserver lifecycle active in `ts/reviewer/procedural.ts`.

## Remaining Core Execution Tracks
- Specialist 6: Complete Math + Reasoning Stepwise Rust `StepValidator` wiring and validation.
- Specialist 7: Complete Physics + Chemistry Numerical units parser (m/s, mol/L, kg, sci notation), tolerances, dimensional correctness.
- Specialist 8: Complete Diagnostic Mock Test Session Engine across 4 domains, 4-tier hierarchy (Subject->Chapter->Topic->Family), Concept/Execution/Transfer/Speed dimensions, batch `SkillState`/`DomainEvidence` updates, and TS report UI.
- Specialist 10: Run full test automation (`just check`, `just test-rust`, `just test-py`, `just test-ts`), security audits, and performance checks.
- Specialist 9: Run Live QtWebEngine Desktop Verification via `desktop-webview-reviewer` CDP attach mode against running Anki dev instance, generate screenshots and `04_live_ui_evidence.json`, `05_live_ui_screenshots/`, `06_diagnostic_live_evidence.json`.
- Independent Verifier: Complete 15-point release gate audit, produce `07_test_summary.md`, and issue `08_release_decision.md`.

## Requirements & Deliverables
All 8 artifacts must be generated and verified:
1. `01_research_findings.md`
2. `02_product_reconciliation.md`
3. `03_architecture_gap_matrix.md`
4. `04_live_ui_evidence.json`
5. `05_live_ui_screenshots/`
6. `06_diagnostic_live_evidence.json`
7. `07_test_summary.md`
8. `08_release_decision.md`

Execute the remaining mission with maximum quality and release gate rigor.

