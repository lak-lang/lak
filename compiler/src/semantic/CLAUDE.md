# semantic Module

Semantic analysis for the Lak programming language.

## Overview

Validates an AST for semantic correctness before code generation. Performs name resolution, type checking, and structural validation. If analysis succeeds, the AST is guaranteed to be semantically valid.

## Module Structure

| File | Responsibility |
|------|----------------|
| `mod.rs` | `SemanticAnalyzer` struct, `analyze()` entry point |
| `error.rs` | `SemanticError`, `SemanticErrorKind` |
| `symbol.rs` | `SymbolTable`, `FunctionInfo`, `VariableInfo` |
| `tests/` | Unit tests (see below) |

### Test Structure

| File | Coverage |
|------|----------|
| `tests/mod.rs` | Test helpers, SemanticError Display tests |
| `tests/function_tests.rs` | Function definition, main validation, undefined function, scope isolation |
| `tests/variable_tests.rs` | Duplicate/undefined variable detection |
| `tests/type_tests.rs` | Type mismatch, overflow, invalid expressions, println arguments, valid programs |
| `tests/symbol_table_tests.rs` | SymbolTable data structure unit tests |

## Analysis Phases

The analyzer runs in three sequential phases:

1. **Function collection**: Gather all function definitions, check for duplicates
2. **Main validation**: Verify `main` exists and has correct signature (`-> void`)
3. **Body analysis**: Analyze each function body (variables, types, expressions)

## Error Types

`SemanticErrorKind`:

| Kind | Description |
|------|-------------|
| `DuplicateFunction` | Function defined multiple times |
| `DuplicateVariable` | Variable defined multiple times in same scope |
| `UndefinedVariable` | Variable used but not defined |
| `UndefinedFunction` | Function called but not defined |
| `TypeMismatch` | Expected vs actual type mismatch |
| `IntegerOverflow` | Integer out of range for target type |
| `InvalidArgument` | Wrong argument count or type |
| `InvalidExpression` | Expression in invalid context (e.g., literal as statement) |
| `MissingMainFunction` | No main function found |
| `InvalidMainSignature` | Main has wrong signature |

## Symbol Table

`SymbolTable` provides:
- Global function namespace (flat)
- Scoped variable lookup (stack-based)

### Scope Management

```rust
symbols.enter_scope();   // Enter function body
// ... analyze statements ...
symbols.exit_scope();    // Exit function body
```

Variables are looked up from innermost to outermost scope.

## Type Checking

Currently supports:
- `i32`: values must be in range `i32::MIN..=i32::MAX`
- `i64`: all lexer-parsed integers are valid

Type checking occurs in:
- `let` statements: initializer must match declared type
- Variable references: variable type must match expected type

## Built-in Functions

`println` is the only built-in function:
- Requires exactly 1 argument
- Argument must be a string literal

User-defined function calls are validated for existence only (no parameters yet).

## Expression Statement Validation

Only function calls are valid as expression statements. The following are rejected:
- String literals (no effect)
- Integer literals (no effect)
- Bare identifiers (no effect)

## Guarantees to Codegen

If `analyze()` returns `Ok(())`, codegen can assume:
- `main` function exists with `void` return type
- All variable references are defined
- All variable types match their usage
- All integer literals fit their target types
- All function calls reference defined functions

## Extension Guidelines

1. New types: add case in `check_integer_range()` and `check_expr_type()`
2. New built-in functions: add case in `analyze_call()`
3. New expression forms: add case in `check_expr_type()` and `analyze_expr_stmt()`
4. New statement forms: add case in `analyze_stmt()`
5. Scoping changes: modify `enter_scope()` / `exit_scope()` in `symbol.rs`
