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
```

# Arguments

| Name     | Description                                           |
| -------- | ----------------------------------------------------- |
| `[PATH]` | Path to push (file or directory). Default value: `.`. |

# Options

| Name          | Description                                                          |
| ------------- | -------------------------------------------------------------------- |
| `-f, --force` | Force push without checking etags. Possible values: `true`, `false`. |
