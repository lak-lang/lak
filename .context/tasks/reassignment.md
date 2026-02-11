# Reassignment for Mutable Variables

## Overview
Implement reassignment to variables declared with `mut`.

### Syntax
```lak
let mut count = 0
count = count + 1
count = 10
```

### Rules
- Only variables declared with `mut` can be reassigned.
- Reassigning immutable variables is a compile error.
- Reassignment types must match the original type.

### Examples
```lak
let x = 5
x = 10              // Compile error: x is immutable

let mut y = 5
y = 10              // OK

let mut z: int = 5
z = "hello"         // Compile error: type mismatch
```
