# if/else Statement

## Phase
Phase 2: Control Flow (High Priority)

## Overview
`if`/`else` 文を実装する。

```lak
if condition {
    // ...
} else if condition2 {
    // ...
} else {
    // ...
}
```

### Features
- 基本的な `if condition { ... }`
- `else` ブランチ
- `else if` チェーン

### Condition
- 条件式は `bool` 型でなければならない
- 比較演算子や論理演算子の結果を使用

## Dependencies
- `bool` type (completed)
- Comparison operators (phase1-03) - recommended
- Logical operators (phase1-04) - recommended

## Dependents
- `if` as expression (phase2-02)
- `return` statement (phase2-03) - can use early return with if
- `while` loop (phase2-04) - similar control flow structure
