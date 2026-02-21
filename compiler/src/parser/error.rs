//! Parse error types.
//!
//! This module defines [`ParseError`], which represents errors that can occur
//! during parsing.
//!
//! # Helper Constructors
//!
//! This module provides specialized constructor methods for common error cases,
//! ensuring consistent error messaging across the compiler. Prefer using these
//! helpers over constructing errors manually with [`ParseError::new()`].
//!
//! Available helper methods are organized by category:
//! - **Statement termination**: [`missing_statement_terminator()`](ParseError::missing_statement_terminator)
//! - **Token expectation**: [`unexpected_token()`](ParseError::unexpected_token),
//!   [`expected_identifier()`](ParseError::expected_identifier),
//!   [`unknown_type()`](ParseError::unknown_type)
//! - **Function call syntax**: [`missing_fn_call_parens_string()`](ParseError::missing_fn_call_parens_string),
//!   [`missing_fn_call_parens_int()`](ParseError::missing_fn_call_parens_int),
//!   [`missing_fn_call_parens_ident()`](ParseError::missing_fn_call_parens_ident)
//! - **Expression errors**: [`unexpected_expression_start()`](ParseError::unexpected_expression_start)
//! - **Unsupported syntax**: [`invalid_mutable_discard()`](ParseError::invalid_mutable_discard),
//!   [`invalid_typed_discard()`](ParseError::invalid_typed_discard)

use crate::token::Span;

/// The kind of parse error.
///
/// This enum allows error handling code to match on error types structurally
/// rather than relying on string matching, which is fragile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseErrorKind {
    /// Expected a statement terminator (newline, '}', or Eof).
    MissingStatementTerminator,
    /// Expected a specific token but found something else.
    UnexpectedToken,
    /// Expected an identifier but found something else.
    ExpectedIdentifier,
    /// Expected a string literal but found something else.
    ExpectedStringLiteral,
    /// Expected a type annotation but found something else.
    ExpectedType,
    /// Expression following identifier without parentheses (likely missing function call syntax).
    MissingFunctionCallParentheses,
    /// Nested member access (e.g., a.b.c) is not yet supported.
    NestedMemberAccessNotSupported,
    /// Empty import path is not allowed.
    EmptyImportPath,
    /// Integer literal exceeds representable range.
    IntegerLiteralOutOfRange,
    /// `if` expression is missing required `else` branch.
    MissingElseInIfExpression,
    /// `if` expression branch does not end with a value expression.
    MissingIfExpressionBranchValue,
    /// Internal parser inconsistency (compiler bug).
    InternalError,
}

/// An error that occurred during parsing.
///
/// `ParseError` contains a human-readable message and the source location
/// where the error occurred, enabling rich error reporting.
///
/// # Construction
///
/// Use [`new()`](Self::new) to create a new error with a specific source location.
///
/// # See Also
///
/// * [`crate::lexer::LexError`] - Similar error type for lexical errors
#[derive(Debug)]
pub struct ParseError {
    /// A human-readable description of the error.
    message: String,
    /// The source location where the error occurred.
    span: Span,
    /// The kind of error, for structured error handling.
    kind: ParseErrorKind,
}

impl ParseError {
    /// Creates a new parse error.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind of error
    /// * `message` - A human-readable description of the error
    /// * `span` - The source location where the error occurred
    pub fn new(kind: ParseErrorKind, message: impl Into<String>, span: Span) -> Self {
        ParseError {
            message: message.into(),
            span,
            kind,
        }
    }

    /// Returns the error message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the source location where the error occurred.
    pub fn span(&self) -> Span {
        self.span
    }

    /// Returns the kind of error.
    pub fn kind(&self) -> ParseErrorKind {
        self.kind
    }

    /// Returns a short, human-readable description of the error kind.
    ///
    /// This is used as the report title in error messages, while `message()`
    /// provides the detailed explanation shown in the label.
    pub fn short_message(&self) -> &'static str {
        match self.kind {
            ParseErrorKind::MissingStatementTerminator => "Missing statement terminator",
            ParseErrorKind::UnexpectedToken => "Unexpected token",
            ParseErrorKind::ExpectedIdentifier => "Expected identifier",
            ParseErrorKind::ExpectedStringLiteral => "Expected string literal",
            ParseErrorKind::ExpectedType => "Unknown type",
            ParseErrorKind::MissingFunctionCallParentheses => "Missing function call parentheses",
            ParseErrorKind::NestedMemberAccessNotSupported => "Nested member access not supported",
            ParseErrorKind::EmptyImportPath => "Empty import path",
            ParseErrorKind::IntegerLiteralOutOfRange => "Integer overflow",
            ParseErrorKind::MissingElseInIfExpression => "Missing else in if expression",
            ParseErrorKind::MissingIfExpressionBranchValue => {
                "Missing branch value in if expression"
            }
            ParseErrorKind::InternalError => "Internal error",
        }
    }

    // =========================================================================
    // Statement termination errors
    // =========================================================================

    /// Creates an error for missing statement terminator.
    pub fn missing_statement_terminator(found: &str, span: Span) -> Self {
        Self::new(
            ParseErrorKind::MissingStatementTerminator,
            format!("Expected newline after statement, found {}", found),
            span,
        )
    }

    // =========================================================================
    // Token expectation errors
    // =========================================================================

    /// Creates an error for unexpected token.
    pub fn unexpected_token(expected: &str, found: &str, span: Span) -> Self {
        Self::new(
            ParseErrorKind::UnexpectedToken,
            format!("Expected {}, found {}", expected, found),
            span,
        )
    }

    /// Creates an error for expected identifier.
    pub fn expected_identifier(found: &str, span: Span) -> Self {
        Self::new(
            ParseErrorKind::ExpectedIdentifier,
            format!("Expected identifier, found {}", found),
            span,
        )
    }

    /// Creates an error for expected string literal.
    pub fn expected_string_literal(found: &str, span: Span) -> Self {
        Self::new(
            ParseErrorKind::ExpectedStringLiteral,
            format!("Expected string literal for import path, found {}", found),
            span,
        )
    }

    /// Creates an error for unknown type.
    pub fn unknown_type(name: &str, span: Span) -> Self {
        Self::new(
            ParseErrorKind::ExpectedType,
            format!(
                "Unknown type: '{}'. Expected 'i32', 'i64', 'string', or 'bool'",
                name
            ),
            span,
        )
    }

    // =========================================================================
    // Function call syntax errors
    // =========================================================================

    /// Creates an error for missing function call parentheses (followed by string).
    pub fn missing_fn_call_parens_string(fn_name: &str, span: Span) -> Self {
        Self::new(
            ParseErrorKind::MissingFunctionCallParentheses,
            format!(
                "Unexpected string literal after '{}'. If this is a function call, add parentheses: {}(...)",
                fn_name, fn_name
            ),
            span,
        )
    }

    /// Creates an error for missing function call parentheses (followed by integer).
    pub fn missing_fn_call_parens_int(fn_name: &str, span: Span) -> Self {
        Self::new(
            ParseErrorKind::MissingFunctionCallParentheses,
            format!(
                "Unexpected integer literal after '{}'. If this is a function call, add parentheses: {}(...)",
                fn_name, fn_name
            ),
            span,
        )
    }

    /// Creates an error for missing function call parentheses (followed by identifier).
    pub fn missing_fn_call_parens_ident(fn_name: &str, next: &str, span: Span) -> Self {
        Self::new(
            ParseErrorKind::MissingFunctionCallParentheses,
            format!(
                "Unexpected identifier '{}' after '{}'. If this is a function call, add parentheses: {}(...)",
                next, fn_name, fn_name
            ),
            span,
        )
    }

    // =========================================================================
    // Expression errors
    // =========================================================================

    /// Creates an error for unexpected token at expression start.
    pub fn unexpected_expression_start(found: &str, span: Span) -> Self {
        Self::new(
            ParseErrorKind::UnexpectedToken,
            format!("Unexpected token: {}", found),
            span,
        )
    }

    // =========================================================================
    // Unsupported syntax errors
    // =========================================================================

    /// Creates an error for an `if` expression without `else`.
    pub fn missing_else_in_if_expression(span: Span) -> Self {
        Self::new(
            ParseErrorKind::MissingElseInIfExpression,
            "if expression requires an else branch",
            span,
        )
    }

    /// Creates an error for an `if` expression branch without a value expression.
    pub fn missing_if_expression_branch_value(branch_name: &str, span: Span) -> Self {
        Self::new(
            ParseErrorKind::MissingIfExpressionBranchValue,
            format!(
                "if expression {} branch must end with a value expression",
                branch_name
            ),
            span,
        )
    }

    /// Creates an error for nested member access (e.g., a.b.c).
    ///
    /// Nested member access is not yet supported. Only simple module.function
    /// access is allowed.
    pub fn nested_member_access_not_supported(span: Span) -> Self {
        Self::new(
            ParseErrorKind::NestedMemberAccessNotSupported,
            "Nested member access (e.g., a.b.c) is not yet supported. Only simple module.function access is allowed.",
            span,
        )
    }

    /// Creates an error for invalid mutable discard binding.
    ///
    /// `let mut _ = expr` is rejected because `_` discards values and has no
    /// storage to mutate.
    pub fn invalid_mutable_discard(span: Span) -> Self {
        Self::new(
            ParseErrorKind::UnexpectedToken,
            "Discard binding '_' cannot be declared mutable",
            span,
        )
    }

    /// Creates an error for invalid typed discard binding.
    ///
    /// `let _: type = expr` is rejected because `_` is discard-only and must
    /// be used as `let _ = expr`.
    pub fn invalid_typed_discard(span: Span) -> Self {
        Self::new(
            ParseErrorKind::UnexpectedToken,
            "Discard binding '_' cannot have a type annotation; use `let _ = expr`",
            span,
        )
    }

    /// Creates an error for empty import path.
    ///
    /// Import paths cannot be empty strings.
    pub fn empty_import_path(span: Span) -> Self {
        Self::new(
            ParseErrorKind::EmptyImportPath,
            "Import path cannot be empty",
            span,
        )
    }

    // =========================================================================
    // Integer literal errors
    // =========================================================================

    /// Creates an error for a positive integer literal that exceeds i64::MAX.
    pub fn integer_literal_out_of_range_positive(unsigned_value: u64, span: Span) -> Self {
        Self::new(
            ParseErrorKind::IntegerLiteralOutOfRange,
            format!(
                "Integer literal '{}' is out of range for i64 (valid range: {} to {})",
                unsigned_value,
                i64::MIN,
                i64::MAX
            ),
            span,
        )
    }

    /// Creates an error for a negative integer literal that exceeds i64::MIN.
    pub fn integer_literal_out_of_range_negative(unsigned_value: u64, span: Span) -> Self {
        let signed_literal = format!("-{}", unsigned_value);
        Self::new(
            ParseErrorKind::IntegerLiteralOutOfRange,
            format!(
                "Integer literal '{}' is out of range for i64 (valid range: {} to {})",
                signed_literal,
                i64::MIN,
                i64::MAX
            ),
            span,
        )
    }

    // =========================================================================
    // Internal errors
    // =========================================================================

    /// Creates an error for internal parser inconsistency.
    ///
    /// This indicates a compiler bug where internal invariants are violated.
    pub fn internal_binary_op_inconsistency(span: Span) -> Self {
        Self::new(
            ParseErrorKind::InternalError,
            "Internal error: binary_op_precedence returned Some, but token_to_binary_op returned None. This is a compiler bug.".to_string(),
            span,
        )
    }

    /// Creates an internal parser error with a custom message.
    ///
    /// Use this for "impossible" states that should never be reached.
    /// This is preferable to `unreachable!()` as it returns a proper error
    /// instead of panicking.
    pub fn internal(message: impl Into<String>, span: Span) -> Self {
        Self::new(ParseErrorKind::InternalError, message, span)
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}: {}",
            self.span.line, self.span.column, self.message
        )
    }
}

impl std::error::Error for ParseError {}
