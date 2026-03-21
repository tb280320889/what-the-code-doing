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
    /// Route a task description to candidate files
    Route {
        /// Natural language task description
        query: String,
        /// Maximum number of results to return
        #[arg(long, default_value = "10")]
        top_k: usize,
    },
    /// Start MCP Server (stdio mode)
    Mcp {},
    /// Print MCP config for an agent [claude|cursor]
    McpInstall {
        /// Agent name (claude or cursor)
        agent: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let result: anyhow::Result<()> = match cli.command {
        Commands::Init {} => commands::init::run_init(&cli.root).map_err(|e| anyhow::anyhow!(e)),
        Commands::Run { full } => commands::run::run_analysis(&cli.root, full).map_err(|e| anyhow::anyhow!(e)),
        Commands::Check {} => commands::check::run_check(&cli.root).map_err(|e| anyhow::anyhow!(e)),
        Commands::Route { query, top_k } => commands::route::run_route(&cli.root, &query, top_k).map_err(|e| anyhow::anyhow!(e)),
        Commands::Mcp {} => commands::mcp::run_mcp(&cli.root).await,
        Commands::McpInstall { agent } => commands::mcp_install::install_for_agent(&agent),
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
