# Dispatch History

## 2026-08-25T02:02:50+05:30
Agent: doc_python_archaeologist
Mission: Exhaustive fact-finding audit of the Python/Qt layer (`qt/aqt/reviewer.py`, Python bridge, hooks, desktop integration, diagnostic session engine, mock tests, and tests).

Ground Truth Areas to Probe:
1. Python/Qt Reviewer Bridge: `qt/aqt/reviewer.py`, JS message handlers (`pycmd`), bridge dispatchers for answer submission, mistake logging, card grading.
2. Hook Lifecycle: Standard Anki reviewer hooks (`reviewer_did_show_question`, `reviewer_did_show_answer`, `reviewer_did_answer_card`, etc.) and StudyLab interceptors.
3. Diagnostic Mock-Test Engine: Session manager, 4-domain question selection (Math, Reasoning, Physics, Chemistry), test session lifecycle, report generation UI bridge.
4. Standard Anki Non-Regression: How standard flashcards are identified vs StudyLab procedural learning objects; verification of zero interference.
5. Tests: Enumerate Python test files (`pylib/`, `qt/`), test runners, and test coverage.
