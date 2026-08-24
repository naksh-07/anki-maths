# Documentation Map

This map directs you to the canonical documentation based on what you need to understand about StudyLab.

| I want to understand... | Read this document |
| :--- | :--- |
| **Product Purpose & Goals** | [PRODUCT_VISION.md](PRODUCT_VISION.md) |
| **System Boundaries & Anki Integration** | [PRODUCT_BOUNDARIES.md](PRODUCT_BOUNDARIES.md) |
| **End-to-End System Architecture** | [SYSTEM_ARCHITECTURE.md](SYSTEM_ARCHITECTURE.md) |
| **Learner Cognitive Model & Evidence** | [LEARNING_MODEL.md](LEARNING_MODEL.md) |
| **Content Packaging & APKG Contracts** | [CONTENT_AND_AUTHORING.md](CONTENT_AND_AUTHORING.md) |
| **Types of Problems / Interventions** | [LEARNING_OBJECTS.md](LEARNING_OBJECTS.md) |
| **Frontend UI States & Flow** | [REVIEWER_STATE_MACHINE.md](REVIEWER_STATE_MACHINE.md) |
| **TS / Rust / Qt Communication Bridge** | [FRONTEND_BACKEND_CONTRACT.md](FRONTEND_BACKEND_CONTRACT.md) |
| **Diagnosis, Escalation, & Circuit Breakers** | [DIAGNOSTIC_AND_REMEDIATION.md](DIAGNOSTIC_AND_REMEDIATION.md) |
| **Non-Negotiable Technical Rules** | [ARCHITECTURE_INVARIANTS.md](ARCHITECTURE_INVARIANTS.md) |
| **Unresolved Architectural Questions** | [OPEN_QUESTIONS.md](OPEN_QUESTIONS.md) |

## Key Source Traceability

When investigating the code, refer to these primary entry points:

- **Domain Evidence:** `rslib/procedural/src/skills/domain_evidence.rs`
- **Remediation:** `rslib/procedural/src/remediation/policy.rs`
- **Problem Family Contract:** `rslib/procedural/src/problems/contract.rs`
- **Frontend State:** `ts/reviewer/procedural.ts`
- **Bridge Router:** `qt/aqt/reviewer.py`
- **Reviewer Interception:** `rslib/src/notetype/render.rs`
- **Content Resolution:** `rslib/procedural/src/service/mod.rs`
- **APKG Packaging:** `generate_procedural_apkg.py`
