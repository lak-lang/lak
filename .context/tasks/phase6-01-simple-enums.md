# Simple Enums

## Phase
Phase 6: Algebraic Data Types (Medium Priority)

## Overview
値を持たないシンプルなenumを実装する。

### Syntax
```lak
enum Color {
    Red
    Green
    Blue
}
```

### Usage
```lak
let c: Color = Color.Red
```

### Variant Resolution
- `match` 内では省略形が使用可能
- その他の場所では完全修飾が必要

```lak
// match 内: 省略形OK
match color {
    Red => "red"
    Green => "green"
    Blue => "blue"
}

// その他: 完全修飾必須
let color = Color.Red
```

## Dependencies
- Type system basics (completed)
- `match` expression (phase6-03) - for pattern matching

## Dependents
- Enums with values (phase6-04)
- `Option<T>` (phase6-02)
- `Result<T, E>` (phase6-05)
