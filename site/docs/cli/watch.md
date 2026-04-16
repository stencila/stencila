---
title: "`stencila watch`"
description: Enable automatic sync for the workspace or a document
---

Enable automatic sync for the workspace or a document

When run without a path, enables workspace-level watching that runs `update.sh` on each git push (for automatic site/outputs publishing).

When run with a path, creates a watch in Stencila Cloud that automatically syncs changes between a remote (Google Docs or M365) and a GitHub repository.

# Usage

```sh
stencila watch [OPTIONS] [PATH] [TARGET]
```

# Examples

```bash
# Enable workspace watch (runs update.sh on each push)
stencila watch

# Enable watch on the tracked remote for a file
stencila watch report.md

# Watch a specific remote (if document has multiple)
stencila watch report.md gdoc
stencila watch report.md https://docs.google.com/document/d/abc123

# Enable watch with one-way sync from remote to repo
stencila watch report.md gdoc --direction from-remote

# Enable watch with ready-for-review PRs
stencila watch report.md gdoc --pr-mode ready

# Note: The document must already be pushed to a remote
stencila push report.md --to gdoc
stencila watch report.md
```

# Arguments

| Name       | Description                                              |
| ---------- | -------------------------------------------------------- |
| `[PATH]`   | The path to the document to watch (optional).            |
| `[TARGET]` | The target remote to watch (only used with a file path). |

# Options

| Name                 | Description                                                      |
| -------------------- | ---------------------------------------------------------------- |
| `-d, --direction`    | The sync direction (only used with a file path).                 |
| `-p, --pr-mode`      | The GitHub PR mode (only used with a file path).                 |
| `--debounce-seconds` | Debounce time in seconds (10-86400, only used with a file path). |

**Possible values of `--direction`**

| Value         | Description                                  |
| ------------- | -------------------------------------------- |
| `bi`          | Bidirectional automation                     |
| `from-remote` | One-way automation from remote to repository |
| `to-remote`   | One-way automation from repository to remote |

**Possible values of `--pr-mode`**

| Value   | Description                           |
| ------- | ------------------------------------- |
| `draft` | Create pull requests as drafts        |
| `ready` | Create pull requests ready for review |
