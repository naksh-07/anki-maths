---
name: apkg-contract-conformance
description: >-
  Authoritative verification protocol and contract conformance skill for StudyLab
  Level 1 Frozen APKG packages, Field 0 JSON procedural payloads, and package validation in Anki (naksh-07/anki-maths).
---

# APKG Contract Conformance & Package Conformance Skill

The `apkg-contract-conformance` skill provides the authoritative specifications and validation procedures for StudyLab's Level 1 Frozen APKG package format (`StudyLab-Source-APKG-Contract(1).txt`, `docs/contracts/`) inside Anki (`naksh-07/anki-maths`).

Use this skill when:
- Generating, updating, or debugging `.apkg` packages or generation scripts (`generate_canonical_source_apkg.py`, `generate_procedural_apkg.py`).
- Auditing the Field 0 JSON procedural payload embedded in card templates.
- Running automated package validation against `artifacts_qa/validate_canonical_apkg.py`.
- Verifying note model definitions, field configurations, and media file bindings.

---

## 1. Level 1 Frozen APKG Contract Overview

StudyLab distributes procedural learning content via standard Anki `.apkg` zip packages. The packaging structure is frozen at Level 1 to guarantee forward and backward compatibility:

```text
┌──────────────────────────────────────────────────────────────────────────────────┐
│                           LEVEL 1 FROZEN APKG STRUCTURE                          │
├──────────────────────────────────────────────────────────────────────────────────┤
│ my_deck.apkg (ZIP Archive)                                                       │
│   ├── collection.anki21 (SQLite database with notes, cards, and models)          │
│   ├── media (JSON map connecting media filenames to numeric keys: {"0": "img.png"})│
│   └── [0..N] (Binary asset files)                                                │
└──────────────────────────────────────────────────────────────────────────────────┘
```

### Note Type Specifications
Every procedural card must use an approved StudyLab note type (e.g., `StudyLab Procedural Math`, `StudyLab Source Question`).
- **Field 0**: Mandatory JSON string containing the procedural anchor payload.
- **Field 1**: Fallback human-readable plain text or KaTeX question statement.
- **Field 2**: Fallback human-readable explanation / worked solution.

---

## 2. Field 0 JSON Procedural Payload Contract

Field 0 MUST be a strictly valid JSON object adhering to the schema:

```json
{
  "version": 1,
  "topic_id": "math_calculus_limits_01",
  "problem_id": "calc_lim_0042",
  "modality": "stepwise",
  "template": "\\lim_{x \\to {a}} \\frac{x^2 - {a}^2}{x - {a}}",
  "variables": {
    "a": { "type": "integer", "min": 2, "max": 9 }
  },
  "solution": {
    "steps": [
      { "step_id": "s1", "instruction": "Factor the numerator", "target": "(x - {a})(x + {a})" },
      { "step_id": "s2", "instruction": "Cancel common factor", "target": "x + {a}" },
      { "step_id": "s3", "instruction": "Substitute x = a", "target": "2 * {a}" }
    ]
  },
  "distractors": [
    { "type": "sign_error", "value": "-2 * {a}" },
    { "type": "zero_factor", "value": "0" }
  ]
}
```

### Invariants for Field 0
1. **Modality Purity**: The `modality` key must be one of: `"mcq"`, `"numerical"`, `"stepwise"`, or `"discrete"`.
2. **Deterministic Interpolation**: Variables declared in `"variables"` must match all placeholders in `"template"` and `"solution"`.
3. **No Dynamic Execution**: Field 0 must contain pure data, never executable JavaScript or Python code.

---

## 3. Package Validation Protocol

Every generated `.apkg` must pass the authoritative validation script:

```bash
python artifacts_qa/validate_canonical_apkg.py
```

### What the Validator Asserts:
1. **Archive Integrity**: Package is a valid ZIP archive containing `collection.anki21` and `media`.
2. **Schema Conformance**: SQLite tables `col`, `notes`, `cards` exist and are uncorrupted.
3. **Field 0 Parsing**: Every note's Field 0 parses as valid JSON and conforms to the anchor contract.
4. **Modality-UI Match**: Notes with `modality: "mcq"` contain valid distractor options; numerical notes contain valid unit specifications.
5. **No Orphan Assets**: Every entry in the `media` map points to an existing file in the archive.

---

## 4. Troubleshooting Common Validation Failures

| Failure Mode | Root Cause | Remediation |
| :--- | :--- | :--- |
| `JSONDecodeError in Field 0` | Unescaped backslashes in LaTeX strings (`\frac` instead of `\\frac`). | Use raw strings or double-escape backslashes in Python generator. |
| `Missing topic_id` | Generator emitted legacy format missing topic anchor. | Update generator script to include `topic_id`. |
| `Invalid Modality` | Unknown modality value (e.g. `"text"`). | Map to canonical values: `"mcq"`, `"numerical"`, `"stepwise"`. |
| `Media Key Mismatch` | Numeric key in `media` JSON does not match archived filename. | Ensure keys are stringified integers (`"0"`, `"1"`). |
