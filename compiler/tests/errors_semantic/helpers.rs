use crate::common::{CompileErrorKind, CompileStage, compile_error_with_kind};
use lak::semantic::SemanticErrorKind;

pub(crate) fn assert_semantic_error(
    source: &str,
    expected_message: &str,
    expected_short_message: &str,
    expected_kind: SemanticErrorKind,
) {
    let result = compile_error_with_kind(source);
    let (stage, message, short_message, kind) = result.expect("Expected compilation to fail");

    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        message
    );
    assert_eq!(message, expected_message);
    assert_eq!(short_message, expected_short_message);
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(expected_kind),
        "Expected {:?} error kind",
        expected_kind
    );
}
