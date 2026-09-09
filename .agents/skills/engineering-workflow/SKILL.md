---
name: engineering-workflow
description: >-
  Authoritative development lifecycle, implementation spine, and quality gate skill
  for the StudyLab procedural learning subsystem inside Anki (naksh-07/anki-maths).
  Defines the multi-phase engineering protocol from understanding and planning to bottom-up
  implementation, independent verification, adversarial auditing, and evidence-based completion gates.
---

# StudyLab Engineering Workflow & Quality Lifecycle

The `engineering-workflow` skill defines the canonical development lifecycle and quality gating process for engineering tasks on the StudyLab procedural intelligence subsystem inside Anki (`naksh-07/anki-maths`).

Use this skill when you need to:
- Plan, structure, and execute changes across Rust core, TypeScript reviewer, Python/Qt bridge, or content tools.
- Coordinate multi-agent workflows across the specialist subagents (`explorer`, `implementer`, `reviewer-verifier`, `challenger-auditor`).
- Implement features following the strict bottom-up dependency ordering and frozen safety invariants.
- Execute independent verification (Tier 2) and adversarial victory auditing (Tier 3/4) before claiming completion.

---

## 1. Specialist Subagent Collaboration Model

Engineering in StudyLab is governed by strict role specialization and permission boundaries:

```text
┌──────────────────────────────────────────────────────────────────────────────────┐
│                     STUDYLAB MULTI-AGENT COLLABORATION FLOW                      │
└──────────────────────────────────────────────────────────────────────────────────┘

   [ User Request / Task ]
              │
              ▼
   ┌──────────────────────┐
   │       EXPLORER       │ ──► Read-only reconnaissance, canonical docs navigation,
   │  (Reconnaissance)    │     polyglot flow tracing, symbol ledger lookups.
   └──────────┬───────────┘
              │ [Reconnaissance Handoff Report]
              ▼
   ┌──────────────────────┐
   │     IMPLEMENTER      │ ──► Write-set exclusivity, bottom-up dependency spine,
   │  (Controlled Writer) │     safety invariants enforcement, local Tier 1 checks.
   └──────────┬───────────┘
              │ [Implementation Handoff Report]
              ▼
   ┌──────────────────────┐
   │  REVIEWER-VERIFIER   │ ──► Independent Tier 2 verification, zero-trust execution,
   │ (Independent Review) │     benchmark integrity (no test weakening), diff audit.
   └──────────┬───────────┘
              │ [Verification Handoff Report: PASSED]
              ▼
   ┌──────────────────────┐
   │  CHALLENGER-AUDITOR  │ ──► Adversarial Tier 3 stress, 16 Invariants audit,
   │ (Victory Gatekeeper) │     Challenger 2 Master Runner, whole-mission acceptance.
   └──────────┬───────────┘
              │ [VICTORY CONFIRMED]
              ▼
   [ Final Mission Delivery ]
```

### Role Summary & Permitted Primitives
- **`explorer`** (`model: flash`): 100% read-only (`view_file`, `grep_search`, `find_by_name`, `list_dir`). No file writes, no command execution.
- **`implementer`** (`model: pro`): Controlled writer (`write_to_file`, `replace_file_content`, `run_command`). Adheres to write-set exclusivity and bottom-up spine.
- **`reviewer-verifier`** (`model: flash`): Read & execute only (`run_command`, read tools). Never edits source code; independently executes tests and audits diffs.
- **`challenger-auditor`** (`model: pro`): Read & execute only (`run_command`, read tools). Ultimate adversarial gatekeeper; runs hostile stress tests and emits `VICTORY CONFIRMED` or `AUDIT FAILED`.

---

## 2. The 6-Stage Practical StudyLab Engineering Lifecycle

Every non-trivial engineering task follows the six sequential stages:

```text
┌────────────────────────────────────────────────────────────────────────┐
│               THE 6-STAGE STUDYLAB ENGINEERING LIFECYCLE               │
├────────────────────────────────────────────────────────────────────────┤
│ Stage 1: Understanding & Wayfinding                                    │
│          - Consult navigator skill and canonical reading paths         │
│          - Establish ground truth via 8-tier hierarchy                 │
│          - Differentiate Source APKG vs Blueprint pathways             │
├────────────────────────────────────────────────────────────────────────┤
│ Stage 2: Task Sizing & Architectural Planning                          │
│          - Size change (Small, Medium, Large)                          │
│          - Define write-set exclusivity boundaries                     │
│          - Identify affected polyglot touchpoints                      │
├────────────────────────────────────────────────────────────────────────┤
│ Stage 3: Bottom-Up Implementation Spine                                │
│          - Tier 1: Contracts & Schemas                                 │
│          - Tier 2: Solvers, Units & Skill Engines                      │
│          - Tier 3: Service Facade & 4 Rust Touchpoints                 │
│          - Tier 4: Python / PyQt Host Reviewer Bridge                  │
│          - Tier 5: TypeScript Open Canvas Reviewer                     │
├────────────────────────────────────────────────────────────────────────┤
│ Stage 4: Local Tier 1 Self-Verification                                │
│          - Fast compilation check (`cargo check -p procedural`)        │
│          - Unit tests (`cargo test -p procedural --lib`)               │
│          - Targeted integration test suites                            │
│          - Formatting & linter checks (`just fix-fmt`, `just lint`)    │
├────────────────────────────────────────────────────────────────────────┤
│ Stage 5: Independent Tier 2 Verification                               │
│          - Reviewer-Verifier zero-trust verification                   │
│          - Benchmark Integrity Mandate (zero test weakening)           │
│          - Diff safety checklist (SQL, HTML escaping, 100-byte limit)  │
│          - Disposition (`PASSED` | `REPAIRABLE` | `UNREPAIRABLE`)      │
├────────────────────────────────────────────────────────────────────────┤
│ Stage 6: Adversarial Tier 3 & Victory Tier 4 Gate                      │
│          - Challenger-Auditor 4-pillar adversarial audit               │
│          - Challenger 2 Master Runner execution                        │
│          - Whole-mission acceptance checklist & Victory Confirmation   │
└────────────────────────────────────────────────────────────────────────┘
```

---

### Stage 1: Understanding & Wayfinding

1. **Wayfinding-First Orientation**:
   - Orient using `.agents/skills/navigator/SKILL.md` and `.agents/skills/navigator/references/project-map.md`.
   - Never run unguided search floods across the repository.
2. **Follow Canonical Reading Paths (`docs/DOCUMENTATION_MAP.md`)**:
   - AI Agent Fast-Start: `docs/STUDYLAB_PRODUCT_CONTRACT.md` ➔ `docs/contracts/` ➔ `.agents/rules/architecture-boundaries.md` ➔ `docs/ARCHITECTURE_INVARIANTS.md` ➔ `docs/FINAL_LIVE_UI_FORENSIC_REPORT.md` ➔ `docs/SYSTEM_ARCHITECTURE.md`.
   - Rust Core: `docs/SYSTEM_ARCHITECTURE.md` ➔ `docs/DATABASE_DATA_CONTRACT.md` ➔ `docs/LEARNING_MODEL.md` ➔ `docs/DIAGNOSTIC_AND_REMEDIATION.md`.
   - Frontend TS: `docs/contracts/` ➔ `docs/FRONTEND_PRODUCT_SPEC.md` ➔ `docs/REVIEWER_STATE_MACHINE.md` ➔ `docs/FRONTEND_BACKEND_CONTRACT.md`.
   - Python / Qt: `docs/PRODUCT_BOUNDARIES.md` ➔ `docs/FRONTEND_BACKEND_CONTRACT.md` ➔ `.agents/rules/architecture-boundaries.md`.
   - Content Tools: `StudyLab-Source-APKG-Contract(1).txt` (Level 1 Frozen) ➔ `docs/APKG_CONTENT_CONTRACT.md` ➔ `tools/studylab_content_factory.py`.
3. **8-Tier Source-of-Truth Grounding**:
   - Tier 1: Executable Source Code (`rslib/`, `ts/`, `qt/`, `pylib/`).
   - Tier 2: Current Passing Test Suites.
   - Tier 5: Frozen Master Contracts (`StudyLab-Source-APKG-Contract(1).txt`, `docs/contracts/`).
   - Tier 7: Historical Phase Reports (`01_`–`08_`, `HANDOFF_REPORT.md`) are point-in-time archaeological logs and NEVER override current code or contracts.
4. **Dual Content Pathway Separation**:
   - *Path 1 (Source APKG)*: Curated static items parsed into `SourceQuestion` (`rslib/procedural/src/anchor/source.rs`). Generators completely bypassed.
   - *Path 2 (Procedural Blueprints)*: Declarative problem blueprints generating dynamic variants via `DeclarativeFamilyContract`.

---

### Stage 2: Task Sizing & Architectural Planning

Before modifying any file, size the task and define write boundaries:

| Task Size | Scope & Criteria | Required Process | Subagents |
|---|---|---|---|
| **Small (Surgical)** | Bug fix within a single file, adding a unit test, updating documentation. | Lightweight plan, direct surgical edit, local verification. | `implementer` ➔ `reviewer-verifier` |
| **Medium (Subsystem)** | Modifying a solver, updating template rendering, adding an IPC command, schema migration. | Formal plan, dependency analysis, bottom-up ordering, Tier 2 verification. | `explorer` ➔ `implementer` ➔ `reviewer-verifier` |
| **Large (Architectural)** | Cross-language changes (Rust + Qt + TS), modifying touchpoints, adding content domains. | Full implementation plan with user review, multi-agent dispatch, full Tier 1–4 verification gates. | `explorer` ➔ `implementer` ➔ `reviewer-verifier` ➔ `challenger-auditor` |

#### Defining Write-Set Exclusivity
- Identify the minimal list of target files before editing.
- Upstream Anki core files outside the 4 explicit Rust touchpoints and 1 Python bridge are **STRICTLY OFF-LIMITS**.
- Non-procedural flashcard code paths (`Basic`, `Cloze`, Image Occlusion) and upstream collection database (`collection.anki2`) must remain 100% untouched.

---

### Stage 3: Bottom-Up Implementation Spine

When implementing or modifying features, enforce the strict bottom-up dependency ordering:

```text
Tier 1: Contracts & Data Schemas
        - Master Contracts: docs/contracts/, StudyLab-Source-APKG-Contract(1).txt
        - Rust Schemas: rslib/procedural/src/storage/{schema,migration}.rs
        - Blueprint Contracts: rslib/procedural/src/problems/contract.rs
Tier 2: Domain Solvers, Units & Cognitive Engines
        - AST Solvers & Steps: rslib/procedural/src/problems/steps/
        - 5D Dimensional Analysis: rslib/procedural/src/units/
        - Cognitive Mastery: rslib/procedural/src/skills/
        - Remediation Queue: rslib/procedural/src/remediation/
Tier 3: Service Facade & Core Rust Touchpoints
        - Storage Store: rslib/procedural/src/storage/store.rs
        - ProceduralService Facade: rslib/procedural/src/service/mod.rs
        - Touchpoint 1 (Storage): rslib/src/collection/mod.rs
        - Touchpoint 2 (Render): rslib/src/notetype/render.rs
        - Touchpoint 3 (Answering & Telemetry): rslib/src/scheduler/answering/mod.rs
        - Touchpoint 4 (APKG Reconciliation): rslib/src/import_export/package/apkg/import/mod.rs
Tier 4: Python / PyQt Host Reviewer Bridge
        - IPC Dispatcher & Button Suppression: qt/aqt/reviewer.py
        - Spacebar/Enter Anti-Bypass: qt/aqt/reviewer.py (onEnterKey)
        - Python Bindings: pylib/
Tier 5: Frontend TypeScript Open Canvas Reviewer
        - State Machine & Teardown: ts/reviewer/procedural.ts
        - Modalities: ts/reviewer/components/{mcq,numerical,stepwise}_container.ts
```

#### Mandatory Safety Rules During Implementation
1. **100% Parameterized SQL (Zero String Interpolation)**:
   - In `rslib/procedural/src/storage/store.rs`, use numbered parameters (`?1, ?2, ...`) or `rusqlite::params!`. Never use `format!` or string concatenation in SQL.
2. **Webview HTML Escaping & XSS Sanitization**:
   - Dynamic strings passed into templates must be escaped via `escape_html()` in `rslib/procedural/src/reviewer/template.rs`.
   - JSON payloads inside `<script>` tags must escape `</script>` as `<\/script>`.
3. **Webview Lifecycle Teardown**:
   - `destroyActive()` must unbind all event listeners, timers, and observers (`ts/reviewer/procedural.ts:1453`).
   - Host bridge must evaluate `globalThis.anki.procedural.destroyActive()` before mounting any card (`qt/aqt/reviewer.py:208, 416`).
4. **100-Byte Custom Data Limit Firewall**:
   - Telemetry must be committed to `<col_path>.procedural` and stripped from `card.custom_data` before saving to `collection.anki2` (`rslib/src/scheduler/answering/mod.rs:501–506`).
5. **Two-P0 Desktop Guardrails (`qt/aqt/reviewer.py`)**:
   - P0-A: `onEnterKey()` traps Space/Enter and delegates to `handleNativeShowAnswer()`.
   - P0-B: Native Anki ease buttons and `#ansbut` are suppressed via `_is_procedural_card()`.
   - Modality Purity: MCQ enforces zero text-input fallback (`MCQContainer`). No "Next Card" or "Next Problem" button.

---

### Stage 4: Local Tier 1 Self-Verification

The `implementer` must run local self-checks before requesting independent verification:

```powershell
# 1. Fast Rust compilation check (0.2s)
cargo check -p procedural

# 2. Rust unit test suite (146 lib tests, 0.09s)
cargo test -p procedural --lib

# 3. Targeted integration test corresponding to change
cargo test -p procedural --test phase36c_all_175_topics_factory_tests
cargo test -p procedural --test canonical_source_contract_tests
cargo test -p procedural --test desktop_validation_master_suite
cargo test -p procedural --test phase40_production_hardening_tests

# 4. Declarative blueprint validation (if touching content/blueprints)
python tools/studylab_content_factory.py --validate

# 5. Format & Linting
just fix-fmt
just lint
```

> [!WARNING]
> Do NOT run unconstrained `cargo test -p procedural` (runs 4+ minute simulations) or `cargo test --workspace` directly (requires FTL build hooks configured via `just test-rust`).

---

### Stage 5: Independent Tier 2 Verification

The `reviewer-verifier` validates the deliverable in an independent context:

1. **Zero Trust Policy**: Never accept self-certification from the implementer.
2. **Benchmark Integrity Mandate (`.agents/rules/verification-safety.md`)**:
   - Inspect `git diff` to ensure no tests were deleted, commented out, skipped, or had assertions loosened.
   - Ensure all passes represent real, genuine computation.
3. **Execution of Covering Test Suites**:
   - Run Rust unit and integration checks.
   - Run Python test suite (`just test-py` or `pytest qt/tests pylib/tests`).
   - Run APKG validators (`artifacts_qa/validate_canonical_source_apkg.py`, `artifacts_qa/validate_canonical_apkg.py`).
4. **Safety Invariants Diff Checklist**:
   - Verify SQL parameterization (`store.rs`).
   - Verify HTML escaping (`template.rs`).
   - Verify `destroyActive()` lifecycle teardown.
   - Verify 100-byte telemetry firewall.
   - Verify Two-P0 desktop guardrails.
   - Verify zero modifications to standard Anki cards or collection schemas.
5. **Disposition Classification**:
   - `PASSED`: All checks green, diff clean, safety invariants satisfied.
   - `REPAIRABLE_FAILURE`: Localized issue (lint error, minor assertion failure) returned to `implementer` with exact reproduction steps.
   - `UNREPAIRABLE_FAILURE`: Fundamental invariant breach (schema pollution, contract violation) requiring architectural re-planning.

---

### Stage 6: Adversarial Tier 3 & Victory Tier 4 Gate

The `challenger-auditor` executes whole-mission Victory Auditing:

1. **Pillar 1: Write-Set Exclusivity & Repository Hygiene**:
   - Verify `git status` and `git diff --stat` against declared scope. Zero stray debug files, zero untouched subsystem pollution.
2. **Pillar 2: Anti-Mocking & Genuine Computation**:
   - Verify real algorithms, zero fake stubs, zero hardcoded test outputs.
3. **Pillar 3: StudyLab Frozen Invariants Audit**:
   - Verify all 16 Invariants, Two-P0 guardrails, and master contracts.
4. **Pillar 4: Adversarial Probes & Master Runner**:
   - Execute Challenger 2 Release Candidate Master Runner:
     ```powershell
     python artifacts_qa/challenger_2_master_runner.py
     ```
   - Verify all 4 audit dimensions pass (Math CAS, DB Persistence, 100-Byte Telemetry, Cold-Start APKG).
5. **Victory Decision**:
   - Issue unambiguous verdict: `VICTORY CONFIRMED` or `AUDIT FAILED`.

---

## 3. Parallel Execution & Worktree Hygiene

When executing parallel tasks or delegating work to concurrent subagents:

1. **Worktree Isolation**:
   - Use isolated git worktrees (`git worktree add ../<branch-name>`) for subagents performing concurrent implementations.
   - Never have multiple subagents write to the same working directory concurrently.
2. **Merge Hygiene**:
   - Cleanly rebase feature branches onto `main`.
   - Run the full verification suite (`cargo check`, `cargo test --lib`, `just lint`, `python artifacts_qa/challenger_2_master_runner.py`) on the integrated branch before final commit.
3. **Clean Teardown**:
   - Remove temporary worktrees (`git worktree remove`) and purge scratch branches.

---

## 4. Standard Handoff Report Protocols

Handoff reports between subagents must follow standardized, structured templates:

### Reconnaissance Handoff (Explorer ➔ Implementer / Lead)
```text
### RECONNAISSANCE HANDOFF REPORT
- OBJECTIVE:           [Goal and investigated flow]
- RECONNAISSANCE_PATH: [Inspected files and canonical docs]
- OBSERVATIONS:        [Factual findings with exact file paths and lines]
- LOGIC_CHAIN:         [Root cause analysis or execution trace]
- EVIDENCE:            [Code snippets, structs, or test results]
- BOUNDARY_IMPACT:     [Rust core | TypeScript UI | Python bridge | Content]
- CAVEATS:             [Edge cases, unverified assumptions]
- CONCLUSION:          [Actionable summary]
- VERIFICATION_METHOD: [Exact command next owner should run]
- NEXT_OWNER:          [implementer | reviewer-verifier | Lead]
```

### Implementation Handoff (Implementer ➔ Reviewer-Verifier)
```text
### IMPLEMENTATION HANDOFF REPORT
- OBJECTIVE:             [Implemented feature or fix]
- WRITE_SET_EXCLUSIVITY: [List of modified files — zero out-of-scope edits]
- CHANGES_APPLIED:       [Summary of code modifications and rationale]
- INVARIANTS_MAINTAINED: [Confirmation of parameterized SQL, HTML escaping, Two-P0, 100-byte limit]
- LOCAL_VERIFICATION:    [Commands run, test counts, exit codes]
- ARTIFACTS_DELIVERED:   [Delivered files or blueprints]
- CAVEATS:               [Known trade-offs or remaining edge cases]
- CONCLUSION:            [Readiness for independent verification]
- NEXT_OWNER:            [reviewer-verifier]
```

### Verification Handoff (Reviewer-Verifier ➔ Challenger-Auditor / Implementer)
```text
### VERIFICATION HANDOFF REPORT
- OBJECTIVE:             [Verification target and files under review]
- VERIFICATION_COMMANDS: [Commands executed with options]
- TEST_RESULTS:          [Pass count / Total count, durations, exit codes]
- DIFF_AUDIT:            [Clean diff confirmed / Unintended changes detected]
- SAFETY_INSPECTION:     [SQL: PASS/FAIL | HTML: PASS/FAIL | 100-Byte: PASS/FAIL]
- REGRESSION_RISK:       [Low | Medium | High with justification]
- CLASSIFICATION:        [PASSED | REPAIRABLE_FAILURE | UNREPAIRABLE_FAILURE]
- FAILURE_DETAILS:       [Error logs, line references if failed]
- CONCLUSION:            [Tier 2 Passed or Actionable Repair Request]
- NEXT_OWNER:            [challenger-auditor | implementer]
```

### Challenger Audit Handoff (Challenger-Auditor ➔ Mission Lead)
```text
### CHALLENGER AUDIT HANDOFF REPORT
- OBJECTIVE:             [Adversarial audit or Victory gate check]
- WRITE_SET_INTEGRITY:   [VERIFIED: No undeclared writes | BREACH DETECTED]
- ANTI_MOCKING_CHECK:    [GENUINE: Real assertions verified | TEST WEAKENING DETECTED]
- INVARIANT_AUDIT:       [Two-P0: PASS/FAIL | 100-Byte: PASS/FAIL | Modality: PASS/FAIL]
- ADVERSARIAL_PROBES:    [Edge cases, hostile inputs, Master Runner outcome]
- ACCEPTANCE_CHECKLIST:  [Prompt requirements audit: All Satisfied | Gaps Found]
- VERDICT:               [VICTORY CONFIRMED | AUDIT FAILED]
- FAILURE_ANALYSIS:      [Root cause if failed, or None]
- NEXT_OWNER:            [Lead (Delivery) | implementer (Remediation)]
```
