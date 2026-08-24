# Content and Authoring

The boundary between static content definitions and dynamic runtime execution is strictly maintained through the APKG content contract.

## Responsibility Separation

### Content Package (APKG) Owns:
- Problem families
- Archetypes
- Parameter domains (`IntegerRange`, `DerivedLinear`, etc.)
- Constraints (Logic rules for instantiated variables)
- Answer derivations and evaluators
- Solution graphs / templates
- Step nodes (explicit reasoning sub-steps and hint principles)
- Hints and variants
- Provenance and topic metadata

### Runtime (Rust backend + DB) Owns:
- Ephemeral generation of variables fulfilling constraints
- Learner attempts (`PracticeAttempt`)
- `SkillState` and `MasteryEvidence` history
- DomainEvidence tracking
- Remediation and targeted next practice
- Recurrence and queue state
- Integration with Anki's scheduling (FSRS)

## Content Resolution Precedence

When a `StudyLab Procedural Anchor` note is due, the backend resolves the payload via `rslib/procedural/src/service/mod.rs` in this strict precedence:

1. **`inline_contract`** (Tier 1)
   - The complete `DeclarativeFamilyContract` is fully bundled as JSON in the anchor. Evaluated first. Enables 100% self-contained deck portability with zero prior setup. **This is the preferred canonical path.**
2. **`content_ref`** (Tier 2)
   - Uses a string ID to resolve against pre-ingested `PracticeItem` rows in the local `procedural.db`. Keeps `.apkg` file size small but requires a sync/import step first.
3. **`legacy proc_schema`** (Tier 3)
   - Uses hardcoded string IDs dispatched to legacy unified practice engine components manually written in Rust. Must not be used for new content.

## Zero-Code Scalability

**Why APKG is self-contained where possible:** To allow StudyLab decks to be shared and executed dynamically on any machine without requiring massive database syncs.

**Why database-dependent content is optional:** It provides a fallback for extremely massive content repositories where bundling full JSON logic per card is prohibitive.

**Why new ordinary topics do not require new Rust generators:** The `DeclarativeArchetypeGenerator` acts as a universal procedural runtime mold. Content authors specify parameter domains and constraints declaratively in Python, which the Rust engine instantiates dynamically. Writing new `.rs` files for ordinary STEM topics is strictly prohibited.

## Authoring Flow
1. **Research/Source:** Analyze the educational domain.
2. **Content Extraction:** Isolate core concepts.
3. **Pattern/Archetype:** Define the procedural variants.
4. **Declarative Contract:** Write the constraints and templates via `tools/studylab_content_factory.py`.
5. **Validation:** Ensure generation domains can be satisfied.
6. **APKG Generation:** Compile via `generate_procedural_apkg.py`.
7. **Runtime:** Anki loads the deck; StudyLab runtime dynamically executes it during review.
