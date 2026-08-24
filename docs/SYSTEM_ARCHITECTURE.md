# System Architecture

The end-to-end architecture of StudyLab operates via a clean separation between static declarative content and the dynamic procedural runtime.

## Component Pipeline

**1. SOURCE MATERIAL**
- Purpose: The raw educational material or research.

**2. STUDY-SOURCE-CORE**
- Purpose: Authoring and defining constraints, archetypes, and templates.

**3. APKG / CONTENT CONTRACT**
- Purpose: The static declarative blueprint. Compiled by Python tooling (`generate_procedural_apkg.py`).
- Precedence: `inline_contract` (Tier 1) > `content_ref` (Tier 2) > `proc_schema` (Tier 3 legacy).
- Owns: Constraints, parameter domains (`IntegerRange`, `CoprimePair`), hints, templates.

**4. PROCEDURAL CARD ANCHOR**
- Purpose: The Anki Note that acts as a scheduling anchor. It contains the `ProceduralPayload` in a field.
- Integration: When Anki schedules this, `rslib/src/notetype/render.rs` intercepts the render pipeline.

**5. CONTENT RESOLUTION**
- Purpose: Extract the `DeclarativeFamilyContract` from the anchor.

**6. PROBLEM FAMILY CONTRACT**
- Purpose: Defines the capabilities and archetypes available for instantiation.

**7. UNIVERSAL / DOMAIN CAPABILITY**
- Purpose: The `DeclarativeArchetypeGenerator` acts as the universal procedural mold.
- Classification: `Declarative`, `ConstraintSolver`, `SymbolicLogic`, `DomainPhysics`, `DomainChemistry`, `DomainGeometry`, `Specialized`.
- Note: New ordinary topics *should not require new Rust generators*. They must use the declarative mold.

**8. PROBLEM INSTANCE**
- Purpose: Ephemeral instantiation of the problem with specific variables satisfying constraints (via `seed_mode`).

**9. LEARNING OBJECT**
- Purpose: The specific interaction type injected into the TS/Vite frontend (e.g., `problem`, `mcq`, `stepwise`).

**10. REVIEWER**
- Purpose: The interactive frontend where the learner solves the problem.

**11. ATTEMPT**
- Purpose: The `PracticeAttempt` capturing answer, latency, and interaction telemetry. Evaluated locally in TS, then shuttled via Python/Qt bridge to Rust.

**12. MASTERY EVIDENCE + DOMAIN EVIDENCE**
- Purpose: Single-attempt metrics (`final_correctness`, `independence`, diagnostic errors). Domain Evidence maps specific failures (e.g., `is_execution_error()`).

**13. SKILL STATE**
- Purpose: Long-term history stored in `procedural.db` (`consecutive_successes`, `historical_independent_count`).

**14. ADAPTIVE DIFFICULTY**
- Purpose: Progressing learner states (`New -> Learning -> Fluent -> Variation -> Transfer -> Mastered`).

**15. REMEDIATION POLICY**
- Purpose: `RemediationPolicy::evaluate(ctx)` determines interventions.
- Mechanics: e.g., Execution errors yield `ProceduralVariant` (simpler numbers). Concept errors demote state and trigger `ConceptCheck`.

**16. REMEDIATION QUEUE**
- Purpose: Managing the sequence of JIT interventions.

**17. TARGETED NEXT PRACTICE**
- Purpose: The subsequent learning object presented to the learner.
