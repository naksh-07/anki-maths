---
name: navigator
description: >-
  Authoritative navigation and architectural orientation skill for the StudyLab procedural
  learning subsystem inside Anki (naksh-07/anki-maths). Provides canonical documentation reading
  paths, subsystem mental models, polyglot boundary maps, and real, verified verification commands.
---

# StudyLab Navigator Skill

The `navigator` skill provides agents and developers with immediate, accurate orientation across the polyglot StudyLab codebase inside Anki (`naksh-07/anki-maths`).

Use this skill when you need to:
- Understand the architecture, responsibilities, or data flow of StudyLab.
- Locate the correct files, symbols, or tests across Rust, TypeScript, Python/Qt, or documentation.
- Determine which canonical specification governs a feature or resolve apparent contradictions.
- Execute verified build, test, and validation commands.

---

## 1. Quick Mental Model (The 30-Second Summary)

```text
┌──────────────────────────────────────────────────────────────────────────────────┐
│                            STUDYLAB ARCHITECTURE ESSENCE                         │
├──────────────────────────────────────────────────────────────────────────────────┤
│ 1. IDENTITY: StudyLab is an adaptive procedural problem-solving engine hosted    │
│    inside Anki. It is NOT a flashcard app or quiz deck.                          │
│ 2. SEPARATION: Anki owns spaced repetition (FSRS), collection DB                 │
│    (collection.anki2), and standard cards (Basic/Cloze). StudyLab owns dynamic   │
│    generation, CAS validation, 5D units, diagnostics, and collection.procedural. │
│ 3. DUAL CONTENT PATHS:                                                           │
│    - Source APKG: Curated static questions (PYQs) governed by frozen Level 1     │
│      contract StudyLab-Source-APKG-Contract(1).txt; generators bypassed.         │
│    - Procedural Blueprints: Declarative family blueprints for 175 topics         │
│      generating dynamic variants.                                                │
│ 4. TOUCHPOINTS: Exactly 4 Rust touchpoints (Storage, Render hook, Answering hook, │
│    APKG reconciliation) and 1 Python Qt bridge (command router, button trap).   │
│ 5. NO NEXT CARD BUTTON: Interaction contract mandates automatic advancement on   │
│    correct answers, and reflection gating with [1..4] mistake classification     │
│    buttons on incorrect answers.                                                 │
└──────────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Canonical Documentation Reading Paths

To prevent cognitive overload, follow the specific reading path tailored to your task (derived from `docs/DOCUMENTATION_MAP.md`):

```text
                             ┌───────────────────────────┐
                             │     READER ENTERS REPO    │
                             └─────────────┬─────────────┘
                                           │
    ┌──────────────────┬───────────────────┼───────────────────┬──────────────────┐
    ▼                  ▼                   ▼                   ▼                  ▼
┌───────────┐      ┌───────────┐       ┌───────────┐       ┌───────────┐      ┌───────────┐
│ AI AGENT  │      │ RUST CORE │       │ FRONTEND  │       │ PYTHON/QT │      │ CONTENT   │
│ FAST-START│      │ DEVELOPER │       │ DEVELOPER │       │ DEVELOPER │      │ AUTHOR    │
└─────┬─────┘      └─────┬─────┘       └─────┬─────┘       └─────┬─────┘      └─────┬─────┘
      │                  │                   │                   │                  │
      ▼                  ▼                   ▼                   ▼                  ▼
 [Path 1]           [Path 2]            [Path 3]            [Path 4]           [Path 5]
```

### Path 1: Clean-Context AI Agent Fast-Start (Orientation in < 3 Minutes)
*Goal: Acquire an accurate mental model without hallucinations or being misled by historical reports.*
1. **`docs/STUDYLAB_PRODUCT_CONTRACT.md`** (Sections 1–4): Core identity, two-system model, 5-stage loop.
2. **`docs/contracts/StudyLab-Runtime-Interaction-Contract.md`** & **`docs/contracts/StudyLab-Runtime-UI-Contract.md`**: Canonical interaction and UI specifications.
3. **`.agents/rules/architecture-boundaries.md`** & **`docs/ARCHITECTURE_INVARIANTS.md`**: The 16 frozen non-negotiables.
4. **`docs/FINAL_LIVE_UI_FORENSIC_REPORT.md`**: Physical desktop verification and Two-P0 reconciliation findings.
5. **`docs/SYSTEM_ARCHITECTURE.md`**: End-to-end execution pipeline trace.
6. **`.agents/skills/navigator/references/project-map.md`**: Exact polyglot directory map and key symbols.

### Path 2: Rust Core Engine Developer (`rslib/procedural/`)
*Goal: Work on problem generators, solvers, dimensional units, storage, or scheduling.*
1. **`docs/SYSTEM_ARCHITECTURE.md`**: Crate architecture of `rslib/procedural/`.
2. **`docs/DATABASE_DATA_CONTRACT.md`** & **`docs/DATA_AND_PERSISTENCE.md`**: 16 SQLite tables, migrations v1–v5, atomic writes.
3. **`docs/LEARNING_MODEL.md`**: EMA mastery tracking ($\alpha=0.20$), 6-gate progression policy.
4. **`docs/DIAGNOSTIC_AND_REMEDIATION.md`**: `StepValidator`, `RemediationQueue`, circuit breakers.

### Path 3: Frontend TypeScript & UX Engineer (`ts/reviewer/`)
*Goal: Modify reviewer UI, answer modalities, keyboard navigation, or layout.*
1. **`docs/contracts/StudyLab-Runtime-Interaction-Contract.md`** & **`docs/contracts/StudyLab-Runtime-UI-Contract.md`**: Master state machine and UI rules.
2. **`docs/FRONTEND_PRODUCT_SPEC.md`**: 9 learning object modalities and semantic purity.
3. **`docs/REVIEWER_STATE_MACHINE.md`**: State transitions, Space/Enter trapping, teardown.
4. **`docs/FRONTEND_BACKEND_CONTRACT.md`**: `pycmd` bridge command protocols and telemetry envelopes.

### Path 4: Python / Qt Host Bridge Engineer (`qt/aqt/reviewer.py`)
*Goal: Maintain webview bridge, shortcut trapping, button suppression, or desktop hooks.*
1. **`docs/PRODUCT_BOUNDARIES.md`**: Host-guest decoupling principles.
2. **`docs/FRONTEND_BACKEND_CONTRACT.md`**: Python link handlers and `_handle_procedural_command`.
3. **`.agents/rules/architecture-boundaries.md`**: Button suppression, spacebar bypass prevention, and `destroyActive()` teardown.

### Path 5: Curriculum Designer & APKG Content Author
*Goal: Author new academic domains, declarative family blueprints, or APKG decks.*
1. **`StudyLab-Source-APKG-Contract(1).txt`**: Level 1 frozen contract for canonical static decks.
2. **`docs/APKG_CONTENT_CONTRACT.md`** & **`docs/CONTENT_AND_AUTHORING.md`**: Declarative archetypes, 15 domains, 24 derivations.
3. **`tools/studylab_content_factory.py`** & **`generate_canonical_source_apkg.py`**: Content generation tools.

---

## 3. Polyglot Architecture & Subsystem Boundaries

StudyLab spans four primary subsystem areas:

1. **Rust Core (`rslib/procedural/` & `rslib/src/`):**
   - Implements the procedural engine, AST solvers, 5D unit registry, SQLite store (`collection.procedural`), and Anki core hooks.
   - Four touchpoints: Storage initialization (`rslib/src/collection/mod.rs`), Card rendering hook (`rslib/src/notetype/render.rs`), Telemetry stripping hook (`rslib/src/scheduler/answering/mod.rs`), and APKG import reconciliation hook (`rslib/src/import_export/package/apkg/import/mod.rs`).
2. **Frontend Webview (`ts/reviewer/`):**
   - Implements the interactive solving canvas inside QtWebEngine (`mw.web`).
   - Modality components: `MCQContainer` (zero text fallback), `NumericalContainer` (5D vector parsing), `StepwiseContainer` (CAS graph).
   - Driven by `ProceduralReviewer` state machine (`ts/reviewer/procedural.ts`).
3. **Desktop Host Bridge (`qt/aqt/reviewer.py`):**
   - Suppresses native Anki ease buttons and `#ansbut` on procedural cards.
   - Routes `procedural_*` IPC bridge commands to the Rust backend and Anki collection.
   - Evaluates `globalThis.anki.procedural.destroyActive()` to prevent event leaks.
4. **Content & Packaging Tools (`tools/`, root scripts, `artifacts_qa/`):**
   - `tools/studylab_content_factory.py`: Universal Phase 36C declarative content factory across 175 STEM topics (`--validate`, `--generate-apkgs`).
   - `artifacts_qa/validate_canonical_apkg.py`: Validates generated 175-topic Universe APKGs.
   - `generate_canonical_source_apkg.py`: Generates deterministic canonical APKG fixtures.
   - `artifacts_qa/validate_canonical_source_apkg.py`: Validates APKGs against the frozen contract.
   - `artifacts_qa/challenger_2_master_runner.py`: Master 4-dimension adversarial audit suite.

---

## 4. Source-of-Truth Rules for Navigating

When you encounter any apparent conflict:
1. **Executable Code & Passing Tests Win:** If a document describes behavior not present in code, code is ground truth (Tier 1/2).
2. **Master Contracts Override General Docs:** `docs/contracts/` and `StudyLab-Source-APKG-Contract(1).txt` override older specs (Tier 5).
3. **Historical Reports Are Not Truth:** Reports (`01_` through `08_`, `HANDOFF_REPORT.md`) are historical context only.
4. **Never Invent Architecture:** Every path, command, and symbol you reference must exist in this repository.

---

## 5. Detailed Project Navigation Map

For the complete file-by-file directory tree, key symbol ledger, and verified command list, consult:
**[`.agents/skills/navigator/references/project-map.md`](references/project-map.md)**
