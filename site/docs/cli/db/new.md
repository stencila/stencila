---
title: "`stencila db new`"
description: Create a new document database
---

Create a new document database

# Usage

```sh
stencila db new [OPTIONS] [PATH]
```

# Examples

```bash
# Create a document database in the current workspace
stencila db new

# Create a document database at a specific path
stencila db new path/to/my-database.kuzu

# Overwrite the database if it already exists
stencila db new temp.kuzu --force
```

# Arguments

| Name     | Description                |
| -------- | -------------------------- |
| `[PATH]` | Path to the database file. |

# Options

| Name          | Description                                                                    |
| ------------- | ------------------------------------------------------------------------------ |
| `-f, --force` | Overwrite the database if it already exists. Possible values: `true`, `false`. |
