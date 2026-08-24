# System Architecture

The end-to-end architecture of StudyLab operates via a clean separation between static declarative content and the dynamic procedural runtime.

## Component Pipeline

**1. SOURCE MATERIAL**
- Purpose: The raw educational syllabus, exam requirements, or domain curriculum.

**2. STUDY-SOURCE-CORE / CONTENT FACTORY**
- Purpose: Authoring and defining parameter domains, constraints, archetypes, and solution graph templates using Python declarative authoring (`tools/studylab_content_factory.py`).

**3. APKG / CONTENT CONTRACT**
- Purpose: The static declarative blueprint, packaged into standard `.apkg` files (`generate_procedural_apkg.py`).
- Precedence: `inline_contract` (Tier 1, fully self-contained JSON) > `content_ref` (Tier 2, resolves against local database rows) > `proc_schema` (Tier 3 legacy string ID dispatch in Rust).
- Owns: Constraints, parameter domains (`IntegerRange`, `DiscreteChoice`, `CoprimePair`, `DerivedLinear`, etc.), hints, templates, answer derivations, and solution graph step nodes.

**4. PROCEDURAL CARD ANCHOR**
- Purpose: The standard Anki Note (notetype `"StudyLab Procedural Anchor"`) that acts as a scheduling anchor. It contains the `ProceduralPayload` JSON in a note field.
- Integration: When Anki schedules this note, `rslib/src/notetype/render.rs:123` intercepts the card rendering pipeline (`render_procedural_anchor`) and injects the interactive problem-solving workspace.

**5. CONTENT RESOLUTION**
- Purpose: Extract and validate the contract (`rslib/procedural/src/service/mod.rs:484-600`). In Tier 1, `inline_contract.validate()` registers the family contract in memory and resolves the difficulty and seed.

**6. PROBLEM FAMILY CONTRACT**
- Purpose: Defines the capabilities, difficulty levels, target latency models, and error categories for the problem family (`rslib/procedural/src/problems/contract.rs`).

**7. UNIVERSAL & SPECIALIZED CAPABILITIES**
- Purpose: The `DeclarativeProblemGenerator` (`rslib/procedural/src/problems/declarative.rs`) serves as the universal runtime mold.
- Classification (`ProblemFamilyCapability`): `Declarative`, `ConstraintSolver`, `SymbolicLogic`, `DomainPhysics`, `DomainChemistry`, `DomainGeometry`, `Specialized`.
- Precedence / Dispatch: `ProblemRegistry::generate` (`rslib/procedural/src/problems/registry.rs:135-180`) attempts declarative generation with validation first; if declarative generation is unregistered or validation fails, it falls through to specialized compiled domain generators (`rslib/procedural/src/problems/generators/`, `reasoning/generators/`, `physics/generators/`, `chemistry/generators/`).
- Authoring Rule: Ordinary new topics must use the declarative mold; writing new Rust code is reserved for genuinely novel execution engines.

**8. PROBLEM INSTANCE**
- Purpose: Deterministic, ephemeral instantiation of a problem (`ProblemInstance`) with concrete values satisfying constraints (using `seed_mode`: `Random`, `Fixed`, or `Daily`).

**9. LEARNING OBJECT**
- Purpose: The pedagogical interaction modality injected into the webview frontend (`ts/reviewer/procedural.ts`), such as `problem` (quick / stepwise), `mcq`, `concept_check`, `strategy_drill`, `worked_example`, `declarative_recall`, `prerequisite_review`, `transfer_retry`, or `representation_drill`.

**10. REVIEWER WORKSPACE**
- Purpose: The interactive frontend where the learner solves the problem, enters numeric/symbolic/step answers, requests hints, or classifies errors (`ts/reviewer/procedural.ts`, `ts/reviewer/components/`).

**11. ATTEMPT & EVALUATION**
- UI Evaluation: TS frontend evaluates inputs immediately based on `inline_contract` derivations for zero-latency feedback.
- Canonical Evaluation: Rust backend (`rslib/procedural/src/problems/steps/step_validator.rs`) provides the authoritative semantic verification.
- Telemetry: TS captures `PracticeAttempt` (latency, correctness, step trace, active mode) and dispatches via `bridgeCommand("procedural_attempt")` to Python/Qt (`qt/aqt/reviewer.py`).

**12. MASTERY EVIDENCE + DOMAIN EVIDENCE**
- Purpose: Single-attempt metrics (`final_correctness`, `independence`, diagnostic error taxonomy). Domain Evidence (`rslib/procedural/src/skills/domain_evidence.rs`) captures structured dimensions across Math, Reasoning, Physics, and Chemistry.

**13. SKILL STATE PERSISTENCE**
- Telemetry Pipeline: TS calls `globalThis.anki.mutateNextCardStates` to inject telemetry into custom data, passed through Qt bridge to the Rust scheduler (`rslib/src/scheduler/answering/mod.rs:350-435`), which records `PracticeAttempt`, `ErrorEvent`, and updates `SkillState` in `procedural.db` (`rslib/procedural/src/storage/store.rs`) while keeping `collection.anki2` completely clean.

**14. ADAPTIVE DIFFICULTY & PROGRESSION**
- Purpose: Composite mastery progression gates evaluated by `ProgressionPolicy::evaluate` (`rslib/procedural/src/skills/progression.rs:13-147`): `New -> Learning -> Fluent -> Variation -> Transfer -> Mastered`.

**15. REMEDIATION POLICY**
- Purpose: `RemediationPolicy::evaluate` (`rslib/procedural/src/remediation/policy.rs:33-400`) maps diagnosed failures to interventions.
- Mechanics: Execution errors yield `ProceduralVariant` (simpler numbers). Concept errors heavily demote state and trigger `ConceptCheck` / `StrategyDrill`. Escalation triggers `WorkedExample` (recurrence == 3), `PrerequisiteReview` (recurrence == 4), or `CircuitBreaker` (recurrence >= 5).

**16. REMEDIATION QUEUE**
- Purpose: Sequences JIT interventions in `procedural.db` (`remediation_queue_items`).

**17. TARGETED NEXT INTERVENTION**
- Purpose: The subsequent learning object presented to the learner before or alongside standard FSRS reviews.
