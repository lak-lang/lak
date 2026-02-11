//! Tests for keyword recognition and disambiguation from identifiers.

use super::*;

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
fn test_keyword_if() {
    let kinds = tokenize_kinds("if");
    assert_eq!(kinds, vec![TokenKind::If, TokenKind::Eof]);
}

#[test]
fn test_if_not_prefix() {
    let kinds = tokenize_kinds("if_else");
    assert_eq!(
        kinds,
        vec![TokenKind::Identifier("if_else".to_string()), TokenKind::Eof]
    );
}

#[test]
fn test_keyword_else() {
    let kinds = tokenize_kinds("else");
    assert_eq!(kinds, vec![TokenKind::Else, TokenKind::Eof]);
}

#[test]
fn test_else_not_prefix() {
    let kinds = tokenize_kinds("elseif");
    assert_eq!(
        kinds,
        vec![TokenKind::Identifier("elseif".to_string()), TokenKind::Eof]
    );
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

#[test]
fn test_keyword_true() {
    let kinds = tokenize_kinds("true");
    assert_eq!(kinds, vec![TokenKind::BoolLiteral(true), TokenKind::Eof]);
}

#[test]
fn test_keyword_false() {
    let kinds = tokenize_kinds("false");
    assert_eq!(kinds, vec![TokenKind::BoolLiteral(false), TokenKind::Eof]);
}

#[test]
fn test_true_not_prefix() {
    // "trueish" should be an identifier, not true + identifier
    let kinds = tokenize_kinds("trueish");
    assert_eq!(
        kinds,
        vec![TokenKind::Identifier("trueish".to_string()), TokenKind::Eof]
    );
}

#[test]
fn test_false_not_prefix() {
    // "falsetto" should be an identifier, not false + identifier
    let kinds = tokenize_kinds("falsetto");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Identifier("falsetto".to_string()),
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_keyword_pub() {
    let kinds = tokenize_kinds("pub");
    assert_eq!(kinds, vec![TokenKind::Pub, TokenKind::Eof]);
}

#[test]
fn test_pub_not_prefix() {
    // "public" should be an identifier, not pub + identifier
    let kinds = tokenize_kinds("public");
    assert_eq!(
        kinds,
        vec![TokenKind::Identifier("public".to_string()), TokenKind::Eof]
    );
}

#[test]
fn test_keyword_import() {
    let kinds = tokenize_kinds("import");
    assert_eq!(kinds, vec![TokenKind::Import, TokenKind::Eof]);
}

#[test]
fn test_import_not_prefix() {
    // "imports" should be an identifier, not import + identifier
    let kinds = tokenize_kinds("imports");
    assert_eq!(
        kinds,
        vec![TokenKind::Identifier("imports".to_string()), TokenKind::Eof]
    );
}

#[test]
fn test_keyword_as() {
    let kinds = tokenize_kinds("as");
    assert_eq!(kinds, vec![TokenKind::As, TokenKind::Eof]);
}

#[test]
fn test_as_not_prefix() {
    // "assign" should be an identifier, not as + identifier
    let kinds = tokenize_kinds("assign");
    assert_eq!(
        kinds,
        vec![TokenKind::Identifier("assign".to_string()), TokenKind::Eof]
    );
}

#[test]
fn test_pub_fn_tokens() {
    let kinds = tokenize_kinds("pub fn test() -> void {}");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Pub,
            TokenKind::Fn,
            TokenKind::Identifier("test".to_string()),
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

#[test]
fn test_import_statement_tokens() {
    let kinds = tokenize_kinds(r#"import "math" as m"#);
    assert_eq!(
        kinds,
        vec![
            TokenKind::Import,
            TokenKind::StringLiteral("math".to_string()),
            TokenKind::As,
            TokenKind::Identifier("m".to_string()),
            TokenKind::Eof
        ]
    );
}
