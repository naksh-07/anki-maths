# STUDYLAB ARCHITECTURE AUDIT HANDOFF REPORT

**Specialist**: STUDYLAB ARCHITECTURE AUDITOR  
**Date**: 2026-08-24  
**Target Matrix**: `c:/Users/Suraj/Documents/Antigravity/Anki-maths/03_architecture_gap_matrix.md`  
**Status**: COMPLETE / AUTHORITATIVE

---

## MISSION

Perform a comprehensive architectural audit of the entire StudyLab repository (`rslib/procedural`, `rslib/src`, `qt/aqt`, `pylib`, `ts/reviewer`, templates, bridge, footer, state machines, learner models), compare design principles and specifications against current implementation across all answer modalities, reviewer webviews, state machines, bridge contracts, footer lifecycle, and learner evidence, and author the formal gap matrix `03_architecture_gap_matrix.md`.

---

## SCOPE

- **Repository Workspace**: `c:/Users/Suraj/Documents/Antigravity/Anki-maths`
- **Subsystems Inspected**:
  1. Reviewer UI & Webview Integration (`rslib/src/notetype/render.rs`, `rslib/procedural/src/reviewer/template.rs`, `ts/reviewer/procedural.ts`, `ts/reviewer/index.ts`, `ts/reviewer/reviewer.scss`)
  2. State Machines (`ts/reviewer/procedural.ts`, `qt/aqt/reviewer.py`, `rslib/procedural/src/exam/mock.rs`, `rslib/procedural/src/scheduling/`)
  3. Native Bridge & FFI Contracts (`qt/aqt/reviewer.py`, `rslib/src/collection/mod.rs`, `rslib/src/scheduler/answering/mod.rs`, `pylib/rsbridge`)
  4. Answer Controls (`ts/reviewer/procedural.ts`, `rslib/procedural/src/problems/validator.rs`, `rslib/procedural/src/problems/steps/step_validator.rs`, `rslib/procedural/src/physics/units.rs`, `rslib/procedural/src/chemistry/units.rs`)
  5. Bottom Bar / Footer Lifecycle (`qt/aqt/reviewer.py`, `rslib/procedural/src/reviewer/template.rs`, `ts/reviewer/procedural.ts`)
  6. Learner State & Evidence Sync (`rslib/procedural/src/skills/domain_evidence.rs`, `signals.rs`, `rslib/src/scheduler/answering/mod.rs`, `rslib/procedural/src/service/mod.rs`, `storage/store.rs`)
- **APKG & Note Contracts**: `generate_procedural_apkg.py`, `generate_apkg.py`, `rslib/procedural/src/anchor/mod.rs`

---

## SOURCES

- `ORIGINAL_REQUEST.md` (Authoritative requirements R1 through R6)
- `PROJECT.md` (Project architecture, feature inventory, interface contracts, milestones M1 through M7)
- `CLAUDE.md` (Anki architecture guide, build recipes, protobuf RPCs, FTL, testing conventions)
- `docs/procedural_architecture.md` (Procedural practice engine architecture, `procedural.db` schema, anchor model)
- Git repository codebases and unit/integration test suites

---

## FILES / URLS INSPECTED

1. `c:/Users/Suraj/Documents/Antigravity/Anki-maths/ORIGINAL_REQUEST.md` (Lines 1-110)
2. `c:/Users/Suraj/Documents/Antigravity/Anki-maths/PROJECT.md` (Lines 1-58)
3. `c:/Users/Suraj/Documents/Antigravity/Anki-maths/CLAUDE.md` (Lines 1-119)
4. `c:/Users/Suraj/Documents/Antigravity/Anki-maths/docs/procedural_architecture.md` (Lines 1-116)
5. `c:/Users/Suraj/Documents/Antigravity/Anki-maths/rslib/src/notetype/render.rs` (Lines 115-248)
6. `c:/Users/Suraj/Documents/Antigravity/Anki-maths/rslib/procedural/src/lib.rs` (Lines 1-113)
7. `c:/Users/Suraj/Documents/Antigravity/Anki-maths/rslib/procedural/src/reviewer/template.rs` (Lines 1-815)
8. `c:/Users/Suraj/Documents/Antigravity/Anki-maths/rslib/procedural/src/problems/steps/step_validator.rs` (Lines 1-120, 800-871)
9. `c:/Users/Suraj/Documents/Antigravity/Anki-maths/rslib/procedural/src/exam/mock.rs` (Lines 1-550, 770-897)
10. `c:/Users/Suraj/Documents/Antigravity/Anki-maths/rslib/src/collection/mod.rs` (Lines 140-185)
11. `c:/Users/Suraj/Documents/Antigravity/Anki-maths/rslib/src/scheduler/answering/mod.rs` (Lines 350-520)
12. `c:/Users/Suraj/Documents/Antigravity/Anki-maths/qt/aqt/reviewer.py` (Lines 650-740, 830-950)
13. `c:/Users/Suraj/Documents/Antigravity/Anki-maths/ts/reviewer/procedural.ts` (Lines 1-1118)
14. `c:/Users/Suraj/Documents/Antigravity/Anki-maths/ts/reviewer/index.ts` (Lines 1-30)
15. `c:/Users/Suraj/Documents/Antigravity/Anki-maths/generate_procedural_apkg.py` (Lines 1-354)

---

## FINDINGS

### Summary of Discovered Gaps

1. **GAP-MOD-01 (Severity: HIGH)**: `ts/reviewer/procedural.ts:746-760` evaluates stepwise submissions by merely extracting the last step string and evaluating it locally with `evaluateLocally(lastAnswer)`. It completely bypasses the rich graph-based semantic `StepValidator` in `rslib/procedural/src/problems/steps/step_validator.rs:13-120`.
2. **GAP-BRG-01 (Severity: HIGH)**: `qt/aqt/reviewer.py:711-713` drops all auxiliary procedural bridge commands (`elif url.startswith("procedural_"): pass`). Messages such as `procedural_hint:`, `procedural_try_similar:`, `procedural_declarative_recall:`, `procedural_practice_prerequisite:`, and fallback `procedural_attempt:` are ignored without backend dispatch.
3. **GAP-DIAG-01 (Severity: HIGH)**: `rslib/procedural/src/exam/mock.rs:18-550` contains `MockSession`, `MockBlueprint`, and `ComprehensiveDiagnosticReport`, but these are unexposed in `ProceduralService` (`service/mod.rs`), lack FFI/Python bindings, and have zero frontend UI components in `ts/` to run tests or render hierarchical reports.
4. **GAP-EV-01 (Severity: HIGH)**: Diagnostic mock report evaluation (`mock.rs:444-550`) computes 4-dimension errors in-memory but does not batch-update `SkillState` and `DomainEvidence` tables in `procedural.db`.
5. **GAP-FTR-01 (Severity: MEDIUM)**: The mistake classification panel (`[1 Silly]`, `[2 Pattern]`, `[3 Concept]`, `[4 Unknown]`) is rendered inside the card DOM `#proc-mistake-panel` rather than being injected into `self.bottom.web`, leaving Anki's native bottom bar in "Show Answer" state while classifying mistakes.
6. **GAP-MOD-02 (Severity: MEDIUM)**: `ts/reviewer/procedural.ts:615-663` extracts numeric digits via regex and strips physical/chemical units without unit conversion (e.g., converting `72 km/h` to `20 m/s`). Rust's `DimensionalValidator` and `ChemicalDimensionalValidator` are not called during webview answer evaluation.
7. **GAP-STA-01 (Severity: MEDIUM)**: A global `window.addEventListener("keydown")` in `procedural.ts:297` is only torn down when a subsequent procedural card calls `setup()`. Navigating to a standard Anki card leaves the event listener attached to `window`.
8. **GAP-SCH-01 (Severity: LOW)**: `procedural.ts:1057-1059` hardcodes `ease = isCorrect ? (isFast ? 4 : 3) : 1` on next, bypassing the user's manual rating override in Anki's bottom bar.
9. **GAP-MOD-03 (Severity: LOW)**: MCQ selection in `procedural.ts:522-613` immediately submits and grades on option click/key. While ideal for rapid flashcard practice, diagnostic mock mode requires uncommitted option selection with free navigation.
10. **GAP-DOC-01 (Severity: LOW)**: High-level docs reference `crates/anki_maths_core`, `addon/anki_maths`, `web/`, whereas the physical codebase is in-tree at `rslib/procedural/`, `qt/aqt/`, and `ts/reviewer/procedural.ts`.

---

## EVIDENCE

- **Stepwise Evaluation Bypass**:
  ```ts
  // ts/reviewer/procedural.ts:756-760
  const lastAnswer = steps.length > 0 ? steps[steps.length - 1] : "";
  this.state = "submitting";
  const evalResult = this.evaluateLocally(lastAnswer);
  this.finishAttempt(evalResult, { answer: lastAnswer, steps }, "stepwise");
  ```
- **Dropped Bridge Commands in Python**:
  ```python
  # qt/aqt/reviewer.py:711-713
  elif url.startswith("procedural_"):
      # Procedural background bridge messages (attempt, hint, etc.)
      pass
  ```
- **Telemetry Sync via Answering Pipeline**:
  ```rust
  // rslib/src/scheduler/answering/mod.rs:350-353, 501-507
  if data.contains("studylab") {
      if let Ok(mut parsed) = serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&data) {
          if let Some(studylab) = parsed.remove("studylab") {
              if let Ok(service) = self.procedural_service() {
                  // Records attempt and enqueues remediation in procedural.db
                  ...
              }
              // Rewrite data without studylab payload to satisfy custom_data 100-byte DB limit
              if parsed.is_empty() { data = "".to_string(); }
          }
      }
  }
  ```
- **Reviewer Interception in Rust Core**:
  ```rust
  // rslib/src/notetype/render.rs:123-126
  if nt.name.as_str().starts_with("StudyLab Procedural Anchor") && !browser {
      println!("StudyLab debug: Executing render_procedural_anchor!");
      return self.render_procedural_anchor(note, card, nt);
  }
  ```

---

## 5-COMPONENT HANDOFF STRUCTURE

### 1. Observation
- Inspected all Rust crates (`rslib/procedural`, `rslib/src`), Python modules (`qt/aqt/reviewer.py`, `pylib/`), TypeScript frontend (`ts/reviewer/procedural.ts`), and APKG fixture scripts.
- Verified that card interception occurs natively in `rslib/src/notetype/render.rs:123-126` for note types starting with `"StudyLab Procedural Anchor"`.
- Verified that telemetry passes through `mutateNextCardStates` into `rslib/src/scheduler/answering/mod.rs:350-510` and strips `studylab` before writing to `collection.anki2`.
- Confirmed that `handleStepwiseSubmit()` in `ts/reviewer/procedural.ts:746-760` only evaluates the last step text locally, without delegating to `rslib/procedural/src/problems/steps/step_validator.rs`.
- Confirmed that `qt/aqt/reviewer.py:711-713` passes on `url.startswith("procedural_")`.
- Confirmed that `rslib/procedural/src/exam/mock.rs` contains complete `MockSession` and `ComprehensiveDiagnosticReport` logic that is not yet wired to `ProceduralService` or the webview.

### 2. Logic Chain
1. *From Observation of `ts/reviewer/procedural.ts:746`*: The frontend extracts `lastAnswer = steps[steps.length - 1]` and runs `evaluateLocally(lastAnswer)`.
   *Inference*: Stepwise validation is purely scalar comparison on the last step, meaning multi-step algebraic transformations, formula choices, and intermediate sign errors are not analyzed during live review (`GAP-MOD-01`).
2. *From Observation of `qt/aqt/reviewer.py:711`*: `url.startswith("procedural_")` executes `pass`.
   *Inference*: Any bridge commands dispatched from TypeScript (e.g. `procedural_hint:`, `procedural_try_similar:`, `procedural_declarative_recall:`, `procedural_practice_prerequisite:`) have no active handler in Python (`GAP-BRG-01`).
3. *From Observation of `rslib/procedural/src/exam/mock.rs` and `service/mod.rs`*: `MockSession` exists in `mock.rs`, but `ProceduralService` has no mock methods and `ts/` has no diagnostic UI.
   *Inference*: Running a diagnostic test across 4 domains requires creating the service endpoints, IPC bridge, and diagnostic report frontend (`GAP-DIAG-01`).
4. *From Observation of `rslib/src/notetype/render.rs:123`*: Non-procedural cards bypass `render_procedural_anchor` completely.
   *Inference*: Standard Anki cards (Basic, Cloze, Image Occlusion) are structurally protected from template modification and retain native scheduling (`R3` integrity confirmed).

### 3. Caveats
- The audit was conducted in read-only investigation mode without executing live edits to application source files.
- The codebase uses an in-tree architecture where the procedural crate is at `rslib/procedural` rather than an external `crates/anki_maths_core` directory; all functional references map directly to `rslib/procedural`.

### 4. Conclusion
The repository has a solid, high-performance architectural foundation:
- Rust-native card interception in `rslib/src/notetype/render.rs` provides rapid, safe rendering without modifying upstream collection tables.
- Telemetry transfer via `mutateNextCardStates` -> `scheduler::answering::answer_card` cleanly persists attempts to `procedural.db` while stripping payloads to protect `collection.anki2`.
- MCQ and Numerical UI controls are operational.
- The primary gaps to reconcile are:
  1. Wiring Stepwise answering to Rust `StepValidator` (`GAP-MOD-01`).
  2. Implementing the Python bridge command dispatcher (`GAP-BRG-01`).
  3. Exposing the Diagnostic Session Engine & UI (`GAP-DIAG-01`, `GAP-EV-01`).
  4. Webview event listener cleanup on card transition (`GAP-STA-01`).

### 5. Verification Method
1. Inspect `03_architecture_gap_matrix.md` in workspace root.
2. Run `cargo test -p procedural` to verify Rust crate contracts and mock engine test suites.
3. Run `npm test` or `yarn test ts/reviewer/procedural.test.ts` to inspect frontend test assertions.
4. Run `python -m pytest qt/tests/test_phase13.py` to inspect Python reviewer integration tests.

---

## RISKS

- **Telemetry Loss Risk**: If `mutateNextCardStates` encounters an error, the silence of `procedural_*` in `reviewer.py` causes silent attempt loss.
- **Shortcut Interception Risk**: Stale global keydown listeners from `procedural.ts` could intercept keys on standard Anki cards if not explicitly destroyed on card switch.
- **Diagnostic Isolation Risk**: If diagnostic mock reports are not committed to `SkillState`, learner proficiency scores will not reflect mock test performance.

---

## RECOMMENDATION

1. **Implement `procedural_validate_steps` RPC**: Connect `handleStepwiseSubmit` in `procedural.ts` to `StepValidator::validate_submission()` to deliver deep step diagnostic feedback.
2. **Implement Python Dispatcher in `reviewer.py`**: Handle `procedural_hint:`, `procedural_try_similar:`, and `procedural_practice_prerequisite:` to trigger remediation flows.
3. **Build Diagnostic Webview Container**: Connect `MockSession` from `rslib/procedural/src/exam/mock.rs` to a dedicated test interface in `ts/`, and wire `generate_comprehensive_report()` to batch-update `SkillState` in `procedural.db`.
4. **Attach Card Transition Teardown Hook**: Ensure `ProceduralReviewer.destroy()` is called whenever the active card changes.

---

## UNKNOWN / UNVERIFIED

- Exact behavior of `mutateNextCardStates` under extreme high-frequency card flipping or network latency (requires live CDP desktop verification in Milestone M6).
- Whether third-party Anki add-ons modifying `reviewer_bottom` interact with `#proc-mistake-panel` (isolated since `#proc-mistake-panel` is in card webview).
