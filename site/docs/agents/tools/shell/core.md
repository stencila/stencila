---
title: "Core"
description: "Guards against recursive/forced file deletion and dangerous moves/overwrites. Guards against destructive git operations that lose history or modify remote state. Guards against meta-execution patterns whose purpose is guard bypass. Guards the agent's own runtime, credentials, and publishing operations"
---

This page lists the safe and destructive patterns in the **Filesystem**, **Git**, **Obfuscation**, and **Stencila** shell guard packs. See [Shell Tool](/docs/agents/tools/shell#guard-pipeline) for how these patterns are evaluated.

## Filesystem

**Pack ID:** `core.filesystem`

Guards against recursive/forced file deletion and dangerous moves/overwrites

### Safe patterns

| Rule ID | Pattern |
|---------|--------|
| `core.filesystem.ls` | `^ls\b[^\|><]*$` |
| `core.filesystem.cat` | `^cat\b[^\|><]*$` |
| `core.filesystem.bat` | `^bat\b[^\|><]*$` |
| `core.filesystem.head` | `^head\b[^\|><]*$` |
| `core.filesystem.tail` | `^tail\b[^\|><]*$` |
| `core.filesystem.less` | `^less\b[^\|><]*$` |
| `core.filesystem.wc` | `^wc\b[^\|><]*$` |
| `core.filesystem.file` | `^file\b[^\|><]*$` |
| `core.filesystem.stat` | `^stat\b[^\|><]*$` |
| `core.filesystem.find` | `^find\b[^\|><]*$` |
| `core.filesystem.du` | `^du\b[^\|><]*$` |
| `core.filesystem.df` | `^df\b[^\|><]*$` |
| `core.filesystem.tree` | `^tree\b[^\|><]*$` |
| `core.filesystem.grep` | `^grep\b[^\|><]*$` |
| `core.filesystem.rg` | `^rg\b[^\|><]*$` |
| `core.filesystem.diff` | `^diff\b[^\|><]*$` |
| `core.filesystem.sort` | `^sort\b[^\|><]*$` |
| `core.filesystem.md5sum` | `^md5sum\b[^\|><]*$` |
| `core.filesystem.sha256sum` | `^sha256sum\b[^\|><]*$` |
| `core.filesystem.realpath` | `^realpath\b[^\|><]*$` |
| `core.filesystem.dirname` | `^dirname\b[^\|><]*$` |
| `core.filesystem.basename` | `^basename\b[^\|><]*$` |
| `core.filesystem.readlink` | `^readlink\b[^\|><]*$` |
| `core.filesystem.test` | `^test\b[^\|><]*$` |
| `core.filesystem.bracket` | `^\[[^\|><]*$` |
| `core.filesystem.double_bracket` | `^\[\[[^\|><]*$` |
| `core.filesystem.cargo_check` | `^cargo\s+check\b[^\|><]*$` |
| `core.filesystem.cargo_clippy` | `^cargo\s+clippy\b[^\|><]*$` |
| `core.filesystem.go_vet` | `^go\s+vet\b[^\|><]*$` |
| `core.filesystem.env` | `^env\b[^\|><]*$` |
| `core.filesystem.printenv` | `^printenv\b[^\|><]*$` |
| `core.filesystem.which` | `^which\b[^\|><]*$` |
| `core.filesystem.whoami` | `^whoami\b[^\|><]*$` |
| `core.filesystem.uname` | `^uname\b[^\|><]*$` |
| `core.filesystem.pwd` | `^pwd\b[^\|><]*$` |
| `core.filesystem.echo` | `^echo\b[^\|><]*$` |
| `core.filesystem.date` | `^date\b[^\|><]*$` |
| `core.filesystem.hostname` | `^hostname\b[^\|><]*$` |
| `core.filesystem.id` | `^id\b[^\|><]*$` |
| `core.filesystem.groups` | `^groups\b[^\|><]*$` |
| `core.filesystem.mkdir` | `^mkdir\b[^\|><]*$` |
| `core.filesystem.touch` | `^touch\b[^\|><]*$` |

### Destructive patterns

| Rule ID | Reason | Suggestion | Confidence |
|---------|--------|------------|:----------:|
| `core.filesystem.recursive_delete_root` | Recursive deletion of root, home, or system directory | Specify the exact subdirectory to delete | High |
| `core.filesystem.recursive_delete_force` | Forced recursive deletion can destroy entire directory trees without confirmation | Delete specific files by name, or remove the `-f` flag and specify exact paths | High |
| `core.filesystem.recursive_delete` | Recursive deletion can destroy directory trees | Delete specific files by name, or list directory contents first with `ls -la` | Medium |
| `core.filesystem.find_destructive` | `find` with action flags can delete or execute arbitrary commands on matched files | Use `find` to list matching files first (`find . -name '*.tmp'`), review the output, then delete specific files by name | Medium |
| `core.filesystem.mv_system_path` | Moving system directories can break the OS | Move specific files within subdirectories instead | Medium |
| `core.filesystem.shred_device` | Shredding devices causes permanent data loss | This operation should not be performed by an agent | High |
| `core.filesystem.chmod_broad` | Recursive or overly permissive chmod can break system security | Set specific permissions on specific files (e.g., `chmod 644 file`) | Medium |
| `core.filesystem.chown_recursive` | Recursive chown on system directories can break the OS | Change ownership of specific files only | Medium |
| `core.filesystem.overwrite_truncate` | Overwriting or appending to system files can break the OS | Use `sudo tee` with explicit paths, or edit specific config files | Medium |
| `core.filesystem.sensitive_read` | Reading sensitive files (credentials, private keys, auth tokens) can leak secrets into the agent's context window | Use targeted commands that don't expose raw secrets (e.g., `ssh-keygen -l -f ~/.ssh/id_rsa` to check a key fingerprint) | Medium |

## Git

**Pack ID:** `core.git`

Guards against destructive git operations that lose history or modify remote state

### Safe patterns

| Rule ID | Pattern |
|---------|--------|
| `core.git.git_status` | `^git\s+status\b[^\|><]*$` |
| `core.git.git_log` | `^git\s+log\b[^\|><]*$` |
| `core.git.git_diff` | `^git\s+diff\b[^\|><]*$` |
| `core.git.git_show` | `^git\s+show\b[^\|><]*$` |
| `core.git.git_branch` | `^git\s+branch\b[^\|><]*$` |
| `core.git.git_tag` | `^git\s+tag\b[^\|><]*$` |
| `core.git.git_remote_v` | `^git\s+remote\s+-v\b[^\|><]*$` |
| `core.git.git_rev_parse` | `^git\s+rev-parse\b[^\|><]*$` |

### Destructive patterns

| Rule ID | Reason | Suggestion | Confidence |
|---------|--------|------------|:----------:|
| `core.git.reset_hard` | Discards all uncommitted changes permanently | Use `git stash` to save changes, or `git reset --soft` to unstage | High |
| `core.git.force_push` | Force push overwrites remote history, potentially losing others' work | Use `git push --force-with-lease` for safer force pushes | High |
| `core.git.clean_force` | Permanently deletes untracked files/directories | Use `git clean -n` (dry run) first to preview what will be deleted | High |
| `core.git.checkout_discard` | Discards uncommitted changes across a broad scope | Checkout specific files: `git checkout -- path/to/file` | High |
| `core.git.restore_discard` | Discards uncommitted or staged changes across a broad scope | Restore specific files: `git restore path/to/file` | High |
| `core.git.rebase_active` | Rebase with `--onto` or `--root` rewrites commit history and can cause data loss | Use `git log` to review history first; simple `git rebase <branch>` is permitted | Medium |
| `core.git.branch_force_delete` | Force-deletes a branch regardless of merge status | Use `git branch -d` which only deletes if fully merged | Medium |
| `core.git.stash_drop_clear` | Permanently deletes stashed changes | Verify stash contents with `git stash show` before dropping | Medium |
| `core.git.worktree_force_remove` | Force-removes a worktree without checking for uncommitted changes | Use `git worktree remove` without `--force` which checks for uncommitted changes first | Medium |
| `core.git.reflog_expire` | Expiring reflog entries permanently destroys the ability to recover previous HEAD positions | Use `git reflog` to inspect entries before expiring; avoid `--expire=now` | High |
| `core.git.gc_prune` | Aggressive garbage collection permanently removes unreachable objects | Use `git gc` without `--prune=now` which uses a safe 2-week grace period | Medium |

## Obfuscation

**Pack ID:** `core.obfuscation`

Guards against meta-execution patterns whose purpose is guard bypass

### Destructive patterns

| Rule ID | Reason | Suggestion | Confidence |
|---------|--------|------------|:----------:|
| `core.obfuscation.pipe_to_shell` | Piping content to a shell interpreter bypasses command inspection | Download/generate the script to a file, review it, then execute | High |
| `core.obfuscation.base64_to_shell` | Base64-encoded commands bypass pattern matching | Write the command directly instead of encoding it | High |
| `core.obfuscation.eval_subshell` | eval of dynamic content bypasses command inspection | Execute the inner command directly | High |
| `core.obfuscation.curl_pipe_shell` | Downloading and executing untrusted code in one step | Download the script first with `curl -o script.sh`, review it, then execute | High |
| `core.obfuscation.python_exec` | Python used as a shell wrapper to bypass command inspection | Execute the shell command directly | Medium |

## Stencila

**Pack ID:** `core.stencila`

Guards the agent's own runtime, credentials, and publishing operations

### Safe patterns

| Rule ID | Pattern |
|---------|--------|
| `core.stencila.stencila_secrets_list` | `^stencila\s+secrets\s+list\b[^\|><]*$` |
| `core.stencila.stencila_auth_status` | `^stencila\s+auth\s+status\b[^\|><]*$` |
| `core.stencila.stencila_cloud_status` | `^stencila\s+cloud\s+status\b[^\|><]*$` |
| `core.stencila.stencila_db_status` | `^stencila\s+db\s+status\b[^\|><]*$` |
| `core.stencila.stencila_db_log` | `^stencila\s+db\s+log\b[^\|><]*$` |
| `core.stencila.stencila_db_verify` | `^stencila\s+db\s+verify\b[^\|><]*$` |
| `core.stencila.stencila_status` | `^stencila\s+status\b[^\|><]*$` |
| `core.stencila.stencila_formats_list` | `^stencila\s+formats\s+list\b[^\|><]*$` |
| `core.stencila.stencila_models_list` | `^stencila\s+models\s+list\b[^\|><]*$` |

### Destructive patterns

| Rule ID | Reason | Suggestion | Confidence |
|---------|--------|------------|:----------:|
| `core.stencila.secrets_modify` | Modifying secrets can break authentication | Use `stencila secrets list` to view secrets | High |
| `core.stencila.auth_modify` | Changing authentication state affects all sessions | Use `stencila auth status` to check auth state | High |
| `core.stencila.cloud_auth` | Changing cloud authentication affects deployments | Use `stencila cloud status` to check cloud state | High |
| `core.stencila.publish` | Publishing makes content publicly visible | Review content manually before publishing | High |
| `core.stencila.uninstall` | Uninstalling removes the runtime | This operation should not be performed by an agent | High |
| `core.stencila.push` | Pushing sends local changes to remote | Use `stencila status` to review changes first | Medium |
| `core.stencila.db_destructive` | Database reset/gc can lose data | Use `stencila db status` to inspect the database first | Medium |
| `core.stencila.clean` | Clean removes generated artifacts | Use `stencila status` to see what will be removed | Medium |

---

This documentation was generated from [`rust/agents/src/tool_guard/shell/packs/core.rs`](https://github.com/stencila/stencila/blob/main/rust/agents/src/tool_guard/shell/packs/core.rs).
