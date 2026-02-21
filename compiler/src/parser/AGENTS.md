# parser Module

Recursive descent parser for the Lak programming language.

## Overview

Transforms a token stream from the lexer into an Abstract Syntax Tree (AST). Implements a recursive descent parsing strategy with predictive parsing.

## Module Structure

| File | Responsibility |
|------|----------------|
| `mod.rs` | `Parser` struct, `parse()` entry point |
| `error.rs` | `ParseError` type |
| `helpers.rs` | Token navigation (`current`, `advance`, `expect`, `skip_newlines`) |
| `fn_def.rs` | Function definition parsing |
| `stmt.rs` | Statement parsing (`let`, `return`, `if`, `while`, `break`, `continue`, expression statements) |
| `types.rs` | Type annotation parsing |
| `expr.rs` | Expression parsing (if-expression, calls, operators, identifiers, literals) |
| `tests/` | Unit tests (see below) |

## Test Structure

Tests are organized by parser component:

| File | Coverage |
|------|----------|
| `tests/mod.rs` | Shared test helpers |
| `tests/fn_def.rs` | Function definitions, parameters, visibility, spans |
| `tests/stmt.rs` | Statements, newlines, `let mut`, discard |
| `tests/expr.rs` | Expressions, precedence, calls, literals, `if` expression |
| `tests/errors.rs` | Error detection and message quality |
| `tests/helpers.rs` | Edge cases, utilities |

## Grammar

```text
program     → import* fn_def* EOF
import      → "import" STRING ("as" IDENTIFIER)?
fn_def      → ("pub")? "fn" IDENTIFIER "(" param_list? ")" "->" return_type "{" stmt* "}"
param_list  → IDENTIFIER ":" type ("," IDENTIFIER ":" type)*
stmt        → let_stmt | return_stmt | if_stmt | while_stmt | break_stmt | continue_stmt | expr_stmt
let_stmt    → "let" "mut"? IDENTIFIER ":" type "=" expr | "let" "_" "=" expr
return_stmt → "return" expr?
if_stmt     → "if" expr "{" stmt* "}" ("else" (if_stmt | "{" stmt* "}"))?
while_stmt  → "while" expr "{" stmt* "}"
break_stmt  → "break"
continue_stmt → "continue"
type        → "i32" | "i64" | "string" | "bool"
return_type → type | "void"
expr_stmt   → expr
expr        → if_expr | unary | binary | call | module_call | IDENTIFIER | STRING | INT | BOOL
call        → IDENTIFIER "(" arguments? ")"
module_call → IDENTIFIER "." IDENTIFIER "(" arguments? ")"
arguments   → expr ("," expr)*
```

Grammar notes:
- Mutable discard bindings are rejected with parse errors because `_` is discard-only:
  - `let mut _ = expr`
  - `let mut _: type = expr`
- `if`/`while`/`return`/`break`/`continue` are parsed as statements.

## Statement Termination

Statements are terminated by:
- `Newline` token (consumed)
- `RightBrace` (not consumed, signals end of block)
- `Eof` (not consumed, signals end of file)

Multiple statements on the same line are not allowed.

## Newline Handling

Newlines are significant for statement termination but ignored in certain contexts:
- After `{` (opening brace)
- After `(` in function calls
- After `,` in argument lists
- Before `)` in function calls

Use `skip_newlines()` in these contexts.

## Error Types

`ParseError` contains:
- `message: String` - Human-readable description with helpful suggestions
- `span: Span` - Source location
- `kind: ParseErrorKind` - Structured error classification

Error messages include context-aware suggestions (e.g., "If this is a function call, add parentheses").

## Helper Methods

| Method | Description |
|--------|-------------|
| `current()` | Returns current token (safe, never panics) |
| `advance()` | Moves to next token (no-op at Eof) |
| `expect(kind)` | Asserts current token matches, advances |
| `expect_identifier()` | Expects identifier, returns name |
| `skip_newlines()` | Skips all consecutive newlines |
| `expect_statement_terminator()` | Expects newline, `}`, or Eof |
| `token_kind_display(kind)` | User-friendly token description |

## Span Tracking

Each AST node includes a `Span` for error reporting:
- `FnDef`: from `fn` keyword to before `{`
- `Stmt`: from first token to last token of statement
- `Expr`: from first token to last token of expression

## Extension Guidelines

1. New statements: add case in `parse_stmt()` and create `parse_*_stmt()` method
2. New expressions: add case in `parse_expr()` or create separate method
3. New types: add case in `parse_type()` in `types.rs`
4. New keywords: handle in appropriate `parse_*` method
5. Helper utilities: add to `helpers.rs`

## Invariants

- Token list is never empty (enforced by `Parser::new`)
- Last token is always `Eof`
- `advance()` does not move past `Eof`
