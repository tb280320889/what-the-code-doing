use wtcd_core::config::GateConfig;
use wtcd_core::types::{ChangeClass, DriftReport, MaterialChange, PolicyResult};

/// Failure codes (D-11: ANRSM-001 through ANRSM-010)
pub const ANRSM_001: &str = "ANRSM-001"; // C2 detected
pub const ANRSM_002: &str = "ANRSM-002"; // C3 detected
pub const ANRSM_003: &str = "ANRSM-003"; // stale mirrors
pub const ANRSM_004: &str = "ANRSM-004"; // orphan mirrors

/// Evaluate gate policy against a drift report (GATE-03)
///
/// Modifies report.policy_result in place.
/// Returns true if the gate passes (exit code 0), false if it fails (exit code 1).
pub fn evaluate_gate(report: &mut DriftReport, config: &GateConfig) -> bool {
    let mut failed_rules: Vec<String> = Vec::new();
    let mut failure_codes: Vec<String> = Vec::new();
    let mut recommendations: Vec<String> = Vec::new();

    // Check fail_on rules
    for rule in &config.fail_on {
        let matches = check_rule_matches(
            &rule.when,
            &report.material_changes,
            config.systemic_threshold,
        );
        if matches {
            failed_rules.push(format!("fail_on:{}", rule.when));
            if !rule.code.is_empty() {
                failure_codes.push(rule.code.clone());
            }
        }
    }

    // Check warn_on rules (don't fail, but add to recommendations)
    for rule in &config.warn_on {
        let matches = check_rule_matches(
            &rule.when,
            &report.material_changes,
            config.systemic_threshold,
        );
        if matches {
            recommendations.push(format!("warn: {} changes detected", rule.when));
        }
    }

    // Auto-detect standard failure codes based on change classes present (D-11)
    let has_c2 = report
        .material_changes
        .iter()
        .any(|m| m.change_class == ChangeClass::C2);
    let has_c3 = report
        .material_changes
        .iter()
        .any(|m| m.change_class == ChangeClass::C3);

    if has_c3 && !failure_codes.contains(&ANRSM_002.to_string()) {
        failure_codes.push(ANRSM_002.to_string());
    }
    if has_c2 && !failure_codes.contains(&ANRSM_001.to_string()) {
        failure_codes.push(ANRSM_001.to_string());
    }

    // Check for stale mirrors
    let has_stale = report
        .affected_artifacts
        .iter()
        .any(|a| a.after_freshness == "stale");
    if has_stale && !failure_codes.contains(&ANRSM_003.to_string()) {
        failure_codes.push(ANRSM_003.to_string());
    }

    // Check for orphan mirrors
    let has_orphan = report
        .material_changes
        .iter()
        .any(|m| m.summary.contains("deleted") || m.summary.contains("orphan"));
    if has_orphan && !failure_codes.contains(&ANRSM_004.to_string()) {
        failure_codes.push(ANRSM_004.to_string());
    }

    // Determine policy status
    let is_fail = !failed_rules.is_empty();
    let has_changes = !report.material_changes.is_empty();
    let status = if is_fail {
        "fail"
    } else if has_changes {
        "warn"
    } else {
        "pass"
    };

    if recommendations.is_empty() && !is_fail && !has_changes {
        recommendations.push("No policy violations detected".to_string());
    }

    report.policy_result = PolicyResult {
        status: status.to_string(),
        failed_rules,
        failure_codes: if failure_codes.is_empty() {
            None
        } else {
            Some(failure_codes)
        },
        recommendations: if recommendations.is_empty() {
            None
        } else {
            Some(recommendations)
        },
    };

    !is_fail
}

/// Check if a rule's `when` condition matches the current material changes
fn check_rule_matches(when: &str, changes: &[MaterialChange], _systemic_threshold: usize) -> bool {
    match when.to_lowercase().as_str() {
        "any" | "all" => !changes.is_empty(),
        "c0" => changes.iter().any(|m| m.change_class == ChangeClass::C0),
        "c1" => changes.iter().any(|m| m.change_class == ChangeClass::C1),
        "c2" => changes.iter().any(|m| m.change_class == ChangeClass::C2),
        "c3" => changes.iter().any(|m| m.change_class == ChangeClass::C3),
        "material" => changes.iter().any(|m| {
            matches!(
                m.change_class,
                ChangeClass::C1 | ChangeClass::C2 | ChangeClass::C3
            )
        }),
        _ => false,
    }
}

/// Get exit code from gate evaluation result
/// 0 = pass, 1 = fail
pub fn get_exit_code(passed: bool) -> i32 {
    if passed {
        0
    } else {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wtcd_core::config::{FailRule, GateConfig, WarnRule};
    use wtcd_core::types::*;

    fn make_report(material_changes: Vec<MaterialChange>) -> DriftReport {
        DriftReport {
            report_version: 1,
            report_id: "test".to_string(),
            repo_commit_before: "abc1234".to_string(),
            repo_commit_after: "working-tree".to_string(),
            generated_at: "2026-01-01T00:00:00Z".to_string(),
            drift_level: if material_changes.is_empty() {
                "none".to_string()
            } else {
                "material".to_string()
            },
            changed_files: vec![],
            affected_artifacts: vec![],
            material_changes,
            policy_result: PolicyResult {
                status: "pending".to_string(),
                failed_rules: vec![],
                failure_codes: None,
                recommendations: None,
            },
        }
    }

    fn make_change(class: ChangeClass, path: &str) -> MaterialChange {
        MaterialChange {
            change_class: class,
            source_path: path.to_string(),
            summary: format!("Change in {}", path),
            evidence: None,
        }
    }

    #[test]
    fn default_warn_only_passes() {
        let config = GateConfig::default();
        let mut report = make_report(vec![make_change(ChangeClass::C2, "src/test.ts")]);
        let passed = evaluate_gate(&mut report, &config);
        assert!(passed); // default fail_on is empty
        assert_eq!(report.policy_result.status, "warn"); // has changes but no fail rules
    }

    #[test]
    fn fail_on_c2_blocks() {
        let config = GateConfig {
            fail_on: vec![FailRule {
                when: "C2".to_string(),
                code: "ANRSM-001".to_string(),
            }],
            warn_on: vec![],
            systemic_threshold: 5,
        };
        let mut report = make_report(vec![make_change(ChangeClass::C2, "src/test.ts")]);
        let passed = evaluate_gate(&mut report, &config);
        assert!(!passed);
        assert_eq!(report.policy_result.status, "fail");
    }

    #[test]
    fn fail_on_c2_ignores_c0() {
        let config = GateConfig {
            fail_on: vec![FailRule {
                when: "C2".to_string(),
                code: "ANRSM-001".to_string(),
            }],
            warn_on: vec![],
            systemic_threshold: 5,
        };
        let mut report = make_report(vec![make_change(ChangeClass::C0, "src/test.ts")]);
        let passed = evaluate_gate(&mut report, &config);
        assert!(passed);
    }

    #[test]
    fn auto_codes_populated() {
        let config = GateConfig::default();
        let mut report = make_report(vec![
            make_change(ChangeClass::C2, "src/a.ts"),
            make_change(ChangeClass::C3, "src/b.ts"),
        ]);
        evaluate_gate(&mut report, &config);
        let codes = report.policy_result.failure_codes.as_ref().unwrap();
        assert!(codes.contains(&"ANRSM-001".to_string()));
        assert!(codes.contains(&"ANRSM-002".to_string()));
    }

    #[test]
    fn exit_code_mapping() {
        assert_eq!(get_exit_code(true), 0);
        assert_eq!(get_exit_code(false), 1);
    }
}
