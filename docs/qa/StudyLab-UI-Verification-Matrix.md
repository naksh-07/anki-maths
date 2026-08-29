# StudyLab UI Verification Matrix

## 1. Fresh Procedural Card
- **Setup**: Load a procedural StudyLab card in Anki.
- **Action**: Observe UI.
- **Expected Visual Result**: Problem statement, input/options, Hint button (if present) are visible. NO Anki bottom rating bar. NO Mistake Classification footer. NO Next Card button.
- **Expected State Result**: `SOLVING` state.
- **Pass/Fail Invariant**: Anki native controls MUST be hidden.

## 2. Correct Answer
- **Setup**: Card in `SOLVING` state.
- **Action**: Submit correct answer.
- **Expected Visual Result**: Transient feedback/solution rendered.
- **Expected State Result**: Transitions through `FEEDBACK/SOLUTION` to automatic advancement.
- **Pass/Fail Invariant**: System advances to next card without requiring a "Next" button click.

## 3. Incorrect Answer
- **Setup**: Card in `SOLVING` state.
- **Action**: Submit incorrect answer.
- **Expected Visual Result**: Single Mistake Classification footer appears at the bottom.
- **Expected State Result**: `MISTAKE CLASSIFICATION` state.
- **Pass/Fail Invariant**: Only exactly ONE mistake footer appears. Native controls remain hidden.

## 4. Classification Buttons (Mouse)
- **Setup**: Card in `MISTAKE CLASSIFICATION` state.
- **Action**: Click `[3 Concept Gap]`.
- **Expected Visual Result**: Selection registered, card advances.
- **Expected State Result**: Advance to next card.
- **Pass/Fail Invariant**: Card advances immediately upon classification.

## 5. Keyboard 1-4
- **Setup**: Card in `MISTAKE CLASSIFICATION` state.
- **Action**: Press `2` on keyboard.
- **Expected Visual Result**: Registers `[2 Pattern Missed]`.
- **Expected State Result**: Advance to next card.
- **Pass/Fail Invariant**: Numeric keys successfully map to classification.

## 6. Space/Enter During Classification
- **Setup**: Card in `MISTAKE CLASSIFICATION` state.
- **Action**: Press `Space` or `Enter`.
- **Expected Visual Result**: No advancement.
- **Expected State Result**: Remains in `MISTAKE CLASSIFICATION` state.
- **Pass/Fail Invariant**: Space/Enter MUST NOT bypass mandatory classification.

## 7. Footer Duplication
- **Setup**: Card in `MISTAKE CLASSIFICATION` state.
- **Action**: Inspect DOM and visual layout.
- **Expected Visual Result**: Only ONE footer exists.
- **Expected State Result**: N/A.
- **Pass/Fail Invariant**: The mistake footer must not appear twice.

## 8. Missing Footer / Wrong Footer Location
- **Setup**: Submit incorrect answer.
- **Action**: Inspect footer location.
- **Expected Visual Result**: Footer is positioned at the absolute bottom of the interaction boundary.
- **Expected State Result**: N/A.
- **Pass/Fail Invariant**: Matches the golden-reference screenshot layout.

## 9. Hint Behavior
- **Setup**: Card in `SOLVING` state.
- **Action**: Click Hint button.
- **Expected Visual Result**: Hint appears in a stable, documented position (e.g., below workspace).
- **Expected State Result**: System records hint telemetry.
- **Pass/Fail Invariant**: Hint does not disrupt the main workspace layout.

## 10. Native Anki Rating Buttons
- **Setup**: Load procedural card.
- **Action**: Inspect bottom of window.
- **Expected Visual Result**: Native Again/Hard/Good/Easy buttons are completely suppressed.
- **Expected State Result**: N/A.
- **Pass/Fail Invariant**: Zero native Anki ease buttons on procedural cards.

## 11. Normal / Non-Procedural Cards
- **Setup**: Load a standard Anki Basic card.
- **Action**: Observe UI and answer card.
- **Expected Visual Result**: Standard Anki formatting. Native bottom rating buttons are visible.
- **Expected State Result**: Standard Anki review state.
- **Pass/Fail Invariant**: Normal cards MUST NOT inherit StudyLab UI or hide native Anki controls.
