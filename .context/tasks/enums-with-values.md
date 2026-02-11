# Enums with Values

## Overview
Implement enums with payload values (algebraic data types).

### Syntax
```lak
enum Shape {
    Circle(f64)              // single value
    Rectangle(f64, f64)      // multiple values
    Point                    // no value (mixed variants allowed)
}
```

### Usage
```lak
let s = Shape.Circle(3.14)
let r = Shape.Rectangle(10.0, 20.0)

match s {
    Circle(r) => println(r)
    Rectangle(w, h) => println(w * h)
    Point => println("point")
}
```

### Pattern Matching
- Bind values contained in variants to variables.
- When using `|` for multiple patterns, common binding variable names are required.

```lak
match shape {
    Circle(r) | Square(r) => r * r    // shared binding r
    Rectangle(w, h) => w * h
}
```
