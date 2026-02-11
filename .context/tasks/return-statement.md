# Return Statement

## Overview
`return` 文を実装する。

### Syntax
- `return expr`: 戻り値を持つ関数用
- `return`: void関数での早期リターン用

### Rules
- 戻り値を持つ関数は `return` 文が必須
- void関数は末尾の `return` を省略可能
- void関数での早期リターンは `return`（値なし）を使用

### Examples
```lak
fn abs(x: int) -> int {
    if x < 0 {
        return -x
    }
    return x              // return 必須
}

fn greet_if(flag: bool) -> void {
    if !flag {
        return            // 早期リターン（値なし）
    }
    println("hello")
                          // 末尾の return は省略可能
}
```

