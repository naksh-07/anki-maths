# StudyLab Architecture Boundaries & Frozen Invariants

**Authority:** Level 1 Subsystem Architectural Invariant Rule  
**Target Repository:** `naksh-07/anki-maths` (StudyLab Procedural Intelligence Subsystem)  
**Status:** CANONICAL FROZEN DIRECTIVE  
**Source of Truth:** Canonical Documentation (`docs/`), Master Contracts (`docs/contracts/`, `StudyLab-Source-APKG-Contract(1).txt`), and Executable Source Code (`rslib/`, `ts/`, `qt/`, `pylib/`)

---

## 1. Product Identity & Core Architectural Directive

```text
┌──────────────────────────────────────────────────────────────────────────────────┐
│                            CORE BOUNDARY DIRECTIVE                               │
├──────────────────────────────────────────────────────────────────────────────────┤
│ "StudyLab is an adaptive procedural problem-solving engine hosted inside Anki.   │
│  It is NOT a flashcard app, math add-on, or interactive quiz deck."             │
│                                                                                  │
│ • Anki is the familiar, distraction-free spaced repetition host shell.           │
│ • StudyLab provides the procedural intelligence layer inside it: dynamic problem │
│   generation, physical 5D dimensional analysis, CAS step validation, cognitive    │
│   diagnostics, and remediation.                                                  │
│ • Non-procedural flashcards (Basic, Cloze, Image Occlusion) must remain 100%    │
│   untouched and operate with native speed and zero regressions.                  │
└──────────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Anki ↔ StudyLab Ownership Boundaries

The boundary between Anki (host shell) and StudyLab (procedural engine) is strict, non-overlapping, and enforced at every layer:

| Functional Domain | Host SRS (Anki) | Procedural Subsystem (StudyLab) | Integration Boundary / Protocol |
|---|---|---|---|
| **Declarative Flashcards** | **Sole Owner** (`Basic`, `Cloze`, etc.) | *Zero involvement* | Standard Mustache rendering in `rslib/src/notetype/render.rs` |
| **Windowing & Application UI** | **Sole Owner** (PyQt6 / QtWebEngine) | Guest interactive webview content | Dual webview container: `mw.web` (main) + `mw.bottomWeb` (bottom bar) |
| **Spaced Repetition Math** | **Sole Owner** (FSRS v4/v5 / SM-2 intervals) | Derives ease signals (1..4) | `derive_fsrs_rating()` in `rslib/procedural/src/scheduling/rating_policy.rs` |
| **Collection Storage** | **Sole Owner** (`collection.anki2` / `anki21`) | *Zero schema pollution* | Isolated SQLite file `<col_path>.procedural` |
| **Sync & Media Server** | **Sole Owner** (`mediasrv`, `syncserver`) | *Zero involvement* | Deck media files served via standard media server |
| **Procedural Generation** | *Zero involvement* | **Sole Owner** (`rslib/procedural/`) | Dynamic parameter sampling across 175 STEM curriculum topics |
| **Stepwise CAS Evaluation** | *Zero involvement* | **Sole Owner** (`StepValidator`) | Semantic root solving & commutative tree comparison in `rslib/procedural/` |
| **Dimensional Analysis** | *Zero involvement* | **Sole Owner** (`5D Vector Engine`) | Physical dimensions $[M][L][T][N][K]$ and 40+ unit registry in `rslib/procedural/src/units/` |
| **Cognitive Mastery Model** | *Zero involvement* | **Sole Owner** (`SkillState`) | Exponential Moving Average ($\alpha=0.20$), 6-gate progression policy in `rslib/procedural/src/skills/` |
| **Mistake Classification** | *Zero involvement* | **Sole Owner** (4-Category Strip) | Reflection gating: `[1 Silly Slip]`, `[2 Pattern Missed]`, `[3 Concept Gap]`, `[4 Prereq Unknown]` |
| **JIT Remediation Queue** | *Zero involvement* | **Sole Owner** (`RemediationQueue`) | 9-tier remediation priority engine with circuit breakers ($\ge 5$ recurrences) |
| **Review Telemetry Boundary** | Card Custom Data (`custom_data`) | Ephemeral JSON payload | **100-Byte Limit Rule:** Telemetry is written to `<col>.procedural`, then stripped from `custom_data` |

---

## 3. Dual Content Architecture: Source APKG vs Procedural Blueprints

StudyLab supports two distinct, compatible content architectures. The procedural engine must respect the separation of these two pathways:

```text
StudyLab Content Architecture
│
├── PATH 1: CANONICAL SOURCE-FIRST PATH (StudyLab Source)
│   └── Curated static APKGs containing immutable questions (MCQs, Numerical items, PYQs)
│       Governed Authoritatively by: StudyLab-Source-APKG-Contract(1).txt (FROZEN Level 1)
│       Note Model: "StudyLab Source"
│       Pipeline: Packaging -> Ingestion -> Reconcile `practice_items` -> Direct Open Canvas Render
│       Invariant: Dynamic parameter generators are completely bypassed; source items are immutable.
│
└── PATH 2: PROCEDURAL BLUEPRINT PATH (StudyLab Procedural Anchor)
    └── Declarative problem family blueprints generating dynamic mathematical variants
        Governed by: DeclarativeFamilyContract & ProceduralPayload Schemas
        Note Model: "StudyLab Procedural Anchor"
        Pipeline: Note Anchor -> 3-Tier Resolution (`inline_contract` -> `content_ref` -> `proc_schema`)
                  -> AST Problem Generation -> Dynamic Variant Render
        Invariant: Operates in parallel; NEVER modifies, redefines, or overrides the Source APKG contract.
```

### Path 1 Contract Invariants (Frozen Source APKG)
- **Authority:** `StudyLab-Source-APKG-Contract(1).txt` is the frozen Level 1 source of truth.
- **Model Ingestion:** Parsed directly into `SourceQuestion` (`rslib/procedural/src/anchor/source.rs`).
- **Deterministic Reconciliation:** Automatically reconciled into `practice_items` via `Collection::reconcile_source_questions()` upon APKG import (`rslib/src/import_export/package/apkg/import/mod.rs:69`).
- **Learner State Firewall:** Curated questions are immutable; all practice attempts, skill state transitions, and error events are persisted strictly into `<collection>.procedural`.

### Path 2 Contract Invariants (Procedural Blueprints)
- **Blueprint Hierarchy:** Resolves blueprints via 3 tiers:
  1. `inline_contract`: Self-contained JSON payload inside the note anchor (preferred for portable decks).
  2. `content_ref`: Normalized identifier resolved against local database schemas.
  3. `proc_schema`: Fallback declarative schema in local registry.
- **Zero-Rust Content Authoring:** Standard curriculum topics use the declarative archetype factory (`tools/studylab_content_factory.py`). Custom Rust solvers are strictly reserved for novel computational engines.

---

## 4. Polyglot Integration Boundaries & Touchpoints

StudyLab integrates across three languages: **Rust** (core engine & storage), **TypeScript/Svelte** (reviewer webview), and **Python/PyQt6** (host GUI & bridge).

```text
┌──────────────────────────────────────────────────────────────────────────────────┐
│                             POLYGLOT DATA & CONTROL FLOW                        │
└──────────────────────────────────────────────────────────────────────────────────┘

 [ Canonical APKG ] ──► [ Collection::import_apkg ] ──► [ col.reconcile_source_questions() ]
                                 │
                                 ▼
                     [ Anki SQLite: collection.anki2 ]
                                 │
     ┌───────────────────────────┴───────────────────────────┐
     ▼ (Standard Note: Basic/Cloze)                          ▼ (StudyLab Note: Source / Anchor)
[ Mustache Renderer ]                                 [ rslib/src/notetype/render.rs ]
     │                                                       │
     │                                     ┌─────────────────┴─────────────────┐
     │                                     ▼                                   ▼
     │                           render_source_anchor()              render_procedural_anchor()
     │                                     │                                   │
     │                                     └─────────────────┬─────────────────┘
     │                                                       ▼
     │                                       [ rslib/procedural/ Template Engine ]
     │                                                       │
     ▼                                                       ▼
[ Qt Reviewer Window ] ◄──────── QWebChannel IPC ────────► [ QtWebEngine Viewport: mw.web ]
(qt/aqt/reviewer.py)                                       (ts/reviewer/procedural.ts)
     │                                                       │
     │  Suppresses #ansbut & ease buttons                    │  State Machine:
     │  Traps Space/Enter during solving                     │  SOLVING -> MISTAKE CLASSIFICATION
     │  Dispatches pycmd("procedural_*")                     │  Zero text input on MCQs
     │  Invokes destroyActive() teardown                     │  Emits bridge commands
     │                                                       │
     ▼                                                       ▼
[ Anki Scheduler Hook: rslib/src/scheduler/answering/mod.rs ]
     ├── Ingests studylab telemetry payload
     ├── Writes attempt to <col_path>.procedural (ACID single-transaction)
     └── Strips studylab payload from custom_data before persisting to collection.anki2
```

### The 4 Explicit Rust Backend Touchpoints
1. **Storage Initialization (`rslib/src/collection/mod.rs:141, 173–183`):**
   `Collection::procedural_service()` lazily initializes `ProceduralService::open(col_path.with_extension("procedural"))`. The procedural database file is strictly decoupled from `collection.anki2`.
2. **Card Rendering Interception Hook (`rslib/src/notetype/render.rs:122–131`):**
   In `render_card()`, the engine inspects the note type name prefix before invoking standard Mustache rendering:
   - Notes starting with `"StudyLab Procedural Anchor"` route to `render_procedural_anchor()`.
   - Notes starting with `"StudyLab Source"` route to `render_source_anchor()`.
   - All standard Anki note types bypass procedural code paths with zero overhead.
3. **Answering & Telemetry Hook (`rslib/src/scheduler/answering/mod.rs:353–505`):**
   When answering cards, the backend inspects `studylab` metadata in the answer parameters, invokes `service.record_practice_attempt_with_variant()`, updates Bayesian/EMA skill states and remediation queues in `collection.procedural`, and **rewrites `card.custom_data` to strip the `studylab` payload** before saving to Anki's SQLite store, satisfying Anki's 100-byte custom data limit.
4. **APKG Import Reconciliation Hook (`rslib/src/import_export/package/apkg/import/mod.rs:69`):**
   Upon importing `.apkg` decks via `Collection::import_apkg`, `col.reconcile_source_questions()` automatically synchronizes imported static Source questions into `practice_items` in `<col>.procedural`.

### Forensic Grounding: Two-P0 Desktop Reconciliation
Physical desktop audits (`docs/FINAL_LIVE_UI_FORENSIC_REPORT.md`, `artifacts_qa/final_p0_reconciliation/p0_reconciliation_evidence.json`, and `tools/live_p0_forensic_verification.py`) established empirical boundaries that every agent must preserve:
- **P0-A Spacebar/Enter Anti-Bypass Trapping:** In standard Anki, pressing Space or Enter reveals the answer and rates the card. On procedural cards, `onEnterKey()` intercepts these keystrokes and routes to `handleNativeShowAnswer()`, blocking learners from bypassing active problem solving or skipping mistake reflection.
- **P0-B Interaction Surface Deduplication:** Native Anki bottom rating buttons (`#ansbut`, `#ease1..4`) must never coexist with StudyLab's solving canvas. The host reviewer bridge (`qt/aqt/reviewer.py`) actively suppresses them via `_is_procedural_card()`.
- **Modality Purity & Single Interaction Surface:** The in-card solving canvas is the sole interaction surface. Superfluous "Next Problem" buttons are eliminated in favor of automatic advance on correct submission and reflection gating on incorrect.

### Python / PyQt6 Host Bridge Touchpoints (`qt/aqt/reviewer.py`)
- **Card Type Detection (`_is_procedural_card`):** Identifies cards whose note type starts with `"StudyLab Procedural Anchor"` or `"StudyLab Source"`.
- **Button Suppression:** On procedural cards, standard Anki bottom bar buttons (`#ansbut` "Show Answer", ease buttons `ease1..4`) are suppressed via `_showAnswerButton()` and `_showEaseButtons()`.
- **Keyboard Shortcut Trap (`onEnterKey`):** Intercepts Enter/Space during procedural review and delegates to `globalThis.anki.procedural.handleNativeShowAnswer()`, preventing learners from accidentally rating or bypassing cards during active solving or mistake reflection.
- **IPC Command Dispatcher (`_handle_procedural_command`):** Listens on `_linkHandler` for `procedural_*` commands:
  - `procedural_attempt:<json>`: Records attempt submission and synchronizes review state.
  - `procedural_mistake:<json>`: Records error classification choice.
  - `procedural_answer:<1..4>`: Submits derived FSRS ease rating to Anki.
  - `procedural_hint:<json>`: Tracks hint usage telemetry.
  - `procedural_validate_steps:<json>`: Routes stepwise CAS evaluation requests.
  - `procedural_try_similar:<json>`: Regenerates problem variant for immediate retry.
  - `procedural_practice_prerequisite:<json>`: Bridges to prerequisite remediation drill.
- **Teardown Lifecycle Hook (`destroyActive()`):** Evaluated inside `Reviewer._showQuestion()` (`line 416`) and `Reviewer.cleanup()` (`line 208`) to dispose all window event listeners, keyboard handlers, and timers before mounting another card.

### Frontend TypeScript Reviewer Touchpoints (`ts/reviewer/`)
- **Canonical State Machine (`docs/contracts/StudyLab-Runtime-Interaction-Contract.md`):**
  ```text
  SOLVING
    ├── [correct]   ──► FEEDBACK / SOLUTION ──► Automatic Advance
    └── [incorrect] ──► MISTAKE CLASSIFICATION
                             ├── [1 Silly Slip]
                             ├── [2 Pattern Missed]
                             ├── [3 Concept Gap]    ──► Record Classification ──► Advance
                             └── [4 Prereq Unknown]
  ```
- **Forbidden UI Elements (`docs/contracts/StudyLab-Runtime-UI-Contract.md`):**
  - **NO "Next Card" or "Next Problem" button:** Advances automatically on correct submission or immediately upon selecting a mistake classification.
  - **NO visible native Anki ease buttons:** Suppressed by host bridge.
  - **NO duplicate mistake panels:** Rendered strictly once in the bottom interaction area.
- **Semantic Modality Purity:**
  - `mcq`: Handled by `MCQContainer` (`ts/reviewer/components/mcq_container.ts`). Renders native option buttons/cards with keyboard hotkeys (`1`–`4`, `A`–`D`). **Zero text-input fallback.**
  - `numerical`: Handled by `NumericalContainer` (`ts/reviewer/components/numerical_container.ts`). Normalizes scientific notation, fractions, equations, and validates units via 5D vector registry.
  - `stepwise`: Handled by `StepwiseContainer` (`ts/reviewer/components/stepwise_container.ts`). Multi-step CAS graph evaluation.
- **Lifecycle Teardown:** `globalThis.anki.procedural.destroyActive()` disposes all registered listeners, cancels pending timers, and disconnects observers.

---

## 5. Summary of the 16 Frozen Architecture Invariants

Refer to `docs/ARCHITECTURE_INVARIANTS.md` for the complete specification. Every engineer and agent must adhere to these 16 frozen rules:

1. **Not a Flashcard System:** Adaptive procedural engine, not a flashcard reviewer or quiz deck.
2. **Do Not Recreate Anki / FSRS:** Anki owns windowing, storage, sync, and temporal intervals; StudyLab owns procedural intelligence.
3. **Problem-Solving Workspace UI:** Reviewer is an interactive solving workspace, not a front/back card flip.
4. **Modality-Matched Semantics:** Input modality matches construct (MCQ = radio hotkeys; Numerical = 5D units; Stepwise = CAS derivation).
5. **Semantic Input Validation:** Numeric/symbolic inputs validated semantically, not via brittle string comparisons.
6. **Canonical Stepwise Validation:** Linear roots and commutative equivalence evaluated via AST graph comparators.
7. **Unified SkillState Progression:** Single unified learner model regardless of card source (`inline_contract`, `content_ref`, or PYQ).
8. **Orthogonal Diagnostic Evidence:** Distinguishes execution slips from conceptual misunderstandings (`is_execution_error()` vs `is_conceptual_error()`).
9. **Blueprint vs History Ownership:** APKGs own static definitions; runtime exclusively owns historical attempt logs in `<col>.procedural`.
10. **Zero-Rust Declarative Authoring:** New curriculum topics authored declaratively via `tools/studylab_content_factory.py`; no unnecessary Rust compilation.
11. **No Internal Leakage to Learner:** Internal schema IDs, debug tags, and raw codes are strictly stripped from student view.
12. **Single Canonical Evaluation Source:** Rust backend contract is authoritative; TypeScript mirrors for zero-latency client feedback.
13. **Standard Anki Zero Non-Regression:** `Basic` and `Cloze` notes bypass procedural hooks with 100% fidelity.
14. **Diagnostic Unified Learner Model:** Mock test outcomes batch-synchronize into `SkillState` in `collection.procedural`.
15. **Tier 1 Inline Contract Precedence:** Declarative blueprints packaged as self-contained `inline_contract` in card anchors for deck portability.
16. **Docs as Supreme Source of Truth:** `StudyLab-Source-APKG-Contract(1).txt` (Level 1) and canonical docs in `docs/` govern architecture. Historical reports (`01_`–`08_`) are archaeological context only.
