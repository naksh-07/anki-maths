# STUDYLAB REVIEWER UI CHANGE CONTRACT
## Canonical Frontend UI Implementation Contract for Procedural Reviewer Alignment

**Document Type:** Frontend UI Change / Implementation Contract  
**Document Version:** 1.0.0  
**Target Repository:** `naksh-07/anki-maths`  
**Scope:** StudyLab procedural reviewer frontend only  
**Status:** Proposed → Ready for Implementation after review  
**Authority Hierarchy:** Subordinate to Level 1 Master Contracts (`StudyLab-Source-APKG-Contract(1).txt`, `docs/ARCHITECTURE_INVARIANTS.md`) and Architectural Specifications (`docs/STUDYLAB_UI_COMPOSITION_CONTRACT.md`, `docs/PRODUCT_BOUNDARIES.md`).

---

## 1. Purpose & Core Principles

### 1.1 Objective
This contract defines the authoritative, implementation-ready specifications for the upcoming StudyLab procedural reviewer frontend correction pass.

The primary objective is:
> **Repair the existing StudyLab composition contract. Do not redesign StudyLab.**

The frontend procedural reviewer implementation has drifted in specific, verifiable ways from the canonical composition contract defined in [`docs/STUDYLAB_UI_COMPOSITION_CONTRACT.md`](file:///c:/Users/Suraj/Documents/Antigravity/Anki-maths/docs/STUDYLAB_UI_COMPOSITION_CONTRACT.md). This pass systematically resolves those specific defects and brings the runtime frontend into complete alignment with the composition contract, without altering the host Anki product shell, without redesigning visual themes, and without expanding scope into backend or scheduling layers.

### 1.2 Hierarchy of Repair Principle
When executing frontend corrections under this contract, every engineer and autonomous agent must strictly adhere to the **Hierarchy of Repair**:

```text
┌──────────────────────────────────────────────────────────────────────────────────┐
│                           HIERARCHY OF REPAIR PRINCIPLE                          │
├──────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│   Level 1: Ownership & Boundary Integrity                                       │
│            (Ensure Anki owns shell/counts; StudyLab owns solving canvas)         │
│               ↓                                                                  │
│   Level 2: DOM Composition & Modality Structure                                  │
│            (Mount correct controls; eliminate missing or duplicate elements)     │
│               ↓                                                                  │
│   Level 3: State Machine Visibility & Transition Flow                            │
│            (Hide/reveal according to active state; enforce reflection gate)      │
│               ↓                                                                  │
│   Level 4: Document Flow & Layout Hierarchy                                      │
│            (In-flow clearance; natural window scrolling; zero overlay traps)     │
│               ↓                                                                  │
│   Level 5: Decorative Styling & Token Tuning                                     │
│            (Colors, padding, borders, shadows — ONLY after Levels 1-4 are sound) │
│                                                                                  │
└──────────────────────────────────────────────────────────────────────────────────┘
```

> **Direct Rule:** When a visual or interaction defect can be resolved through ownership, DOM composition, state visibility, or natural document flow, fix that before modifying decorative CSS rules.

---

## 2. Product Ownership Boundary

StudyLab operates as a guest subsystem embedded within the native Anki desktop application. StudyLab must visually and behaviorally coexist with the host rather than behave like a detached web application or standalone website.

```text
┌──────────────────────────────────────────────────────────────────────────────────┐
│                      DESKTOP PRODUCT OWNERSHIP TOPOLOGY                          │
├────────────────────────────────────────┬─────────────────────────────────────────┤
│          ANKI-OWNED / FROZEN           │          STUDYLAB-OWNED / IN SCOPE      │
│         (Strictly Out of Scope)        │         (Frontend Reviewer Only)        │
├────────────────────────────────────────┼─────────────────────────────────────────┤
│ • Native Win32 Window & Chrome         │ • Procedural Problem Prompt (`#proc-prompt`)│
│ • Native Application Menus & Hotkeys   │ • Header Breadcrumbs & Provenance Badges│
│ • Top Navigation Toolbar (Decks, Add,   │ • Hint Button & Inline Scaffolding      │
│   Browse, Stats, Sync)                 │ • Numerical Solving Input & Unit Pill   │
│ • Native Reviewer Shell Container      │ • MCQ Option Cards (Radiogroups A..D)   │
│ • Standard Basic Cards                 │ • Stepwise Workspace Rows & CAS Badges  │
│ • Standard Cloze Cards                 │ • Wrong Status Indicator & Comparison   │
│ • Native Card Answer Buttons (#ansbut) │ • 4-Category Metacognitive Reflection   │
│ • Native Ease Rating Bar (Again, Hard, │ • Canonical LaTeX Derivation Review     │
│   Good, Easy) on Standard Cards        │ • Explicit In-Card "Next Problem" CTA   │
│ • Native Edit & More Context Menus     │ • Procedural Responsive/Layout Flow     │
│ • Anki Database (`collection.anki2`)   │ • Reviewer Keyboard Focus & Hotkeys     │
└────────────────────────────────────────┴─────────────────────────────────────────┘
```

### 2.1 Anki-Owned / Frozen (Out of Scope)
The following surfaces are **strictly out of scope** and must remain completely untouched:
- Top application toolbar: `Decks`, `Add`, `Browse`, `Stats`, `Sync`.
- Native menus and menu bar items.
- Native Win32/Qt desktop window chrome, title bars, and window borders.
- Standard declarative flashcard review pipelines:
  - Standard `Basic` card presentation and evaluation.
  - Standard `Cloze` card rendering and deletion reveals.
  - Standard image occlusion reviews.
- Native reviewer bottom action bar on non-procedural cards (`#ansbut`, ease ratings `1`..`4`).
- Native `Edit Current` (`E`) and `More` (`M`) context menus.
- Any composition, layout, or script outside `#procedural-card`.

### 2.2 StudyLab-Owned / In Scope
Only the procedural reviewer surface inside `#procedural-card` is mutable during this pass:
- Problem stem presentation and LaTeX typesetting.
- Progressive Hint trigger and inline hint drawer.
- Modality-matched solving surfaces: Numerical input (with live 5D vector preview pill), MCQ option cards (with zero text fallback), Stepwise multi-row derivation.
- Error outcome presentation and answer comparison.
- Metacognitive 4-category mistake classification reflection panel.
- Solution derivation trace and performance speed pill.
- Explicit `Next Problem` progression control.
- Viewport layout, vertical clearance, and keyboard focus routing on procedural cards.

---

## 3. Current-State Findings & Verified Implementation Drift

Every finding in this section has been forensically verified against the current repository source code (`rslib/procedural/src/reviewer/template.rs`, `ts/reviewer/procedural.ts`, `ts/reviewer/reviewer.scss`, and `qt/aqt/reviewer.py`).

### 3.1 Finding A: Quick Solve & MCQ Hint Button Visibility
- **Capability Status (Verified):** Functional in TypeScript. [`ProceduralReviewer`](file:///c:/Users/Suraj/Documents/Antigravity/Anki-maths/ts/reviewer/procedural.ts#L667) implements `requestHint()`, tracks `hintsUsed` and `hintTimestamps`, and dispatches `procedural_hint:<json>`.
- **HTML Template Reality (Verified):** In [`rslib/procedural/src/reviewer/template.rs`](file:///c:/Users/Suraj/Documents/Antigravity/Anki-maths/rslib/procedural/src/reviewer/template.rs#L476-L496), for `object_type == "problem"` / `"quick"`:
  ```html
  <!-- Quick Solve Mode -->
  <div id="proc-quick-container">
      <div class="proc-step-row">
          <input type="text" id="proc-answer-input" class="proc-input" placeholder="Type final answer..." autocomplete="off" />
          <button type="button" id="proc-submit-btn" class="proc-btn proc-btn-primary">Submit</button>
      </div>
  </div>

  <!-- Stepwise Solving Mode -->
  <div id="proc-stepwise-container" class="hidden">
      ...
      <div class="proc-controls">
          <button type="button" id="proc-add-step-btn" class="proc-btn proc-btn-secondary">+ Add Step</button>
          <button type="button" id="proc-hint-btn" class="proc-btn proc-btn-secondary">💡 Request Hint</button>
          <button type="button" id="proc-reset-steps-btn" class="proc-btn proc-btn-secondary">Reset</button>
          <button type="button" id="proc-check-steps-btn" class="proc-btn proc-btn-primary">Check Solution</button>
      </div>
  </div>
  ```
- **The Defect:**
  1. In Quick Solve mode, `#proc-hint-btn` is physically located inside `#proc-stepwise-container`, which has `class="hidden"`. `#proc-quick-container` contains only `#proc-answer-input` and `#proc-submit-btn`. The hint button is therefore **never visible or clickable** in Quick Solve mode.
  2. In MCQ mode ([`template.rs:185-234`](file:///c:/Users/Suraj/Documents/Antigravity/Anki-maths/rslib/procedural/src/reviewer/template.rs#L185-L234)), `#proc-hint-btn` is **completely omitted** from the template.
- **Contract Mandate:** The composition contract ([`docs/STUDYLAB_UI_COMPOSITION_CONTRACT.md`](file:///c:/Users/Suraj/Documents/Antigravity/Anki-maths/docs/STUDYLAB_UI_COMPOSITION_CONTRACT.md#L532)) explicitly requires `#proc-hint-btn` to be visible and operable in `ready`, `solving`, and `stepwise` states across `problem`, `quick`, and `mcq` objects.

### 3.2 Finding B: Classification Footer & Viewport Occlusion
- **Current CSS Reality (Verified):** In [`ts/reviewer/reviewer.scss:1137-1157`](file:///c:/Users/Suraj/Documents/Antigravity/Anki-maths/ts/reviewer/reviewer.scss#L1137-L1157):
  ```scss
  .proc-interaction-footer {
      position: fixed;
      bottom: 0;
      left: 0;
      width: 100%;
      background: var(--proc-surface);
      border-top: 1px solid var(--proc-border);
      padding: 12px 20px;
      z-index: 1000;
      display: none;
      flex-direction: column;
      align-items: center;
      box-shadow: 0 -4px 12px rgba(0, 0, 0, 0.08);
  }
  .proc-interaction-footer:has(> :not(.hidden)) {
      display: flex;
  }
  ```
  Container padding clearance in [`reviewer.scss:252-254`](file:///c:/Users/Suraj/Documents/Antigravity/Anki-maths/ts/reviewer/reviewer.scss#L252-L254):
  ```scss
  .procedural-card-container:has(.proc-interaction-footer:has(> :not(.hidden))) {
      padding-bottom: 90px;
  }
  ```
- **The Defect:**
  1. The reflection strip uses a hardcoded fixed overlay (`position: fixed; bottom: 0`).
  2. On smaller viewports (e.g. minimum supported $682 \times 607$), with long problem stems, multi-line reflection buttons, or high DPI scaling, the fixed footer overlays and occludes problem content, active focus outlines, or scrollable content.
  3. The hardcoded `padding-bottom: 90px` is brittle and arbitrary; when buttons wrap on narrow displays, the footer height exceeds 90px, causing bottom content to be permanently trapped beneath the overlay.
- **Contract Mandate:** The reflection footer must never obscure problem content, solution derivations, focused controls, or scrollable text. Layout fixes must prioritize document flow and proper containment over arbitrary padding overrides.

### 3.3 Finding C: Classification → Feedback Progression & Auto-Advance Drift
- **Canonical Contract Requirement:**
  $$\text{wrong\_answer} \longrightarrow \text{mistake\_classification} \longrightarrow \text{category selection} \longrightarrow \text{feedback} \longrightarrow \text{explicit Next Problem}$$
- **Current Code Reality (Verified):** In [`ts/reviewer/procedural.ts:1315-1328`](file:///c:/Users/Suraj/Documents/Antigravity/Anki-maths/ts/reviewer/procedural.ts#L1315-L1328):
  ```typescript
  // Next button has been removed per StudyLab Phase 2 requirements
  const nextBtn = this.container.querySelector<HTMLElement>("#proc-next-btn, .proc-next-btn");
  if (nextBtn) {
      nextBtn.classList.add("hidden");
      nextBtn.style.display = "none";
  }

  // Auto-advance transient feedback after a short delay
  setTimeout(() => {
      if (this.state === "feedback") {
          this.handleNext();
      }
  }, 1500);
  ```
  And in [`rslib/procedural/src/reviewer/template.rs:593-631`](file:///c:/Users/Suraj/Documents/Antigravity/Anki-maths/rslib/procedural/src/reviewer/template.rs#L593-L631), `#proc-next-btn` is **completely absent from the HTML template**.
- **The Defect (Critical Implementation Drift):**
  1. When a learner classifies a mistake, `finalizeAndShowFeedback()` unhides the solution, but immediately arms a `1500ms` timer that forces the card to advance to the next card.
  2. The learner is given only 1.5 seconds to read a multi-step mathematical derivation before the card vanishes!
  3. The explicit `#proc-next-btn` required by [`docs/STUDYLAB_UI_COMPOSITION_CONTRACT.md`](file:///c:/Users/Suraj/Documents/Antigravity/Anki-maths/docs/STUDYLAB_UI_COMPOSITION_CONTRACT.md#L349) is actively suppressed in code and omitted from the template.
- **Contract Mandate:** In the incorrect flow, after mistake classification, the UI must transition to `feedback` with the solution fully visible and remain in `feedback` until the learner explicitly clicks `Next Problem` (or presses `Space`/`Enter`). Auto-advance is strictly prohibited on incorrect/reflection cards.

### 3.4 Finding D: Solution Visibility & Scrolling Post-Classification
- **Current Code Reality (Verified):**
  1. When `finalizeAndShowFeedback()` runs, `mistakePanel` is given `classList.add("hidden")` and `display: none` ([`procedural.ts:1125`](file:///c:/Users/Suraj/Documents/Antigravity/Anki-maths/ts/reviewer/procedural.ts#L1125)).
  2. Because `.proc-interaction-footer` uses `:has(> :not(.hidden))`, when the mistake panel is hidden, the footer collapses.
  3. However, because `#proc-solution-container` has `overflow-x: auto` and no minimum vertical margin before the window bottom, and because the card auto-advances in 1.5s, the solution is never comfortably reachable.
  4. If `#proc-next-btn` is restored inside `.proc-interaction-footer`, the footer remains visible, re-triggering the fixed overlay occlusion of the solution body.
- **Contract Mandate:** After mistake classification:
  - The reflection panel must cleanly disappear.
  - The canonical solution must be fully visible and naturally scrollable in standard document flow.
  - The solution must not sit underneath a fixed overlay.
  - Main window scrolling must remain fluid without nested scroll container traps or unwanted horizontal overflow.

---

## 4. Canonical Target States

The frontend reviewer must cleanly render and transition across five canonical states:

```text
┌──────────────────────────────────────────────────────────────────────────────────┐
│                         CANONICAL STATE FLOW & TOPOLOGY                          │
├──────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│   [State A: Numerical Solving]  OR  [State B: MCQ Solving]                       │
│   • Problem Stem (Hero)             • Problem Stem (Hero)                        │
│   • Input + Unit Preview Pill       • 4 Option Radio Cards (A..D)                │
│   • Submit CTA + Hint Control       • Hint Control                               │
│                   │                                 │                            │
│                   └────────────────┬────────────────┘                            │
│                                    │ Submit Attempt                              │
│                                    ▼                                             │
│                       [Local & Host Validation]                                  │
│                                    │                                             │
│                 ┌──────────────────┴──────────────────┐                          │
│                 │ (Correct)                           │ (Incorrect)              │
│                 ▼                                     ▼                          │
│        [State D: Feedback]                   [State C: Wrong / Reflection]       │
│        • Correct Status                      • Incorrect Status                  │
│        • Deduplicated Comparison             • User Answer (Expected Withheld)   │
│        • Solution Derivation                 • 4 Reflection Buttons (1..4)       │
│        • Explicit Next Problem               • Space/Enter Anti-Bypass Trapped   │
│                 │                                     │                          │
│                 │                                     │ (Select 1..4)            │
│                 │                                     ▼                          │
│                 │                            [State D: Feedback / Solution]      │
│                 │                            • Category Confirmation             │
│                 │                            • Deduplicated Comparison           │
│                 │                            • Full LaTeX Derivation             │
│                 │                            • Explicit Next Problem             │
│                 │                                     │                          │
│                 └──────────────────┬──────────────────┘                          │
│                                    │ Explicit Advance (Click / Space / Enter)    │
│                                    ▼                                             │
│                         [State E: Next Handover]                                 │
│                         • Dispatches procedural_answer:<ease>                    │
│                         • Teardown & Anki Next Card Render                       │
│                                                                                  │
└──────────────────────────────────────────────────────────────────────────────────┘
```

### 4.1 State A: Numerical / Quick Solve
- **Purpose:** Active mathematical/scientific calculation and dimensional entry.
- **Visible Elements:**
  - Restrained header: topic breadcrumbs, difficulty badge (`Level X: ...`), authentic exam provenance tag (if applicable).
  - Problem prompt as visual hero (`#proc-prompt`).
  - Active numerical input (`#proc-answer-input`) with placeholder `Type final answer...`.
  - Live unit/dimensional preview pill (`.proc-num-preview-pill` / `#proc-num-preview`) updating on keystrokes.
  - Primary CTA: `Submit Answer` (`#proc-submit-btn`, class `proc-btn proc-btn-primary`).
  - Scaffolding Control: `💡 Request Hint` (`#proc-hint-btn`, class `proc-btn proc-btn-secondary`).
  - Mode Switcher: `.proc-mode-switch` (only when problem supports dual Quick/Stepwise solving).
- **Hidden / Suppressed Elements:**
  - Result panel (`#proc-result-panel`) strictly hidden.
  - Solution derivation (`#proc-solution-container`) strictly hidden.
  - Mistake classification panel (`#proc-mistake-panel`) strictly hidden.
  - Next Problem CTA (`#proc-next-btn`) strictly hidden.
  - Native Anki `#ansbut` and ease buttons strictly suppressed by host bridge.
  - Ticking stopwatch millisecond numbers strictly suppressed.

### 4.2 State B: MCQ Solving
- **Purpose:** Structured conceptual or quantitative multiple-choice evaluation.
- **Visible Elements:**
  - Problem prompt as visual hero (`#proc-prompt`).
  - Exactly 4 discrete option cards (`.proc-option-item`, `role="radio"`) inside `.proc-option-group`.
  - Each card contains hotkey badge (`[A]`, `[B]`, `[C]`, `[D]`) and formatted mathematical label.
  - Scaffolding Control: `💡 Request Hint` (`#proc-hint-btn`) where supported by content contract.
- **Hidden / Suppressed Elements:**
  - Free-text input field (`#proc-answer-input`) and quick container completely removed or suppressed via `enforceZeroTextInputFallback()`.
  - Result panel, solution container, mistake panel, and `#proc-next-btn` strictly hidden.
  - Native Anki `#ansbut` and ease buttons strictly suppressed.

### 4.3 State C: Wrong / Reflection
- **Purpose:** Metacognitive failure pause forcing immediate root-cause attribution.
- **Visible Elements:**
  - Problem prompt remains visible for reference.
  - Concise inline status indicator: `✗ Incorrect Answer` (`.proc-status-incorrect`).
  - User's submitted answer: `Your answer: <value>`.
  - Metacognitive reflection panel (`#proc-mistake-panel`) containing heading:
    *"Classify error (1-4) to reflect and optimize spaced repetition:"*
  - Exactly four category buttons (`.proc-mistake-btn` with `data-key="1..4"`):
    1. `[1 Silly Slip]` (`silly_mistake`: Calculation, arithmetic, sign, or unit error)
    2. `[2 Pattern Missed]` (`pattern_not_recognized`: Schema recognition or structural blindspot)
    3. `[3 Concept Gap]` (`formula_or_concept_misapplied`: Wrong formula, physical law, or theorem)
    4. `[4 Prereq Unknown]` (`concept_not_known`: Missing foundational prerequisite knowledge)
- **Hidden / Suppressed Elements:**
  - Canonical solution derivation (`#proc-solution-container`) **strictly hidden** until a category is chosen (`ANTI-08`).
  - Expected/correct answer value **strictly withheld** during reflection (`ANTI-02`).
  - Next Problem button (`#proc-next-btn`) strictly hidden.
  - Solving inputs (`#proc-quick-container`, `.proc-option-group`) hidden or disabled.
  - Native Anki `#ansbut` and ease buttons strictly suppressed.
- **Keyboard Lock:** `Space` and `Enter` keys are trapped (`preventDefault()`, `stopPropagation()`). Advancing without classifying is physically impossible.

### 4.4 State D: Feedback / Solution
- **Purpose:** Canonical derivation study, mental model correction, and outcome review.
- **Visible Elements:**
  - Reflection panel is **removed or hidden**.
  - Reflection footer does not overlay or trap the solution.
  - Concise outcome status:
    - Correct attempt: `✓ Correct Answer`
    - Incorrect attempt: `✗ Incorrect Answer (Categorized: <Category Name>)`
  - Deduplicated comparison row: `Your answer: X · Correct answer: Y` ([`ANTI-02`](file:///c:/Users/Suraj/Documents/Antigravity/Anki-maths/docs/STUDYLAB_UI_COMPOSITION_CONTRACT.md#L140)).
  - Streamlined speed quadrant pill: compact muted badge (e.g. `⚡ Fast & Accurate · 8.4s` or `💡 Concept Gap · 32.1s`) ([`ANTI-04`](file:///c:/Users/Suraj/Documents/Antigravity/Anki-maths/docs/STUDYLAB_UI_COMPOSITION_CONTRACT.md#L142)).
  - Canonical LaTeX derivation trace (`#proc-solution-container`) fully visible, typeset via MathJax, and naturally scrollable.
  - **Explicit Primary Action:** `#proc-next-btn` (`Next Problem ➔ (Space / Enter)`).
  - Optional Secondary Actions (only where supported by contract):
    - `Try Similar Problem (Alt+T)` (`#proc-try-similar-btn`)
    - `Practice Prerequisite (Alt+P)` (`#proc-practice-prereq-btn`)
    - `Review in Anki (Alt+R)` (`#proc-anki-recall-btn`)
- **Hidden / Suppressed Elements:**
  - All interactive solving inputs (textboxes, option cards).
  - Reflection panel and buttons.
  - Native Anki ease rating bar (in-card `#proc-next-btn` drives advance).
  - Automatic timer advancing the card (auto-advance strictly prohibited).

### 4.5 State E: Next Handover
- **Purpose:** Clean dispatch of calibrated rating to host Anki and teardown of webview listeners.
- **Behavior:**
  - Learner activates `#proc-next-btn` via click, `Space`, or `Enter`.
  - Dispatches `procedural_answer:<ease>` across bridge (where ease is $1$ Again, $2$ Hard, $3$ Good, $4$ Easy).
  - Evaluates `destroyActive()` to purge event listeners and timers.
  - Anki host mounts subsequent card in queue.

---

## 5. Control Ownership Matrix

This matrix establishes unambiguous control ownership, applicable states, priority, and keyboard interactions across the entire reviewer surface:

| # | Exact Control Label | DOM Selector | Owner | State Applicability | Visibility Rule | Priority | Allowed Keyboard | State Transition | Mutable vs Frozen |
|---|---|---|---|---|---|---|---|---|---|
| **1** | `💡 Request Hint` | `#proc-hint-btn` | StudyLab | `ready`, `solving`, `stepwise` | Visible in Solving; Hidden in Reflection/Feedback | Secondary / Scaffolding | `H`, `?` | Dispatches `procedural_hint`; discloses inline tier | **MUTABLE** (Restore to Quick & MCQ) |
| **2** | `Submit Answer` | `#proc-submit-btn` | StudyLab | `ready`, `solving` | Visible in Numerical Solving; Hidden in Reflection/Feedback | Primary CTA (Numerical) | `Enter` | `solving` $\to$ `submitting` | **MUTABLE** (Preserve current styling) |
| **3** | Option Cards (`A`..`D`) | `.proc-option-item` | StudyLab | `ready`, `solving` | Visible in MCQ Solving; Locked in Feedback | Primary Interactive (MCQ) | `1`..`4`, `A`..`D`, Arrows | Selects option; confirms on Space/Enter | **MUTABLE** (Preserve ARIA radiogroup) |
| **4** | `Check Solution` | `#proc-check-steps-btn` | StudyLab | `stepwise` | Visible in Stepwise Workspace | Primary CTA (Stepwise) | `Ctrl+Enter` | Evaluates multi-step CAS graph | **MUTABLE** (Preserve current binding) |
| **5** | `+ Add Step` | `#proc-add-step-btn` | StudyLab | `stepwise` | Visible in Stepwise Workspace | Secondary Utility | `Alt+A` | Appends step row | **MUTABLE** (Preserve current binding) |
| **6** | `Reset` | `#proc-reset-steps-btn` | StudyLab | `stepwise` | Visible in Stepwise Workspace | Ghost Utility | `Alt+R` | Clears step inputs | **MUTABLE** (Preserve current binding) |
| **7** | `1 Silly Slip` | `.proc-mistake-btn[data-key="1"]` | StudyLab | `mistake_classification` | Visible ONLY during Error Reflection | Primary Reflection Gate | `1` | Records slip; reveals solution; $\to$ `feedback` | **MUTABLE** (Preserve category mapping) |
| **8** | `2 Pattern Missed` | `.proc-mistake-btn[data-key="2"]` | StudyLab | `mistake_classification` | Visible ONLY during Error Reflection | Primary Reflection Gate | `2` | Records pattern; reveals solution; $\to$ `feedback` | **MUTABLE** (Preserve category mapping) |
| **9** | `3 Concept Gap` | `.proc-mistake-btn[data-key="3"]` | StudyLab | `mistake_classification` | Visible ONLY during Error Reflection | Primary Reflection Gate | `3` | Records concept; reveals solution; $\to$ `feedback` | **MUTABLE** (Preserve category mapping) |
| **10** | `4 Prereq Unknown` | `.proc-mistake-btn[data-key="4"]` | StudyLab | `mistake_classification` | Visible ONLY during Error Reflection | Primary Reflection Gate | `4` | Records prereq; reveals solution; $\to$ `feedback` | **MUTABLE** (Preserve category mapping) |
| **11** | Solution Derivation | `#proc-solution-container` | StudyLab | `feedback` | Strictly Hidden in Reflection; Visible in Feedback | Content Hero (Feedback) | None (Scrollable) | Unhidden post-reflection | **MUTABLE** (Ensure natural scroll clearance) |
| **12** | `Next Problem ➔` | `#proc-next-btn` | StudyLab | `feedback` | Strictly Hidden in Solving & Reflection; Visible in Feedback | **Primary CTA (Progression)** | `Space`, `Enter` | `feedback` $\to$ `next` | **MUTABLE** (Restore as explicit action) |
| **13** | `Try Similar Problem` | `#proc-try-similar-btn` | StudyLab | `feedback`, `worked_example` | Visible where variant generation supported | Secondary Remedial | `Alt+T` | Dispatches `procedural_try_similar` | **MUTABLE** (Preserve current binding) |
| **14** | `Practice Prerequisite` | `#proc-practice-prereq-btn`| StudyLab | `feedback` | Visible where prereq link exists | Secondary Remedial | `Alt+P` | Dispatches `procedural_practice_prerequisite` | **MUTABLE** (Preserve current binding) |
| **15** | `Review in Anki` | `#proc-anki-recall-btn` | StudyLab | `feedback`, `declarative_recall`| Visible on recall anchor cards | Secondary Remedial | `Alt+R` | Dispatches `procedural_declarative_recall` | **MUTABLE** (Preserve current binding) |
| **16** | `Show Answer` | `#ansbut` (Bottom bar) | Native Anki | Standard Cards ONLY | Suppressed on Procedural Cards | Native Action | `Space` | Shows standard flashcard back | **FROZEN** (Untouched on Basic/Cloze) |
| **17** | `Again` (Ease 1) | Ease Button 1 (Bottom bar) | Native Anki | Standard Cards ONLY | Suppressed on Procedural Cards | Native Rating | `1` | Rates standard card Again | **FROZEN** (Untouched on Basic/Cloze) |
| **18** | `Hard` (Ease 2) | Ease Button 2 (Bottom bar) | Native Anki | Standard Cards ONLY | Suppressed on Procedural Cards | Native Rating | `2` | Rates standard card Hard | **FROZEN** (Untouched on Basic/Cloze) |
| **19** | `Good` (Ease 3) | Ease Button 3 (Bottom bar) | Native Anki | Standard Cards ONLY | Suppressed on Procedural Cards | Native Rating | `3` | Rates standard card Good | **FROZEN** (Untouched on Basic/Cloze) |
| **20** | `Easy` (Ease 4) | Ease Button 4 (Bottom bar) | Native Anki | Standard Cards ONLY | Suppressed on Procedural Cards | Native Rating | `4` | Rates standard card Easy | **FROZEN** (Untouched on Basic/Cloze) |
| **21** | `Edit Current` | Native Menu / Tool | Native Anki | All Cards | Active in Anki Shell | Host Tool | `E` | Opens Anki Note Editor | **FROZEN** (Untouched) |
| **22** | `More` | Native Menu / Tool | Native Anki | All Cards | Active in Anki Shell | Host Tool | `M` | Opens Anki Context Menu | **FROZEN** (Untouched) |

---

## 6. State Transition Contract

The state machine for procedural reviews must strictly conform to the following deterministic flow:

```text
       ┌──────────┐
       │  ready   │ ◄── Card mounted, inputs armed, hotkeys bound
       └────┬─────┘
            │ Keystroke / Focus
            ▼
       ┌──────────┐
 ┌────►│ solving  │ ◄── Active timer in background; input enabled
 │     └────┬─────┘
 │          │
 │ (Hint: H/?)
 └──────────┤ (Discloses inline hint without leaving solving loop)
            │
            │ Submit Attempt (Enter / Click / Ctrl+Enter)
            ▼
       ┌──────────┐
       │submitting│ ◄── Inputs locked; local AST & dimensional validation (<50ms)
       └────┬─────┘
            │
     ┌──────┴──────────────────────────────┐
     │ Correct                             │ Incorrect
     ▼                                     ▼
┌──────────┐                         ┌─────────────┐
│ feedback │                         │ wrong_answer│ ◄── Inline status: ✗ Incorrect
└────┬─────┘                         └──────┬──────┘
     │                                      │ Immediate transition
     │                                      ▼
     │                               ┌───────────────────────────┐
     │                               │   mistake_classification  │ ◄── Space/Enter TRAPPED
     │                               │ (Mandatory Reflection 1-4)│     Solution STRICTLY HIDDEN
     │                               └──────────────┬────────────┘
     │                                              │
     │                                              │ Category selected (1..4)
     │                                              ▼
     │                                       ┌──────────┐
     │                                       │ feedback │ ◄── Solution UNHIDDEN
     │                                       └────┬─────┘     Auto-advance DISABLED
     │                                            │
     └──────────────────────┬─────────────────────┘
                            │
                            │ Explicit Action: Click #proc-next-btn OR Space / Enter
                            ▼
                     ┌──────────────┐
                     │     next     │ ◄── Dispatches procedural_answer:<ease>
                     └──────┬───────┘
                            │
                            ▼
                     ┌──────────────┐
                     │   teardown   │ ◄── destroyActive() cleans listeners & timers
                     └──────────────┘
```

### 6.1 State Transition Invariants
1. **Mandatory Reflection Gate:** In the incorrect branch, transitioning from `wrong_answer` to `feedback` **requires** selecting one of the four mistake categories (`1`..`4`). Advancing without classifying is physically and logically blocked.
2. **Deferred Solution Reveal (`ANTI-08`):** The canonical solution derivation (`#proc-solution-container`) and expected answer values must remain strictly hidden throughout `mistake_classification`. The solution is unhidden **only** upon valid category selection.
3. **Category Selection Is Not an Ease Rating:** Selecting a mistake category records cognitive error attribution in telemetry; it **must not** directly trigger card advancement or equate directly to an Anki ease button.
4. **Explicit Next Problem Action:** In `feedback` state, card progression is driven strictly by user action on `#proc-next-btn` (or hotkey `Space`/`Enter`). Automatic advance via timer (`setTimeout`) is strictly prohibited.
5. **No Coexisting Rating Bars:** Standard Anki ease buttons (`Again`, `Hard`, `Good`, `Easy`) and `#ansbut` must remain suppressed on procedural cards; `#proc-next-btn` is the sole card progression control.

---

## 7. Visual Rules & Design Language Preservation

The existing StudyLab visual language defined in [`docs/STUDYLAB_UI_COMPOSITION_CONTRACT.md`](file:///c:/Users/Suraj/Documents/Antigravity/Anki-maths/docs/STUDYLAB_UI_COMPOSITION_CONTRACT.md) and [`ts/reviewer/reviewer.scss`](file:///c:/Users/Suraj/Documents/Antigravity/Anki-maths/ts/reviewer/reviewer.scss) must be preserved with 100% fidelity:

### 7.1 Visual Hierarchy Rules
1. **Problem Statement as Hero:** The problem stem (`#proc-prompt`) is the optical center of gravity. Font size $18\text{px}$, font weight 500, line height 1.6, rendered with clean MathJax typography.
2. **Open Canvas Composition:** Constrained to `max-width: 720px; margin: 0 auto;`. Unconfined, flat layout using subtle 1px dividers (`--proc-divider`) rather than heavy bounding rectangles or drop-shadow boxes.
3. **Restrained Chrome:** Header breadcrumbs use muted 12px text. No generic `VARIANT: PRACTICE` tags; display only authentic competitive exam provenance tags (e.g. `[ JEE Main 2024 · Shift 1 ]`).
4. **No Nested Cards (`ANTI-07`):** Box-in-a-box nesting (container $\to$ card $\to$ inner card $\to$ solution box) is prohibited. Sections flow sequentially down the canvas.
5. **Calm Feedback Borders (`ANTI-01`):** No full-bleed saturated red or green background boxes. Feedback uses open canvas with a subtle 3px left accent border (`--proc-accent-left-correct` or `--proc-accent-left-incorrect`).
6. **Deduplicated Answers (`ANTI-02`):** Consolidated into a single comparison row: `Your answer: X · Correct answer: Y`. No repeated "Expected Answer: Y" labels.
7. **No Visible Stopwatch (`ANTI-03`):** No live ticking stopwatch millisecond displays during solving. Time is tracked silently in memory and displayed calmly in feedback.
8. **Compact Speed Pill (`ANTI-04`):** Compact, muted status badge (e.g. `⚡ Fast & Accurate · 8.4s` or `💡 Concept Gap · 32.1s`) without loud competing banners.
9. **No Internal ID Leakage (`ANTI-06`):** Schema IDs (`schema.math.linear.v1`) and family tags must never appear in learner-facing text.

### 7.2 Design Tokens
Do **NOT** invent new color palettes or font systems. All visual rules must use canonical CSS variables:
- Surfaces: `--proc-bg`, `--proc-surface`, `--proc-surface-subtle`, `--proc-surface-hover`.
- Text: `--proc-text-primary`, `--proc-text-secondary`, `--proc-text-muted`.
- Borders: `--proc-border`, `--proc-border-subtle`, `--proc-border-focus`, `--proc-divider`.
- Accents: `--proc-primary`, `--proc-success`, `--proc-error`, `--proc-warning`.

---

## 8. Layout & Viewport Contract

### 8.1 Acceptance Criteria for Layout & Document Flow
1. **Standard Desktop Viewport ($1024 \times 768$ and above):**
   - The 720px open canvas is centered horizontally.
   - Solving surface and reflection controls are immediately visible without vertical crowding.
2. **Minimum Supported Desktop Viewport ($682 \times 607$):**
   - Problem stem, options or input box, and action controls fit without clipping or button collisions.
   - Hotkey badges (`[1]`, `[A]`) align cleanly without text truncation.
3. **Long Solution Derivations:**
   - Solutions containing multi-step derivations or large display formulas must be fully reachable via standard window/page vertical scrolling.
   - No content may be occluded by fixed bars, footers, or overlays.
4. **Fixed Overlay Elimination & Document Flow Ownership:**
   - The mistake classification strip must either integrate directly into the natural document flow or guarantee non-occluding vertical clearance through dynamic flex/sticky layout.
   - Hardcoded, fixed bottom overlays that float above active content without guaranteed clearance are prohibited.
5. **Scrolling Discipline:**
   - Single document-level vertical scroll (`mw.web` / `#qa`).
   - **Zero nested scroll containers:** Containers must not trap mousewheel scroll events or produce unsightly double scrollbars.
   - **Zero horizontal page overflow:** The container must enforce `overflow-x: hidden`. Individual wide mathematical equations must handle internal overflow gracefully (`overflow-x: auto`) without expanding the page width.
6. **Keyboard Focus Clearance:**
   - Active keyboard focus (`:focus-visible`) must never be positioned underneath a sticky or fixed element. The focused element must be fully within the visible viewport.

---

## 9. Accessibility & Keyboard Contract

### 9.1 Keyboard Interaction Rules
1. **Focus Visibility:** Every interactive element must render a crisp, high-contrast focus indicator using `outline: 2px solid var(--proc-border-focus); outline-offset: 2px;` via `:focus-visible`.
2. **Full Mouse-Free Operability:** Every operational state must be 100% navigable and executable via keyboard alone.
3. **Modality Solving Shortcuts:**
   - **Numerical:** Focus resides in `#proc-answer-input`. `Enter` submits the answer.
   - **MCQ:** Keys `1`..`4` and `A`..`D` select candidate option cards immediately. Arrow keys navigate options via roving `tabindex`. `Space` or `Enter` on focused card confirms selection.
   - **Stepwise:** `Enter` advances to next step row; `Ctrl+Enter` triggers `Check Solution`; `Alt+A` adds step; `Alt+R` resets.
   - **Hint:** Keys `H` or `?` trigger progressive hint disclosure in solving states.
4. **Reflection Gate Anti-Bypass Lock:**
   - During `mistake_classification`, `Space`, `Enter`, and `Tab` are intercepted and blocked (`preventDefault()`, `stopPropagation()`).
   - Keys `1`, `2`, `3`, and `4` select the respective mistake categories.
5. **Feedback & Advancement:**
   - In `feedback` state, pressing `Space` or `Enter` activates `#proc-next-btn` and advances the card.
   - Keys `1`, `2`, `3`, `4` optionally override the calibrated FSRS rating before advance.
   - `Alt+T` triggers `Try Similar Problem` (if enabled).
6. **No Dead Ends:** The keyboard user must never encounter an unfocusable control, trapped focus loop, or invisible active element.

---

## 10. Responsive & Visual Verification Matrix

Every change must be verified across 11 mandatory target states across both host desktop composition and embedded webview composition:

```text
┌──────────────────────────────────────────────────────────────────────────────────┐
│                    11-STATE RESPONSIVE VERIFICATION MATRIX                       │
├────┬────────────────────────────┬────────────────────────────────────────────────┤
│ #  │ Target State               │ Primary Verification Assertions                │
├────┼────────────────────────────┼────────────────────────────────────────────────┤
│ 1  │ Numerical Solving          │ Problem stem hero, live unit pill, Hint visible│
│ 2  │ Numerical Wrong            │ Status ✗ Incorrect, user answer, expected hid  │
│ 3  │ Mistake Classification     │ 4 buttons visible, Space/Enter blocked         │
│ 4  │ Numerical Feedback/Solution│ Solution unhidden, explicit Next Problem CTA   │
│ 5  │ MCQ Solving                │ 4 radio cards, zero textboxes, Hint visible    │
│ 6  │ MCQ Wrong                  │ Status ✗ Incorrect, user option, expected hid  │
│ 7  │ MCQ Classification         │ 4 buttons visible, Space/Enter blocked         │
│ 8  │ MCQ Feedback/Solution      │ Solution unhidden, explicit Next Problem CTA   │
│ 9  │ Standard Basic Card        │ 100% untouched native Anki reviewer & #ansbut  │
│ 10 │ Standard Cloze Card        │ 100% untouched native Anki cloze rendering     │
│ 11 │ Native Anki Shell          │ Native top toolbar, menus, and window chrome   │
└────┴────────────────────────────┴────────────────────────────────────────────────┘
```

### 10.1 Dual Inspection Rule
For all StudyLab states:
1. **Complete Desktop Composition:** Must be verified on the running Win32 application window, evaluating title bar, top toolbar, main webview canvas, bottom toolbar suppression, and OS window borders.
2. **Embedded WebView Composition:** Must be inspected within `mw.web` DOM to verify element rects, computed styles, and accessibility attributes.
> **Mandatory Rule:** A headless Chromium CDP or DOM screenshot alone **MUST NOT** be accepted as proof of final desktop UI quality. Verification must confirm physical window reality using `studylab-desktop-ui-review` and `desktop-webview-reviewer`.

---

## 11. Regression Gates

The following regression gates are strict, binary go/no-go acceptance gates:

### 11.1 StudyLab Subsystem Gates
- [ ] **Hint Accessibility:** `#proc-hint-btn` is rendered, properly positioned, and usable in Quick Solve and MCQ modalities.
- [ ] **Reflection Gate Integrity:** Reflection panel mounts on error; Space and Enter are trapped; classification is mandatory.
- [ ] **Deferred Reveal:** `#proc-solution-container` and expected answers remain strictly hidden during reflection.
- [ ] **Explicit Progression:** `#proc-next-btn` is rendered in feedback; auto-advance timer on incorrect flow is completely eliminated.
- [ ] **Scroll & Viewport Reachability:** Long solution content is naturally scrollable; no solution content is obscured by fixed footers.
- [ ] **Deduplication:** Zero duplicate Next buttons; zero duplicate answer labels; zero leaked internal schema IDs.
- [ ] **Ease Bar Suppression:** Native Anki bottom ease buttons and `#ansbut` are completely suppressed on all procedural cards.

### 11.2 Host Anki Zero-Regression Gates
- [ ] **Standard Basic Cards:** 100% untouched rendering, native `#ansbut`, and native ease rating bar.
- [ ] **Standard Cloze Cards:** 100% untouched cloze reveal mechanics and ease rating.
- [ ] **Application Views:** `Decks`, `Add`, `Browse`, `Stats`, and `Sync` operate with zero behavioral or visual deviation.
- [ ] **Host Menus:** Native menus and shortcut tables function unmodified.
- [ ] **Window Lifecycle:** `destroyActive()` disposes all listeners on card advance with zero event or memory leaks.

> **Failure Rule:** Any behavioral or visual regression detected in an Anki-owned surface immediately invalidates the change.

---

## 12. Implementation Boundaries

This is a **frontend-first correction pass**. The mutable implementation surface is strictly bounded:

### 12.1 Primary Mutable Files
1. [`ts/reviewer/procedural.ts`](file:///c:/Users/Suraj/Documents/Antigravity/Anki-maths/ts/reviewer/procedural.ts): State machine transitions, Hint binding, explicit Next Problem handling, auto-advance elimination, keyboard routing.
2. [`ts/reviewer/reviewer.scss`](file:///c:/Users/Suraj/Documents/Antigravity/Anki-maths/ts/reviewer/reviewer.scss): Layout flow, reflection footer clearance, non-occluding layout, solution scrolling, focus visible styles.
3. [`rslib/procedural/src/reviewer/template.rs`](file:///c:/Users/Suraj/Documents/Antigravity/Anki-maths/rslib/procedural/src/reviewer/template.rs): Reviewer HTML template generation, mounting `#proc-hint-btn` in Quick and MCQ solving surfaces, mounting `#proc-next-btn` in feedback DOM.
4. Relevant test suites: [`ts/reviewer/procedural.test.ts`](file:///c:/Users/Suraj/Documents/Antigravity/Anki-maths/ts/reviewer/procedural.test.ts) and unit tests in `template.rs`.

### 12.2 Explicitly Out-of-Scope Files & Subsystems
- **`qt/aqt/reviewer.py`:** Strictly out of scope for this pass unless an unresolvable host bridge defect is forensically proven to block frontend execution.
- **Backend Rust Engines:** `rslib/procedural/src/problems/`, `physics/`, `chemistry/`, `reasoning/`, `steps/` are frozen and out of scope.
- **Scheduling & Persistence:** FSRS interval math (`rating_policy.rs`), SQLite schemas (`storage/`), and migrations (`migration.rs`) are frozen and out of scope.
- **APKG Importers & Content Factory:** `generate_canonical_source_apkg.py`, `tools/studylab_content_factory.py` are out of scope.

---

## 13. Non-Goals

The following activities are strictly prohibited during this pass:
- ❌ No redesign of StudyLab's core visual identity.
- ❌ No redesign or cosmetic modification of native Anki surfaces.
- ❌ No replacement of existing design tokens or CSS custom properties.
- ❌ No component-framework migrations (e.g. rewriting in React, Vue, or Svelte).
- ❌ No wholesale SCSS stylesheet rewrites.
- ❌ No modifications to backend FSRS or SM-2 scheduling algorithms.
- ❌ No changes to `procedural.db` database schemas or migrations.
- ❌ No introduction of new learning object modalities.
- ❌ No addition of analytics, telemetry dashboards, or student profile UI.
- ❌ No speculative improvements or exploratory refactoring.

---

## 14. Definition of Done

An implementation pass under this contract is considered **Done** if and only if all ten criteria are verified:

1. [ ] **Composition:** `#proc-hint-btn` is rendered and operable in Quick Solve and MCQ; `#proc-next-btn` is rendered in feedback state; `#proc-quick-container` is completely absent from MCQ cards.
2. [ ] **State Transitions:** Error flow executes strictly: `wrong_answer` $\to$ `mistake_classification` $\to$ category selection $\to$ `feedback` (solution unhidden) $\to$ explicit `Next Problem`. Auto-advance is eliminated on error flow.
3. [ ] **Controls:** Exactly 4 mistake classification buttons are visible during reflection; Next Problem is visible only in feedback; all native ease buttons and `#ansbut` remain suppressed on procedural cards.
4. [ ] **Scroll & Viewport:** Long solutions scroll naturally; the reflection footer does not occlude problem or solution text; zero nested scrollbars; zero page-level horizontal overflow.
5. [ ] **Keyboard & A11y:** Full mouse-free workflow verified; Space/Enter trapped in reflection; `1`..`4` hotkeys select mistake categories; `:focus-visible` styling is prominent.
6. [ ] **Anki Zero-Regression:** Standard Basic and Cloze cards display native `#ansbut` and ease buttons without procedural interference.
7. [ ] **Desktop Visual Verification:** Full Win32 running application window verified via `studylab-desktop-ui-review` across light and night modes.
8. [ ] **WebView Verification:** Element bounding rects, computed styles, and accessibility attributes verified inside `mw.web`.
9. [ ] **Automated Tests:** All Rust unit tests (`cargo test -p procedural --lib`), Vitest reviewer tests (`npm run vitest:once`), and Python GUI tests (`just test-py`) pass with 100% success.
10. [ ] **Cryptographic Evidence:** Baseline and final visual evidence bundles collected and sealed.

---

## 15. Evidence Requirements

Any agent or engineer completing this work must deliver an evidence package containing:
1. **Baseline Screenshots:** Captures of current defects before modification (missing hint button, occluding footer, auto-advancing feedback).
2. **Final Desktop Screenshots:** Full-window Win32 captures across all operational states:
   - Numerical solving with hint button visible.
   - Numerical error reflection with 4 categories and solution hidden.
   - Numerical feedback with full solution and explicit Next Problem button.
   - MCQ solving with zero text input fallback.
   - MCQ error reflection and feedback.
   - Standard Basic card showing native `#ansbut` and ease buttons.
   - Standard Cloze card showing native cloze review.
3. **Dual Verification Proof:** Pair of Win32 desktop window screenshot and QtWebEngine CDP viewport screenshot for each state.
4. **Test Suite Output Logs:** Verifying 100% passing status across Rust, TypeScript, and Python test suites.
5. **Exact File Diff Manifest:** Explicit list of files modified, confirming zero edits to frozen Anki or backend files.

---

## 16. Implementation Agent Safety Rules

Every autonomous agent assigned to execute this contract must strictly adhere to the following rules:

1. **Read Before Editing:** Read this contract, [`docs/STUDYLAB_UI_COMPOSITION_CONTRACT.md`](file:///c:/Users/Suraj/Documents/Antigravity/Anki-maths/docs/STUDYLAB_UI_COMPOSITION_CONTRACT.md), and current source files completely before formulating code edits.
2. **Inspect Code Reality First:** Never assume documentation matches current code. Inspect the target files (`template.rs`, `procedural.ts`, `reviewer.scss`) directly.
3. **Strict Mutable Surface Adherence:** Edit only files declared inside the mutable implementation boundary.
4. **Never Hack Host to Fix Guest:** Do not modify Anki host code (`qt/aqt/reviewer.py`) to bypass or cover up frontend CSS/DOM defects.
5. **Structural Fixes First:** Prioritize ownership, DOM mounting, state visibility, and natural document flow over arbitrary CSS padding patches.
6. **No Speculative Behaviors:** Implement only behaviors explicitly defined in canonical contracts. Do not invent new buttons, modals, or options.
7. **Stop and Report on Conflict:** If a requirement in this contract materially conflicts with an underlying architectural invariant, stop and report the contradiction immediately.
8. **No Virtual Victory:** Never declare completion based on passing unit tests or headless CDP snapshots alone. Inspect the real running desktop application window.
9. **Run Full Regression Gates:** Execute the complete verification commands before declaring done.
10. **Minimal Localized Patches:** Keep code diffs concise, clean, documented, and strictly scoped to the defect under repair.

---
*Canonical Master Contract authored and certified for StudyLab Procedural Reviewer Frontend Alignment.*
