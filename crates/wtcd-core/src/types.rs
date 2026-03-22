use serde::{Deserialize, Serialize};

/// Confidence band for parse results (D-12..D-14)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ConfidenceBand {
    /// Complete parse, no errors
    High,
    /// Partial parse, tree-sitter error recovery used
    Low,
    /// Complete failure, no extraction possible
    None,
}

/// Exported symbol kind (D-16)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ExportKind {
    Function,
    Class,
    Const,
    Let,
    Var,
    Type,
    Interface,
    Enum,
}

/// An exported symbol from a file (D-02, D-16)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedSymbol {
    pub name: String,
    pub kind: ExportKind,
    pub line: u32,
    /// Whether this symbol is from generated/uncertain code
    #[serde(default)]
    pub is_generated: bool,
    /// Per-symbol confidence band
    #[serde(default = "default_high_confidence")]
    pub confidence: ConfidenceBand,
}

fn default_high_confidence() -> ConfidenceBand {
    ConfidenceBand::High
}

/// Import kind (D-17)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ImportKind {
    Named,
    Default,
    Namespace,
}

/// A dependency edge (D-05, D-17)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyEdge {
    pub source: String,
    pub symbols: Vec<String>,
    pub kind: ImportKind,
}

/// A function/method signature (D-18)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionSignature {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: String,
}

/// A function parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub type_annotation: String,
}

/// Side effect kind (D-06, D-19)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SideEffectKind {
    Io,
    Network,
    Storage,
    Log,
}

/// A detected side effect (D-06)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SideEffect {
    pub kind: SideEffectKind,
    pub target: String,
    pub line: u32,
}

/// Parse result for a single file (D-02)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileResult {
    pub file_path: String,
    pub confidence: ConfidenceBand,
    pub exports: Vec<ExportedSymbol>,
    pub imports: Vec<DependencyEdge>,
    pub signatures: Vec<FunctionSignature>,
    pub side_effects: Vec<SideEffect>,
    pub parse_time_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

/// Module-level aggregated result (Phase 7)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleResult {
    pub module_id: String,
    pub language: String,
    pub files: Vec<String>,
    pub exports: Vec<String>,
    pub dependencies: Vec<String>,
    pub side_effects: Vec<String>,
    pub responsibility: String,
    pub semantic_fingerprint: String,
    pub fan_in: usize,
    pub fan_out: usize,
    pub drift_level: String,
    /// Module-level confidence (min across all files)
    #[serde(default = "default_high_confidence")]
    pub confidence: ConfidenceBand,
}

/// Repository-level knowledge output (Phase 8)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeResult {
    pub module_count: usize,
    pub language_distribution: std::collections::BTreeMap<String, usize>,
    pub total_files: usize,
    pub total_exports: usize,
    pub token_compression_ratio: f64,
}

/// Run-level summary statistics (D-07)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunSummary {
    pub total_files: usize,
    pub parsed_ok: usize,
    pub confidence_low: usize,
    pub confidence_none: usize,
    pub total_exports: usize,
    pub total_imports: usize,
    pub elapsed_ms: u64,
}

/// Top-level JSON output (D-01, D-04)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunOutput {
    pub api_version: String,
    pub files: Vec<FileResult>,
    pub errors: Vec<String>,
    pub summary: RunSummary,
}

// ─── Drift Detection Types (Phase 3) ──────────────────────────────────────

/// Material change classification (DRFT-02)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "UPPERCASE")]
pub enum ChangeClass {
    /// Source changed, semantic unchanged (formatting/comments)
    C0,
    /// Semantic changed, exports/signatures unchanged (internal logic)
    C1,
    /// Exports or signatures changed (contract change)
    C2,
    /// Systemic: imported by >= N files (configurable threshold)
    C3,
}

impl std::fmt::Display for ChangeClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChangeClass::C0 => write!(f, "C0"),
            ChangeClass::C1 => write!(f, "C1"),
            ChangeClass::C2 => write!(f, "C2"),
            ChangeClass::C3 => write!(f, "C3"),
        }
    }
}

/// A single material change entry in the drift report (maps to drift-report.schema.json material_changes[])
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialChange {
    pub change_class: ChangeClass,
    pub source_path: String,
    pub summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence: Option<Vec<String>>,
}

/// Freshness state for an affected artifact (maps to drift-report.schema.json affected_artifacts[])
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffectedArtifact {
    pub artifact_id: String,
    pub before_freshness: String,
    pub after_freshness: String,
    pub drift_level: String,
    pub reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommended_action: Option<String>,
}

/// Policy evaluation result (maps to drift-report.schema.json policy_result)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyResult {
    pub status: String, // "pass", "warn", "fail"
    pub failed_rules: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_codes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommendations: Option<Vec<String>>,
}

/// Complete drift report (maps to drift-report.schema.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftReport {
    pub report_version: u32, // always 1
    pub report_id: String,
    pub repo_commit_before: String,
    pub repo_commit_after: String,
    pub generated_at: String,
    pub drift_level: String, // "none", "low", "material", "blocking"
    pub changed_files: Vec<String>,
    pub affected_artifacts: Vec<AffectedArtifact>,
    pub material_changes: Vec<MaterialChange>,
    pub policy_result: PolicyResult,
}
