use rmcp::ErrorData as McpError;
use wtcd_core::error::WtcdError;

/// Convert WtcdError to MCP error data
pub fn wtcd_error_to_mcp(e: WtcdError) -> McpError {
    McpError::internal_error(e.to_string(), None)
}

/// Convert anyhow::Error to MCP error data
pub fn anyhow_to_mcp(e: anyhow::Error) -> McpError {
    McpError::internal_error(e.to_string(), None)
}
