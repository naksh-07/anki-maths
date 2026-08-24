# StudyLab Content Model

The StudyLab content ecosystem fundamentally decouples problem generation from Rust source code to guarantee zero-code scalability.

## 1. Declarative Content Contracts
Content is packaged declaratively. An `.apkg` deck contains `ProceduralCardAnchor` metadata, which holds an `inline_contract` (`DeclarativeFamilyContract`).
- **No Rust Generators:** Creating a new ordinary topic **MUST NOT** require writing a new Rust generator.
- **Tooling:** Content is authored using the `study-source-core` authoring skill via Python scripts (`tools/studylab_content_factory.py`), which compile topics into `.apkg` contracts.

## 2. Universal Procedural Runtime
The Rust backend operates as a generic interpretation engine (`DeclarativeArchetypeGenerator`). 
It ingests declarative contracts, evaluates domain bounds (e.g., `IntegerRange`, `DerivedLinear`), applies parameter constraints (`NotEqual`, `NonZero`), and dynamically substitutes variables into a `template_ref` to create ephemeral `ProblemInstance`s.

## 3. Definitions
- **Universal Runtime Mold**: The Rust engine capable of running *any* declarative contract efficiently (proven at 0.289 ms / topic).
- **Problem Instance**: (`ProblemInstance`) A specific, ephemeral problem deterministically generated from a `seed` and a contract. It contains the `rendered_prompt`, parameter maps, and the `correct_answer` (including solution graphs).
- **Attempt**: (`PracticeAttempt`) The learner's interaction event, capturing user answer, latency, and correctness.

## 4. Explicit Prohibitions
- **PROHIBITED:** Writing a new `.rs` file in `rslib/procedural/src/problems/` to support a standard Math or Physics topic. All ordinary content expansion must remain strictly declarative.

---
### Traceability & Code Evidence
- **Declarative Contracts:** `rslib/procedural/src/anchor/mod.rs` (`ProceduralCardAnchor`), `rslib/procedural/src/problems/contract.rs`.
- **Runtime Interpretation:** `rslib/procedural/src/problems/declarative.rs`.
- **Scalability Proof:** Documented 175 topics scaled without backend changes (`.agents/specialist4_content_contract/handoff.md`).
