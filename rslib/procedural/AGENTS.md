# Directory-Scoped Rules: Procedural Rust Engine (`rslib/procedural/`)

**Authority:** Level 1 Subsystem Architectural Invariant  
**Target:** `rslib/procedural/**`  
**Governing Subagents:** `cas-math-specialist`, `persistence-guard`, `implementer`, `reviewer-verifier`

---

## 1. Strict Parameterized SQL Invariant (Zero SQL Injection)

1. **100% Prepared Statements**: Every SQL query executed against `<collection>.procedural` MUST use numbered positional parameters (`?1, ?2, ...`):
   ```rust
   // REQUIRED PATTERN:
   conn.prepare("SELECT skill_id, mastery FROM skill_state WHERE user_id = ?1 AND skill_id = ?2")?
   ```
2. **ZERO String Formatting in SQL**: NEVER use `format!`, `concat!`, or string interpolation to build SQL statements:
   ```rust
   // FORBIDDEN (VIOLATION):
   let sql = format!("SELECT * FROM attempts WHERE id = '{}'", attempt_id);
   ```
3. **Identifier Safety**: If table names or column names must be dynamic, they must be validated against a strict compile-time whitelist enum.

---

## 2. Database Schema & Migration Invariants

1. **Complete Database Isolation**: The procedural engine must interact solely with `<collection_path>.procedural`. NEVER touch or issue DDL against `collection.anki2` or `collection.anki21`.
2. **Idempotent Migrations**: All migration files in `rslib/procedural/src/storage/migration/` must check the current PRAGMA `user_version`, execute within an explicit transaction, and increment `user_version` atomically.
3. **No Destructive Schema Changes**: Never issue `DROP TABLE` or `DROP COLUMN` on production tables across migrations v1 through v5.

---

## 3. Mathematical & Symbolic Rigor

1. **Closed-Form Step Validation**: Mathematical derivations in `src/problems/steps/` must be deterministic and acyclic.
2. **Boundary Clamping**: Random parameter generation must explicitly guard against division by zero ($x \neq 0$), square roots of negatives in real domains, and logarithmic zero arguments.
3. **Rational & Epsilon Float Checks**: Numerical equality checks must use rational fractions or epsilon tolerance ($\le 10^{-7}$).

---

## 4. Verification Requirements

Before submitting any modification in this directory:
```bash
cargo check -p procedural
cargo test -p procedural --lib
```
All 146+ procedural unit tests must pass with zero warnings.
