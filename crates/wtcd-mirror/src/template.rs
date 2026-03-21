use std::time::SystemTime;

use anyhow::Result;
use chrono::Utc;

use crate::types::*;
use wtcd_core::types::*;

// ─── Path Mapping (MIRR-03) ─────────────────────────────────────────────────

/// Maps source path to mirror path: mirror/file/<source_relative_path>.md
pub fn source_to_mirror_path(source_path: &str, mirror_root: &str) -> String {
    format!("{}/{}.md", mirror_root, source_path)
}

/// Maps mirror path back to source path
pub fn mirror_to_source_path(mirror_path: &str, mirror_root: &str) -> Option<String> {
    let prefix = format!("{}/", mirror_root);
    let suffix = ".md";

    if !mirror_path.starts_with(&prefix) || !mirror_path.ends_with(suffix) {
        return None;
    }

    let start = prefix.len();
    let end = mirror_path.len() - suffix.len();
    if start >= end {
        return None;
    }

    Some(mirror_path[start..end].to_string())
}

// ─── YAML Frontmatter Generation ────────────────────────────────────────────

fn derive_language(file_path: &str) -> String {
    let ext = std::path::Path::new(file_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    match ext {
        "ts" | "tsx" => "typescript",
        "js" | "jsx" => "javascript",
        "py" => "python",
        "rs" => "rust",
        "go" => "go",
        _ => "unknown",
    }
    .to_string()
}

fn derive_module_id(file_path: &str, source_roots: &[String]) -> String {
    let path = std::path::Path::new(file_path);
    let components: Vec<_> = path
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .collect();

    if components.is_empty() {
        return "global".to_string();
    }

    // If first component matches a source root, use second component
    if source_roots.iter().any(|r| r == &components[0]) {
        if components.len() > 1 {
            return components[1].clone();
        }
    }

    // Fallback: use first component
    components[0].clone()
}

fn derive_risk_tags(side_effects: &[SideEffect]) -> Vec<String> {
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

/// Generate a MirrorHeader from FileResult + fingerprints (D-01..D-04)
pub fn generate_header(
    file_result: &FileResult,
    fingerprints: &FingerprintResult,
    source_commit: &str,
    module_id: &str,
) -> MirrorHeader {
    let export_names: Vec<String> = file_result.exports.iter().map(|e| e.name.clone()).collect();
    let dep_sources: Vec<String> = file_result
        .imports
        .iter()
        .map(|i| i.source.clone())
        .collect();
    let risk_tags = derive_risk_tags(&file_result.side_effects);

    MirrorHeader {
        anrsm_version: 1,
        artifact_type: "file_mirror".to_string(),
        artifact_id: format!("file_mirror:{}", file_result.file_path),
        source_path: file_result.file_path.clone(),
        source_language: derive_language(&file_result.file_path),
        module_id: module_id.to_string(),
        source_commit: source_commit.to_string(),
        source_fingerprint: fingerprints.source.clone(),
        semantic_fingerprint: fingerprints.semantic.clone(),
        freshness_state: FreshnessState::Fresh,
        confidence_band: match file_result.confidence {
            ConfidenceBand::High => MirrorConfidenceBand::High,
            ConfidenceBand::Low => MirrorConfidenceBand::Low,
            ConfidenceBand::None => MirrorConfidenceBand::Low,
        },
        generator_name: "anrsm".to_string(),
        generator_version: "0.1.0".to_string(),
        generated_at: Utc::now().to_rfc3339(),
        exports: if export_names.is_empty() {
            None
        } else {
            Some(export_names)
        },
        dependencies: if dep_sources.is_empty() {
            None
        } else {
            Some(dep_sources)
        },
        risk_tags: if risk_tags.is_empty() {
            None
        } else {
            Some(risk_tags)
        },
        contracts: None,
        adjacent_artifacts: None,
        manual_appendix_present: None,
    }
}

/// Serialize header to YAML string with frontmatter delimiters
pub fn header_to_yaml(header: &MirrorHeader) -> Result<String> {
    let yaml = yaml_serde::to_string(header)
        .map_err(|e| anyhow::anyhow!("YAML serialization failed: {}", e))?;
    Ok(format!("---\n{}\n---", yaml))
}

// ─── Mirror Body Generation (8 sections) ────────────────────────────────────

/// Generate 8-section body from FileResult (D-01, D-02, D-03)
pub fn generate_body(file_result: &FileResult) -> MirrorBody {
    MirrorBody {
        responsibilities: generate_responsibilities(file_result),
        external_contracts: generate_external_contracts(file_result),
        preconditions: generate_preconditions(file_result),
        state_control_flow: generate_state_control_flow(file_result),
        side_effects: generate_side_effects_section(file_result),
        invariants_risks: generate_invariants_risks(file_result),
        change_impact: generate_change_impact(file_result),
        expand_conditions: generate_expand_conditions(file_result),
    }
}

fn generate_responsibilities(result: &FileResult) -> String {
    if result.exports.is_empty() {
        return "内部实现模块，不对外暴露符号。".to_string();
    }

    let names: Vec<&str> = result.exports.iter().map(|e| e.name.as_str()).collect();
    let file_hint = std::path::Path::new(&result.file_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("模块");

    format!("暴露 {}；{} 相关逻辑。", names.join("、"), file_hint)
}

fn generate_external_contracts(result: &FileResult) -> String {
    if result.signatures.is_empty() {
        return "无显式函数契约。".to_string();
    }

    let contracts: Vec<String> = result
        .signatures
        .iter()
        .map(|s| {
            let params: Vec<String> = s
                .parameters
                .iter()
                .map(|p| format!("{}: {}", p.name, p.type_annotation))
                .collect();
            format!(
                "对外暴露 {}({}) → {}",
                s.name,
                params.join(", "),
                s.return_type
            )
        })
        .collect();

    contracts.join("；\n")
}

fn generate_preconditions(result: &FileResult) -> String {
    if result.imports.is_empty() {
        return "无外部依赖。".to_string();
    }

    let sources: Vec<&str> = result.imports.iter().map(|i| i.source.as_str()).collect();
    format!("依赖 {} 提供的符号和类型。", sources.join("、"))
}

fn generate_state_control_flow(result: &FileResult) -> String {
    if result.signatures.is_empty() {
        let conf_note = if result.confidence == ConfidenceBand::Low {
            "（解析置信度低，可能遗漏分支。）"
        } else {
            ""
        };
        return format!("无可提取的控制流信息。{}", conf_note);
    }

    let flows: Vec<String> = result
        .signatures
        .iter()
        .map(|s| {
            let effects_desc: Vec<String> = result
                .side_effects
                .iter()
                .map(|se| match se.kind {
                    SideEffectKind::Io => "文件I/O".to_string(),
                    SideEffectKind::Network => "网络请求".to_string(),
                    SideEffectKind::Storage => "存储操作".to_string(),
                    SideEffectKind::Log => "日志记录".to_string(),
                })
                .collect();

            if effects_desc.is_empty() {
                format!("{}: 纯函数逻辑。", s.name)
            } else {
                format!("{}: 包含 {}。", s.name, effects_desc.join("、"))
            }
        })
        .collect();

    let conf_note = if result.confidence == ConfidenceBand::Low {
        "（解析置信度低，可能遗漏分支。）"
    } else {
        ""
    };

    format!("{}{}", flows.join("；\n"), conf_note)
}

fn generate_side_effects_section(result: &FileResult) -> String {
    if result.side_effects.is_empty() {
        return "无检测到的副作用。".to_string();
    }

    let effects: Vec<String> = result
        .side_effects
        .iter()
        .map(|se| {
            let kind_str = match se.kind {
                SideEffectKind::Io => "文件I/O",
                SideEffectKind::Network => "网络请求",
                SideEffectKind::Storage => "存储操作",
                SideEffectKind::Log => "日志记录",
            };
            format!("{}: {} (行 {})", kind_str, se.target, se.line)
        })
        .collect();

    effects.join("；\n")
}

fn generate_invariants_risks(result: &FileResult) -> String {
    let mut risks = Vec::new();

    let has_network = result
        .side_effects
        .iter()
        .any(|se| se.kind == SideEffectKind::Network);
    let has_storage = result
        .side_effects
        .iter()
        .any(|se| se.kind == SideEffectKind::Storage);
    let has_exports = !result.exports.is_empty();
    let has_effects = !result.side_effects.is_empty();

    if has_network || has_storage {
        risks.push("涉及外部系统调用，需关注错误处理和幂等性。");
    }
    if has_exports && has_effects {
        risks.push("导出符号包含副作用调用方，调用方需了解副作用边界。");
    }
    if result.confidence == ConfidenceBand::Low {
        risks.push("解析置信度低，不变量可能不完整。");
    }

    if risks.is_empty() {
        return "未发现特定风险。".to_string();
    }

    risks.join("\n")
}

fn generate_change_impact(result: &FileResult) -> String {
    if result.imports.is_empty() {
        return "无已知的依赖关系影响分析。".to_string();
    }

    let sources: Vec<&str> = result.imports.iter().map(|i| i.source.as_str()).collect();
    format!("变更可能受上游 {} 的影响。", sources.join("、"))
}

fn generate_expand_conditions(result: &FileResult) -> String {
    let mut conditions = Vec::new();

    if !result.side_effects.is_empty() {
        conditions.push("修改副作用处理逻辑时必须展开源码。");
    }
    if !result.signatures.is_empty() {
        conditions.push("修改函数签名或返回类型时必须展开源码。");
    }
    if result.confidence == ConfidenceBand::Low {
        conditions.push("解析置信度低，关键决策前必须展开源码。");
    }
    conditions.push("涉及错误处理、并发控制或状态管理时必须展开源码。");

    conditions.join("\n")
}

// ─── Body Serialization ─────────────────────────────────────────────────────

/// Serialize MirrorBody to markdown string
pub fn body_to_markdown(body: &MirrorBody) -> String {
    format!(
        "\
## 职责
{}

## 对外契约
{}

## 输入与前置条件
{}

## 状态与控制流
{}

## 副作用与外部依赖
{}

## 关键不变量与风险
{}

## 变更影响
{}

## 何时必须展开源码
{}",
        body.responsibilities,
        body.external_contracts,
        body.preconditions,
        body.state_control_flow,
        body.side_effects,
        body.invariants_risks,
        body.change_impact,
        body.expand_conditions,
    )
}

// ─── Full Mirror Assembly ───────────────────────────────────────────────────

/// Generate complete MirrorFile (header + body)
pub fn generate_mirror(
    file_result: &FileResult,
    fingerprints: &FingerprintResult,
    source_commit: &str,
    module_id: &str,
) -> MirrorFile {
    MirrorFile {
        header: generate_header(file_result, fingerprints, source_commit, module_id),
        body: generate_body(file_result),
    }
}

/// Serialize complete mirror to string
pub fn mirror_to_string(mirror: &MirrorFile) -> String {
    let yaml = header_to_yaml(&mirror.header).expect("header YAML failed");
    let body = body_to_markdown(&mirror.body);
    format!("{}\n\n{}\n", yaml, body)
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_file_result() -> FileResult {
        FileResult {
            file_path: "src/auth/login.ts".to_string(),
            confidence: ConfidenceBand::High,
            exports: vec![
                ExportedSymbol {
                    name: "login".to_string(),
                    kind: ExportKind::Function,
                    line: 10,
                },
                ExportedSymbol {
                    name: "logout".to_string(),
                    kind: ExportKind::Function,
                    line: 25,
                },
            ],
            imports: vec![
                DependencyEdge {
                    source: "src/core/http/client.ts".to_string(),
                    symbols: vec!["fetch".to_string()],
                    kind: ImportKind::Named,
                },
                DependencyEdge {
                    source: "src/core/auth/session.ts".to_string(),
                    symbols: vec!["createSession".to_string()],
                    kind: ImportKind::Named,
                },
            ],
            signatures: vec![FunctionSignature {
                name: "login".to_string(),
                parameters: vec![Parameter {
                    name: "credentials".to_string(),
                    type_annotation: "Credentials".to_string(),
                }],
                return_type: "Promise<Session>".to_string(),
            }],
            side_effects: vec![
                SideEffect {
                    kind: SideEffectKind::Network,
                    target: "auth service".to_string(),
                    line: 15,
                },
                SideEffect {
                    kind: SideEffectKind::Log,
                    target: "console".to_string(),
                    line: 20,
                },
            ],
            parse_time_ms: 5,
            error_message: None,
        }
    }

    fn sample_fingerprints() -> FingerprintResult {
        FingerprintResult {
            source: "sha256:aabbccdd".to_string(),
            semantic: "sha256:eeff0011".to_string(),
            fp_version: "1".to_string(),
        }
    }

    // ── Path mapping tests ──

    #[test]
    fn path_map_basic() {
        assert_eq!(
            source_to_mirror_path("src/auth/login.ts", "mirror/file"),
            "mirror/file/src/auth/login.ts.md"
        );
    }

    #[test]
    fn path_map_js() {
        assert_eq!(
            source_to_mirror_path("lib/utils/helpers.js", "mirror/file"),
            "mirror/file/lib/utils/helpers.js.md"
        );
    }

    #[test]
    fn path_map_round_trip() {
        let source = "src/components/Button.tsx";
        let mirror = source_to_mirror_path(source, "mirror/file");
        let back = mirror_to_source_path(&mirror, "mirror/file");
        assert_eq!(back, Some(source.to_string()));
    }

    #[test]
    fn path_map_no_extension() {
        assert_eq!(
            source_to_mirror_path("Makefile", "mirror/file"),
            "mirror/file/Makefile.md"
        );
    }

    #[test]
    fn mirror_to_source_invalid() {
        assert_eq!(
            mirror_to_source_path("other/path.ts.md", "mirror/file"),
            None
        );
        assert_eq!(mirror_to_source_path("mirror/file/", "mirror/file"), None);
    }

    // ── Header tests ──

    #[test]
    fn header_derives_fields_correctly() {
        let result = sample_file_result();
        let fp = sample_fingerprints();
        let header = generate_header(&result, &fp, "abc1234", "auth");

        assert_eq!(header.anrsm_version, 1);
        assert_eq!(header.artifact_type, "file_mirror");
        assert_eq!(header.artifact_id, "file_mirror:src/auth/login.ts");
        assert_eq!(header.source_path, "src/auth/login.ts");
        assert_eq!(header.source_language, "typescript");
        assert_eq!(header.module_id, "auth");
        assert_eq!(header.source_commit, "abc1234");
        assert_eq!(header.source_fingerprint, "sha256:aabbccdd");
        assert_eq!(header.semantic_fingerprint, "sha256:eeff0011");
        assert_eq!(header.freshness_state, FreshnessState::Fresh);
        assert_eq!(header.confidence_band, MirrorConfidenceBand::High);
        assert_eq!(header.generator_name, "anrsm");
        assert_eq!(header.generator_version, "0.1.0");
        assert!(header.generated_at.contains("T")); // RFC 3339
        assert_eq!(
            header.exports,
            Some(vec!["login".to_string(), "logout".to_string()])
        );
        assert_eq!(
            header.dependencies,
            Some(vec![
                "src/core/http/client.ts".to_string(),
                "src/core/auth/session.ts".to_string()
            ])
        );
        assert_eq!(
            header.risk_tags,
            Some(vec!["external_api".to_string(), "logging".to_string()])
        );
    }

    #[test]
    fn header_confidence_mapping() {
        let mut result = sample_file_result();

        result.confidence = ConfidenceBand::High;
        let header = generate_header(&result, &sample_fingerprints(), "x", "m");
        assert_eq!(header.confidence_band, MirrorConfidenceBand::High);

        result.confidence = ConfidenceBand::Low;
        let header = generate_header(&result, &sample_fingerprints(), "x", "m");
        assert_eq!(header.confidence_band, MirrorConfidenceBand::Low);

        result.confidence = ConfidenceBand::None;
        let header = generate_header(&result, &sample_fingerprints(), "x", "m");
        assert_eq!(header.confidence_band, MirrorConfidenceBand::Low);
    }

    #[test]
    fn yaml_serialization_parseable() {
        let result = sample_file_result();
        let header = generate_header(&result, &sample_fingerprints(), "abc1234", "auth");
        let yaml = header_to_yaml(&header).unwrap();
        assert!(yaml.starts_with("---\n"));
        assert!(yaml.ends_with("\n---"));
        assert!(yaml.contains("artifact_type: file_mirror"));
        assert!(yaml.contains("source_fingerprint: sha256:aabbccdd"));
    }

    // ── Body tests ──

    #[test]
    fn body_has_eight_sections() {
        let result = sample_file_result();
        let body = generate_body(&result);
        assert!(!body.responsibilities.is_empty());
        assert!(!body.external_contracts.is_empty());
        assert!(!body.preconditions.is_empty());
        assert!(!body.state_control_flow.is_empty());
        assert!(!body.side_effects.is_empty());
        assert!(!body.invariants_risks.is_empty());
        assert!(!body.change_impact.is_empty());
        assert!(!body.expand_conditions.is_empty());
    }

    #[test]
    fn body_exports_in_responsibilities() {
        let result = sample_file_result();
        let body = generate_body(&result);
        assert!(body.responsibilities.contains("login"));
        assert!(body.responsibilities.contains("logout"));
    }

    #[test]
    fn body_side_effects_listed() {
        let result = sample_file_result();
        let body = generate_body(&result);
        assert!(body.side_effects.contains("网络请求"));
        assert!(body.side_effects.contains("auth service"));
        assert!(body.side_effects.contains("日志记录"));
    }

    #[test]
    fn body_imports_in_preconditions() {
        let result = sample_file_result();
        let body = generate_body(&result);
        assert!(body.preconditions.contains("src/core/http/client.ts"));
        assert!(body.preconditions.contains("src/core/auth/session.ts"));
    }

    #[test]
    fn body_empty_file_no_panic() {
        let result = FileResult {
            file_path: "empty.ts".to_string(),
            confidence: ConfidenceBand::None,
            exports: vec![],
            imports: vec![],
            signatures: vec![],
            side_effects: vec![],
            parse_time_ms: 0,
            error_message: None,
        };
        let body = generate_body(&result);
        assert!(body.responsibilities.contains("不对外暴露"));
        assert!(body.external_contracts.contains("无显式"));
        assert!(body.preconditions.contains("无外部依赖"));
        assert!(body.side_effects.contains("无检测到的副作用"));
    }

    // ── Section header test ──

    #[test]
    fn body_markdown_has_correct_section_headers() {
        let result = sample_file_result();
        let body = generate_body(&result);
        let md = body_to_markdown(&body);
        assert!(md.contains("## 职责"));
        assert!(md.contains("## 对外契约"));
        assert!(md.contains("## 输入与前置条件"));
        assert!(md.contains("## 状态与控制流"));
        assert!(md.contains("## 副作用与外部依赖"));
        assert!(md.contains("## 关键不变量与风险"));
        assert!(md.contains("## 变更影响"));
        assert!(md.contains("## 何时必须展开源码"));
    }

    // ── Full assembly test ──

    #[test]
    fn full_mirror_assembly() {
        let result = sample_file_result();
        let fp = sample_fingerprints();
        let mirror = generate_mirror(&result, &fp, "abc1234", "auth");

        let full = mirror_to_string(&mirror);
        assert!(full.contains("---"));
        assert!(full.contains("artifact_type: file_mirror"));
        assert!(full.contains("## 职责"));
        assert!(full.contains("## 对外契约"));
        assert!(full.contains("## 输入与前置条件"));
        assert!(full.contains("## 状态与控制流"));
        assert!(full.contains("## 副作用与外部依赖"));
        assert!(full.contains("## 关键不变量与风险"));
        assert!(full.contains("## 变更影响"));
        assert!(full.contains("## 何时必须展开源码"));
    }

    #[test]
    fn full_mirror_deterministic() {
        let result = sample_file_result();
        let fp = sample_fingerprints();
        let mirror1 = generate_mirror(&result, &fp, "abc1234", "auth");
        let mirror2 = generate_mirror(&result, &fp, "abc1234", "auth");

        // Body should be deterministic
        assert_eq!(
            body_to_markdown(&mirror1.body),
            body_to_markdown(&mirror2.body)
        );
    }
}
