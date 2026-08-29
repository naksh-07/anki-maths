# StudyLab Runtime UI Contract

**Status:** Canonical Master Contract

## 1. Scope
This document defines the canonical visual and UX source of truth for the StudyLab procedural UI embedded in Anki. It supersedes all prior documentation, forensic reports, and gap maps.

## 2. StudyLab vs Anki Responsibility Boundary
- **StudyLab**: Owns the visual layout of procedural card content, the solving workspace, the mistake classification footer, and the Hint button.
- **Anki**: Acts solely as the host/runtime infrastructure. Native Anki rating controls must NOT be part of the StudyLab UX. Normal Anki/non-procedural cards must NOT inherit the StudyLab UI.

## 3. Canonical Card Anatomy

A procedural StudyLab card MUST conform to the following structural flow:

**CARD CONTENT**
- Metadata / Breadcrumbs
- Problem statement / Question
- Options / Input Workspace
- Hint button (when available)
- Feedback / Solution (transiently shown during correct flow)

**BOTTOM STUDYLAB INTERACTION FOOTER**
- Only shown during the Mistake Classification phase.

**NATIVE ANKI RATING CONTROLS**
- Hidden and strictly suppressed for all procedural StudyLab cards.

## 4. Forbidden UI Elements
- There is **NO** "Next Card" or "Next Problem" button.
- There are **NO** visible native Anki "Again / Hard / Good / Easy" rating buttons.
- There is **NO** duplicate mistake-classification footer. The classification footer must appear exactly once at the bottom of the review UI.

## 5. Classification Footer Contract
When a learner submits an incorrect answer, a mistake classification footer MUST appear at the bottom of the StudyLab interaction area.

**Canonical Classification Buttons:**
1. `[1 Silly Slip]`
2. `[2 Pattern Missed]`
3. `[3 Concept Gap]`
4. `[4 Prereq Unknown]`

**Positioning & Invariants:**
- Located at the bottom of the StudyLab interaction footer.
- Rendered in a single row.
- Follows the visual layout described in the golden-reference screenshot (`deckbrowser_test.png` or equivalent reference).
- Must NEVER be duplicated elsewhere on the screen.

## 6. Hint Contract
- Hint is a StudyLab-owned control.
- Must have a stable, documented position below the solving input workspace.

## 7. Procedural vs Normal-Card Behavior
- Normal, non-procedural cards fall back entirely to Anki's native behavior and must retain Anki's native rating bottom bar. They must NOT show the StudyLab interaction footer.

## 8. Explicit Contradiction Resolution
- **Rule Superseded**: Previous specifications required a "Next Problem" button (e.g., in `STUDYLAB_UI_COMPOSITION_CONTRACT.md` or forensic reports).
  - **New Rule**: There is NO Next Card/Problem button. Correct flow automatically advances; Incorrect flow advances immediately upon mistake classification.
- **Rule Superseded**: Previous architectures exposed native Anki ease buttons or integrated them with StudyLab UI.
  - **New Rule**: Native Anki controls are strictly suppressed for StudyLab procedural cards.
- **Rule Superseded**: Duplicate footers or DOM containers based on old architectural limits.
  - **New Rule**: Only one classification footer exists. It is logically owned by StudyLab, placed at the bottom, without forcing arbitrary DOM duplication.
