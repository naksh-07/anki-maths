# StudyLab Learning Model

StudyLab explicitly tracks deep cognitive vectors rather than standard binary "pass/fail" flashcard logic. 

## 1. Learner Progression Pipeline
A learner moves through a strict taxonomic progression:
**Subject** $\rightarrow$ **Chapter** $\rightarrow$ **Topic** $\rightarrow$ **Problem Family** $\rightarrow$ **Pattern** $\rightarrow$ **Difficulty** $\rightarrow$ **Attempt** $\rightarrow$ **Error** $\rightarrow$ **Domain Evidence** $\rightarrow$ **Transfer** $\rightarrow$ **Remediation** $\rightarrow$ **Mastery**

### Mastery Phases
Procedural progression is tracked in `SkillState` via distinct phases:
- `New` $\rightarrow$ `Learning` $\rightarrow$ `Fluent` $\rightarrow$ `Variation` $\rightarrow$ `Transfer` $\rightarrow$ `Mastered` $\rightarrow$ `Hibernating`

## 2. Mastery Evidence & Speed Quadrant
Every attempt logs a `PracticeAttempt` capturing `MasteryEvidence`. This includes:
- **Decision Quality** & **Independence** (Did they need hints?)
- **Speed Quadrant Evaluation:** Balances accuracy against latency benchmarks (`MovingLatencyStats`).
  - ⚡ *Fluency Strength* (Accurate & Fast)
  - ⏱ *Speed Opportunity* (Accurate but Slow)
  - ⚠️ *Strategy Trap* (Fast but Incorrect)
  - 💡 *Concept Setup* (Slow & Incorrect)

## 3. Domain-Specific Evidence Model
StudyLab analyzes cognitive performance specific to the rules of four major disciplines:

1. **Mathematics (`MathEvidence`)**:
   Evaluates `pattern_recognition`, `method_selection`, `execution`, and `structural_transfer`. Backed by a semantic `StepValidator` and symbolic CAS.
2. **Reasoning (`ReasoningEvidence`)**:
   Evaluates `constraint_extraction`, `decision_path`, `deduction`, and `trap_checking`.
3. **Physics (`PhysicsEvidence`)**:
   Evaluates `physical_model_selection`, `equation_setup`, `calculation`, and `unit_validity`. Pedagogically enforces dimensional analysis ($[M]^m [L]^l [T]^t$).
4. **Chemistry (`ChemistryEvidence`)**:
   Evaluates `conservation` (stoichiometry), `intermediate_quantity`, and `mechanism_pathway`.

---
### Traceability & Code Evidence
- **Progression & States:** Defined in `rslib/procedural/src/skills/signals.rs`.
- **Domain Vectors:** Structs defined in `rslib/procedural/src/skills/domain_evidence.rs` (`MathEvidence`, `PhysicsEvidence`, etc.).
- **Evaluation Taxonomy:** Helper methods like `is_execution_error()` and `is_conceptual_error()` are actively used to drive state changes.
