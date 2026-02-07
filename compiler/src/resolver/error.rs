//! Module resolution error types.
//!
//! This module defines [`ResolverError`], which represents errors that can occur
//! during module resolution.

use crate::token::Span;

/// The kind of module resolution error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolverErrorKind {
    /// Import path could not be resolved to a file.
    FileNotFound,
    /// Import path is invalid (e.g., empty, malformed).
    InvalidImportPath,
    /// Circular import detected.
    CircularImport,
    /// File I/O error.
    IoError,
    /// Module name is invalid.
    InvalidModuleName,
    /// Lexical analysis failed.
    LexError,
    /// Parsing failed.
    ParseError,
    /// Standard library imports are not yet supported.
    StandardLibraryNotSupported,
}

/// An error that occurred during module resolution.
#[derive(Debug)]
pub struct ResolverError {
    /// A human-readable description of the error.
    message: String,
    /// The source location where the error occurred (if available).
    span: Option<Span>,
    /// The kind of error, for structured error handling.
    kind: ResolverErrorKind,
    /// The filename of the source file where the error occurred (if different from the entry module).
    source_filename: Option<String>,
    /// The source content of the file where the error occurred (if available).
    source_content: Option<String>,
}

impl ResolverError {
    /// Creates a new resolver error without span.
    pub fn new(kind: ResolverErrorKind, message: impl Into<String>) -> Self {
        ResolverError {
            message: message.into(),
            span: None,
            kind,
            source_filename: None,
            source_content: None,
        }
    }

    /// Creates a new resolver error with span.
    pub fn with_span(kind: ResolverErrorKind, message: impl Into<String>, span: Span) -> Self {
        ResolverError {
            message: message.into(),
            span: Some(span),
            kind,
            source_filename: None,
            source_content: None,
        }
    }

    /// Creates a new error with a span and source context from the file where the error occurred.
    ///
    /// Used for lex/parse errors in imported modules where the error's source
    /// location is in a different file than the entry module.
    pub fn with_source_context(
        kind: ResolverErrorKind,
        message: impl Into<String>,
        span: Span,
        source_filename: impl Into<String>,
        source_content: impl Into<String>,
    ) -> Self {
        ResolverError {
            message: message.into(),
            span: Some(span),
            kind,
            source_filename: Some(source_filename.into()),
            source_content: Some(source_content.into()),
        }
    }

    /// Returns the error message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the source location where the error occurred.
    pub fn span(&self) -> Option<Span> {
        self.span
    }

    /// Returns the kind of error.
    pub fn kind(&self) -> ResolverErrorKind {
        self.kind
    }

    /// Returns the source filename where the error occurred, if different from the entry module.
    pub fn source_filename(&self) -> Option<&str> {
        self.source_filename.as_deref()
    }

    /// Returns the source content of the file where the error occurred, if available.
    pub fn source_content(&self) -> Option<&str> {
        self.source_content.as_deref()
    }

    /// Returns a short, human-readable description of the error kind.
    pub fn short_message(&self) -> &'static str {
        match self.kind {
            ResolverErrorKind::FileNotFound => "Module not found",
            ResolverErrorKind::InvalidImportPath => "Invalid import path",
            ResolverErrorKind::CircularImport => "Circular import",
            ResolverErrorKind::IoError => "I/O error",
            ResolverErrorKind::InvalidModuleName => "Invalid module name",
            ResolverErrorKind::LexError => "Lexical error in module",
            ResolverErrorKind::ParseError => "Parse error in module",
            ResolverErrorKind::StandardLibraryNotSupported => "Standard library not supported",
        }
    }

    // =========================================================================
    // File resolution errors
    // =========================================================================

    /// Creates a "file not found" error.
    pub fn file_not_found(path: &str, span: Span) -> Self {
        Self::with_span(
            ResolverErrorKind::FileNotFound,
            format!("Cannot find module '{}'. Check that the file exists.", path),
            span,
        )
    }

    /// Creates a "circular import" error.
    pub fn circular_import(cycle: &str, span: Span) -> Self {
        Self::with_span(
            ResolverErrorKind::CircularImport,
            format!("Circular import detected: {}", cycle),
            span,
        )
    }

    /// Creates an "invalid import path" error.
    pub fn invalid_import_path(reason: &str, span: Span) -> Self {
        Self::with_span(
            ResolverErrorKind::InvalidImportPath,
            format!("Invalid import path: {}", reason),
            span,
        )
    }

    /// Creates an "I/O error" error.
    pub fn io_error(message: &str) -> Self {
        Self::new(
            ResolverErrorKind::IoError,
            format!("I/O error: {}", message),
        )
    }

    /// Creates an "I/O error" error with span.
    pub fn io_error_with_span(message: &str, span: Span) -> Self {
        Self::with_span(
            ResolverErrorKind::IoError,
            format!("I/O error: {}", message),
            span,
        )
    }

    /// Creates an "invalid module name" error.
    pub fn invalid_module_name(path: &str, span: Span) -> Self {
        Self::with_span(
            ResolverErrorKind::InvalidModuleName,
            format!(
                "Cannot extract module name from path '{}'. Module names must be valid identifiers.",
                path
            ),
            span,
        )
    }

    // =========================================================================
    // Unsupported features
    // =========================================================================

    /// Creates a "standard library not supported" error.
    pub fn standard_library_not_supported(path: &str, span: Span) -> Self {
        Self::with_span(
            ResolverErrorKind::StandardLibraryNotSupported,
            format!(
                "Standard library imports are not yet supported: '{}'. Use relative paths like './module' instead.",
                path
            ),
            span,
        )
    }
}

impl std::fmt::Display for ResolverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(span) = self.span {
            write!(f, "{}:{}: {}", span.line, span.column, self.message)
        } else {
            write!(f, "{}", self.message)
        }
    }
}

impl std::error::Error for ResolverError {}
