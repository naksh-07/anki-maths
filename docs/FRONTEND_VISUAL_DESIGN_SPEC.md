# StudyLab Frontend Visual Design Specification

**Document Version:** 1.1.0 (Reconciled with STUDYLAB_UI_COMPOSITION_CONTRACT.md)  
**Target Repository:** `Anki-maths` (StudyLab Subsystem)  
**Status:** AUTHORITATIVE SPECIFICATION  
**Integrity Mode:** Benchmark Mode (100% Grounded in Executable Styles, QtWebEngine Renders, and UI Audits)  
**Authoritative Reference:** `docs/STUDYLAB_UI_COMPOSITION_CONTRACT.md`, `PROJECT.md`

---

## 1. Visual Product Contract & Core Philosophy

### 1.1 The Problem Is the Visual Hero
In StudyLab, the mathematical, physical, or logical problem statement is the **primary visual hero** of the interface. The user interface is completely subordinate to the learner's cognitive problem-solving task.

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                           CORE VISUAL DESIGN DIRECTIVE                           │
├──────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│   "Calm, native, minimal, focused, professional, and dense enough for            │
│    mathematical reasoning without visual noise or web-widget chrome."            │
│                                                                                  │
│   • Blends seamlessly into Anki's native desktop light and dark themes.          │
│   • Replaces web-app widget clutter with clean, restrained Open Canvas layout.   │
│   • Elevates mathematical equations (LaTeX/MathJax) to primary contrast.         │
│   • Maintains a single focused interaction surface per state.                    │
│   • Eliminates giant colored containers, nested cards, and telemetry dumps.      │
│                                                                                  │
└──────────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Prohibited Visual Anti-Patterns Ledger

To ensure StudyLab never degenerates into a noisy web application, the frontend strictly enforces the elimination of all 8 visual anti-patterns (`ANTI-01` through `ANTI-08`):

```
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                VISUAL ANTI-PATTERN LEDGER                                        │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
```

| ID | Anti-Pattern | Design Violation Description | Mandated Open Canvas Correction |
|---|---|---|---|
| **ANTI-01** | **Giant Feedback Containers** | Full-bleed saturated red/green colored background wrappers covering the card surface | Replaced by open canvas typography with subtle inline status indicator (`✓ Correct` / `✗ Incorrect`) and a 3px left accent border |
| **ANTI-02** | **Duplicate Expected Answer Labels** | Displaying "Expected Answer" and "You answered" multiple times across template and script | Consolidated single comparison row (`Your answer: X · Correct answer: Y`) rendered exactly once |
| **ANTI-03** | **Ticking Stopwatch During Solving** | Active numeric clock updating every 200ms during problem solving | Stopwatch runs silently in telemetry background; total elapsed time displayed calmly post-submission alongside speed pill |
| **ANTI-04** | **Competing Speed Badges** | Multi-badge speed quadrant clutter competing with results (e.g. `⚡ Fluency Strength (Accurate & Fast)`) | Single compact muted status pill (`⚡ Fast & Accurate · 8.4s`) using subdued surface tokens |
| **ANTI-05** | **Generic Practice / Variant Chrome** | "VARIANT: PRACTICE" badges and generic practice headers | Suppressed; preserve only verified competitive exam provenance (`[ JEE Main 2024 ]`) |
| **ANTI-06** | **Raw Internal Schema & Generator IDs** | Internal schema strings (`schema.phys.kinematics...`, `family_id`) | Strictly forbidden from learner view; retained 100% in HTML data attributes |
| **ANTI-07** | **Nested Card Boxes in Worked Examples & Solutions** | Box-in-a-box nested containers with heavy borders and background insets | Flat open canvas layout with 1px subtle horizontal dividers and 3px left accent rules |
| **ANTI-08** | **Premature Solution Reveal in Reflection Gate** | Displaying full solution derivation while learner is in mistake classification | `#proc-solution-container` strictly hidden in DOM until mistake category (1–4) is selected |

---

## 3. Design Tokens & Theme Integration

StudyLab utilizes CSS custom properties that inherit dynamically from Anki's native desktop theme (`body.nightMode` vs. default light mode).

### 3.1 Color Palette & Surface Tokens

```scss
/* Canonical Design Tokens (ts/reviewer/reviewer.scss) */
:root {
    /* Base Backgrounds & Surfaces */
    --proc-bg: #ffffff;
    --proc-surface: #f8fafc;
    --proc-surface-subtle: #f1f5f9;
    --proc-surface-hover: #e2e8f0;
    
    /* Typography & Foreground */
    --proc-text-primary: #0f172a;
    --proc-text-secondary: #475569;
    --proc-text-muted: #94a3b8;
    
    /* Borders & Dividers */
    --proc-border: #e2e8f0;
    --proc-border-focus: #3b82f6;
    --proc-border-subtle: #cbd5e1;
    --proc-divider: 1px solid var(--proc-border);
    
    /* Semantic State Accents */
    --proc-primary: #2563eb;
    --proc-primary-hover: #1d4ed8;
    --proc-primary-light: #eff6ff;
    
    --proc-success: #059669;
    --proc-success-bg: #ecfdf5;
    --proc-success-border: #10b981;
    
    --proc-error: #dc2626;
    --proc-error-bg: #fef2f2;
    --proc-error-border: #ef4444;
    
    --proc-warning: #d97706;
    --proc-warning-bg: #fffbeb;
    --proc-warning-border: #f59e0b;

    /* Open Canvas Callout & Accent Tokens */
    --proc-accent-left-correct: 3px solid var(--proc-success);
    --proc-accent-left-incorrect: 3px solid var(--proc-error);
    --proc-accent-left-worked: 3px solid var(--proc-primary);
    --proc-accent-left-decision: 3px solid #6366f1;
    --proc-pill-bg: var(--proc-surface-subtle);
    --proc-pill-text: var(--proc-text-secondary);
}

/* Anki Dark Mode (Night Mode) */
body.nightMode {
    --proc-bg: #1e1e2e;
    --proc-surface: #252538;
    --proc-surface-subtle: #2d2d44;
    --proc-surface-hover: #363652;
    
    --proc-text-primary: #f8fafc;
    --proc-text-secondary: #cbd5e1;
    --proc-text-muted: #64748b;
    
    --proc-border: #334155;
    --proc-border-focus: #60a5fa;
    --proc-border-subtle: #475569;
    --proc-divider: 1px solid var(--proc-border);
    
    --proc-primary: #3b82f6;
    --proc-primary-hover: #60a5fa;
    --proc-primary-light: #1e293b;
    
    --proc-success: #10b981;
    --proc-success-bg: #064e3b;
    --proc-success-border: #059669;
    
    --proc-error: #f87171;
    --proc-error-bg: #450a0a;
    --proc-error-border: #dc2626;
    
    --proc-warning: #fbbf24;
    --proc-warning-bg: #451a03;
    --proc-warning-border: #d97706;

    /* Open Canvas Callout & Accent Tokens (Night Mode) */
    --proc-accent-left-correct: 3px solid var(--proc-success);
    --proc-accent-left-incorrect: 3px solid var(--proc-error);
    --proc-accent-left-worked: 3px solid var(--proc-primary);
    --proc-accent-left-decision: 3px solid #818cf8;
    --proc-pill-bg: var(--proc-surface-subtle);
    --proc-pill-text: var(--proc-text-secondary);
}
```

---

## 4. Typography & Mathematical Typesetting

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                             TYPOGRAPHY HIERARCHY                                 │
├────────────────────────────┬──────────────────┬──────────┬───────────────────────┤
│ Element                    │ Font Stack       │ Size     │ Weight & Line Height  │
├────────────────────────────┼──────────────────┼──────────┼───────────────────────┤
│ **Problem Stem**           │ System UI / Sans │ 18px     │ Medium (500), 1.6     │
│ **Mathematical Formulas**  │ MathJax (TeX)    │ 18–20px  │ Scaled TeX Math Fonts │
│ **MCQ Option Text**        │ System UI / Sans │ 15px     │ Regular (400), 1.5    │
│ **Keyboard Badges**        │ SF Mono / Consolas│ 11px     │ Bold (700), 1.0       │
│ **Metadata Breadcrumbs**   │ System UI / Sans │ 12px     │ Regular (400), 1.4    │
│ **Hint / Explanation Text**│ System UI / Sans │ 14px     │ Regular (400), 1.5    │
│ **Input Values & Units**   │ SF Mono / System │ 16px     │ Semi-Bold (600), 1.4  │
└────────────────────────────┴──────────────────┴──────────┴───────────────────────┘
```

### Typography Invariants:
1. **Formula Legibility:** MathJax equations must render with generous line spacing ($1.6\times$) to prevent fraction bars and superscripts from overlapping adjacent text lines.
2. **Clear Font Pairing:** LaTeX symbols ($\Delta G^\circ$, $\sqrt{2gh}$, $\int_0^1$) blend naturally with surrounding system sans-serif text without sudden weight shifts.

---

## 5. Spacing System & Layout Grid

StudyLab uses a strict **4px/8px incremental grid system**:

- `spacing-xs`: `4px` (Badge padding, keyboard key margins)
- `spacing-sm`: `8px` (Button internal padding, option item gap)
- `spacing-md`: `12px` (Container padding, input field padding)
- `spacing-lg`: `16px` (Component separation, prompt bottom margin)
- `spacing-xl`: `24px` (Major section divider, solution box margin)
- `spacing-2xl`: `32px` (Top/bottom reviewer wrapper margin)

### Container Max-Width Constraint
To optimize reading velocity and reduce eye-travel fatigue during complex multi-line math derivations, the reviewer container `#proc-root` enforces:

```scss
#proc-root {
    max-width: 720px;
    margin: 0 auto;
    padding: 16px 24px;
    box-sizing: border-box;
}
```

---

## 6. Component Visual Specifications

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                         REVIEWER COMPONENT WIREFRAME                             │
├──────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│   Physics › Kinematics › 1D Free Fall                     [ JEE Main 2024 ]      │
│   ────────────────────────────────────────────────────────────────────────────   │
│                                                                                  │
│   A stone is dropped from a height of $45\,\text{m}$. Taking $g = 10\,\text{m/s}^2$,│
│   calculate the speed of the stone just before striking the ground.             │
│                                                                                  │
│   ┌────────────────────────────────────────────────────────────────────────┐     │
│   │  30 m/s                                                                │     │
│   └────────────────────────────────────────────────────────────────────────┘     │
│   [ Parsed: 30 m/s (Dimension: [Length]¹ [Time]⁻¹) ]                             │
│                                                                                  │
│   [ 💡 Request Hint ]                                      [ Submit Answer ]     │
│                                                                                  │
└──────────────────────────────────────────────────────────────────────────────────┘
```

### 6.1 Header & Breadcrumbs
- **Position:** Top of the card container.
- **Styling:** Subdued text color (`--proc-text-muted`), font size `12px`.
- **Format:** `Subject › Topic › Skill` on the left; optional authentic competitive exam badge on the right (e.g. `[ JEE Main 2024 ]`).
- **Forbidden (`ANTI-05`, `ANTI-06`):** Generic practice tags (`Variant: practice`), raw schema IDs (`schema.phys.kinematics...`), and rainbow badge spam are **strictly prohibited**.

### 6.2 Problem Area (Visual Hero)
- **Styling:** High contrast (`--proc-text-primary`), font size `18px`, line height `1.6`.
- **Background:** Transparent (inherits base canvas).
- **Padding:** `16px 0 20px 0`.

### 6.3 Input Surfaces

#### A. Quick / Numerical Input Field
- **Dimensions:** Full width, height `44px`, font size `16px`.
- **Border:** `1px solid var(--proc-border)`.
- **Focus Ring:** `2px solid var(--proc-border-focus)` with `outline: none`.
- **Live Preview Pill:** Rendered directly below the input (`.proc-num-preview-pill`), font size `12px`, background `--proc-surface-subtle`, text `--proc-text-secondary`.

#### B. MCQ Option Cards
- **Dimensions:** Full width, min-height `48px`, margin-bottom `8px`.
- **Border:** `1px solid var(--proc-border)`.
- **Border Radius:** `6px`.
- **States:**
  - *Default:* Background `--proc-surface`.
  - *Hover:* Background `--proc-surface-hover`, border `--proc-border-subtle`.
  - *Focused / Selected:* Border `2px solid var(--proc-primary)`, background `--proc-primary-light`.
  - *Correct Outcome:* Border `2px solid var(--proc-success-border)`, background `--proc-success-bg`.
  - *Incorrect Outcome:* Border `2px solid var(--proc-error-border)`, background `--proc-error-bg`.

#### C. Concept Check Diagnostic Cards
- **Layout:** 3–4 statement cards focusing on qualitative mental models.
- **Distractor Callout:** Selecting a distractor reveals an inline diagnostic callout with a 3px left accent border (`--proc-accent-left-incorrect`) explaining the exact misconception.

#### D. Strategy Drill Comparison Cards
- **Layout:** 2–4 strategy candidate cards with compact step count and speed ratings.
- **Optimality Callout:** Highlights optimal strategy with 3px left accent rule.

#### E. Stepwise Derivation Workspace
- **Row Container:** `1px solid var(--proc-border)`, border-radius `6px`, padding `10px 14px`, margin-bottom `10px`.
- **Step Label:** Font size `12px`, bold, `--proc-text-secondary`.
- **Validation Badge:** Compact right-aligned pill (`✔ Valid` green / `❌ Invalid` red / `⚠️ Consistent` yellow).

#### F. Worked Example Open Canvas Trace
- **Card Styling (`ANTI-07`):** Open canvas layout with subtle horizontal dividers.
- **Key Decision Point:** 3px solid left accent border (`--proc-accent-left-decision`) with transparent/subtle background.
- **Common Pitfalls:** 3px solid left accent border (`--proc-warning-border`).
- **Action Gate:** Single prominent primary button: `[ ✔ I Have Reviewed and Understood This Solution — Try Similar Problem ]`. Zero textboxes or radio inputs.

---

### 6.4 Mistake Classification Reflection Strip
- **Position:** Appears directly below the evaluated input upon incorrect answer.
- **Layout:** Compact 4-button horizontal flex strip (`gap: 8px`).
- **Buttons (`data-key="1..4"`):**
  - `[1 Silly Slip]` (`silly_mistake`)
  - `[2 Pattern Missed]` (`pattern_not_recognized`)
  - `[3 Concept Gap]` (`formula_or_concept_misapplied`)
  - `[4 Prereq Unknown]` (`concept_not_known`)
- **Anti-Bypass Visual Lock (`ANTI-08`):** `#proc-next-btn` is strictly hidden. `#proc-solution-container` remains strictly hidden until 1–4 classification is chosen. Space/Enter keys are trapped.

---

### 6.5 Open Canvas Solution & Feedback Area
- **Position:** Displayed below the interaction surface in `feedback` state.
- **Open Canvas Layout (`ANTI-01`, `ANTI-07`):** No giant red/green wrapper boxes. Uses subtle inline status (`✓ Correct` or `✗ Incorrect (Categorized: Concept Gap)`).
- **Deduplicated Answer Row (`ANTI-02`):** `Your answer: 30 m/s · Correct answer: 30 m/s` rendered exactly once.
- **Speed Pill (`ANTI-04`):** Compact muted status pill (e.g. `⚡ Fast & Accurate · 8.4s`).
- **Derivation Trace:** Clean LaTeX step progression with 1px dividers.
- **Next Action Button:** High-contrast primary CTA: `Next Problem ➔ (Space / Enter)` (`#proc-next-btn`). Native Anki ease buttons suppressed.

---

## 7. Visual Design Acceptance Checklist

Before any frontend template or style modification is certified:
- [x] **Theme Sync:** Validated in both light mode and Anki `nightMode`.
- [x] **Zero Chrome:** No dashboard headers, web navbars, or decorative gradients.
- [x] **Contrast:** All text satisfies WCAG AAA contrast ratio ($\ge 7:1$ for body text).
- [x] **Focus States:** Every interactive element exhibits a crisp keyboard focus ring.
- [x] **No Layout Shifts:** Transitioning between states causes zero sudden jumping.
- [x] **Single Input Surface:** Exactly one input modality is visible per screen.
- [x] **Anti-Pattern Compliance:** Zero occurrences of `ANTI-01` through `ANTI-08`.
- [x] **14 Target States:** Satisfies all 14 visual verification states (including native Basic/Cloze isolation).
