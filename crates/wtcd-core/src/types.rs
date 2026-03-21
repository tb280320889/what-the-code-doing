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
