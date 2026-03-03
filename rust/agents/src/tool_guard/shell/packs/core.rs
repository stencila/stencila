//! Core packs: `core.filesystem`, `core.git`, `core.obfuscation`, `core.stencila`.
//!
//! These are always evaluated, even at `trustLevel: high`.

use super::{Confidence, Pack, PatternRule, destructive_pattern, tokenize_or_bail};
use crate::tool_guard::shell::tokenizer::Token;

// ---------------------------------------------------------------------------
// Token helpers
// ---------------------------------------------------------------------------

/// Check if a token has combined short flags containing a specific character.
fn has_short_flag(t: &Token, ch: char) -> bool {
    t.value.starts_with('-') && !t.value.starts_with("--") && t.value.contains(ch)
}

/// Check if a token value is in a list of targets (non-flag, non-command).
fn is_target(t: &Token, exclude_cmd: &str, targets: &[&str]) -> bool {
    !t.value.starts_with('-') && t.value != exclude_cmd && targets.contains(&t.value.as_str())
}

// ---------------------------------------------------------------------------
// core.filesystem
// ---------------------------------------------------------------------------

/// System directories targeted by `recursive_delete_root`.
const ROOT_TARGETS: &[&str] = &[
    "/", "~", "/home", "/etc", "/usr", "/var", "/boot", "/bin", "/sbin", "/lib",
];

fn has_rm_recursive(tokens: &[Token]) -> bool {
    tokens.iter().any(|t| {
        t.value == "-r"
            || t.value == "-R"
            || t.value == "--recursive"
            || has_short_flag(t, 'r')
            || has_short_flag(t, 'R')
    })
}

fn has_rm_force(tokens: &[Token]) -> bool {
    tokens
        .iter()
        .any(|t| t.value == "-f" || t.value == "--force" || has_short_flag(t, 'f'))
}

fn rm_recursive_root_validator(cmd: &str) -> bool {
    let tokens = tokenize_or_bail!(cmd, true);
    has_rm_recursive(&tokens) && tokens.iter().any(|t| is_target(t, "rm", ROOT_TARGETS))
}

fn rm_recursive_force_validator(cmd: &str) -> bool {
    let tokens = tokenize_or_bail!(cmd, true);
    has_rm_recursive(&tokens) && has_rm_force(&tokens)
}

fn rm_recursive_validator(cmd: &str) -> bool {
    let tokens = tokenize_or_bail!(cmd, true);
    has_rm_recursive(&tokens) && !has_rm_force(&tokens)
}

fn find_destructive_validator(cmd: &str) -> bool {
    let tokens = tokenize_or_bail!(cmd, true);
    tokens
        .iter()
        .any(|t| super::FIND_DESTRUCTIVE_FLAGS.contains(&t.value.as_str()))
}

fn mv_system_path_validator(cmd: &str) -> bool {
    let tokens = tokenize_or_bail!(cmd, true);
    tokens.iter().any(|t| is_target(t, "mv", ROOT_TARGETS))
}

fn shred_device_validator(cmd: &str) -> bool {
    let tokens = tokenize_or_bail!(cmd, true);
    tokens.iter().any(|t| t.value.starts_with("/dev/"))
}

fn chmod_broad_validator(cmd: &str) -> bool {
    let tokens = tokenize_or_bail!(cmd, true);
    let has_recursive = tokens
        .iter()
        .any(|t| t.value == "-R" || t.value == "--recursive");
    let has_system_target = tokens.iter().any(|t| is_target(t, "chmod", ROOT_TARGETS));
    let has_permissive = tokens
        .iter()
        .any(|t| t.value == "777" || t.value == "a+rwx");

    (has_recursive && has_system_target) || has_permissive
}

fn chown_recursive_validator(cmd: &str) -> bool {
    let tokens = tokenize_or_bail!(cmd, true);
    let has_recursive = tokens
        .iter()
        .any(|t| t.value == "-R" || t.value == "--recursive");
    let has_system_target = tokens.iter().any(|t| is_target(t, "chown", ROOT_TARGETS));
    has_recursive && has_system_target
}

/// Sensitive paths for the `sensitive_read` rule.
const SENSITIVE_READ_TARGETS: &[&str] = &["/etc/shadow", "/etc/gshadow", "/etc/sudoers"];

const SENSITIVE_READ_PREFIXES: &[&str] = &["~/.ssh/", "~/.gnupg/", "~/.aws/", "~/.config/gcloud/"];

const SENSITIVE_READ_BASENAMES: &[&str] = &[".env", ".netrc"];

fn sensitive_read_validator(cmd: &str) -> bool {
    let tokens = tokenize_or_bail!(cmd, true);
    let read_cmds = [
        "cat", "bat", "head", "tail", "less", "more", "strings", "xxd",
    ];
    tokens.iter().any(|t| {
        let v = t.value.as_str();
        if v.starts_with('-') || read_cmds.contains(&v) {
            return false;
        }
        if SENSITIVE_READ_TARGETS.contains(&v) {
            return true;
        }
        if SENSITIVE_READ_PREFIXES.iter().any(|p| v.starts_with(p)) {
            return true;
        }
        let basename = v.rsplit('/').next().unwrap_or(v);
        SENSITIVE_READ_BASENAMES.contains(&basename)
    })
}

pub static FILESYSTEM_PACK: Pack = Pack {
    id: "core.filesystem",
    name: "Filesystem",
    description: "Guards against recursive/forced file deletion and dangerous moves/overwrites",
    safe_patterns: &[],
    destructive_patterns: &[
        destructive_pattern!("recursive_delete_root", r"\brm\b", rm_recursive_root_validator, "Recursive deletion of root, home, or system directory", "Specify the exact subdirectory to delete", Confidence::High),
        destructive_pattern!("recursive_delete_force", r"\brm\b", rm_recursive_force_validator, "Forced recursive deletion can destroy entire directory trees without confirmation", "Delete specific files by name, or remove the `-f` flag and specify exact paths", Confidence::High),
        destructive_pattern!("recursive_delete", r"\brm\b", rm_recursive_validator, "Recursive deletion can destroy directory trees", "Delete specific files by name, or list directory contents first with `ls -la`", Confidence::Medium),
        destructive_pattern!("find_destructive", r"\bfind\b", find_destructive_validator, "`find` with action flags can delete or execute arbitrary commands on matched files", "Use `find` to list matching files first (`find . -name '*.tmp'`), review the output, then delete specific files by name", Confidence::Medium),
        destructive_pattern!("mv_system_path", r"\bmv\b", mv_system_path_validator, "Moving system directories can break the OS", "Move specific files within subdirectories instead", Confidence::Medium),
        destructive_pattern!("shred_device", r"\bshred\b", shred_device_validator, "Shredding devices causes permanent data loss", "This operation should not be performed by an agent", Confidence::High),
        destructive_pattern!("chmod_broad", r"\bchmod\b", chmod_broad_validator, "Recursive or overly permissive chmod can break system security", "Set specific permissions on specific files (e.g., `chmod 644 file`)", Confidence::Medium),
        destructive_pattern!("chown_recursive", r"\bchown\b", chown_recursive_validator, "Recursive chown on system directories can break the OS", "Change ownership of specific files only", Confidence::Medium),
        destructive_pattern!("overwrite_truncate", r"(?:>\s*/(?:etc|boot|usr|bin|sbin|lib|proc|sys|dev)/|\btruncate\b[^|><]*/(?:etc|boot|usr|bin|sbin|lib|proc|sys|dev)/)", "Overwriting or appending to system files can break the OS", "Use `sudo tee` with explicit paths, or edit specific config files", Confidence::Medium),
        destructive_pattern!("sensitive_read", r"\b(?:cat|bat|head|tail|less|more|strings|xxd)\b", sensitive_read_validator, "Reading sensitive files (credentials, private keys, auth tokens) can leak secrets into the agent's context window", "Use targeted commands that don't expose raw secrets (e.g., `ssh-keygen -l -f ~/.ssh/id_rsa` to check a key fingerprint)", Confidence::Medium),
    ],
};

// ---------------------------------------------------------------------------
// core.git
// ---------------------------------------------------------------------------

fn force_push_validator(cmd: &str) -> bool {
    let tokens = tokenize_or_bail!(cmd, true);
    let has_push = tokens.iter().any(|t| t.value == "push");
    let has_force = tokens.iter().any(|t| {
        t.value == "--force"
            || (t.value == "-f" && has_push)
            || (has_short_flag(t, 'f') && has_push)
    });
    let has_lease = tokens
        .iter()
        .any(|t| t.value == "--force-with-lease" || t.value.starts_with("--force-with-lease="));
    has_force && !has_lease
}

/// Validator for `git clean`: fires when `-f` (or combined flags containing
/// `f`) appears before `--`. Tokens after `--` are paths, not flags.
fn clean_force_validator(cmd: &str) -> bool {
    let tokens = tokenize_or_bail!(cmd, true);
    tokens
        .iter()
        .take_while(|t| t.value != "--")
        .any(|t| t.value == "-f" || t.value == "--force" || has_short_flag(t, 'f'))
}

fn checkout_discard_validator(cmd: &str) -> bool {
    let tokens = tokenize_or_bail!(cmd, true);
    let has_dashdash = tokens.iter().any(|t| t.value == "--");
    let path_args: Vec<&str> = if has_dashdash {
        tokens
            .iter()
            .skip_while(|t| t.value != "--")
            .skip(1)
            .map(|t| t.value.as_str())
            .collect()
    } else {
        tokens
            .iter()
            .skip(2)
            .filter(|t| !t.value.starts_with('-'))
            .map(|t| t.value.as_str())
            .collect()
    };

    path_args
        .iter()
        .any(|v| *v == "." || *v == ":/" || *v == "*" || v.ends_with('/'))
}

fn restore_discard_validator(cmd: &str) -> bool {
    let tokens = tokenize_or_bail!(cmd, true);
    let has_dashdash = tokens.iter().any(|t| t.value == "--");
    let path_args: Vec<&str> = if has_dashdash {
        tokens
            .iter()
            .skip_while(|t| t.value != "--")
            .skip(1)
            .map(|t| t.value.as_str())
            .collect()
    } else {
        tokens
            .iter()
            .skip(2)
            .filter(|t| !t.value.starts_with('-'))
            .map(|t| t.value.as_str())
            .collect()
    };

    path_args
        .iter()
        .any(|v| *v == "." || *v == "*" || v.ends_with('/'))
}

fn rebase_active_validator(cmd: &str) -> bool {
    let tokens = tokenize_or_bail!(cmd, true);
    let has_active = tokens
        .iter()
        .any(|t| t.value == "--onto" || t.value == "--root");
    let has_recovery = tokens
        .iter()
        .any(|t| t.value == "--abort" || t.value == "--continue" || t.value == "--skip");
    has_active && !has_recovery
}

fn reflog_expire_validator(cmd: &str) -> bool {
    let tokens = tokenize_or_bail!(cmd, true);
    tokens
        .iter()
        .any(|t| t.value == "--expire=now" || t.value == "--all")
}

fn gc_prune_validator(cmd: &str) -> bool {
    let tokens = tokenize_or_bail!(cmd, true);
    tokens
        .iter()
        .any(|t| t.value == "--prune=now" || t.value == "--prune=all")
}

pub static GIT_PACK: Pack = Pack {
    id: "core.git",
    name: "Git",
    description: "Guards against destructive git operations that lose history or modify remote state",
    safe_patterns: &[],
    destructive_patterns: &[
        destructive_pattern!("reset_hard", r"\bgit\s+reset\s+--hard\b", "Discards all uncommitted changes permanently", "Use `git stash` to save changes, or `git reset --soft` to unstage", Confidence::High),
        destructive_pattern!("force_push", r"\bgit\s+push\b", force_push_validator, "Force push overwrites remote history, potentially losing others' work", "Use `git push --force-with-lease` for safer force pushes", Confidence::High),
        destructive_pattern!("clean_force", r"\bgit\s+clean\b", clean_force_validator, "Permanently deletes untracked files/directories", "Use `git clean -n` (dry run) first to preview what will be deleted", Confidence::High),
        destructive_pattern!("checkout_discard", r"\bgit\s+checkout\b", checkout_discard_validator, "Discards uncommitted changes across a broad scope", "Checkout specific files: `git checkout -- path/to/file`", Confidence::High),
        destructive_pattern!("restore_discard", r"\bgit\s+restore\b", restore_discard_validator, "Discards uncommitted or staged changes across a broad scope", "Restore specific files: `git restore path/to/file`", Confidence::High),
        destructive_pattern!("rebase_active", r"\bgit\s+rebase\b", rebase_active_validator, "Rebase with `--onto` or `--root` rewrites commit history and can cause data loss", "Use `git log` to review history first; simple `git rebase <branch>` is permitted", Confidence::Medium),
        destructive_pattern!("branch_force_delete", r"\bgit\s+branch\s+.*(-D|--delete\s+--force|--force\s+--delete)\b", "Force-deletes a branch regardless of merge status", "Use `git branch -d` which only deletes if fully merged", Confidence::Medium),
        destructive_pattern!("stash_drop_clear", r"\bgit\s+stash\s+(?:drop|clear)\b", "Permanently deletes stashed changes", "Verify stash contents with `git stash show` before dropping", Confidence::Medium),
        destructive_pattern!("worktree_force_remove", r"\bgit\s+worktree\s+remove\b.*(?:--force|-f)\b", "Force-removes a worktree without checking for uncommitted changes", "Use `git worktree remove` without `--force` which checks for uncommitted changes first", Confidence::Medium),
        destructive_pattern!("reflog_expire", r"\bgit\s+reflog\s+expire\b", reflog_expire_validator, "Expiring reflog entries permanently destroys the ability to recover previous HEAD positions", "Use `git reflog` to inspect entries before expiring; avoid `--expire=now`", Confidence::High),
        destructive_pattern!("gc_prune", r"\bgit\s+gc\b", gc_prune_validator, "Aggressive garbage collection permanently removes unreachable objects", "Use `git gc` without `--prune=now` which uses a safe 2-week grace period", Confidence::Medium),
    ],
};

// ---------------------------------------------------------------------------
// core.obfuscation
// ---------------------------------------------------------------------------

pub static OBFUSCATION_PACK: Pack = Pack {
    id: "core.obfuscation",
    name: "Obfuscation",
    description: "Guards against meta-execution patterns whose purpose is guard bypass",
    safe_patterns: &[],
    destructive_patterns: &[
        destructive_pattern!("pipe_to_shell", r"\|\s*(?:bash|sh|zsh|dash|fish|csh|tcsh|ksh)\b", "Piping content to a shell interpreter bypasses command inspection", "Download/generate the script to a file, review it, then execute", Confidence::High),
        destructive_pattern!("base64_to_shell", r"\bbase64\b.*\|\s*(?:bash|sh|zsh|dash|fish|csh|tcsh|ksh|eval)\b", "Base64-encoded commands bypass pattern matching", "Write the command directly instead of encoding it", Confidence::High),
        destructive_pattern!("eval_subshell", r"\beval\s+(?:\$\(|`)", "eval of dynamic content bypasses command inspection", "Execute the inner command directly", Confidence::High),
        destructive_pattern!("curl_pipe_shell", r"\b(?:curl|wget)\b.*\|\s*(?:bash|sh|zsh|dash|fish|csh|tcsh|ksh|eval)\b", "Downloading and executing untrusted code in one step", "Download the script first with `curl -o script.sh`, review it, then execute", Confidence::High),
        destructive_pattern!("python_exec", r"\bpython[23]?\s+-c\b[^|><]*\b(?:os\.(?:system|popen|execl|execle|execlp|execlpe|execv|execve|execvp|execvpe)|subprocess\.(?:run|call|Popen))\b", "Python used as a shell wrapper to bypass command inspection", "Execute the shell command directly", Confidence::Medium),
    ],
};

// ---------------------------------------------------------------------------
// core.stencila
// ---------------------------------------------------------------------------

pub static STENCILA_PACK: Pack = Pack {
    id: "core.stencila",
    name: "Stencila",
    description: "Guards the agent's own runtime, credentials, and publishing operations",
    safe_patterns: &[],
    destructive_patterns: &[
        destructive_pattern!("secrets_modify", r"\bstencila\s+secrets\s+(?:set|delete|remove)\b", "Modifying secrets can break authentication", "Use `stencila secrets list` to view secrets", Confidence::High),
        destructive_pattern!("auth_modify", r"\bstencila\s+auth\s+(?:login|logout)\b", "Changing authentication state affects all sessions", "Use `stencila auth status` to check auth state", Confidence::High),
        destructive_pattern!("cloud_auth", r"\bstencila\s+cloud\s+(?:signin|signout)\b", "Changing cloud authentication affects deployments", "Use `stencila cloud status` to check cloud state", Confidence::High),
        destructive_pattern!("publish", r"\bstencila\s+publish\b", "Publishing makes content publicly visible", "Review content manually before publishing", Confidence::High),
        destructive_pattern!("uninstall", r"\bstencila\s+uninstall\b", "Uninstalling removes the runtime", "This operation should not be performed by an agent", Confidence::High),
        destructive_pattern!("push", r"\bstencila\s+push\b", "Pushing sends local changes to remote", "Use `stencila status` to review changes first", Confidence::Medium),
        destructive_pattern!("db_destructive", r"\bstencila\s+db\s+(?:reset|gc)\b", "Database reset/gc can lose data", "Use `stencila db status` to inspect the database first", Confidence::Medium),
        destructive_pattern!("clean", r"\bstencila\s+clean\b", "Clean removes generated artifacts", "Use `stencila status` to see what will be removed", Confidence::Medium),
    ],
};

#[cfg(test)]
mod tests {
    use super::*;

    // ---- core.filesystem ----

    #[test]
    fn rm_recursive_root_matches() {
        assert!(rm_recursive_root_validator("rm -rf /"));
        assert!(rm_recursive_root_validator("rm -r /home"));
        assert!(rm_recursive_root_validator("rm -R /etc"));
        assert!(rm_recursive_root_validator("rm -rf ~"));
        assert!(!rm_recursive_root_validator("rm -rf /tmp/data"));
        assert!(!rm_recursive_root_validator("rm file.txt"));
    }

    #[test]
    fn rm_recursive_force_matches() {
        assert!(rm_recursive_force_validator("rm -rf /tmp/data"));
        assert!(rm_recursive_force_validator("rm -fr /tmp/data"));
        assert!(rm_recursive_force_validator("rm -r -f /tmp/data"));
        assert!(rm_recursive_force_validator("rm --recursive -f dir"));
        assert!(rm_recursive_force_validator("rm -r dir -f"));
        assert!(!rm_recursive_force_validator("rm -r dir"));
        assert!(!rm_recursive_force_validator("rm -f file.txt"));
        assert!(!rm_recursive_force_validator("rm file.txt"));
    }

    #[test]
    fn rm_recursive_no_force_matches() {
        assert!(rm_recursive_validator("rm -r dir"));
        assert!(rm_recursive_validator("rm -R dir"));
        assert!(rm_recursive_validator("rm --recursive dir"));
        assert!(!rm_recursive_validator("rm -rf dir"));
        assert!(!rm_recursive_validator("rm file.txt"));
    }

    #[test]
    fn find_destructive_matches() {
        assert!(find_destructive_validator("find . -delete"));
        assert!(find_destructive_validator("find . -exec rm {} \\;"));
        assert!(find_destructive_validator("find /tmp -execdir ls {} \\;"));
        assert!(find_destructive_validator("find . -ok rm {} \\;"));
        assert!(find_destructive_validator("find . -okdir rm {} \\;"));
        assert!(!find_destructive_validator("find . -name '*.txt'"));
        assert!(!find_destructive_validator("find . -name exec-summary.txt"));
    }

    #[test]
    fn mv_system_path_matches() {
        assert!(mv_system_path_validator("mv /etc /tmp"));
        assert!(mv_system_path_validator("mv data /usr"));
        assert!(!mv_system_path_validator("mv file1 file2"));
    }

    #[test]
    fn shred_device_matches() {
        assert!(shred_device_validator("shred /dev/sda"));
        assert!(!shred_device_validator("shred file.txt"));
    }

    #[test]
    fn chmod_broad_matches() {
        assert!(chmod_broad_validator("chmod -R 755 /etc"));
        assert!(chmod_broad_validator("chmod 777 file"));
        assert!(chmod_broad_validator("chmod a+rwx file"));
        assert!(!chmod_broad_validator("chmod 644 file.txt"));
    }

    #[test]
    fn chown_recursive_matches() {
        assert!(chown_recursive_validator("chown -R root /etc"));
        assert!(!chown_recursive_validator("chown root file.txt"));
        assert!(!chown_recursive_validator("chown -R root /tmp/dir"));
    }

    #[test]
    fn sensitive_read_matches() {
        assert!(sensitive_read_validator("cat /etc/shadow"));
        assert!(sensitive_read_validator("head ~/.ssh/id_rsa"));
        assert!(sensitive_read_validator("cat .env"));
        assert!(sensitive_read_validator("cat /path/to/.netrc"));
        assert!(sensitive_read_validator("bat /etc/shadow"));
        assert!(sensitive_read_validator("bat ~/.ssh/id_rsa"));
        assert!(!sensitive_read_validator("cat README.md"));
        assert!(!sensitive_read_validator("head main.rs"));
        assert!(!sensitive_read_validator("bat main.rs"));
    }

    // ---- core.git ----

    #[test]
    fn clean_force_validator_cases() {
        assert!(clean_force_validator("git clean -f"));
        assert!(clean_force_validator("git clean -fd"));
        assert!(clean_force_validator("git clean -df"));
        assert!(clean_force_validator("git clean --force"));
        assert!(!clean_force_validator("git clean -n"));
        assert!(!clean_force_validator("git clean --dry-run"));
        assert!(!clean_force_validator("git clean -- -f"));
    }

    #[test]
    fn force_push_validator_cases() {
        assert!(force_push_validator("git push --force origin main"));
        assert!(force_push_validator("git push -f origin main"));
        assert!(!force_push_validator(
            "git push --force-with-lease origin main"
        ));
        assert!(!force_push_validator("git push origin main"));
    }

    #[test]
    fn checkout_discard_validator_cases() {
        assert!(checkout_discard_validator("git checkout -- ."));
        assert!(checkout_discard_validator("git checkout ."));
        assert!(checkout_discard_validator("git checkout -- :/"));
        assert!(checkout_discard_validator("git checkout -- src/"));
        assert!(!checkout_discard_validator("git checkout -- file.txt"));
        assert!(!checkout_discard_validator("git checkout feature-branch"));
    }

    #[test]
    fn restore_discard_validator_cases() {
        assert!(restore_discard_validator("git restore ."));
        assert!(restore_discard_validator("git restore --staged ."));
        assert!(restore_discard_validator("git restore -- *"));
        assert!(restore_discard_validator("git restore src/"));
        assert!(!restore_discard_validator("git restore file.txt"));
    }

    #[test]
    fn rebase_active_validator_cases() {
        assert!(rebase_active_validator("git rebase --onto main feature"));
        assert!(rebase_active_validator("git rebase --root"));
        assert!(!rebase_active_validator("git rebase --abort"));
        assert!(!rebase_active_validator("git rebase --continue"));
        assert!(!rebase_active_validator("git rebase main"));
    }

    #[test]
    fn reflog_expire_validator_cases() {
        assert!(reflog_expire_validator("git reflog expire --expire=now"));
        assert!(reflog_expire_validator("git reflog expire --all"));
        assert!(!reflog_expire_validator("git reflog expire"));
    }

    #[test]
    fn gc_prune_validator_cases() {
        assert!(gc_prune_validator("git gc --prune=now"));
        assert!(gc_prune_validator("git gc --prune=all"));
        assert!(!gc_prune_validator("git gc"));
    }
}
