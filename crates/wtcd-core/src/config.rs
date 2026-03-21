use serde::{Deserialize, Serialize};

/// Top-level configuration (anrsm.yaml / wtcd.yaml) (D-08)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub version: u32,
    #[serde(default = "default_repo_root")]
    pub repo_root: String,
    pub scope: ScopeConfig,
    #[serde(default)]
    pub mirror: Option<MirrorConfig>,
    #[serde(default)]
    pub output: Option<OutputConfig>,
    #[serde(default)]
    pub gate: Option<GateConfig>,
}

fn default_repo_root() -> String {
    ".".to_string()
}

/// Scope configuration block (D-09, D-11)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeConfig {
    pub source_roots: Vec<String>,
    #[serde(default)]
    pub exclude_patterns: Vec<String>,
}

/// Mirror configuration block (D-08, D-15)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MirrorConfig {
    #[serde(default = "default_mirror_output_dir")]
    pub output_dir: String,
    #[serde(default = "default_fp_version")]
    pub fp_version: String,
    #[serde(default)]
    pub include_manual_appendix: bool,
}

fn default_mirror_output_dir() -> String {
    "mirror/file".to_string()
}

fn default_fp_version() -> String {
    "1".to_string()
}

/// Output configuration block
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OutputConfig {
    #[serde(default = "default_output_format")]
    pub format: String,
}

fn default_output_format() -> String {
    "json".to_string()
}

/// Gate configuration block (D-09: top-level gate: block in anrsm.yaml)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GateConfig {
    /// Fail_on rules: which change classes trigger exit code 1 (D-10: default none)
    #[serde(default)]
    pub fail_on: Vec<FailRule>,
    /// Warn_on rules: which change classes produce warnings but don't fail
    #[serde(default)]
    pub warn_on: Vec<WarnRule>,
    /// C3 threshold: minimum importer count for systemic classification (D-08)
    #[serde(default = "default_systemic_threshold")]
    pub systemic_threshold: usize,
}

fn default_systemic_threshold() -> usize {
    5
}

/// A fail rule specifies which change classes block (D-11)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailRule {
    /// Change class to match: "C1", "C2", "C3", or "any"
    pub when: String,
    /// Failure code: ANRSM-001 through ANRSM-010
    pub code: String,
}

/// A warn rule specifies which change classes produce warnings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarnRule {
    /// Change class to match: "C0", "C1", "C2", "C3", or "any"
    pub when: String,
}

impl Config {
    /// Load config from YAML string
    pub fn from_yaml(yaml: &str) -> crate::error::Result<Self> {
        yaml_serde::from_str(yaml).map_err(|e| crate::error::WtcdError::YamlError(e.to_string()))
    }

    /// Serialize config to YAML string
    pub fn to_yaml(&self) -> crate::error::Result<String> {
        yaml_serde::to_string(self).map_err(|e| crate::error::WtcdError::YamlError(e.to_string()))
    }
}
