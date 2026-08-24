# StudyLab Architecture Invariants

To prevent product drift and maintain the strict two-system architecture, the following invariants are absolute. No AI agent, contributor, or future feature may violate these rules.

## The 10 Hard Invariants

1. **StudyLab is not a flashcard system.** It is an environment for structured STEM problem-solving.
2. **Anki's flashcard functionality is not duplicated.** Anki remains the host environment.
3. **StudyLab does not create a second FSRS.** Anki’s FSRS/SM-2 remains the sole driver of *when* content is reviewed.
4. **StudyLab does not create a second learner model.** All data aggregates back to the canonical `SkillState` and `DomainEvidence`.
5. **Problem-solving intelligence belongs to StudyLab.** Generating, parsing, and validating procedural content is owned by the Rust core, entirely decoupled from Anki’s `render_card`.
6. **Ordinary content expansion must remain declarative.** New topics are compiled via `study-source-core` into `.apkg` files (`inline_contract`).
7. **New ordinary topics must not require topic-specific Rust generators.** The `DeclarativeArchetypeGenerator` serves as the universal procedural runtime.
8. **Reviewer UI follows a problem-solving workflow.** The UI must never regress into simple "Show Answer" flipping. It must support stepwise checking, numerical analysis, and mistake taxonomy tracking.
9. **Normal Anki cards remain untouched.** The interception layer (`render_procedural_anchor`) explicitly preserves all non-StudyLab notes.
10. **Internal implementation concepts never define the learner-facing product.** Product definition dictates technical implementation, not the reverse.

## Contradictions & Guardrails
- *Guardrail (Front-end/Back-end Taxonomies):* Be aware of taxonomy naming mismatches between TypeScript telemetry (`silly_mistake`, `concept_not_known`) and Rust backend constraints (`ErrorCategory::Careless`, `ErrorCategory::Concept`). Ensure proper translation when serializing bridging telemetry.
- *Guardrail (UI Validation Bypass):* The TS/Vite UI must rely on the Rust `StepValidator` for rich multi-step graph validation; local TS scalar evaluations must not replace authoritative backend parsing (`GAP-MOD-01` resolved in Phase 40).
