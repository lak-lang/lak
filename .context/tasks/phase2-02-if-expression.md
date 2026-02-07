# if as Expression

## Phase
Phase 2: Control Flow (High Priority)

## Overview
`if` を式として使用できるようにする。値を返すことができる。

```lak
let max = if a > b { a } else { b }
```

### Rules
- `else` が必須
- 両方のブランチの型が一致する必要がある
- ブロック内の最後の式がブランチの値になる

### Examples
```lak
// OK: 両方のブランチが int
let max = if a > b { a } else { b }

// OK: ネスト可能
let result = if x > 0 {
    if x > 100 { 100 } else { x }
} else {
    0
}

// Compile error: else がない
let value = if condition { 42 }

// Compile error: 型が一致しない
let value = if condition { 42 } else { "hello" }
```

## Dependencies
- `if`/`else` statement (phase2-01)
- Type checking for branch matching

## Dependents
- `match` expression (phase6-03) - similar expression semantics
