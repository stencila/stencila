---
title: "`stencila themes remove`"
description: Remove a user theme
---

Remove a user theme

# Usage

```sh
stencila themes remove [OPTIONS] <NAME>
```

# Examples

```bash
# Remove a user theme
stencila themes remove my-theme

# Force remove without confirmation
stencila themes remove my-theme --force

# Use the rm alias
stencila themes rm my-theme
```

# Arguments

| Name     | Description                      |
| -------- | -------------------------------- |
| `<NAME>` | The name of the theme to remove. |

# Options

| Name          | Description                                                              |
| ------------- | ------------------------------------------------------------------------ |
| `-f, --force` | Remove the theme without confirmation. Possible values: `true`, `false`. |
