---
title: Workspace Database
description: Overview of the workspace database and how to sync it across collaborators
---

The workspace database (`.stencila/db.sqlite3`) is a SQLite database that stores state for your Stencila workspace. Currently it holds workflow run history (execution results, node metrics, context snapshots, artifacts, and logs). The database is organized by domain, so additional domains may be added in the future.

The database lives inside the `.stencila/` directory and is excluded from git by default. This keeps your repository clean but means collaborators don't automatically get your database when they clone the project. The **database sync** system solves this by letting you push and pull database state through Stencila Cloud, with git tracking a small manifest file that describes how to reconstruct the database.

# How sync works

The sync system splits storage into two parts:

- **Git** tracks a small JSON manifest file (`.stencila/db.json`) that describes the database state: which base snapshot to use and which changesets to apply on top of it.

- **Stencila Cloud** stores the actual data as content-addressed blobs: compressed database snapshots and incremental changesets.

The primary commands are `push` (upload local changes) and `pull` (download and apply remote changes). After pushing, you commit the updated manifest to git so collaborators can see it.

# Getting started

See the [syncing guide](sync.md) for step-by-step instructions on setting up sync, pushing your first database state, and keeping a team in sync.

# Further reading

- [Syncing guide](sync.md) — first-time setup, push/pull workflow, team collaboration
- [Under the hood](under-the-hood.md) — how SQLite sessions, content addressing, and schema migrations work
- [Troubleshooting](troubleshooting.md) — verify, reset, and common issues
