# StudyLab Diagnostic Model

StudyLab diagnostic sessions are explicit **measurement instruments for learner weaknesses**, designed to discover precise areas of deficiency. They are NOT generic percentage scoreboards or synthetic mock exams.

## 1. Multi-Dimensional Measurement
A diagnostic session evaluates learner capability across four orthogonal dimensions:
- **Concept Errors**: Did they use the wrong formula or misapply a core theorem?
- **Execution Slips (Calculation)**: Did they know the concept but make a minor arithmetic error?
- **Transfer Deficits**: Can they apply the pattern when the superficial context changes?
- **Speed Deficits**: Can they execute the steps fluently?

## 2. Speed Quadrant Analysis
Every interaction is benchmarked against a dynamically calibrated `target_latency_ms`. Submissions are plotted on a Speed Quadrant (`ts/reviewer/procedural.ts` `computeSpeedQuadrant`):
- ⚡ **Fluency Strength**: Accurate & Fast
- ⏱ **Speed Opportunity**: Accurate but Slow
- ⚠️ **Strategy Trap**: Fast but Incorrect
- 💡 **Concept Setup**: Slow & Incorrect

## 3. Aggregation & Targeted Remediation
- **The UI Report:** The mock hierarchical report (`ts/reviewer/diagnostic/diagnostic_report.ts`) aggregates skill gaps into the 4 dimensions.
- **Backend Analytics:** The Rust backend (`rslib/procedural/src/exam/analytics.rs`) consumes `ProceduralReviewOutcome` to generate a `ComprehensiveDiagnosticReport`.
- **State Integration:** Through `apply_diagnostic_report_to_store` (`mock.rs:855`), findings are atomically mapped into `SkillState` and `VersionedDomainEvidence`, enabling the system to queue high-priority remediation. Practice algorithms use priority tiers like `PriorityTier::WeaknessAndDiagnostics` to patch identified weak skills instead of executing arbitrary generic repetitions.

---
### Traceability & Code Evidence
- **Mock Session Engine:** `rslib/procedural/src/exam/mock.rs` and `rslib/procedural/src/diagnostics/mod.rs`.
- **UI Reporting:** `ts/reviewer/diagnostic/diagnostic_report.ts`.
- **State Updates:** `apply_diagnostic_report_to_store` in `mock.rs:855`.
