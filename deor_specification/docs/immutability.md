<!-- title: Deor Specification -->
<!-- [Deor Specification Index](index.md) -->
<!-- themes: blackboard -->
# Immutability & Record Update

## Mutability Rules

A struct handed to a function or dropped into a list shouldn't change out from under the code still holding it, so structs are immutable — the only way to get a "changed" version is `with`, which builds a new one, or recomposing it entirely. Lists and primitives don't carry that risk (growing a list or bumping a counter isn't rewriting data someone else is relying on), so they stay mutable.

| Kind | Mutability | Notes |
|---|---|---|
| Primitives (`int`, `float`, `bool`, ...) | Mutable value types | `val = val + 1` always legal |
| `struct` | **Immutable** | No field-assignment syntax exists. The only way to get a "changed" struct is `with` |
| `list` | Mutable container | `at end =`/`remove at` for growable lists; elements may themselves be immutable structs |

---

## Equality

`is` is **always structural** — Deor derives `PartialEq` on all structs, so equality compares field values, not identity. In the generated Rust, `is` maps to `==`.

---

## Record Update (`with`)

`with` produces a new struct with one or more fields overridden. The original is unchanged. Each field name must already exist as a variable in scope — the same rule as struct literals.

Works with both binding forms — `as` (type inferred from the source struct) and a typed declaration (`Type name = source with (...)`, as seen in [Updating a Struct Inside a List](collections.md#updating-a-struct-inside-a-list)):

- Single field: `new_room as room with (area)` — parens always required, even for one field
- Multiple fields: `new_room as room with (area, name)`

The transpiler enforces the parens: `with` not immediately followed by `(` is a validation error, for both forms.

Deor:
```deor
Squarefeet area = 2
new_room as room with (area)

area = 20
name as "Bigger Office"
bigger_office as office with (area, name)
```

Rust:
```rust
let mut area: Option<Squarefeet> = Squarefeet::new(2);
let new_room = Room { area, ..room.clone() };

area = Squarefeet::new(20);
let name = "Bigger Office".to_string();
let bigger_office = Room { area, name, ..office.clone() };
```

**Conversion notes:** near 1:1 with Rust's built-in functional record update (`..` spread) syntax, except the spread source is cloned (`..room.clone()`, not `..room`) — every Deor struct derives `Clone`, so `with` never consumes the source; `room`/`office` are still fully usable after the update. Overridden fields that are validator types route through their constructor like any other assignment to that type. The `with` pattern mirrors `in` destructuring: `in` pulls fields out of a struct, `with` pushes variables into one.
