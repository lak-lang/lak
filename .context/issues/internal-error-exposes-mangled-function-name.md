# Internal Error Exposes Mangled Function Name in User-Facing Message

## Summary

A user-facing internal compiler error message includes a mangled internal function symbol (`_L5_entry_foo`) instead of the source-level function name.

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

- The error message contains `_L5_entry_foo`, which is an internal mangled symbol name.
- This internal name is shown directly in user-visible diagnostics.

Relevant code locations:

- `compiler/src/codegen/mod.rs:459` (mangled naming pattern)
- `compiler/src/codegen/error.rs:966` (error message construction)

## Expected Behavior

User-facing diagnostics should refer to source-level identifiers (for example, `foo`) and avoid exposing internal mangled symbol names such as `_L5_entry_foo`.
