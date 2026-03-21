use serde::{Deserialize, Serialize};

/// Mirror header — maps to mirror-header.schema.json YAML front matter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MirrorHeader {
    pub anrsm_version: u32,
    pub artifact_type: String,
    pub artifact_id: String,
    pub source_path: String,
    pub source_language: String,
    pub module_id: String,
    pub source_commit: String,
    pub source_fingerprint: String,
    pub semantic_fingerprint: String,
    pub freshness_state: FreshnessState,
    pub confidence_band: MirrorConfidenceBand,
    pub generator_name: String,
    pub generator_version: String,
    pub generated_at: String,
    // Optional fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exports: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependencies: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub risk_tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contracts: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adjacent_artifacts: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manual_appendix_present: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FreshnessState {
    Fresh,
    Stale,
    Invalid,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MirrorConfidenceBand {
    High,
    Medium,
    Low,
}

/// Complete mirror file content (header + body sections)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MirrorFile {
    pub header: MirrorHeader,
    pub body: MirrorBody,
}

/// 8 standard mirror sections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MirrorBody {
    pub responsibilities: String,
    pub external_contracts: String,
    pub preconditions: String,
    pub state_control_flow: String,
    pub side_effects: String,
    pub invariants_risks: String,
    pub change_impact: String,
    pub expand_conditions: String,
}

/// Result of fingerprint computation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FingerprintResult {
    pub source: String,
    pub semantic: String,
    pub fp_version: String,
}
