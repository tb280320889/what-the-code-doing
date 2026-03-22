use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::fingerprint;
use crate::template;
use crate::types::*;
use wtcd_core::types::*;

// ─── Constants ──────────────────────────────────────────────────────────────

const APPENDIX_START: &str = "<!-- manual_appendix_start -->";
const APPENDIX_END: &str = "<!-- manual_appendix_end -->";

// ─── AppendixResult ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct AppendixResult {
    pub appendix_preserved: bool,
    pub appendix_content: Option<String>,
}

// ─── MirrorWriteResult ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct MirrorWriteResult {
    pub mirror_path: String,
    pub source_fingerprint: String,
    pub semantic_fingerprint: String,
    pub appendix_preserved: bool,
}

// ─── Writing ────────────────────────────────────────────────────────────────

/// Write content to disk, creating parent directories as needed.
pub fn write_mirror(mirror_path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = mirror_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(mirror_path, content)?;
    Ok(())
}

/// Extract content between manual_appendix_start and manual_appendix_end markers.
pub fn extract_appendix(content: &str) -> Option<String> {
    let start_idx = content.find(APPENDIX_START)?;
    let end_idx = content.find(APPENDIX_END)?;

    if end_idx <= start_idx {
        return None;
    }

    let appendix_start = start_idx + APPENDIX_START.len();
    let extracted = content[appendix_start..end_idx].trim();

    if extracted.is_empty() {
        return None;
    }

    Some(extracted.to_string())
}

/// Build the appendix section for injection into mirror content.
fn build_appendix_section(appendix_content: &str) -> String {
    format!(
        "\n\n{}\n{}\n{}\n",
        APPENDIX_START, appendix_content, APPENDIX_END
    )
}

/// Write mirror content, preserving any existing manual_appendix (D-10).
pub fn write_mirror_with_appendix(
    mirror_path: &Path,
    new_content: &str,
    preserve_appendix: bool,
) -> Result<AppendixResult> {
    let appendix_content = if preserve_appendix && mirror_path.exists() {
        let existing = fs::read_to_string(mirror_path)?;
        extract_appendix(&existing)
    } else {
        None
    };

    let final_content = if let Some(ref appendix) = appendix_content {
        // Strip any existing appendix markers from new_content before appending
        let stripped = strip_appendix_markers(new_content);
        format!("{}{}", stripped, build_appendix_section(appendix))
    } else {
        new_content.to_string()
    };

    write_mirror(mirror_path, &final_content)?;

    Ok(AppendixResult {
        appendix_preserved: appendix_content.is_some(),
        appendix_content,
    })
}

/// Remove appendix markers and content from a string.
fn strip_appendix_markers(content: &str) -> String {
    if let Some(start_idx) = content.find(APPENDIX_START) {
        let end_idx = content.find(APPENDIX_END);
        if let Some(end) = end_idx {
            let after_end = end + APPENDIX_END.len();
            let mut result = content[..start_idx].to_string();
            if after_end < content.len() {
                result.push_str(&content[after_end..]);
            }
            return result.trim_end().to_string();
        }
    }
    content.to_string()
}

// ─── Reading ────────────────────────────────────────────────────────────────

/// Read an existing mirror file and parse its YAML frontmatter header.
pub fn read_mirror_header(mirror_path: &Path) -> Option<MirrorHeader> {
    let content = fs::read_to_string(mirror_path).ok()?;

    // Parse YAML between first two --- delimiters
    let parts: Vec<&str> = content.splitn(3, "---").collect();
    if parts.len() < 3 {
        return None;
    }

    let yaml_content = parts[1];
    yaml_serde::from_str(yaml_content).ok()
}

// ─── Orphan Cleanup (D-12) ──────────────────────────────────────────────────

/// Find mirror files that have no corresponding source file.
pub fn find_orphan_mirrors(source_paths: &[String], mirror_root: &Path) -> Result<Vec<PathBuf>> {
    let mut orphans = Vec::new();

    if !mirror_root.exists() {
        return Ok(orphans);
    }

    walk_md_files(mirror_root, mirror_root, &mut |relative_md| {
        let mirror_path_str = format!("{}/{}", mirror_root.display(), relative_md);

        if let Some(source_path) =
            template::mirror_to_source_path(&mirror_path_str, &mirror_root.display().to_string())
        {
            if !source_paths.contains(&source_path) {
                orphans.push(mirror_root.join(relative_md));
            }
        }
    })?;

    Ok(orphans)
}

/// Delete orphan mirror files (D-12). Returns count of deleted files.
pub fn delete_orphans(orphans: &[PathBuf]) -> Result<usize> {
    let mut count = 0;
    for path in orphans {
        if path.exists() {
            fs::remove_file(path)?;
            count += 1;
        }
    }
    Ok(count)
}

/// Recursively find all .md files relative to base.
fn walk_md_files(dir: &Path, base: &Path, callback: &mut dyn FnMut(&str)) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            walk_md_files(&path, base, callback)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            if let Ok(relative) = path.strip_prefix(base) {
                callback(&relative.to_string_lossy());
            }
        }
    }

    Ok(())
}

// ─── Full I/O Pipeline ──────────────────────────────────────────────────────

/// Generate and write a complete mirror file for a single source file.
pub fn generate_and_write_mirror(
    source_path: &str,
    source_content: &str,
    file_result: &FileResult,
    source_commit: &str,
    module_id: &str,
    mirror_root: &str,
    preserve_appendix: bool,
) -> Result<MirrorWriteResult> {
    // 1. Compute fingerprints
    let fingerprints = fingerprint::compute_fingerprints(source_content, file_result);

    // 2. Compute mirror path
    let mirror_path = template::source_to_mirror_path(source_path, mirror_root);

    // 3. Generate mirror
    let mirror = template::generate_mirror(file_result, &fingerprints, source_commit, module_id);
    let content = template::mirror_to_string(&mirror);

    // 4. Write with appendix preservation
    let appendix_result =
        write_mirror_with_appendix(Path::new(&mirror_path), &content, preserve_appendix)?;

    Ok(MirrorWriteResult {
        mirror_path,
        source_fingerprint: fingerprints.source,
        semantic_fingerprint: fingerprints.semantic,
        appendix_preserved: appendix_result.appendix_preserved,
    })
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn sample_file_result() -> FileResult {
        FileResult {
            file_path: "src/test.ts".to_string(),
            confidence: ConfidenceBand::High,
            exports: vec![ExportedSymbol {
                name: "greet".to_string(),
                kind: ExportKind::Function,
                line: 1,
                is_generated: false,
                confidence: ConfidenceBand::High,
            }],
            imports: vec![],
            signatures: vec![],
            side_effects: vec![],
            parse_time_ms: 0,
            error_message: None,
        }
    }

    // ── Write tests ──

    #[test]
    fn write_creates_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.md");
        write_mirror(&path, "hello").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "hello");
    }

    #[test]
    fn write_creates_parent_dirs() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("a/b/c/test.md");
        write_mirror(&path, "hello").unwrap();
        assert!(path.exists());
    }

    #[test]
    fn write_overwrites_existing() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.md");
        write_mirror(&path, "old").unwrap();
        write_mirror(&path, "new").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "new");
    }

    // ── Appendix tests ──

    #[test]
    fn extract_appendix_with_markers() {
        let content =
            "body\n<!-- manual_appendix_start -->\nmy notes\n<!-- manual_appendix_end -->\n";
        let appendix = extract_appendix(content);
        assert_eq!(appendix, Some("my notes".to_string()));
    }

    #[test]
    fn extract_appendix_no_markers() {
        let content = "body only";
        assert_eq!(extract_appendix(content), None);
    }

    #[test]
    fn extract_appendix_empty_between_markers() {
        let content = "<!-- manual_appendix_start -->\n\n<!-- manual_appendix_end -->";
        assert_eq!(extract_appendix(content), None);
    }

    #[test]
    fn write_preserves_appendix() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.md");

        // Write initial content with appendix
        let initial = format!(
            "body content\n{}\nuser notes here\n{}\n",
            APPENDIX_START, APPENDIX_END
        );
        write_mirror(&path, &initial).unwrap();

        // Overwrite with new body, preserving appendix
        let new_content = "new body content\n";
        let result = write_mirror_with_appendix(&path, new_content, true).unwrap();

        assert!(result.appendix_preserved);
        assert_eq!(result.appendix_content, Some("user notes here".to_string()));

        let final_content = fs::read_to_string(&path).unwrap();
        assert!(final_content.contains("new body content"));
        assert!(final_content.contains("user notes here"));
    }

    #[test]
    fn write_overwrites_appendix_when_disabled() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.md");

        let initial = format!("body\n{}\nuser notes\n{}\n", APPENDIX_START, APPENDIX_END);
        write_mirror(&path, &initial).unwrap();

        let result = write_mirror_with_appendix(&path, "new body", false).unwrap();
        assert!(!result.appendix_preserved);

        let content = fs::read_to_string(&path).unwrap();
        assert!(!content.contains("user notes"));
    }

    #[test]
    fn write_no_appendix_when_no_existing() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.md");

        let result = write_mirror_with_appendix(&path, "new body", true).unwrap();
        assert!(!result.appendix_preserved);

        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, "new body");
    }

    // ── Read tests ──

    #[test]
    fn read_valid_mirror_header() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.md");
        let content = r#"---
anrsm_version: 1
artifact_type: file_mirror
artifact_id: file_mirror:src/test.ts
source_path: src/test.ts
source_language: typescript
module_id: test
source_commit: abc1234
source_fingerprint: sha256:aabb
semantic_fingerprint: sha256:ccdd
freshness_state: fresh
confidence_band: high
generator_name: anrsm
generator_version: 0.1.0
generated_at: 2026-01-01T00:00:00Z
---

## 职责
test
"#;
        fs::write(&path, content).unwrap();

        let header = read_mirror_header(&path).unwrap();
        assert_eq!(header.anrsm_version, 1);
        assert_eq!(header.artifact_type, "file_mirror");
        assert_eq!(header.source_path, "src/test.ts");
    }

    #[test]
    fn read_nonexistent_returns_none() {
        let path = Path::new("/nonexistent/test.md");
        assert!(read_mirror_header(path).is_none());
    }

    #[test]
    fn read_invalid_yaml_returns_none() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.md");
        fs::write(&path, "---\ninvalid: yaml: here\n---\n").unwrap();
        assert!(read_mirror_header(&path).is_none());
    }

    // ── Orphan tests ──

    #[test]
    fn test_find_orphan_mirrors() {
        let dir = TempDir::new().unwrap();
        let mirror_root = dir.path().join("mirror/file");

        // Create mirror files
        fs::create_dir_all(mirror_root.join("src")).unwrap();
        fs::write(mirror_root.join("src/exists.ts.md"), "a").unwrap();
        fs::write(mirror_root.join("src/orphan.ts.md"), "b").unwrap();

        let source_paths = vec!["src/exists.ts".to_string()];
        let orphans = find_orphan_mirrors(&source_paths, &mirror_root).unwrap();

        assert_eq!(orphans.len(), 1);
        assert!(orphans[0].to_string_lossy().contains("orphan.ts.md"));
    }

    #[test]
    fn delete_orphans_removes_files() {
        let dir = TempDir::new().unwrap();
        let orphan = dir.path().join("orphan.md");
        fs::write(&orphan, "orphan").unwrap();

        let count = delete_orphans(std::slice::from_ref(&orphan)).unwrap();
        assert_eq!(count, 1);
        assert!(!orphan.exists());
    }

    #[test]
    fn no_orphans_returns_empty() {
        let dir = TempDir::new().unwrap();
        let mirror_root = dir.path().join("mirror/file");
        fs::create_dir_all(&mirror_root).unwrap();

        let source_paths = vec!["src/test.ts".to_string()];
        let orphans = find_orphan_mirrors(&source_paths, &mirror_root).unwrap();
        assert!(orphans.is_empty());
    }

    #[test]
    fn empty_mirror_root_returns_empty() {
        let dir = TempDir::new().unwrap();
        let mirror_root = dir.path().join("nonexistent");
        let orphans = find_orphan_mirrors(&[], &mirror_root).unwrap();
        assert!(orphans.is_empty());
    }

    // ── Pipeline test ──

    #[test]
    fn generate_and_write_creates_mirror() {
        let dir = TempDir::new().unwrap();
        let result = sample_file_result();
        let mirror_root = dir.path().join("mirror/file");

        let write_result = generate_and_write_mirror(
            "src/test.ts",
            "export function greet() {}",
            &result,
            "abc1234",
            "test",
            &mirror_root.to_string_lossy(),
            false,
        )
        .unwrap();

        assert!(write_result.source_fingerprint.starts_with("sha256:"));
        assert!(write_result.semantic_fingerprint.starts_with("sha256:"));
        assert!(!write_result.appendix_preserved);

        let mirror_path = Path::new(&write_result.mirror_path);
        assert!(mirror_path.exists());

        let content = fs::read_to_string(mirror_path).unwrap();
        assert!(content.contains("artifact_type: file_mirror"));
        assert!(content.contains("## 职责"));
        assert!(content.contains("## 对外契约"));
        assert!(content.contains("greet"));
    }
}
