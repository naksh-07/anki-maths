# StudyLab Product Vision

## 1. Why does StudyLab exist?
StudyLab exists to solve the problem of mastering complex procedural skills that cannot be acquired through rote memorization alone. It provides a dedicated, adaptive problem-solving environment where the focus is on *how* a learner solves a problem, rather than merely *whether* they remembered a static fact.

## 2. What problem does it solve?
Traditional spaced repetition systems (like Anki) are optimized for declarative memory (vocabulary, facts, geography). However, procedural knowledge—such as deriving a mathematical proof, balancing a chemical equation, or applying physical laws—requires structured practice, step-by-step validation, and targeted cognitive remediation. StudyLab bridges this gap by offering a high-performance procedural engine inside a spaced repetition host.

## 3. Who is it for?
StudyLab is for STEM learners, students, and autodidacts who are studying:
- Mathematics (calculus, algebra)
- Reasoning & Logic
- Physics (numericals, dimensional analysis)
- Chemistry (stoichiometry, organic mechanisms)

## 4. What does it help the learner get better at?
It helps the learner develop **procedural mastery and transfer**. By tracking the exact nature of their mistakes (e.g., conceptual gaps vs. careless slips) and analyzing execution speed, StudyLab helps learners identify hidden weaknesses, improve their problem-solving fluency, and successfully transfer concepts to isomorphic or structurally novel problems.

## 5. What does it deliberately NOT do?
- **StudyLab is not an Anki replacement.** It relies entirely on Anki for user data persistence, deck organization, and top-level scheduling (FSRS/SM-2).
- **StudyLab is not a flashcard app.** It does not test simple recall. It does not use standard text-based front/back flipping.
- **StudyLab is not a generic quiz app.** It is a deep pedagogical instrument designed specifically for structured STEM problem solving.

---
### Traceability & Code Evidence
- **Two-System Architecture:** Detailed in `02_product_reconciliation.md`.
- **Procedural Anchor Model:** Evidenced by the `StudyLab Procedural Anchor` note type which delegates rendering entirely to the Rust core (`rslib/src/notetype/render.rs:123`).
- **Domain Specificity:** The procedural engine explicitly models Math, Reasoning, Physics, and Chemistry capabilities (`rslib/procedural/src/skills/domain_evidence.rs`).
