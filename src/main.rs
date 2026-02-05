mod ast;
mod codegen;
mod lexer;
mod parser;
mod token;

use clap::{Parser, Subcommand};
use codegen::Codegen;
use inkwell::context::Context;
use lexer::Lexer;
use parser::Parser as LakParser;
use std::path::Path;
use std::process::Command;

#[derive(Parser)]
#[command(name = "lak")]
#[command(about = "The Lak programming language", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build a Lak program
    Build {
        /// The source file to compile
        file: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build { file } => {
            if let Err(e) = build(&file) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }
}

fn build(file: &str) -> Result<(), String> {
    let source =
        std::fs::read_to_string(file).map_err(|e| format!("Failed to read file: {}", e))?;

    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize().map_err(|e| e.to_string())?;

    let mut parser = LakParser::new(tokens);
    let program = parser.parse().map_err(|e| e.to_string())?;

    let context = Context::create();
    let mut codegen = Codegen::new(&context, "lak_module");
    codegen.compile(&program)?;

    let source_path = Path::new(file);
    let stem = source_path
        .file_stem()
        .ok_or_else(|| format!("Cannot determine filename from path: {}", file))?
        .to_str()
        .ok_or_else(|| format!("Filename contains invalid UTF-8: {}", file))?;

    let object_path = format!("{}.o", stem);
    let output_path = stem.to_string();

    codegen.write_object_file(Path::new(&object_path))?;

    let output = Command::new("cc")
        .args([&object_path, "-o", &output_path])
        .output()
        .map_err(|e| format!("Failed to run linker: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let exit_code = output
            .status
            .code()
            .map(|c| c.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        return Err(format!(
            "Linker failed with exit code {}:\n{}",
            exit_code, stderr
        ));
    }

    if let Err(e) = std::fs::remove_file(&object_path) {
        if e.kind() != std::io::ErrorKind::NotFound {
            eprintln!(
                "Warning: Failed to remove temporary file '{}': {}",
                object_path, e
            );
        }
    }

    println!("Built: {}", output_path);
    Ok(())
}
