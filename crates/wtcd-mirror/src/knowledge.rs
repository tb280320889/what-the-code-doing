use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use crate::module::build_module_graph;
use wtcd_core::index::RoutingIndex;
use wtcd_core::types::{ChangeClass, KnowledgeResult, ModuleResult};

pub fn build_knowledge_result(modules: &[ModuleResult], source_tokens: usize) -> KnowledgeResult {
    let mut language_distribution: BTreeMap<String, usize> = BTreeMap::new();
    let mut total_files = 0usize;
    let mut total_exports = 0usize;
    let mut mirror_tokens = 0usize;

    for m in modules {
        *language_distribution.entry(m.language.clone()).or_insert(0) += m.files.len();
        total_files += m.files.len();
        total_exports += m.exports.len();
        mirror_tokens += m.responsibility.split_whitespace().count()
            + m.exports.len()
            + m.dependencies.len()
            + m.side_effects.len();
    }

    let ratio = if source_tokens == 0 {
        0.0
    } else {
        mirror_tokens as f64 / source_tokens as f64
    };

    KnowledgeResult {
        module_count: modules.len(),
        language_distribution,
        total_files,
        total_exports,
        token_compression_ratio: ratio,
    }
}

pub fn generate_repo_overview(modules: &[ModuleResult], knowledge: &KnowledgeResult) -> String {
    let mut entry_modules = Vec::new();
    for m in modules {
        if m.fan_in == 0 || m.module_id.contains("main") || m.module_id.contains("app") {
            entry_modules.push(m.module_id.clone());
        }
    }
    entry_modules.sort();
    entry_modules.dedup();

    format!(
        "# Repository Overview\n\n- modules: {}\n- files: {}\n- exports: {}\n- languages: {}\n- entry_modules: {}\n- token_compression_ratio: {:.4}",
        knowledge.module_count,
        knowledge.total_files,
        knowledge.total_exports,
        knowledge
            .language_distribution
            .iter()
            .map(|(k, v)| format!("{k}:{v}"))
            .collect::<Vec<_>>()
            .join(", "),
        if entry_modules.is_empty() {
            "none".to_string()
        } else {
            entry_modules.join(", ")
        },
        knowledge.token_compression_ratio
    )
}

pub fn generate_module_dep_mermaid(modules: &[ModuleResult]) -> String {
    let graph = build_module_graph(modules);
    let mut lines = vec!["graph LR".to_string()];
    let mut nodes = BTreeSet::new();

    for m in modules {
        nodes.insert(m.module_id.clone());
    }
    for n in nodes {
        lines.push(format!("  {}[\"{}\"]", sanitize_node(&n), n));
    }

    for (src, targets) in graph {
        for dst in targets {
            lines.push(format!(
                "  {} --> {}",
                sanitize_node(&src),
                sanitize_node(&dst)
            ));
        }
    }

    lines.join("\n")
}

pub fn generate_export_index(modules: &[ModuleResult]) -> String {
    let mut rows = Vec::new();
    for m in modules {
        for e in &m.exports {
            rows.push(format!("- {}::{}", m.module_id, e));
        }
    }
    rows.sort();
    format!("# Global Export Index\n\n{}", rows.join("\n"))
}

pub fn community_clusters(modules: &[ModuleResult]) -> Vec<Vec<String>> {
    let graph = build_module_graph(modules);
    let mut undirected: HashMap<String, HashSet<String>> = HashMap::new();
    for m in modules {
        undirected.entry(m.module_id.clone()).or_default();
    }
    for (src, targets) in &graph {
        for dst in targets {
            undirected
                .entry(src.clone())
                .or_default()
                .insert(dst.clone());
            undirected
                .entry(dst.clone())
                .or_default()
                .insert(src.clone());
        }
    }

    let mut seen = HashSet::new();
    let mut clusters = Vec::new();
    for node in undirected.keys() {
        if seen.contains(node) {
            continue;
        }
        let mut stack = vec![node.clone()];
        let mut cluster = Vec::new();
        while let Some(n) = stack.pop() {
            if !seen.insert(n.clone()) {
                continue;
            }
            cluster.push(n.clone());
            if let Some(nei) = undirected.get(&n) {
                for x in nei {
                    if !seen.contains(x) {
                        stack.push(x.clone());
                    }
                }
            }
        }
        cluster.sort();
        clusters.push(cluster);
    }
    clusters.sort_by_key(|c| c.len());
    clusters.reverse();
    clusters
}

pub fn hotspot_map_from_drift(drift_history: &[Vec<ChangeClass>]) -> BTreeMap<String, usize> {
    let mut map = BTreeMap::new();
    for (idx, classes) in drift_history.iter().enumerate() {
        let score = classes
            .iter()
            .map(|c| match c {
                ChangeClass::C0 => 0usize,
                ChangeClass::C1 => 1usize,
                ChangeClass::C2 => 3usize,
                ChangeClass::C3 => 5usize,
            })
            .sum();
        map.insert(format!("window_{}", idx + 1), score);
    }
    map
}

pub fn suggest_read_paths(modules: &[ModuleResult], routing: &RoutingIndex) -> Vec<String> {
    let mut suggestions = Vec::new();
    let mut ranked = modules.to_vec();
    ranked.sort_by(|a, b| (b.fan_out + b.fan_in).cmp(&(a.fan_out + a.fan_in)));

    for m in ranked.into_iter().take(5) {
        let mut files = routing
            .entries
            .iter()
            .filter(|e| e.module_id == m.module_id)
            .map(|e| e.source_path.clone())
            .collect::<Vec<_>>();
        files.sort();
        files.dedup();
        if !files.is_empty() {
            suggestions.push(format!("{} -> {}", m.module_id, files.join(" -> ")));
        }
    }
    suggestions
}

pub fn adr_skeleton_for_module(module: &ModuleResult) -> Option<String> {
    let drift = module.drift_level.to_uppercase();
    if drift != "C2" && drift != "C3" {
        return None;
    }

    Some(format!(
        "# ADR: {} drift {}\n\n## Status\nProposed\n\n## Context\n- module: {}\n- drift: {}\n- exports: {}\n\n## Decision\n[To be filled]\n\n## Consequences\n[To be filled]\n",
        module.module_id,
        drift,
        module.module_id,
        drift,
        module.exports.join(", ")
    ))
}

fn sanitize_node(n: &str) -> String {
    n.replace(['/', ':', '-', '.'], "_")
}

#[cfg(test)]
mod tests {
    use super::*;
    use wtcd_core::types::ConfidenceBand;

    fn m(id: &str, deps: &[&str], drift: &str) -> ModuleResult {
        ModuleResult {
            module_id: id.to_string(),
            language: "typescript".to_string(),
            files: vec![format!("{id}/a.ts")],
            exports: vec!["A".to_string()],
            dependencies: deps.iter().map(|d| (*d).to_string()).collect(),
            side_effects: vec![],
            responsibility: "demo".to_string(),
            semantic_fingerprint: "sha256:abc".to_string(),
            fan_in: 0,
            fan_out: deps.len(),
            drift_level: drift.to_string(),
            confidence: ConfidenceBand::High,
        }
    }

    #[test]
    fn builds_knowledge_and_documents() {
        let modules = vec![m("src/auth", &["src/core"], "C1"), m("src/core", &[], "C2")];
        let knowledge = build_knowledge_result(&modules, 1000);
        assert_eq!(knowledge.module_count, 2);
        let overview = generate_repo_overview(&modules, &knowledge);
        assert!(overview.contains("modules: 2"));

        let mermaid = generate_module_dep_mermaid(&modules);
        assert!(mermaid.contains("graph LR"));

        let export_index = generate_export_index(&modules);
        assert!(export_index.contains("src/auth::A"));
    }

    #[test]
    fn supports_clusters_hotspots_readpath_and_adr() {
        let modules = vec![m("src/auth", &["src/core"], "C3"), m("src/core", &[], "C2")];
        let clusters = community_clusters(&modules);
        assert!(!clusters.is_empty());

        let hotspots = hotspot_map_from_drift(&[
            vec![ChangeClass::C1, ChangeClass::C2],
            vec![ChangeClass::C3],
        ]);
        assert_eq!(hotspots.get("window_2").copied().unwrap_or(0), 5);

        let routing = RoutingIndex {
            api_version: "1".to_string(),
            entries: vec![
                wtcd_core::index::RoutingIndexEntry {
                    artifact_id: "file_mirror:src/auth/a.ts".to_string(),
                    module_id: "src/auth".to_string(),
                    source_path: "src/auth/a.ts".to_string(),
                    exports: vec!["A".to_string()],
                    keywords: vec![],
                    dependencies: vec![],
                    risk_tags: vec![],
                    freshness: "fresh".to_string(),
                    confidence: "high".to_string(),
                },
                wtcd_core::index::RoutingIndexEntry {
                    artifact_id: "file_mirror:src/core/b.ts".to_string(),
                    module_id: "src/core".to_string(),
                    source_path: "src/core/b.ts".to_string(),
                    exports: vec!["B".to_string()],
                    keywords: vec![],
                    dependencies: vec![],
                    risk_tags: vec![],
                    freshness: "fresh".to_string(),
                    confidence: "high".to_string(),
                },
            ],
        };

        let suggestions = suggest_read_paths(&modules, &routing);
        assert!(!suggestions.is_empty());

        let adr = adr_skeleton_for_module(&modules[0]).unwrap();
        assert!(adr.contains("ADR"));
    }
}
