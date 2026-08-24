# StudyLab Reviewer Philosophy

The StudyLab reviewer is not a flashcard with extra buttons; it is a dedicated **problem-solving workspace** embedded within Anki. 

## 1. Core Workflow
The reviewer lifecycle strictly follows a problem-solving progression:
`[loading] -> [ready] -> [solving] -> [submitting] -> [mistake_classification] -> [feedback] -> [next]`

## 2. Authentic Modalities
The system natively supports structured problem inputs, rejecting "synthetic text fallbacks":
- **MCQ / Structured:** Authentic selectable option buttons with 1-4 or A-D keyboard shortcuts (`ts/reviewer/components/mcq_container.ts`). Position bias is mitigated via deterministic shuffling.
- **Numerical:** Features unit-aware parsing and dimensional analysis, handling specific units, tolerances, and scientific notation (`NumericalParser.parseScalar`).
- **Stepwise:** Supports multi-line sequential derivations with individual step validation (`stepwiseContainerComponent.handleCheckSolution()`).

## 3. The Mistake Taxonomy Strip
To prevent passive learning and mere "Show Answer" reflex, StudyLab intercepts incorrect answers before revealing canonical solutions.

- **Mistake Classification Trap:** If an attempt is incorrect, the UI transitions to the `mistake_classification` state (`ts/reviewer/procedural.ts`).
- **Active Reflection:** A non-intrusive UI strip traps input events until the learner categorizes their failure context (`ts/reviewer/components/mistake_footer.ts`):
  1. `[1 Silly Slip]` (Arithmetic/calculation slip)
  2. `[2 Pattern Missed]` (Failed to identify structure)
  3. `[3 Concept Gap]` (Wrong formula/theorem)
  4. `[4 Prereq Unknown]` (Fundamental gap)

## 4. Remediation Dispatch
Once classified, the solution is fully revealed. For severe categories (Concept Gap, Prereq Unknown), the Python bridge (`qt/aqt/reviewer.py`) dispatches `proceduralRemediation` signals to trigger Concept Checks, Strategy Drills, or Prerequisite bridges, queued by the Rust `RemediationPrecedence` policy.

---
### Traceability & Code Evidence
- **State Machine:** `ts/reviewer/procedural.ts` and `02_product_reconciliation.md`.
- **Modality Handling:** `ts/reviewer/components/mcq_container.ts` and `NumericalParser.parseScalar`.
- **Mistake Trapping:** `ts/reviewer/components/mistake_footer.ts` and `rslib/procedural/src/practice/request.rs`.
