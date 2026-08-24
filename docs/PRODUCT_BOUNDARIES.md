# StudyLab Product Boundaries

StudyLab functions as a "two-system learning workstation". It must never duplicate or replace the core functionalities of its host environment (Anki).

## 1. Responsibility Matrix

| Component | Owner | Role & Responsibility |
| :--- | :--- | :--- |
| **Spaced Repetition (Scheduling)** | **Anki / FSRS** | Determines *when* a memory anchor is due. StudyLab relies entirely on Anki's scheduling queues. |
| **Card Data / Deck Organization** | **Anki** | The `collection.anki2` database holds the standard decks and lightweight "Memory Anchors". |
| **Problem Solving Intelligence** | **StudyLab** | Generates the problem, validates steps, and diagnoses errors via the isolated Rust engine. |
| **Telemetry & Learner State** | **StudyLab** | `SkillState`, `PracticeAttempt`, and `DomainEvidence` are stored safely in an isolated `procedural.db` to protect Anki's schema. |
| **Problem Content Definition** | **APKG** | Content is declarative. `.apkg` files package the `inline_contract` metadata, not hardcoded topic logic. |
| **Content Execution** | **Universal Runtime** | StudyLab's Rust core acts as a generic interpreter that dynamically evaluates the APKG declarative contracts. |

## 2. The Procedural Anchor Model
StudyLab operates using a Trojan-horse architecture. Cards in Anki are created with a specific note type: `StudyLab Procedural Anchor`. 
When Anki attempts to render these cards, StudyLab intercepts the rendering pipeline (`rslib/src/notetype/render.rs:123`), bypasses the standard HTML/CSS front/back output, and injects the StudyLab TS/Vite webview UI.

## 3. Data Flow & Telemetry Sync
StudyLab's telemetry is synced back to the Rust backend via a Python bridge (`qt/aqt/reviewer.py`). To comply with Anki's database limitations, telemetry is temporarily stuffed into Anki's `custom_data` object, extracted by the StudyLab scheduler (`rslib/src/scheduler/answering/mod.rs:350-510`), persisted to `procedural.db`, and then stripped from Anki's DB.

---
### Traceability & Code Evidence
- **Rendering Interception:** `rslib/src/notetype/render.rs:123` (`render_procedural_anchor(...)`).
- **Telemetry Extraction:** `rslib/src/scheduler/answering/mod.rs:350-510`.
- **Database Isolation:** `procedural.db` is strictly maintained separate from `collection.anki2` to prevent schema pollution.
