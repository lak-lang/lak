//! Module resolution error types.
//!
//! This module defines [`ResolverError`], which represents errors that can occur
//! during module resolution.

use std::path::Path;

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

/// Source context for errors that occur in a different file than the entry module.
///
/// Used for lex/parse errors in imported modules where the error's source
/// location is in a different file.
#[derive(Debug)]
pub struct SourceContext {
    /// The filename of the source file.
    filename: String,
    /// The source content of the file.
    content: String,
}

impl SourceContext {
    /// Creates a new SourceContext.
    fn new(filename: impl Into<String>, content: impl Into<String>) -> Self {
        SourceContext {
            filename: filename.into(),
            content: content.into(),
        }
    }
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
    /// Source context for errors in imported modules (filename and content of the file
    /// where the error occurred).
    source_context: Option<Box<SourceContext>>,
    /// Optional help text with suggestions for fixing the error.
    help: Option<String>,
}

impl ResolverError {
    /// Creates a new resolver error without span.
    pub fn new(kind: ResolverErrorKind, message: impl Into<String>) -> Self {
        ResolverError {
            message: message.into(),
            span: None,
            kind,
            source_context: None,
            help: None,
        }
    }

    /// Creates a new resolver error without span but with help text.
    ///
    /// Use this for errors without a source location that benefit from
    /// additional guidance on how to fix them.
    pub fn new_with_help(
        kind: ResolverErrorKind,
        message: impl Into<String>,
        help: impl Into<String>,
    ) -> Self {
        ResolverError {
            message: message.into(),
            span: None,
            kind,
            source_context: None,
            help: Some(help.into()),
        }
    }

    /// Creates a new resolver error with span.
    pub fn with_span(kind: ResolverErrorKind, message: impl Into<String>, span: Span) -> Self {
        ResolverError {
            message: message.into(),
            span: Some(span),
            kind,
            source_context: None,
            help: None,
        }
    }

    /// Creates a new resolver error with span and help text.
    ///
    /// Use this for errors that benefit from additional guidance on how to fix them.
    pub fn with_span_and_help(
        kind: ResolverErrorKind,
        message: impl Into<String>,
        span: Span,
        help: impl Into<String>,
    ) -> Self {
        ResolverError {
            message: message.into(),
            span: Some(span),
            kind,
            source_context: None,
            help: Some(help.into()),
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
            source_context: Some(Box::new(SourceContext::new(
                source_filename,
                source_content,
            ))),
            help: None,
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
        self.source_context
            .as_ref()
            .map(|ctx| ctx.filename.as_str())
    }

    /// Returns the source content of the file where the error occurred, if available.
    pub fn source_content(&self) -> Option<&str> {
        self.source_context.as_ref().map(|ctx| ctx.content.as_str())
    }

    /// Returns the help text, if available.
    pub fn help(&self) -> Option<&str> {
        self.help.as_deref()
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
        Self::with_span_and_help(
            ResolverErrorKind::FileNotFound,
            format!("Cannot find module '{}'", path),
            span,
            "check that the file exists and the path is correct",
        )
    }

    /// Creates a "circular import" error.
    pub fn circular_import(cycle: &str, span: Span) -> Self {
        Self::with_span_and_help(
            ResolverErrorKind::CircularImport,
            format!("Circular import detected: {}", cycle),
            span,
            "break the cycle by removing one of the imports or restructuring your modules",
        )
    }

    /// Creates an "invalid import path" error.
    pub fn invalid_import_path(reason: &str, span: Span) -> Self {
        Self::with_span_and_help(
            ResolverErrorKind::InvalidImportPath,
            format!("Invalid import path: {}", reason),
            span,
            "ensure the importing file is located in a valid directory",
        )
    }

    /// Creates an "import path with extension" error.
    pub fn import_path_with_extension(path: &str, span: Span) -> Self {
        let path_without_ext = Path::new(path).with_extension("");
        Self::with_span_and_help(
            ResolverErrorKind::InvalidImportPath,
            format!("Import path must not include file extension: '{}'", path),
            span,
            format!("use '{}' instead", path_without_ext.display()),
        )
    }

    /// Creates an I/O error for failing to resolve an import path.
    pub fn io_error_resolve_import(import_path: &str, error: &std::io::Error, span: Span) -> Self {
        Self::with_span(
            ResolverErrorKind::IoError,
            format!(
                "I/O error: Failed to resolve import path '{}': {}",
                import_path, error
            ),
            span,
        )
    }

    /// Creates an I/O error for failing to canonicalize a file path.
    pub fn io_error_canonicalize(path: &Path, error: &std::io::Error) -> Self {
        Self::new(
            ResolverErrorKind::IoError,
            format!(
                "I/O error: Failed to resolve path '{}': {}",
                path.display(),
                error
            ),
        )
    }

    /// Creates an "invalid module name" error.
    pub fn invalid_module_name(path: &str, span: Span) -> Self {
        Self::with_span_and_help(
            ResolverErrorKind::InvalidModuleName,
            format!("Cannot extract module name from path '{}'", path),
            span,
            "module names must be valid identifiers (start with an ASCII letter or underscore, contain only ASCII letters, digits, and underscores)",
        )
    }

    // =========================================================================
    // Unsupported features
    // =========================================================================

    /// Creates a "standard library not supported" error.
    pub fn standard_library_not_supported(path: &str, span: Span) -> Self {
        Self::with_span_and_help(
            ResolverErrorKind::StandardLibraryNotSupported,
            format!("Standard library imports are not yet supported: '{}'", path),
            span,
            "use relative paths like './module' instead",
        )
    }

    // =========================================================================
    // File read errors
    // =========================================================================

    /// Creates an I/O error for failing to read a module file.
    pub fn io_error_read_file(path: &Path, error: &std::io::Error, span: Option<Span>) -> Self {
        let message = format!("I/O error: Failed to read '{}': {}", path.display(), error);
        match span {
            Some(s) => Self::with_span(ResolverErrorKind::IoError, message, s),
            None => Self::new(ResolverErrorKind::IoError, message),
        }
    }

    /// Creates a circular import error without span.
    pub fn circular_import_no_span(cycle: &str) -> Self {
        Self::new_with_help(
            ResolverErrorKind::CircularImport,
            format!("Circular import detected: {}", cycle),
            "break the cycle by removing one of the imports or restructuring your modules",
        )
    }

    // =========================================================================
    // Module lex/parse errors
    // =========================================================================

    /// Shared helper for lex/parse errors in modules.
    fn module_phase_error(
        kind: ResolverErrorKind,
        phase: &str,
        path: &Path,
        error_message: &str,
        span: Span,
        source_content: Option<String>,
    ) -> Self {
        let message = format!(
            "{} error in module '{}': {}",
            phase,
            path.display(),
            error_message
        );
        if let Some(content) = source_content {
            Self::with_source_context(kind, message, span, path.display().to_string(), content)
        } else {
            Self::with_span(kind, message, span)
        }
    }

    /// Creates a lex error for a module.
    ///
    /// When `source_content` is `Some`, attaches source context for error reporting
    /// (used for imported modules). When `None`, creates a plain error (used for entry modules).
    pub fn lex_error_in_module(
        path: &Path,
        error_message: &str,
        span: Span,
        source_content: Option<String>,
    ) -> Self {
        Self::module_phase_error(
            ResolverErrorKind::LexError,
            "Lexical",
            path,
            error_message,
            span,
            source_content,
        )
    }

    /// Creates a parse error for a module.
    ///
    /// When `source_content` is `Some`, attaches source context for error reporting
    /// (used for imported modules). When `None`, creates a plain error (used for entry modules).
    pub fn parse_error_in_module(
        path: &Path,
        error_message: &str,
        span: Span,
        source_content: Option<String>,
    ) -> Self {
        Self::module_phase_error(
            ResolverErrorKind::ParseError,
            "Parse",
            path,
            error_message,
            span,
            source_content,
        )
    }

    // =========================================================================
    // Module name errors (without span)
    // =========================================================================

    /// Creates an "invalid module name" error without span.
    ///
    /// Used for entry modules where the import span is not available. The identifier
    /// rules are embedded directly in the message rather than in a separate help
    /// field, keeping the error self-contained for the plain `eprintln!` output path.
    pub fn invalid_module_name_no_span(path: &Path) -> Self {
        Self::new(
            ResolverErrorKind::InvalidModuleName,
            format!(
                "Cannot extract module name from path '{}'. Module names must be valid identifiers (start with an ASCII letter or underscore, contain only ASCII letters, digits, and underscores).",
                path.display()
            ),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::Span;

    fn dummy_span() -> Span {
        Span::new(0, 0, 1, 1)
    }

    // =========================================================================
    // File read error helpers
    // =========================================================================

    #[test]
    fn test_io_error_read_file_without_span_constructor() {
        let path = Path::new("/tmp/missing.lak");
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = ResolverError::io_error_read_file(path, &io_err, None);
        assert_eq!(err.kind(), ResolverErrorKind::IoError);
        assert!(err.span().is_none());
        assert_eq!(
            err.message(),
            "I/O error: Failed to read '/tmp/missing.lak': file not found"
        );
        assert!(err.source_filename().is_none());
        assert!(err.source_content().is_none());
    }

    #[test]
    fn test_io_error_read_file_with_span_constructor() {
        let path = Path::new("/tmp/missing.lak");
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let err = ResolverError::io_error_read_file(path, &io_err, Some(dummy_span()));
        assert_eq!(err.kind(), ResolverErrorKind::IoError);
        assert!(err.span().is_some());
        assert_eq!(
            err.message(),
            "I/O error: Failed to read '/tmp/missing.lak': access denied"
        );
        assert!(err.source_filename().is_none());
    }

    // =========================================================================
    // Circular import helpers
    // =========================================================================

    #[test]
    fn test_circular_import_no_span_constructor() {
        let err = ResolverError::circular_import_no_span("a -> b -> a");
        assert_eq!(err.kind(), ResolverErrorKind::CircularImport);
        assert!(err.span().is_none());
        assert_eq!(err.message(), "Circular import detected: a -> b -> a");
    }

    // =========================================================================
    // Module lex/parse error helpers
    // =========================================================================

    #[test]
    fn test_lex_error_in_module_constructor() {
        let path = Path::new("/tmp/entry.lak");
        let err =
            ResolverError::lex_error_in_module(path, "unexpected character", dummy_span(), None);
        assert_eq!(err.kind(), ResolverErrorKind::LexError);
        assert!(err.span().is_some());
        assert_eq!(
            err.message(),
            "Lexical error in module '/tmp/entry.lak': unexpected character"
        );
        assert!(err.source_filename().is_none());
        assert!(err.source_content().is_none());
    }

    #[test]
    fn test_lex_error_in_imported_module_constructor() {
        let path = Path::new("/tmp/utils.lak");
        let source = "fn bad() { @ }".to_string();
        let err = ResolverError::lex_error_in_module(
            path,
            "unexpected '@'",
            dummy_span(),
            Some(source.clone()),
        );
        assert_eq!(err.kind(), ResolverErrorKind::LexError);
        assert!(err.span().is_some());
        assert_eq!(
            err.message(),
            "Lexical error in module '/tmp/utils.lak': unexpected '@'"
        );
        // source_context should be set
        assert_eq!(err.source_filename(), Some("/tmp/utils.lak"));
        assert_eq!(err.source_content(), Some("fn bad() { @ }"));
    }

    #[test]
    fn test_parse_error_in_module_constructor() {
        let path = Path::new("/tmp/entry.lak");
        let err = ResolverError::parse_error_in_module(path, "expected '}'", dummy_span(), None);
        assert_eq!(err.kind(), ResolverErrorKind::ParseError);
        assert!(err.span().is_some());
        assert_eq!(
            err.message(),
            "Parse error in module '/tmp/entry.lak': expected '}'"
        );
        assert!(err.source_filename().is_none());
        assert!(err.source_content().is_none());
    }

    #[test]
    fn test_parse_error_in_imported_module_constructor() {
        let path = Path::new("/tmp/helper.lak");
        let source = "fn broken( {}".to_string();
        let err = ResolverError::parse_error_in_module(
            path,
            "expected ')'",
            dummy_span(),
            Some(source.clone()),
        );
        assert_eq!(err.kind(), ResolverErrorKind::ParseError);
        assert!(err.span().is_some());
        assert_eq!(
            err.message(),
            "Parse error in module '/tmp/helper.lak': expected ')'"
        );
        // source_context should be set
        assert_eq!(err.source_filename(), Some("/tmp/helper.lak"));
        assert_eq!(err.source_content(), Some("fn broken( {}"));
    }

    // =========================================================================
    // Module name error helpers
    // =========================================================================

    #[test]
    fn test_invalid_module_name_no_span_constructor() {
        let path = Path::new("///");
        let err = ResolverError::invalid_module_name_no_span(path);
        assert_eq!(err.kind(), ResolverErrorKind::InvalidModuleName);
        assert!(err.span().is_none());
        assert_eq!(
            err.message(),
            "Cannot extract module name from path '///'. Module names must be valid identifiers (start with an ASCII letter or underscore, contain only ASCII letters, digits, and underscores)."
        );
    }

    // =========================================================================
    // Display trait
    // =========================================================================

    #[test]
    fn test_display_with_span() {
        let err = ResolverError::file_not_found("./missing", dummy_span());
        let display = format!("{}", err);
        assert_eq!(display, "1:1: Cannot find module './missing'");
    }

    #[test]
    fn test_display_without_span() {
        let err = ResolverError::circular_import_no_span("a -> b -> a");
        let display = format!("{}", err);
        assert_eq!(display, "Circular import detected: a -> b -> a");
    }

    #[test]
    fn test_io_error_canonicalize_constructor() {
        let path = Path::new("/tmp/nonexistent.lak");
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = ResolverError::io_error_canonicalize(path, &io_err);
        assert_eq!(err.kind(), ResolverErrorKind::IoError);
        assert!(err.span().is_none());
        assert_eq!(
            err.message(),
            "I/O error: Failed to resolve path '/tmp/nonexistent.lak': file not found"
        );
    }

    // =========================================================================
    // Help text tests
    // =========================================================================

    #[test]
    fn test_file_not_found_help() {
        let err = ResolverError::file_not_found("./missing", dummy_span());
        assert_eq!(
            err.help(),
            Some("check that the file exists and the path is correct")
        );
    }

    #[test]
    fn test_circular_import_help() {
        let err = ResolverError::circular_import("a -> b -> a", dummy_span());
        assert_eq!(
            err.help(),
            Some("break the cycle by removing one of the imports or restructuring your modules")
        );
    }

    #[test]
    fn test_invalid_import_path_help() {
        let err =
            ResolverError::invalid_import_path("Cannot determine parent directory", dummy_span());
        assert_eq!(
            err.help(),
            Some("ensure the importing file is located in a valid directory")
        );
    }

    #[test]
    fn test_import_path_with_extension_constructor() {
        let err = ResolverError::import_path_with_extension("./utils.lak", dummy_span());
        assert_eq!(err.kind(), ResolverErrorKind::InvalidImportPath);
        assert_eq!(
            err.message(),
            "Import path must not include file extension: './utils.lak'"
        );
        assert_eq!(err.help(), Some("use './utils' instead"));
    }

    #[test]
    fn test_import_path_with_extension_help() {
        let err = ResolverError::import_path_with_extension("./lib/parser.lak", dummy_span());
        assert_eq!(err.help(), Some("use './lib/parser' instead"));
    }

    #[test]
    fn test_invalid_module_name_help() {
        let err = ResolverError::invalid_module_name("./123invalid", dummy_span());
        assert_eq!(
            err.help(),
            Some(
                "module names must be valid identifiers (start with an ASCII letter or underscore, contain only ASCII letters, digits, and underscores)"
            )
        );
    }

    #[test]
    fn test_standard_library_not_supported_help() {
        let err = ResolverError::standard_library_not_supported("math", dummy_span());
        assert_eq!(
            err.help(),
            Some("use relative paths like './module' instead")
        );
    }

    #[test]
    fn test_io_error_resolve_import_no_help() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let err = ResolverError::io_error_resolve_import("./utils", &io_err, dummy_span());
        assert!(err.help().is_none());
    }

    #[test]
    fn test_circular_import_no_span_help() {
        let err = ResolverError::circular_import_no_span("a -> b -> a");
        assert_eq!(
            err.help(),
            Some("break the cycle by removing one of the imports or restructuring your modules")
        );
    }

    #[test]
    fn test_io_error_read_file_no_help() {
        let path = Path::new("/tmp/missing.lak");
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = ResolverError::io_error_read_file(path, &io_err, Some(dummy_span()));
        assert!(err.help().is_none());
    }

    #[test]
    fn test_io_error_canonicalize_no_help() {
        let path = Path::new("/tmp/nonexistent.lak");
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = ResolverError::io_error_canonicalize(path, &io_err);
        assert!(err.help().is_none());
    }

    #[test]
    fn test_lex_error_in_module_no_help() {
        let path = Path::new("/tmp/entry.lak");
        let err =
            ResolverError::lex_error_in_module(path, "unexpected character", dummy_span(), None);
        assert!(err.help().is_none());
    }

    #[test]
    fn test_parse_error_in_module_no_help() {
        let path = Path::new("/tmp/entry.lak");
        let err = ResolverError::parse_error_in_module(path, "expected '}'", dummy_span(), None);
        assert!(err.help().is_none());
    }

    #[test]
    fn test_invalid_module_name_no_span_no_help() {
        let path = Path::new("///");
        let err = ResolverError::invalid_module_name_no_span(path);
        assert!(err.help().is_none());
    }
}
