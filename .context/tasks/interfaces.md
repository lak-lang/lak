# Interfaces

## Overview
Implement interfaces with Go-style implicit implementation.

### Syntax
```lak
interface Stringer {
    fn to_string(self) -> string
}

interface Writer {
    fn write(mut self, data: string)
}
```

### Implicit Implementation
If a struct has all required methods, it satisfies the interface automatically.

```lak
struct User {
    pub name: string

    pub fn to_string(self) -> string {
        return self.name
    }
}

// User automatically satisfies Stringer
fn print_string(s: Stringer) {
    println(s.to_string())
}

let u = User { name: "alice" }
print_string(u)                  // OK
```

### self in Interfaces
- `self`: read-only methods
- `mut self`: mutating methods
