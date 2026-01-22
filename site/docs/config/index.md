---
title: Configuration Reference
description: Reference documentation for stencila.toml configuration files
---

Stencila uses TOML `stencila.toml` files for project configuration. This reference documents all available configuration options and how configuration is resolved and validated.

# Overview

Stencila configuration is layered and location-aware. Configuration can come from user-level files and from a workspace, and the final configuration is produced by merging those files in a defined order. Values are validated after merge and relative paths are resolved relative to the workspace root.

Use configuration to:

- declare workspace metadata
- define remotes for sync
- define outputs and spreads
- configure site structure, routes, layout, navigation, and related settings

# File Locations

Stencila reads configuration from two locations:

- User config directory: `~/.config/stencila/`
- Workspace directory: the directory containing the nearest `stencila.toml` or `stencila.local.toml`

The workspace directory can contain:

- `stencila.toml` for shared configuration
- `stencila.local.toml` for machine- or user-specific overrides

The user config directory contains:

- `stencila.toml` for user defaults

Local files are intended for settings that should not be committed (secrets, personal overrides, local paths).

# Workspace Resolution

When Stencila needs a workspace configuration, it:

1. Starts from the current working directory (or the path provided).
2. Walks up the directory tree looking for `stencila.toml`.
3. If none is found, walks up looking for `stencila.local.toml`.
4. If neither exists, falls back to the user config directory if it contains `stencila.toml`.
5. If no config exists anywhere, uses the start path as the workspace directory.

The directory that contains the selected config file is the workspace directory for resolution and validation.

# Merge Order and Precedence

Configuration files are merged in this order (lowest to highest precedence):

1. `~/.config/stencila/stencila.toml`
2. `<workspace>/stencila.toml`
3. `<workspace>/stencila.local.toml`

Files that do not exist are skipped. When the same key appears in multiple files, the value from the higher-precedence file replaces the lower one.

# Relative Paths

Some configuration fields accept relative paths. These are resolved relative to the workspace root (the directory containing `stencila.toml` or `stencila.local.toml`), regardless of which file defined them. This keeps path interpretation consistent across merged configuration.

# Example: Layered Configuration

This example shows how user and workspace config combine, with later files overriding earlier values.

`~/.config/stencila/stencila.toml`
```toml
[site]
formats = ["md"]
```

`<workspace>/stencila.toml`
```toml
[site]
root = "docs"
formats = ["md", "pdf"]
```

`<workspace>/stencila.local.toml`
```toml
[site]
formats = ["md"] # Local override for this machine
```

Final result for `site.formats` is `["md"]`, and `site.root` remains `docs`.

# Validation

After all files are merged, configuration is validated. Validation includes:

- workspace configuration validation
- site configuration validation
- navigation item validation (must be internal routes)
- route configuration validation
- remote configuration validation
- output configuration validation

These checks go beyond schema-level validation and enforce runtime constraints.

# Watching for Changes

When a workspace `stencila.toml` exists, the config system watches for changes to both `stencila.toml` and `stencila.local.toml` in the workspace directory. When either file changes, configuration is reloaded and subscribers receive updated config snapshots. Failed reloads keep the last known valid configuration.

# CLI Behavior

The CLI provides helpers for editing configuration:

- `stencila config set` and `stencila config unset` update a key path in the nearest, local, or user config file.
- Edits preserve existing formatting and comments.
- `stencila init` can generate a starting `stencila.toml` based on repository analysis.

Some keys are managed by dedicated commands and should not be set directly. For example:

- `workspace.id` is assigned when creating a workspace.
- `site.domain` is set via `stencila site domain set` and requires domain validation.

See [`stencila config`](../cli/config/index.md) and [`stencila init`](../cli/init.md) for full CLI usage.

```bash
stencila config get
stencila config set site.root docs
stencila config unset site.root
```

```bash
stencila init
stencila init --yes
```

## Sections

| Section | Description |
|---------|-------------|
| [`[workspace]`](workspace) | Workspace configuration. |
| [`[remotes]`](remotes) | Remote synchronization configuration. |
| [`[outputs]`](outputs) | Outputs configuration. |
| [`[site]`](site) | Configuration for a site |
