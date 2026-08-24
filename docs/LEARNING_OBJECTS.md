# Learning Objects

StudyLab exposes multiple learning interaction types, referred to as `LearningObjectKind` in the TypeScript frontend (`procedural.ts`). These are **not** flashcard types; they are specific pedagogical interventions and problem-solving modalities injected during a study session.

## `problem` (Standard Procedural Practice)
- **Purpose:** Full execution of a procedural problem from start to finish.
- **When Used:** The default adaptive practice mode for testing fluency.
- **What the Learner Does:** Calculates and inputs the final answer, or works through optional steps.
- **Input Modality:** Freeform text, math entry, or specialized domain inputs (e.g., units).
- **Evidence Produced:** Comprehensive `DomainEvidence` (execution, concept, bounds).
- **Leads To:** State promotion to `Fluent` or `Mastered`, or triggers remediation on failure.

## `mcq` (Multiple Choice)
- **Purpose:** To isolate specific decisions or when full calculation is unnecessary.
- **When Used:** Diagnostic sweeps, or when the `inline_contract` specifies a `DiscreteChoice` parameter.
- **What the Learner Does:** Selects one option from dynamically generated distractors.
- **Input Modality:** Radio buttons/buttons.
- **Evidence Produced:** Decision path accuracy.
- **Leads To:** Fast progression through concept checks.

## `concept_check`
- **Purpose:** To verify underlying conceptual understanding when an execution failure is suspected to be deeper.
- **When Used:** JIT remediation triggered by the `RemediationPolicy` after a concept-flagged error.
- **What the Learner Does:** Answers a conceptual, non-calculative question about the previous problem.
- **Input Modality:** MCQ or short textual.
- **Evidence Produced:** Concept validity.
- **Leads To:** Restoration to problem-solving, or further demotion to prerequisite review.

## `strategy_drill`
- **Purpose:** To practice selecting the correct theorem or formula without executing the math.
- **When Used:** When `method_selection` failures are detected.
- **What the Learner Does:** Chooses the formula or setup for a given problem.
- **Input Modality:** MCQ / Expression builder.
- **Evidence Produced:** `pattern_recognition` and `method_selection` evidence.
- **Leads To:** Next problem in the queue.

## `worked_example`
- **Purpose:** To model the correct solution path.
- **When Used:** Triggered by canonical escalation (e.g., recurrence == 3 for conceptual errors).
- **What the Learner Does:** Reads and acknowledges the step-by-step derivation.
- **Input Modality:** Reading / Acknowledgment button.
- **Evidence Produced:** None directly; resets immediate failure state.
- **Leads To:** A `procedural_variant` of the same concept to test immediate retention.

## `declarative_recall`
- **Purpose:** Standard factual recall injected seamlessly into the procedural flow.
- **When Used:** When a formula or constant needs to be memorized to solve a class of problems.
- **What the Learner Does:** Types or reveals the answer.
- **Input Modality:** Text/Math input.
- **Evidence Produced:** Raw recall latency and correctness.
- **Leads To:** Resumption of procedural practice.

## `prerequisite_review`
- **Purpose:** To repair fundamental foundational gaps.
- **When Used:** Critical urgency circuit breakers (e.g., recurrence == 4).
- **What the Learner Does:** Practices a foundational skill one level down the dependency tree.
- **Input Modality:** `problem` or `mcq`.
- **Evidence Produced:** Prerequisite fluency.
- **Leads To:** Return to the parent problem family once mastery is restored.

## `procedural_variant` (Transfer Retry)
- **Purpose:** To test if the learner can apply the skill to a different set of numbers or a slightly different context.
- **When Used:** After an execution error (generates simpler numbers) or after a worked example.
- **What the Learner Does:** Solves a new instance of the same archetype.
- **Input Modality:** Depends on the base problem.
- **Evidence Produced:** `structural_transfer` or basic execution evidence.
- **Leads To:** Finalization of the attempt sequence.

## `quick` / `stepwise`
*Note: `stepwise` operates primarily as an `activeMode` within a `problem` rather than a standalone object, guiding the user node-by-node.*
