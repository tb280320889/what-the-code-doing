use crate::output::format_json;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::time::Instant;
use wtcd_adapters::register_all_adapters;
use wtcd_core::types::*;

pub fn run_analysis(repo_root: &Path, full: bool) -> wtcd_core::error::Result<()> {
    let overall_start = Instant::now();

    // 1. Load config (D-10: must have anrsm.yaml)
    let config = wtcd_scope::load_config(repo_root)?;

    // 2. Register adapters
    let registry = register_all_adapters()
        .map_err(|e| wtcd_core::error::WtcdError::ConfigError(e.to_string()))?;

    // 3. Determine files to parse
    let files_to_parse: Vec<std::path::PathBuf> = if full {
        // Full mode: scan all files (existing behavior, INCR-03)
        wtcd_scope::scan(repo_root, &config.scope)?
    } else {
        // Incremental mode: diff → changed + neighbors (INCR-01, INCR-02)
        match wtcd_diff::git_diff::diff_working_tree_vs_head(repo_root) {
            Ok(diff) if !diff.is_clean => {
                // Get all scoped files (for dep graph)
                let all_files = wtcd_scope::scan(repo_root, &config.scope)?;

                // Parse all files to build dep graph
                let mut all_results = Vec::new();
                for fp in &all_files {
                    let relative = fp
                        .strip_prefix(repo_root)
                        .unwrap_or(fp)
                        .to_string_lossy()
                        .to_string();
                    let adapter = registry.find_adapter(&relative);
                    let result = match adapter {
                        Some(a) => match std::fs::read_to_string(fp) {
                            Ok(s) => a.parse(&s, &relative),
                            Err(_) => continue,
                        },
                        None => continue,
                    };
                    all_results.push(result);
                }

                // Build reverse dep graph
                let dep_graph = wtcd_core::depgraph::ReverseDepGraph::build(&all_results);

                // Changed files from diff
                let mut changed_set = HashSet::new();
                for cf in &diff.changed_files {
                    changed_set.insert(cf.path.clone());
                }

                // Expand to neighbors
                let neighbors = dep_graph.expand_affected(&changed_set);
                let mut parse_set = changed_set;
                parse_set.extend(neighbors);

                // Filter all_files to only parse_set
                all_files
                    .into_iter()
                    .filter(|fp| {
                        let relative = fp
                            .strip_prefix(repo_root)
                            .unwrap_or(fp)
                            .to_string_lossy()
                            .to_string();
                        parse_set.contains(&relative)
                    })
                    .collect()
            }
            _ => {
                // Clean working tree or diff failed — fall back to full
                wtcd_scope::scan(repo_root, &config.scope)?
            }
        }
    };

    // 4. Parse each file
    let mut file_results = Vec::new();
    for file_path in &files_to_parse {
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

    // 5. Get source commit hash (using gix via wtcd-diff)
    let source_commit =
        wtcd_diff::git_diff::get_head_commit(repo_root).unwrap_or_else(|_| "unknown".to_string());

    // 6. Generate mirror files
    let mirror_config = config.mirror.clone().unwrap_or_default();
    let mirror_root = repo_root.join(&mirror_config.output_dir);

    let mut mirror_generated = 0usize;
    let mut mirror_errors = 0usize;
    for (file_path, file_result) in files_to_parse.iter().zip(file_results.iter()) {
        if file_result.confidence == ConfidenceBand::None {
            continue; // Skip files that couldn't be parsed
        }

        let relative = file_path
            .strip_prefix(repo_root)
            .unwrap_or(file_path)
            .to_string_lossy()
            .to_string();

        let source_content = std::fs::read_to_string(file_path).unwrap_or_default();

        let module_id = derive_module_id(&relative, &config.scope.source_roots);

        match wtcd_mirror::io::generate_and_write_mirror(
            &relative,
            &source_content,
            file_result,
            &source_commit,
            &module_id,
            &mirror_config.output_dir,
            mirror_config.include_manual_appendix,
        ) {
            Ok(_) => mirror_generated += 1,
            Err(e) => {
                eprintln!("Warning: failed to generate mirror for {}: {}", relative, e);
                mirror_errors += 1;
            }
        }
    }

    // 7. Clean up orphan mirrors
    let source_paths: Vec<String> = files_to_parse
        .iter()
        .map(|f| {
            f.strip_prefix(repo_root)
                .unwrap_or(f)
                .to_string_lossy()
                .to_string()
        })
        .collect();
    if let Ok(orphans) = wtcd_mirror::io::find_orphan_mirrors(&source_paths, &mirror_root) {
        let _ = wtcd_mirror::io::delete_orphans(&orphans);
    }

    // 7b. Build and write routing index (D-14, RTIX-01)
    let module_id_map: HashMap<String, String> = files_to_parse
        .iter()
        .map(|f| {
            let relative = f
                .strip_prefix(repo_root)
                .unwrap_or(f)
                .to_string_lossy()
                .to_string();
            let module_id = derive_module_id(&relative, &config.scope.source_roots);
            (relative, module_id)
        })
        .collect();

    let routing_index = wtcd_core::index::build_routing_index(&file_results, &module_id_map);
    let index_path = mirror_root
        .parent()
        .unwrap_or_else(|| std::path::Path::new("mirror"))
        .join("routing_index.json");

    if let Err(e) = wtcd_core::index::write_routing_index(&routing_index, &index_path) {
        eprintln!("Warning: failed to write routing index: {}", e);
    } else {
        eprintln!(
            "Routing index: {} entries → {}",
            routing_index.entries.len(),
            index_path.display()
        );
    }

    // 8. Build summary (D-07)
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

    // 9. Output JSON (D-01, D-04, CORE-06)
    let output = RunOutput {
        api_version: "1".to_string(),
        files: file_results,
        errors: vec![],
        summary,
    };

    format_json(&output);

    // Print mirror stats to stderr
    eprintln!(
        "Mirror: {} generated, {} errors",
        mirror_generated, mirror_errors
    );

    Ok(())
}

fn derive_module_id(relative_path: &str, source_roots: &[String]) -> String {
    let path = std::path::Path::new(relative_path);
    let components: Vec<_> = path
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .collect();

    if components.is_empty() {
        return "global".to_string();
    }

    // Skip source root prefix if it matches
    if source_roots.iter().any(|r| r == &components[0]) {
        if components.len() > 1 {
            return components[1].clone();
        }
    }

    // Fallback: use first directory component or global
    path.parent()
        .and_then(|p| p.components().next())
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .unwrap_or_else(|| "global".to_string())
}
