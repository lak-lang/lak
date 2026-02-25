# ast Module

Abstract Syntax Tree definitions for the Lak programming language.

## Overview

Defines the data structures that represent parsed Lak programs. The AST is produced by the parser and consumed by the code generator.

## Module Structure

| File | Responsibility |
|------|----------------|
| `mod.rs` | Module documentation, public re-exports |
| `types.rs` | Type annotations (`Type` enum: integer/float primitives, bool, string) |
| `expr.rs` | Expression nodes (`Expr`, `ExprKind`) |
| `stmt.rs` | Statement nodes (`Stmt`, `StmtKind`) |
| `program.rs` | Top-level nodes (`Program`, `ImportDecl`, `FnDef`, `FnParam`, `Visibility`) |
| `tests.rs` | Unit tests |

## AST Hierarchy

```
Program
  ├─ ImportDecl
  └─ FnDef (function definition)
      ├─ FnParam
      └─ Stmt (statement)
          ├─ StmtKind::Expr(Expr)
          ├─ StmtKind::Let { is_mutable, name, ty: Type, init: Expr }
          ├─ StmtKind::Assign { name, value: Expr }
          ├─ StmtKind::Discard(Expr)
          ├─ StmtKind::Return(Option<Expr>)
          ├─ StmtKind::If { condition, then_branch, else_branch }
          ├─ StmtKind::While { condition, body }
          ├─ StmtKind::Break
          └─ StmtKind::Continue
```

## Type System

`Type` enum represents type annotations:
- `I8` - 8-bit signed integer (`i8`)
- `I16` - 16-bit signed integer (`i16`)
- `I32` - 32-bit signed integer (`i32`)
- `I64` - 64-bit signed integer (`i64`)
- `U8` - 8-bit unsigned integer (`u8`)
- `U16` - 16-bit unsigned integer (`u16`)
- `U32` - 32-bit unsigned integer (`u32`)
- `U64` - 64-bit unsigned integer (`u64`)
- `F32` - 32-bit floating-point (`f32`)
- `F64` - 64-bit floating-point (`f64`)
- `Bool` - boolean (`bool`)
- `String` - UTF-8 string (`string`)

Implements `Display` for user-facing error messages.

## Expressions

`Expr` combines an `ExprKind` with source location (`Span`).

`ExprKind` variants:
- `StringLiteral(String)` - String literal (unescaped)
- `IntLiteral(i128)` - Integer literal (wide storage for deferred range checking)
- `FloatLiteral(f64)` - Floating-point literal (parsed as `f64`, may be contextually typed as `f32`)
- `BoolLiteral(bool)` - Boolean literal
- `Identifier(String)` - Variable reference
- `Call { callee, args }` - Function call
- `BinaryOp { left, op, right }` - Binary operator expression
- `UnaryOp { op, operand }` - Unary operator expression
- `IfExpr { condition, then_block, else_block }` - If-expression value form
- `MemberAccess { object, member }` - Member access syntax node (module access path)
- `ModuleCall { module, function, args }` - Module-qualified function call

Expression nodes are recursive and include helper logic for numeric operand
adaptation in binary expressions.

## Statements

`Stmt` combines a `StmtKind` with source location (`Span`).

`StmtKind` variants:
- `Expr(Expr)` - Expression statement (side effects only)
- `Let { is_mutable, name, ty, init }` - Variable declaration (`let` / `let mut`)
- `Assign { name, value }` - Reassignment of an existing variable (`x = expr`)
- `Discard(Expr)` - Explicit discard (`let _ = expr`)
- `Return(Option<Expr>)` - Return statement
- `If { condition, then_branch, else_branch }` - Conditional branching
- `While { condition, body }` - Loop
- `Break` - Exit the innermost loop
- `Continue` - Continue the innermost loop

## Program Structure

`Program` is the root AST node containing:
- `imports: Vec<ImportDecl>` - Import declarations
- `functions: Vec<FnDef>` - Function definitions

`ImportDecl` represents `import "path"` / `import "path" as alias` declarations.

`FnDef` represents a function definition:
- `visibility: Visibility` - `Public` or `Private`
- `name: String` - Function name
- `params: Vec<FnParam>` - Parameter definitions (`name`, `ty`, `span`)
- `return_type: String` - Return type as parsed source text (e.g., `void`, `i32`, `f64`)
- `return_type_span: Span` - Location of return type token
- `body: Vec<Stmt>` - Function body statements
- `span: Span` - Location from `pub`/`fn` to `{`

### Invariants

`FnDef` maintains these invariants (enforced by parser):
- `name` is a non-empty valid identifier
- each parameter name is a non-empty valid identifier
- `return_type` is a non-empty source type name (`void` or primitive type validation is semantic phase)
- `return_type_span` points to the actual return type token
- `span` encompasses the function signature
- `span.start <= span.end`

### Testing Helper

`FnDef::for_testing()` creates instances with dummy spans for unit tests.

## Span Tracking

All AST nodes include a `Span` for error reporting:
- `Expr`: first token to last token of expression
- `Stmt`: first token to last token of statement
- `ImportDecl`: `import` keyword to path/alias
- `FnDef`: `pub`/`fn` keyword to before `{`

## Dependencies

All AST types depend on `Span` from `crate::token`.

Internal dependencies flow: `Type` → `Expr` → `Stmt` → `Program`/`FnDef`

## Extension Guidelines

1. New types: add variant to `Type` in `types.rs`, update `Display` impl
2. New expressions: add variant to `ExprKind` in `expr.rs`
3. New statements: add variant to `StmtKind` in `stmt.rs`
4. New top-level constructs: add to `program.rs` and update `Program`
5. Testing helpers: add cfg(test) constructors to appropriate modules
