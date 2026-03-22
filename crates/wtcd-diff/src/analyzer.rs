use crate::classifier;
use crate::types::*;
use chrono::Utc;
use std::collections::HashSet;
use uuid::Uuid;
use wtcd_core::depgraph::ReverseDepGraph;
use wtcd_core::types::*;

/// Analyze drift for a set of changed files (DRFT-03)
///
/// Takes changed files from git diff, current parse results, and mirror root.
/// Returns a DriftReport conforming to drift-report.schema.json.
pub fn analyze_drift(
    diff_result: &DiffResult,
    file_results: &[FileResult],
    mirror_root: &str,
    systemic_threshold: usize,
) -> DriftReport {
    // Build reverse dependency graph from all file results (D-13)
    let dep_graph = ReverseDepGraph::build(file_results);

    // Build lookup: path -> FileResult
    let result_map: std::collections::HashMap<&str, &FileResult> = file_results
        .iter()
        .map(|r| (r.file_path.as_str(), r))
        .collect();

    let mut material_changes: Vec<MaterialChange> = Vec::new();
    let mut affected_artifacts: Vec<AffectedArtifact> = Vec::new();
    let mut all_changed_paths: HashSet<String> = HashSet::new();

    // Classify each changed file
    for changed_file in &diff_result.changed_files {
        all_changed_paths.insert(changed_file.path.clone());

        let current_result = result_map.get(changed_file.path.as_str()).copied();

        let source_content = match changed_file.status {
            FileStatus::Deleted => String::new(),
            _ => {
                // Read current source from disk
                std::fs::read_to_string(&changed_file.path).unwrap_or_default()
            }
        };

        let result = current_result.cloned().unwrap_or_else(|| FileResult {
            file_path: changed_file.path.clone(),
            confidence: ConfidenceBand::None,
            exports: vec![],
            imports: vec![],
            signatures: vec![],
            side_effects: vec![],
            parse_time_ms: 0,
            error_message: Some("File not in parse results".to_string()),
        });

        if let Some((change_class, summary, evidence)) = classifier::classify_change(
            changed_file,
            &source_content,
            &result,
            mirror_root,
            systemic_threshold,
            &dep_graph,
        ) {
            material_changes.push(MaterialChange {
                change_class,
                source_path: changed_file.path.clone(),
                summary: summary.clone(),
                evidence: Some(evidence),
            });

            // Determine drift level for affected artifact
            let drift_level = match change_class {
                ChangeClass::C0 => "low",
                ChangeClass::C1 => "material",
                ChangeClass::C2 => "material",
                ChangeClass::C3 => "blocking",
            };

            affected_artifacts.push(AffectedArtifact {
                artifact_id: format!("file_mirror:{}", changed_file.path),
                before_freshness: "fresh".to_string(),
                after_freshness: "stale".to_string(),
                drift_level: drift_level.to_string(),
                reason: summary,
                recommended_action: Some("Run anrsm run to update mirror".to_string()),
            });
        }
    }

    // Expand to affected neighbors (DRFT-04)
    let neighbors = dep_graph.expand_affected(&all_changed_paths);
    for neighbor in &neighbors {
        // Neighbors are indirectly affected — their mirrors are stale
        affected_artifacts.push(AffectedArtifact {
            artifact_id: format!("file_mirror:{}", neighbor),
            before_freshness: "fresh".to_string(),
            after_freshness: "stale".to_string(),
            drift_level: "low".to_string(),
            reason: "Imports a changed file".to_string(),
            recommended_action: Some("Run anrsm run to update mirror".to_string()),
        });
    }

    // Compute overall drift level
    let overall_drift_level = if material_changes
        .iter()
        .any(|m| m.change_class == ChangeClass::C3)
    {
        "blocking"
    } else if material_changes.iter().any(|m| {
        matches!(
            m.change_class,
            ChangeClass::C2 | ChangeClass::C1 | ChangeClass::C0
        )
    }) {
        "material"
    } else if !material_changes.is_empty() {
        "low"
    } else {
        "none"
    };

    DriftReport {
        report_version: 1,
        report_id: Uuid::new_v4().to_string(),
        repo_commit_before: diff_result.head_commit.clone(),
        repo_commit_after: "working-tree".to_string(),
        generated_at: Utc::now().to_rfc3339(),
        drift_level: overall_drift_level.to_string(),
        changed_files: diff_result
            .changed_files
            .iter()
            .map(|f| f.path.clone())
            .collect(),
        affected_artifacts,
        material_changes,
        policy_result: PolicyResult {
            status: "pending".to_string(),
            failed_rules: vec![],
            failure_codes: None,
            recommendations: None,
        },
    }
}
