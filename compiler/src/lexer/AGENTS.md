# lexer Module

Lexical analysis (tokenization) of Lak source code.

## Overview

Converts source code text into a stream of `Token`s for the parser. Handles character-by-character scanning, source position tracking, and automatic newline insertion.

## Module Structure

| File | Responsibility |
|------|----------------|
| `mod.rs` | `Lexer` struct, `tokenize()` entry point |
| `error.rs` | `LexError` type |
| `cursor.rs` | Position tracking (`current_char`, `advance`, `is_eof`) |
| `skip.rs` | Whitespace/comment skipping, newline emission logic |
| `tokens.rs` | Token recognition (`next_token`, `read_*` methods) |
| `tests/` | Unit tests (see below) |

## Test Structure

Tests are organized by category in `tests/` directory:

| File | Coverage |
|------|----------|
| `mod.rs` | Helper functions (`tokenize_kinds`, `tokenize_error`) |
| `basic_tokens.rs` | Punctuation, braces, arrow |
| `identifiers.rs` | Identifier parsing (ASCII only, rejects Unicode) |
| `keywords.rs` | Keyword recognition and identifier disambiguation |
| `strings.rs` | String literals and escape sequences |
| `integers.rs` | Integer and float literal parsing |
| `comments.rs` | Comment handling |
| `compound.rs` | Function calls, nested expressions |
| `newlines.rs` | Newline token emission rules |
| `spans.rs` | Position tracking verification |
| `errors.rs` | Error cases |
| `edge_cases.rs` | Corner cases, platform compatibility |
| `whitespace.rs` | Whitespace handling (ASCII vs non-ASCII whitespace) |

## Supported Tokens

| Category | Tokens |
|----------|--------|
| Keywords | `fn`, `let`, `mut`, `if`, `else`, `return`, `while`, `break`, `continue`, `pub`, `import`, `as` |
| Identifiers | ASCII alphabetic (a-z, A-Z)/underscore start, ASCII alphanumeric/underscore continue (non-ASCII rejected) |
| Integer literals | ASCII digit sequences, stored as `u64` (overflow → error) |
| Float literals | `digits '.' digits`, stored as `f64` |
| Boolean literals | `true`, `false` |
| String literals | Double-quoted, supports `\n`, `\t`, `\r`, `\\`, `\"` |
| Punctuation / Operators | `(`, `)`, `{`, `}`, `,`, `:`, `.`, `=`, `->`, `+`, `-`, `*`, `/`, `%`, `!`, `&&`, `||`, `==`, `!=`, `<`, `>`, `<=`, `>=` |
| Special | `Newline` (auto-inserted), `Eof` |

## Automatic Newline Insertion

Inspired by Go's semicolon insertion. `Newline` tokens are emitted after:
- Identifiers
- Literals (string, integer, float, boolean)
- `return`, `break`, `continue`
- `)` (right parenthesis)
- `}` (right brace)

This enables statement termination without explicit semicolons.

## Error Types

`LexError` contains:
- `message: String` - Human-readable description
- `span: Span` - Source location
- `kind: LexErrorKind` - Structured error classification

Errors occur for:
- Unexpected characters
- Unterminated string literals
- Unknown escape sequences
- Integer overflow (exceeds `u64::MAX`)
- Invalid float literals
- Invalid non-ASCII whitespace characters

## Lifetime `'a`

```rust
pub struct Lexer<'a> { ... }
```

`'a` ties the lexer to the input string slice. The input must remain valid while tokenizing.

## Position Tracking

- `pos: usize` - Byte position in input
- `line: usize` - Line number (1-indexed)
- `column: usize` - Column number (1-indexed)
- Handles multi-byte UTF-8 characters correctly

## Extension Guidelines

1. New keywords: add pattern in `read_identifier()` in `tokens.rs`
2. New punctuation: add case in `next_token()` in `tokens.rs`
3. New literal types: add `read_*` method in `tokens.rs`
4. Newline insertion rules: modify `should_emit_newline()` in `skip.rs`
