---
title: File Tools
description: Tools for reading, writing, editing, and searching files, and the guard rules that protect against risky file operations.
---

The file tools give agents the ability to read, write, and search files within the workspace. Each tool call is evaluated by the [file guard](#guard-rules) before execution.

## `read_file`

Reads the contents of a file with line-number annotations. Supports partial reads via offset and limit parameters.

| Parameter | Type | Required | Description |
| --------- | ---- | :------: | ----------- |
| `file_path` | string | ✅ | Absolute path to the file to read |
| `offset` | integer (min: 1) | | Line number to start reading from (1-based). Defaults to 1 |
| `limit` | integer (min: 1) | | Maximum number of lines to read. Defaults to 2000 |

Output is formatted with line numbers: each line is prefixed with its number and a `|` separator. Image files are returned as inline image data if under the size limit, or as a text placeholder otherwise.

## `write_file`

Creates or overwrites a file, creating any parent directories as needed.

| Parameter | Type | Required | Description |
| --------- | ---- | :------: | ----------- |
| `file_path` | string | ✅ | Absolute path to the file to write |
| `content` | string | ✅ | The content to write to the file |

## `edit_file`

Performs an exact string replacement in a file. By default the `old_string` must appear exactly once; set `replace_all` to true for multiple replacements.

| Parameter | Type | Required | Description |
| --------- | ---- | :------: | ----------- |
| `file_path` | string | ✅ | Absolute path to the file to edit |
| `old_string` | string | ✅ | The exact string to find and replace |
| `new_string` | string | ✅ | The replacement string |
| `replace_all` | boolean | | If true, replace all occurrences. Defaults to false |

Returns an error if the file does not exist, the `old_string` is not found, or there are multiple matches without `replace_all`.

## `apply_patch`

Parses and applies a patch in the v4a format, supporting file creation, deletion, update, and rename in a single operation.

| Parameter | Type | Required | Description |
| --------- | ---- | :------: | ----------- |
| `patch` | string | ✅ | The patch content in v4a format |

The patch format uses markers like `*** Add File:`, `*** Delete File:`, `*** Update File:`, and `*** Move to:` to describe operations. Update hunks use `@@` context hints for positioning and support fuzzy matching (whitespace normalization, smart quote normalization) when exact matches fail.

> [!info]
> This tool is only available when using models that support the v4a patch format.

## `grep`

Searches file contents using a regular expression pattern, returning matching lines with file paths and line numbers.

| Parameter | Type | Required | Description |
| --------- | ---- | :------: | ----------- |
| `pattern` | string | ✅ | Regular expression pattern to search for |
| `path` | string | | Directory or file to search in. Defaults to the working directory |
| `glob_filter` | string | | Glob pattern to filter files (e.g., `"*.rs"`, `"*.py"`) |
| `case_insensitive` | boolean | | If true, perform case-insensitive matching. Defaults to false |
| `max_results` | integer (min: 1) | | Maximum number of matching lines to return. Defaults to 100 |

## `glob`

Finds files matching a glob pattern, sorted by modification time (newest first).

| Parameter | Type | Required | Description |
| --------- | ---- | :------: | ----------- |
| `pattern` | string | ✅ | Glob pattern to match files (e.g., `"**/*.rs"`, `"src/*.py"`) |
| `path` | string | | Base directory for the search. Defaults to the working directory |

## `list_dir`

Lists the contents of a directory. Directories are shown with a trailing slash; files include their size.

| Parameter | Type | Required | Description |
| --------- | ---- | :------: | ----------- |
| `path` | string | ✅ | Absolute path to the directory to list |
| `depth` | integer (min: 1) | | Maximum depth to recurse. Defaults to 1 (immediate children only) |

Returns one entry per line. Empty directories return `"Empty directory."`.

> [!info]
> This tool is only available when using Gemini models.

## `read_many_files`

Reads multiple files in a single call. Returns the contents of each file with a header delimiter. Errors for individual files are reported inline without aborting the batch.

| Parameter | Type | Required | Description |
| --------- | ---- | :------: | ----------- |
| `paths` | array of strings | ✅ | Absolute file paths to read |

Each file's output is prefixed with an `=== /path/to/file ===` header. Image files are reported as placeholders with their media type. If a file cannot be read, the error message appears in place of its content so that other files in the batch are still returned.

> [!info]
> This tool is only available when using Gemini models.

## Guard Rules

The file guard normalizes every path (tilde expansion, `..` resolution, relative to absolute) and then checks it against a set of rules. For tools that operate on multiple paths (e.g., `apply_patch`), each path is evaluated and the strictest verdict wins.

| Rule ID | Applies to | Reason | Suggestion | Low | Medium | High |
| ------- | ---------- | ------ | ---------- | :-: | :----: | :-: |
| `file.system_path_read` | Read | Reading virtual/device filesystem paths can expose kernel state, process internals, and credentials | Use specific inspection commands instead (e.g., `uname` for system info, `env` for environment) | Deny | Deny | Deny |
| `file.sensitive_path_read` | Read | Reading credential and key files can leak secrets into the agent's context window | Use targeted commands that don't expose raw secrets (e.g., `ssh-keygen -l -f` to check a key fingerprint) | Deny | Deny | Warn |
| `file.outside_workspace_read` | Read | Read target is outside the session workspace root | Verify the path is intended, or copy the file into the workspace first | Deny | Warn | Allow |
| `file.system_path_write` | Write | Writing to system paths can break OS configuration and stability | Use application-level config files in the project directory instead | Deny | Deny | Deny |
| `file.sensitive_path_write` | Write | Writing to credential files or shell startup files is a persistence and credential-tampering vector | Modify project-local configuration instead of user-level dotfiles | Deny | Deny | Deny |
| `file.outside_workspace_write` | Write | Write target is outside the session workspace root | Write to a path within the project workspace, or verify the target path is intended | Deny | Warn | Allow |
| `file.protected_file_overwrite` | Write | Writing to .git/ internals can corrupt repository state | Edit hooks or config via `git config` or manual review outside the agent | Deny | Deny | Warn |
| `file.apply_patch_delete_many` | Patch | Bulk file deletion in a single patch may indicate a hallucinated cleanup (≥5 files) | Break the patch into smaller steps deleting fewer than 5 files each, or verify the file list is correct | Deny | Warn | Warn |

### Path Lists

#### System read paths

These paths trigger `file.system_path_read` (prefix match):

- `/proc/`
- `/sys/`
- `/dev/`

#### Sensitive paths (read)

These paths trigger `file.sensitive_path_read`:

- `/etc/shadow` (exact)
- `/etc/gshadow` (exact)
- `/etc/sudoers` (exact)
- `~/.ssh/` (prefix)
- `~/.gnupg/` (prefix)
- `~/.aws/` (prefix)
- `~/.config/gcloud/` (prefix)
- `.env` (basename)
- `.netrc` (basename)

#### Sensitive paths (write)

These paths trigger `file.sensitive_path_write` (superset of read-sensitive paths):

- `~/.ssh/` (prefix)
- `~/.gnupg/` (prefix)
- `~/.aws/` (prefix)
- `~/.config/gcloud/` (prefix)
- `.env` (basename)
- `.netrc` (basename)
- `~/.bashrc` (exact)
- `~/.bash_profile` (exact)
- `~/.profile` (exact)
- `~/.zshrc` (exact)
- `~/.zprofile` (exact)

#### System write paths

These paths trigger `file.system_path_write` (prefix match):

- `/etc/`
- `/usr/`
- `/boot/`
- `/sbin/`
- `/bin/`
- `/lib/`
- `/proc/`
- `/sys/`
- `/dev/`

#### Protected directory components

These directory names trigger `file.protected_file_overwrite` when they appear as a path component:

- `.git`
