# Methods

## Overview
Implement methods for structs.

### Syntax
Define methods inside the struct definition. The first parameter is `self`.

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
- `self`: immutable (read-only)
- `mut self`: mutable (can modify)

### Visibility
- Methods are private by default.
- Use the `pub` keyword to make methods public.

### Calling Methods
```lak
let u = User { name: "alice", age: 30 }
let greeting = u.greet()        // "Hello, alice"
let adult = u.is_adult()        // true
```

### mut self Restriction
`mut self` methods can only be called on variables declared with `mut`.

```lak
let u = User { name: "alice", age: 30 }
u.set_name("bob")               // Compile error: u is immutable

let mut u2 = User { name: "alice", age: 30 }
u2.set_name("bob")              // OK
```
