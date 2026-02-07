# Float Types

## Phase
Phase 5: Additional Types (Medium Priority)

## Overview
浮動小数点型 `f32` と `f64` を実装する。

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
- 算術: `+`, `-`, `*`, `/`（`%` は未定義）
- 比較: `==`, `!=`, `<`, `>`, `<=`, `>=`
- 単項: `-`

### Default Format
- `3.14`, `-0.5` など

## Dependencies
- Arithmetic operators (completed)
- Comparison operators (phase1-03)
- Unary minus (completed)

## Dependents
- `println` float support
- Mathematical operations
