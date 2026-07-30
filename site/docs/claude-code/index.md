---
title: Claude Code Plugin
description: Use Claude Code to author, execute, convert and publish Stencila documents.
---

# Claude Code plugin

Stencila ships a first-party plugin for [Claude Code](https://claude.com/claude-code)
that makes Claude competent with Stencila documents and the `stencila` CLI:
it teaches Claude Stencila Markdown syntax, how execution and conversion
work, and adds explicit commands, agents, and workspace-aware hooks.

The plugin is for **Stencila users** working on their own documents and
projects in Claude Code. It complements — and is separate from —
[Stencila's own agents](../agents/), which run through
`stencila agents` and Stencila's TUI.

## Installation

In Claude Code:

```
/plugin marketplace add stencila/stencila
/plugin install stencila@stencila
```

The plugin does **not** bundle the Stencila CLI. If you don't already have
it, install it:

```sh
curl -LsSf https://stencila.io/install.sh | bash
```

or on Windows, download the ZIP for your platform from the
[latest release](https://github.com/stencila/stencila/releases). Then run
`/stencila:doctor` in Claude Code to verify everything is found.

## What you get

### Knowledge skills

Claude loads these automatically when they are relevant — no invocation
needed:

| Skill | Teaches Claude |
| ----- | -------------- |
| `authoring` | Stencila Markdown syntax: executable chunks, inline expressions, parameters, figures, math, metadata |
| `execution` | Kernels, staleness-driven execution, and debugging chunks that don't re-run |
| `conversion` | Format conversion, lossless vs lossy targets, importing papers by DOI/arXiv/PMC id |
| `publishing` | Themes, sites, Ghost/Zenodo publishing, content credentials |
| `workspaces` | `stencila.toml`, document tracking, the `.stencila/` directory |

The `authoring` skill's reference files are generated from this
documentation site, so they stay in sync with the CLI.

### Commands

Typed explicitly when you want an action performed:

| Command | Does |
| ------- | ---- |
| `/stencila:doctor` | Check the CLI installation, version, kernels, linters and tools — run this first when anything is wrong |
| `/stencila:render` | Render a document, surfacing execution errors |
| `/stencila:execute` | Execute a document's code |
| `/stencila:lint` | Lint documents and summarise diagnostics |
| `/stencila:convert` | Convert between formats, reporting losses |
| `/stencila:new` | Create a new tracked document |
| `/stencila:preview` | Open a live preview in the browser |
| `/stencila:snap` | Screenshot a document so Claude can visually verify it |

### Agents

- **`doc-author`** — drafts and edits executable documents, and never
  reports a document finished without rendering it and confirming zero
  execution errors.
- **`repro-checker`** — audits documents for reproducibility: re-executes
  everything without saving and reports erroring chunks, stale outputs,
  undeclared dependencies, and hard-coded paths.

### Hooks

- On session start, the plugin detects whether the project is a Stencila
  workspace and, if so, tells Claude where the CLI is, its version, and
  what documents and kernels are available.
- After Claude writes or edits a Stencila document, the plugin lints it and
  feeds any diagnostics back so Claude self-corrects.

Projects with no Stencila content pay nothing: both hooks exit immediately
and silently.

## Configuration

Via `/plugin` → manage → **stencila** → configure:

| Option | Default | Purpose |
| ------ | ------- | ------- |
| `stencila_path` | `stencila` | Path to the CLI binary — set this if you use a local build or a non-standard install location |
| `auto_lint` | `true` | Lint Stencila documents after Claude edits them |
| `auto_fix` | `false` | Auto-fix lint issues (modifies files on disk) |

## Relationship to Stencila's agent stack

Stencila has its own [agents](../agents/), [skills](../skills/) and
[workflows](../workflows/) that run via `stencila agents run` with any
model provider. The Claude Code plugin does not replace them: it brings
Stencila competence to sessions you run in Claude Code, while Stencila's
own stack remains the way to run agents from Stencila itself. The two
share the same underlying Agent Skills format, so knowledge is portable
between them.
