# StudyLab (Anki Desktop Procedural Learning Subsystem)

[![Build Status](https://github.com/ankitects/anki/actions/workflows/ci.yml/badge.svg)](https://github.com/ankitects/anki/actions/workflows/ci.yml)
[![Documentation](https://img.shields.io/badge/docs-StudyLab%20Canonical%20Specs-blue)](./docs/README.md)
[![Rust Procedural Crate](https://img.shields.io/badge/rust-procedural%20v0.0.0-orange)](./rslib/procedural/)

> **Product North Star:**  
> *"Anki is the familiar, distraction-free spaced repetition shell; StudyLab provides the procedural intelligence layer inside it."*

---

## 1. What is StudyLab?

**StudyLab** is an in-tree procedural problem-solving and adaptive cognitive learning engine hosted inside the Anki desktop runtime.

Traditional spaced repetition systems optimize **declarative paired-associate memory retrieval** ($Q \to A$). StudyLab provides a rich, multi-domain cognitive problem-solving workspace for quantitative and analytical STEM disciplines: **Mathematics**, **Physics**, **Chemistry**, and **Logical Reasoning**.

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                           CORE ARCHITECTURAL INVARIANT                           │
├──────────────────────────────────────────────────────────────────────────────────┤
│ "StudyLab is not a flashcard app; it is an adaptive procedural problem-solving   │
│  engine hosted inside Anki."                                                     │
│                                                                                  │
│ • Anki owns collection management, FSRS scheduling, and standard flashcards.     │
│ • StudyLab owns parametric problem generation, 5D unit parsing, CAS step         │
│   validation, mistake classification, diagnostic assessment, and remediation.    │
└──────────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Two-System Architecture

StudyLab strictly separates declarative memory retention from procedural problem-solving skills:

| Dimension | System 1: Declarative Host (Anki) | System 2: Procedural Engine (StudyLab) |
|---|---|---|
| **Cognitive Unit** | Chunks / Paired Associates ($Q \to A$) | Production Rules & Multi-Step Derivations |
| **Core Question** | *"Did you remember this fact/formula?"* | *"Can you apply concepts and solve correctly?"* |
| **Interaction Surface** | Card Flip / Reveal ($Q \to \text{Show Answer}$) | Modality-Matched Inputs (MCQ, Stepwise, Numerical) |
| **Error Diagnosis** | Binary Recall (Forgot vs Remembered) | 4 Cognitive Dimensions (Concept, Execution, Transfer, Speed) |
| **Scheduling Algorithm** | FSRS-5 / SM-2 Spaced Repetition | 10-Tier Cognitive Scheduler + JIT Remediation Queue |
| **Primary Persistence** | `collection.anki21` (100-byte `cards.data`) | Isolated SQLite Store (`collection.procedural`, 16 tables) |
| **Implementation Layer** | `rslib/src/collection/`, `pylib/`, `qt/` | `rslib/procedural/`, `ts/reviewer/procedural.ts` |

---

## 3. Multi-Domain Curriculum Coverage (175 Topics / 177 Notes)

StudyLab packages a complete, source-grounded academic curriculum covering **175 benchmark topics** across four core domains:

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                        175 CURRICULUM TOPICS TAXONOMY                            │
├───────────────────┬───────────────────┬───────────────────┬──────────────────────┤
│  MATHEMATICS (59) │    PHYSICS (40)   │   CHEMISTRY (46)  │ LOGICAL REASONING(30)│
├───────────────────┼───────────────────┼───────────────────┼──────────────────────┤
│ • Linear Systems  │ • 1D Kinematics   │ • Stoichiometry   │ • CSP Seating        │
│ • Number Theory   │ • Work & Energy   │ • ICE Equilibrium │ • Syllogisms         │
│ • Percentages     │ • Dimensional Alg │ • Buffer pH       │ • Kinship DAG        │
│ • Geometry & Area │ • Sanity Checks   │ • Reaction Rates  │ • Direction Vectors  │
│ • Arithmetic Sums │ • Unit Registry   │ • Electrochem     │ • Floor / Grid Logic │
└───────────────────┴───────────────────┴───────────────────┴──────────────────────┘
```

- **175 Curriculum Topics**: The canonical academic target universe (59 Math, 30 Reasoning, 40 Physics, 46 Chemistry).
- **177 Generated Notes**: In the official distribution package (`StudyLab_Full_Universe_175.apkg`), there are 177 procedural notes because two Mathematics topics (`statistics_variance_std_dev` and `data_interpretation_basics`) package specialized sub-modality anchor notes.

---

## 4. Canonical Specification Suite (`docs/`)

The repository contains 10 authoritative, frozen master specification documents:

| # | Master Specification | Primary Scope & Focus |
|---|---|---|
| 1 | **[`docs/STUDYLAB_PRODUCT_CONTRACT.md`](./docs/STUDYLAB_PRODUCT_CONTRACT.md)** | Product North Star, 5-stage learner loop, 8-level diagnostic hierarchy, Speed-Accuracy model. |
| 2 | **[`docs/FRONTEND_PRODUCT_SPEC.md`](./docs/FRONTEND_PRODUCT_SPEC.md)** | 9 learning object modalities, semantic modality invariant, Cognitive Tutor inner loop. |
| 3 | **[`docs/FRONTEND_UI_STATE_SPEC.md`](./docs/FRONTEND_UI_STATE_SPEC.md)** | 14 frontend states, transitions, keyboard behavior, native Anki button suppression. |
| 4 | **[`docs/FRONTEND_BUTTON_CONTRACT.md`](./docs/FRONTEND_BUTTON_CONTRACT.md)** | Canonical 23-control button matrix, priority hierarchy, and mutual exclusions. |
| 5 | **[`docs/FRONTEND_VISUAL_DESIGN_SPEC.md`](./docs/FRONTEND_VISUAL_DESIGN_SPEC.md)** | "Problem is the Visual Hero", design tokens, dark mode, prohibited anti-patterns. |
| 6 | **[`docs/APKG_CONTENT_CONTRACT.md`](./docs/APKG_CONTENT_CONTRACT.md)** | Declarative blueprints, `ProceduralPayload` schema, and 175-topic curriculum taxonomy. |
| 7 | **[`docs/APKG_FRONTEND_CONTRACT.md`](./docs/APKG_FRONTEND_CONTRACT.md)** | 4-tier cross-layer mapping (APKG → Rust → SQLite → Python → TypeScript). |
| 8 | **[`docs/DATABASE_DATA_CONTRACT.md`](./docs/DATABASE_DATA_CONTRACT.md)** | Dedicated `collection.procedural` store, 16 tables, 22 indexes, v1-v5 migrations. |
| 9 | **[`docs/FRONTEND_ACCEPTANCE_MATRIX.md`](./docs/FRONTEND_ACCEPTANCE_MATRIX.md)** | 12-screen testable acceptance criteria, WCAG 2.1 AA compliance, Perfect Window criteria. |
| 10 | **[`docs/FRONTEND_CURRENT_STATE_GAP_MAP.md`](./docs/FRONTEND_CURRENT_STATE_GAP_MAP.md)** | Screenshot-grounded forensic gap audit, zero P0 defects, and remediation ledger. |

For the complete architectural reading guide, see **[`docs/README.md`](./docs/README.md)** and **[`docs/DOCUMENTATION_MAP.md`](./docs/DOCUMENTATION_MAP.md)**.

---

## 5. Development & Verification Quickstart

All build, test, and verification workflows are managed through `just`:

```powershell
# 1. Run full project formatting and checks
just check

# 2. Execute Rust procedural crate unit & integration tests
cargo test -p procedural

# 3. Execute 175-topic universal content factory test
cargo test -p procedural --test phase36c_all_175_topics_factory_tests

# 4. Execute TypeScript frontend test suite
npm --prefix ts test ts/reviewer/procedural.test.ts

# 5. Validate canonical 175-topic APKG package
python artifacts_qa/validate_canonical_apkg.py

# 6. Launch Anki desktop in development mode with remote debugging
just run
```

---

## 6. Upstream Anki Host & License

This repository is built on the computer version of [Anki](https://apps.ankiweb.net).
- Contribution Guidelines: [Contribution Guidelines](./docs/contributing.md)
- Development Guide: [Development](./docs/development.md)
- Contributors: [CONTRIBUTORS](./CONTRIBUTORS)
- License: [GNU AGPLv3](./LICENSE)
