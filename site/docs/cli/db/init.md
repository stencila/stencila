---
title: "`stencila db init`"
description: Initialize the workspace database
---

Initialize the workspace database

Creates the `.stencila/db.sqlite3` database file (if it does not already exist) and runs all pending schema migrations. This is a local-only, idempotent operation — no cloud credentials are required.

After initialization, use `stencila db push` to sync the database to Stencila Cloud.

# Usage

```sh
stencila db init
```
