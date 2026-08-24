# Architecture Invariants

These rules are strictly non-negotiable for anyone developing or modifying the StudyLab system.

1. **StudyLab is not a flashcard system.** It is a procedural problem-solving engine.
2. **Do not recreate Anki's responsibilities.** FSRS handles scheduling. Anki handles flashcards. `collection.anki2` holds standard data. StudyLab intelligence lives strictly in `procedural.db` and the procedural backend.
3. **Reviewer UI follows problem-solving workflow.** It is a workspace (loading -> ready -> solving -> submitting -> feedback), not a card flip (front -> back).
4. **Correct answer modality must match learning-object semantics.** `mcq` means actual options. `problem` implies calculation or derived steps.
5. **Generic fill-in input is not a universal modality.** Text inputs must be semantically validated (math equivalence, unit equivalence), not just string matched.
6. **Stepwise uses canonical semantic validation.** Intermediate steps are part of the `inline_contract` and must be validated, not just matched against static strings.
7. **Learner state is unified.** `SkillState` tracking and progressions (`Learning` to `Mastered`) operate identically regardless of whether the content came via `inline_contract` or `content_ref`.
8. **Domain evidence is diagnostic, not fake precision.** An execution error is fundamentally different from a conceptual representation error. Ensure mistakes map accurately to `DomainEvidence`.
9. **APKG/content owns definitions; runtime owns learner history.** The APKG is the static declarative blueprint. The runtime instantiates it.
10. **Ordinary new content should not require topic-specific Rust generators.** The `DeclarativeArchetypeGenerator` is the mold. All new STEM topics must use declarative constraints.
11. **Internal IDs/debug/remediation identifiers never leak to learners.** The learner only sees the pedagogical content.
12. **No duplicate interaction surfaces for one semantic action.** The TS frontend evaluates locally for speed, but canonical evaluation belongs to the Rust backend contract. Do not fork learning logic.
13. **Normal Anki cards remain untouched.** The interception in `render.rs` strictly targets `"StudyLab Procedural Anchor"` notes. Standard cards render normally.
14. **Diagnostic mode does not create a parallel learner model.** It is a measurement sweep to inform standard practice state.
15. **Tier 1 `inline_contract` is the preferred content resolution path.** Unless file size is explicitly prohibitive, package constraints directly into the APKG.
16. **`.agents/` is historical.** The `docs/` directory is the canonical source of truth for architecture.
