---
title: "`stencila lint`"
description: Lint one or more documents
---

Lint one or more documents

# Usage

```sh
stencila lint [OPTIONS] [FILES]...
```

# Examples

```bash
# Lint a single document
stencila lint document.smd

# Lint multiple documents
stencila lint *.qmd docs/*

# Auto-format documents during linting
stencila lint report.myst --format

# Auto-fix linting issues
stencila lint article.smd --fix

# Output diagnostics as YAML
stencila lint article.myst --as yaml
```

# Arguments

| Name      | Description        |
| --------- | ------------------ |
| `[FILES]` | The files to lint. |

# Options

| Name       | Description                                                                              |
| ---------- | ---------------------------------------------------------------------------------------- |
| `--format` | Format the file if necessary. Possible values: `true`, `false`.                          |
| `--fix`    | Fix any linting issues. Possible values: `true`, `false`.                                |
| `--cache`  | Cache the document after formatting and/or fixing it. Possible values: `true`, `false`.  |
| `-a, --as` | Output any linting diagnostics as JSON or YAML. Possible values: `json`, `yaml`, `toml`. |
