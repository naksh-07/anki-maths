# StudyLab Open Questions & Architectural Decisions Register

**Document Version:** 1.0.0 (Canonical)  
**Status:** AUTHORITATIVE CANONICAL REGISTER  
**Integrity Mode:** 100% Grounded in Executable Source Code & Test Evidence  

This register formally documents architectural questions that have been resolved and verified by the codebase, alongside genuinely open product choices and technical explorations requiring future human stakeholder decisions.

---

## 1. Resolved Questions (Verified by Executable Code & Tests)

The following architectural questions were historically raised during Phase 01–03 analysis and are now fully resolved and verified:

### 1.1 Dynamic Capability Dispatch & Generator Fallback
- **Resolution:** Verified in `ProblemRegistry::generate` (`rslib/procedural/src/problems/registry.rs:135-180`).
- **Mechanism:** When generating a problem instance, `ProblemRegistry` first checks `get_declarative_generator(family_id)`. If present, it executes the universal `DeclarativeProblemGenerator` and validates the result with `get_validator(family_id)`. If declarative generation is unregistered or validation fails, it automatically falls through to the specialized compiled generator in `self.generators_by_family` (e.g. `physics/generators/`, `reasoning/generators/`, `chemistry/generators/`).

### 1.2 Stepwise Semantic Equivalence & Downstream Consistency
- **Resolution:** Verified in `StepValidator` and `MathSemanticComparator` (`rslib/procedural/src/problems/steps/step_validator.rs:1-350`).
- **Mechanism:** The engine normalizes algebraic strings, solves linear equations for matching roots (e.g. $2x + 6 = 16 \iff 2x = 10 \iff x = 5$), verifies commutative addition, localizes the first error step, and tags subsequent consistent steps as `PartiallyValid` with `is_downstream_consistent = true`.

### 1.3 Anki Collection Database Separation & Schema Integrity
- **Resolution:** Verified in `ProceduralStore` (`rslib/procedural/src/storage/store.rs:28-53`) and `Collection` initialization (`rslib/src/collection/mod.rs:173-183`).
- **Mechanism:** The procedural learning store is housed in a dedicated SQLite database (`<collection_name>.procedural`), maintaining zero schema alterations or table injections inside Anki's upstream `collection.anki2`.

### 1.4 Ephemeral Telemetry Ingestion & Custom Data Stripping
- **Resolution:** Verified in `rslib/src/scheduler/answering/mod.rs:353-505`.
- **Mechanism:** When answering a card with `custom_data["studylab"]`, the Rust answering pipeline ingests the telemetry payload, commits it atomically to `procedural.db`, and strips the `studylab` key from `card.custom_data` prior to SQLite commit, strictly respecting Anki's 100-byte column limit.

### 1.5 Anti-Bypass Metacognitive Reflection Trapping
- **Resolution:** Verified in `ts/reviewer/procedural.ts:310-360` and `ts/reviewer/components/mistake_footer.ts`.
- **Mechanism:** During the `mistake_classification` state, Space and Enter key events are captured and stopped (`e.preventDefault()`, `e.stopPropagation()`), preventing students from skipping mistake attribution until a category (`1`–`4`) is explicitly selected.

### 1.6 MCQ Modality Mock Exam Mode
- **Resolution:** Verified in `ts/reviewer/components/mcq_container.ts:1-250`.
- **Mechanism:** When initialized with `mode: "mock"`, selecting an option highlights it with `.selected` but strictly suppresses `.correct`, `.incorrect`, and spoiler styling until full diagnostic test submission.

---

## 2. Genuinely Open Product Decisions & Technical Explorations

The following 5 questions represent genuine, unresolved architectural choices and future explorations that require stakeholder evaluation:

---

### 1. Automated Ease 2 ("Hard") Rating Heuristic for FSRS
- **QUESTION:** Should high-friction correct attempts (e.g., correct after multiple hints or step retries) programmatically trigger FSRS Ease `2` ("Hard")?
- **WHY IT MATTERS:** FSRS optimizes interval stability based on rating fidelity. Currently, `ts/reviewer/procedural.ts:1224-1228` only programmatically emits Ease `1` (Again / Incorrect), `3` (Good / Slow Correct), and `4` (Easy / Fast Correct). Native key `2` is accepted by `qt/aqt/reviewer.py:706` if clicked manually on the bottom ease bar, but automated completion never emits Ease `2`.
- **SOURCE EVIDENCE:** `ts/reviewer/procedural.ts:1224-1228` vs `qt/aqt/reviewer.py:706-738`.
- **WHAT IS UNKNOWN:** Whether cognitive friction (e.g. $\text{hint\_count} > 0$ with final correct answer) should map to Ease `2` or whether binary Pass/Fail with latency is preferable for procedural FSRS scheduling.
- **WHO MUST DECIDE:** Pedagogical & Learning Science Lead.
- **PROPOSED NEXT EVIDENCE:** Longitudinal retention comparison between binary ease mapping and 4-tier ease mapping on procedural problem families.

---

### 2. Multi-Device Synchronization Policy for `procedural.db`
- **QUESTION:** What is the long-term synchronization architecture for `procedural.db` across desktop, mobile (AnkiMobile / AnkiDroid), and web?
- **WHY IT MATTERS:** Standard AnkiWeb syncs `collection.anki2` and media files, but does not natively sync auxiliary SQLite databases like `procedural.db`. Tier 1 `inline_contract` decks are 100% self-contained and run on any client, but longitudinal `SkillState` remains local to each device unless synchronized.
- **SOURCE EVIDENCE:** `rslib/src/collection/mod.rs:175` (`col_path.with_extension("procedural")`).
- **WHAT IS UNKNOWN:** Whether StudyLab will implement a custom syncserver extension (`syncserver/`), embed compressed skill state deltas in Anki card custom data, or maintain local-first independent skill histories per device.
- **WHO MUST DECIDE:** Core Architecture & Platform Lead.
- **PROPOSED NEXT EVIDENCE:** Benchmarks of syncing compressed skill state snapshots inside `mutateNextCardStates` custom data vs dedicated SQLite sync protocols.

---

### 3. Client-Side WebAssembly (Wasm) Engine Evaluation for Mobile Clients
- **QUESTION:** Should `rslib/procedural` be compiled into a lightweight WebAssembly (`.wasm`) bundle embedded in exported `.apkg` packages for complete offline AST validation on mobile clients?
- **WHY IT MATTERS:** Desktop Anki runs the full compiled Rust binary via PyO3/C-ABI, while mobile clients execute Tier 1 declarative contracts via JavaScript in `ts/reviewer/procedural.ts`. Compiling `StepValidator` to Wasm would guarantee 100% parity for multi-step algebraic derivation on mobile webviews without requiring native mobile Rust bindings.
- **SOURCE EVIDENCE:** `rslib/procedural/Cargo.toml` and `ts/reviewer/components/stepwise_container.ts`.
- **WHAT IS UNKNOWN:** Size footprint of a standalone `procedural_wasm.wasm` build and performance across low-end mobile devices.
- **WHO MUST DECIDE:** Mobile Platform & Tooling Lead.
- **PROPOSED NEXT EVIDENCE:** Prototype `wasm-pack` compilation of `rslib/procedural` and benchmark against AnkiDroid WebView.

---

### 4. Real-Time Free-Form Handwritten Equation OCR & Canvas Input Integration
- **QUESTION:** Should StudyLab provide a tablet/stylus handwriting canvas for quantitative step derivation with on-device stroke-to-LaTeX recognition?
- **WHY IT MATTERS:** For advanced mathematics and physics derivations, keyboard formula entry introduces extraneous cognitive load compared to natural pen-and-paper scratchwork.
- **SOURCE EVIDENCE:** `ts/reviewer/components/stepwise_container.ts` and `ts/reviewer/components/numerical_container.ts`.
- **WHAT IS UNKNOWN:** Feasibility of bundling lightweight on-device handwriting recognition (e.g. WebAssembly MathOCR) without external cloud API dependencies.
- **WHO MUST DECIDE:** UX & Product Lead.
- **PROPOSED NEXT EVIDENCE:** Technical spike testing open-source canvas stroke-to-LaTeX recognition libraries within QtWebEngine.

---

### 5. Partial-Credit Multi-Step Mastery Credit-Assignment Policy
- **QUESTION:** When a student makes an early step error but derives downstream steps consistently (`isDownstreamConsistent == true`), how much fractional credit ($0.0 < \text{score} < 1.0$) should be allocated towards EMA mastery updates?
- **WHY IT MATTERS:** Currently, `StepValidator` flags downstream consistent steps as `PartiallyValid` and assigns a composite score, but `SkillState::record_attempt` treats binary correctness as the primary EMA anchor ($\text{Outcome} \in \{1.0, 0.0\}$).
- **SOURCE EVIDENCE:** `rslib/procedural/src/problems/steps/step_validator.rs:250-290` vs `rslib/procedural/src/skills/mod.rs:80-115`.
- **WHAT IS UNKNOWN:** Whether fractional outcome weighting ($0.5$ for downstream consistent attempts) accelerates or destabilizes mastery estimation compared to strict binary thresholds.
- **WHO MUST DECIDE:** Cognitive Science & Psychometrics Lead.
- **PROPOSED NEXT EVIDENCE:** Simulated psychometric recovery analysis comparing binary vs continuous credit assignment on multi-step item banks.
