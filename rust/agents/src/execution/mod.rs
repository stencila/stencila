//! Execution environment abstraction and implementations (spec 4.1-4.2).
//!
//! All tool operations pass through [`ExecutionEnvironment`], decoupling tool
//! logic from where commands run. The default [`LocalExecutionEnvironment`]
//! runs on the local filesystem; swap in a different implementation for
//! Docker, Kubernetes, SSH, or WASM. [`ScopedExecutionEnvironment`] wraps
//! another environment to restrict operations to a subdirectory.

mod local;
mod scoped;

pub use local::LocalExecutionEnvironment;
pub use scoped::ScopedExecutionEnvironment;

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use async_trait::async_trait;

use crate::error::AgentResult;
use crate::types::{DirEntry, ExecResult, GrepOptions};

// ---------------------------------------------------------------------------
// FileContent (spec 3.3 line 500 — multimodal support)
// ---------------------------------------------------------------------------

/// Content returned by [`ExecutionEnvironment::read_file`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileContent {
    /// Line-numbered text in `NNN | content` format.
    Text(String),
    /// Raw image bytes with a detected MIME type.
    Image { data: Vec<u8>, media_type: String },
}

// ---------------------------------------------------------------------------
// EnvVarPolicy (spec 4.2)
// ---------------------------------------------------------------------------

/// How child processes inherit environment variables.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EnvVarPolicy {
    /// Inherit all vars except those matching the denylist.
    /// Allowlist takes precedence over denylist.
    #[default]
    InheritFiltered,
    /// Start clean — only allowlist vars are inherited.
    InheritNone,
    /// Inherit everything without any filtering.
    InheritAll,
}

// ---------------------------------------------------------------------------
// Env var filtering constants (spec 4.2)
// ---------------------------------------------------------------------------

/// Suffixes that cause a variable to be denied (case-insensitive).
const DENY_SUFFIXES: &[&str] = &["_API_KEY", "_SECRET", "_TOKEN", "_PASSWORD", "_CREDENTIAL"];

/// Variables that are always inherited regardless of denylist.
static ALLOWLIST: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    HashSet::from([
        "PATH",
        "HOME",
        "USER",
        "SHELL",
        "LANG",
        "TERM",
        "TMPDIR",
        "GOPATH",
        "CARGO_HOME",
        "NVM_DIR",
    ])
});

/// Filter environment variables according to the given policy.
///
/// Takes an iterator of `(key, value)` pairs (instead of reading
/// `std::env::vars()` directly) so callers can pass controlled data in tests.
#[must_use]
pub fn filter_env_vars(
    vars: impl Iterator<Item = (String, String)>,
    policy: &EnvVarPolicy,
) -> HashMap<String, String> {
    match policy {
        EnvVarPolicy::InheritAll => vars.collect(),
        EnvVarPolicy::InheritNone => vars
            .filter(|(k, _)| ALLOWLIST.contains(k.as_str()))
            .collect(),
        EnvVarPolicy::InheritFiltered => vars
            .filter(|(k, _)| {
                if ALLOWLIST.contains(k.as_str()) {
                    return true;
                }
                let upper = k.to_uppercase();
                !DENY_SUFFIXES.iter().any(|suffix| upper.ends_with(suffix))
            })
            .collect(),
    }
}

// ---------------------------------------------------------------------------
// ExecutionEnvironment trait (spec 4.1)
// ---------------------------------------------------------------------------

/// Abstraction for where tool operations run.
///
/// The local implementation ([`LocalExecutionEnvironment`]) accesses the
/// filesystem directly. Other implementations can target Docker containers,
/// Kubernetes pods, SSH hosts, or WASM runtimes.
#[async_trait]
pub trait ExecutionEnvironment: Send + Sync {
    // -- File operations --

    /// Read a file, returning line-numbered text or raw image data.
    ///
    /// * `offset` — 1-based starting line (default: 1).
    /// * `limit`  — max lines to return (default: 2000).
    async fn read_file(
        &self,
        path: &str,
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> AgentResult<FileContent>;

    /// Write `content` to `path`, creating parent directories as needed.
    async fn write_file(&self, path: &str, content: &str) -> AgentResult<()>;

    /// Check whether `path` exists.
    async fn file_exists(&self, path: &str) -> bool;

    /// Delete a file at `path`.
    async fn delete_file(&self, path: &str) -> AgentResult<()>;

    /// List directory entries up to the given `depth` (1 = immediate children).
    async fn list_directory(&self, path: &str, depth: usize) -> AgentResult<Vec<DirEntry>>;

    // -- Command execution --

    /// Execute a shell command with a timeout.
    ///
    /// On timeout: SIGTERM → 2 s wait → SIGKILL (Unix). Partial output is
    /// returned with `timed_out: true`.
    async fn exec_command(
        &self,
        command: &str,
        timeout_ms: u64,
        working_dir: Option<&str>,
        env_vars: Option<&HashMap<String, String>>,
    ) -> AgentResult<ExecResult>;

    // -- Search operations --

    /// Regex search across files, returning formatted matches.
    async fn grep(&self, pattern: &str, path: &str, options: &GrepOptions) -> AgentResult<String>;

    /// Glob pattern matching, returning matched file paths.
    async fn glob_files(&self, pattern: &str, path: &str) -> AgentResult<Vec<String>>;

    // -- Metadata --

    /// The working directory for this environment.
    fn working_directory(&self) -> &str;

    /// Platform identifier (`linux`, `darwin`, `windows`, `wasm`).
    fn platform(&self) -> &str;

    /// OS version string.
    fn os_version(&self) -> String;

    // -- Lifecycle (default no-ops) --

    /// Initialize the environment (e.g., start a container).
    async fn initialize(&self) -> AgentResult<()> {
        Ok(())
    }

    /// Clean up the environment (e.g., stop a container).
    async fn cleanup(&self) -> AgentResult<()> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Path helpers
// ---------------------------------------------------------------------------

/// Resolve a potentially relative path against `base`, then canonicalize.
///
/// If the full path exists, [`std::fs::canonicalize`] resolves symlinks.
/// Otherwise, the longest existing ancestor prefix is canonicalized (to
/// catch symlinks in intermediate directories) and the remaining
/// non-existent tail is appended after lexical normalization.
pub(crate) fn resolve_and_canonicalize(path: &Path, base: &Path) -> PathBuf {
    let resolved = if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    };

    // Fast path: full path exists — canonicalize resolves all symlinks.
    if let Ok(c) = resolved.canonicalize() {
        return c;
    }

    // Slow path: find the deepest existing ancestor, canonicalize it,
    // then append the remaining (non-existent) components with lexical
    // normalization. This catches symlinks in existing parent dirs
    // (e.g. `scope/link_to_outside/new_file.txt`).
    let mut existing = resolved.as_path();
    let mut tail_components = Vec::new();
    loop {
        if existing.exists() {
            break;
        }
        if let Some(file_name) = existing.file_name() {
            tail_components.push(file_name.to_os_string());
            existing = existing.parent().unwrap_or(Path::new("/"));
        } else {
            break;
        }
    }

    let base_canonical = existing
        .canonicalize()
        .unwrap_or_else(|_| normalize_path(existing));

    // Re-append non-existent tail with lexical normalization
    let mut result = base_canonical;
    for component in tail_components.into_iter().rev() {
        let c = Path::new(&component);
        if c == Path::new(".") {
            continue;
        } else if c == Path::new("..") {
            result.pop();
        } else {
            result.push(c);
        }
    }
    result
}

/// Lexically normalize a path by collapsing `.` and `..` components.
///
/// Unlike [`std::fs::canonicalize`], this does not touch the filesystem and
/// works for paths that don't exist yet (e.g., `write_file` targets).
/// It does **not** resolve symlinks.
pub fn normalize_path(path: &Path) -> PathBuf {
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {} // skip `.`
            std::path::Component::ParentDir => {
                // Pop unless we're at a root or the stack is empty/relative
                if let Some(last) = components.last() {
                    match last {
                        std::path::Component::RootDir | std::path::Component::Prefix(_) => {
                            // Can't go above root — just ignore the `..`
                        }
                        std::path::Component::ParentDir => {
                            // Already have unresolved `..`, push another
                            components.push(component);
                        }
                        _ => {
                            components.pop();
                        }
                    }
                } else {
                    // Relative path going above start — keep the `..`
                    components.push(component);
                }
            }
            _ => components.push(component),
        }
    }
    if components.is_empty() {
        PathBuf::from(".")
    } else {
        components.iter().collect()
    }
}
