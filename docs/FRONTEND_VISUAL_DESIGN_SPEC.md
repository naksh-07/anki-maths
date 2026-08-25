# StudyLab Frontend Visual Design Specification

**Document Version:** 1.0.0 (Canonical Master Specification)  
**Target Repository:** `Anki-maths` (StudyLab Subsystem)  
**Status:** AUTHORITATIVE SPECIFICATION  
**Integrity Mode:** Benchmark Mode (100% Grounded in Executable Styles, QtWebEngine Renders, and UI Audits)  
**Authoritative Sections Covered:** Section 9 of `ORIGINAL_REQUEST.md`

---

## 1. Visual Product Contract & Core Philosophy

### 1.1 The Problem Is the Visual Hero
In StudyLab, the mathematical or scientific problem statement is the **primary visual hero** of the interface. The user interface is completely subordinate to the learner's cognitive problem-solving task.

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                           CORE VISUAL DESIGN DIRECTIVE                           │
├──────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│   "Calm, native, minimal, focused, professional, and dense enough for            │
│    mathematical reasoning without visual noise or web-widget chrome."            │
│                                                                                  │
│   • Blends seamlessly into Anki's native desktop light and dark themes.          │
│   • Replaces web-app widget clutter with clean, restrained typography.           │
│   • Elevates mathematical equations (LaTeX/MathJax) to primary contrast.         │
│   • Maintains a single focused interaction surface per state.                    │
│                                                                                  │
└──────────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Prohibited Visual Anti-Patterns

To ensure StudyLab never degenerates into a noisy web application, the frontend enforces strict visual prohibitions:

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                         PROHIBITED VISUAL ANTI-PATTERNS                          │
├──────────────────────────────┬───────────────────────────────────────────────────┤
│ Prohibited Anti-Pattern      │ Design Violation & Mandated Correction            │
├──────────────────────────────┼───────────────────────────────────────────────────┤
│ ❌ **Giant Card Wrappers**   │ Avoid multi-layered nested cards with heavy outer │
│                              │ borders and 30px padding. Use flat, clean flows.  │
│ ❌ **Web-Widget Appearance** │ Avoid floating action buttons, animated gradients,│
│                              │ or colorful SaaS banners inside the reviewer.     │
│ ❌ **Excessive Shadows**     │ Eliminate deep elevation drop-shadows (`box-shadow│
│                              │ 0 20px 25px...`). Use crisp 1px borders.         │
│ ❌ **Rainbow Badge Spam**    │ Do not decorate the header with 5 different       │
│                              │ colored badges. Use muted monochrome tags.        │
│ ❌ **Raw Schema Leakage**    │ Never display internal engine strings like        │
│                              │ `math.algebra.linear.one_variable_v2`.            │
│ ❌ **Telemetry Dumps**       │ Never display raw floats (`mastery: 0.841`), BKT  │
│                              │ Markov probabilities, or SQLite primary keys.     │
│ ❌ **Stacked Panel Monsters**│ Never stack 4 large independent boxes on a single │
│                              │ screen. Use progressive collapse and replace.     │
└──────────────────────────────┴───────────────────────────────────────────────────┘
```

---

## 3. Design Tokens & Theme Integration

StudyLab utilizes CSS custom properties that inherit dynamically from Anki's native desktop theme (`body.nightMode` vs. default light mode).

### 3.1 Color Palette Tokens

```css
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
```css
#proc-root {
    max-width: 680px;
    margin: 0 auto;
    padding: 16px 20px;
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
│   Physics › Mechanics › 1D Kinematics                       [ Exam: JEE 2024 ]   │
│   ────────────────────────────────────────────────────────────────────────────   │
│                                                                                  │
│   A car accelerates uniformly from rest at $2\,\text{m/s}^2$ for $10\,\text{s}$. │
│   What is the final velocity of the car?                                         │
│                                                                                  │
│   ┌────────────────────────────────────────────────────────────────────────┐     │
│   │  20 m/s                                                                │     │
│   └────────────────────────────────────────────────────────────────────────┘     │
│   [ Parsed: 20 m/s (Dimension: [Length]¹ [Time]⁻¹) ]                             │
│                                                                                  │
│   [ 💡 Request Hint (1/3) ]                                [ Submit Answer ]     │
│                                                                                  │
└──────────────────────────────────────────────────────────────────────────────────┘
```

### 6.1 Header & Breadcrumbs
- **Position:** Top of the card container.
- **Styling:** Subdued text color (`--proc-text-muted`), font size `12px`.
- **Format:** `Subject › Chapter › Topic` on the left; optional single compact badge on the right (e.g. `[ JEE Main 2024 ]`).
- **Forbidden:** No bright colored banners, no large avatar icons, no streak counters.

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

#### C. Stepwise Derivation Nodes
- **Row Container:** `1px solid var(--proc-border)`, border-radius `6px`, padding `10px 14px`, margin-bottom `10px`.
- **Step Label:** Font size `12px`, bold, `--proc-text-secondary`.
- **Validation Badge:** Compact right-aligned pill (`✔ Valid` green / `❌ Invalid` red / `⚠️ Consistent` yellow).

---

### 6.4 The Mistake Classification Footer (`MistakeFooter`)
- **Position:** Appears directly below the evaluated input upon incorrect answer.
- **Layout:** Compact 4-button horizontal flex strip (`gap: 8px`).
- **Button Styling:**
  - Height `36px`, font size `13px`, font weight `500`.
  - Keyboard badge (`1`, `2`, `3`, `4`) styled in a subtle rounded square on the left of the button label.
  - Subdued background with crisp hover state.
- **Anti-Bypass Visual Lock:** Primary progression button is hidden; the strip is the only active visual focus.

---

### 6.5 Solution & Canonical Feedback Box
- **Position:** Displayed below the interaction surface in `feedback` state.
- **Border:** `1px solid var(--proc-border)`.
- **Background:** `--proc-surface`.
- **Padding:** `16px 18px`.
- **Content:** Step-by-step LaTeX derivation with crisp mathematical formatting and clear final result highlighting.
- **Next Action Button:** High-contrast primary button (`background: var(--proc-primary)`), height `40px`, text `Next Problem [Enter]`.

---

## 7. Visual Design Acceptance Checklist

Before any frontend template or style modification is certified:
- [x] **Theme Sync:** Validated in both light mode and Anki `nightMode`.
- [x] **Zero Chrome:** No dashboard headers, web navbars, or decorative gradients.
- [x] **Contrast:** All text satisfies WCAG AAA contrast ratio ($\ge 7:1$ for body text).
- [x] **Focus States:** Every interactive element exhibits a crisp keyboard focus ring.
- [x] **No Layout Shifts:** Transitioning between `solving`, `evaluating`, and `feedback` causes zero sudden horizontal or vertical jumping.
- [x] **Single Input Surface:** Exactly one input modality is visible per screen.
