# StudyLab UI Forensic Investigation Report

## 1. Objective and Methodology
The objective was to reproduce the procedural-card flow (specifically the MCQ incorrect path) and observe the behavior of StudyLab's interaction controls vs. Native Anki controls without modifying any application code.
We executed a clean build, started Anki with QtWebEngine remote debugging enabled (port 9222), and used a CDP client to drive the UI via JavaScript evaluation and capture DOM/screenshot states.

### Commands Run
- 	ools\ninja pylib qt (Clean build)
- .\run_debug.bat c:\Users\Suraj\Documents\Antigravity\Anki-maths\dist\apkgs\studylab-demo-v1.0.apkg (Launch with debug port)
- Evaluated pycmd('open:1787954315074') and pycmd('study') to begin review.
- Evaluated document.querySelectorAll('.proc-option-item')[0].click() to simulate an incorrect answer.
- Dispatched Spacebar keydown events and pycmd('ans') to observe shortcut bypass behavior.

## 2. Observations and DOM Evidence

### StudyLab Mistake Classification Footer (Duplicates?)
- **Observation:** The Mistake Classification footer (#proc-mistake-panel) is correctly injected into #proc-interaction-footer when an incorrect answer is selected. 
- **Duplicate Check:** DOM dumps (count.py) show exactly ONE instance of #proc-mistake-panel. 
- **Source:** The MistakeFooter component in 	s/reviewer/components/mistake_footer.ts explicitly searches for existing #proc-mistake-panel elements and deduplicates them (existingPanels[0]) before appending. This successfully prevents duplicates.

### Next Card Button
- **Observation:** The StudyLab Next Card button (#proc-next-btn) is **completely missing** from the DOM in both main_webview and ottom_toolbar.
- **Evidence:** Regex searches on the dumped main_webview.html yielded 0 matches for proc-next-btn. 
- **Cause:** It appears the button was removed directly from the Anki Note Type HTML template (perhaps as a heavy-handed mitigation for previous duplicate issues). This creates a soft-lock where users cannot click a Next button to proceed.

### Native Anki Footer & Navigation Bypassing
- **Observation:** The standard Anki Native "Show Answer" and "Ease" (Again/Hard/Good/Easy) buttons are intentionally suppressed for procedural cards by _showAnswerButton() in qt/aqt/reviewer.py. Only "Edit" and "More" remain visible.
- **The Spacebar Bypass Bug:**
  - Pressing the Space key (or invoking pycmd('ans')) triggers _showAnswer() in eviewer.py.
  - For procedural cards, Python delegates to JavaScript: globalThis.anki.procedural.handleNativeShowAnswer().
  - However, handleNativeShowAnswer() in procedural.ts **no-ops** if the card is already in the eedback or mistake_classification state.
  - As a result, Python transitions its internal state to "answer", while the StudyLab UI remains stuck in mistake classification.
  - If the user presses Space **again**, Python evaluates selectedAnswerButton() in the bottom toolbar. Since no ease buttons exist, it returns empty, causing eviewer.py to default to _answerCard(3) (Ease: Good).
  - **Conclusion:** This silently skips the mandatory mistake reflection and blindly advances the user to the next card.

## 3. Identification of Owners
- **Mistake Classification Footer:** Owned by 	s/reviewer/components/mistake_footer.ts. (No duplicates found due to explicit deduplication logic).
- **Next Card Button:** Historically owned by the Note Type template HTML and procedural.ts (which toggles its visibility). Currently missing from the template.
- **Native Anki Footer / Spacebar Navigation:** Owned by qt/aqt/reviewer.py (_showAnswer, _answerCard, _linkHandler). The desync is caused by procedural.ts not handling handleNativeShowAnswer for all states, allowing Python to proceed with native shortcut logic.

## 4. DOM Snapshots Captured
- main_webview_mistake.html: Shows the DOM state during mistake classification (no Next button, 1 Mistake panel).
- ottom_toolbar_mistake.html: Shows the suppressed Native Anki footer (only Edit/More buttons).
