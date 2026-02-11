# Methods

## Overview
構造体のメソッドを実装する。

### Syntax
メソッドは構造体定義の内部に定義する。最初のパラメータは `self`。

```lak
struct User {
    pub name: string
    age: int

    pub fn greet(self) -> string {
        return "Hello, " + self.name
    }

    pub fn is_adult(self) -> bool {
        return self.age >= 18
    }

    fn set_name(mut self, name: string) {
        self.name = name
    }
}
```

### self Parameter
- `self`: immutable（読み取りのみ）
- `mut self`: mutable（変更可能）

### Visibility
- メソッドはデフォルトでprivate
- `pub` キーワードで公開

### Calling Methods
```lak
let u = User { name: "alice", age: 30 }
let greeting = u.greet()        // "Hello, alice"
let adult = u.is_adult()        // true
```

### mut self Restriction
`mut self` メソッドは `mut` で宣言された変数に対してのみ呼び出し可能。

```lak
let u = User { name: "alice", age: 30 }
u.set_name("bob")               // Compile error: u is immutable

let mut u2 = User { name: "alice", age: 30 }
u2.set_name("bob")              // OK
```

