---
title: "`stencila themes validate`"
description: Validate a theme file
---

Validate a theme file

Checks that the CSS can be parsed and that custom properties correspond to known builtin design tokens (see `stencila themes tokens`).

# Usage

```sh
stencila themes validate [OPTIONS] <FILE>
```

# Examples

```bash
# Validate a workspace theme
stencila themes validate theme.css

# Treat unknown tokens as errors
stencila themes validate theme.css --strict

# Output validation result as JSON
stencila themes validate theme.css --as json
```

# Arguments

| Name     | Description                       |
| -------- | --------------------------------- |
| `<FILE>` | Path to the CSS file to validate. |

# Options

| Name       | Description                                                                            |
| ---------- | -------------------------------------------------------------------------------------- |
| `--strict` | Treat unknown tokens as errors (non-zero exit code). Possible values: `true`, `false`. |
| `--as`     | Output as a machine-readable format. Possible values: `json`, `yaml`, `toml`.          |
