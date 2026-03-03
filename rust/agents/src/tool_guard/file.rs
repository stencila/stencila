//! File tool guard: path-based risk checks for file read/write tools.

use std::path::{Path, PathBuf};

use serde_json::Value;

use super::paths::{
    self, SENSITIVE_PATHS, SENSITIVE_WRITE_PATHS, SYSTEM_READ_PATHS, SYSTEM_WRITE_PATHS,
};
use super::{GuardVerdict, TrustLevel};

/// Number of `DeleteFile` operations in a single `apply_patch` that triggers
/// the bulk-deletion rule.
const APPLY_PATCH_DELETE_THRESHOLD: usize = 5;

// ---------------------------------------------------------------------------
// Read rule strings
// ---------------------------------------------------------------------------

const SYSTEM_READ_REASON: &str = "Reading virtual/device filesystem paths can expose kernel state, process internals, and credentials (e.g., /proc/self/environ)";
const SYSTEM_READ_SUGGESTION: &str = "Use specific inspection commands instead (e.g., `uname` for system info, `env` for environment)";

const SENSITIVE_READ_REASON: &str = "Reading credential and key files can leak secrets into the agent's context window";
const SENSITIVE_READ_SUGGESTION: &str = "Use targeted commands that don't expose raw secrets (e.g., `ssh-keygen -l -f` to check a key fingerprint)";

const OUTSIDE_WORKSPACE_READ_REASON: &str = "Read target is outside the session workspace root";
const OUTSIDE_WORKSPACE_READ_SUGGESTION: &str =
    "Verify the path is intended, or copy the file into the workspace first";

// ---------------------------------------------------------------------------
// Write rule strings
// ---------------------------------------------------------------------------

const SYSTEM_WRITE_REASON: &str = "Writing to system paths can break OS configuration and stability";
const SYSTEM_WRITE_SUGGESTION: &str =
    "Use application-level config files in the project directory instead";

const SENSITIVE_WRITE_REASON: &str = "Writing to credential files or shell startup files is a persistence and credential-tampering vector";
const SENSITIVE_WRITE_SUGGESTION: &str =
    "Modify project-local configuration instead of user-level dotfiles";

const OUTSIDE_WORKSPACE_WRITE_REASON: &str = "Write target is outside the session workspace root";
const OUTSIDE_WORKSPACE_WRITE_SUGGESTION: &str =
    "Write to a path within the project workspace, or verify the target path is intended";

const PROTECTED_OVERWRITE_REASON: &str =
    "Writing to .git/ internals can corrupt repository state";
const PROTECTED_OVERWRITE_SUGGESTION: &str =
    "Edit hooks or config via `git config` or manual review outside the agent";

const DELETE_MANY_REASON: &str =
    "Bulk file deletion in a single patch may indicate a hallucinated cleanup";
const DELETE_MANY_SUGGESTION: &str = "Break the patch into smaller steps deleting fewer than 5 files each, or verify the file list is correct";

// ---------------------------------------------------------------------------
// FileToolGuard
// ---------------------------------------------------------------------------

pub struct FileToolGuard {
    workspace_root: PathBuf,
    home_dir: PathBuf,
}

impl FileToolGuard {
    pub fn new(workspace_root: PathBuf) -> Self {
        let home_dir = std::env::var("HOME")
            .ok()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/"));
        Self {
            workspace_root,
            home_dir,
        }
    }

    #[cfg(test)]
    pub fn new_with_home(workspace_root: PathBuf, home_dir: PathBuf) -> Self {
        Self {
            workspace_root,
            home_dir,
        }
    }

    pub fn evaluate(
        &self,
        tool_name: &str,
        args: &Value,
        working_dir: &Path,
        trust_level: TrustLevel,
    ) -> GuardVerdict {
        match tool_name {
            "read_file" => {
                let path = match args.get("file_path").and_then(|v| v.as_str()) {
                    Some(p) => p,
                    None => return GuardVerdict::Allow,
                };
                self.evaluate_read(path, working_dir, trust_level)
            }
            "read_many_files" => {
                let paths = match args.get("paths").and_then(|v| v.as_array()) {
                    Some(arr) => arr,
                    None => return GuardVerdict::Allow,
                };
                let mut strictest = GuardVerdict::Allow;
                for p in paths {
                    if let Some(s) = p.as_str() {
                        strictest =
                            strictest_verdict(strictest, self.evaluate_read(s, working_dir, trust_level));
                        if matches!(strictest, GuardVerdict::Deny { .. }) {
                            return strictest;
                        }
                    }
                }
                strictest
            }
            "write_file" | "edit_file" => {
                let path = match args.get("file_path").and_then(|v| v.as_str()) {
                    Some(p) => p,
                    None => return GuardVerdict::Allow,
                };
                self.evaluate_write(path, working_dir, trust_level, 0)
            }
            "apply_patch" => self.evaluate_apply_patch(args, working_dir, trust_level),
            "grep" => {
                let path = args
                    .get("path")
                    .and_then(|v| v.as_str())
                    .unwrap_or_else(|| working_dir.to_str().unwrap_or("."));
                self.evaluate_read(path, working_dir, trust_level)
            }
            _ => GuardVerdict::Allow,
        }
    }

    fn evaluate_read(
        &self,
        raw_path: &str,
        working_dir: &Path,
        trust_level: TrustLevel,
    ) -> GuardVerdict {
        let normalized = paths::normalize_path(raw_path, working_dir, &self.home_dir);
        let mut strictest = GuardVerdict::Allow;

        // Rule: file.system_path_read — Deny/Deny/Deny
        if paths::path_matches_list(
            &normalized,
            SYSTEM_READ_PATHS,
            &self.home_dir,
        ) {
            let verdict = match trust_level {
                TrustLevel::Low | TrustLevel::Medium | TrustLevel::High => GuardVerdict::Deny {
                    reason: SYSTEM_READ_REASON,
                    suggestion: SYSTEM_READ_SUGGESTION,
                    rule_id: "file.system_path_read",
                },
            };
            strictest = strictest_verdict(strictest, verdict);
            if matches!(strictest, GuardVerdict::Deny { .. }) {
                return strictest;
            }
        }

        // Rule: file.sensitive_path_read — Deny/Deny/Warn
        if paths::path_matches_list(
            &normalized,
            SENSITIVE_PATHS,
            &self.home_dir,
        ) {
            let verdict = match trust_level {
                TrustLevel::Low | TrustLevel::Medium => GuardVerdict::Deny {
                    reason: SENSITIVE_READ_REASON,
                    suggestion: SENSITIVE_READ_SUGGESTION,
                    rule_id: "file.sensitive_path_read",
                },
                TrustLevel::High => GuardVerdict::Warn {
                    reason: SENSITIVE_READ_REASON,
                    suggestion: SENSITIVE_READ_SUGGESTION,
                    rule_id: "file.sensitive_path_read",
                },
            };
            strictest = strictest_verdict(strictest, verdict);
            if matches!(strictest, GuardVerdict::Deny { .. }) {
                return strictest;
            }
        }

        // Rule: file.outside_workspace_read — Deny/Warn/Allow
        if !normalized.starts_with(&self.workspace_root) {
            let verdict = match trust_level {
                TrustLevel::Low => GuardVerdict::Deny {
                    reason: OUTSIDE_WORKSPACE_READ_REASON,
                    suggestion: OUTSIDE_WORKSPACE_READ_SUGGESTION,
                    rule_id: "file.outside_workspace_read",
                },
                TrustLevel::Medium => GuardVerdict::Warn {
                    reason: OUTSIDE_WORKSPACE_READ_REASON,
                    suggestion: OUTSIDE_WORKSPACE_READ_SUGGESTION,
                    rule_id: "file.outside_workspace_read",
                },
                TrustLevel::High => GuardVerdict::Allow,
            };
            strictest = strictest_verdict(strictest, verdict);
        }

        strictest
    }

    fn evaluate_write(
        &self,
        raw_path: &str,
        working_dir: &Path,
        trust_level: TrustLevel,
        delete_count: usize,
    ) -> GuardVerdict {
        let normalized = paths::normalize_path(raw_path, working_dir, &self.home_dir);
        self.evaluate_write_normalized(&normalized, trust_level, delete_count)
    }

    fn evaluate_write_normalized(
        &self,
        normalized: &Path,
        trust_level: TrustLevel,
        delete_count: usize,
    ) -> GuardVerdict {
        let mut strictest = GuardVerdict::Allow;

        // Rule: file.system_path_write — Deny/Deny/Deny
        if paths::path_matches_list(
            normalized,
            SYSTEM_WRITE_PATHS,
            &self.home_dir,
        ) {
            let verdict = match trust_level {
                TrustLevel::Low | TrustLevel::Medium | TrustLevel::High => GuardVerdict::Deny {
                    reason: SYSTEM_WRITE_REASON,
                    suggestion: SYSTEM_WRITE_SUGGESTION,
                    rule_id: "file.system_path_write",
                },
            };
            strictest = strictest_verdict(strictest, verdict);
            if matches!(strictest, GuardVerdict::Deny { .. }) {
                return strictest;
            }
        }

        // Rule: file.sensitive_path_write — Deny/Deny/Deny
        if paths::path_matches_list(
            normalized,
            SENSITIVE_WRITE_PATHS,
            &self.home_dir,
        ) {
            let verdict = match trust_level {
                TrustLevel::Low | TrustLevel::Medium | TrustLevel::High => GuardVerdict::Deny {
                    reason: SENSITIVE_WRITE_REASON,
                    suggestion: SENSITIVE_WRITE_SUGGESTION,
                    rule_id: "file.sensitive_path_write",
                },
            };
            strictest = strictest_verdict(strictest, verdict);
            if matches!(strictest, GuardVerdict::Deny { .. }) {
                return strictest;
            }
        }

        // Rule: file.outside_workspace_write — Deny/Warn/Allow
        if !normalized.starts_with(&self.workspace_root) {
            let verdict = match trust_level {
                TrustLevel::Low => GuardVerdict::Deny {
                    reason: OUTSIDE_WORKSPACE_WRITE_REASON,
                    suggestion: OUTSIDE_WORKSPACE_WRITE_SUGGESTION,
                    rule_id: "file.outside_workspace_write",
                },
                TrustLevel::Medium => GuardVerdict::Warn {
                    reason: OUTSIDE_WORKSPACE_WRITE_REASON,
                    suggestion: OUTSIDE_WORKSPACE_WRITE_SUGGESTION,
                    rule_id: "file.outside_workspace_write",
                },
                TrustLevel::High => GuardVerdict::Allow,
            };
            strictest = strictest_verdict(strictest, verdict);
            if matches!(strictest, GuardVerdict::Deny { .. }) {
                return strictest;
            }
        }

        // Rule: file.protected_file_overwrite — Deny/Deny/Warn
        if paths::has_protected_component(normalized) {
            let verdict = match trust_level {
                TrustLevel::Low | TrustLevel::Medium => GuardVerdict::Deny {
                    reason: PROTECTED_OVERWRITE_REASON,
                    suggestion: PROTECTED_OVERWRITE_SUGGESTION,
                    rule_id: "file.protected_file_overwrite",
                },
                TrustLevel::High => GuardVerdict::Warn {
                    reason: PROTECTED_OVERWRITE_REASON,
                    suggestion: PROTECTED_OVERWRITE_SUGGESTION,
                    rule_id: "file.protected_file_overwrite",
                },
            };
            strictest = strictest_verdict(strictest, verdict);
            if matches!(strictest, GuardVerdict::Deny { .. }) {
                return strictest;
            }
        }

        // Rule: file.apply_patch_delete_many — Deny/Warn/Warn
        if delete_count >= APPLY_PATCH_DELETE_THRESHOLD {
            let verdict = match trust_level {
                TrustLevel::Low => GuardVerdict::Deny {
                    reason: DELETE_MANY_REASON,
                    suggestion: DELETE_MANY_SUGGESTION,
                    rule_id: "file.apply_patch_delete_many",
                },
                TrustLevel::Medium | TrustLevel::High => GuardVerdict::Warn {
                    reason: DELETE_MANY_REASON,
                    suggestion: DELETE_MANY_SUGGESTION,
                    rule_id: "file.apply_patch_delete_many",
                },
            };
            strictest = strictest_verdict(strictest, verdict);
        }

        strictest
    }

    fn evaluate_apply_patch(
        &self,
        args: &Value,
        working_dir: &Path,
        trust_level: TrustLevel,
    ) -> GuardVerdict {
        let patch_text = match args.get("patch").and_then(|v| v.as_str()) {
            Some(t) => t,
            None => return GuardVerdict::Allow,
        };

        let (write_paths, delete_count) = parse_patch_paths(patch_text);

        if write_paths.is_empty() {
            return GuardVerdict::Allow;
        }

        let mut strictest = GuardVerdict::Allow;
        for raw_path in &write_paths {
            let normalized = paths::normalize_path(raw_path, working_dir, &self.home_dir);
            let verdict =
                self.evaluate_write_normalized(&normalized, trust_level, delete_count);
            strictest = strictest_verdict(strictest, verdict);
            if matches!(strictest, GuardVerdict::Deny { .. }) {
                return strictest;
            }
        }

        strictest
    }
}

// ---------------------------------------------------------------------------
// Patch parsing
// ---------------------------------------------------------------------------

fn parse_patch_paths(patch_text: &str) -> (Vec<String>, usize) {
    let mut paths = Vec::new();
    let mut delete_count: usize = 0;

    for line in patch_text.lines() {
        let trimmed = line.trim();
        if let Some(path) = trimmed.strip_prefix("*** Add File: ") {
            paths.push(path.trim().to_string());
        } else if let Some(path) = trimmed.strip_prefix("*** Delete File: ") {
            paths.push(path.trim().to_string());
            delete_count += 1;
        } else if let Some(path) = trimmed.strip_prefix("*** Update File: ") {
            paths.push(path.trim().to_string());
        } else if let Some(path) = trimmed.strip_prefix("*** Move to: ") {
            paths.push(path.trim().to_string());
        }
    }

    (paths, delete_count)
}

use super::strictest_verdict;

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn test_guard() -> FileToolGuard {
        FileToolGuard::new_with_home(
            PathBuf::from("/workspace"),
            PathBuf::from("/home/testuser"),
        )
    }

    fn verdict_rule_id(verdict: &GuardVerdict) -> &str {
        match verdict {
            GuardVerdict::Deny { rule_id, .. } | GuardVerdict::Warn { rule_id, .. } => rule_id,
            GuardVerdict::Allow => "allow",
        }
    }

    fn is_deny(verdict: &GuardVerdict) -> bool {
        matches!(verdict, GuardVerdict::Deny { .. })
    }

    fn is_warn(verdict: &GuardVerdict) -> bool {
        matches!(verdict, GuardVerdict::Warn { .. })
    }

    fn is_allow(verdict: &GuardVerdict) -> bool {
        matches!(verdict, GuardVerdict::Allow)
    }

    // 1. /proc/self/environ → system_path_read Deny at all trust levels
    #[test]
    fn proc_self_environ_denied_at_all_trust_levels() {
        let guard = test_guard();
        let wd = Path::new("/workspace");
        let args = json!({"file_path": "/proc/self/environ"});

        for trust in [TrustLevel::Low, TrustLevel::Medium, TrustLevel::High] {
            let v = guard.evaluate("read_file", &args, wd, trust);
            assert!(is_deny(&v), "expected deny at {trust:?}, got {v:?}");
            assert_eq!(
                verdict_rule_id(&v),
                "file.system_path_read",
                "expected system_path_read at {trust:?}"
            );
        }
    }

    // 2. ~/.ssh/id_rsa read → Deny at medium, Warn at high
    #[test]
    fn ssh_key_read_deny_medium_warn_high() {
        let guard = test_guard();
        let wd = Path::new("/workspace");
        let args = json!({"file_path": "/home/testuser/.ssh/id_rsa"});

        let v_med = guard.evaluate("read_file", &args, wd, TrustLevel::Medium);
        assert!(is_deny(&v_med), "expected deny at medium, got {v_med:?}");
        assert_eq!(verdict_rule_id(&v_med), "file.sensitive_path_read");

        let v_high = guard.evaluate("read_file", &args, wd, TrustLevel::High);
        assert!(is_warn(&v_high), "expected warn at high, got {v_high:?}");
        assert_eq!(verdict_rule_id(&v_high), "file.sensitive_path_read");
    }

    // 3. outside_workspace_read → Warn at medium
    #[test]
    fn outside_workspace_read_warn_at_medium() {
        let guard = test_guard();
        let wd = Path::new("/workspace");
        let args = json!({"file_path": "../other-project/file"});

        let v = guard.evaluate("read_file", &args, wd, TrustLevel::Medium);
        assert!(is_warn(&v), "expected warn, got {v:?}");
        assert_eq!(verdict_rule_id(&v), "file.outside_workspace_read");
    }

    // 4. .git/hooks/pre-commit write → Deny at medium (protected_file_overwrite)
    #[test]
    fn git_hooks_write_denied_at_medium() {
        let guard = test_guard();
        let wd = Path::new("/workspace");
        let args = json!({"file_path": ".git/hooks/pre-commit"});

        let v = guard.evaluate("write_file", &args, wd, TrustLevel::Medium);
        assert!(is_deny(&v), "expected deny, got {v:?}");
        assert_eq!(verdict_rule_id(&v), "file.protected_file_overwrite");
    }

    // 5. apply_patch with 5+ DeleteFile → Warn at medium
    #[test]
    fn apply_patch_delete_many_warn_at_medium() {
        let guard = test_guard();
        let wd = Path::new("/workspace");

        let mut patch = String::new();
        for i in 0..5 {
            patch.push_str(&format!("*** Delete File: src/old_{i}.rs\n"));
        }
        let args = json!({"patch": patch});

        let v = guard.evaluate("apply_patch", &args, wd, TrustLevel::Medium);
        assert!(is_warn(&v), "expected warn, got {v:?}");
        assert_eq!(verdict_rule_id(&v), "file.apply_patch_delete_many");
    }

    // 6. apply_patch multi-path: strictest wins
    #[test]
    fn apply_patch_multi_path_strictest_wins() {
        let guard = test_guard();
        let wd = Path::new("/workspace");

        let patch = "\
*** Update File: src/main.rs
*** Add File: /etc/passwd
";
        let args = json!({"patch": patch});

        let v = guard.evaluate("apply_patch", &args, wd, TrustLevel::Medium);
        assert!(is_deny(&v), "expected deny, got {v:?}");
        assert_eq!(verdict_rule_id(&v), "file.system_path_write");
    }

    // 7. ../other-project/file write → Warn at medium (outside_workspace_write)
    #[test]
    fn outside_workspace_write_warn_at_medium() {
        let guard = test_guard();
        let wd = Path::new("/workspace");
        let args = json!({"file_path": "../other-project/file"});

        let v = guard.evaluate("edit_file", &args, wd, TrustLevel::Medium);
        assert!(is_warn(&v), "expected warn, got {v:?}");
        assert_eq!(verdict_rule_id(&v), "file.outside_workspace_write");
    }

    // 8. /etc/hosts write → Deny at all levels (system_path_write)
    #[test]
    fn etc_hosts_write_denied_at_all_levels() {
        let guard = test_guard();
        let wd = Path::new("/workspace");
        let args = json!({"file_path": "/etc/hosts"});

        for trust in [TrustLevel::Low, TrustLevel::Medium, TrustLevel::High] {
            let v = guard.evaluate("write_file", &args, wd, trust);
            assert!(is_deny(&v), "expected deny at {trust:?}, got {v:?}");
            assert_eq!(verdict_rule_id(&v), "file.system_path_write");
        }
    }

    // 9. Basename matching: .env in subdir
    #[test]
    fn env_file_in_subdir_matches_sensitive() {
        let guard = test_guard();
        let wd = Path::new("/workspace");
        let args = json!({"file_path": "./subdir/.env"});

        let v = guard.evaluate("read_file", &args, wd, TrustLevel::Medium);
        assert!(is_deny(&v), "expected deny, got {v:?}");
        assert_eq!(verdict_rule_id(&v), "file.sensitive_path_read");
    }

    // 10. Unmatched path returns Allow
    #[test]
    fn unmatched_path_returns_allow() {
        let guard = test_guard();
        let wd = Path::new("/workspace");
        let args = json!({"file_path": "src/main.rs"});

        let v = guard.evaluate("read_file", &args, wd, TrustLevel::Medium);
        assert!(is_allow(&v), "expected allow, got {v:?}");

        let v = guard.evaluate("write_file", &args, wd, TrustLevel::Medium);
        assert!(is_allow(&v), "expected allow, got {v:?}");
    }

    // 11. Unknown tool returns Allow
    #[test]
    fn unknown_tool_returns_allow() {
        let guard = test_guard();
        let wd = Path::new("/workspace");
        let args = json!({"file_path": "/etc/passwd"});

        let v = guard.evaluate("some_unknown_tool", &args, wd, TrustLevel::Medium);
        assert!(is_allow(&v), "expected allow, got {v:?}");
    }

    // 12. outside_workspace_read Warn at medium trust (explicit spec §13 test)
    #[test]
    fn outside_workspace_read_warn_at_medium_explicit() {
        let guard = test_guard();
        let wd = Path::new("/workspace");
        let args = json!({"file_path": "/tmp/somefile.txt"});

        let v = guard.evaluate("read_file", &args, wd, TrustLevel::Medium);
        assert!(is_warn(&v), "expected warn, got {v:?}");
        assert_eq!(verdict_rule_id(&v), "file.outside_workspace_read");
    }

    // 13. read_many_files with mixed paths: strictest verdict wins
    #[test]
    fn read_many_files_mixed_paths_strictest_wins() {
        let guard = test_guard();
        let wd = Path::new("/workspace");
        let args = json!({"paths": ["src/main.rs", "/proc/cpuinfo"]});

        let v = guard.evaluate("read_many_files", &args, wd, TrustLevel::Medium);
        assert!(is_deny(&v), "expected deny, got {v:?}");
        assert_eq!(verdict_rule_id(&v), "file.system_path_read");
    }

    // 14. grep with missing path uses working_dir
    #[test]
    fn grep_missing_path_uses_working_dir() {
        let guard = test_guard();
        let wd = Path::new("/workspace");
        let args = json!({"pattern": "TODO"});

        let v = guard.evaluate("grep", &args, wd, TrustLevel::Medium);
        assert!(is_allow(&v), "workspace path should be allowed, got {v:?}");
    }

    #[test]
    fn grep_with_outside_path_warns_at_medium() {
        let guard = test_guard();
        let wd = Path::new("/workspace");
        let args = json!({"pattern": "TODO", "path": "/tmp/other"});

        let v = guard.evaluate("grep", &args, wd, TrustLevel::Medium);
        assert!(is_warn(&v), "expected warn, got {v:?}");
        assert_eq!(verdict_rule_id(&v), "file.outside_workspace_read");
    }

    #[test]
    fn apply_patch_delete_many_deny_at_low() {
        let guard = test_guard();
        let wd = Path::new("/workspace");

        let mut patch = String::new();
        for i in 0..6 {
            patch.push_str(&format!("*** Delete File: src/old_{i}.rs\n"));
        }
        let args = json!({"patch": patch});

        let v = guard.evaluate("apply_patch", &args, wd, TrustLevel::Low);
        assert!(is_deny(&v), "expected deny, got {v:?}");
        assert_eq!(verdict_rule_id(&v), "file.apply_patch_delete_many");
    }

    #[test]
    fn apply_patch_move_to_path_evaluated() {
        let guard = test_guard();
        let wd = Path::new("/workspace");

        let patch = "\
*** Update File: src/main.rs
*** Move to: /etc/shadow
";
        let args = json!({"patch": patch});

        let v = guard.evaluate("apply_patch", &args, wd, TrustLevel::Medium);
        assert!(is_deny(&v), "expected deny, got {v:?}");
    }

    #[test]
    fn protected_file_overwrite_warn_at_high() {
        let guard = test_guard();
        let wd = Path::new("/workspace");
        let args = json!({"file_path": ".git/hooks/pre-commit"});

        let v = guard.evaluate("write_file", &args, wd, TrustLevel::High);
        assert!(is_warn(&v), "expected warn, got {v:?}");
        assert_eq!(verdict_rule_id(&v), "file.protected_file_overwrite");
    }

    #[test]
    fn parse_patch_paths_extracts_all_directives() {
        let patch = "\
*** Add File: src/new.rs
some content
*** Delete File: src/old.rs
*** Update File: src/main.rs
*** Move to: src/main_v2.rs
*** Delete File: tests/old_test.rs
";
        let (paths, delete_count) = parse_patch_paths(patch);
        assert_eq!(
            paths,
            vec![
                "src/new.rs",
                "src/old.rs",
                "src/main.rs",
                "src/main_v2.rs",
                "tests/old_test.rs"
            ]
        );
        assert_eq!(delete_count, 2);
    }

    #[test]
    fn outside_workspace_read_deny_at_low() {
        let guard = test_guard();
        let wd = Path::new("/workspace");
        let args = json!({"file_path": "/tmp/somefile.txt"});

        let v = guard.evaluate("read_file", &args, wd, TrustLevel::Low);
        assert!(is_deny(&v), "expected deny, got {v:?}");
        assert_eq!(verdict_rule_id(&v), "file.outside_workspace_read");
    }

    #[test]
    fn outside_workspace_read_allow_at_high() {
        let guard = test_guard();
        let wd = Path::new("/workspace");
        let args = json!({"file_path": "/tmp/somefile.txt"});

        let v = guard.evaluate("read_file", &args, wd, TrustLevel::High);
        assert!(is_allow(&v), "expected allow, got {v:?}");
    }

    #[test]
    fn sensitive_write_denied_at_all_levels() {
        let guard = test_guard();
        let wd = Path::new("/workspace");
        let args = json!({"file_path": "/home/testuser/.ssh/id_rsa"});

        for trust in [TrustLevel::Low, TrustLevel::Medium, TrustLevel::High] {
            let v = guard.evaluate("write_file", &args, wd, trust);
            assert!(is_deny(&v), "expected deny at {trust:?}, got {v:?}");
        }
    }

    #[test]
    fn below_delete_threshold_no_warning() {
        let guard = test_guard();
        let wd = Path::new("/workspace");

        let mut patch = String::new();
        for i in 0..4 {
            patch.push_str(&format!("*** Delete File: src/old_{i}.rs\n"));
        }
        let args = json!({"patch": patch});

        let v = guard.evaluate("apply_patch", &args, wd, TrustLevel::Medium);
        assert!(is_allow(&v), "expected allow, got {v:?}");
    }
}
