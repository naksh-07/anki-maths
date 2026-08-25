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

## 2026-08-25T19:59:05Z

# STUDYLAB — FINAL DOCUMENTATION + PRODUCT IDENTITY FREEZE
# NO CODE / NO BACKEND REDESIGN / NO APKG REGENERATION

Working directory: c:\Users\Suraj\Documents\Antigravity\Anki-maths
Integrity mode: development

## MISSION
Perform ONE final documentation and product-identity hardening pass across the StudyLab repository.

The product must be unambiguous to a fresh human developer or AI agent:
StudyLab is a procedural problem-solving and adaptive learning layer hosted inside the Anki runtime.
It is NOT an Anki flashcard application.
It is NOT a flashcard deck project.
It is NOT a generic quiz addon.
It is NOT intended to replace Anki's existing declarative recall system.

The repository documentation, packaging/install instructions, project language, and frontend product documentation must consistently reflect this identity.

THIS IS DOCUMENTATION / PROJECT-METADATA HARDENING ONLY.

DO NOT MODIFY:
- Rust procedural engine behavior
- scheduler/FSRS logic
- database schema
- APKG content generation
- frontend implementation
- Qt reviewer behavior
- existing Anki upstream runtime semantics

Only modify documentation, project metadata, installation/distribution documentation, examples, manifests, and other clearly human-facing project-description artifacts where appropriate.

============================================================
1. READ THE WHOLE DOCUMENTATION SURFACE
============================================================
Read:
CLAUDE.md, README.md, PROJECT.md, ORIGINAL_REQUEST.md
Everything under: docs/
Also inspect:
- package manifests
- install/build instructions
- distribution/package documentation
- contributor/developer documentation
- scripts that print user-facing project/package names
- top-level project metadata
- examples
- setup instructions
- release instructions
- APKG documentation
- screenshots/evidence documentation
- architecture diagrams
- project descriptions

============================================================
2. GLOBAL PRODUCT IDENTITY AUDIT
============================================================
Search the repository documentation and human-facing project text for language that incorrectly describes this repository as:
- an Anki flashcard app
- a flashcard deck
- a math flashcard project
- an Anki addon whose primary purpose is flashcards
- a card-flip/reveal product
- a generic study deck
- a replacement for Anki
- an Anki clone

Also search for ambiguous names such as: "Anki Maths", "Anki flashcards", "math flashcard", "flashcard app", "flashcard system", "deck app".
Classify occurrences:
A. MUST UPDATE: Human-facing project/product description that misidentifies StudyLab.
B. KEEP AS-IS: Accurate description of Anki upstream/runtime semantics or historical compatibility statement that is technically necessary.
C. CONTEXTUAL: Historical/reference text that should be clarified rather than rewritten.
Do not blindly replace the word "Anki".

============================================================
3. CANONICAL PRODUCT LANGUAGE
============================================================
Establish one canonical vocabulary:
Preferred: StudyLab, StudyLab procedural engine, StudyLab procedural learning layer, StudyLab problem-solving workspace, StudyLab adaptive problem practice, StudyLab diagnostic learning system.
Use: "hosted inside the Anki desktop runtime" when the relationship with Anki is relevant.
Define:
ANKI: Host runtime + collection + scheduler + normal declarative flashcard system.
STUDYLAB: Procedural problem-solving + semantic evaluation + diagnosis + remediation + adaptive problem selection.

============================================================
4. THE HARD PRODUCT BOUNDARY
============================================================
Add this canonical boundary to the appropriate top-level docs:
Anki already solves: declarative recall, flashcards, card scheduling, spaced repetition, standard Basic/Cloze review.
StudyLab solves: procedural problem solving, quantitative practice, reasoning, physics/chemistry numerical work, strategy selection, stepwise derivation, mistake diagnosis, weakness modeling, remediation, next-problem selection.
The repository must NEVER imply that StudyLab's primary job is to create another flashcard experience.

============================================================
5. FRONTEND IDENTITY
============================================================
The frontend documentation must define:
StudyLab UI = problem-solving workspace inside Anki
NOT: flashcard UI, card flip UI, generic quiz website, embedded web-app dashboard.
Freeze the learner loop:
PROBLEM → ONE CORRECT INTERACTION → MINIMAL FEEDBACK → DIAGNOSIS ONLY WHEN USEFUL → ONE CLEAR NEXT ACTION
Behind the scenes:
ATTEMPT → EVIDENCE → SKILL UPDATE → WEAKNESS SIGNAL → REMEDIATION → NEXT USEFUL PROBLEM

============================================================
6. APKG IDENTITY
============================================================
Clarify that APKGs used by StudyLab are packaging/import artifacts for the StudyLab procedural content universe. They are NOT the product itself.
Document: canonical full-universe APKG, why it exists, what it contains, how it is generated, how it is validated, why it is imported into Anki, how StudyLab consumes the procedural anchors.
Do NOT describe APKGs as the "StudyLab flashcard deck".
Preferred wording: "StudyLab procedural content package for the Anki host runtime."

============================================================
7. INSTALLATION / PACKAGING IDENTITY
============================================================
Audit all installation and setup documentation. Clarify that StudyLab installation/use is a distinct project workflow from installing standard Anki.
Explicit separation: Anki host/runtime + StudyLab procedural content/runtime integration.
Do NOT claim: "Install Anki Maths as a flashcard package."

============================================================
8. PACKAGE / DISTRIBUTION NAMING AUDIT
============================================================
Where the artifact is StudyLab-specific, use StudyLab terminology.
Audit APKG filenames in documentation, release filenames, package descriptions, installer descriptions, project titles, etc.

============================================================
9. ANKI REFERENCES: DO NOT OVER-CORRECT
============================================================
Do NOT replace every occurrence of "Anki". KEEP accurate statements about Anki source tree, Anki reviewer, Anki collection, Anki FSRS, Anki QtWebEngine, Anki APIs, Anki note models, Anki Basic/Cloze behavior, upstream Anki files.
Update only the HUMAN PRODUCT IDENTITY around them.

============================================================
10. DB / DATA DOCUMENTATION IDENTITY
============================================================
Separate:
ANKI DATA: collection + standard card/scheduling state
STUDYLAB DATA: procedural attempt evidence, skill evidence, mistake classification, weakness signals, remediation evidence, problem-generation evidence, diagnostic state.
Freeze: SOURCE OF TRUTH, OWNER, PERSISTENCE, DERIVED/CACHED, WRITER, READER for each major data category.

============================================================
11. 175 VS 177 TERMINOLOGY
============================================================
175 = canonical curriculum/topic universe
177 = current generated procedural learning objects/cards, if this remains the validated repository count.

============================================================
12. FINAL FRONTEND CONTRACT
============================================================
Freeze canonical principles:
1. Problem, 2. One correct interaction, 3. Minimal feedback, 4. Diagnosis only when useful, 5. One clear next action.
Engine layer: 6. Learn from attempt → update evidence → choose next useful problem.

============================================================
13. FINAL BUTTON CONTRACT
============================================================
Ensure documentation defines every relevant control: Quick Solve, Step-by-Step Solve, Submit, Check Solution, Add Step, Request Hint, Reset, Try Similar Problem, Next Problem, Silly Slip, Pattern Missed, Concept Gap, Prereq Unknown, Show Answer, Again, Hard, Good, Easy, More.
For each: owner, state, modality, purpose, priority, keyboard, transition, allowed coexistence, forbidden coexistence.

============================================================
14. FINAL VISUAL CONTRACT
============================================================
Convert screenshots-derived anti-patterns into hard product rules.
Reject: flashcard-like reveal UI, generic quiz-form feel, giant result cards, giant green/red dashboards, duplicate expected answer, duplicate time metrics, target time + elapsed time + speed badge clutter, raw schema/debug labels, unnecessary variant labels, duplicated transition controls, telemetry dumps, unnecessary stacked panels.
Problem/content remains the visual hero.

============================================================
15. FINAL APKG CONTRACT
============================================================
Freeze documentation for canonical package, allowed regression fixture, note model, field order, ProceduralPayload, object_type, metadata, parameters, constraints, answer derivation, solution graph, hints, provenance, taxonomy, remediation metadata.

============================================================
16. FINAL CROSS-LAYER CONTRACT
============================================================
Definitive map: APKG → RUST → DATABASE → FRONTEND → ANKI.
For each concept: source of truth, owner, persistence, consumer, learner-visible effect.

============================================================
17. DOCUMENT HIERARCHY
============================================================
Ensure these exist and are canonical:
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

============================================================
18. REPOSITORY-WIDE HUMAN-FACING DOC AUDIT
============================================================
Audit README, project overview, architecture summaries, installation, development setup, release documentation, contributor guide, examples, package metadata, artifact descriptions, script help text, test walkthroughs, screenshot captions, APKG instructions, project descriptions, planning documents.

============================================================
19. KEEP / UPDATE / IGNORE REPORT
============================================================
Produce a table: FILE, REFERENCE, ACTION, REASON (UPDATE, KEEP, IGNORE).

============================================================
20. FRESH-AI PRODUCT IDENTITY SELF TEST
============================================================
Test questions:
- What is StudyLab?
- What is Anki?
- What does Anki own?
- What does StudyLab own?
- Is StudyLab a flashcard app?
- Why does StudyLab use APKG?
- What belongs in the StudyLab DB?
- What does the frontend show?
- What is the learner interaction loop?
- Why is the UI intentionally minimal?
- How does StudyLab diagnose weakness?

============================================================
21. GIT / CHANGE SAFETY
============================================================
Ensure no Rust/TS/Python/Qt behavioral code changed, no DB schema changed, no APKG generated, no production data deleted. Only documentation/project-description/metadata changes.

============================================================
22. FINAL VERDICT
============================================================
Return:
🟢 STUDYLAB DOCUMENTATION + PRODUCT IDENTITY FROZEN
or
🟡 DOCUMENTATION STILL HAS AMBIGUITIES

