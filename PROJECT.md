# Project: StudyLab Final Reconciliation & Product Identity Freeze

## Architecture
- **Host**: Native Anki desktop application (Python 3.13 backend + PyQt6 / QtWebEngine UI).
- **Core Engine**: In-tree Rust crate (`rslib/procedural/`, package `procedural`) providing high-performance declarative problem generation, 5D physical unit algebra, CAS step verification, cognitive mastery models, and isolated SQLite storage (`collection.procedural`).
- **Bridge**: Native Python desktop reviewer bridge (`qt/aqt/reviewer.py`) and Rust interception hook (`rslib/src/notetype/render.rs`) managing webview injection, `Show Answer` suppression, and IPC command handling.
- **Web Frontend**: Svelte/TypeScript reviewer subsystem (`ts/reviewer/procedural.ts`, `ts/reviewer/components/`) running inside Anki's review webview, rendering modality-matched interaction surfaces (`MCQContainer`, `NumericalContainer`, `StepwiseContainer`, `MistakeFooter`).

## 175 Curriculum Topics vs 177 Procedural Notes
- **175 Curriculum Topics**: The canonical academic target universe encompasses exactly 175 benchmark topics across four domains:
  - **Mathematics**: 59 topics
  - **Logical Reasoning**: 30 topics
  - **Physics**: 40 topics
  - **Chemistry**: 46 topics
- **177 Generated Procedural Notes**: The canonical release package `StudyLab_Full_Universe_175.apkg` contains 177 note records because two Mathematics topics (`statistics_variance_std_dev` and `data_interpretation_basics`) package specialized sub-modality anchor notes.

## Feature Inventory
| # | Category | Feature | Description | Milestone | Source |
|---|---|---|---|---|---|
| 1 | Product Identity | Hosted Procedural Engine | Procedural problem-solving and diagnostics hosted inside Anki runtime | M1 | `STUDYLAB_PRODUCT_CONTRACT.md § 1` |
| 2 | Core Learning Loop | 5-Stage Linear Learner Loop | Problem → 1 Interaction → Minimal Feedback → Diagnosis Only When Useful → 1 Next Action | M1 | `STUDYLAB_PRODUCT_CONTRACT.md § 2` |
| 3 | Core Learning Loop | Engine Attempt Ingestion | Silent 6th stage: background extraction of evidence and skill EMA updates | M1 | `DATABASE_DATA_CONTRACT.md § 4` |
| 4 | Diagnostic Hierarchy | 8-Level Diagnostic Hierarchy | Subject → Chapter → Topic → Skill → Family → Attempt → Error → Remediation | M4 | `STUDYLAB_PRODUCT_CONTRACT.md § 3` |
| 5 | Diagnostic Hierarchy | Speed-Accuracy Quadrants | 4-quadrant latency analysis ($Q_1$ Fast/Accurate .. $Q_4$ Slow/Inaccurate) | M4 | `FRONTEND_UI_STATE_SPEC.md § 4` |
| 6 | Modality Invariant | Semantic Modality Enforcement | Zero text-box fallback: MCQs and discrete drills never render a text input | M2 | `FRONTEND_PRODUCT_SPEC.md § 1.2` |
| 7 | Learning Object | `problem` (Quantitative) | Multi-step quantitative calculation with dimensional unit parsing | M2 | `FRONTEND_PRODUCT_SPEC.md § 2.1` |
| 8 | Learning Object | `quick` (Fluency) | Rapid single-step arithmetic fluency without scaffolding tabs | M2 | `FRONTEND_PRODUCT_SPEC.md § 2.2` |
| 9 | Learning Object | `mcq` (Discrete Choice) | 4-choice radio group with keyboard navigation (1-4, A-D, Arrows) | M2 | `FRONTEND_PRODUCT_SPEC.md § 2.3` |
| 10 | Learning Object | `stepwise` (CAS Derivation) | Cognitive Tutor Inner Loop: Step-by-step CAS intermediate validation | M2 | `FRONTEND_PRODUCT_SPEC.md § 2.4` |
| 11 | Learning Object | `concept_check` | Targeted distractor diagnostics testing core principles without math overhead | M2 | `FRONTEND_PRODUCT_SPEC.md § 2.5` |
| 12 | Learning Object | `strategy_drill` | Method selection & optimality training (e.g. Energy vs Kinematics) | M2 | `FRONTEND_PRODUCT_SPEC.md § 2.6` |
| 13 | Learning Object | `worked_example` | Stepwise canonical trace walkthrough with "Try Similar Problem" trigger | M2 | `FRONTEND_PRODUCT_SPEC.md § 2.7` |
| 14 | Learning Object | `declarative_recall` | Bridge to standard declarative spaced repetition for atomic definitions | M2 | `FRONTEND_PRODUCT_SPEC.md § 2.8` |
| 15 | Learning Object | `prerequisite_review` | Directed DAG navigation to remediate missing foundational knowledge | M2 | `FRONTEND_PRODUCT_SPEC.md § 2.9` |
| 16 | Reviewer UI States | 14-State Machine Lifecycle | Strict UI state machine (`loading`, `ready`, `solving`, `feedback`, `next`, etc.) | M3 | `FRONTEND_UI_STATE_SPEC.md § 2` |
| 17 | Metacognitive Trap | Anti-Bypass Mistake Gate | 4-choice mistake classification (`1 Silly`..`4 Prereq`) trapping Space/Enter | M3 | `FRONTEND_UI_STATE_SPEC.md § 3.5` |
| 18 | Button Contract | Canonical Master Button Matrix | Single-interaction-surface rule across 23 controls with mutual exclusions | M3 | `FRONTEND_BUTTON_CONTRACT.md § 2` |
| 19 | Anki Decoupling | Host-Guest Decoupling | Native Anki ease buttons suppressed; 100-byte `cards.data` column protected | M3 | `STUDYLAB_PRODUCT_CONTRACT.md § 5` |
| 20 | Visual Design | Visual Product Contract | Problem statement is visual hero; strict token inheritance; no rainbow badges | M3 | `FRONTEND_VISUAL_DESIGN_SPEC.md § 1-4` |
| 21 | APKG Architecture | Declarative Blueprint Packaging | Single full-universe package (`StudyLab_Full_Universe_175.apkg`) with payloads | M2 | `APKG_CONTENT_CONTRACT.md § 1-4` |
| 22 | Cross-Layer Pipeline | 4-Tier Cross-Layer Mapping | Deterministic trace: APKG Note → Rust Engine → SQLite → Python → TypeScript | M1 | `APKG_FRONTEND_CONTRACT.md § 1-4` |
| 23 | Database Schema | Dedicated SQLite Store | 16 tables in `collection.procedural` with WAL mode and v1-v5 migrations | M1 | `DATABASE_DATA_CONTRACT.md § 1-4` |
| 24 | Quality Assurance | Screen-by-Screen Acceptance | Testable acceptance criteria for 12 screens under "Perfect Window" standards | M5 | `FRONTEND_ACCEPTANCE_MATRIX.md § 1-3` |
| 25 | Forensic Gap Map | Screenshot-Grounded Gap Map | Forensic comparison of live desktop state against canonical contracts | M6 | `FRONTEND_CURRENT_STATE_GAP_MAP.md § 1-3` |

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| M1 | Core In-Tree Architecture & Physical Separation | `rslib/procedural/`, `collection.procedural`, schema v1-v5 | None | COMPLETE |
| M2 | Modality Contracts & Universal Content Factory | 175 topics / 177 notes across 4 domains, 9 learning objects | M1 | COMPLETE |
| M3 | Native Reviewer Integration & UI State Machine | `qt/aqt/reviewer.py`, `ts/reviewer/procedural.ts`, 23 buttons, mistake footer | M2 | COMPLETE |
| M4 | Multi-Domain Diagnostic Engine & Speed Quadrants | 4-tier report, 4-quadrant latency model, domain evidence sync | M2 | COMPLETE |
| M5 | Quality Assurance, Security & Performance | Unit & integration test suites, XSS escaping, SQL parameterization | M1-M4 | COMPLETE |
| M6 | Live QtWebEngine Desktop Verification | Remote CDP verification (Port 9222), 8 live phases, SHA-256 evidence | M3-M5 | COMPLETE |
| M7 | Release Gating & Product Identity Freeze | 10 canonical specifications in `docs/`, 15-point release decision | M1-M6 | COMPLETE |

## Interface Contracts

### Webview JS ↔ Python Reviewer Bridge (`qt/aqt/reviewer.py`)
Dispatched via `pycmd(command)`:
- `procedural_attempt:<json>`: Dispatches user answer, score, latency, and interaction telemetry.
- `procedural_mistake:<json>`: Logs learner mistake classification (`silly_mistake`, `pattern_not_recognized`, `formula_or_concept_misapplied`, `concept_not_known`).
- `procedural_hint:<json>`: Tracks hint tier progression (Principle → Operation → Intermediate).
- `procedural_validate_steps:<json>`: Evaluates intermediate stepwise equations via Rust CAS.
- `procedural_try_similar:<json>`: Triggers generation of an isomorphic problem instance with fresh parameters.
- `procedural_practice_prerequisite:<json>`: Navigates learner to foundational prerequisite practice.
- `procedural_declarative_recall:<json>`: Bridges to standard Anki declarative card for definition/formula recall.
- `procedural_answer:<ease>`: Directly invokes Anki's native `_answerCard(ease)` (`1=Again`, `2=Hard`, `3=Good`, `4=Easy`), advancing the card in the FSRS scheduler.

### Reviewer ↔ Rust Engine Hook (`rslib/`)
- **Card Interception**: `rslib/src/notetype/render.rs:122-126` intercepts note types named `"StudyLab Procedural Anchor"`, resolving `ProceduralPayload` from field 0.
- **Telemetry Stripping**: `rslib/src/scheduler/answering/mod.rs:353-505` extracts `custom_data["studylab"]` to write attempt logs and update skill states in `collection.procedural`, stripping telemetry before committing to `collection.anki21` to preserve the 100-byte limit.

## Code Layout
- `rslib/procedural/`: In-tree Rust procedural engine (CAS, step validator, 175-topic catalog, 5D units, mastery store, diagnostic mock engine).
- `qt/aqt/reviewer.py`: Desktop reviewer bridge, `Show Answer` suppression, and IPC command dispatcher.
- `ts/reviewer/`: Svelte/TypeScript reviewer components (`procedural.ts`, `mcq_container.ts`, `numerical_container.ts`, `stepwise_container.ts`, `mistake_footer.ts`).
- `tools/studylab_content_factory.py`: Universal Content Factory generating all 175 topic blueprints.
- `docs/`: Canonical 10-document StudyLab master specification suite.
- `01_research_findings.md` through `08_release_decision.md`: Forensic audit records and empirical verification artifacts.
