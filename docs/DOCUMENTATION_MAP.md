# StudyLab Documentation Map & Source-Truth Guide

**Document Version:** 1.0.0 (Canonical)  
**Target Repository:** `Anki-maths` (StudyLab Subsystem)  
**Status:** AUTHORITATIVE CANONICAL SITEMAP & READING GUIDE  
**Integrity Mode:** 100% Grounded in Executable Source Code & Test Evidence  

---

## 1. Master Documentation Index & Subsystem Matrix

The table below catalogs the complete documentation suite for StudyLab. Each document serves a dedicated architectural, pedagogical, or operational purpose.

### 1.1 Canonical Master Specifications (The 10 Frozen Contracts)

| # | Master Specification | Primary Focus & Scope | Target Audience | Key Symbols & Artifacts Covered |
|---|---|---|---|---|
| 1 | **[STUDYLAB_PRODUCT_CONTRACT.md](STUDYLAB_PRODUCT_CONTRACT.md)** | Product North Star, 5-stage learner loop, 8-level diagnostic hierarchy, Speed-Accuracy model. | Product Leads & AI Agents | Two-System Architecture, ACT-R Production Rules, 5-tier diagnostic field ledger. |
| 2 | **[FRONTEND_PRODUCT_SPEC.md](FRONTEND_PRODUCT_SPEC.md)** | 9 learning object modalities, semantic modality invariant, Cognitive Tutor inner loop. | Frontend & UX Devs | `problem`, `quick`, `mcq`, `stepwise`, `concept_check`, `strategy_drill`, `worked_example`. |
| 3 | **[FRONTEND_UI_STATE_SPEC.md](FRONTEND_UI_STATE_SPEC.md)** | 14 frontend states, transitions, keyboard behavior, native Anki button suppression. | Frontend Devs | `ProceduralUIState`, anti-bypass mistake gate, Space/Enter keyboard isolation. |
| 4 | **[FRONTEND_BUTTON_CONTRACT.md](FRONTEND_BUTTON_CONTRACT.md)** | Canonical master button matrix across 23 controls, priorities, and mutual exclusions. | Frontend Devs | Master button matrix, CTA hierarchy, forbidden coexistence rules. |
| 5 | **[FRONTEND_VISUAL_DESIGN_SPEC.md](FRONTEND_VISUAL_DESIGN_SPEC.md)** | "Problem is the Visual Hero", CSS design tokens (`--proc-*`), dark mode, anti-patterns. | Designers & Frontend | Design tokens, typography scale, prohibited visual clutter and telemetry dumps. |
| 6 | **[APKG_CONTENT_CONTRACT.md](APKG_CONTENT_CONTRACT.md)** | Content architecture: Canonical StudyLab Source APKG (`StudyLab-Source-APKG-Contract(1).txt`) and Procedural Declarative Blueprints. | Content Creators & Architects | `SourceQuestion`, `StudyLab Source`, `ProceduralPayload`, `ProceduralCardAnchor`, 175 topics. |
| 7 | **[APKG_FRONTEND_CONTRACT.md](APKG_FRONTEND_CONTRACT.md)** | 4-tier cross-layer mapping (APKG → Rust → SQLite → Python/Qt → TypeScript). | System Architects | End-to-end data pipeline, field immutability, anti-cheat sanitization boundaries. |
| 8 | **[DATABASE_DATA_CONTRACT.md](DATABASE_DATA_CONTRACT.md)** | Dedicated `collection.procedural` store, 16 tables, 22 indexes, v1-v5 migrations. | Database & Rust Devs | 16 tables, WAL pragma, atomic transactions, durable vs derived data taxonomy. |
| 9 | **[FRONTEND_ACCEPTANCE_MATRIX.md](FRONTEND_ACCEPTANCE_MATRIX.md)** | 12-screen testable acceptance criteria, WCAG 2.1 AA compliance, Perfect Window criteria. | QA & Auditors | Screen-by-screen acceptance, accessibility matrix, performance latency budgets. |
| 10 | **[FRONTEND_CURRENT_STATE_GAP_MAP.md](FRONTEND_CURRENT_STATE_GAP_MAP.md)** | Screenshot-grounded forensic gap audit, zero P0 defects, and remediation ledger. | QA & Auditors | Win32 GDI + CDP evidence audit, defect resolutions, and P1/P2 polish ledger. |

### 1.2 Subsystem Architecture & Domain Deep Dives

| Document | Primary Focus & Scope | Target Audience | Key Symbols & Artifacts Covered |
| :--- | :--- | :--- | :--- |
| **[README.md](README.md)** | Subsystem entry point, core identity, and quickstart. | All Readers | Core Invariant, Justfile targets, project directory layout. |
| **[SYSTEM_ARCHITECTURE.md](SYSTEM_ARCHITECTURE.md)** | End-to-end multi-layer pipeline, 17-step lifecycle, crate structure. | System Architects | `rslib/procedural/`, Webview $\to$ Qt $\to$ Rust $\to$ SQLite execution pipeline. |
| **[LEARNING_MODEL.md](LEARNING_MODEL.md)** | Cognitive model, EMA mastery tracking, 8 progression states, 6 mastery gates. | ML & Cognitive Eng | `SkillState`, EMA $\alpha=0.20$, 6-Gate Promotion Policy, 4-Tier Domain Hierarchy. |
| **[CONTENT_AND_AUTHORING.md](CONTENT_AND_AUTHORING.md)** | Declarative authoring, 15 parameter domains, 24 derivations, content factory. | Content Creators | `DeclarativeFamilyContract`, `ParameterDomain`, `AnswerDerivation`, APKG packaging. |
| **[LEARNING_OBJECTS.md](LEARNING_OBJECTS.md)** | Interactive answer modalities, physical units (5D vectors), stepwise reasoning. | Frontend & UX Devs | `MCQContainer`, `NumericalContainer`, `StepwiseContainer`, `MistakeFooter`. |
| **[REVIEWER_STATE_MACHINE.md](REVIEWER_STATE_MACHINE.md)** | 11-state transition lifecycle, speed quadrants, keyboard isolation, teardown. | Frontend Devs | `ProceduralUIState`, `computeSpeedQuadrant`, Space/Enter trap, `destroyActive`. |
| **[FRONTEND_BACKEND_CONTRACT.md](FRONTEND_BACKEND_CONTRACT.md)** | IPC bridge command protocols, `pycmd` routing, telemetry packaging, hook lifecycle. | Full-Stack Eng | `procedural_*` bridge protocol, `mutateNextCardStates`, ephemeral customData. |
| **[DATA_AND_PERSISTENCE.md](DATA_AND_PERSISTENCE.md)** | `procedural.db` SQLite schema, v1–v5 migrations, indexes, atomic transactions. | Database & Rust Devs | 11 tables, 17 indexes, WAL pragmas, `record_practice_attempt_atomic()`. |
| **[DIAGNOSTIC_AND_REMEDIATION.md](DIAGNOSTIC_AND_REMEDIATION.md)** | Diagnostic mock sessions, 4-dimension error analysis, 9-tier remediation queue. | Remediation Eng | `MockSession`, 4-tier report hierarchy, circuit breaker ($\ge 5$ recurrences). |
| **[ARCHITECTURE_INVARIANTS.md](ARCHITECTURE_INVARIANTS.md)** | Non-negotiable system invariants, security defenses, performance budgets. | Architects & QA | XSS sanitization, 100-byte custom data limit, memory leak prevention. |
| **[DOCUMENTATION_TRUTH_MATRIX.md](DOCUMENTATION_TRUTH_MATRIX.md)** | Canonical 18-area truth matrix, historical gap resolutions, forensic citations. | Forensic Auditors | Reconciles Phase 01–03 drift against executable code and test evidence. |
| **[DEEPSEARCH_EVIDENCE.md](DEEPSEARCH_EVIDENCE.md)** | Peer-reviewed cognitive psychology and learning sciences literature synthesis. | Researchers | Anderson ACT-R, VanLehn Cognitive Tutors, Sweller CLT, Metcalfe Hypercorrection. |
| **[OPEN_QUESTIONS.md](OPEN_QUESTIONS.md)** | Pruned register of genuinely open product choices and architecture explorations. | Product Leads | FSRS Ease 2 heuristic, multi-device SQLite sync, Wasm mobile engine. |
| **[DOCUMENTATION_MAP.md](DOCUMENTATION_MAP.md)** | Comprehensive sitemap, reader personas, reading paths, source index. | All Readers | This document. |

---

## 2. Reader Personas & Tailored Reading Paths

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

### Path 1: New AI Agent / LLM Clean-Context Fast-Start
*Goal: Acquire full mental model of StudyLab without hallucinations in under 3 minutes.*
1. Read **[STUDYLAB_PRODUCT_CONTRACT.md](STUDYLAB_PRODUCT_CONTRACT.md)** (Sections 1–4) for North Star, 5-stage loop, and core boundaries.
2. Read **[DOCUMENTATION_TRUTH_MATRIX.md](DOCUMENTATION_TRUTH_MATRIX.md)** for ground truth invariants and historical gap resolutions.
3. Read **[SYSTEM_ARCHITECTURE.md](SYSTEM_ARCHITECTURE.md)** for pipeline execution flow.
4. Consult **[DOCUMENTATION_MAP.md](DOCUMENTATION_MAP.md)** (Section 4) for exact file paths.

### Path 2: Core Rust Engine Developer
*Goal: Implement new generators, validation algorithms, storage migrations, or scheduling policies.*
1. Read **[SYSTEM_ARCHITECTURE.md](SYSTEM_ARCHITECTURE.md)** (Crate layout `rslib/procedural/`).
2. Read **[DATABASE_DATA_CONTRACT.md](DATABASE_DATA_CONTRACT.md)** & **[DATA_AND_PERSISTENCE.md](DATA_AND_PERSISTENCE.md)** (Schema DDL, pragmas, atomic transactions).
3. Read **[LEARNING_MODEL.md](LEARNING_MODEL.md)** (EMA mastery, signals, 6 progression gates).
4. Read **[DIAGNOSTIC_AND_REMEDIATION.md](DIAGNOSTIC_AND_REMEDIATION.md)** (`StepValidator`, `RemediationQueue`, circuit breakers).

### Path 3: Frontend TypeScript & UX Engineer
*Goal: Modify webview UI, add answer modalities, tweak keyboard behavior, or optimize styles.*
1. Read **[FRONTEND_PRODUCT_SPEC.md](FRONTEND_PRODUCT_SPEC.md)** & **[FRONTEND_UI_STATE_SPEC.md](FRONTEND_UI_STATE_SPEC.md)** (9 modalities, 14 states, keyboard trapping).
2. Read **[FRONTEND_BUTTON_CONTRACT.md](FRONTEND_BUTTON_CONTRACT.md)** & **[FRONTEND_VISUAL_DESIGN_SPEC.md](FRONTEND_VISUAL_DESIGN_SPEC.md)** (Master button matrix, design tokens).
3. Read **[FRONTEND_BACKEND_CONTRACT.md](FRONTEND_BACKEND_CONTRACT.md)** (Bridge commands, `mutateNextCardStates`).

### Path 4: Python / Qt Desktop Integration Engineer
*Goal: Maintain webview container bridge, link handlers, hook lifecycles, and desktop menus.*
1. Read **[FRONTEND_BACKEND_CONTRACT.md](FRONTEND_BACKEND_CONTRACT.md)** (`_handle_procedural_command`, link handlers).
2. Read **[STUDYLAB_PRODUCT_CONTRACT.md](STUDYLAB_PRODUCT_CONTRACT.md)** (Card separation, FSRS rating bridge).
3. Read **[ARCHITECTURE_INVARIANTS.md](ARCHITECTURE_INVARIANTS.md)** (Memory leak prevention, `destroyActive()`).

### Path 5: Curriculum Designer & Content Author
*Goal: Create new academic subjects, chapters, declarative problem blueprints, or export APKG decks.*
1. Read **[APKG_CONTENT_CONTRACT.md](APKG_CONTENT_CONTRACT.md)** & **[CONTENT_AND_AUTHORING.md](CONTENT_AND_AUTHORING.md)** (Declarative archetypes, 15 domains, 24 derivations).
2. Read **[LEARNING_OBJECTS.md](LEARNING_OBJECTS.md)** (Modality contracts, decision points, hint structures).
3. Run `python tools/studylab_content_factory.py` and `python generate_procedural_apkg.py`.

### Path 6: Cognitive Scientist & Learning Science Auditor
*Goal: Verify pedagogical alignment with cognitive load theory, knowledge tracing, and retrieval practice.*
1. Read **[DEEPSEARCH_EVIDENCE.md](DEEPSEARCH_EVIDENCE.md)** (Literature synthesis, ACT-R, VanLehn, Sweller).
2. Read **[LEARNING_MODEL.md](LEARNING_MODEL.md)** (EMA vs BKT, 6-gate mastery policy).
3. Read **[DOCUMENTATION_TRUTH_MATRIX.md](DOCUMENTATION_TRUTH_MATRIX.md)** (Research facts vs engineering heuristics).

---

## 3. Source-of-Truth Hierarchy

When reconciling discrepancies across documentation, phase reports, and code, all claims must be resolved strictly according to the following **8-Tier Source-of-Truth Hierarchy**:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     SOURCE-OF-TRUTH HIERARCHY                           │
├───────┬──────────────────────────────────┬──────────────────────────────┤
│ Tier  │ Authority Source                 │ Resolution Role              │
├───────┼──────────────────────────────────┼──────────────────────────────┤
│ **1** │ Current Executable Source Code   │ Supreme Ground Truth         │
│ **2** │ Current Passing Test Suites      │ Behavioral Ground Truth      │
│ **3** │ Current Schemas / Migrations     │ Structural Ground Truth      │
│ **4** │ Current Verified Artifacts       │ Empirical Ground Truth       │
│ **5** │ Explicit Product Requirements    │ Intent Ground Truth          │
│ **6** │ Canonical Suite Documentation    │ Explanatory Interface        │
│ **7** │ Historical Phase Reports (01-08) │ Archaeological Context       │
│ **8** │ General / Unverified Assumptions │ Subordinate (Discard if non- │
│       │                                  │ conforming)                  │
└───────┴──────────────────────────────────┴──────────────────────────────┘
```

---

## 4. Canonical Source Code Traceability Directory

When verifying or modifying the codebase, refer directly to these authoritative physical source files:

### 4.1 Rust Subsystem (`rslib/procedural/` & `rslib/`)
- **Strong Types & Academic Domains:** `rslib/procedural/src/core/mod.rs`
- **Cognitive Decision Points:** `rslib/procedural/src/core/decision.rs`
- **Declarative Contracts & Archetypes:** `rslib/procedural/src/problems/contract.rs`
- **Universal Declarative Generator:** `rslib/procedural/src/problems/declarative.rs`
- **Problem Registry & Dispatch:** `rslib/procedural/src/problems/registry.rs`
- **Step Semantic Validator & Equivalence:** `rslib/procedural/src/problems/steps/step_validator.rs`
- **Solution Graph & Step Nodes:** `rslib/procedural/src/problems/steps/step_graph.rs`
- **Dimensional Vector Algebra & Unit Registry:** `rslib/procedural/src/units/mod.rs`
- **Physics Sanity & Kinematics:** `rslib/procedural/src/physics/sanity.rs`, `kinematics.rs`
- **Chemistry Stoichiometry & Equilibrium:** `rslib/procedural/src/chemistry/reaction.rs`, `generators/`
- **Reasoning CSP Solver & Syllogisms:** `rslib/procedural/src/reasoning/csp.rs`, `syllogism.rs`
- **EMA Mastery & 6-Gate Policy:** `rslib/procedural/src/skills/mod.rs`, `progression.rs`
- **Domain Diagnostic Evidence Structs:** `rslib/procedural/src/skills/domain_evidence.rs`
- **Remediation Policy & Priority Queue:** `rslib/procedural/src/remediation/policy.rs`, `queue.rs`
- **Unified 10-Tier Scheduler:** `rslib/procedural/src/scheduling/unified.rs`
- **FSRS Rating Derivation:** `rslib/procedural/src/scheduling/rating_policy.rs`
- **SQLite Migrations (v1–v5) & DDL:** `rslib/procedural/src/storage/schema.rs`, `migration.rs`
- **Procedural Store & Atomic Transactions:** `rslib/procedural/src/storage/store.rs`
- **Webview HTML & XSS Sanitization:** `rslib/procedural/src/reviewer/template.rs`, `mod.rs`
- **Anki Card Interception Hook:** `rslib/src/notetype/render.rs:122-126`
- **Anki Answering Telemetry Stripping Hook:** `rslib/src/scheduler/answering/mod.rs:353-505`

### 4.2 TypeScript Reviewer Subsystem (`ts/reviewer/`)
- **Reviewer State Machine & API:** `ts/reviewer/procedural.ts`
- **MCQ Modality Container:** `ts/reviewer/components/mcq_container.ts`
- **Numerical & Unit Modality Container:** `ts/reviewer/components/numerical_container.ts`
- **Stepwise Multi-Step Reasoning Container:** `ts/reviewer/components/stepwise_container.ts`
- **Mistake Classification Footer:** `ts/reviewer/components/mistake_footer.ts`
- **Anki Next-Card State Mutation Bridge:** `ts/reviewer/answering.ts`
- **Diagnostic Session Controller:** `ts/reviewer/diagnostic/diagnostic_session.ts`
- **Diagnostic Report Controller:** `ts/reviewer/diagnostic/diagnostic_report.ts`

### 4.3 Python Desktop & Tooling Subsystem (`qt/`, `tools/`)
- **Python Reviewer Bridge & Command Router:** `qt/aqt/reviewer.py:697-825`
- **Declarative Content Factory:** `tools/studylab_content_factory.py`
- **APKG Deck Generator Tool:** `generate_procedural_apkg.py`
- **End-to-End Headless Integration Test:** `qt/tests/test_phase13.py`
