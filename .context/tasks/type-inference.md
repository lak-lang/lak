# Type Inference

## Overview
Implement type inference to allow variable declarations without type annotations.

### Syntax
```lak
let x = 5                   // Inferred as i64
let name = "hello"          // Inferred as string
let flag = true             // Inferred as bool
let pair = (1, "hello")     // Inferred as (i64, string)
```

### Rules
- Infer type from the right-hand expression.
- Literal types use default types (integers are i64, floating-point values are f64).
- Explicit type annotations take precedence over inference.

### Examples
```lak
let x = 5                   // i64
let y: i64 = 5              // i64 (explicit)
let z = 5 + 10              // i64 (from expression)
```
