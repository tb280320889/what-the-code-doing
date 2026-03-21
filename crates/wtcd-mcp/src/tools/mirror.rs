use rmcp::model::*;

pub async fn handle_mirror() -> Result<CallToolResult, rmcp::ErrorData> {
    Ok(CallToolResult::error(vec![Content::text(
        "wtcd_mirror: Phase 2 能力 — 镜像文件生成尚未实现",
    )]))
}
