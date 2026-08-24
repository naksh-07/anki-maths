# StudyLab Learning Model & Cognitive Architecture

**Document Version:** 1.0.0 (Canonical)  
**Author:** Learning & Pedagogy Subsystem Architect  
**Date:** 2026-08-25  
**Target Repository:** `Anki-maths` (StudyLab Subsystem)  
**Status:** AUTHORITATIVE SPECIFICATION  
**Integrity Mode:** Benchmark Mode (100% Grounded in Executable Code, Tests, and Cognitive Evidence)  

---

## 1. Executive Summary & Cognitive Philosophy

StudyLab is a procedural learning and diagnostic assessment engine integrated natively within the Anki desktop ecosystem. Unlike standard flashcard systems that optimize declarative paired-associate memory retrieval ($Q \rightarrow A$), StudyLab models complex, generative problem-solving across STEM and analytical disciplines (Mathematics, Physics, Chemistry, and Logical Reasoning).

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                           TWO-MEMORY ARCHITECTURE                               │
├──────────────────────────────────────┬──────────────────────────────────────────┤
│        DECLARATIVE MEMORY (ANKI)     │       PROCEDURAL MEMORY (STUDYLAB)       │
├──────────────────────────────────────┼──────────────────────────────────────────┤
│ • Memory Chunks ($Q \to A$)          │ • Production Rules ($\text{IF-THEN}$)    │
│ • Static Fact / Vocabulary Recall    │ • Dynamic Multi-Step Derivations         │
│ • Ebbinghaus Forgetting Curve        │ • Knowledge Component (KC) Compilation   │
│ • Spaced Intervals (FSRS / SM-2)     │ • Interactive Modalities & Step Cas      │
│ • Stores in `collection.anki2`       │ • Stores in `<collection>.procedural`    │
└──────────────────────────────────────┴──────────────────────────────────────────┘
```

### Core Invariant
> **"StudyLab is not a flashcard system; it is a procedural problem-solving engine hosted inside Anki."**
> 
> Grounded in the **Two-Memory Architecture** (Anderson & Lebiere 1998 ACT-R; Anderson & Schunn 2000), declarative retrieval practice alone cannot produce procedural competence. Spaced repetition systems (Anki with FSRS; Mai et al. 2024) manage the temporal macro-schedule ("when to review"); StudyLab manages the cognitive micro-session ("how to practice, validate, diagnose, and remediate").

---

## 2. Cognitive Taxonomy & Knowledge Component Modeling

Standard educational tests evaluate performance as an indivisible Bernoulli trial ($Y \in \{0, 1\}$). In contrast, StudyLab treats problem-solving as the execution of interconnected **Knowledge Components (KCs)**—mental structures combining production rules, declarative schemas, and constraint-satisfaction heuristics (Koedinger, Corbett, & Perfetti 2012).

### 2.1 The Multidimensional Problem-Solving Construct
StudyLab measures six orthogonal dimensions of cognitive competence:
1. **Latent Knowledge Component Mastery:** The cumulative probability that a student has acquired the underlying production rule (Corbett & Anderson 1995 BKT; Pavlik, Cen, & Koedinger 2009 PFA).
2. **Taxonomic Error Etiology:** Whether an error is an ungrounded conceptual gap, a flawed strategy, a mechanical calculation slip, or a reading trap (Brown & Burton 1978; VanLehn 1990).
3. **Metacognitive Self-Attribution:** Post-error self-calibration and reflection, activating the **Hypercorrection Effect** (Metcalfe 2017; Chi et al. 1989).
4. **Procedural vs. Conceptual Decoupling:** Ensuring calculation or unit slips do not inappropriately demote conceptual mastery (Rittle-Johnson, Siegler, & Alibali 2001; Schneider & Stern 2010).
5. **Behavioral Fluency & Automaticity:** Response latency and cognitive load tracking (Binder 1996; Anderson 1993).
6. **Structural Transferability:** Verification that competence transfers across isomorphic, structural, and novel problem variants (Barnett & Ceci 2002; Polya 1945).

---

## 3. Four-Tier Curricular & Diagnostic Hierarchy

StudyLab organizes all academic content and diagnostic signals across a strict **4-Tier Curricular Hierarchy**:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      4-TIER CURRICULAR HIERARCHY                        │
├─────────┬──────────────────┬────────────────────────────────────────────┤
│ Tier    │ Level            │ Concrete Example                           │
├─────────┼──────────────────┼────────────────────────────────────────────┤
│ Tier 1  │ **Subject**      │ Mathematics / Physics / Chemistry / Logic  │
│ Tier 2  │ **Chapter**      │ Arithmetic / Mechanics / Physical Chem     │
│ Tier 3  │ **Topic**        │ Percentages / 1D Kinematics / ICE Tables   │
│ Tier 4  │ **ProblemFamily**│ `family.math.percentage.successive`        │
└─────────┴──────────────────┴────────────────────────────────────────────┘
```

1. **Tier 1 — Subject (`Domain`):** Top-level academic discipline (`Mathematics`, `Physics`, `Chemistry`, `Reasoning`, or `Custom`). Enforces discipline-specific validation rules, units, and sanity checks.
2. **Tier 2 — Chapter (`ChapterPracticeProfile`):** Curricular grouping (e.g. *Kinematics*, *Thermodynamics*, *Percentage & Ratio*). Configures chapter capabilities, prerequisites, decision points, and exam weights.
3. **Tier 3 — Topic:** Specific conceptual unit (e.g. *Successive Percentage Change*, *1D Kinematic Equations*, *Buffer Solutions*). Acts as the primary grouping node in diagnostic reporting.
4. **Tier 4 — Problem Family (`ProblemFamilyContract` / `ProblemFamilyId`):** The atomic generative engine. Governs parameter domains, algebraic constraints, solution graphs, and step-level validation.

---

## 4. Multi-Domain Pedagogical Evidence Taxonomy

StudyLab captures strongly-typed diagnostic signals across all supported STEM disciplines (`rslib/procedural/src/skills/domain_evidence.rs`). Every practice attempt produces a `DomainEvidencePayload`:

```rust
// rslib/procedural/src/skills/domain_evidence.rs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "domain_kind", content = "evidence")]
pub enum DomainEvidencePayload {
    Math(MathEvidence),
    Reasoning(ReasoningEvidence),
    Physics(PhysicsEvidence),
    Chemistry(ChemistryEvidence),
}
```

### 4.1 Mathematics Evidence (`MathEvidence`)
| Evidence Field | Type | Pedagogical Meaning |
|---|---|---|
| `pattern_recognition` | `Option<bool>` | Identifying the underlying algebraic identity, theorem, or canonical form. |
| `method_selection` | `Option<bool>` | Selecting the optimal solving strategy (e.g. substitution vs elimination). |
| `execution` | `Option<bool>` | Mechanical arithmetic, expansion, and symbolic manipulation precision. |
| `verification` | `Option<bool>` | Verifying answers against domain bounds (e.g. extraneous roots, division by zero). |
| `structural_transfer` | `Option<bool>` | Applying the mathematical method to structurally disguised variants. |

### 4.2 Logical Reasoning Evidence (`ReasoningEvidence`)
| Evidence Field | Type | Pedagogical Meaning |
|---|---|---|
| `pattern_recognition` | `Option<bool>` | Recognizing structural patterns in series, grids, or syllogisms. |
| `representation` | `Option<bool>` | Constructing valid mental models (e.g. linear seating arrays, 2D attribute grids). |
| `constraint_extraction`| `Option<bool>` | Accurately extracting all explicit and implicit problem constraints without loss. |
| `decision_path` | `Option<bool>` | Systematic search tree traversal without arbitrary deductive leaps. |
| `deduction` | `Option<bool>` | Applying valid rules of inference (Modus Ponens/Tollens) without formal fallacies. |
| `trap_checking` | `Option<bool>` | Detecting distractor traps, contradictory premises, or edge cases. |
| `structural_transfer` | `Option<bool>` | Transferring logical reasoning to isomorphic relational structures. |

### 4.3 Physics Evidence (`PhysicsEvidence`)
| Evidence Field | Type | Pedagogical Meaning |
|---|---|---|
| `physical_model_selection` | `Option<bool>` | Identifying applicable physical framework (e.g. Work-Energy vs Newton's Laws). |
| `representation` | `Option<bool>` | Constructing valid coordinate frames and Free-Body Diagrams (FBDs). |
| `governing_principle` | `Option<bool>` | Correctly stating fundamental laws (e.g. Conservation of Momentum, Energy). |
| `equation_setup` | `Option<bool>` | Substituting given variables into symbolic equations before computation. |
| `calculation` | `Option<bool>` | Arithmetic and numerical computation precision. |
| `unit_validity` | `Option<bool>` | Dimensional consistency ($[L][T]^{-1}$) and SI unit conversions ($5/18$ factor). |
| `boundary_validity` | `Option<bool>` | Verifying physical feasibility ($v \le c, T \ge 0\text{ K}, t \ge 0$). |
| `verification` | `Option<bool>` | Sanity-checking magnitude and sign of output. |
| `transfer` | `Option<bool>` | Solving rotated coordinate frames or multi-concept combined problems. |

### 4.4 Chemistry Evidence (`ChemistryEvidence`)
Chemistry evidence is partitioned into three specialized sub-disciplines:
- **Physical Chemistry (`PhysicalChemistryEvidence`):**
  - `model_setup`: Setting up equilibrium, thermodynamic state functions, or electrochemical cells.
  - `equation_selection`: Choosing rate laws, Nernst equation, Henderson-Hasselbalch, or ICE tables.
  - `intermediate_quantity`: Computing intermediate mole ratios, limiting reagents, or reaction quotients ($Q$).
  - `calculation`: Precision in arithmetic and logarithmic/exponential calculations.
  - `conservation`: Enforcing mass balance, elemental conservation, and charge conservation.
  - `verification`: Verifying non-negative concentrations ($[X] \ge 0$) and physical feasibility.
  - `transfer`: Transferring principles to non-standard multi-phase systems.
- **Organic Chemistry (`OrganicChemistryEvidence`):**
  - `substrate_recognition`: Identifying functional groups and nucleophilic/electrophilic centers.
  - `mechanism_pathway`: Selecting reaction pathway ($S_N1$, $S_N2$, $E1$, $E2$, EAS).
  - `reagent_interpretation`: Identifying reagent role (oxidizing, reducing, nucleophile, base).
  - `product_prediction`: Predicting major vs minor products and regioselectivity (Markovnikov).
  - `exception_handling`: Stereochemical inversion/retention ($R/S$), carbocation rearrangement.
  - `transfer`: Applying reactions to complex polyfunctional molecules.
- **Inorganic Chemistry (`InorganicChemistryEvidence`):**
  - `trend_reasoning`: Predicting periodic trends (ionization energy, electron affinity, radii).
  - `exception_handling`: Explaining anomalous configurations ($d^4/d^9$, inert pair effect).
  - `qualitative_reasoning`: Coordination geometry, crystal field splitting ($\Delta_o$), magnetic spin.
  - `transfer`: Generalizing bonding models across unfamiliar transition metal complexes.

### 4.5 Diagnostic Signal Classifiers
Helper predicates in `rslib/procedural/src/skills/domain_evidence.rs:119-189` classify errors for targeted remediation:
- **`is_execution_error()`:** Returns `true` when high-level representation and governing principles were correct, but mechanical calculation, sign, or unit errors occurred.
- **`is_conceptual_error()`:** Returns `true` when model selection, governing principle, substrate recognition, or trend reasoning failed.
- **`is_intermediate_error()`:** Returns `true` when intermediate multi-step quantities (e.g. mole ratios, reaction quotients) failed despite correct setup.

---

## 5. Learner State Representation (`SkillState`)

Learner competency is modeled in `rslib/procedural/src/skills/mod.rs` and persisted in the `skill_states` SQLite table inside `<collection>.procedural`.

```rust
// rslib/procedural/src/skills/mod.rs
pub struct SkillState {
    pub skill_id: SkillId,
    pub mastery: f64,
    pub confidence: f64,
    pub total_attempts: u32,
    pub successful_attempts: u32,
    pub consecutive_successes: u32,
    pub consecutive_failures: u32,
    pub last_practiced_at: Option<DateTime<Utc>>,
    pub recent_attempts: VecDeque<AttemptOutcome>,
    pub latency_stats: MovingLatencyStats,
    pub error_counts: ErrorFrequencyCounts,
    pub variant_history: HashMap<String, VariantPerformance>,
    pub historical_independent_count: u32,
    pub historical_hint_count: u32,
    pub delayed_retention_successes: u32,
    pub distinct_structural_forms_passed: HashSet<String>,
    pub progression_state: PracticeProgressionState,
}
```

### 5.1 Mastery Accumulation & Exponential Smoothing
Mastery updates upon every attempt using an Exponential Moving Average (EMA) formulation:

$$\text{Mastery}_{t} = (1 - \alpha) \cdot \text{Mastery}_{t-1} + \alpha \cdot \text{Outcome}$$

where:
- $\alpha = 0.20$ (smoothing weight; represents an 80/20 balance between prior skill history and latest performance).
- $\text{Outcome} = 1.0$ for successful attempts; $\text{Outcome} = 0.0$ for failed attempts.

### 5.2 Estimation Confidence
Confidence reflects sample size maturity, scaling linearly to saturation at 10 attempts:

$$\text{Confidence} = \min\left(\frac{\text{Total Attempts}}{10.0}, 1.0\right)$$

### 5.3 Longitudinal Metrics
- **Sliding History Window ($N=5$):** `recent_attempts` buffers the last 5 attempts to compute short-term moving accuracy ($\text{recent\_accuracy}$).
- **Moving Latency Statistics (`MovingLatencyStats`):** Tracks running mean latency ($\bar{T}$), min latency, max latency, and moving variance ($\sigma^2$).
- **Longitudinal Independence Ratio:**
  $$\text{Independence Ratio} = \frac{\text{Historical Independent Count}}{\text{Total Lifetime Attempts}}$$
- **Delayed Retention Counter:** Incremented only when a problem is solved independently after a delay separation of $\ge 12\text{ hours}$ ($43{,}200{,}000\text{ ms}$) from previous practice.

---

## 6. Progression State Machine & 6-Gate Mastery Policy

StudyLab models progression across **8 Progression States** (`rslib/procedural/src/skills/signals.rs`):

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                           PROGRESSION STATE MACHINE                             │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                 │
│   [0: New] ──(1 attempt)──► [1: Learning]                                       │
│                                │                                                │
│          (Acc≥80%, Streak≥3,   │                                                │
│           Indep, 0 Concept)    ▼                                                │
│                            [2: Fluent]                                          │
│                                │                                                │
│          (Distinct Variants≥2, │                                                │
│           Acc≥80%, Streak≥2)   ▼                                                │
│                            [3: Variation]                                       │
│                                │                                                │
│          (Structural Forms≥2,  │                                                │
│           Acc≥80%, 0 Concept)  ▼                                                │
│                            [4: Transfer]                                        │
│                                │                                                │
│          (6-Gate Composite     │                                                │
│           Mastery Policy)      ▼                                                │
│                            [5: Mastered] ──► [6: Retired] / [7: Hibernating]    │
│                                                                                 │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### 6.1 State Definitions
- **`New` (0):** Unpracticed skill node (0 recorded attempts).
- **`Learning` (1):** Initial acquisition phase. Speed does **not** penalize the learner; focus is on schema formation and principle discovery.
- **`Fluent` (2):** Basic procedural execution stabilized. Speed constraints begin gating automaticity.
- **`Variation` (3):** Practicing surface and parameter variations (isomorphic forms, varied numerical ranges).
- **`Transfer` (4):** Advanced stage testing structural transformation, rotated contexts, and multi-concept combinations.
- **`Mastered` (5):** Robust, high-fluency, transfer-verified procedural mastery.
- **`Retired` (6):** Archived skill past active curricular need.
- **`Hibernating` (7):** Suspended skill pending prerequisite restoration.

### 6.2 Demotion Rules
Progression is non-monotonic; state decay occurs upon detected conceptual collapse:
- **From `Mastered` to `Transfer`:** 3 consecutive failures or recent sliding accuracy $< 50\%$.
- **From `Transfer` to `Variation`:** 3 consecutive failures or severe conceptual error.
- **From `Variation` to `Fluent`:** 3 consecutive failures.
- **From `Fluent` to `Learning`:** Recent accuracy drops below $60\%$ or recurrence of fundamental concept errors.

### 6.3 The 6-Gate Mastery Promotion Policy (`Transfer` $\to$ `Mastered`)
Advancement to `Mastered` requires simultaneously satisfying all **6 Composite Gates** (`rslib/procedural/src/skills/progression.rs:95-147`):

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    6-GATE COMPOSITE MASTERY POLICY                      │
├───────┬──────────────────────────┬──────────────────────────────────────┤
│ Gate  │ Criterion                │ Threshold Required                   │
├───────┼──────────────────────────┼──────────────────────────────────────┤
│ **1** │ Accuracy & Streak        │ Recent accuracy $\ge 90\%$ AND       │
│       │                          │ Consecutive successes $\ge 4$        │
│ **2** │ Structural Diversity     │ $\ge 3$ distinct structural/transfer │
│       │                          │ forms passed independently           │
│ **3** │ Transfer Verification    │ Active `transfer_evidence == true`   │
│       │                          │ on novel context problem             │
│ **4** │ Longitudinal Independence│ Lifetime unassisted solve ratio      │
│       │                          │ $\ge 70\%$                           │
│ **5** │ Delayed Retention        │ $\ge 1$ delayed retention success    │
│       │                          │ ($\ge 12\text{h}$ delay) OR $\ge 8$ attempts │
│ **6** │ Cognitive Decision Score │ Strategic decision quality $\ge 80\%$│
│       │                          │ with 0 recent strategy errors        │
└───────┴──────────────────────────┴──────────────────────────────────────┘
```

---

## 7. Four Speed Quadrants & Fluency Classification

Upon answer submission, `computeSpeedQuadrant(isCorrect, timeTakenMs, targetTimeMs)` in `ts/reviewer/procedural.ts:704-735` evaluates response latency against the problem's calibrated target time budget ($T_{target}$):

```
                       SPEED (Time vs Target Time)
                     Fast (T ≤ T_target)     Slow (T > T_target)
                   ┌──────────────────────┬──────────────────────┐
    Correct        │  ⚡ FLUENCY STRENGTH │ ⏱ SPEED OPPORTUNITY  │
                   │  (Accurate & Fast)   │ (Accurate but Slow)  │
ACCURACY           ├──────────────────────┼──────────────────────┤
    Incorrect      │  ⚠️ STRATEGY TRAP    │ 💡 CONCEPT / SETUP   │
                   │ (Fast but Incorrect) │  (Slow & Incorrect)  │
                   └──────────────────────┴──────────────────────┘
```

### 7.1 Quadrant Descriptions & Pedagogical Responses
1. **`fluency_strength` (⚡ Accurate & Fast):**
   - Condition: `isCorrect == true && timeTakenMs <= targetTimeMs`.
   - Pedagogical Meaning: Compiled production rule; high automaticity; minimal working memory load.
   - Response: Advance difficulty level ($L_k \to L_{k+1}$) or introduce structural variations.
2. **`speed_opportunity` (⏱ Accurate but Slow):**
   - Condition: `isCorrect == true && timeTakenMs > targetTimeMs`.
   - Pedagogical Meaning: Deliberate, effortful derivation; correct schema but uncompiled execution.
   - Response: Maintain current difficulty; schedule timed fluency drills without increasing complexity.
3. **`strategy_trap` (⚠️ Fast but Incorrect):**
   - Condition: `isCorrect == false && timeTakenMs <= targetTimeMs`.
   - Pedagogical Meaning: Impulsive execution, surface pattern matching, or falling for distractor traps.
   - Response: Trigger `StrategyDrill` or `RepresentationDrill` to enforce pre-computation reflection.
4. **`concept_setup` (💡 Slow & Incorrect):**
   - Condition: `isCorrect == false && timeTakenMs > targetTimeMs`.
   - Pedagogical Meaning: Severe cognitive impasse; working memory overload; missing prerequisite schema.
   - Response: Trigger immediate `ConceptCheck`, `WorkedExample`, or prerequisite drill-down.

### 7.2 Speed as an Asymmetric Gate
Grounded in Cognitive Load Theory (Sweller 1988; Corbett & Anderson 1995), speed is an **asymmetric gate**:
- In `New` and `Learning` states, slow latency **never** penalizes the learner. High deliberative latency is expected during initial schema acquisition.
- Latency constraints are applied **only** once accuracy has stabilized ($\ge 80\%$) to test for automaticity in the `Fluent` and `Variation` states.

---

## 8. Four-Tier Mistake Reflection Taxonomy

When an attempt is incorrect, StudyLab immediately transitions to `mistake_classification` (`ts/reviewer/components/mistake_footer.ts`), rendering a compact 4-button footer strip in the reading flow:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      MISTAKE CLASSIFICATION STRIP                       │
├────────┬────────────────────────────────┬───────────────────────────────┤
│ Key    │ Button Label                   │ Internal Category & Meaning   │
├────────┼────────────────────────────────┼───────────────────────────────┤
│ **1**  │ `[1 Silly Slip]`               │ `silly_mistake`               │
│        │                                │ Arithmetic / sign / unit slip │
├────────┼────────────────────────────────┼───────────────────────────────┤
│ **2**  │ `[2 Pattern Missed]`           │ `pattern_not_recognized`      │
│        │                                │ Failed to recognize schema    │
├────────┼────────────────────────────────┼───────────────────────────────┤
│ **3**  │ `[3 Concept Gap]`              │ `formula_or_concept_misapplied`│
│        │                                │ Wrong formula / law breached  │
├────────┼────────────────────────────────┼───────────────────────────────┤
│ **4**  │ `[4 Prereq Unknown]`           │ `concept_not_known`           │
│        │                                │ Missing foundational knowledge│
└────────┴────────────────────────────────┴───────────────────────────────┘
```

### 8.1 Reflection Gate & Space/Enter Trapping
To prevent mindless skipping and force metacognitive self-explanation:
1. In `mistake_classification` state, Space and Enter key events are strictly trapped (`e.preventDefault()`, `e.stopPropagation()`).
2. The learner **must** explicitly classify their mistake by pressing `1`, `2`, `3`, or `4` (or clicking a button).
3. Grounded in the **Hypercorrection Effect** (Metcalfe 2017), requiring immediate post-error self-attribution substantially increases delayed retention and accelerates schema repair.

### 8.2 Research Fact vs. Engineering Heuristic
- **Research Fact:** Post-error metacognitive reflection, attribution, and hypercorrection enhance schema repair (Metcalfe 2017; Chi et al. 1989).
- **Engineering Decision:** The 4-choice button strip (`[1 Silly]`, `[2 Pattern]`, `[3 Concept]`, `[4 Unknown]`) with 1–4 keyboard shortcuts is an **ergonomic UX design** optimized for rapid single-keystroke interaction within desktop Anki.

---

## 9. Verification & Codebase Traceability Matrix

| Concept / Formula | Source Code Reference | Test Evidence Suite |
|---|---|---|
| Domain Evidence Structs | `rslib/procedural/src/skills/domain_evidence.rs:1-189` | `rslib/procedural/tests/phase28_domain_evidence_contract.rs` (7 tests) |
| EMA Mastery ($\alpha=0.2$) | `rslib/procedural/src/skills/mod.rs:74-114` | `rslib/procedural/src/skills/tests` |
| 6 Composite Mastery Gates | `rslib/procedural/src/skills/progression.rs:95-147` | `rslib/procedural/tests/desktop_validation_master_suite.rs` (Section 10-14) |
| Speed Quadrant Computation | `ts/reviewer/procedural.ts:704-735` | `ts/reviewer/procedural.test.ts` (27 tests) |
| Mistake Footer Trapping | `ts/reviewer/components/mistake_footer.ts:25-120` | `ts/tests/e2e/procedural-smoke.spec.ts` |
| Atomic State Persistence | `rslib/procedural/src/storage/store.rs:150-220` | `rslib/procedural/src/storage/tests` (Migrations v1-v5) |
