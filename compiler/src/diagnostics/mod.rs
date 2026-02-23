use std::ops::Range;

use ariadne::{Color, Config, IndexType, Label, Report, ReportKind, Source};
use lak::semantic::{SemanticError, SemanticErrorKind};

use crate::{CompileError, ModuleSemanticContext};

fn print_range_report(
    filename: &str,
    source: &str,
    span_range: Range<usize>,
    short_message: &str,
    label_message: &str,
    help: Option<&str>,
) -> std::io::Result<()> {
    let mut report = Report::build(ReportKind::Error, (filename, span_range.clone()))
        .with_config(Config::default().with_index_type(IndexType::Byte))
        .with_message(short_message)
        .with_label(
            Label::new((filename, span_range))
                .with_message(label_message)
                .with_color(Color::Red),
        );

    if let Some(help_message) = help {
        report = report.with_help(help_message);
    }

    report.finish().eprint((filename, Source::from(source)))
}

fn semantic_no_span_label_message(error: &SemanticError) -> &str {
    if error.kind() == SemanticErrorKind::MissingMainFunction {
        "main function not found"
    } else {
        error.message()
    }
}

fn semantic_no_span_help_message(error: &SemanticError) -> Option<&str> {
    if error.kind() == SemanticErrorKind::MissingMainFunction {
        Some("add a main function: fn main() -> void { ... }")
    } else {
        error.help()
    }
}

fn end_of_source_range(source: &str) -> Range<usize> {
    if source.is_empty() {
        0..0
    } else {
        let end = source.len().saturating_sub(1);
        end..source.len()
    }
}

fn report_semantic_error(filename: &str, source: &str, error: &SemanticError) {
    if let Some(span) = error.span() {
        if let Err(report_err) = print_range_report(
            filename,
            source,
            span.start..span.end,
            error.short_message(),
            error.message(),
            error.help(),
        ) {
            eprintln!(
                "Error: {} (at {}:{})",
                error.message(),
                span.line,
                span.column
            );
            if let Some(help_message) = error.help() {
                eprintln!("Help: {}", help_message);
            }
            eprintln!("(Failed to display detailed error report: {})", report_err);
        }
        return;
    }

    let help_message = semantic_no_span_help_message(error);
    if let Err(report_err) = print_range_report(
        filename,
        source,
        end_of_source_range(source),
        error.short_message(),
        semantic_no_span_label_message(error),
        help_message,
    ) {
        eprintln!("Error in {}: {}", filename, error.message());
        if let Some(help) = help_message {
            eprintln!("Help: {}", help);
        }
        eprintln!("(Failed to display detailed error report: {})", report_err);
    }
}

pub(crate) fn report_error(filename: &str, source: &str, error: &CompileError) {
    match error {
        CompileError::Resolve(error) => {
            let (report_filename, report_source) = if let (Some(src_file), Some(src_content)) =
                (error.source_filename(), error.source_content())
            {
                (src_file, src_content)
            } else {
                (filename, source)
            };

            if let Some(span) = error.span() {
                if let Err(report_err) = print_range_report(
                    report_filename,
                    report_source,
                    span.start..span.end,
                    error.short_message(),
                    error.message(),
                    error.help(),
                ) {
                    eprintln!(
                        "Error: {}: {} (at {}:{})",
                        error.short_message(),
                        error.message(),
                        span.line,
                        span.column
                    );
                    if let Some(help) = error.help() {
                        eprintln!("Help: {}", help);
                    }
                    eprintln!("(Failed to display detailed error report: {})", report_err);
                }
            } else {
                eprintln!("Error: {}", error.message());
                if let Some(help) = error.help() {
                    eprintln!("Help: {}", help);
                }
            }
        }
        CompileError::Semantic(error) => {
            report_semantic_error(filename, source, error);
        }
        CompileError::ModuleSemantic(ctx) => {
            let ModuleSemanticContext {
                error,
                filename: module_file,
                source: module_source,
            } = ctx.as_ref();
            report_semantic_error(module_file, module_source, error);
        }
        CompileError::Codegen(error) => {
            if let Some(span) = error.span() {
                if let Err(report_err) = print_range_report(
                    filename,
                    source,
                    span.start..span.end,
                    error.short_message(),
                    error.message(),
                    None,
                ) {
                    eprintln!(
                        "Error: {}: {} (at {}:{})",
                        error.short_message(),
                        error.message(),
                        span.line,
                        span.column
                    );
                    eprintln!("(Failed to display detailed error report: {})", report_err);
                }
            } else {
                eprintln!("Error in {}: {}", filename, error.message());
            }
        }
        CompileError::Link(error) => {
            eprintln!("Error: {}", error);
        }
        CompileError::PathNotUtf8 { .. }
        | CompileError::FileReadError { .. }
        | CompileError::PathResolutionError { .. }
        | CompileError::TempDirCreationError(_)
        | CompileError::ExecutableRunError(_)
        | CompileError::EntryModuleNotFound { .. }
        | CompileError::FilenameError { .. } => {
            eprintln!("Error: {}", error);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semantic_no_span_label_message_for_missing_main() {
        let error = SemanticError::missing_main("No main function");
        assert_eq!(
            semantic_no_span_label_message(&error),
            "main function not found"
        );
    }

    #[test]
    fn test_semantic_no_span_help_message_for_missing_main() {
        let error = SemanticError::missing_main("No main function");
        assert_eq!(
            semantic_no_span_help_message(&error),
            Some("add a main function: fn main() -> void { ... }")
        );
    }

    #[test]
    fn test_semantic_no_span_label_message_passthrough() {
        let error = SemanticError::without_span(
            SemanticErrorKind::TypeMismatch,
            "Type mismatch: expected 'i32', got 'string'",
        );
        assert_eq!(
            semantic_no_span_label_message(&error),
            "Type mismatch: expected 'i32', got 'string'"
        );
    }

    #[test]
    fn test_end_of_source_range_for_empty_source() {
        assert_eq!(end_of_source_range(""), 0..0);
    }

    #[test]
    fn test_end_of_source_range_for_non_empty_source() {
        assert_eq!(end_of_source_range("abc"), 2..3);
    }
}
