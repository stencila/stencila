---
title: "`stencila config set`"
description: Set a configuration value
---

Set a configuration value

# Usage

```sh
stencila config set [OPTIONS] <KEY> <VALUE>
```

# Examples

```bash
# Set in nearest stencila.toml (or create in CWD)
stencila config set site.id mysite123

# Set in user config
stencila config set --user site.id mysite123

# Set in local override
stencila config set --local site.id mysite123

# Set nested value
stencila config set site.settings.theme dark

# Set boolean
stencila config set site.settings.enabled true

# Set number
stencila config set site.settings.port 8080
```

# Arguments

| Name      | Description                                   |
| --------- | --------------------------------------------- |
| `<KEY>`   | Config key in dot notation (e.g., `site.id`). |
| `<VALUE>` | Value to set.                                 |

# Options

| Name      | Description                                                                              |
| --------- | ---------------------------------------------------------------------------------------- |
| `--user`  | Set in user config (~/.config/stencila/stencila.toml). Possible values: `true`, `false`. |
| `--local` | Set in local override (stencila.local.yaml). Possible values: `true`, `false`.           |
