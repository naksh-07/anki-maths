# StudyLab Documentation & Source-Truth Reconciliation Execution Plan

## Objective
Make the StudyLab documentation complete, authoritative, and perfectly reconciled with the executable codebase, contracts, tests, and deep cognitive-science research, so a fresh AI agent can understand the entire system without guessing, conversation history, or phase reports.

## Phase Breakdown & Subagent Mapping

### Phase 1: Fact-Finding & Primary Research Track
- **Subagent 1: DEEPSEARCH PEDAGOGICAL RESEARCHER** (`teamwork_preview_worker`)
  - Target: Conduct DeepSearch cognitive science / psychometric research across questions A-G and author `docs/DEEPSEARCH_EVIDENCE.md`.
- **Subagent 2: RUST ENGINE CODEBASE ARCHAEOLOGIST** (`teamwork_preview_spec_miner`)
  - Target: Mine exact ground truth from Rust `rslib/procedural`, schemas, migrations, StepValidator, DB persistence, mastery models, and tests.
- **Subagent 3: TYPESCRIPT REVIEWER ARCHAEOLOGIST** (`teamwork_preview_spec_miner`)
  - Target: Mine exact ground truth from `ts/reviewer`, MCQ/Numerical/Stepwise containers, mistake footer, state machine, MutationObserver teardown, TS tests.
- **Subagent 4: PYTHON/QT BRIDGE ARCHAEOLOGIST** (`teamwork_preview_spec_miner`)
  - Target: Mine exact ground truth from `qt/aqt/reviewer.py`, language bridge, hook dispatching, desktop integration, diagnostic session engine, mock tests.

### Phase 2: Documentation Truth Matrix Synthesis
- **Subagent 5: TRUTH MATRIX ARCHITECT** (`teamwork_preview_worker`)
  - Target: Synthesize findings from Phase 1 into `docs/DOCUMENTATION_TRUTH_MATRIX.md` with every required area, actual code evidence, test evidence, product intent, status, and required doc change.

### Phase 3: Canonical Document Authoring & Cross-Reconciliation
- **Subagent 6: VISION & ARCHITECTURE DOC WRITER** (`teamwork_preview_worker`)
  - Target: Author/Reconcile `docs/README.md`, `docs/PRODUCT_VISION.md`, `docs/PRODUCT_BOUNDARIES.md`, `docs/SYSTEM_ARCHITECTURE.md`, `docs/ARCHITECTURE_INVARIANTS.md`.
- **Subagent 7: LEARNING & PEDAGOGY DOC WRITER** (`teamwork_preview_worker`)
  - Target: Author/Reconcile `docs/LEARNING_MODEL.md`, `docs/CONTENT_AND_AUTHORING.md`, `docs/LEARNING_OBJECTS.md`, `docs/DIAGNOSTIC_AND_REMEDIATION.md`.
- **Subagent 8: CONTRACTS & PERSISTENCE DOC WRITER** (`teamwork_preview_worker`)
  - Target: Author/Reconcile `docs/REVIEWER_STATE_MACHINE.md`, `docs/FRONTEND_BACKEND_CONTRACT.md`, `docs/DATA_AND_PERSISTENCE.md`, `docs/DOCUMENTATION_MAP.md`, `docs/OPEN_QUESTIONS.md`.

### Phase 4: Independent Verification, Auditing, Clean-Context AI Simulation & 18-Point Final Freeze
- **Subagent 9: CROSS-DOC CONSISTENCY REVIEWER** (`teamwork_preview_reviewer`)
  - Target: Cross-doc consistency audit, term definition integrity, quality scoring per doc (target >= 90/100).
- **Subagent 10: CLEAN-CONTEXT AI SIMULATION CHALLENGER** (`teamwork_preview_challenger`)
  - Target: Emulate clean-context AI agent answering all 16 core questions solely from canonical docs.
- **Subagent 11: FORENSIC AUDITOR & FINAL FREEZE EVALUATOR** (`teamwork_preview_auditor`)
  - Target: 18-point final freeze checklist verification, benchmark integrity check (zero code modifications), overall suite score >= 95/100, official freeze verdict.
