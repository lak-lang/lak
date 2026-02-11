# Struct Definition

## Overview
構造体の定義を実装する。

### Syntax
```lak
struct User {
    pub name: string
    age: int          // Private (default)
}
```

### Visibility
- フィールドはデフォルトでprivate
- `pub` キーワードで公開

### Rules
- フィールドは名前と型を持つ
- 同じフィールド名は使用できない
- 構造体自体も `pub` で公開可能

```lak
pub struct User {
    pub name: string
    age: int
}
```

