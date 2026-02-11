# Type Inference

## Overview
Implement type inference to allow variable declarations without type annotations.

### Syntax
```lak
let x = 5                   // Inferred as int
let name = "hello"          // Inferred as string
let flag = true             // Inferred as bool
let pair = (1, "hello")     // Inferred as (int, string)
```

### Rules
- Infer type from the right-hand expression.
- Literal types use default types (integers are int, floating-point values are f64).
- Explicit type annotations take precedence over inference.

### Examples
```lak
let x = 5                   // int
let y: i64 = 5              // i64 (explicit)
let z = 5 + 10              // int (from expression)
```
