# StudyLab APKG Content Contract & Architecture Specification

**Document Version:** 2.0.0 (Canonical Architecture Hierarchy)  
**Document Authority:** Level 3 Content Specification  
**Authoritative Canonical Contract:** `StudyLab-Source-APKG-Contract(1).txt` (Level 1 — FROZEN)  
**Target Subsystems:** Canonical Source APKG Ingestion, Procedural Blueprints Packaging (`tools/studylab_content_factory.py`), Rust Deserializer (`rslib/procedural/src/anchor/`), Notetype Interception (`rslib/src/notetype/render.rs`)  
**Status:** CANONICAL SPECIFICATION  
**Contract Freeze Status:**
```text
Canonical StudyLab Source APKG Contract
Status: FROZEN

Phase 1: COMPLETE
Phase 2: COMPLETE
Phase 3: COMPLETE
Phase 4: COMPLETE & FROZEN
```

---

## 1. Documentation Authority & Content Architecture Scope

This document defines the content specifications for StudyLab `.apkg` packages. StudyLab supports two distinct, compatible content architectures:

```text
StudyLab APKG Content Architecture
│
├── PART I: CANONICAL SOURCE-FIRST PATH (StudyLab Source)
│   └── Canonical static APKGs containing immutable curated questions (MCQ / Numerical)
│       Governed Authoritatively by: StudyLab-Source-APKG-Contract(1).txt
│
└── PART II: PROCEDURAL PATH (StudyLab Procedural Anchor)
    └── Declarative problem blueprints generating dynamic mathematical variants
        Governed by: DeclarativeFamilyContract & ProceduralPayload Schemas
```

> [!IMPORTANT]
> **Authority Invariant:** `StudyLab-Source-APKG-Contract(1).txt` is the frozen Level 1 source of truth for the canonical StudyLab Source APKG. The procedural blueprint architecture (Part II) is a separate compatible content pathway that does **not** redefine, modify, or override the canonical Source APKG contract.

---

# PART I — CANONICAL STUDYLAB SOURCE APKG CONTRACT

StudyLab natively ingests, reconciles, and renders curated static source questions (such as official Previous Year Questions or textbook exercises) without executing dynamic parameter generation.

All static source packages must adhere strictly to `StudyLab-Source-APKG-Contract(1).txt`.

## 2. Canonical Source Note Model & Field Specification

### 2.1 Note Model & Interception
- **Note Type Name:** Notes starting with `"StudyLab Source"` (e.g. `"StudyLab Source Question"`, `"StudyLab Source Anchor"`).
- **Interception Hook:** Intercepted in [`rslib/src/notetype/render.rs`](file:///c:/Users/Suraj/Documents/Antigravity/Anki-maths/rslib/src/notetype/render.rs) via `render_source_anchor`.
- **Target Ingestion Model:** Parsed and validated directly into `SourceQuestion` ([`rslib/procedural/src/anchor/source.rs`](file:///c:/Users/Suraj/Documents/Antigravity/Anki-maths/rslib/procedural/src/anchor/source.rs)).
- **Reviewer Rendering:** Renders directly in the Open Canvas UI (`ts/reviewer/procedural.ts`) with zero dynamic parameter sampling.

### 2.2 Canonical Field Specification

| Field Name | Category | Required / Optional | Data Type | Description & Validation Rules |
|---|---|---|---|---|
| **`Prompt`** | Content | **Mandatory** | Plain Text / LaTeX | The primary question text or problem statement. Must not be empty. |
| **`QuestionType`** | Semantics | **Mandatory** | String Enum | Explicit question type: `"mcq"` (or `"multiple_choice"`) or `"numerical"` (or `"numeric"`). Never inferred. |
| **`CorrectAnswer`** | Content | **Mandatory** | String | The canonical answer. For MCQ: must match or resolve to one of the provided `Options`. For Numerical: must parse as numeric finite float. |
| **`Options`** | Content | Mandatory for MCQ | JSON Array / Newlines | Array of at least 2 option strings (e.g. `["A", "B", "C", "D"]`). Omitted or ignored for Numerical. |
| **`Difficulty`** | Semantics | Optional | Float String | Authored source difficulty rating in range `[1.0, 5.0]`. Preserved as immutable source metadata. |
| **`Subject`** | Semantics | Optional | String | Discipline: `"mathematics"`, `"physics"`, `"chemistry"`, `"reasoning"`. |
| **`Chapter`** | Semantics | Optional | String | Topic grouping (e.g. `"Algebra"`, `"Kinematics"`). |
| **`Topic`** | Semantics | Optional | String | Specific problem concept (e.g. `"Linear Equations"`, `"Projectile Motion"`). |
| **`Skill`** | Semantics | Optional | String | Fine-grained skill identifier (e.g. `"math.algebra.linear_two_step"`). |
| **`ProblemType`** | Semantics | Optional | String | Pedagogical categorization (e.g. `"standard"`, `"trap_check"`, `"transfer"`). |
| **`Hint`** | Content | Optional | Plain Text / LaTeX | Pedagogical hint revealed on learner request. |
| **`Solution`** | Content | Optional | Plain Text / LaTeX | Full written derivation or solution walkthrough. |
| **`Steps`** | Content | Optional | JSON Array / Newlines | Stepwise breakdown of the derivation graph. |
| **`Explanation`** | Content | Optional | Plain Text / LaTeX | Conceptual explanation or distractor analysis. |
| **`Source`** | Provenance | Optional | String | Source collection title (e.g. `"Official PYQ Corpus"`). |
| **`Exam`** | Provenance | Optional | String | Competitive exam name (e.g. `"RRB ALP"`, `"SSC CGL"`, `"JEE Main"`). |
| **`Year`** | Provenance | Optional | Integer String | Examination year (e.g. `"2024"`). Must be parseable as integer. |
| **`Shift`** | Provenance | Optional | String | Examination shift/session (e.g. `"Shift 1"`, `"Morning"`). |
| **`Paper`** | Provenance | Optional | String | Specific paper or tier (e.g. `"Paper 1 (CBT-1)"`). |
| **`SourceQuestionID`**| Provenance | Optional | String | Authored canonical question identifier (e.g. `"RRB_ALP_2024_S1_Q42"`). |

### 2.3 Structured Validation Errors (`SourceContractError`)
If a note fails contract validation, `SourceQuestion::extract_from_card_fields` emits a structured, actionable error rather than crashing or guessing:
- `MissingRequiredField`: When `Prompt`, `QuestionType`, or `CorrectAnswer` is missing or empty.
- `InvalidQuestionType`: When `QuestionType` is unrecognized (e.g. `"essay"`, `"garbage"`).
- `InvalidDifficulty`: When `Difficulty` is out of bounds (`< 1.0` or `> 5.0`), non-finite (`NaN`, `inf`), or unparseable.
- `MissingMcqOptions`: When an MCQ card has missing `Options` or fewer than 2 non-empty choices.
- `InvalidCorrectAnswer`: When an MCQ answer fails to match any option, or Numerical answer is non-numeric / non-finite (`NaN`, `inf`).
- `InvalidProvenance`: When `Year` fails integer parsing.

### 2.4 Ingestion, Reconciliation & Runtime Translation
```text
Anki Note Fields
  │
  ▼
SourceQuestion::extract_from_card_fields (Strict Validation)
  │
  ▼
SourceQuestion::into_practice_item (Deterministic ID: `pi_src_<guid>`)
  │
  ▼
ProceduralService::reconcile_source_questions (SQL UPSERT into `practice_items`)
  │
  ▼
ProceduralService::resolve_source_target (Mounts Open Canvas Reviewer UI)
```

### 2.5 Learner State Firewall Invariant
- Imported source question records in `practice_items` (`Prompt`, `Options`, `CorrectAnswer`, `Difficulty`, `Provenance`, `QuestionType`) are **100% static and immutable**.
- Learner attempts, mistake reflections, mastery updates, and scheduling states mutate exclusively runtime-owned tables (`practice_attempts`, `skill_states`, `error_events`) in `collection.procedural`.

---

# PART II — PROCEDURAL CONTENT & DECLARATIVE BLUEPRINTS ARCHITECTURE

> [!NOTE]
> This section describes the procedural content architecture. It does not define the canonical StudyLab Source APKG contract.

## 3. Procedural Content Pipeline Overview

StudyLab supports generating dynamic, mathematically sound problem variations from declarative blueprints packaged in standard Anki `.apkg` files.

```text
┌─────────────────────────────────────────────────────────────────────────────────┐
│                      STUDYLAB DECLARATIVE CONTENT PIPELINE                      │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                 │
│   [Curriculum / Topic Taxonomy]                                                 │
│              │                                                                  │
│              ▼                                                                  │
│   [Declarative Blueprint] ──────────► `tools/studylab_content_factory.py`        │
│              │                                                                  │
│              ▼                                                                  │
│   [APKG Compilation]      ──────────► `generate_procedural_apkg.py`              │
│              │                        (Creates .apkg with ProceduralPayload)    │
│              ▼                                                                  │
│   [Anki Package Import]   ──────────► `collection.anki2` (StudyLab Procedural)  │
│              │                                                                  │
│              ▼                                                                  │
│   [Card Review Render]    ──────────► `rslib/src/notetype/render.rs`            │
│              │                        (Intercepts "StudyLab Procedural Anchor")  │
│              ▼                                                                  │
│   [Runtime Generation]    ──────────► `DeclarativeProblemGenerator` (Rust)      │
│              │                        (Samples parameters, evaluates derivation)│
│              ▼                                                                  │
│   [Interactive Webview]   ──────────► `ts/reviewer/procedural.ts`               │
│                                                                                 │
└─────────────────────────────────────────────────────────────────────────────────┘
```

## 4. 3-Tier Content Resolution Hierarchy

When the Rust backend prepares to render a card with the `"StudyLab Procedural Anchor"` note type, `ProceduralService::resolve_procedural_target` (`rslib/procedural/src/service/mod.rs:484-600`) resolves the executable blueprint using a strict **3-Tier Precedence Hierarchy**:

```text
┌───────────────────────────────────────────────────────────────────────────────────────┐
│                          3-TIER CONTENT RESOLUTION HIERARCHY                          │
├───────┬─────────────────────────┬─────────────────────────────────────────────────────┤
│ Tier  │ Strategy                │ Resolution Mechanics & Usage                        │
├───────┼─────────────────────────┼─────────────────────────────────────────────────────┤
│ **1** │ `inline_contract`       │ **Canonical Default for Portable APKGs:**           │
│       │ (Self-Contained Deck)   │ The complete `DeclarativeFamilyContract` is stored  │
│       │                         │ directly within the `ProceduralPayload` JSON field. │
│       │                         │ Requires no pre-existing database records; decks   │
│       │                         │ work immediately upon import on any Anki client.    │
├───────┼─────────────────────────┼─────────────────────────────────────────────────────┤
│ **2** │ `content_ref`           │ **Local Database Reference:**                       │
│       │ (Pre-Ingested Library)  │ Contains a string key (e.g. `"item-math-001"`) that │
│       │                         │ resolves against the local SQLite `practice_items`  │
│       │                         │ table in `procedural.db`. Used for curated corpora. │
├───────┼─────────────────────────┼─────────────────────────────────────────────────────┤
│ **3** │ `proc_schema`           │ **Legacy Hardcoded Engine:**                        │
│       │ (Backward Compatibility)│ Dispatches to compiled Rust procedural generators   │
│       │                         │ (e.g. `"successive_percentage"` in `catalog.rs`).   │
│       │                         │ Retained for backward compatibility and test suites.│
└───────┴─────────────────────────┴─────────────────────────────────────────────────────┘
```

---

## 5. Procedural Note Type & Field Schema

Decks authored for procedural generation conform to the following note model:

### 5.1 Note Model Definition
- **Model Name:** `StudyLab Procedural Anchor` (Checked verbatim in `rslib/src/notetype/render.rs:123`)
- **Card Template Name:** `Procedural Practice Card`
- **Question Format (`qfmt`):** `<div style='padding:20px;font-family:sans-serif;color:#6366f1'>Loading StudyLab Procedural Card...</div>{{ProceduralPayload}}`
- **Answer Format (`afmt`):** `{{ProceduralPayload}}`
- **CSS:**
```css
.card {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
    font-size: 16px;
    color: #1e293b;
    background-color: #f8fafc;
    padding: 0;
    margin: 0;
}
.nightMode .card {
    color: #f1f5f9;
    background-color: #0f172a;
}
```

### 5.2 Procedural Field Schema
Anki notes for procedural generation contain up to 8 defined fields, with `ProceduralPayload` at ordinal position 0:

| Field Name | Ordinal | Required / Optional | Data Type | Purpose & Contents |
|---|---|---|---|---|
| **`ProceduralPayload`** | `0` | **Mandatory** | JSON String | Strict `ProceduralCardAnchor` JSON object containing execution directives, seed modes, and the embedded `inline_contract`. |
| **`TopicTitle`** / **`Topic`** | `1` | Optional | Plain Text | Human-readable topic name (e.g. `"LCM and HCF"`, `"Kinematic Velocity"`). Used for deck browser and breadcrumbs. |
| **`Domain`** / **`Subject`** | `2` | Optional | Plain Text | Academic discipline: `"mathematics"`, `"physics"`, `"chemistry"`, `"reasoning"`. |
| **`Provenance`** / **`ProceduralMetadata`** | `3` | Optional | JSON String | Content origin citation: exam name, year, shift, paper, official question ID. |
| **`Difficulty`** | `4` | Optional | Float String | Target base difficulty rating (`"1.0"` to `"5.0"`). |
| **`LearningObjectType`** | `5` | Optional | String | Modality: `"problem"`, `"quick"`, `"mcq"`, `"stepwise"`, `"concept_check"`, `"strategy_drill"`, `"worked_example"`. |
| **`Front`** | `6` | Fallback | Plain Text / HTML | Fallback question text for non-StudyLab standard Anki installations. |
| **`Back`** | `7` | Fallback | Plain Text / HTML | Fallback solution text for non-StudyLab standard Anki installations. |


---

## 6. `ProceduralPayload` JSON Schema

The `ProceduralPayload` field contains the serialized `ProceduralCardAnchor` struct (`rslib/procedural/src/anchor/mod.rs:27-51`).

### 6.1 Schema Definition
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "ProceduralCardAnchor",
  "type": "object",
  "required": ["proc_schema"],
  "properties": {
    "proc_schema": {
      "type": "string",
      "description": "Unique schema identifier (e.g. 'schema.math.number_system.lcm_hcf.v1')"
    },
    "content_ref": {
      "type": "string",
      "description": "Optional identifier referencing pre-ingested SQLite practice_items record"
    },
    "difficulty_override": {
      "type": "number",
      "minimum": 1.0,
      "maximum": 5.0,
      "description": "Optional difficulty override overriding the archetype default"
    },
    "seed_mode": {
      "type": "object",
      "description": "RNG seeding strategy for deterministic or dynamic generation",
      "oneOf": [
        { "type": "string", "enum": ["random", "daily"] },
        {
          "type": "object",
          "required": ["fixed"],
          "properties": {
            "fixed": { "type": "integer", "minimum": 0 }
          }
        }
      ]
    },
    "custom_params": {
      "type": "object",
      "description": "Optional runtime parameter overrides applied during sampling"
    },
    "inline_contract": {
      "type": "object",
      "description": "Complete self-contained DeclarativeFamilyContract"
    }
  }
}
```

### 6.2 Seed Mode Semantics (`SeedMode`)
- `{"seed_mode": "random"}`: **Dynamic Practice Mode.** Every time the card is reviewed, the Rust engine generates a brand new instance with newly sampled parameters and calculations.
- `{"seed_mode": {"fixed": 42}}`: **Deterministic Verification Mode.** Generates the exact same parameter instance on every render. Used for test suites, benchmark cards, and worked examples.
- `{"seed_mode": "daily"}`: **Daily Variant Mode.** Seeds the RNG with `(CardID + EpochDay)`, ensuring a consistent problem throughout a calendar day that refreshes the next morning.

---

## 7. Declarative Family Contract JSON Schema

The `inline_contract` object conforms to `DeclarativeFamilyContract` (`rslib/procedural/src/problems/contract.rs:70-89`), containing the complete mathematical, pedagogical, and structural specification of the problem family.

### 7.1 Root Contract Specification
```json
{
  "contract": {
    "family_id": "family.math.number_system.lcm_hcf",
    "skill_id": "math.number_system.lcm_hcf",
    "domain": "mathematics",
    "default_schema": "schema.math.number_system.lcm_hcf.v1",
    "capability": "declarative",
    "min_difficulty": 1.0,
    "max_difficulty": 5.0,
    "supported_variants": ["lcm_two_numbers", "hcf_two_numbers"],
    "variant_categories": ["parameter", "structural"],
    "target_latency_model": {
      "1": 25000,
      "2": 35000,
      "3": 45000,
      "4": 60000,
      "5": 75000
    },
    "structural_tags": ["number_system", "arithmetic", "factors"],
    "decision_points": ["prime_factorization", "division_method"],
    "error_categories": ["common_factor_omission", "arithmetic_slip"],
    "prerequisites": [],
    "provenance": {
      "source": "PYQ Corpus",
      "exam": "RRB ALP",
      "year": 2024,
      "shift": 1
    },
    "metadata": {
      "title": "LCM and HCF",
      "category": "Number System"
    }
  },
  "archetypes": [
    {
      "archetype_id": "math.ns.lcm_two_num",
      "difficulty_level": 1,
      "variant_category": "parameter",
      "variant_name": "lcm_two_numbers",
      "object_type": "problem",
      "parameters": [
        {
          "name": "num1",
          "domain": { "type": "integer_range", "min": 6, "max": 24, "step": null, "non_zero": null }
        },
        {
          "name": "num2",
          "domain": { "type": "integer_range", "min": 8, "max": 36, "step": null, "non_zero": null }
        }
      ],
      "constraints": [],
      "prompt_template": "Find the Least Common Multiple (LCM) of \\({num1}\\) and \\({num2}\\).",
      "answer_derivation": {
        "type": "lcm_array",
        "params": ["num1", "num2"]
      },
      "answer_formatted_template": "{answer}",
      "solution_template": "Prime factorize both numbers: {num1} and {num2}. Take highest power of each prime factor. LCM = {answer}.",
      "step_nodes": [
        {
          "id": "step_factorize",
          "step_type": "arithmetic",
          "label": "Prime Factorization",
          "description_template": "Factorize {num1} and {num2}",
          "expected_expression_template": "LCM({num1}, {num2}) = {answer}",
          "alternate_templates": [],
          "hint_principle": "Prime factorization reveals the base components of both numbers.",
          "hint_operation": "Write each number as a product of prime powers.",
          "hint_intermediate": "Examine the common and distinct prime factors."
        }
      ],
      "target_time_ms": 25000
    }
  ]
}
```

---

## 8. Parameter Domain Catalog (16 Types)

`ParameterDomain` (`rslib/procedural/src/problems/contract.rs:188-285`) specifies how variables are sampled, constrained, or derived in dependency order:

```text
┌───────────────────────────────────────────────────────────────────────────────────────┐
│                           PARAMETER DOMAIN TAXONOMY (16 TYPES)                        │
├─────────────────────────┬─────────────────────────────────────────────────────────────┤
│ Category                │ Parameter Domains                                           │
├─────────────────────────┼─────────────────────────────────────────────────────────────┤
│ **Direct Sampling**     │ `integer_range`, `float_range`, `discrete_choice`,          │
│                         │ `permutation_choice`                                        │
├─────────────────────────┼─────────────────────────────────────────────────────────────┤
│ **Arithmetic & Math**   │ `derived_linear`, `derived_product`, `derived_sum`,         │
│                         │ `derived_difference`, `derived_quotient`, `derived_power`,  │
│                         │ `derived_percentage`                                        │
├─────────────────────────┼─────────────────────────────────────────────────────────────┤
│ **Geometric & Algebraic**│ `derived_hypotenuse`, `derived_pythagorean_leg`,            │
│                         │ `derived_signed_string`                                     │
├─────────────────────────┼─────────────────────────────────────────────────────────────┤
│ **Number Theory**       │ `prime_factor_grid`, `coprime_pair`                         │
└─────────────────────────┴─────────────────────────────────────────────────────────────┘
```

### 8.1 Exhaustive Domain Specification

| # | Type Identifier | Required Fields | Mathematical Behavior & Example |
|---|---|---|---|
| 1 | `integer_range` | `min: i64`, `max: i64`, `step: Option<i64>`, `non_zero: Option<bool>` | Uniform integer sampling in $[min, max]$. If `step=2`, selects arithmetic progression $min, min+2, \dots$. |
| 2 | `float_range` | `min: f64`, `max: f64`, `precision: Option<usize>` | Uniform floating point sampling rounded to `precision` decimal places. |
| 3 | `discrete_choice` | `values: Vec<Value>` | Samples 1 item uniformly from an explicit array (e.g. `["m/s", "km/h", "cm/s"]` or MCQ option distractors). |
| 4 | `permutation_choice` | `pool: Vec<String>`, `count: usize` | Samples $k$ distinct elements without replacement from a pool (used in reasoning seating & arrangement problems). |
| 5 | `derived_linear` | `a_param: String`, `x_param: String`, `b_param: String` | Computes $target = a \cdot x + b$ from previously sampled parameters. |
| 6 | `derived_product` | `a_param: String`, `b_param: String` | Computes $target = a \cdot b$. |
| 7 | `derived_sum` | `a_param: String`, `b_param: String` | Computes $target = a + b$. |
| 8 | `derived_difference` | `a_param: String`, `b_param: String` | Computes $target = a - b$. |
| 9 | `derived_quotient` | `a_param: String`, `b_param: String`, `precision: Option<usize>` | Computes $target = a / b$ with safe division-by-zero protection. |
| 10 | `derived_power` | `base_param: String`, `exp_param: String` | Computes $target = \text{base}^{\text{exp}}$. |
| 11 | `derived_percentage` | `base_param: String`, `rate_param: String` | Computes $target = (\text{base} \cdot \text{rate}) / 100.0$. |
| 12 | `derived_signed_string` | `param: String` | Formats signed string for algebraic templates: converts `-5` into `"- 5"` and `3` into `"+ 3"`. |
| 13 | `derived_hypotenuse` | `a_param: String`, `b_param: String` | Computes $c = \sqrt{a^2 + b^2}$. |
| 14 | `derived_pythagorean_leg` | `c_param: String`, `a_param: String` | Computes $b = \sqrt{c^2 - a^2}$ ensuring $c > a$. |
| 15 | `prime_factor_grid` | `primes: Vec<u64>`, `max_power: usize` | Generates composite numbers by sampling prime power products $\prod p_i^{e_i}$. |
| 16 | `coprime_pair` | `min: u64`, `max: u64` | Samples random integer pair $(a, b)$ satisfying $\gcd(a, b) = 1$. |

---

## 9. Constraint Engine & Rejection Sampling

`ConstraintSpec` (`rslib/procedural/src/problems/contract.rs:290-340`) enforces mathematical and pedagogical validity before a generated problem instance is accepted.

### 9.1 Rejection Sampling Loop
During `DeclarativeProblemGenerator::generate()`:
1. Parameters are resolved in dependency order using a deterministic PRNG (`StdRng::seed_from_u64(seed)`).
2. The sampled parameters are validated against all declared `ConstraintSpec` rules.
3. If any constraint fails, the engine mutates the seed and re-samples.
4. **Safety Bound:** A hard ceiling of `MAX_REJECTION_ATTEMPTS = 50` prevents infinite loops on over-constrained blueprints, raising `ProceduralError::ConstraintTimeout` if unsatisfiable.

### 9.2 All 7 Constraint Specifications

| # | Constraint Variant | Fields | Logical Validation Rule |
|---|---|---|---|
| 1 | `not_equal` | `param_a: String`, `param_b: String` | Asserts $param\_a \neq param\_b$. |
| 2 | `non_zero` | `param: String` | Asserts $param \neq 0$. |
| 3 | `divisible` | `dividend_param: String`, `divisor_param: String` | Asserts $dividend \pmod{divisor} = 0$. |
| 4 | `greater_than` | `param_a: String`, `param_b: String` | Asserts $param\_a > param\_b$. |
| 5 | `less_than` | `param_a: String`, `param_b: String` | Asserts $param\_a < param\_b$. |
| 6 | `sum_equals` | `params: Vec<String>`, `target_sum: i64` | Asserts $\sum params_i = target\_sum$. |
| 7 | `predicate` | `expression: String` | Evaluates boolean expression against parameter map. |

---

## 10. Answer Derivation Catalog (24+ Variants across 6 Domains)

`AnswerDerivation` (`rslib/procedural/src/problems/contract.rs:345-480`) deterministically evaluates the canonical correct answer from resolved parameters:

```text
┌───────────────────────────────────────────────────────────────────────────────────────┐
│                          ANSWER DERIVATION CATALOG (24+ TYPES)                        │
├─────────────────────────┬─────────────────────────────────────────────────────────────┤
│ Domain / Category       │ Derivation Variants                                         │
├─────────────────────────┼─────────────────────────────────────────────────────────────┤
│ **Direct & Arithmetic** │ `direct_param`, `direct_string_param`, `product`,           │
│                         │ `quotient`, `percentage_amount`, `remainder`                │
├─────────────────────────┼─────────────────────────────────────────────────────────────┤
│ **Linear Equations**    │ `linear_two_step`, `linear_variables_both_sides`,           │
│                         │ `linear_distributive`, `linear_fractional`                  │
├─────────────────────────┼─────────────────────────────────────────────────────────────┤
│ **Number Theory & Seq** │ `lcm_array`, `gcd_array`, `arithmetic_series_sum`           │
├─────────────────────────┼─────────────────────────────────────────────────────────────┤
│ **Geometry**            │ `pythagoras_hypotenuse`, `pythagoras_leg`,                  │
│                         │ `triangle_area`, `circle_area`                              │
├─────────────────────────┼─────────────────────────────────────────────────────────────┤
│ **Physics Mechanics**   │ `kinematic_velocity`, `kinematic_displacement`,             │
│                         │ `kinematic_stopping_distance`, `kinematic_time`,            │
│                         │ `kinematic_work_energy`, `ideal_gas_law_pressure`,          │
│                         │ `ideal_gas_law_volume`                                      │
├─────────────────────────┼─────────────────────────────────────────────────────────────┤
│ **Chemistry**           │ `stoichiometric_moles_to_mass`,                             │
│                         │ `stoichiometric_mass_to_moles`, `stoichiometric_mole_ratio`,│
│                         │ `stoichiometric_mass_to_mass`, `equilibrium_kc`             │
├─────────────────────────┼─────────────────────────────────────────────────────────────┤
│ **Symbolic Logic**      │ `symbolic_logic_evaluation`                                 │
└─────────────────────────┴─────────────────────────────────────────────────────────────┘
```

### 10.1 Mathematical Formulations

#### 1. Linear Equations
- **`linear_two_step` ($ax + b = c$):** Solves $x = \frac{c - b}{a}$. Requires `a_param`, `b_param`, `c_param`.
- **`linear_variables_both_sides` ($ax + b = cx + d$):** Solves $x = \frac{d - b}{a - c}$. Requires `a_param`, `b_param`, `c_param`, `d_param`.
- **`linear_distributive` ($a(bx + c) = d$):** Solves $x = \frac{\frac{d}{a} - c}{b}$.
- **`linear_fractional` ($\frac{x}{a} + b = c$):** Solves $x = a \cdot (c - b)$.

#### 2. Physics Kinematics & Gas Laws
- **`kinematic_velocity` ($v = u + at$):** Computes final velocity.
- **`kinematic_displacement` ($s = ut + \frac{1}{2}at^2$):** Computes displacement under constant acceleration.
- **`kinematic_stopping_distance` ($d = \frac{u^2}{2a}$):** Computes braking distance.
- **`kinematic_work_energy` ($E_k = \frac{1}{2}mv^2$):** Computes kinetic energy.
- **`ideal_gas_law_pressure` ($P = \frac{nRT}{V}$):** Computes equilibrium gas pressure ($R = 8.314\text{ J/(mol}\cdot\text{K)}$).

#### 3. Chemistry Stoichiometry & Equilibrium
- **`stoichiometric_moles_to_mass` ($m = n \cdot M$):** Converts molar amount to mass in grams.
- **`stoichiometric_mass_to_moles` ($n = \frac{m}{M}$):** Converts mass to moles.
- **`equilibrium_kc` ($K_c = \frac{[C]^c [D]^d}{[A]^a [B]^b}$):** Evaluates chemical equilibrium mass-action constant.

#### 4. Symbolic Logic
- **`symbolic_logic_evaluation`:** Evaluates propositional expressions with boolean operators ($\land$, $\lor$, $\to$, $\leftrightarrow$, $\oplus$, $\neg$).

---

## 11. Step Nodes & 3-Tier Progressive Hint Architecture

Every declarative archetype declares an array of `DeclarativeStepNode` objects (`rslib/procedural/src/problems/contract.rs:485-520`). These nodes power both interactive stepwise solving and progressive hint delivery.

### 11.1 Step Node Schema
```json
{
  "id": "step_isolate_term",
  "step_type": "equation_rearrangement",
  "label": "Isolate Variable Term",
  "description_template": "Subtract {b} from both sides of the equation",
  "expected_expression_template": "{a}x = {c_minus_b}",
  "alternate_templates": ["{a}*x = {c_minus_b}"],
  "hint_principle": "Inverse operations maintain equality across both sides.",
  "hint_operation": "Subtract the constant term {b} from both sides.",
  "hint_intermediate": "The equation reduces to {a}x = {c_minus_b}."
}
```

### 11.2 Complete Step Type Vocabulary (`StepType`)
The engine recognizes 27 distinct step types across the 4 core domains (`rslib/procedural/src/problems/steps/step_graph.rs:7-77`):

```text
General / Math:
  formula_selection, transformation, substitution, arithmetic, simplification,
  equation_rearrangement, comparison, unit_conversion, intermediate_result, final_answer

Physics:
  identify_knowns, select_model, choose_coordinate_system, select_equation,
  physical_sanity_check

Chemistry:
  identify_chemical_species, balance_equation, convert_mass_to_moles,
  apply_stoichiometric_ratio, identify_limiting_reagent,
  construct_equilibrium_expression, chemical_sanity_check

Reasoning:
  identify_schema, select_strategy, build_representation, apply_constraint,
  propagate_constraint, make_inference, create_case, eliminate_case,
  check_contradiction, verify_conclusion
```

### 11.3 3-Tier Progressive Hint Architecture
When the learner clicks **Request Hint** (`H`), hints are revealed in strict pedagogical order:
1. **Tier 1 — Principle (`hint_principle`):** Explains the underlying physical law, mathematical theorem, or strategy without doing the calculation (e.g. *"Recall the third kinematic relation relating velocity, acceleration, and displacement"*).
2. **Tier 2 — Operation (`hint_operation`):** Specifies the exact algebraic operation or formula substitution required (e.g. *"Rearrange $v^2 = u^2 + 2as$ for $s = \frac{v^2 - u^2}{2a}$"*).
3. **Tier 3 — Intermediate (`hint_intermediate`):** Shows the substituted intermediate equation state (e.g. *"Substituting $u=20, v=0, a=-4$ yields $s = \frac{0 - 400}{-8}$"*).

---

## 12. Typography, LaTeX & Formatting Rules

To maintain high visual fidelity and avoid rendering bugs, StudyLab enforces strict string formatting rules:

1. **LaTeX Delimiters:**
   - Inline math must use standard LaTeX delimiters: `\\( ... \\)`.
   - Display/block equations must use: `\\[ ... \\]` or `$$ ... $$`.
   - Dollar signs (`$ ... $`) are converted to `\\( ... \\)` during template rendering.
2. **JSON Escaping Preservation:**
   - Backslashes in LaTeX commands (`\\frac`, `\\sqrt`, `\\times`) must be properly escaped in JSON payloads (`\\\\frac`).
   - The content factory automatically validates that no raw unescaped control characters exist.
3. **XSS Sanitization & HTML Escaping:**
   - Prompts and options are passed through `escape_html` before webview mounting, neutralizing `<script>` and `<img>` injection vectors while preserving MathJax delimiters.
4. **Sign Formatting:**
   - Numerical parameter substitutions must use `derived_signed_string` for linear coefficients to prevent invalid representations like `2x + -5 = 10`.

---

## 13. Procedural APKG Hygiene & Packaging Rules

To eliminate runtime defects, every generated procedural APKG package must strictly adhere to the following 7 hygiene invariants:

1. **Exact Notetype Identity:** The notetype name must be exactly `"StudyLab Procedural Anchor"`. Any alteration breaks backend card interception.
2. **Non-Empty Payload Field:** Field index 0 must be named `ProceduralPayload` and must contain valid, parseable JSON matching `ProceduralCardAnchor`.
3. **Serde Enum Conformance:** All enum strings in JSON payloads (`step_type`, `answer_derivation`, `capability`, `seed_mode`) must match Rust Serde definitions with `snake_case` naming.
4. **Deterministic Validation Gate:** All contracts must pass `DeclarativeFamilyContract::validate()` and `from_json_str_strict()` before compilation into `.apkg`.
5. **No Broken JSON Anchors:** If a payload fails JSON parsing, `ProceduralCardAnchor::from_json_str` safely logs a diagnostic and falls back to standard review rather than crashing Anki.
6. **Zero Duplicate Note Models:** Only 1 unified note model per APKG file.
7. **Single Canonical Full-Universe Artifact:** The official release package is `dist/apkgs/StudyLab_Full_Universe_175.apkg` (SHA-256: `6FC030BED4E572B60BA163B23E0011FF70E91BE479EF77372A9FD4ADAD6F0F1C`).

---

## 14. Procedural Topic Taxonomy (175 Topics)

The StudyLab curriculum spans **175 benchmark topics** across 4 major disciplines:

```text
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                        STUDYLAB FULL UNIVERSE TOPIC TAXONOMY                           │
├─────────────────────────┬──────────────┬───────────────────────────────────────────────┤
│ Discipline              │ Topic Count  │ Core Modalities & Content Focus               │
├─────────────────────────┼──────────────┼───────────────────────────────────────────────┤
│ **Mathematics**         │ 59 Topics    │ Numerical free-answer, Stepwise, WorkedEx     │
│ **Reasoning**           │ 30 Topics    │ 100% Discrete Choices (MCQ), CSP Constraints  │
│ **Physics**             │ 40 Topics    │ Numerical + Units, Stepwise, ConceptCheck     │
│ **Chemistry**           │ 46 Topics    │ Physical (18), Inorganic (14), Organic (14)   │
├─────────────────────────┼──────────────┼───────────────────────────────────────────────┤
│ **Total Universe**      │ 175 Topics   │ Full Cross-Discipline Procedural Coverage     │
└─────────────────────────┴──────────────┴───────────────────────────────────────────────┘
```

### 14.1 Mathematics (59 Topics across 9 Chapters)
1. **Number System & Arithmetic (8 topics):** LCM & HCF, Divisibility Rules, Unit Digit, Prime Factorization, Surds & Indices, Fractions & Decimals, Remainders, Number Properties.
2. **Percentages & Commercial Math (8 topics):** Successive Percentage, Profit & Loss, Marked Price & Discount, Simple Interest, Compound Interest, Installments, Ratio & Proportion, Partnership.
3. **Algebra & Polynomials (8 topics):** Linear Equations 1-Var, Linear Equations 2-Var, Quadratic Equations, Algebraic Identities, Remainder Theorem, Factor Theorem, Progressions (AP/GP), Inequalities.
4. **Time, Work & Distance (6 topics):** Time & Work, Pipes & Cisterns, Speed Time Distance, Relative Speed & Trains, Boats & Streams, Races & Circular Motion.
5. **Averages, Mixtures & Alligations (5 topics):** Simple Averages, Weighted Averages, Alligation Rule, Replacement of Liquids, Age Problems.
6. **Geometry (8 topics):** Lines & Angles, Triangle Properties, Similarity & Congruence, Circles & Tangents, Quadrilaterals, Polygons, Coordinate Geometry, Distance & Section Formula.
7. **Mensuration 2D & 3D (6 topics):** Plane Figures Area/Perimeter, Circles & Sectors, Prism & Cylinder, Cone & Sphere, Frustum, Combined Solids.
8. **Trigonometry (5 topics):** Trigonometric Ratios, Trigonometric Identities, Heights & Distances, Maximum/Minimum Values, Inverse Trigonometry.
9. **Statistics & Modern Math (5 topics):** Mean Median Mode, Standard Deviation, Permutation & Combination, Basic Probability, Set Theory.

### 14.2 Reasoning (30 Topics across 6 Chapters — 100% Discrete MCQ Modality)
1. **Arrangements & Puzzles (6 topics):** Linear Seating (North/South), Circular Seating, Floor & Flat Puzzles, Box Stacking, Scheduling Puzzles, Matrix Grid Puzzles.
2. **Logic & Syllogisms (5 topics):** Syllogisms (Standard), Syllogisms (Possibility/Only a few), Statement & Assumptions, Statement & Conclusions, Course of Action.
3. **Coding-Decoding & Series (5 topics):** Letter Coding, Number Coding, Coded Relations, Alphanumeric Series, Number/Letter Analogy.
4. **Direction & Distance (4 topics):** Cardinal Direction & Distance, Shadow-Based Direction, Coded Direction, Shortest Route Path.
5. **Blood Relations (4 topics):** Direct Relations, Pointing to Photograph, Coded Blood Relations, Family Tree Generation.
6. **Inequalities & Data Sufficiency (6 topics):** Direct Inequalities, Coded Inequalities, Math Data Sufficiency, Logical Data Sufficiency, Input-Output Machine, Venn Diagrams.

### 14.3 Physics (40 Topics across 8 Chapters)
1. **Units, Measurements & Vectors (4 topics):** Dimensional Analysis, Error Analysis, Vector Addition/Dot/Cross, Significant Figures.
2. **Kinematics (5 topics):** Uniform Motion, Uniformly Accelerated Motion, Projectile Motion, Relative Motion 1D, Relative Motion 2D.
3. **Laws of Motion & Friction (5 topics):** Newton's Second Law, Pulley & Wedge Constraints, Static & Kinetic Friction, Inclined Plane Dynamics, Circular Motion Dynamics.
4. **Work, Energy & Power (5 topics):** Work-Energy Theorem, Conservative Forces & Potential Energy, Power, Elastic/Inelastic Collisions, Vertical Circle Motion.
5. **Rotational Motion & Gravitation (6 topics):** Moment of Inertia, Torque & Equilibrium, Angular Momentum Conservation, Rolling Motion, Gravitational Potential & Field, Kepler's Laws & Satellites.
6. **Properties of Matter & Fluids (5 topics):** Elasticity & Hooke's Law, Hydrostatic Pressure & Pascal's Law, Archimedes Principle & Buoyancy, Equation of Continuity & Bernoulli, Viscosity & Surface Tension.
7. **Thermal Physics (5 topics):** Thermal Expansion & Calorimetry, Kinetic Theory of Gases, First Law of Thermodynamics, Heat Engines & Carnot Cycle, Heat Transfer (Conduction/Radiation).
8. **Electrostatics & Current Electricity (5 topics):** Coulomb's Law & Electric Field, Gauss's Law & Flux, Electric Potential & Capacitors, Ohm's Law & Kirchhoff's Rules, Heating Effects & Power.

### 14.4 Chemistry (46 Topics across 3 Disciplines)
1. **Physical Chemistry (18 topics):** Mole Concept & Molar Mass, Empirical & Molecular Formula, Limiting Reagent, Solution Concentration (Molarity/Molality), Ideal Gas Equation, Dalton's Law of Partial Pressure, Raoult's Law & Colligative Properties, First Law & Enthalpy, Hess's Law & Bond Energies, Chemical Equilibrium ($K_c/K_p$), Le Chatelier's Principle, Ionic Equilibrium ($pH$ & Buffers), Solubility Product ($K_{sp}$), Galvanic Cells & Nernst Equation, Faraday's Laws of Electrolysis, Rate Laws & Order of Reaction, Arrhenius Equation & Activation Energy, Radioactivity Kinetics.
2. **Inorganic Chemistry (14 topics):** Quantum Numbers & Electronic Configuration, Periodic Trends (IE/EA/EN), Chemical Bonding & Lewis Structures, VSEPR & Molecular Geometry, Hybridization ($sp, sp^2, sp^3$), Hydrogen Bonding, Coordination Compounds & Nomenclature, Crystal Field Theory ($d$-orbital splitting), Metallurgy & Extraction, $s$-Block Alkali/Alkaline Earth, $p$-Block Group 15/16/17, $d$-Block & $f$-Block Transitions, Qualitative Salt Analysis (Cations), Qualitative Salt Analysis (Anions).
3. **Organic Chemistry (14 topics):** IUPAC Nomenclature, Structural & Stereoisomerism, Inductive & Resonance Effects, Alkanes & Free Radical Halogenation, Alkenes & Markovnikov Addition, Alkynes & Acidic Hydrogen, Electrophilic Aromatic Substitution, Haloalkanes ($S_N1/S_N2$), Alcohols & Lucas Reagent, Aldehydes/Ketones & Nucleophilic Addition, Carboxylic Acids & Derivatives, Amines & Diazonium Salts, Carbohydrates & Amino Acids, Polymerization & Everyday Chemistry.

---

## 15. Procedural Summary & Acceptance Checklist

| Requirement Area | Specification Standard | Verification Status |
|---|---|---|
| **Note Type Identity** | Exactly `"StudyLab Procedural Anchor"` | Verified in `rslib/src/notetype/render.rs:122` |
| **Field 0 Invariant** | `ProceduralPayload` JSON parsed into `ProceduralCardAnchor` | Verified in `rslib/procedural/src/anchor/mod.rs` |
| **Resolution Precedence**| Tier 1 `inline_contract` > Tier 2 `content_ref` > Tier 3 `proc_schema` | Verified in `ProceduralService::resolve_procedural_target` |
| **Parameter Domains** | 16 discrete parameter domains fully documented | Verified in `rslib/procedural/src/problems/contract.rs` |
| **Constraints & Derivation**| 7 constraint types, 24+ derivation formulas, 27 step types | Verified in `step_graph.rs` and `contract.rs` |
| **Progressive Hints** | 3-tier sequence (`hint_principle` $\to$ `hint_operation` $\to$ `hint_intermediate`) | Verified in `studylab_content_factory.py` |
| **Full Universe Coverage**| 175 topics (59 Math, 30 Reasoning, 40 Physics, 46 Chemistry) | Verified in `studylab_content_factory.py` |
| **Hygiene & Packaging** | Validated AST, zero unescaped strings, single canonical `.apkg` | Verified in Phase 36C test suite |

