---
title: "`stencila site push`"
description: Push site content to Stencila Cloud
---

Push site content to Stencila Cloud

# Usage

```sh
stencila site push [OPTIONS] [PATH]
```

# Examples

```bash
# Push site content to cloud (uses site.root if configured)
stencila site push

# Push a specific directory
stencila site push ./site/docs

# Push a specific file
stencila site push ./site/report.md

# Force push (ignore unchanged files)
stencila site push --force

# Push using the development web distribution (for maintainers)
stencila site push --dev
```

# Arguments

| Name     | Description                                           |
| -------- | ----------------------------------------------------- |
| `[PATH]` | Path to push (file or directory). Default value: `.`. |

# Options

| Name          | Description                                                                                              |
| ------------- | -------------------------------------------------------------------------------------------------------- |
| `-f, --force` | Force push without checking etags. Possible values: `true`, `false`.                                     |
| `--dev`       | Use the development web distribution instead of the versioned release. Possible values: `true`, `false`. |
