# Learning Model

The Learner Model is a cognitive model that tracks proficiency across a structured hierarchy and measures multidimensional diagnostic signals based on Domain Evidence.

## Core Hierarchy
1. Subject
2. Chapter
3. Topic
4. Problem Family
5. Pattern / Variant
6. Difficulty
7. Attempt

## Diagnostic Dimensions (Domain Evidence)

The backend (`rslib/procedural/src/skills/domain_evidence.rs`) captures strongly-typed diagnostic signals.

### Mathematics (`MathEvidence`)
- **pattern_recognition**: Identifying the correct form or theorem.
- **method_selection**: Choosing the right strategy.
- **execution**: Accurate calculation (e.g., arithmetic slip).
- **verification**: Checking answers against bounds.
- **structural_transfer**: Applying the concept to an unfamiliar variant.

### Logical Reasoning (`ReasoningEvidence`)
- **pattern_recognition**
- **representation**
- **constraint_extraction**
- **decision_path**
- **deduction**
- **trap_checking**
- **structural_transfer**

### Physics (`PhysicsEvidence`)
- **physical_model_selection**: Choosing correct principles (e.g., kinematics vs energy).
- **representation**: Free-body diagrams, vectors.
- **governing_principle**: Valid equation selection.
- **equation_setup**: Accurate substitution.
- **calculation**: Math execution.
- **unit_validity**: Dimensional consistency.
- **boundary_validity**: Physical realism.
- **verification**: Sensibility checks.
- **transfer**

### Chemistry (`ChemistryEvidence`)
*Divided into Physical, Organic, Inorganic subclasses.*
- **Physical**: model_setup, equation_selection, intermediate_quantity, calculation, conservation, verification, transfer.
- **Organic**: substrate_recognition, mechanism_pathway, reagent_interpretation, product_prediction, exception_handling, transfer.
- **Inorganic**: trend_reasoning, exception_handling, qualitative_reasoning, transfer.

## Interpretation of Signals

- **What they mean:** They provide a precise cognitive taxonomy of *why* an attempt failed. An execution error (math slip) means the concept is understood but mechanics failed. A representation error means the concept itself is weak.
- **What they are NOT:** They are not a second spaced repetition model. They do not dictate "when" a card is seen next (FSRS does that); they dictate "what" the learner sees next (Remediation).
- **Downstream Consumer:** Consumed by `MasteryEvidence`, which feeds `SkillState` and `ProgressionPolicy`. For example, conceptual errors demote the state from `Fluent` to `Learning`, while execution errors might only trigger a `ProceduralVariant` with simpler numbers.
