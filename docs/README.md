# StudyLab Canonical Documentation

Welcome to the StudyLab documentation. StudyLab is the procedural learning and problem-solving intelligence subsystem embedded within this Anki fork.

## What is StudyLab?
StudyLab is an adaptive problem-solving environment optimized for procedural STEM learning (Math, Physics, Chemistry, Reasoning). It evaluates *how* a learner solves a multi-step problem, tracks diagnostic error taxonomies (e.g., conceptual misunderstanding vs. careless execution), and automatically triggers targeted remediation (like simpler variants, worked examples, or prerequisite drills).

## What is it NOT?
- It is **not** a flashcard application or an Anki replacement.
- It is **not** a second spaced-repetition algorithm (it defers to FSRS).
- It is **not** a generic quiz app. 

## Who is it for?
Students and self-learners mastering quantitative or logically rigorous subjects where recognizing patterns and executing structural steps is required, rather than just memorizing static facts.

## What domains does it support?
- Mathematics
- Logical Reasoning
- Physics numericals/problem solving
- Chemistry (Physical, Organic, Inorganic)
- Other structured problem-solving domains fitting the declarative mold.

## How is it different from Anki?
Anki asks: *"Did you remember this answer?"*
StudyLab asks: *"Can you execute the steps to solve this problem, and if you failed, where exactly did your cognitive process break down?"*
StudyLab defers all flashcard scheduling to Anki but intercepts the review session to inject a rich, diagnostic problem-solving workspace. Its state (`SkillState`, `DomainEvidence`) is kept entirely isolated in `procedural.db`, while Anki retains `collection.anki2`.

## How does content enter?
Content enters declaratively. Python tooling compiles constraints, parameter domains, and templates into `.apkg` files containing `inline_contract`s. No new Rust backend code is required for ordinary new topics; the universal `DeclarativeProblemGenerator` (`rslib/procedural/src/problems/declarative.rs`) instantiates the problems dynamically.

## How does learning state work?
Learner attempts produce `DomainEvidence` (e.g., an execution math slip). This evidence informs `MasteryEvidence`, which updates long-term `SkillState`. A conceptual error heavily demotes state and triggers JIT remediation (like a `ConceptCheck`). Progression requires passing strict accuracy, speed, and transfer gates.

## Where should an AI agent start reading?
Agents should start by reading [ARCHITECTURE_INVARIANTS.md](ARCHITECTURE_INVARIANTS.md) to understand non-negotiable rules, and [DOCUMENTATION_MAP.md](DOCUMENTATION_MAP.md) to navigate the rest of this directory.

---

### Navigation
- [PRODUCT_VISION.md](PRODUCT_VISION.md)
- [PRODUCT_BOUNDARIES.md](PRODUCT_BOUNDARIES.md)
- [SYSTEM_ARCHITECTURE.md](SYSTEM_ARCHITECTURE.md)
- [LEARNING_MODEL.md](LEARNING_MODEL.md)
- [CONTENT_AND_AUTHORING.md](CONTENT_AND_AUTHORING.md)
- [LEARNING_OBJECTS.md](LEARNING_OBJECTS.md)
- [REVIEWER_STATE_MACHINE.md](REVIEWER_STATE_MACHINE.md)
- [FRONTEND_BACKEND_CONTRACT.md](FRONTEND_BACKEND_CONTRACT.md)
- [DIAGNOSTIC_AND_REMEDIATION.md](DIAGNOSTIC_AND_REMEDIATION.md)
- [ARCHITECTURE_INVARIANTS.md](ARCHITECTURE_INVARIANTS.md)
- [DOCUMENTATION_MAP.md](DOCUMENTATION_MAP.md)
- [OPEN_QUESTIONS.md](OPEN_QUESTIONS.md)
