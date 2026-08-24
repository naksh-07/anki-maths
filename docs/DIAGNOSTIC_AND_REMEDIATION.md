# Diagnostic and Remediation

The diagnostic and remediation system forms an automated closed loop ensuring learners do not plateau on foundational gaps or spin their wheels on unmastered concepts.

---

## The Closed Loop Pipeline

```text
Attempt (PracticeAttempt)
   │
   ▼
Evidence (MasteryEvidence & DomainEvidence)
   │
   ▼
Diagnosis (ErrorCategory & StepErrorType)
   │
   ▼
Remediation Policy (RemediationPolicy::evaluate)
   │
   ▼
Targeted Intervention (LearningObjectKind)
   │
   ▼
Remeasurement (record_attempt_outcome & SkillState Update)
```

---

## Adaptive Practice vs Diagnostic Session

Differentiated via the `PracticeObjective` enum (`rslib/procedural/src/practice/request.rs:71-92`):

1. **`PracticeObjective::Practice` (Normal Adaptive Practice):**
   - Focuses on progressive fluency reinforcement, adaptive difficulty escalation (Levels 1–5), and long-term retention.
   - Triggers targeted interventions immediately upon detected error patterns.
2. **`PracticeObjective::Diagnose` (Diagnostic Sweep):**
   - Executes a rapid assessment sweep across a chapter or topic hierarchy to isolate knowledge gaps.
   - Evaluates performance without aggressive mid-test adaptation.
   - Diagnostic mode is a **measurement tool**, not a parallel learner model. Results flush directly into the unified `SkillState` table.
3. **Additional Objectives:** `Learn` (initial concept acquisition), `Speed` (fluency & latency training), `Transfer` (structural variant testing), `Exam` (mixed-domain simulation), and `Mock` (timed test battery).

---

## Diagnostic Hierarchy

1. **Subject** (Mathematics, Physics, Chemistry, Reasoning)
2. **Chapter** (e.g., Work, Power & Energy; Organic Reaction Mechanisms)
3. **Topic** (e.g., Conservation of Mechanical Energy; Electrophilic Aromatic Substitution)
4. **Problem Family** (e.g., `family.physics.work_energy.mechanics`)

---

## Remediation Policy & Domain Mappings (`RemediationPolicy::evaluate`)

The remediation engine (`rslib/procedural/src/remediation/policy.rs:33-400`) maps diagnosed error signals to specific pedagogical interventions:

### 1. Domain-Specific Interventions

- **Mathematics:**
  - *Execution Slip (`m.execution == false`):* Emits `ProceduralVariant` with `"simpler_numbers"` and difficulty reset to isolate arithmetic mechanics.
  - *Method Selection Error (`m.method_selection == false`):* Emits `StrategyDrill` to practice choosing formulas without arithmetic load.
  - *Pattern Recognition Error (`m.pattern_recognition == false`):* Emits `ConceptCheck` to re-anchor the underlying algebraic identity.
- **Logical Reasoning:**
  - *Deduction / Decision Flaw (`r.deduction == false || r.decision_path == false`):* Emits `StrategyDrill`.
  - *Representation Error (`r.representation == false || r.constraint_extraction == false`):* Emits `RepresentationDrill` (practicing spatial grids or truth trees).
- **Physics:**
  - *Unit / Dimensional Slip (`p.unit_validity == false`):* Emits `ProceduralVariant` ("unit_conversion") or `DeclarativeRecall` if repeated.
  - *Calculation Error (`p.calculation == false`):* Emits `ProceduralVariant` ("simpler_numbers").
  - *Governing Principle Error (`p.governing_principle == false`):* Demotes state and triggers `ConceptCheck`.
- **Chemistry:**
  - *Intermediate Quantity Error (`intermediate_quantity == false`):* Emits `ProceduralVariant` ("guided_steps") for multi-step stoichiometry/ICE tables.
  - *Mechanism / Trend Error (`mechanism_pathway == false || trend_reasoning == false`):* Emits `ConceptCheck`.

---

## Canonical Escalation Hierarchy

When errors recur within a short sliding window, `RemediationPolicy` escalates intervention severity:

```text
Recurrence 1–2  ──> Local Variant / Concept Check (Targeted Drill)
Recurrence 3    ──> Worked Example (Explanatory Modeling with Acknowledgement)
Recurrence 4    ──> Prerequisite Review (Critical Urgency: Drops 1 Dependency Level)
Recurrence >= 5 ──> Circuit Breaker (Advisory Urgency: Cooldown & Halts Session)
```

1. **Recurrence == 3 (Worked Example):**
   - Emits `RemediationActionKind::WorkedExample` (`RemediationUrgency::Critical`).
   - Presents full expert derivation; requires learner acknowledgement before queuing a transfer retry.
2. **Recurrence == 4 (Prerequisite Review):**
   - Emits `RemediationActionKind::PrerequisiteReview` (`RemediationUrgency::Critical`).
   - Traverses the dependency graph downward to review the immediate foundational prerequisite family.
3. **Recurrence >= 5 (Circuit Breaker):**
   - Emits `RemediationActionKind::CircuitBreaker` (`RemediationUrgency::Advisory`).
   - Halts practice on the specific problem family to prevent frustration and "wheel-spinning". Places the family on cooldown until the next study session.
