# Non-void Return Types

## Overview
void以外の戻り値型を持つ関数を実装する。

### Syntax
```lak
fn add(a: int, b: int) -> int {
    return a + b
}

fn is_positive(x: int) -> bool {
    return x > 0
}
```

### Features
- 任意の型を戻り値として返せる
- `return expr` で値を返す
- 戻り値の型チェック

### Return Value Reception
戻り値を持つ関数を呼び出した場合、戻り値を受け取る必要がある。
不要な戻り値は `_` で破棄できる。

```lak
let result = add(1, 2)     // OK
add(1, 2)                   // Compile error: 戻り値が受け取られていない
let _ = add(1, 2)           // OK: 破棄
```

