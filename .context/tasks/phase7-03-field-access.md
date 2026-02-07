# Field Access

## Phase
Phase 7: User-Defined Types (Medium Priority)

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

## Dependencies
- Struct definition (phase7-01)
- Struct instantiation (phase7-02)
- Modules (phase1-08) - for visibility

## Dependents
- Methods (phase7-04) - self.field access
