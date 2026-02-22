# String Comparison Operators

## Overview
Enable ordered comparison operators for `string`: `<`, `>`, `<=`, `>=`.

### Target Operators
- `<`: less than
- `>`: greater than
- `<=`: less than or equal
- `>=`: greater than or equal

### Semantics
- Compare strings lexicographically by Unicode scalar value order.
- Result type is `bool`.
- Both operands must be `string`.
- Existing numeric ordered comparisons remain unchanged.

### Type Rules
- `string` vs `string` is valid.
- Mixed-type ordered comparison is invalid (for example, `string < i64`).
- Existing equality operators (`==`, `!=`) are out of scope for this task.

### Examples
```lak
let a = "apple"
let b = "banana"

println(a < b)     // true
println(a <= b)    // true
println(b > a)     // true
println(b >= b)    // true
```

```lak
let x = "z"
let y = "10"

println(x > y)     // true (lexicographical comparison)
```

```lak
let name = "lak"
let n = 3

println(name < n)  // type error: ordered comparison requires same comparable type
```
