---
name: workspaces
description: Stencila workspace configuration and document tracking - stencila.toml sections, stencila init, track/status/untrack/clean, and what lives in the .stencila/ directory. Use when configuring a Stencila project, setting up remotes or outputs, interpreting stencila status, or deciding what to commit or clean.
user-invocable: false
---

# Stencila workspaces

A workspace is a directory containing `stencila.toml` (or
`stencila.local.toml`). Stencila finds the workspace by walking up from the
current directory. Configuration merges, lowest to highest precedence:
`~/.config/stencila/stencila.toml` → `<workspace>/stencila.toml` →
`<workspace>/stencila.local.toml` (for uncommitted, machine-local
overrides).

Initialize a workspace non-interactively:

```sh
NO_COLOR=1 stencila init --yes
```

Options: `--root <ROOT>` (site root directory), `--home <HOME>` (home page
file), `--outputs html,pdf` (output formats for executable documents).

## stencila.toml sections

| Section | Purpose |
|---|---|
| `[workspace]` | Workspace identity: Stencila Cloud `id` and `watch` ids (assigned automatically; rarely hand-edited) |
| `[remotes]` | Map local paths to remote service URLs (Google Docs, Microsoft 365, a Stencila site) for push/pull/sync |
| `[outputs]` | Output files to render and upload: `"report.pdf" = "report.smd"`, static files, glob patterns, and parameter spreads like `"{region}/report.pdf" = { source = "report.smd", arguments = { region = ["north", "south"] } }` |
| `[site]` | Site structure: `root`, routes, layout, navigation, formats |
| `[agents]` | Defaults for Stencila's own agents (e.g. default agent, commit attribution) |
| `[content-credentials]` | C2PA signing of rendered outputs (see the `publishing` skill) |

Example:

```toml
[remotes]
"report.smd" = "https://docs.google.com/document/d/..."

[outputs]
"report.pdf" = "report.smd"

[site]
root = "docs"
```

## Document tracking

Tracking records the state of documents so Stencila can detect staleness
and sync with remotes:

```sh
NO_COLOR=1 stencila track doc.smd --yes      # start tracking
NO_COLOR=1 stencila status --yes             # tracking status of documents
NO_COLOR=1 stencila status --as json --yes   # machine-readable
NO_COLOR=1 stencila untrack doc.smd --yes    # stop tracking
NO_COLOR=1 stencila clean --yes              # clean up the workspace
```

`stencila new doc.smd` creates a new, already-tracked document.
`stencila move` renames a tracked document so tracking follows it — prefer
it over plain `mv` for tracked files. `status --no-remotes` skips the
network round-trip when you only need local status.

## The .stencila/ directory

Created inside the workspace, it mixes Stencila's working state with
user-authored content:

- `artifacts/` — cached intermediate artifacts from conversion (downloads,
  OCR output, extracted media); safe to delete, re-created on demand
- `cache/` and `db.sqlite3` — document cache and workspace database
- `agents/`, `skills/`, `workflows/` — user-authored definitions for
  Stencila's own agent stack (distinct from this Claude Code plugin)

Stencila writes its own `.stencila/.gitignore` that ignores the transient
state (artifacts, cache, database) while keeping user-authored content
committable — do not add `.stencila/` wholesale to the project
`.gitignore`, and do not hand-edit the transient files.
