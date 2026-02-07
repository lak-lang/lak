# Option<T> in Prelude

## Phase
Phase 6: Algebraic Data Types (Medium Priority)

## Overview
`Option<T>` 型をpreludeに追加する。値の有無を表現するためのenum。

### Definition
```lak
enum Option<T> {
    Some(T)
    None
}
```

### Usage
```lak
let name: Option<string> = Option.None
let value = Option.Some("alice")

match name {
    Some(n) => println(n)           // match内では省略形
    None => println("anonymous")
}
```

### Features
- preludeで自動的に利用可能（importなしで使用可能）
- null安全性を提供
- パターンマッチングで安全に値を取り出す

## Dependencies
- Simple enums (phase6-01)
- Enums with values (phase6-04)
- Generics (phase9-02) - for `<T>`
- `match` expression (phase6-03)

## Dependents
- Collection `get` methods
- Optional function parameters
- Error handling patterns
