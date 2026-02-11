# Map Type

## Overview
Implement the map type to hold key-value pairs.

### Syntax
```lak
let ages = {"alice": 30, "bob": 25}    // Inferred as Map<string, int>
let empty: Map<string, int> = {}       // Empty map requires type annotation
```

### Key Type Restriction
Keys must be primitive types only:
- `int`, `uint`, `i8`-`i64`, `u8`-`u64`
- `string`
- `bool`

### Element Access
No index syntax (`[]`) for direct access. Use methods.

```lak
let ages = {"alice": 30}
let age = ages.get("alice")            // Option<int>
```

### Mutability
Declare with `mut` to modify contents.

```lak
let ages = {"alice": 30}
ages.set("bob", 25)                    // Compile error: immutable

let mut ages2 = {"alice": 30}
ages2.set("bob", 25)                   // OK
```

### Methods
- `get(key: K) -> Option<V>`
- `set(key: K, value: V)` (requires mut)
- `remove(key: K)` (requires mut)
- `contains(key: K) -> bool`
- `len() -> int`
