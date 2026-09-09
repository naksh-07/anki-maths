# Directory-Scoped Rules: TypeScript & Svelte Reviewer (`ts/reviewer/`)

**Authority:** Level 1 Subsystem Architectural Invariant  
**Target:** `ts/reviewer/**`  
**Governing Subagents:** `visual-qa-inspector`, `implementer`, `reviewer-verifier`

---

## 1. The Zero Next-Card Button Invariant

1. **Never Render "Next Card" Button**: The StudyLab solving canvas must NEVER render a "Next Card" or "Continue" button during active problem-solving or immediately after answering.
2. **Automatic Advance on Correct**: When a student provides a correct answer, show the 3px green left accent (`--proc-accent-left-correct`), brief positive feedback, and automatically advance to the next card after a standard delay (1000ms).
3. **Cognitive Classification on Incorrect**: When a student answers incorrectly, immediately render the [1..4] cognitive mistake classification grid (Conceptual, Calculation, Misread, Guess) to diagnose the root cause before advancing.

---

## 2. Semantic Modality Invariant

Semantic modality must ALWAYS match UI modality. Fallbacks that violate this mapping are strictly forbidden:

1. **Multiple-Choice Questions (`modality: "mcq"`)**:
   - MUST render discrete option cards with letter badges (A, B, C, D).
   - NEVER render a free-form text input box or textarea as a fallback.
2. **Numerical Drills (`modality: "numerical"`)**:
   - MUST render a specialized numerical keypad/input with a unit selector badge.
3. **Stepwise Derivations (`modality: "stepwise"`)**:
   - MUST render discrete step containers validating each intermediate derivation sequentially.

---

## 3. Open Canvas 720px Layout & CSS Tokens

1. **Container Width**: Max-width is strictly capped at `720px`. The container must be horizontally centered within the webview viewport (`margin: 0 auto`).
2. **Design Tokens**: All colors, borders, and shadows must use CSS variables from `variables.scss`:
   - Canvas background: `var(--proc-canvas-bg)`
   - Text color: `var(--proc-text-primary)`
   - Correct accent: `var(--proc-accent-left-correct)`
   - Error accent: `var(--proc-accent-left-incorrect)`
3. **Night Mode Conformance**: All styles must render cleanly without contrast regression when `body.nightMode` is active.

---

## 4. Verification Requirements

Before submitting changes in this directory:
```bash
npm --prefix ts test ts/reviewer/procedural.test.ts
just lint
```
