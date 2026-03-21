use crate::output::format_json;
use std::path::Path;
use std::time::Instant;
use wtcd_adapters::register_all_adapters;
use wtcd_core::types::*;

pub fn run_analysis(repo_root: &Path) -> wtcd_core::error::Result<()> {
    let overall_start = Instant::now();

    // 1. Load config (D-10: must have anrsm.yaml)
    let config = wtcd_scope::load_config(repo_root)?;

    // 2. Register adapters
    let registry = register_all_adapters()
        .map_err(|e| wtcd_core::error::WtcdError::ConfigError(e.to_string()))?;

    // 3. Scan files
    let files = wtcd_scope::scan(repo_root, &config.scope)?;

    // 4. Parse each file
    let mut file_results = Vec::new();
    for file_path in &files {
        let relative = file_path
            .strip_prefix(repo_root)
            .unwrap_or(file_path)
            .to_string_lossy()
            .to_string();

        let adapter = registry.find_adapter(&relative);
        let result = match adapter {
            Some(adapter) => match std::fs::read_to_string(file_path) {
                Ok(source) => adapter.parse(&source, &relative),
                Err(e) => FileResult {
                    file_path: relative.clone(),
                    confidence: ConfidenceBand::None,
                    exports: vec![],
                    imports: vec![],
                    signatures: vec![],
                    side_effects: vec![],
                    parse_time_ms: 0,
                    error_message: Some(format!("Cannot read file: {}", e)),
                },
            },
            None => FileResult {
                file_path: relative.clone(),
                confidence: ConfidenceBand::None,
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

    // 5. Build summary (D-07)
    let total_exports: usize = file_results.iter().map(|r| r.exports.len()).sum();
    let total_imports: usize = file_results.iter().map(|r| r.imports.len()).sum();
    let summary = RunSummary {
        total_files: file_results.len(),
        parsed_ok: file_results
            .iter()
            .filter(|r| r.confidence == ConfidenceBand::High)
            .count(),
        confidence_low: file_results
            .iter()
            .filter(|r| r.confidence == ConfidenceBand::Low)
            .count(),
        confidence_none: file_results
            .iter()
            .filter(|r| r.confidence == ConfidenceBand::None)
            .count(),
        total_exports,
        total_imports,
        elapsed_ms: overall_start.elapsed().as_millis() as u64,
    };

    // 6. Output JSON (D-01, D-04, CORE-06)
    let output = RunOutput {
        api_version: "1".to_string(),
        files: file_results,
        errors: vec![],
        summary,
    };

    format_json(&output);
    Ok(())
}
