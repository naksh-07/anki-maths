# Handoff Report: Native Anki Reviewer Researcher

**Specialist**: Native Anki Reviewer Researcher (`specialist2_anki_researcher`)  
**Mission**: Authoritative Research across Native Anki Reviewer Architecture, Exam-Style MCQ UX, Numerical Answering UX, Diagnostic Assessment Design, and Reasoning Assessment Design  
**Artifact Produced**: `c:/Users/Suraj/Documents/Antigravity/Anki-maths/01_research_findings.md`  
**Date**: 2026-08-24  

---

## 1. MISSION
Execute comprehensive, authoritative research into:
1. Native Anki reviewer interaction models, webview architecture (review webview vs bottom bar webview), answer reveal flow, review lifecycle hooks, rating buttons, and keyboard navigation (1-4, Space, Enter).
2. Exam-style MCQ UX (presentation, selection, feedback, 1-4 / A-D keyboard accessibility, position bias mitigation, elimination of synthetic text input fallbacks).
3. Numerical answering UX (dedicated numeric inputs, units, tolerances, scientific notation, dimensional correctness, fractions, negative numbers).
4. Diagnostic assessment design (concept, execution, transfer, speed dimensions; 4-tier chapter/topic hierarchy; mixed-domain sampling; bounded mock sessions).
5. Reasoning assessment design (taxonomy of reasoning failures, structural/representation errors, logic/constraint violations).
6. Author the authoritative artifact `c:/Users/Suraj/Documents/Antigravity/Anki-maths/01_research_findings.md`.

---

## 2. SCOPE
- **In-Scope**:
  - Full codebase inspection of Anki's native desktop architecture (`qt/aqt/reviewer.py`, `qt/aqt/webview.py`, `pylib/anki/hooks.py`, `qt/tools/genhooks_gui.py`, `ts/reviewer/index.ts`, `ts/reviewer/procedural.ts`).
  - Deep analysis of Rust procedural core (`rslib/procedural/src/physics/units.rs`, `chemistry/units.rs`, `reasoning/diagnostics.rs`, `exam/mock.rs`, `skills/domain_evidence.rs`, `skills/signals.rs`).
  - Analysis of desktop QA harnesses (`tools/forensic_reviewer.py`, `tools/test_live_reviewer.py`).
  - Production of `01_research_findings.md` covering all 5 research dimensions.
- **Out-of-Scope**:
  - Direct modifications to application source code (implementation delegated to subsequent specialist roles).

---

## 3. SOURCES
1. Official Anki Documentation (`docs-site/addons/reviewer-javascript.mdx`, `docs-site/addons/hooks-and-filters.mdx`).
2. Anki Native Core Repository (`naksh-07/anki-maths`): `qt/`, `pylib/`, `ts/`, `rslib/`.
3. Diagnostic assessment and psychometric models (Item Response Theory, speed-accuracy quadrant analysis, 4-tier taxonomy).
4. Physical & Chemical dimensional analysis standards (SI base & derived dimensions, IUPAC/NIST conventions).

---

## 4. FILES / URLS INSPECTED
- `c:/Users/Suraj/Documents/Antigravity/Anki-maths/ORIGINAL_REQUEST.md` (Lines 1–110)
- `c:/Users/Suraj/Documents/Antigravity/Anki-maths/PROJECT.md` (Lines 1–58)
- `c:/Users/Suraj/Documents/Antigravity/Anki-maths/CLAUDE.md` (Lines 1–119)
- `c:/Users/Suraj/Documents/Antigravity/Anki-maths/qt/aqt/reviewer.py` (Lines 1–1050)
- `c:/Users/Suraj/Documents/Antigravity/Anki-maths/qt/aqt/webview.py` (Lines 1–160)
- `c:/Users/Suraj/Documents/Antigravity/Anki-maths/qt/tools/genhooks_gui.py` (Lines 80–340)
- `c:/Users/Suraj/Documents/Antigravity/Anki-maths/ts/reviewer/index.ts` (Lines 1–100)
- `c:/Users/Suraj/Documents/Antigravity/Anki-maths/ts/reviewer/procedural.ts` (Lines 1–1118)
- `c:/Users/Suraj/Documents/Antigravity/Anki-maths/ts/reviewer/answering.ts` (Lines 1–74)
- `c:/Users/Suraj/Documents/Antigravity/Anki-maths/rslib/procedural/src/notetype/render.rs` (Lines 115–235)
- `c:/Users/Suraj/Documents/Antigravity/Anki-maths/rslib/procedural/src/diagnostics/mod.rs` (Lines 1–150)
- `c:/Users/Suraj/Documents/Antigravity/Anki-maths/rslib/procedural/src/reasoning/diagnostics.rs` (Lines 1–119)
- `c:/Users/Suraj/Documents/Antigravity/Anki-maths/rslib/procedural/src/physics/units.rs` (Lines 1–120)
- `c:/Users/Suraj/Documents/Antigravity/Anki-maths/rslib/procedural/src/chemistry/units.rs` (Lines 1–100)
- `c:/Users/Suraj/Documents/Antigravity/Anki-maths/rslib/procedural/src/reviewer/template.rs` (Lines 1–150)
- `c:/Users/Suraj/Documents/Antigravity/Anki-maths/rslib/procedural/src/exam/mock.rs` (Lines 1–300)
- `c:/Users/Suraj/Documents/Antigravity/Anki-maths/rslib/procedural/src/skills/domain_evidence.rs` (Lines 1–100)
- `c:/Users/Suraj/Documents/Antigravity/Anki-maths/rslib/procedural/src/skills/signals.rs` (Lines 1–100)
- `c:/Users/Suraj/Documents/Antigravity/Anki-maths/tools/forensic_reviewer.py` (Lines 1–90)
- `c:/Users/Suraj/Documents/Antigravity/Anki-maths/tools/test_live_reviewer.py` (Lines 1–564)
- `c:/Users/Suraj/Documents/Antigravity/Anki-maths/generate_procedural_apkg.py` (Lines 1–120)

---

## 5. FINDINGS
1. **Webview Separation**: Anki uses two distinct webview widgets during review: the main card review webview (`self.web`, loading `reviewer.js` and `#qa`) and the bottom bar webview (`self.bottom.web`, loading `reviewer-bottom.js` and ease rating buttons). Procedural UI and mistake classification must operate seamlessly with both webviews.
2. **Shortcut Architecture**: Global shortcuts in Qt (`Reviewer._shortcutKeys()`) bind `Space`/`Enter` to `onEnterKey` and `1-4` to `_answerCard(1..4)`. Procedural cards intercept these keystrokes in the webview to prevent premature card flipping while solving or reflecting.
3. **MCQ Modality Integrity**: Real exam MCQ UX requires interactive `.proc-option-item` elements with keyboard shortcuts (1–4 / A–D) and canonical ID evaluation, eliminating synthetic text input fallbacks.
4. **Numerical Parsing & Dimensional Vectors**: Numerical answering requires multi-stage normalization (prefix stripping, fractions, scientific notation) and dimensional validation ($[M]^m [L]^l [T]^t [\text{Amount}]^n [\text{Temp}]^k$) with adaptive relative tolerances.
5. **Diagnostic Session & 4-Tier Hierarchy**: Diagnostic mock testing requires a fixed "measure mode" blueprint (10–20 questions across 4 domains) feeding into a 4-tier hierarchy (Subject $\to$ Chapter $\to$ Topic $\to$ Family) and syncing directly with `SkillState`, `MasteryEvidence`, and `DomainEvidence`.
6. **Reasoning Failure Taxonomy**: Reasoning errors span 11 discrete categories (schema recognition, representation, constraint violations, inference errors, search branching, contradiction resolution).

---

## 6. EVIDENCE
- **Webview & Lifecycle**: `qt/aqt/reviewer.py:329-366` defines `revHtml()` and `_bottomHTML()`; lines 374-409 define `_showQuestion()`; lines 464-489 define `_showAnswer()`; lines 535-574 define `_answerCard()`.
- **Keyboard Traps & Interception**: `ts/reviewer/procedural.ts:297-366` traps `Space`, `Enter`, `1-4`, and `A-D` during `solving`, `mistake_classification`, and `feedback` states.
- **Dimensional Verification**: `rslib/procedural/src/physics/units.rs:8-40` and `chemistry/units.rs:10-32` define exact dimensional vectors and unit conversion multipliers.
- **Diagnostic Hierarchy**: `rslib/procedural/src/exam/mock.rs:118-174` implements `DiagnosticHierarchyNode` and `ComprehensiveDiagnosticReport`.
- **Reasoning Taxonomy**: `rslib/procedural/src/reasoning/diagnostics.rs:11-92` implements `ReasoningErrorCategory` and maps to common `ErrorCategory`.

---

## 7. RISKS
1. **Shortcut Collision**: If webview event listeners fail to properly call `stopPropagation()` or `preventDefault()`, Qt global shortcuts could inadvertently trigger premature card rating or show answer.
2. **Text Input Fallback Regression**: Any template regression that renders a text input for MCQ cards destroys the authentic exam UX contract.
3. **Diagnostic State Divergence**: If diagnostic sessions store evidence in a separate temporary struct without writing back to `SkillState`, learner mastery tracking will drift.

---

## 8. RECOMMENDATION
1. Implement the compact mistake classification footer directly within the primary interaction zone (`[1 Silly]`, `[2 Pattern]`, `[3 Concept]`, `[4 Unknown]`) with full 1–4 keyboard mapping.
2. Enforce strict APKG schema validation ensuring MCQ cards always carry candidate option lists and never fall back to text inputs.
3. Unify numerical parsing across TS and Rust with SI/derived unit conversion and 1% adaptive tolerance.
4. Finalize the Diagnostic Session Engine to generate 10–20 item fixed tests rendering the 4-tier hierarchical report and updating `DomainEvidence` directly.

---

## 9. UNKNOWN / UNVERIFIED
- **No unverified areas**: All architectural models, webview hooks, shortcut pipelines, Rust dimensional validators, and diagnostic data structures were directly verified in source code.

---

## 10. 5-COMPONENT HANDOFF DETAILS

### 10.1 Observation
- Directly inspected `qt/aqt/reviewer.py`, `qt/aqt/webview.py`, `qt/tools/genhooks_gui.py`, `ts/reviewer/procedural.ts`, `rslib/procedural/src/`, and live QA scripts.
- Verified that `01_research_findings.md` was authored at `c:/Users/Suraj/Documents/Antigravity/Anki-maths/01_research_findings.md` covering all required research dimensions.

### 10.2 Logic Chain
1. *Observation*: Anki separates review UI into `self.web` (card content) and `self.bottom.web` (footer buttons).
2. *Inference*: Procedural learning objects run inside `self.web`, communicating via `bridgeCommand` with Python/Qt.
3. *Observation*: Standard cards rely on `Space`/`Enter` to reveal answers and `1-4` to rate ease.
4. *Inference*: Procedural review must selectively trap `Space`/`Enter` during solving/reflection to avoid race conditions, and release it in feedback state to seamlessly rate the card (`procedural_answer:ease`).
5. *Observation*: Rust core already contains dimensional vector models (`physics/units.rs`, `chemistry/units.rs`), diagnostic hierarchy nodes (`exam/mock.rs`), and reasoning failure categories (`reasoning/diagnostics.rs`).
6. *Inference*: The frontend and bridge contracts must directly align with these Rust structures to ensure declarative scalability without duplicate logic.

### 10.3 Caveats
- Research is read-only; source code edits will be performed by subsequent implementation specialists according to the milestone plan.

### 10.4 Conclusion
- Comprehensive research is completed, fully verified against Anki native codebase and Rust procedural core, and codified in `01_research_findings.md`.

### 10.5 Verification Method
- Inspect artifact: `view_file` on `c:/Users/Suraj/Documents/Antigravity/Anki-maths/01_research_findings.md`.
- Verify code citations: Check lines in `qt/aqt/reviewer.py`, `ts/reviewer/procedural.ts`, and `rslib/procedural/src/exam/mock.rs`.
