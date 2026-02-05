use clap::{Parser, Subcommand};

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
    Build,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build => {
            println!("Hello, World!");
        }
    }
}
