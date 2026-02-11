# List Type

## Overview
Implement the list type.

### Syntax
```lak
let numbers = [1, 2, 3]                // Inferred as List<int>
let names: List<string> = []           // Empty list requires type annotation
```

### Element Access
No index syntax (`[]`) for direct access. Use methods.

```lak
let numbers = [1, 2, 3]
let first = numbers.get(0)             // Option<int>
let sub = numbers.slice(0, 2)          // [1, 2]
```

### Mutability
Declare with `mut` to modify contents.

```lak
let numbers = [1, 2, 3]
numbers.push(4)                        // Compile error: immutable

let mut items = [1, 2, 3]
items.push(4)                          // OK
```

### Methods
- `get(index: int) -> Option<T>`
- `slice(start: int, end: int) -> List<T>`
- `push(item: T)` (requires mut)
- `len() -> int`
