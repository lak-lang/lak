# while Loop with break/continue

## Overview
Implement the `while` loop and `break`/`continue` statements.

### Syntax
```lak
while condition {
    // ...
}
```

### Features
- Repeat while the condition is true
- Infinite loop with `while true { ... }`
- Exit loop with `break`
- Continue to the next iteration with `continue`

### Examples
```lak
// Conditional loop
while condition {
    // ...
}

// Infinite loop
while true {
    if should_stop {
        break
    }
}

// continue
while condition {
    if skip_this {
        continue
    }
    // ...
}
```
