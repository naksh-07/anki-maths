# Original User Request

## 2026-08-24T20:30:44Z

# Teamwork Project Prompt — Draft

> Status: Step 9 — Ready for launch — awaiting user approval
> Goal: Craft prompt → get user approval → delegate to teamwork_preview
> Requested team: Use a very large team of agents (Full team requested with 10+ specialists).

Make the StudyLab documentation complete enough that a fresh AI agent can understand the entire system correctly without conversation history, hidden assumptions, old phase reports, or guessing. The repository itself must become the source of truth for future AI work. This is a documentation and source-truth reconciliation mission ONLY. NO production-code behavior changes, frontend redesign, backend refactor, APKG schema changes, or database migration changes.

Working directory: C:\Users\Suraj\Documents\Antigravity\Anki-maths
Integrity mode: benchmark (strict adherence to source-of-truth hierarchy, no code modifications allowed)

## Core Directives & Source of Truth

**Source-of-Truth Hierarchy:**
Resolve claims in this order:
1. current executable source code
2. current tests
3. current schemas / contracts / migrations
4. current verified artifacts
5. explicit product requirements
6. current canonical docs
7. historical phase reports
8. general assumptions

*Existing docs are NOT automatically correct. Existing code is NOT automatically product intent. If intent and implementation disagree: DOCUMENT THE DRIFT. Do not silently rewrite either.*

**DeepSearch Integration:**
You must use the `deepsearch` capability/skill for external research, historical/architectural reconciliation, difficult technical questions, primary-source verification, and identifying claims that cannot be established from repository source. Do not use DeepSearch merely to produce generic prose.

## Requirements

### R1. Full Repository Archaeology
Inspect the following before writing: `docs/`, `.agents/`, `ORIGINAL_REQUEST`, all StudyLab procedural Rust, scheduler/answering integration, skills, SkillState, MasteryEvidence, DomainEvidence, remediation, adaptive difficulty, ProblemFamilyContract, declarative runtime, learning objects, reviewer template, TypeScript reviewer, Python/Qt bridge, persistence/store, migrations, APKG tooling, diagnostic/mock engine, tests, release/hardening artifacts, and relevant git history.

### R2. Documentation Gap Matrix
Before editing docs, build `docs/DOCUMENTATION_TRUTH_MATRIX.md`.
Columns: AREA, CURRENT CLAIM, ACTUAL CODE EVIDENCE, TEST EVIDENCE, PRODUCT INTENT, STATUS (GREEN/YELLOW/RED), REQUIRED DOC CHANGE.
Areas MUST include: product identity, Anki boundary, content architecture, APKG boundary, procedural runtime, learning objects, frontend state machine, frontend/backend bridge, learner state, database persistence, domain evidence, adaptive difficulty, remediation, diagnostic sessions, security, performance, developer workflow, release workflow.

### R3. Final Canonical Document Set Production
Keep this structure unless evidence proves it insufficient. You MUST add `DATA_AND_PERSISTENCE.md`.
Produce/Update: `docs/README.md`, `docs/PRODUCT_VISION.md`, `docs/PRODUCT_BOUNDARIES.md`, `docs/SYSTEM_ARCHITECTURE.md`, `docs/LEARNING_MODEL.md`, `docs/CONTENT_AND_AUTHORING.md`, `docs/LEARNING_OBJECTS.md`, `docs/REVIEWER_STATE_MACHINE.md`, `docs/FRONTEND_BACKEND_CONTRACT.md`, `docs/DATA_AND_PERSISTENCE.md`, `docs/DIAGNOSTIC_AND_REMEDIATION.md`, `docs/ARCHITECTURE_INVARIANTS.md`, `docs/DOCUMENTATION_MAP.md`, `docs/OPEN_QUESTIONS.md`, `docs/DOCUMENTATION_TRUTH_MATRIX.md`, and `docs/DEEPSEARCH_EVIDENCE.md`.
*Refer to the original user request for the exact required contents of each of these files.*

### R4. DeepSearch Research Track
Use DeepSearch to answer these specific questions and reconcile with code:
A. What exactly should a problem-solving learning system measure beyond correctness?
B. How should diagnostic practice separate: concept, execution, transfer, speed?
C. What evidence supports structured/stepwise problem-solving assessment?
D. What should be considered a learning-object modality versus a flashcard?
E. What are the cleanest boundaries between a host SRS system and a procedural learning engine?
F. For Math/Reasoning/Physics/Chemistry, what failure dimensions are pedagogically meaningful and which are weak proxies?
G. Which current StudyLab architectural claims are unsupported by external evidence and must be treated as product decisions rather than research facts?

### R5. Architecture Invariants & Open Questions
Freeze non-negotiables in `ARCHITECTURE_INVARIANTS.md` (e.g., StudyLab is not a flashcard product, do not recreate Anki's flashcards/FSRS, etc.). Update `OPEN_QUESTIONS.md` to remove anything answered by code/research, keeping only genuine unknowns (unresolved product choices, ambiguous architecture intent).

## Acceptance Criteria

### Documentation Consistency Audit
- [ ] No contradictory definitions exist across canonical docs for core terms: Anki, StudyLab, flashcard, reviewer, learning object, evaluation, SkillState, DomainEvidence, APKG, procedural.db, diagnostic, remediation, FSRS.
- [ ] "StudyLab is not a flashcard system" is strictly upheld and never drifts into "StudyLab is an enhanced flashcard reviewer."

### Documentation Self-Test (Verification)
- [ ] A clean-context AI agent given ONLY `docs/README.md` and the canonical docs can answer the 16 core questions defined in the mission brief (What is StudyLab, What does Anki own, Trace a problem, Explain remediation, etc.) without conversation history.

### Documentation Quality Score
- [ ] Every individual canonical document scores >= 90/100 (Accuracy, Completeness, Traceability, Clarity, AI usefulness).
- [ ] The overall documentation suite scores >= 95/100.

### Final Freeze Check
- [ ] ALL 18 items in the "FINAL FREEZE CHECK" list (from vision explicit to fresh-AI self-test succeeds) are verified GREEN.

### Final Report Delivery
- [ ] Deliver a Final Report matching the requested structure (Executive Verdict, Vision vs Code Alignment, Missing/Corrected, Statuses for all areas, DeepSearch findings, Scorecard, Open Questions, Self-Test, and FINAL FREEZE VERDICT).

## 2026-08-26T06:42:07Z

# Teamwork Project Prompt — Draft

> Status: Launched
> Goal: Craft prompt → get user approval → delegate to teamwork_preview
> Requested team: Full team (implied by the broad scope and audit requirements)

Perform ONE FINAL FULL-SYSTEM AUDIT for StudyLab, a procedural problem-solving and adaptive learning engine hosted inside the Anki desktop runtime, treating the repository as a RELEASE CANDIDATE. 

Working directory: c:\Users\Suraj\Documents\Antigravity\Anki-maths
Integrity mode: benchmark

## Requirements

### R1. Documentation ↔ Code Reconciliation
Trace the system end-to-end (APKG → Anki collection → ProceduralPayload → Rust → DB → Qt Reviewer → Webview → TS state machine). Read all canonical docs in docs/ and compare them against the actual implementation. Fix any contradictions (e.g., duplicate controls, wrong answer entry modalities).

### R2. Frontend & Modality Audit
Audit frontend state transitions, render lifecycle, keyboard handling, and UI/UX for all modalities (mcq, concept_check, strategy_drill, stepwise, etc.). Ensure there are no UI leakages between cards, no duplicate Next buttons, and no textboxes where they shouldn't exist. Fix all P0 and P1 bugs.

### R3. Desktop Layout & Visual Forensic Review
Ensure the UI is designed for a laptop (test 1366×768 to 1920×1080). It must not feel like a web app or giant dashboard. Primary problems must be visible immediately without awkward scrolling.

### R4. APKG & Database Verification
Verify generated canonical APKG and Database persistence (attempt persistence, skill state, orphan cleanup, migration safety). Ensure Anki's normal functionality (Basic, Cloze) remains completely untouched by StudyLab CSS/events.

### R5. Bug Register and Fix Policy
Produce docs/FINAL_RELEASE_AUDIT.md. Fix all P0 (release blockers) and P1 (important before release) bugs. Do NOT redesign the backend, change DB schemas casually, or change FSRS behavior. Perform the smallest correct fix.

### R6. Second Mandatory Audit
After fixing issues, rebuild, rerun automated tests, rerun APKG validation, rerun the full live desktop matrix, and compare BEFORE vs AFTER. No defect may be marked resolved without verification.

## Verification Resources
- Use the desktop-webview-reviewer for live, visible desktop verifications.
- Automated tests: 
pm run vitest:once and cargo test -p procedural.
- Reviewer tests: Python/Qt tests and build verification.

## Acceptance Criteria

### Live Desktop Verification
- [ ] The audit MUST use the REAL visible Windows Anki DEV GUI via desktop-webview-reviewer. Headless runs or fake viewports are strictly prohibited.
- [ ] Captured dual screenshots (native desktop + webview) proving real execution for all tested user flows.

### Product Quality & Regressions
- [ ] Normal Anki (Basic, Cloze) is clean and unaffected.
- [ ] No P0/P1 bugs remain in the bug register.
- [ ] No raw telemetry/debug UI is visible.
- [ ] Canonical APKG is valid and DB persistence is valid.

### Release Artifacts
- [ ] docs/FINAL_RELEASE_AUDIT.md is complete with Bug ID, Root Cause, Fix, and Live Evidence.
- [ ] docs/FINAL_RELEASE_NOTES.md is generated.
- [ ] rtifacts_qa/final_release_audit/evidence.json is produced containing all screenshots, hashes, and validation results.
- [ ] Final release verdict (🟢 RELEASE READY, 🟡 RELEASE CANDIDATE, or 🔴 RELEASE BLOCKED) is declared based on the objective criteria above.
