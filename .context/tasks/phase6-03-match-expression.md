# match Expression

## Phase
Phase 6: Algebraic Data Types (Medium Priority)

## Overview
`match` 式を実装する。網羅性チェック付きのパターンマッチング。

### Syntax
```lak
match expr {
    pattern1 => expr1
    pattern2 => expr2
    _ => default
}
```

### Features
- 値のバインディング: `Some(x) =>`
- 網羅性チェック: enumは全バリアントを処理するか `_` が必要
- ワイルドカード: `_`
- 複数パターン: `|` で区切る
- 式としての使用: 全armが同じ型を返す

### Examples
```lak
// 式として使用
let area = match shape {
    Circle(r) => 3.14 * r * r
    Rectangle(w, h) => w * h
    Point => 0.0
}

// ワイルドカード
match value {
    1 => "one"
    2 => "two"
    _ => "other"
}

// 複数パターン
match color {
    Red | Green => "warm"
    Blue => "cool"
}
```

## Dependencies
- Simple enums (phase6-01)
- `if`/`else` (phase2-01) - similar expression semantics
- Comparison operators (phase1-03) - for literal patterns

## Dependents
- `Option<T>` handling (phase6-02)
- `Result<T, E>` handling (phase6-05)
- Enum destructuring
