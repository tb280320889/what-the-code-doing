use std::path::Path;

/// Run drift check: diff → parse changed → classify → gate → report (CORE-04, DRFT-01, DRFT-03)
pub fn run_check(repo_root: &Path) -> wtcd_core::error::Result<()> {
    // 1. Load config
    let config = wtcd_scope::load_config(repo_root)?;

    // 2. Get gate config (default: warn-only)
    let gate_config = config.gate.clone().unwrap_or_default();

    // 3. Get mirror config
    let mirror_config = config.mirror.clone().unwrap_or_default();
    let mirror_root = repo_root.join(&mirror_config.output_dir);

    // 4. Diff working tree vs HEAD (DRFT-01)
    let diff_result = wtcd_diff::git_diff::diff_working_tree_vs_head(repo_root)
        .map_err(|e| wtcd_core::error::WtcdError::ConfigError(format!("Git diff failed: {}", e)))?;

    // 5. If no changes, output empty report and exit 0
    if diff_result.is_clean {
        let empty_report = wtcd_core::types::DriftReport {
            report_version: 1,
            report_id: uuid::Uuid::new_v4().to_string(),
            repo_commit_before: diff_result.head_commit.clone(),
            repo_commit_after: "working-tree".to_string(),
            generated_at: chrono::Utc::now().to_rfc3339(),
            drift_level: "none".to_string(),
            changed_files: vec![],
            affected_artifacts: vec![],
            material_changes: vec![],
            policy_result: wtcd_core::types::PolicyResult {
                status: "pass".to_string(),
                failed_rules: vec![],
                failure_codes: None,
                recommendations: Some(vec!["No changes detected".to_string()]),
            },
        };
        format_json_drift(&empty_report);
        std::process::exit(0);
    }

    // 6. Parse only changed files (incremental)
    let registry = wtcd_adapters::register_all_adapters()
        .map_err(|e| wtcd_core::error::WtcdError::ConfigError(e.to_string()))?;

    let mut changed_file_results = Vec::new();
    for changed in &diff_result.changed_files {
        let full_path = repo_root.join(&changed.path);
        let adapter = registry.find_adapter(&changed.path);

        let result = match adapter {
            Some(adapter) => match std::fs::read_to_string(&full_path) {
                Ok(source) => adapter.parse(&source, &changed.path),
                Err(_) => wtcd_core::types::FileResult {
                    file_path: changed.path.clone(),
                    confidence: wtcd_core::types::ConfidenceBand::None,
                    exports: vec![],
                    imports: vec![],
                    signatures: vec![],
                    side_effects: vec![],
                    parse_time_ms: 0,
                    error_message: Some("Cannot read file".to_string()),
                },
            },
            None => wtcd_core::types::FileResult {
                file_path: changed.path.clone(),
                confidence: wtcd_core::types::ConfidenceBand::None,
                exports: vec![],
                imports: vec![],
                signatures: vec![],
                side_effects: vec![],
                parse_time_ms: 0,
                error_message: Some("No language adapter found".to_string()),
            },
        };
        changed_file_results.push(result);
    }

    // Also need ALL file results for reverse dep graph
    let all_files = wtcd_scope::scan(repo_root, &config.scope)?;
    let mut all_file_results = Vec::new();
    for file_path in &all_files {
        let relative = file_path
            .strip_prefix(repo_root)
            .unwrap_or(file_path)
            .to_string_lossy()
            .to_string();

        let adapter = registry.find_adapter(&relative);
        let result = match adapter {
            Some(adapter) => match std::fs::read_to_string(file_path) {
                Ok(source) => adapter.parse(&source, &relative),
                Err(_) => wtcd_core::types::FileResult {
                    file_path: relative.clone(),
                    confidence: wtcd_core::types::ConfidenceBand::None,
                    exports: vec![],
                    imports: vec![],
                    signatures: vec![],
                    side_effects: vec![],
                    parse_time_ms: 0,
                    error_message: Some("Cannot read".to_string()),
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
                error_message: None,
            },
        };
        all_file_results.push(result);
    }

    // 7. Run drift analysis (DRFT-02, DRFT-03, DRFT-04)
    let mirror_root_str = mirror_root.to_string_lossy().to_string();
    let mut report = wtcd_diff::analyzer::analyze_drift(
        &diff_result,
        &all_file_results,
        &mirror_root_str,
        gate_config.systemic_threshold,
    );

    // 8. Evaluate gate policy (GATE-03)
    let passed = wtcd_diff::gate::evaluate_gate(&mut report, &gate_config);

    // 9. Output report to stdout (DRFT-03)
    format_json_drift(&report);

    // 10. Exit with appropriate code (GATE-04, D-12)
    std::process::exit(wtcd_diff::gate::get_exit_code(passed));
}

/// Format and print drift report as JSON to stdout
fn format_json_drift(report: &wtcd_core::types::DriftReport) {
    match serde_json::to_string_pretty(report) {
        Ok(json) => println!("{}", json),
        Err(e) => {
            eprintln!("{{\"error\": \"Failed to serialize drift report: {}\"}}", e);
        }
    }
}
