//! The Lak programming language compiler library.
//!
//! This library provides the core components of the Lak compiler:
//! lexical analysis, parsing, and code generation.
//!
//! # Modules
//!
//! - [`token`] - Token types and source location tracking
//! - [`lexer`] - Lexical analysis (tokenization)
//! - [`parser`] - Recursive descent parser
//! - [`ast`] - Abstract Syntax Tree definitions
//! - [`codegen`] - LLVM code generation
//!
//! # Example
//!
//! ```no_run
//! use lak::lexer::Lexer;
//! use lak::parser::Parser;
//! use lak::codegen::Codegen;
//! use inkwell::context::Context;
//! use std::path::Path;
//!
//! // Source code to compile
//! let source = r#"println("Hello, World!")"#;
//!
//! // Lexical analysis
//! let mut lexer = Lexer::new(source);
//! let tokens = lexer.tokenize().expect("Lexer error");
//!
//! // Parsing
//! let mut parser = Parser::new(tokens);
//! let program = parser.parse().expect("Parse error");
//!
//! // Code generation
//! let context = Context::create();
//! let mut codegen = Codegen::new(&context, "my_program");
//! codegen.compile(&program).expect("Codegen error");
//!
//! // Write object file
//! codegen.write_object_file(Path::new("output.o")).expect("Write error");
//! ```

pub mod ast;
pub mod codegen;
pub mod lexer;
pub mod parser;
pub mod token;
