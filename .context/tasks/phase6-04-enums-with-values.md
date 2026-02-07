# Enums with Values

## Phase
Phase 6: Algebraic Data Types (Medium Priority)

## Overview
値を持つenum（代数的データ型）を実装する。

### Syntax
```lak
enum Shape {
    Circle(f64)              // 単一値
    Rectangle(f64, f64)      // 複数値
    Point                    // 値なし（混合可能）
}
```

### Usage
```lak
let s = Shape.Circle(3.14)
let r = Shape.Rectangle(10.0, 20.0)

match s {
    Circle(r) => println(r)
    Rectangle(w, h) => println(w * h)
    Point => println("point")
}
```

### Pattern Matching
- バリアントに含まれる値を変数にバインド
- 複数パターンで `|` を使う場合、共通のバインディング変数が必要

```lak
match shape {
    Circle(r) | Square(r) => r * r    // 共通の r
    Rectangle(w, h) => w * h
}
```

## Dependencies
- Simple enums (phase6-01)
- `match` expression (phase6-03)
- Tuple-like value storage

## Dependents
- `Option<T>` (phase6-02) - `Some(T)`
- `Result<T, E>` (phase6-05) - `Ok(T)`, `Err(E)`
