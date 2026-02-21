//! Token kinds for the Lak lexer.
//!
//! This module defines the [`TokenKind`] enum representing all possible
//! token types in the Lak language.

/// The kind of token recognized by the lexer.
///
/// This enum represents all possible token types in the Lak language.
/// Each variant may carry associated data (e.g., the actual string value
/// for identifiers and string literals).
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    /// An identifier (function name, variable name, etc.).
    ///
    /// Identifiers start with an ASCII alphabetic character (a-z, A-Z) or underscore,
    /// followed by any number of ASCII alphanumeric characters (a-z, A-Z, 0-9) or underscores.
    /// Non-ASCII Unicode characters are explicitly rejected by the lexer.
    Identifier(String),

    /// A string literal enclosed in double quotes.
    ///
    /// The contained `String` is the unescaped value (escape sequences
    /// like `\n` are already converted to their actual characters).
    StringLiteral(String),

    /// The `fn` keyword for function definitions.
    Fn,

    /// The `let` keyword for variable declarations.
    Let,

    /// The `mut` keyword for mutable variable declarations.
    Mut,

    /// The `if` keyword for conditional branching.
    If,

    /// The `else` keyword for alternate conditional branch.
    Else,

    /// The `return` keyword for returning from functions.
    Return,

    /// The `while` keyword for loop statements.
    While,

    /// The `break` keyword for exiting loops.
    Break,

    /// The `continue` keyword for continuing to the next loop iteration.
    Continue,

    /// The `pub` keyword for public visibility.
    Pub,

    /// The `import` keyword for module imports.
    Import,

    /// The `as` keyword for import aliases.
    As,

    /// A left parenthesis `(`.
    LeftParen,

    /// A right parenthesis `)`.
    RightParen,

    /// A left brace `{`.
    LeftBrace,

    /// A right brace `}`.
    RightBrace,

    /// An arrow `->` for return type annotation.
    Arrow,

    /// A comma `,`.
    Comma,

    /// A colon `:` for type annotation.
    Colon,

    /// A dot `.` for member access.
    Dot,

    /// An equals sign `=` for variable initialization in let statements.
    Equals,

    /// A plus sign `+` for addition.
    Plus,

    /// A minus sign `-` for subtraction.
    Minus,

    /// An asterisk `*` for multiplication.
    Star,

    /// A forward slash `/` for division.
    Slash,

    /// A percent sign `%` for modulo.
    Percent,

    /// A bang (exclamation mark) `!`.
    Bang,

    /// Double ampersand `&&` for logical AND.
    AndAnd,

    /// Double pipe `||` for logical OR.
    OrOr,

    /// Double equals `==` for equality comparison.
    EqualEqual,

    /// Not equals `!=` for inequality comparison.
    BangEqual,

    /// Less than `<` for comparison.
    LessThan,

    /// Greater than `>` for comparison.
    GreaterThan,

    /// Less than or equal `<=` for comparison.
    LessEqual,

    /// Greater than or equal `>=` for comparison.
    GreaterEqual,

    /// An integer literal (e.g., 42, 100).
    IntLiteral(u64),

    /// A boolean literal (`true` or `false`).
    BoolLiteral(bool),

    /// A newline that acts as a statement terminator.
    ///
    /// Only emitted after certain tokens (identifiers, literals, `return`,
    /// `break`, `continue`, `)`, `}`) following Go-style automatic semicolon
    /// insertion rules. Newlines in other contexts are skipped and do not
    /// produce tokens.
    Newline,

    /// End of file marker.
    ///
    /// The lexer always appends this as the final token, ensuring
    /// the token vector is never empty.
    Eof,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_kind_identifier() {
        let kind = TokenKind::Identifier("my_func".to_string());
        assert!(matches!(kind, TokenKind::Identifier(ref s) if s == "my_func"));
    }

    #[test]
    fn test_token_kind_string_literal() {
        let kind = TokenKind::StringLiteral("hello world".to_string());
        assert!(matches!(kind, TokenKind::StringLiteral(ref s) if s == "hello world"));
    }

    #[test]
    fn test_token_kind_punctuation() {
        assert!(matches!(TokenKind::LeftParen, TokenKind::LeftParen));
        assert!(matches!(TokenKind::RightParen, TokenKind::RightParen));
        assert!(matches!(TokenKind::LeftBrace, TokenKind::LeftBrace));
        assert!(matches!(TokenKind::RightBrace, TokenKind::RightBrace));
        assert!(matches!(TokenKind::Comma, TokenKind::Comma));
        assert!(matches!(TokenKind::Arrow, TokenKind::Arrow));
    }

    #[test]
    fn test_token_kind_fn() {
        assert!(matches!(TokenKind::Fn, TokenKind::Fn));
    }

    #[test]
    fn test_token_kind_let() {
        assert!(matches!(TokenKind::Let, TokenKind::Let));
    }

    #[test]
    fn test_token_kind_mut() {
        assert!(matches!(TokenKind::Mut, TokenKind::Mut));
    }

    #[test]
    fn test_token_kind_while() {
        assert!(matches!(TokenKind::While, TokenKind::While));
    }

    #[test]
    fn test_token_kind_break() {
        assert!(matches!(TokenKind::Break, TokenKind::Break));
    }

    #[test]
    fn test_token_kind_continue() {
        assert!(matches!(TokenKind::Continue, TokenKind::Continue));
    }

    #[test]
    fn test_token_kind_colon() {
        assert!(matches!(TokenKind::Colon, TokenKind::Colon));
    }

    #[test]
    fn test_token_kind_equals() {
        assert!(matches!(TokenKind::Equals, TokenKind::Equals));
    }

    #[test]
    fn test_token_kind_int_literal() {
        let kind = TokenKind::IntLiteral(42);
        assert!(matches!(kind, TokenKind::IntLiteral(42)));
    }

    #[test]
    fn test_token_kind_eof() {
        assert!(matches!(TokenKind::Eof, TokenKind::Eof));
    }

    #[test]
    fn test_token_kind_partial_eq() {
        let kind1 = TokenKind::Identifier("foo".to_string());
        let kind2 = TokenKind::Identifier("foo".to_string());
        let kind3 = TokenKind::Identifier("bar".to_string());

        assert_eq!(kind1, kind2);
        assert_ne!(kind1, kind3);
        assert_eq!(TokenKind::LeftParen, TokenKind::LeftParen);
        assert_ne!(TokenKind::LeftParen, TokenKind::RightParen);
    }
}
