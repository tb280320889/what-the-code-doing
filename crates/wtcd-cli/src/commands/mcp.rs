use std::path::PathBuf;

use rmcp::service::ServiceExt;
use rmcp::transport::stdio;
use tracing_subscriber::EnvFilter;
use wtcd_mcp::server::WtcdMcpServer;

pub async fn run_mcp(root: &PathBuf) -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    tracing::info!("Starting WTCD MCP Server (stdio)");
    let server = WtcdMcpServer::new(root.clone());
    let service = server
        .serve(stdio())
        .await
        .inspect_err(|e| {
            tracing::error!("MCP server error: {}", e);
        })?;
    tracing::info!("WTCD MCP Server connected");
    service.waiting().await?;
    Ok(())
}
