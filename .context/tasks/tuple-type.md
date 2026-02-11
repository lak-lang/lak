# Tuple Type

## Overview
Implement tuple types for grouping multiple values.

### Syntax
```lak
let pair: (int, string) = (1, "hello")
let triple = (1, "a", true)            // Inferred as (int, string, bool)
```

### Element Access
Access elements with `.0`, `.1`, etc.

```lak
let pair = (1, "hello")
let n = pair.0          // 1
let s = pair.1          // "hello"

// Nested access
let nested = ((1, 2), "test")
let a = nested.0.1      // 2
```

### Destructuring
```lak
let pair = (1, "hello")
let x, y = pair         // x = 1, y = "hello"
```
