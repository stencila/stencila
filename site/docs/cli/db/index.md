---
title: "`stencila db`"
description: Manage the workspace and other document databases
---

Manage the workspace and other document databases

# Usage

```sh
stencila db <COMMAND>
```

# Examples

```bash
# Run pending migrations on workspace database
stencila db migrate

# Check migration status
stencila db migrations status

# Validate migrations without applying
stencila db migrate --dry-run

# Work with a specific database
stencila db migrate /path/to/database.db
```

# Subcommands

| Command                       | Description                                  |
| ----------------------------- | -------------------------------------------- |
| [`new`](new.md)               | Create a new document database               |
| [`add`](add.md)               | Add documents to the workspace database      |
| [`remove`](remove.md)         | Remove documents from the workspace database |
| [`query`](query.md)           | Query a workspace database                   |
| [`migrate`](migrate.md)       | Run pending database migrations              |
| [`migrations`](migrations.md) | Show applied and pending migrations          |
