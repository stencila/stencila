---
title: "`stencila site domain clear`"
description: Remove the custom domain from the site
---

Remove the custom domain from the site

# Usage

```sh
stencila site domain clear [OPTIONS]
```

# Examples

```bash
# Remove custom domain from the current workspace's site
stencila site domain clear

# Remove custom domain from another workspace's site
stencila site domain clear --path /path/to/workspace
```

# Options

| Name         | Description                                                     |
| ------------ | --------------------------------------------------------------- |
| `-p, --path` | Path to the workspace directory containing .stencila/site.yaml. |
