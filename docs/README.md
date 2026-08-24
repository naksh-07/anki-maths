# StudyLab Canonical Documentation

## What is StudyLab?
StudyLab is a deep procedural problem-solving engine embedded inside Anki. It is designed for learners studying Mathematics, Reasoning, Physics, and Chemistry. It provides a structured workspace for problem-solving practice, error diagnosis, and adaptive remediation, measuring *how* a student solves a problem (accuracy, speed, conceptual gaps, transfer), not merely *whether* they remembered a static fact.

## What is it NOT?
- It is **NOT** a flashcard app or a generic quiz app.
- It is **NOT** a replacement for Anki. Anki serves as the underlying host environment and spaced repetition scheduler.
- It is **NOT** designed for declarative memory (e.g., vocabulary, geography).

## What should an AI agent read before changing it?
Before attempting to modify the StudyLab architecture or UI, you must internalize the product definition to prevent product drift. Read the canonical documentation in this order:

1. [PRODUCT_VISION.md](./PRODUCT_VISION.md): Understand the core purpose and audience.
2. [PRODUCT_BOUNDARIES.md](./PRODUCT_BOUNDARIES.md): Understand the division of labor between Anki and StudyLab.
3. [LEARNING_MODEL.md](./LEARNING_MODEL.md): Review how mastery and domain-specific evidence are tracked.
4. [CONTENT_MODEL.md](./CONTENT_MODEL.md): Understand the declarative `.apkg` contracts and the universal procedural runtime.
5. [REVIEWER_PHILOSOPHY.md](./REVIEWER_PHILOSOPHY.md): Read the UI constraints, including the Mistake Taxonomy Strip and modality parsing.
6. [DIAGNOSTIC_MODEL.md](./DIAGNOSTIC_MODEL.md): Learn how mock sessions measure weaknesses across four cognitive dimensions.
7. [ARCHITECTURE_INVARIANTS.md](./ARCHITECTURE_INVARIANTS.md): The absolute hard rules that must not be broken under any circumstances.
