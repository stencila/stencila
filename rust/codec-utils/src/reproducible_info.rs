use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use common::eyre::{OptionExt, Result, bail};

/// Determine the relative path and commit of a file within a Git repository
///
/// Used by codecs within implementations of `Codecs::from_path` to set the `source`
/// and `commit` properties of the decoded node.
///
/// # Returns
///
/// * If `path` is not in a Git repo → `(None, None)`
/// * If `git` is not available  → `(Some(relative_path), None)`
/// * If the file is untracked → `(Some(relative_path), Some("untracked"))`
/// * If the file is dirty → `(Some(relative_path), Some("dirty")))`
/// * Otherwise → `(Some(relative_path), Some(head_commit_sha))`
pub fn reproducible_info(path: &Path) -> Result<(Option<String>, Option<String>)> {
    let path = path.canonicalize()?;

    let Ok(repo_root) = closest_git_repo(&path) else {
        return Ok((None, None));
    };

    let relative_path = path
        .strip_prefix(&repo_root)?
        .to_str()
        .ok_or_eyre("Path is not valid UTF-8")?
        .to_string();

    // Is git available?
    if which::which("git").is_err() {
        return Ok((Some(relative_path), None));
    }

    // Is the file tracked?
    let tracked = Command::new("git")
        .arg("-C")
        .arg(&repo_root)
        .args(["ls-files", "--error-unmatch", "--"])
        .arg(&relative_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?
        .success();

    if !tracked {
        return Ok((Some(relative_path), Some("untracked".into())));
    }

    // Is the file clean (no staged / unstaged changes)?
    // `git status --porcelain` prints nothing when the file is unchanged.
    let clean = Command::new("git")
        .arg("-C")
        .arg(&repo_root)
        .args(["status", "--porcelain", "--"])
        .arg(&relative_path)
        .output()?
        .stdout
        .is_empty();

    if !clean {
        return Ok((Some(relative_path), Some("dirty".into())));
    }

    // File is tracked *and* clean – return the current HEAD commit SHA.
    let head_out = Command::new("git")
        .arg("-C")
        .arg(&repo_root)
        .args(["rev-parse", "HEAD"])
        .output()?;

    if !head_out.status.success() {
        let message = String::from_utf8(head_out.stderr).unwrap_or_default();
        bail!("Unable to get commit SHA: {message}")
    }

    let commit = String::from_utf8(head_out.stdout)?.trim().to_string();

    Ok((Some(relative_path), Some(commit)))
}

/// Get the path of the closest Git repository to a path
///
/// If the `path` is a file then starts with the parent directory of that file.
/// Walks up the directory tree until a `.git` directory is found.
pub fn closest_git_repo(path: &Path) -> Result<PathBuf> {
    let mut current_dir = if path.is_file() {
        path.parent().ok_or_eyre("File has no parent directory")?
    } else {
        path
    };

    loop {
        let git_dir = current_dir.join(".git");
        if git_dir.exists() {
            return Ok(current_dir.to_path_buf());
        }

        let Some(parent_dir) = current_dir.parent() else {
            break;
        };
        current_dir = parent_dir;
    }

    bail!("Path is not within a Git repository: {}", path.display())
}
