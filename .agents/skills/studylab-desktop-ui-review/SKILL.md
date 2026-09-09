---
name: studylab-desktop-ui-review
description: >-
  Authoritative desktop-first visual and UX review skill for the StudyLab procedural
  subsystem inside the running Anki desktop application (naksh-07/anki-maths). Establishes
  a strict full-window review methodology, visual-quality framework, and anti-pattern ledger,
  using desktop-webview-reviewer as the physical execution and forensic inspection layer.
---

# StudyLab Desktop UI Review & Visual Quality Skill

The `studylab-desktop-ui-review` skill establishes the authoritative, desktop-first visual and UX review methodology for the StudyLab procedural learning subsystem running inside the real Anki desktop application on Windows (`naksh-07/anki-maths`).

Use this skill whenever you need to:
- Review, audit, or judge the visual quality, layout, hierarchy, and UX of StudyLab screens.
- Diagnose and repair UI defects across the Anki Qt host, webview container, and procedural solving canvas.
- Verify visual fixes across light and dark (`body.nightMode`) desktop themes.
- Enforce the Two-P0 desktop boundaries (button suppression, spacebar trapping, single interaction surface).
- Formulate forensic desktop visual acceptance criteria before claiming completion.

---

## 1. Quick Mental Model & The Desktop-First Doctrine

### 1.1 The Foundational Law
> **"StudyLab UI must NEVER be reviewed as an isolated web page."**
> 
> The actual product under review is the complete running Anki desktop application on Windows:
> 1. **Native Anki host window** (Win32 window frame, title bar, native menus, window state).
> 2. **Anki reviewer chrome** (Top navigation toolbar, bottom action bar, webview frame container).
> 3. **Embedded StudyLab WebView** (`mw.web` / QtWebEngine viewport).
> 4. **StudyLab procedural canvas** (`#procedural-card` / `.procedural-card-container` inside `#qa`).
> 5. **Interaction controls and states** (`SOLVING`, `MISTAKE CLASSIFICATION`, `FEEDBACK/SOLUTION`, `WORKED EXAMPLE`, etc.).
> 6. **Transitions between native and web surfaces** (shortcut interception, button suppression, theme changes, card advancing).

```text
┌──────────────────────────────────────────────────────────────────────────────────┐
│                      THE RUNNING ANKI DESKTOP REALITY                            │
├──────────────────────────────────────────────────────────────────────────────────┤
│ Win32 Top-Level Window (HWND: Visible, Mapped, DWM Borders, Title, Menus)        │
│ ┌──────────────────────────────────────────────────────────────────────────────┐ │
│ │ Top Toolbar Webview ("top toolbar" - Decks, Add, Browse, Stats, Sync)        │ │
│ ├──────────────────────────────────────────────────────────────────────────────┤ │
│ │ Main Application Webview ("main webview" - mw.web)                           │ │
│ │   ┌────────────────────────────────────────────────────────────────────────┐ │ │
│ │   │ Card Root (#qa)                                                        │ │ │
│ │   │   ┌──────────────────────────────────────────────────────────────────┐ │ │ │
│ │   │   │ StudyLab Procedural Canvas (#procedural-card: 720px open canvas) │ │ │ │
│ │   │   │   • Header (Breadcrumbs + Difficulty + Provenance)               │ │ │ │
│ │   │   │   • Visual Hero: Mathematical / Physical Problem Prompt          │ │ │ │
│ │   │   │   • Modality Workspace (MCQ options / Numerical / Stepwise CAS)  │ │ │ │
│ │   │   │   • Single Action Area (Submit / Hint)                           │ │ │ │
│ │   │   │   • Result Panel & Reflection Strip (Only active state)          │ │ │ │
│ │   │   └──────────────────────────────────────────────────────────────────┘ │ │ │
│ │   └────────────────────────────────────────────────────────────────────────┘ │ │
│ ├──────────────────────────────────────────────────────────────────────────────┤ │
│ │ Bottom Toolbar Webview ("bottom toolbar" - aqt.reviewer.bottom)              │ │
│ │   [CRITICAL: Must be SUPPRESSED on procedural cards (_is_procedural_card)]   │ │
│ └──────────────────────────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────────────────────┘
```

### 1.2 Architectural Layering: StudyLab vs Desktop WebView Reviewer
- **`desktop-webview-reviewer` (Underlying Execution Layer)**: Provides the universal desktop inspection, Win32/UIA window discovery, Chromium CDP attachment, ephemeral reference management (`n1e1`, `w1e2`), semantic input dispatch, and cryptographic evidence sealing (`evidence.json`). **Do NOT duplicate or reimplement these capabilities.**
- **`studylab-desktop-ui-review` (Domain Strategy & Quality Layer)**: Directs the desktop reviewer. Defines *what* to inspect, the 7-step inspection order, StudyLab-specific anti-patterns, visual quality criteria, state machine transitions, and acceptance gates.

---

## 2. The Hard 7-Step Review Methodology

Every visual or UX evaluation of StudyLab must strictly follow this seven-step sequence. Skipping directly to DOM inspection or judging CSS in isolation is prohibited.

```text
Step 1: Inspect Real Running Desktop Application First
  ↓
Step 2: Establish Native Desktop / Window Reality
  ↓
Step 3: Capture & Inspect Complete Full-Window Composition
  ↓
Step 4: Inspect Embedded WebView & DOM in Desktop Context
  ↓
Step 5: Evaluate Host-Guest Integration & Boundary Integrity
  ↓
Step 6: Diagnose Components, Layout Rhythm & Design Tokens
  ↓
Step 7: Post-Change Full Desktop + WebView Re-Verification
```

### Step 1: Inspect the Real Running Desktop Application First
- Never review mock HTML files, Storybook sandboxes, or headless browser dumps when assessing product visual quality.
- Connect to the running Anki desktop instance via `desktop_inspect` or launch the live debug instance:
  - Debug Media Server: `http://127.0.0.1:40000` (or configured dev port).
  - Remote Debugging Port: `127.0.0.1:9222` (QtWebEngine CDP).
- Execute `desktop_inspect` targeting the native window plane first.

### Step 2: Establish Native Desktop / Window Reality Before Judging WebView
- Verify the physical Win32 OS window reality:
  - `IsWindowVisible(HWND) == TRUE`
  - Window is foregrounded (`SetForegroundWindow`), mapped, and not minimized (`IsIconic == FALSE`).
  - Window is not cloaked (`DWMWA_CLOAKED == 0`).
  - Window geometry is valid (width $\ge 682\text{px}$, height $\ge 607\text{px}$).
- Confirm the window title matches the running Anki profile (e.g. `User 1 - Anki StudyLab`).
- **Rule**: If the native window is occluded, iconic, or zero-sized, abort the review with verdict `UNVERIFIED`. Never judge DOM styling on a non-visible desktop surface.

### Step 3: Capture & Inspect the Complete Desktop Composition
- Capture the full desktop window composition, not just the inner webview viewport:
  - Inspect the full visual bounding frame: OS title bar $\to$ top toolbar $\to$ main review canvas $\to$ bottom toolbar area $\to$ window borders.
  - Observe how the StudyLab canvas sits within the broader Anki window frame.
  - Check for vertical rhythm, breathing room, window margins, and visual center of gravity across the entire application window.

### Step 4: Inspect Embedded WebView & DOM in Context
- Target the main webview (`main webview` / `mw.web`) using `desktop_inspect`.
- Locate `#procedural-card` (or `.procedural-card-container`) embedded inside Anki's `#qa` container.
- Verify:
  - The container is horizontally centered with `max-width: 720px; margin: 0 auto;`.
  - Content fits comfortably within the viewport without unexpected horizontal scrollbars (`overflow-x: hidden`).
  - Font scaling matches Anki's user preference scale without clipping or overlapping text.

### Step 5: Evaluate Host $\leftrightarrow$ Guest Integration & Boundary Integrity
- Audit the interaction boundary between the Anki host shell and the StudyLab guest webview:
  1. **Bottom Toolbar Suppression (P0-B)**: The native Anki bottom rating buttons (`#ansbut` "Show Answer", ease buttons `1..4` / `Again, Hard, Good, Easy`) must be strictly suppressed and hidden during procedural solving and reflection (`qt/aqt/reviewer.py:_is_procedural_card()`).
  2. **Keyboard Trapping (P0-A)**: `Space` and `Enter` keydown events must be trapped by `onEnterKey()` and delegated to `handleNativeShowAnswer()`. Pressing Space/Enter must never bypass problem solving or skip mandatory mistake classification.
  3. **Theme Synchronization**: When Anki switches to Night Mode, `body.nightMode` must be present on `mw.web`, and `--proc-*` CSS variables must transition to dark surface values (`--proc-bg: #1e1e2e`). No white flashes or mismatched background boxes.
  4. **Teardown Cleanliness**: When advancing cards, `globalThis.anki.procedural.destroyActive()` must execute to prevent lingering listeners or key traps.

### Step 6: Only Then Diagnose Individual Components, CSS & Layout
- With the host context established, diagnose micro-level styling and components:
  - Problem prompt typography and contrast.
  - Mathematical typesetting (MathJax rendering, equation centering, KaTeX spacing).
  - Modality input controls (MCQ radio buttons, numerical input box, stepwise rows).
  - Feedback indicators and mistake classification strip.
  - Spacing rhythm against design tokens (`--proc-surface`, `--proc-border`, `--proc-divider`).

### Step 7: Post-Change Full Desktop + WebView Re-Verification
- After making any CSS, template, or TypeScript changes:
  - Re-run the full 7-step review cycle on the live desktop application.
  - **Rule**: Declaring victory from a passing unit test or DOM snapshot without verifying the full running desktop window is strictly forbidden.

---

## 3. The Prohibited Visual & Methodological Anti-Patterns Ledger

Every review must actively check for and eliminate the 6 desktop review anti-patterns (`ANTI-DESK-01` through `ANTI-DESK-06`) and the 8 frontend visual anti-patterns (`ANTI-01` through `ANTI-08`):

### Desktop Methodology Anti-Patterns
| ID | Anti-Pattern | Violation Description | Required Methodology |
|---|---|---|---|
| **ANTI-DESK-01** | **CDP / DOM Viewport Blindness** | Reviewing only a cropped CDP screenshot of `#procedural-card` and declaring visual success. | Capture and evaluate the complete Win32 window composition, including top/bottom toolbars and OS borders. |
| **ANTI-DESK-02** | **Host Isolation & Abandoned Chrome** | Polishing the inner webview while ignoring leaking Anki bottom buttons, window scrollbars, or broken menu states. | Actively audit the host-guest boundary (`qt/aqt/reviewer.py`). Ensure `#ansbut` and ease buttons are suppressed. |
| **ANTI-DESK-03** | **Web-App Syndrome** | Designing StudyLab like a standalone SaaS website (floating headers, sticky web navbars, giant colored banners, web-app scrollbars). | Adhere to Anki's native desktop aesthetic: calm, restrained, distraction-free Open Canvas with zero web chrome. |
| **ANTI-DESK-04** | **Decontextualized Component Evaluation** | Judging an isolated component (e.g. an MCQ option or button) outside of Anki's font-scaling, window width, and theme background. | Always review components rendered in their live desktop parent hierarchy (`#qa` $\to$ `mw.web` $\to$ Anki Main Window). |
| **ANTI-DESK-05** | **Virtual Victory Illusion** | Declaring UI success because a DOM element has `offsetParent != null` or a test passes, while the element is clipped or occluded on screen. | Verify physical screen coordinates fall within visible, non-occluded client rects via `desktop_assert` and visual review. |
| **ANTI-DESK-06** | **Premature Styling / Design Language Ignorance** | Modifying CSS rules or colors without inspecting existing design tokens in `ts/reviewer/reviewer.scss` or violating master contracts. | Use canonical CSS variables (`--proc-*`), adhere to `docs/contracts/`, and preserve the 720px Open Canvas structure. |

### Visual Design Anti-Patterns (from `docs/FRONTEND_VISUAL_DESIGN_SPEC.md`)
- **ANTI-01 (Giant Feedback Containers)**: No full-bleed saturated green/red background blocks covering the card. Use calm typography with a subtle 3px left accent border (`--proc-accent-left-correct`).
- **ANTI-02 (Duplicate Answer Labels)**: Single comparison row (`Your answer: X · Correct answer: Y`).
- **ANTI-03 (Ticking Stopwatch)**: No live numeric stopwatch ticking during solving; record time silently in background.
- **ANTI-04 (Competing Speed Badges)**: Single compact status pill (`⚡ Fast & Accurate · 8.4s`).
- **ANTI-05 (Generic Practice Chrome)**: No "VARIANT: PRACTICE" headers; preserve only verified competitive exam tags.
- **ANTI-06 (Raw Internal IDs)**: Internal schema strings (`math.algebra...`) must never be visible to learners.
- **ANTI-07 (Nested Card Boxes)**: No box-in-a-box nested containers in worked examples or feedback; use flat open canvas with subtle dividers.
- **ANTI-08 (Premature Solution Reveal)**: Solutions must be hidden while the learner is classifying mistakes.

---

## 4. Comprehensive StudyLab Visual Quality Framework

Evaluate the live running application against these 12 quality dimensions:

```text
┌──────────────────────────────────────────────────────────────────────────────────┐
│                   STUDYLAB VISUAL QUALITY EVALUATION FRAMEWORK                   │
├──────────────────────────────────────────────────────────────────────────────────┤
│ 1. Full-Window Composition      │ Balanced 720px open canvas inside Anki window │
│ 2. Hierarchy & Hero Invariant   │ Problem statement is the primary visual hero   │
│ 3. Spacing & Layout Rhythm      │ 8px/16px/24px rhythm, clean footer clearance   │
│ 4. Typography & Math            │ Crisp MathJax/KaTeX formulas, native fonts    │
│ 5. Visual Weight & Restraint    │ Subdued surfaces, 3px left accents, no blare   │
│ 6. Interaction Clarity          │ Clear hotkeys [A], [1], single primary action │
│ 7. Component Consistency        │ Uniform button styles (.proc-btn-*), badges   │
│ 8. Responsive & Window Scaling  │ Flawless from 682x607 up to 1920x1080         │
│ 9. Host ↔ Guest Integration    │ Seamless theme sync (Night Mode), no flashes   │
│ 10. Core State Coverage         │ Pristine ready, solving, correct, error states │
│ 11. Control Deduplication       │ Zero duplicate Next buttons; zero leaked ease  │
│ 12. Deliberate Product Feel     │ Calm, focused, distraction-free learning tool  │
└──────────────────────────────────────────────────────────────────────────────────┘
```

### 4.1 Full-Window Composition & Spatial Rhythm
- **The 720px Open Canvas**: The solving canvas (`.procedural-card-container`) is constrained to `max-width: 720px` and centered (`margin: 0 auto`).
- **Vertical Centering & Balance**: In standard window sizes ($1024 \times 768$ and above), the problem is vertically comfortable, neither squashed against the top navigation nor lost at the bottom.
- **Chrome Proportions**: Anki's top toolbar occupies minimal vertical space (~40px); StudyLab owns the central viewport; the bottom area remains calm and unobtrusive.

### 4.2 Visual Hierarchy & The Visual Hero Invariant
- **The Problem Statement is the Hero**:
  - The problem prompt (`.proc-prompt`, font size 1.25rem–1.4rem, font-weight 500/600) is the most prominent text on screen.
  - Equations inside the prompt stand out with high contrast and ample line height.
- **Metadata is Subordinate**:
  - Breadcrumbs (`.proc-breadcrumbs`, font size 12px, muted color) and difficulty badges (`.proc-diff-badge`) are quiet, sitting neatly above the prompt.
- **Controls are Secondary**:
  - Submit buttons, hint toggles, and options support the problem, never overshadowing it.

### 4.3 Spacing, Padding & Footer Reveal Rhythm
- **Consistent Grid**: 8px baseline (`8px`, `12px`, `16px`, `20px`, `24px`, `32px`).
- **Footer Reveal Clearance**: When the fixed bottom interaction footer mounts during mistake classification, the container expands smoothly (`.procedural-card-container:has(.proc-interaction-footer:has(> :not(.hidden))) { padding-bottom: 120px; }`) so content is never occluded behind bottom controls.
- **No Double Margins**: Ensure margins do not collapse awkwardly or create empty 80px voids between sections.

### 4.4 Typography, Mathematical Typesetting & Figures
- **Native Typography**: Inherits clean desktop system fonts (`-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif`).
- **Mathematical Formulations**:
  - Display math (`$$...$$`) is centered, properly padded, and typeset cleanly via MathJax.
  - Inline math (`$...$`) aligns with the text baseline without distorting line spacing.
  - Equations must never overflow the container horizontally; check for `overflow-x: auto` on complex formulas.
- **Tabular Figures**: Timers and numerical inputs use monospaced or tabular numbers (`font-variant-numeric: tabular-nums`).

### 4.5 Visual Weight & Surface Restraint
- **Open Canvas Restraint**: Use white/transparent backgrounds in light mode and dark canvas in night mode. Avoid heavy card borders or shadow overlays.
- **Semantic Left Accents**:
  - Correct feedback: `3px solid var(--proc-success)` with subtle background (`--proc-success-bg`).
  - Incorrect feedback: `3px solid var(--proc-error)` with subtle background (`--proc-error-bg`).
  - Worked example: `3px solid var(--proc-primary)` with subtle background (`--proc-surface-subtle`).
- **Restrained Pills**: Status pills use subdued surface tokens (`--proc-pill-bg`, `--proc-pill-text`), avoiding bright neon fills.

### 4.6 Interaction Clarity, Focus & Affordances
- **Clickable Affordances**: Buttons and options have clear cursor pointers, hover states, and active pressed feedback.
- **Hotkey Badges**: Options display hotkey indicators clearly (`[A]`, `[B]`, `[C]`, `[D]` or `[1]`, `[2]`, `[3]`, `[4]`).
- **Single Primary Action per State**:
  - In `solving`: One primary button (`#proc-submit-btn`), with secondary actions (`#proc-hint-btn`) visually deprioritized.
  - In `mistake_classification`: Four balanced category buttons in a single row.
  - In `worked_example`: Single acknowledgement gate button (`[ ✔ I Have Reviewed and Understood This Solution ]`).

### 4.7 Component Consistency & Token Discipline
- **Token Compliance**: All colors, borders, and surfaces must reference canonical CSS custom properties defined in `ts/reviewer/reviewer.scss`:
  - Surfaces: `--proc-bg`, `--proc-surface`, `--proc-surface-subtle`, `--proc-surface-hover`.
  - Text: `--proc-text-primary`, `--proc-text-secondary`, `--proc-text-muted`.
  - Borders: `--proc-border`, `--proc-border-subtle`, `--proc-border-focus`, `--proc-divider`.
  - Semantics: `--proc-primary`, `--proc-success`, `--proc-error`, `--proc-warning`.
- **Button System**: Uniform usage of `.proc-btn`, `.proc-btn-primary`, and `.proc-btn-secondary`.

### 4.8 Responsive & Window-Size Adaptability
- Test and verify across three target desktop resolutions:
  1. **Minimum Supported (`682 x 607`)**: Core problem, options, and controls remain fully accessible without clipping, button wrapping collisions, or missing footer buttons.
  2. **Standard Desktop (`1024 x 768`)**: Ideal reading width, comfortable margins, perfectly centered 720px canvas.
  3. **Maximized / High-DPI (`1920 x 1080`)**: Canvas remains constrained to 720px; does not stretch across wide displays like a broken banner.

### 4.9 Native-Host $\leftrightarrow$ WebView Integration & Night Mode
- **Night Mode Synchronization**:
  - Test with Anki in light mode and night mode (`body.nightMode`).
  - Confirm background seamlessly transitions (`--proc-bg: #1e1e2e`), text becomes light (`--proc-text-primary: #f8fafc`), and formula colors remain legible.
  - Zero bright white flashes during card transitions in dark mode.

### 4.10 Comprehensive State Coverage
Verify visual quality across all 7 operational states:
1. **`solving` (Numerical)**: Input box `.proc-input` cleanly styled; magnitude preview pill visible; submit button aligned.
2. **`solving` (MCQ)**: Option cards `.proc-option-item` render with radio semantics (`role="radio"`); hotkey pills aligned; zero text input fallback.
3. **`correct` (Transient Feedback)**: Muted inline status (`✓ Correct`); speed quadrant pill (`⚡ Fast & Accurate · 8.4s`); automatic advance triggers cleanly without user click.
4. **`mistake_classification` (Incorrect Flow)**:
   - Mistake classification strip mounted at the bottom in a single row (`[1 Silly Slip]`, `[2 Pattern Missed]`, `[3 Concept Gap]`, `[4 Prereq Unknown]`).
   - `#proc-solution-container` remains hidden until a category is clicked (`ANTI-08`).
   - `Space` and `Enter` keys are locked (`onEnterKey()` anti-bypass trap).
5. **`stepwise` (Multi-Step Workspace)**: Step rows with step numbers, descriptions, input boxes, and action row (`+ Add Step`, `💡 Request Hint`, `Reset`, `Check Solution`).
6. **`worked_example` (Schema Remediation)**: Step derivation trace rendered open-canvas; zero solving inputs; acknowledgement gate button prominently placed.
7. **`error` / Safe Fallback**: Informative, non-destructive error box (`Procedural Engine Error: ...`) when payload deserialization fails, with zero application crash.

### 4.11 Duplicate & Competing Controls Elimination
- **NO "Next Card" or "Next Problem" Button**: Per `docs/contracts/StudyLab-Runtime-UI-Contract.md`, there is NO user-facing Next Card button anywhere in StudyLab. Correct flow automatically advances; incorrect flow advances upon selecting a mistake category.
- **NO Visible Anki Ease Buttons**: Anki's `#ansbut` and ease buttons (`Again`, `Hard`, `Good`, `Easy`) must remain suppressed throughout procedural review.

### 4.12 The "Deliberate StudyLab Product" Litmus Test
Ask the ultimate evaluative question:
> **"Does this feel like a calm, distraction-free, professional mathematical workbench that belongs natively inside Anki — or does it look like someone pasted a messy web quiz into a window?"**

If the answer is anything less than a native, deliberate StudyLab workbench, diagnose and resolve the visual defects before approving.

---

## 5. Integration with `desktop-webview-reviewer`

The `studylab-desktop-ui-review` skill leverages `desktop-webview-reviewer` for all physical interactions and observations:

```text
┌────────────────────────────────────────────────────────┐
│             studylab-desktop-ui-review                 │
│   (Review Strategy, Quality Framework, Invariants)     │
└───────────────────────────┬────────────────────────────┘
                            │ Delegates physical execution
┌───────────────────────────▼────────────────────────────┐
│              desktop-webview-reviewer                  │
│  (Win32 / CDP Discovery, Ephemeral Refs, Assertions,   │
│   Cryptographic Evidence Sealing, Forensic Logs)       │
└────────────────────────────────────────────────────────┘
```

### 5.1 Invocation Protocol
1. **Launch / Attach**:
   - Use `desktop_launch` or `desktop_attach` to bind to the running Anki desktop instance.
2. **Dual-Perspective Inspection**:
   - Execute `desktop_inspect(target_type="native")` to establish Win32 window reality (`HWND`, geometry, foreground).
   - Execute `desktop_inspect(target_type="webview")` to inspect the embedded DOM within `mw.web`.
3. **Semantic Interaction**:
   - Use `desktop_click(ref="w1e3")`, `desktop_type(ref="w1e4", text="...")`, or `desktop_press_key(key="Enter")` to drive user flows.
   - Respect ephemeral reference discipline: references (`w1e1`, `n1e2`) are session-scoped and expire across observations.
4. **Physical Reality Verification**:
   - Use `desktop_assert` to verify that actions caused physical state changes (e.g. classification strip appeared, prompt updated).
5. **Forensic Evidence Collection**:
   - After completing an audit or verifying a fix, invoke `desktop_collect_evidence` to seal the run:
     - Saves dual screenshots, DOM snapshots, action receipts, and SHA-256 hash manifest.
     - Evidence bundle is permanently recorded under `C:\Users\Suraj\Documents\Antigravity\Data-Collection\evidence`.

---

## 6. Canonical Sources of Truth & Path Ledger

When evaluating or modifying StudyLab desktop UI, verify against these authoritative files:

| Subsystem Domain | Primary File Path | Authority & Responsibilities |
|---|---|---|
| **Master UI Contract** | `docs/contracts/StudyLab-Runtime-UI-Contract.md` | Canonical master contract: anatomy, suppressed controls, NO Next Card button. |
| **Master Interaction Contract** | `docs/contracts/StudyLab-Runtime-Interaction-Contract.md` | Master state machine, automatic advance, keyboard contract, Space/Enter trap. |
| **Visual Design Spec** | `docs/FRONTEND_VISUAL_DESIGN_SPEC.md` | Open Canvas design tokens, anti-patterns `ANTI-01..08`, night mode variables. |
| **UI Composition Contract** | `docs/STUDYLAB_UI_COMPOSITION_CONTRACT.md` | 12 screen compositions, 23-control matrix, modality specifications. |
| **Architecture Invariants** | `docs/ARCHITECTURE_INVARIANTS.md` | The 16 frozen architectural rules, host-guest boundaries. |
| **Canonical Desktop Verifier** | `tools/verify_desktop_ui.py` | Authoritative runner backed by `desktop-webview-reviewer` (Hard Preflight, Real Cards, Dual Evidence). |
| **Historical Forensic Report** | `docs/FINAL_LIVE_UI_FORENSIC_REPORT.md` | Historical reference (INVALIDATED / NON-CERTIFYING due to duplicate hashes and unrendered engine error). |
| **TypeScript State Machine** | `ts/reviewer/procedural.ts` | `ProceduralReviewer`, `handleNativeShowAnswer()`, `destroyActive()`. |
| **Reviewer Stylesheet** | `ts/reviewer/reviewer.scss` | `:root` design tokens, `body.nightMode`, `.procedural-card-container`. |
| **Python Reviewer Bridge** | `qt/aqt/reviewer.py` | `_is_procedural_card()`, button suppression, `onEnterKey()`, `_handle_procedural_command()`. |
| **Rust Template Core** | `rslib/procedural/src/reviewer/template.rs` | `render_reviewer_html()`, `escape_html()`, Open Canvas HTML generation. |
| **Desktop Reviewer Capability** | `desktop-webview-reviewer` (MCP & CLI) | Physical desktop inspection and cryptographic evidence collection engine (`desktop-reviewer.exe`). |

---

## 7. Supporting References

For the operational audit rubric, state-by-state evaluation checklist, and pre-flight scoring sheet, consult:
**[`references/desktop-visual-framework.md`](references/desktop-visual-framework.md)**
