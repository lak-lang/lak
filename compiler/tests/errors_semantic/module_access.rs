use crate::helpers::assert_semantic_error;
use lak::semantic::SemanticErrorKind;

// ========================================
// Module access error tests (Phase 2)
// ========================================

#[test]
fn test_compile_error_member_call_not_implemented() {
    // Module-qualified function call (module.function()) without an import returns ModuleNotImported
    assert_semantic_error(
        r#"import "./math" as math

fn main() -> void {
    math.add()
}"#,
        "Module-qualified function call 'math.add()' requires an import statement. Add: import \"./math\"",
        "Module not imported",
        SemanticErrorKind::ModuleNotImported,
    );
}

#[test]
fn test_compile_error_member_access_not_implemented() {
    // Member access without call (module.value) triggers ModuleAccessNotImplemented
    assert_semantic_error(
        r#"import "./lib" as lib

fn main() -> void {
    lib.value
}"#,
        "Module-qualified access (e.g., module.function) is not yet implemented",
        "Module access not implemented",
        SemanticErrorKind::ModuleAccessNotImplemented,
    );
}

#[test]
fn test_compile_error_member_access_in_let() {
    // Member access in let statement also triggers ModuleAccessNotImplemented
    assert_semantic_error(
        r#"import "./config" as cfg

fn main() -> void {
    let x: i32 = cfg.value
}"#,
        "Module-qualified access (e.g., module.function) is not yet implemented",
        "Module access not implemented",
        SemanticErrorKind::ModuleAccessNotImplemented,
    );
}
