# Unhelpful Error Message for Invalid Top-Level Statements

## Summary

When an executable statement such as `let` is written at the top level, the error message says "expected `fn` keyword but found `let` keyword," which is too technical and does not clearly explain the cause or fix for users.

## Details

When compiling the following source code:

```lak
let x: i32 = 10

fn main() -> void {
  println("Hello, world!")
}
```

The compiler outputs this error:

```
Error: Parse error in module
   ╭─[ ./main.lak:1:1 ]
   │
 1 │ let x: i32 = 10
   │ ─┬─
   │  ╰─── Parse error in module '...': Expected 'fn' keyword, found 'let' keyword
───╯
```

Problems:

- It does not explain why `fn` is expected.
- It does not clearly communicate that executable top-level statements are disallowed.
- It does not tell the user to move the `let` statement inside a function.
- The error title, "Parse error in module," is too generic to identify this specific issue.

Relevant code locations:

- Parser top-level parsing: `compiler/src/parser/mod.rs` (`parse()` method, around lines 124-158)
  - The grammar is `program → import* fn_def* EOF`, and after imports it directly calls `parse_fn_def()`.
- `self.expect(&TokenKind::Fn)` in `parse_fn_def()`: `compiler/src/parser/fn_def.rs:31`
  - If a token other than `fn` appears, it produces a generic "Expected X, found Y" error.
- Error message construction: `compiler/src/parser/helpers.rs:128-139` (`expect()` method)

## Expected State

According to the language specification (`.context/SPEC.md`, "Top-Level Constraints" section), only declarations (functions, structs, enums, interfaces) are allowed at the top level, and executable statements such as `let` are explicitly forbidden. The error message should reflect this rule so users can understand both the cause and the fix.
