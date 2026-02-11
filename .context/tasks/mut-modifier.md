# mut Modifier

## Overview
Implement the `mut` modifier to control variable mutability.

### Syntax
```lak
let x = 5                   // Immutable
let mut count = 0           // Mutable
let mut flag: bool = true   // Mutable + explicit type
```

### Rules
- Variables are immutable by default.
- Adding `mut` makes them mutable.
- Reassigning immutable variables is a compile error.
