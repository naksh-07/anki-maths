# StudyLab Verification & Safety Rules

**Authority:** Level 1 Subsystem Verification and Safety Rule  
**Target Repository:** `naksh-07/anki-maths` (StudyLab Procedural Intelligence Subsystem)  
**Status:** CANONICAL FROZEN DIRECTIVE  
**Source of Truth:** Canonical Documentation (`docs/`), Master Contracts (`docs/contracts/`), and Executable Test Harnesses

---

## 1. Benchmark Integrity Mandate

```text
┌──────────────────────────────────────────────────────────────────────────────────┐
│                            BENCHMARK INTEGRITY MANDATE                           │
├──────────────────────────────────────────────────────────────────────────────────┤
│ ALL IMPLEMENTATIONS MUST BE GENUINE.                                             │
│                                                                                  │
│ • DO NOT create dummy, facade, or stub implementations that produce              │
│   correct-looking outputs without genuine underlying computational logic.        │
│ • DO NOT hardcode test results, expected outputs, or verification strings.       │
│ • DO NOT silently drift from architectural invariants or bypass validation.      │
│ • Every component must maintain real state and produce real behavior.            │
│ • DO NOT WEAKEN TESTS TO PASS: Never modify, comment out, skip, or delete        │
│   an existing test to make your change look successful. Failing tests are        │
│   signals about defects in the implementation, not tests to be weakened.        │
└──────────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Source-of-Truth Hierarchy & Authority Rules

When reconciling conflicting assertions across documentation, historical handoff reports, and source code, all claims must be strictly resolved according to the **8-Tier Source-of-Truth Hierarchy**:

```text
┌───────────────────────────────────────────────────────────────────────────────┐
│                     8-TIER SOURCE-OF-TRUTH HIERARCHY                         │
├───────┬──────────────────────────────────────┬────────────────────────────────┤
│ Tier  │ Authority Source                     │ Resolution Role                │
├───────┼──────────────────────────────────────┼────────────────────────────────┤
│ **1** │ Current Executable Source Code       │ Supreme Ground Truth           │
│ **2** │ Current Passing Test Suites          │ Behavioral Ground Truth        │
│ **3** │ Current Database Schemas / Migrations│ Structural Ground Truth        │
│ **4** │ Current Verified Artifacts           │ Empirical Ground Truth         │
│ **5** │ Frozen Master Specifications         │ Intent Ground Truth            │
│       │ - StudyLab-Source-APKG-Contract(1).txt│ Level 1 FROZEN Source Contract │
│       │ - docs/contracts/StudyLab-Runtime-*  │ Level 1 FROZEN Runtime Spec    │
│ **6** │ Canonical Suite Documentation        │ Explanatory Architectural Spec │
│       │ (`docs/ARCHITECTURE_INVARIANTS.md`,   │                                │
│       │  `docs/DOCUMENTATION_MAP.md`, etc.)  │                                │
│ **7** │ Historical Phase Reports (01 to 08)  │ Archaeological Context ONLY    │
│       │ (and historical handoff logs)        │ (Never Architectural Truth)    │
│ **8** │ General / Unverified Assumptions     │ Subordinate (Discard if non-   │
│       │                                      │ conforming)                    │
└───────┴──────────────────────────────────────┴────────────────────────────────┘
```

> [!WARNING]
> **Historical Reports Are Not Architectural Truth:**
> Phase reports (`01_research_findings.md` through `08_release_decision.md` and `HANDOFF_REPORT.md`) record point-in-time explorations and historical bug reproductions. Where a phase report contradicts current canonical contracts (e.g. historical reports discussing a "Next Problem" button or native ease button coexistence), current canonical contracts and executable code take precedence.

---

## 3. Mandatory Safety & Security Invariants

Every change touching StudyLab must verify adherence to the following four non-negotiable safety rules:

### 3.1 100% Parameterized SQL (Zero String Interpolation)
- **Rule:** Never use `format!`, string concatenation, or unescaped variables in SQL queries executed against `collection.procedural`.
- **Enforcement:** Every query must use numbered parameters (`?1, ?2, ...`) or `rusqlite::params!`.
- **Target Location:** `rslib/procedural/src/storage/store.rs`.

### 3.2 Webview HTML Escaping & XSS Sanitization
- **Rule:** All dynamic strings, problem statements, options, hints, and derivations passed into webview HTML templates must be sanitized.
- **Enforcement:** Text strings must pass through `escape_html()` in `rslib/procedural/src/reviewer/template.rs`.
- **Script Breakout Defense:** All JSON payloads embedded in `<script>` tags must escape `</script>` as `<\/script>`.

### 3.3 Webview Lifecycle Teardown & Event Listener Safety
- **Rule:** When navigating between cards or ending review sessions, no event listeners, keyboard shortcuts, or background timers may leak.
- **Enforcement:**
  - `ProceduralReviewer.destroy()` must unbind all window event listeners (`keydown`, `click`, etc.).
  - `globalThis.anki.procedural.destroyActive()` must be evaluated before mounting any card (`qt/aqt/reviewer.py:208, 416`).
  - Standard Anki cards (`Basic`, `Cloze`) must never experience keyboard interception from procedural listeners.

### 3.4 Telemetry 100-Byte Custom Data Limit
- **Rule:** Anki's upstream collection enforces a strict 100-byte database limit on `card.custom_data`.
- **Enforcement:** Rich procedural attempt telemetry (`domain_evidence`, mistake classifications, step traces) must be committed to `<collection>.procedural` and **stripped from `card.custom_data`** before saving to `collection.anki2` (`rslib/src/scheduler/answering/mod.rs:501–506`).

### 3.5 Forensic Desktop Invariants (Two-P0 Guardrails)
- **P0-A Anti-Bypass Trapping:** In standard Anki, pressing Space or Enter reveals the answer and rates the card. On procedural cards, `onEnterKey()` must intercept these keystrokes and delegate to `globalThis.anki.procedural.handleNativeShowAnswer()`, blocking learners from bypassing active problem solving or skipping mistake reflection (`qt/aqt/reviewer.py:704–710`).
- **P0-B Interaction Surface Deduplication:** Native Anki bottom rating buttons (`#ansbut`, `#ease1..4`) must never coexist with StudyLab's solving canvas. The host reviewer bridge (`qt/aqt/reviewer.py`) actively suppresses them via `_is_procedural_card()`.
- **Modality Purity & Single Interaction Surface:** The in-card solving canvas is the sole interaction surface. Superfluous "Next Problem" buttons are strictly eliminated in favor of automatic advance on correct submission and reflection gating on incorrect (`docs/contracts/StudyLab-Runtime-UI-Contract.md`).

---

## 4. Verified Verification Command Reference

All commands below have been tested and verified to work on this repository. Execute them from the repository root:

### 4.1 Rust Engine Verification Commands
- **Rust procedural crate compile check:**
  ```powershell
  cargo check -p procedural
  ```
- **Rust procedural unit test suite (146 lib unit tests):**
  ```powershell
  cargo test -p procedural --lib
  ```
- **175-Topic Universal Factory integration test:**
  ```powershell
  cargo test -p procedural --test phase36c_all_175_topics_factory_tests
  ```
- **Canonical Source APKG Contract test (18 tests):**
  ```powershell
  cargo test -p procedural --test canonical_source_contract_tests
  ```
- **Desktop Validation Master Suite (10 lifecycle & soak tests):**
  ```powershell
  cargo test -p procedural --test desktop_validation_master_suite
  ```
- **Production Hardening tests (7 concurrency & resilience tests):**
  ```powershell
  cargo test -p procedural --test phase40_production_hardening_tests
  ```
- **Self-Contained APKG test (2 tests):**
  ```powershell
  cargo test -p procedural --test phase35_apkg_self_contained
  ```

> [!NOTE]
> Avoid blanket `cargo test -p procedural` without `--lib` or specific `--test` targets: it runs multi-minute longitudinal simulations (`phase30`, `phase32` taking >4 minutes) and legacy phase tests with local path dependencies. Also avoid `cargo test --workspace` directly, as upstream Anki crates require build-time FTL translation submodules configured via `just test-rust`.

### 4.2 Dual Content APKG Generation & Validation Commands

#### Path 1: Canonical Source Static APKGs
- **Generate Canonical Source APKG fixture:**
  ```powershell
  python generate_canonical_source_apkg.py artifacts_qa/canonical_source_test_fixture.apkg
  ```
- **Validate Canonical Source APKG fixture against Level 1 contract:**
  ```powershell
  python artifacts_qa/validate_canonical_source_apkg.py artifacts_qa/canonical_source_test_fixture.apkg
  ```

#### Path 2: Procedural Blueprint APKGs & 175-Topic Universe
- **Validate all 175 topic declarative blueprints:**
  ```powershell
  python tools/studylab_content_factory.py --validate
  ```
- **Generate full 175-topic Universe APKGs:**
  ```powershell
  python tools/studylab_content_factory.py --generate-apkgs dist/apkgs
  ```
- **Validate generated Universe APKG (`dist/apkgs/StudyLab_Full_Universe_175.apkg`):**
  ```powershell
  python artifacts_qa/validate_canonical_apkg.py
  ```
- **Generate Procedural Blueprint single fixture:**
  ```powershell
  python generate_procedural_apkg.py dist/apkgs/procedural_fixture.apkg
  ```

#### Master Adversarial Audit
- **Run Challenger 2 Release Candidate Master Audit (4 Dimensions: Math CAS, SQLite Persistence, 100-Byte Telemetry, Cold-Start APKG):**
  ```powershell
  python artifacts_qa/challenger_2_master_runner.py
  ```

### 4.3 Python Desktop & Host GUI Verification
- **Run Python test suite (pylib + qt):**
  ```powershell
  just test-py
  ```
- **Run pytest directly (alternative):**
  ```powershell
  pytest qt/tests pylib/tests
  ```

### 4.4 Full System & Quality Gates
- **Code formatting check:**
  ```powershell
  just fmt
  ```
- **Auto-fix formatting:**
  ```powershell
  just fix-fmt
  ```
- **Comprehensive linter & type check:**
  ```powershell
  just lint
  ```
- **Full system build & check (final release gate):**
  ```powershell
  just check
  ```

> [!NOTE]
> Do not invoke `./ninja`, `./run`, or scripts under `./tools` directly — use the `just` recipes or standard `cargo` / `python` invocations as defined above.
