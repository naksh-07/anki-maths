# STUDYLAB PRODUCT RECONCILIATION & UX ARCHAEOLOGY
## Authoritative Product Vision, Learner Experience Architecture & Multi-Domain Interaction Model

**Document Status**: AUTHORITATIVE / RELEASE GRADE  
**Date**: 2026-08-24  
**Author**: Product Vision / UX Archaeologist Specialist  
**Target Core**: `anki-maths` / StudyLab Subsystem  

---

## 1. Executive Summary & The Product North Star

### 1.1 The Core Product North Star
> **"Anki is the familiar shell; StudyLab provides the procedural learning layer inside it ('Anki, but it understands how I solve problems')."**

StudyLab does not seek to replace Anki, reinvent its user interface from scratch, or disrupt the habits of millions of learners who rely on Anki's world-class spaced repetition infrastructure. Instead, StudyLab elevates Anki from a purely **declarative flashcard engine** (fact recall) into a comprehensive **two-system learning workstation** capable of teaching and testing **procedural problem solving, multi-step derivation, mathematical calculation, scientific reasoning, and diagnostic mistake attribution**.

```
+-----------------------------------------------------------------------------+
|                                 ANKI SHELL                                  |
|  +-----------------------------------------------------------------------+  |
|  | Standard Deck Browser / Card Browser / Stats / Sync / Preferences     |  |
|  +-----------------------------------------------------------------------+  |
|  |                            REVIEWER WINDOW                            |  |
|  |  +-----------------------------------------------------------------+  |  |
|  |  | TOP / STATUS BAR (Decks, Counts, Flags, Menus)                  |  |  |
|  |  +-----------------------------------------------------------------+  |  |
|  |  | MAIN WEBVIEW SURFACE:                                           |  |  |
|  |  |                                                                 |  |  |
|  |  |  [Standard Card]                   [StudyLab Procedural Card]   |  |  |
|  |  |  - Front text/image                - Dynamic Parameterized Item |  |  |
|  |  |  - (Press Space)                   - Quick/Stepwise/MCQ Mode    |  |  |
|  |  |  - Back text/cloze                 - Semantic Step Validation   |  |  |
|  |  |                                    - Speed Quadrant Analysis    |  |  |
|  |  |                                    - Compact Mistake Footer     |  |  |
|  |  |                                    - Targeted Remediation       |  |  |
|  |  +-----------------------------------------------------------------+  |  |
|  |  | NATIVE FOOTER (Show Answer / Again / Hard / Good / Easy)       |  |  |
|  |  +-----------------------------------------------------------------+  |  |
|  +-----------------------------------------------------------------------+  |
+-----------------------------------------------------------------------------+
```

### 1.2 The Three Non-Negotiable Tenets
1. **Pristine Upstream Coexistence (Zero Regressions)**:
   Standard Anki note types (`Basic`, `Cloze`, `Image Occlusion`) and standard user interactions (keyboard shortcuts, ease ratings, sync protocols, database schema in `collection.anki2`) remain 100% upstream-identical and completely unencumbered.
2. **The Procedural Anchor Model**:
   Procedural cards live in Anki's collection as lightweight *Memory Anchors* (using note type `StudyLab Procedural Anchor` and payload `ProceduralPayload`). Anki's FSRS/SM-2 spaced repetition scheduler determines *when* a skill is due; StudyLab dynamically instantiates, validates, and diagnoses *how* that skill is practiced.
3. **Meta-Cognitive Reflection Over Mere Repetition**:
   When a learner fails a procedural problem, traditional flashcards offer only binary failure ("Again"). StudyLab immediately presents a compact, keyboard-navigable **Mistake Taxonomy Strip** (`[1 Silly Slip]`, `[2 Pattern Missed]`, `[3 Concept Gap]`, `[4 Prereq Unknown]`), captures deep cognitive telemetry, and branches into structured remediation (Concept Checks, Strategy Drills, Worked Examples, or Prerequisite bridges).

---

## 2. UX Archaeology & Historical Evolution (Phases 1 to 41)

Through systematic forensic audit of the codebase, Git history, Rust workspace crates, TypeScript review components, and test suites, the architectural evolution of StudyLab reflects a deliberate journey from prototype to production-grade procedural engine.

### Chronological Archaeology Table
| Phase Milestone | Primary Focus | Codebase Artifacts & Evidence | UX Impact & Evolution |
|:---|:---|:---|:---|
| **Phase 1-3** | Subsystem Isolation & `procedural.db` | `rslib/procedural/`, `docs/procedural_architecture.md` | Isolated Rust crate and separate SQLite store created. Upstream `collection.anki2` protected against schema pollution. |
| **Phase 4-7** | Math Generators & Anchor Interceptor | `rslib/src/notetype/render.rs:122`, `problems/generators/` | Rust renderer intercepts `StudyLab Procedural Anchor` cards; LCM/HCF and percentage generators integrated. |
| **Phase 12-16** | Deterministic APKG Fixture Pipeline | `generate_procedural_apkg.py`, `generate_apkg.py` | Standalone Python generator produces test APKGs with rich declarative payloads and seed control. |
| **Phase 17-19** | Hot-Path Performance & Difficulty Audits | `tests/phase17_production_hot_path_audit.rs`, `phase18_difficulty_archetype_audit.rs` | Millisecond-level AST generation; difficulty scales 1.0 to 5.0 with calibrated target time thresholds. |
| **Phase 20-24** | Content Contracts & Longitudinal Modeling | `tests/phase20_content_driven_architecture_tests.rs`, `phase24_quality_and_longitudinal_outcome_audit.rs` | Shifted from hardcoded Rust generators to declarative JSON content contracts and archetypes. |
| **Phase 26B/C** | Universal `content_ref` Resolution | `rslib/procedural/src/content/`, `tests/phase26c_universal_content_resolution.rs` | APKG size optimized by decoupling payload into reference identifiers resolved via local database. |
| **Phase 28-32** | Domain Evidence & Learning Dynamics | `tests/phase28_domain_evidence_contract.rs`, `phase32_fair_matched_baseline_tests.rs` | Multi-domain evidence accumulators (`DomainEvidence`, `MasteryEvidence`) tracking skill transfer across subjects. |
| **Phase 35-36** | 175-Topic Content Factory Across 4 Domains | `tests/phase36b_content_factory_tests.rs`, `phase36c_all_175_topics_factory_tests.rs` | Full vertical slices for Mathematics, Reasoning, Physics, and Chemistry authored under uniform schemas. |
| **Phase 40-41** | Production Hardening & Mistake Taxonomy | `ts/reviewer/procedural.ts`, `rslib/procedural/src/reviewer/template.rs` | Complete frontend lifecycle, compact mistake classification strip (1-4), worked example modals, and live desktop verification. |

---

## 3. Core Product Philosophy & Pedagogical Foundations

### 3.1 The Two-System Learning Engine
StudyLab reconciles two distinct modes of human memory:
1. **System 1 / Declarative Memory (Recall)**:
   - *Goal*: Fast, associative retrieval of atomic facts, definitions, formulas, and vocabulary.
   - *Paradigm*: Traditional Anki flashcards (Question -> Reveal -> Rate).
2. **System 2 / Procedural Fluency (Application & Reasoning)**:
   - *Goal*: Multi-step algorithmic execution, formula manipulation, pattern recognition, and error-free calculation under time constraints.
   - *Paradigm*: StudyLab Procedural Review (Dynamic Prompt -> Active Input/Stepwise Solving -> Semantic CAS Evaluation -> Mistake Attribution -> Remediation).

### 3.2 The Closed-Loop Procedural Learning Cycle
```
                   +-----------------------------+
                   | 1. Due Card Trigger (Anki)  |
                   +--------------+--------------+
                                  |
                                  v
                   +-----------------------------+
                   | 2. Dynamic Instance Generate|
                   |    (Seed, Parameters, CAS)  |
                   +--------------+--------------+
                                  |
                                  v
                   +-----------------------------+
                   | 3. Active Problem Solving   |
                   |    (Quick, Steps, or MCQ)   |
                   +--------------+--------------+
                                  |
                 +----------------+----------------+
                 |                                 |
                 v [Correct]                       v [Incorrect]
    +---------------------------+    +---------------------------+
    | 4A. Speed Quadrant Eval   |    | 4B. Mistake Classification|
    |  - Fluency Strength (Fast)|    |  [1 Silly Slip]           |
    |  - Speed Opp (Slow)       |    |  [2 Pattern Missed]       |
    |  - Target Time Benchmark  |    |  [3 Concept Gap]          |
    +-------------+-------------+    |  [4 Prereq Unknown]       |
                  |                  +-------------+-------------+
                  |                                |
                  |                                v
                  |                  +---------------------------+
                  |                  | 5. Targeted Remediation   |
                  |                  |  - Concept Check / Drill  |
                  |                  |  - Worked Example Card    |
                  |                  |  - Anki Fact Card Bridge  |
                  |                  +-------------+-------------+
                  |                                |
                  +---------------+----------------+
                                  |
                                  v
                   +-----------------------------+
                   | 6. Evidence & Schedule Sync |
                   |  - customData.studylab      |
                   |  - procedural.db logs       |
                   |  - FSRS/SM-2 Ease Handover  |
                   +-----------------------------+
```

### 3.3 Speed Quadrant Analysis & Time-Aware Fluency
Mastery is a function of both **accuracy** and **fluency (processing speed)**. StudyLab categorizes every completed attempt against its calibrated target latency ($T_{\text{target}}$):

| Quadrant | Accuracy | Latency ($T_{\text{actual}}$ vs $T_{\text{target}}$) | Diagnostic Meaning | Pedagogical Action |
|:---|:---|:---|:---|:---|
| ⚡ **Fluency Strength** | Correct | $T_{\text{actual}} \le T_{\text{target}}$ | High procedural automaticity and mastery. | Schedule interval expands; advance to higher difficulty or transfer variants. |
| ⏱ **Speed Opportunity** | Correct | $T_{\text{actual}} > T_{\text{target}}$ | Conceptual grasp present, but computational friction or inefficient method. | Offer strategy drill or shortcut worked example; maintain interval. |
| ⚠️ **Strategy Trap** | Incorrect | $T_{\text{actual}} \le T_{\text{target}}$ | Impulsive error, misread constraint, or falling for common distractor. | Trigger Silly/Pattern reflection; present distractor breakdown. |
| 💡 **Concept Setup** | Incorrect | $T_{\text{actual}} > T_{\text{target}}$ | Fundamental confusion, missing schema, or execution breakdown. | Trigger Concept/Prerequisite classification; offer step-by-step worked solution. |

---

## 4. Learner Experience Flow & Reviewer Lifecycle

### 4.1 Frontend UI State Machine
The TypeScript reviewer (`ts/reviewer/procedural.ts`) and Rust template (`rslib/procedural/src/reviewer/template.rs`) implement a strict, leak-free state machine:

```
[loading] ---> [ready] ---> [solving] <=======> [hint]
                               |
                               +---> [submitting]
                                         |
                       +-----------------+-----------------+
                       |                                   |
                       v [isCorrect == true]               v [isCorrect == false]
                  [feedback]                  [mistake_classification]
                       |                                   |
                       |                                   v (Select 1-4)
                       |                              [feedback]
                       |                                   |
                       +-----------------+-----------------+
                                         |
                                         v (Press Space / Next)
                                      [next] ---> [teardown]
```

### 4.2 State Transition Table
| State | Visible UI Elements | Active Keyboard Shortcuts | Backend Bridge Message |
|:---|:---|:---|:---|
| `loading` | Spinner / Loading Placeholder | None | None |
| `ready` | Breadcrumbs, Badges, Prompt, Input Controls | Auto-focus on input or first option | Initial DOM ready |
| `solving` | Stopwatch, Prompt, Inputs/Tabs, Hint Button | `Enter` (Submit), `Space` (Submit/Reveal), `1-4`/`A-D` (Options), `Tab` | None |
| `hint` | Hint container with progressive level ($L_1 \to L_3$) | `Enter`, `Space` | `procedural_hint:{"hint_level": N}` |
| `submitting` | Disabled controls, evaluation spinner | None | Local evaluation / CAS check |
| `mistake_classification` | Compact Mistake Strip `[1 Silly Slip]` .. `[4 Prereq Unknown]` | `1` (Silly), `2` (Pattern), `3` (Concept), `4` (Prereq); `Space`/`Enter` trapped | Recorded in attempt telemetry |
| `feedback` | Result Banner, Speed Quadrant, Canonical Solution, Worked Example button | `Space`, `Enter` (Advance to Next) | `procedural_attempt:{...}`, `ans` |
| `worked_example` | Worked Example card with key decision points and common pitfalls | `Enter`, `Space`, "Try Similar" button | `procedural_try_similar:{...}` |
| `next` | Transitory state before Anki card advance | Native Anki Ease shortcuts (`1`, `2`, `3`, `4`) | `procedural_answer:{ease}` |
| `teardown` | Cleared intervals, detached listeners, reset global pointers | Native Anki controls | Cleanup |

### 4.3 The Compact Mistake Classification Strip
To preserve Anki's rapid review rhythm while ensuring genuine cognitive reflection, mistake reflection is designed as a lightweight, inline action strip rather than an intrusive multi-step modal:
- **Location**: Rendered directly above the solution container within the primary reading flow.
- **Keyboard Optimization**: Single keystroke (`1`, `2`, `3`, or `4`) explicitly selects the category, emits classification telemetry, and smoothly reveals the full canonical solution.
- **Reflection Protection**: Pressing `Space` or `Enter` is trapped to prevent accidental bypass without recording a valid classification, preserving domain evidence and remediation integrity.

---

## 5. Multi-Domain Interaction Paradigms

StudyLab unifies four major academic domains under a consistent pedagogical framework while honoring the unique interaction requirements of each subject:

```
+---------------------------------------------------------------------------------------+
|                               STUDYLAB MULTI-DOMAIN SUITE                             |
+--------------------+--------------------+--------------------+------------------------+
|    MATHEMATICS     |     REASONING      |      PHYSICS       |       CHEMISTRY        |
+--------------------+--------------------+--------------------+------------------------+
| - Symbolic CAS     | - Discrete States  | - Physical Dim     | - Chemical Dim         |
| - Fractions & Surds| - Grid Layouts     |   [M]^m [L]^l [T]^t|   [M][L][T][N][K]      |
| - Stepwise Algebra | - Logic DAGs       | - SI & Non-SI Units| - Molar Concentration  |
| - Dynamic LaTeX    | - Discrete Options | - Kinematics/Energy| - Stoichiometry/Equil. |
+--------------------+--------------------+--------------------+------------------------+
```

### 5.1 Mathematics (Quantitative Aptitude & Advanced Math)
- **Input Modalities**:
  - *Numerical / Expression*: Accepts decimals (`3.14`), exact fractions (`3/4`), negative numbers (`-15`), algebraic variables (`2x + 5`), and equation forms (`x = 7`).
  - *Stepwise Scratchpad*: Allows multi-line equation transformations with independent step validation via Rust CAS.
- **Pedagogical Invariants**:
  - Exact fractional equality recognized (e.g. `0.75` equals `3/4`).
  - Tolerance bands for non-terminating decimals (default $\pm 1\%$).

### 5.2 Logical & Analytical Reasoning
- **Input Modalities**:
  - *Structured MCQ*: Authentic radio-group option buttons with keyboard selection (`1-4` or `A-D`). Zero freeform typing fallback.
  - *Constraint & Grid Visualizers*: Visual representations of seating arrangements (linear, circular), floor puzzles, and family trees.
- **Pedagogical Invariants**:
  - Elimination of position bias through deterministic seed-based option shuffling.
  - Logic verification powered by Constraint Satisfaction Problem (CSP) solvers and Propositional Logic DAG evaluators.

### 5.3 Physics
- **Input Modalities**:
  - *Unit-Aware Numerical Input*: Accepts compound unit strings (e.g. `12 m/s`, `5 kg`, `150 J`, `45 km/h`, `9.8 m/s²`).
  - *Scientific Notation*: Full support for `1.5e-3`, `3.0 * 10^8`, and `6.63e-34`.
- **Pedagogical Invariants**:
  - Strict Dimensional Analysis ($[M]^m [L]^l [T]^t$). A student answering in `m/s` for a distance problem receives an immediate diagnostic notice: *"Unit Incompatibility: expected dimension [L], received [L][T]⁻¹"*.
  - Automatic SI unit conversion and scale normalization.

### 5.4 Chemistry
- **Input Modalities**:
  - *Chemical Unit & Concentration Input*: Supports `mol`, `mmol`, `mol/L`, `M`, `mM`, `g/mol`, `kJ/mol`, `pH`, and percentages.
  - *Subscript / Reaction Formula Rendering*: Subscripts ($H_2SO_4$, $Fe^{3+}$) and equilibrium expressions.
- **Pedagogical Invariants**:
  - Multi-dimensional chemical tracking ($[M]^m [L]^l [T]^t [N]^n [K]^k$ where $[N]$ is amount of substance and $[K]$ is temperature).
  - Stoichiometric mass conservation and ionic charge balance verification.

---

## 6. Diagnostic & Mock-Test Assessment Experience

In addition to regular spaced practice, StudyLab provides a dedicated **Diagnostic Mock-Test Session Engine**:

### 6.1 Diagnostic vs. Adaptive Review Mode
| Characteristic | Spaced Practice Review | Diagnostic Mock Session |
|:---|:---|:---|
| **Primary Goal** | Targeted practice & spaced reinforcement | Bounded measurement of current capability |
| **Adaptation** | Dynamically adapts problem difficulty mid-session | Fixed measuring blueprint (no mid-test adaptation) |
| **Hints & Feedback** | Progressive hints available on demand; instant feedback | Zero hints; zero mid-test feedback (exam condition) |
| **Time Model** | Per-card target time advisory | Fixed overall time budget with per-item countdown |
| **Outcome Delivery** | Immediate per-card rating and scheduling | Comprehensive Hierarchical Diagnostic Report upon submission |

### 6.2 The 4-Tier Hierarchical Diagnostic Report
Upon completing a 10-20 question diagnostic test across mixed domains, StudyLab synthesizes an actionable performance breakdown:

```
[Domain Level]               Mathematics (82%)            Physics (64%)
                                  |                             |
[Chapter Level]             Number System (90%)         Kinematics (50%)
                                  |                             |
[Topic / Skill Level]       LCM & HCF (100%)            Relative Velocity (40%)
                                  |                             |
[Problem Family Level]      Prime Factors (100%)        Two-Body Motion (33%)
```

### 6.3 4-Dimension Diagnostic Error Breakdown
The diagnostic engine aggregates errors into four orthogonal dimensions:
1. **Concept Deficit**: Misapplication of fundamental theorems or governing laws.
2. **Calculation / Execution Slip**: Arithmetic error or sign confusion despite sound setup.
3. **Transfer Gap**: Inability to apply known principles in novel or non-standard configurations.
4. **Speed Deficit**: Correct accuracy but excessive latency exceeding benchmark by $>30\%$.

---

## 7. Design System, Accessibility & Non-Regression Contracts

### 7.1 Design Tokens & Theme Integration
StudyLab inherits Anki's native CSS variables and theme classes (`.nightMode`):
- **Light Theme**: Clean slate background (`#f8fafc`), crisp borders (`#cbd5e1`), high-contrast slate text (`#1e293b`).
- **Dark Theme (`.nightMode`)**: Deep charcoal background (`#0f172a`), muted borders (`#334155`), bright legible text (`#f1f5f9`).
- **Accent Colors**:
  - Success / Correct: Emerald (`#10b981` / `#065f46`)
  - Error / Incorrect: Crimson (`#ef4444` / `#991b1b`)
  - Warning / Speed Opportunity: Amber (`#f59e0b` / `#92400e`)
  - Informational / Fluency: Indigo/Blue (`#6366f1` / `#3b82f6`)

### 7.2 Accessibility & Keyboard Usability
- Full keyboard navigation: every interactive element (input fields, tabs, option cards, mistake buttons, next button) possesses clear `tabindex`, visible `:focus-visible` outlines, and ARIA attributes (`role="radiogroup"`, `role="radio"`, `aria-checked`).
- Zero mouse-dependency: power users can review, solve, classify, and rate complex procedural cards entirely from the home row.

### 7.3 The Non-Regression Contract
1. **Zero Global Namespace Pollution**: All procedural methods reside strictly on `globalThis.anki.procedural` or within scoped module closures.
2. **Memory Leak Prevention**: All event listeners, timers, and DOM observers are registered in a `disposables` collection and systematically destroyed on card transition (`destroy()` / `teardown`).
3. **Pristine Standard Card Fallback**: If a card is not of note type `StudyLab Procedural Anchor`, Anki's standard review pipeline executes with 0ms overhead and zero DOM alteration.

---

## 8. Summary Gap Matrix & Guidance for Specialist Workforce

To ensure seamless coordination across the StudyLab Final Reconciliation Mission, the following handoff directives are established:

| Specialist Role | Core Responsibilities & Focus Areas | Authoritative Artifact Dependency |
|:---|:---|:---|
| **Specialist 2 (Native Anki Reviewer)** | Review lifecycle hooks, webview injection points, keyboard shortcut precedence, footer rating integration. | `01_research_findings.md` |
| **Specialist 3 (Architecture Auditor)** | Rust FFI bridge, SQLite storage isolation, memory management, state machine boundaries. | `03_architecture_gap_matrix.md` |
| **Specialist 4 (Content Contract / APKG)** | APKG generation schemas, `content_ref` resolution, 175-topic catalog integrity, metadata validation. | Modality Contracts (`R2`) |
| **Specialist 5 (MCQ / Modality)** | Authentic selectable options, 1-4/A-D keyboard binding, eliminating text-box hacks. | Modality Contracts (`R2`) |
| **Specialist 6 (Math & Reasoning Pedagogy)** | Symbolic CAS equivalence, step-by-step validator, logic DAGs, constraint satisfiers. | Modality Contracts (`R2`) |
| **Specialist 7 (Physics & Chemistry Numerical UX)** | Dimensional analysis, unit parsers (`m/s`, `kg`, `mol/L`), scientific notation, tolerance bands. | Modality Contracts (`R2`) |
| **Specialist 8 (Diagnostic / Assessment)** | Diagnostic session runner, 10-20 question blueprints, 4-tier hierarchy reports, mastery evidence feeding. | Diagnostic Engine (`R4`) |
| **Specialist 9 (QtWebEngine / Desktop Reviewer)** | Remote CDP attach verification, automated UI test matrix, screenshot and evidence capture. | Live Desktop Verification (`R5`) |
| **Specialist 10 (Security & Performance)** | Memory leak detection, XSS sanitation, test suite verification (`just check`, `just test-*`). | Security Hardening (`R5`, `R6`) |

---

## 9. Conclusion
StudyLab fulfills the vision of procedural mastery inside Anki. By respecting Anki's native shell, anchoring procedural objects cleanly via metadata, and providing intelligent step evaluation, speed quadrant diagnostics, and mistake reflection, StudyLab delivers a rigorous, delightful, and pedagogically sound learning environment.
