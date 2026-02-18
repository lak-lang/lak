//! Code generation error tests for user-facing diagnostics.

mod common;

use common::{CompileErrorKind, CompileStage, compile_error_with_kind};
use lak::codegen::CodegenErrorKind;

#[test]
fn test_codegen_internal_error_uses_source_function_name() {
    let result = compile_error_with_kind(
        r#"fn foo() -> i64 {
    while true {
        break
        return 1
    }
}

fn main() -> void {
}"#,
    );

    let (stage, msg, _short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Codegen),
        "Expected Codegen error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        kind,
        CompileErrorKind::Codegen(CodegenErrorKind::InternalError),
        "Expected InternalError error kind"
    );
    assert_eq!(
        msg,
        "Internal error: function 'foo' with return type 'i64' reached end without return. Semantic analysis should have rejected this. This is a compiler bug."
    );
    assert!(
        !msg.contains("_L"),
        "internal diagnostics must not expose mangled names: {}",
        msg
    );
}
