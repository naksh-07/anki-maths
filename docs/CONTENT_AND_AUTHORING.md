# Content and Authoring

The boundary between static content definitions and dynamic runtime execution is strictly maintained through the APKG content contract.

## Responsibility Separation

### Content Package (APKG) Owns:
- Problem family definitions and IDs
- Archetypes and parameter specifications
- Parameter domains (`IntegerRange`, `FloatRange`, `DiscreteChoice`, `DerivedLinear`, `DerivedProduct`, `DerivedSum`, `DerivedDifference`, `DerivedQuotient`, `DerivedSignedString`, `DerivedPower`, `DerivedPercentage`, `DerivedHypotenuse`, `DerivedPythagoreanLeg`, `PermutationChoice`, `PrimeFactorGrid`, `CoprimePair`)
- Constraints (`NotEqual`, `NonZero`, `Divisible`, `GreaterThan`, `LessThan`, `SumEquals`, `Predicate`)
- Answer derivations (22+ algorithms covering linear arithmetic, Pythagoras, kinematics, stoichiometry, equilibrium $K_c$, ideal gas law, and symbolic logic truth evaluation)
- Solution graphs and step nodes (`StepNodeSpec`, hints, explanations)
- Hints and structural variants (`Authentic`, `Variation`, `Transfer`, `PyqVariant`)
- Provenance and topic/chapter metadata

### Runtime (Rust backend + DB) Owns:
- Ephemeral generation of concrete variables fulfilling constraints (via `seed_mode`)
- Learner attempts (`PracticeAttempt`)
- `SkillState` and `MasteryEvidence` history
- DomainEvidence tracking (`MathEvidence`, `ReasoningEvidence`, `PhysicsEvidence`, `ChemistryEvidence`)
- Remediation and targeted next practice
- Recurrence and queue state
- Integration with Anki's scheduling (FSRS)

## Content Resolution Precedence

When a `StudyLab Procedural Anchor` note is due, the backend resolves the payload via `rslib/procedural/src/service/mod.rs:484-600` in this strict precedence:

1. **`inline_contract`** (Tier 1)
   - The complete `DeclarativeFamilyContract` is fully bundled as JSON in the anchor note field. Evaluated first. Enables 100% self-contained deck portability with zero prior setup. **This is the preferred canonical path.**
2. **`content_ref`** (Tier 2)
   - Uses a string ID to resolve against pre-ingested `PracticeItem` rows in the local `procedural.db`. Keeps `.apkg` file size small but requires a sync/import step first.
3. **`legacy proc_schema`** (Tier 3)
   - Uses hardcoded string IDs dispatched to legacy unified practice engine components manually written in Rust. Must not be used for new content.

## Three-Tier Content & Capability Architecture

To prevent architectural confusion, StudyLab explicitly distinguishes three levels of content and execution capability:

1. **Ordinary New Content (Declarative):**
   - Implemented in Python (`tools/studylab_content_factory.py` and `generate_procedural_apkg.py`).
   - Requires **zero Rust code modifications**. Covers all standard arithmetic, algebraic, geometric, kinematics, stoichiometry, and logic problem families.
2. **Universal Procedural Runtime Capability:**
   - Implemented in Rust as `DeclarativeProblemGenerator` (`rslib/procedural/src/problems/declarative.rs`).
   - Acts as the universal procedural engine that instantiates any valid `DeclarativeFamilyContract`, resolves parameters, enforces constraints, and computes derived answers.
3. **Specialized Domain Generators (Compiled Rust):**
   - Implemented in `rslib/procedural/src/problems/generators/`, `reasoning/generators/`, `physics/generators/`, and `chemistry/generators/`.
   - Used for complex algorithmic derivations (e.g. non-linear physics simulations, complex organic reaction pathways, or circular seating arrangements with multi-relational graph constraints) where dynamic declarative constraints require specialized solver algorithms.
   - `ProblemRegistry::generate` (`rslib/procedural/src/problems/registry.rs:135-180`) attempts declarative generation first and automatically falls through to specialized generators if declarative generation is unregistered or validation fails.

## Authoring Flow
1. **Research/Source:** Analyze syllabus requirements or past-year question patterns.
2. **Concept Extraction:** Isolate parameters, invariants, and failure modes.
3. **Pattern/Archetype Definition:** Define parameter domains and constraints.
4. **Declarative Contract:** Write the contract via `tools/studylab_content_factory.py`.
5. **Validation:** Ensure parameter ranges and constraint satisfaction pass `DeclarativeFamilyContract::validate()`.
6. **APKG Compilation:** Build deck via `generate_procedural_apkg.py`.
7. **Runtime Execution:** Anki loads the deck; StudyLab runtime dynamically executes it during review sessions.
