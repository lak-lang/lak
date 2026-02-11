# Simple Enums

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

