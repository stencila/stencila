---
title: "`stencila tools`"
description: Manage tools and environments used by Stencila
---

Manage tools and environments used by Stencila

Provides a unified interface for managing various tools including programming languages, package managers, linters, and converters. It automatically detects and integrates with environment managers like devbox, mise, and uv to provide isolated and reproducible environments.

# Usage

```sh
stencila tools [COMMAND]
```

# Examples

```bash
# List all available tools
stencila tools

# Show details about a specific tool
stencila tools show python

# Install a tool
stencila tools install mise

# Install multiple tools
stencila tools install mise uv ruff

# Install all dependencies from config files
stencila tools install

# Detect environment configuration in current directory
stencila tools env

# Run a command with automatic environment detection
stencila tools run -- python script.py
```

# Subcommands

| Command                 | Description                                                  |
| ----------------------- | ------------------------------------------------------------ |
| [`list`](list.md)       | List available tools and their installation status           |
| [`show`](show.md)       | Show information about a specific tool                       |
| [`install`](install.md) | Install a tool or setup development environment              |
| [`env`](env.md)         | Detect environment manager configuration for a directory     |
| [`run`](run.md)         | Run a command with automatic environment detection and setup |
