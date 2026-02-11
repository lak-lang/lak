# Struct Instantiation

## Overview
構造体のインスタンス作成を実装する。

### Syntax
```lak
let u = User { name: "alice", age: 30 }
```

### Rules
- 全てのフィールドを指定する必要がある（デフォルト値なし）
- privateフィールドを持つ構造体は同一モジュール内でのみリテラル作成可能

```lak
let u = User { name: "alice", age: 30 }  // OK (same module)

let u2 = User { name: "bob" }            // Compile error: age not specified
```

### Factory Pattern
外部からのインスタンス作成を許可するにはファクトリ関数を公開する。

```lak
// user.lak
pub fn new_user(name: string, age: int) -> User {
    return User { name: name, age: age }
}

// main.lak
import "./user"
let u = user.new_user("alice", 30)
```

