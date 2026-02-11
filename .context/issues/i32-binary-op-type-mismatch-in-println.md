# Using an i32 Variable in a Binary Operation Inside println Causes an Internal Codegen Error

## Summary

When an `i32` variable is used in a binary operation (arithmetic or comparison) inside a `println()` argument, the compiler produces an internal codegen error instead of either compiling successfully or emitting a meaningful type error.

## Details

### Steps to Reproduce

**Case 1: Arithmetic Operation (`+`)**

```lak
fn main() -> void {
  let x: i32 = 6
  println(5 + x)
}
```

```
Error: Internal error
   ╭─[ ._/main.lak:5:15 ]
   │
 5 │   println(5 + x)
   │               ┬
   │               ╰── Internal error: type mismatch for variable 'x' in codegen. Expected 'i64', but variable has type 'i32'. Semantic analysis should have caught this. This is a compiler bug.
───╯
```

**Case 2: Comparison Operation (`>`)**

```lak
fn main() -> void {
  let x: i32 = 6
  println(5 > x)
}
```

```
Error: Internal error
   ╭─[ ._/main.lak:5:15 ]
   │
 5 │   println(5 > x)
   │               ┬
   │               ╰── Internal error: type mismatch for variable 'x' in codegen. Expected 'i64', but variable has type 'i32'. Semantic analysis should have caught this. This is a compiler bug.
───╯
```

### Related Code Locations

- **`compiler/src/codegen/builtins.rs`**: `get_expr_type()` returns `Type::I64` for all integer literals. Binary operation type inference uses only the left operand, so when the left operand is the integer literal `5`, the full expression is inferred as `i64`, and dispatch goes to `generate_println_i64()`.
- **`compiler/src/codegen/expr.rs`** (lines 192-206): In `generate_expr_value()` for `Identifier`, there is a type check against `expected_ty`. Since `expected_ty` is `i64` (derived from the integer literal) but variable `x` is `i32`, codegen fails.
- **`compiler/src/semantic/mod.rs`**: `validate_expr_for_println()` does not fully type-check binary operations inside `println()` arguments, so the mismatch between integer literal (`i64`) and `i32` variable is not caught during semantic analysis.

## Expected Behavior

The compiler should do one of the following:
- Compile successfully (if implicit integer conversion or literal type adaptation is intended)
- Report a clear semantic analysis error for the `i64`/`i32` mismatch in the binary operation

It should not produce an internal codegen error for user code.
