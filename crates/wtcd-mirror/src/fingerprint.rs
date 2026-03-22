use sha2::{Digest, Sha256};
use std::path::Path;

use crate::types::FingerprintResult;
use wtcd_core::types::*;

// ─── Source Fingerprint ─────────────────────────────────────────────────────

/// Compute source fingerprint: SHA-256 of raw source bytes.
/// Any byte change produces a different hash — no normalization (D-05).
pub fn compute_source_fingerprint(source: &str) -> String {
    let hash = Sha256::digest(source.as_bytes());
    format!("sha256:{}", hex_encode(hash.as_slice()))
}

/// Compute source fingerprint from a file on disk.
pub fn compute_source_fingerprint_from_path(path: &Path) -> anyhow::Result<String> {
    let source = std::fs::read_to_string(path)?;
    Ok(compute_source_fingerprint(&source))
}

// ─── Semantic Fingerprint ───────────────────────────────────────────────────

/// Canonical export for deterministic serialization
#[derive(Serialize, Ord, PartialOrd, Eq, PartialEq)]
struct CanonicalExport {
    kind: String,
    name: String,
}

/// Canonical import for deterministic serialization
#[derive(Serialize, Ord, PartialOrd, Eq, PartialEq)]
struct CanonicalImport {
    kind: String,
    source: String,
    symbols: Vec<String>,
}

/// Canonical signature for deterministic serialization
#[derive(Serialize, Ord, PartialOrd, Eq, PartialEq)]
struct CanonicalSignature {
    name: String,
    parameters: Vec<CanonicalParameter>,
    return_type: String,
}

/// Canonical parameter
#[derive(Serialize, Ord, PartialOrd, Eq, PartialEq)]
struct CanonicalParameter {
    name: String,
    type_annotation: String,
}

/// Canonical side effect for deterministic serialization
#[derive(Serialize, Ord, PartialOrd, Eq, PartialEq)]
struct CanonicalSideEffect {
    kind: String,
    target: String,
}

/// Canonical semantic object — excludes line numbers, sorted collections (D-06)
#[derive(Serialize)]
struct CanonicalSemantic {
    confidence: String,
    exports: Vec<CanonicalExport>,
    imports: Vec<CanonicalImport>,
    side_effects: Vec<CanonicalSideEffect>,
    signatures: Vec<CanonicalSignature>,
}

use serde::Serialize;

fn confidence_to_string(band: &ConfidenceBand) -> String {
    match band {
        ConfidenceBand::High => "high".to_string(),
        ConfidenceBand::Low => "low".to_string(),
        ConfidenceBand::None => "none".to_string(),
    }
}

fn export_kind_to_string(kind: &ExportKind) -> String {
    match kind {
        ExportKind::Function => "function".to_string(),
        ExportKind::Class => "class".to_string(),
        ExportKind::Const => "const".to_string(),
        ExportKind::Let => "let".to_string(),
        ExportKind::Var => "var".to_string(),
        ExportKind::Type => "type".to_string(),
        ExportKind::Interface => "interface".to_string(),
        ExportKind::Enum => "enum".to_string(),
    }
}

fn import_kind_to_string(kind: &ImportKind) -> String {
    match kind {
        ImportKind::Named => "named".to_string(),
        ImportKind::Default => "default".to_string(),
        ImportKind::Namespace => "namespace".to_string(),
    }
}

fn side_effect_kind_to_string(kind: &SideEffectKind) -> String {
    match kind {
        SideEffectKind::Io => "io".to_string(),
        SideEffectKind::Network => "network".to_string(),
        SideEffectKind::Storage => "storage".to_string(),
        SideEffectKind::Log => "log".to_string(),
    }
}

/// Build canonical semantic object from FileResult (D-06 normalization)
fn build_canonical(result: &FileResult) -> CanonicalSemantic {
    // Exports: sorted by name, {name, kind} only (no line numbers)
    let mut exports: Vec<CanonicalExport> = result
        .exports
        .iter()
        .map(|e| CanonicalExport {
            name: e.name.clone(),
            kind: export_kind_to_string(&e.kind),
        })
        .collect();
    exports.sort();

    // Imports: sorted by source, symbols sorted within each import
    let mut imports: Vec<CanonicalImport> = result
        .imports
        .iter()
        .map(|i| {
            let mut symbols = i.symbols.clone();
            symbols.sort();
            CanonicalImport {
                source: i.source.clone(),
                symbols,
                kind: import_kind_to_string(&i.kind),
            }
        })
        .collect();
    imports.sort();

    // Signatures: sorted by name, parameters sorted by name
    let mut signatures: Vec<CanonicalSignature> = result
        .signatures
        .iter()
        .map(|s| {
            let mut params: Vec<CanonicalParameter> = s
                .parameters
                .iter()
                .map(|p| CanonicalParameter {
                    name: p.name.clone(),
                    type_annotation: p.type_annotation.clone(),
                })
                .collect();
            params.sort();
            CanonicalSignature {
                name: s.name.clone(),
                parameters: params,
                return_type: s.return_type.clone(),
            }
        })
        .collect();
    signatures.sort();

    // Side effects: sorted by kind then target (no line numbers)
    let mut side_effects: Vec<CanonicalSideEffect> = result
        .side_effects
        .iter()
        .map(|se| CanonicalSideEffect {
            kind: side_effect_kind_to_string(&se.kind),
            target: se.target.clone(),
        })
        .collect();
    side_effects.sort();

    CanonicalSemantic {
        confidence: confidence_to_string(&result.confidence),
        exports,
        imports,
        side_effects,
        signatures,
    }
}

/// Compute semantic fingerprint: canonical serialization → SHA-256 (D-07).
/// Stable across whitespace/comment/formatting changes (D-06).
pub fn compute_semantic_fingerprint(result: &FileResult) -> String {
    let canonical = build_canonical(result);
    let json = serde_json::to_string(&canonical).expect("canonical serialization failed");
    let hash = Sha256::digest(json.as_bytes());
    format!("sha256:{}", hex_encode(hash.as_slice()))
}

// ─── Public API ─────────────────────────────────────────────────────────────

/// Compute both source and semantic fingerprints (D-08: fp_version = "1").
pub fn compute_fingerprints(source: &str, result: &FileResult) -> FingerprintResult {
    FingerprintResult {
        source: compute_source_fingerprint(source),
        semantic: compute_semantic_fingerprint(result),
        fp_version: "1".to_string(),
    }
}

/// Compute fingerprints from a file on disk.
pub fn compute_fingerprints_from_path(
    path: &Path,
    result: &FileResult,
) -> anyhow::Result<FingerprintResult> {
    let source = std::fs::read_to_string(path)?;
    Ok(compute_fingerprints(&source, result))
}

// ─── Helpers ────────────────────────────────────────────────────────────────

fn hex_encode(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>()
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_file_result() -> FileResult {
        FileResult {
            file_path: "src/test.ts".to_string(),
            confidence: ConfidenceBand::High,
            exports: vec![
                ExportedSymbol {
                    name: "greet".to_string(),
                    kind: ExportKind::Function,
                    line: 1,
                    is_generated: false,
                    confidence: ConfidenceBand::High,
                },
                ExportedSymbol {
                    name: "VERSION".to_string(),
                    kind: ExportKind::Const,
                    line: 5,
                    is_generated: false,
                    confidence: ConfidenceBand::High,
                },
            ],
            imports: vec![DependencyEdge {
                source: "node:path".to_string(),
                symbols: vec!["join".to_string()],
                kind: ImportKind::Named,
            }],
            signatures: vec![FunctionSignature {
                name: "greet".to_string(),
                parameters: vec![Parameter {
                    name: "name".to_string(),
                    type_annotation: "string".to_string(),
                }],
                return_type: "string".to_string(),
            }],
            side_effects: vec![SideEffect {
                kind: SideEffectKind::Log,
                target: "console.log".to_string(),
                line: 3,
            }],
            parse_time_ms: 5,
            error_message: None,
        }
    }

    // ── Source fingerprint tests ──

    #[test]
    fn source_fingerprint_empty_string() {
        let fp = compute_source_fingerprint("");
        assert!(fp.starts_with("sha256:"));
        assert_eq!(fp.len(), 71); // "sha256:" (7) + 64 hex chars
    }

    #[test]
    fn source_fingerprint_deterministic() {
        let fp1 = compute_source_fingerprint("hello world");
        let fp2 = compute_source_fingerprint("hello world");
        assert_eq!(fp1, fp2);
    }

    #[test]
    fn source_fingerprint_single_char_change() {
        let fp1 = compute_source_fingerprint("hello world");
        let fp2 = compute_source_fingerprint("hello World");
        assert_ne!(fp1, fp2);
    }

    #[test]
    fn source_fingerprint_prefix_format() {
        let fp = compute_source_fingerprint("test");
        assert!(fp.starts_with("sha256:"));
        let hex_part = &fp[7..];
        assert_eq!(hex_part.len(), 64);
        assert!(hex_part.chars().all(|c| c.is_ascii_hexdigit()));
    }

    // ── Semantic fingerprint tests ──

    #[test]
    fn semantic_fingerprint_deterministic() {
        let result = sample_file_result();
        let fp1 = compute_semantic_fingerprint(&result);
        let fp2 = compute_semantic_fingerprint(&result);
        assert_eq!(fp1, fp2);
    }

    #[test]
    fn semantic_fingerprint_different_exports() {
        let mut result = sample_file_result();
        let fp1 = compute_semantic_fingerprint(&result);
        result.exports.push(ExportedSymbol {
            name: "newFunc".to_string(),
            kind: ExportKind::Function,
            line: 10,
            is_generated: false,
            confidence: ConfidenceBand::High,
        });
        let fp2 = compute_semantic_fingerprint(&result);
        assert_ne!(fp1, fp2);
    }

    #[test]
    fn semantic_fingerprint_stable_on_line_number_changes() {
        let result1 = sample_file_result();
        let mut result2 = sample_file_result();
        // Change line numbers — should NOT affect semantic fingerprint
        result2.exports[0].line = 99;
        result2.exports[1].line = 100;
        result2.side_effects[0].line = 200;
        let fp1 = compute_semantic_fingerprint(&result1);
        let fp2 = compute_semantic_fingerprint(&result2);
        assert_eq!(fp1, fp2);
    }

    #[test]
    fn semantic_fingerprint_stable_on_import_reorder() {
        let result1 = FileResult {
            file_path: "test.ts".to_string(),
            confidence: ConfidenceBand::High,
            exports: vec![],
            imports: vec![
                DependencyEdge {
                    source: "z-module".to_string(),
                    symbols: vec!["a".to_string(), "b".to_string()],
                    kind: ImportKind::Named,
                },
                DependencyEdge {
                    source: "a-module".to_string(),
                    symbols: vec!["z".to_string(), "y".to_string()],
                    kind: ImportKind::Named,
                },
            ],
            signatures: vec![],
            side_effects: vec![],
            parse_time_ms: 0,
            error_message: None,
        };
        let mut result2 = result1.clone();
        // Swap order of imports
        result2.imports.swap(0, 1);
        // Also reorder symbols within
        result2.imports[1].symbols.swap(0, 1);
        let fp1 = compute_semantic_fingerprint(&result1);
        let fp2 = compute_semantic_fingerprint(&result2);
        assert_eq!(fp1, fp2);
    }

    #[test]
    fn semantic_fingerprint_different_confidence() {
        let mut result1 = sample_file_result();
        result1.confidence = ConfidenceBand::High;
        let mut result2 = sample_file_result();
        result2.confidence = ConfidenceBand::Low;
        let fp1 = compute_semantic_fingerprint(&result1);
        let fp2 = compute_semantic_fingerprint(&result2);
        assert_ne!(fp1, fp2);
    }

    #[test]
    fn semantic_fingerprint_prefix_format() {
        let result = sample_file_result();
        let fp = compute_semantic_fingerprint(&result);
        assert!(fp.starts_with("sha256:"));
        let hex_part = &fp[7..];
        assert_eq!(hex_part.len(), 64);
        assert!(hex_part.chars().all(|c| c.is_ascii_hexdigit()));
    }

    // ── Public API tests ──

    #[test]
    fn compute_fingerprints_returns_correct_fp_version() {
        let result = sample_file_result();
        let fp = compute_fingerprints("test source", &result);
        assert_eq!(fp.fp_version, "1");
        assert!(fp.source.starts_with("sha256:"));
        assert!(fp.semantic.starts_with("sha256:"));
    }

    #[test]
    fn compute_fingerprints_whitespace_changes() {
        let result = sample_file_result();
        let fp1 = compute_fingerprints("function greet(name: string) { return name; }", &result);
        let fp2 = compute_fingerprints(
            "function  greet(  name:  string )\n{  return  name;  }",
            &result,
        );
        // Source fingerprint changes (whitespace differs)
        assert_ne!(fp1.source, fp2.source);
        // Semantic fingerprint stays same (same FileResult)
        assert_eq!(fp1.semantic, fp2.semantic);
    }

    #[test]
    fn compute_fingerprints_export_changes() {
        let result1 = sample_file_result();
        let result2 = {
            let mut r = sample_file_result();
            r.exports.push(ExportedSymbol {
                name: "newExport".to_string(),
                kind: ExportKind::Function,
                line: 10,
                is_generated: false,
                confidence: ConfidenceBand::High,
            });
            r
        };
        let fp1 = compute_fingerprints("test", &result1);
        let fp2 = compute_fingerprints("test", &result2);
        // Source stays same
        assert_eq!(fp1.source, fp2.source);
        // Semantic changes
        assert_ne!(fp1.semantic, fp2.semantic);
    }
}
