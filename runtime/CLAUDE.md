# runtime Crate

The Lak runtime library. Provides functions called by compiled Lak programs at runtime.

## Overview

This crate is compiled as a **static library** (`staticlib`) and linked with generated object files to produce final executables. It provides I/O and other runtime support functions.

## Crate Configuration

```toml
[lib]
crate-type = ["staticlib"]
```

Output: `liblak_runtime.a`

## ABI

All exported functions use the **C calling convention** (`extern "C"`) for compatibility with LLVM-generated code.

Functions are marked with `#[unsafe(no_mangle)]` to preserve their names in the compiled library.

## Exported Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `lak_println` | `unsafe fn(s: *const c_char)` | Print string with newline |
| `lak_println_i32` | `fn(value: i32)` | Print i32 with newline |
| `lak_println_i64` | `fn(value: i64)` | Print i64 with newline |
| `lak_println_bool` | `fn(value: bool)` | Print boolean ("true"/"false") with newline |
| `lak_panic` | `unsafe fn(message: *const c_char) -> !` | Print panic message to stderr and exit(1) |

### `lak_println`

```rust
pub unsafe extern "C" fn lak_println(s: *const c_char)
```

- Prints the given C string followed by a newline to stdout
- Handles null pointer gracefully (prints empty line)
- Handles invalid UTF-8 with lossy conversion
- Called by Lak's `println("...")` built-in function

### `lak_println_i32` / `lak_println_i64`

```rust
pub extern "C" fn lak_println_i32(value: i32)
pub extern "C" fn lak_println_i64(value: i64)
```

- Print integer value followed by a newline to stdout
- Called by Lak's `println()` when argument is an integer type

### `lak_println_bool`

```rust
pub extern "C" fn lak_println_bool(value: bool)
```

- Prints "true" or "false" followed by a newline to stdout
- Called by Lak's `println()` when argument is a boolean type

### `lak_panic`

```rust
pub unsafe extern "C" fn lak_panic(message: *const c_char) -> !
```

- Prints `panic: {message}` to stderr and terminates with exit code 1
- Handles null pointer (`panic: (no message)`)
- Handles invalid UTF-8 with lossy conversion
- Never returns (diverging function)
- Called by Lak's `panic()` built-in function

## Safety

All exported functions are `unsafe` because they accept raw pointers from C/LLVM code. The compiler guarantees valid pointers through semantic analysis.

## Integration with Compiler

1. Compiler generates calls to runtime functions (`lak_println`, `lak_println_i32`, `lak_println_i64`, `lak_println_bool`, `lak_panic`) in LLVM IR
2. `build.rs` in compiler crate sets `LAK_RUNTIME_PATH` to `liblak_runtime.a`
3. System linker (`cc`) links the object file with the runtime

## Coding Guidelines

### Error Handling

- **Avoid `panic!`**: Handle errors gracefully instead of panicking. A panic in the runtime will crash the user's Lak program.
- **Graceful degradation**: For invalid inputs (null pointers, invalid UTF-8), either return early, use safe defaults, or log to stderr. Do not crash.
- **Exception**: `panic!` is acceptable in tests.

## Extension Guidelines

When adding new runtime functions:

1. Use `extern "C"` calling convention
2. Use `#[unsafe(no_mangle)]` attribute
3. Accept C-compatible types (`*const c_char`, `i32`, `i64`, etc.)
4. Handle null pointers and edge cases gracefully (no `panic!`)
5. Add corresponding:
   - Declaration in `codegen/builtins.rs` (`declare_*`)
   - Code generation in `codegen/builtins.rs` (`generate_*`)
   - Validation in `semantic/mod.rs` (`analyze_call`)

## Testing

Tests use `CString` to create valid C strings for testing:

```rust
let s = CString::new("Hello").unwrap();
unsafe { lak_println(s.as_ptr()) };
```
