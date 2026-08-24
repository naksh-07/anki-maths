# Documentation Map

This map directs future AI agents and developers to the canonical documentation based on what they need to understand about StudyLab.

| Topic / Subsystem | Read this document |
| :--- | :--- |
| **Product Purpose, Target Learner & Non-Goals** | [PRODUCT_VISION.md](PRODUCT_VISION.md) |
| **System Boundaries & Host Responsibilities (Anki vs StudyLab)** | [PRODUCT_BOUNDARIES.md](PRODUCT_BOUNDARIES.md) |
| **End-to-End Pipeline & System Architecture** | [SYSTEM_ARCHITECTURE.md](SYSTEM_ARCHITECTURE.md) |
| **Learner Cognitive Model, Domain Evidence & Mastery Gates** | [LEARNING_MODEL.md](LEARNING_MODEL.md) |
| **Content Packaging, Precedence Tiers & Authoring Factory** | [CONTENT_AND_AUTHORING.md](CONTENT_AND_AUTHORING.md) |
| **Learning Objects, Modalities & Pedagogical Interventions** | [LEARNING_OBJECTS.md](LEARNING_OBJECTS.md) |
| **Reviewer UI State Machine, Actions & Keyboard Shortcuts** | [REVIEWER_STATE_MACHINE.md](REVIEWER_STATE_MACHINE.md) |
| **Frontend/Backend IPC Bridge, Telemetry & Persistence Contract** | [FRONTEND_BACKEND_CONTRACT.md](FRONTEND_BACKEND_CONTRACT.md) |
| **Diagnostic Closed Loop, Escalation & Circuit Breakers** | [DIAGNOSTIC_AND_REMEDIATION.md](DIAGNOSTIC_AND_REMEDIATION.md) |
| **Non-Negotiable Architecture Invariants** | [ARCHITECTURE_INVARIANTS.md](ARCHITECTURE_INVARIANTS.md) |
| **Unresolved Architectural Questions Register** | [OPEN_QUESTIONS.md](OPEN_QUESTIONS.md) |

---

## Canonical Source Code Traceability

When verifying or modifying the codebase, refer directly to these authoritative source paths:

- **Domain Evidence Structs & Enums:** `rslib/procedural/src/skills/domain_evidence.rs`
- **SkillState & Progression Gates:** `rslib/procedural/src/skills/mod.rs` & `progression.rs`
- **Remediation Policies & Escalation:** `rslib/procedural/src/remediation/policy.rs` & `objects.rs`
- **Problem Family Contract & Parameter Domains:** `rslib/procedural/src/problems/contract.rs`
- **Universal Declarative Generator:** `rslib/procedural/src/problems/declarative.rs`
- **Problem Registry & Dynamic Dispatch:** `rslib/procedural/src/problems/registry.rs`
- **Step Semantic Validator:** `rslib/procedural/src/problems/steps/step_validator.rs`
- **Storage, Schema & Migrations 1–5:** `rslib/procedural/src/storage/schema.rs` & `store.rs`
- **Content Resolution Precedence (Tiers 1–3):** `rslib/procedural/src/service/mod.rs:484-600`
- **Anki Card Interception Hook:** `rslib/src/notetype/render.rs:123`
- **Scheduler Custom Data Extraction Hook:** `rslib/src/scheduler/answering/mod.rs:350-435`
- **Webview UI State Machine & Components:** `ts/reviewer/procedural.ts` & `ts/reviewer/components/`
- **Python / Qt Reviewer Bridge Router:** `qt/aqt/reviewer.py:724-774`
- **Python Declarative Content Factory:** `tools/studylab_content_factory.py`
- **APKG Deck Generation Tool:** `generate_procedural_apkg.py`

