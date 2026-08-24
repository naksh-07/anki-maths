# Architecture Invariants

These rules are strictly non-negotiable for anyone developing, verifying, or modifying the StudyLab subsystem.

1. **StudyLab is not a flashcard system.** It is an adaptive procedural problem-solving engine.
2. **Do not recreate Anki's responsibilities.** FSRS handles scheduling. Anki handles flashcards. `collection.anki2` holds standard collection data. StudyLab intelligence lives strictly in `procedural.db` and `rslib/procedural/`.
3. **Reviewer UI follows problem-solving workflow.** It is an active workspace (`loading` $\rightarrow$ `ready` $\rightarrow$ `solving` $\rightarrow$ `submitting` $\rightarrow$ `feedback` $\rightarrow$ `next`), not a card flip/reveal workflow (`front` $\rightarrow$ `back`).
4. **Correct answer modality must match learning-object semantics.** `mcq` means actual radio buttons with keyboard hotkeys (`1`–`4`, `A`–`D`). Numerical means dimensional input with unit registry checks. Stepwise means multi-node deduction.
5. **Generic fill-in input is not a universal modality.** Inputs must be semantically validated (mathematical equivalence, dimensional consistency), not just string matched.
6. **Stepwise uses canonical semantic validation.** Intermediate steps are part of the `inline_contract` and are validated against `StepValidator` (`rslib/procedural/src/problems/steps/step_validator.rs`).
7. **Learner state is unified.** `SkillState` tracking and composite mastery progressions (`New` $\rightarrow$ `Mastered`) operate identically regardless of whether the content came via `inline_contract` or `content_ref`.
8. **Domain evidence is diagnostic, not fake precision.** An execution error is fundamentally different from a conceptual representation error. Mistakes map accurately to `DomainEvidence` (`is_execution_error()` vs `is_conceptual_error()`).
9. **APKG/content owns definitions; runtime owns learner history.** The APKG is the static declarative blueprint. The runtime instantiates it and records attempts in `procedural.db`.
10. **Ordinary new content should not require topic-specific Rust generators.** The `DeclarativeProblemGenerator` (`rslib/procedural/src/problems/declarative.rs`) is the universal procedural mold. All standard STEM topics must use declarative constraints in Python (`tools/studylab_content_factory.py`).
11. **Internal IDs/debug/remediation identifiers never leak to learners.** The learner only sees cleanly formatted pedagogical content.
12. **No duplicate interaction surfaces for one semantic action.** The TS frontend evaluates locally for zero-latency feedback, but canonical evaluation belongs to the Rust backend contract. Do not fork learning logic.
13. **Normal Anki cards remain completely untouched.** The interception in `rslib/src/notetype/render.rs:123` strictly targets notes starting with `"StudyLab Procedural Anchor"`. Standard cards render normally.
14. **Diagnostic mode does not create a parallel learner model.** It is a measurement sweep to inform standard practice state in `skill_states`.
15. **Tier 1 `inline_contract` is the preferred content resolution path.** Unless file size is explicitly prohibitive, package constraints directly into the APKG for full portability.
16. **`.agents/` is historical.** The `docs/` directory is the canonical source of truth for architecture.
