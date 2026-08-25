# StudyLab Product Contract & Core Architectural Boundary

**Document Version:** 1.0.0 (Canonical Master Specification)  
**Target Repository:** `Anki-maths` (StudyLab Procedural Intelligence Subsystem)  
**Status:** AUTHORITATIVE SPECIFICATION  
**Integrity Mode:** Benchmark Mode (100% Grounded in Executable Code, Tests, and Cognitive Research)  
**Authoritative Sections Covered:** Sections 1, 2, 3, 4, 8, 10, 20, 24 of `ORIGINAL_REQUEST.md`

---

## 1. StudyLab North Star (Section 1)

### 1.1 Canonical Definition
**StudyLab is a procedural learning and problem-solving engine hosted inside Anki.**

StudyLab is designed to transform STEM and analytical learning from passive, static declarative memorization into deep, generative procedural mastery. It brings intelligent tutoring, parametric problem generation, step-level semantic validation, multi-dimensional cognitive diagnostics, and just-in-time (JIT) remediation directly into the learner's existing daily spaced-repetition workflow.

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                             STUDYLAB NORTH STAR                                  │
│                                                                                  │
│   "This is Anki, but it understands how I solve problems."                       │
│                                                                                  │
│   • Evaluates HOW a learner solves a problem, not just whether they recall it.   │
│   • Preserves Anki's calm, focused, daily spaced-repetition habit.               │
│   • Replaces static card-flips with generative, modality-matched problem spaces. │
└──────────────────────────────────────────────────────────────────────────────────┘
```

---

### 1.2 Explicit Negative Definitions: What StudyLab Is NOT
To prevent architectural drift and UI bloat, StudyLab is formally bounded by what it is **NOT**:

1. **NOT a flashcard application:** It does not treat problems as atomic, static text pairs ($Q \rightarrow A$).
2. **NOT a card-flip/reveal system:** It does not rely on subjective self-grading or "Show Answer" flips for procedural tasks.
3. **NOT a generic quiz app:** It does not present ungrounded multiple-choice trivia without underlying cognitive models and structural schemas.
4. **NOT a web application embedded inside Anki:** It does not render heavy, disconnected web-app chrome, navigation bars, multi-tab dashboards, or distracting animations inside Anki's reviewer.
5. **NOT a replacement for Anki's spaced-repetition system:** It does not re-implement or compete with FSRS / SM-2 temporal scheduling algorithms.
6. **NOT a second flashcard database:** It does not create parallel deck or note structures that fragment the user's collection.
7. **NOT a dashboard that exposes backend telemetry:** It does not dump raw psychometric parameters, matrix equations, or database schemas into the solving interface.

---

### 1.3 Division of Responsibilities: Anki Host vs. StudyLab Engine

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                         RESPONSIBILITY ALLOCATION MATRIX                         │
├────────────────────────────────────────┬─────────────────────────────────────────┤
│          ANKI HOST SUBSYSTEM           │        STUDYLAB PROCEDURAL ENGINE       │
├────────────────────────────────────────┼─────────────────────────────────────────┤
│ • Declarative flashcards (Basic/Cloze) │ • Procedural problem solving            │
│ • Atomic fact & vocabulary recall      │ • Parameterized problem instantiation   │
│ • Card scheduling & review intervals   │ • Numerical & dimensional vector solving│
│ • Free Spaced Repetition Scheduler     │ • MCQ reasoning & distractor analysis   │
│   (FSRS v4.5/v5) / SM-2                │ • Structured reasoning & CSP logic      │
│ • Collection management & note types   │ • Step-by-step semantic derivation (CAS)│
│ • Profile management & AnkiWeb sync    │ • Conceptual & strategy diagnosis       │
│ • Media server & window hosting        │ • Mistake classification (4-tier)       │
│ • Standard Basic/Cloze review lifecycle│ • Just-in-Time (JIT) multi-tier         │
│ • Top and bottom toolbar webviews      │   remediation (Concept/Worked Ex)       │
│ • `collection.anki2` SQLite database   │ • Skill evidence extraction & tracking  │
│                                        │ • Weakness detection & skill state      │
│                                        │ • Adaptive next-problem selection       │
│                                        │ • `col.procedural` SQLite database      │
└────────────────────────────────────────┴─────────────────────────────────────────┘
```

---

### 1.4 Canonical Review Lifecycle Contrast
The traditional "Question → Flip / Reveal → Grade" interaction describes **ONLY** Anki's standard declarative flashcard subsystem (Basic, Cloze). It MUST NOT be interpreted as StudyLab's learner workflow.

```text
ANKI STANDARD REVIEW:
    Question → Flip / Reveal → Grade

STUDYLAB PROCEDURAL REVIEW:
    Problem → Interactive Work → Evaluate → Diagnose → Next
```

---

## 2. Core Learning Loop (Section 2)

### 2.1 The 5-Stage Learner-Facing Interaction Priority
The learner-facing experience in StudyLab is strictly linear, minimal, and focused. During active problem solving, the learner experiences exactly five steps in priority order:

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                         CORE LEARNER-FACING LOOP                                 │
├──────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│   1. PROBLEM              ──────► Clean stem, LaTeX typesetting, zero clutter    │
│          │                                                                       │
│          ▼                                                                       │
│   2. ONE INTERACTION      ──────► Modality-matched input (MCQ / Num / Stepwise)  │
│          │                                                                       │
│          ▼                                                                       │
│   3. MINIMAL FEEDBACK     ──────► Instant verification (Correct / Incorrect)     │
│          │                                                                       │
│          ▼                                                                       │
│   4. USEFUL DIAGNOSIS     ──────► Only when error occurs (4-choice reflection)   │
│          │                                                                       │
│          ▼                                                                       │
│   5. ONE NEXT ACTION      ──────► Single prominent button to proceed / remediate │
│                                                                                  │
└──────────────────────────────────────────────────────────────────────────────────┘
```

1. **PROBLEM:** The learner is presented with a freshly generated, parameterized problem statement with clean mathematical typography. No extraneous metadata, difficulty badges, or telemetry clutter is visible.
2. **ONE CORRECT INTERACTION:** The learner engages with exactly *one* active interaction surface tailored to the learning object's semantic modality (e.g., radio cards for MCQ, dimensional text input for numerical, step graph for stepwise).
3. **MINIMAL FEEDBACK:** Upon submission, the engine provides immediate, concise validation without overwhelming visual fanfare or large modal overlays.
4. **DIAGNOSIS ONLY WHEN USEFUL:** If the attempt is correct, deep diagnosis is bypassed. If incorrect, the system provides a calm, focused metacognitive reflection strip (`[1 Silly]`, `[2 Pattern]`, `[3 Concept]`, `[4 Unknown]`) or distractor-specific insight.
5. **ONE CLEAR NEXT ACTION:** The interface presents a single, unambiguous primary call-to-action (e.g., `Next Problem [Enter]`, `Try Similar Problem`, `Review Prerequisite`).

---

### 2.2 The 6th Stage: Engine Learning from Attempt
Behind the scenes, completely decoupled from the immediate visual surface, the StudyLab procedural engine executes the 6th stage of the loop:

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                    STAGE 6: ENGINE ATTEMPT INGESTION PIPELINE                    │
├──────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│   [Learner Attempt Submission]                                                   │
│          │                                                                       │
│          ▼                                                                       │
│   1. Extract Domain Evidence       ──► `DomainEvidencePayload` (Math/Phys/Chem)  │
│          │                                                                       │
│          ▼                                                                       │
│   2. Classify Cognitive Error      ──► `ErrorCategory` & `StepErrorType`         │
│          │                                                                       │
│          ▼                                                                       │
│   3. Update Skill State            ──► `SkillState` in `procedural.db` (EMA/Win) │
│          │                                                                       │
│          ▼                                                                       │
│   4. Update Weakness Signals       ──► Longitudinal weakness & prerequisite gaps │
│          │                                                                       │
│          ▼                                                                       │
│   5. Evaluate Remediation Policy   ──► `remediation_queue_items` enqueue         │
│          │                                                                       │
│          ▼                                                                       │
│   6. Derive FSRS Review Outcome    ──► Rating (`Again`, `Hard`, `Good`, `Easy`)  │
│                                                                                  │
└──────────────────────────────────────────────────────────────────────────────────┘
```

**Key Invariant:** Stage 6 is pure data, psychometrics, and engine orchestration. It operates silently in the background and **MUST NEVER** generate permanent UI clutter, telemetry dumps, or visual noise during active problem solving.

---

## 3. Diagnostic Objective Hierarchy (Section 3)

### 3.1 What StudyLab Is Actually Trying to Learn About the User
Traditional flashcard engines track a single scalar dimension: *"Did the user remember the card?"* StudyLab measures multi-dimensional cognitive problem-solving competence across eight diagnostic questions:

1. **Which subject is weak?** (Mathematics, Physics, Chemistry, Logical Reasoning)
2. **Which chapter is weak?** (e.g., Mechanics vs. Electromagnetism; Algebra vs. Calculus)
3. **Which topic is weak?** (e.g., 1D Kinematics vs. Rotational Dynamics; Quadratic Equations vs. Linear Systems)
4. **Which concept / skill is weak?** (e.g., Conservation of Energy, Stopping Distance formula, Sign Convention)
5. **Which problem family is weak?** (e.g., `physics.mechanics.kinematics.stopping_distance`)
6. **Which solving strategy is weak?** (e.g., Work-Energy Method vs. Newton-Kinematics Integration)
7. **What is the cognitive nature of the error?**
   - Is it a **Conceptual / Mental Model** failure? (Selected wrong physical law or invalid formula)
   - Is it a **Calculation / Execution** slip? (Arithmetic error, sign flip, unit conversion slip with correct physics)
   - Is it a **Reasoning / Transfer** failure? (Unable to recognize the schema in a rotated or disguised context)
8. **What is the learner's speed-accuracy profile?**
   - Is the user **slow but accurate**? (Developing deliberate competence; needs fluency practice)
   - Is the user **fast but inaccurate**? (Impulsive guessing or intuitive bug; needs deliberate scaffolding)
   - Which **prerequisites** are missing in the knowledge dependency graph?
   - Which **remediation object** should be served next?
   - Which **problem variant or family** should be scheduled next?

---

### 3.2 The 8-Level Diagnostic Objective Hierarchy
StudyLab models learner competence across an 8-level structural hierarchy:

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                       DIAGNOSTIC OBJECTIVE HIERARCHY                             │
├──────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│   Level 1: SUBJECT                  (e.g., Physics)                              │
│      │                                                                           │
│      ▼                                                                           │
│   Level 2: CHAPTER                  (e.g., Mechanics)                            │
│      │                                                                           │
│      ▼                                                                           │
│   Level 3: TOPIC                    (e.g., Kinematics 1D)                        │
│      │                                                                           │
│      ▼                                                                           │
│   Level 4: SKILL / CONCEPT          (e.g., Stopping Distance & Deceleration)     │
│      │                                                                           │
│      ▼                                                                           │
│   Level 5: PROBLEM FAMILY           (e.g., `physics.mechanics.stopping_distance`)│
│      │                                                                           │
│      ▼                                                                           │
│   Level 6: ATTEMPT EVIDENCE         (e.g., Final answer, latency, steps, hints)  │
│      │                                                                           │
│      ▼                                                                           │
│   Level 7: ERROR / STRATEGY EVIDENCE(e.g., `MathEvidence`, `ErrorCategory`)      │
│      │                                                                           │
│      ▼                                                                           │
│   Level 8: REMEDIATION DECISION     (e.g., Queue ConceptCheck / WorkedExample)   │
│                                                                                  │
└──────────────────────────────────────────────────────────────────────────────────┘
```

---

### 3.3 Cognitive & Speed Metrics: Orthogonal Measurement Matrix
To avoid confounding execution slips with conceptual breakdowns, StudyLab enforces an **Orthogonal Diagnostic Measurement Matrix** (Pellegrino et al., 2001; Corbett & Anderson, 1995):

| Measurement Dimension | Cognitive Construct | Observed Evidence | Failure Symptom | Target Engine Remediation |
|---|---|---|---|---|
| **1. Concept** | Mental model, governing laws, theorem selection | `is_conceptual_error() == true`, `ErrorCategory::Concept` | Misapplied law, invalid principle, distractor trap | `ConceptCheck` / Schema Representation Drill |
| **2. Execution** | Algebraic manipulation, arithmetic precision, units | `is_execution_error() == true`, `ErrorCategory::Silly` | Sign flip, arithmetic slip, unit factor omission ($5/18$) | Precision drill, step-by-step CAS feedback |
| **3. Transfer** | Schema recognition across disguised/isomorphic stems | `transfer_evidence == false`, `ErrorCategory::Pattern` | Failure on contextual rotation or novel variable bindings | Isomorphic problem variants, `StrategyDrill` |
| **4. Speed (Fluency)** | Cognitive load chunking and automaticity | `time_taken_ms` vs `target_time_ms` ($> 1.25\times$) | Excessive latency despite correct final response | Timed fluency drill (gated only after accuracy $\ge 80\%$) |

#### The Speed-Accuracy Quadrant Model
Response time and accuracy are mapped into a 4-quadrant diagnostic profile:

```
                  Accuracy (High)
                        ▲
                        │
       Quadrant 2:      │      Quadrant 1:
    SLOW & ACCURATE     │   FAST & ACCURATE
 (Deliberate Competence)│ (Mastery & Fluency)
                        │
 ─── Latency (High) ────┼──── Latency (Low) ───►
                        │
       Quadrant 4:      │      Quadrant 3:
   SLOW & INACCURATE    │  FAST & INACCURATE
 (Fundamental Deficit)  │ (Impulsive / Guessing)
                        │
                        ▼
                  Accuracy (Low)
```

- **Quadrant 1 ($Q_1$): Fast & Accurate** $\to$ Promotes skill state toward `Fluent` / `Mastered`.
- **Quadrant 2 ($Q_2$): Slow & Accurate** $\to$ Validates conceptual acquisition; does NOT penalize in early `Learning` stage; schedules fluency practice later.
- **Quadrant 3 ($Q_3$): Fast & Inaccurate** $\to$ Flags impulsive guessing or intuitive malrules; traps input for deliberate metacognitive reflection.
- **Quadrant 4 ($Q_4$): Slow & Inaccurate** $\to$ Flags high cognitive friction and fundamental skill breakdown; triggers JIT Worked Examples or Prerequisite Review.

---

## 4. Learner UX Principle (Section 4)

### 4.1 Calm, Minimal, Progressive Disclosure
The central UI design philosophy of StudyLab is **Progressive Disclosure** (Sweller, 1988; Nielsen, 1994). The frontend MUST NOT expose the diagnostic engine by default.

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                         PROGRESSIVE DISCLOSURE RULES                             │
├──────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│   DEFAULT VIEW:                                                                  │
│   [Problem Statement] ──► [Modality-Matched Input] ──► [Submit Button]           │
│                                                                                  │
│   ON HINT REQUEST:                                                               │
│   Reveals progressive 3-tier hint panel (Principle ──► Operation ──► Step).      │
│                                                                                  │
│   ON WRONG ANSWER:                                                               │
│   Reveals 4-choice mistake classification footer; hides primary progression.     │
│                                                                                  │
│   ON DIAGNOSTIC MODE COMPLETION:                                                 │
│   Reveals comprehensive 4-tier scorecard and remediation recommendations.        │
│                                                                                  │
└──────────────────────────────────────────────────────────────────────────────────┘
```

### 4.2 Prohibited Visual Anti-Patterns
The frontend reviewer MUST strictly eliminate:
1. **Raw Telemetry Dumps:** No display of raw EMA mastery floats ($\text{mastery} = 0.842$), internal attempt counters, BKT probabilities, or database IDs.
2. **Schema & Engine Leakage:** No display of raw internal schema identifiers (`math.algebra.linear.one_variable_v2`), generator seed hashes, or code filenames.
3. **Stacked Panel Monsters:** No stacking of multiple large boxes (problem box + hint box + CAS validator box + diagnostic chart) on a single screen.
4. **Web-Widget Appearance:** No generic dashboard cards, gradient borders, excessive drop shadows, or floating action buttons that clash with Anki's native aesthetic.

---

## 5. Native Anki Boundary (Section 8)

### 5.1 Host-Guest Architecture & Non-Interference
StudyLab operates as a guest subsystem hosted inside Anki. The boundary is governed by four core invariants:

1. **Non-Interference:** Standard Anki note types (`Basic`, `Cloze`, custom user cards) are completely unaffected. They render with native Anki HTML, CSS, and toolbar behavior with zero overhead.
2. **Reviewer Interception:** Only notes whose note type starts with `"StudyLab Procedural Anchor"` are intercepted by `rslib/src/notetype/render.rs` to render the procedural workspace.
3. **Database Decoupling:** Anki's `collection.anki2` database handle is never touched by StudyLab procedural routines. StudyLab maintains its own SQLite database file: `<collection_path>.procedural` (`procedural.db`).
4. **100-Byte Ephemeral Telemetry Limit:** Rich session telemetry is transmitted via `custom_data["studylab"]` across the IPC bridge, processed atomically by the Rust answering engine into `procedural.db`, and **stripped** before the card record is committed to `collection.anki2` to guarantee compliance with Anki's 100-byte column restriction.

---

### 5.2 State-by-State Control Ownership Matrix

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                    CONTROL VISIBILITY & OWNERSHIP MATRIX                         │
├──────────────────────┬────────────────────────┬──────────────────────────────────┤
│ UI Reviewer State    │ Native Anki Bottom Bar │ StudyLab Procedural Workspace    │
├──────────────────────┼────────────────────────┼──────────────────────────────────┤
│ **`loading`**        │ All buttons HIDDEN     │ Loading spinner                  │
│ **`ready`**          │ All buttons HIDDEN     │ Problem prompt + Input container │
│ **`solving`**        │ All buttons HIDDEN     │ Active input + Submit + Hint CTA │
│ **`submitting`**     │ All buttons HIDDEN     │ Disabled inputs + Evaluating spin│
│ **`mistake_class.`** │ All buttons HIDDEN     │ 4-tier Mistake Strip (`1`–`4`)   │
│ **`feedback`**       │ `Again`, `Hard`,       │ Canonical solution +             │
│                      │ `Good`, `Easy` SHOWN   │ `Next Problem` CTA (or Anki Ease)│
│ **`worked_example`** │ All buttons HIDDEN     │ Solution trace + Ack Button      │
│ **`next`**           │ Delegated to Anki      │ Teardown & next card transition  │
│ **`diagnostic_mock`**│ All buttons HIDDEN     │ Palette + Countdown + Submissions│
│ **`diagnostic_rep.`**│ All buttons HIDDEN     │ Scorecard + Remediation CTAs     │
└──────────────────────┴────────────────────────┴──────────────────────────────────┘
```

**Zero Duplicate Ownership Guarantee:** At no point during active solving or mistake classification are both Anki bottom ease buttons and StudyLab action controls visible simultaneously.

---

## 6. Diagnostic UI Contract (Section 10)

Every metric, signal, and diagnostic field computed by the StudyLab engine is categorized into one of five strict display tiers:

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                         DIAGNOSTIC FIELD CLASSIFICATION                          │
├──────────────────────────────┬───────────────────────────────────────────────────┤
│ Tier Classification          │ UI Visibility & Display Rule                      │
├──────────────────────────────┼───────────────────────────────────────────────────┤
│ **ENGINE ONLY**              │ Computed & stored in `procedural.db`; NEVER shown │
│ **LEARNER OPTIONAL**         │ Hidden by default; available on explicit expand   │
│ **LEARNER AFTER ERROR**      │ Revealed only during post-error reflection        │
│ **LEARNER IN DIAGNOSTIC**    │ Displayed only in comprehensive post-test report  │
│ **NEVER DISPLAY**            │ Internal hashes/pointers; strictly forbidden      │
└──────────────────────────────┴───────────────────────────────────────────────────┘
```

### Comprehensive Field Classification Ledger

| Field / Metric Name | Rust / TS Code Source | Display Classification | Learner Display Format & Rationale |
|---|---|---|---|
| `SkillState.mastery` | `rslib/procedural/src/skills/mod.rs` | **ENGINE ONLY** | Internal continuous EMA float ($\alpha=0.2$). Never shown as a raw float; mapped only to discrete state badges (`Learning`, `Fluent`, `Mastered`) if requested. |
| `historical_independent_count` | `rslib/procedural/src/skills/mod.rs` | **ENGINE ONLY** | Cumulative raw counter. Stored in DB for progression gate evaluation. |
| `historical_hint_count` | `rslib/procedural/src/skills/mod.rs` | **ENGINE ONLY** | Internal scaffolding dependence metric. |
| `sliding_window_attempts` | `rslib/procedural/src/skills/mod.rs` | **ENGINE ONLY** | Internal circular buffer ($N=5$). |
| `solution_graph_fingerprint` | `rslib/procedural/src/skills/signals.rs` | **NEVER DISPLAY** | Cryptographic hash of solution path. Internal deduplication only. |
| `generator_seed` / `instance_id` | `rslib/procedural/src/problems/` | **NEVER DISPLAY** | Deterministic PRNG seed. Irrelevant to learner cognition. |
| `StepwiseValidation.is_valid` | `rslib/procedural/src/problems/steps/` | **LEARNER IMMEDIATE** | Rendered as green checkmark (`✔ Valid`) or red cross (`❌`) on intermediate step line. |
| `StepwiseValidation.is_downstream_consistent` | `rslib/procedural/src/problems/steps/` | **LEARNER AFTER ERROR** | Rendered as yellow badge (`⚠️ Consistent with prior step error`) so the learner knows their algebra was sound despite a prior slip. |
| `ErrorCategory` (`Silly`, `Pattern`, `Concept`, `Prereq`)| `ts/reviewer/components/mistake_footer.ts`| **LEARNER AFTER ERROR** | Rendered as the 4-choice interactive button strip for metacognitive self-classification. |
| `ConceptCheckOption.feedback` | `rslib/procedural/src/remediation/objects.rs` | **LEARNER AFTER ERROR** | Displayed below the chosen distractor option to explain why that specific mental model is flawed. |
| `StrategyDrill.optimality_feedback` | `rslib/procedural/src/remediation/objects.rs` | **LEARNER AFTER ERROR** | Explains why the selected strategy is optimal vs. sub-optimal (e.g. *Energy method requires 1 step vs. 4 steps in Kinematics*). |
| `hints_used` / `hint_level` | `ts/reviewer/procedural.ts` | **LEARNER OPTIONAL** | Rendered as progressive hint text when user clicks `💡 Request Hint (1/3)`. |
| `canonical_steps` | `rslib/procedural/src/reviewer/template.rs` | **LEARNER AFTER SUBMIT**| Full LaTeX derivation revealed in feedback panel after submission or mistake classification. |
| `SpeedAccuracyQuadrant` | `rslib/procedural/src/exam/mock.rs` | **LEARNER IN DIAGNOSTIC** | Displayed in diagnostic mock test summary scorecard as Speed vs. Accuracy breakdown. |
| `4-Tier Hierarchy Scorecard` | `rslib/procedural/src/exam/mock.rs` | **LEARNER IN DIAGNOSTIC** | Subject $\to$ Chapter $\to$ Topic $\to$ Family accuracy tree shown in post-test review screen. |
| `target_anki_card_id` | `rslib/procedural/src/remediation/objects.rs` | **ENGINE ONLY** | Internal reference linking a procedural prerequisite to a declarative flashcard. |

---

## 7. Product Diagnostic Vision (Section 20)

### 7.1 Aggregated Diagnostic Intelligence
StudyLab's long-term vision extends beyond single-item reviews into a comprehensive **Diagnostic Knowledge Mesh**. By continuously updating `SkillState` across all procedural practice and mock exams, the engine builds an aggregated multi-dimensional profile of the student's cognitive strengths and vulnerabilities.

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                     AGGREGATED DIAGNOSTIC INTELLIGENCE MESH                      │
├──────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│   [Practice Attempts] ──┐                                                        │
│                         ├─► [Unified SkillState Matrix] ─► [Weakness Heatmap]    │
│   [Diagnostic Mocks]  ──┘          │                              │              │
│                                    ▼                              ▼              │
│                        [Prerequisite Deficit DAG]     [Adaptive Blueprint Gen]   │
│                                    │                              │              │
│                                    ▼                              ▼              │
│                        [JIT Remediation Queue]        [Targeted Exam Simulation] │
│                                                                                  │
└──────────────────────────────────────────────────────────────────────────────────┘
```

### 7.2 Core Diagnostic Capabilities
1. **Weakness Heatmap Modeling:** Aggregates error categories across the Subject $\to$ Chapter $\to$ Topic hierarchy, pinpointing whether low accuracy is driven by conceptual deficits (requiring foundational study) or calculation slips (requiring fluency drills).
2. **Prerequisite Dependency Tracing:** Traverses directed acyclic graphs (DAGs) of skills. If a student repeatedly fails rotational dynamics problems, the engine traces back and tests prerequisite linear kinematics and moment of inertia skills.
3. **Adaptive Mock Test Blueprinting:** Generates tailored diagnostic exam batteries that over-sample the student's borderline and unverified skills while maintaining authentic competitive exam distributions (e.g., JEE/NEET/GRE blueprints).
4. **Mastery Certification & Transfer Verification:** Gating promotion to `Mastered` only when the student proves multi-context far-transfer across isomorphic problem families under unassisted, timed conditions.

---

## 8. Document Hierarchy Index (Section 24)

To ensure absolute clarity across the engineering and documentation corpus, the 10 canonical StudyLab specifications are mapped below with their authoritative scope:

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                         CANONICAL DOCUMENTATION INDEX                            │
├────────────────────────────────────────┬─────────────────────────────────────────┤
│ Document Path                          │ Authoritative Scope & Primary Questions │
├────────────────────────────────────────┼─────────────────────────────────────────┤
│ `docs/STUDYLAB_PRODUCT_CONTRACT.md`    │ Product North Star, Core Loop, Diagnostic│
│                                        │ Hierarchy, UX Principles, Anki Boundary │
├────────────────────────────────────────┼─────────────────────────────────────────┤
│ `docs/FRONTEND_PRODUCT_SPEC.md`        │ Learning Objects (9), Modalities, Wrong/│
│                                        │ Correct Answer Contracts, Stepwise/MCQ  │
├────────────────────────────────────────┼─────────────────────────────────────────┤
│ `docs/FRONTEND_UI_STATE_SPEC.md`       │ Frontend State Machine, Transitions,    │
│                                        │ Visible/Hidden Controls, Data Exposure  │
├────────────────────────────────────────┼─────────────────────────────────────────┤
│ `docs/FRONTEND_BUTTON_CONTRACT.md`     │ Canonical Button Matrix, Hotkeys, CTA   │
│                                        │ Priorities, Coexistence & Exclusion Rules│
├────────────────────────────────────────┼─────────────────────────────────────────┤
│ `docs/FRONTEND_VISUAL_DESIGN_SPEC.md`  │ Visual Design Contract, Typography,     │
│                                        │ Spacing, Component Styling, Theme Sync  │
├────────────────────────────────────────┼─────────────────────────────────────────┤
│ `docs/APKG_CONTENT_CONTRACT.md`        │ APKG Package Schema, ProceduralPayload, │
│                                        │ Note Types, Field Definitions, Seeds    │
├────────────────────────────────────────┼─────────────────────────────────────────┤
│ `docs/APKG_FRONTEND_CONTRACT.md`       │ APKG ↔ Frontend Data Binding, Anchor   │
│                                        │ Deserialization, Template Injection     │
├────────────────────────────────────────┼─────────────────────────────────────────┤
│ `docs/DATABASE_DATA_CONTRACT.md`       │ SQLite Schema (`procedural.db`), Tables,│
│                                        │ Migrations v1–v5, Data Ownership, WAL   │
├────────────────────────────────────────┼─────────────────────────────────────────┤
│ `docs/FRONTEND_ACCEPTANCE_MATRIX.md`   │ Screen-by-Screen Acceptance Criteria,   │
│                                        │ Usability Tests, "Perfect Window" Rules │
├────────────────────────────────────────┼─────────────────────────────────────────┤
│ `docs/FRONTEND_CURRENT_STATE_GAP_MAP.md│ Screenshot-Grounded Forensic Gap Audit, │
│                                        │ Fixed Defects, Known Differences       │
└────────────────────────────────────────┴─────────────────────────────────────────┘
```

### Authority Rule
If any ambiguity or conflict arises between documents:
1. `STUDYLAB_PRODUCT_CONTRACT.md` is the **supreme authority** on product boundaries, learning loop priorities, and Anki decoupling.
2. `FRONTEND_PRODUCT_SPEC.md` is the **supreme authority** on learning object semantics, interaction modalities, and answering contracts.
3. `FRONTEND_VISUAL_DESIGN_SPEC.md` is the **supreme authority** on visual layout, typography, and styling invariants.
4. `DATABASE_DATA_CONTRACT.md` is the **supreme authority** on persistence schemas and database ownership.
5. `APKG_CONTENT_CONTRACT.md` is the **supreme authority** on package structure and anchor payload formatting.
