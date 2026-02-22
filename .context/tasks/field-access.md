# Field Access

## Overview
Implement access to struct fields.

### Syntax
```lak
let u = User { name: "alice", age: 30 }
let n = u.name          // "alice"
let a = u.age           // 30
```

### Visibility Rules
- `pub` field: accessible from anywhere.
- private field: accessible only within the same module.

```lak
// user.lak
struct User {
    pub name: string
    age: i64              // private
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
