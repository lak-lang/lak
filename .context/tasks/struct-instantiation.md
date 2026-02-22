# Struct Instantiation

## Overview
Implement struct instance creation.

### Syntax
```lak
let u = User { name: "alice", age: 30 }
```

### Rules
- All fields must be specified (no default values).
- Struct literals with private fields can be created only within the same module.

```lak
let u = User { name: "alice", age: 30 }  // OK (same module)

let u2 = User { name: "bob" }            // Compile error: age not specified
```

### Factory Pattern
To allow external instantiation, expose a factory function.

```lak
// user.lak
pub fn new_user(name: string, age: i64) -> User {
    return User { name: name, age: age }
}

// main.lak
import "./user"
let u = user.new_user("alice", 30)
```
