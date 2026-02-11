# match Expression

## Overview
Implement `match` expressions with exhaustiveness checking.

### Syntax
```lak
match expr {
    pattern1 => expr1
    pattern2 => expr2
    _ => default
}
```

### Features
- Value binding: `Some(x) =>`
- Exhaustiveness checking: enums must cover all variants or include `_`
- Wildcard: `_`
- Multiple patterns: separated by `|`
- Expression use: all arms must return the same type

### Examples
```lak
// Used as an expression
let area = match shape {
    Circle(r) => 3.14 * r * r
    Rectangle(w, h) => w * h
    Point => 0.0
}

// Wildcard
match value {
    1 => "one"
    2 => "two"
    _ => "other"
}

// Multiple patterns
match color {
    Red | Green => "warm"
    Blue => "cool"
}
```
