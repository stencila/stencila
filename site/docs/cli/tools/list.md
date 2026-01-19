---
title: "`stencila tools list`"
description: List available tools and their installation status
---

List available tools and their installation status

Displays a table of all tools that Stencila can manage, including their type, required version, available version, and installation path. The versions and paths shown reflect the currently active environment managers (devbox, mise, etc.) if configured in the current directory, otherwise system-wide installations.

# Usage

```sh
stencila tools list [OPTIONS]
```

# Examples

```bash
# List all tools
stencila tools list

# List only installed tools
stencila tools list --installed

# List only installable tools
stencila tools list --installable

# List only execution tools (programming languages)
stencila tools list --type execution

# Export tool list as Model Context Protocol tool specifications
stencila tools list --as json

# Display tool list as YAML
stencila tools list --as yaml
```

# Options

| Name                          | Description                                                                                                                               |
| ----------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------- |
| `-t, --type <TYPE>`           | Only list tools of a particular type. Possible values: `collaboration`, `conversion`, `environments`, `execution`, `linting`, `packages`. |
| `--installed <INSTALLED>`     | Only list tools that are installed. Possible values: `true`, `false`.                                                                     |
| `--installable <INSTALLABLE>` | Only list tools that can be installed automatically. Possible values: `true`, `false`.                                                    |
| `-a, --as <AS>`               | Output format for tool specifications. Possible values: `json`, `yaml`, `toml`.                                                           |
