use crate::types::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;

// ─── Index Types (D-01, D-02) ───────────────────────────────────────────────

/// A single entry in the routing index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingIndexEntry {
    pub artifact_id: String,
    pub module_id: String,
    pub source_path: String,
    pub exports: Vec<String>,
    pub keywords: Vec<String>,
    pub dependencies: Vec<String>,
    pub risk_tags: Vec<String>,
    pub freshness: String,
    pub confidence: String,
}

/// Top-level routing index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingIndex {
    pub api_version: String,
    pub entries: Vec<RoutingIndexEntry>,
}

// ─── Route Output Types (D-16) ──────────────────────────────────────────────

/// A single query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteResult {
    pub source_path: String,
    pub artifact_id: String,
    pub relevance_score: f64,
    pub freshness: String,
    pub exports: Vec<String>,
    pub confidence: String,
}

/// Top-level CLI output for route command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteOutput {
    pub api_version: String,
    pub query: String,
    pub total_candidates: usize,
    pub results: Vec<RouteResult>,
}

// ─── Keyword Extraction (D-09) ──────────────────────────────────────────────

/// Split a camelCase identifier into lowercase tokens.
///
/// Example: "parseAuthTokens" → ["parse", "auth", "tokens"]
fn split_camel_case(s: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();

    for ch in s.chars() {
        if ch.is_uppercase() {
            if !current.is_empty() {
                tokens.push(current.to_lowercase());
                current.clear();
            }
            current.push(ch);
        } else if ch == '_' || ch == '-' {
            if !current.is_empty() {
                tokens.push(current.to_lowercase());
                current.clear();
            }
        } else {
            current.push(ch);
        }
    }
    if !current.is_empty() {
        tokens.push(current.to_lowercase());
    }

    tokens
}

/// Extract keywords from source_path, exports, and module_id (D-09).
///
/// - source_path: split by `/`, then each segment by `_` (snake_case) and camelCase
/// - exports: split each by `_` and camelCase
/// - module_id: split by `_` and camelCase
///
/// All keywords are lowercased and deduplicated.
pub fn extract_keywords(source_path: &str, exports: &[String], module_id: &str) -> Vec<String> {
    let mut keywords = HashSet::new();

    // Split source_path by `/`
    for segment in source_path.split('/') {
        // Split by `.` (remove extensions)
        for part in segment.split('.') {
            // Split by `_` (snake_case)
            for sub in part.split('_') {
                for token in split_camel_case(sub) {
                    if !token.is_empty() {
                        keywords.insert(token);
                    }
                }
            }
        }
    }

    // Export names as keywords
    for export in exports {
        for sub in export.split('_') {
            for token in split_camel_case(sub) {
                if !token.is_empty() {
                    keywords.insert(token);
                }
            }
        }
    }

    // Module ID as keywords
    for sub in module_id.split('_') {
        for token in split_camel_case(sub) {
            if !token.is_empty() {
                keywords.insert(token);
            }
        }
    }

    let mut result: Vec<String> = keywords.into_iter().collect();
    result.sort();
    result
}

// ─── Risk Tags (mirrors wtcd-mirror/src/template.rs pattern) ───────────────

/// Derive risk tags from side effects.
pub fn derive_risk_tags(side_effects: &[SideEffect]) -> Vec<String> {
    let mut tags: Vec<String> = side_effects
        .iter()
        .map(|se| match se.kind {
            SideEffectKind::Network => "external_api".to_string(),
            SideEffectKind::Io => "file_io".to_string(),
            SideEffectKind::Storage => "persistence".to_string(),
            SideEffectKind::Log => "logging".to_string(),
        })
        .collect();
    tags.sort();
    tags.dedup();
    tags
}

// ─── Index Builder (D-12) ───────────────────────────────────────────────────

/// Build a routing index from file results (D-12).
///
/// Filters out files with ConfidenceBand::None (D-04: only mirrored files).
pub fn build_routing_index(
    file_results: &[FileResult],
    module_ids: &HashMap<String, String>,
) -> RoutingIndex {
    let entries: Vec<RoutingIndexEntry> = file_results
        .iter()
        .filter(|fr| fr.confidence != ConfidenceBand::None)
        .map(|fr| {
            let export_names: Vec<String> = fr.exports.iter().map(|e| e.name.clone()).collect();
            let dep_sources: Vec<String> = fr.imports.iter().map(|i| i.source.clone()).collect();
            let risk_tags = derive_risk_tags(&fr.side_effects);
            let module_id = module_ids
                .get(&fr.file_path)
                .cloned()
                .unwrap_or_else(|| "global".to_string());
            let keywords = extract_keywords(&fr.file_path, &export_names, &module_id);

            RoutingIndexEntry {
                artifact_id: format!("file_mirror:{}", fr.file_path),
                module_id,
                source_path: fr.file_path.clone(),
                exports: export_names,
                keywords,
                dependencies: dep_sources,
                risk_tags,
                freshness: "fresh".to_string(),
                confidence: match fr.confidence {
                    ConfidenceBand::High => "high".to_string(),
                    ConfidenceBand::Low => "low".to_string(),
                    ConfidenceBand::None => "none".to_string(),
                },
            }
        })
        .collect();

    RoutingIndex {
        api_version: "1".to_string(),
        entries,
    }
}

// ─── Index I/O (D-01) ───────────────────────────────────────────────────────

/// Write routing index to a JSON file (D-01).
pub fn write_routing_index(index: &RoutingIndex, path: &Path) -> crate::error::Result<()> {
    let json = serde_json::to_string_pretty(index).map_err(|e| {
        crate::error::WtcdError::ConfigError(format!("Failed to serialize index: {}", e))
    })?;
    std::fs::write(path, json)?;
    Ok(())
}

/// Read routing index from a JSON file.
pub fn read_routing_index(path: &Path) -> crate::error::Result<RoutingIndex> {
    let content = std::fs::read_to_string(path)?;
    let index: RoutingIndex = serde_json::from_str(&content).map_err(|e| {
        crate::error::WtcdError::ConfigError(format!("Failed to parse index: {}", e))
    })?;
    Ok(index)
}

// ─── Query Engine (D-05, D-06, D-07, D-08) ─────────────────────────────────

/// Freshness weight mapping (D-06).
fn freshness_weight(freshness: &str) -> f64 {
    match freshness {
        "fresh" => 1.0,
        "stale" => 0.7,
        "invalid" => 0.3,
        _ => 0.5, // unknown
    }
}

/// Confidence weight for scoring (Phase 11).
fn confidence_weight(confidence: &str) -> f64 {
    match confidence {
        "high" => 1.0,
        "low" => 0.5,
        _ => 0.0, // "none" — should be filtered at index build time
    }
}

/// Confidence sort value for tie-breaking (D-08).
fn confidence_sort_value(confidence: &str) -> i32 {
    match confidence {
        "high" => 2,
        "low" => 1,
        _ => 0,
    }
}

/// Tokenize a query string into lowercase tokens.
fn tokenize_query(query: &str) -> Vec<String> {
    query
        .to_lowercase()
        .split(|c: char| c.is_whitespace() || c == '.' || c == '_' || c == '-' || c == '/')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

/// Execute a route query against the routing index (D-05, D-07, D-08).
pub fn route_query(query: &str, index: &RoutingIndex, top_k: usize) -> RouteOutput {
    let query_tokens = tokenize_query(query);

    if query_tokens.is_empty() {
        return RouteOutput {
            api_version: "1".to_string(),
            query: query.to_string(),
            total_candidates: 0,
            results: vec![],
        };
    }

    let mut results: Vec<RouteResult> = Vec::new();

    for entry in &index.entries {
        // Count matching keywords (case-insensitive)
        let mut matches = 0usize;
        let keyword_set: HashSet<&str> = entry.keywords.iter().map(|k| k.as_str()).collect();

        for token in &query_tokens {
            if keyword_set.contains(token.as_str()) {
                matches += 1;
            }
        }

        if matches == 0 {
            continue;
        }

        let score = matches as f64 / query_tokens.len() as f64
            * freshness_weight(&entry.freshness)
            * confidence_weight(&entry.confidence);

        results.push(RouteResult {
            source_path: entry.source_path.clone(),
            artifact_id: entry.artifact_id.clone(),
            relevance_score: score,
            freshness: entry.freshness.clone(),
            exports: entry.exports.clone(),
            confidence: entry.confidence.clone(),
        });
    }

    // Sort: relevance desc, confidence desc, source_path asc (D-08)
    results.sort_by(|a, b| {
        b.relevance_score
            .partial_cmp(&a.relevance_score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                confidence_sort_value(&b.confidence).cmp(&confidence_sort_value(&a.confidence))
            })
            .then_with(|| a.source_path.cmp(&b.source_path))
    });

    let total_candidates = results.len();
    results.truncate(top_k);

    RouteOutput {
        api_version: "1".to_string(),
        query: query.to_string(),
        total_candidates,
        results,
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Keyword Extraction Tests ──

    #[test]
    fn extract_keywords_snake_case_path() {
        let keywords = extract_keywords("src/user_profile.ts", &[], "global");
        assert!(keywords.contains(&"src".to_string()));
        assert!(keywords.contains(&"user".to_string()));
        assert!(keywords.contains(&"profile".to_string()));
        assert!(keywords.contains(&"ts".to_string()));
    }

    #[test]
    fn extract_keywords_camel_case_exports() {
        let exports = vec!["parseAuthTokens".to_string()];
        let keywords = extract_keywords("src/auth.ts", &exports, "auth");
        assert!(keywords.contains(&"parse".to_string()));
        assert!(keywords.contains(&"auth".to_string()));
        assert!(keywords.contains(&"tokens".to_string()));
    }

    #[test]
    fn extract_keywords_nested_path() {
        let keywords = extract_keywords("src/auth/login.ts", &[], "global");
        assert!(keywords.contains(&"src".to_string()));
        assert!(keywords.contains(&"auth".to_string()));
        assert!(keywords.contains(&"login".to_string()));
    }

    #[test]
    fn extract_keywords_deduplicates() {
        // "login" appears in both path and export — should be deduplicated
        let exports = vec!["login".to_string()];
        let keywords = extract_keywords("src/auth/login.ts", &exports, "auth");
        let login_count = keywords.iter().filter(|k| *k == "login").count();
        assert_eq!(login_count, 1);
    }

    // ── Risk Tags Tests ──

    #[test]
    fn derive_risk_tags_maps_correctly() {
        let side_effects = vec![
            SideEffect {
                kind: SideEffectKind::Network,
                target: "api".to_string(),
                line: 1,
            },
            SideEffect {
                kind: SideEffectKind::Io,
                target: "fs".to_string(),
                line: 2,
            },
        ];
        let tags = derive_risk_tags(&side_effects);
        assert!(tags.contains(&"external_api".to_string()));
        assert!(tags.contains(&"file_io".to_string()));
        assert_eq!(tags.len(), 2);
    }

    #[test]
    fn derive_risk_tags_deduplicates() {
        let side_effects = vec![
            SideEffect {
                kind: SideEffectKind::Network,
                target: "api1".to_string(),
                line: 1,
            },
            SideEffect {
                kind: SideEffectKind::Network,
                target: "api2".to_string(),
                line: 2,
            },
        ];
        let tags = derive_risk_tags(&side_effects);
        assert_eq!(tags.len(), 1);
        assert!(tags.contains(&"external_api".to_string()));
    }

    // ── Build Routing Index Tests ──

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
                        is_generated: false,
                        confidence: ConfidenceBand::High,
                    },
                    ExportedSymbol {
                        name: "logout".to_string(),
                        kind: ExportKind::Function,
                        line: 10,
                        is_generated: false,
                        confidence: ConfidenceBand::High,
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
                    is_generated: false,
                    confidence: ConfidenceBand::High,
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

    #[test]
    fn build_routing_index_filters_none_confidence() {
        let results = sample_file_results();
        let module_ids = sample_module_ids();
        let index = build_routing_index(&results, &module_ids);
        assert_eq!(index.api_version, "1");
        assert_eq!(index.entries.len(), 2); // broken.ts filtered out
    }

    #[test]
    fn build_routing_index_entry_fields() {
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

    #[test]
    fn build_routing_index_default_module_id() {
        let results = vec![FileResult {
            file_path: "root.ts".to_string(),
            confidence: ConfidenceBand::High,
            exports: vec![],
            imports: vec![],
            signatures: vec![],
            side_effects: vec![],
            parse_time_ms: 0,
            error_message: None,
        }];
        let module_ids = HashMap::new(); // no entry for "root.ts"
        let index = build_routing_index(&results, &module_ids);
        assert_eq!(index.entries[0].module_id, "global");
    }

    // ── Write/Read Round-trip ──

    #[test]
    fn write_read_round_trip() {
        let dir = std::env::temp_dir().join("wtcd_index_test");
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

    // ── Route Query Tests (from Plan 04-02) ──

    #[test]
    fn route_query_exact_match() {
        let results = sample_file_results();
        let module_ids = sample_module_ids();
        let index = build_routing_index(&results, &module_ids);

        let output = route_query("login", &index, 10);
        assert!(!output.results.is_empty());
        assert_eq!(output.results[0].source_path, "src/auth/login.ts");
    }

    #[test]
    fn route_query_partial_match() {
        let results = sample_file_results();
        let module_ids = sample_module_ids();
        let index = build_routing_index(&results, &module_ids);

        let output = route_query("auth login", &index, 10);
        assert!(!output.results.is_empty());
        assert_eq!(output.results[0].source_path, "src/auth/login.ts");
        assert_eq!(output.results[0].relevance_score, 1.0); // 2/2 * 1.0
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
    fn route_query_freshness_affects_score() {
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
        assert!((login_result.unwrap().relevance_score - 0.7).abs() < f64::EPSILON);
    }

    #[test]
    fn route_query_tie_break_confidence() {
        let results = vec![
            FileResult {
                file_path: "a.ts".to_string(),
                confidence: ConfidenceBand::High,
                exports: vec![ExportedSymbol {
                    name: "foo".to_string(),
                    kind: ExportKind::Function,
                    line: 1,
                    is_generated: false,
                    confidence: ConfidenceBand::High,
                }],
                imports: vec![],
                signatures: vec![],
                side_effects: vec![],
                parse_time_ms: 0,
                error_message: None,
            },
            FileResult {
                file_path: "b.ts".to_string(),
                confidence: ConfidenceBand::Low,
                exports: vec![ExportedSymbol {
                    name: "foo".to_string(),
                    kind: ExportKind::Function,
                    line: 1,
                    is_generated: false,
                    confidence: ConfidenceBand::High,
                }],
                imports: vec![],
                signatures: vec![],
                side_effects: vec![],
                parse_time_ms: 0,
                error_message: None,
            },
        ];
        let module_ids = HashMap::new();
        let index = build_routing_index(&results, &module_ids);

        let output = route_query("foo", &index, 10);
        assert_eq!(output.results.len(), 2);
        // High confidence should come first
        assert_eq!(output.results[0].confidence, "high");
    }

    #[test]
    fn route_query_tie_break_alphabetical() {
        let results = vec![
            FileResult {
                file_path: "z.ts".to_string(),
                confidence: ConfidenceBand::High,
                exports: vec![ExportedSymbol {
                    name: "foo".to_string(),
                    kind: ExportKind::Function,
                    line: 1,
                    is_generated: false,
                    confidence: ConfidenceBand::High,
                }],
                imports: vec![],
                signatures: vec![],
                side_effects: vec![],
                parse_time_ms: 0,
                error_message: None,
            },
            FileResult {
                file_path: "a.ts".to_string(),
                confidence: ConfidenceBand::High,
                exports: vec![ExportedSymbol {
                    name: "foo".to_string(),
                    kind: ExportKind::Function,
                    line: 1,
                    is_generated: false,
                    confidence: ConfidenceBand::High,
                }],
                imports: vec![],
                signatures: vec![],
                side_effects: vec![],
                parse_time_ms: 0,
                error_message: None,
            },
        ];
        let module_ids = HashMap::new();
        let index = build_routing_index(&results, &module_ids);

        let output = route_query("foo", &index, 10);
        assert_eq!(output.results.len(), 2);
        // Same score and confidence — alphabetical order
        assert_eq!(output.results[0].source_path, "a.ts");
        assert_eq!(output.results[1].source_path, "z.ts");
    }

    #[test]
    fn route_query_top_k_limits() {
        let results = sample_file_results();
        let module_ids = sample_module_ids();
        let index = build_routing_index(&results, &module_ids);

        let output = route_query("src", &index, 1);
        assert!(output.results.len() <= 1);
    }

    #[test]
    fn route_query_top_k_zero_returns_empty() {
        let results = sample_file_results();
        let module_ids = sample_module_ids();
        let index = build_routing_index(&results, &module_ids);

        let output = route_query("auth", &index, 0);
        assert!(output.results.is_empty());
    }

    #[test]
    fn route_query_case_insensitive() {
        let results = sample_file_results();
        let module_ids = sample_module_ids();
        let index = build_routing_index(&results, &module_ids);

        let output = route_query("Auth", &index, 10);
        assert!(!output.results.is_empty());
        assert!(output
            .results
            .iter()
            .any(|r| r.source_path == "src/auth/login.ts"));
    }

    #[test]
    fn route_query_compound_keyword_match() {
        let results = sample_file_results();
        let module_ids = sample_module_ids();
        let index = build_routing_index(&results, &module_ids);

        // "formatDate" was extracted as ["format", "date"] — "format" should match
        let output = route_query("format", &index, 10);
        assert!(output
            .results
            .iter()
            .any(|r| r.source_path == "src/utils/format.ts"));
    }

    // ── Tokenize Query Tests ──

    #[test]
    fn tokenize_query_basic() {
        assert_eq!(
            tokenize_query("modify auth logic"),
            vec!["modify", "auth", "logic"]
        );
    }

    #[test]
    fn tokenize_query_with_punctuation() {
        assert_eq!(
            tokenize_query("update user.profile.ts"),
            vec!["update", "user", "profile", "ts"]
        );
    }
}
