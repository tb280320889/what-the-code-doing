use rmcp::model::*;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::server::WtcdMcpServer;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ParseRequest {
    /// 要解析的文件路径（相对项目根目录）
    #[schemars(description = "要解析的文件路径（相对项目根目录）")]
    pub file_path: String,
    /// 可选：指定语言适配器（auto 为自动检测）
    #[schemars(description = "语言适配器（auto/ts/js），默认 auto")]
    #[serde(default)]
    pub language: Option<String>,
}

pub async fn handle_parse(
    server: &WtcdMcpServer,
    params: ParseRequest,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let project_root = server.project_root.clone();

    let result = tokio::task::spawn_blocking(move || -> Result<String, String> {
        // 1. Register adapters
        let registry = wtcd_adapters::register_all_adapters()
            .map_err(|e| format!("Failed to register adapters: {}", e))?;

        // 2. Resolve file path
        let file_path = std::path::Path::new(&params.file_path);
        let absolute_path = if file_path.is_absolute() {
            file_path.to_path_buf()
        } else {
            project_root.join(file_path)
        };

        // 3. Read source
        let source = std::fs::read_to_string(&absolute_path)
            .map_err(|e| format!("Cannot read file {}: {}", params.file_path, e))?;

        // 4. Find adapter and parse
        let relative = params.file_path.clone();
        let adapter = registry.find_adapter(&relative);
        let file_result = match adapter {
            Some(a) => a.parse(&source, &relative),
            None => wtcd_core::types::FileResult {
                file_path: relative.clone(),
                confidence: wtcd_core::types::ConfidenceBand::None,
                exports: vec![],
                imports: vec![],
                signatures: vec![],
                side_effects: vec![],
                parse_time_ms: 0,
                error_message: Some(format!(
                    "No language adapter found for file: {}",
                    relative
                )),
            },
        };

        serde_json::to_string_pretty(&file_result)
            .map_err(|e| format!("Serialization error: {}", e))
    })
    .await
    .map_err(|e| rmcp::ErrorData::internal_error(e.to_string(), None))?;

    match result {
        Ok(json) => Ok(CallToolResult::success(vec![Content::text(json)])),
        Err(msg) => Ok(CallToolResult::error(vec![Content::text(msg)])),
    }
}
