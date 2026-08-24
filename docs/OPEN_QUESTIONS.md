# Open Questions & Architectural Decisions Register

This register documents resolved architectural questions alongside genuinely open product and design decisions that require human stakeholder resolution.

---

## Resolved Questions (Verified by Code)

### 1. Dynamic Capability Dispatch & Specialized Domain Generators
- **Resolution:** Verified in `ProblemRegistry::generate` (`rslib/procedural/src/problems/registry.rs:135-180`).
- **Mechanism:** When generating a problem instance, `ProblemRegistry` first checks `get_declarative_generator(family_id)`. If present, it executes the universal `DeclarativeProblemGenerator` and validates the result with `get_validator(family_id)`. If declarative generation is unregistered or validation fails, it automatically falls through to the specialized compiled generator in `self.generators_by_family` (e.g. `rslib/procedural/src/physics/generators/`, `reasoning/generators/`, `chemistry/generators/`).

---

## Genuinely Open Product Decisions

### 1. Automated Ease 2 ("Hard") Rating Heuristic for FSRS
- **QUESTION:** Should high-friction correct attempts (e.g., correct after multiple hints or step retries) programmatically trigger FSRS Ease `2` ("Hard")?
- **WHY IT MATTERS:** FSRS optimizes interval stability based on rating fidelity. Currently, `ts/reviewer/procedural.ts:1224-1228` only programmatically emits Ease `1` (Incorrect), `3` (Slow Correct), and `4` (Fast Correct). Native key `2` is accepted by `qt/aqt/reviewer.py:706` if clicked manually, but automated completion never emits Ease `2`.
- **SOURCE EVIDENCE:** `ts/reviewer/procedural.ts:1224-1228` vs `qt/aqt/reviewer.py:706-738`.
- **WHAT IS UNKNOWN:** Whether cognitive friction (e.g. $\text{hint\_count} > 0$ with final correct answer) should map to Ease `2` or whether binary Pass/Fail with latency is preferable for procedural FSRS scheduling.
- **WHO MUST DECIDE:** Pedagogical & Learning Science Lead.
- **PROPOSED NEXT EVIDENCE:** Longitudinal retention comparison between binary ease mapping and 4-tier ease mapping on procedural problem families.

### 2. Multi-Device Synchronization Policy for `procedural.db`
- **QUESTION:** What is the long-term synchronization architecture for `procedural.db` across desktop, mobile (AnkiMobile/AnkiDroid), and web?
- **WHY IT MATTERS:** Standard AnkiWeb syncs `collection.anki2` and media, but does not natively sync auxiliary SQLite databases like `procedural.db`. Tier 1 `inline_contract` decks are 100% self-contained and run on any client, but longitudinal `SkillState` remains local to each device unless synchronized.
- **SOURCE EVIDENCE:** `rslib/src/collection/mod.rs:175` (`col_path.with_extension("procedural")`).
- **WHAT IS UNKNOWN:** Whether StudyLab will implement a custom syncserver extension (`syncserver/`), embed compressed skill state deltas in Anki card custom data, or maintain local-first independent skill histories per device.
- **WHO MUST DECIDE:** Core Architecture & Platform Lead.
- **PROPOSED NEXT EVIDENCE:** Benchmarks of syncing compressed skill state snapshots inside `mutateNextCardStates` custom data vs dedicated SQLite sync protocols.
