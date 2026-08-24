# Agent Artifacts Directory (`.agents/`)

> [!IMPORTANT]
> **Historical Artifact Notice:** All documents and folders within `.agents/` are historical planning, dispatch, briefing, and investigative artifacts generated during earlier development phases.
> 
> The **frozen canonical documentation** for StudyLab architecture, learning models, contracts, state machines, and system invariants resides strictly in [`docs/`](../docs/README.md).

---

## Directory Classification

| Directory / Artifact | Classification | Purpose & Status |
| :--- | :---: | :--- |
| `docs/*.md` | **CANONICAL** | Authoritative, source-verified documentation of the active codebase. |
| `.agents/ORIGINAL_REQUEST.md` | **HISTORICAL** | Initial prompt and mission brief from previous engineering waves. |
| `.agents/orchestrator/` | **HISTORICAL** | Orchestration briefings and handoff logs from prior phase dispatches. |
| `.agents/sentinel/` | **HISTORICAL** | Sentinel monitoring artifacts from earlier execution runs. |
| `.agents/specialist1_ux_archaeologist/` through `specialist10_qa_security/` | **HISTORICAL / EVIDENCE** | Specialized research notes, APKG inspection scripts, and gap analyses. |
| `.agents/independent_verifier_auditor/` | **EVIDENCE** | Independent verification audits and desktop review logs. |
| `scratch/` / Temporary Logs | **TEMPORARY** | Transient execution scratchpads. |

---

## AI Agent Guidance
If you are an AI assistant tasked with understanding, maintaining, or extending StudyLab, **always refer to [`docs/README.md`](../docs/README.md) as the primary source of truth**. Do not rely on `.agents/` artifacts for canonical architecture or type signatures.
