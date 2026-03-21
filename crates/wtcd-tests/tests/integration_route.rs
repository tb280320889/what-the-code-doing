use std::collections::HashMap;
use wtcd_core::index::*;
use wtcd_core::types::*;

fn sample_file_results() -> Vec<FileResult> {
    vec![
        FileResult {
            file_path: "src/auth/login.ts".to_string(),
            confidence: ConfidenceBand::High,
            exports: vec![
                ExportedSymbol {
                    name: "login".to_string(),
                    kind: ExportKind::Function,
                    line: 1,
                },
                ExportedSymbol {
                    name: "logout".to_string(),
                    kind: ExportKind::Function,
                    line: 10,
                },
            ],
            imports: vec![DependencyEdge {
                source: "src/core/http.ts".to_string(),
                symbols: vec!["fetch".to_string()],
                kind: ImportKind::Named,
            }],
            signatures: vec![],
            side_effects: vec![SideEffect {
                kind: SideEffectKind::Network,
                target: "auth api".to_string(),
                line: 5,
            }],
            parse_time_ms: 2,
            error_message: None,
        },
        FileResult {
            file_path: "src/utils/format.ts".to_string(),
            confidence: ConfidenceBand::High,
            exports: vec![ExportedSymbol {
                name: "formatDate".to_string(),
                kind: ExportKind::Function,
                line: 1,
            }],
            imports: vec![],
            signatures: vec![],
            side_effects: vec![],
            parse_time_ms: 1,
            error_message: None,
        },
        FileResult {
            file_path: "src/broken.ts".to_string(),
            confidence: ConfidenceBand::None,
            exports: vec![],
            imports: vec![],
            signatures: vec![],
            side_effects: vec![],
            parse_time_ms: 0,
            error_message: Some("parse failed".to_string()),
        },
    ]
}

fn sample_module_ids() -> HashMap<String, String> {
    let mut m = HashMap::new();
    m.insert("src/auth/login.ts".to_string(), "auth".to_string());
    m.insert("src/utils/format.ts".to_string(), "utils".to_string());
    m
}

// ── Index Building Tests ──

#[test]
fn build_index_filters_none_confidence() {
    let results = sample_file_results();
    let module_ids = sample_module_ids();
    let index = build_routing_index(&results, &module_ids);

    assert_eq!(index.api_version, "1");
    assert_eq!(index.entries.len(), 2); // broken.ts filtered out
}

#[test]
fn index_entry_has_correct_fields() {
    let results = sample_file_results();
    let module_ids = sample_module_ids();
    let index = build_routing_index(&results, &module_ids);

    let auth_entry = index
        .entries
        .iter()
        .find(|e| e.source_path == "src/auth/login.ts")
        .unwrap();
    assert_eq!(auth_entry.artifact_id, "file_mirror:src/auth/login.ts");
    assert_eq!(auth_entry.module_id, "auth");
    assert_eq!(auth_entry.exports, vec!["login", "logout"]);
    assert!(auth_entry.keywords.contains(&"auth".to_string()));
    assert!(auth_entry.keywords.contains(&"login".to_string()));
    assert!(auth_entry
        .dependencies
        .contains(&"src/core/http.ts".to_string()));
    assert!(auth_entry.risk_tags.contains(&"external_api".to_string()));
    assert_eq!(auth_entry.freshness, "fresh");
    assert_eq!(auth_entry.confidence, "high");
}

// ── Route Query Tests ──

#[test]
fn route_query_ranks_by_relevance() {
    let results = sample_file_results();
    let module_ids = sample_module_ids();
    let index = build_routing_index(&results, &module_ids);

    let output = route_query("modify auth login", &index, 10);
    assert!(!output.results.is_empty());
    assert_eq!(output.results[0].source_path, "src/auth/login.ts");
    assert!(output.results[0].relevance_score > 0.0);
}

#[test]
fn route_query_no_match_returns_empty() {
    let results = sample_file_results();
    let module_ids = sample_module_ids();
    let index = build_routing_index(&results, &module_ids);

    let output = route_query("nonexistent xyz", &index, 10);
    assert!(output.results.is_empty());
}

#[test]
fn route_query_top_k_limits() {
    let results = sample_file_results();
    let module_ids = sample_module_ids();
    let index = build_routing_index(&results, &module_ids);

    let output = route_query("src", &index, 1);
    assert!(output.results.len() <= 1);
}

// ── I/O Round-trip Tests ──

#[test]
fn write_read_round_trip() {
    let dir = std::env::temp_dir().join("wtcd_route_test");
    let _ = std::fs::create_dir_all(&dir);
    let index_path = dir.join("routing_index.json");

    let results = sample_file_results();
    let module_ids = sample_module_ids();
    let index = build_routing_index(&results, &module_ids);

    write_routing_index(&index, &index_path).unwrap();
    let loaded = read_routing_index(&index_path).unwrap();

    assert_eq!(loaded.api_version, index.api_version);
    assert_eq!(loaded.entries.len(), index.entries.len());

    let _ = std::fs::remove_file(&index_path);
    let _ = std::fs::remove_dir(&dir);
}

#[test]
fn read_missing_index_returns_error() {
    let result = read_routing_index(std::path::Path::new("/nonexistent/routing_index.json"));
    assert!(result.is_err());
}

// ── Freshness Tests ──

#[test]
fn freshness_affects_score() {
    let results = sample_file_results();
    let module_ids = sample_module_ids();
    let mut index = build_routing_index(&results, &module_ids);

    // Mark one entry as stale
    if let Some(entry) = index
        .entries
        .iter_mut()
        .find(|e| e.source_path == "src/auth/login.ts")
    {
        entry.freshness = "stale".to_string();
    }

    let output = route_query("login", &index, 10);
    let login_result = output
        .results
        .iter()
        .find(|r| r.source_path == "src/auth/login.ts");
    assert!(login_result.is_some());
    // Score should be reduced by freshness weight (0.7 instead of 1.0)
    assert!(login_result.unwrap().relevance_score < 1.0);
}

// ── JSON Serialization Tests ──

#[test]
fn route_output_serializes_to_json() {
    let results = sample_file_results();
    let module_ids = sample_module_ids();
    let index = build_routing_index(&results, &module_ids);

    let output = route_query("auth", &index, 10);
    let json = serde_json::to_string_pretty(&output).unwrap();

    assert!(json.contains("\"api_version\""));
    assert!(json.contains("\"query\""));
    assert!(json.contains("\"results\""));
    assert!(json.contains("\"relevance_score\""));
}
