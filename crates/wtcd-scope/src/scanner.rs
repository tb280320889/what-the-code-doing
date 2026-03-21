use globset::{Glob, GlobSet, GlobSetBuilder};
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};
use wtcd_core::config::ScopeConfig;
use wtcd_core::error::{Result, WtcdError};

/// Supported file extensions (Phase 1)
const SUPPORTED_EXTENSIONS: &[&str] = &["ts", "tsx", "js", "jsx"];

/// Scan source roots for supported files, applying exclude patterns and .gitignore
pub fn scan(repo_root: &Path, scope: &ScopeConfig) -> Result<Vec<PathBuf>> {
    let exclude_set = build_exclude_globset(&scope.exclude_patterns)?;
    let mut results = Vec::new();

    for source_root in &scope.source_roots {
        let root_path = repo_root.join(source_root);
        if !root_path.exists() {
            continue;
        }

        let walker = WalkBuilder::new(&root_path)
            .git_ignore(true)
            .hidden(true)
            .build();

        for entry in walker.flatten() {
            let path = entry.path();

            // Skip directories
            if path.is_dir() {
                continue;
            }

            // Check file extension
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if !SUPPORTED_EXTENSIONS.contains(&ext) {
                continue;
            }

            // Check exclude patterns (relative to repo root)
            let relative = path.strip_prefix(repo_root).unwrap_or(path);
            if exclude_set.is_match(relative) {
                continue;
            }

            results.push(path.to_path_buf());
        }
    }

    // Sort for determinism (SCOP-02)
    results.sort();
    Ok(results)
}

fn build_exclude_globset(patterns: &[String]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        let glob = Glob::new(pattern)
            .map_err(|e| WtcdError::ScopeError(format!("Invalid glob '{}': {}", pattern, e)))?;
        builder.add(glob);
    }
    builder
        .build()
        .map_err(|e| WtcdError::ScopeError(format!("Failed to build glob set: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn create_temp_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("wtcd-scan-test-{}", name));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn scope_with_roots(roots: Vec<&str>) -> ScopeConfig {
        ScopeConfig {
            source_roots: roots.into_iter().map(String::from).collect(),
            exclude_patterns: vec![],
        }
    }

    fn scope_with_exclude(roots: Vec<&str>, exclude: Vec<&str>) -> ScopeConfig {
        ScopeConfig {
            source_roots: roots.into_iter().map(String::from).collect(),
            exclude_patterns: exclude.into_iter().map(String::from).collect(),
        }
    }

    // Test 1: scan 枚举指定 source_roots 下的所有 .ts/.js 文件
    #[test]
    fn test_scan_finds_ts_js_files() {
        let dir = create_temp_dir("find-ts");
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::write(dir.join("src/a.ts"), "").unwrap();
        fs::write(dir.join("src/b.js"), "").unwrap();
        fs::write(dir.join("src/c.tsx"), "").unwrap();
        fs::write(dir.join("src/d.jsx"), "").unwrap();

        let scope = scope_with_roots(vec!["src"]);
        let results = scan(&dir, &scope).unwrap();

        assert_eq!(results.len(), 4);
        // Verify all are .ts/.tsx/.js/.jsx
        for path in &results {
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            assert!(
                ["ts", "tsx", "js", "jsx"].contains(&ext),
                "Unexpected extension: {}",
                ext
            );
        }

        let _ = fs::remove_dir_all(&dir);
    }

    // Test 2: exclude_patterns 正确过滤匹配的文件
    #[test]
    fn test_scan_exclude_patterns() {
        let dir = create_temp_dir("exclude");
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::write(dir.join("src/app.ts"), "").unwrap();
        fs::write(dir.join("src/excluded.ts"), "").unwrap();

        let scope = scope_with_exclude(vec!["src"], vec!["**/excluded.ts"]);
        let results = scan(&dir, &scope).unwrap();

        assert_eq!(results.len(), 1);
        assert!(results[0].ends_with("app.ts"));

        let _ = fs::remove_dir_all(&dir);
    }

    // Test 3: .gitignore 中的规则被尊重
    #[test]
    fn test_scan_respects_gitignore() {
        let dir = create_temp_dir("gitignore");
        // ignore crate needs .git directory to enable gitignore
        fs::create_dir_all(dir.join(".git")).unwrap();
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::write(dir.join("src/keep.ts"), "").unwrap();
        fs::write(dir.join("src/generated.ts"), "").unwrap();
        fs::write(dir.join(".gitignore"), "src/generated.ts\n").unwrap();

        let scope = scope_with_roots(vec!["src"]);
        let results = scan(&dir, &scope).unwrap();

        // .gitignore should exclude generated.ts
        assert_eq!(results.len(), 1);
        assert!(results[0].ends_with("keep.ts"));

        let _ = fs::remove_dir_all(&dir);
    }

    // Test 4: 非 .ts/.tsx/.js/.jsx 文件被过滤掉
    #[test]
    fn test_scan_filters_non_js_extensions() {
        let dir = create_temp_dir("filter-ext");
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::write(dir.join("src/app.ts"), "").unwrap();
        fs::write(dir.join("src/readme.md"), "").unwrap();
        fs::write(dir.join("src/data.json"), "").unwrap();
        fs::write(dir.join("src/style.css"), "").unwrap();
        fs::write(dir.join("src/script.py"), "").unwrap();

        let scope = scope_with_roots(vec!["src"]);
        let results = scan(&dir, &scope).unwrap();

        // Only .ts should be included
        assert_eq!(results.len(), 1);
        assert!(results[0].ends_with("app.ts"));

        let _ = fs::remove_dir_all(&dir);
    }

    // Test 5: 结果按路径排序（确定性）
    #[test]
    fn test_scan_results_sorted() {
        let dir = create_temp_dir("sorted");
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::write(dir.join("src/z.ts"), "").unwrap();
        fs::write(dir.join("src/a.ts"), "").unwrap();
        fs::write(dir.join("src/m.ts"), "").unwrap();

        let scope = scope_with_roots(vec!["src"]);
        let results = scan(&dir, &scope).unwrap();

        assert_eq!(results.len(), 3);
        // Verify sorted order
        for i in 1..results.len() {
            assert!(
                results[i - 1] <= results[i],
                "Results not sorted: {:?}",
                results
            );
        }

        let _ = fs::remove_dir_all(&dir);
    }

    // Test 6: 多个 source_roots 被扫描
    #[test]
    fn test_scan_multiple_source_roots() {
        let dir = create_temp_dir("multi-root");
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::create_dir_all(dir.join("apps")).unwrap();
        fs::write(dir.join("src/a.ts"), "").unwrap();
        fs::write(dir.join("apps/b.ts"), "").unwrap();

        let scope = scope_with_roots(vec!["src", "apps"]);
        let results = scan(&dir, &scope).unwrap();

        assert_eq!(results.len(), 2);

        let _ = fs::remove_dir_all(&dir);
    }

    // Test 7: 不存在的 source_root 被跳过
    #[test]
    fn test_scan_skips_nonexistent_roots() {
        let dir = create_temp_dir("nonexist");
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::write(dir.join("src/a.ts"), "").unwrap();

        let scope = scope_with_roots(vec!["src", "does_not_exist"]);
        let results = scan(&dir, &scope).unwrap();

        // Should only find files in existing root
        assert_eq!(results.len(), 1);

        let _ = fs::remove_dir_all(&dir);
    }
}
