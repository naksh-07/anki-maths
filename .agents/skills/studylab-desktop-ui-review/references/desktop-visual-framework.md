# StudyLab Desktop Visual Quality Framework & Audit Rubric

This reference document provides the operational checklists, DOM selector mappings, state transition requirements, and forensic scoring rubrics used by the `studylab-desktop-ui-review` skill.

---

## 1. Pre-Flight Desktop Verification Checklist (Gate #0)

Before analyzing any visual elements, verify the physical desktop reality using `desktop_inspect` on the native plane (`target_type="native"`):

| Check Item | Acceptance Criteria | Forensic Method | Failure Disposition |
|---|---|---|---|
| **OS Window Visibility** | `IsWindowVisible(HWND) == TRUE` | `desktop_inspect(target_type="native")` | ABORT: `UNVERIFIED` |
| **Window State** | `IsIconic(HWND) == FALSE` (not minimized) | Win32 Window Longs | Bring window to foreground |
| **DWM Cloaking** | `DWMWA_CLOAKED == 0` (rendered on active desktop) | DwmGetWindowAttribute | Unhide / restore desktop |
| **Window Dimensions** | Width $\ge 682\text{px}$, Height $\ge 607\text{px}$ | Native Rect inspection | Resize window to standard ($1024 \times 768$) |
| **Foreground Focus** | Window has OS keyboard focus | `SetForegroundWindow` | Request user / focus window |
| **Process Tree** | Running genuine Python/PyQt process (not mock) | Process PID inspection | Launch via dev script |

---

## 2. Host $\leftrightarrow$ Guest Boundary Inspection Matrix

Review the boundary integrity between the Anki Qt host and the StudyLab webview:

```text
┌────────────────────────────────────────────────────────────────────────┐
│ HOST BOUNDARY AUDIT                                                    │
├────────────────────────────────────────────────────────────────────────┤
│ [ ] Top Toolbar: Cleanly rendered, standard Anki navigation links      │
│ [ ] Main Webview: Contains #qa and #procedural-card without clipping    │
│ [ ] Bottom Toolbar: Native #ansbut and #ease1..4 are HIDDEN            │
│ [ ] Space/Enter Trap: onEnterKey() delegates to procedural handler     │
│ [ ] Theme Sync: body.nightMode matches Qt palette (light vs dark)      │
│ [ ] Teardown: destroyActive() called before next card mount            │
│ [ ] Normal Card Isolation: Basic/Cloze cards retain native ease bars   │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 3. Comprehensive 12-State Visual Audit Rubric

### State 1: `solving_numerical` (Numerical Quick Solve)
- **Target Container:** `.procedural-card-container[data-object-type="problem"]`
- **DOM Selectors:**
  - Prompt: `.proc-prompt` (Hero typography, crisp MathJax equation)
  - Input: `input#proc-answer-input.proc-input` (visible, focused or focusable, clean border)
  - Submit: `button#proc-submit-btn.proc-btn-primary` (aligned with input row)
  - Hint: `button#proc-hint-btn.proc-btn-secondary` (subdued, below input)
  - Bottom Bar: Anki ease buttons strictly suppressed.
- **Visual Checks:** Input has ample padding ($10\text{px}\times14\text{px}$), placeholder text is legible, submit button has primary blue fill (`--proc-primary`).

### State 2: `solving_mcq` (Multiple Choice Question)
- **Target Container:** `.procedural-card-container[data-object-type="mcq"]`
- **DOM Selectors:**
  - Options Container: `.proc-options-list`
  - Options: `.proc-option-item[role="radio"]` (4 structured option cards)
  - Hotkey Badge: `.proc-opt-key` (`[A]`, `[B]`, `[C]`, `[D]` or `[1]`, `[2]`, `[3]`, `[4]`)
  - Option Text: `.proc-opt-text`
  - **Zero Text Input:** `input[type="text"]` MUST NOT exist in DOM.
- **Visual Checks:** Radio items have distinct hover state (`--proc-surface-hover`), selected state has left accent border and subtle background tint (`--proc-primary-light`).

### State 3: `correct_feedback` (Transient Correct Submission)
- **Target Container:** `#proc-result-panel.proc-result.correct`
- **DOM Selectors:**
  - Title: `#proc-result-title` (`✓ Correct` with `--proc-success` color)
  - Comparison: `#proc-result-feedback` (`Your answer: X · Correct answer: X`)
  - Speed Pill: `.proc-speed-quadrant` (`⚡ Fast & Accurate · 4.2s`)
  - Bottom Toolbar: Leaked ease buttons strictly absent.
- **Visual Checks:** Subtle 3px green left border (`--proc-accent-left-correct`), NO giant saturated full-bleed green background (`ANTI-01`), advances automatically to next card.

### State 4: `wrong_answer` (Incorrect Submission Immediate Transition)
- **Target Container:** `#proc-result-panel.proc-result.incorrect`
- **DOM Selectors:**
  - Title: `#proc-result-title` (`✗ Incorrect Answer` with `--proc-error` color)
  - Comparison: `#proc-result-feedback` (`Your answer: X · Correct answer: Y`)
  - Transition: Immediately exposes mistake classification strip.
- **Visual Checks:** 3px red left border (`--proc-accent-left-incorrect`), prompt remains visible above feedback for context.

### State 5: `mistake_classification` (Metacognitive Reflection Strip)
- **Target Container:** `.proc-interaction-footer:not(.hidden)`
- **DOM Selectors:**
  - Mistake Strip: `#proc-mistake-panel`
  - Category 1: `#proc-mistake-btn-1` (`[1 Silly Slip]`)
  - Category 2: `#proc-mistake-btn-2` (`[2 Pattern Missed]`)
  - Category 3: `#proc-mistake-btn-3` (`[3 Concept Gap]`)
  - Category 4: `#proc-mistake-btn-4` (`[4 Prereq Unknown]`)
  - Solution Container: `#proc-solution-container.hidden` (MUST BE HIDDEN - `ANTI-08`)
  - Next Button: NO Next Card or Next Problem button (`docs/contracts/StudyLab-Runtime-UI-Contract.md`).
- **Visual Checks:** Buttons arranged in a single horizontal row, equal visual weight, container expanded with `padding-bottom: 120px` to prevent occlusion.

### State 6: `post_reflection_solution` (Solution Derivation Reveal)
- **Target Container:** `#proc-solution-container:not(.hidden)`
- **DOM Selectors:**
  - Solution Body: `.proc-solution-body` (multi-step LaTeX derivation)
  - Derivation Steps: Cleanly formatted equations with explanatory text.
- **Visual Checks:** High contrast formula text, no nested card boxes (`ANTI-07`), flat open canvas with subtle dividers (`--proc-divider`).

### State 7: `stepwise_workspace` (Multi-Step CAS Derivation)
- **Target Container:** `#proc-stepwise-container`
- **DOM Selectors:**
  - Steps List: `#proc-steps-list`
  - Step Rows: `.proc-step-row` with step labels and step inputs `.proc-step-input`
  - Action Row: `#proc-add-step-btn`, `#proc-hint-btn`, `#proc-check-steps-btn`
- **Visual Checks:** Each step row has clear visual separation; active step is highlighted; validation states (green tick / amber question) render cleanly next to step equations.

### State 8: `worked_example` (Schema Remediation - Tier 70)
- **Target Container:** `.proc-worked-box` / `#procedural-card[data-object-type="worked_example"]`
- **DOM Selectors:**
  - Trace Steps: `.proc-worked-step` (step descriptions + highlighted equations)
  - Key Decision Point: `.proc-decision-highlight` (subtle accent highlight)
  - Input Box: ZERO text input box in DOM.
  - Acknowledgement Gate: `button#proc-understand-btn` (`[ ✔ I Have Reviewed and Understood This Solution ]`)
- **Visual Checks:** Low cognitive load layout, calm reading rhythm, prominent acknowledgement button.

### State 9: `concept_check` (Misconception Diagnostic - Tier 50)
- **Target Container:** `.procedural-card-container[data-object-type="concept_check"]`
- **DOM Selectors:**
  - Qualitative Prompt: `.proc-prompt`
  - Conceptual Distractors: `.proc-option-item` mapped to specific misconception feedback
  - Formative Explanation: Rendered upon option selection.
- **Visual Checks:** Distractor feedback provides diagnostic explanation rather than blunt "Wrong" banner.

### State 10: `strategy_drill` (Method Selection - Tier 60)
- **Target Container:** `.procedural-card-container[data-object-type="strategy_drill"]`
- **DOM Selectors:**
  - Strategy Options: Competing solution methods (e.g. Energy Conservation vs Kinematic Trajectory)
  - Efficiency Rating: Highlights optimal path with cognitive complexity rationale.
- **Visual Checks:** Clear comparison layout, cards indicate relative efficiency without clutter.

### State 11: `normal_anki_isolation` (Standard Non-Procedural Flashcard)
- **Target Container:** Standard Anki Card DOM (`.card`, `#qa`)
- **DOM Selectors:**
  - StudyLab Container: `.procedural-card-container` MUST NOT exist.
  - Procedural Classes: ZERO `.proc-*` elements.
  - Native Ease Buttons: `#ansbut` / `#ease1..4` are VISIBLE and functional.
- **Visual Checks:** 100% native Anki styling, zero CSS pollution, zero lingering keyboard traps.

### State 12: `engine_error_containment` (Graceful Fallback)
- **Target Container:** `.proc-advisory-box` or `.procedural-card-container`
- **DOM Selectors:**
  - Error Notice: `Procedural Engine Error: ...`
  - Crash Isolation: Anki Qt host does NOT crash or close; user can advance or navigate back to decks.
- **Visual Checks:** Safe, non-destructive advisory; raw stack traces or unhandled exceptions are hidden from learners.

---

## 4. Operational Desktop Reviewer Execution Script

When running physical reviews using `desktop-webview-reviewer`:

```python
# Canonical Desktop Review Flow
# 1. Inspect Native Window
native_state = desktop_inspect(target_type="native")
assert native_state["is_visible"] == True
assert native_state["rect"]["width"] >= 682
assert native_state["rect"]["height"] >= 607

# 2. Inspect Webview Hierarchy
web_state = desktop_inspect(target_type="webview")
card_ref = web_state.find_by_selector("#procedural-card")
assert card_ref is not None

# 3. Verify Bottom Bar Suppression
bottom_state = desktop_inspect(target_type="bottom_bar")
assert bottom_state.find_by_selector("#ansbut") is None or bottom_state.is_hidden("#ansbut")

# 4. Interact via Ephemeral Reference
desktop_click(ref=web_state.find_by_selector(".proc-option-item[data-opt-id='B']"))

# 5. Assert Post-Action Reality
desktop_assert(
    selector="#proc-result-panel",
    condition="is_visible",
    timeout_ms=1000
)

# 6. Seal Cryptographic Evidence
evidence = desktop_collect_evidence(
    session_id="studylab-review-session",
    description="State 3: Correct feedback verification"
)
```

---

## 5. Post-Fix Verification Scoring Sheet

Use this scoring sheet in walkthroughs and reports:

```text
================================================================================
STUDYLAB DESKTOP VISUAL QUALITY AUDIT SCORE SHEET
================================================================================
Review Target:        [Component / State / Flow under test]
Window HWND:          [Win32 Handle]
Window Dimensions:    [e.g. 1024x768 / 682x607]
Theme Mode:           [Light Mode | body.nightMode]

DIMENSION                             STATUS      OBSERVATIONS
--------------------------------------------------------------------------------
1. Full-Window Composition            [PASS/FAIL] [Balanced 720px centered canvas]
2. Hierarchy & Visual Hero            [PASS/FAIL] [Problem prompt is clear hero]
3. Spacing & Layout Rhythm            [PASS/FAIL] [8px grid, clean footer clearance]
4. Typography & MathJax               [PASS/FAIL] [Crisp LaTeX, no baseline drift]
5. Visual Weight & Restraint          [PASS/FAIL] [Subtle left accents, no neon]
6. Interaction Clarity                [PASS/FAIL] [Clear hotkeys, distinct states]
7. Component Consistency              [PASS/FAIL] [Token-compliant styles]
8. Window Size Adaptability           [PASS/FAIL] [No overflow at 682x607]
9. Host ↔ Guest Integration           [PASS/FAIL] [Theme synced, ease suppressed]
10. State Machine Fidelity            [PASS/FAIL] [Expected state active]
11. Control Deduplication             [PASS/FAIL] [Zero duplicate Next/Ease btns]
12. Deliberate Product Feel           [PASS/FAIL] [Calm native STEM workbench]
--------------------------------------------------------------------------------
VERDICT: [PASS | FAIL | UNVERIFIED]
EVIDENCE BUNDLE: [desktop://evidence/<id>/manifest.json]
================================================================================
```
