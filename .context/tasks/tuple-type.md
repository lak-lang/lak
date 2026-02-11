# Tuple Type

## Overview
タプル型を実装する。複数の値をグループ化する。

### Syntax
```lak
let pair: (int, string) = (1, "hello")
let triple = (1, "a", true)            // Inferred as (int, string, bool)
```

### Element Access
`.0`, `.1` などでアクセス。

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

