# User Function Names Can Collide with Runtime Builtin Symbols (`lak_*`)

## Summary

User-defined function names can collide with runtime builtin symbols such as `lak_println` and `lak_panic`. This causes observable failures including silent behavior changes and linker errors.

## Details

### Reproduction A: `lak_println` collision changes runtime behavior

```lak
fn lak_println() -> void {}

fn main() -> void {
  println("hello")
  lak_println()
}
```

Observed execution result (`cargo run -- run <file>`):

- Process exit code: `0`
- Program stdout: empty (`STDOUT_BYTES=0`)

Control case without collision:

```lak
fn user_println() -> void {}

fn main() -> void {
  println("hello")
  user_println()
}
```

Observed execution result:

```
hello
```

### Reproduction B: `lak_panic` collision produces linker failure

```lak
fn lak_panic() -> void {
  println("fake panic")
}

fn main() -> void {
  panic("x")
}
```

Observed build result (`cargo run -- build <file> -o <out>`):

- Process exit code: `1`
- stderr includes:

```
Error: Linker failed with exit code 1
duplicate symbol '_lak_panic' in:
ld: 1 duplicate symbols
```

### Related Code Locations

- Runtime builtin declarations:
  - `compiler/src/codegen/builtins.rs:23`
  - `compiler/src/codegen/builtins.rs:85`
  - `compiler/src/codegen/builtins.rs:98`
  - `compiler/src/codegen/builtins.rs:111`
  - `compiler/src/codegen/builtins.rs:124`
  - `compiler/src/codegen/builtins.rs:140`
  - `compiler/src/codegen/builtins.rs:167`
- Builtins are declared before user functions:
  - `compiler/src/codegen/mod.rs:345`
  - `compiler/src/codegen/mod.rs:348`
- Function collection accepts user function names as-is:
  - `compiler/src/semantic/mod.rs:148`
  - `compiler/src/semantic/mod.rs:158`

## Expected Behavior

According to the language specification, source-level builtins are `println(value: any) -> void` and `panic(message: string) -> never` (`.context/SPEC.md:786`, `.context/SPEC.md:788`), and local overriding is described for those source-level names (`.context/SPEC.md:790` to `.context/SPEC.md:799`).  
Using user-defined names like `lak_println` or `lak_panic` does not appear in the language-level API and does not describe a behavior where source-level builtin semantics are replaced or link-time duplicate-symbol failure is introduced.

