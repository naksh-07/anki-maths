# Original User Request

## 2026-08-25T19:08:00Z

# STUDYLAB — CANONICAL PRODUCT + FRONTEND + APKG + DATA CONTRACT
# DOCUMENTATION-FIRST / NO CODE CHANGES

Working directory: c:/Users/Suraj/Documents/Antigravity/Anki-maths
Integrity mode: development

## MISSION

Create the definitive documentation for StudyLab's:
- product vision
- learner UX
- frontend states and controls
- learning-object modalities
- APKG/content contract
- database/data ownership
- frontend ↔ backend ↔ APKG ↔ DB boundaries
- acceptance criteria

Backend/procedural engine is considered largely implemented and stable.
Do NOT redesign or rewrite the backend unless the documentation audit finds a real contract contradiction that must be documented.

THIS MISSION IS DOCUMENTATION ONLY.

DO NOT MODIFY:
- TypeScript
- Rust
- Python
- Qt reviewer
- database schema
- APKG generation
- CSS

First make the product contract unambiguous.
Implementation comes later.

============================================================
1. STUDYLAB NORTH STAR
============================================================

Canonical definition:
StudyLab is a procedural learning and problem-solving engine hosted inside Anki.

StudyLab is NOT:
- a flashcard application
- a card-flip/reveal system
- a generic quiz app
- a web application embedded inside Anki
- a replacement for Anki's spaced-repetition system
- a second flashcard database
- a dashboard that exposes backend telemetry

Anki already handles:
- declarative flashcards
- recall
- card scheduling
- FSRS
- collection management
- standard Basic/Cloze review

StudyLab handles:
- procedural problem solving
- parameterized problems
- numerical solving
- MCQ reasoning
- structured reasoning
- step-by-step solving
- conceptual diagnosis
- strategy diagnosis
- mistake classification
- remediation
- skill evidence
- weakness detection
- adaptive next-problem selection

The learner should feel:
"This is Anki, but it understands how I solve problems."

============================================================
2. CORE LEARNING LOOP
============================================================

The learner-facing experience MUST follow this priority:
1. PROBLEM
2. ONE CORRECT INTERACTION
3. MINIMAL FEEDBACK
4. DIAGNOSIS ONLY WHEN USEFUL
5. ONE CLEAR NEXT ACTION

Behind the scenes the engine additionally performs:
6. LEARN FROM THE ATTEMPT
   → update skill evidence
   → update weakness signals
   → update remediation evidence
   → select the next useful problem

The sixth stage is primarily engine/data behavior, NOT permanent UI clutter.

============================================================
3. WHAT STUDYLAB IS ACTUALLY TRYING TO LEARN ABOUT THE USER
============================================================

The documentation must explicitly define that StudyLab is not primarily tracking "Did the user remember a card?"
It is trying to understand:
- Which subject is weak?
- Which chapter is weak?
- Which topic is weak?
- Which concept/skill is weak?
- Which problem family is weak?
- Which solving strategy is weak?
- Is the error conceptual?
- Is it calculation-related?
- Is the reasoning/transfer step weak?
- Is the user too slow?
- Is the user fast but unreliable?
- Which prerequisites are missing?
- Which remediation should come next?
- Which similar/different problem should be shown next?

This hierarchy should be documented:
Subject → Chapter → Topic → Skill → Problem Family → Attempt Evidence → Error/Strategy Evidence → Remediation Decision

============================================================
4. LEARNER UX PRINCIPLE
============================================================

The frontend MUST NOT expose the entire diagnostic engine by default.
Engine data may be rich.
Learner UI must remain simple.

DEFAULT PRESENTATION:
Problem ↓ One correct interaction ↓ Minimal feedback ↓ Diagnosis only when useful ↓ One clear next action

Only reveal additional diagnostic information when it helps learning.

============================================================
5. LEARNING OBJECT CONTRACT
============================================================

Document the exact semantics and UI for:
problem, quick, mcq, stepwise, concept_check, strategy_drill, worked_example, declarative_recall, prerequisite_review

For each define:
- educational purpose
- learner goal
- presentation
- interaction
- allowed controls
- forbidden controls
- answer modality
- success state
- wrong state
- feedback state
- diagnosis rules
- remediation rules
- next action

Hard invariant:
SEMANTIC MODALITY MUST ALWAYS MATCH UI MODALITY.
Never use a generic textbox as a fallback for MCQ, ConceptCheck, StrategyDrill, WorkedExample.
Numerical free-answer remains valid only for genuinely numerical/free-answer objects.

============================================================
6. FRONTEND STATE CONTRACT
============================================================

Define every learner-visible state:
loading, ready, solving, submitting, mistake_classification, feedback, next, plus any object-specific states.

For each state define:
- visible content
- visible controls
- hidden controls
- primary CTA
- secondary CTA
- keyboard behavior
- native Anki controls
- transition
- backend event
- learner-visible data
- hidden engine data

============================================================
7. BUTTON/CONTROL CONTRACT
============================================================

Create one canonical button matrix.
For every control define: exact label, purpose, object types, states, priority, ownership, location, keyboard shortcut, transition, telemetry, coexistence rules, forbidden combinations.
Controls include: Quick Solve, Step-by-Step Solve, Submit, Check Solution, Add Step, Request Hint, Reset, Try Similar Problem, Next Problem, 1 Silly Slip, 2 Pattern Missed, 3 Concept Gap, 4 Prereq Unknown, Again, Hard, Good, Easy, Show Answer, More.
Explicitly define mutually exclusive control sets.

============================================================
8. NATIVE ANKI BOUNDARY
============================================================

Create an exact responsibility matrix.
ANKI OWNS: collection, FSRS, scheduling, normal Basic/Cloze cards, standard flashcard review, standard Anki controls where appropriate.
STUDYLAB OWNS: procedural interaction, evaluation, semantic feedback, diagnosis, remediation, procedural attempt evidence, next-problem selection.
Define state-by-state visibility of Show Answer, Again, Hard, Good, Easy, Next Problem. No duplicate interaction ownership.

============================================================
9. VISUAL PRODUCT CONTRACT
============================================================

Define canonical design rules. The problem/content must be the visual hero.
The UI should feel: native, calm, minimal, focused, professional, dense enough for reasoning, free of unnecessary chrome.
Avoid: giant card wrappers, web-widget appearance, excessive shadows, excessive badges, raw schema names, debug labels, telemetry dumps, repeated information, stacked giant panels.
Define rules for header, breadcrumbs, badges, typography, spacing, problem area, answer area, feedback, hints, diagnosis, solution, next action.

============================================================
10. DIAGNOSTIC UI CONTRACT
============================================================

Classify every diagnostic field computed by the engine:
ENGINE ONLY / LEARNER OPTIONAL / LEARNER AFTER ERROR / LEARNER IN DIAGNOSTIC MODE / NEVER DISPLAY.
Do NOT automatically show every metric.

============================================================
11. WRONG-ANSWER CONTRACT & 12. CORRECT-ANSWER CONTRACT
============================================================

Define calm, minimal, pedagogically useful flows for both correct and wrong answers.

============================================================
13. STEPWISE CONTRACT, 14. MCQ CONTRACT, 15. WORKED EXAMPLE CONTRACT
============================================================

Exhaustively document each modality's distinct UI rules and control surface.

============================================================
16. APKG CONTRACT & 17. APKG HYGIENE
============================================================

Definitive canonical APKG specification: schema, fields, ProceduralPayload, object_type, parameters, constraints, derivation, step_nodes, provenance, hints, metadata.
Single canonical full-universe APKG policy.

============================================================
18. APKG → BACKEND → DB → FRONTEND CONTRACT
============================================================

Cross-layer mapping matrix for all fields and direction of ownership.

============================================================
19. DATABASE / DATA OWNERSHIP CONTRACT
============================================================

Separate Anki collection data from StudyLab procedural data. Define durable vs derived vs cached vs reproducible data.

============================================================
20. PRODUCT DIAGNOSTIC VISION
============================================================

Aggregated diagnostic intelligence hierarchy for future mock tests/weakness modeling.

============================================================
21. SCREEN-BY-SCREEN ACCEPTANCE CONTRACT & 22. CURRENT GAP MAP
============================================================

Screen-by-screen expected states and screenshot-grounded gap map.

============================================================
23. "PERFECT WINDOW" DEFINITION
============================================================

Clear usability and acceptance criteria.

============================================================
24. DOCUMENT HIERARCHY
============================================================

Create/update:
- docs/STUDYLAB_PRODUCT_CONTRACT.md
- docs/FRONTEND_PRODUCT_SPEC.md
- docs/FRONTEND_UI_STATE_SPEC.md
- docs/FRONTEND_BUTTON_CONTRACT.md
- docs/FRONTEND_VISUAL_DESIGN_SPEC.md
- docs/APKG_CONTENT_CONTRACT.md
- docs/APKG_FRONTEND_CONTRACT.md
- docs/DATABASE_DATA_CONTRACT.md
- docs/FRONTEND_ACCEPTANCE_MATRIX.md
- docs/FRONTEND_CURRENT_STATE_GAP_MAP.md

Index specifying which doc is authoritative for which question.

============================================================
25. CONTRADICTION CLEANUP & 26. FRESH-AI SELF TEST
============================================================

Resolve all contradictions across the documentation. Run the 12-question self-test to verify zero guesswork is needed.

## ACCEPTANCE CRITERIA
- All 10 documentation files created/updated in docs/ with zero code changes.
- All 27 mission sections thoroughly addressed.
- Final verdict clearly stated (GREEN or YELLOW).
