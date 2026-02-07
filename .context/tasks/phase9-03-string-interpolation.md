# String Interpolation

## Phase
Phase 9: Advanced Features (Lower Priority)

## Overview
文字列補間を実装する。`${}` 構文で式を埋め込む。

### Syntax
```lak
let name = "lak"
let version = 1
println("${name} v${version}")    // "lak v1"
```

### Features
- 変数の埋め込み: `"${name}"`
- 式の埋め込み: `"${a + b}"`
- ネスト可能

### Default Formatting
全ての型にデフォルトフォーマットがある:
- 整数: `42`, `-10`
- 浮動小数点: `3.14`, `-0.5`
- 真偽値: `true`, `false`
- 文字列: `hello`（引用符なし）
- 構造体: `User { name: "alice", age: 30 }`
- enum: `Option.Some(42)`, `Color.Red`

### Stringer Priority
`Stringer` インターフェースを実装している型は `to_string()` を優先使用。

```lak
struct User {
    pub name: string

    pub fn to_string(self) -> string {
        return "User: " + self.name
    }
}

let u = User { name: "alice" }
println("${u}")    // "User: alice" (uses to_string())
```

## Dependencies
- String type (completed)
- Interfaces (phase9-01) - for Stringer
- Expression evaluation

## Dependents
- Formatted output
- Debugging
