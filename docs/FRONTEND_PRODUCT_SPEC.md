# StudyLab Frontend Product Specification & Learning Object Contracts

**Document Version:** 1.0.0 (Canonical Master Specification)  
**Target Repository:** `Anki-maths` (StudyLab Procedural Subsystem)  
**Status:** AUTHORITATIVE SPECIFICATION  
**Integrity Mode:** Benchmark Mode (100% Grounded in Executable Code, Passing Tests, and UI Verification)  
**Authoritative Sections Covered:** Sections 5, 11, 12, 13, 14, 15 of `ORIGINAL_REQUEST.md`

---

## 1. Primary Product & Modality Invariant

### 1.1 Core Frontend Definition
**The StudyLab frontend is a focused, high-precision problem-solving workspace embedded within Anki.**

It is NOT a flashcard card-flip UI, NOT a generic quiz page, and NOT a second Anki reviewer. The interface exists solely to facilitate active problem solving, step-level reasoning, and metacognitive reflection with zero extraneous cognitive load.

### 1.2 The Absolute Modality Invariant
```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                           THE ABSOLUTE MODALITY INVARIANT                        │
├──────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│   SEMANTIC MODALITY MUST ALWAYS MATCH UI MODALITY.                               │
│                                                                                  │
│   • If the learning object is MCQ, render authentic selectable radio cards.      │
│   • If the learning object is Numerical, render dimensional unit-aware inputs.   │
│   • If the learning object is Stepwise, render interactive solution step nodes.  │
│   • If the learning object is Worked Example, render annotated solution traces.  │
│                                                                                  │
│   HARD PROHIBITION: Never use a generic fill-in-the-blank textbox as a fallback  │
│   for MCQ, ConceptCheck, StrategyDrill, or WorkedExample.                        │
│                                                                                  │
└──────────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Learning Object Contract (Section 5)

StudyLab defines exactly **nine canonical learning objects** (`LearningObjectKind`). Each object serves a specific educational objective and enforces a distinct interaction surface:

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                      THE 9 STUDYLAB LEARNING OBJECTS                            │
├──────────────────────────────┬──────────────────────────────────────────────────┤
│ 1. `problem`                 │ Comprehensive quantitative / symbolic solving    │
│ 2. `quick`                   │ High-speed numerical fluency / single-step solve │
│ 3. `mcq`                     │ Rapid conceptual choice & discrete alternatives  │
│ 4. `stepwise`                │ Multi-step algebraic derivation & CAS validation │
│ 5. `concept_check`           │ Targeted distractor diagnostics & schema repair  │
│ 6. `strategy_drill`          │ Method selection & efficiency optimization       │
│ 7. `worked_example`          │ Low-load expert modeling with acknowledgment gate│
│ 8. `declarative_recall`      │ Spaced repetition bridge for atomic formulas/defs│
│ 9. `prerequisite_review`     │ Directed remediation of missing foundational KCs │
└──────────────────────────────┴──────────────────────────────────────────────────┘
```

---

### Detailed Contract for All 9 Learning Objects

---

### Object 1: `problem` (Comprehensive Procedural Problem)
- **Educational Purpose:** Assess and build procedural problem-solving fluency in multi-step quantitative and symbolic domains (Mathematics, Physics, Chemistry).
- **Learner Goal:** Formulate the mathematical model, perform calculations, and submit the correct final numerical/symbolic result with proper units.
- **Presentation:** High-resolution problem stem with formatted LaTeX equations; clean input surface; optional mode toggle to stepwise solving.
- **Interaction:** Learner types the final expression or magnitude + unit into `NumericalContainer`, or clicks the mode tab to switch to `StepwiseContainer`.
- **Allowed Controls:** `[Submit Answer]` (Primary), `[Mode Switch: Quick / Stepwise]`, `[💡 Request Hint]`.
- **Forbidden Controls:** MCQ radio options, arbitrary Anki ease buttons during solving.
- **Answer Modality:** `NumericalContainer` (5D physical unit algebra) or `MathSemanticComparator` expression input.
- **Success State:** Instant green validation outline; displays canonical solution derivation; records `is_correct: true`; reveals `Next Problem` CTA.
- **Wrong State:** Input frozen; displays concise error message; displays 4-choice `MistakeFooter` (`[1 Silly]`, `[2 Pattern]`, `[3 Concept]`, `[4 Prereq]`); traps `Space`/`Enter` keys until classified.
- **Feedback State:** Displays full canonical steps with intermediate explanations and latency quadrant indicators.
- **Diagnosis Rules:** Evaluates calculation precision, dimensional validity, and domain boundary constraints (`MathEvidence` / `PhysicsEvidence`).
- **Remediation Rules:** On `Concept` or `Prereq` error, queues `ConceptCheck` or `PrerequisiteReview`. On `Pattern` error, queues `StrategyDrill`.
- **Next Action:** Primary CTA: `Next Problem [Enter]`.

---

### Object 2: `quick` (Fluency & Quick Solve)
- **Educational Purpose:** Rapid retrieval practice and automaticity building for single-operation formulas, mental estimation, and routine calculations.
- **Learner Goal:** Compute and enter the final numerical or algebraic answer quickly without scaffolding.
- **Presentation:** Focused, ultra-minimal layout: question stem directly above a single high-contrast input box with live magnitude preview pill.
- **Interaction:** Single text input with real-time scalar parsing and SI unit conversion.
- **Allowed Controls:** Single input box `#proc-answer-input`, `[Submit Answer]` (or `Enter` key).
- **Forbidden Controls:** Stepwise derivation tools, mode switcher tabs, hint requests (unscaffolded fluency test).
- **Answer Modality:** Single scalar or unit-aware numerical string (`NumericalParser`).
- **Success State:** Instant green border; validates latency against `target_time_ms`; records fluency score; advances.
- **Wrong State:** Red input outline; reveals correct value and 4-tier mistake classification strip.
- **Feedback State:** Minimal feedback: expected answer vs. submitted answer with conversion notes if unit was omitted.
- **Diagnosis Rules:** Detects arithmetic slips vs. unit conversion factor omissions ($5/18$, $\times 1000$).
- **Remediation Rules:** On repeated calculation slips, queues arithmetic precision drills.
- **Next Action:** `Next Problem [Enter]`.

---

### Object 3: `mcq` (Multiple Choice Question)
- **Educational Purpose:** Test discrete conceptual understanding, structure identification, and rapid decision-making across standardized test archetypes.
- **Learner Goal:** Identify and select the uniquely correct option among plausible distractors.
- **Presentation:** Problem stem followed by 4 distinct, full-width option cards (`role="radiogroup"`).
- **Interaction:** Keyboard hotkeys `1`–`4`, `A`–`D`, Arrow navigation, or direct click. Instant submit on `Enter` or double-press.
- **Allowed Controls:** Option selection cards (`role="radio"`), `[Submit Answer]`, `[💡 Request Hint]`.
- **Forbidden Controls:** Free-text input (`#proc-answer-input` strictly hidden and disabled), stepwise containers, mode switcher.
- **Answer Modality:** Discrete option selection (`MCQContainer`).
- **Success State:** Selected card highlights in emerald green (`.correct`); distractor cards fade; displays explanation.
- **Wrong State:** Selected card turns crimson (`.incorrect`); correct card highlights in emerald green; opens `MistakeFooter`.
- **Feedback State:** Explains why the correct option is true and provides targeted feedback on why selected distractor is false.
- **Diagnosis Rules:** Maps chosen option directly to distractor catalog to identify the specific bug or misconception.
- **Remediation Rules:** Evaluates `ErrorCategory` based on distractor metadata; queues targeted conceptual repair if distractor is a known misconception.
- **Next Action:** `Next Problem [Enter]`.

---

### Object 4: `stepwise` (Multi-Step CAS Derivation)
- **Educational Purpose:** Guide and assess complex multi-step derivations (Cognitive Tutor Inner Loop, $d \approx 0.76$), catching intermediate errors before compounding.
- **Learner Goal:** Complete each logical derivation step sequentially, arriving at the verified terminal solution.
- **Presentation:** Stacked sequential step nodes with validation status indicators (`✔ Valid`, `❌ Invalid`, `⚠️ Consistent`).
- **Interaction:** Learner types algebraic or numerical statements per step line. Evaluated in real-time via Rust `StepValidator`.
- **Allowed Controls:** `[+ Add Step]`, `[🗑 Remove Step]`, `[💡 Request Step Hint (Tier 1-3)]`, `[Check Solution]`, `[Reset]`.
- **Forbidden Controls:** Single quick solve box, MCQ option cards.
- **Answer Modality:** `StepwiseContainer` with symbolic CAS and algebraic equivalence matching.
- **Success State:** All required solution graph nodes marked `✔ Valid`; overall problem marked correct.
- **Wrong State:** The specific failing step is highlighted with diagnostic explanation; subsequent valid steps retain `ConsistentWithPriorError` credit.
- **Feedback State:** Detailed step-by-step audit showing where reasoning derailed and how to repair the derivation.
- **Diagnosis Rules:** Categorizes step errors into formula selection, sign flip, invalid algebraic transformation, or premature rounding.
- **Remediation Rules:** On step 1 formula failure $\to$ `ConceptCheck`. On algebraic manipulation failure $\to$ algebraic precision drill.
- **Next Action:** `Next Problem [Enter]`.

---

### Object 5: `concept_check` (Targeted Concept Diagnostic)
- **Educational Purpose:** Disambiguate mental models and test foundational principles immediately following a conceptual error, eliminating calculation overhead.
- **Learner Goal:** Select the conceptually sound principle, law, or qualitative prediction.
- **Presentation:** Clean qualitative scenario with 3–4 conceptual statement cards.
- **Interaction:** Single-click or keystroke (`1-4`, `A-D`) selection. Zero arithmetic required.
- **Allowed Controls:** Concept radio cards, `[Submit Choice]`.
- **Forbidden Controls:** Free-text input, numerical unit tools, stepwise derivation panels.
- **Answer Modality:** `MCQContainer` bound to `ConceptCheckData`.
- **Success State:** Option turns green; displays concise confirmation: *"✔ Correct principle applied."*
- **Wrong State:** Selected option turns red; immediately reveals specific diagnostic feedback bound to that distractor (e.g., *"⚠️ Additive Fallacy: Percentages do not add linearly when the base changes."*).
- **Feedback State:** In-depth explanation of the governing physical law or mathematical axiom.
- **Diagnosis Rules:** Directly updates `SkillState.domain_evidence.is_conceptual_error` and logs the diagnosed misconception tag.
- **Remediation Rules:** If failed, escalates to `WorkedExampleObject` or `PrerequisiteReviewObject`. If passed, returns to parametric problem practice.
- **Next Action:** Primary CTA: `[ Continue Practice ]`.

---

### Object 6: `strategy_drill` (Strategy Selection & Optimality)
- **Educational Purpose:** Train expert-level schema recognition and optimal strategy selection (e.g., choosing Conservation of Energy vs. Newton-Kinematics integration).
- **Learner Goal:** Identify the most efficient, least error-prone problem-solving pathway for a given problem stem.
- **Presentation:** Problem context box followed by candidate strategy descriptions.
- **Interaction:** Selection of candidate strategy via keyboard (`1-4`) or mouse click.
- **Allowed Controls:** Strategy option cards, `[Submit Strategy]`.
- **Forbidden Controls:** Free-text input, algebraic solvers, stepwise derivation tools.
- **Answer Modality:** `MCQContainer` bound to `StrategyDrillData`.
- **Success State:** Highlights selected optimal strategy; displays optimality rationale: *"⭐ Optimal Strategy: Energy method solves this in 1 line without calculating intermediate accelerations."*
- **Wrong State:** If a valid but sub-optimal strategy is chosen, highlights in amber: *"⚠️ Valid but Inefficient: Kinematics requires 4 steps and is prone to arithmetic slips."* If an invalid strategy is chosen, highlights in red.
- **Feedback State:** Side-by-side comparison of execution cost across different mathematical approaches.
- **Diagnosis Rules:** Records `method_selection` competency in `DomainEvidencePayload`.
- **Remediation Rules:** Returns learner to problem solving with strategy hint active.
- **Next Action:** Primary CTA: `[ Apply Strategy to Problem ]`.

---

### Object 7: `worked_example` (Expert Schema Modeling)
- **Educational Purpose:** Scaffold novice learners and break destructive failure loops by providing a low-cognitive-load expert solution trace (Sweller, 1988; Renkl & Atkinson, 2003).
- **Learner Goal:** Study the canonical expert trace, understand key decision points, and acknowledge comprehension before attempting a transfer problem.
- **Presentation:** Rich reading card featuring: Problem Context, Highlighted Key Decision Point, Sequential Canonical Steps, Method Rationale, and Common Pitfalls.
- **Interaction:** Passive reading and cognitive internalization followed by an explicit mandatory acknowledgment action.
- **Allowed Controls:** `[ ✔ I Have Reviewed and Understood This Solution ]` / `[ Try Similar Problem ]`.
- **Forbidden Controls:** All solving inputs (zero text boxes, zero MCQ radio options, zero stepwise inputs).
- **Answer Modality:** `WorkedExampleView` (Reading & Acknowledgment Modality).
- **Success State:** Clicking the acknowledgment button records the study event and immediately queues a fresh, seeded `TransferRetry` variant of the problem.
- **Wrong State:** N/A (Non-evaluative instructional modality).
- **Feedback State:** Full annotated solution remains visible during review.
- **Diagnosis Rules:** Logs `scaffolding_exposure: "worked_example"` in `SkillState`. Does NOT award mastery points.
- **Remediation Rules:** Automatically triggers an active isomorphic problem to test immediate schema transfer.
- **Next Action:** Primary CTA: `[ Try Similar Problem Now ]`.

---

### Object 8: `declarative_recall` (Spaced Repetition Bridge)
- **Educational Purpose:** Bridge procedural problem solving with standard declarative spaced repetition for atomic constants, formulas, nomenclature, or theorem statements.
- **Learner Goal:** Recall and verify an atomic declarative fact required by a procedural skill.
- **Presentation:** Standard Anki card layout or focused formula callout card.
- **Interaction:** Mental recall followed by Anki standard flip or tooltip verification.
- **Allowed Controls:** Native Anki rating controls (`Again`, `Hard`, `Good`, `Easy`) or `[ Continue to Procedural Practice ]`.
- **Forbidden Controls:** Complex multi-step procedural containers.
- **Answer Modality:** Declarative cued recall.
- **Success State:** Synchronizes memory stability with Anki FSRS scheduler.
- **Wrong State:** Schedules earlier declarative flashcard review.
- **Feedback State:** Displays exact formula, definition, or constant value.
- **Diagnosis Rules:** Isolates declarative memory gaps from procedural execution gaps.
- **Remediation Rules:** Links procedural anchor card with declarative note ID (`target_anki_card_id`).
- **Next Action:** Standard Anki review progression.

---

### Object 9: `prerequisite_review` (Prerequisite DAG Navigation)
- **Educational Purpose:** Identify and remediate missing foundational knowledge when a learner is blocked on advanced topics due to lower-tier skill gaps.
- **Learner Goal:** Navigate to and practice the missing prerequisite skill before resuming current topic.
- **Presentation:** Diagnostic advisory card displaying: Identified Gap, Prerequisite Skill Name, Dependency Hierarchy, and Launch Button.
- **Interaction:** Learner reviews the diagnostic recommendation and clicks to launch targeted prerequisite practice.
- **Allowed Controls:** `[ Practice Prerequisite: {skill_name} ]`, `[ Skip & Return to Deck ]`.
- **Forbidden Controls:** Active problem solving inputs for the parent skill.
- **Answer Modality:** Navigational & Diagnostic Advisory Modality.
- **Success State:** Launches a dedicated practice session on the prerequisite skill (`target_skill_id`).
- **Wrong State:** N/A.
- **Feedback State:** Explains why current topic depends on the prerequisite (e.g., *"Cannot master Circular Motion without mastering Centripetal Acceleration formulas"*).
- **Diagnosis Rules:** Records longitudinal prerequisite deficiency in `procedural.db`.
- **Remediation Rules:** Inserts prerequisite practice items at head of practice queue.
- **Next Action:** Primary CTA: `[ Start Prerequisite Practice ]`.

---

## 3. Stepwise Reasoning Contract (Section 13)

### 3.1 Architecture of the Cognitive Tutor Inner Loop
The `StepwiseContainer` (`ts/reviewer/components/stepwise_container.ts`) implements Kurt VanLehn's (2006, 2011) **Inner Loop**, validating each intermediate algebraic step in real time.

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                    STEPWISE REASONING CONTAINER ANATOMY                          │
├──────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│   Step 1: Formula Selection                               [ ✔ Valid ]            │
│   ┌──────────────────────────────────────────────────────────────────────┐       │
│   │  v^2 = u^2 + 2as                                                     │       │
│   └──────────────────────────────────────────────────────────────────────┘       │
│                                                                                  │
│   Step 2: Substitution & Linear Equivalence               [ ✔ Valid ]            │
│   ┌──────────────────────────────────────────────────────────────────────┐       │
│   │  0 = 400 - 20s  (Equiv: 20s = 400 => s = 20)                         │       │
│   └──────────────────────────────────────────────────────────────────────┘       │
│                                                                                  │
│   Step 3: Final Magnitude & Units                         [ Active Input ]       │
│   ┌──────────────────────────────────────────────────────────────────────┐       │
│   │  s = 20 m                                                            │       │
│   └──────────────────────────────────────────────────────────────────────┘       │
│                                                                                  │
│   [ + Add Step ]  [ 🗑 Remove Step ]  [ 💡 Request Step Hint (Tier 1/3) ]        │
│                                                                                  │
└──────────────────────────────────────────────────────────────────────────────────┘
```

### 3.2 Step Node Lifecycle & Validation States
Each step node in the solution graph transitions through discrete states:
1. **`Empty / Inactive`:** Placeholder container awaiting learner input.
2. **`Active`:** Currently focused input box.
3. **`Validating`:** Submitting expression to CAS / Rust `StepValidator` across IPC bridge.
4. **`Valid` (Green Outline & Checkmark):** Algebraically or semantically sound step.
5. **`Invalid` (Red Outline & Warning):** Algebraically invalid step, wrong formula, or sign error.
6. **`PartiallyValid / ConsistentWithPriorError` (Yellow Badge):** The step is algebraically consistent with a *previous* incorrect step. The learner receives partial reasoning credit while flagging the root error.

### 3.3 3-Tier Progressive Scaffolding Hints
When solving in stepwise mode, clicking `[ 💡 Request Hint ]` delivers progressive scaffolding without spoiling the final answer:
- **Tier 1 (Principle Hint):** Reveals the governing physical or mathematical principle (e.g., *"Apply Work-Energy Theorem to relate change in kinetic energy to work done by friction"*).
- **Tier 2 (Operation Hint):** Specifies the concrete mathematical operation (e.g., *"Isolate variable $s$ by subtracting $u^2$ and dividing by $2a$"*).
- **Tier 3 (Intermediate Relation Hint):** Provides the direct intermediate equation (e.g., *"Substitute values: $0 = (20)^2 + 2(-10)s$"*).

---

## 4. Multiple Choice Question (MCQ) Contract (Section 14)

### 4.1 Zero-Textbox Fallback Enforcement
```typescript
// Enforced in ts/reviewer/components/mcq_container.ts
export function enforceZeroTextInputFallback(root: HTMLElement): void {
    // 1. Hide free-text input and stepwise containers
    root.querySelector("#proc-quick-container")?.setAttribute("style", "display: none !important;");
    root.querySelector("#proc-stepwise-container")?.setAttribute("style", "display: none !important;");
    
    // 2. Disable and hide input element
    const input = root.querySelector<HTMLInputElement>("#proc-answer-input");
    if (input) {
        input.disabled = true;
        input.setAttribute("aria-hidden", "true");
    }
    
    // 3. Remove mode-switch toggle
    root.querySelector(".proc-mode-switch")?.remove();
}
```

### 4.2 Accessibility & Keyboard Mapping
- **Container ARIA:** `role="radiogroup"` with `aria-label="Multiple Choice Options"`.
- **Option ARIA:** `role="radio"`, `aria-checked="true|false"`, with roving `tabindex="0|-1"`.
- **Keyboard Map:**
  - `1`, `2`, `3`, `4`: Selects option 1, 2, 3, or 4 directly.
  - `A`, `B`, `C`, `D` (or `a`, `b`, `c`, `d`): Selects option A, B, C, or D directly.
  - `ArrowDown` / `ArrowRight`: Moves active focus to next option (wraps to 0).
  - `ArrowUp` / `ArrowLeft`: Moves active focus to previous option (wraps to $N-1$).
  - `Enter` / `Space`: Selects focused option and submits answer.

---

## 5. Worked Example Contract (Section 15)

### 5.1 Cognitive Architecture & Visual Layout
The `WorkedExampleView` (`ts/reviewer/components/worked_example_view.ts`) models expert problem solving to reduce extraneous cognitive load during initial schema acquisition.

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                          WORKED EXAMPLE VIEW LAYOUT                              │
├──────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│   📖 EXPERT SOLUTION WALKTHROUGH                                                 │
│   A dishonest shopkeeper sells goods at cost price but uses a 900g weight        │
│   for a 1kg requirement. Find his actual profit percentage.                      │
│                                                                                  │
│   ┌──────────────────────────────────────────────────────────────────────┐       │
│   │ 🔑 KEY DECISION POINT:                                               │       │
│   │ The base of the profit percentage is the ACTUAL quantity given       │       │
│   │ (900g), NOT the nominal quantity (1000g).                            │       │
│   └──────────────────────────────────────────────────────────────────────┘       │
│                                                                                  │
│   CANONICAL DERIVATION:                                                          │
│   1. Error in weight = $1000\text{ g} - 900\text{ g} = 100\text{ g}$.            │
│   2. Cost incurred by seller corresponds to $900\text{ g}$.                      │
│   3. Profit $\% = \frac{\text{Error}}{\text{True Weight}} \times 100             │
│                 = \frac{100}{900} \times 100 = 11\frac{1}{9}\%$.                 │
│                                                                                  │
│   METHOD RATIONALE:                                                              │
│   Profit is always calculated on the seller's actual cost/outlay.                │
│                                                                                  │
│   COMMON PITFALL TO AVOID:                                                       │
│   Do not divide 100 by 1000 (gives 10%, which is incorrect).                     │
│                                                                                  │
│   ────────────────────────────────────────────────────────────────────           │
│   [ ✔ I Have Reviewed and Understood This Solution — Try Similar Problem ]       │
│                                                                                  │
└──────────────────────────────────────────────────────────────────────────────────┘
```

---

## 6. Wrong-Answer Contract (Section 11)

### 6.1 Pedagogical Philosophy: Error as a Formative Catalyst
When a learner submits an incorrect response, the system does NOT treat it as a punitive dead end. Grounded in the **Hypercorrection Effect** (Metcalfe, 2017) and **Self-Explanation Theory** (Chi et al., 1989), errors are converted into active metacognitive learning moments.

### 6.2 The 5-Step Wrong-Answer Flow
```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                         WRONG-ANSWER WORKFLOW                                    │
├──────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│   1. SUBMISSION EVALUATION ──► Detects mismatch (numerical/CAS/MCQ).             │
│          │                                                                       │
│          ▼                                                                       │
│   2. INSTANT INPUT FREEZE  ──► Freezes input field with submitted value.         │
│          │                     Displays calm error notice:                       │
│          │                     "❌ Incorrect: Expected 7, Submitted 9".          │
│          ▼                                                                       │
│   3. METACOGNITIVE TRAP    ──► Activates 4-choice MistakeFooter:                 │
│          │                     [1 Silly]  [2 Pattern]  [3 Concept]  [4 Prereq]   │
│          │                     Intercepts Space and Enter keys (Anti-Bypass).    │
│          ▼                                                                       │
│   4. ERROR CLASSIFICATION  ──► Learner presses 1–4 to self-categorize mistake.   │
│          │                     Writes classification to `procedural.db`.         │
│          ▼                                                                       │
│   5. CANONICAL REVEAL      ──► Reveals full step-by-step LaTeX solution.         │
│                                Unlocks `Next Problem [Enter]` CTA.               │
│                                                                                  │
└──────────────────────────────────────────────────────────────────────────────────┘
```

---

## 7. Correct-Answer Contract (Section 12)

### 7.1 Calm, Minimal Pedagogical Flow
When a learner submits a correct response, StudyLab maintains a calm, focused aesthetic without distracting confetti, sound effects, or gamified popups.

### 7.2 The Correct-Answer Progression
1. **Immediate Visual Confirmation:** Active input border transitions to subtle emerald green (`#10b981`).
2. **Canonical Derivation Reveal:** Displays the full canonical LaTeX solution steps so the learner can verify their mental derivation against the optimal method.
3. **Speed-Accuracy Quadrant Indicator:** Displays a subtle latency indicator showing whether the solution was executed within target fluency parameters ($Q_1$ Fast & Accurate vs. $Q_2$ Slow & Accurate).
4. **Single Clear Next Action:** The primary CTA `Next Problem [Enter]` is highlighted and armed for single-keystroke progression.
