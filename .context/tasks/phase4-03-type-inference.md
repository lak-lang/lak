# Type Inference

## Phase
Phase 4: Variables Enhancement (Medium Priority)

## Overview
型注釈なしでの変数宣言を可能にする型推論を実装する。

### Syntax
```lak
let x = 5                   // Inferred as int
let name = "hello"          // Inferred as string
let flag = true             // Inferred as bool
let pair = (1, "hello")     // Inferred as (int, string)
```

### Rules
- 右辺の式から型を推論
- リテラルの型はデフォルト型（整数は int、浮動小数点は f64）
- 明示的な型注釈は推論より優先

### Examples
```lak
let x = 5                   // int
let y: i64 = 5              // i64 (explicit)
let z = 5 + 10              // int (from expression)
```

## Dependencies
- Variable declarations (completed)
- Expression type checking (completed)

## Dependents
- Tuple destructuring (future)
- Generic type inference (phase9-02)
