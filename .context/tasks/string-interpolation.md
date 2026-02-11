# String Interpolation

## Overview
Implement string interpolation. Embed expressions with `${}` syntax.

### Syntax
```lak
let name = "lak"
let version = 1
println("${name} v${version}")    // "lak v1"
```

### Features
- Variable interpolation: `"${name}"`
- Expression interpolation: `"${a + b}"`
- Nesting is supported

### Default Formatting
All types have a default format:
- Integers: `42`, `-10`
- Floating point: `3.14`, `-0.5`
- Booleans: `true`, `false`
- Strings: `hello` (without quotes)
- Structs: `User { name: "alice", age: 30 }`
- Enums: `Option.Some(42)`, `Color.Red`

### Stringer Priority
For types implementing the `Stringer` interface, prioritize `to_string()`.

```lak
struct User {
    pub name: string

    pub fn to_string(self) -> string {
        return "User: " + self.name
    }
}

let u = User { name: "alice" }
println("${u}")    // "User: alice" (uses to_string())
```
