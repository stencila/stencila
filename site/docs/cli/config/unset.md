---
title: "`stencila config unset`"
description: Remove a configuration value
---

Remove a configuration value

# Usage

```sh
stencila config unset [OPTIONS] <KEY>
```

# Examples

```bash
# Remove from nearest stencila.toml
stencila config unset site.id

# Remove from user config
stencila config unset --user site.id

# Remove from local override
stencila config unset --local site.id

# Remove nested value
stencila config unset site.settings.theme
```

# Arguments

| Name    | Description                                   |
| ------- | --------------------------------------------- |
| `<KEY>` | Config key in dot notation (e.g., `site.id`). |

# Options

| Name      | Description                                                   |
| --------- | ------------------------------------------------------------- |
| `--user`  | Remove from user config. Possible values: `true`, `false`.    |
| `--local` | Remove from local override. Possible values: `true`, `false`. |
