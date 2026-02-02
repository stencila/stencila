---
title: "`stencila site uploads`"
description: Manage site file uploads
---

Manage site file uploads

Configure the file upload feature that allows users to upload files to the repository via GitHub PRs. This enables non-technical users to contribute data updates (e.g., CSV files) without needing to use git directly.

# Usage

```sh
stencila site uploads [OPTIONS] [COMMAND]
```

# Examples

```bash
# Show current upload settings
stencila site uploads

# Enable uploads with defaults
stencila site uploads on

# Enable uploads for data directory
stencila site uploads on --path data

# Disable uploads
stencila site uploads off

# Configure allowed file types
stencila site uploads config --allowed-types csv --allowed-types json
```

# Subcommands

| Command               | Description               |
| --------------------- | ------------------------- |
| [`on`](on.md)         | Enable uploads            |
| [`off`](off.md)       | Disable uploads           |
| [`config`](config.md) | Configure upload settings |

# Options

| Name         | Description                      |
| ------------ | -------------------------------- |
| `-p, --path` | Path to the workspace directory. |
