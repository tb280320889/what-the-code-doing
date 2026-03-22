use crate::types::FileResult;
use std::collections::{HashMap, HashSet};

/// Reverse dependency graph: for each file, who imports it? (D-13, D-14)
///
/// Build from Vec<FileResult> — each FileResult.imports gives forward edges.
/// This struct inverts them to provide reverse lookup: path -> Vec<importer>
#[derive(Debug, Clone)]
pub struct ReverseDepGraph {
    /// path -> list of files that import it
    /// e.g., "src/utils.ts" -> ["src/app.ts", "src/handler.ts"]
    importers: HashMap<String, Vec<String>>,
    /// path -> list of files it imports (forward edges, kept for queries)
    imports: HashMap<String, Vec<String>>,
}

impl ReverseDepGraph {
    /// Build reverse dependency graph from all file results (D-14)
    pub fn build(file_results: &[FileResult]) -> Self {
        let mut importers: HashMap<String, Vec<String>> = HashMap::new();
        let mut imports: HashMap<String, Vec<String>> = HashMap::new();

        for result in file_results {
            let importer_path = &result.file_path;

            for edge in &result.imports {
                let imported_path = Self::resolve_import_path(importer_path, &edge.source);

                // Forward edge: this file imports these
                imports
                    .entry(importer_path.clone())
                    .or_default()
                    .push(imported_path.clone());

                // Reverse edge: the imported file is imported by this file
                importers
                    .entry(imported_path)
                    .or_default()
                    .push(importer_path.clone());
            }
        }

        Self { importers, imports }
    }

    /// Get direct importers of a file (who depends on this file?)
    pub fn get_importers(&self, path: &str) -> &[String] {
        self.importers
            .get(path)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Get direct imports of a file (what does this file depend on?)
    pub fn get_imports(&self, path: &str) -> &[String] {
        self.imports.get(path).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Get the import count for a file (how many files import it?)
    pub fn importer_count(&self, path: &str) -> usize {
        self.importers.get(path).map(|v| v.len()).unwrap_or(0)
    }

    /// Given a set of changed files, find all files that are directly affected
    /// (files that import any of the changed files)
    pub fn expand_affected(&self, changed: &HashSet<String>) -> HashSet<String> {
        let mut affected = HashSet::new();
        for path in changed {
            for importer in self.get_importers(path) {
                if !changed.contains(importer) {
                    affected.insert(importer.clone());
                }
            }
        }
        affected
    }

    /// All file paths in the graph
    pub fn all_paths(&self) -> HashSet<String> {
        let mut paths = HashSet::new();
        for k in self.importers.keys() {
            paths.insert(k.clone());
        }
        for k in self.imports.keys() {
            paths.insert(k.clone());
        }
        paths
    }

    /// Resolve a relative import specifier to a canonical path
    /// E.g., "./utils" from "src/app.ts" -> "src/utils.ts"
    /// E.g., "../lib/helper" from "src/components/Button.ts" -> "src/lib/helper.ts"
    /// E.g., "./mod" from "src/main.rs" -> "src/mod.rs"
    fn resolve_import_path(importer_path: &str, import_source: &str) -> String {
        use std::path::Path;

        let importer_dir = Path::new(importer_path).parent().unwrap_or(Path::new("."));

        // Handle relative imports
        if import_source.starts_with('.') {
            let resolved = importer_dir.join(import_source);
            let normalized = resolved.components().collect::<std::path::PathBuf>();

            let mut path_str = normalized.to_string_lossy().to_string();

            // Add extension if missing — infer from importer's language
            if Path::new(&path_str).extension().is_none() && !path_str.contains("/index") {
                let importer_ext = Path::new(importer_path)
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("");

                let candidates = Self::extension_candidates(importer_ext);
                if !candidates.is_empty() {
                    // Use first candidate as default (we can't check filesystem in-memory)
                    path_str = format!("{}{}", path_str, candidates[0]);
                }
            }

            path_str
        } else {
            // Package import (e.g., "lodash", "@scope/pkg") — keep as-is
            import_source.to_string()
        }
    }

    /// Return extension candidates for import resolution based on the importer file's extension.
    /// First entry is the default when no filesystem check is possible.
    fn extension_candidates(importer_ext: &str) -> &'static [&'static str] {
        match importer_ext {
            // TypeScript/JavaScript
            "ts" | "tsx" | "js" | "jsx" => &[".ts", ".tsx", ".js", ".jsx"],
            // Rust
            "rs" => &[".rs"],
            // Python
            "py" => &[".py"],
            // Go
            "go" => &[".go"],
            // C
            "c" => &[".c", ".h"],
            // C++
            "cpp" | "cc" | "cxx" | "hpp" | "hh" | "hxx" => &[".cpp", ".cc", ".h", ".hpp"],
            // C#
            "cs" => &[".cs"],
            // Dart
            "dart" => &[".dart"],
            // Java
            "java" => &[".java"],
            // Kotlin
            "kt" | "kts" => &[".kt", ".kts"],
            // Swift
            "swift" => &[".swift"],
            // Zig
            "zig" => &[".zig"],
            // Unknown — fall back to TS/JS
            _ => &[".ts", ".tsx", ".js", ".jsx"],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    fn make_file_result(path: &str, imports: Vec<(&str, Vec<&str>)>) -> FileResult {
        FileResult {
            file_path: path.to_string(),
            confidence: crate::types::ConfidenceBand::High,
            exports: vec![],
            imports: imports
                .into_iter()
                .map(|(source, symbols)| DependencyEdge {
                    source: source.to_string(),
                    symbols: symbols.into_iter().map(|s| s.to_string()).collect(),
                    kind: crate::types::ImportKind::Named,
                })
                .collect(),
            signatures: vec![],
            side_effects: vec![],
            parse_time_ms: 0,
            error_message: None,
        }
    }

    #[test]
    fn build_reverse_graph_basic() {
        let files = vec![
            make_file_result("src/app.ts", vec![("./utils", vec!["helper"])]),
            make_file_result("src/utils.ts", vec![]),
        ];
        let graph = ReverseDepGraph::build(&files);

        assert_eq!(graph.get_importers("src/utils.ts"), &["src/app.ts"]);
        assert_eq!(graph.importer_count("src/utils.ts"), 1);
        assert_eq!(graph.get_importers("src/app.ts").len(), 0);
    }

    #[test]
    fn expand_affected_finds_importers() {
        let files = vec![
            make_file_result("src/app.ts", vec![("./utils", vec!["helper"])]),
            make_file_result("src/handler.ts", vec![("./utils", vec!["helper"])]),
            make_file_result("src/utils.ts", vec![]),
        ];
        let graph = ReverseDepGraph::build(&files);

        let mut changed = HashSet::new();
        changed.insert("src/utils.ts".to_string());

        let affected = graph.expand_affected(&changed);
        assert!(affected.contains("src/app.ts"));
        assert!(affected.contains("src/handler.ts"));
        assert!(!affected.contains("src/utils.ts")); // not in affected (it's in changed)
    }

    #[test]
    fn expand_affected_no_duplicates() {
        let files = vec![
            make_file_result("src/app.ts", vec![("./utils", vec!["a", "b"])]),
            make_file_result("src/utils.ts", vec![]),
        ];
        let graph = ReverseDepGraph::build(&files);

        let mut changed = HashSet::new();
        changed.insert("src/utils.ts".to_string());

        let affected = graph.expand_affected(&changed);
        assert_eq!(affected.len(), 1); // app.ts appears once despite importing 2 symbols
    }

    #[test]
    fn package_imports_preserved() {
        let files = vec![make_file_result(
            "src/app.ts",
            vec![("lodash", vec!["merge"])],
        )];
        let graph = ReverseDepGraph::build(&files);

        assert_eq!(graph.importer_count("lodash"), 1);
    }

    #[test]
    fn empty_files_gives_empty_graph() {
        let graph = ReverseDepGraph::build(&[]);
        assert_eq!(graph.all_paths().len(), 0);
    }
}
