---
name: cas-symbolic-validation
description: >-
  Authoritative runbook for Computer Algebra System (CAS) symbolic validation,
  AST expression tree verification, stepwise derivation graph parsing, and cyclic derivation prevention
  in the StudyLab procedural engine inside Anki (naksh-07/anki-maths).
---

# CAS Symbolic Validation & Derivation Integrity Skill

The `cas-symbolic-validation` skill provides the authoritative protocol for validating mathematical expressions, Computer Algebra System (CAS) derivation trees, and stepwise transition graphs in the StudyLab procedural intelligence engine inside Anki (`naksh-07/anki-maths`).

Use this skill when:
- Designing, reviewing, or debugging multi-step math problem generators in `rslib/procedural/src/problems/steps/`.
- Validating mathematical equivalence between user inputs and expected canonical AST representations.
- Verifying that step validation graphs contain zero cycles ($S_a \to S_b \to S_a$) and guarantee finite forward progress.
- Preventing boundary condition failures (division by zero, domain violation, extraneous roots, or unhandled branch cuts).

---

## 1. StudyLab CAS Architecture & AST Models

StudyLab models symbolic mathematics using strongly typed Abstract Syntax Trees in Rust:

```text
┌──────────────────────────────────────────────────────────────────────────────────┐
│                           STUDYLAB CAS AST HIERARCHY                             │
├──────────────────────────────────────────────────────────────────────────────────┤
│ Expression (Root Node)                                                           │
│   ├── BinaryOp(Add | Sub | Mul | Div | Pow)                                      │
│   ├── UnaryOp(Neg | Abs | Sqrt | Ln | Sin | Cos | Tan)                           │
│   ├── Variable(Symbol, DomainConstraint)                                        │
│   ├── Constant(Rational | Integer | FloatExact)                                  │
│   └── FunctionCall(Name, Vec<Expression>)                                        │
└──────────────────────────────────────────────────────────────────────────────────┘
```

### Key Source Locations
- `rslib/procedural/src/problems/steps/`: Step validation rules, substitution engines, and expression equivalence logic.
- `rslib/procedural/src/problems/contract.rs`: Blueprint contracts defining required steps, distractors, and solution formulas.
- `ts/reviewer/`: Svelte KaTeX rendering layer converting AST tokens to visual math.

---

## 2. StepValidator Graph & Cyclic Derivation Prevention

In stepwise math solving, every problem is represented as a Directed Acyclic Graph (DAG) of valid derivations:

1. **DAG Invariant**: The transition graph from initial problem state $S_0$ to terminal solved state $S_{terminal}$ must be strictly acyclic.
2. **Cycle Detection Protocol**:
   - Trace all outgoing transformation edges from step node $S_i$.
   - Maintain a visited hashset of normalized algebraic forms.
   - If $hash(normalize(S_{next})) \in Visited$, reject the generator blueprint as `CYCLIC_DERIVATION_ERROR`.
3. **Dead-End Elimination**: Every non-terminal step must have at least one valid progression path to $S_{terminal}$.

---

## 3. Symbolic Equivalence & Algebraic Verification Protocol

When checking whether a student input $E_{input}$ is equivalent to expected step $E_{expected}$:

### Step 1: Structural Equivalence Check
First attempt canonical syntactic normalization:
- Flatten associative additions and multiplications: $(a + b) + c \to a + b + c$.
- Order terms deterministically (e.g., degree-descending, lexicographical by variable name).
- Combine like constant terms: $2x + 3x \to 5x$.

### Step 2: Semantic Equivalence Check (Zero-Difference Test)
If syntactic forms differ, compute the symbolic difference:
$$\Delta = E_{input} - E_{expected}$$
Simplify $\Delta$ using algebraic expansion, common denominator reduction, and trigonometric identities:
- If $\Delta \equiv 0$, the input is algebraically valid.
- If $\Delta \neq 0$, classify the error into a cognitive mistake category (e.g., sign reversal, missed factor, dropped constant).

### Step 3: Sequential Thinking Integration
For multi-phase derivations (e.g., integration by parts, matrix inverses):
- Use the `sequential-thinking` MCP tool to log intermediate states explicitly.
- Verify that no intermediate step introduces an undefined operation ($1/0$, $\ln(0)$, $\sqrt{-1}$ without complex field active).

---

## 4. Boundary Conditions & Numerical Integrity

1. **Division-by-Zero Guard**:
   - Any fraction $\frac{P(x)}{Q(x)}$ must have non-zero denominator conditions enforced: $Q(x) \neq 0$.
   - Generated random parameters must be clamped away from roots of $Q(x)$.
2. **Branch Cuts & Domains**:
   - Logarithms $\ln(u)$ require $u > 0$.
   - Square roots $\sqrt{u}$ in real domains require $u \ge 0$.
   - Trigonometric domains: $\tan(x)$ requires $x \neq \frac{\pi}{2} + k\pi$.
3. **Float Precision Guarantee**:
   - Numerical evaluators must avoid naive IEEE-754 direct equality. Always use rational numbers or epsilon-bounded comparisons ($\epsilon = 10^{-7}$).

---

## 5. Verification Commands

Run targeted Rust unit tests for problems and step validation:

```bash
# Fast unit check of problem step validation logic
cargo test -p procedural --lib problems

# Full procedural crate test suite
cargo test -p procedural --lib
```
