use std::path::Path;
use wtcd_core::error::{Result, WtcdError};

/// Default config template (D-29)
const DEFAULT_CONFIG: &str = r#"# WTCD — What The Code Doing
version: 1
repo_root: .

scope:
  source_roots:
    - src
    - app
    - lib
    - packages
  exclude_patterns:
    - "**/node_modules/**"
    - "**/dist/**"
    - "**/build/**"
    - "**/.next/**"
    - "**/coverage/**"
    - "**/*.d.ts"
    - "**/*.min.js"

mirror: {}

output:
  format: json
"#;

pub fn run_init(repo_root: &Path) -> Result<()> {
    let config_path = repo_root.join("anrsm.yaml");

    if config_path.exists() {
        return Err(WtcdError::ConfigError(
            "anrsm.yaml already exists. Delete it first to reinitialize.".to_string(),
        ));
    }

    std::fs::write(&config_path, DEFAULT_CONFIG)
        .map_err(|e| WtcdError::ConfigError(format!("Cannot write anrsm.yaml: {}", e)))?;

    // Create .anrsm directory
    let anrsm_dir = repo_root.join(".anrsm");
    if !anrsm_dir.exists() {
        std::fs::create_dir_all(&anrsm_dir)?;
    }

    // JSON output (D-01, CORE-06)
    let output = serde_json::json!({
        "status": "ok",
        "message": "Initialized wtcd configuration",
        "config_path": config_path.display().to_string(),
    });
    println!("{}", serde_json::to_string_pretty(&output).unwrap());

    Ok(())
}
