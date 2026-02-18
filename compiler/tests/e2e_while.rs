//! End-to-end tests for while statements.

mod common;

use common::{compile_and_run, executable_name, lak_binary};
use std::fs;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;
use tempfile::tempdir;

#[test]
fn test_while_false_skips_body() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    while false {
        println("loop")
    }
    println("done")
}
"#,
    )
    .unwrap();
    assert_eq!(output, "done\n");
}

#[test]
fn test_while_true_breaks() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    while true {
        println("once")
        break
    }
    println("done")
}
"#,
    )
    .unwrap();
    assert_eq!(output, "once\ndone\n");
}

#[test]
fn test_continue_inside_while() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    while true {
        if false {
            continue
        }
        break
    }
    println("ok")
}
"#,
    )
    .unwrap();
    assert_eq!(output, "ok\n");
}

#[test]
fn test_non_void_function_return_inside_while_true() {
    let output = compile_and_run(
        r#"
fn helper() -> i64 {
    while true {
        return 1
    }
}

fn main() -> void {
    println(helper())
}
"#,
    )
    .unwrap();
    assert_eq!(output, "1\n");
}

#[test]
fn test_nested_while_break_scopes_are_isolated() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    while true {
        println("outer-start")
        while true {
            println("inner")
            break
        }
        println("outer-after-inner")
        break
    }
    println("done")
}
"#,
    )
    .unwrap();
    assert_eq!(output, "outer-start\ninner\nouter-after-inner\ndone\n");
}

#[test]
fn test_while_loop_back_edge_executes_multiple_iterations() {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("loopback.lak");
    let executable_path = temp.path().join(executable_name("loopback"));

    fs::write(
        &source_path,
        r#"fn main() -> void {
    while true {
        println("tick")
    }
}"#,
    )
    .unwrap();

    let build_output = Command::new(lak_binary())
        .args([
            "build",
            source_path.to_str().unwrap(),
            "-o",
            executable_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        build_output.status.success(),
        "build failed: {}",
        String::from_utf8_lossy(&build_output.stderr)
    );

    let mut child = Command::new(&executable_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let stdout = child.stdout.take().expect("child stdout must be piped");
    let (tx, rx) = mpsc::channel();
    let reader_handle = std::thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            let line = match line {
                Ok(line) => line,
                Err(_) => break,
            };
            if tx.send(line).is_err() {
                break;
            }
        }
    });

    let first = rx
        .recv_timeout(Duration::from_secs(5))
        .expect("expected first loop iteration output");
    let second = rx
        .recv_timeout(Duration::from_secs(5))
        .expect("expected second loop iteration output");
    assert_eq!(first, "tick");
    assert_eq!(second, "tick");

    let _ = child.kill();
    let _ = child.wait();
    drop(rx);
    let _ = reader_handle.join();
}
