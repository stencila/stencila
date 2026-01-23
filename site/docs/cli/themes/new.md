---
title: "`stencila themes new`"
description: Create a new theme
---

Create a new theme

# Usage

```sh
stencila themes new [OPTIONS] [NAME]
```

# Examples

```bash
# Create a new workspace theme in the current folder
stencila themes new

# Create a named user theme in the config folder
stencila themes new my-theme

# Force overwrite an existing user theme
stencila themes new my-theme --force
```

# Arguments

| Name     | Description                      |
| -------- | -------------------------------- |
| `[NAME]` | The name of the theme to create. |

# Options

| Name          | Description                                                                      |
| ------------- | -------------------------------------------------------------------------------- |
| `-f, --force` | Overwrite the theme file if it already exists. Possible values: `true`, `false`. |
