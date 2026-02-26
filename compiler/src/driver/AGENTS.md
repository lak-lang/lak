# driver Module

Compilation driver orchestration for `lak build` and `lak run`.

## Overview

Coordinates the full pipeline from source file input to executable output.
This module owns phase ordering, cross-phase error unification, and runtime execution.

## Module Structure

| File | Responsibility |
|------|----------------|
| `mod.rs` | Driver entry points (`build`, `run`), shared pipeline (`compile_to_executable`), unified error types |
| `link.rs` | Linker invocation and linker-setup error mapping |

## Entry Points

- `build(file, output) -> Result<(), Box<CompileErrorWithContext>>`
- `run(file) -> Result<i32, Box<CompileErrorWithContext>>`

Both entry points read source, construct `CompileContext`, run the shared compilation
pipeline, and return rich errors with source context for diagnostics.

## Pipeline Stages

1. Canonicalize entry path
2. Resolve modules (`ModuleResolver`)
3. Semantic analysis for imported modules (`analyze_module`)
4. Semantic analysis for entry module (`analyze_with_modules`)
5. LLVM code generation (`Codegen`)
6. Object file emission
7. Native linking (`link::link`)

Single-module input uses `Codegen::compile_with_inferred_types`; multi-module input uses
`Codegen::compile_modules_with_inferred_types`.

## Error Model

`CompileError` unifies all phases:
- resolution (`ResolverError`)
- semantic analysis (`SemanticError`)
- imported-module semantic context (`ModuleSemanticContext`)
- codegen (`CodegenError`)
- linking (`LinkError`)
- I/O and path infrastructure errors

`CompileErrorWithContext` carries:
- source filename
- source text
- the underlying `CompileError`

This separation keeps compilation logic pure while allowing diagnostics to render with
source context at the call site.

## Linking and Execution

- `link.rs` translates `LinkerSetupError` to driver-level `CompileError::Link(...)`.
- `format_exit_status()` formats linker failures, including Unix signals.
- `get_exit_code_with_signal()` maps process termination to shell-compatible exit codes
  (`128 + signal` on Unix).

## Extension Guidelines

1. Keep `compile_to_executable()` side-effect minimal (no user-facing printing).
2. Add new phase errors as structured `CompileError` variants, not free-form strings.
3. Preserve per-module semantic validation ordering before entry-module analysis.
4. When adding pipeline stages, update both `build` and `run` through shared logic.
5. Keep platform-specific behavior (`#[cfg(...)]`) isolated in linker/exit helpers.
