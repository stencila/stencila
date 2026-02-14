//! Scoped execution environment (spec 7.2 — working_dir enforcement).
//!
//! Wraps another [`ExecutionEnvironment`] and restricts all filesystem
//! operations to a subdirectory. Created when a subagent is spawned with
//! `working_dir`.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;

use super::{ExecutionEnvironment, FileContent, normalize_path, resolve_and_canonicalize};
use crate::error::{AgentError, AgentResult};
use crate::types::{DirEntry, ExecResult, GrepOptions};

// ---------------------------------------------------------------------------
// ScopedExecutionEnvironment (spec 7.2 — working_dir enforcement)
// ---------------------------------------------------------------------------

/// A wrapper that restricts all filesystem operations to a subdirectory.
///
/// Created when a subagent is spawned with `working_dir`. All path arguments
/// are validated against `scope_dir`; operations outside the scope return
/// [`AgentError::PermissionDenied`].
///
/// **Limitations:**
/// - `exec_command` enforces the working directory but cannot prevent
///   commands like `cat /etc/passwd` from accessing arbitrary paths. Full
///   shell sandboxing requires OS-level mechanisms (namespaces, seccomp).
/// - `list_directory`, `grep`, and `glob_files` validate the starting path
///   and post-filter results to remove entries whose real path (after
///   symlink resolution) falls outside the scope directory.
pub struct ScopedExecutionEnvironment {
    inner: Arc<dyn ExecutionEnvironment>,
    scope_dir: PathBuf,
    scope_dir_str: String,
}

impl std::fmt::Debug for ScopedExecutionEnvironment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScopedExecutionEnvironment")
            .field("scope_dir", &self.scope_dir)
            .finish_non_exhaustive()
    }
}

impl ScopedExecutionEnvironment {
    /// Create a scoped environment that restricts operations to `scope_dir`.
    ///
    /// If `scope_dir` is relative, it is resolved against the inner
    /// environment's working directory. The resolved scope directory must
    /// fall within (or equal) the inner environment's working directory,
    /// matching the spec's intent that `working_dir` is "a subdirectory
    /// to scope the agent to."
    pub fn new(inner: Arc<dyn ExecutionEnvironment>, scope_dir: &str) -> AgentResult<Self> {
        let canonical =
            resolve_and_canonicalize(Path::new(scope_dir), Path::new(inner.working_directory()));

        // Enforce that scope_dir is within the parent's working directory
        let parent_canonical = Path::new(inner.working_directory())
            .canonicalize()
            .unwrap_or_else(|_| normalize_path(Path::new(inner.working_directory())));
        if !canonical.starts_with(&parent_canonical) {
            return Err(AgentError::PermissionDenied {
                path: scope_dir.to_string(),
            });
        }

        let scope_dir_str = canonical.to_string_lossy().to_string();

        Ok(Self {
            inner,
            scope_dir: canonical,
            scope_dir_str,
        })
    }

    /// Validate that `path` falls within the scope directory.
    ///
    /// Relative paths are resolved against `self.scope_dir` (not the inner
    /// environment's working directory). Returns the absolute path as a
    /// `String` to pass to the inner environment.
    ///
    /// For paths that don't yet exist on disk (e.g. `write_file` targets),
    /// each existing ancestor is canonicalized to resolve symlinks, catching
    /// cases like `scope/symlink_to_outside/newfile.txt`.
    fn validate_and_resolve(&self, path: &str) -> AgentResult<String> {
        let canonical = resolve_and_canonicalize(Path::new(path), &self.scope_dir);

        if canonical.starts_with(&self.scope_dir) {
            Ok(canonical.to_string_lossy().to_string())
        } else {
            Err(AgentError::PermissionDenied {
                path: path.to_string(),
            })
        }
    }

    /// Check whether an already-resolved path falls within the scope.
    ///
    /// Uses [`resolve_and_canonicalize`] to resolve symlinks for existing
    /// paths, with lexical normalization as a fallback. This is the single
    /// point of truth for all post-filter containment checks.
    fn is_path_in_scope(&self, path: &Path) -> bool {
        let canonical = resolve_and_canonicalize(path, &self.scope_dir);
        canonical.starts_with(&self.scope_dir)
    }
}

#[async_trait]
impl ExecutionEnvironment for ScopedExecutionEnvironment {
    async fn read_file(
        &self,
        path: &str,
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> AgentResult<FileContent> {
        let validated = self.validate_and_resolve(path)?;
        self.inner.read_file(&validated, offset, limit).await
    }

    async fn write_file(&self, path: &str, content: &str) -> AgentResult<()> {
        let validated = self.validate_and_resolve(path)?;
        self.inner.write_file(&validated, content).await
    }

    async fn file_exists(&self, path: &str) -> bool {
        match self.validate_and_resolve(path) {
            Ok(validated) => self.inner.file_exists(&validated).await,
            Err(_) => false, // out-of-scope paths don't "exist"
        }
    }

    async fn delete_file(&self, path: &str) -> AgentResult<()> {
        let validated = self.validate_and_resolve(path)?;
        self.inner.delete_file(&validated).await
    }

    async fn list_directory(&self, path: &str, depth: usize) -> AgentResult<Vec<DirEntry>> {
        let validated = self.validate_and_resolve(path)?;
        let entries = self.inner.list_directory(&validated, depth).await?;
        // Post-filter: remove entries whose real path (after symlink
        // resolution) falls outside the scope directory.
        let validated_path = PathBuf::from(&validated);
        Ok(entries
            .into_iter()
            .filter(|entry| self.is_path_in_scope(&validated_path.join(&entry.name)))
            .collect())
    }

    async fn exec_command(
        &self,
        command: &str,
        timeout_ms: u64,
        working_dir: Option<&str>,
        env_vars: Option<&HashMap<String, String>>,
    ) -> AgentResult<ExecResult> {
        // Validate working_dir override if provided; default to scope_dir
        let validated_cwd = match working_dir {
            Some(dir) => Some(self.validate_and_resolve(dir)?),
            None => Some(self.scope_dir_str.clone()),
        };
        self.inner
            .exec_command(command, timeout_ms, validated_cwd.as_deref(), env_vars)
            .await
    }

    async fn grep(&self, pattern: &str, path: &str, options: &GrepOptions) -> AgentResult<String> {
        let validated = self.validate_and_resolve(path)?;
        let result = self.inner.grep(pattern, &validated, options).await?;
        // Post-filter grep results. Lines are formatted as
        // `{relative_path}:{line_number}:{content}`. The path may contain
        // `:<digits>:` (valid on Linux filenames), so `extract_grep_path`
        // uses the search directory to disambiguate via the filesystem.
        let validated_path = PathBuf::from(&validated);
        let filtered: Vec<&str> = result
            .lines()
            .filter(|line| {
                let file_part = extract_grep_path(line, Some(&validated_path));
                self.is_path_in_scope(&validated_path.join(file_part))
            })
            .collect();
        Ok(filtered.join("\n"))
    }

    async fn glob_files(&self, pattern: &str, path: &str) -> AgentResult<Vec<String>> {
        let validated = self.validate_and_resolve(path)?;
        let results = self.inner.glob_files(pattern, &validated).await?;
        // Post-filter: glob returns absolute paths — check each is in scope.
        Ok(results
            .into_iter()
            .filter(|p| self.is_path_in_scope(Path::new(p)))
            .collect())
    }

    fn working_directory(&self) -> &str {
        &self.scope_dir_str
    }

    fn platform(&self) -> &str {
        self.inner.platform()
    }

    fn os_version(&self) -> String {
        self.inner.os_version()
    }

    // initialize/cleanup are no-ops — inner is shared with parent
}

/// Extract the file path from a grep result line.
///
/// Grep lines are formatted as `{relative_path}:{line_number}:{content}`.
/// Since filenames can contain `:<digits>:` on Linux, the format is
/// inherently ambiguous. We collect every `:<digits>:` position and, when
/// a `search_dir` is provided, pick the first candidate whose path exists
/// as a file — this resolves the ambiguity via the filesystem. Without a
/// search directory (or when no candidate exists), falls back to the first
/// `:<digits>:` (left-to-right), which is correct for the common case
/// where filenames don't contain that pattern.
///
/// Falls back to the first colon if no `:<digits>:` pattern is found.
fn extract_grep_path<'a>(line: &'a str, search_dir: Option<&Path>) -> &'a str {
    let bytes = line.as_bytes();
    let mut candidates = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b':' {
            // Check if the segment after this colon is all digits followed
            // by another colon (i.e. a line-number field candidate).
            let rest = &bytes[i + 1..];
            let digit_len = rest.iter().take_while(|b| b.is_ascii_digit()).count();
            if digit_len > 0 && i + 1 + digit_len < bytes.len() && rest[digit_len] == b':' {
                candidates.push(i);
            }
        }
        i += 1;
    }

    // When a search directory is available, try each candidate left-to-right
    // and pick the first whose extracted path exists as a file.
    if let Some(base) = search_dir {
        for &pos in &candidates {
            let candidate = &line[..pos];
            if base.join(candidate).is_file() {
                return candidate;
            }
        }
    }

    // Fallback: first candidate (left-to-right) or first colon
    match candidates.first() {
        Some(&pos) => &line[..pos],
        None => line.split(':').next().unwrap_or(line),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Pure parsing tests (no filesystem, search_dir = None) ---

    #[test]
    fn extract_grep_path_simple() {
        assert_eq!(
            extract_grep_path("src/main.rs:42:fn main()", None),
            "src/main.rs"
        );
    }

    #[test]
    fn extract_grep_path_colon_in_filename() {
        assert_eq!(
            extract_grep_path("src/foo:bar.rs:10:let x = 1;", None),
            "src/foo:bar.rs"
        );
    }

    #[test]
    fn extract_grep_path_no_match_fallback() {
        // No :<digits>: pattern — falls back to first colon
        assert_eq!(
            extract_grep_path("no-line-number:content", None),
            "no-line-number"
        );
    }

    #[test]
    fn extract_grep_path_colon_in_content() {
        assert_eq!(extract_grep_path("file.rs:1:key: value", None), "file.rs");
    }

    // --- Filesystem-disambiguated tests ---

    #[test]
    fn extract_grep_path_digits_colon_in_filename() {
        // Filename contains :<digits>: — filesystem check finds the real file
        let dir = tempfile::tempdir().expect("tempdir");
        let file_path = dir.path().join("data:42:results.txt");
        std::fs::write(&file_path, "hello").expect("write");

        assert_eq!(
            extract_grep_path("data:42:results.txt:5:hello world", Some(dir.path())),
            "data:42:results.txt"
        );
    }

    #[test]
    fn extract_grep_path_traversal_in_content_with_digits() {
        // Content contains :<digits>: with path-traversal segments.
        // Without filesystem disambiguation this would extract
        // "file.rs:1:../../tmp" as the path, which could normalize
        // outside the scope. With a search_dir, the real file is found.
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("file.rs"), "x").expect("write");

        assert_eq!(
            extract_grep_path("file.rs:1:../../tmp:2:x", Some(dir.path())),
            "file.rs"
        );
    }
}
