---
title: "`stencila site list`"
description: List all routes (configured and file-implied)
---

List all routes (configured and file-implied)

# Usage

```sh
stencila site list [OPTIONS]
```

# Examples

```bash
# List all routes (configured and file-implied)
stencila site
stencila site list

# Show expanded spread route variants
stencila site list --expanded

# Show routes for static files (e.g. images)
stencila site list --statics

# Filter routes by route prefix
stencila site list --route /docs

# Filter routes by source file path prefix
stencila site list --path docs/
```

# Options

| Name                    | Description                                                                        |
| ----------------------- | ---------------------------------------------------------------------------------- |
| `--expanded <EXPANDED>` | Show expanded spread route variants. Possible values: `true`, `false`.             |
| `--statics <STATICS>`   | Show routes for static files (e.g. images, CSS). Possible values: `true`, `false`. |
| `--route`               | Filter by route prefix.                                                            |
| `--path`                | Filter by source file path prefix.                                                 |
