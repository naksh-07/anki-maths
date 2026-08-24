# StudyLab Documentation Map & Source-Truth Guide

**Document Version:** 1.0.0 (Canonical)  
**Target Repository:** `Anki-maths` (StudyLab Subsystem)  
**Status:** AUTHORITATIVE CANONICAL SITEMAP & READING GUIDE  
**Integrity Mode:** 100% Grounded in Executable Source Code & Test Evidence  

---

## 1. Master Documentation Index & Subsystem Matrix

The table below catalogs the canonical documentation suite for StudyLab. Each document serves a dedicated architectural, pedagogical, or operational purpose.

| Document | Primary Focus & Scope | Target Audience | Key Symbols & Artifacts Covered |
| :--- | :--- | :--- | :--- |
| **[README.md](README.md)** | Subsystem entry point, core identity, and quickstart. | All Readers | Core Invariant, Justfile targets, project directory layout. |
| **[PRODUCT_VISION.md](PRODUCT_VISION.md)** | Product North Star, Two-Memory Architecture, target learners, non-goals. | Product & AI Agents | ACT-R Declarative vs Procedural Memory, System 1 vs System 2. |
| **[PRODUCT_BOUNDARIES.md](PRODUCT_BOUNDARIES.md)** | System ownership boundaries (Anki responsibilities vs StudyLab responsibilities). | Core Engineers | Spacing scheduler vs procedural engine, 100-byte custom data boundary. |
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
1. Read **[DOCUMENTATION_TRUTH_MATRIX.md](DOCUMENTATION_TRUTH_MATRIX.md)** (Sections 1–4) for ground truth invariants and historical gap resolutions.
2. Read **[SYSTEM_ARCHITECTURE.md](SYSTEM_ARCHITECTURE.md)** for pipeline execution flow.
3. Read **[PRODUCT_BOUNDARIES.md](PRODUCT_BOUNDARIES.md)** for Anki vs StudyLab ownership.
4. Consult **[DOCUMENTATION_MAP.md](DOCUMENTATION_MAP.md)** (Section 4) for exact file paths.

### Path 2: Core Rust Engine Developer
*Goal: Implement new generators, validation algorithms, storage migrations, or scheduling policies.*
1. Read **[SYSTEM_ARCHITECTURE.md](SYSTEM_ARCHITECTURE.md)** (Crate layout `rslib/procedural/`).
2. Read **[DATA_AND_PERSISTENCE.md](DATA_AND_PERSISTENCE.md)** (Schema DDL, pragmas, atomic transactions).
3. Read **[LEARNING_MODEL.md](LEARNING_MODEL.md)** (EMA mastery, signals, 6 progression gates).
4. Read **[DIAGNOSTIC_AND_REMEDIATION.md](DIAGNOSTIC_AND_REMEDIATION.md)** (`StepValidator`, `RemediationQueue`, circuit breakers).

### Path 3: Frontend TypeScript & UX Engineer
*Goal: Modify webview UI, add answer modalities, tweak keyboard behavior, or optimize styles.*
1. Read **[REVIEWER_STATE_MACHINE.md](REVIEWER_STATE_MACHINE.md)** (11 states, speed quadrants, keyboard trapping).
2. Read **[LEARNING_OBJECTS.md](LEARNING_OBJECTS.md)** (`MCQContainer`, `NumericalContainer`, `StepwiseContainer`).
3. Read **[FRONTEND_BACKEND_CONTRACT.md](FRONTEND_BACKEND_CONTRACT.md)** (Bridge commands, `mutateNextCardStates`).

### Path 4: Python / Qt Desktop Integration Engineer
*Goal: Maintain webview container bridge, link handlers, hook lifecycles, and desktop menus.*
1. Read **[FRONTEND_BACKEND_CONTRACT.md](FRONTEND_BACKEND_CONTRACT.md)** (`_handle_procedural_command`, link handlers).
2. Read **[PRODUCT_BOUNDARIES.md](PRODUCT_BOUNDARIES.md)** (Card separation, FSRS rating bridge).
3. Read **[ARCHITECTURE_INVARIANTS.md](ARCHITECTURE_INVARIANTS.md)** (Memory leak prevention, `destroyActive()`).

### Path 5: Curriculum Designer & Content Author
*Goal: Create new academic subjects, chapters, declarative problem blueprints, or export APKG decks.*
1. Read **[CONTENT_AND_AUTHORING.md](CONTENT_AND_AUTHORING.md)** (Declarative archetypes, 15 domains, 24 derivations).
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
