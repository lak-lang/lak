//! End-to-end tests for arithmetic operators.
//!
//! These tests verify arithmetic semantics, precedence/associativity,
//! runtime panic behavior, and overflow checks.

mod common;

use common::{compile_and_run, lak_binary};
use std::fs;
use std::process::{Command, Output};
use tempfile::tempdir;

#[derive(Debug)]
struct SuccessCase {
    id: &'static str,
    source: &'static str,
    expected_stdout: &'static str,
}

#[derive(Debug)]
struct LakRunFailureCase {
    id: &'static str,
    file_name: &'static str,
    source: &'static str,
    expected_stderr: &'static str,
    non_success_message: &'static str,
}

#[derive(Debug)]
struct CompileAndRunFailureCase {
    id: &'static str,
    source: &'static str,
    expected_error: &'static str,
}

fn extract_main_body<'a>(source: &'a str, case_id: &str) -> &'a str {
    let trimmed = source.trim();
    assert!(
        trimmed.starts_with("fn main() -> void"),
        "success case '{}' must define fn main() -> void",
        case_id
    );

    let open = trimmed.find('{').unwrap_or_else(|| {
        panic!(
            "success case '{}' must contain '{{' in main function",
            case_id
        )
    });
    let close = trimmed.rfind('}').unwrap_or_else(|| {
        panic!(
            "success case '{}' must contain '}}' in main function",
            case_id
        )
    });
    assert!(
        close > open,
        "success case '{}' has invalid main function body",
        case_id
    );

    trimmed[open + 1..close].trim()
}

fn run_batched_success_cases(batch_id: &str, cases: &[SuccessCase]) {
    let case_list = cases
        .iter()
        .map(|case| case.id)
        .collect::<Vec<_>>()
        .join(", ");

    let mut source = String::new();
    let mut expected = String::new();

    for (index, case) in cases.iter().enumerate() {
        source.push_str("fn __lak_case_");
        source.push_str(&index.to_string());
        source.push_str("() -> void {\n");
        for line in extract_main_body(case.source, case.id).lines() {
            source.push_str("    ");
            source.push_str(line.trim_end());
            source.push('\n');
        }
        source.push_str("}\n\n");
    }

    source.push_str("fn main() -> void {\n");

    for (index, case) in cases.iter().enumerate() {
        let marker_start = format!("__lak_case_start:{}__", case.id);
        let marker_end = format!("__lak_case_end:{}__", case.id);

        source.push_str("    println(\"");
        source.push_str(&marker_start);
        source.push_str("\")\n");
        source.push_str("    __lak_case_");
        source.push_str(&index.to_string());
        source.push_str("()\n");
        source.push_str("    println(\"");
        source.push_str(&marker_end);
        source.push_str("\")\n");

        expected.push_str(&marker_start);
        expected.push('\n');
        expected.push_str(case.expected_stdout);
        expected.push_str(&marker_end);
        expected.push('\n');
    }

    source.push_str("}\n");

    let output = compile_and_run(&source).unwrap_or_else(|err| {
        panic!(
            "batched success cases '{}' failed [{}]: {}",
            batch_id, case_list, err
        )
    });

    assert_eq!(
        output, expected,
        "batched success cases '{}' output mismatch [{}]",
        batch_id, case_list
    );
}

fn run_lak_source(file_name: &str, source: &str) -> Output {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join(file_name);
    fs::write(&source_path, source).unwrap();

    Command::new(lak_binary())
        .args(["run", source_path.to_str().unwrap()])
        .output()
        .unwrap()
}

fn assert_lak_run_failure_cases(cases: &[LakRunFailureCase]) {
    // Failure-mode sensitive cases stay isolated per scenario (R-08 control).
    for case in cases {
        let output = run_lak_source(case.file_name, case.source);

        assert!(
            !output.status.success(),
            "{} (case: {})",
            case.non_success_message,
            case.id
        );
        assert_eq!(
            output.status.code(),
            Some(1),
            "unexpected exit code for case '{}'",
            case.id
        );
        assert_eq!(
            String::from_utf8_lossy(&output.stderr),
            case.expected_stderr,
            "stderr mismatch for case '{}'",
            case.id
        );
    }
}

fn assert_compile_and_run_failure_cases(cases: &[CompileAndRunFailureCase]) {
    for case in cases {
        let result = compile_and_run(case.source);
        assert_eq!(
            result.unwrap_err(),
            case.expected_error,
            "unexpected compile_and_run failure text for case '{}'",
            case.id
        );
    }
}

const SUCCESS_CASES: &[SuccessCase] = &[
    SuccessCase {
        id: "test_addition",
        source: r#"fn main() -> void {
    let x: i32 = 3 + 5
    println(x)
}"#,
        expected_stdout: "8\n",
    },
    SuccessCase {
        id: "test_subtraction",
        source: r#"fn main() -> void {
    let x: i32 = 10 - 3
    println(x)
}"#,
        expected_stdout: "7\n",
    },
    SuccessCase {
        id: "test_multiplication",
        source: r#"fn main() -> void {
    let x: i32 = 4 * 5
    println(x)
}"#,
        expected_stdout: "20\n",
    },
    SuccessCase {
        id: "test_division",
        source: r#"fn main() -> void {
    let x: i32 = 20 / 4
    println(x)
}"#,
        expected_stdout: "5\n",
    },
    SuccessCase {
        id: "test_modulo",
        source: r#"fn main() -> void {
    let x: i32 = 17 % 5
    println(x)
}"#,
        expected_stdout: "2\n",
    },
    SuccessCase {
        id: "test_float_addition_f64",
        source: r#"fn main() -> void {
    let x: f64 = 3.5 + 2.25
    println(x)
}"#,
        expected_stdout: "5.75\n",
    },
    SuccessCase {
        id: "test_float_subtraction_f64",
        source: r#"fn main() -> void {
    let x: f64 = 5.5 - 2.25
    println(x)
}"#,
        expected_stdout: "3.25\n",
    },
    SuccessCase {
        id: "test_float_multiplication_f64",
        source: r#"fn main() -> void {
    let x: f64 = 1.5 * 2.5
    println(x)
}"#,
        expected_stdout: "3.75\n",
    },
    SuccessCase {
        id: "test_float_division_f64",
        source: r#"fn main() -> void {
    let x: f64 = 7.5 / 2.0
    println(x)
}"#,
        expected_stdout: "3.75\n",
    },
    SuccessCase {
        id: "test_float_arithmetic_mixed_f32_f64_promotes_to_f64",
        source: r#"fn main() -> void {
    let a: f32 = 1.5
    let b: f64 = 2.25
    let c: f64 = a + b
    println(c)
}"#,
        expected_stdout: "3.75\n",
    },
    SuccessCase {
        id: "test_float_unary_minus",
        source: r#"fn main() -> void {
    let x: f64 = -(3.5)
    println(x)
}"#,
        expected_stdout: "-3.5\n",
    },
    SuccessCase {
        id: "test_precedence_mul_before_add",
        source: r#"fn main() -> void {
    let x: i32 = 2 + 3 * 4
    println(x)
}"#,
        expected_stdout: "14\n",
    },
    SuccessCase {
        id: "test_precedence_div_before_sub",
        source: r#"fn main() -> void {
    let x: i32 = 10 - 6 / 2
    println(x)
}"#,
        expected_stdout: "7\n",
    },
    SuccessCase {
        id: "test_precedence_mod_before_add",
        source: r#"fn main() -> void {
    let x: i32 = 5 + 7 % 3
    println(x)
}"#,
        expected_stdout: "6\n",
    },
    SuccessCase {
        id: "test_precedence_complex",
        source: r#"fn main() -> void {
    let x: i32 = 1 + 2 * 3 + 4
    println(x)
}"#,
        expected_stdout: "11\n",
    },
    SuccessCase {
        id: "test_left_associative_sub",
        source: r#"fn main() -> void {
    let x: i32 = 10 - 5 - 2
    println(x)
}"#,
        expected_stdout: "3\n",
    },
    SuccessCase {
        id: "test_left_associative_div",
        source: r#"fn main() -> void {
    let x: i32 = 100 / 10 / 2
    println(x)
}"#,
        expected_stdout: "5\n",
    },
    SuccessCase {
        id: "test_left_associative_mod",
        source: r#"fn main() -> void {
    let x: i32 = 100 % 30 % 7
    println(x)
}"#,
        expected_stdout: "3\n",
    },
    SuccessCase {
        id: "test_parens_override_precedence",
        source: r#"fn main() -> void {
    let x: i32 = (2 + 3) * 4
    println(x)
}"#,
        expected_stdout: "20\n",
    },
    SuccessCase {
        id: "test_nested_parens",
        source: r#"fn main() -> void {
    let x: i32 = ((1 + 2) * 3) + 4
    println(x)
}"#,
        expected_stdout: "13\n",
    },
    SuccessCase {
        id: "test_parens_both_sides",
        source: r#"fn main() -> void {
    let x: i32 = (1 + 2) * (3 + 4)
    println(x)
}"#,
        expected_stdout: "21\n",
    },
    SuccessCase {
        id: "test_complex_expression",
        source: r#"fn main() -> void {
    let x: i32 = (1 + 2) * (3 + 4) - 5
    println(x)
}"#,
        expected_stdout: "16\n",
    },
    SuccessCase {
        id: "test_all_operators",
        source: r#"fn main() -> void {
    let x: i32 = 10 + 20 - 5 * 2 / 2 % 3
    println(x)
}"#,
        expected_stdout: "28\n",
    },
    SuccessCase {
        id: "test_arithmetic_with_variables",
        source: r#"fn main() -> void {
    let a: i32 = 10
    let b: i32 = 5
    let sum: i32 = a + b
    let diff: i32 = a - b
    let prod: i32 = a * b
    let quot: i32 = a / b
    let rem: i32 = a % b
    println(sum)
    println(diff)
    println(prod)
    println(quot)
    println(rem)
}"#,
        expected_stdout: "15\n5\n50\n2\n0\n",
    },
    SuccessCase {
        id: "test_variable_in_complex_expression",
        source: r#"fn main() -> void {
    let x: i32 = 5
    let y: i32 = 3
    let z: i32 = (x + y) * (x - y)
    println(z)
}"#,
        expected_stdout: "16\n",
    },
    SuccessCase {
        id: "test_i64_arithmetic",
        source: r#"fn main() -> void {
    let x: i64 = 1000000000 + 1000000000
    println(x)
}"#,
        expected_stdout: "2000000000\n",
    },
    SuccessCase {
        id: "test_i64_large_multiplication",
        source: r#"fn main() -> void {
    let x: i64 = 1000000 * 1000000
    println(x)
}"#,
        expected_stdout: "1000000000000\n",
    },
    SuccessCase {
        id: "test_negative_result",
        source: r#"fn main() -> void {
    let x: i32 = 5 - 10
    println(x)
}"#,
        expected_stdout: "-5\n",
    },
    SuccessCase {
        id: "test_negative_modulo",
        source: r#"fn main() -> void {
    let a: i32 = 3 - 10
    let b: i32 = a % 4
    println(b)
}"#,
        expected_stdout: "-3\n",
    },
    SuccessCase {
        id: "test_println_binary_op_directly",
        source: r#"fn main() -> void {
    println(3 + 5)
}"#,
        expected_stdout: "8\n",
    },
    SuccessCase {
        id: "test_println_binary_op_literal_plus_literal",
        source: r#"fn main() -> void {
    println(3 + 4)
}"#,
        expected_stdout: "7\n",
    },
    SuccessCase {
        id: "test_println_complex_expression",
        source: r#"fn main() -> void {
    println(2 * 3 + 4)
}"#,
        expected_stdout: "10\n",
    },
    SuccessCase {
        id: "test_println_binary_op_mixed_literal_left_i32",
        source: r#"fn main() -> void {
    let x: i32 = 6
    println(1 + x)
}"#,
        expected_stdout: "7\n",
    },
    SuccessCase {
        id: "test_println_binary_op_mixed_literal_right_i32",
        source: r#"fn main() -> void {
    let x: i32 = 6
    println(x + 1)
}"#,
        expected_stdout: "7\n",
    },
    SuccessCase {
        id: "test_println_binary_op_mixed_literal_left_i64",
        source: r#"fn main() -> void {
    let x: i64 = 6
    println(1 + x)
}"#,
        expected_stdout: "7\n",
    },
    SuccessCase {
        id: "test_println_binary_op_mixed_literal_right_i64",
        source: r#"fn main() -> void {
    let x: i64 = 6
    println(x + 1)
}"#,
        expected_stdout: "7\n",
    },
    SuccessCase {
        id: "test_println_binary_op_mixed_negative_literal_left_i32",
        source: r#"fn main() -> void {
    let x: i32 = 6
    println(-1 + x)
}"#,
        expected_stdout: "5\n",
    },
    SuccessCase {
        id: "test_println_binary_op_mixed_negative_literal_left_i64",
        source: r#"fn main() -> void {
    let x: i64 = 6
    println(-1 + x)
}"#,
        expected_stdout: "5\n",
    },
    SuccessCase {
        id: "test_println_nested_binary_op_literal_adaptation_i32",
        source: r#"fn main() -> void {
    let x: i32 = 3
    println((x + 1) * 2)
}"#,
        expected_stdout: "8\n",
    },
    SuccessCase {
        id: "test_println_nested_binary_op_literal_adaptation_i64",
        source: r#"fn main() -> void {
    let x: i64 = 3
    println((x + 1) * 2)
}"#,
        expected_stdout: "8\n",
    },
    SuccessCase {
        id: "test_println_binary_op_mixed_literal_subtraction_i32",
        source: r#"fn main() -> void {
    let x: i32 = 3
    println(10 - x)
}"#,
        expected_stdout: "7\n",
    },
    SuccessCase {
        id: "test_println_binary_op_mixed_literal_subtraction_i64",
        source: r#"fn main() -> void {
    let x: i64 = 3
    println(10 - x)
}"#,
        expected_stdout: "7\n",
    },
    SuccessCase {
        id: "test_println_binary_op_mixed_literal_multiplication_i32",
        source: r#"fn main() -> void {
    let x: i32 = 4
    println(3 * x)
}"#,
        expected_stdout: "12\n",
    },
    SuccessCase {
        id: "test_println_binary_op_mixed_literal_multiplication_i64",
        source: r#"fn main() -> void {
    let x: i64 = 4
    println(3 * x)
}"#,
        expected_stdout: "12\n",
    },
    SuccessCase {
        id: "test_println_binary_op_mixed_literal_division_i32",
        source: r#"fn main() -> void {
    let x: i32 = 4
    println(20 / x)
}"#,
        expected_stdout: "5\n",
    },
    SuccessCase {
        id: "test_println_binary_op_mixed_literal_division_i64",
        source: r#"fn main() -> void {
    let x: i64 = 4
    println(20 / x)
}"#,
        expected_stdout: "5\n",
    },
    SuccessCase {
        id: "test_println_binary_op_mixed_literal_modulo_i32",
        source: r#"fn main() -> void {
    let x: i32 = 4
    println(22 % x)
}"#,
        expected_stdout: "2\n",
    },
    SuccessCase {
        id: "test_println_binary_op_mixed_literal_modulo_i64",
        source: r#"fn main() -> void {
    let x: i64 = 4
    println(22 % x)
}"#,
        expected_stdout: "2\n",
    },
    SuccessCase {
        id: "test_zero_division_result",
        source: r#"fn main() -> void {
    let x: i32 = 0 / 5
    println(x)
}"#,
        expected_stdout: "0\n",
    },
    SuccessCase {
        id: "test_identity_operations",
        source: r#"fn main() -> void {
    let x: i32 = 42
    let a: i32 = x + 0
    let b: i32 = x * 1
    let c: i32 = x - 0
    println(a)
    println(b)
    println(c)
}"#,
        expected_stdout: "42\n42\n42\n",
    },
    SuccessCase {
        id: "test_division_by_nonzero_still_works",
        source: r#"fn main() -> void {
    let x: i32 = 0
    let y: i32 = 5
    let z: i32 = x / y
    println(z)
}"#,
        expected_stdout: "0\n",
    },
    SuccessCase {
        id: "test_division_by_negative",
        source: r#"fn main() -> void {
    let divisor: i32 = 0 - 2
    let x: i32 = 10 / divisor
    println(x)
}"#,
        expected_stdout: "-5\n",
    },
    SuccessCase {
        id: "test_unary_minus_literal",
        source: r#"fn main() -> void {
    let x: i32 = -5
    println(x)
}"#,
        expected_stdout: "-5\n",
    },
    SuccessCase {
        id: "test_unary_minus_variable",
        source: r#"fn main() -> void {
    let a: i32 = 10
    let b: i32 = -a
    println(b)
}"#,
        expected_stdout: "-10\n",
    },
    SuccessCase {
        id: "test_unary_minus_precedence",
        source: r#"fn main() -> void {
    let x: i32 = -2 + 3
    println(x)
}"#,
        expected_stdout: "1\n",
    },
    SuccessCase {
        id: "test_unary_minus_multiply",
        source: r#"fn main() -> void {
    let x: i32 = -2 * 3
    println(x)
}"#,
        expected_stdout: "-6\n",
    },
    SuccessCase {
        id: "test_double_unary_minus",
        source: r#"fn main() -> void {
    let x: i32 = --5
    println(x)
}"#,
        expected_stdout: "5\n",
    },
    SuccessCase {
        id: "test_unary_minus_with_parens",
        source: r#"fn main() -> void {
    let x: i32 = -(3 + 2)
    println(x)
}"#,
        expected_stdout: "-5\n",
    },
    SuccessCase {
        id: "test_unary_minus_in_println",
        source: r#"fn main() -> void {
    println(-5)
}"#,
        expected_stdout: "-5\n",
    },
    SuccessCase {
        id: "test_unary_minus_expression_in_println",
        source: r#"fn main() -> void {
    println(-(3 + 2))
}"#,
        expected_stdout: "-5\n",
    },
    SuccessCase {
        id: "test_unary_minus_i64",
        source: r#"fn main() -> void {
    let x: i64 = -1000000000
    println(x)
}"#,
        expected_stdout: "-1000000000\n",
    },
    SuccessCase {
        id: "test_unary_minus_complex",
        source: r#"fn main() -> void {
    let a: i32 = 3
    let b: i32 = 4
    let c: i32 = -a * -b
    println(c)
}"#,
        expected_stdout: "12\n",
    },
    SuccessCase {
        id: "test_unary_minus_with_division",
        source: r#"fn main() -> void {
    let x: i32 = -10 / 2
    println(x)
}"#,
        expected_stdout: "-5\n",
    },
    SuccessCase {
        id: "test_unary_minus_with_modulo",
        source: r#"fn main() -> void {
    let x: i32 = -10 % 3
    println(x)
}"#,
        expected_stdout: "-1\n",
    },
    SuccessCase {
        id: "test_subtraction_vs_unary_minus",
        source: r#"fn main() -> void {
    let a: i32 = 5
    let b: i32 = 3
    let x: i32 = a - -b
    println(x)
}"#,
        expected_stdout: "8\n",
    },
    SuccessCase {
        id: "test_unary_minus_triple_negation",
        source: r#"fn main() -> void {
    let x: i32 = ---5
    println(x)
}"#,
        expected_stdout: "-5\n",
    },
    SuccessCase {
        id: "test_unary_minus_in_function_arg",
        source: r#"fn main() -> void {
    println(-42)
}"#,
        expected_stdout: "-42\n",
    },
    SuccessCase {
        id: "test_unary_minus_variable_in_function_arg",
        source: r#"fn main() -> void {
    let x: i32 = 100
    println(-x)
}"#,
        expected_stdout: "-100\n",
    },
    SuccessCase {
        id: "test_no_overflow_i32_max_safe",
        source: r#"fn main() -> void {
    let x: i32 = 2147483646
    let y: i32 = x + 1
    println(y)
}"#,
        expected_stdout: "2147483647\n",
    },
    SuccessCase {
        id: "test_no_overflow_negation_safe",
        source: r#"fn main() -> void {
    let x: i32 = 2147483647
    let y: i32 = -x
    println(y)
}"#,
        expected_stdout: "-2147483647\n",
    },
    SuccessCase {
        id: "test_no_overflow_i64_max_safe",
        source: r#"fn main() -> void {
    let x: i64 = 9223372036854775806
    let y: i64 = x + 1
    println(y)
}"#,
        expected_stdout: "9223372036854775807\n",
    },
    SuccessCase {
        id: "test_i64_min_literal",
        source: r#"fn main() -> void {
    let x: i64 = -9223372036854775808
    println(x)
}"#,
        expected_stdout: "-9223372036854775808\n",
    },
    SuccessCase {
        id: "test_i64_min_direct_println",
        source: r#"fn main() -> void {
    println(-9223372036854775808)
}"#,
        expected_stdout: "-9223372036854775808\n",
    },
    SuccessCase {
        id: "test_i32_min_literal",
        source: r#"fn main() -> void {
    let x: i32 = -2147483648
    println(x)
}"#,
        expected_stdout: "-2147483648\n",
    },
    SuccessCase {
        id: "test_division_no_false_positive_min_div_two",
        source: r#"fn main() -> void {
    let x: i32 = -2147483647 - 1
    let y: i32 = x / 2
    println(y)
}"#,
        expected_stdout: "-1073741824\n",
    },
    SuccessCase {
        id: "test_division_no_false_positive_neg_one_safe",
        source: r#"fn main() -> void {
    let x: i32 = 10
    let y: i32 = 0 - 1
    let z: i32 = x / y
    println(z)
}"#,
        expected_stdout: "-10\n",
    },
    SuccessCase {
        id: "test_modulo_no_false_positive_min_mod_two",
        source: r#"fn main() -> void {
    let x: i32 = -2147483647 - 1
    let y: i32 = x % 2
    println(y)
}"#,
        expected_stdout: "0\n",
    },
    SuccessCase {
        id: "test_modulo_no_false_positive_neg_one_safe",
        source: r#"fn main() -> void {
    let x: i32 = 10
    let y: i32 = 0 - 1
    let z: i32 = x % y
    println(z)
}"#,
        expected_stdout: "0\n",
    },
    SuccessCase {
        id: "test_division_no_false_positive_i64_min_div_two",
        source: r#"fn main() -> void {
    let x: i64 = -9223372036854775807 - 1
    let y: i64 = x / 2
    println(y)
}"#,
        expected_stdout: "-4611686018427387904\n",
    },
    SuccessCase {
        id: "test_division_no_false_positive_i64_neg_one_safe",
        source: r#"fn main() -> void {
    let x: i64 = 10
    let y: i64 = 0 - 1
    let z: i64 = x / y
    println(z)
}"#,
        expected_stdout: "-10\n",
    },
    SuccessCase {
        id: "test_modulo_no_false_positive_i64_min_mod_two",
        source: r#"fn main() -> void {
    let x: i64 = -9223372036854775807 - 1
    let y: i64 = x % 2
    println(y)
}"#,
        expected_stdout: "0\n",
    },
    SuccessCase {
        id: "test_modulo_no_false_positive_i64_neg_one_safe",
        source: r#"fn main() -> void {
    let x: i64 = 10
    let y: i64 = 0 - 1
    let z: i64 = x % y
    println(z)
}"#,
        expected_stdout: "0\n",
    },
    SuccessCase {
        id: "test_unsigned_division_uses_unsigned_semantics",
        source: r#"fn main() -> void {
    let a: u8 = 200
    let b: u8 = 3
    let q: u8 = a / b
    println(q)
}"#,
        expected_stdout: "66\n",
    },
    SuccessCase {
        id: "test_unsigned_modulo_uses_unsigned_semantics",
        source: r#"fn main() -> void {
    let a: u8 = 200
    let b: u8 = 3
    let r: u8 = a % b
    println(r)
}"#,
        expected_stdout: "2\n",
    },
];

const LAK_RUN_FAILURE_CASES: &[LakRunFailureCase] = &[
    LakRunFailureCase {
        id: "test_division_by_zero_literal",
        file_name: "div_zero_lit.lak",
        source: r#"fn main() -> void {
    let x: i32 = 10 / 0
    println(x)
}"#,
        expected_stderr: "panic: division by zero\n",
        non_success_message: "division by zero should panic",
    },
    LakRunFailureCase {
        id: "test_division_by_zero_variable",
        file_name: "div_zero_var.lak",
        source: r#"fn main() -> void {
    let x: i32 = 10
    let y: i32 = 0
    let z: i32 = x / y
    println(z)
}"#,
        expected_stderr: "panic: division by zero\n",
        non_success_message: "case should fail",
    },
    LakRunFailureCase {
        id: "test_modulo_by_zero_literal",
        file_name: "mod_zero_lit.lak",
        source: r#"fn main() -> void {
    let x: i32 = 10 % 0
    println(x)
}"#,
        expected_stderr: "panic: modulo by zero\n",
        non_success_message: "modulo by zero should panic",
    },
    LakRunFailureCase {
        id: "test_modulo_by_zero_variable",
        file_name: "mod_zero_var.lak",
        source: r#"fn main() -> void {
    let a: i32 = 17
    let b: i32 = 5 - 5
    let c: i32 = a % b
    println(c)
}"#,
        expected_stderr: "panic: modulo by zero\n",
        non_success_message: "case should fail",
    },
    LakRunFailureCase {
        id: "test_division_by_zero_i64",
        file_name: "div_zero_i64.lak",
        source: r#"fn main() -> void {
    let x: i64 = 1000000000
    let y: i64 = 0
    let z: i64 = x / y
    println(z)
}"#,
        expected_stderr: "panic: division by zero\n",
        non_success_message: "case should fail",
    },
    LakRunFailureCase {
        id: "test_nested_division_zero_check",
        file_name: "nested_div.lak",
        source: r#"fn main() -> void {
    let x: i32 = 10 + 20 / 0
    println(x)
}"#,
        expected_stderr: "panic: division by zero\n",
        non_success_message: "case should fail",
    },
    LakRunFailureCase {
        id: "test_chained_division_zero_check",
        file_name: "chained_div.lak",
        source: r#"fn main() -> void {
    let x: i32 = 100 / 10 / 0
    println(x)
}"#,
        expected_stderr: "panic: division by zero\n",
        non_success_message: "case should fail",
    },
    LakRunFailureCase {
        id: "test_modulo_by_zero_i64",
        file_name: "mod_zero_i64.lak",
        source: r#"fn main() -> void {
    let x: i64 = 1000000000
    let y: i64 = 0
    let z: i64 = x % y
    println(z)
}"#,
        expected_stderr: "panic: modulo by zero\n",
        non_success_message: "case should fail",
    },
    LakRunFailureCase {
        id: "test_unary_minus_i32_min_overflow",
        file_name: "neg_i32_min.lak",
        source: r#"fn main() -> void {
    let x: i32 = -2147483647 - 1
    let y: i32 = -x
    println(y)
}"#,
        expected_stderr: "panic: integer overflow\n",
        non_success_message: "negation overflow should panic",
    },
    LakRunFailureCase {
        id: "test_unary_minus_i64_min_overflow",
        file_name: "neg_i64_min.lak",
        source: r#"fn main() -> void {
    let x: i64 = -9223372036854775808
    let y: i64 = -x
    println(y)
}"#,
        expected_stderr: "panic: integer overflow\n",
        non_success_message: "negation overflow should panic",
    },
    LakRunFailureCase {
        id: "test_addition_overflow_i32",
        file_name: "add_overflow_i32.lak",
        source: r#"fn main() -> void {
    let x: i32 = 2147483647
    let y: i32 = x + 1
    println(y)
}"#,
        expected_stderr: "panic: integer overflow\n",
        non_success_message: "addition overflow should panic",
    },
    LakRunFailureCase {
        id: "test_addition_overflow_i64",
        file_name: "add_overflow_i64.lak",
        source: r#"fn main() -> void {
    let x: i64 = 9223372036854775807
    let y: i64 = x + 1
    println(y)
}"#,
        expected_stderr: "panic: integer overflow\n",
        non_success_message: "addition overflow should panic",
    },
    LakRunFailureCase {
        id: "test_subtraction_overflow_i32",
        file_name: "sub_overflow_i32.lak",
        source: r#"fn main() -> void {
    let x: i32 = -2147483647 - 1
    let y: i32 = x - 1
    println(y)
}"#,
        expected_stderr: "panic: integer overflow\n",
        non_success_message: "subtraction overflow should panic",
    },
    LakRunFailureCase {
        id: "test_subtraction_overflow_i64",
        file_name: "sub_overflow_i64.lak",
        source: r#"fn main() -> void {
    let x: i64 = -9223372036854775807 - 1
    let y: i64 = x - 1
    println(y)
}"#,
        expected_stderr: "panic: integer overflow\n",
        non_success_message: "subtraction overflow should panic",
    },
    LakRunFailureCase {
        id: "test_multiplication_overflow_i32",
        file_name: "mul_overflow_i32.lak",
        source: r#"fn main() -> void {
    let x: i32 = 100000
    let y: i32 = x * x
    println(y)
}"#,
        expected_stderr: "panic: integer overflow\n",
        non_success_message: "multiplication overflow should panic",
    },
    LakRunFailureCase {
        id: "test_multiplication_overflow_i64",
        file_name: "mul_overflow_i64.lak",
        source: r#"fn main() -> void {
    let x: i64 = 10000000000000
    let y: i64 = x * x
    println(y)
}"#,
        expected_stderr: "panic: integer overflow\n",
        non_success_message: "multiplication overflow should panic",
    },
    LakRunFailureCase {
        id: "test_addition_negative_overflow_i32",
        file_name: "add_neg_overflow_i32.lak",
        source: r#"fn main() -> void {
    let x: i32 = -2147483647 - 1
    let y: i32 = -1
    let z: i32 = x + y
    println(z)
}"#,
        expected_stderr: "panic: integer overflow\n",
        non_success_message: "negative addition overflow should panic",
    },
    LakRunFailureCase {
        id: "test_addition_negative_overflow_i64",
        file_name: "add_neg_overflow_i64.lak",
        source: r#"fn main() -> void {
    let x: i64 = -9223372036854775807 - 1
    let y: i64 = -1
    let z: i64 = x + y
    println(z)
}"#,
        expected_stderr: "panic: integer overflow\n",
        non_success_message: "negative addition overflow should panic",
    },
    LakRunFailureCase {
        id: "test_chained_overflow_i32",
        file_name: "chained_overflow_i32.lak",
        source: r#"fn main() -> void {
    let x: i32 = 2147483647
    let y: i32 = x + 1 - 1
    println(y)
}"#,
        expected_stderr: "panic: integer overflow\n",
        non_success_message: "chained expression should panic at first overflow",
    },
    LakRunFailureCase {
        id: "test_division_overflow_i32_min_div_neg_one",
        file_name: "div_overflow_i32.lak",
        source: r#"fn main() -> void {
    let x: i32 = -2147483647 - 1
    let y: i32 = 0 - 1
    let z: i32 = x / y
    println(z)
}"#,
        expected_stderr: "panic: integer overflow\n",
        non_success_message: "i32 MIN / -1 should panic with overflow",
    },
    LakRunFailureCase {
        id: "test_division_overflow_i64_min_div_neg_one",
        file_name: "div_overflow_i64.lak",
        source: r#"fn main() -> void {
    let x: i64 = -9223372036854775807 - 1
    let y: i64 = 0 - 1
    let z: i64 = x / y
    println(z)
}"#,
        expected_stderr: "panic: integer overflow\n",
        non_success_message: "i64 MIN / -1 should panic with overflow",
    },
    LakRunFailureCase {
        id: "test_modulo_overflow_i32_min_mod_neg_one",
        file_name: "mod_overflow_i32.lak",
        source: r#"fn main() -> void {
    let x: i32 = -2147483647 - 1
    let y: i32 = 0 - 1
    let z: i32 = x % y
    println(z)
}"#,
        expected_stderr: "panic: integer overflow\n",
        non_success_message: "i32 MIN % -1 should panic with overflow",
    },
    LakRunFailureCase {
        id: "test_modulo_overflow_i64_min_mod_neg_one",
        file_name: "mod_overflow_i64.lak",
        source: r#"fn main() -> void {
    let x: i64 = -9223372036854775807 - 1
    let y: i64 = 0 - 1
    let z: i64 = x % y
    println(z)
}"#,
        expected_stderr: "panic: integer overflow\n",
        non_success_message: "i64 MIN % -1 should panic with overflow",
    },
    LakRunFailureCase {
        id: "test_signed_narrow_addition_overflow_i8_panics",
        file_name: "add_overflow_i8.lak",
        source: r#"fn main() -> void {
    let x: i8 = 127
    let y: i8 = x + 1
    println(y)
}"#,
        expected_stderr: "panic: integer overflow\n",
        non_success_message: "i8 addition overflow should panic",
    },
    LakRunFailureCase {
        id: "test_signed_narrow_subtraction_overflow_i16_panics",
        file_name: "sub_overflow_i16.lak",
        source: r#"fn main() -> void {
    let x: i16 = -32767 - 1
    let y: i16 = x - 1
    println(y)
}"#,
        expected_stderr: "panic: integer overflow\n",
        non_success_message: "i16 subtraction overflow should panic",
    },
    LakRunFailureCase {
        id: "test_unsigned_subtraction_underflow_panics",
        file_name: "sub_underflow_u8.lak",
        source: r#"fn main() -> void {
    let x: u8 = 0
    let y: u8 = x - 1
    println(y)
}"#,
        expected_stderr: "panic: integer overflow\n",
        non_success_message: "u8 subtraction underflow should panic",
    },
    LakRunFailureCase {
        id: "test_unsigned_multiplication_overflow_panics",
        file_name: "mul_overflow_u16.lak",
        source: r#"fn main() -> void {
    let x: u16 = 65535
    let y: u16 = x * 2
    println(y)
}"#,
        expected_stderr: "panic: integer overflow\n",
        non_success_message: "u16 multiplication overflow should panic",
    },
];

const COMPILE_AND_RUN_FAILURE_CASES: &[CompileAndRunFailureCase] = &[CompileAndRunFailureCase {
    id: "test_unsigned_addition_overflow_panics",
    source: r#"fn main() -> void {
    let a: u8 = 255
    let b: u8 = a + 1
    println(b)
}"#,
    expected_error: "Executable failed with exit code: Some(1)",
}];

#[test]
fn test_success_cases_batched() {
    run_batched_success_cases("e2e_arithmetic_success", SUCCESS_CASES);
}

#[test]
fn test_lak_run_failure_cases_isolated() {
    assert_lak_run_failure_cases(LAK_RUN_FAILURE_CASES);
}

#[test]
fn test_compile_and_run_failure_cases() {
    assert_compile_and_run_failure_cases(COMPILE_AND_RUN_FAILURE_CASES);
}
