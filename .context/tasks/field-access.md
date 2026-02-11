# Field Access

## Overview
構造体のフィールドへのアクセスを実装する。

### Syntax
```lak
let u = User { name: "alice", age: 30 }
let n = u.name          // "alice"
let a = u.age           // 30
```

### Visibility Rules
- `pub` フィールド: どこからでもアクセス可能
- private フィールド: 同一モジュール内からのみアクセス可能

```lak
// user.lak
struct User {
    pub name: string
    age: int              // private
}

// main.lak
import "./user"
let u = user.new_user("alice", 30)
println(u.name)          // OK: pub field
println(u.age)           // Compile error: private field
```

### Nested Access
```lak
let team = Team { leader: User { name: "alice", age: 30 } }
let name = team.leader.name
```

