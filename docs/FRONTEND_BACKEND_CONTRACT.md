# Frontend / Backend Bridge Contract & IPC Protocol Specification

**Document Version:** 1.0.0 (Canonical)  
**Target Subsystem:** TypeScript Reviewer (`ts/reviewer/`), Python/Qt Reviewer (`qt/aqt/reviewer.py`), and Rust Procedural Engine (`rslib/procedural/`, `rslib/`)  
**Status:** AUTHORITATIVE CANONICAL SPECIFICATION  
**Integrity Mode:** 100% Grounded in Executable Source Code & Test Evidence  

---

## 1. Architectural Overview & Tripartite Contract

StudyLab operates a synchronized tripartite architecture spanning the webview frontend, Python desktop host, and Rust native storage/scheduling engine:

```text
┌─────────────────────────────────────────────────────────────────────────┐
│                          WEBVIEW SURFACE (TypeScript)                   │
│   - ProceduralReviewer State Machine (ts/reviewer/procedural.ts)        │
│   - Interactive Modality Containers (mcq, numerical, stepwise)          │
│   - Client-side AST Normalization & Tolerance Checks                    │
│   - Telemetry Packaging via globalThis.anki.mutateNextCardStates        │
└────────────────────────────────────┬────────────────────────────────────┘
                                     │
                                     │ bridgeCommand("<command>")
                                     ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                        PYTHON / QT DESKTOP HOST (PyQt6)                 │
│   - Reviewer Link Handler (qt/aqt/reviewer.py:_linkHandler)             │
│   - Procedural Command Router (_handle_procedural_command)              │
│   - Webview Lifecycle & Teardown Hooks (destroyActive)                  │
│   - FSRS Answer Rescheduling Trigger (_answerCard)                      │
└────────────────────────────────────┬────────────────────────────────────┘
                                     │
                                     │ PyO3 / C-ABI Native FFI
                                     ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                       RUST PROCEDURAL CORE (rslib/procedural)           │
│   - Card Interception Hook (rslib/src/notetype/render.rs)               │
│   - Ephemeral Telemetry Ingestion (rslib/src/scheduler/answering/)      │
│   - StepValidator & MathSemanticComparator (rslib/procedural/steps)     │
│   - Atomic SQLite Persistence in procedural.db (ProceduralStore)        │
│   - 10-Tier Unified Scheduler & RemediationQueue Engine                 │
└─────────────────────────────────────────────────────────────────────────┘
```

### Tripartite Responsibilities:
1. **Frontend (TypeScript):** Captures high-frequency micro-interactions (keystrokes, timing, progressive hints, mistake reflections), renders MathJax, enforces accessibility, and formats telemetry.
2. **Desktop Host (Python/Qt):** Manages webview lifecycle, routes bridge commands, coordinates audio/text hooks, and triggers Anki card answering.
3. **Core Engine (Rust):** Provides authoritative semantic validation, parses dimensional unit algebra, executes atomic SQLite transactions, and drives FSRS rating derivation.

---

## 2. Comprehensive Bridge Command Protocol Catalog

All frontend-to-backend communication flows via `bridgeCommand("<command>")` (implemented in `@tslib/bridgecommand`). Python receives these commands via `Reviewer._linkHandler(self, url: str)` and routes them through `Reviewer._handle_procedural_command(self, url: str)` in `qt/aqt/reviewer.py:697-825`.

| Command Protocol Signature | Sender (TypeScript) | Receiver (Python `reviewer.py`) | JSON Payload Schema | Side Effect & Target Subsystem |
|---|---|---|---|---|
| `procedural_answer:<ease>` | `ProceduralReviewer.handleNext()` (`procedural.ts:1228`) | `_linkHandler`<br>(Lines 703-708) | None (Integer `<ease>` in URL: `1`, `3`, or `4`) | Sets `self.state = "answer"`; invokes `self._answerCard(val)` to execute FSRS review and pull the next card. |
| `procedural_attempt:<json>` | `ProceduralReviewer.finishAttempt()` (`procedural.ts:1183`) | `_on_procedural_attempt`<br>(Lines 758, 783-789) | `AttemptResultPayload` | Stores attempt snapshot in `self._last_procedural_attempt`; sets `state = "answer"`; reveals native ease buttons. |
| `procedural_hint:<json>` | `ProceduralReviewer.requestHint()` (`procedural.ts:649`) | `_on_procedural_hint`<br>(Lines 756, 779-782) | `HintRequestPayload` | Stores in `self._last_procedural_hint`; tracks hint exposure level and latency. |
| `procedural_validate_steps:<json>` | `StepwiseContainer.evaluateSteps()` (`stepwise_container.ts`) | `_on_procedural_validate_steps`<br>(Lines 760, 775-778) | `StepwiseValidationPayload` | Stores in `self._last_procedural_stepwise_validation`; records step error localization. |
| `procedural_mistake:<json>` | `ProceduralReviewer.selectMistakeCategory()` (`procedural.ts:988`) | `_on_procedural_mistake`<br>(Lines 762, 790-793) | `MistakeSelectionPayload` | Stores in `self._last_procedural_mistake`; captures student self-attribution for `DomainEvidence`. |
| `procedural_try_similar:<json>` | `ProceduralReviewer.handleTrySimilar()` (`procedural.ts:1195`) | `_on_procedural_try_similar`<br>(Lines 764, 794-802) | `TrySimilarPayload` | Displays tooltip `"Generating similar variant for {family_id}..."`; calls `self._showQuestion()` to re-render. |
| `procedural_practice_prerequisite:<json>` | `ProceduralReviewer.handlePracticePrerequisite()` (`procedural.ts:1214`) | `_on_procedural_practice_prerequisite`<br>(Lines 766, 803-809) | `PrerequisitePracticePayload` | Displays tooltip `"Practice Prerequisite: {target_skill_id}"`; triggers remedial navigation. |
| `procedural_declarative_recall:<json>` | `ProceduralReviewer.handleDeclarativeRecallAction()` (`procedural.ts:1204`) | `_on_procedural_declarative_recall`<br>(Lines 768, 810-823) | `DeclarativeRecallPayload` | Resolves target Anki card from `collection.anki2` or displays tooltip `"Declarative recall requested (tag: {tag})"`. |
| `statesMutated` | State mutation hook closure (`reviewer.py:1372`) | `_linkHandler`<br>(Lines 722-723) | None | Sets `self._states_mutated = True`, unblocking deferred ease button rendering. |
| `ans` | `ProceduralReviewer.finishAttempt()` (`procedural.ts:1186`) | `_linkHandler`<br>(Lines 698-699) | None | Synchronizes Anki webview state to show bottom ease buttons. |
| `ease<1..4>` | Native bottom ease buttons (`reviewer.py:1059`) | `_linkHandler`<br>(Lines 700-702) | None | Rates card with manual ease 1..4. |

---

## 3. JSON Payload Specifications

### 3.1 `AttemptResultPayload` (`procedural_attempt`)
Dispatched upon problem completion to record performance metrics:

```typescript
// TypeScript Interface (ts/reviewer/procedural.ts)
interface AttemptResultPayload {
    instanceId: string;
    familyId: string;
    schemaId: string;
    skillId: string;
    domain: string;
    answer: string;
    mode: "quick" | "stepwise" | "mcq" | "worked_example";
    steps: Array<{
        stepId: string;
        input: string;
        isCorrect: boolean;
        isDownstreamConsistent?: boolean;
    }>;
    hintsUsed: number;
    timeTakenMs: number;
    targetTimeMs: number;
    isCorrect: boolean;
    score: number; // 0.0 to 1.0
    speedQuadrant: "fluency_strength" | "speed_opportunity" | "strategy_trap" | "concept_setup";
    mistakeType?: "silly_mistake" | "pattern_not_recognized" | "formula_or_concept_misapplied" | "concept_not_known";
    domainEvidence?: Record<string, any>;
}
```

```rust
// Rust Deserialization Target (rslib/procedural/src/diagnostics/mod.rs)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttemptResultPayload {
    pub instance_id: ProblemInstanceId,
    pub family_id: ProblemFamilyId,
    pub schema_id: SchemaId,
    pub skill_id: SkillId,
    pub domain: Domain,
    pub answer: String,
    pub mode: String,
    pub steps: Vec<SubmittedStepRecord>,
    pub hints_used: u32,
    pub time_taken_ms: u64,
    pub target_time_ms: u64,
    pub is_correct: bool,
    pub score: f64,
    pub speed_quadrant: String,
    pub mistake_type: Option<String>,
    pub domain_evidence: Option<DomainEvidencePayload>,
}
```

### 3.2 `MistakeSelectionPayload` (`procedural_mistake`)
Dispatched when student categorizes their error in the `MistakeFooter`:

```json
{
  "instance_id": "inst-math-successive-9921",
  "family_id": "family.math.percentage.successive",
  "mistake_type": "pattern_not_recognized"
}
```

### 3.3 `HintRequestPayload` (`procedural_hint`)
Dispatched upon each progressive hint exposure:

```json
{
  "instance_id": "inst-phys-kinematics-102",
  "hint_level": 2,
  "step_id": "step_2_velocity_squared",
  "elapsed_ms": 14200
}
```

### 3.4 `StepwiseValidationPayload` (`procedural_validate_steps`)
Dispatched during multi-step derivation evaluation:

```json
{
  "instance_id": "inst-chem-stoich-401",
  "step_index": 1,
  "step_id": "step_moles_reactants",
  "expression": "n = 5.4 / 27.0 = 0.2 mol",
  "is_valid": true,
  "is_downstream_consistent": false,
  "first_error_step": null
}
```

### 3.5 `TrySimilarPayload` (`procedural_try_similar`)
Dispatched when regenerating problem instance:

```json
{
  "instance_id": "inst-reasoning-seating-881",
  "family_id": "family.reasoning.circular_seating.8person"
}
```

### 3.6 `PrerequisitePracticePayload` (`procedural_practice_prerequisite`)
Dispatched when launching remedial practice:

```json
{
  "instance_id": "inst-math-quad-12",
  "target_skill_id": "algebra.linear_factorization",
  "executable_schema_id": "schema.algebra.linear_factorization.v1"
}
```

---

## 4. Telemetry Packaging & Custom Data Lifecycle

### 4.1 Webview State Mutation Handshake
Telemetry is packaged into Anki's next-card state protobuf via `globalThis.anki.mutateNextCardStates` (`ts/reviewer/answering.ts:1-50`):

```typescript
globalThis.anki.mutateNextCardStates(
    (globalThis.anki as any)._state_mutation_key,
    async (states, customData) => {
        const telemetry = {
            v: 1,
            actualTimeMs: timeTakenMs,
            targetTimeMs: this.options.targetTimeMs,
            isCorrect: outcome.isCorrect,
            hintsUsed: this.hintsUsed,
            mistakeType: this.mistakeType,
            mode: this.mode,
            proceduralPerformance: {
                classification: outcome.isCorrect ? (isFast ? "fast_correct" : "slow_correct") : "incorrect",
                timeRatio: timeTakenMs / this.options.targetTimeMs,
                mistakeType: this.mistakeType,
                hintsUsed: this.hintsUsed,
            },
            proceduralRemediation: {
                needed: !outcome.isCorrect || this.hintsUsed >= 2,
                reason: this.mistakeType || (outcome.isCorrect ? "fluency" : "unclassified"),
                skillId: this.options.skillId,
                schemaId: this.options.schemaId,
                familyId: this.options.familyId,
                topicId: this.options.topicId,
            },
            attemptResult: {
                instanceId: this.options.instanceId,
                answer: data.answer,
                mode: this.mode,
                steps: data.steps,
                hintsUsed: this.hintsUsed,
                timeTakenMs,
                isCorrect: outcome.isCorrect,
                score: outcome.score,
                speedQuadrant: quadrantInfo.quadrant,
            },
        };

        for (const state of ["again", "hard", "good", "easy"]) {
            if (customData[state]) {
                customData[state].studylab = telemetry;
            }
        }
    }
);
```

### 4.2 Rust Scheduler Ingestion & Ephemeral Stripping
To maintain strict database isolation and respect Anki's **100-byte column limit** on `cards.data` (`custom_data`), the Rust answering pipeline intercepts, extracts, and strips the `studylab` telemetry payload prior to database commit:

```rust
// rslib/src/scheduler/answering/mod.rs:353-505
// 1. Extract JSON from card custom_data
if let Some(custom_data_str) = &card.custom_data {
    if let Ok(mut custom_data_json) = serde_json::from_str::<serde_json::Value>(custom_data_str) {
        if let Some(studylab_val) = custom_data_json.get_mut("studylab") {
            let telemetry: StudyLabTelemetry = serde_json::from_value(studylab_val.take())?;
            
            // 2. Persist directly to procedural.db (isolated SQLite database)
            if let Some(service) = &self.procedural_service {
                service.record_practice_attempt_atomic(
                    &telemetry.to_practice_attempt(card.id.0),
                    &telemetry.to_error_events(),
                    telemetry.variant_ref.as_deref(),
                    telemetry.target_time_ms,
                )?;
                
                // 3. Evaluate Remediation Queue Policy
                if telemetry.procedural_remediation.needed {
                    service.remediation_queue().enqueue(
                        telemetry.to_remediation_action()
                    )?;
                }
            }
            
            // 4. Strip studylab key so collection.anki2 custom_data never exceeds 100 bytes
            custom_data_json.as_object_mut().map(|obj| obj.remove("studylab"));
            card.custom_data = Some(serde_json::to_string(&custom_data_json)?);
        }
    }
}
```

---

## 5. Python Reviewer Hook Interception Lifecycle

The Python desktop host (`qt/aqt/reviewer.py`) executes a deterministic hook lifecycle:

```
[Card Load]
   │
   ├─► `Reviewer._showQuestion()`
   │      ├─ 1. Card Separation Check: `_is_procedural_card()`
   │      ├─ 2. Webview Cleanup: `globalThis.anki.procedural.destroyActive()` (prevents shortcut leaks)
   │      ├─ 3. State Setup: `self.state = "question"`, `self._states_mutated = False`
   │      ├─ 4. Card Preparation: `_mungeQA(q)`
   │      ├─ 5. HTML Evaluation: `self.web.eval("_showQuestion(...)")`
   │      ├─ 6. State Mutation Key Injection: `_run_state_mutation_hook()`
   │      └─ 7. UI Hooks: `gui_hooks.reviewer_did_show_question(card)`
   │
[Answer Reveal]
   │
   ├─► `Reviewer._showAnswer()`
   │      ├─ 1. State Setup: `self.state = "answer"`
   │      ├─ 2. Ease Button Synchronization: Calls `_showEaseButtons()`
   │      │     └─ If `not self._states_mutated`: defers 50ms via `progress.single_shot`
   │      └─ 3. UI Hooks: `gui_hooks.reviewer_did_show_answer(card)`
   │
[Card Answering]
   │
   ├─► `Reviewer._answerCard(ease)`
   │      ├─ 1. Hook Interception: `gui_hooks.reviewer_will_answer_card((True, ease), self, card)`
   │      ├─ 2. Backend Submission: `answer_card(self.mw, answer).run_in_background()`
   │      ├─ 3. Post-Answer Hook: `gui_hooks.reviewer_did_answer_card(self, card, ease)`
   │      └─ 4. Queue Advancement: `self.nextCard()`
   │
[Reviewer Teardown]
   │
   └─► `Reviewer.cleanup()`
          ├─ 1. Teardown Hook: `globalThis.anki.procedural.destroyActive()`
          ├─ 2. State Cleanup: `self.card = None`
          └─ 3. GUI Hooks: `gui_hooks.reviewer_will_end()`
```

---

## 6. Evaluation Boundaries: Client-Side vs Canonical Authority

To ensure maximum responsiveness without sacrificing mathematical or cognitive rigor, StudyLab demarcates client-side convenience from backend authority:

```
┌───────────────────────────────────────┬───────────────────────────────────────┐
│     CLIENT-SIDE EVALUATION (TS)       │      CANONICAL AUTHORITY (RUST)       │
├───────────────────────────────────────┼───────────────────────────────────────┤
│ • Zero-latency input feedback (<16ms) │ • Canonical Mathematical Truth        │
│ • Floating-point tolerance check      │ • StepValidator Linear Root Equality  │
│ • 5D Unit algebra & conversion table  │ • Downstream Consistency Tracking     │
│ • MCQ Option ID literal comparison    │ • Physical / Chemical Sanity Invariant│
│ • Speed Quadrant UI badges            │ • CSP Constraint Uniqueness Solvers   │
│ • Space/Enter reflection trapping     │ • EMA Mastery & 6-Gate Progression    │
│ • Telemetry formatting & dispatch     │ • ACID Atomic Storage in procedural.db│
└───────────────────────────────────────┴───────────────────────────────────────┘
```

> **Rule:** Client-side evaluation provides optimistic UI feedback; Rust backend validation maintains absolute cognitive and empirical ground truth.

---

## 7. Security, Sanitization & Fault Tolerance

1. **XSS Prevention & HTML Sanitization:**
   - User inputs and parameter strings are escaped using `escape_html()` (`rslib/procedural/src/reviewer/template.rs:18-35`) before injection into HTML templates.
   - LaTeX delimiters (`$...$`, `\(...\)`) are preserved while escaping dangerous characters (`&`, `<`, `>`, `"`, `'`).
2. **`<script>` Tag Breakout Defense:**
   - JSON data embedded inside `<script>` blocks is sanitized using `escape_json_for_script()` (`template.rs:37-45`), replacing `</` with `<\/` to prevent premature script block termination attacks.
3. **Malformed Bridge Payload Fault Tolerance:**
   - In `qt/aqt/reviewer.py:750-770`, all bridge commands wrap JSON parsing in `try/except Exception`. Malformed payloads log an error message and store `{"raw": str}` without crashing the Qt review loop.
4. **SQL Parameterization:**
   - 100% of SQLite database queries in `rslib/procedural/src/storage/` use parameterized arguments (`?1, ?2, ...` or `rusqlite::params!`), completely eliminating SQL injection vectors.
