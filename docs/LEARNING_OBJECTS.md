# Learning Objects

StudyLab exposes multiple learning interaction types, mapped to `LearningObjectKind` in the TypeScript frontend (`ts/reviewer/procedural.ts:38-47`) and `RemediationIntervention` in the Rust backend (`rslib/procedural/src/remediation/objects.rs:656-664`). 

**CRITICAL DISTINCTION:** These are **not** Anki card types. They are specific pedagogical interventions and problem-solving modalities injected dynamically during a study session.

---

## 1. `problem` (Standard Procedural Practice)
- **Rust Backend:** `LearningObjectKind::ProceduralProblem`
- **Purpose:** Full end-to-end execution of a procedural problem.
- **When Used:** The default adaptive practice mode for evaluating fluency.
- **Learner Action:** Calculates and inputs the final answer, or navigates intermediate steps in stepwise mode.
- **Input Modality:** Freeform numerical entry with physical units (`NumericalContainer`), expression parsing, or symbolic text.
- **Evidence Produced:** Comprehensive `DomainEvidence` (`MathEvidence`, `PhysicsEvidence`, `ChemistryEvidence`, `ReasoningEvidence`).
- **State Transition:** Promotes skill state (`New -> Learning -> Fluent -> Mastered`) or triggers remediation on failure.

## 2. `mcq` (Multiple Choice)
- **Rust Backend:** Handled via `DiscreteChoice` parameter domains / `CognitiveDecisionPoint`.
- **Purpose:** To isolate specific cognitive decisions, conceptual branching, or when full numerical calculation is unnecessary.
- **When Used:** Diagnostic sweeps, concept checks, or when the `inline_contract` specifies discrete categorical options.
- **Learner Action:** Selects one option from dynamically generated distractors using keyboard (`1`–`4`, `A`–`D`) or mouse clicks.
- **Input Modality:** Radio button list (`.proc-option-item`) with ARIA selection attributes.
- **Evidence Produced:** Decision path accuracy and trap avoidance evidence.
- **State Transition:** Fast progression through concept checks or diagnostic sweeps.

## 3. `concept_check`
- **Rust Backend:** `RemediationIntervention::ConceptCheck`
- **Purpose:** To verify underlying conceptual understanding when an execution failure is suspected to stem from deeper theoretical confusion.
- **When Used:** JIT remediation triggered by `RemediationPolicy` after a concept-flagged error (`is_conceptual_error()`).
- **Learner Action:** Answers a qualitative or conceptual question testing the core principle without heavy calculation.
- **Input Modality:** MCQ or short conceptual selection.
- **Evidence Produced:** Concept validity (`pattern_recognition`, `governing_principle`, `trend_reasoning`).
- **State Transition:** Restores progression upon success; demotes to `PrerequisiteReview` upon repeat failure.

## 4. `strategy_drill`
- **Rust Backend:** `RemediationIntervention::StrategyDrill`
- **Purpose:** To practice recognizing the correct solving method or formula setup without executing arithmetic.
- **When Used:** Triggered when `method_selection` or `decision_path` failures are detected.
- **Learner Action:** Selects the governing formula, coordinate system, or strategic approach for a given problem scenario.
- **Input Modality:** MCQ or setup choice selector.
- **Evidence Produced:** `method_selection`, `physical_model_selection`, and `equation_setup` evidence.
- **State Transition:** Advances to targeted practice upon successful strategy identification.

## 5. `representation_drill`
- **Rust Backend:** `RemediationIntervention::RepresentationDrill`
- **Purpose:** To practice constructing valid abstract models (e.g., Free-Body Diagrams, seating grids, circuit topologies, ICE tables).
- **When Used:** Triggered when `representation` or `constraint_extraction` errors occur in Physics, Chemistry, or Logical Reasoning.
- **Learner Action:** Organizes constraints or selects correct spatial/diagrammatic configurations.
- **Input Modality:** Interactive grid, diagram selector, or structured constraint matcher.
- **Evidence Produced:** `representation` and `constraint_extraction` evidence.
- **State Transition:** Unlocks formula execution once representation accuracy is verified.

## 6. `worked_example`
- **Rust Backend:** `RemediationIntervention::WorkedExample`
- **Purpose:** To model the expert solution path and cognitive steps.
- **When Used:** Triggered by canonical escalation when a learner experiences repeat conceptual failures (`recurrence == 3`).
- **Learner Action:** Reads and acknowledges the step-by-step derivation with highlighted critical decision points.
- **Input Modality:** Reading view with required acknowledgement button.
- **Evidence Produced:** Cognitive reset; marks worked example exposure.
- **State Transition:** Immediately queues a `procedural_variant` (Transfer Retry) to test immediate retention.

## 7. `declarative_recall`
- **Rust Backend:** `RemediationIntervention::DeclarativeRecall`
- **Purpose:** Fast factual recall injected seamlessly into the procedural flow.
- **When Used:** When a missing constant, unit conversion factor, or declarative formula blocks procedural execution.
- **Learner Action:** Recalls and enters the factual value or definition.
- **Input Modality:** Direct numeric/text input.
- **Evidence Produced:** Recall latency and accuracy.
- **State Transition:** Returns immediately to the blocked procedural problem.

## 8. `prerequisite_review`
- **Rust Backend:** `RemediationIntervention::PrerequisiteReview`
- **Purpose:** To repair fundamental foundational gaps in prerequisite skills.
- **When Used:** Critical urgency escalation (`recurrence == 4`) when a learner cannot solve problems due to missing prerequisite fluency.
- **Learner Action:** Practices foundational skills one level down the knowledge graph.
- **Input Modality:** `problem` or `mcq`.
- **Evidence Produced:** Prerequisite mastery evidence.
- **State Transition:** Returns to parent problem family once prerequisite fluency is restored.

## 9. `procedural_variant` (Transfer Retry)
- **Rust Backend:** `RemediationIntervention::TransferRetry` / `PyqVariant`
- **Purpose:** To test whether the learner can apply the skill to a new numerical instantiation or structurally varied problem.
- **When Used:** After an execution error (generates "simpler_numbers") or after acknowledging a `worked_example`.
- **Learner Action:** Solves a fresh instance of the archetype.
- **Input Modality:** Matches the base problem modality.
- **Evidence Produced:** `structural_transfer` and independent execution evidence.
- **State Transition:** Finalizes the attempt sequence and updates `SkillState`.

## 10. `circuit_breaker`
- **Rust Backend:** `RemediationIntervention::CircuitBreaker`
- **Purpose:** To halt unhelpful practice on a specific problem family and prevent learner frustration/wheel-spinning.
- **When Used:** Advisory urgency escalation when a learner fails $\ge 5$ times in a session (`recurrence >= 5`).
- **Learner Action:** Informational banner advising the learner to pause or switch topics.
- **Input Modality:** Non-interactive advisory banner.
- **Evidence Produced:** Wheel-spinning flag recorded in telemetry.
- **State Transition:** Puts the problem family on cooldown until the next study session.

---

## Active Modes: `quick` vs `stepwise`
Within a `problem` learning object, the learner can operate in two distinct modes:
- **`quick` (Default):** The learner works on scratchpad/paper and submits the final numeric/expression answer directly.
- **`stepwise`:** The learner expands intermediate steps (`StepwiseContainer`), validating each line of mathematical or logical deduction against the solution graph before entering the final result.
