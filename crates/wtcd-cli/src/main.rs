mod commands;
mod output;

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "wtcd", version = env!("CARGO_PKG_VERSION"), about = "What The Code Doing — AI-native repo semantic mirror")]
struct Cli {
    /// Output directory root
    #[arg(long, default_value = ".")]
    root: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Initialize wtcd configuration in current repo
    Init {},
    /// Run analysis on scoped files
    Run {
        /// Force full rebuild (skip incremental optimization)
        #[arg(long)]
        full: bool,
    },
    /// Check for drift between source and mirrors
    Check {},
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Commands::Init {} => commands::init::run_init(&cli.root),
        Commands::Run { full } => commands::run::run_analysis(&cli.root, full),
        Commands::Check {} => commands::check::run_check(&cli.root),
    };

    if let Err(e) = result {
        let error_json = serde_json::json!({
            "status": "error",
            "message": e.to_string(),
        });
        eprintln!("{}", serde_json::to_string_pretty(&error_json).unwrap());
        std::process::exit(1);
    }
}
