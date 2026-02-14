//! Local execution environment (spec 4.2).
//!
//! Runs tool operations on the local filesystem with the local shell.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use tokio::io::AsyncReadExt;

use super::{EnvVarPolicy, ExecutionEnvironment, FileContent, filter_env_vars};
use crate::error::{AgentError, AgentResult};
use crate::types::{DirEntry, ExecResult, GrepOptions};

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

        let start = offset.unwrap_or(1).saturating_sub(1); // 1-based -> 0-based
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
        let base = resolved.clone();

        // Run directory listing in a blocking task since ignore::Walk is sync
        let entries = tokio::task::spawn_blocking(move || list_dir_walk(&base, depth))
            .await
            .map_err(|e| AgentError::Io {
                message: format!("list_directory task failed: {e}"),
            })?
            .map_err(|e| AgentError::from_io(e, &resolved))?;

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

        // Ensure child process is killed when its handle is dropped (e.g.
        // when an abort signal cancels the owning future via tokio::select!).
        cmd.kill_on_drop(true);

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

        // Timeout path: SIGTERM -> 2 s wait -> SIGKILL (spec 4.2 / 5.4)
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

        // Validate glob filter upfront (fixes P2: invalid glob silently ignored)
        let glob_filter = match &options.glob_filter {
            Some(filter) => {
                let pat = glob::Pattern::new(filter).map_err(|e| AgentError::ValidationError {
                    reason: format!("invalid glob_filter: {e}"),
                })?;
                Some(pat)
            }
            None => None,
        };

        let max = options.max_results as usize;
        let base = resolved.clone();

        // Run the I/O-heavy grep in a blocking task
        let results = tokio::task::spawn_blocking(move || {
            grep_recursive(&base, &re, glob_filter.as_ref(), max)
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
// Private helpers
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

/// List directory entries up to `depth` using `ignore::WalkBuilder`.
///
/// Uses `ignore::WalkBuilder` for traversal with built-in symlink cycle
/// detection (fixes P0: unbounded recursive directory listing through
/// symlink cycles).
fn list_dir_walk(base: &Path, depth: usize) -> Result<Vec<DirEntry>, std::io::Error> {
    if !base.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("path not found: {}", base.display()),
        ));
    }
    if !base.is_dir() {
        return Err(std::io::Error::other(format!(
            "not a directory: {}",
            base.display()
        )));
    }

    let walker = ignore::WalkBuilder::new(base)
        .standard_filters(false)
        .max_depth(Some(depth))
        .build();

    let mut entries = Vec::new();
    for entry in walker {
        let entry = match entry {
            Ok(e) => e,
            Err(err) => {
                // Propagate errors for the root path (e.g. permission denied);
                // skip errors in subdirectories.
                if let Some(io_err) = walk_error_for_root(&err) {
                    return Err(io_err);
                }
                continue;
            }
        };

        // Skip the root directory itself
        if entry.path() == base {
            continue;
        }

        let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);

        let name = entry
            .path()
            .strip_prefix(base)
            .unwrap_or(entry.path())
            .to_string_lossy()
            .to_string();

        let size = if is_dir {
            None
        } else {
            entry.metadata().ok().map(|m| m.len())
        };

        entries.push(DirEntry { name, is_dir, size });
    }

    Ok(entries)
}

/// If `err` is a walker error at depth 0 (the root), return an `io::Error`.
///
/// Root-level errors (e.g. permission denied on the search root) are
/// propagated to the caller. Errors in subdirectories return `None` and
/// should be skipped.
fn walk_error_for_root(err: &ignore::Error) -> Option<std::io::Error> {
    if err.depth() == Some(0) {
        let kind = err
            .io_error()
            .map(|e| e.kind())
            .unwrap_or(std::io::ErrorKind::Other);
        Some(std::io::Error::new(kind, err.to_string()))
    } else {
        None
    }
}

/// Grep files under `base`, collecting up to `max` matches.
///
/// If `base` is a file, grep only that file. If a directory, walk using
/// `ignore::WalkBuilder` which handles symlink cycle detection.
/// Returns an error if the path does not exist (spec: "path not found").
fn grep_recursive(
    base: &Path,
    re: &regex::Regex,
    glob_filter: Option<&glob::Pattern>,
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
        // Use ignore::WalkBuilder for directory traversal with symlink cycle
        // detection (fixes P0: unbounded recursive grep through symlink cycles)
        let walker = ignore::WalkBuilder::new(base)
            .standard_filters(false)
            .build();

        for entry in walker {
            if results.len() >= max {
                break;
            }

            let entry = match entry {
                Ok(e) => e,
                Err(err) => {
                    // Propagate errors for the root path; skip subdirectory errors.
                    if let Some(io_err) = walk_error_for_root(&err) {
                        return Err(io_err);
                    }
                    continue;
                }
            };

            // Skip directories
            let Some(file_type) = entry.file_type() else {
                continue;
            };
            if file_type.is_dir() {
                continue;
            }

            let path = entry.path();

            // Apply glob filter against relative path (not just basename)
            if let Some(pat) = glob_filter {
                let relative = path.strip_prefix(base).unwrap_or(path).to_string_lossy();
                if !pat.matches(&relative) {
                    continue;
                }
            }

            grep_file(path, base, re, max, &mut results);
        }
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
