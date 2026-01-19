---
title: "`stencila tools env`"
description: Detect environment manager configuration for a directory
---

Detect environment manager configuration for a directory

Searches the specified directory (and parent directories) for configuration files that indicate the presence of environment or package managers. This helps understand what development environment is configured for a project.

Displays both the manager information and the content of the configuration files for inspection.

# Usage

```sh
stencila tools env [PATH]
```

# Examples

```bash
# Check current directory for environment configuration
stencila tools env

# Check a specific project directory
stencila tools env /path/to/project

# Check parent directory
stencila tools env ..
```

# Arguments

| Name     | Description                                                                       |
| -------- | --------------------------------------------------------------------------------- |
| `[PATH]` | The directory to check for environment manager configuration. Default value: `.`. |
