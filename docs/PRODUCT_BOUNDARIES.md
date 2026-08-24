# Product Boundaries

## Responsibility Table

| Component | Owner | Responsibility |
| :--- | :--- | :--- |
| **Flashcards** | Anki | Standard declarative knowledge and memorization. |
| **Basic/Cloze** | Anki | Traditional Anki note types. |
| **Scheduling** | Anki (FSRS) | Determining exactly *when* a card or procedural anchor is due. |
| **FSRS** | Anki | The spaced repetition algorithm and optimization. |
| **Problem Generation** | StudyLab (Rust core) | Dynamic instantiation of constraints and parameters from declarative templates. |
| **Problem Evaluation** | StudyLab (TS/Rust) | Parsing and validating mathematical, logical, or textual answers. |
| **Step Validation** | StudyLab (Rust core) | Validating intermediate reasoning steps (stepwise mode). |
| **Domain Evidence** | StudyLab (Rust core) | Producing taxonomy-specific diagnostics (e.g., execution error vs. conceptual error). |
| **Mastery** | StudyLab (Rust core) | `MasteryEvidence` based on independence, speed, and accuracy gates. |
| **Remediation** | StudyLab (Rust core) | Emitting JIT interventions (Concept Checks, Strategy Drills, Prerequisite Reviews). |
| **Diagnostics** | StudyLab (Rust core) | Sweeping across topics to detect weaknesses vs standard adaptive practice. |
| **Content Authoring** | APKG Tooling (Python) | Packaging constraints, archetypes, and parameters into `.apkg` files. |
| **Provenance** | APKG | Source metadata and topic tagging within the declarative contract. |
| **Learner State** | StudyLab (`procedural.db`) | Storing `SkillState`, attempts, and evidence without polluting `collection.anki2`. |

## Core Integration Principle
**Anki is the host and integration environment, not StudyLab's product identity.**

StudyLab integrates into Anki via a "Trojan-horse" architecture. Anki schedules a generic note type (`StudyLab Procedural Anchor`). When this note is due, StudyLab intercepts the render pipeline and injects its own TS/Vite problem-solving webview. The internal state (`procedural.db`) remains completely decoupled from Anki's database (`collection.anki2`) to ensure zero schema contamination.
