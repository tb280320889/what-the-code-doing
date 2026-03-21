use anyhow::{anyhow, Result};

/// Generate MCP config snippet for supported agents
pub fn install_for_agent(agent: &str) -> Result<()> {
    match agent {
        "claude" => print_claude_config(),
        "cursor" => print_cursor_config(),
        other => Err(anyhow!(
            "Unsupported agent '{}'. Supported: claude, cursor",
            other
        )),
    }
}

fn print_claude_config() -> Result<()> {
    let config = serde_json::json!({
        "mcpServers": {
            "wtcd": {
                "command": "wtcd",
                "args": ["mcp"],
                "env": {}
            }
        }
    });
    println!("# Claude Code MCP Configuration");
    println!("# Add this to ~/.claude.json or run:");
    println!("#   claude mcp add --transport stdio wtcd -- wtcd mcp");
    println!();
    println!("{}", serde_json::to_string_pretty(&config)?);
    Ok(())
}

fn print_cursor_config() -> Result<()> {
    let config = serde_json::json!({
        "mcpServers": {
            "wtcd": {
                "command": "wtcd",
                "args": ["mcp"]
            }
        }
    });
    println!("# Cursor MCP Configuration");
    println!("# Save to .cursor/mcp.json in your project root");
    println!();
    println!("{}", serde_json::to_string_pretty(&config)?);
    Ok(())
}
