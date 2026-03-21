use rmcp::model::*;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::server::WtcdMcpServer;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RunRequest {
    /// 项目根目录路径（默认使用服务器启动时的工作目录）
    #[schemars(description = "项目根目录路径（默认使用服务器启动时的工作目录）")]
    #[serde(default)]
    pub project_root: Option<String>,
}

pub async fn handle_run(
    server: &WtcdMcpServer,
    params: RunRequest,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let server_root = server.project_root.clone();

    let result = tokio::task::spawn_blocking(move || -> Result<String, String> {
        let repo_root = match &params.project_root {
            Some(root) => std::path::PathBuf::from(root),
            None => server_root,
        };

        // 1. Load config
        let config = wtcd_scope::load_config(&repo_root)
            .map_err(|e| format!("Failed to load config: {}", e))?;

        // 2. Register adapters
        let registry = wtcd_adapters::register_all_adapters()
            .map_err(|e| format!("Failed to register adapters: {}", e))?;

        // 3. Scan files
        let files_to_parse = wtcd_scope::scan(&repo_root, &config.scope)
            .map_err(|e| format!("Failed to scan files: {}", e))?;

        // 4. Parse each file
        let mut file_results = Vec::new();
        for file_path in &files_to_parse {
            let relative = file_path
                .strip_prefix(&repo_root)
                .unwrap_or(file_path)
                .to_string_lossy()
                .to_string();

            let adapter = registry.find_adapter(&relative);
            let result = match adapter {
                Some(a) => match std::fs::read_to_string(file_path) {
                    Ok(source) => a.parse(&source, &relative),
                    Err(e) => wtcd_core::types::FileResult {
                        file_path: relative.clone(),
                        confidence: wtcd_core::types::ConfidenceBand::None,
                        exports: vec![],
                        imports: vec![],
                        signatures: vec![],
                        side_effects: vec![],
                        parse_time_ms: 0,
                        error_message: Some(format!("Cannot read file: {}", e)),
                    },
                },
                None => wtcd_core::types::FileResult {
                    file_path: relative.clone(),
                    confidence: wtcd_core::types::ConfidenceBand::None,
                    exports: vec![],
                    imports: vec![],
                    signatures: vec![],
                    side_effects: vec![],
                    parse_time_ms: 0,
                    error_message: Some("No language adapter found".to_string()),
                },
            };
            file_results.push(result);
        }

        // 5. Build summary
        let total_exports: usize = file_results.iter().map(|r| r.exports.len()).sum();
        let total_imports: usize = file_results.iter().map(|r| r.imports.len()).sum();
        let summary = wtcd_core::types::RunSummary {
            total_files: file_results.len(),
            parsed_ok: file_results
                .iter()
                .filter(|r| r.confidence == wtcd_core::types::ConfidenceBand::High)
                .count(),
            confidence_low: file_results
                .iter()
                .filter(|r| r.confidence == wtcd_core::types::ConfidenceBand::Low)
                .count(),
            confidence_none: file_results
                .iter()
                .filter(|r| r.confidence == wtcd_core::types::ConfidenceBand::None)
                .count(),
            total_exports,
            total_imports,
            elapsed_ms: 0, // timing not critical for MCP tool
        };

        // 6. Build output
        let output = wtcd_core::types::RunOutput {
            api_version: "1".to_string(),
            files: file_results,
            errors: vec![],
            summary,
        };

        serde_json::to_string_pretty(&output)
            .map_err(|e| format!("Serialization error: {}", e))
    })
    .await
    .map_err(|e| rmcp::ErrorData::internal_error(e.to_string(), None))?;

    match result {
        Ok(json) => Ok(CallToolResult::success(vec![Content::text(json)])),
        Err(msg) => Ok(CallToolResult::error(vec![Content::text(msg)])),
    }
}
