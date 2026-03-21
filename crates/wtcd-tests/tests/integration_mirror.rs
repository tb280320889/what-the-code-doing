use std::fs;
use tempfile::TempDir;
use wtcd_adapters::register_all_adapters;
use wtcd_core::types::*;
use wtcd_mirror::{fingerprint, io, template};

/// Helper: parse a fixture file and return FileResult
fn parse_fixture(relative_path: &str) -> FileResult {
    let registry = register_all_adapters().unwrap();
    let fixture_path = format!(
        "{}/../../tests/fixtures/{}",
        env!("CARGO_MANIFEST_DIR"),
        relative_path
    );
    let source = fs::read_to_string(&fixture_path).unwrap();
    let adapter = registry.find_adapter(relative_path).unwrap();
    adapter.parse(&source, relative_path)
}

#[test]
fn mirror_generation_creates_files() {
    let dir = TempDir::new().unwrap();
    let mirror_root = dir.path().join("mirror/file");

    let result = parse_fixture("ts/basic_exports.ts");
    let source = fs::read_to_string(format!(
        "{}/../../tests/fixtures/ts/basic_exports.ts",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();

    let write_result = io::generate_and_write_mirror(
        "ts/basic_exports.ts",
        &source,
        &result,
        "test1234",
        "ts",
        &mirror_root.to_string_lossy(),
        false,
    )
    .unwrap();

    // Verify mirror file exists
    let mirror_path = std::path::Path::new(&write_result.mirror_path);
    assert!(mirror_path.exists(), "Mirror file should exist");

    // Read and verify content
    let content = fs::read_to_string(mirror_path).unwrap();
    assert!(content.contains("---"), "Should have YAML frontmatter");
    assert!(
        content.contains("artifact_type: file_mirror"),
        "Should contain artifact_type"
    );
    assert!(
        content.contains("source_fingerprint: sha256:"),
        "Should contain source fingerprint"
    );
    assert!(
        content.contains("semantic_fingerprint: sha256:"),
        "Should contain semantic fingerprint"
    );
    assert!(content.contains("## 职责"), "Should have 职责 section");
    assert!(
        content.contains("## 对外契约"),
        "Should have 对外契约 section"
    );
    assert!(
        content.contains("## 输入与前置条件"),
        "Should have 前置条件 section"
    );
    assert!(
        content.contains("## 状态与控制流"),
        "Should have 状态与控制流 section"
    );
    assert!(
        content.contains("## 副作用与外部依赖"),
        "Should have 副作用 section"
    );
    assert!(
        content.contains("## 关键不变量与风险"),
        "Should have 不变量 section"
    );
    assert!(
        content.contains("## 变更影响"),
        "Should have 变更影响 section"
    );
    assert!(
        content.contains("## 何时必须展开源码"),
        "Should have 展开条件 section"
    );
}

#[test]
fn mirror_header_fields_correct() {
    let dir = TempDir::new().unwrap();
    let mirror_root = dir.path().join("mirror/file");

    let result = parse_fixture("ts/basic_exports.ts");
    let source = fs::read_to_string(format!(
        "{}/../../tests/fixtures/ts/basic_exports.ts",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();

    let write_result = io::generate_and_write_mirror(
        "ts/basic_exports.ts",
        &source,
        &result,
        "abc1234",
        "ts",
        &mirror_root.to_string_lossy(),
        false,
    )
    .unwrap();

    let header = io::read_mirror_header(std::path::Path::new(&write_result.mirror_path)).unwrap();

    assert_eq!(header.anrsm_version, 1);
    assert_eq!(header.artifact_type, "file_mirror");
    assert_eq!(header.source_path, "ts/basic_exports.ts");
    assert_eq!(header.source_language, "typescript");
    assert_eq!(header.module_id, "ts");
    assert_eq!(header.source_commit, "abc1234");
    assert_eq!(header.freshness_state, wtcd_mirror::FreshnessState::Fresh);
    assert_eq!(
        header.confidence_band,
        wtcd_mirror::MirrorConfidenceBand::High
    );
    assert_eq!(header.generator_name, "anrsm");
    assert_eq!(header.generator_version, "0.1.0");
}

#[test]
fn mirror_orphan_cleanup() {
    let dir = TempDir::new().unwrap();
    let mirror_root = dir.path().join("mirror/file");

    // Create an orphan mirror file (no corresponding source)
    fs::create_dir_all(mirror_root.join("src")).unwrap();
    fs::write(
        mirror_root.join("src/deleted.ts.md"),
        "---\nanrsm_version: 1\n---\n## 职责\nold\n",
    )
    .unwrap();

    // Also create a valid mirror for an existing source
    fs::create_dir_all(mirror_root.join("ts")).unwrap();
    fs::write(
        mirror_root.join("ts/basic_exports.ts.md"),
        "---\nanrsm_version: 1\n---\n## 职责\nvalid\n",
    )
    .unwrap();

    // Only ts/basic_exports.ts exists as a source
    let source_paths = vec!["ts/basic_exports.ts".to_string()];
    let orphans = io::find_orphan_mirrors(&source_paths, &mirror_root).unwrap();

    assert_eq!(orphans.len(), 1, "Should find 1 orphan");
    assert!(
        orphans[0].to_string_lossy().contains("deleted.ts.md"),
        "Should be the deleted.ts mirror"
    );

    let count = io::delete_orphans(&orphans).unwrap();
    assert_eq!(count, 1, "Should delete 1 orphan");
    assert!(!mirror_root.join("src/deleted.ts.md").exists());
    assert!(mirror_root.join("ts/basic_exports.ts.md").exists());
}

#[test]
fn fingerprint_stability_across_runs() {
    let result = parse_fixture("ts/basic_exports.ts");
    let source = fs::read_to_string(format!(
        "{}/../../tests/fixtures/ts/basic_exports.ts",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();

    let fp1 = fingerprint::compute_fingerprints(&source, &result);
    let fp2 = fingerprint::compute_fingerprints(&source, &result);

    assert_eq!(
        fp1.source, fp2.source,
        "Source fingerprint should be stable"
    );
    assert_eq!(
        fp1.semantic, fp2.semantic,
        "Semantic fingerprint should be stable"
    );
    assert_eq!(fp1.fp_version, "1", "fp_version should be '1'");
}

#[test]
fn mirror_appendix_preservation() {
    let dir = TempDir::new().unwrap();
    let mirror_root = dir.path().join("mirror/file");

    let result = parse_fixture("ts/basic_exports.ts");
    let source = fs::read_to_string(format!(
        "{}/../../tests/fixtures/ts/basic_exports.ts",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();

    // First write
    let wr1 = io::generate_and_write_mirror(
        "ts/basic_exports.ts",
        &source,
        &result,
        "abc1234",
        "ts",
        &mirror_root.to_string_lossy(),
        true,
    )
    .unwrap();
    assert!(!wr1.appendix_preserved);

    // Manually add appendix to the mirror
    let mirror_path = std::path::Path::new(&wr1.mirror_path);
    let mut content = fs::read_to_string(mirror_path).unwrap();
    content.push_str(
        "\n<!-- manual_appendix_start -->\nUser notes here\n<!-- manual_appendix_end -->\n",
    );
    fs::write(mirror_path, &content).unwrap();

    // Second write with appendix preservation
    let wr2 = io::generate_and_write_mirror(
        "ts/basic_exports.ts",
        &source,
        &result,
        "abc1234",
        "ts",
        &mirror_root.to_string_lossy(),
        true,
    )
    .unwrap();
    assert!(wr2.appendix_preserved);

    // Verify appendix content preserved
    let final_content = fs::read_to_string(mirror_path).unwrap();
    assert!(
        final_content.contains("User notes here"),
        "Appendix content should be preserved"
    );
}

#[test]
fn mirror_body_exports_mentioned() {
    let result = parse_fixture("ts/basic_exports.ts");
    let body = template::generate_body(&result);

    // basic_exports.ts should have greet, UserService, MAX_RETRIES
    assert!(
        body.responsibilities.contains("greet"),
        "Should mention greet export"
    );
    assert!(
        body.responsibilities.contains("UserService"),
        "Should mention UserService export"
    );
}
