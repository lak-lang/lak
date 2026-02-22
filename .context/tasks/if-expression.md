# if as Expression

## Overview
Allow `if` to be used as an expression so it can return a value.

```lak
let max = if a > b { a } else { b }
```

### Rules
- `else` is required.
- Both branches must have the same type.
- The last expression in each block becomes the branch value.

### Examples
```lak
// OK: both branches are i64
let max = if a > b { a } else { b }

// OK: nested expression
let result = if x > 0 {
    if x > 100 { 100 } else { x }
} else {
    0
}

// Compile error: missing else
let value = if condition { 42 }

// Compile error: mismatched types
let value = if condition { 42 } else { "hello" }
```
