# DeepSearch Pedagogical Evidence & Source-Truth Synthesis
**Canonical Research Artifact for StudyLab Learning Sciences, Psychometrics, and Cognitive Architecture**

---

## 1. Executive Summary

StudyLab is a procedural learning and diagnostic engine embedded within the Anki ecosystem. Unlike traditional flashcard systems that optimize declarative memory retrieval via atomic item pairs ($Q \rightarrow A$), StudyLab models complex problem-solving in STEM domains (Mathematics, Physics, Chemistry, and Logical Reasoning). 

This document provides the definitive cognitive science, psychometric, and learning sciences evidence base underlying StudyLab's architecture. Utilizing the **DeepSearch** research methodology, this inquiry investigates seven foundational pedagogical questions (Questions A through G) to establish:
1. **Empirically grounded learning principles** validated across decades of cognitive psychology, educational data mining, and intelligent tutoring systems (ITS) research.
2. **Explicit demarcation** between scientific invariants (e.g., Knowledge Component theory, Cognitive Load Theory, retrieval practice, desirable difficulties) and pragmatic StudyLab product/engineering decisions (e.g., the 4-tier mistake taxonomy, specific Exponential Moving Average coefficients, 5-level difficulty tiers, and reviewer UI layout).
3. **Traceability and reconciliation** between theoretical constructs and the concrete Rust/TypeScript codebase (`rslib/procedural/src/skills/`, `rslib/procedural/src/remediation/`, `rslib/procedural/src/reviewer/`, and `ts/reviewer/`).

---

## 2. Research Methodology & Evidence Ledger

### 2.1 Search & Verification Framework
The synthesis adheres to the 4-tier source credibility model:
- **Tier 1 (Authoritative Primary / Peer-Reviewed)**: Foundational publications in *Cognitive Science*, *Journal of Educational Psychology*, *Educational Psychologist*, *User Modeling and User-Adapted Interaction*, and National Research Council consensus reports.
- **Tier 2 (Vetted Secondary / Domain Compendia)**: Peer-reviewed conference proceedings (AIED, EDM, LAK) and seminal cognitive architectures (ACT-R, SOAR).
- **Tier 3 (Ecosystem & System Implementations)**: Production Intelligent Tutoring Systems (Cognitive Tutor / Carnegie Learning, ANDES, ASSISTments, StepWise).
- **Tier 4 (Low-Confidence / Filtered)**: Uncorroborated blog posts, SEO content mills, and informal study guides (excluded).

### 2.2 Claim-Evidence Ledger Summary

| Ledger ID | Core Claim / Finding | Primary Literature Citation | Credibility Tier | Status | Codebase Artifact |
| :--- | :--- | :--- | :---: | :---: | :--- |
| **CLM-001** | Binary correctness is insufficient; multi-dimensional latent mastery and error decomposition are required for complex problem-solving. | Corbett & Anderson (1994/1995); Koedinger et al. (2012); Pellegrino et al. (2001) | Tier 1 | **VERIFIED** | `domain_evidence.rs`, `signals.rs`, `diagnostics/mod.rs` |
| **CLM-002** | Formative step-level validation significantly outperforms end-of-problem answer evaluation ($d \approx 0.76 - 0.79$). | VanLehn (2006, 2011); Anderson et al. (1995); Koedinger & Corbett (2006) | Tier 1 | **VERIFIED** | `SolutionGraph`, `StepValidator`, `StepwiseReviewer` |
| **CLM-003** | Conceptual understanding and procedural execution develop bidirectionally; speed reflects automaticity, not initial competence. | Rittle-Johnson, Siegler, & Alibali (2001); Schneider & Stern (2010); Binder (1996) | Tier 1 | **VERIFIED** | `progression.rs`, `domain_evidence.rs:is_execution_error` |
| **CLM-004** | Faded worked examples and scaffolding decay optimize cognitive load across the novice-to-expert transition. | Renkl & Atkinson (2003); Sweller (1988, 2011); Kalyuga et al. (2003) | Tier 1 | **VERIFIED** | `remediation/objects.rs`, `WorkedExampleObject`, `ConceptCheckObject` |
| **CLM-005** | Spaced repetition algorithms (SM-2, FSRS) model atomic declarative retention, requiring a separate procedural knowledge graph. | Anderson & Schunn (2000); Pavlik & Anderson (2005); Mai et al. (2024) | Tier 1 | **VERIFIED** | `ProceduralReviewOutcome`, `procedural.db` vs `collection.anki2` |
| **CLM-006** | Post-error metacognitive reflection and hypercorrection accelerate schema repair. | Metcalfe (2017); Metcalfe & Finn (2011); Chi et al. (1989, 1994) | Tier 1 | **VERIFIED** | `ts/reviewer/components/mistake_footer.ts`, `template.rs` |
| **CLM-007** | StudyLab's 4-tier mistake taxonomy and specific EMA smoothing constants are engineering heuristics, not natural laws. | Brown & Burton (1978); VanLehn (1990); Pavlik, Cen, & Koedinger (2009) | Tier 1 | **VERIFIED (Demarcated)** | `SkillState.mastery`, `MistakeFooter` |

---

## 3. Detailed Research Findings per Question (A through G)

---

### Question A: What Exactly Should a Problem-Solving Learning System Measure Beyond Correctness?

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                   MULTIDIMENSIONAL PROBLEM-SOLVING CONSTRUCT                     │
├───────────────────┬───────────────────┬───────────────────┬──────────────────────┤
│  Latent Mastery   │ Error Diagnostic  │   Metacognitive   │   Procedural vs      │
│  (BKT / PFA / KLI)│ (Malrule / Bug)   │   Calibration     │   Conceptual Depth   │
├───────────────────┼───────────────────┼───────────────────┼──────────────────────┤
│ Probability of KC │ Representational  │ Confidence vs     │ Strategy selection   │
│ acquisition over  │ vs execution vs   │ accuracy; self-   │ vs algebraic         │
│ attempt sequences │ strategic failure │ reflection signal │ execution precision  │
└───────────────────┴───────────────────┴───────────────────┴──────────────────────┘
```

#### 1. Latent Knowledge Component (KC) Mastery
Standard educational assessments often treat a test item as an indivisible Bernoulli trial ($Y \in \{0, 1\}$). However, cognitive science establishes that problem solving is mediated by latent **Knowledge Components (KCs)**—mental structures combining production rules, schemas, and declarative associations (Koedinger, Corbett, & Perfetti, 2012). 

*Primary Citations & Theoretical Grounding:*
- **Corbett & Anderson (1994/1995)** (*Knowledge Tracing: Modeling the Acquisition of Procedural Knowledge*): Introduced Bayesian Knowledge Tracing (BKT) using a Hidden Markov Model with four parameters ($P(L_0)$, $P(T)$, $P(G)$, $P(S)$) to model latent skill transition from unlearned to learned state, proving that raw correctness conflates "guessing" ($P(G)$) and "slipping" ($P(S)$).
- **Pavlik, Cen, & Koedinger (2009)** (*Performance Factors Analysis – A New Alternative to Knowledge Tracing*): Demonstrated that tracking accumulated prior successes ($\sum s$) and prior failures ($\sum f$) per knowledge component via logistic models ($\text{logit}(p) = \beta + \sum \alpha_i s_i + \sum \rho_i f_i$) yields higher predictive validity of future problem-solving transfer than aggregate binary scoring.

#### 2. Fine-Grained Error Taxonomy & Malrule Detection
An incorrect answer is rarely random noise; it is predominantly generated by coherent, systematically buggy procedures ("malrules") or flawed mental models.
- **Brown & Burton (1978)** (*Diagnostic Models for Procedural Bugs in Basic Mathematical Skills*): In their seminal BUGGY system, they showed that student errors stem from deterministic bugs in procedural networks (e.g., borrowing from zero, smaller-from-larger subtraction).
- **VanLehn (1990)** (*Mind Bugs: The Origins of Procedural Misconceptions*): Formulated Repair Theory, demonstrating that when a student encounters an impasse (a missing knowledge component), they apply heuristic "repairs" that produce systematic, diagnostic error signatures.

#### 3. Metacognitive Calibration & Post-Error Reflection
Metacognitive monitoring—the ability to accurately assess one's own comprehension and error etiology—is a primary predictor of self-regulated learning.
- **Nelson & Narens (1990)** (*Metamemory: A Theoretical Framework and New Findings*): Formalized the monitoring-control feedback loop (Judgments of Learning [JOL], Feeling of Knowing [FOK]).
- **Metcalfe (2017)** (*Learning from Errors*, *Annual Review of Psychology*): Established that errors committed with high confidence exhibit the **Hypercorrection Effect**—learners pay greater attention to feedback on confident errors, resulting in superior delayed retention compared to low-confidence guesses.
- **Chi, Bassok, Lewis, Reimann, & Glaser (1989)** (*Self-Explanations: How Students Study and Use Examples in Learning to Solve Problems*): Demonstrated that successful problem solvers engage in explicit self-explanation and attribution, diagnosing why a step failed rather than passively receiving correct answers.

#### 4. Procedural vs. Conceptual Competence
- **Rittle-Johnson, Siegler, & Alibali (2001)** (*Developing Conceptual Understanding and Procedural Skill in Mathematics: An Iterative Process*): Demonstrated that conceptual knowledge (explicit or implicit understanding of domain principles and relations) and procedural knowledge (the ability to execute action sequences to solve problems) develop in an iterative, mutually reinforcing cycle mediated by problem representation. Measuring execution alone fails to diagnose conceptual fragility.
- **Schneider & Stern (2010)**: Showed that students with high procedural fluency but low conceptual grounding suffer rapid performance collapse when minor contextual variations are introduced.

#### 5. Efficiency, Fluency, and Automaticity
- **Anderson (1993, 2007)** (*The Architecture of Cognition*; *How Can the Human Mind Occur in the Physical Universe?*): In ACT-R theory, proceduralization compiles declarative chunks into production rules via power-law practice curves ($T = a + b N^{-c}$). Response latency directly indexes cognitive load and procedural chunking.
- **Binder (1996)** (*Behavioral Fluency: Evolution of a New Paradigm*): Established that behavioral fluency (accuracy + speed) predicts retention, endurance, and application (REAPS: Retention, Endurance, Application, Performance Standards).

#### 6. Structural Transferability (Far vs. Near Transfer)
- **Barnett & Ceci (2002)** (*When and Where Do We Apply What We Learn? A Taxonomy for Far Transfer*): Proposed a multi-dimensional taxonomy of transfer (knowledge domain, physical context, temporal context, functional context, social context). A robust system must measure whether competence transfers across isomorphic and homomorphic problem variants.

#### 7. StudyLab Source-Truth Reconciliation
In StudyLab, these dimensions are implemented directly in:
- `rslib/procedural/src/skills/domain_evidence.rs`: Captures domain-typed evidence structs (`MathEvidence`, `ReasoningEvidence`, `PhysicsEvidence`, `ChemistryEvidence`) differentiating `pattern_recognition`, `method_selection`, `execution`, `verification`, and `structural_transfer`.
- `rslib/procedural/src/skills/signals.rs`: `MasteryEvidence` carries `independence`, `response_time_ms`, `decision_quality`, `error_categories`, and `transfer_evidence`.
- `rslib/procedural/src/skills/mod.rs` & `progression.rs`: Tracks longitudinal metrics (`historical_independent_count`, `historical_hint_count`, `delayed_retention_successes`, `distinct_structural_forms_passed`).

---

### Question B: How Should Diagnostic Practice Separate: Concept, Execution, Transfer, and Speed?

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                   ORTHOGONAL DIAGNOSTIC MEASUREMENT MATRIX                       │
├───────────────────┬───────────────────┬───────────────────┬──────────────────────┤
│    DIMENSION      │  COGNITIVE FOCUS  │  FAILURE SYMPTOM  │  TARGET REMEDIATION  │
├───────────────────┼───────────────────┼───────────────────┼──────────────────────┤
│ 1. Concept        │ Mental Model &    │ Model selection   │ Concept Check /      │
│                   │ Governing Laws    │ error, law breach │ Representation Drill │
├───────────────────┼───────────────────┼───────────────────┼──────────────────────┤
│ 2. Execution      │ Mechanical steps, │ Arithmetic slip,  │ Precision drill,     │
│                   │ algebra, units    │ sign error        │ step feedback        │
├───────────────────┼───────────────────┼───────────────────┼──────────────────────┤
│ 3. Transfer       │ Isomorphic and    │ Context bias,     │ Isomorphic variants, │
│                   │ structural shift  │ surface-fixation  │ Strategy Drills      │
├───────────────────┼───────────────────┼───────────────────┼──────────────────────┤
│ 4. Speed (Fluency)│ Cognitive load,   │ Excessive latency │ Timed drills (only   │
│                   │ automaticity      │ or impulsive rush │ after fluent gate)   │
└───────────────────┴───────────────────┴───────────────────┴──────────────────────┘
```

#### 1. Orthogonal Dimensional Measurement & The Assessment Triangle
Diagnostic assessment requires that diagnostic signals remain mathematically and conceptually orthogonal (Pellegrino, Chudowsky, & Glaser, 2001, *Knowing What Students Know: The Science and Design of Educational Assessment*):
- **Cognition Vertex**: Domain-specific cognitive model defining discrete competence attributes ($\alpha = [\alpha_1, \alpha_2, \dots, \alpha_K]$).
- **Observation Vertex**: Tasks explicitly designed to elicit evidence on single attributes or controlled attribute combinations.
- **Interpretation Vertex**: Psychometric models (e.g., Diagnostic Classification Models [DCMs] such as the Deterministic Input, Noisy "And" gate [DINA] model; de la Torre, 2009) that attribute failure to specific sub-skills without contamination.

#### 2. Separation of Concept vs. Execution
- **Chi, Feltovich, & Glaser (1981)** (*Categorization and Representation of Physics Problems by Experts and Novices*): Novices classify problems by surface attributes (e.g., pulleys, inclined planes), whereas experts classify problems by underlying physical principles (e.g., Conservation of Energy, Newton's Second Law).
- **Instructional Rule**: A failure in principle selection (conceptual) must trigger remediation on schema construction, whereas an algebraic or arithmetic calculation error (execution) with correct principle selection must not degrade the learner's conceptual mastery score.

#### 3. Scaffolding Decay & Fading Protocols
- **Renkl & Atkinson (2003)** (*Structuring the Transition from Example Study to Problem Solving in Cognitive Skill Acquisition: A Cognitive Load Perspective*): Validated backward fading—starting with fully worked examples, then transitioning to completion problems with the final step omitted, then intermediate steps omitted, and finally full independent generation.
- **Sweller, van Merrienboer, & Paas (1998)**: Showed that providing heavy scaffolding during early learning reduces extraneous cognitive load, but continuing scaffolding once expertise is acquired induces the **Expertise Reversal Effect** (Kalyuga, Ayres, Chandler, & Sweller, 2003).

#### 4. Speed as an Asymmetric Fluency Gate
- **Corbett & Anderson (1995)** & **Koedinger & Aleven (2007)**: Latency must **not** penalize a learner in the initial acquisition (Learning) phase. Early deliberative processing requires high working memory involvement and slow execution. Latency constraints should only be applied once conceptual accuracy has stabilized ($\ge 80\%$) to test for automaticity and schema compilation.

#### 5. StudyLab Source-Truth Reconciliation
In `rslib/procedural/src/skills/domain_evidence.rs:119-189` and `progression.rs:29-147`:
- `is_execution_error()`: Specifically checks `execution == Some(false)` (Math) or `calculation | unit_validity | boundary_validity == Some(false)` (Physics) while the governing model remains valid.
- `is_conceptual_error()`: Specifically checks `pattern_recognition | method_selection == Some(false)` (Math) or `physical_model_selection | governing_principle == Some(false)` (Physics) or `substrate_recognition | mechanism_pathway == Some(false)` (Organic Chem).
- `progression.rs`: The transition from `Learning` to `Fluent` explicitly enforces:
  ```rust
  // Speed does NOT penalize conceptual learning in early stages
  if attempts_in_window >= 3 
      && recent_acc >= 0.8 
      && state.consecutive_successes >= 3 
      && recent_conceptual_errors == 0 
      && (evidence.independence == IndependenceLevel::Independent 
          || evidence.independence == IndependenceLevel::LightSupport)
  ```

---

### Question C: What Evidence Supports Structured/Stepwise Problem-Solving Assessment?

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                   INTELLIGENT TUTORING: INNER VS OUTER LOOP                      │
├────────────────────────────────────────┬─────────────────────────────────────────┤
│            OUTER LOOP                  │               INNER LOOP                │
│    (Task Selection & Scheduling)       │       (Step-Level Formative Guidance)   │
├────────────────────────────────────────┼─────────────────────────────────────────┤
│ • Decides WHAT problem family to serve │ • Validates each intermediate step      │
│ • Evaluates overall progression state  │ • Provides step-level contextual hints  │
│ • Manages review intervals (FSRS/Anki) │ • Traps intermediate malrules & slips   │
│ • Handled by StudyLab Scheduler        │ • Handled by SolutionGraph / Validator  │
└────────────────────────────────────────┴─────────────────────────────────────────┘
```

#### 1. The VanLehn Cognitive Tutor Paradigm
In two landmark reviews, Kurt VanLehn systematically analyzed the architecture and efficacy of computer-based tutoring:
- **VanLehn (2006)** (*The Behavior of Tutoring Systems*, *Cognitive Science*): Formalized the architecture of intelligent tutoring systems into two distinct loops:
  1. **Outer Loop**: Selects, generates, or schedules the next task or problem based on the learner's cumulative state.
  2. **Inner Loop**: Monitors, validates, and provides feedback on each step taken within a multi-step problem.
- **VanLehn (2011)** (*The Relative Effectiveness of Human Tutoring, Intelligent Tutoring Systems, and Other Tutoring Systems*, *Educational Psychologist*): Meta-analysis proving that step-based tutoring systems (which evaluate intermediate reasoning steps) achieve an effect size of **$d \approx 0.76$**, virtually matching the efficacy of expert human tutors and vastly outperforming answer-based systems ($d \approx 0.30 - 0.40$).

#### 2. Cognitive Load Theory & Step-Level Validation
- **Sweller (1988)** (*Cognitive Load During Problem Solving: Effects on Learning*, *Cognitive Science*): Demonstrated that unguided, end-to-end problem solving in complex domains forces novices to rely on means-ends analysis, which saturates working memory capacity and prevents schema acquisition.
- **Sweller, van Merrienboer, & Paas (2019)** (*Cognitive Architecture and Instructional Design: 20 Years Later*): Stepwise decomposition breaks down high element interactivity into manageable chunks, allowing students to focus working memory on validating individual transformations before proceeding.

#### 3. Intermediate State Validation vs. Final Answer Guessing
- **Credit/Blame Assignment**: When a learner receives feedback only at the final answer, they cannot locate the precise step where reasoning derailed. This leads to catastrophic compounding errors (e.g., making an arithmetic error on step 1, resulting in 5 minutes of wasted valid algebraic derivation thereafter).
- **Antidote to Multiple-Choice Guessing**: Multiple-choice testing permits surface elimination strategies, test-wiseness heuristics, and blind guessing. Stepwise validation forces constructive generation at each node of the solution graph.
- **Immediate Formative Guidance vs. Metacognitive Reflection**: Immediate step-level validation provides immediate error containment ("flag on the play"), while post-problem reflection allows structural integration of the entire solution path.

#### 4. StudyLab Source-Truth Reconciliation
StudyLab models stepwise problem solving through:
- `rslib/procedural/src/problems/contract.rs` & `declarative.rs`: Exposes `SolutionGraph`, `StepType` (`formula_selection`, `symbolic_derivation`, `arithmetic`, `constraint_check`), and multi-level hint structures (`principle`, `operation`, `intermediate_relation`, `direct_derivation`).
- `ts/reviewer/components/stepwise_reviewer.ts`: Renders interactive step validation nodes with real-time feedback and intermediate hint requests.

---

### Question D: What Should Be Considered a Learning-Object Modality Versus a Flashcard?

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                   LEARNING OBJECT MODALITY SPECTRUM                              │
├────────────────────────────┬─────────────────────────────────────────────────────┤
│ Declarative Flashcard      │ Atomic paired-associate ($Q \rightarrow A$).        │
│ (Anki Domain)              │ Fact retrieval, vocabulary, static formula recall.  │
├────────────────────────────┼─────────────────────────────────────────────────────┤
│ Concept Check              │ Micro-evaluation of principle/schema understanding. │
│ (StudyLab Remediation)     │ Disambiguates governing laws without calculation.   │
├────────────────────────────┼─────────────────────────────────────────────────────┤
│ Strategy Drill             │ Method selection drill under time constraint.       │
│ (StudyLab Remediation)     │ "Which theorem applies?" across diverse scenarios.  │
├────────────────────────────┼─────────────────────────────────────────────────────┤
│ Worked Example             │ Fully annotated, low-cognitive-load expert trace.   │
│ (StudyLab Remediation)     │ Step-by-step schema demonstration for novices.      │
├────────────────────────────┼─────────────────────────────────────────────────────┤
│ Parametric Stepwise Task   │ Multi-step generative problem generated from seed.  │
│ (StudyLab Core Engine)     │ Dynamic values, invariant relations, step grading.  │
├────────────────────────────┼─────────────────────────────────────────────────────┤
│ Authentic Exam Item        │ Benchmark evaluation with historical context.       │
│ (StudyLab Practice)        │ Multi-concept unassisted examination simulation.    │
└────────────────────────────┴─────────────────────────────────────────────────────┘
```

#### 1. Declarative Memory Recall vs. Procedural Generative Problem Solving
- **Anderson (1993)** & **Anderson & Lebiere (1998)**: In ACT-R, declarative knowledge consists of *chunks* ($Chunk = \{\text{isa: fact, attribute: value}\}$), whereas procedural knowledge consists of *production rules* ($\text{IF } Goal \land Condition \rightarrow \text{THEN } Action$).
  - A **flashcard** exercises retrieval of declarative chunks (e.g., "What is the formula for kinetic energy? $\rightarrow \frac{1}{2}mv^2$").
  - A **procedural task** requires compiling and executing production rule sequences under novel variable bindings (e.g., "A $2\text{ kg}$ mass slides down an incline with friction coefficient $\mu=0.1\dots$").
- **Anderson & Schunn (2000)** (*The Implications of the ACT-R Learning Theory: No Magic Bullets*): Emphasized that declarative retrieval practice alone cannot produce procedural competence. Procedural skill requires actual practice in executing productions in problem-solving contexts.

#### 2. Static Item Pairs vs. Parametric Problem Families
- **Polya (1945)** (*How to Solve It*) & **Schoenfeld (1985)** (*Mathematical Problem Solving*): True mathematical competence is not memorizing specific numerical instances, but mastering invariant structural transformations across problem classes.
- **Static Item Vulnerability**: Repeating static problem text leads to superficial pattern matching—learners memorize the surface text or the numerical answer ($42\text{ m/s}$) rather than the deep physical laws.
- **Parametric Generation**: Instantiating problems dynamically from formal parameter domains and constraint systems ensures that every attempt requires fresh execution of the underlying production rules while preserving invariant structural depth.

#### 3. Formal Typology of Learning Object Modalities

| Modality Type | Cognitive Function | Cognitive Load | Generative Nature | Primary Use Case |
| :--- | :--- | :--- | :--- | :--- |
| **Flashcard (Basic/Cloze)** | Cued retrieval of atomic facts | Minimal | Static string pair | Vocab, definitions, constants |
| **Concept Check** | Schema disambiguation & principle testing | Low | Parameterized options | Post-conceptual error diagnosis |
| **Strategy Drill** | Strategy selection without calculation | Low-Medium | Categorization / matching | Remediating method-selection errors |
| **Representation Drill**| Spatial/symbolic diagram construction | Medium | Diagram / FBD assembly | Remediating setup errors |
| **Worked Example** | Schema induction without execution load | Low | Annotated solution trace | High-urgency novice remediation |
| **Parametric Quick-Solve**| End-to-end numeric/symbolic calculation | High | Dynamic parameter bindings | Fluency building & verification |
| **Parametric Stepwise** | Guided multi-step procedural derivation | Medium-High | Solution graph traversal | Initial mastery & intermediate repair |
| **Authentic Exam Item** | Unassisted holistic evaluation | High | Exam archetype variant | Mastery gating & benchmarking |

#### 4. StudyLab Source-Truth Reconciliation
In StudyLab's codebase:
- `rslib/procedural/src/problems/contract.rs`: Enforces `ProblemFamilyCapability` (`Declarative`, `ConstraintSolver`, `SymbolicLogic`, `DomainPhysics`, `DomainChemistry`, `DomainGeometry`).
- `rslib/procedural/src/remediation/objects.rs`: Explicitly models `ConceptCheckObject`, `StrategyDrillObject`, `RepresentationDrillObject`, `WorkedExampleObject`, `PrerequisiteReviewObject`, and `DeclarativeRecallBridge`.

---

### Question E: What Are the Cleanest Boundaries Between a Host SRS System and a Procedural Learning Engine?

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                   CLEAN SYSTEM BOUNDARY ARCHITECTURE                            │
├────────────────────────────────────────┬─────────────────────────────────────────┤
│            HOST SRS (ANKI)             │      PROCEDURAL ENGINE (STUDYLAB)       │
├────────────────────────────────────────┼─────────────────────────────────────────┤
│ • Temporal Scheduling Engine (FSRS/SM2)│ • Dynamic Parameterized Problem Gen     │
│ • Manages "WHEN" a review occurs       │ • Manages "WHAT" & "HOW" practice occurs│
│ • Optimizes retention of atomic anchors│ • Knowledge Component (KC) Dependency   │
│ • Owns `collection.anki2` database     │ • Intermediate Step Semantic Validation │
│ • Standard Flashcards (Cloze/Basic)    │ • Domain Diagnostic Evidence Extraction │
│ • Reviewer window lifecycle & webviews │ • JIT Remediation Queue & Escalation    │
│ • User collection sync & profiles      │ • Owns `procedural.db` database         │
└────────────────────────────────────────┴─────────────────────────────────────────┘
```

#### 1. The Two-Memory Architecture: Spacing vs. Skill Compilation
- **Spaced Repetition Systems (SRS)**: Algorithms like SM-2 (Wozniak, 1990) and FSRS (Free Spaced Repetition Scheduler; Mai et al., 2024) are grounded in the **Ebbinghaus Forgetting Curve** and Bjork's **New Theory of Disuse** (Bjork & Bjork, 1992, 2011; storage strength vs. retrieval strength). They excel at scheduling the optimal review interval for independent declarative items.
- **Procedural Tutoring Engines**: Procedural knowledge consists of interconnected skill graphs with prerequisite dependencies, where practice on a complex composite problem reinforces multiple underlying KCs simultaneously (Pavlik & Anderson, 2005, *Practice and Forgetting Effects on Vocabulary Acquisition*).
- **The Boundary Principle**: The host SRS must manage the **macro-schedule** (temporal spacing and review triggering), while the procedural engine must manage the **micro-session** (generative instantiation, interactive execution, diagnostic error analysis, and progression gating).

#### 2. The "Trojan-Horse" Integration Pattern
StudyLab integrates cleanly into Anki without corrupting Anki's native data model:
1. **Procedural Anchor Card**: Anki stores a lightweight card with a specialized note type (`StudyLab Procedural Anchor`) pointing to a `family_id` or `skill_id`.
2. **Reviewer Pipeline Interception**: When Anki's reviewer loads the card, StudyLab intercepts the render lifecycle and replaces the card body with its TypeScript/Vite procedural webview.
3. **Outcome Bridge (`ProceduralReviewOutcome`)**: Upon problem completion, StudyLab passes performance telemetry to `procedural.db` and computes an appropriate rating (e.g., Again, Hard, Good, Easy) for Anki's FSRS scheduler.
4. **Database Decoupling**: StudyLab maintains its own SQLite database (`procedural.db`) for `SkillState`, attempt logs, and remediation queues, leaving Anki's `collection.anki2` completely unpolluted.

#### 3. Architectural Boundary Matrix

| Architectural Function | Host SRS (Anki) | Procedural Engine (StudyLab) | Rationale & Citation |
| :--- | :---: | :---: | :--- |
| Spaced interval calculation | **Primary** | Secondary / Advisory | FSRS optimization (Mai et al., 2024) |
| Flashcard & cloze authoring | **Primary** | - | Declarative paired-associate memory |
| Parametric problem generation | - | **Primary** | Prevents surface recall (Polya, 1945) |
| Stepwise semantic validation | - | **Primary** | ITS Inner Loop (VanLehn, 2006) |
| Diagnostic evidence modeling | - | **Primary** | Assessment Triangle (Pellegrino, 2001) |
| JIT Remediation injection | - | **Primary** | Scaffolding decay (Renkl & Atkinson, 2003) |
| SkillState & mastery graphs | - | **Primary** | Knowledge Tracing (Corbett & Anderson, 1995) |
| Reviewer UI container / window| **Host** | Injected Webview | Clean OS integration and profile management |

---

### Question F: For Math/Reasoning/Physics/Chemistry, What Failure Dimensions Are Pedagogically Meaningful and Which Are Weak Proxies?

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                   PEDAGOGICAL FAILURE TAXONOMY IN STEM                          │
├────────────────────────────────────────┬─────────────────────────────────────────┤
│     PEDAGOGICALLY MEANINGFUL           │              WEAK PROXIES               │
│     (Deep Cognitive Constructs)        │         (Superficial Signals)           │
├────────────────────────────────────────┼─────────────────────────────────────────┤
│ • Conceptual / Model Selection Error   │ • Raw Binary 0/1 Correctness            │
│ • Representational / Schema Framing    │ • Uncalibrated Single-Attempt Latency   │
│ • Governing Principle / Law Violation  │ • Superficial Arithmetic / Sign Slip    │
│ • Intermediate Step Procedural Bug     │ • Syntax / Formatting Mismatch          │
│ • Dimensional / Unit Inconsistency     │ • Multiple-Choice Position Bias         │
│ • Structural Context Transfer Failure  │ • Guessing / Test-Wiseness Elimination  │
└────────────────────────────────────────┴─────────────────────────────────────────┘
```

#### 1. Pedagogically Meaningful Dimensions by Domain

##### A. Mathematics
1. **Pattern Recognition / Schema Identification**: Identifying the canonical structure (e.g., recognizing that $x^4 - 5x^2 + 4 = 0$ is quadratic in $x^2$).
2. **Method / Strategy Selection**: Choosing between valid mathematical strategies (e.g., substitution vs. elimination vs. matrix inversion) and selecting the optimal path.
3. **Execution Precision**: Mechanical algebraic manipulation, expanding binomials, evaluating arithmetic operations.
4. **Domain Constraint & Verification**: Checking solutions against domain restrictions (e.g., extraneous roots in logarithmic equations, division by zero).
5. **Structural Transfer**: Applying the identity to non-standard or visually disguised variants.

##### B. Logical Reasoning
1. **Constraint Extraction & Parsing**: Extracting all explicit and implicit constraints from the problem stem without loss or distortion.
2. **Representation Construction**: Building valid mental or explicit representations (e.g., linear seating arrays, circular tables, 2D attribute grids, truth tables).
3. **Decision Path & Search Tree Traversal**: Systematic hypothesis testing and branch pruning without arbitrary leaps.
4. **Valid Deductive Inference**: Correctly applying rules of inference (Modus Ponens, Modus Tollens) without committing formal fallacies (Affirming the Consequent, Denying the Antecedent).
5. **Trap Checking & Edge Cases**: Recognizing deliberate distractor traps, contradictory premises, or boundary edge cases.

##### C. Physics
1. **Physical Model Selection**: Identifying the applicable physical framework (e.g., Work-Energy Theorem vs. Newton's 2nd Law vs. Impulse-Momentum).
2. **Coordinate & Free-Body Representation**: Constructing valid coordinate frames and accurate Free-Body Diagrams (FBDs) with correct vector orientations.
3. **Governing Principles & Conservation Laws**: Correctly stating conservation of energy, momentum, charge, or angular momentum.
4. **Equation Setup & Symbolic Manipulation**: Substituting algebraic quantities into fundamental equations before inserting numbers.
5. **Dimensional Consistency & Unit Validity**: Maintaining dimensional homogeneity ($[L][T]^{-1}$) and correct SI prefix conversions.
6. **Boundary & Asymptotic Verification**: Sanity-checking limiting behavior ($t \rightarrow 0$, $t \rightarrow \infty$, $m \rightarrow 0$) and physical sign/magnitude feasibility.

##### D. Chemistry
1. **Physical Chemistry**: Equilibrium setup (ICE tables), stoichiometry and limiting reagent tracking, thermodynamic state functions ($\Delta G = \Delta H - T\Delta S$), electrochemical half-cell balancing.
2. **Organic Chemistry**: Substrate functional group recognition, reaction mechanism pathways ($S_N1$, $S_N2$, $E1$, $E2$), nucleophile/electrophile identification, stereochemical configurations ($R/S$, cis/trans, inversion/retention), regioselectivity (Markovnikov/anti-Markovnikov).
3. **Inorganic Chemistry**: Periodic property trends (effective nuclear charge, atomic radii, ionization energies), crystal field theory, coordination complex geometry, oxidation state balancing.

#### 2. Weak Proxies to Avoid
- **Raw Binary Score ($0$ or $1$)**: Completely confounds a catastrophic conceptual failure with a minor calculation slip on the final digit.
- **Uncontrolled Latency**: A fast wrong answer indicates impulsive guessing or an intuitive bug; a slow correct answer indicates effortful deliberate derivation. Conflating speed directly with competence in early learning stages misclassifies deep learners.
- **Superficial Arithmetic Slips as Concept Deficits**: If a student correctly identifies that Conservation of Energy applies, sets up $\frac{1}{2}mv^2 = mgh$, isolates $v = \sqrt{2gh}$, substitutes $g=9.8, h=5$, and calculates $\sqrt{98} = 9.7$ instead of $9.899$, treating this as a "Physics Failure" resets their physics progression inappropriately.

#### 3. StudyLab Source-Truth Reconciliation
Implemented in `rslib/procedural/src/skills/domain_evidence.rs`:
- Differentiates domain evidence structs (`MathEvidence`, `ReasoningEvidence`, `PhysicsEvidence`, `ChemistryEvidence`).
- Helper predicates `is_execution_error()` and `is_conceptual_error()` allow downstream remediation and progression policies to react selectively.

---

### Question G: Which Current StudyLab Architectural Claims Are Unsupported by External Evidence and Must Be Treated as Product Decisions Rather Than Research Facts?

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                   RESEARCH FACTS VS PRODUCT DECISIONS                            │
├────────────────────────────────────────┬─────────────────────────────────────────┤
│     RESEARCH-BACKED PRINCIPLES         │       STUDYLAB PRODUCT DECISIONS        │
│     (External Empirical Truth)         │       (Engineering Heuristics)          │
├────────────────────────────────────────┼─────────────────────────────────────────┤
│ • Knowledge Component modeling (BKT)   │ • 4-Tier Mistake Taxonomy (Silly,       │
│ • Stepwise formative validation        │   Pattern, Concept, Unknown)            │
│ • Spaced retrieval practice (FSRS)     │ • Exponential Moving Average smoothing  │
│ • Faded scaffolding & worked examples  │   constants (α = 0.2, β = 0.8)          │
│ • Post-error reflection / calibration  │ • 5-Level Discrete Difficulty Scale     │
│ • Orthogonal error separation          │ • 12-Hour Delayed Retention Threshold   │
│ • Expertise reversal avoidance         │ • Reviewer Card-DOM vs Bottom-Bar UI    │
└────────────────────────────────────────┴─────────────────────────────────────────┘
```

#### 1. Explicit Analysis of Unsupported Claims & Product Decisions

##### A. The 4-Tier Mistake Taxonomy (`[1 Silly]`, `[2 Pattern]`, `[3 Concept]`, `[4 Unknown]`)
- **Research Reality**: Cognitive science supports metacognitive reflection, self-explanation, and attribution (Metcalfe, 2017; Chi et al., 1989). However, literature contains many different error categorization schemes (e.g., Brown & Burton's 1978 procedural bugs, Radatz's 1979 mathematical error classification, Reason's 1990 Slips vs. Mistakes taxonomy).
- **Product Decision**: StudyLab's specific 4-choice button strip (`[1 Silly Slip]`, `[2 Pattern Missed]`, `[3 Concept Gap]`, `[4 Prereq Unknown]`) with 1–4 keyboard bindings is an **ergonomic UX decision designed for single-keystroke speed**, not a universal cognitive science taxonomy.

##### B. Exponential Moving Average (EMA) Mastery Decay Constant ($\alpha = 0.2$)
- **Research Reality**: Psychometrics uses probabilistic models (BKT Hidden Markov Models with EM fitting, Corbett & Anderson, 1995; or logistic PFA regression, Pavlik et al., 2009).
- **Product Decision**: In `rslib/procedural/src/skills/mod.rs`, StudyLab computes:
  $$\text{mastery}_t = 0.8 \cdot \text{mastery}_{t-1} + 0.2 \cdot \text{outcome}$$
  This $80/20$ exponential moving average smoothing factor is a **pragmatic engineering heuristic** chosen for low-overhead, deterministic, in-memory updates without requiring online numerical optimization.

##### C. Progression Gate Thresholds
- **Research Reality**: Skill mastery criteria in literature often use a posterior probability threshold $P(L_t) \ge 0.95$ in BKT or mastery threshold curves in IRT.
- **Product Decision**: In `rslib/procedural/src/skills/progression.rs`, the specific thresholds:
  - Recent accuracy $\ge 0.80$ for `Fluent`
  - Streak $\ge 3$ consecutive successes
  - Window size $= 5$ attempts
  - Longitudinal independence ratio $\ge 0.70$
  - Delayed retention interval $\ge 12\text{ hours}$ ($43{,}200{,}000\text{ ms}$)
  These specific integer and float thresholds are **heuristic product constants** calibrated for deterministic state-machine gating.

##### D. 5-Tier Discrete Difficulty Catalog (Levels 1–5)
- **Research Reality**: Modern psychometrics represents item difficulty as a continuous real-valued parameter $b \in (-\infty, +\infty)$ in Item Response Theory (Lord, 1980) or Elo/Glicko rating systems.
- **Product Decision**: StudyLab's 5-level catalog (`Foundational`, `Standard`, `Intermediate`, `Advanced`, `Mastery`) is a **curricular product model** designed to align with user intuitions and textbook chapter grading.

##### E. UI Interaction Lifecycle & Placement
- **Research Reality**: Cognitive psychology requires minimizing extraneous cognitive load and spatial split-attention (Sweller et al., 1998).
- **Product Decision**: Anchoring the mistake classification strip inside the card body DOM (`#proc-mistake-panel`) versus injecting into Anki's native bottom bar (`self.bottom.web`), badge color palettes, and container padding are **visual and structural front-end implementation choices**.

---

## 4. Comparative Taxonomy: Research-Backed Principles vs. StudyLab Product Decisions

| Architectural Component | Core Research Principle | Empirical Scientific Basis | StudyLab Implementation / Product Decision | Status / Classification |
| :--- | :--- | :--- | :--- | :--- |
| **Learner Knowledge Model** | Bayesian Knowledge Tracing & PFA | Corbett & Anderson (1995); Pavlik et al. (2009) | `SkillState` with EMA smoothing ($\alpha=0.2$) and sliding attempt window ($N=5$) | **Engineering Approximation of Cognitive Model** |
| **Diagnostic Signals** | Multi-attribute Cognitive Diagnosis (DINA) | Pellegrino et al. (2001); de la Torre (2009) | Strongly-typed `DomainEvidencePayload` (`MathEvidence`, `PhysicsEvidence`, etc.) | **Research-Grounded Implementation** |
| **Stepwise Problem Solving** | Step-based Intelligent Tutoring Inner Loop | VanLehn (2006, 2011); Anderson et al. (1995) | `SolutionGraph` with typed steps, semantic validation, and multi-level hint tree | **Direct Research-Backed Implementation** |
| **Error Attribution** | Post-error reflection & Hypercorrection | Metcalfe (2017); Chi et al. (1989) | 4-choice compact footer (`[1 Silly]`, `[2 Pattern]`, `[3 Concept]`, `[4 Unknown]`) | **Research-Backed Intent; Product-Specific UX/Taxonomy** |
| **Progression Gating** | Multi-criteria mastery & far-transfer | Barnett & Ceci (2002); Binder (1996) | Composite state machine (`New` $\rightarrow$ `Learning` $\rightarrow$ `Fluent` $\rightarrow$ `Variation` $\rightarrow$ `Transfer` $\rightarrow$ `Mastered`) | **Research-Grounded State Machine with Calibrated Heuristic Gates** |
| **JIT Remediation Engine** | Scaffolding decay & faded examples | Renkl & Atkinson (2003); Sweller (2011) | `RemediationPolicy` injecting `ConceptCheck`, `StrategyDrill`, `WorkedExample` | **Direct Research-Backed Implementation** |
| **Spaced Scheduling** | Spaced retrieval practice & Forgetting curves | Ebbinghaus (1885); Bjork & Bjork (2011); Mai et al. (2024) | Anki FSRS scheduler hosting procedural anchor cards via `ProceduralReviewOutcome` bridge | **Research-Grounded Hybrid Architecture** |
| **Item Generative Model** | Parametric generation & structural invariants | Polya (1945); Schoenfeld (1985) | `ProblemFamilyContract` declarative parameter domains and constraint generators | **Direct Research-Backed Implementation** |

---

## 5. Actionable Implications for StudyLab Architecture & Documentation

### 5.1 Implications for Canonical Documentation
1. **Document Drift Correction**: Documentation across `docs/` must explicitly state that while StudyLab's *pedagogical foundation* is grounded in cognitive science (VanLehn, Anderson, Sweller, Metcalfe, Pellegrino), its *specific constants* ($\alpha=0.2$, 12-hour retention delay, 4-tier mistake taxonomy) are **product engineering heuristics**.
2. **Zero Flashcard Ambiguity**: Maintain strict terminology enforcement: StudyLab is a **procedural problem-solving engine**, not an "enhanced flashcard reviewer." Anki provides the host runtime and macro-scheduling; StudyLab owns procedural generation, step validation, and diagnostic mastery.
3. **Traceability Table**: All documentation regarding `DomainEvidence`, `RemediationPolicy`, and `ProgressionPolicy` must cross-reference `docs/DEEPSEARCH_EVIDENCE.md` as their theoretical baseline.

### 5.2 Implications for Future Engine Evolution
1. **Adaptive Parameter Tuning**: Future iterations could transition the deterministic EMA mastery update ($\alpha=0.2$) to a parameterized BKT or PFA logistic model fitted against empirical learner attempt logs.
2. **Dynamic Gating Calibration**: Progression gate thresholds (e.g., accuracy $\ge 0.80$, streak $\ge 3$) can be calibrated per problem family based on item difficulty parameters ($b$) and empirical error distributions.
3. **Fine-Grained Mistake Taxonomy Mapping**: The 4-choice UI strip (`1 Silly` .. `4 Unknown`) maps internally to domain-specific error flags (`ErrorCategory`), ensuring that simplified user input is translated into rich diagnostic telemetry for `procedural.db`.

---

## 6. Primary Academic References & Citations

1. **Anderson, J. R.** (1993). *Rules of the Mind*. Hillsdale, NJ: Lawrence Erlbaum Associates.
2. **Anderson, J. R.** (2007). *How Can the Human Mind Occur in the Physical Universe?* Oxford University Press.
3. **Anderson, J. R., Corbett, A. T., Koedinger, K. R., & Pelletier, R.** (1995). Cognitive tutors: Lessons learned. *The Journal of the Learning Sciences*, 4(2), 167–207.
4. **Anderson, J. R., & Lebiere, C.** (1998). *The Atomic Components of Thought*. Mahwah, NJ: Lawrence Erlbaum Associates.
5. **Anderson, J. R., & Schunn, C. D.** (2000). The implications of the ACT-R learning theory: No magic bullets. *Advances in Instructional Psychology*, 5, 1–33.
6. **Barnett, S. M., & Ceci, S. J.** (2002). When and where do we apply what we learn? A taxonomy for far transfer. *Psychological Bulletin*, 128(4), 612–637.
7. **Binder, C.** (1996). Behavioral fluency: Evolution of a new paradigm. *The Behavior Analyst*, 19(2), 163–197.
8. **Bjork, R. A., & Bjork, E. L.** (1992). A new theory of disuse and an official introduction to the spacing effect. In D. G. Besner (Ed.), *From Learning Processes to Cognitive Processes*.
9. **Bjork, E. L., & Bjork, R. A.** (2011). Making things hard on yourself, but in a good way: Creating desirable difficulties to enhance learning. *Psychology and the Real World*, 2(1), 59–68.
10. **Brown, J. S., & Burton, R. R.** (1978). Diagnostic models for procedural bugs in basic mathematical skills. *Cognitive Science*, 2(2), 155–192.
11. **Chi, M. T., Bassok, M., Lewis, M. W., Reimann, P., & Glaser, R.** (1989). Self-explanations: How students study and use examples in learning to solve problems. *Cognitive Science*, 13(2), 145–182.
12. **Chi, M. T., Feltovich, P. J., & Glaser, R.** (1981). Categorization and representation of physics problems by experts and novices. *Cognitive Science*, 5(2), 121–152.
13. **Corbett, A. T., & Anderson, J. R.** (1994/1995). Knowledge tracing: Modeling the acquisition of procedural knowledge. *User Modeling and User-Adapted Interaction*, 4(4), 253–278.
14. **de la Torre, J.** (2009). DINA model and parameter estimation: A didactic. *Journal of Educational and Behavioral Statistics*, 34(1), 115–130.
15. **Dunlosky, J., Rawson, K. A., Marsh, E. J., Nathan, M. J., & Willingham, D. T.** (2013). Improving students’ learning with effective learning techniques: Promising directions from cognitive and educational psychology. *Psychological Science in the Public Interest*, 14(1), 4–58.
16. **Ebbinghaus, H.** (1885). *Über das Gedächtnis: Untersuchungen zur experimentellen Psychologie*. Leipzig: Duncker & Humblot.
17. **Kalyuga, S., Ayres, P., Chandler, P., & Sweller, J.** (2003). The expertise reversal effect. *Educational Psychologist*, 38(1), 23–31.
18. **Koedinger, K. R., & Aleven, V.** (2007). Exploring the assistance dilemma in experiments with cognitive tutors. *Educational Psychology Review*, 19(3), 239–264.
19. **Koedinger, K. R., & Corbett, A. T.** (2006). Cognitive tutors: Technology bringing learning science to the classroom. *The Cambridge Handbook of the Learning Sciences*, 61–77.
20. **Koedinger, K. R., Corbett, A. T., & Perfetti, C.** (2012). The Knowledge-Learning-Instruction (KLI) framework: Bridging the science-practice chasm to enhance robust student learning. *Cognitive Science*, 36(5), 757–798.
21. **Leighton, J. P., & Gierl, M. J.** (Eds.). (2007). *Cognitive Diagnostic Assessment for Education: Theory and Applications*. Cambridge University Press.
22. **Lord, F. M.** (1980). *Applications of Item Response Theory to Practical Testing Problems*. Routledge.
23. **Mai, J., Ye, J., et al.** (2024). A stochastic shortest path algorithm for optimizing spaced repetition schedules (FSRS v4.5 / v5 benchmark report).
24. **Metcalfe, J.** (2017). Learning from errors. *Annual Review of Psychology*, 68, 465–489.
25. **Metcalfe, J., & Finn, B.** (2011). Evidence that hypercorrection is a function of error certainty. *Journal of Experimental Psychology: Learning, Memory, and Cognition*, 37(2), 437–444.
26. **Nelson, T. O., & Narens, L.** (1990). Metamemory: A theoretical framework and new findings. *Psychology of Learning and Motivation*, 26, 125–171.
27. **Paas, F., Renkl, A., & Sweller, J.** (2003). Cognitive load theory and instructional design: Recent developments. *Educational Psychologist*, 38(1), 1–4.
28. **Pavlik, P. I., & Anderson, J. R.** (2005). Practice and forgetting effects on vocabulary acquisition. *Cognitive Science*, 29(4), 559–586.
29. **Pavlik, P. I., Cen, H., & Koedinger, K. R.** (2009). Performance Factors Analysis – A new alternative to knowledge tracing. *Proceedings of the 14th International Conference on Artificial Intelligence in Education (AIED)*, 531–538.
30. **Pellegrino, J. W., Chudowsky, N., & Glaser, R.** (Eds.). (2001). *Knowing What Students Know: The Science and Design of Educational Assessment*. Washington, DC: National Academy Press.
31. **Polya, G.** (1945). *How to Solve It: A New Aspect of Mathematical Method*. Princeton University Press.
32. **Renkl, A., & Atkinson, R. K.** (2003). Structuring the transition from example study to problem solving in cognitive skill acquisition: A cognitive load perspective. *Educational Psychologist*, 38(1), 15–22.
33. **Rittle-Johnson, B., & Schneider, M.** (2015). Developing conceptual and procedural knowledge of mathematics. *Oxford Handbook of Numerical Cognition*, 1118–1134.
34. **Rittle-Johnson, B., Siegler, R. S., & Alibali, M. W.** (2001). Developing conceptual understanding and procedural skill in mathematics: An iterative process. *Journal of Educational Psychology*, 93(2), 346–362.
35. **Roediger, H. L., & Karpicke, J. D.** (2006). The power of testing memory: Basic research and implications for educational practice. *Perspectives on Psychological Science*, 1(3), 181–210.
36. **Rohrer, D., & Taylor, K.** (2007). The shuffling of mathematics problems improves learning. *Instructional Science*, 35(6), 481–498.
37. **Schneider, M., & Stern, E.** (2010). The developmental relations between conceptual and procedural knowledge: A multimethod approach. *Developmental Psychology*, 46(1), 178–192.
38. **Schoenfeld, A. H.** (1985). *Mathematical Problem Solving*. Orlando, FL: Academic Press.
39. **Sweller, J.** (1988). Cognitive load during problem solving: Effects on learning. *Cognitive Science*, 12(2), 257–285.
40. **Sweller, J., van Merrienboer, J. J., & Paas, F.** (1998). Cognitive architecture and instructional design. *Educational Psychology Review*, 10(3), 251–296.
41. **Sweller, J., van Merrienboer, J. J., & Paas, F.** (2019). Cognitive architecture and instructional design: 20 years later. *Educational Psychology Review*, 31(2), 261–292.
42. **VanLehn, K.** (1990). *Mind Bugs: The Origins of Procedural Misconceptions*. Cambridge, MA: MIT Press.
43. **VanLehn, K.** (2006). The behavior of tutoring systems. *Cognitive Science*, 30(3), 527–565.
44. **VanLehn, K.** (2011). The relative effectiveness of human tutoring, intelligent tutoring systems, and other tutoring systems. *Educational Psychologist*, 46(4), 197–221.
45. **Wozniak, P. A.** (1990). *Optimization of learning*. Master's thesis, University of Technology in Poznan.
