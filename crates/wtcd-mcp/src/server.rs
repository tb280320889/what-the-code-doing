use std::path::PathBuf;

use rmcp::{
    handler::server::router::tool::ToolRouter, model::*, tool, tool_handler, tool_router,
    ServerHandler,
};

#[derive(Clone)]
pub struct WtcdMcpServer {
    pub tool_router: ToolRouter<Self>,
    pub project_root: PathBuf,
}

// Tool methods are defined via include! in the #[tool_router] block below.
// Each tool module exports a macro that expands to the tool method definition.

#[tool_router]
impl WtcdMcpServer {
    pub fn new(project_root: PathBuf) -> Self {
        Self {
            tool_router: Self::tool_router(),
            project_root,
        }
    }

    // ── wtcd_parse ─────────────────────────────────────────────────────────
    #[tool(description = "解析单个 TS/JS 文件，提取导出符号、依赖边、函数签名和副作用")]
    async fn wtcd_parse(
        &self,
        rmcp::handler::server::wrapper::Parameters(params): rmcp::handler::server::wrapper::Parameters<crate::tools::parse::ParseRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        crate::tools::parse::handle_parse(self, params).await
    }

    // ── wtcd_run ───────────────────────────────────────────────────────────
    #[tool(description = "扫描整个项目范围内的文件，执行全量语义分析")]
    async fn wtcd_run(
        &self,
        rmcp::handler::server::wrapper::Parameters(params): rmcp::handler::server::wrapper::Parameters<crate::tools::run::RunRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        crate::tools::run::handle_run(self, params).await
    }

    // ── wtcd_check (stub) ──────────────────────────────────────────────────
    #[tool(description = "检测镜像漂移 (Phase 3 能力，当前为 stub)")]
    async fn wtcd_check(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        crate::tools::check::handle_check().await
    }

    // ── wtcd_mirror (stub) ─────────────────────────────────────────────────
    #[tool(description = "生成镜像文件 (Phase 2 能力，当前为 stub)")]
    async fn wtcd_mirror(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        crate::tools::mirror::handle_mirror().await
    }
}

#[tool_handler]
impl ServerHandler for WtcdMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_protocol_version(ProtocolVersion::V_2025_03_26)
            .with_server_info(Implementation::new("wtcd-mcp", env!("CARGO_PKG_VERSION")))
            .with_instructions("WTCD MCP Server — 提供代码语义解析、路由、检查和镜像生成工具")
    }
}
