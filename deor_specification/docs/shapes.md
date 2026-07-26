<!-- title: Deor Specification -->
<!-- [Deor Specification Index](index.md) -->
<!-- themes: blackboard -->
# Shapes

Deor has no generics — you can't just write "a list of Room" or "a function that takes a Room" inline wherever a type is expected. A `shape` is how you name that kind of type once so you can reuse it: `shape roomList = list of Room` gives "list of Room" a real name you can write in signatures and struct fields from then on. Shapes are the only way to use lists, function types, and named byte buffers in Deor — there is no anonymous inline syntax like `list of Room` outside of a shape declaration.

## Declaration

```deor
shape roomList = list of Room
shape filterFunc = func of Room to bool
```

Shapes are declared at the top level of a file, after imports and before structs. Two kinds exist: list shapes and func shapes. For discriminated variant types, see [Enums](docs/enums.md).

---

## List Shapes

A list shape names a specific element type:

```deor
shape roomList = list of Room
shape intList = list of int
shape rollList = list of Roll
```

```rust
type RoomList = Vec<Room>;
type IntList = Vec<i64>;
type RollList = Vec<Roll>;
```

List shape variables are declared and used like any other typed variable:

```deor
roomList result = empty
roomList rooms = [kitchen, office, bedroom]
int cnt = len(rooms)
```

```rust
let mut result: Vec<Room> = Vec::new();
let rooms: Vec<Room> = vec![kitchen.clone(), office.clone(), bedroom.clone()];
let cnt: i64 = rooms.len() as i64;
```

In function signatures and struct fields, the shape name stands in for the full type:

```deor
fn roomList occupied_rooms(roomList rooms)
    ...

struct House
    string address
    roomList rooms
```

```rust
fn occupied_rooms(rooms: Vec<Room>) -> Vec<Room> { ... }

struct House {
    address: String,
    rooms: Vec<Room>,
}
```

---

## Func Shapes

A func shape names a specific function signature. This is the only way to pass functions as values in Deor — no lambdas, no closures, just named top-level functions matched to a declared signature.

```deor
shape filterFunc = func of Room to bool     # takes Room, returns bool
shape handlerFunc = func of Error           # takes Error, returns nothing
shape supplierFunc = func to bool           # no input, returns bool
```

```rust
type FilterFunc = fn(Room) -> bool;
type HandlerFunc = fn(Error);
type SupplierFunc = fn() -> bool;
```

### Void forms

When input or output is absent, the corresponding `of`/`to` clause is omitted:

| Form | Meaning |
|---|---|
| `func of Room to bool` | takes Room, returns bool |
| `func of Error` | takes Error, returns nothing |
| `func to bool` | no input, returns bool |

### Passing functions as values

A function matching a func shape can be passed as a regular typed argument. No special syntax at the call site — it is just a named variable of the declared type.

```deor
shape filterFunc = func of Room to bool

fn roomList filter(roomList items, filterFunc predicate)
    roomList result = empty
    for item in items
        if predicate(item)
            result at end = item
    return result

fn bool by_name(Room room)
    name in room
    return name is "Kitchen"

filter(rooms, by_name)    # by_name satisfies filterFunc — passed as a regular argument
```

```rust
fn filter(items: Vec<Room>, predicate: fn(Room) -> bool) -> Vec<Room> {
    let mut result: Vec<Room> = Vec::new();
    for item in items {
        if predicate(item.clone()) {
            result.push(item.clone());
        }
    }
    result
}
```

The named-args rule applies: `by_name` must be a named top-level function, not an inline expression. This is how Deor handles what other languages do with lambdas — the function is named, top-level, and typed through the shape.

### Single-param constraint

Func shapes accept at most one input type and one output type. Multi-input shapes are a transpiler error — bundle context into a struct first:

```deor
# Transpiler error — multi-input not allowed
shape badFunc = func of (Room, string) to bool

# Correct — bundle into a struct
struct RoomQuery
    Room room
    string query

shape roomQueryFunc = func of RoomQuery to bool
```

### Func shapes in structs

Func shapes cannot be used as struct fields — structs are pure data. A struct carrying a function would be a closure in disguise, which Deor does not allow.

```deor
# Transpiler error
struct Filter
    roomList items
    filterFunc predicate    # not allowed — func shape as struct field

# Correct — pass the function as a parameter instead
fn roomList apply_filter(roomList items, filterFunc predicate)
    ...
```

---

## Naming Convention

Shape names are camelCase — enforced by the transpiler. By convention (not enforced), the name ends with the shape's kind:

| Kind | Suffix | Examples |
|---|---|---|
| List shapes | `List` | `roomList`, `intList`, `rollList` |
| Func shapes | `Func` | `filterFunc`, `predicateFunc`, `handlerFunc` |

camelCase distinguishes shapes from every other identifier category:
- Primitives and keywords: lowercase (`int`, `list`, `func`, `of`)
- User-defined types: PascalCase (`Room`, `Roll`)
- Variables, functions, fields: snake_case (`room_list`, `filter_func`)
- Enums: PascalCase (`ColorTag`, `StatusTag`) — like structs and validator types

Seeing a camelCase identifier means: this is a shape.

---

## File Ordering

Shapes belong to the structural declarations group — see [Enforced Practices — Ordering](docs/enforced_practices.md#ordering) for the enforced group order. Shapes and structs may reference each other freely (a list shape can name a struct; a struct field can be a shape), so their relative order doesn't matter to the transpiler. See [Best Practices](docs/best_practices.md#order-of-declaration) for the suggested arrangement within that group.

```deor
struct Room
    string name

shape roomList = list of Room    # references Room, declared above
shape filterFunc = func of Room to bool

struct House
    string address
    roomList rooms               # references roomList, declared above
```

---

## Importing Shapes

Shapes are importable like any other top-level declaration. Co-locating a shape with the functions that use it is idiomatic:

```deor
# rooms.deor
shape roomList = list of Room
shape filterFunc = func of Room to bool

fn roomList filter(roomList items, filterFunc predicate)
    ...
```

```deor
# main.deor
import "rooms.deor"
```

---

## Conversion Notes

| Deor | Rust |
|---|---|
| `shape roomList = list of Room` | `type RoomList = Vec<Room>;` |
| `shape filterFunc = func of Room to bool` | `type FilterFunc = fn(Room) -> bool;` |
| `shape handlerFunc = func of Error` | `type HandlerFunc = fn(Error);` |
| `roomList result = empty` | `let mut result: Vec<Room> = Vec::new();` |
| `filter(rooms, by_name)` | `filter(&rooms, by_name)` |

Func shapes use Rust `fn` pointers, not closures — they cannot capture environment, consistent with Deor's no-lambda rule.
