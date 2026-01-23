---
title: "`stencila themes show`"
description: Show the resolved theme CSS
---

Show the resolved theme CSS

# Usage

```sh
stencila themes show [OPTIONS] [NAME]
```

# Examples

```bash
# Show the default resolved theme
stencila themes show

# Show a specific theme by name
stencila themes show tufte

# Show a user theme
stencila themes show my-theme

# Show theme with resolved CSS variables
stencila themes show stencila --verbose
```

# Arguments

| Name     | Description                    |
| -------- | ------------------------------ |
| `[NAME]` | The name of the theme to show. |

# Options

| Name            | Description                                                    |
| --------------- | -------------------------------------------------------------- |
| `-v, --verbose` | Show resolved CSS variables. Possible values: `true`, `false`. |
