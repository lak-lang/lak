# Missing Return Error for `if true` in Non-void Function

## Summary

A non-void function is rejected with a missing return error when its body is `if true { return ... }`, even though the only reachable path returns a value.

## Details

The following program fails semantic analysis:

```lak
fn foo() -> i32 {
  if true {
    return 3
  }
}

fn main() -> void {
  let _ = foo()
}
```

Compiler output:

```text
Error: Type mismatch
   ╭─[ /tmp/if_true_return_issue.lak:1:13 ]
   │
 1 │ fn foo() -> i32 {
   │             ─┬─
   │              ╰─── Function 'foo' with return type 'i32' must return a value on all code paths
───╯
```

Related code locations:

- `compiler/src/semantic/mod.rs:238` checks `always_returns` for non-void functions.
- `compiler/src/semantic/mod.rs:276` - `compiler/src/semantic/mod.rs:293` computes `if` statement return flow.
- `compiler/src/semantic/mod.rs:289` sets `else_returns = false` when `else` is omitted.

## Expected Behavior

According to `.context/SPEC.md`:

- In "Functions / Return Statement", functions with return values require a `return` statement.
- In "Control Flow / if Expression", `if` used as a statement may omit `else`.

For this function body, `if true` makes the `then` branch the only reachable branch, and that branch executes `return 3`, so the function is treated as returning an `i32` on all reachable code paths.
