//! Path normalization and path-list matching for file and shell guards.
//!
//! See tool-guards-spec §6.5 for the full specification.

use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Compiled-in path lists (spec §6.5)
// ---------------------------------------------------------------------------

pub const SYSTEM_READ_PATHS: &[&str] = &["/proc/", "/sys/", "/dev/"];

pub const SENSITIVE_PATHS: &[&str] = &[
    "/etc/shadow",
    "/etc/gshadow",
    "/etc/sudoers",
    "~/.ssh/",
    "~/.gnupg/",
    "~/.aws/",
    "~/.config/gcloud/",
    ".env",
    ".netrc",
];

pub const SENSITIVE_WRITE_PATHS: &[&str] = &[
    "~/.ssh/",
    "~/.gnupg/",
    "~/.aws/",
    "~/.config/gcloud/",
    ".env",
    ".netrc",
    "~/.bashrc",
    "~/.bash_profile",
    "~/.profile",
    "~/.zshrc",
    "~/.zprofile",
];

pub const SYSTEM_WRITE_PATHS: &[&str] = &[
    "/etc/", "/usr/", "/boot/", "/sbin/", "/bin/", "/lib/", "/proc/", "/sys/", "/dev/",
];

pub const PROTECTED_DIR_COMPONENTS: &[&str] = &[".git"];

// ---------------------------------------------------------------------------
// Path normalization (§3.1)
// ---------------------------------------------------------------------------

/// Normalize a raw path string into an absolute `PathBuf`.
///
/// - Expands leading `~` or `~/` to `home_dir`.
/// - Resolves `..` components lexically (no filesystem access).
/// - Makes relative paths absolute against `working_dir`.
/// - On Windows: normalizes `\` to `/` and preserves drive letter prefix.
pub fn normalize_path(raw_path: &str, working_dir: &Path, home_dir: &Path) -> PathBuf {
    // Windows: normalize backslashes to forward slashes
    let normalized_slashes: std::borrow::Cow<str> = if raw_path.contains('\\') {
        std::borrow::Cow::Owned(raw_path.replace('\\', "/"))
    } else {
        std::borrow::Cow::Borrowed(raw_path)
    };
    let raw = normalized_slashes.as_ref();

    // Tilde expansion
    let expanded = if raw == "~" {
        home_dir.to_path_buf()
    } else if let Some(rest) = raw.strip_prefix("~/") {
        home_dir.join(rest)
    } else {
        PathBuf::from(raw)
    };

    // Make relative paths absolute
    let absolute = if expanded.is_relative() {
        working_dir.join(&expanded)
    } else {
        expanded
    };

    // Resolve `.` and `..` lexically
    resolve_lexical(&absolute)
}

/// Lexically resolve `.` and `..` components without filesystem access.
fn resolve_lexical(path: &Path) -> PathBuf {
    let mut components = Vec::new();

    for component in path.components() {
        match component {
            std::path::Component::CurDir => {
                // Skip `.`
            }
            std::path::Component::ParentDir => {
                // Pop the last normal component, but never pop past root/prefix
                match components.last() {
                    Some(std::path::Component::Normal(_)) => {
                        components.pop();
                    }
                    _ => {
                        // At root or prefix — ignore the `..`
                    }
                }
            }
            other => {
                components.push(other);
            }
        }
    }

    if components.is_empty() {
        PathBuf::from("/")
    } else {
        components.iter().collect()
    }
}

// ---------------------------------------------------------------------------
// Path matching (§6.5.1)
// ---------------------------------------------------------------------------

/// Check whether `normalized` matches any entry in `list`.
///
/// Three matching modes (determined by the entry's shape):
///
/// - **Exact:** entry contains at least one `/`, has no trailing `/`, and no
///   `*` → the normalized path must equal the expanded entry exactly.
/// - **Prefix:** entry ends with `/` → the normalized path's string
///   representation must start with the expanded entry.
/// - **Basename:** entry contains NO `/` (e.g., `.env`) → the last component
///   of the normalized path must equal the entry.
///
/// Entries starting with `~/` have `~` expanded to `home_dir` before matching.
pub fn path_matches_list(normalized: &Path, list: &[&str], home_dir: &Path) -> bool {
    let norm_str = normalized.to_string_lossy();

    for &entry in list {
        // Expand tilde in the entry
        let expanded: std::borrow::Cow<str> = if entry == "~" {
            std::borrow::Cow::Owned(home_dir.to_string_lossy().into_owned())
        } else if let Some(rest) = entry.strip_prefix("~/") {
            let mut s = home_dir.to_string_lossy().into_owned();
            s.push('/');
            s.push_str(rest);
            std::borrow::Cow::Owned(s)
        } else {
            std::borrow::Cow::Borrowed(entry)
        };

        if let Some(without_slash) = expanded.strip_suffix('/') {
            // Prefix mode
            if norm_str.starts_with(expanded.as_ref()) {
                return true;
            }
            // Also match the directory itself (without trailing slash)
            if *norm_str == *without_slash {
                return true;
            }
        } else if expanded.contains('/') {
            // Exact mode
            if *norm_str == *expanded {
                return true;
            }
        } else {
            // Basename mode: match last component
            if let Some(file_name) = normalized.file_name()
                && file_name == expanded.as_ref()
            {
                return true;
            }
        }
    }

    false
}

/// Check whether any component of `normalized` is a protected directory name.
///
/// Currently checks against `PROTECTED_DIR_COMPONENTS` (`.git`).
pub fn has_protected_component(normalized: &Path) -> bool {
    for component in normalized.components() {
        if let std::path::Component::Normal(name) = component {
            for &protected in PROTECTED_DIR_COMPONENTS {
                if name == protected {
                    return true;
                }
            }
        }
    }
    false
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn home() -> PathBuf {
        PathBuf::from("/home/testuser")
    }

    fn work() -> PathBuf {
        PathBuf::from("/home/testuser/project")
    }

    // -- normalize_path tests -----------------------------------------------

    #[test]
    fn tilde_expands_to_home() {
        assert_eq!(
            normalize_path("~", &work(), &home()),
            PathBuf::from("/home/testuser")
        );
    }

    #[test]
    fn tilde_slash_expands_to_home_subpath() {
        assert_eq!(
            normalize_path("~/documents/file.txt", &work(), &home()),
            PathBuf::from("/home/testuser/documents/file.txt")
        );
    }

    #[test]
    fn tilde_ssh_expands() {
        assert_eq!(
            normalize_path("~/.ssh/id_rsa", &work(), &home()),
            PathBuf::from("/home/testuser/.ssh/id_rsa")
        );
    }

    #[test]
    fn dotdot_resolved_lexically() {
        assert_eq!(
            normalize_path("/home/testuser/project/../other/file.txt", &work(), &home()),
            PathBuf::from("/home/testuser/other/file.txt")
        );
    }

    #[test]
    fn dotdot_does_not_escape_root() {
        assert_eq!(
            normalize_path("/../../etc/passwd", &work(), &home()),
            PathBuf::from("/etc/passwd")
        );
    }

    #[test]
    fn relative_path_resolved_against_working_dir() {
        assert_eq!(
            normalize_path("src/main.rs", &work(), &home()),
            PathBuf::from("/home/testuser/project/src/main.rs")
        );
    }

    #[test]
    fn relative_dotdot_resolved() {
        assert_eq!(
            normalize_path("../sibling/file.txt", &work(), &home()),
            PathBuf::from("/home/testuser/sibling/file.txt")
        );
    }

    #[test]
    fn dot_current_dir_resolved() {
        assert_eq!(
            normalize_path("./src/main.rs", &work(), &home()),
            PathBuf::from("/home/testuser/project/src/main.rs")
        );
    }

    #[test]
    fn absolute_path_unchanged() {
        assert_eq!(
            normalize_path("/etc/passwd", &work(), &home()),
            PathBuf::from("/etc/passwd")
        );
    }

    #[test]
    fn multiple_dotdot_components() {
        assert_eq!(
            normalize_path("a/b/c/../../d", &work(), &home()),
            PathBuf::from("/home/testuser/project/a/d")
        );
    }

    #[test]
    fn backslash_normalized_to_forward_slash() {
        assert_eq!(
            normalize_path("src\\lib\\module.rs", &work(), &home()),
            PathBuf::from("/home/testuser/project/src/lib/module.rs")
        );
    }

    // -- path_matches_list: exact match tests --------------------------------

    #[test]
    fn exact_match_etc_shadow() {
        let path = PathBuf::from("/etc/shadow");
        assert!(path_matches_list(&path, SENSITIVE_PATHS, &home()));
    }

    #[test]
    fn exact_match_etc_shadow_no_false_prefix() {
        let path = PathBuf::from("/etc/shadow.bak");
        assert!(!path_matches_list(&path, SENSITIVE_PATHS, &home()));
    }

    #[test]
    fn exact_match_tilde_expanded() {
        let path = PathBuf::from("/home/testuser/.bashrc");
        assert!(path_matches_list(&path, SENSITIVE_WRITE_PATHS, &home()));
    }

    // -- path_matches_list: prefix match tests --------------------------------

    #[test]
    fn prefix_match_proc() {
        let path = PathBuf::from("/proc/1/status");
        assert!(path_matches_list(&path, SYSTEM_READ_PATHS, &home()));
    }

    #[test]
    fn prefix_match_ssh_dir() {
        let path = PathBuf::from("/home/testuser/.ssh/id_rsa");
        assert!(path_matches_list(&path, SENSITIVE_PATHS, &home()));
    }

    #[test]
    fn prefix_match_ssh_dir_itself() {
        let path = PathBuf::from("/home/testuser/.ssh");
        assert!(path_matches_list(&path, SENSITIVE_PATHS, &home()));
    }

    #[test]
    fn prefix_match_etc_write() {
        let path = PathBuf::from("/etc/nginx/nginx.conf");
        assert!(path_matches_list(&path, SYSTEM_WRITE_PATHS, &home()));
    }

    #[test]
    fn prefix_no_false_match() {
        let path = PathBuf::from("/processing/data.txt");
        assert!(!path_matches_list(&path, SYSTEM_READ_PATHS, &home()));
    }

    // -- path_matches_list: basename match tests ------------------------------

    #[test]
    fn basename_match_dotenv() {
        let path = PathBuf::from("/home/testuser/project/.env");
        assert!(path_matches_list(&path, SENSITIVE_PATHS, &home()));
    }

    #[test]
    fn basename_match_dotenv_in_subdirectory() {
        let path = PathBuf::from("/home/testuser/project/subdir/.env");
        assert!(path_matches_list(&path, SENSITIVE_PATHS, &home()));
    }

    #[test]
    fn basename_match_netrc() {
        let path = PathBuf::from("/home/testuser/.netrc");
        assert!(path_matches_list(&path, SENSITIVE_PATHS, &home()));
    }

    #[test]
    fn basename_no_partial_match() {
        let path = PathBuf::from("/home/testuser/project/.env.example");
        assert!(!path_matches_list(&path, SENSITIVE_PATHS, &home()));
    }

    #[test]
    fn basename_no_match_different_name() {
        let path = PathBuf::from("/home/testuser/project/config.toml");
        assert!(!path_matches_list(&path, SENSITIVE_PATHS, &home()));
    }

    // -- has_protected_component tests ----------------------------------------

    #[test]
    fn git_hooks_is_protected() {
        let path = PathBuf::from("/home/testuser/project/.git/hooks/pre-commit");
        assert!(has_protected_component(&path));
    }

    #[test]
    fn git_dir_itself_is_protected() {
        let path = PathBuf::from("/home/testuser/project/.git");
        assert!(has_protected_component(&path));
    }

    #[test]
    fn git_config_is_protected() {
        let path = PathBuf::from("/home/testuser/project/.git/config");
        assert!(has_protected_component(&path));
    }

    #[test]
    fn no_git_component_not_protected() {
        let path = PathBuf::from("/home/testuser/project/git/something");
        assert!(!has_protected_component(&path));
    }

    #[test]
    fn gitignore_not_protected() {
        let path = PathBuf::from("/home/testuser/project/.gitignore");
        assert!(!has_protected_component(&path));
    }

    #[test]
    fn normal_path_not_protected() {
        let path = PathBuf::from("/home/testuser/project/src/main.rs");
        assert!(!has_protected_component(&path));
    }

    // -- Integration: normalize then match ------------------------------------

    #[test]
    fn normalized_relative_dotenv_matches() {
        let normalized = normalize_path("./subdir/.env", &work(), &home());
        assert!(path_matches_list(&normalized, SENSITIVE_PATHS, &home()));
    }

    #[test]
    fn normalized_tilde_ssh_matches() {
        let normalized = normalize_path("~/.ssh/id_rsa", &work(), &home());
        assert!(path_matches_list(&normalized, SENSITIVE_PATHS, &home()));
    }

    #[test]
    fn normalized_dotdot_into_etc_matches() {
        // From /home/testuser/project, four `..` levels reach root
        let normalized = normalize_path("../../../../etc/shadow", &work(), &home());
        assert_eq!(normalized, PathBuf::from("/etc/shadow"));
        assert!(path_matches_list(&normalized, SENSITIVE_PATHS, &home()));
    }

    #[test]
    fn normalized_git_hooks_protected() {
        let normalized = normalize_path(".git/hooks/pre-commit", &work(), &home());
        assert!(has_protected_component(&normalized));
    }

    #[test]
    fn normalized_system_write_path() {
        let normalized = normalize_path("/usr/local/bin/something", &work(), &home());
        assert!(path_matches_list(&normalized, SYSTEM_WRITE_PATHS, &home()));
    }
}
