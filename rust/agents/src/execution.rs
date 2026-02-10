//! Execution environment abstraction and local implementation (spec 4.1-4.2).
//!
//! All tool operations pass through [`ExecutionEnvironment`], decoupling tool
//! logic from where commands run. The default [`LocalExecutionEnvironment`]
//! runs on the local filesystem; swap in a different implementation for
//! Docker, Kubernetes, SSH, or WASM.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use async_trait::async_trait;
use tokio::io::AsyncReadExt;

use crate::error::{AgentError, AgentResult};
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
// LocalExecutionEnvironment (spec 4.2)
// ---------------------------------------------------------------------------

/// Runs tools on the local filesystem with the local shell.
pub struct LocalExecutionEnvironment {
    working_dir: PathBuf,
    env_policy: EnvVarPolicy,
}

impl std::fmt::Debug for LocalExecutionEnvironment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LocalExecutionEnvironment")
            .field("working_dir", &self.working_dir)
            .field("env_policy", &self.env_policy)
            .finish()
    }
}

impl LocalExecutionEnvironment {
    /// Create a new local execution environment rooted at `working_dir`.
    pub fn new(working_dir: impl Into<PathBuf>) -> Self {
        Self {
            working_dir: working_dir.into(),
            env_policy: EnvVarPolicy::default(),
        }
    }

    /// Set the environment variable policy.
    #[must_use]
    pub fn with_env_policy(mut self, policy: EnvVarPolicy) -> Self {
        self.env_policy = policy;
        self
    }

    /// Return the environment variable policy.
    #[must_use]
    pub fn env_policy(&self) -> &EnvVarPolicy {
        &self.env_policy
    }

    /// Resolve a potentially relative path against the working directory.
    fn resolve_path(&self, path: &str) -> PathBuf {
        let p = Path::new(path);
        if p.is_absolute() {
            p.to_path_buf()
        } else {
            self.working_dir.join(p)
        }
    }
}

#[async_trait]
impl ExecutionEnvironment for LocalExecutionEnvironment {
    async fn read_file(
        &self,
        path: &str,
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> AgentResult<FileContent> {
        let resolved = self.resolve_path(path);

        // Detect image files by extension
        if let Some(media_type) = image_media_type(&resolved) {
            let data = tokio::fs::read(&resolved)
                .await
                .map_err(|e| AgentError::from_io(e, &resolved))?;
            return Ok(FileContent::Image { data, media_type });
        }

        let raw = tokio::fs::read_to_string(&resolved)
            .await
            .map_err(|e| AgentError::from_io(e, &resolved))?;

        let start = offset.unwrap_or(1).saturating_sub(1); // 1-based → 0-based
        let max_lines = limit.unwrap_or(2000);

        let numbered = raw
            .lines()
            .enumerate()
            .skip(start)
            .take(max_lines)
            .map(|(i, line)| format!("{:>6} | {line}", i + 1))
            .collect::<Vec<_>>()
            .join("\n");

        Ok(FileContent::Text(numbered))
    }

    async fn write_file(&self, path: &str, content: &str) -> AgentResult<()> {
        let resolved = self.resolve_path(path);
        if let Some(parent) = resolved.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| AgentError::from_io(e, parent))?;
        }
        tokio::fs::write(&resolved, content)
            .await
            .map_err(|e| AgentError::from_io(e, &resolved))
    }

    async fn file_exists(&self, path: &str) -> bool {
        let resolved = self.resolve_path(path);
        tokio::fs::try_exists(&resolved).await.unwrap_or(false)
    }

    async fn delete_file(&self, path: &str) -> AgentResult<()> {
        let resolved = self.resolve_path(path);
        tokio::fs::remove_file(&resolved)
            .await
            .map_err(|e| AgentError::from_io(e, &resolved))
    }

    async fn list_directory(&self, path: &str, depth: usize) -> AgentResult<Vec<DirEntry>> {
        let resolved = self.resolve_path(path);
        let depth = depth.max(1);
        let mut entries = Vec::new();
        list_dir_recursive(&resolved, &resolved, depth, &mut entries).await?;
        Ok(entries)
    }

    async fn exec_command(
        &self,
        command: &str,
        timeout_ms: u64,
        working_dir: Option<&str>,
        env_vars: Option<&HashMap<String, String>>,
    ) -> AgentResult<ExecResult> {
        let cwd = working_dir
            .map(|d| self.resolve_path(d))
            .unwrap_or_else(|| self.working_dir.clone());

        let filtered = filter_env_vars(std::env::vars(), &self.env_policy);

        #[cfg(unix)]
        let mut cmd = {
            let mut c = tokio::process::Command::new("/bin/bash");
            c.arg("-c").arg(command);
            c
        };
        #[cfg(windows)]
        let mut cmd = {
            let mut c = tokio::process::Command::new("cmd.exe");
            c.arg("/c").arg(command);
            c
        };
        cmd.current_dir(&cwd);
        cmd.env_clear();
        for (k, v) in &filtered {
            cmd.env(k, v);
        }
        if let Some(extra) = env_vars {
            for (k, v) in extra {
                cmd.env(k, v);
            }
        }
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        // New process group for clean killability (Unix)
        #[cfg(unix)]
        cmd.process_group(0);

        let start = std::time::Instant::now();
        let mut child = cmd.spawn().map_err(|e| AgentError::Io {
            message: format!("failed to spawn command: {e}"),
        })?;

        let child_pid = child.id();

        // Take stdout/stderr handles; spawn reader tasks so they survive a
        // timeout cancellation.
        let mut stdout_pipe = child.stdout.take();
        let mut stderr_pipe = child.stderr.take();

        let stdout_task = tokio::spawn(async move {
            let mut buf = String::new();
            if let Some(ref mut out) = stdout_pipe {
                let _ = out.read_to_string(&mut buf).await;
            }
            buf
        });
        let stderr_task = tokio::spawn(async move {
            let mut buf = String::new();
            if let Some(ref mut err) = stderr_pipe {
                let _ = err.read_to_string(&mut buf).await;
            }
            buf
        });

        let timeout = std::time::Duration::from_millis(timeout_ms);
        let timed_out = match tokio::time::timeout(timeout, child.wait()).await {
            Ok(Ok(status)) => {
                let stdout = stdout_task.await.unwrap_or_default();
                let stderr = stderr_task.await.unwrap_or_default();
                let duration_ms = start.elapsed().as_millis() as u64;
                return Ok(ExecResult {
                    stdout,
                    stderr,
                    exit_code: status.code().unwrap_or(-1),
                    timed_out: false,
                    duration_ms,
                });
            }
            Ok(Err(e)) => {
                return Err(AgentError::Io {
                    message: format!("error waiting for process: {e}"),
                });
            }
            Err(_elapsed) => true,
        };

        // Timeout path: SIGTERM → 2 s wait → SIGKILL (spec 4.2 / 5.4)
        // Both signals target the entire process group so grandchildren are killed.
        if timed_out {
            #[cfg(unix)]
            if let Some(pid) = child_pid {
                sigterm_process_group(pid);
            }
            #[cfg(not(unix))]
            {
                let _ = child.start_kill();
            }

            tokio::time::sleep(std::time::Duration::from_secs(2)).await;

            #[cfg(unix)]
            if let Some(pid) = child_pid {
                sigkill_process_group(pid);
            }
            #[cfg(not(unix))]
            {
                let _ = child.start_kill();
            }

            let _ = child.wait().await; // reap
        }

        let stdout = stdout_task.await.unwrap_or_default();
        let mut stderr = stderr_task.await.unwrap_or_default();
        let duration_ms = start.elapsed().as_millis() as u64;

        // Spec 4.2 line 954: append timeout message so the LLM knows it can retry
        stderr.push_str(&format!(
            "\n[ERROR: Command timed out after {timeout_ms}ms. Partial output is shown above.\n\
             You can retry with a longer timeout by setting the timeout_ms parameter.]"
        ));

        Ok(ExecResult {
            stdout,
            stderr,
            exit_code: -1,
            timed_out: true,
            duration_ms,
        })
    }

    async fn grep(&self, pattern: &str, path: &str, options: &GrepOptions) -> AgentResult<String> {
        let resolved = self.resolve_path(path);
        let re = regex::RegexBuilder::new(pattern)
            .case_insensitive(options.case_insensitive)
            .build()
            .map_err(|e| AgentError::ValidationError {
                reason: e.to_string(),
            })?;

        let max = options.max_results as usize;
        let glob_filter = options.glob_filter.clone();
        let base = resolved.clone();

        // Run the I/O-heavy grep in a blocking task
        let results = tokio::task::spawn_blocking(move || {
            grep_recursive(&base, &re, glob_filter.as_deref(), max)
        })
        .await
        .map_err(|e| AgentError::Io {
            message: format!("grep task failed: {e}"),
        })?
        .map_err(|e| AgentError::from_io(e, &resolved))?;

        Ok(results.join("\n"))
    }

    async fn glob_files(&self, pattern: &str, path: &str) -> AgentResult<Vec<String>> {
        let resolved = self.resolve_path(path);

        // Spec: "errors: path not found"
        if !resolved.exists() {
            return Err(AgentError::FileNotFound {
                path: resolved.display().to_string(),
            });
        }

        let full_pattern = resolved.join(pattern);
        let pattern_str = full_pattern.to_string_lossy().to_string();

        let paths = tokio::task::spawn_blocking(move || -> AgentResult<Vec<String>> {
            let entries = glob::glob(&pattern_str).map_err(|e| AgentError::ValidationError {
                reason: e.to_string(),
            })?;

            // Collect paths with their modification times for sorting
            let mut with_mtime: Vec<(String, std::time::SystemTime)> = Vec::new();
            for entry in entries {
                match entry {
                    Ok(p) => {
                        let mtime = p
                            .metadata()
                            .and_then(|m| m.modified())
                            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                        with_mtime.push((p.to_string_lossy().to_string(), mtime));
                    }
                    Err(e) => {
                        return Err(AgentError::Io {
                            message: e.to_string(),
                        });
                    }
                }
            }

            // Spec: "sorted by modification time (newest first)"
            with_mtime.sort_by(|a, b| b.1.cmp(&a.1));

            Ok(with_mtime.into_iter().map(|(path, _)| path).collect())
        })
        .await
        .map_err(|e| AgentError::Io {
            message: format!("glob task failed: {e}"),
        })??;

        Ok(paths)
    }

    fn working_directory(&self) -> &str {
        self.working_dir.to_str().unwrap_or(".")
    }

    fn platform(&self) -> &str {
        // Spec 4.1 line 743: returns "darwin" (not std::env::consts::OS "macos")
        match std::env::consts::OS {
            "macos" => "darwin",
            other => other,
        }
    }

    fn os_version(&self) -> String {
        format!("{} {}", std::env::consts::OS, std::env::consts::ARCH)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Return the MIME type for image file extensions, or None for non-images.
fn image_media_type(path: &Path) -> Option<String> {
    let ext = path.extension()?.to_str()?.to_lowercase();
    match ext.as_str() {
        "png" => Some("image/png".to_string()),
        "jpg" | "jpeg" => Some("image/jpeg".to_string()),
        "gif" => Some("image/gif".to_string()),
        "webp" => Some("image/webp".to_string()),
        "svg" => Some("image/svg+xml".to_string()),
        _ => None,
    }
}

/// Recursively list directory entries up to `remaining_depth`.
async fn list_dir_recursive(
    base: &Path,
    dir: &Path,
    remaining_depth: usize,
    entries: &mut Vec<DirEntry>,
) -> AgentResult<()> {
    let mut read_dir = tokio::fs::read_dir(dir)
        .await
        .map_err(|e| AgentError::from_io(e, dir))?;

    while let Some(entry) = read_dir
        .next_entry()
        .await
        .map_err(|e| AgentError::from_io(e, dir))?
    {
        let metadata = entry
            .metadata()
            .await
            .map_err(|e| AgentError::from_io(e, &entry.path()))?;
        let is_dir = metadata.is_dir();

        // Build a name relative to the base directory
        let name = entry
            .path()
            .strip_prefix(base)
            .unwrap_or(&entry.path())
            .to_string_lossy()
            .to_string();

        let size = if is_dir { None } else { Some(metadata.len()) };

        entries.push(DirEntry { name, is_dir, size });

        if is_dir && remaining_depth > 1 {
            Box::pin(list_dir_recursive(
                base,
                &entry.path(),
                remaining_depth - 1,
                entries,
            ))
            .await?;
        }
    }

    Ok(())
}

/// Grep files under `base`, collecting up to `max` matches.
///
/// If `base` is a file, grep only that file. If a directory, recurse.
/// Returns an error if the path does not exist (spec: "path not found").
fn grep_recursive(
    base: &Path,
    re: &regex::Regex,
    glob_filter: Option<&str>,
    max: usize,
) -> Result<Vec<String>, std::io::Error> {
    if !base.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("path not found: {}", base.display()),
        ));
    }

    let mut results = Vec::new();

    if base.is_file() {
        // Single-file grep: ignore glob_filter (file was explicitly requested)
        grep_file(base, base.parent().unwrap_or(base), re, max, &mut results);
    } else {
        grep_walk(base, base, re, glob_filter, max, &mut results);
    }

    Ok(results)
}

/// Grep a single file, appending matches to `results`.
fn grep_file(path: &Path, root: &Path, re: &regex::Regex, max: usize, results: &mut Vec<String>) {
    let Ok(content) = std::fs::read_to_string(path) else {
        return; // skip binary/unreadable files
    };
    let relative = path.strip_prefix(root).unwrap_or(path).to_string_lossy();

    for (i, line) in content.lines().enumerate() {
        if results.len() >= max {
            return;
        }
        if re.is_match(line) {
            results.push(format!("{relative}:{}:{line}", i + 1));
        }
    }
}

fn grep_walk(
    root: &Path,
    dir: &Path,
    re: &regex::Regex,
    glob_filter: Option<&str>,
    max: usize,
    results: &mut Vec<String>,
) {
    let Ok(read_dir) = std::fs::read_dir(dir) else {
        return;
    };

    for entry in read_dir.flatten() {
        if results.len() >= max {
            return;
        }

        let path = entry.path();
        if path.is_dir() {
            grep_walk(root, &path, re, glob_filter, max, results);
            continue;
        }

        // Apply glob filter if provided
        if let Some(filter) = glob_filter {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let pattern = glob::Pattern::new(filter);
            if let Ok(ref pat) = pattern
                && !pat.matches(name)
            {
                continue;
            }
        }

        grep_file(&path, root, re, max, results);
    }
}

/// Send a signal to every process in a process group on Unix.
///
/// These are the only `unsafe` calls in the crate — isolated here so the rest
/// of the code remains safe. The calls are sound because `pid` comes from
/// `tokio::process::Child::id()` (a valid PID) and negating it targets the
/// process group per POSIX `kill(2)`.
#[cfg(unix)]
#[allow(unsafe_code)]
fn sigterm_process_group(pid: u32) {
    // SAFETY: `kill(-pid, SIGTERM)` sends SIGTERM to every process in the
    // group identified by `pid`. The PID is obtained from a just-spawned child.
    unsafe {
        libc::kill(-(pid as i32), libc::SIGTERM);
    }
}

#[cfg(unix)]
#[allow(unsafe_code)]
fn sigkill_process_group(pid: u32) {
    // SAFETY: same rationale as `sigterm_process_group` — targets the group.
    unsafe {
        libc::kill(-(pid as i32), libc::SIGKILL);
    }
}
