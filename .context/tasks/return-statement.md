# Return Statement

## Overview
Implement the `return` statement.

### Syntax
- `return expr`: for functions with a return value
- `return`: for early return in void functions

### Rules
- Functions with a return value must have a `return` statement.
- Void functions may omit a trailing `return`.
- For early return in void functions, use `return` without a value.

### Examples
```lak
fn abs(x: i64) -> i64 {
    if x < 0 {
        return -x
    }
    return x              // return is required
}

fn greet_if(flag: bool) -> void {
    if !flag {
        return            // early return (no value)
    }
    println("hello")
                          // trailing return can be omitted
}
```
