# Float Types

## Overview
Implement floating-point types `f32` and `f64`.

### Types
| Type | Description |
|------|-------------|
| `f32` | 32-bit floating point |
| `f64` | 64-bit floating point |

### Literals
```lak
let x = 3.14            // f64 (default)
let y: f32 = 3.14       // f32 (explicit)
let z = -0.5            // f64
```

### Operators
- Arithmetic: `+`, `-`, `*`, `/` (`%` is undefined)
- Comparison: `==`, `!=`, `<`, `>`, `<=`, `>=`
- Unary: `-`

### Numeric Mixing Rules
- `f32` and `f64` in the same arithmetic/comparison expression are promoted to `f64`.
- Integer and float mixed arithmetic/comparison is a compile error unless an explicit cast is provided.
- Implicit narrowing conversion is not allowed.

### Default Format
- `3.14`, `-0.5`, etc.
