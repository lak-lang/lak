//! Source location tracking for the Lak compiler.
//!
//! This module provides the [`Span`] struct which tracks both byte offsets
//! and human-readable positions for error reporting.

/// A span representing a range in the source code.
///
/// `Span` tracks both byte offsets (for slicing the source string) and
/// human-readable positions (line and column numbers) for error reporting.
///
/// # Fields
///
/// * `start` - The starting byte offset (inclusive). Must be a valid UTF-8 boundary.
/// * `end` - The ending byte offset (exclusive). Must be a valid UTF-8 boundary.
/// * `line` - The 1-indexed line number where this span begins
/// * `column` - The 1-indexed column number where this span begins
///
/// # Safety
///
/// `start` and `end` must be valid UTF-8 character boundaries in the source.
/// The lexer ensures this invariant by using `char::len_utf8()` for position
/// tracking. Violating this can cause panics when slicing the source string.
///
/// # Examples
///
/// ```
/// use lak::token::Span;
///
/// let span = Span::new(0, 5, 1, 1);
/// assert_eq!(span.start, 0);
/// assert_eq!(span.end, 5);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    /// The starting byte offset (inclusive) in the source string.
    /// Must be a valid UTF-8 character boundary.
    pub start: usize,
    /// The ending byte offset (exclusive) in the source string.
    /// Must be a valid UTF-8 character boundary.
    pub end: usize,
    /// The 1-indexed line number where this span begins.
    pub line: usize,
    /// The 1-indexed column number where this span begins.
    pub column: usize,
}

impl Span {
    /// Creates a new `Span` with the given byte offsets and position.
    ///
    /// # Arguments
    ///
    /// * `start` - The starting byte offset (inclusive)
    /// * `end` - The ending byte offset (exclusive)
    /// * `line` - The 1-indexed line number
    /// * `column` - The 1-indexed column number
    ///
    /// # Returns
    ///
    /// A new `Span` instance with the specified values.
    pub fn new(start: usize, end: usize, line: usize, column: usize) -> Self {
        Span {
            start,
            end,
            line,
            column,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_new() {
        let span = Span::new(0, 5, 1, 1);
        assert_eq!(span.start, 0);
        assert_eq!(span.end, 5);
        assert_eq!(span.line, 1);
        assert_eq!(span.column, 1);
    }

    #[test]
    fn test_span_equality() {
        let span1 = Span::new(10, 20, 2, 5);
        let span2 = Span::new(10, 20, 2, 5);
        assert_eq!(span1, span2);
    }

    #[test]
    fn test_span_inequality() {
        let span1 = Span::new(0, 5, 1, 1);
        let span2 = Span::new(0, 6, 1, 1);
        assert_ne!(span1, span2);
    }

    #[test]
    fn test_span_copy() {
        let span1 = Span::new(0, 5, 1, 1);
        let span2 = span1; // Copy
        assert_eq!(span1, span2);
        // span1 is still usable after copy
        assert_eq!(span1.start, 0);
    }
}
