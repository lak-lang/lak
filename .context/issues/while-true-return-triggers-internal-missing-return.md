# Internal Missing-Return Error for `while true { return ... }` in Non-void Function

## Summary

A non-void function that uses `while true` and returns from inside the loop body fails with an internal missing-return error during compilation.

## Details

When compiling the following program:

```lak
fn foo() -> i64 {
  while true {
    return 1
  }
}

fn main() -> void {
}
```

Compiler output:

```text
Error in /tmp/while_true_return_issue_with_main.lak: Internal error: function '_L5_entry_foo' with return type 'i64' reached end without return. Semantic analysis should have rejected this. This is a compiler bug.
```

Observed behavior:

- Compilation fails with an internal compiler error.
- The failure is reported as a missing return at function end.

Relevant code locations:

- `compiler/src/semantic/mod.rs:302` (`analyze_while`)
- `compiler/src/codegen/mod.rs:718` (missing return at function end)
- `compiler/src/codegen/error.rs:966` (internal error message text)

## Expected Behavior

According to `.context/SPEC.md`:

- `Functions / Return Statement`: functions with return values require `return`.
- `Control Flow / while Loop`: `while true` is the infinite-loop form.

For this program, the loop body contains `return 1`, so this function should not end with a user-visible internal missing-return error.
