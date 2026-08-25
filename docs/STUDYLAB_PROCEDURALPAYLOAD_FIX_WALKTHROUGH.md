# ProceduralPayload Runtime Defect Fix & Validation

## 1. Root Cause
The live Anki UI error `"ProceduralPayload field is missing or empty."` was technically a symptom of an upstream content factory defect masking as a runtime fault. 

When the Anki Rust backend (`rslib/src/notetype/render.rs`) reads the `ProceduralPayload` field, it attempts to deserialize it into `ProceduralCardAnchor` using strict validation (`procedural::anchor::ProceduralCardAnchor::from_json_str`). If validation fails, it safely swallows the error (returning `Ok(None)`) and logs the defect to standard error, falling back to rendering the "ProceduralPayload field is missing or empty" banner.

The JSON payload was malformed because `tools/studylab_content_factory.py` emitted invalid variants for the `StepType` and `AnswerDerivation` enums that did not match the Rust definitions in `step_graph.rs` and `contract.rs`. Specifically:
- `logical_inference` was used instead of `make_inference`
- `algebraic_manipulation` was used instead of `equation_rearrangement`
- `conceptual_verification` was used instead of `verify_conclusion`
- `strategic_decision` was used instead of `select_strategy`
- `physical_law_application` was used instead of `select_equation`
- `chemical_stoichiometry` was used instead of `apply_stoichiometric_ratio`
- `pythagoras_hypotenuse` mistakenly used `leg_a_param`/`leg_b_param` instead of the expected `a_param`/`b_param`.

Since these strings are used with `#[serde(rename_all = "snake_case")]` in Rust, `serde_json` refused to parse the entire anchor, leading to the payload being treated as entirely missing.

## 2. Files Changed
- `tools/studylab_content_factory.py`: Fixed the injected enums and param keys to perfectly align with `step_graph.rs` and `contract.rs`.

## 3. Stale Artifacts Removed
- Purged all duplicate or old APKG models from `dist/apkgs/`.
- Cleared the existing `C:\Users\Suraj\AppData\Roaming\AnkiStudyLab` profile folder.

## 4. Build Performed
- Ran the core Anki build system: `tools\ninja.bat pylib qt`.
- Compilation succeeded for release artifacts.

## 5. Fresh Canonical APKG
Generated exactly one official distribution package with zero invalid payload strings:
- `dist/apkgs/StudyLab_Full_Universe_175.apkg`

## 6. APKG SHA-256
`6FC030BED4E572B60BA163B23E0011FF70E91BE479EF77372A9FD4ADAD6F0F1C`

## 7. Fresh DEV Profile Used
- Deleted `%APPDATA%\AnkiStudyLab`
- The system will regenerate the default profile on launch.

## 8. Runtime Validation Result
- Verified `from_json_str_strict` parsing against all 175 generated contracts directly via Rust `cargo test`.
- Launched headless instance attached to Pyenv and evaluated the `card.question()` HTML from the Anki collection runtime engine.
- Result: **PASS**. The fallback banner is gone, replaced entirely by valid structured `proc-option-group` rendering contexts.

## 9. ProceduralPayload Field Proof
The payload is properly inserted as `ord: 0` inside the `col` / `notes` tables, and is now perfectly matching the shape expected by `procedural::anchor::ProceduralCardAnchor`.

## 10. Final PowerShell Launch Command
```powershell
Start-Process cmd.exe -ArgumentList '/c','_launch_dev.bat','dist\apkgs\StudyLab_Full_Universe_175.apkg'
```

## 11. Remaining Limitations
- While the payload matches structural schemas, content difficulty balancing across varying `target_time_ms` values in the actual frontend might require UX tuning.
- The user will need to confirm whether the specific procedural UX elements are aesthetically finalized (a Visual UI Audit was expressly disabled for this mission).
