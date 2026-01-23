---
title: "`stencila push`"
description: Push content to Stencila Cloud and remote services
---

Push content to Stencila Cloud and remote services

This unified command can push sites, outputs, and remote documents. Use flags to specify what to push, or use subcommands for more control: - `stencila site push` for site-specific options - `stencila outputs push` for output-specific options

# Usage

```sh
stencila push [OPTIONS] [PATH] [-- <ARGS>...]
```

# Examples

```bash
# Push everything (site, outputs, and remotes)
stencila push --all

# Push only site content
stencila push --site

# Push only outputs
stencila push --outputs

# Push only remotes (Google Docs, M365)
stencila push --remotes

# Push a document to Google Docs
stencila push document.smd --to gdoc

# Push a document to Microsoft 365
stencila push document.smd --to m365

# Push to specific remote
stencila push document.smd --to https://docs.google.com/document/d/abc123

# Push with execution first
stencila push report.smd --to gdoc -- arg1=value1

# Force create new document
stencila push document.smd --to gdoc --new

# Spread push to GDocs (creates multiple docs)
stencila push report.smd --to gdoc --spread -- region=north,south
```

# Arguments

| Name     | Description                                         |
| -------- | --------------------------------------------------- |
| `[PATH]` | The path of the document to push (for remote push). |
| `[ARGS]` | Arguments to pass to the document for execution.    |

# Options

| Name                 | Description                                                                                                                                    |
| -------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------- |
| `--site`             | Push site content to Stencila Cloud. Possible values: `true`, `false`.                                                                         |
| `--outputs`          | Push outputs to Stencila Cloud. Possible values: `true`, `false`.                                                                              |
| `--remotes`          | Push to remote document services (Google Docs, Microsoft 365). Possible values: `true`, `false`.                                               |
| `--all`              | Push everything (site, outputs, and remotes). Possible values: `true`, `false`.                                                                |
| `-t, --to`           | The target to push to (for remote push).                                                                                                       |
| `-n, --new`          | Create a new document instead of updating an existing one. Possible values: `true`, `false`.                                                   |
| `--no-execute`       | Do not execute the document before pushing it. Possible values: `true`, `false`.                                                               |
| `--no-config`        | Do not save remote to stencila.toml. Possible values: `true`, `false`.                                                                         |
| `-w, --watch`        | Enable watch after successful push. Possible values: `true`, `false`.                                                                          |
| `-d, --direction`    | The sync direction (only used with --watch).                                                                                                   |
| `-p, --pr-mode`      | The GitHub PR mode (only used with --watch). Possible values: `draft` (Create PRs as drafts (default)), `ready` (Create PRs ready for review). |
| `--debounce-seconds` | Debounce time in seconds (10-86400, only used with --watch).                                                                                   |
| `--spread`           | Enable spread push mode for multi-variant execution.                                                                                           |
| `--case`             | Explicit cases for spread=cases mode.                                                                                                          |
| `--title`            | Title template for GDocs/M365 spread push.                                                                                                     |
| `--fail-fast`        | Stop on first error instead of continuing with remaining variants. Possible values: `true`, `false`.                                           |
| `--spread-max`       | Maximum number of spread runs allowed (default: 100). Default value: `100`.                                                                    |

**Possible values of `--direction`**

| Value         | Description                                                                         |
| ------------- | ----------------------------------------------------------------------------------- |
| `bi`          | Bi-directional sync: changes from remote create PRs, changes to repo push to remote |
| `from-remote` | One-way sync from remote: only remote changes create PRs                            |
| `to-remote`   | One-way sync to remote: only repo changes push to remote                            |

**Possible values of `--spread`**

| Value   | Description                                            |
| ------- | ------------------------------------------------------ |
| `grid`  | Cartesian product of multi-valued parameters (default) |
| `zip`   | Positional pairing of multi-valued parameters          |
| `cases` | Explicitly enumerated parameter sets via `--case`      |
