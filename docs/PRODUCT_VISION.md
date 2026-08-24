# StudyLab Product Vision & Cognitive Science Foundation

**Document Version:** 1.0.0 (Canonical Master Specification)  
**Target Repository:** `Anki-maths` (StudyLab Procedural Intelligence Subsystem)  
**Status:** AUTHORITATIVE SPECIFICATION  
**Integrity Mode:** Benchmark Mode (100% Grounded in Executable Code, Tests, and Verified Cognitive Research)  

---

## 1. Executive Summary & Product North Star

### 1.1 The Fundamental Problem: The "Illusion of Competence" in STEM Learning
For decades, Spaced Repetition Systems (SRS)—most notably Anki—have transformed language acquisition, medical terminology memorization, and factual retention. By leveraging the Ebbinghaus Forgetting Curve and algorithms like SM-2 and FSRS, learners achieve long-term retention of atomic declarative facts ($Q \rightarrow A$) with minimal time investment.

However, when applied to quantitative, analytical, and procedural disciplines (**Mathematics**, **Physics**, **Chemistry**, and **Logical Reasoning**), standard flashcards fail catastrophically. 

When a student reviews a static flashcard containing a calculus integration problem or a physics kinematics derivation, they do not execute the underlying cognitive procedures. Instead, they quickly recognize the surface text, recall the final number or formula from memory ($42\text{ m/s}$, $\frac{1}{2}kx^2$), and mark the card "Easy." This creates the **Illusion of Competence** (Bjork & Bjork, 2011; Karpicke & Roediger, 2008): the learner confuses *declarative familiarity with the problem statement* with *generative procedural problem-solving competence*. When presented with a novel variation on an exam, the student experiences complete problem-solving failure.

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                             PRODUCT NORTH STAR                                   │
├──────────────────────────────────────────────────────────────────────────────────┤
│ "Transform spaced practice from passive declarative memorization into deep,      │
│  generative procedural mastery."                                                 │
│                                                                                  │
│ • StudyLab exists to evaluate HOW a learner solves a problem, not just whether   │
│   they remember an answer.                                                       │
│ • It dynamically instantiates parametric problem spaces, validates intermediate  │
│   reasoning steps, diagnoses the root cognitive etiology of errors, and queues   │
│   targeted just-in-time remediation.                                             │
└──────────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Cognitive Science Foundations: The Evidence Base

StudyLab's entire architecture is grounded in foundational learning sciences, cognitive psychology, and intelligent tutoring systems (ITS) research. This section synthesizes the empirical evidence governing StudyLab's design (grounded in `docs/DEEPSEARCH_EVIDENCE.md`).

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                     THE 6 COGNITIVE PILLARS OF STUDYLAB                          │
├───────────────────┬───────────────────┬───────────────────┬──────────────────────┤
│ 1. Two-Memory     │ 2. Cognitive Load │ 3. ITS Inner/Outer│ 4. Desirable         │
│    Architecture   │    & Fading       │    Loops (VanLehn)│    Difficulties      │
│ (ACT-R Production)│ (Sweller/Renkl)   │ (Step Validation) │ (Bjork / Retention)  │
├───────────────────┼───────────────────┼───────────────────┼──────────────────────┤
│ 5. Hypercorrection│ 6. Bidirectional  │ 7. Orthogonal     │ 8. Knowledge         │
│    & Reflection   │    Development    │    Diagnostics    │    Components (KC)   │
│ (Metcalfe / Chi)  │ (Rittle-Johnson)  │ (Pellegrino / DCM)│ (Corbett & Anderson) │
└───────────────────┴───────────────────┴───────────────────┴──────────────────────┘
```

### 2.1 The Two-Memory Architecture (ACT-R Cognitive Theory)
- **Primary Citations:** Anderson (1993, 2007); Anderson & Lebiere (1998); Anderson & Schunn (2000).
- **Cognitive Mechanism:** Human cognitive architecture consists of two distinct memory systems:
  1. **Declarative Memory:** Stores facts and associations as *chunks* ($Chunk = \{\text{isa: fact, attribute: value}\}$). Retrieval is governed by activation strength and recency.
  2. **Procedural Memory:** Stores knowledge as *production rules* ($\text{IF } Goal \land Condition \rightarrow \text{THEN } Action$). Production rules cannot be formed through passive reading or cued retrieval; they can only be compiled and tuned through active execution in goal-directed problem solving.
- **StudyLab Application:** Anki manages declarative memory retrieval (formulas, definitions, constants); StudyLab provides the execution runtime for compiling and tuning production rules across parametric problem spaces.

### 2.2 Cognitive Load Theory & Scaffolding Decay
- **Primary Citations:** Sweller (1988, 2011); Sweller, van Merrienboer, & Paas (1998, 2019); Renkl & Atkinson (2003); Kalyuga, Ayres, Chandler, & Sweller (2003).
- **Cognitive Mechanism:** Complex multi-step problems exhibit high **element interactivity**, saturating working memory capacity. If novices are forced to solve unguided problems, they resort to backward *means-ends analysis*, which consumes all cognitive bandwidth and prevents schema acquisition.
- **Scaffolding & Fading Protocol:** Effective instruction requires **backward fading**:
  $$\text{Worked Example} \longrightarrow \text{Completion Problem} \longrightarrow \text{Stepwise Scaffolding} \longrightarrow \text{Independent Generation}$$
  Continuing heavy scaffolding once expertise is acquired induces the **Expertise Reversal Effect** (Kalyuga et al., 2003).
- **StudyLab Application:** StudyLab implements progressive scaffolding (Worked Examples $\to$ 3-Tier Progressive Hints $\to$ Stepwise Solutions $\to$ Quick-Solve Generation), automatically fading support as learner mastery advances.

### 2.3 Intelligent Tutoring Systems (ITS) Inner vs. Outer Loop
- **Primary Citations:** VanLehn (2006, 2011); Koedinger & Corbett (2006); Anderson, Corbett, Koedinger, & Pelletier (1995).
- **Cognitive Mechanism:** Kurt VanLehn formalized intelligent tutoring into two distinct functional loops:
  1. **Outer Loop:** Selects, sequences, and schedules the next problem or learning activity based on cumulative learner mastery.
  2. **Inner Loop:** Monitors, validates, and provides formative feedback on *each intermediate step* within a multi-step problem.
- **Empirical Effect Size:** Meta-analyses show that step-based tutoring systems (which evaluate intermediate reasoning steps) achieve an effect size of **$d \approx 0.76$** (virtually matching expert 1-on-1 human tutoring), vastly outperforming answer-based systems ($d \approx 0.30 - 0.40$).
- **StudyLab Application:** Anki and StudyLab's `UnifiedPracticeEngine` act as the **Outer Loop** (macro-scheduling and problem selection); StudyLab's `StepValidator` and `SolutionGraph` act as the **Inner Loop** (formative step validation, root equivalence, and downstream consistency).

### 2.4 Metacognitive Calibration & The Hypercorrection Effect
- **Primary Citations:** Metcalfe (2017); Metcalfe & Finn (2011); Chi, Bassok, Lewis, Reimann, & Glaser (1989); Nelson & Narens (1990).
- **Cognitive Mechanism:** High-confidence errors produce the **Hypercorrection Effect**: when learners discover they are wrong on a problem they were confident about, they allocate heightened attention to feedback, resulting in superior delayed retention. Furthermore, self-explanation and attribution (diagnosing *why* a step failed) actively repair broken mental models.
- **StudyLab Application:** The frontend `MistakeFooter` (`ts/reviewer/components/mistake_footer.ts`) traps Space and Enter keys upon error, forcing deliberate metacognitive self-classification (`[1 Silly]`, `[2 Pattern]`, `[3 Concept]`, `[4 Unknown]`) before proceeding.

### 2.5 Orthogonal Diagnostic Dimensions & The Assessment Triangle
- **Primary Citations:** Pellegrino, Chudowsky, & Glaser (2001, *Knowing What Students Know*); Chi, Feltovich, & Glaser (1981); de la Torre (2009).
- **Cognitive Mechanism:** Raw binary correctness ($0$ or $1$) conflates fundamentally different cognitive failures. A student who correctly identifies the governing physical principle (Conservation of Energy) but commits an arithmetic sign slip in step 4 must not have their conceptual mastery demoted.
- **StudyLab Application:** `rslib/procedural/src/skills/domain_evidence.rs` separates `pattern_recognition`, `method_selection`, `execution`, `verification`, and `structural_transfer` into orthogonal evidence dimensions.

### 2.6 Iterative Conceptual and Procedural Development
- **Primary Citations:** Rittle-Johnson, Siegler, & Alibali (2001); Schneider & Stern (2010); Binder (1996).
- **Cognitive Mechanism:** Conceptual understanding and procedural fluency develop iteratively: improvements in conceptual knowledge lead to better procedural execution, and successful procedural practice solidifies and abstracts conceptual schemas. Fluency (speed + accuracy) reflects procedural compilation, not initial competence.
- **StudyLab Application:** Response latency is not penalized during the initial acquisition (`Learning`) stage; speed gates are applied only once conceptual accuracy has stabilized ($\ge 80\%$) in the `Fluent` and `Transfer` stages.

---

## 3. The End-to-End Learner Journey

StudyLab organizes problem solving into a continuous 10-stage cognitive cycle:

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                         THE 10-STAGE LEARNER JOURNEY                             │
├──────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│   1. Prompt Ingestion ──────► 2. Schema Recognition ──────► 3. Mental Modeling   │
│   (Read dynamic stem)         (Identify canonical form)      (Construct FBD/grid)│
│            ▲                                                         │           │
│            │                                                         ▼           │
│   10. Far-Transfer ◄──────── 9. Interleaved ◄──────── 4. Method / Strategy       │
│       Mastery Gate              Practice                 Selection               │
│       (6 composite gates)       (Anti-priming tiers)     (Choose optimal law)    │
│            ▲                                                         │           │
│            │                                                         ▼           │
│   8. Targeted JIT ◄───────── 7. Error Self- ◄──────── 5. Stepwise Execution      │
│      Remediation                Attribution              & Unit Algebra          │
│      (Concept / Worked Ex)      (Trap Space/Enter)       (Semantic validator)    │
│                                                                      │           │
│                                                                      ▼           │
│                                                           6. Sanity Verification │
│                                                              (Physical bounds)   │
└──────────────────────────────────────────────────────────────────────────────────┘
```

1. **Prompt Ingestion:** The learner encounters a freshly seeded parametric problem instance with clean LaTeX/MathJax formatting.
2. **Schema Recognition:** The learner identifies the underlying structural archetype (e.g. recognizing a quadratic in disguise or an Atwood machine).
3. **Mental / Explicit Modeling:** The learner establishes coordinate systems, free-body diagrams, or CSP grids.
4. **Strategic Method Selection:** The learner selects the governing physical principle, algebraic transformation, or logical strategy.
5. **Stepwise Execution:** The learner derives intermediate steps or quick calculations, validated in real time by `StepValidator`.
6. **Sanity Verification:** The learner verifies dimensional consistency ($[L][T]^{-1}$) and physical bounds ($v < c, t > 0$).
7. **Error Self-Attribution:** Upon failure, the learner classifies the root cause via the 4-choice mistake strip (`Silly`, `Pattern`, `Concept`, `Unknown`).
8. **Targeted JIT Remediation:** The system injects micro-remedial objects (Concept Checks, Strategy Drills, Worked Examples, or Prerequisite Reviews).
9. **Interleaved Variation:** The scheduler rotates across schemas to prevent cognitive priming and surface-matching traps.
10. **Far-Transfer Mastery Gating:** Progression to `Mastered` requires satisfying all 6 composite gates (accuracy, diversity, transfer, independence, retention delay, decision quality).

---

## 4. Target User Personas & Real-World Use Cases

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                           TARGET USER PERSONAS                                   │
├───────────────────┬───────────────────┬───────────────────┬──────────────────────┤
│ 1. Competitive    │ 2. University     │ 3. Self-Directed  │ 4. Foundational      │
│    Exam Aspirant  │    STEM Student   │    Professional   │    High-Schooler     │
├───────────────────┼───────────────────┼───────────────────┼──────────────────────┤
│ • Targets: JEE,   │ • Targets: Eng,   │ • Targets: Data   │ • Targets: AP,       │
│   NEET, CAT, GRE  │   Physics, Math   │   Science, Quant  │   A-Levels, IB       │
│ • Needs: Authentic│ • Needs: Multi-   │ • Needs: Logic    │ • Needs: Faded       │
│   PYQ benchmarks, │   step derivation,│   puzzles, stats, │   worked examples,   │
│   speed fluency,  │   dimensional     │   probabilistic   │   concept checks,    │
│   trap avoidance  │   algebra         │   reasoning       │   prerequisite DAGs  │
└───────────────────┴───────────────────┴───────────────────┴──────────────────────┘
```

### Persona 1: The Competitive Exam Aspirant (JEE / NEET / CAT / GRE)
- **Pain Point:** Has memorized hundreds of past-year questions (PYQs) but freezes on exam day when numbers, orientations, or compound concepts change.
- **StudyLab Solution:** Converts authentic PYQs into parametric generator families. The student practices the exact exam blueprint with thousands of seeds, building invariant problem-solving reflexes.

### Persona 2: The University STEM Undergraduate (Physics / Calculus / Chemistry)
- **Pain Point:** Spends 20 minutes on a complex multi-step homework problem only to get it wrong due to a sign slip in step 2, with no credit/blame attribution.
- **StudyLab Solution:** The stepwise reviewer validates each line of algebra, marks downstream derivations as `PartiallyValid` if internally consistent, and flags the exact sign slip at step 2.

### Persona 3: The Quantitative Professional (Data Science / Algorithms / Finance)
- **Pain Point:** Needs to maintain sharp analytical reasoning, combinatorial logic, and rapid mental estimation without wading through bloated web courses.
- **StudyLab Solution:** Fast daily diagnostic practice in CSP seating, combinatorial arrays, and mental numerical approximations integrated seamlessly into their existing Anki review habit.

---

## 5. Comprehensive Comparison: Procedural Engine vs. Flashcard Memory

| Architectural Dimension | Traditional Flashcard System (Anki Host) | StudyLab Procedural Engine |
|---|---|---|
| **Underlying Cognitive Construct** | Declarative memory traces ($Q \to A$ paired associates). | Procedural production rules ($\text{IF } Goal \land Condition \to Action$). |
| **Problem Generation** | Static, immutable text strings authored per note. | Dynamic parametric generation via formal parameter domains and constraint solvers. |
| **Answer Modalities** | Binary self-assessment ("Show Answer" $\to$ flip card). | Modality-matched interactive containers: MCQ, 5D Numerical, Stepwise, Worked Examples. |
| **Evaluation Mechanism** | Subjective self-grading by the user (Did I know this?). | Deterministic semantic validation, algebraic root equivalence, dimensional checking. |
| **Intermediate Steps** | Invisible; only final answer is displayed on back. | Formatively validated step-by-step with downstream consistency tracking (`PartiallyValid`). |
| **Error Diagnosis** | Monolithic binary failure (Again vs Good). | Multi-dimensional taxonomy: Concept, Strategy, Execution, Calculation, Representation, Prerequisite. |
| **Post-Error Action** | Reschedules identical static card for earlier review. | Queues targeted JIT remediation: Concept Check, Strategy Drill, Worked Example, or Prerequisite Review. |
| **Macro Scheduling** | Temporal spaced repetition (FSRS / SM-2). | Defers temporal intervals to Anki; manages micro-session problem instantiation and progression. |
| **Progression Model** | Review intervals and stability factors ($S, D$). | 8 discrete progression states ($New \to Mastered$) governed by 6 composite promotion gates. |
| **Persistence Layer** | `collection.anki2` (`notes`, `cards`, `revlog`). | `<collection>.procedural` (`procedural.db`: 11 tables, 17 indexes, WAL mode). |

---

## 6. Research Invariants vs. Product Engineering Heuristics

To maintain absolute architectural integrity, StudyLab formally demarcates **scientific facts** (universal cognitive principles) from **product decisions** (pragmatic engineering heuristics), as established in `docs/DEEPSEARCH_EVIDENCE.md` (Question G):

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                   RESEARCH FACTS VS PRODUCT DECISIONS                            │
├────────────────────────────────────────┬─────────────────────────────────────────┤
│     RESEARCH-BACKED PRINCIPLES         │       STUDYLAB PRODUCT DECISIONS        │
│     (External Scientific Invariants)   │       (Calibrated Engineering Choices)  │
├────────────────────────────────────────┼─────────────────────────────────────────┤
│ • Knowledge Component modeling (BKT)   │ • 4-Tier Mistake Taxonomy (Silly,       │
│ • Stepwise formative validation        │   Pattern, Concept, Unknown)            │
│ • Spaced retrieval practice (FSRS)     │ • Exponential Moving Average smoothing  │
│ • Faded scaffolding & worked examples  │   constants (α = 0.2, β = 0.8)          │
│ • Post-error reflection / calibration  │ • 5-Level Discrete Difficulty Scale     │
│ • Orthogonal error separation          │ • 12-Hour Delayed Retention Threshold   │
│ • Expertise reversal avoidance         │ • 6-Gate Composite Mastery Policy       │
└────────────────────────────────────────┴─────────────────────────────────────────┘
```

1. **4-Tier Mistake Taxonomy:** Metacognitive attribution is a cognitive necessity (Metcalfe 2017). However, the specific 4-button strip (`[1 Silly]`, `[2 Pattern]`, `[3 Concept]`, `[4 Unknown]`) mapped to keyboard hotkeys `1`–`4` is an **ergonomic UX design** optimized for single-keystroke review speed.
2. **Exponential Moving Average (EMA) Mastery ($\alpha=0.20$):** Continuous skill estimation is required (Corbett & Anderson 1995). The $80/20$ EMA formulation ($\text{Mastery}_t = 0.8\text{M}_{t-1} + 0.2\text{Outcome}$) is a **deterministic in-memory engineering heuristic** that avoids the computational overhead of online expectation-maximization fitting.
3. **5-Level Discrete Difficulty Catalog ($L_1 \dots L_5$):** Item Response Theory models continuous latent item difficulty $b \in (-\infty, +\infty)$. StudyLab's 5 discrete tiers (`Foundational`, `Standard`, `Intermediate`, `Advanced`, `Mastery`) are a **curricular product model** designed for authoring clarity and clear learner progression.
4. **12-Hour Delayed Retention Gate:** Spacing intervals are essential for storage strength (Bjork & Bjork 2011). The $12\text{-hour}$ threshold ($43{,}200{,}000\text{ ms}$) is a **calibrated scheduling constant** enforcing overnight sleep consolidation.

---

## 7. Explicit Non-Goals & Architectural Boundaries

To preserve system clarity, StudyLab explicitly rejects the following non-goals:
- ❌ **Recreating Anki's Flashcard System:** StudyLab will never implement standard basic or cloze flashcard note types. Anki already excels at this.
- ❌ **Forking or Replacing FSRS:** StudyLab will never create a competing temporal scheduler; it translates procedural performance into FSRS-compatible ratings.
- ❌ **Building a Standalone LMS or Heavy Web App:** StudyLab remains a lightweight, native, distraction-free desktop subsystem running inside Anki's fast Qt/C++ runtime.
- ❌ **Hardcoding Static Question Decks in Rust:** Ordinary new topics must never require compiled Rust code; all content is authored declaratively in Python and packaged into standard `.apkg` blueprints.

---

## 8. Long-Term Vision & Strategic Horizons

1. **Universal Declarative STEM Blueprinting:** Expanding the declarative archetype engine so educators can generate thousands of valid, rigorous problem instances from pure JSON schemas without writing code.
2. **Offline WebAssembly Evaluation for Mobile:** Compiling `rslib/procedural/` to WebAssembly (`wasm32-unknown-unknown`) to enable identical zero-latency step validation and procedural practice on AnkiMobile (iOS) and AnkiDroid (Android).
3. **Cross-Collection Prerequisite Graphs:** Enabling decentralized sharing of skill dependency DAGs, allowing learners to import prerequisite mastery profiles across multiple academic subjects.

---

*For technical architecture and boundaries, see [docs/PRODUCT_BOUNDARIES.md](PRODUCT_BOUNDARIES.md), [docs/SYSTEM_ARCHITECTURE.md](SYSTEM_ARCHITECTURE.md), and [docs/ARCHITECTURE_INVARIANTS.md](ARCHITECTURE_INVARIANTS.md).*
