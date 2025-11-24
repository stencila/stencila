use std::{
    env,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use eyre::{OptionExt, Result, bail};
use reqwest::Url;
use sha2::{Digest, Sha256};

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
            origin: get_origin(&repo_root),
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
            origin: get_origin(&repo_root),
            path: Some(relative_path),
            commit: Some("dirty".into()),
        });
    }

    // File is tracked *and* clean – return the commit SHA when the file was last changed.
    let head_out = Command::new("git")
        .arg("-C")
        .arg(&repo_root)
        .args(["log", "-1", "--format=%H", "--", &relative_path])
        .output()?;

    if !head_out.status.success() {
        let message = String::from_utf8(head_out.stderr).unwrap_or_default();
        bail!("Unable to get commit SHA: {message}")
    }

    let commit = String::from_utf8(head_out.stdout)?.trim().to_string();

    Ok(GitInfo {
        origin: get_origin(&repo_root),
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

/// Validate that a file exists on the default branch of its Git repository
///
/// This is used to ensure that Stencila Cloud watches can successfully create
/// branches off the default branch that contain the file.
///
/// # Returns
///
/// * `Ok(())` if the file exists on the default branch
/// * `Err` if the file is not in a git repo, or doesn't exist on the default branch
pub fn validate_file_on_default_branch(path: &Path) -> Result<()> {
    let path_display = path.display();

    let path = path.canonicalize()?;

    // Get the repository root
    let repo_root = closest_git_repo(&path)?;

    // Get the relative path within the repo
    let relative_path = path
        .strip_prefix(&repo_root)?
        .to_str()
        .ok_or_eyre("Path is not valid UTF-8")?;

    // Get the default branch name
    let default_branch = get_default_branch(&repo_root)?;

    // Check if the file exists on the default branch using git ls-tree
    let output = Command::new("git")
        .arg("-C")
        .arg(&repo_root)
        .args([
            "ls-tree",
            "--name-only",
            &format!("origin/{default_branch}"),
            "--",
        ])
        .arg(relative_path)
        .output()?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to check file on default branch: {error}");
    }

    if output.stdout.is_empty() {
        bail!(
            "File `{path_display}` does not exist on the default branch `{default_branch}`.\n\
             This is necessary for watch syncs to work properly. Please commit and push the file to `{default_branch}` first.",
        );
    }

    Ok(())
}

/// Get the default branch name for a Git repository
///
/// Tries multiple methods to determine the default branch:
/// 1. `git symbolic-ref refs/remotes/origin/HEAD` (fastest)
/// 2. `git remote show origin` (slower but more reliable)
/// 3. Falls back to "main" if both fail
fn get_default_branch(repo_root: &Path) -> Result<String> {
    // Try symbolic-ref first (fast)
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(["symbolic-ref", "refs/remotes/origin/HEAD"])
        .output()?;

    if output.status.success() {
        let branch = String::from_utf8(output.stdout)?
            .trim()
            .strip_prefix("refs/remotes/origin/")
            .unwrap_or("main")
            .to_string();
        return Ok(branch);
    }

    // Fallback: try remote show origin (slower but more reliable)
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(["remote", "show", "origin"])
        .output()?;

    if output.status.success() {
        let output_str = String::from_utf8(output.stdout)?;
        for line in output_str.lines() {
            if let Some(branch) = line.trim().strip_prefix("HEAD branch: ") {
                return Ok(branch.to_string());
            }
        }
    }

    // Last fallback: assume "main"
    Ok("main".to_string())
}

/// Get the current branch name for a Git repository
///
/// # Arguments
///
/// * `path` - Optional path within a Git repository. If `None`, uses current directory.
///
/// # Returns
///
/// * `Some(branch_name)` if in a Git repository and on a branch
/// * `None` if not in a Git repository, git is not available, or in detached HEAD state
pub fn get_current_branch(path: Option<&Path>) -> Option<String> {
    // Determine the working directory
    let repo_root = if let Some(p) = path {
        closest_git_repo(p).ok()?
    } else {
        let current_dir = env::current_dir().ok()?;
        closest_git_repo(&current_dir).ok()?
    };

    // Check if git is available
    if which::which("git").is_err() {
        return None;
    }

    // Get the current branch name
    let output = Command::new("git")
        .arg("-C")
        .arg(&repo_root)
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let branch_name = String::from_utf8(output.stdout).ok()?.trim().to_string();

    // Return None if in detached HEAD state
    if branch_name == "HEAD" {
        None
    } else {
        Some(branch_name)
    }
}

/// Get the git remote origin URL for a repository
fn get_origin(repo_root: &Path) -> Option<String> {
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

/// Create a DNS-safe slug from a branch name for use as a subdomain
///
/// This function creates a deterministic, DNS-compatible slug from a Git branch name
/// that can be used as a subdomain for preview deployments. The subdomain format is
/// `{branchSlug}--{siteId}.stencila.site` where `siteId` is 10 characters, requiring
/// a maximum of 51 characters for the branch slug (63-char DNS limit - 10 - 2).
///
/// # Rules
///
/// * Converts to lowercase
/// * Replaces `/` with `-` (e.g., `feature/foo` → `feature-foo`)
/// * Replaces non-alphanumeric characters (except `-`) with `-`
/// * Collapses consecutive `-` into a single `-`
/// * Trims leading and trailing `-`
/// * If length > 51 characters:
///   - Truncates to 42 characters
///   - Appends `-` followed by first 8 hex characters of SHA256 hash
///   - Total length: 51 characters
///
/// # Arguments
///
/// * `branch_name` - The Git branch name to slugify
///
/// # Returns
///
/// A DNS-safe slug suitable for use as a subdomain (max 51 characters)
///
/// # Examples
///
/// ```
/// # use stencila_codec_utils::slugify_branch_name;
/// assert_eq!(slugify_branch_name("feature/new-api"), "feature-new-api");
/// assert_eq!(slugify_branch_name("FEATURE/FOO"), "feature-foo");
/// assert_eq!(slugify_branch_name("fix_bug#123"), "fix-bug-123");
/// ```
pub fn slugify_branch_name(branch_name: &str) -> String {
    // Step 1: Convert to lowercase
    let mut slug = branch_name.to_lowercase();

    // Step 2: Replace slashes and non-alphanumeric chars with hyphens
    slug = slug
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect();

    // Step 3: Collapse consecutive hyphens
    while slug.contains("--") {
        slug = slug.replace("--", "-");
    }

    // Step 4: Trim leading and trailing hyphens
    slug = slug.trim_matches('-').to_string();

    // Step 5: Handle length limit with deterministic hashing
    const MAX_LENGTH: usize = 51;
    const HASH_LENGTH: usize = 8;

    if slug.len() > MAX_LENGTH {
        // Create SHA256 hash of the original branch name for uniqueness
        let mut hasher = Sha256::new();
        hasher.update(branch_name.as_bytes());
        let hash_result = hasher.finalize();
        let hash_hex = format!("{:x}", hash_result);

        // We want: prefix + "-" + 8-char-hash = 51 chars total
        // So maximum prefix length is 51 - 1 - 8 = 42
        let max_prefix_length = MAX_LENGTH - 1 - HASH_LENGTH;

        // Truncate to max prefix length
        slug.truncate(max_prefix_length);

        // Trim trailing hyphens from truncated slug
        slug = slug.trim_end_matches('-').to_string();

        // Build final slug with hash
        slug.push('-');
        slug.push_str(&hash_hex[..HASH_LENGTH]);
    }

    slug
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

    #[test]
    fn test_slugify_branch_name_basic() {
        // Simple branch names
        assert_eq!(slugify_branch_name("main"), "main");
        assert_eq!(slugify_branch_name("develop"), "develop");
        assert_eq!(slugify_branch_name("master"), "master");
    }

    #[test]
    fn test_slugify_branch_name_with_slashes() {
        // Feature branches with slashes
        assert_eq!(slugify_branch_name("feature/new-api"), "feature-new-api");
        assert_eq!(slugify_branch_name("bugfix/fix-login"), "bugfix-fix-login");
        assert_eq!(slugify_branch_name("release/v1.2.3"), "release-v1-2-3");
    }

    #[test]
    fn test_slugify_branch_name_case_conversion() {
        // Uppercase to lowercase
        assert_eq!(slugify_branch_name("FEATURE/FOO"), "feature-foo");
        assert_eq!(slugify_branch_name("BugFix/Bar"), "bugfix-bar");
        assert_eq!(slugify_branch_name("Main"), "main");
    }

    #[test]
    fn test_slugify_branch_name_special_chars() {
        // Special characters replaced with hyphens
        assert_eq!(slugify_branch_name("fix_bug#123"), "fix-bug-123");
        assert_eq!(slugify_branch_name("feature@v2"), "feature-v2");
        assert_eq!(slugify_branch_name("test:branch"), "test-branch");
        assert_eq!(slugify_branch_name("my.branch"), "my-branch");
    }

    #[test]
    fn test_slugify_branch_name_consecutive_hyphens() {
        // Multiple consecutive hyphens collapsed
        assert_eq!(slugify_branch_name("feature--foo"), "feature-foo");
        assert_eq!(slugify_branch_name("test___bar"), "test-bar");
        assert_eq!(slugify_branch_name("a///b"), "a-b");
    }

    #[test]
    fn test_slugify_branch_name_trim_hyphens() {
        // Leading and trailing hyphens trimmed
        assert_eq!(slugify_branch_name("-feature-"), "feature");
        assert_eq!(slugify_branch_name("__test__"), "test");
        assert_eq!(slugify_branch_name("/branch/"), "branch");
    }

    #[test]
    fn test_slugify_branch_name_long_names() {
        // Test truncation with hash
        // 60 character branch name (exceeds 51 char limit)
        let long_branch = "feature/very-long-branch-name-that-exceeds-max-length-limit";
        let slug = slugify_branch_name(long_branch);

        // Should be at most 51 characters (may be slightly less if trailing hyphens trimmed)
        assert!(slug.len() <= 51);

        // Should end with hyphen followed by 8 hex chars
        assert!(slug.contains('-'));
        let parts: Vec<&str> = slug.rsplitn(2, '-').collect();
        assert_eq!(parts[0].len(), 8);
        assert!(parts[0].chars().all(|c| c.is_ascii_hexdigit()));

        // Should be deterministic
        let slug2 = slugify_branch_name(long_branch);
        assert_eq!(slug, slug2);
    }

    #[test]
    fn test_slugify_branch_name_deterministic_hashing() {
        // Different long branch names should produce different slugs
        let branch1 = "feature/very-long-branch-name-that-exceeds-maximum-length-allowed-for-dns";
        let branch2 = "feature/very-long-branch-name-that-exceeds-maximum-length-allowed-for-web";

        let slug1 = slugify_branch_name(branch1);
        let slug2 = slugify_branch_name(branch2);

        // Both should be at most 51 chars
        assert!(slug1.len() <= 51);
        assert!(slug2.len() <= 51);

        // Should have different hashes
        assert_ne!(slug1, slug2);

        // Same input should produce same output
        assert_eq!(slug1, slugify_branch_name(branch1));
        assert_eq!(slug2, slugify_branch_name(branch2));
    }

    #[test]
    fn test_slugify_branch_name_unicode() {
        // Unicode characters replaced with hyphens and trimmed
        assert_eq!(slugify_branch_name("feature/café"), "feature-caf");
        assert_eq!(slugify_branch_name("test-😀"), "test");
    }

    #[test]
    fn test_slugify_branch_name_edge_cases() {
        // Empty-ish inputs
        assert_eq!(slugify_branch_name("---"), "");
        assert_eq!(slugify_branch_name("///"), "");
        assert_eq!(slugify_branch_name("___"), "");

        // Single character
        assert_eq!(slugify_branch_name("a"), "a");
        assert_eq!(slugify_branch_name("1"), "1");

        // Numbers
        assert_eq!(slugify_branch_name("v1.2.3"), "v1-2-3");
        assert_eq!(slugify_branch_name("123"), "123");
    }

    #[test]
    fn test_get_current_branch() {
        // This test will only work if we're in a git repository
        // We'll test with the current directory
        if let Some(branch) = get_current_branch(None) {
            // Should return a non-empty string
            assert!(!branch.is_empty());
            // Should not be "HEAD" (detached state)
            assert_ne!(branch, "HEAD");
        }
        // If None is returned, we're either not in a git repo or in detached HEAD,
        // which is also a valid result
    }

    #[test]
    fn test_get_current_branch_with_path() {
        // Test with current file's directory
        let current_file = std::path::Path::new(file!());
        if let Some(parent) = current_file.parent()
            && let Some(branch) = get_current_branch(Some(parent))
        {
            assert!(!branch.is_empty());
            assert_ne!(branch, "HEAD");
        }
    }
}
