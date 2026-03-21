use std::collections::HashSet;
use std::fs;
use tempfile::TempDir;
use wtcd_core::config::{FailRule, GateConfig};
use wtcd_core::depgraph::ReverseDepGraph;
use wtcd_core::types::*;
use wtcd_diff::analyzer::analyze_drift;
use wtcd_diff::gate::{evaluate_gate, get_exit_code};
use wtcd_diff::types::{ChangedFile, DiffResult, FileStatus};

// ─── Helpers ────────────────────────────────────────────────────────────────

fn make_file_result(path: &str, imports: Vec<(&str, Vec<&str>)>) -> FileResult {
    FileResult {
        file_path: path.to_string(),
        confidence: ConfidenceBand::High,
        exports: vec![],
        imports: imports
            .into_iter()
            .map(|(source, symbols)| DependencyEdge {
                source: source.to_string(),
                symbols: symbols.into_iter().map(|s| s.to_string()).collect(),
                kind: ImportKind::Named,
            })
            .collect(),
        signatures: vec![],
        side_effects: vec![],
        parse_time_ms: 0,
        error_message: None,
    }
}

// ─── Drift Classification Integration ──────────────────────────────────────

#[test]
fn test_analyze_no_changes() {
    let diff = DiffResult {
        changed_files: vec![],
        head_commit: "abc1234".to_string(),
        is_clean: true,
    };
    let report = analyze_drift(&diff, &[], "mirror/file", 5);
    assert_eq!(report.drift_level, "none");
    assert!(report.material_changes.is_empty());
}

#[test]
fn test_analyze_neighbor_expansion() {
    let files = vec![
        make_file_result("src/app.ts", vec![("./utils", vec!["helper"])]),
        make_file_result("src/utils.ts", vec![]),
    ];

    let diff = DiffResult {
        changed_files: vec![ChangedFile {
            path: "src/utils.ts".to_string(),
            status: FileStatus::Modified,
        }],
        head_commit: "abc1234".to_string(),
        is_clean: false,
    };

    // Create a temp file for utils.ts so the analyzer can read it
    let dir = TempDir::new().unwrap();
    fs::create_dir_all(dir.path().join("src")).unwrap();
    fs::write(dir.path().join("src/utils.ts"), "export const x = 1;").unwrap();
    std::env::set_current_dir(dir.path()).unwrap();

    let report = analyze_drift(&diff, &files, "mirror/file", 5);

    // Should have neighbor in affected_artifacts
    let neighbor_artifacts: Vec<_> = report
        .affected_artifacts
        .iter()
        .filter(|a| a.artifact_id.contains("app.ts"))
        .collect();
    assert!(
        !neighbor_artifacts.is_empty(),
        "Expected neighbor expansion for app.ts, got: {:?}",
        report.affected_artifacts
    );
}

// ─── Gate Policy Integration ────────────────────────────────────────────────

#[test]
fn test_gate_warn_only_default() {
    let config = GateConfig::default();
    let mut report = DriftReport {
        report_version: 1,
        report_id: "test".to_string(),
        repo_commit_before: "abc1234".to_string(),
        repo_commit_after: "working-tree".to_string(),
        generated_at: "2026-01-01T00:00:00Z".to_string(),
        drift_level: "material".to_string(),
        changed_files: vec!["src/test.ts".to_string()],
        affected_artifacts: vec![],
        material_changes: vec![MaterialChange {
            change_class: ChangeClass::C2,
            source_path: "src/test.ts".to_string(),
            summary: "Contract change".to_string(),
            evidence: None,
        }],
        policy_result: PolicyResult {
            status: "pending".to_string(),
            failed_rules: vec![],
            failure_codes: None,
            recommendations: None,
        },
    };

    let passed = evaluate_gate(&mut report, &config);
    assert!(passed); // default: warn-only
    assert_eq!(report.policy_result.status, "warn");
    assert_eq!(get_exit_code(passed), 0);
}

#[test]
fn test_gate_fail_on_c2() {
    let config = GateConfig {
        fail_on: vec![FailRule {
            when: "C2".to_string(),
            code: "ANRSM-001".to_string(),
        }],
        warn_on: vec![],
        systemic_threshold: 5,
    };
    let mut report = DriftReport {
        report_version: 1,
        report_id: "test".to_string(),
        repo_commit_before: "abc1234".to_string(),
        repo_commit_after: "working-tree".to_string(),
        generated_at: "2026-01-01T00:00:00Z".to_string(),
        drift_level: "material".to_string(),
        changed_files: vec!["src/test.ts".to_string()],
        affected_artifacts: vec![],
        material_changes: vec![MaterialChange {
            change_class: ChangeClass::C2,
            source_path: "src/test.ts".to_string(),
            summary: "Contract change".to_string(),
            evidence: None,
        }],
        policy_result: PolicyResult {
            status: "pending".to_string(),
            failed_rules: vec![],
            failure_codes: None,
            recommendations: None,
        },
    };

    let passed = evaluate_gate(&mut report, &config);
    assert!(!passed);
    assert_eq!(report.policy_result.status, "fail");
    assert_eq!(get_exit_code(passed), 1);
    assert!(report
        .policy_result
        .failure_codes
        .as_ref()
        .unwrap()
        .contains(&"ANRSM-001".to_string()));
}

// ─── ReverseDepGraph Integration ────────────────────────────────────────────

#[test]
fn test_depgraph_integration() {
    let files = vec![
        make_file_result("src/app.ts", vec![("./utils", vec!["helper"])]),
        make_file_result("src/handler.ts", vec![("./utils", vec!["helper"])]),
        make_file_result("src/utils.ts", vec![]),
    ];

    let graph = ReverseDepGraph::build(&files);

    assert_eq!(graph.importer_count("src/utils.ts"), 2);
    assert_eq!(graph.importer_count("src/app.ts"), 0);

    let mut changed = HashSet::new();
    changed.insert("src/utils.ts".to_string());
    let affected = graph.expand_affected(&changed);
    assert!(affected.contains("src/app.ts"));
    assert!(affected.contains("src/handler.ts"));
    assert!(!affected.contains("src/utils.ts"));
}

// ─── ChangeClass Enum ───────────────────────────────────────────────────────

#[test]
fn test_change_class_serialization() {
    assert_eq!(serde_json::to_string(&ChangeClass::C0).unwrap(), "\"C0\"");
    assert_eq!(serde_json::to_string(&ChangeClass::C1).unwrap(), "\"C1\"");
    assert_eq!(serde_json::to_string(&ChangeClass::C2).unwrap(), "\"C2\"");
    assert_eq!(serde_json::to_string(&ChangeClass::C3).unwrap(), "\"C3\"");
}

// ─── DriftReport Serialization ──────────────────────────────────────────────

#[test]
fn test_drift_report_json_roundtrip() {
    let report = DriftReport {
        report_version: 1,
        report_id: "test-uuid".to_string(),
        repo_commit_before: "abc1234".to_string(),
        repo_commit_after: "working-tree".to_string(),
        generated_at: "2026-01-01T00:00:00Z".to_string(),
        drift_level: "material".to_string(),
        changed_files: vec!["src/test.ts".to_string()],
        affected_artifacts: vec![AffectedArtifact {
            artifact_id: "file_mirror:src/test.ts".to_string(),
            before_freshness: "fresh".to_string(),
            after_freshness: "stale".to_string(),
            drift_level: "material".to_string(),
            reason: "Contract change".to_string(),
            recommended_action: Some("Run anrsm run".to_string()),
        }],
        material_changes: vec![MaterialChange {
            change_class: ChangeClass::C2,
            source_path: "src/test.ts".to_string(),
            summary: "Contract change".to_string(),
            evidence: Some(vec!["exports changed".to_string()]),
        }],
        policy_result: PolicyResult {
            status: "warn".to_string(),
            failed_rules: vec![],
            failure_codes: Some(vec!["ANRSM-001".to_string()]),
            recommendations: Some(vec!["warn: C2 changes detected".to_string()]),
        },
    };

    let json = serde_json::to_string_pretty(&report).unwrap();
    assert!(json.contains("\"report_version\": 1"));
    assert!(json.contains("\"drift_level\": \"material\""));
    assert!(json.contains("\"C2\""));

    let parsed: DriftReport = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.report_version, 1);
    assert_eq!(parsed.drift_level, "material");
}
