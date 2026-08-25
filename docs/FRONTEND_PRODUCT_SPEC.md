# StudyLab Frontend Product Specification

## 1. PRIMARY PRODUCT TRUTH

StudyLab frontend is:
**A focused problem-solving workspace embedded in the Anki environment.**

It is NOT:
- a flashcard UI
- a card-flip interface
- a generic quiz page
- a generic form with different labels
- a second Anki reviewer

**Product Principle:**
The learner sees the minimum UI necessary to perform the intended learning interaction.
The CONTENT / LEARNING OBJECT is primary. The UI is subordinate to the learning task.

## 2. FRONTEND PRODUCT CONTRACT

- **PRIMARY ACTION:** Directly mapped to the learning objective of the current state (e.g., enter answer, classify mistake, proceed).
- **SECONDARY ACTIONS:** Scaffolding support (e.g., hint, mode toggle) hidden or subdued until needed.
- **FORBIDDEN ACTIONS:** Bypassing semantic evaluation, accidentally skipping mistake classification, duplicating Anki's scheduler logic.
- **VISUAL PRIORITY:** The problem prompt (math, physics, etc.) is the visual hero.
- **INFORMATION DENSITY:** Low cognitive load. Extraneous metadata and generic wrappers are strictly hidden during active solving.
- **STATE OWNERSHIP:** StudyLab owns the `solving`, `submitting`, `mistake_classification`, and initial `feedback` state. Anki owns the scheduler and the outer reviewer shell.

## 3. LEARNING OBJECT → UI CONTRACT

Do not call these "card types". They are StudyLab learning interactions.

### 1. `problem`
- **PURPOSE**: Test quantitative/symbolic procedural fluency.
- **LEARNER GOAL**: Produce correct final magnitude, unit, or algebraic expression.
- **PRIMARY MODALITY**: NumericalContainer or Quick Solve text input (toggleable to Stepwise).
- **VISIBLE ELEMENTS**: Question stem, active input container, submit button, mode switch tabs.
- **OPTIONAL ELEMENTS**: Hint button, steps list (if stepwise).
- **FORBIDDEN ELEMENTS**: MCQ options.
- **PRIMARY ACTION**: Type final answer and submit.
- **COMPLETION CONDITION**: Answer submitted and evaluated.
- **EVIDENCE GENERATED**: `AttemptResultPayload`.
- **NEXT STATE**: Correct → `feedback`; Incorrect → `mistake_classification`.

### 2. `mcq`
- **PURPOSE**: Rapid conceptual choice & zero-text input testing.
- **LEARNER GOAL**: Identify the correct option among distractors quickly.
- **PRIMARY MODALITY**: MCQContainer.
- **VISIBLE ELEMENTS**: Question stem, option list (`role="radiogroup"`).
- **OPTIONAL ELEMENTS**: Hint button.
- **FORBIDDEN ELEMENTS**: Free-text input, stepwise container, mode switch toggle.
- **PRIMARY ACTION**: Press 1-4, A-D, or click an option; press Enter.
- **COMPLETION CONDITION**: Valid option selected and evaluated.
- **EVIDENCE GENERATED**: `AttemptResultPayload`.
- **NEXT STATE**: Correct → `feedback`; Incorrect → `mistake_classification`.

### 3. `quick`
- **PURPOSE**: Fast-path for simple problems and strong fluency.
- **LEARNER GOAL**: Provide the direct final answer.
- **PRIMARY MODALITY**: Single text/numerical input field.
- **VISIBLE ELEMENTS**: Input container, submit button.
- **OPTIONAL ELEMENTS**: Live Preview Pill.
- **FORBIDDEN ELEMENTS**: Stepwise derivation inputs, MCQ list.
- **PRIMARY ACTION**: Type answer, press enter.
- **COMPLETION CONDITION**: Final answer validated.
- **EVIDENCE GENERATED**: `AttemptResultPayload`.
- **NEXT STATE**: Correct → `feedback`; Incorrect → `mistake_classification`.

### 4. `stepwise`
- **PURPOSE**: Multi-step algebraic CAS validation to assess reasoning step-by-step.
- **LEARNER GOAL**: Derive the final answer through logically valid intermediate steps.
- **PRIMARY MODALITY**: StepwiseContainer (Cognitive Tutor Inner Loop).
- **VISIBLE ELEMENTS**: Step inputs, validation statuses, Add Step button.
- **OPTIONAL ELEMENTS**: Progressive hints (3-tier).
- **FORBIDDEN ELEMENTS**: Quick solve single input.
- **PRIMARY ACTION**: Enter formula/step, press evaluate, proceed to next step.
- **COMPLETION CONDITION**: Final required step matches expected final answer.
- **EVIDENCE GENERATED**: `StepwiseValidationPayload` (validity per step, downstream consistency) → `AttemptResultPayload`.
- **NEXT STATE**: Step-by-step success/error feedback. MistakeFooter on final step failure.

### 5. `concept_check`
- **PURPOSE**: Diagnose specific misconceptions using targeted distractors.
- **LEARNER GOAL**: Select the conceptually correct statement/value.
- **PRIMARY MODALITY**: MCQContainer.
- **VISIBLE ELEMENTS**: Radio options.
- **OPTIONAL ELEMENTS**: Diagnostic feedback specific to distractors.
- **FORBIDDEN ELEMENTS**: Free-text input, Stepwise tools.
- **PRIMARY ACTION**: Select option.
- **COMPLETION CONDITION**: Option selected and submitted.
- **EVIDENCE GENERATED**: `AttemptResultPayload` (captures misconception).
- **NEXT STATE**: Display diagnostic feedback.

### 6. `strategy_drill`
- **PURPOSE**: Train optimal strategy selection (e.g., Energy vs Kinematics).
- **LEARNER GOAL**: Select the most efficient problem-solving strategy.
- **PRIMARY MODALITY**: MCQContainer.
- **VISIBLE ELEMENTS**: Radio options representing strategies.
- **OPTIONAL ELEMENTS**: Strategy optimality feedback.
- **FORBIDDEN ELEMENTS**: Free-text input, Stepwise tools.
- **PRIMARY ACTION**: Select strategy.
- **COMPLETION CONDITION**: Option selected and submitted.
- **EVIDENCE GENERATED**: Evaluates choice against `preferred_option_id`.
- **NEXT STATE**: Feedback on strategy optimality.

### 7. `worked_example`
- **PURPOSE**: Low-cognitive-load expert modeling for high-recurrence failure loops.
- **LEARNER GOAL**: Read and internalize the expert solution trace.
- **PRIMARY MODALITY**: WorkedExampleView / reading modality.
- **VISIBLE ELEMENTS**: Expert solution trace, mandatory acknowledgement gate button.
- **OPTIONAL ELEMENTS**: N/A.
- **FORBIDDEN ELEMENTS**: Inputs for solving (no quick/stepwise, no mcq).
- **PRIMARY ACTION**: Read trace and click "[ ✔ I Have Reviewed and Understood This Solution ]".
- **COMPLETION CONDITION**: Explicit acknowledgement clicked.
- **EVIDENCE GENERATED**: Acknowledgement logged (no mastery points).
- **NEXT STATE**: Queues a fresh `TransferRetry` variant immediately.

### 8. `declarative_recall`
- **PURPOSE**: Fallback or bridge to standard spaced repetition for factual knowledge.
- **LEARNER GOAL**: Recall fact mentally.
- **PRIMARY MODALITY**: Standard Anki card view.
- **VISIBLE ELEMENTS**: Tooltip or standard Anki text.
- **OPTIONAL ELEMENTS**: N/A.
- **FORBIDDEN ELEMENTS**: Complex procedural containers.
- **PRIMARY ACTION**: Mental recall and standard Anki flip.
- **COMPLETION CONDITION**: Displays tooltip / resolves target Anki card.
- **EVIDENCE GENERATED**: `DeclarativeRecallPayload`.
- **NEXT STATE**: Next Anki card.

### 9. `prerequisite_review`
- **PURPOSE**: Launch remedial practice for missing fundamental skills.
- **LEARNER GOAL**: Practice the missing prerequisite skill.
- **PRIMARY MODALITY**: Tooltip and remedial navigation.
- **VISIBLE ELEMENTS**: Tooltip "Practice Prerequisite: {target_skill_id}".
- **OPTIONAL ELEMENTS**: N/A.
- **FORBIDDEN ELEMENTS**: Current problem UI.
- **PRIMARY ACTION**: Triggered by system or user choice.
- **COMPLETION CONDITION**: Tooltip displayed, triggers remedial navigation.
- **EVIDENCE GENERATED**: `PrerequisitePracticePayload`.
- **NEXT STATE**: Loads prerequisite problem instance.

## 4. ANSWER MODALITY CONTRACT

- **MCQ**: Zero-text input fallback enforced. Keyboard accessible (1-4, A-D, Arrows). Evaluated against canonical ID or index.
- **Numerical**: Uses a 5D physical vector (`PhysicalDimension`) handling unit algebra & conversions (50+ units). Identifies tolerance modes.
- **Physics numerical**: Incorporates diagnostic traps like non-negativity checks, dimension incompatibility, and unit trap warnings (e.g., forgot 5/18 factor for km/h).
- **Chemistry numerical**: Parses/scales molar units, checking stoichiometry limits and representation constraints.
- **Structured / Reasoning**: Uses `MathSemanticComparator` for multi-tier evaluation (string normalization, linear equation equivalence, commutative addition).
- **Stepwise**: Validates intermediate reasoning steps via Rust `StepValidator`. Tracks downstream consistency to map logic over purely final values. Supports 3-tier progressive hints.
- **ConceptCheck**: Diagnostics map directly to distractors.
- **StrategyDrill**: Evaluates against `preferred_option_id`.

IMPORTANT: If the backend declares a modality, frontend MUST render that modality. Frontend must NOT infer modality merely because a DOM node exists or is absent.

## 5. GENERIC TEXT INPUT RULE

**"Type final answer..." is NOT a default StudyLab interaction.**

It may exist ONLY where the canonical learning-object modality is a free numerical/expression answer. It MUST NOT appear for:
- MCQ
- ConceptCheck
- StrategyDrill
- other structured choices

Never use generic fill-in-the-blank as a universal fallback. For MCQs, the `enforceZeroTextInputFallback()` rule must hide free-text input and disable mode switch toggles entirely.

## 6. PROCEDURAL STATE MACHINE

1. **`loading`**
   - **PURPOSE**: Initial constructor state.
   - **VISIBLE UI**: Loading spinner, container placeholder.
   - **ALLOWED/FORBIDDEN ACTIONS**: Automated bootstrap only. No student input.
   - **KEYBOARD**: Suppressed.
   - **BRIDGE EVENT**: None.
   - **BACKEND/PERSISTENCE EFFECT**: None.
   - **NEXT STATE**: `ready` or `error`.

2. **`ready`**
   - **PURPOSE**: Problem statement rendered; answer container focused.
   - **VISIBLE UI**: Problem prompt, active input container, mode switcher. Native Anki ease buttons hidden.
   - **ALLOWED/FORBIDDEN ACTIONS**: Reading prompt, focusing input. Cannot submit empty response.
   - **KEYBOARD**: Hotkeys armed (1-4, A-D, numeric, arrows).
   - **BRIDGE EVENT**: None.
   - **BACKEND/PERSISTENCE EFFECT**: Initializes active solving stopwatch.
   - **NEXT STATE**: `solving`.

3. **`solving`**
   - **PURPOSE**: Active problem-solving state with running stopwatch and live input evaluation.
   - **VISIBLE UI**: Prompt, active input fields, hint button.
   - **ALLOWED/FORBIDDEN ACTIONS**: Entering answers, selecting options. Space/Enter MUST NOT propagate to native Anki.
   - **KEYBOARD**: Enter/Ctrl+Enter, H/?, 1-4/A-D, Arrow keys.
   - **BRIDGE EVENT**: None during active typing.
   - **BACKEND/PERSISTENCE EFFECT**: Accumulates active solving time in ms.
   - **NEXT STATE**: `submitting` or `hint`.

4. **`hint`**
   - **PURPOSE**: Scaffolded hint display state.
   - **VISIBLE UI**: Expandable hint card with progressive content; "Resume Solving" button.
   - **ALLOWED/FORBIDDEN ACTIONS**: Reading hint. No direct answer auto-population.
   - **KEYBOARD**: Esc or Enter closes hint panel.
   - **BRIDGE EVENT**: `procedural_hint:<json>`.
   - **BACKEND/PERSISTENCE EFFECT**: Stores hint context, increments `hintsUsed`.
   - **NEXT STATE**: `solving`.

5. **`submitting`**
   - **PURPOSE**: Transient evaluation state.
   - **VISIBLE UI**: Inputs temporarily disabled; inline feedback active.
   - **ALLOWED/FORBIDDEN ACTIONS**: Client-side AST normalization. No input modification.
   - **KEYBOARD**: Input events suppressed.
   - **BRIDGE EVENT**: None.
   - **BACKEND/PERSISTENCE EFFECT**: Evaluates correctness score, captures time, computes speed quadrant.
   - **NEXT STATE**: `feedback` (correct) or `mistake_classification` (incorrect).

6. **`mistake_classification`**
   - **PURPOSE**: Metacognitive reflection immediately on incorrect attempts.
   - **VISIBLE UI**: Result panel, `MistakeFooter` strip with 4 classification buttons.
   - **ALLOWED/FORBIDDEN ACTIONS**: Student selects error category. No bypassing classification.
   - **KEYBOARD**: Space and Enter strictly trapped. Number keys 1-4 route to category selection.
   - **BRIDGE EVENT**: `procedural_mistake:<json>`.
   - **BACKEND/PERSISTENCE EFFECT**: Captures self-attribution, ingested into `DomainEvidence`.
   - **NEXT STATE**: `feedback` (after delay).

7. **`feedback`**
   - **PURPOSE**: Comprehensive outcome review state.
   - **VISIBLE UI**: Green/Red banner, canonical solution. Remediation buttons. Native Anki ease buttons revealed.
   - **ALLOWED/FORBIDDEN ACTIONS**: Reviewing derivation, proceeding, or starting remediation. Re-editing is forbidden.
   - **KEYBOARD**: Numbers 1-4 for native Anki ease ratings.
   - **BRIDGE EVENT**: `globalThis.anki.mutateNextCardStates(...)`, `procedural_attempt:<json>`, `ans`.
   - **BACKEND/PERSISTENCE EFFECT**: Stores attempt snapshot, queues telemetry for atomic write to `procedural.db`.
   - **NEXT STATE**: `worked_example` or `next`.

8. **`worked_example`**
   - **PURPOSE**: Guided review state ("Try Similar Problem").
   - **VISIBLE UI**: Step-by-step canonical solution, input fields suppressed, "Generate New Variant" button.
   - **ALLOWED/FORBIDDEN ACTIONS**: Studying expert derivation. Direct rating without reviewing is forbidden.
   - **KEYBOARD**: Enter or Space triggers new variant generation.
   - **BRIDGE EVENT**: `procedural_try_similar:<json>`.
   - **BACKEND/PERSISTENCE EFFECT**: Records worked example exposure in recurrence memory. Calls `_showQuestion()`.
   - **NEXT STATE**: `ready`.

9. **`next`**
   - **PURPOSE**: Lifecycle completion state.
   - **VISIBLE UI**: Smooth transition to next scheduled card.
   - **ALLOWED/FORBIDDEN ACTIONS**: Automated cleanup.
   - **KEYBOARD**: Native Anki review hotkeys resume.
   - **BRIDGE EVENT**: `procedural_answer:<ease>`.
   - **BACKEND/PERSISTENCE EFFECT**: Commits attempt to `procedural.db` and updates FSRS states via `_answerCard(val)`.
   - **NEXT STATE**: `loading` or standard Anki template.

10. **`error`**
    - **PURPOSE**: Fault-tolerant error boundary.
    - **VISIBLE UI**: Red warning banner with diagnostic error info.
    - **ALLOWED/FORBIDDEN ACTIONS**: User clicks "Skip Card". Crashing Anki webview is forbidden.
    - **KEYBOARD**: Native Anki shortcuts restored.
    - **BRIDGE EVENT**: Logs error to console and Python bridge.
    - **NEXT STATE**: `teardown`.

11. **`teardown`**
    - **PURPOSE**: Terminal cleanup state.
    - **VISIBLE UI**: Container unmounted.
    - **ALLOWED/FORBIDDEN ACTIONS**: Garbage collection. No delayed callbacks.
    - **KEYBOARD**: 100% restored to native Anki.
    - **BRIDGE EVENT**: Python `destroyActive()` webview cleanup hook.
    - **NEXT STATE**: Terminal state.

## 7. ONE-INTERACTION-SURFACE INVARIANT

**HARD RULE**: At any state, there must not be two visible controls that perform the same semantic action.

DO NOT show StudyLab Submit + another procedural Submit + another equivalent Anki action.
DO NOT show StudyLab Show Answer + another procedural Show Answer.
DO NOT show StudyLab rating + duplicate Anki rating.

*Violation Documented (See Gap Matrix)*: In the `feedback` state, the native Anki ease buttons are revealed via `ans`, but StudyLab also renders a custom "Next Problem" button. This violates the one-interaction-surface invariant.

## 8. ANKI BOUNDARY

**WHAT ANKI OWNS:**
- flashcards
- Basic/Cloze review
- scheduler
- FSRS
- standard review lifecycle
- `collection.anki2` database

**WHAT STUDYLAB OWNS:**
- procedural interaction
- problem solving
- evaluation semantics
- domain evidence
- mistake diagnosis
- remediation
- diagnostic learning
- `procedural.db` database

Anki is the HOST ENVIRONMENT. Anki does not define StudyLab's learner interaction.

## 9. NATIVE ANKI CONTROL VISIBILITY

- **Show Answer**: HIDDEN/INTERCEPTED during `solving`. (Currently VISIBLE due to Anki limitation; clicking it bypasses StudyLab and leaks state. Must be addressed or documented as a known defect to be intercepted differently).
- **Again, Hard, Good, Easy**: VISIBLE during `feedback`. DEFERRED during `solving`.
- **Edit**: VISIBLE.
- **More**: VISIBLE.
- **Timer**: VISIBLE (Anki's native session timer).

## 10. WRONG-ANSWER CONTRACT

Canonical flow:
`WRONG` → `short result` → `Mistake Classification` → `[1 Silly] [2 Pattern] [3 Concept] [4 Prerequisite]` → `telemetry` → `feedback / solution` → `remediation / next practice`

- **MUST BE VISIBLE**: The four mistake classification categories.
- **MUST BE HIDDEN**: Standard "Next" or "Show Answer" buttons until classified.
- **MUST BE RECORDED**: The selected mistake category (ingested into `DomainEvidence`).
- **SPACE / ENTER DOES**: Strictly trapped. Must not bypass classification.
- **1–4 DOES**: Routes to category selection.
- **BYPASS**: Accidental classification bypass is FORBIDDEN.

## 11. RESULT / FEEDBACK CONTRACT

Define the minimum content after evaluation:

- **Correct**: Concise correctness, useful solution context, next transition.
- **Wrong**: Concise error, reflection, then solution/remediation.

Avoid dumping target time, actual time, expected answer, raw metadata, multiple badges, or all possible diagnostics unless the state genuinely needs them. (Currently violated in implementation).

## 12. TIMER / METRICS CONTRACT

Metric used by learning engine != UI element shown to learner.
Target latency exists in the backend without becoming permanent visual chrome.
The ticking `proc-timer` updating every 200ms during solving violates cognitive scaffolding and induces anxiety. It must be implicit or subdued until the `feedback` state.

## 13. HEADER / METADATA CONTRACT

Maximum useful learner-facing metadata:
- **Preferred**: Domain, Chapter/Topic, Skill/Problem family where useful.
- **Optional**: Difficulty, provenance/PYQ.
- **Forbidden**: Raw schema IDs, family IDs, internal capability names, remediation action IDs, debug labels.
Every metadata element must justify its learner value.

## 14. VISUAL DENSITY CONTRACT

Content is visual hero. Controls are compact. No unnecessary card-within-card pattern. No decorative UI without semantic purpose. No repeated pills. No redundant footer. No giant dashboard-like panels. The StudyLab reviewer should feel like one coherent workspace.

## 15. QUICK SOLVE CONTRACT

Quick Solve is the fastest valid way to solve the current problem.
- It is NOT a generic text field fallback.
- **Required**: Modality-correct interaction, minimal controls, immediate submit/evaluation. Single text/numerical input field, with a `.proc-num-preview-pill` for magnitudes and normalized SI units below the field.

## 16. STEPWISE CONTRACT

Stepwise is a reasoning workspace.
- It should not feel like a form builder.
- **Documented**: Step representation (Cognitive Tutor Inner Loop), Add Step, Hint, Reset, Validation.
- **Canonical Evaluator**: Rust `StepValidator`. Validates intermediate reasoning steps, tracks `is_downstream_consistent` (`PartiallyValid`) to prevent catastrophic cascading score penalties.

## 17. MCQ CONTRACT

- Real options, selectable state.
- Keyboard accessible (1-4, A-D).
- Correct/incorrect feedback, option ordering, scoring, evidence generation.
- **NO generic text input.**

## 18. DOMAIN-SPECIFIC PRESENTATION

Define only meaningful domain-specific additions. Avoid decorative giant domain labels.
- **Mathematics**: Formulas / equations / structure. Algebraic step validation.
- **Reasoning**: Representation / constraints / relationships. Logic grids, decision paths.
- **Physics**: Units / physical quantities / equations. Free-body diagrams.
- **Chemistry**: Units / quantities / equations / reaction reasoning. ICE tables.

## 19. LEARNING INTERVENTIONS

ConceptCheck, StrategyDrill, WorkedExample, DeclarativeRecall, PrerequisiteReview, TransferRetry, ProceduralVariant.
They must feel like targeted learning interventions, not alternate flashcard skins.
(e.g., ConceptCheck maps specific diagnostics directly to distractors).

## 20. FRONTEND ↔ BACKEND CONTRACT

- **UI Validation**: Handled by TS frontend components (AST normalization, format checks).
- **Canonical Semantic Evaluation**: Handled by Rust backend / Python bridge.
- **Telemetry & Learner-State Persistence**: Handled by Rust backend (writes to `procedural.db`).
- **Bridge Events**: `procedural_hint:<json>`, `procedural_mistake:<json>`, `procedural_attempt:<json>`, `procedural_try_similar:<json>`, `procedural_answer:<ease>`, `ans`.

## 21. ACCESSIBILITY CONTRACT

- **Keyboard**: Full keyboard navigation (1-4, A-D, Space, Enter).
- **Focus**: Maintain logical focus flow. Visible focus rings.
- **ARIA**: Proper roles (`radiogroup`, etc).
- **No Accidental Keyboard Bypass**: Space/Enter strictly trapped during critical states (solving, mistake classification).
- **Preserve**: MCQ 1-4/A-D, mistake 1-4.

## 22. NORMAL ANKI REGRESSION CONTRACT

Normal Basic/Cloze cards remain normal Anki. StudyLab only activates for StudyLab procedural anchors (NoteType starts with "StudyLab Procedural Anchor"). No CSS bleed. No keyboard bleed. No footer corruption. No scheduler corruption. Webview safe teardown via `MutationObserver` and `destroyActive()`.

## 23. LIVE VISUAL ACCEPTANCE CONTRACT

What "good" looks like in live DEV desktop:
- Problem is visual hero.
- No flashcard illusion.
- No duplicate controls.
- Correct modality.
- Minimal chrome.
- No implementation leakage.
- Coherent state progression.

## 24. CODE TRACEABILITY

- `docs/REVIEWER_STATE_MACHINE.md`
- `docs/LEARNING_OBJECTS.md`
- `docs/FRONTEND_BACKEND_CONTRACT.md`
- `ts/reviewer/components/*`
- `ts/reviewer/procedural.ts`
- `qt/aqt/reviewer.py`

## 25. GAP MATRIX

| Requirement | Current Implementation | Match | Gap | Priority |
|---|---|---|---|---|
| One-Interaction-Surface (Next) (P0-B) | Single in-card "Next Problem" button inside `proc-result-panel`. Footer ease buttons suppressed for procedural cards. `Next Problem`, `Space`, or `Enter` advances card with calibrated ease (`1`..`4`) matching Rust rating policy. Numeric keys `1`..`4` in `feedback` state provide explicit override. Standard cards retain normal ease buttons with 0% regression. | 🟢 | Reconciled & Resolved with live desktop verification. | P0 (RESOLVED) |
| "Show Answer" Native Button Bypass (P0-A) | `#ansbut` button suppressed in bottom bar for procedural cards. Native Show Answer (`_showAnswer()`) delegates to `globalThis.anki.procedural.handleNativeShowAnswer()` without destroying DOM. Empty input triggers unassisted surrender (`isCorrect: false`), routing to `mistake_classification` with anti-bypass Space/Enter trapping before solution reveal. | 🟢 | Reconciled & Resolved with live desktop verification. | P0 (RESOLVED) |
| Modality-matched UI for non-Problem objects | Uses generic `MCQContainer` for ConceptCheck/StrategyDrill without distinct frontend behavior. | 🔴 | Frontend lacks specialization for specific learning interventions. | P1 |
| `WorkedExample` Frontend Component | Backend supports `WorkedExampleObject`, TS uses generic result panel. | 🔴 | Missing dedicated `WorkedExampleView` / `worked_example_container.ts`. | P1 |
| Result/Feedback Information Density | Dumps generic text blocks, raw time stats, multiple badges in `proc-result-panel`. | 🔴 | Information overload; violates minimalistic feedback contract. | P2 |
| Timer / Metrics Anxiety | `proc-timer` ticks every 200ms during `solving`. | 🔴 | Violates cognitive scaffolding (distracting). | P2 |
| Header / Metadata Density | Incorporates multiple badges, diff tags, variant tags (`proc-header`). | 🔴 | Looks like a developer dashboard. | P2 |
| Visual Density / Panel Stacking | Stacks quick, stepwise, hint, result, mistake panels vertically. Stepwise feels like generic form. | 🔴 | Excessive vertical UI chrome. | P1 |
| Domain-Specific Widgets | Uses generic `NumericalContainer` across domains. | 🔴 | Missing specific physics/chem/reasoning UI elements (e.g., ICE tables). | P2 |

---
**SPECIFICATION STATUS: P0 RESOLVED & LIVE VERIFIED**
