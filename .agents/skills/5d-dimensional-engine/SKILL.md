---
name: 5d-dimensional-engine
description: >-
  Authoritative rulesheet and protocol for 5D physical dimensional analysis
  ([M]^a [L]^b [T]^c [I]^d [Theta]^e), unit conversions, dimensional homogeneity,
  and physical validation in the StudyLab procedural engine inside Anki (naksh-07/anki-maths).
---

# 5D Physical Dimensional Engine & Unit Analysis Skill

The `5d-dimensional-engine` skill provides the authoritative protocol for verifying physical units, dimensional homogeneity, and unit conversions in the StudyLab procedural engine inside Anki (`naksh-07/anki-maths`).

Use this skill when:
- Implementing or debugging physics problems in `rslib/procedural/src/units/` or domain problem generators.
- Validating that student numerical entries provide compatible physical dimensions (e.g., Joules vs Watts).
- Checking dimensional homogeneity in derived equations ($[A] + [B]$ requires $[A] = [B]$).
- Converting compound metric units ($km/h \leftrightarrow m/s$, $N \cdot m \leftrightarrow J$, $Pa \leftrightarrow N/m^2$).

---

## 1. The 5D Physical Dimension Vector

StudyLab represents all physical quantities as an exact 5-dimensional integer exponent tuple:

$$[Q] = [M]^a [L]^b [T]^c [I]^d [\Theta]^e$$

Where:
- **$M$**: Mass (Base unit: kilogram, $\text{kg}$)
- **$L$**: Length (Base unit: meter, $\text{m}$)
- **$T$**: Time (Base unit: second, $\text{s}$)
- **$I$**: Electric Current (Base unit: ampere, $\text{A}$)
- **$\Theta$**: Thermodynamic Temperature (Base unit: kelvin, $\text{K}$)

```text
┌──────────────────────────────────────────────────────────────────────────────────┐
│                         CANONICAL 5D DIMENSION TABLE                             │
├──────────────────────┬─────────────┬───────────┬───┬───┬────┬───┬───┬────────────┤
│ Physical Quantity    │ SI Unit     │ Symbol    │ M │ L │  T │ I │ Θ │ Vector     │
├──────────────────────┼─────────────┼───────────┼───┼───┼────┼───┼───┼────────────┤
│ Dimensionless / Rad  │ (pure)      │ 1         │ 0 │ 0 │  0 │ 0 │ 0 │ (0,0,0,0,0)│
│ Velocity             │ m / s       │ v         │ 0 │ 1 │ -1 │ 0 │ 0 │ (0,1,-1,0,0│
│ Acceleration         │ m / s²      │ a         │ 0 │ 1 │ -2 │ 0 │ 0 │ (0,1,-2,0,0│
│ Force                │ Newton (N)  │ kg·m/s²   │ 1 │ 1 │ -2 │ 0 │ 0 │ (1,1,-2,0,0│
│ Energy / Work        │ Joule (J)   │ N·m       │ 1 │ 2 │ -2 │ 0 │ 0 │ (1,2,-2,0,0│
│ Power                │ Watt (W)    │ J/s       │ 1 │ 2 │ -3 │ 0 │ 0 │ (1,2,-3,0,0│
│ Pressure             │ Pascal (Pa) │ N/m²      │ 1 │-1 │ -2 │ 0 │ 0 │ (1,-1,-2,0,│
│ Electric Charge      │ Coulomb (C) │ A·s       │ 0 │ 0 │  1 │ 1 │ 0 │ (0,0,1,1,0)│
│ Potential Difference │ Volt (V)    │ W/A       │ 1 │ 2 │ -3 │-1 │ 0 │ (1,2,-3,-1,│
└──────────────────────┴─────────────┴───────────┴───┴───┴────┴───┴───┴────────────┘
```

---

## 2. Core Dimensional Rules

### Rule 1: Dimensional Homogeneity in Addition and Subtraction
For any physical equation $X = Y + Z$:
$$[X] = [Y] = [Z]$$
Addition or subtraction of quantities with disparate dimension vectors (e.g., $5\text{ m} + 3\text{ s}$) is strictly illegal and must trigger a compile-time or generation error.

### Rule 2: Multiplicative Dimensional Composition
For $X = Y \cdot Z$ and $W = Y / Z$:
$$[X] = [Y] + [Z] \quad (\text{vector addition of exponents})$$
$$[W] = [Y] - [Z] \quad (\text{vector subtraction of exponents})$$

### Rule 3: Dimensionless Transcendental Arguments
The arguments of transcendental functions ($\sin, \cos, \tan, \exp, \ln$) must be strictly **dimensionless** ($[0, 0, 0, 0, 0]$):
- $\sin(\omega t)$ requires $[\omega] = [T]^{-1}$ so that $[\omega t] = [T]^0$.

---

## 3. Unit Conversion & Prefix Scaling Protocol

StudyLab computes value conversions through base SI multipliers:
$$\text{Value}_{\text{SI}} = \text{Value}_{\text{input}} \times \text{Multiplier}$$

### Metric Prefixes
- $\text{nano- (n)} = 10^{-9}$
- $\text{micro- (\mu)} = 10^{-6}$
- $\text{milli- (m)} = 10^{-3}$
- $\text{centi- (c)} = 10^{-2}$
- $\text{kilo- (k)} = 10^{3}$
- $\text{mega- (M)} = 10^{6}$
- $\text{giga- (G)} = 10^{9}$

### Conversion Verification Procedure
1. Verify dimension vector match: $[Unit_{from}] == [Unit_{to}]$.
2. If vectors do not match, emit diagnostic error: `DIMENSION_MISMATCH_ERROR`.
3. Compute scaling ratio $R = \text{Scale}_{from} / \text{Scale}_{to}$.
4. Transform numerical value: $V_{to} = V_{from} \times R$.

---

## 4. Verification Commands

Run unit tests covering 5D dimensional analysis and unit parsing:

```bash
# Fast unit check of physical unit operations
cargo test -p procedural --lib units

# Verify unit parsing and conversion routines
cargo test -p procedural --lib tests::unit
```
