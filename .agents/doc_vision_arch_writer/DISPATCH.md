# DISPATCH RECORD

## 2026-08-25T02:14:09+05:30
MISSION:
Author and produce the canonical, authoritative documentation for StudyLab's Vision, Boundaries, Architecture, and Invariants in `docs/`:
1. `docs/README.md` (Top-level entry point, North Star, system purpose, architecture summary, navigation guide).
2. `docs/PRODUCT_VISION.md` (Product North Star, procedural mastery vs flashcard memory, cognitive science foundation).
3. `docs/PRODUCT_BOUNDARIES.md` (Clean, non-negotiable boundaries between Anki host SRS and StudyLab procedural engine).
4. `docs/SYSTEM_ARCHITECTURE.md` (Rust backend rslib/procedural, TS reviewer frontend, Python/Qt bridge, data flow, telemetry, isolation).
5. `docs/ARCHITECTURE_INVARIANTS.md` (Frozen non-negotiable invariants: StudyLab is not a flashcard system, Anki owns SRS & card state, StudyLab owns procedural engine & mastery, no FSRS corruption, strict memory safety & teardown).
