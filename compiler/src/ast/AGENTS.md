# ast Module

Abstract Syntax Tree definitions for the Lak programming language.

## Overview

Defines the data structures that represent parsed Lak programs. The AST is produced by the parser and consumed by the code generator.

## Module Structure

| File | Responsibility |
|------|----------------|
| `mod.rs` | Module documentation, public re-exports |
| `types.rs` | Type annotations (`Type` enum: i32, i64) |
| `expr.rs` | Expression nodes (`Expr`, `ExprKind`) |
| `stmt.rs` | Statement nodes (`Stmt`, `StmtKind`) |
| `program.rs` | Top-level structure (`Program`, `FnDef`) |
| `tests.rs` | Unit tests |

## AST Hierarchy

```
Program
  └─ FnDef (function definition)
      └─ Stmt (statement)
          ├─ StmtKind::Expr(Expr)
          ├─ StmtKind::Let { is_mutable, name, ty: Type, init: Expr }
          ├─ StmtKind::Discard(Expr)
          ├─ StmtKind::Return(Option<Expr>)
          ├─ StmtKind::If { condition, then_branch, else_branch }
          ├─ StmtKind::While { condition, body }
          ├─ StmtKind::Break
          └─ StmtKind::Continue
```

## Type System

`Type` enum represents type annotations:
- `I32` - 32-bit signed integer (`i32`)
- `I64` - 64-bit signed integer (`i64`)

Implements `Display` for user-facing error messages.

## Expressions

`Expr` combines an `ExprKind` with source location (`Span`).

`ExprKind` variants:
- `StringLiteral(String)` - String literal (unescaped)
- `IntLiteral(i64)` - Integer literal
- `Identifier(String)` - Variable reference
- `Call { callee, args }` - Function call

Expression nodes are recursive: function calls contain argument expressions.

## Statements

`Stmt` combines a `StmtKind` with source location (`Span`).

`StmtKind` variants:
- `Expr(Expr)` - Expression statement (side effects only)
- `Let { is_mutable, name, ty, init }` - Variable declaration (`let` / `let mut`)
- `Discard(Expr)` - Explicit discard (`let _ = expr`)
- `Return(Option<Expr>)` - Return statement
- `If { condition, then_branch, else_branch }` - Conditional branching
- `While { condition, body }` - Loop
- `Break` - Exit the innermost loop
- `Continue` - Continue the innermost loop

## Program Structure

`Program` is the root AST node containing a list of function definitions.

`FnDef` represents a function definition:
- `name: String` - Function name
- `return_type: String` - Return type (currently only "void")
- `return_type_span: Span` - Location of return type token
- `body: Vec<Stmt>` - Function body statements
- `span: Span` - Location from `fn` to `{`

### Invariants

`FnDef` maintains these invariants (enforced by parser):
- `name` is a non-empty valid identifier
- `return_type` is a valid type name
- `return_type_span` points to the actual return type token
- `span` encompasses the function signature
- `span.start <= span.end`

### Testing Helper

`FnDef::for_testing()` creates instances with dummy spans for unit tests.

## Span Tracking

All AST nodes include a `Span` for error reporting:
- `Expr`: first token to last token of expression
- `Stmt`: first token to last token of statement
- `FnDef`: `fn` keyword to before `{`

## Dependencies

All AST types depend on `Span` from `crate::token`.

Internal dependencies flow: `Type` → `Expr` → `Stmt` → `Program`/`FnDef`

## Extension Guidelines

1. New types: add variant to `Type` in `types.rs`, update `Display` impl
2. New expressions: add variant to `ExprKind` in `expr.rs`
3. New statements: add variant to `StmtKind` in `stmt.rs`
4. New top-level constructs: add to `program.rs` and update `Program`
5. Testing helpers: add cfg(test) constructors to appropriate modules
