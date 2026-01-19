---
title: "`stencila site branch list`"
description: List all deployed branches
---

List all deployed branches

# Usage

```sh
stencila site branch list [OPTIONS]
```

# Examples

```bash
# List branches for the current workspace's site
stencila site branch list

# List branches for another workspace's site
stencila site branch list --path /path/to/workspace
```

# Options

| Name         | Description                                                     |
| ------------ | --------------------------------------------------------------- |
| `-p, --path` | Path to the workspace directory containing .stencila/site.yaml. |
