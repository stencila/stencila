---
title: "`stencila pull`"
description: Pull a document from a remote service
---

Pull a document from a remote service

# Usage

```sh
stencila pull [OPTIONS] <PATH> [URL]
```

# Examples

```bash
# Pull from a specific Google Doc URL
stencila pull document.smd https://docs.google.com/document/d/abc123

# Pull from the tracked remote (if only one exists)
stencila pull document.smd

# Pull from tracked Google Doc (when multiple remotes exist)
stencila pull document.smd --from gdoc

# Pull from tracked Microsoft 365 document
stencila pull document.smd --from m365

# Pull from a GitHub Issue URL
stencila pull document.smd https://github.com/org/repo/issues/123

# Pull without merging (replace local file)
stencila pull document.smd --no-merge

# Pull without saving to stencila.toml
stencila pull document.smd --from gdoc --no-config

# Pull and enable bi-directional watch
stencila pull document.smd --from gdoc --watch

# Pull all documents from email attachments using embedded path metadata
stencila pull - --from https://api.stencila.cloud/v1/watches/wAbC12345/email/attachments
```

# Arguments

| Name     | Description                     |
| -------- | ------------------------------- |
| `<PATH>` | The path to the local document. |
| `[URL]`  | The URL to pull from.           |

# Options

| Name                 | Description                                                                                                                                    |
| -------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------- |
| `-f, --from`         | Select which remote service to pull from.                                                                                                      |
| `--no-merge`         | Do not merge, just replace. Possible values: `true`, `false`.                                                                                  |
| `--no-config`        | Do not save remote to stencila.toml. Possible values: `true`, `false`.                                                                         |
| `-w, --watch`        | Enable watch after successful pull. Possible values: `true`, `false`.                                                                          |
| `-d, --direction`    | The sync direction (only used with --watch).                                                                                                   |
| `-p, --pr-mode`      | The GitHub PR mode (only used with --watch). Possible values: `draft` (Create PRs as drafts (default)), `ready` (Create PRs ready for review). |
| `--debounce-seconds` | Debounce time in seconds (10-86400, only used with --watch).                                                                                   |

**Possible values of `--direction`**

| Value         | Description                                                                         |
| ------------- | ----------------------------------------------------------------------------------- |
| `bi`          | Bi-directional sync: changes from remote create PRs, changes to repo push to remote |
| `from-remote` | One-way sync from remote: only remote changes create PRs                            |
| `to-remote`   | One-way sync to remote: only repo changes push to remote                            |
