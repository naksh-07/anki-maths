> **NOTE**: The canonical source of truth for all frontend UI/UX, modalities, state transitions, and interaction invariants is now defined in [FRONTEND_PRODUCT_SPEC.md](./FRONTEND_PRODUCT_SPEC.md). Any frontend-specific implementation details in this document are superseded by the new specification.

# StudyLab Learning Objects & Interactive Modality Contracts

**Document Version:** 1.0.0 (Canonical)  
**Author:** Learning Objects & Frontend Systems Architect  
**Date:** 2026-08-25  
**Target Repository:** `Anki-maths` (StudyLab Subsystem)  
**Status:** AUTHORITATIVE SPECIFICATION  
**Integrity Mode:** Benchmark Mode (100% Grounded in Executable Code, Passing Tests, and Verified UI Screenshots)  

---

## 1. Executive Summary & Modality Architecture

In standard flashcard applications, cards are constrained to static question/answer pairs. In StudyLab, learning interactions are powered by **Modality-Matched Interactive Learning Objects**. 

Grounded in Cognitive Load Theory (Sweller 1988; Paas, Renkl, & Sweller 2003), modality-matched containers eliminate artificial typing friction, prevent transcription errors, and focus the learner's working memory exclusively on domain-specific cognitive operations.

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                      STUDYLAB LEARNING OBJECT MODALITIES                        │
├────────────────────────────────┬────────────────────────────────────────────────┤
│ Modality Container             │ Pedagogical Focus                              │
├────────────────────────────────┼────────────────────────────────────────────────┤
│ **MCQContainer**               │ Rapid conceptual choice & zero-text input      │
├────────────────────────────────┼────────────────────────────────────────────────┤
│ **NumericalContainer**         │ 5D physical vectors, units, & tolerances       │
├────────────────────────────────┼────────────────────────────────────────────────┤
│ **StepwiseContainer**          │ Multi-step algebraic CAS & downstream validity │
├────────────────────────────────┼────────────────────────────────────────────────┤
│ **WorkedExampleView**          │ Low-load expert schema modeling with ack gate  │
├────────────────────────────────┼────────────────────────────────────────────────┤
│ **MistakeFooter**              │ Metacognitive reflection & Space/Enter trapping│
└────────────────────────────────┴────────────────────────────────────────────────┘
```

---

## 2. Modality 1: Multiple Choice Question (`MCQContainer`)

The `MCQContainer` (`ts/reviewer/components/mcq_container.ts`) provides high-speed, zero-friction choice selection for conceptual checks, strategy drills, diagnostic sweeps, and discrete problem archetypes.

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          MCQ CONTAINER DOM LAYOUT                       │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│   (A) Option Label One                       [ Key: 1 / A ]             │
│   ┌─────────────────────────────────────────────────────────────┐       │
│   │  ●  Option text with LaTeX: $\Delta G = \Delta H - T\Delta S$│      │
│   └─────────────────────────────────────────────────────────────┘       │
│   (B) Option Label Two                       [ Key: 2 / B ]             │
│   ┌─────────────────────────────────────────────────────────────┐       │
│   │  ○  Option text with chemical formula                       │       │
│   └─────────────────────────────────────────────────────────────┘       │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 2.1 Zero-Text Input Fallback Enforcement
To prevent modal confusion and typing friction, `enforceZeroTextInputFallback()` strictly enforces:
- Hides free-text input `#proc-quick-container` and stepwise input `#proc-stepwise-container`.
- Disables `#proc-answer-input` (`disabled = true`, `aria-hidden = true`).
- Removes the mode-switch toggle (`.proc-mode-switch`).

### 2.2 ARIA Accessibility & Roving Tabindex
- The container receives `role="radiogroup"` with `aria-label="Multiple Choice Options"`.
- Each option row receives `role="radio"`, `aria-checked="true|false"`, and a roving `tabindex="0|-1"`.
- Active option receives focus and keyboard navigation tracking.

### 2.3 Comprehensive Keyboard Navigation Map
| Key Event | Action & Behavior |
|---|---|
| `1`, `2`, `3`, `4`, ... | Direct numeric selection mapping 1-to-1 with 0-indexed options. |
| `A`, `B`, `C`, `D` / `a`, `b`, `c`, `d` | Direct alphabetic selection (case-insensitive). |
| `ArrowDown` / `ArrowRight` | Moves focus to next option with circular wraparound ($N-1 \to 0$). |
| `ArrowUp` / `ArrowLeft` | Moves focus to previous option with circular wraparound ($0 \to N-1$). |
| `Enter` / `Space` | Selects currently focused option and submits answer. |

### 2.4 Canonical Evaluation & Matching Engine
`MCQContainer` evaluates submissions against diverse backend contract schemas, matching:
1. `canonical_id` or `correct_option_id` (e.g. `"opt_b"`)
2. `correct_option` (letter `"B"` or `"b"`)
3. Numeric string or integer index (`0` or `1`)
4. Formatted string or option text (`"15 m/s"`)

### 2.5 Mock Exam Mode (GAP-MOD-03 Resolution)
When `mode === "mock"`:
- Selecting an option highlights it with `.selected` but **suppresses immediate spoiler feedback**.
- Styles `.correct`, `.incorrect`, and disabled locks are prevented during solving.
- Dispatches `procedural_mock_selection` to Python bridge.
- Provides on-demand batch evaluation via `mcq.evaluate()` upon test submission.

### 2.6 Sub-Modalities: ConceptCheck & StrategyDrill
- **`ConceptCheckData`:** Displays diagnostic feedback highlighting specific misconceptions if an invalid distractor is chosen.
- **`StrategyDrillData`:** Evaluates choice against `preferred_option_id`, providing feedback on strategy optimality (e.g. *Energy conservation is faster than kinematic integration*).

---

## 3. Modality 2: Numerical & Dimensional Vector Engine (`NumericalContainer`)

The `NumericalContainer` (`ts/reviewer/components/numerical_container.ts`) processes quantitative answers across Physics, Chemistry, and Mathematics, combining a 5-dimensional physical unit algebra with intelligent string parsing.

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    NUMERICAL CONTAINER WITH LIVE PREVIEW                │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│   Enter Value:                                                          │
│   ┌─────────────────────────────────────────────────────────────┐       │
│   │  v = 72 km/h                                                │       │
│   └─────────────────────────────────────────────────────────────┘       │
│                                                                         │
│   [ Live Preview: 20 m/s (Dimension: [Length]¹ [Time]⁻¹) ]              │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 3.1 Five-Dimensional Physical Vector (`PhysicalDimension`)
Every physical and chemical quantity is represented as a 5-tuple exponent vector:

$$\mathbf{D} = [M]^m \cdot [L]^l \cdot [T]^t \cdot [N]^n \cdot [K]^k$$

where:
- $[M]$ = Mass ($\text{kg}$)
- $[L]$ = Length ($\text{m}$)
- $[T]$ = Time ($\text{s}$)
- $[N]$ = Amount of Substance ($\text{mol}$)
- $[K]$ = Thermodynamic Temperature ($\text{K}$)

`PhysicalDimension` supports full dimensional algebra:
- **Multiplication:** $\mathbf{D}_1 \times \mathbf{D}_2 \implies [m_1+m_2, l_1+l_2, t_1+t_2, n_1+n_2, k_1+k_2]$
- **Division:** $\mathbf{D}_1 / \mathbf{D}_2 \implies [m_1-m_2, l_1-l_2, t_1-t_2, n_1-n_2, k_1-k_2]$
- **Power:** $\mathbf{D}^p \implies [p \cdot m, p \cdot l, p \cdot t, p \cdot n, p \cdot k]$
- **Compatibility Check:** `isCompatibleWith(other)` verifies dimensional homogeneity before evaluating magnitudes.

### 3.2 Comprehensive Unit Registry (50+ Units)
The `UnitRegistry` supports linear conversions and temperature offsets across Physics and Chemistry:

| Domain | Units Supported | SI Base Conversion |
|---|---|---|
| **Length / Distance** | $\text{m}, \text{km}, \text{cm}, \text{mm}, \mu\text{m}, \text{nm}, \text{\AA}$ | Linear multiplier to $\text{m}$ ($1\text{ \AA} = 10^{-10}\text{ m}$) |
| **Mass** | $\text{kg}, \text{g}, \text{mg}, \mu\text{g}, \text{ton}, \text{amu}$ | Linear multiplier to $\text{kg}$ ($1\text{ g} = 10^{-3}\text{ kg}$) |
| **Time** | $\text{s}, \text{ms}, \mu\text{s}, \text{min}, \text{h}, \text{day}$ | Linear multiplier to $\text{s}$ ($1\text{ h} = 3600\text{ s}$) |
| **Velocity / Speed** | $\text{m/s}, \text{km/h}, \text{cm/s}, \text{mph}, \text{knot}$ | $72\text{ km/h} \times \frac{5}{18} = 20\text{ m/s}$ |
| **Acceleration** | $\text{m/s}^2, \text{cm/s}^2, g$ | Linear multiplier to $\text{m/s}^2$ |
| **Force & Pressure** | $\text{N}, \text{kN}, \text{dyn}, \text{Pa}, \text{kPa}, \text{MPa}, \text{bar}, \text{atm}, \text{mmHg}, \text{torr}$ | $1\text{ atm} = 101{,}325\text{ Pa}$ |
| **Energy & Power** | $\text{J}, \text{kJ}, \text{MJ}, \text{cal}, \text{kcal}, \text{eV}, \text{keV}, \text{MeV}, \text{W}, \text{kW}$ | $1\text{ eV} = 1.602176634 \times 10^{-19}\text{ J}$ |
| **Chemistry Molar** | $\text{mol}, \text{mmol}, \mu\text{mol}, \text{M}, \text{mM}, \mu\text{M}, \text{g/mol}, \text{kJ/mol}$ | $1.2\text{ mM} = 0.0012\text{ M}$ |
| **Temperature** | $\text{K}, ^\circ\text{C}, ^\circ\text{F}$ | $T_K = T_C + 273.15$; $T_K = (T_F - 32)\times\frac{5}{9} + 273.15$ |

### 3.3 Input Parsing Engine (`NumericalParser`)
`NumericalParser` handles messy student inputs without rejection:
1. **Equation Prefix Stripping:** Automatically removes prefixes like `v = 15.5 m/s`, `[H+] = 1.0e-7 M`, `ans = 100`, `x = 5`.
2. **Currency & Comma Cleaning:** Strips `$`, `€`, `£`, `₹`, and thousand-separator commas (`$1,250.50` $\to$ `1250.5`).
3. **Percent Parsing:** Converts `75%` $\to$ magnitude `75`, unit `PERCENT`.
4. **Unicode Superscript Normalization:** Normalizes superscript exponents (`10⁻³`, `10²³`, `m·s⁻¹`, `g/cm³`).
5. **Scientific Notation Formats:** Recognizes `1.2e-3`, `1.2E-3`, `6.022 x 10^23`, `6.022 * 10^23`, `6.022 × 10²³`.
6. **Fractional Forms:** Parses rational fractions directly: `3/4`, `3/4 m/s`, `-1/2 kg`.

### 3.4 Tolerance Engine
Numerical verification supports three tolerance modes:
- **Absolute Tolerance:** $|V_{actual} - V_{expected}| \le \text{tol.absolute}$
- **Relative Tolerance:** $|V_{actual} - V_{expected}| \le \text{tol.relative} \times |V_{expected}|$ (Default: $0.5\%$)
- **Combined Tolerance:** $|V_{actual} - V_{expected}| \le \max(\text{tol.abs}, |V_{exp}| \times \text{tol.rel})$

### 3.5 Diagnostic Sanity & Trap Heuristics
- **Physical Non-Negativity:** Throws instant diagnostic feedback if negative values are entered for strictly non-negative quantities (mass, distance, absolute temperature, molar amounts).
- **Dimension Incompatibility:** Distinguishes between pure numerical errors and dimension errors (e.g. submitting `kg` when `m/s` was expected).
- **Unit Trap Warnings:** Detects missing standard unit conversions (e.g. forgot $5/18$ factor for $\text{km/h} \to \text{m/s}$, or forgot $\div 1000$ for $\text{g} \to \text{kg}$ and $\text{mM} \to \text{M}$).
- **Live Preview Pill:** Renders a real-time `.proc-num-preview-pill` directly below the input showing the parsed magnitude and normalized SI unit.

---

## 4. Modality 3: Stepwise Algebraic Reasoning (`StepwiseContainer` & Rust `StepValidator`)

The `StepwiseContainer` (`ts/reviewer/components/stepwise_container.ts`) and Rust `StepValidator` (`rslib/procedural/src/problems/steps/`) implement Kurt VanLehn's **Cognitive Tutor Inner Loop** ($d \approx 0.76$ effect size), validating each intermediate reasoning step before allowing the student to proceed.

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    STEPWISE REASONING CONTAINER                         │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│   Step 1: Formula Selection                               [ ✔ Valid ]   │
│   ┌─────────────────────────────────────────────────────────────┐       │
│   │  v^2 = u^2 + 2as                                            │       │
│   └─────────────────────────────────────────────────────────────┘       │
│                                                                         │
│   Step 2: Substitution & Linear Equivalence               [ ✔ Valid ]   │
│   ┌─────────────────────────────────────────────────────────────┐       │
│   │  0 = 400 - 20s  (Equiv: 20s = 400 => s = 20)                │       │
│   └─────────────────────────────────────────────────────────────┘       │
│                                                                         │
│   Step 3: Final Magnitude & Units                         [ Active ]    │
│   ┌─────────────────────────────────────────────────────────────┐       │
│   │  s = 20 m                                                   │       │
│   └─────────────────────────────────────────────────────────────┘       │
│                                                                         │
│   [ + Add Intermediate Step ]  [ ? Request Step Hint (Level 1/3) ]       │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 4.1 Mathematical Semantic Comparator (`MathSemanticComparator`)
Unlike naive string matching, `MathSemanticComparator` performs multi-tier semantic evaluation:
1. **String Normalization:** Strips whitespace, LaTeX formatting escapes (`\`), currency symbols, and converts to lowercase.
2. **Linear Equation Equivalence:**
   - Evaluates standard form $Ax = B$ by extracting coefficients and solving for root $x = B/A$.
   - Recognizes that `2x + 6 = 16`, `2x = 10`, `x = 5`, and `5 = x` are **semantically identical mathematical states**.
3. **Commutative Addition Matching:** Sorts additive terms alphabetically/numerically (`2x + 6` $\equiv$ `6 + 2x`).
4. **Multiplier vs Percentage Equivalence:** Bridges numeric representations (`1.20` $\equiv$ `120%` $\equiv$ `+20%`).

### 4.2 Downstream Consistency Tracking (`PartiallyValid`)
When evaluating a multi-step derivation:
- If Step $k$ contains an algebraic error, its status is marked `Invalid` and its erroneous root value $V_{err}$ is cached.
- If Step $k+1$ correctly derives its next expression from $V_{err}$, `StepValidator` marks Step $k+1$ as `StepValidationStatus::PartiallyValid` with `is_downstream_consistent = true`.
- **Pedagogical Significance:** Prevents catastrophic cascading score penalties. The system accurately assigns credit/blame to Step $k$ while rewarding valid logical deduction in downstream steps.

### 4.3 Taxonomic Step Error Diagnosis (35+ Variants)
`StepValidator` classifies step errors into four domain taxonomies:
- **Mathematics:** `FormulaSelectionError`, `SetupError`, `TransformationError`, `ArithmeticError`, `SignError`, `RatioInversionError`, `AlligationSwapError`, `InequalitySignFlipError`, `IdentityCrossTermError`.
- **Physics:** `ModelSelectionError`, `RepresentationError`, `EquationSetupError`, `SignConventionError`, `AlgebraExecutionError`, `PhysicalPlausibilityError`.
- **Chemistry:** `ChemicalRepresentationError`, `EquationBalanceError`, `StoichiometricRatioError`, `LimitingReagentError`, `RegimeSelectionError`, `ConservationViolationError`.
- **Reasoning:** `SchemaRecognitionError`, `StrategySelectionError`, `ConstraintApplicationError`, `InferenceError`, `SearchCaseError`, `ContradictionHandlingError`, `ReadingTrapError`.

### 4.4 Progressive 3-Tier Step Hints
Each step node in the `SolutionGraph` provides 3 escalating hint tiers:
1. **Tier 1 — Principle Hint:** States the applicable physical law, identity, or theorem (e.g. *Use Conservation of Mechanical Energy: $E_i = E_f$*).
2. **Tier 2 — Operation Hint:** Explains the next algebraic or logical operation (e.g. *Isolate $v$ by subtracting potential energy from initial total energy*).
3. **Tier 3 — Intermediate Relation Hint:** Provides the concrete intermediate equation (e.g. *$\frac{1}{2}mv^2 = mgh \implies v = \sqrt{2gh}$*).

---

## 5. Modality 4: Worked Example Modality (`WorkedExampleObject`)

Grounded in Renkl & Atkinson's (2003) Scaffolding Decay Theory, `WorkedExampleObject` (`rslib/procedural/src/remediation/objects.rs`) provides low-cognitive-load expert modeling for students stuck in high-recurrence failure loops ($\text{recurrence} == 3$).

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      WORKED EXAMPLE READING MODALITY                    │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│   📖 EXPERT SOLUTION TRACE                                              │
│                                                                         │
│   Step 1: Identify Governing Principle                                 │
│   • Conservation of Linear Momentum applies because no external         │
│     horizontal forces act on the system during the collision.           │
│                                                                         │
│   Step 2: State Symbolic Equation                                       │
│   • $m_1 u_1 + m_2 u_2 = (m_1 + m_2) v_f$                               │
│                                                                         │
│   Step 3: Substitute & Compute                                          │
│   • $(2\text{ kg})(6\text{ m/s}) + (4\text{ kg})(0) = (6\text{ kg}) v_f$│
│   • $12 = 6 v_f \implies v_f = 2.0\text{ m/s}$                          │
│                                                                         │
│   ───────────────────────────────────────────────────────────────────   │
│   [ ✔ I Have Reviewed and Understood This Solution ] (Mandatory)        │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 5.1 Mandatory Acknowledgement Gate
To prevent superficial skimming, the worked example view requires explicit acknowledgement before proceeding. Reading a worked example **does not award mastery points**; instead, it immediately queues a fresh `TransferRetry` variant to verify active schema acquisition.

---

## 6. Modality 5: Post-Error Reflection Strip (`MistakeFooter`)

When an incorrect answer is submitted, the input area is replaced with `MistakeFooter` (`ts/reviewer/components/mistake_footer.ts`):

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      MISTAKE CLASSIFICATION STRIP                       │
├────────┬────────────────────────────────┬───────────────────────────────┤
│ Key    │ Button Label                   │ Telemetry Category            │
├────────┼────────────────────────────────┼───────────────────────────────┤
│ **1**  │ `[1 Silly Slip]`               │ `silly_mistake`               │
├────────┼────────────────────────────────┼───────────────────────────────┤
│ **2**  │ `[2 Pattern Missed]`           │ `pattern_not_recognized`      │
├────────┼────────────────────────────────┼───────────────────────────────┤
│ **3**  │ `[3 Concept Gap]`              │ `formula_or_concept_misapplied`│
├────────┼────────────────────────────────┼───────────────────────────────┤
│ **4**  │ `[4 Prereq Unknown]`           │ `concept_not_known`           │
└────────┴────────────────────────────────┴───────────────────────────────┘
```

### 6.1 Reflection Gate & Space/Enter Trapping
- In `mistake_classification` state, Space and Enter key events are strictly trapped (`e.preventDefault()`, `e.stopPropagation()`).
- The learner cannot accidentally skip reflection. Pressing `1`, `2`, `3`, or `4` records the classification, dispatches `procedural_mistake` to the bridge, and unlocks the feedback transition after 150ms.

---

## 7. Lifecycle, Teardown & Event Safety

To guarantee 100% non-interference with standard Anki flashcards:
1. **Container Scoping:** All StudyLab DOM elements are strictly encapsulated inside `#procedural-card`.
2. **`MutationObserver` Unmount Hook:** A `MutationObserver` on `document.body` monitors `#procedural-card`. If the container is unmounted during card transition, `ProceduralReviewer.destroy()` is automatically invoked.
3. **Python Bridge Cleanup:** `qt/aqt/reviewer.py:207, 410` calls `globalThis.anki.procedural.destroyActive()` prior to loading any card.
4. **Listener Teardown:** `ProceduralReviewer.destroy()` iterates through `this.disposables`, nulls timers, and disposes all `window` and `document` event listeners, guaranteeing **zero memory leaks and zero shortcut leakage across cards**.

---

## 8. Verification & Codebase Traceability Matrix

| Component | Source Code Reference | Test Evidence Suite | Verified Artifact |
|---|---|---|---|
| `MCQContainer` | `ts/reviewer/components/mcq_container.ts:1-350` | `mcq_container.test.ts` (12 tests) | `05_live_ui_screenshots/01_math_mcq.png` |
| `NumericalContainer` & 5D Vector | `ts/reviewer/components/numerical_container.ts:1-600` | `numerical_container.test.ts` (28 tests) | `05_live_ui_screenshots/04_phys_units.png` |
| `StepwiseContainer` & CAS | `ts/reviewer/components/stepwise_container.ts:1-450` | `stepwise_container.test.ts` (7 tests) | `05_live_ui_screenshots/02_math_stepwise.png` |
| `MistakeFooter` & Trapping | `ts/reviewer/components/mistake_footer.ts:1-180` | `procedural.test.ts` (27 tests) | `05_live_ui_screenshots/03_mistake_footer.png` |
| Worked Example Object | `rslib/procedural/src/remediation/objects.rs:1-150` | `remediation_engine_tests.rs` (6 tests) | `05_live_ui_screenshots/06_worked_example.png` |
| Reviewer Teardown & Safety | `ts/reviewer/procedural.ts:1239-1278` | `desktop_validation_master_suite.rs` (Section 7, 1000 transitions) | `07_test_summary.md` (Section 5) |

