---
title: "`stencila tools install`"
description: Install a tool or setup development environment
---

Install a tool or setup development environment

When provided with one or more tool names as arguments, installs those tools. When run without arguments, automatically detects and installs environment managers, tools, and dependencies based on configuration files found in the project directory.

# Usage

```sh
stencila tools install [OPTIONS] [TOOL]...
```

# Examples

```bash
Tool Installation Examples
# Install mise (tool version manager)
stencila tools install mise

# Install uv (Python package manager)
stencila tools install uv

# Install multiple tools at once
stencila tools install mise uv ruff

# Force reinstall an already installed tool
stencila tools install --force ruff

Environment Setup Examples
# Install all dependencies from config files in current directory
stencila tools install

# Install dependencies from config files in specific directory
stencila tools install -C /path/to/project

# Show what would be installed without actually installing
stencila tools install --dry-run

# Skip Python dependencies during setup
stencila tools install --skip-python

Setup phases (when no tool specified)
1. Install environment managers (mise, devbox, etc.) if needed
2. Install tools from environment manager configs
3. Setup Python dependencies (pyproject.toml, requirements.txt)
4. Setup R dependencies (renv.lock, DESCRIPTION)

Supported tools
# See which tools can be installed
stencila tools list --installable
```

# Arguments

| Name      | Description                                                                                           |
| --------- | ----------------------------------------------------------------------------------------------------- |
| `[NAMES]` | The name(s) of the tool(s) to install (if not provided, installs all dependencies from config files). |

# Options

| Name            | Description                                                                                                       |
| --------------- | ----------------------------------------------------------------------------------------------------------------- |
| `-C, --path`    | The directory to setup when installing from config files (defaults to current directory).                         |
| `--skip-env`    | Skip environment manager tool installation (only when installing from configs). Possible values: `true`, `false`. |
| `--skip-python` | Skip Python dependency installation (only when installing from configs). Possible values: `true`, `false`.        |
| `--skip-r`      | Skip R dependency installation (only when installing from configs). Possible values: `true`, `false`.             |
| `-f, --force`   | Force installation even if the tool is already installed. Possible values: `true`, `false`.                       |
| `--dry-run`     | Show which tools would be installed without actually installing them. Possible values: `true`, `false`.           |
