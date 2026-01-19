---
title: "`stencila init`"
description: Initialize a workspace with stencila.toml configuration
---

Initialize a workspace with stencila.toml configuration

# Usage

```sh
stencila init [OPTIONS] [DIR]
```

# Examples

```bash
# Initialize current directory with interactive prompts
stencila init

# Initialize with all defaults (non-interactive)
stencila init --yes

# Initialize a specific directory
stencila init ./my-project

# Initialize with specific options
stencila init --root docs --home index.md

# Initialize with outputs for executable documents
stencila init --outputs docx,pdf
```

# Arguments

| Name    | Description                                                |
| ------- | ---------------------------------------------------------- |
| `[DIR]` | The workspace directory to initialize. Default value: `.`. |

# Options

| Name              | Description                                                              |
| ----------------- | ------------------------------------------------------------------------ |
| `-y, --yes <YES>` | Accept all defaults without prompting. Possible values: `true`, `false`. |
| `--root`          | Site root directory (skip interactive prompt).                           |
| `--home`          | Home page file (skip interactive prompt).                                |
| `--outputs`       | Output formats for executable documents (comma-separated).               |

# Note

This creates a stencila.toml configuration file with site settings,
routes, and output configurations based on repository analysis.
