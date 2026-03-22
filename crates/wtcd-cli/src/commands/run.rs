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

        let module_id = wtcd_mirror::module::detect_module_id(&relative, &source_content);

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
    let source_map: HashMap<String, String> = files_to_parse
        .iter()
        .filter_map(|f| {
            let relative = f
                .strip_prefix(repo_root)
                .unwrap_or(f)
                .to_string_lossy()
                .to_string();
            std::fs::read_to_string(f).ok().map(|src| (relative, src))
        })
        .collect();

    let module_id_map: HashMap<String, String> = files_to_parse
        .iter()
        .map(|f| {
            let relative = f
                .strip_prefix(repo_root)
                .unwrap_or(f)
                .to_string_lossy()
                .to_string();
            let source = source_map.get(&relative).map(|s| s.as_str()).unwrap_or("");
            let module_id = wtcd_mirror::module::detect_module_id(&relative, source);
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

    // 7c. Build module aggregation and module mirrors (Phase 7)
    let grouped = wtcd_mirror::module::group_by_module_id(&file_results, &source_map);
    let mut modules: Vec<ModuleResult> = grouped
        .iter()
        .map(|(module_id, files)| {
            wtcd_mirror::module::aggregate_module(module_id, files, None, None)
        })
        .collect();
    let graph = wtcd_mirror::module::build_module_graph(&modules);
    modules = modules
        .into_iter()
        .map(|mut m| {
            let (fan_in, fan_out) = wtcd_mirror::module::fanin_fanout(&graph, &m.module_id);
            m.fan_in = fan_in;
            m.fan_out = fan_out;
            m
        })
        .collect();

    if let Err(e) = write_module_mirrors(repo_root, &mirror_config.module_output_dir, &modules) {
        eprintln!("Warning: failed to write module mirrors: {}", e);
    }

    // 7d. Build knowledge layer documents (Phase 8)
    let source_tokens = source_map
        .values()
        .map(|s| s.split_whitespace().count())
        .sum::<usize>();
    let knowledge = wtcd_mirror::knowledge::build_knowledge_result(&modules, source_tokens);
    let overview = wtcd_mirror::knowledge::generate_repo_overview(&modules, &knowledge);
    let depgraph_mermaid = wtcd_mirror::knowledge::generate_module_dep_mermaid(&modules);
    let export_index = wtcd_mirror::knowledge::generate_export_index(&modules);
    let clusters = wtcd_mirror::knowledge::community_clusters(&modules);
    let hotspots = wtcd_mirror::knowledge::hotspot_map_from_drift(
        &modules
            .iter()
            .map(|m| vec![parse_drift(&m.drift_level)])
            .collect::<Vec<_>>(),
    );
    let read_paths = wtcd_mirror::knowledge::suggest_read_paths(&modules, &routing_index);
    let adrs = modules
        .iter()
        .filter_map(wtcd_mirror::knowledge::adr_skeleton_for_module)
        .collect::<Vec<_>>();

    if let Err(e) = write_knowledge_docs(
        repo_root,
        &mirror_config.knowledge_output_dir,
        &knowledge,
        &overview,
        &depgraph_mermaid,
        &export_index,
        &clusters,
        &hotspots,
        &read_paths,
        &adrs,
    ) {
        eprintln!("Warning: failed to write knowledge docs: {}", e);
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

fn write_module_mirrors(
    repo_root: &Path,
    module_output_dir: &str,
    modules: &[ModuleResult],
) -> wtcd_core::error::Result<()> {
    let module_root = repo_root.join(module_output_dir);
    std::fs::create_dir_all(&module_root)?;

    for module in modules {
        let module_filename = format!("{}.md", sanitize_module_file_name(&module.module_id));
        let path = module_root.join(module_filename);
        let content = format!(
            "---\nanrsm_version: 1\nartifact_type: module_mirror\nartifact_id: module_mirror:{}\nmodule_id: {}\nlanguage: {}\nsemantic_fingerprint: {}\nfan_in: {}\nfan_out: {}\ndrift_level: {}\n---\n\n## 责任\n{}\n\n## 文件\n{}\n\n## 导出\n{}\n\n## 依赖\n{}\n\n## 副作用\n{}\n",
            module.module_id,
            module.module_id,
            module.language,
            module.semantic_fingerprint,
            module.fan_in,
            module.fan_out,
            module.drift_level,
            module.responsibility,
            if module.files.is_empty() {
                "-".to_string()
            } else {
                module.files.join("\n")
            },
            if module.exports.is_empty() {
                "-".to_string()
            } else {
                module.exports.join("\n")
            },
            if module.dependencies.is_empty() {
                "-".to_string()
            } else {
                module.dependencies.join("\n")
            },
            if module.side_effects.is_empty() {
                "-".to_string()
            } else {
                module.side_effects.join("\n")
            }
        );
        std::fs::write(path, content)?;
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn write_knowledge_docs(
    repo_root: &Path,
    knowledge_output_dir: &str,
    knowledge: &KnowledgeResult,
    overview: &str,
    depgraph_mermaid: &str,
    export_index: &str,
    clusters: &[Vec<String>],
    hotspots: &std::collections::BTreeMap<String, usize>,
    read_paths: &[String],
    adrs: &[String],
) -> wtcd_core::error::Result<()> {
    let root = repo_root.join(knowledge_output_dir);
    std::fs::create_dir_all(&root)?;

    let stats = format!(
        "# Language & File Statistics\n\n- module_count: {}\n- total_files: {}\n- total_exports: {}\n- token_compression_ratio: {:.4}\n- language_distribution: {}\n",
        knowledge.module_count,
        knowledge.total_files,
        knowledge.total_exports,
        knowledge.token_compression_ratio,
        knowledge
            .language_distribution
            .iter()
            .map(|(k, v)| format!("{k}:{v}"))
            .collect::<Vec<_>>()
            .join(", ")
    );

    let clusters_doc = format!(
        "# Semantic Clusters\n\n{}",
        clusters
            .iter()
            .enumerate()
            .map(|(idx, c)| format!("- cluster_{}: {}", idx + 1, c.join(", ")))
            .collect::<Vec<_>>()
            .join("\n")
    );

    let hotspot_doc = format!(
        "# Hotspot Map\n\n{}",
        hotspots
            .iter()
            .map(|(k, v)| format!("- {k}: {v}"))
            .collect::<Vec<_>>()
            .join("\n")
    );

    let path_doc = format!(
        "# Agent Read Paths\n\n{}",
        if read_paths.is_empty() {
            "- none".to_string()
        } else {
            read_paths
                .iter()
                .map(|p| format!("- {}", p))
                .collect::<Vec<_>>()
                .join("\n")
        }
    );

    std::fs::write(root.join("overview.md"), overview)?;
    std::fs::write(root.join("module-deps.mmd"), depgraph_mermaid)?;
    std::fs::write(root.join("export-index.md"), export_index)?;
    std::fs::write(root.join("stats.md"), stats)?;
    std::fs::write(root.join("clusters.md"), clusters_doc)?;
    std::fs::write(root.join("hotspots.md"), hotspot_doc)?;
    std::fs::write(root.join("read-paths.md"), path_doc)?;

    if !adrs.is_empty() {
        let adr_root = root.join("adr");
        std::fs::create_dir_all(&adr_root)?;
        for (idx, adr) in adrs.iter().enumerate() {
            std::fs::write(adr_root.join(format!("ADR-{:03}.md", idx + 1)), adr)?;
        }
    }

    Ok(())
}

fn sanitize_module_file_name(module_id: &str) -> String {
    module_id.replace(['/', ':', '\\'], "__")
}

fn parse_drift(level: &str) -> wtcd_core::types::ChangeClass {
    match level.to_uppercase().as_str() {
        "C3" => wtcd_core::types::ChangeClass::C3,
        "C2" => wtcd_core::types::ChangeClass::C2,
        "C1" => wtcd_core::types::ChangeClass::C1,
        _ => wtcd_core::types::ChangeClass::C0,
    }
}
