---
title: "`stencila site show`"
description: Show details of the workspace site
---

Show details of the workspace site

# Usage

```sh
stencila site show [OPTIONS]
```

# Examples

```bash
# View details of the current workspace's site
stencila site
stencila site show

# View details of another workspace's site
stencila site show --path /path/to/workspace
```

# Options

| Name         | Description                                                     |
| ------------ | --------------------------------------------------------------- |
| `-p, --path` | Path to the workspace directory containing .stencila/site.yaml. |
