use serde::{Deserialize, Serialize};

/// Status of a file in the working tree relative to HEAD
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FileStatus {
    /// File was modified (exists in HEAD and working tree, content differs)
    Modified,
    /// File is new (exists in working tree, not in HEAD)
    Added,
    /// File was deleted (exists in HEAD, not in working tree)
    Deleted,
}

/// A single changed file from git diff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangedFile {
    pub path: String,
    pub status: FileStatus,
}

/// Result of git diff operation
#[derive(Debug, Clone)]
pub struct DiffResult {
    /// Files that were modified, added, or deleted
    pub changed_files: Vec<ChangedFile>,
    /// HEAD commit hash (short form)
    pub head_commit: String,
    /// Whether the working tree is clean (no changes)
    pub is_clean: bool,
}
