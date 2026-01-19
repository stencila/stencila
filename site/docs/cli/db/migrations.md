---
title: "`stencila db migrations`"
description: Show applied and pending migrations
---

Show applied and pending migrations

# Usage

```sh
stencila db migrations [OPTIONS] [DB]
```

# Examples

```bash
# Show applied and pending migrations for the workspace database
stencila db migrations

# Output migrations as JSON
stencila db migrations --as json

# Show migrations for a specific database
stencila db migrations path/to/database.kuzu
```

# Arguments

| Name   | Description                |
| ------ | -------------------------- |
| `[DB]` | Path to the database file. |

# Options

| Name            | Description                                             |
| --------------- | ------------------------------------------------------- |
| `-a, --as <AS>` | Output format. Possible values: `json`, `yaml`, `toml`. |
