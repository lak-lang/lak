//! Expression parsing tests.
//!
//! Tests for:
//! - Basic expression parsing (calls, literals, identifiers)
//! - Nested function calls
//! - Integer literals
//! - Variable references

use super::*;

// ===================
// Basic expression parsing (within function body)
// ===================

#[test]
fn test_call_no_args() {
    let expr = parse_first_expr("func()");
    match expr.kind {
        ExprKind::Call { callee, args } => {
            assert_eq!(callee, "func");
            assert!(args.is_empty());
        }
        _ => panic!("Expected Call expression"),
    }
}

#[test]
fn test_call_one_arg() {
    let expr = parse_first_expr(r#"println("hello")"#);
    match expr.kind {
        ExprKind::Call { callee, args } => {
            assert_eq!(callee, "println");
            assert_eq!(args.len(), 1);
            assert!(matches!(&args[0].kind, ExprKind::StringLiteral(s) if s == "hello"));
        }
        _ => panic!("Expected Call expression"),
    }
}

#[test]
fn test_call_multiple_args() {
    let expr = parse_first_expr(r#"f("a", "b", "c")"#);
    match expr.kind {
        ExprKind::Call { callee, args } => {
            assert_eq!(callee, "f");
            assert_eq!(args.len(), 3);
            assert!(matches!(&args[0].kind, ExprKind::StringLiteral(s) if s == "a"));
            assert!(matches!(&args[1].kind, ExprKind::StringLiteral(s) if s == "b"));
            assert!(matches!(&args[2].kind, ExprKind::StringLiteral(s) if s == "c"));
        }
        _ => panic!("Expected Call expression"),
    }
}

// ===================
// Nested calls
// ===================

#[test]
fn test_nested_call_single() {
    let expr = parse_first_expr("outer(inner())");
    match expr.kind {
        ExprKind::Call { callee, args } => {
            assert_eq!(callee, "outer");
            assert_eq!(args.len(), 1);
            match &args[0].kind {
                ExprKind::Call {
                    callee: inner_callee,
                    args: inner_args,
                } => {
                    assert_eq!(inner_callee, "inner");
                    assert!(inner_args.is_empty());
                }
                _ => panic!("Expected nested Call"),
            }
        }
        _ => panic!("Expected Call expression"),
    }
}

#[test]
fn test_nested_call_with_arg() {
    let expr = parse_first_expr(r#"outer(inner("x"))"#);
    match expr.kind {
        ExprKind::Call { callee, args } => {
            assert_eq!(callee, "outer");
            assert_eq!(args.len(), 1);
            match &args[0].kind {
                ExprKind::Call {
                    callee: inner_callee,
                    args: inner_args,
                } => {
                    assert_eq!(inner_callee, "inner");
                    assert_eq!(inner_args.len(), 1);
                    assert!(matches!(&inner_args[0].kind, ExprKind::StringLiteral(s) if s == "x"));
                }
                _ => panic!("Expected nested Call"),
            }
        }
        _ => panic!("Expected Call expression"),
    }
}

#[test]
fn test_deeply_nested() {
    let expr = parse_first_expr("a(b(c(d())))");
    match expr.kind {
        ExprKind::Call { callee, args } => {
            assert_eq!(callee, "a");
            assert_eq!(args.len(), 1);
            // Verify structure: a -> b -> c -> d
            match &args[0].kind {
                ExprKind::Call { callee: b, args } => {
                    assert_eq!(b, "b");
                    match &args[0].kind {
                        ExprKind::Call { callee: c, args } => {
                            assert_eq!(c, "c");
                            match &args[0].kind {
                                ExprKind::Call { callee: d, args } => {
                                    assert_eq!(d, "d");
                                    assert!(args.is_empty());
                                }
                                _ => panic!("Expected d call"),
                            }
                        }
                        _ => panic!("Expected c call"),
                    }
                }
                _ => panic!("Expected b call"),
            }
        }
        _ => panic!("Expected Call expression"),
    }
}

#[test]
fn test_nested_multiple_args() {
    let expr = parse_first_expr(r#"f(g(), h(), "x")"#);
    match expr.kind {
        ExprKind::Call { callee, args } => {
            assert_eq!(callee, "f");
            assert_eq!(args.len(), 3);
            assert!(matches!(&args[0].kind, ExprKind::Call { callee, .. } if callee == "g"));
            assert!(matches!(&args[1].kind, ExprKind::Call { callee, .. } if callee == "h"));
            assert!(matches!(&args[2].kind, ExprKind::StringLiteral(s) if s == "x"));
        }
        _ => panic!("Expected Call expression"),
    }
}

// ===================
// Expression types
// ===================

#[test]
fn test_string_literal_as_arg() {
    let expr = parse_first_expr(r#"f("str")"#);
    match expr.kind {
        ExprKind::Call { args, .. } => {
            assert!(matches!(&args[0].kind, ExprKind::StringLiteral(s) if s == "str"));
        }
        _ => panic!("Expected Call"),
    }
}

#[test]
fn test_call_as_arg() {
    let expr = parse_first_expr("f(g())");
    match expr.kind {
        ExprKind::Call { args, .. } => {
            assert!(matches!(&args[0].kind, ExprKind::Call { callee, .. } if callee == "g"));
        }
        _ => panic!("Expected Call"),
    }
}

// ===================
// Integer literal parsing
// ===================

#[test]
fn test_int_literal_zero() {
    let program = parse("fn main() -> void { let x: i32 = 0 }").unwrap();
    match &program.functions[0].body[0].kind {
        StmtKind::Let { init, .. } => {
            assert!(matches!(init.kind, ExprKind::IntLiteral(0)));
        }
        _ => panic!("Expected Let statement"),
    }
}

#[test]
fn test_int_literal_large() {
    let program = parse("fn main() -> void { let x: i64 = 9223372036854775807 }").unwrap();
    match &program.functions[0].body[0].kind {
        StmtKind::Let { init, .. } => {
            assert!(matches!(
                init.kind,
                ExprKind::IntLiteral(9223372036854775807)
            ));
        }
        _ => panic!("Expected Let statement"),
    }
}

#[test]
fn test_int_literal_i64_min() {
    // -9223372036854775808 is i64::MIN, only representable via negation folding
    let program = parse("fn main() -> void { let x: i64 = -9223372036854775808 }").unwrap();
    match &program.functions[0].body[0].kind {
        StmtKind::Let { init, .. } => {
            assert!(matches!(init.kind, ExprKind::IntLiteral(i64::MIN)));
        }
        _ => panic!("Expected Let statement"),
    }
}

#[test]
fn test_int_literal_neg_i64_max() {
    // -9223372036854775807 hits the unsigned_value <= i64::MAX branch
    let program = parse("fn main() -> void { let x: i64 = -9223372036854775807 }").unwrap();
    match &program.functions[0].body[0].kind {
        StmtKind::Let { init, .. } => {
            assert!(matches!(
                init.kind,
                ExprKind::IntLiteral(-9223372036854775807)
            ));
        }
        _ => panic!("Expected Let statement"),
    }
}

// ===================
// Variable reference parsing
// ===================

#[test]
fn test_variable_reference_in_init() {
    let program = parse("fn main() -> void { let a: i32 = 1\nlet b: i32 = a }").unwrap();
    match &program.functions[0].body[1].kind {
        StmtKind::Let { init, .. } => {
            assert!(matches!(&init.kind, ExprKind::Identifier(s) if s == "a"));
        }
        _ => panic!("Expected Let statement"),
    }
}

// ===================
// Binary operation parsing
// ===================

#[test]
fn test_binary_op_addition() {
    let program = parse("fn main() -> void { let x: i32 = 1 + 2 }").unwrap();
    match &program.functions[0].body[0].kind {
        StmtKind::Let { init, .. } => match &init.kind {
            ExprKind::BinaryOp { left, op, right } => {
                assert_eq!(*op, BinaryOperator::Add);
                assert!(matches!(left.kind, ExprKind::IntLiteral(1)));
                assert!(matches!(right.kind, ExprKind::IntLiteral(2)));
            }
            _ => panic!("Expected BinaryOp"),
        },
        _ => panic!("Expected Let statement"),
    }
}

#[test]
fn test_binary_op_precedence_mul_before_add() {
    // 1 + 2 * 3 should parse as 1 + (2 * 3)
    let program = parse("fn main() -> void { let x: i32 = 1 + 2 * 3 }").unwrap();
    match &program.functions[0].body[0].kind {
        StmtKind::Let { init, .. } => match &init.kind {
            ExprKind::BinaryOp { left, op, right } => {
                assert_eq!(*op, BinaryOperator::Add);
                assert!(matches!(left.kind, ExprKind::IntLiteral(1)));
                // right should be 2 * 3
                match &right.kind {
                    ExprKind::BinaryOp {
                        left: inner_left,
                        op: inner_op,
                        right: inner_right,
                    } => {
                        assert_eq!(*inner_op, BinaryOperator::Mul);
                        assert!(matches!(inner_left.kind, ExprKind::IntLiteral(2)));
                        assert!(matches!(inner_right.kind, ExprKind::IntLiteral(3)));
                    }
                    _ => panic!("Expected nested BinaryOp for 2 * 3"),
                }
            }
            _ => panic!("Expected BinaryOp"),
        },
        _ => panic!("Expected Let statement"),
    }
}

#[test]
fn test_binary_op_left_associativity() {
    // 10 - 5 - 2 should parse as (10 - 5) - 2
    let program = parse("fn main() -> void { let x: i32 = 10 - 5 - 2 }").unwrap();
    match &program.functions[0].body[0].kind {
        StmtKind::Let { init, .. } => match &init.kind {
            ExprKind::BinaryOp { left, op, right } => {
                assert_eq!(*op, BinaryOperator::Sub);
                assert!(matches!(right.kind, ExprKind::IntLiteral(2)));
                // left should be 10 - 5
                match &left.kind {
                    ExprKind::BinaryOp {
                        left: inner_left,
                        op: inner_op,
                        right: inner_right,
                    } => {
                        assert_eq!(*inner_op, BinaryOperator::Sub);
                        assert!(matches!(inner_left.kind, ExprKind::IntLiteral(10)));
                        assert!(matches!(inner_right.kind, ExprKind::IntLiteral(5)));
                    }
                    _ => panic!("Expected nested BinaryOp for 10 - 5"),
                }
            }
            _ => panic!("Expected BinaryOp"),
        },
        _ => panic!("Expected Let statement"),
    }
}

#[test]
fn test_binary_op_parentheses_override_precedence() {
    // (1 + 2) * 3 should parse as (1 + 2) * 3 (addition first)
    let program = parse("fn main() -> void { let x: i32 = (1 + 2) * 3 }").unwrap();
    match &program.functions[0].body[0].kind {
        StmtKind::Let { init, .. } => match &init.kind {
            ExprKind::BinaryOp { left, op, right } => {
                assert_eq!(*op, BinaryOperator::Mul);
                assert!(matches!(right.kind, ExprKind::IntLiteral(3)));
                // left should be 1 + 2
                match &left.kind {
                    ExprKind::BinaryOp {
                        left: inner_left,
                        op: inner_op,
                        right: inner_right,
                    } => {
                        assert_eq!(*inner_op, BinaryOperator::Add);
                        assert!(matches!(inner_left.kind, ExprKind::IntLiteral(1)));
                        assert!(matches!(inner_right.kind, ExprKind::IntLiteral(2)));
                    }
                    _ => panic!("Expected nested BinaryOp for 1 + 2"),
                }
            }
            _ => panic!("Expected BinaryOp"),
        },
        _ => panic!("Expected Let statement"),
    }
}

#[test]
fn test_binary_op_all_operators() {
    let operators = [
        ("+", BinaryOperator::Add),
        ("-", BinaryOperator::Sub),
        ("*", BinaryOperator::Mul),
        ("/", BinaryOperator::Div),
        ("%", BinaryOperator::Mod),
    ];

    for (op_str, expected_op) in operators {
        let source = format!("fn main() -> void {{ let x: i32 = 1 {} 2 }}", op_str);
        let program = parse(&source).unwrap();
        match &program.functions[0].body[0].kind {
            StmtKind::Let { init, .. } => match &init.kind {
                ExprKind::BinaryOp { op, .. } => {
                    assert_eq!(*op, expected_op, "Failed for operator {}", op_str);
                }
                _ => panic!("Expected BinaryOp for {}", op_str),
            },
            _ => panic!("Expected Let statement for {}", op_str),
        }
    }
}

#[test]
fn test_binary_op_with_variables() {
    let program =
        parse("fn main() -> void { let a: i32 = 1\nlet b: i32 = 2\nlet c: i32 = a + b }").unwrap();
    match &program.functions[0].body[2].kind {
        StmtKind::Let { init, .. } => match &init.kind {
            ExprKind::BinaryOp { left, op, right } => {
                assert_eq!(*op, BinaryOperator::Add);
                assert!(matches!(&left.kind, ExprKind::Identifier(s) if s == "a"));
                assert!(matches!(&right.kind, ExprKind::Identifier(s) if s == "b"));
            }
            _ => panic!("Expected BinaryOp"),
        },
        _ => panic!("Expected Let statement"),
    }
}

// ===================
// Unary operation parsing
// ===================

#[test]
fn test_unary_minus_literal() {
    // -5 is folded into IntLiteral(-5) by the parser
    let program = parse("fn main() -> void { let x: i32 = -5 }").unwrap();
    match &program.functions[0].body[0].kind {
        StmtKind::Let { init, .. } => {
            assert!(matches!(init.kind, ExprKind::IntLiteral(-5)));
        }
        _ => panic!("Expected Let statement"),
    }
}

#[test]
fn test_unary_minus_identifier() {
    let program = parse("fn main() -> void { let a: i32 = 5\nlet b: i32 = -a }").unwrap();
    match &program.functions[0].body[1].kind {
        StmtKind::Let { init, .. } => match &init.kind {
            ExprKind::UnaryOp { op, operand } => {
                assert_eq!(*op, UnaryOperator::Neg);
                assert!(matches!(&operand.kind, ExprKind::Identifier(s) if s == "a"));
            }
            _ => panic!("Expected UnaryOp"),
        },
        _ => panic!("Expected Let statement"),
    }
}

#[test]
fn test_unary_minus_precedence() {
    // -2 * 3 should parse as (-2) * 3, with -2 folded into IntLiteral(-2)
    let program = parse("fn main() -> void { let x: i32 = -2 * 3 }").unwrap();
    match &program.functions[0].body[0].kind {
        StmtKind::Let { init, .. } => match &init.kind {
            ExprKind::BinaryOp { left, op, right } => {
                assert_eq!(*op, BinaryOperator::Mul);
                assert!(matches!(left.kind, ExprKind::IntLiteral(-2)));
                assert!(matches!(right.kind, ExprKind::IntLiteral(3)));
            }
            _ => panic!("Expected BinaryOp"),
        },
        _ => panic!("Expected Let statement"),
    }
}

#[test]
fn test_double_unary_minus() {
    // --5 should parse as UnaryOp(Neg, IntLiteral(-5)) due to folding
    let program = parse("fn main() -> void { let x: i32 = --5 }").unwrap();
    match &program.functions[0].body[0].kind {
        StmtKind::Let { init, .. } => match &init.kind {
            ExprKind::UnaryOp { op, operand } => {
                assert_eq!(*op, UnaryOperator::Neg);
                assert!(matches!(operand.kind, ExprKind::IntLiteral(-5)));
            }
            _ => panic!("Expected UnaryOp"),
        },
        _ => panic!("Expected Let statement"),
    }
}

#[test]
fn test_unary_minus_with_parens() {
    // -(3 + 2) should parse correctly
    let program = parse("fn main() -> void { let x: i32 = -(3 + 2) }").unwrap();
    match &program.functions[0].body[0].kind {
        StmtKind::Let { init, .. } => match &init.kind {
            ExprKind::UnaryOp { op, operand } => {
                assert_eq!(*op, UnaryOperator::Neg);
                match &operand.kind {
                    ExprKind::BinaryOp {
                        left,
                        op: bin_op,
                        right,
                    } => {
                        assert_eq!(*bin_op, BinaryOperator::Add);
                        assert!(matches!(left.kind, ExprKind::IntLiteral(3)));
                        assert!(matches!(right.kind, ExprKind::IntLiteral(2)));
                    }
                    _ => panic!("Expected BinaryOp inside parens"),
                }
            }
            _ => panic!("Expected UnaryOp"),
        },
        _ => panic!("Expected Let statement"),
    }
}

#[test]
fn test_subtraction_vs_unary_minus() {
    // 5 - -3 should parse as BinaryOp(Sub, 5, IntLiteral(-3)) due to folding
    let program = parse("fn main() -> void { let x: i32 = 5 - -3 }").unwrap();
    match &program.functions[0].body[0].kind {
        StmtKind::Let { init, .. } => match &init.kind {
            ExprKind::BinaryOp { left, op, right } => {
                assert_eq!(*op, BinaryOperator::Sub);
                assert!(matches!(left.kind, ExprKind::IntLiteral(5)));
                assert!(matches!(right.kind, ExprKind::IntLiteral(-3)));
            }
            _ => panic!("Expected BinaryOp"),
        },
        _ => panic!("Expected Let statement"),
    }
}

#[test]
fn test_unary_minus_in_function_arg() {
    // println(-42) should fold to IntLiteral(-42)
    let program = parse("fn main() -> void { println(-42) }").unwrap();
    match &program.functions[0].body[0].kind {
        StmtKind::Expr(expr) => match &expr.kind {
            ExprKind::Call { callee, args } => {
                assert_eq!(callee, "println");
                assert_eq!(args.len(), 1);
                assert!(matches!(args[0].kind, ExprKind::IntLiteral(-42)));
            }
            _ => panic!("Expected Call expression"),
        },
        _ => panic!("Expected Expr statement"),
    }
}

#[test]
fn test_unary_minus_with_newline() {
    // -\n5 should fold to IntLiteral(-5) (skip_newlines then fold)
    let program = parse("fn main() -> void { let x: i32 = -\n5 }").unwrap();
    match &program.functions[0].body[0].kind {
        StmtKind::Let { init, .. } => {
            assert!(matches!(init.kind, ExprKind::IntLiteral(-5)));
        }
        _ => panic!("Expected Let statement"),
    }
}

#[test]
fn test_triple_unary_minus() {
    // ---5 should parse as UnaryOp(Neg, UnaryOp(Neg, IntLiteral(-5))) due to folding
    let program = parse("fn main() -> void { let x: i32 = ---5 }").unwrap();
    match &program.functions[0].body[0].kind {
        StmtKind::Let { init, .. } => match &init.kind {
            ExprKind::UnaryOp { op: op1, operand } => {
                assert_eq!(*op1, UnaryOperator::Neg);
                match &operand.kind {
                    ExprKind::UnaryOp { op: op2, operand } => {
                        assert_eq!(*op2, UnaryOperator::Neg);
                        assert!(matches!(operand.kind, ExprKind::IntLiteral(-5)));
                    }
                    _ => panic!("Expected inner UnaryOp"),
                }
            }
            _ => panic!("Expected outer UnaryOp"),
        },
        _ => panic!("Expected Let statement"),
    }
}

// ===================
// Member access parsing
// ===================

#[test]
fn test_member_access_basic() {
    let expr = parse_first_expr("math.add");
    match expr.kind {
        ExprKind::MemberAccess { object, member } => {
            assert!(matches!(&object.kind, ExprKind::Identifier(s) if s == "math"));
            assert_eq!(member, "add");
        }
        _ => panic!("Expected MemberAccess expression, got {:?}", expr.kind),
    }
}

#[test]
fn test_member_access_span() {
    // "math.add" in context "fn test() -> void { math.add }"
    // The span should cover "math.add"
    let expr = parse_first_expr("math.add");
    match &expr.kind {
        ExprKind::MemberAccess { object, member } => {
            // object span should cover "math"
            assert!(object.span.end > object.span.start);
            // member name
            assert_eq!(member, "add");
            // expr span should cover entire "math.add"
            assert!(expr.span.end > expr.span.start);
            assert!(expr.span.start <= object.span.start);
        }
        _ => panic!("Expected MemberAccess expression"),
    }
}

#[test]
fn test_member_call_basic() {
    // math.add(1, 2) is converted to ModuleCall with separate module and function
    let expr = parse_first_expr("math.add(1, 2)");
    match expr.kind {
        ExprKind::ModuleCall {
            module,
            function,
            args,
        } => {
            assert_eq!(module, "math");
            assert_eq!(function, "add");
            assert_eq!(args.len(), 2);
            assert!(matches!(args[0].kind, ExprKind::IntLiteral(1)));
            assert!(matches!(args[1].kind, ExprKind::IntLiteral(2)));
        }
        _ => panic!("Expected ModuleCall expression, got {:?}", expr.kind),
    }
}

#[test]
fn test_member_call_no_args() {
    let expr = parse_first_expr("module.func()");
    match expr.kind {
        ExprKind::ModuleCall {
            module,
            function,
            args,
        } => {
            assert_eq!(module, "module");
            assert_eq!(function, "func");
            assert!(args.is_empty());
        }
        _ => panic!("Expected ModuleCall expression"),
    }
}

#[test]
fn test_member_call_string_arg() {
    let expr = parse_first_expr(r#"io.print("hello")"#);
    match expr.kind {
        ExprKind::ModuleCall {
            module,
            function,
            args,
        } => {
            assert_eq!(module, "io");
            assert_eq!(function, "print");
            assert_eq!(args.len(), 1);
            assert!(matches!(&args[0].kind, ExprKind::StringLiteral(s) if s == "hello"));
        }
        _ => panic!("Expected ModuleCall expression"),
    }
}

#[test]
fn test_member_access_vs_member_call() {
    // Without parentheses: MemberAccess
    let access_expr = parse_first_expr("foo.bar");
    assert!(matches!(access_expr.kind, ExprKind::MemberAccess { .. }));

    // With parentheses: ModuleCall
    let call_expr = parse_first_expr("foo.bar()");
    assert!(matches!(call_expr.kind, ExprKind::ModuleCall { .. }));
}
