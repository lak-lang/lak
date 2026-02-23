//! The Lak programming language compiler CLI.
//!
//! This binary provides `lak build` and `lak run` commands and delegates
//! compilation/link/run orchestration to the driver module.

use clap::{Parser, Subcommand};

mod diagnostics;
mod driver;

/// Command-line interface for the Lak compiler.
#[derive(Parser)]
#[command(name = "lak")]
#[command(about = "The Lak programming language", long_about = None)]
struct Cli {
    /// The subcommand to execute.
    #[command(subcommand)]
    command: Commands,
}

/// Available CLI subcommands.
#[derive(Subcommand)]
enum Commands {
    /// Build a Lak program into a native executable.
    Build {
        /// The source file to compile (e.g., `hello.lak`).
        file: String,

        /// Output path for the executable (e.g., `-o myprogram`).
        /// If not specified, uses the input filename without extension.
        #[arg(short = 'o', long = "output")]
        output: Option<String>,
    },
    /// Compile and run a Lak program.
    Run {
        /// The source file to run (e.g., `hello.lak`).
        file: String,
    },
}

/// Entry point for the Lak compiler.
fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build { file, output } => {
            if let Err(error) = driver::build(&file, output.as_deref()) {
                report_and_exit(*error);
            }
        }
        Commands::Run { file } => match driver::run(&file) {
            Ok(exit_code) => std::process::exit(exit_code),
            Err(error) => report_and_exit(*error),
        },
    }
}

fn report_and_exit(error: driver::CompileErrorWithContext) -> ! {
    diagnostics::report_error(error.filename(), error.source(), error.error());
    std::process::exit(1);
}
