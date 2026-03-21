use rmcp::model::*;

pub async fn handle_check() -> Result<CallToolResult, rmcp::ErrorData> {
    Ok(CallToolResult::error(vec![Content::text(
        "wtcd_check: Phase 3 能力 — 镜像漂移检测尚未实现",
    )]))
}
