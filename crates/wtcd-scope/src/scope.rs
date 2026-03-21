use std::path::{Path, PathBuf};
use wtcd_core::config::Config;
use wtcd_core::error::{Result, WtcdError};

/// Config file name (D-08, D-09)
pub const CONFIG_FILENAME: &str = "anrsm.yaml";

/// Load configuration from a directory
/// Returns error if anrsm.yaml doesn't exist (D-10)
pub fn load_config(repo_root: &Path) -> Result<Config> {
    let config_path = repo_root.join(CONFIG_FILENAME);
    if !config_path.exists() {
        return Err(WtcdError::ConfigError(format!(
            "No {} found. Run `wtcd init` first.",
            CONFIG_FILENAME
        )));
    }
    let yaml = std::fs::read_to_string(&config_path)
        .map_err(|e| WtcdError::ConfigError(format!("Cannot read {}: {}", CONFIG_FILENAME, e)))?;
    Config::from_yaml(&yaml)
}

/// Find the config file by searching up from given directory
pub fn find_config(start_dir: &Path) -> Option<PathBuf> {
    let mut dir = start_dir.to_path_buf();
    loop {
        let candidate = dir.join(CONFIG_FILENAME);
        if candidate.exists() {
            return Some(candidate);
        }
        if !dir.pop() {
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn valid_yaml() -> &'static str {
        r#"
version: 1
repo_root: .
scope:
  source_roots:
    - src
    - apps
  exclude_patterns:
    - "**/node_modules/**"
    - "**/dist/**"
"#
    }

    fn create_temp_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("wtcd-test-{}", name));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    // Test 1: load_config 从有效的 YAML 字符串成功解析 Config
    #[test]
    fn test_load_config_valid_yaml() {
        let dir = create_temp_dir("valid");
        fs::write(dir.join(CONFIG_FILENAME), valid_yaml()).unwrap();

        let config = load_config(&dir).unwrap();
        assert_eq!(config.version, 1);
        assert_eq!(config.repo_root, ".");

        let _ = fs::remove_dir_all(&dir);
    }

    // Test 2: load_config 在无效 YAML 时返回错误
    #[test]
    fn test_load_config_invalid_yaml() {
        let dir = create_temp_dir("invalid");
        fs::write(dir.join(CONFIG_FILENAME), "{{invalid: yaml: [}").unwrap();

        let result = load_config(&dir);
        assert!(result.is_err());
        match result.unwrap_err() {
            WtcdError::YamlError(_) => {} // expected
            other => panic!("Expected YamlError, got: {:?}", other),
        }

        let _ = fs::remove_dir_all(&dir);
    }

    // Test 3: load_config 在缺少 required 字段时返回解析错误
    #[test]
    fn test_load_config_missing_required_field() {
        let dir = create_temp_dir("missing-field");
        // Missing 'version' and 'scope' required fields
        fs::write(dir.join(CONFIG_FILENAME), "repo_root: .\n").unwrap();

        let result = load_config(&dir);
        assert!(result.is_err());

        let _ = fs::remove_dir_all(&dir);
    }

    // Test 4: source_roots 和 exclude_patterns 被正确保留
    #[test]
    fn test_load_config_preserves_scope_fields() {
        let dir = create_temp_dir("scope-fields");
        fs::write(dir.join(CONFIG_FILENAME), valid_yaml()).unwrap();

        let config = load_config(&dir).unwrap();
        assert_eq!(config.scope.source_roots, vec!["src", "apps"]);
        assert_eq!(
            config.scope.exclude_patterns,
            vec!["**/node_modules/**", "**/dist/**"]
        );

        let _ = fs::remove_dir_all(&dir);
    }

    // Test 5: load_config 在无配置文件时返回 ConfigError
    #[test]
    fn test_load_config_missing_file() {
        let dir = create_temp_dir("no-config");

        let result = load_config(&dir);
        assert!(result.is_err());
        match result.unwrap_err() {
            WtcdError::ConfigError(msg) => {
                assert!(msg.contains("anrsm.yaml"));
            }
            other => panic!("Expected ConfigError, got: {:?}", other),
        }

        let _ = fs::remove_dir_all(&dir);
    }

    // Test 6: find_config 向上搜索找到配置文件
    #[test]
    fn test_find_config_searches_up() {
        let root = create_temp_dir("find-root");
        let nested = root.join("a/b/c");
        fs::create_dir_all(&nested).unwrap();
        fs::write(root.join(CONFIG_FILENAME), valid_yaml()).unwrap();

        let found = find_config(&nested);
        assert!(found.is_some());
        assert_eq!(found.unwrap(), root.join(CONFIG_FILENAME));

        let _ = fs::remove_dir_all(&root);
    }

    // Test 7: find_config 返回 None 当配置不存在
    #[test]
    fn test_find_config_not_found() {
        let dir = create_temp_dir("find-none");

        let found = find_config(&dir);
        assert!(found.is_none());

        let _ = fs::remove_dir_all(&dir);
    }
}
