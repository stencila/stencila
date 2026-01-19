---
title: "`stencila site reviews`"
description: Manage site reviews configuration
---

Manage site reviews configuration

Site reviews allow readers to submit comments and suggestions on site pages. The `public` and `anon` settings are enforced by Stencila Cloud and synced between local config and the cloud.

# Usage

```sh
stencila site reviews [OPTIONS] [COMMAND]
```

# Examples

```bash
# Show current review settings
stencila site reviews

# Enable reviews with defaults
stencila site reviews on

# Disable reviews
stencila site reviews off

# Enable public submissions
stencila site reviews config --public

# Disable anonymous submissions
stencila site reviews config --no-anon
```

# Subcommands

| Command               | Description               |
| --------------------- | ------------------------- |
| [`on`](on.md)         | Enable reviews            |
| [`off`](off.md)       | Disable reviews           |
| [`config`](config.md) | Configure review settings |

# Options

| Name         | Description                      |
| ------------ | -------------------------------- |
| `-p, --path` | Path to the workspace directory. |
