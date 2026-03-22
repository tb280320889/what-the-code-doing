use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use crate::fingerprint::compute_source_fingerprint;
use wtcd_core::types::{ChangeClass, FileResult, ModuleResult};

fn detect_language(path: &str) -> &'static str {
    match std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
    {
        "py" | "pyi" => "python",
        "go" => "go",
        "ts" | "tsx" => "typescript",
        "js" | "jsx" => "javascript",
        _ => "unknown",
    }
}

fn path_to_module_fallback(path: &str) -> String {
    let p = std::path::Path::new(path);
    if let Some(parent) = p.parent() {
        let s = parent.to_string_lossy().to_string();
        if !s.is_empty() && s != "." {
            return s;
        }
    }
    "global".to_string()
}

pub fn detect_module_id(path: &str, source: &str) -> String {
    let language = detect_language(path);
    match language {
        "python" => {
            if path.ends_with("/__init__.py") {
                return path.trim_end_matches("/__init__.py").to_string();
            }
            path_to_module_fallback(path)
        }
        "go" => {
            for line in source.lines() {
                let clean = line.trim();
                if let Some(rest) = clean.strip_prefix("package ") {
                    let pkg = rest.split_whitespace().next().unwrap_or("").trim();
                    if !pkg.is_empty() {
                        let dir = path_to_module_fallback(path);
                        return format!("{dir}:{pkg}");
                    }
                }
            }
            path_to_module_fallback(path)
        }
        _ => path_to_module_fallback(path),
    }
}

pub fn group_by_module_id<'a>(
    file_results: &'a [FileResult],
    source_map: &HashMap<String, String>,
) -> BTreeMap<String, Vec<&'a FileResult>> {
    let mut grouped: BTreeMap<String, Vec<&FileResult>> = BTreeMap::new();
    for fr in file_results {
        let module_id = source_map
            .get(&fr.file_path)
            .map(|src| detect_module_id(&fr.file_path, src))
            .unwrap_or_else(|| path_to_module_fallback(&fr.file_path));
        grouped.entry(module_id).or_default().push(fr);
    }
    grouped
}

pub fn aggregate_module(
    module_id: &str,
    files: &[&FileResult],
    module_deps_fanin: Option<(usize, usize)>,
    module_drift: Option<ChangeClass>,
) -> ModuleResult {
    let mut export_set = BTreeSet::new();
    let mut dep_set = BTreeSet::new();
    let mut side_effect_set = BTreeSet::new();
    let mut file_paths = Vec::new();
    let mut languages = BTreeSet::new();
    let mut child_semantic = Vec::new();

    for f in files {
        file_paths.push(f.file_path.clone());
        languages.insert(detect_language(&f.file_path).to_string());
        for e in &f.exports {
            export_set.insert(e.name.clone());
        }
        for d in &f.imports {
            dep_set.insert(d.source.clone());
        }
        for s in &f.side_effects {
            side_effect_set.insert(format!("{:?}:{}", s.kind, s.target));
        }
        child_semantic.push(compute_source_fingerprint(&format!(
            "{}:{:?}:{:?}:{:?}",
            f.file_path, f.exports, f.imports, f.signatures
        )));
    }

    child_semantic.sort();
    let aggregate_input = child_semantic.join("|");
    let semantic_fingerprint = compute_source_fingerprint(&aggregate_input);

    let (fan_in, fan_out) = module_deps_fanin.unwrap_or((0, dep_set.len()));
    let drift_level = module_drift
        .map(|c| format!("{:?}", c))
        .unwrap_or_else(|| "C0".to_string());

    let responsibility = if export_set.is_empty() {
        format!("模块 {module_id} 提供内部实现，无显式导出。")
    } else {
        format!(
            "模块 {module_id} 聚合导出 {}，覆盖 {} 个文件。",
            export_set.iter().cloned().collect::<Vec<_>>().join("、"),
            file_paths.len()
        )
    };

    ModuleResult {
        module_id: module_id.to_string(),
        language: if languages.len() == 1 {
            languages
                .into_iter()
                .next()
                .unwrap_or_else(|| "unknown".to_string())
        } else {
            "mixed".to_string()
        },
        files: file_paths,
        exports: export_set.into_iter().collect(),
        dependencies: dep_set.into_iter().collect(),
        side_effects: side_effect_set.into_iter().collect(),
        responsibility,
        semantic_fingerprint,
        fan_in,
        fan_out,
        drift_level,
    }
}

pub fn build_module_graph(modules: &[ModuleResult]) -> HashMap<String, HashSet<String>> {
    let mut graph: HashMap<String, HashSet<String>> = HashMap::new();
    let all_modules: Vec<String> = modules.iter().map(|m| m.module_id.clone()).collect();

    for m in modules {
        let mut targets = HashSet::new();
        for dep in &m.dependencies {
            for other in &all_modules {
                if dep.contains(other) && other != &m.module_id {
                    targets.insert(other.clone());
                }
            }
        }
        graph.insert(m.module_id.clone(), targets);
    }

    graph
}

pub fn fanin_fanout(graph: &HashMap<String, HashSet<String>>, module_id: &str) -> (usize, usize) {
    let fan_out = graph.get(module_id).map(|s| s.len()).unwrap_or(0);
    let fan_in = graph
        .values()
        .filter(|targets| targets.contains(module_id))
        .count();
    (fan_in, fan_out)
}

pub fn rollup_drift_level(classes: &[ChangeClass]) -> ChangeClass {
    if classes.iter().any(|c| matches!(c, ChangeClass::C3)) {
        return ChangeClass::C3;
    }
    if classes.iter().any(|c| matches!(c, ChangeClass::C2)) {
        return ChangeClass::C2;
    }
    if classes.iter().any(|c| matches!(c, ChangeClass::C1)) {
        return ChangeClass::C1;
    }
    ChangeClass::C0
}

#[cfg(test)]
mod tests {
    use super::*;
    use wtcd_core::types::{
        ConfidenceBand, DependencyEdge, ExportKind, ExportedSymbol, FileResult, FunctionSignature,
        ImportKind, SideEffect,
    };

    fn fr(path: &str, exports: &[&str], deps: &[&str]) -> FileResult {
        FileResult {
            file_path: path.to_string(),
            confidence: ConfidenceBand::High,
            exports: exports
                .iter()
                .map(|e| ExportedSymbol {
                    name: (*e).to_string(),
                    kind: ExportKind::Function,
                    line: 1,
                })
                .collect(),
            imports: deps
                .iter()
                .map(|d| DependencyEdge {
                    source: (*d).to_string(),
                    symbols: vec![],
                    kind: ImportKind::Named,
                })
                .collect(),
            signatures: vec![FunctionSignature {
                name: "f".to_string(),
                parameters: vec![],
                return_type: "void".to_string(),
            }],
            side_effects: vec![SideEffect {
                kind: wtcd_core::types::SideEffectKind::Log,
                target: "log".to_string(),
                line: 1,
            }],
            parse_time_ms: 1,
            error_message: None,
        }
    }

    #[test]
    fn detects_python_and_go_modules() {
        assert_eq!(detect_module_id("pkg/__init__.py", ""), "pkg");
        assert_eq!(
            detect_module_id("svc/user/service.go", "package user\n"),
            "svc/user:user"
        );
    }

    #[test]
    fn groups_and_aggregates_modules() {
        let files = vec![
            fr("src/a/x.ts", &["A"], &["src/b"]),
            fr("src/a/y.ts", &["B"], &[]),
            fr("src/b/z.ts", &["C"], &["src/a"]),
        ];
        let mut source_map = HashMap::new();
        source_map.insert("src/a/x.ts".to_string(), "export const A = 1".to_string());
        source_map.insert("src/a/y.ts".to_string(), "export const B = 1".to_string());
        source_map.insert("src/b/z.ts".to_string(), "export const C = 1".to_string());

        let grouped = group_by_module_id(&files, &source_map);
        assert!(grouped.contains_key("src/a"));
        assert!(grouped.contains_key("src/b"));

        let a = aggregate_module("src/a", grouped.get("src/a").unwrap(), None, None);
        let b = aggregate_module("src/b", grouped.get("src/b").unwrap(), None, None);
        let graph = build_module_graph(&[a.clone(), b.clone()]);
        let (fanin_a, fanout_a) = fanin_fanout(&graph, "src/a");
        assert_eq!(fanout_a, 1);
        assert_eq!(fanin_a, 1);
        assert!(!a.semantic_fingerprint.is_empty());
    }

    #[test]
    fn drift_rollup_works() {
        assert!(matches!(
            rollup_drift_level(&[ChangeClass::C0, ChangeClass::C1]),
            ChangeClass::C1
        ));
        assert!(matches!(
            rollup_drift_level(&[ChangeClass::C2, ChangeClass::C1]),
            ChangeClass::C2
        ));
        assert!(matches!(
            rollup_drift_level(&[ChangeClass::C3, ChangeClass::C2]),
            ChangeClass::C3
        ));
    }
}
