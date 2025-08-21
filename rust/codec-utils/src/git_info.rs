use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use common::{
    eyre::{OptionExt, Result, bail},
    reqwest::Url,
};

/// Information about a file within a Git repository
#[derive(Debug, Clone)]
pub struct GitInfo {
    /// The remote origin URL of the repository
    pub origin: Option<String>,

    /// The relative path of the file within the repository
    pub path: Option<String>,

    /// The commit SHA, or "untracked"/"dirty" for uncommitted changes
    pub commit: Option<String>,
}

/// Determine the relative path and commit of a file within a Git repository
///
/// Used by codecs within implementations of `Codecs::from_path` to set the `source`
/// and `commit` properties of the decoded node.
///
/// # Returns
///
/// * If `path` is not in a Git repo → `GitInfo` with all `None` values
/// * If `git` is not available  → `GitInfo` with relative_path only
/// * If the file is untracked → `GitInfo` with relative_path and commit="untracked"
/// * If the file is dirty → `GitInfo` with relative_path and commit="dirty"
/// * Otherwise → `GitInfo` with origin, relative_path, and commit SHA
pub fn git_info(path: &Path) -> Result<GitInfo> {
    let path = path.canonicalize()?;

    let Ok(repo_root) = closest_git_repo(&path) else {
        return Ok(GitInfo {
            origin: None,
            path: None,
            commit: None,
        });
    };

    let relative_path = path
        .strip_prefix(&repo_root)?
        .to_str()
        .ok_or_eyre("Path is not valid UTF-8")?
        .to_string();

    // Is git available?
    if which::which("git").is_err() {
        return Ok(GitInfo {
            origin: None,
            path: Some(relative_path),
            commit: None,
        });
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
        return Ok(GitInfo {
            origin: get_git_origin(&repo_root),
            path: Some(relative_path),
            commit: Some("untracked".into()),
        });
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
        return Ok(GitInfo {
            origin: get_git_origin(&repo_root),
            path: Some(relative_path),
            commit: Some("dirty".into()),
        });
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

    Ok(GitInfo {
        origin: get_git_origin(&repo_root),
        path: Some(relative_path),
        commit: Some(commit),
    })
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

/// Get the git remote origin URL for a repository
fn get_git_origin(repo_root: &Path) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(["remote", "get-url", "origin"])
        .output()
        .ok()?;

    if output.status.success() {
        String::from_utf8(output.stdout)
            .ok()
            .and_then(|origin| normalize_git_url(&origin))
    } else {
        None
    }
}

/// Normalize a git URL to a consistent format while removing credentials
///
/// Converts various git URL formats to normalized https:// or file:// URLs
/// and removes any embedded credentials (username:password@).
fn normalize_git_url(url: &str) -> Option<String> {
    let url = url.trim();

    // Handle SCP-style SSH URLs (git@host:path)
    if url.contains('@')
        && url.contains(':')
        && !url.starts_with("http")
        && !url.starts_with("ssh://")
        && !url.starts_with("file://")
        && let Some(at_pos) = url.find('@')
        && let Some(colon_pos) = url[at_pos..].find(':')
    {
        let host = &url[at_pos + 1..at_pos + colon_pos];
        let path = &url[at_pos + colon_pos + 1..];
        let path = path.strip_suffix(".git").unwrap_or(path);
        return Some(format!("https://{host}/{path}"));
    }

    // Try to parse as a proper URL
    let mut parsed_url = Url::parse(url).ok()?;

    // Remove credentials
    parsed_url.set_username("").ok();
    parsed_url.set_password(None).ok();

    // Convert HTTP to HTTPS for web URLs
    if parsed_url.scheme() == "http" {
        parsed_url.set_scheme("https").ok();
    }

    // Convert SSH to HTTPS for web URLs
    if parsed_url.scheme() == "ssh" {
        let host = parsed_url.host_str().unwrap_or("");
        let path = parsed_url.path();
        let path = path.strip_suffix(".git").unwrap_or(path);
        return Some(format!("https://{host}{path}"));
    }

    // Remove query parameters and fragments
    parsed_url.set_query(None);
    parsed_url.set_fragment(None);

    let url = parsed_url.to_string();

    // Remove .git suffix from path
    if let Some(result) = url.strip_suffix(".git") {
        Some(result.into())
    } else {
        Some(url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_git_url() {
        // HTTPS URLs
        assert_eq!(
            normalize_git_url("https://github.com/owner/repo").expect("should parse"),
            "https://github.com/owner/repo"
        );
        assert_eq!(
            normalize_git_url("https://github.com/owner/repo.git").expect("should parse"),
            "https://github.com/owner/repo"
        );

        // HTTPS URLs with credentials
        assert_eq!(
            normalize_git_url("https://user:pass@github.com/owner/repo").expect("should parse"),
            "https://github.com/owner/repo"
        );
        assert_eq!(
            normalize_git_url("https://alice@github.com/owner/repo").expect("should parse"),
            "https://github.com/owner/repo"
        );

        // HTTP URLs (should convert to HTTPS)
        assert_eq!(
            normalize_git_url("http://github.com/owner/repo").expect("should parse"),
            "https://github.com/owner/repo"
        );

        // HTTPS URLs with query parameters or fragments
        assert_eq!(
            normalize_git_url("https://github.com/owner/repo?tab=readme").expect("should parse"),
            "https://github.com/owner/repo"
        );
        assert_eq!(
            normalize_git_url("https://github.com/owner/repo#readme").expect("should parse"),
            "https://github.com/owner/repo"
        );

        // SSH URLs
        assert_eq!(
            normalize_git_url("ssh://git@github.com/owner/repo").expect("should parse"),
            "https://github.com/owner/repo"
        );
        assert_eq!(
            normalize_git_url("ssh://alice@github.com/owner/repo").expect("should parse"),
            "https://github.com/owner/repo"
        );
        assert_eq!(
            normalize_git_url("ssh://user:pass@github.com/owner/repo.git").expect("should parse"),
            "https://github.com/owner/repo"
        );

        // SCP-style SSH URLs
        assert_eq!(
            normalize_git_url("git@github.com:owner/repo").expect("should parse"),
            "https://github.com/owner/repo"
        );
        assert_eq!(
            normalize_git_url("git@github.com:owner/repo.git").expect("should parse"),
            "https://github.com/owner/repo"
        );

        // Other SSH formats
        assert_eq!(
            normalize_git_url("user@gitlab.com:group/project").expect("should parse"),
            "https://gitlab.com/group/project"
        );

        // File URLs - should remain unchanged
        assert_eq!(
            normalize_git_url("file:///local/repo.git").expect("should parse"),
            "file:///local/repo"
        );
        assert_eq!(
            normalize_git_url("file:///home/user/repos/project").expect("should parse"),
            "file:///home/user/repos/project"
        );
    }
}
