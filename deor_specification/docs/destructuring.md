<!-- title: Deor Specification -->
<!-- [Deor Specification Index](index.md) -->
<!-- themes: blackboard -->
# Destructuring
`in` extracts one or more fields from a struct into the current scope. This is the only way to access struct fields — there is no dot syntax in source, so every field a block of code touches is named up front in one place, instead of scattered across dot-chains wherever they happen to get used.

Parentheses are always required, even for a single field. Deor calls this **bagging**: items come out of (or go into) the struct under their original field name — there is no aliasing.

## Single Field
Deor:
```deor
(area) in room
```
Rust:
```rust
let area = room.area.clone();
```

## Multiple Fields
Deor:
```deor
(area, name) in room
```
Rust:
```rust
let area = room.area.clone();
let name = room.name.clone();
```

Each extracted field becomes its own `let field = src.field.clone();` binding — not a Rust pattern destructure.

---

## Partial Extraction
You can extract a subset of a struct's fields. Any combination is valid — the struct may have more fields than you extract.

```deor
struct Room
    Squarefeet area
    string name
    bool occupied

(name) in room          # valid — ignores area and occupied
(area, occupied) in room  # valid — any subset, any order
```

---

## Shadowing
If a name being extracted already exists in scope, the new binding silently shadows it — the same block-scoped shadowing rules apply as anywhere else in Deor. See [Enforced Practices — Variable Shadowing](docs/enforced_practices.md#variable-shadowing) for the full mechanics.

```deor
name as "Alice"
(name) in employee    # name now refers to employee.name — "Alice" is gone
```

Use this deliberately to "update" a name after processing, or avoid it by choosing distinct names.

A **further thought** on shadowing: Deor does not shy away from ```macros```, as a result shadowing is necessary to prevent collision, Deor also 
accepts its Rust lineage and since Rust supports this it seemed like an obvious feature to keep. This also aligns with the default import philosophy 
which is to silently swallow same-imports as they are already imported. If you don't want to silently swallow a duplicate definition, the best approach 
is to use a ```const```, as they will not allow shadowing.

---

## Move Destructuring
`move (f1, f2) in source` extracts fields without cloning, consuming `source` for any field moved out — see [Move — Destructuring](docs/move.md#destructuring) for the full example and rules.