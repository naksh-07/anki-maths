---
name: optimization
description: >-
  Resource efficiency, progressive disclosure, and context management skill for working
  on the StudyLab procedural learning subsystem inside Anki (naksh-07/anki-maths).
  Defines efficient repository exploration, avoidance of heavy generated/build artifacts,
  minimal sufficient context, and host compilation/test optimization without sacrificing correctness.
---

# StudyLab Optimization & Context Efficiency Skill

The `optimization` skill provides agents and developers with concrete strategies for maximizing engineering velocity, context efficiency, and host machine resource utilization when working on the StudyLab procedural intelligence subsystem inside Anki (`naksh-07/anki-maths`).

Use this skill when you need to:
- Navigate and explore the codebase without exhausting context windows or triggering unguided search floods.
- Identify and avoid heavy generated, binary, and build artifacts (`.apkg`, SQLite databases, build directories).
- Execute fast, progressive verification loops instead of running expensive, unconstrained test suites.
- Maintain minimal sufficient context across multi-agent handoffs without sacrificing correctness or rigor.

---

## 1. Core Philosophy: Correctness First, Efficiency Always

```text
┌──────────────────────────────────────────────────────────────────────────────────┐
│                           CORE OPTIMIZATION PRINCIPLE                            │
├──────────────────────────────────────────────────────────────────────────────────┤
│ "Optimization never means cutting corners on verification."                      │
│                                                                                  │
│ • DO NOT skip required verification steps or weaken test assertions.             │
│ • DO NOT guess at code or contracts to avoid reading documentation.              │
│ • DO eliminate redundant directory exploration, unconstrained grep floods, and   │
│   costly multi-minute longitudinal simulations when fast checks exist.           │
│ • Use progressive disclosure: retrieve the smallest sufficient slice of data     │
│   needed to make an informed, correct engineering decision.                      │
└──────────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Progressive Disclosure & Wayfinding Discipline

Navigating a massive polyglot repository like Anki + StudyLab requires structured, wayfinding-first exploration:

```text
┌────────────────────────────────────────────────────────────────────────┐
│                   PROGRESSIVE DISCLOSURE PROTOCOL                      │
├────────────────────────────────────────────────────────────────────────┤
│ Level 1: Authoritative Orientation                                     │
│          - Consult .agents/skills/navigator/SKILL.md (30-sec model)    │
│          - Check .agents/skills/navigator/references/project-map.md    │
│            for verified paths, key symbols, and commands               │
├────────────────────────────────────────────────────────────────────────┤
│ Level 2: Canonical Documentation Reading Paths                         │
│          - Follow docs/DOCUMENTATION_MAP.md Paths 1–5                  │
│          - Jump directly to the relevant specification                 │
│          - Skip historical reports (01_ through 08_)                   │
├────────────────────────────────────────────────────────────────────────┤
│ Level 3: Targeted Line-Bounded File Slices                             │
│          - Use view_file with explicit StartLine and EndLine           │
│          - Read only relevant struct/function implementations          │
├────────────────────────────────────────────────────────────────────────┤
│ Level 4: Surgical In-File Grep / Search                                │
│          - Use grep_search with SearchPath set to a single directory   │
│            or file, filtering by extension                             │
└────────────────────────────────────────────────────────────────────────┘
```

### 2.1 Wayfinding Before Searching
- **The Anti-Pattern**: Running broad `find_by_name` with `Pattern: "*"` or repository-wide `grep_search` across the root folder. This pollutes the context window with thousands of irrelevant lines from upstream Anki, build caches, and third-party dependencies.
- **The Correct Pattern**: Consult the Key Symbol & Integration Contract Ledger in `.agents/skills/navigator/references/project-map.md`. The exact file path and line numbers for every major symbol (`ProceduralService`, `SourceQuestion`, `StepValidator`, `DeclarativeFamilyContract`, `MCQContainer`, `onEnterKey`, `_is_procedural_card`, `destroyActive`) are pre-indexed.

### 2.2 Canonical Documentation Reading Paths
Rather than reading through dozens of markdown files in `docs/`:
- **For Fast General Orientation**: Follow Path 1 (`docs/STUDYLAB_PRODUCT_CONTRACT.md` ➔ `docs/contracts/` ➔ `.agents/rules/architecture-boundaries.md` ➔ `docs/ARCHITECTURE_INVARIANTS.md` ➔ `docs/FINAL_LIVE_UI_FORENSIC_REPORT.md`).
- **For Rust Engine**: Follow Path 2 (`docs/SYSTEM_ARCHITECTURE.md` ➔ `docs/DATABASE_DATA_CONTRACT.md` ➔ `docs/LEARNING_MODEL.md` ➔ `docs/DIAGNOSTIC_AND_REMEDIATION.md`).
- **For Frontend TypeScript**: Follow Path 3 (`docs/contracts/` ➔ `docs/FRONTEND_PRODUCT_SPEC.md` ➔ `docs/REVIEWER_STATE_MACHINE.md` ➔ `docs/FRONTEND_BACKEND_CONTRACT.md`).
- **For Python / Qt Host Bridge**: Follow Path 4 (`docs/PRODUCT_BOUNDARIES.md` ➔ `docs/FRONTEND_BACKEND_CONTRACT.md` ➔ `.agents/rules/architecture-boundaries.md`).
- **For Content Authors**: Follow Path 5 (`StudyLab-Source-APKG-Contract(1).txt` ➔ `docs/APKG_CONTENT_CONTRACT.md` ➔ `tools/studylab_content_factory.py`).

### 2.3 Bypass Historical Context Noise
- Historical phase reports (`01_research_findings.md` through `08_release_decision.md` and `HANDOFF_REPORT.md`) record point-in-time explorations and historical bug reproductions.
- **Optimization Rule**: Never read historical reports to understand how a component works today. Go directly to Tier 1 executable source code and Tier 5 master contracts (`docs/contracts/`, `StudyLab-Source-APKG-Contract(1).txt`).

### 2.4 Targeted Line-Bounded File Slices
- Never dump an entire 1,500-line file into context when investigating a single function.
- Always provide `StartLine` and `EndLine` parameters when calling `view_file` (e.g. lines 120–165 of `rslib/src/notetype/render.rs` to inspect the card rendering hook).

---

## 3. Repository Context Traps & Artifact Exclusions

StudyLab generates and maintains heavy binary and cache artifacts that must never be ingested into agent context:

```text
┌──────────────────────────────────────────────────────────────────────────────────┐
│                        STUDYLAB CONTEXT EXCLUSION DIRECTORY                      │
├───────────────────────────┬──────────────────────────────────────────────────────┤
│ Heavy Artifact Category   │ Exact Path Patterns to Avoid / Exclude               │
├───────────────────────────┼──────────────────────────────────────────────────────┤
│ **Binary APKG Packages**  │ `dist/apkgs/`, `artifacts_qa/*.apkg`, `*.apkg`       │
│ **SQLite Databases**      │ `*.anki2`, `*.anki21`, `*.procedural`, `*.sqlite*`   │
│ **Build Outputs**         │ `target/`, `out/`, `dist/`, `node_modules/`, `bazel-*`│
│ **Python Caches**         │ `__pycache__/`, `.pytest_cache/`, `.ruff_cache/`     │
│ **Visual QA Screenshots** │ `artifacts_qa/final_release_audit/`, `*.png`          │
│ **Upstream Anki Code**    │ `rslib/src/` (outside touchpoints), `ts/lib/`        │
└───────────────────────────┴──────────────────────────────────────────────────────┘
```

### 3.1 Binary Decks & APKG Packages
- **Trap**: Attempting to view or grep `.apkg` files, which are binary ZIP archives containing SQLite databases and media.
- **Rule**: Never call `view_file` or `grep_search` on `.apkg` files. To verify APKG structure or contract adherence, run the verified Python validation harnesses:
  - For Canonical Source APKGs: `python artifacts_qa/validate_canonical_source_apkg.py <fixture.apkg>`
  - For Procedural Blueprint Universe APKGs: `python artifacts_qa/validate_canonical_apkg.py`

### 3.2 SQLite Database Files
- **Trap**: Attempting to read `collection.anki2` or `<col>.procedural` directly as text.
- **Rule**: Never inspect raw database files. Inspect schema definitions and migrations in `rslib/procedural/src/storage/{schema,migration}.rs`, or run verified Rust storage tests (`cargo test -p procedural storage::`).

### 3.3 Upstream Anki Host vs StudyLab Scope
- **Trap**: Searching the entire Anki codebase for concepts that are purely StudyLab-specific (e.g. grepping all of `rslib/` for `SkillState` or `StepValidator`).
- **Rule**: StudyLab core logic resides strictly in `rslib/procedural/`. Upstream Anki files only touch StudyLab at the 4 explicit Rust touchpoints (`rslib/src/collection/mod.rs`, `rslib/src/notetype/render.rs`, `rslib/src/scheduler/answering/mod.rs`, `rslib/src/import_export/package/apkg/import/mod.rs`) and 1 Python bridge file (`qt/aqt/reviewer.py`). Constrain searches to these locations.

### 3.4 Search Precision Directives
- When using `grep_search`:
  - Always set `SearchPath` to the specific subdirectory (e.g. `rslib/procedural/src/problems` instead of repo root).
  - Use `Includes` glob filters (e.g. `Includes: ["*.rs"]` or `Includes: ["*.ts"]`).
  - Set `MatchPerLine: true` with targeted identifiers.
- When using `find_by_name`:
  - Constrain `SearchDirectory` and specify `Extensions` (e.g. `Extensions: ["rs"]`).

---

## 4. Host Machine & Execution Resource Optimization

Running commands blindly can stall execution, trigger host OOM crashes, or waste minutes on unnecessary work. Follow the progressive test escalation ladder:

```text
┌────────────────────────────────────────────────────────────────────────┐
│                   PROGRESSIVE TEST ESCALATION LADDER                   │
├────────┬──────────────────────────────────────────┬───────────┬────────┤
│ Step   │ Command                                  │ Duration  │ Target │
├────────┼──────────────────────────────────────────┼───────────┼────────┤
│ Step 1 │ cargo check -p procedural                │ ~0.2s     │ Fast   │
│ Step 2 │ cargo test -p procedural --lib           │ ~0.09s    │ Unit   │
│ Step 3 │ cargo test -p procedural --test <target> │ ~0.08–2.7s│ Target │
│ Step 4 │ python tools/studylab_content_factory.py │ ~1.0s     │ Content│
│        │   --validate                             │           │        │
│ Step 5 │ python artifacts_qa/                     │ ~1.2s     │ Master │
│        │   challenger_2_master_runner.py          │           │ Release│
└────────┴──────────────────────────────────────────┴───────────┴────────┘
```

### 4.1 Compiler-First Validation (`cargo check`)
- Always run `cargo check -p procedural` before invoking `cargo test`.
- `cargo check` verifies syntax, borrow checker invariants, and type correctness across the procedural crate in ~0.2 seconds without running the LLVM code generator or linker.
- Fix all compiler and type errors at this stage before executing any tests.

### 4.2 Progressive Test Escalation
1. **Unit Tests First (`cargo test -p procedural --lib`)**:
   - Executes all 146 procedural library unit tests in ~0.09 seconds.
   - Covers CAS step validation, 5D units, skill tracking, difficulty calculations, and reviewer templates.
2. **Targeted Integration Tests**:
   - Run only the integration suite relevant to your change:
     - 175-Topic Factory: `cargo test -p procedural --test phase36c_all_175_topics_factory_tests` (0.08s)
     - Canonical Source Contract: `cargo test -p procedural --test canonical_source_contract_tests` (0.01s)
     - Desktop Validation Master: `cargo test -p procedural --test desktop_validation_master_suite` (2.75s)
     - Production Hardening: `cargo test -p procedural --test phase40_production_hardening_tests` (0.08s)
3. **Master Release Runner (`challenger_2_master_runner.py`)**:
   - Run `python artifacts_qa/challenger_2_master_runner.py` only during final verification gates (Tier 3/4). It tests all 4 dimensions in ~1.2 seconds.

### 4.3 Prohibited Costly Commands (What NEVER to Run)

| Prohibited Command | Failure Mode / Resource Cost | Correct Alternative |
|---|---|---|
| `cargo test -p procedural` (unconstrained) | Executes legacy phase tests and multi-minute longitudinal simulations (`phase30`, `phase32` taking >4 minutes). | `cargo test -p procedural --lib` or specific `--test <target>` |
| `cargo test --workspace` (directly) | Fails with missing FTL translation build-time module errors. | `just test-rust` |
| `python tools/studylab_content_factory.py --generate-apkgs` (during dev) | Generates dozens of APKG binary files to disk; unnecessary for logic validation. | `python tools/studylab_content_factory.py --validate` |
| `./ninja`, `./run` directly | Bypasses managed Anki environment configuration. | Use `just` recipes (`just test-py`, `just lint`, `just check`) |
| Unconstrained background processes | Leaks background daemons or unmanaged test sockets. | Pure CLI execution via `run_command` with synchronous timeouts |

---

## 5. Context Budgeting & Communication Hygiene

### 5.1 The Smallest Sufficient Context Principle
- In multi-agent pipelines, every token passed to a subagent incurs processing cost and risks attention dilution.
- Supply subagents with exact file paths, line ranges, and concise instructions rather than dumping entire files or historical transcripts.
- When delegating to `implementer`, declare the exact write-set (files to modify) up front.
- When delegating to `reviewer-verifier`, provide the exact test commands and modified diff rather than broad instructions.

### 5.2 Standardized Structured Handoffs
- Always format agent-to-agent and turn-to-turn handoffs using the standardized templates from `.agents/skills/engineering-workflow/SKILL.md`:
  - `RECONNAISSANCE HANDOFF REPORT`
  - `IMPLEMENTATION HANDOFF REPORT`
  - `VERIFICATION HANDOFF REPORT`
  - `CHALLENGER AUDIT HANDOFF REPORT`
- Keep reports dense, factual, and backed by verifiable terminal outputs and line numbers.
