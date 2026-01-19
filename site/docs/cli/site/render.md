---
title: "`stencila site render`"
description: Render site content to a directory
---

Render site content to a directory

# Usage

```sh
stencila site render [OPTIONS] <OUTPUT>
```

# Examples

```bash
# Render site to a directory
stencila site render ./dist

# Render specific routes
stencila site render ./dist --route /docs/

# Render from a specific source
stencila site render ./dist --source ./content
```

# Arguments

| Name       | Description                          |
| ---------- | ------------------------------------ |
| `<OUTPUT>` | Output directory for rendered files. |

# Options

| Name           | Description                                                                   |
| -------------- | ----------------------------------------------------------------------------- |
| `-s, --source` | Source directory (uses site.root if configured, otherwise current directory). |
| `--route`      | Filter by route prefix (only render matching routes).                         |
| `--path`       | Filter by source file path prefix.                                            |
