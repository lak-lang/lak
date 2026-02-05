//! Unit tests for the lexer.

use super::*;
use crate::token::TokenKind;

/// Helper function to tokenize input and return only the kinds.
fn tokenize_kinds(input: &str) -> Vec<TokenKind> {
    let mut lexer = Lexer::new(input);
    lexer
        .tokenize()
        .unwrap_or_else(|e| panic!("Tokenization failed for input {:?}: {}", input, e))
        .into_iter()
        .map(|t| t.kind)
        .collect()
}

/// Helper function to tokenize input and return the error.
fn tokenize_error(input: &str) -> LexError {
    let mut lexer = Lexer::new(input);
    match lexer.tokenize() {
        Ok(tokens) => panic!(
            "Expected tokenization to fail for input {:?}, but it succeeded with {} tokens",
            input,
            tokens.len()
        ),
        Err(e) => e,
    }
}

// ===================
// Basic tokens
// ===================

#[test]
fn test_empty_input() {
    let kinds = tokenize_kinds("");
    assert_eq!(kinds, vec![TokenKind::Eof]);
}

#[test]
fn test_whitespace_only() {
    let kinds = tokenize_kinds("   \n\t");
    assert_eq!(kinds, vec![TokenKind::Eof]);
}

#[test]
fn test_left_paren() {
    let kinds = tokenize_kinds("(");
    assert_eq!(kinds, vec![TokenKind::LeftParen, TokenKind::Eof]);
}

#[test]
fn test_right_paren() {
    let kinds = tokenize_kinds(")");
    assert_eq!(kinds, vec![TokenKind::RightParen, TokenKind::Eof]);
}

#[test]
fn test_comma() {
    let kinds = tokenize_kinds(",");
    assert_eq!(kinds, vec![TokenKind::Comma, TokenKind::Eof]);
}

#[test]
fn test_multiple_punctuation() {
    let kinds = tokenize_kinds("(,)");
    assert_eq!(
        kinds,
        vec![
            TokenKind::LeftParen,
            TokenKind::Comma,
            TokenKind::RightParen,
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_left_brace() {
    let kinds = tokenize_kinds("{");
    assert_eq!(kinds, vec![TokenKind::LeftBrace, TokenKind::Eof]);
}

#[test]
fn test_right_brace() {
    let kinds = tokenize_kinds("}");
    assert_eq!(kinds, vec![TokenKind::RightBrace, TokenKind::Eof]);
}

#[test]
fn test_arrow() {
    let kinds = tokenize_kinds("->");
    assert_eq!(kinds, vec![TokenKind::Arrow, TokenKind::Eof]);
}

#[test]
fn test_punctuation_with_spaces() {
    let kinds = tokenize_kinds("( , )");
    assert_eq!(
        kinds,
        vec![
            TokenKind::LeftParen,
            TokenKind::Comma,
            TokenKind::RightParen,
            TokenKind::Eof
        ]
    );
}

// ===================
// Identifiers
// ===================

#[test]
fn test_identifier_simple() {
    let kinds = tokenize_kinds("println");
    assert_eq!(
        kinds,
        vec![TokenKind::Identifier("println".to_string()), TokenKind::Eof]
    );
}

#[test]
fn test_identifier_with_underscore() {
    let kinds = tokenize_kinds("my_func");
    assert_eq!(
        kinds,
        vec![TokenKind::Identifier("my_func".to_string()), TokenKind::Eof]
    );
}

#[test]
fn test_identifier_starts_with_underscore() {
    let kinds = tokenize_kinds("_private");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Identifier("_private".to_string()),
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_identifier_with_numbers() {
    let kinds = tokenize_kinds("func123");
    assert_eq!(
        kinds,
        vec![TokenKind::Identifier("func123".to_string()), TokenKind::Eof]
    );
}

#[test]
fn test_identifier_underscore_only() {
    let kinds = tokenize_kinds("_");
    assert_eq!(
        kinds,
        vec![TokenKind::Identifier("_".to_string()), TokenKind::Eof]
    );
}

#[test]
fn test_multiple_identifiers() {
    let kinds = tokenize_kinds("foo bar");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Identifier("foo".to_string()),
            TokenKind::Identifier("bar".to_string()),
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_identifier_unicode() {
    let kinds = tokenize_kinds("日本語");
    assert_eq!(
        kinds,
        vec![TokenKind::Identifier("日本語".to_string()), TokenKind::Eof]
    );
}

// ===================
// Keywords
// ===================

#[test]
fn test_keyword_fn() {
    let kinds = tokenize_kinds("fn");
    assert_eq!(kinds, vec![TokenKind::Fn, TokenKind::Eof]);
}

#[test]
fn test_fn_not_prefix() {
    // "fn_test" should be an identifier, not fn + identifier
    let kinds = tokenize_kinds("fn_test");
    assert_eq!(
        kinds,
        vec![TokenKind::Identifier("fn_test".to_string()), TokenKind::Eof]
    );
}

#[test]
fn test_keyword_let() {
    let kinds = tokenize_kinds("let");
    assert_eq!(kinds, vec![TokenKind::Let, TokenKind::Eof]);
}

#[test]
fn test_let_not_prefix() {
    // "letter" should be an identifier, not let + identifier
    let kinds = tokenize_kinds("letter");
    assert_eq!(
        kinds,
        vec![TokenKind::Identifier("letter".to_string()), TokenKind::Eof]
    );
}

#[test]
fn test_colon() {
    let kinds = tokenize_kinds(":");
    assert_eq!(kinds, vec![TokenKind::Colon, TokenKind::Eof]);
}

#[test]
fn test_equals() {
    let kinds = tokenize_kinds("=");
    assert_eq!(kinds, vec![TokenKind::Equals, TokenKind::Eof]);
}

#[test]
fn test_function_definition_tokens() {
    let kinds = tokenize_kinds("fn main() -> void {}");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Fn,
            TokenKind::Identifier("main".to_string()),
            TokenKind::LeftParen,
            TokenKind::RightParen,
            TokenKind::Arrow,
            TokenKind::Identifier("void".to_string()),
            TokenKind::LeftBrace,
            TokenKind::RightBrace,
            TokenKind::Eof
        ]
    );
}

// ===================
// String literals
// ===================

#[test]
fn test_string_empty() {
    let kinds = tokenize_kinds(r#""""#);
    assert_eq!(
        kinds,
        vec![TokenKind::StringLiteral("".to_string()), TokenKind::Eof]
    );
}

#[test]
fn test_string_simple() {
    let kinds = tokenize_kinds(r#""hello""#);
    assert_eq!(
        kinds,
        vec![
            TokenKind::StringLiteral("hello".to_string()),
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_string_with_spaces() {
    let kinds = tokenize_kinds(r#""hello world""#);
    assert_eq!(
        kinds,
        vec![
            TokenKind::StringLiteral("hello world".to_string()),
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_string_escape_newline() {
    let kinds = tokenize_kinds(r#""a\nb""#);
    assert_eq!(
        kinds,
        vec![TokenKind::StringLiteral("a\nb".to_string()), TokenKind::Eof]
    );
}

#[test]
fn test_string_escape_tab() {
    let kinds = tokenize_kinds(r#""a\tb""#);
    assert_eq!(
        kinds,
        vec![TokenKind::StringLiteral("a\tb".to_string()), TokenKind::Eof]
    );
}

#[test]
fn test_string_escape_carriage_return() {
    let kinds = tokenize_kinds(r#""a\rb""#);
    assert_eq!(
        kinds,
        vec![TokenKind::StringLiteral("a\rb".to_string()), TokenKind::Eof]
    );
}

#[test]
fn test_string_escape_backslash() {
    let kinds = tokenize_kinds(r#""a\\b""#);
    assert_eq!(
        kinds,
        vec![TokenKind::StringLiteral("a\\b".to_string()), TokenKind::Eof]
    );
}

#[test]
fn test_string_escape_quote() {
    let kinds = tokenize_kinds(r#""a\"b""#);
    assert_eq!(
        kinds,
        vec![TokenKind::StringLiteral("a\"b".to_string()), TokenKind::Eof]
    );
}

#[test]
fn test_string_multiple_escapes() {
    let kinds = tokenize_kinds(r#""\n\t\r""#);
    assert_eq!(
        kinds,
        vec![
            TokenKind::StringLiteral("\n\t\r".to_string()),
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_string_all_escapes_combined() {
    let kinds = tokenize_kinds(r#""\n\t\r\\\"end""#);
    assert_eq!(
        kinds,
        vec![
            TokenKind::StringLiteral("\n\t\r\\\"end".to_string()),
            TokenKind::Eof
        ]
    );
}

// ===================
// Comments
// ===================

#[test]
fn test_comment_single_line() {
    let kinds = tokenize_kinds("// comment\n");
    assert_eq!(kinds, vec![TokenKind::Eof]);
}

#[test]
fn test_comment_at_eof() {
    let kinds = tokenize_kinds("// comment");
    assert_eq!(kinds, vec![TokenKind::Eof]);
}

#[test]
fn test_comment_after_code() {
    let kinds = tokenize_kinds("println // comment\n");
    assert_eq!(
        kinds,
        vec![TokenKind::Identifier("println".to_string()), TokenKind::Eof]
    );
}

#[test]
fn test_comment_between_tokens() {
    let kinds = tokenize_kinds("a // c\nb");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Identifier("a".to_string()),
            TokenKind::Identifier("b".to_string()),
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_multiple_comments() {
    let kinds = tokenize_kinds("// first\n// second\nfoo");
    assert_eq!(
        kinds,
        vec![TokenKind::Identifier("foo".to_string()), TokenKind::Eof]
    );
}

// ===================
// Compound expressions
// ===================

#[test]
fn test_println_call() {
    let kinds = tokenize_kinds(r#"println("hello")"#);
    assert_eq!(
        kinds,
        vec![
            TokenKind::Identifier("println".to_string()),
            TokenKind::LeftParen,
            TokenKind::StringLiteral("hello".to_string()),
            TokenKind::RightParen,
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_nested_call() {
    let kinds = tokenize_kinds(r#"outer(inner("x"))"#);
    assert_eq!(
        kinds,
        vec![
            TokenKind::Identifier("outer".to_string()),
            TokenKind::LeftParen,
            TokenKind::Identifier("inner".to_string()),
            TokenKind::LeftParen,
            TokenKind::StringLiteral("x".to_string()),
            TokenKind::RightParen,
            TokenKind::RightParen,
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_multiple_args() {
    let kinds = tokenize_kinds(r#"func("a", "b", "c")"#);
    assert_eq!(
        kinds,
        vec![
            TokenKind::Identifier("func".to_string()),
            TokenKind::LeftParen,
            TokenKind::StringLiteral("a".to_string()),
            TokenKind::Comma,
            TokenKind::StringLiteral("b".to_string()),
            TokenKind::Comma,
            TokenKind::StringLiteral("c".to_string()),
            TokenKind::RightParen,
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_multiple_statements() {
    let kinds = tokenize_kinds("foo()\nbar()");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Identifier("foo".to_string()),
            TokenKind::LeftParen,
            TokenKind::RightParen,
            TokenKind::Identifier("bar".to_string()),
            TokenKind::LeftParen,
            TokenKind::RightParen,
            TokenKind::Eof
        ]
    );
}

// ===================
// Span verification
// ===================

#[test]
fn test_span_positions() {
    let mut lexer = Lexer::new("foo");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].span.start, 0);
    assert_eq!(tokens[0].span.end, 3);
}

#[test]
fn test_span_line_column() {
    let mut lexer = Lexer::new("foo");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].span.line, 1);
    assert_eq!(tokens[0].span.column, 1);
}

#[test]
fn test_span_multiline() {
    let mut lexer = Lexer::new("a\nb");
    let tokens = lexer.tokenize().unwrap();

    // First token 'a' on line 1
    assert_eq!(tokens[0].span.line, 1);
    assert_eq!(tokens[0].span.column, 1);

    // Second token 'b' on line 2
    assert_eq!(tokens[1].span.line, 2);
    assert_eq!(tokens[1].span.column, 1);
}

#[test]
fn test_span_string_literal() {
    let mut lexer = Lexer::new(r#""hello""#);
    let tokens = lexer.tokenize().unwrap();

    // String includes quotes in span
    assert_eq!(tokens[0].span.start, 0);
    assert_eq!(tokens[0].span.end, 7); // includes both quotes
}

#[test]
fn test_span_after_whitespace() {
    let mut lexer = Lexer::new("   foo");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].span.start, 3);
    assert_eq!(tokens[0].span.end, 6);
    assert_eq!(tokens[0].span.column, 4);
}

#[test]
fn test_arrow_span() {
    let mut lexer = Lexer::new("->");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].span.start, 0);
    assert_eq!(tokens[0].span.end, 2); // `->` is 2 characters
    assert_eq!(tokens[0].span.line, 1);
    assert_eq!(tokens[0].span.column, 1);
}

#[test]
fn test_brace_span() {
    let mut lexer = Lexer::new("{ }");
    let tokens = lexer.tokenize().unwrap();

    // Left brace
    assert_eq!(tokens[0].span.start, 0);
    assert_eq!(tokens[0].span.end, 1);

    // Right brace
    assert_eq!(tokens[1].span.start, 2);
    assert_eq!(tokens[1].span.end, 3);
}

// ===================
// Error cases
// ===================

#[test]
fn test_error_unterminated_string() {
    let err = tokenize_error(r#""hello"#);
    assert!(err.message.contains("Unterminated string"));
}

#[test]
fn test_error_newline_in_string() {
    let err = tokenize_error("\"hello\nworld\"");
    assert!(err.message.contains("newline in string"));
}

#[test]
fn test_error_unknown_escape() {
    let err = tokenize_error(r#""\x""#);
    assert!(err.message.contains("Unknown escape sequence"));
}

#[test]
fn test_error_unexpected_char_at() {
    let err = tokenize_error("@");
    assert!(err.message.contains("Unexpected character"));
}

#[test]
fn test_error_unexpected_char_hash() {
    let err = tokenize_error("#");
    assert!(err.message.contains("Unexpected character"));
}

// ===================
// Integer literals
// ===================

#[test]
fn test_integer_literal_simple() {
    let kinds = tokenize_kinds("123");
    assert_eq!(kinds, vec![TokenKind::IntLiteral(123), TokenKind::Eof]);
}

#[test]
fn test_integer_literal_zero() {
    let kinds = tokenize_kinds("0");
    assert_eq!(kinds, vec![TokenKind::IntLiteral(0), TokenKind::Eof]);
}

#[test]
fn test_integer_literal_large() {
    let kinds = tokenize_kinds("9223372036854775807");
    assert_eq!(
        kinds,
        vec![TokenKind::IntLiteral(9223372036854775807), TokenKind::Eof]
    );
}

#[test]
fn test_let_statement_tokens() {
    let kinds = tokenize_kinds("let x: i32 = 42");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Let,
            TokenKind::Identifier("x".to_string()),
            TokenKind::Colon,
            TokenKind::Identifier("i32".to_string()),
            TokenKind::Equals,
            TokenKind::IntLiteral(42),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn test_error_unexpected_char_dollar() {
    let err = tokenize_error("$");
    assert!(err.message.contains("Unexpected character"));
}

#[test]
fn test_error_minus_without_arrow() {
    let err = tokenize_error("-");
    assert!(err.message.contains("Expected '>' after '-'"));
}

#[test]
fn test_error_minus_with_other_char() {
    let err = tokenize_error("-x");
    assert!(err.message.contains("Expected '>' after '-'"));
}

#[test]
fn test_error_span_location() {
    let err = tokenize_error("foo @");
    assert_eq!(err.span.start, 4);
    assert_eq!(err.span.column, 5);
}

#[test]
fn test_lex_error_display() {
    use crate::token::Span;
    let err = LexError {
        message: "Test error".to_string(),
        span: Span::new(0, 1, 2, 3),
    };
    let display = format!("{}", err);
    assert!(display.contains("2:3"));
    assert!(display.contains("Test error"));
}

// ===================
// Edge cases
// ===================

#[test]
fn test_error_escape_at_eof() {
    let err = tokenize_error(r#""hello\"#);
    assert!(err.message.contains("Unterminated string"));
}

#[test]
fn test_windows_line_endings() {
    let kinds = tokenize_kinds("a\r\nb");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Identifier("a".to_string()),
            TokenKind::Identifier("b".to_string()),
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_multiple_consecutive_backslashes() {
    let kinds = tokenize_kinds(r#""\\\\""#);
    assert_eq!(
        kinds,
        vec![TokenKind::StringLiteral("\\\\".to_string()), TokenKind::Eof]
    );
}

#[test]
fn test_multiple_consecutive_quotes() {
    let kinds = tokenize_kinds(r#""\"\"""#);
    assert_eq!(
        kinds,
        vec![TokenKind::StringLiteral("\"\"".to_string()), TokenKind::Eof]
    );
}

#[test]
fn test_integer_literal_leading_zeros() {
    // Leading zeros are treated as decimal (not octal)
    let kinds = tokenize_kinds("007");
    assert_eq!(kinds, vec![TokenKind::IntLiteral(7), TokenKind::Eof]);
}

#[test]
fn test_integer_literal_all_zeros() {
    let kinds = tokenize_kinds("000");
    assert_eq!(kinds, vec![TokenKind::IntLiteral(0), TokenKind::Eof]);
}
