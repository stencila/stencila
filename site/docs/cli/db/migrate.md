---
title: "`stencila db migrate`"
description: Run pending database migrations
---

Run pending database migrations

# Usage

```sh
stencila db migrate [OPTIONS] [DB]
```

# Examples

```bash
# Apply pending migrations to workspace database
stencila db migrate

# Preview what migrations would be applied
stencila db migrate --dry-run

# Apply migrations to a specific database
stencila db migrate path/to/my-database.kuzu
```

# Arguments

| Name   | Description                |
| ------ | -------------------------- |
| `[DB]` | Path to the database file. |

# Options

| Name                      | Description                                                                 |
| ------------------------- | --------------------------------------------------------------------------- |
| `-d, --dry-run <DRY_RUN>` | Preview migrations without applying them. Possible values: `true`, `false`. |
