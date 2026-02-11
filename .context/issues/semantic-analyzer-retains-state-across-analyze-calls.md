# `SemanticAnalyzer` Retains Symbols Across Multiple `analyze` Calls

## Summary

Reusing a single `SemanticAnalyzer` instance for multiple independent programs causes the second analysis to fail with a duplicate function error.  
The second program can be identical to the first and still fail on `main` duplication.

## Details

### Reproduction

Reproduction program:

```rust
use lak::lexer::Lexer;
use lak::parser::Parser;
use lak::semantic::SemanticAnalyzer;

fn parse_program(src: &str) -> lak::ast::Program {
    let mut lexer = Lexer::new(src);
    let tokens = lexer.tokenize().expect("lex failed");
    let mut parser = Parser::new(tokens);
    parser.parse().expect("parse failed")
}

fn main() {
    let src = "fn main() -> void {}";
    let p1 = parse_program(src);
    let p2 = parse_program(src);

    let mut analyzer = SemanticAnalyzer::new();

    match analyzer.analyze(&p1) {
        Ok(()) => println!("first analyze: ok"),
        Err(e) => println!("first analyze: err: {}", e),
    }

    match analyzer.analyze(&p2) {
        Ok(()) => println!("second analyze: ok"),
        Err(e) => println!("second analyze: err: {}", e),
    }
}
```

Observed output:

```text
first analyze: ok
second analyze: err: 1:1: Function 'main' is already defined at 1:1
```

### Related code locations

- `compiler/src/semantic/mod.rs:63` stores `symbols` in `SemanticAnalyzer`.
- `compiler/src/semantic/mod.rs:93` (`analyze`) calls `collect_functions`.
- `compiler/src/semantic/mod.rs:128` (`analyze_module`) calls `collect_functions`.
- `compiler/src/semantic/mod.rs:148` to `compiler/src/semantic/mod.rs:166` inserts function definitions into `self.symbols`.

## Expected Behavior

Module and entry-point semantics are specified per source/module unit (`.context/SPEC.md:803`, `.context/SPEC.md:851`).  
When independent valid programs are analyzed sequentially with the same analyzer instance, each analysis result is expected to reflect only declarations from the program given to that call.
