# codegen Module

LLVM IR generation and native object file output.

## Overview

Transforms Lak AST into LLVM IR and generates native object files. Uses Inkwell (safe Rust bindings for LLVM).

## Module Structure

| File | Responsibility |
|------|----------------|
| `mod.rs` | `Codegen` struct, strict entry points (`compile_with_inferred_types()` / `compile_modules_with_inferred_types()`), compatibility entry points (`compile()` / `compile_modules()`), `mangle_name()`, `derive_mangle_prefix()`, `compute_mangle_prefixes()`, `compute_entry_mangle_prefix()`, `path_components_to_strings()`, `get_mangle_prefix()` |
| `error.rs` | `CodegenError`, `CodegenErrorKind` |
| `binding.rs` | `VarBinding` (stack allocation and type info for variables) |
| `builtins.rs` | Built-in/runtime bindings (`println` dispatch, `panic`, string comparison helpers) |
| `expr.rs` | Expression codegen (literals, variable references, calls) |
| `stmt.rs` | Statement codegen (expression/let/assign/discard/return/if/while/break/continue) |
| `target.rs` | Target machine initialization and object file output |
| `tests.rs` | Unit tests |

## Preconditions

**Important**: This module assumes the AST has passed semantic analysis.

- Undefined variables, type mismatches, duplicate variables are already caught
- Semantic analysis resolves inferred `let` bindings in symbol metadata; the AST is treated as immutable after parsing, so parser placeholders (`Type::Inferred`) may intentionally remain in AST nodes
- Driver passes semantic inferred-binding results to codegen (`compile_with_inferred_types` / `compile_modules_with_inferred_types`) so strict codegen paths do not re-infer AST placeholders
  - "side-channel" here means compiler-internal metadata maps keyed by AST spans, separate from AST mutation
- Compatibility `compile` / `compile_modules` do not re-infer unresolved placeholders; callers must use strict entry points when AST still contains `Type::Inferred`
- Violations are handled by returning `CodegenError::InternalError` (not `panic!` or `debug_assert!`)
- Errors in this module include infrastructure issues (LLVM failures, target errors), module path validation, and invariant violations reported as `CodegenError::InternalError` (for example unresolved inferred types)

## Error Types

`CodegenErrorKind`:
- `InternalError` - Invariant violations that indicate compiler bugs (LLVM IR generation failures, module path resolution errors, etc.)
- `TargetError` - Target initialization or object file output failures
- `InvalidModulePath` - Module path validation errors (non-UTF-8 components, duplicate prefixes)

## Lifetime `'ctx`

```rust
pub struct Codegen<'ctx> { ... }
```

`'ctx` is tied to the LLVM `Context`. The context must outlive the `Codegen` instance.

## Variable Management

- `VarBinding` holds stack allocation (`alloca`) and type information
- Managed via scoped stack `variables: Vec<HashMap<String, VarBinding>>`
- Cleared per function
- `StmtKind::Assign` reuses the existing variable binding (`alloca`) and emits `store`
- AST placeholders are not rewritten in place; in strict mode, `StmtKind::Let` uses semantic side-channel inferred types before creating `alloca`
- Mutability is validated in semantic analysis; codegen assumes reassignment is semantically valid

## Type Mapping

| Lak Type | LLVM Type |
|----------|-----------|
| `i8` | `i8` |
| `i16` | `i16` |
| `i32` | `i32` |
| `i64` | `i64` |
| `u8` | `i8` |
| `u16` | `i16` |
| `u32` | `i32` |
| `u64` | `i64` |
| `f32` | `f32` |
| `f64` | `f64` |
| `string` | `ptr` |
| `bool` | `i1` |
| `inferred` | N/A (internal marker; must be resolved before LLVM conversion) |

## Runtime Integration

- Runtime functions declared as external:
  - `lak_println`
  - `lak_println_i8`, `lak_println_i16`, `lak_println_i32`, `lak_println_i64`
  - `lak_println_u8`, `lak_println_u16`, `lak_println_u32`, `lak_println_u64`
  - `lak_println_f32`, `lak_println_f64`
  - `lak_println_bool`
  - `lak_streq`, `lak_strcmp` (string comparison runtime helpers)
  - `lak_panic`
- Implemented in the `runtime/` crate
- Final binary links against the runtime library

## Generated Code Characteristics

- Uses C calling convention
- `main` function generated with `int main()` signature
- All user-defined functions except entry `main` are emitted with mangled symbol names
- Name mangling applies to both single-file and multi-module codegen paths (strict and compatibility entry points)
- Returns 0 on success

## Extension Guidelines

1. New expressions/statements: add patterns to `expr.rs` / `stmt.rs`
2. New built-in functions: add to `builtins.rs` with `declare_*` and `generate_*`, and add the function name to `BUILTIN_NAMES`
3. New types: add to `get_llvm_type()` and `VarBinding::new()`
4. Return `CodegenError::InternalError` for conditions guaranteed by semantic analysis (not `debug_assert!` or `panic!`)
