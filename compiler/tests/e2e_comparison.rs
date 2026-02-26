//! End-to-end tests for comparison operators.
//!
//! These tests verify that comparison operations are correctly compiled
//! and executed, including operator precedence, type checking, and println integration.

mod common;

use common::compile_and_run;

#[derive(Debug)]
struct SuccessCase {
    id: &'static str,
    source: &'static str,
    expected_stdout: &'static str,
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

const SUCCESS_CASES: &[SuccessCase] = &[
    SuccessCase {
        id: "test_equal_true",
        source: r#"fn main() -> void {
    let result: bool = 5 == 5
    println(result)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_equal_false",
        source: r#"fn main() -> void {
    let result: bool = 5 == 3
    println(result)
}"#,
        expected_stdout: "false\n",
    },
    SuccessCase {
        id: "test_not_equal_true",
        source: r#"fn main() -> void {
    let result: bool = 5 != 3
    println(result)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_not_equal_false",
        source: r#"fn main() -> void {
    let result: bool = 5 != 5
    println(result)
}"#,
        expected_stdout: "false\n",
    },
    SuccessCase {
        id: "test_less_than_true",
        source: r#"fn main() -> void {
    let result: bool = 3 < 5
    println(result)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_less_than_false",
        source: r#"fn main() -> void {
    let result: bool = 5 < 3
    println(result)
}"#,
        expected_stdout: "false\n",
    },
    SuccessCase {
        id: "test_less_than_equal_values",
        source: r#"fn main() -> void {
    let result: bool = 5 < 5
    println(result)
}"#,
        expected_stdout: "false\n",
    },
    SuccessCase {
        id: "test_greater_than_true",
        source: r#"fn main() -> void {
    let result: bool = 5 > 3
    println(result)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_greater_than_false",
        source: r#"fn main() -> void {
    let result: bool = 3 > 5
    println(result)
}"#,
        expected_stdout: "false\n",
    },
    SuccessCase {
        id: "test_greater_than_equal_values",
        source: r#"fn main() -> void {
    let result: bool = 5 > 5
    println(result)
}"#,
        expected_stdout: "false\n",
    },
    SuccessCase {
        id: "test_unsigned_greater_than_high_bit_values",
        source: r#"fn main() -> void {
    let a: u8 = 200
    let b: u8 = 100
    let result: bool = a > b
    println(result)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_unsigned_less_than_high_bit_values",
        source: r#"fn main() -> void {
    let a: u16 = 50000
    let b: u16 = 30000
    let result: bool = a < b
    println(result)
}"#,
        expected_stdout: "false\n",
    },
    SuccessCase {
        id: "test_less_equal_true_less",
        source: r#"fn main() -> void {
    let result: bool = 3 <= 5
    println(result)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_less_equal_true_equal",
        source: r#"fn main() -> void {
    let result: bool = 5 <= 5
    println(result)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_less_equal_false",
        source: r#"fn main() -> void {
    let result: bool = 5 <= 3
    println(result)
}"#,
        expected_stdout: "false\n",
    },
    SuccessCase {
        id: "test_greater_equal_true_greater",
        source: r#"fn main() -> void {
    let result: bool = 5 >= 3
    println(result)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_greater_equal_true_equal",
        source: r#"fn main() -> void {
    let result: bool = 5 >= 5
    println(result)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_greater_equal_false",
        source: r#"fn main() -> void {
    let result: bool = 3 >= 5
    println(result)
}"#,
        expected_stdout: "false\n",
    },
    SuccessCase {
        id: "test_float_comparison_less_than",
        source: r#"fn main() -> void {
    let result: bool = 1.5 < 2.0
    println(result)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_float_comparison_mixed_f32_f64",
        source: r#"fn main() -> void {
    let a: f32 = 2.5
    let b: f64 = 2.5
    let result: bool = a == b
    println(result)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_comparison_with_i32_variables",
        source: r#"fn main() -> void {
    let x: i32 = 5
    let y: i32 = 3
    let result: bool = x > y
    println(result)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_comparison_i32_equal",
        source: r#"fn main() -> void {
    let x: i32 = 5
    let y: i32 = 5
    let result: bool = x == y
    println(result)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_comparison_i32_not_equal",
        source: r#"fn main() -> void {
    let x: i32 = 5
    let y: i32 = 3
    let result: bool = x != y
    println(result)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_comparison_i32_less_than",
        source: r#"fn main() -> void {
    let x: i32 = 3
    let y: i32 = 5
    let result: bool = x < y
    println(result)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_comparison_i32_less_equal",
        source: r#"fn main() -> void {
    let x: i32 = 5
    let y: i32 = 5
    let result: bool = x <= y
    println(result)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_comparison_i32_greater_equal",
        source: r#"fn main() -> void {
    let x: i32 = 5
    let y: i32 = 3
    let result: bool = x >= y
    println(result)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_comparison_with_i64_variables",
        source: r#"fn main() -> void {
    let x: i64 = 1000000000000
    let y: i64 = 999999999999
    let result: bool = x > y
    println(result)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_comparison_i64_equal",
        source: r#"fn main() -> void {
    let x: i64 = 1000000000000
    let y: i64 = 1000000000000
    let result: bool = x == y
    println(result)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_comparison_i64_not_equal",
        source: r#"fn main() -> void {
    let x: i64 = 1000000000000
    let y: i64 = 999999999999
    let result: bool = x != y
    println(result)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_comparison_i64_less_than",
        source: r#"fn main() -> void {
    let x: i64 = 999999999999
    let y: i64 = 1000000000000
    let result: bool = x < y
    println(result)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_comparison_i64_less_equal",
        source: r#"fn main() -> void {
    let x: i64 = 1000000000000
    let y: i64 = 1000000000000
    let result: bool = x <= y
    println(result)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_comparison_i64_greater_equal",
        source: r#"fn main() -> void {
    let x: i64 = 1000000000000
    let y: i64 = 999999999999
    let result: bool = x >= y
    println(result)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_comparison_with_inferred_i64_variable",
        source: r#"fn main() -> void {
    let x = 7
    let result: bool = x > 3
    println(result)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_comparison_inferred_and_explicit_i64_variables",
        source: r#"fn main() -> void {
    let x = 10
    let y: i64 = 5
    let result: bool = x > y
    println(result)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_comparison_negative_numbers",
        source: r#"fn main() -> void {
    let x: i32 = -5
    let y: i32 = -3
    let result: bool = x < y
    println(result)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_comparison_negative_greater_than",
        source: r#"fn main() -> void {
    let x: i32 = -3
    let y: i32 = -5
    let result: bool = x > y
    println(result)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_comparison_negative_equal",
        source: r#"fn main() -> void {
    let x: i32 = -5
    let y: i32 = -5
    let result: bool = x == y
    println(result)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_comparison_negative_not_equal",
        source: r#"fn main() -> void {
    let x: i32 = -5
    let y: i32 = -3
    let result: bool = x != y
    println(result)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_comparison_negative_less_equal",
        source: r#"fn main() -> void {
    let x: i32 = -5
    let y: i32 = -3
    let result: bool = x <= y
    println(result)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_comparison_negative_greater_equal",
        source: r#"fn main() -> void {
    let x: i32 = -3
    let y: i32 = -5
    let result: bool = x >= y
    println(result)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_comparison_negative_vs_positive",
        source: r#"fn main() -> void {
    let x: i32 = -1
    let y: i32 = 0
    let result: bool = x > y
    println(result)
}"#,
        expected_stdout: "false\n",
    },
    SuccessCase {
        id: "test_comparison_positive_vs_negative",
        source: r#"fn main() -> void {
    let x: i32 = 0
    let y: i32 = -1
    let result: bool = x >= y
    println(result)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_precedence_arithmetic_before_comparison",
        source: r#"fn main() -> void {
    let result: bool = 2 + 3 < 4 * 2
    println(result)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_precedence_comparison_before_equality",
        source: r#"fn main() -> void {
    let result: bool = 1 < 2 == true
    println(result)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_precedence_comparison_before_inequality",
        source: r#"fn main() -> void {
    let result: bool = 3 > 2 != false
    println(result)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_println_comparison_directly",
        source: r#"fn main() -> void {
    println(5 > 3)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_println_equality_with_arithmetic",
        source: r#"fn main() -> void {
    println(2 + 3 == 5)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_println_less_than_directly",
        source: r#"fn main() -> void {
    println(3 < 5)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_println_not_equal_directly",
        source: r#"fn main() -> void {
    println(5 != 3)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_println_less_equal_directly",
        source: r#"fn main() -> void {
    println(3 <= 5)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_println_greater_equal_directly",
        source: r#"fn main() -> void {
    println(5 >= 3)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_println_bool_not_equal_directly",
        source: r#"fn main() -> void {
    println(true != false)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_println_string_not_equal_directly",
        source: r#"fn main() -> void {
    println("hello" != "world")
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_println_comparison_with_i32_variable_and_literal_left",
        source: r#"fn main() -> void {
    let x: i32 = 6
    println(5 > x)
}"#,
        expected_stdout: "false\n",
    },
    SuccessCase {
        id: "test_println_comparison_with_i32_variable_and_literal_right",
        source: r#"fn main() -> void {
    let x: i32 = 6
    println(x < 10)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_println_comparison_with_i64_variable_and_literal_left",
        source: r#"fn main() -> void {
    let x: i64 = 6
    println(5 > x)
}"#,
        expected_stdout: "false\n",
    },
    SuccessCase {
        id: "test_println_comparison_with_i64_variable_and_literal_right",
        source: r#"fn main() -> void {
    let x: i64 = 6
    println(x < 10)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_println_equality_with_i32_variable_and_literal_left",
        source: r#"fn main() -> void {
    let x: i32 = 6
    println(6 == x)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_println_inequality_with_i32_variable_and_literal_right",
        source: r#"fn main() -> void {
    let x: i32 = 6
    println(x != 10)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_println_equality_with_i64_variable_and_literal_left",
        source: r#"fn main() -> void {
    let x: i64 = 6
    println(6 == x)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_println_inequality_with_i64_variable_and_literal_right",
        source: r#"fn main() -> void {
    let x: i64 = 6
    println(x != 10)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_bool_equal_true",
        source: r#"fn main() -> void {
    let result: bool = true == true
    println(result)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_bool_equal_false",
        source: r#"fn main() -> void {
    let result: bool = true == false
    println(result)
}"#,
        expected_stdout: "false\n",
    },
    SuccessCase {
        id: "test_bool_not_equal_true",
        source: r#"fn main() -> void {
    let result: bool = true != false
    println(result)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_bool_not_equal_false",
        source: r#"fn main() -> void {
    let result: bool = false != false
    println(result)
}"#,
        expected_stdout: "false\n",
    },
    SuccessCase {
        id: "test_bool_equal_with_variables",
        source: r#"fn main() -> void {
    let x: bool = true
    let y: bool = true
    let result: bool = x == y
    println(result)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_println_bool_equality_directly",
        source: r#"fn main() -> void {
    println(true == true)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_string_equal_true",
        source: r#"fn main() -> void {
    let result: bool = "hello" == "hello"
    println(result)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_string_equal_false",
        source: r#"fn main() -> void {
    let result: bool = "hello" == "world"
    println(result)
}"#,
        expected_stdout: "false\n",
    },
    SuccessCase {
        id: "test_string_not_equal_true",
        source: r#"fn main() -> void {
    let result: bool = "hello" != "world"
    println(result)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_string_not_equal_false",
        source: r#"fn main() -> void {
    let result: bool = "hello" != "hello"
    println(result)
}"#,
        expected_stdout: "false\n",
    },
    SuccessCase {
        id: "test_string_equal_with_variables",
        source: r#"fn main() -> void {
    let x: string = "hello"
    let y: string = "hello"
    let result: bool = x == y
    println(result)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_string_not_equal_with_variables",
        source: r#"fn main() -> void {
    let x: string = "hello"
    let y: string = "world"
    let result: bool = x != y
    println(result)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_string_equal_empty_strings",
        source: r#"fn main() -> void {
    let result: bool = "" == ""
    println(result)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_println_string_equality_directly",
        source: r#"fn main() -> void {
    println("hello" == "hello")
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_string_less_than_true",
        source: r#"fn main() -> void {
    let result: bool = "apple" < "banana"
    println(result)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_string_greater_than_true",
        source: r#"fn main() -> void {
    let result: bool = "banana" > "apple"
    println(result)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_string_less_than_false",
        source: r#"fn main() -> void {
    let result: bool = "banana" < "apple"
    println(result)
}"#,
        expected_stdout: "false\n",
    },
    SuccessCase {
        id: "test_string_greater_than_false",
        source: r#"fn main() -> void {
    let result: bool = "apple" > "banana"
    println(result)
}"#,
        expected_stdout: "false\n",
    },
    SuccessCase {
        id: "test_string_less_equal_true_equal",
        source: r#"fn main() -> void {
    let result: bool = "lak" <= "lak"
    println(result)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_string_less_equal_true_less",
        source: r#"fn main() -> void {
    let result: bool = "apple" <= "banana"
    println(result)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_string_less_equal_false",
        source: r#"fn main() -> void {
    let result: bool = "banana" <= "apple"
    println(result)
}"#,
        expected_stdout: "false\n",
    },
    SuccessCase {
        id: "test_string_greater_equal_true_equal",
        source: r#"fn main() -> void {
    let result: bool = "lak" >= "lak"
    println(result)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_string_greater_equal_true_greater",
        source: r#"fn main() -> void {
    let result: bool = "banana" >= "apple"
    println(result)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_string_greater_equal_false",
        source: r#"fn main() -> void {
    let result: bool = "apple" >= "banana"
    println(result)
}"#,
        expected_stdout: "false\n",
    },
    SuccessCase {
        id: "test_string_less_than_equal_values",
        source: r#"fn main() -> void {
    let result: bool = "lak" < "lak"
    println(result)
}"#,
        expected_stdout: "false\n",
    },
    SuccessCase {
        id: "test_string_greater_than_equal_values",
        source: r#"fn main() -> void {
    let result: bool = "lak" > "lak"
    println(result)
}"#,
        expected_stdout: "false\n",
    },
    SuccessCase {
        id: "test_string_ordering_with_variables",
        source: r#"fn main() -> void {
    let a: string = "alpha"
    let b: string = "beta"
    println(a < b)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_println_string_ordering_directly",
        source: r#"fn main() -> void {
    println("apple" < "banana")
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_string_ordering_lexicographical_numeric_text",
        source: r#"fn main() -> void {
    let x: string = "z"
    let y: string = "10"
    println(x > y)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_comparison_of_comparisons_equal",
        source: r#"fn main() -> void {
    let a: bool = 5 > 3
    let b: bool = 2 < 4
    let result: bool = a == b
    println(result)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_comparison_of_comparisons_not_equal",
        source: r#"fn main() -> void {
    let a: bool = 5 > 3
    let b: bool = 2 > 4
    let result: bool = a != b
    println(result)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_comparison_zero_equal",
        source: r#"fn main() -> void {
    let result: bool = 0 == 0
    println(result)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_comparison_zero_less_than",
        source: r#"fn main() -> void {
    let x: i32 = 0
    let y: i32 = 1
    let result: bool = x < y
    println(result)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_comparison_negative_less_than_zero",
        source: r#"fn main() -> void {
    let x: i32 = -1
    let y: i32 = 0
    let result: bool = x < y
    println(result)
}"#,
        expected_stdout: "true\n",
    },
    SuccessCase {
        id: "test_comparison_zero_greater_equal_negative",
        source: r#"fn main() -> void {
    let x: i32 = 0
    let y: i32 = -1
    let result: bool = x >= y
    println(result)
}"#,
        expected_stdout: "true\n",
    },
];

#[test]
fn test_success_cases() {
    // Pure comparison output checks are independent and safe to batch (R-08 control).
    run_batched_success_cases("e2e_comparison_success", SUCCESS_CASES);
}
