# Simple Enums

## Overview
Implement simple enums without payload values.

### Syntax
```lak
enum Color {
    Red
    Green
    Blue
}
```

### Usage
```lak
let c: Color = Color.Red
```

### Variant Resolution
- Short form can be used inside `match`.
- Fully qualified names are required elsewhere.

```lak
// Inside match: short form is OK
match color {
    Red => "red"
    Green => "green"
    Blue => "blue"
}

// Elsewhere: fully qualified form is required
let color = Color.Red
```
