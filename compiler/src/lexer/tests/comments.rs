//! Tests for comment handling and interaction with newlines.

use super::*;

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
fn test_identifier_then_comment_at_eof_no_newline() {
    // No Newline token should be emitted because there's no actual newline in the input
    let kinds = tokenize_kinds("foo // comment without newline");
    assert_eq!(
        kinds,
        vec![TokenKind::Identifier("foo".to_string()), TokenKind::Eof]
    );
}

#[test]
fn test_comment_after_code() {
    // Comment consumes the rest of the line, but the newline at the end
    // still emits a Newline token (because the last real token was an identifier)
    let kinds = tokenize_kinds("println // comment\n");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Identifier("println".to_string()),
            TokenKind::Newline,
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_comment_between_tokens() {
    // Newline after comment on line with identifier emits Newline
    let kinds = tokenize_kinds("a // c\nb");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Identifier("a".to_string()),
            TokenKind::Newline,
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
