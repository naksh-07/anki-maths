# StudyLab Diagnostic Assessment & Remediation Subsystem

**Document Version:** 1.0.0 (Canonical)  
**Author:** Diagnostic Systems & Remediation Architect  
**Date:** 2026-08-25  
**Target Repository:** `Anki-maths` (StudyLab Subsystem)  
**Status:** AUTHORITATIVE SPECIFICATION  
**Integrity Mode:** Benchmark Mode (100% Grounded in Executable Code, Tests, and Diagnostic Telemetry Artifacts)  

---

## 1. Executive Summary & Closed-Loop Architecture

StudyLab combines standardized diagnostic assessment with just-in-time (JIT) multi-tier remediation to form a **Closed-Loop Learning Architecture**. 

Grounded in Kurt VanLehn's (1990) Repair Theory and the Assessment Triangle (Pellegrino, Chudowsky, & Glaser 2001), the system ensures that:
1. Student errors are not merely tallied as binary failures, but are diagnosed into actionable cognitive and domain-specific etiologies.
2. Interventions escalate systematically to halt unproductive wheel-spinning.
3. Diagnostic assessment results synchronize atomically into the persistent learner model (`SkillState` in `procedural.db`), steering subsequent spaced practice without creating siloed test data.

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                    CLOSED-LOOP DIAGNOSTIC & REMEDIATION PIPELINE                │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                 │
│   [Practice Attempt / Diagnostic Session]                                       │
│          │                                                                      │
│          ▼                                                                      │
│   [Domain Evidence Extraction] ──► `DomainEvidencePayload`                      │
│          │                                                                      │
│          ▼                                                                      │
│   [Cognitive Error Diagnosis]  ──► `ErrorCategory` & `StepErrorType`            │
│          │                                                                      │
│          ▼                                                                      │
│   [Remediation Policy]         ──► `RemediationPolicy::evaluate()`              │
│          │                                                                      │
│          ▼                                                                      │
│   [Targeted Intervention]      ──► `ConceptCheck`, `WorkedExample`, `Prereq`    │
│          │                                                                      │
│          ▼                                                                      │
│   [Learner State Update]       ──► Atomic Sync to `SkillState` in SQLite DB     │
│                                                                                 │
└─────────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Adaptive Practice vs. Diagnostic Session

StudyLab differentiates between adaptive review practice and standardized diagnostic mock testing via the `PracticeObjective` enum (`rslib/procedural/src/practice/request.rs`):

```
┌─────────────────────────────────────────────────────────────────────────┐
│              ADAPTIVE PRACTICE VS DIAGNOSTIC SESSIONS                   │
├───────────────────────────────┬─────────────────────────────────────────┤
│ ADAPTIVE PRACTICE             │ DIAGNOSTIC MOCK SESSION                 │
│ (`PracticeObjective::Practice`)│ (`PracticeObjective::Diagnose` / `Mock`)│
├───────────────────────────────┼─────────────────────────────────────────┤
│ • Dynamic difficulty scaling  │ • Fixed, standardized blueprint         │
│ • Immediate error feedback    │ • Zero mid-test feedback / spoilers     │
│ • Injects immediate JIT hints │ • Real-time countdown timer & palette   │
│ • Mutates FSRS card intervals │ • Comprehensive post-test report        │
│ • Micro-session optimization  │ • Macro baseline profiling & sync       │
└───────────────────────────────┴─────────────────────────────────────────┘
```

### 2.1 Diagnostic Mode as a Measurement Tool
Diagnostic sessions are **measurement instruments**, not parallel learner models. Upon test completion, diagnostic evidence flushes directly into the unified `SkillState` table in `procedural.db` via `ProceduralService::record_diagnostic_report_evidence()`.

---

## 3. Diagnostic Mock Session Engine (`MockSession`)

The Diagnostic Mock Engine (`rslib/procedural/src/exam/mock.rs`) creates timed, unadapted test batteries across all 4 STEM domains.

### 3.1 Multi-Domain Sampling & Blueprint
`ProceduralService::create_diagnostic_session(total_questions, time_limit_ms, seed)` constructs a balanced `MockBlueprint`:
- **Item Battery:** Typically 10 to 20 questions sampled across **Mathematics**, **Physics**, **Chemistry**, and **Logical Reasoning**.
- **Fixed Scoring:** Evaluates with standard unadapted weights ($+1.0$ for correct, $0.0$ for incorrect/unanswered; zero negative penalization in diagnostic mode).
- **Time Allocation:** Configurable timed test window (e.g. 20 minutes for 15 items).

### 3.2 Frontend Diagnostic Session Controller (`DiagnosticSessionController`)
Implemented in `ts/reviewer/diagnostic/diagnostic_session.ts`:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                   DIAGNOSTIC SESSION USER INTERFACE                     │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│   [ ⏱ Time Remaining: 14:35 ]                         [ 🏁 Submit Test ]│
│                                                                         │
│   QUESTION PALETTE:                                                     │
│   ┌───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┐       │
│   │ 1 │ 2 │ 3 │ 4 │ 5 │ 6 │ 7 │ 8 │ 9 │10 │11 │12 │13 │14 │15 │       │
│   └───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───┘       │
│    ■ Answered (5)   □ Unvisited (8)   ◆ Marked for Review (2)           │
│                                                                         │
│   ───────────────────────────────────────────────────────────────────   │
│   Question 4 of 15 (Physics - Mechanics)                                │
│   A 2 kg block accelerates at 3 m/s². What is the net force applied?    │
│                                                                         │
│   [ (A) 6 N ]  [ (B) 1.5 N ]  [ (C) 5 N ]  [ (D) 9 N ]                  │
│                                                                         │
│   [ ◀ Previous ]  [ 🏷 Mark for Review (M) ]  [ 🗑 Clear ]  [ Next ▶ ]   │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

#### Key Capabilities & Keyboard Shortcuts
- **Question Palette Grid:** Color-coded status tiles (Unvisited, Answered, Marked for Review).
- **Navigation Shortcuts:** `ArrowLeft` / `ArrowRight` (previous/next question), `M` / `m` (toggle mark for review), `1`–`4` or `A`–`D` (direct option selection).
- **Countdown Timer:** Updates every second; triggers `.proc-timer-warning` at $\le 120\text{s}$ and auto-submits on expiration ($0\text{s}$).

---

## 4. Comprehensive 4-Tier Diagnostic Reporting

Upon test submission, `MockSession::generate_comprehensive_report()` (`rslib/procedural/src/exam/mock.rs:561-645`) generates a `ComprehensiveDiagnosticReport` rendered by `DiagnosticReportController` (`ts/reviewer/diagnostic/diagnostic_report.ts`).

```
┌─────────────────────────────────────────────────────────────────────────┐
│                   COMPREHENSIVE DIAGNOSTIC REPORT                       │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│   📊 DIAGNOSTIC SCORECARD                                               │
│   • Overall Score: 73.3% (11 / 15 Correct)                              │
│   • Time Spent: 14m 22s (Avg: 57.5s / question)                         │
│                                                                         │
│   ───────────────────────────────────────────────────────────────────   │
│   4-DIMENSION ERROR BREAKDOWN:                                          │
│   ┌────────────────────────┬────────────────────────┐                   │
│   │ 💡 Concept Deficits: 2 │ 🧮 Calculation Slips: 1│                   │
│   ├────────────────────────┼────────────────────────┤                   │
│   │ 🔄 Transfer Gaps: 1    │ ⏱ Speed Deficits: 2    │                   │
│   └────────────────────────┴────────────────────────┘                   │
│                                                                         │
│   ───────────────────────────────────────────────────────────────────   │
│   4-TIER CURRICULAR HIERARCHY DRILL-DOWN:                               │
│   ▼ [Mathematics] ────────────────────────── Accuracy: 100% (4/4)       │
│   ▼ [Physics] ────────────────────────────── Accuracy: 50%  (2/4)       │
│       ▼ Chapter: Mechanics                                              │
│           ▼ Topic: Kinematics 1D                                        │
│               • family.physics.kinematics.stopping_distance [❌ Concept]│
│   ▼ [Chemistry] ──────────────────────────── Accuracy: 75%  (3/4)       │
│   ▼ [Reasoning] ──────────────────────────── Accuracy: 66%  (2/3)       │
│                                                                         │
│   ───────────────────────────────────────────────────────────────────   │
│   RECOMMENDED REMEDIATION ACTIONS:                                      │
│   • [Practice Concept Check: Physics Kinematics Stopping Distance]      │
│   • [Start Timed Fluency Drill: Chemistry Stoichiometry]                │
│                                                                         │
│   [ 🚀 Start Recommended Remediation Path ]                             │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 4.1 The Four Diagnostic Error Dimensions
1. **💡 Concept Deficit:** Misconception, incorrect governing principle, or substrate failure (`concept_not_known`, `formula_or_concept_misapplied`).
2. **🧮 Execution / Calculation Slip:** Arithmetic mistake, algebraic sign flip, or unit conversion factor slip with correct conceptual setup.
3. **🔄 Transfer Deficit:** Failure to apply knowledge to unfamiliar, multi-concept, or structurally rotated problem variants (`pattern_not_recognized`).
4. **⏱ Speed Deficit:** Solved accurately, but response latency exceeded $> 1.25 \times$ target time budget.

### 4.2 Four-Tier Hierarchical Aggregation
The report organizes performance down the complete academic tree:
$$\text{Subject} \longrightarrow \text{Chapter} \longrightarrow \text{Topic} \longrightarrow \text{ProblemFamily}$$
allowing learners to pinpoint whether weakness is isolated to a specific formula or spans an entire chapter.

---

## 5. Batch SQLite Store Evidence Synchronization

When a diagnostic test finishes, `ProceduralService::record_diagnostic_report_evidence` (`rslib/procedural/src/service/mod.rs`) executes an atomic transaction updating the persistent database:

```rust
// rslib/procedural/src/service/mod.rs
pub fn record_diagnostic_report_evidence(
    &self,
    session: &MockSession,
    report: &ComprehensiveDiagnosticReport,
) -> Result<()>
```

### Transaction Steps
1. Opens an atomic SQLite transaction in `<collection>.procedural`.
2. For each tested question, creates a `PracticeAttempt` row with full telemetry and `origin = Origin::AuthenticPyq` or `DerivedVariant`.
3. Updates `SkillState` in `skill_states`:
   - Increments `total_attempts` and `successful_attempts`.
   - Appends outcome to `recent_attempts` sliding window.
   - Updates `latency_stats` with response time.
   - Records `domain_evidence` into `custom_state`.
4. Enqueues required remediation items in `remediation_queue_items` for all identified concept and strategy failures.
5. Commits the transaction atomically.

---

## 6. Multi-Tier Remediation Engine (`rslib/procedural/src/remediation/`)

The Remediation Engine provides automated, just-in-time cognitive interventions when errors occur during practice.

### 6.1 Nine-Tier Precedence Hierarchy
`RemediationPolicy` (`rslib/procedural/src/remediation/policy.rs`) categorizes interventions across **9 Precedence Tiers** ($10 \dots 90$):

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    REMEDIATION PRECEDENCE HIERARCHY                     │
├───────┬────────────────────────┬────────────────────────────────────────┤
│ Tier  │ Action Kind            │ Description & Trigger                  │
├───────┼────────────────────────┼────────────────────────────────────────┤
│ **90**│ `CircuitBreaker`       │ Cooldown halting wheel-spinning        │
│       │ (Advisory Urgency)     │ ($\ge 5$ repeat failures in session).  │
├───────┼────────────────────────┼────────────────────────────────────────┤
│ **80**│ `PrerequisiteReview`   │ Traverses DAG downward to review       │
│       │ (Critical Urgency)     │ foundational prerequisite skill.       │
├───────┼────────────────────────┼────────────────────────────────────────┤
│ **70**│ `WorkedExample`        │ Step-by-step annotated expert trace    │
│       │ (Critical Urgency)     │ with mandatory acknowledgement gate.   │
├───────┼────────────────────────┼────────────────────────────────────────┤
│ **60**│ `StrategyDrill`        │ Decision-point drill testing strategy  │
│       │ (Normal Urgency)       │ choice without arithmetic execution.   │
├───────┼────────────────────────┼────────────────────────────────────────┤
│ **50**│ `ConceptCheck`         │ Micro-object testing governing formula │
│       │ (Normal Urgency)       │ or conceptual principle.               │
├───────┼────────────────────────┼────────────────────────────────────────┤
│ **40**│ `RepresentationDrill`  │ Diagrammatic, coordinate frame, or     │
│       │ (Normal Urgency)       │ seating grid representation drill.     │
├───────┼────────────────────────┼────────────────────────────────────────┤
│ **30**│ `DeclarativeRecall`    │ Bridges to foundational Anki card/tag  │
│       │ (Normal Urgency)       │ for formula or constant recall.        │
├───────┼────────────────────────┼────────────────────────────────────────┤
│ **20**│ `ProceduralVariant`    │ Generates instance with simpler numbers│
│       │ (Low Urgency)          │ or isolated parameter complexity.      │
├───────┼────────────────────────┼────────────────────────────────────────┤
│ **10**│ `TransferRetry`        │ Fallback to standard structural level  │
│       │ (Low Urgency)          │ after a transfer failure.              │
└───────┴────────────────────────┴────────────────────────────────────────┘
```

### 6.2 Domain-Specific Error Mappings
`RemediationPolicy::evaluate()` maps diagnostic flags directly to pedagogical interventions:

| Discipline | Error Signal | Remediation Action | Operational Effect |
|---|---|---|---|
| **Mathematics** | `m.execution == false` | `ProceduralVariant` | Generates `"simpler_numbers"` variant to isolate arithmetic. |
| **Mathematics** | `m.method_selection == false` | `StrategyDrill` | Tests method selection (substitution vs elimination) without calculation. |
| **Mathematics** | `m.pattern_recognition == false`| `ConceptCheck` | Tests recognition of algebraic identity. |
| **Physics** | `p.unit_validity == false` | `ProceduralVariant` | Generates `"unit_conversion"` variant ($5/18$ factor focus). |
| **Physics** | `p.physical_model_selection == false`| `ConceptCheck` | Disambiguates Work-Energy vs Kinematics principles. |
| **Physics** | `p.representation == false` | `RepresentationDrill` | Practices Free-Body Diagram vector orientations. |
| **Chemistry** | `c.intermediate_quantity == false`| `ProceduralVariant` | Injects `"guided_steps"` for multi-step stoichiometry / ICE tables. |
| **Chemistry** | `c.mechanism_pathway == false` | `ConceptCheck` | Tests substrate electrophilicity ($S_N1$ vs $S_N2$). |
| **Reasoning** | `r.representation == false` | `RepresentationDrill` | Practices linear seating arrays or 2D constraint grids. |
| **Reasoning** | `r.decision_path == false` | `StrategyDrill` | Practices search tree branch pruning. |

### 6.3 Four-Stage Recurrence Escalation
When a learner repeatedly fails the same skill within a short sliding window, `RemediationPolicy` escalates intervention severity:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    RECURRENCE ESCALATION LIFECYCLE                      │
├───────────────────┬──────────────────────────┬──────────────────────────┤
│ Recurrence Stage  │ Action Kind Triggered    │ Urgency & Protocol       │
├───────────────────┼──────────────────────────┼──────────────────────────┤
│ **Recurrence 1–2**│ Targeted Micro-Object    │ Normal Urgency: Local    │
│                   │ (`ConceptCheck`, Variant)│ drill on error category  │
├───────────────────┼──────────────────────────┼──────────────────────────┤
│ **Recurrence 3**  │ `WorkedExample`          │ Critical Urgency: Expert │
│                   │                          │ modeling + ack gate      │
├───────────────────┼──────────────────────────┼──────────────────────────┤
│ **Recurrence 4**  │ `PrerequisiteReview`     │ Critical Urgency: Drops  │
│                   │                          │ 1 level down DAG         │
├───────────────────┼──────────────────────────┼──────────────────────────┤
│ **Recurrence ≥ 5**│ `CircuitBreaker`         │ Advisory Urgency: Pauses │
│                   │                          │ family; halts loop       │
└───────────────────┴──────────────────────────┴──────────────────────────┘
```

### 6.4 Same-Skill Queue Compaction
`RemediationQueue::enqueue()` (`rslib/procedural/src/remediation/queue.rs`) prevents queue bloat:
- If an existing remediation item for the same `skill_id` exists in the queue, it is **compacted** into a single authoritative item.
- Preserves the **highest urgency**, increments the **recurrence counter**, and updates to the **highest precedence action kind**.

---

## 7. Prerequisite Knowledge Graph (`PrerequisiteGraphService`)

The prerequisite knowledge graph (`rslib/procedural/src/skills/prerequisites.rs`) models hierarchical skill dependencies as a Directed Acyclic Graph (DAG).

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    PREREQUISITE KNOWLEDGE GRAPH (DAG)                   │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│   [Linear Equations 1-Var] ────► [Linear Equations 2-Var]               │
│             │                              │                            │
│             ▼                              ▼                            │
│   [Quadratic Equations]    ────► [Quadratic Word Problems]              │
│                                            │                            │
│                                            ▼                            │
│                                  [Optimization & Extrema]               │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 7.1 Readiness Evaluation Policy (`PrerequisitePolicy::evaluate_readiness`)
Before presenting advanced problems, the engine checks prerequisite readiness:
- **`Ready`:** All direct prerequisites have $\text{Mastery} \ge 0.70$ and progression state $\ge \text{Fluent}$.
- **`SoftAdvisory`:** Prerequisite mastery is in $[0.50, 0.70)$. Allows practice with an advisory note.
- **`Blocked`:** Prerequisite mastery $< 0.50$ or missing. Blocks advanced practice and recommends prerequisite review.

### 7.2 Topological Cycle Detection
`PrerequisiteGraphService::find_cycles()` uses Tarjan's strongly connected components / DFS traversal to guarantee graph acyclicity, preventing infinite circular dependencies during compilation.

---

## 8. Progressive 3-Tier Step Hints

When operating in Stepwise mode (`StepwiseContainer`), learners can request on-demand hints across 3 escalating levels:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                       3-TIER STEP HINT SCATTER                          │
├─────────┬──────────────────────────┬────────────────────────────────────┤
│ Level   │ Hint Tier                │ Pedagogical Function               │
├─────────┼──────────────────────────┼────────────────────────────────────┤
│ Level 1 │ `Principle`              │ Recalls governing law or identity  │
│ Level 2 │ `Operation`              │ Explains next algebraic action     │
│ Level 3 │ `IntermediateRelation`   │ Reveals concrete intermediate step │
└─────────┴──────────────────────────┴────────────────────────────────────┘
```

### 8.1 Hint Scoring & FSRS Rating Penalties
- Using Level 1 hint: Minor score penalty ($0.8 \times$ multiplier); allows "Good" rating if executed quickly.
- Using Level 2–3 hints: Caps rating at "Hard" or "Again" depending on recurrence.
- Using $\ge 3$ hints: Automatically maps to "Again" (1) in `derive_fsrs_rating()` to trigger spaced re-testing.

---

## 9. Verification & Codebase Traceability Matrix

| Subsystem Component | Source Code Reference | Test Evidence Suite |
|---|---|---|
| Diagnostic Mock Session | `rslib/procedural/src/exam/mock.rs:1-650` | `rslib/procedural/tests/diagnostic_mock_session_tests.rs` (5 tests) |
| Diagnostic Session UI | `ts/reviewer/diagnostic/diagnostic_session.ts:1-350` | `diagnostic_session.test.ts` (10 tests) |
| Diagnostic Report View | `ts/reviewer/diagnostic/diagnostic_report.ts:1-400` | `diagnostic_report.test.ts` (5 tests) |
| Batch Store Synchronization | `rslib/procedural/src/service/mod.rs` (`record_diagnostic_report_evidence`) | `rslib/procedural/src/service/tests` |
| Remediation Policy Engine | `rslib/procedural/src/remediation/policy.rs:1-400` | `rslib/procedural/tests/remediation_engine_tests.rs` (6 tests) |
| Queue Compaction & Loop Defense | `rslib/procedural/src/remediation/queue.rs:1-150` | `rslib/procedural/src/remediation/tests` |
| Prerequisite Graph & DAG | `rslib/procedural/src/skills/prerequisites.rs:1-200` | `rslib/procedural/src/skills/tests` |
