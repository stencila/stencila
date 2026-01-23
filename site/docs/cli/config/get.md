---
title: "`stencila config get`"
description: Get configuration value(s)
---

Get configuration value(s)

# Usage

```sh
stencila config get [OPTIONS] [KEY]
```

# Examples

```bash
# Show entire configuration
stencila config get

# Show as JSON
stencila config get --as json

# Get a specific value
stencila config get site.id

# Get nested value
stencila config get site.settings.theme

# Get array element
stencila config get packages[0].name
```

# Arguments

| Name    | Description                                   |
| ------- | --------------------------------------------- |
| `[KEY]` | Config key in dot notation (e.g., `site.id`). |

# Options

| Name       | Description                                                                                  |
| ---------- | -------------------------------------------------------------------------------------------- |
| `-a, --as` | Output format (toml, json, or yaml, default: toml). Possible values: `json`, `yaml`, `toml`. |
