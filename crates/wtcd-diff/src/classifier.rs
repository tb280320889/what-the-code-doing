use crate::types::{ChangedFile, FileStatus};
use std::collections::HashSet;
use wtcd_core::depgraph::ReverseDepGraph;
use wtcd_core::types::*;
use wtcd_mirror::fingerprint::{compute_semantic_fingerprint, compute_source_fingerprint};
use wtcd_mirror::io::read_mirror_header;

/// Classify a single changed file's drift level (D-05, D-06, D-07)
pub fn classify_change(
    changed_file: &ChangedFile,
    current_source: &str,
    current_result: &FileResult,
    mirror_root: &str,
    systemic_threshold: usize,
    dep_graph: &ReverseDepGraph,
) -> Option<(ChangeClass, String, Vec<String>)> {
    match changed_file.status {
        FileStatus::Added => {
            // New files = Material Change (D-02) — needs mirror generation
            let summary = format!("New file: {}", changed_file.path);
            let evidence = vec![
                format!("File added to repository"),
                format!("Exports: {}", current_result.exports.len()),
            ];

            // Check if systemic (D-07)
            let importer_count = dep_graph.importer_count(&changed_file.path);
            if importer_count >= systemic_threshold {
                return Some((ChangeClass::C3, summary, evidence));
            }

            if !current_result.exports.is_empty() {
                Some((ChangeClass::C2, summary, evidence))
            } else {
                Some((ChangeClass::C1, summary, evidence))
            }
        }
        FileStatus::Deleted => {
            // Deleted files = orphan in report (D-03)
            let summary = format!("File deleted: {}", changed_file.path);
            let evidence = vec!["Source file no longer exists".to_string()];
            let importer_count = dep_graph.importer_count(&changed_file.path);
            if importer_count >= systemic_threshold {
                Some((ChangeClass::C3, summary, evidence))
            } else if importer_count > 0 {
                Some((ChangeClass::C2, summary, evidence))
            } else {
                Some((ChangeClass::C1, summary, evidence))
            }
        }
        FileStatus::Modified => classify_modified(
            changed_file,
            current_source,
            current_result,
            mirror_root,
            systemic_threshold,
            dep_graph,
        ),
    }
}

/// Classify a modified file by comparing fingerprints
fn classify_modified(
    changed_file: &ChangedFile,
    current_source: &str,
    current_result: &FileResult,
    mirror_root: &str,
    systemic_threshold: usize,
    dep_graph: &ReverseDepGraph,
) -> Option<(ChangeClass, String, Vec<String>)> {
    // Compute current fingerprints
    let current_source_fp = compute_source_fingerprint(current_source);
    let current_semantic_fp = compute_semantic_fingerprint(current_result);

    // Try to read existing mirror header
    let mirror_path = format!("{}/{}.md", mirror_root, changed_file.path);
    let existing_header = read_mirror_header(std::path::Path::new(&mirror_path));

    match existing_header {
        Some(header) => {
            // Compare fingerprints (D-05, D-06)
            let source_changed = header.source_fingerprint != current_source_fp;
            let semantic_changed = header.semantic_fingerprint != current_semantic_fp;

            if !source_changed {
                // Source didn't change — no drift
                return None;
            }

            if source_changed && !semantic_changed {
                // C0: source changed, semantic unchanged (D-05)
                Some((
                    ChangeClass::C0,
                    format!("Formatting/comment change in {}", changed_file.path),
                    vec![
                        format!(
                            "source_fingerprint changed: {} -> {}",
                            header.source_fingerprint, current_source_fp
                        ),
                        "semantic_fingerprint unchanged".to_string(),
                    ],
                ))
            } else {
                // Semantic changed — check if exports changed (D-06)
                let exports_changed =
                    exports_differ(header.exports.as_deref(), &current_result.exports);
                let signatures_changed =
                    signatures_differ(header.contracts.as_deref(), &current_result.signatures);

                let importer_count = dep_graph.importer_count(&changed_file.path);

                if importer_count >= systemic_threshold {
                    // C3: systemic (D-07)
                    Some((
                        ChangeClass::C3,
                        format!(
                            "Systemic change in {} (imported by {} files)",
                            changed_file.path, importer_count
                        ),
                        vec![
                            format!("semantic_fingerprint changed"),
                            format!(
                                "importer_count: {} >= threshold: {}",
                                importer_count, systemic_threshold
                            ),
                        ],
                    ))
                } else if exports_changed || signatures_changed {
                    // C2: contract change (D-06)
                    let mut evidence = vec!["semantic_fingerprint changed".to_string()];
                    if exports_changed {
                        evidence.push("exports changed".to_string());
                    }
                    if signatures_changed {
                        evidence.push("signatures changed".to_string());
                    }
                    Some((
                        ChangeClass::C2,
                        format!("Contract change in {}", changed_file.path),
                        evidence,
                    ))
                } else {
                    // C1: internal semantic change (D-06)
                    Some((
                        ChangeClass::C1,
                        format!("Internal change in {}", changed_file.path),
                        vec![
                            "semantic_fingerprint changed".to_string(),
                            "exports and signatures unchanged".to_string(),
                        ],
                    ))
                }
            }
        }
        None => {
            // No existing mirror — treat as new file (D-15: always parse new files)
            if !current_result.exports.is_empty() {
                Some((
                    ChangeClass::C2,
                    format!("New file (no existing mirror): {}", changed_file.path),
                    vec!["No mirror found — needs generation".to_string()],
                ))
            } else {
                Some((
                    ChangeClass::C1,
                    format!("New file (no existing mirror): {}", changed_file.path),
                    vec!["No mirror found — needs generation".to_string()],
                ))
            }
        }
    }
}

/// Compare exports from mirror header vs current parse result
fn exports_differ(header_exports: Option<&[String]>, current_exports: &[ExportedSymbol]) -> bool {
    let header_set: HashSet<&str> = header_exports
        .unwrap_or(&[])
        .iter()
        .map(|s| s.as_str())
        .collect();
    let current_set: HashSet<&str> = current_exports.iter().map(|e| e.name.as_str()).collect();

    header_set != current_set
}

/// Compare signatures from mirror header contracts vs current parse result
fn signatures_differ(
    header_contracts: Option<&[String]>,
    current_signatures: &[FunctionSignature],
) -> bool {
    let header_set: HashSet<&str> = header_contracts
        .unwrap_or(&[])
        .iter()
        .map(|s| s.as_str())
        .collect();
    // Build contract strings from current signatures
    let current_set: HashSet<String> = current_signatures
        .iter()
        .map(|s| {
            format!(
                "{}({})",
                s.name,
                s.parameters
                    .iter()
                    .map(|p| format!("{}: {}", p.name, p.type_annotation))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })
        .collect();
    let current_str_set: HashSet<&str> = current_set.iter().map(|s| s.as_str()).collect();

    // Compare as strings
    if header_set.len() != current_str_set.len() {
        return true;
    }
    for s in &header_set {
        if !current_str_set.contains(s) {
            return true;
        }
    }
    false
}
