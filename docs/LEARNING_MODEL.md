# Learning Model

The Learner Model is a cognitive model that tracks proficiency across a structured hierarchy and measures multidimensional diagnostic signals based on Domain Evidence.

## Core Hierarchy
1. **Subject** (e.g., Mathematics, Physics, Chemistry, Reasoning)
2. **Chapter** (e.g., Kinematics, Thermodynamics, Coordinate Geometry)
3. **Topic** (e.g., Projectile Motion, 1D Acceleration)
4. **Problem Family** (e.g., `family.physics.kinematics.1d`)
5. **Pattern / Variant** (e.g., `Authentic`, `PyqVariant`, `Variation`, `Transfer`)
6. **Difficulty Level** (1–5)
7. **Practice Attempt** (`PracticeAttempt`)

## Diagnostic Dimensions (Domain Evidence)

The backend (`rslib/procedural/src/skills/domain_evidence.rs`) captures strongly-typed diagnostic signals across all supported STEM disciplines.

### Mathematics (`MathEvidence`)
- `pattern_recognition: Option<bool>`: Identifying the underlying theorem, form, or algebraic identity.
- `method_selection: Option<bool>`: Choosing the appropriate solving strategy (e.g., substitution vs elimination).
- `execution: Option<bool>`: Accurate mechanical calculation and arithmetic manipulation.
- `verification: Option<bool>`: Checking answers against domain constraints and sensible ranges.
- `structural_transfer: Option<bool>`: Successfully applying the method to structurally transformed variants.

### Logical Reasoning (`ReasoningEvidence`)
- `pattern_recognition: Option<bool>`: Recognizing structural relationships in sequences, grids, or syllogisms.
- `representation: Option<bool>`: Valid abstract modeling (e.g., diagramming seating arrangements, truth trees).
- `constraint_extraction: Option<bool>`: Accurately parsing all explicit and implicit problem constraints.
- `decision_path: Option<bool>`: Systematic traversal of candidate solutions without arbitrary leaps.
- `deduction: Option<bool>`: Valid deductive inferences from premises.
- `trap_checking: Option<bool>`: Identifying distractor traps or boundary edge cases.
- `structural_transfer: Option<bool>`: Transfer to isomorphic logical structures.

### Physics (`PhysicsEvidence`)
- `physical_model_selection: Option<bool>`: Identifying governing physical framework (e.g., work-energy vs kinematics).
- `representation: Option<bool>`: Constructing valid free-body diagrams, coordinate frames, or circuit graphs.
- `governing_principle: Option<bool>`: Stating correct fundamental laws (e.g., Newton's 2nd Law, Gauss's Law).
- `equation_setup: Option<bool>`: Accurately substituting given values into algebraic forms.
- `calculation: Option<bool>`: Numerical calculation precision.
- `unit_validity: Option<bool>`: Dimensional consistency and standard SI unit conversions.
- `boundary_validity: Option<bool>`: Physical plausibility at asymptotic/boundary limits.
- `verification: Option<bool>`: Sanity check of result magnitude and sign.
- `transfer: Option<bool>`: Multi-concept or rotated coordinate system transfer.

### Chemistry (`ChemistryEvidence`)
*Divided into Physical, Organic, and Inorganic subclasses:*
- **Physical Chemistry:** `model_setup`, `equation_selection`, `intermediate_quantity` (e.g., limiting reagent, ICE table), `calculation`, `conservation`, `verification`, `transfer`.
- **Organic Chemistry:** `substrate_recognition`, `mechanism_pathway` ($S_N1$, $S_N2$, $E1$, $E2$), `reagent_interpretation`, `product_prediction`, `exception_handling` (e.g., stereochemical inversion, rearrangement), `transfer`.
- **Inorganic Chemistry:** `trend_reasoning` (periodic properties, lattice energies), `exception_handling` (e.g., anomalous ionization energies), `qualitative_reasoning` (coordination complexes, color/magnetic properties), `transfer`.

### Diagnostic Signal Classifiers
- `is_execution_error()`: Calculation, arithmetic, or unit slip where model setup and governing principles were correct.
- `is_conceptual_error()`: Fundamental failure in representation, governing principle, or trend reasoning.
- `is_intermediate_error()`: Error in calculating multi-step intermediate quantities (e.g., mole ratio) with correct initial setup.

---

## SkillState & Unified Learner Model

Learner state is persisted in `skill_states` inside `procedural.db` (`rslib/procedural/src/skills/mod.rs:74-114`).

- **Core Metrics:** `mastery` ($0.0 \dots 1.0$), `confidence` ($0.0 \dots 1.0$), `consecutive_successes`, `consecutive_failures`, `total_attempts`, `successful_attempts`.
- **Smoothing:** Exponential moving average with weight $0.2$:
  $$\text{mastery}_{t} = 0.8 \cdot \text{mastery}_{t-1} + 0.2 \cdot \text{outcome}$$
- **Sliding History Window:** Tracks last 5 attempts (`recent_attempts`), moving latency statistics, and error frequency counts.
- **Longitudinal Tracking:** Tracks `historical_independent_count`, `historical_hint_count`, and `delayed_retention_successes`.

---

## Composite Mastery Progression Gates

Progressions through learner states (`rslib/procedural/src/skills/progression.rs:13-147`) require fulfilling multi-criteria composite gates:

1. **New $\rightarrow$ Learning:** Triggered upon the first recorded attempt (`total_attempts >= 1`).
2. **Learning $\rightarrow$ Fluent:** Requires $\ge 3$ attempts in window, recent accuracy $\ge 0.80$, $\ge 3$ consecutive successes, zero recent conceptual errors, and independent attempt modality.
3. **Fluent $\rightarrow$ Variation:** Requires $\ge 2$ distinct variants attempted, $\ge 2$ consecutive successes, accuracy $\ge 0.80$, and longitudinal independence ratio $\ge 0.50$.
4. **Variation $\rightarrow$ Transfer:** Requires $\ge 2$ distinct structural forms passed, $\ge 2$ consecutive successes, zero recent conceptual errors, and longitudinal independence $\ge 0.60$.
5. **Transfer $\rightarrow$ Mastered (Strict Composite Gate):**
   - **Accuracy:** Recent accuracy $\ge 0.90$ with $\ge 4$ consecutive successes.
   - **Structural Diversity:** $\ge 3$ distinct structural forms passed.
   - **Transfer:** Verified `transfer_evidence == true`.
   - **Longitudinal Independence:** Independence ratio $\ge 0.70$.
   - **Delayed Retention:** $\ge 1$ delayed retention success (delay $\ge 12$ hours / $43{,}200{,}000\text{ ms}$) or $\ge 8$ total robust attempts.
   - **Decision Quality:** Decision quality score $\ge 0.80$ with zero strategy and zero conceptual errors.

---

## Interpretation of Signals

- **Pedagogical Meaning:** Domain Evidence provides a precise cognitive breakdown of why an attempt succeeded or failed.
- **Separation of Concerns:** Domain Evidence feeds `RemediationPolicy` (to choose *what* learning object to inject next) and `ProgressionPolicy` (to evaluate *mastery gates*), while Anki's FSRS scheduler controls *when* the card anchor is due.
