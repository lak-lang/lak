//! Unit tests for the lexer module.

use super::*;
use crate::token::TokenKind;

/// Helper function to tokenize input and return only the kinds.
pub(super) fn tokenize_kinds(input: &str) -> Vec<TokenKind> {
    let mut lexer = Lexer::new(input);
    lexer
        .tokenize()
        .unwrap_or_else(|e| panic!("Tokenization failed for input {:?}: {}", input, e))
        .into_iter()
        .map(|t| t.kind)
        .collect()
}

/// Helper function to tokenize input and return the error.
pub(super) fn tokenize_error(input: &str) -> LexError {
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

mod basic_tokens;
mod comments;
mod compound;
mod edge_cases;
mod errors;
mod identifiers;
mod integers;
mod keywords;
mod newlines;
mod spans;
mod strings;
mod whitespace;
