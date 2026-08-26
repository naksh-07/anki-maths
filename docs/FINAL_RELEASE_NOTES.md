# StudyLab Release Notes — Release Candidate v1.0.0-RC

**Product:** StudyLab Engine (for Anki Desktop)  
**Version:** 1.0.0-RC  
**Release Date:** 2026-08-26  
**Target Runtime:** Anki Desktop 26.08+ (Qt6.11 / QtWebEngine Chromium 140)  
**Distribution Package:** `dist/apkgs/StudyLab_Full_Universe_175.apkg`  
**License:** Inherited / AGPLv3 (Anki Desktop Host Ecosystem)  

---

## 1. Executive Summary

StudyLab is a procedural problem-solving, cognitive diagnosis, and adaptive practice engine embedded natively as a guest subsystem within Anki desktop. By pairing Anki's battle-tested temporal spaced-repetition scheduler (FSRS-5 / SM-2) with StudyLab's generative mathematical and scientific runtime, learners can build deep conceptual fluency, master multi-step quantitative derivations, diagnose reasoning pitfalls, and remediate prerequisites without leaving the distraction-free Anki environment.

This Release Candidate (v1.0.0-RC) represents the full-system synthesis of StudyLab across Mathematics, Logical Reasoning, Classical & Modern Physics, and General Chemistry, backed by a complete 175-topic curriculum, 16-table SQLite persistence, zero-overhead card isolation, and a clean Open Canvas 720px desktop interface.

---

## 2. Key Features & Capabilities

### 2.1 175-Topic Full Curriculum Universe
- **Curriculum Breadth:** 175 canonical topics (177 procedural anchor notes) packaged in `dist/apkgs/StudyLab_Full_Universe_175.apkg`:
  - **Mathematics (59 Topics):** Linear/Quadratic Equations, Polynomials, Exponents & Logarithms, Arithmetic & Geometric Progressions, Combinatorics, Probability, Trigonometry, Coordinate Geometry, Calculus, Modular Arithmetic, and Word Problems (Time & Work, Speed & Distance, Mixtures & Alligations, Profit & Loss).
  - **Logical Reasoning (30 Topics):** Syllogisms, Linear/Circular Seating Arrangements, Floor Grid Puzzles, Direction Sense, Blood Relations, Series Completion, Coding-Decoding, Data Sufficiency, Truth-Teller/Liar Logic, and Venn Diagram Analysis.
  - **Physics (40 Topics):** 1D/2D Kinematics, Newton's Laws, Work-Energy-Power, Gravitation, Rotational Mechanics, Fluid Statics & Dynamics, Thermodynamics & Heat Transfer, Waves & Optics, Electrostatics, Current Electricity, Magnetism, and Modern Physics (Photoelectric Effect, Nuclear Decay).
  - **Chemistry (46 Topics):** Stoichiometry, Molar Mass, Ideal Gas Laws, Atomic Structure, Periodic Trends, Chemical Bonding & Molecular Geometry, Thermodynamics & Thermochemistry, Chemical Equilibrium, Acid-Base Equilibria & Buffers, Electrochemistry, Kinetics & Rate Laws, and Basic Organic Reaction Regimes.

### 2.2 Generative Procedural Engines
- **Declarative Blueprints (Zero Rust per Topic):** Blueprints define 15 parameter domain types (Uniform Integers, Rationals, Real Decimals, Prime Sequences, Physical Constants) and 24 answer derivation strategies.
- **Parametric Novelty & Seed Reproducibility:** Every problem variant is deterministically generated from an integer seed, eliminating memorization while guaranteeing full step-level validation and reproducibility.
- **5D Dimensional Unit Analysis ($[M]^m [L]^l [T]^t [N]^n [K]^k$):** Built-in unit registry of 40+ physical units ($N$, $J$, $W$, $Pa$, $m/s$, $kg\cdot m/s^2$, $mol$, $M$, $V$, $\Omega$) with automated conversion, prefix scaling ($k, M, G, m, \mu, n$), and dimension compatibility checking.
- **StepValidator & Computer Algebra System (CAS):** Formative step-level evaluation with commutative equivalence, algebraic simplification, linear system reduction, downstream carry-through handling, and 3-tier progressive hint disclosure.

### 2.3 Interactive Modalities & Metacognitive Gating
- **Numerical Quick Solve:** Instant keyboard-driven answer entry with LaTeX rendering, scientific notation, fractions, and real-time unit preview.
- **Multiple Choice Questions (MCQ):** 4 discrete radio cards ($A$–$D$) with full ARIA radiogroup accessibility and keyboard hotkeys (`1`–`4`, `A`–`D`). Strict zero-textbox fallback invariant.
- **ConceptCheck & StrategyDrill:** Targeted qualitative diagnostic cards comparing alternate solution pathways, isolating core concepts, and detailing specific misconception rationales.
- **Stepwise Workspace:** Dedicated step-by-step algebra derivation scratchpad with instant semantic validation and error localization.
- **WorkedExample:** Scaffolding mode with key decision reflection boxes and explicit metacognitive acknowledgment gates.
- **4-Category Mistake Reflection Strip:** Upon incorrect submission, solution details and next-card navigation are locked until the user attributes their mistake (`1 Silly Slip`, `2 Pattern Missed`, `3 Concept Gap`, `4 Prereq Unknown`). Space/Enter keys are trapped to prevent mindless skipping.

### 2.4 Longitudinal Storage & Adaptive Remediation
- **Decoupled SQLite Database (`<collection>.procedural`):** 16 relational tables with 22 indexes, WAL journaling, and single-transaction ACID atomicity.
- **Mastery Modeling (EMA $\alpha=0.20$):** Exponential Moving Average mastery estimation paired with 6 composite promotion gates (accuracy, latency, difficulty level, transfer performance, zero active remediation, and prerequisite readiness).
- **9-Tier Remediation Policy:** Automatic queue insertion of targeted remedial activities (Immediate Transfer Retry, ConceptCheck, StrategyDrill, Foundational Prerequisite Review, Worked Example, or Cooldown Circuit Breakers).

### 2.5 Open Canvas Desktop Reviewer UI
- **720px Centered Problem Container:** Designed specifically for standard laptop displays (1366×768 to 1920×1080), putting primary problem stems immediately in the focal viewport without nested box clutter.
- **Subtle 3px Left Accent Borders:** Replaces heavy colorful card boxes with clean, professional typography and light status borders.
- **Fixed Bottom Interaction Footer:** Floating bottom action bar ensuring single, non-overlapping CTA buttons across all 11 UI states.
- **Muted Speed Quadrant Pills:** Non-distracting inline badges (`⚡ Fast & Accurate · 6.2s`, `⏱ Accurate · Paced · 24.5s`).

---

## 3. Breaking Changes & Migration Safety

### 3.1 Zero Impact on Upstream Anki Collections
- **Standard Anki Cards (Basic, Cloze):** 100% untouched. Standard notes bypass all procedural interceptors in `rslib/src/notetype/render.rs:122-126` and `qt/aqt/reviewer.py:687`.
- **AnkiWeb Sync & Bandwidth Safety:** Telemetry envelopes in `card.custom_data["studylab"]` are extracted into `<collection>.procedural` and stripped in Rust before writing to `collection.anki2`. The 100-byte limit on `cards.data` is strictly respected.
- **Database Migrations (v1 through v5):** All 5 schema migrations are executed idempotently inside transactional boundaries upon opening the collection. Downgrading or inspecting the database is safe and fully backward-compatible.

---

## 4. Validation & Test Coverage Summary

The release candidate has undergone comprehensive, victory-grade automated and live verification:
- **TypeScript Reviewer Test Suite:** 18 test files, 154 passed, 0 failed (`npm run vitest:once`).
- **Rust Procedural Crate & Solvers:** 134 library tests, 240+ integration tests passed (`cargo test -p procedural`).
- **175-Topic Factory Benchmark:** 177 topics rendered in 50.6ms with 100% contract validity.
- **Canonical APKG Validator:** 177 procedural notes verified for field schemas, inline contracts, provenance, and self-contained import (`artifacts_qa/validate_canonical_apkg.py`).
- **Python Qt Reviewer Tests:** 17 ninja tasks passed (`tools/ninja.bat check:pytest`).
- **Live Desktop Webview Matrix:** 14 canonical UI states captured in real visible Windows Anki DEV GUI (`HWND: 3473492`, `PID: 18060`, `CDP Port: 9222`) with dual PNG screenshots and SHA-256 hashes in `artifacts_qa/final_release_audit/evidence.json`.

---

## 5. Deployment & Operator Instructions

### 5.1 Building the Release Package
To build the canonical APKG package from source:
```powershell
.\out\pyenv\Scripts\python.exe tools\studylab_content_factory.py --generate-apkgs dist/apkgs
```

### 5.2 Validating the Canonical APKG
To verify integrity of the generated APKG:
```powershell
.\out\pyenv\Scripts\python.exe artifacts_qa\validate_canonical_apkg.py
```

### 5.3 Installing in Anki Desktop
1. Launch Anki Desktop (version 26.08+ recommended).
2. Go to **File -> Import...** (`Ctrl+I`).
3. Select `dist/apkgs/StudyLab_Full_Universe_175.apkg`.
4. Click **Import**. All 175 curriculum decks across Mathematics, Reasoning, Physics, and Chemistry will populate with zero external dependencies.

### 5.4 Operator Diagnostic Tools
- **Live Visual Audit Runner:**
  ```powershell
  .\out\pyenv\Scripts\python.exe artifacts_qa\live_visual_audit_runner.py
  ```
- **Inspect Procedural Database:**
  SQLite store is located at `<Anki_Profile_Folder>/User 1/collection.procedural`. Connect with any SQLite viewer (e.g. `sqlite3 collection.procedural`) to inspect `skills`, `skill_states`, and `practice_attempts`.
