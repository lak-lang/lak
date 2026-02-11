# Non-void Return Types

## Overview
Implement functions that return non-void types.

### Syntax
```lak
fn add(a: int, b: int) -> int {
    return a + b
}

fn is_positive(x: int) -> bool {
    return x > 0
}
```

### Features
- Any type can be returned.
- Return values with `return expr`.
- Enforce return type checking.

### Return Value Reception
When calling a function with a return value, the return value must be received.
Unused return values can be discarded with `_`.

```lak
let result = add(1, 2)     // OK
add(1, 2)                   // Compile error: return value is not received
let _ = add(1, 2)           // OK: discard
```
