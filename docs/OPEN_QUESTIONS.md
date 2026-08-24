# Open Questions

This register tracks unresolved architectural questions that could not be fully answered through source inspection of the current procedural components. 

## 1. Internal Execution of Complex Capabilities (e.g., DomainPhysics)

- **QUESTION:** How do more complex capability layers like `DomainPhysics` hook into the runtime generators underneath the `DeclarativeArchetypeGenerator`?
- **WHY IT MATTERS:** While the contract surfaces (`DeclarativeFamilyContract`) and diagnostic policies (`RemediationPolicy`) are well understood, the specific internal rust execution mapping for advanced physics models is opaque. If a new physics archetype requires a constraint not expressible in standard `DeclarativeArchetype`, it's unclear if `DomainPhysics` intercepts the generation or just adds a semantic evaluation layer.
- **EVIDENCE ALREADY FOUND:** `ProblemFamilyCapability` categorizes `DomainPhysics` as a Level 2 generator in `rslib/procedural/src/problems/contract.rs`. The `DeclarativeArchetypeGenerator` handles standard instantiation.
- **WHAT IS STILL MISSING:** The exact handoff mechanism where `DeclarativeArchetypeGenerator` yields to `DomainPhysics` logic during `seed_mode` resolution.
- **WHAT WOULD RESOLVE IT:** Deep source tracing of `rslib/procedural/src/problems/physics/` (if it exists) to observe how it implements the `DeclarativeFamilyContract` traits.
