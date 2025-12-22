---
title: Configuration Reference
description: Reference documentation for stencila.toml configuration files
---

# Configuration Reference

Stencila uses `stencila.toml` files for project configuration. This reference documents all available configuration options.

## Configuration Files

- `stencila.toml` - Main configuration file
- `stencila.local.toml` - Local overrides (typically gitignored)

## Sections

| Section | Description |
|---------|-------------|
| [`[site]`](site) | Configuration for a site |
| [`[site.routes]`](site#routes) | Custom routes for serving content Routes map URL paths to files, redirects, or spread configurations |
| [`[remotes]`](remotes) | Remote synchronization configuration Maps local paths to remote service URLs |

## Examples

