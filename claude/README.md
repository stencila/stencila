# Stencila plugin for Claude Code

A [Claude Code](https://claude.com/claude-code) plugin that makes Claude
competent at authoring, executing, converting and publishing
[Stencila](https://stencila.io) documents.

## Installation

In Claude Code:

```
/plugin marketplace add stencila/stencila
/plugin install stencila@stencila
```

The plugin does **not** bundle the `stencila` CLI. Install it separately:

```sh
curl -LsSf https://stencila.io/install.sh | bash
```

or on Windows download the release ZIP from
<https://github.com/stencila/stencila/releases>. Run `/stencila:doctor` in
Claude Code to check your installation.

## Components

### Knowledge skills

Loaded automatically by Claude when relevant.

| Skill | Teaches |
|---|---|
| `authoring` | Stencila Markdown syntax: executable code, math, figures, control flow, metadata |
| `execution` | Kernels, staleness-driven execution, and debugging chunks that don't re-run |
| `conversion` | Format conversion, lossless vs lossy targets, and ingesting by DOI/arXiv/PMC id |
| `publishing` | Rendering to sites, themes, PDF/DOCX, Ghost/Zenodo, and content credentials |
| `workspaces` | `stencila.toml` configuration, document tracking, and the `.stencila/` directory |

### Action commands

Typed explicitly by the user; never fired automatically.

| Command | Does |
|---|---|
| `/stencila:doctor` | Check the CLI installation, version, kernels, linters and tools |
| `/stencila:render` | Render a document and surface execution errors |
| `/stencila:execute` | Execute a document's code |
| `/stencila:lint` | Lint documents and summarise diagnostics |
| `/stencila:convert` | Convert between formats, reporting losses |
| `/stencila:new` | Create a new tracked document |
| `/stencila:preview` | Open a live preview |
| `/stencila:snap` | Screenshot a document for visual verification |

### Agents

| Agent | Purpose |
|---|---|
| `doc-author` | Drafts and edits executable documents; verifies by rendering before reporting done |
| `repro-checker` | Audits documents for reproducibility; reports rather than fixes |

### Hooks

| Hook | Purpose |
|---|---|
| `SessionStart` | Detects a Stencila workspace and injects CLI path, version and document inventory |
| `PostToolUse` | Lints Stencila documents after Claude edits them (configurable) |

Both hooks exit immediately and silently in projects with no Stencila content.

## Configuration

Via `/plugin` → manage → stencila → configure:

| Option | Default | Purpose |
|---|---|---|
| `stencila_path` | `stencila` | Path to the CLI binary (useful for local builds) |
| `auto_lint` | `true` | Lint Stencila documents after edits |
| `auto_fix` | `false` | Auto-fix lint issues (modifies files on disk) |

## Development

This directory lives in the [stencila/stencila](https://github.com/stencila/stencila)
monorepo. The reference files under `skills/authoring/references/` are
generated from `site/docs/documents/` — do not edit them by hand; run
`make generated` here instead.

```sh
make -C claude lint   # validate manifests, check documented flags against the CLI
make -C claude test   # parse every smd example in the skills
```

To try the plugin without installing it:

```sh
claude --plugin-dir ./claude
```
