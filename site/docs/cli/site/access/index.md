---
title: "`stencila site access`"
description: Manage access restrictions for the workspace site
---

Manage access restrictions for the workspace site

# Usage

```sh
stencila site access [OPTIONS] [COMMAND]
```

# Examples

```bash
# Show current access restrictions
stencila site access

# Make site public (remove all restrictions)
stencila site access --public

# Enable team access restriction
stencila site access team

# Disable team access restriction
stencila site access team --off

# Set a password for the site
stencila site access password

# Clear the password
stencila site access password --clear
```

# Subcommands

| Command                   | Description                    |
| ------------------------- | ------------------------------ |
| [`team`](team.md)         | Manage team access restriction |
| [`password`](password.md) | Manage password protection     |

# Options

| Name         | Description                                                                              |
| ------------ | ---------------------------------------------------------------------------------------- |
| `--public`   | Make the site public (remove all access restrictions). Possible values: `true`, `false`. |
| `-p, --path` | Path to the workspace directory.                                                         |
