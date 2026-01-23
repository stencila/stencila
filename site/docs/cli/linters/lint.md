---
title: "`stencila linters lint`"
description: Lint a file
---

Lint a file

Mainly intended for testing linters during development of Stencila. To lint a document use `stencila lint`.

# Usage

```sh
stencila linters lint [OPTIONS] <FILE>
```

# Examples

```bash
# Lint a Python file
stencila linters lint script.py

# Lint and format a JavaScript file
stencila linters lint app.js --format

# Lint and fix issues where possible
stencila linters lint code.r --fix

# Lint with both formatting and fixing
stencila linters lint code.py --format --fix
```

# Arguments

| Name     | Description       |
| -------- | ----------------- |
| `<FILE>` | The file to lint. |

# Options

| Name           | Description                                                                           |
| -------------- | ------------------------------------------------------------------------------------- |
| `-l, --linter` | The name of the linter to use.                                                        |
| `--format`     | Format the content of the file. Possible values: `true`, `false`.                     |
| `--fix`        | Fix warnings and errors in the file where possible. Possible values: `true`, `false`. |
