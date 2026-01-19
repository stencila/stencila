---
title: "`stencila site branch`"
description: Manage branches for the workspace site
---

Manage branches for the workspace site

# Usage

```sh
stencila site branch <COMMAND>
```

# Examples

```bash
# List all deployed branches
stencila site branch list

# Delete a feature branch
stencila site branch delete feature-xyz

# Delete a branch without confirmation
stencila site branch delete feature-xyz --force
```

# Subcommands

| Command               | Description                   |
| --------------------- | ----------------------------- |
| [`list`](list.md)     | List all deployed branches    |
| [`delete`](delete.md) | Delete a branch from the site |
