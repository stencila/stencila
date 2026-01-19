---
title: "`stencila tools show`"
description: Show information about a specific tool
---

Show information about a specific tool

Displays information about a tool including its name, URL, description, version requirements, installation status, and file path. The version and path shown reflect the currently active environment managers (devbox, mise, etc.) if configured in the current directory, otherwise system-wide installation.

# Usage

```sh
stencila tools show <TOOL>
```

# Examples

```bash
# Show details about Pandoc
stencila tools show pandoc

# Show details about uv
stencila tools show uv

Supported tools
# See which tools are installed
stencila tools list --installed
```

# Arguments

| Name     | Description                               |
| -------- | ----------------------------------------- |
| `<NAME>` | The name of the tool to show details for. |
