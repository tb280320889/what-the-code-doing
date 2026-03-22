use anyhow::Result;
use gix::bstr::ByteSlice;
use std::path::Path;

use crate::types::*;

/// Get the short commit hash for HEAD
pub fn get_head_commit(repo_root: &Path) -> Result<String> {
    let repo = gix::open(repo_root)?;
    let head_id = repo.head_id()?;
    Ok(head_id.shorten_or_id().to_string())
}

/// Diff working tree against HEAD — returns changed files (D-01, D-04)
///
/// Compares the current working tree state against the HEAD commit.
/// Returns all files that are modified, added, or deleted.
pub fn diff_working_tree_vs_head(repo_root: &Path) -> Result<DiffResult> {
    let repo = gix::open(repo_root)?;
    let head_id = repo.head_id()?;
    let head_commit_str = head_id.shorten_or_id().to_string();

    let workdir = repo
        .work_dir()
        .ok_or_else(|| anyhow::anyhow!("No workdir"))?;

    let mut changed_files = Vec::new();

    // Build set of tracked files from the index
    let index = repo.index()?;
    let mut tracked_paths: std::collections::HashSet<String> = std::collections::HashSet::new();
    for entry in index.entries() {
        let path = entry.path(&index);
        tracked_paths.insert(path.to_str_lossy().to_string());
    }

    // Use gix status for index-worktree modifications
    let status = repo.status(gix::progress::Discard)?;
    let iter = status.into_index_worktree_iter(Vec::new())?;

    for item in iter {
        let item = item?;
        if let gix::status::index_worktree::iter::Item::Modification { rela_path, .. } = item {
            let path_str = rela_path.to_str_lossy().to_string();
            let workdir_path = workdir.join(&path_str);

            if !workdir_path.exists() {
                // Tracked file deleted from working tree
                changed_files.push(ChangedFile {
                    path: path_str,
                    status: FileStatus::Deleted,
                });
            } else {
                // Tracked file modified in working tree
                changed_files.push(ChangedFile {
                    path: path_str,
                    status: FileStatus::Modified,
                });
            }
        }
    }

    // Walk working tree to find untracked (added) files
    walk_dir_for_untracked(workdir, workdir, &tracked_paths, &mut changed_files)?;

    Ok(DiffResult {
        is_clean: changed_files.is_empty(),
        changed_files,
        head_commit: head_commit_str,
    })
}

/// Recursively walk working tree directory, collecting untracked files
fn walk_dir_for_untracked(
    dir: &Path,
    base: &Path,
    tracked: &std::collections::HashSet<String>,
    changed: &mut Vec<ChangedFile>,
) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        // Skip .git directory
        if entry.file_name() == ".git" {
            continue;
        }

        if path.is_dir() {
            walk_dir_for_untracked(&path, base, tracked, changed)?;
        } else {
            let relative = path
                .strip_prefix(base)
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();

            if !tracked.contains(&relative) {
                // Untracked file = Added
                changed.push(ChangedFile {
                    path: relative,
                    status: FileStatus::Added,
                });
            }
        }
    }
    Ok(())
}

/// Get the HEAD commit hash — convenience wrapper
pub fn head_commit_hash(repo_root: &Path) -> Result<String> {
    get_head_commit(repo_root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::process::Command;
    use tempfile::TempDir;

    fn init_repo(dir: &TempDir) {
        Command::new("git")
            .args(["init"])
            .current_dir(dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(dir.path())
            .output()
            .unwrap();
    }

    fn commit_file(dir: &TempDir, path: &str, content: &str) {
        let full_path = dir.path().join(path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&full_path, content).unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", &format!("add {}", path)])
            .current_dir(dir.path())
            .output()
            .unwrap();
    }

    #[test]
    fn test_head_commit_returns_hash() {
        let dir = TempDir::new().unwrap();
        init_repo(&dir);
        commit_file(&dir, "test.txt", "hello");

        let hash = get_head_commit(dir.path()).unwrap();
        assert!(!hash.is_empty());
        assert!(hash.len() <= 10); // short hash
    }

    #[test]
    fn test_diff_detects_modified_file() {
        let dir = TempDir::new().unwrap();
        init_repo(&dir);
        commit_file(&dir, "test.txt", "hello");

        // Modify the file
        fs::write(dir.path().join("test.txt"), "modified").unwrap();

        let result = diff_working_tree_vs_head(dir.path()).unwrap();
        assert!(!result.is_clean);
        assert_eq!(result.changed_files.len(), 1);
        assert_eq!(result.changed_files[0].path, "test.txt");
        assert_eq!(result.changed_files[0].status, FileStatus::Modified);
    }

    #[test]
    fn test_diff_detects_added_file() {
        let dir = TempDir::new().unwrap();
        init_repo(&dir);
        commit_file(&dir, "existing.txt", "hello");

        // Add new file (untracked)
        fs::write(dir.path().join("new.txt"), "new content").unwrap();

        let result = diff_working_tree_vs_head(dir.path()).unwrap();
        assert!(!result.is_clean);
        let added: Vec<_> = result
            .changed_files
            .iter()
            .filter(|f| f.status == FileStatus::Added)
            .collect();
        assert!(
            !added.is_empty(),
            "Expected added files, got: {:?}",
            result.changed_files
        );
        assert!(added.iter().any(|f| f.path == "new.txt"));
    }

    #[test]
    fn test_diff_detects_deleted_file() {
        let dir = TempDir::new().unwrap();
        init_repo(&dir);
        commit_file(&dir, "to_delete.txt", "bye");

        // Delete the file
        fs::remove_file(dir.path().join("to_delete.txt")).unwrap();

        let result = diff_working_tree_vs_head(dir.path()).unwrap();
        assert!(!result.is_clean);
        assert!(result
            .changed_files
            .iter()
            .any(|f| f.path == "to_delete.txt" && f.status == FileStatus::Deleted));
    }

    #[test]
    fn test_diff_clean_working_tree() {
        let dir = TempDir::new().unwrap();
        init_repo(&dir);
        commit_file(&dir, "test.txt", "unchanged");

        let result = diff_working_tree_vs_head(dir.path()).unwrap();
        assert!(result.is_clean);
        assert!(result.changed_files.is_empty());
    }
}
