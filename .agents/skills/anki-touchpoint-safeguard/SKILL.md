---
name: anki-touchpoint-safeguard
description: >-
  Authoritative safety guard and touchpoint firewall skill for StudyLab inside Anki (naksh-07/anki-maths).
  Enforces strict confinement of all procedural integrations to the 4 approved upstream Anki touchpoints,
  forbidding modifications to standard flashcards, upstream schemas, or core Anki subsystems.
---

# Anki Touchpoint Safeguard & Host Integrity Skill

The `anki-touchpoint-safeguard` skill defines the absolute boundaries and safety invariants governing how the StudyLab procedural engine interfaces with upstream Anki (`naksh-07/anki-maths`).

Use this skill when:
- Reviewing pull requests, git diffs, or proposed code modifications that touch files under `rslib/src/`, `pylib/`, or `qt/`.
- Verifying that non-procedural flashcard workflows (`Basic`, `Cloze`, Image Occlusion) remain 100% unaffected.
- Auditing that Anki's upstream collection database (`collection.anki21`) is never polluted with rich procedural telemetry.

---

## 1. The 4 Approved Upstream Touchpoints

By architectural decree, StudyLab is permitted to interface with upstream Anki at **ONLY 4 discrete touchpoint files**:

```text
┌──────────────────────────────────────────────────────────────────────────────────┐
│                     THE 4 APPROVED UPSTREAM ANKI TOUCHPOINTS                     │
├────────────────────┬───────────────────────────────────────────┬─────────────────┤
│ Touchpoint         │ File Path                                 │ Responsibility  │
├────────────────────┼───────────────────────────────────────────┼─────────────────┤
│ 1. Storage Init    │ `rslib/src/collection/mod.rs`             │ Initialize the  │
│                    │                                           │ isolated SQLite │
│                    │                                           │ .procedural db  │
├────────────────────┼───────────────────────────────────────────┼─────────────────┤
│ 2. Card Render     │ `rslib/src/notetype/render.rs`            │ Detect procedur-│
│                    │                                           │ al cards & hook │
│                    │                                           │ Svelte reviewer │
├────────────────────┼───────────────────────────────────────────┼─────────────────┤
│ 3. Telemetry Strip │ `rslib/src/scheduler/answering/mod.rs`    │ Strip rich data │
│                    │                                           │ to <= 100 bytes;│
│                    │                                           │ map rating 1..4 │
├────────────────────┼───────────────────────────────────────────┼─────────────────┤
│ 4. APKG Import     │ `rslib/src/import_export/package/apkg/`   │ Reconcile Level │
│                    │ `import/mod.rs`                           │ 1 note models on│
│                    │                                           │ package import  │
└────────────────────┴───────────────────────────────────────────┴─────────────────┘
```

Any modification to `rslib/src/` outside these 4 files is an immediate **ARCHITECTURAL BOUNDARY BREACH** and will fail review automatically.

---

## 2. Detailed Touchpoint Specifications

### Touchpoint 1: Storage Initialization (`rslib/src/collection/mod.rs`)
- **Action**: When Anki opens a collection path `<col_path>`, instantiate `ProceduralStore::open(<col_path>.procedural)`.
- **Constraint**: The procedural store must use a separate SQLite database file. It must never create tables, alter schemas, or execute DDL inside Anki's primary `collection.anki21` database.

### Touchpoint 2: Card Rendering Hook (`rslib/src/notetype/render.rs`)
- **Action**: During template rendering, inspect the note type's name and configuration. If the card is identified as a procedural card (`is_procedural() == true`), wrap the output payload for the Svelte Reviewer Open Canvas.
- **Constraint**: If the card is NOT procedural (`Basic`, `Cloze`, `Cloze-Overlapping`), standard Anki Mustache rendering must proceed with zero overhead and zero alterations.

### Touchpoint 3: Telemetry Stripping & Rating Bridge (`rslib/src/scheduler/answering/mod.rs`)
- **Action**: When an answer is submitted for a procedural card:
  1. Record rich telemetry (step derivations, mistake categories, latency) into `collection.procedural`.
  2. Derive an FSRS-compatible rating ($1 = \text{Again}, 2 = \text{Hard}, 3 = \text{Good}, 4 = \text{Easy}$).
  3. Strip and compress card metadata to ensure `card.data` committed to Anki does not exceed 100 bytes.
- **Constraint**: Never pass raw JSON telemetry strings to Anki's scheduler.

### Touchpoint 4: APKG Import Reconciliation (`rslib/src/import_export/package/apkg/import/mod.rs`)
- **Action**: During package import, inspect note models. If a note model matches the Level 1 Frozen contract, register its procedural blueprint anchors.
- **Constraint**: Standard decks must import using Anki's standard import pipeline without alteration.

---

## 3. Strict Prohibitions (Zero Tolerance)

1. **NO Schema Alterations**: Never add tables or columns to `collection.anki21`.
2. **NO Flashcard Pollution**: Never alter rendering, templates, or styling for `Basic` or `Cloze` notes.
3. **NO Direct Upstream File Edits**: Do NOT modify files under `rslib/src/` other than the 4 listed above. All procedural logic lives strictly in `rslib/procedural/`.
4. **NO UI Bypass**: Never allow procedural cards to bypass PyQt ease button suppression.

---

## 4. Audit Verification Procedure

Before approving any diff:
```bash
# Verify git touchpoint hygiene
git diff --name-only origin/main...HEAD | grep "^rslib/src/"
```
If the output contains any file other than the 4 approved touchpoints, reject the change.
