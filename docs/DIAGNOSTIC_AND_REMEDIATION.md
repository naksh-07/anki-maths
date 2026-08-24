# Diagnostic and Remediation

The diagnostic and remediation system forms a closed loop ensuring learners do not plateau on foundational gaps. 

## The Closed Loop
`attempt` → `evidence` → `diagnosis` → `remediation` → `targeted practice` → `remeasurement`

## Adaptive Practice vs Diagnostic Session
These are differentiated via the `PracticeObjective` in the Rust backend.
- **`PracticeObjective::Practice` (Normal Adaptive Practice):** Focuses on fluency reinforcement, standard skill progression, and long-term retention.
- **`PracticeObjective::Diagnose` (Diagnostic Session):** A rapid sweep across a hierarchy to isolate exact weaknesses. Diagnostic is a *measurement* tool, not a parallel learner model. It does not permanently bypass standard spaced repetition.

## Diagnostic Hierarchy
1. Subject
2. Chapter
3. Topic
4. Problem Family

## Diagnostic Dimensions
Within any problem family, failures are measured against dimensions:
- **Concept:** Core understanding (e.g., representation error, invalid physical model).
- **Execution / Calculation:** Mechanical accuracy (e.g., math slip, unit conversion error).
- **Transfer:** Ability to apply knowledge to structural variants.
- **Speed:** Latency vs expected fluency.

## Remediation Policy (`RemediationPolicy::evaluate`)

Remediation policies map specific failures to JIT (Just-In-Time) learning objects.

### 1. Execution Errors
- **Cause:** Identified via `DomainEvidence::is_execution_error()`. The concept is sound, but math/mechanics failed.
- **Action:** Triggers a `ProceduralVariant` with "simpler_numbers" and lower difficulty to quickly isolate and repair the mechanical slip.

### 2. Conceptual Errors
- **Cause:** Identified via `DomainEvidence::is_conceptual_error()`. The core principle is misunderstood.
- **Action:** Heavily demotes `SkillState` (e.g., from `Fluent` back to `Learning`). Triggers a `ConceptCheck` or `StrategyDrill`.

### 3. Escalation
- **Recurrence == 3:** If a learner fails conceptually 3 times in a short window, standard practice pauses and a canonical `WorkedExample` is shown.
- **Recurrence == 4 (Critical):** Triggers a JIT `PrerequisiteReview` to drop one step down the dependency tree.

### 4. Circuit Breaker
- **Recurrence >= 5:** Emits a `CircuitBreaker` action with an `Advisory` urgency. Halts practice on this family to prevent "wheel-spinning", placing it on a cooldown until the next session.
