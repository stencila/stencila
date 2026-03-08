---
title: "`stencila db`"
description: Manage the workspace database
---

Manage the workspace database

# Usage

```sh
stencila db <COMMAND>
```

# Examples

```bash
# Initialize the workspace database
stencila db init

# Show sync status
stencila db status

# Push database state to cloud
stencila db push

# Pull database state from cloud
stencila db pull

# Show changeset history
stencila db log

# Verify local db matches manifest
stencila db verify

# Rebuild database from manifest
stencila db reset

# Create a new baseline snapshot
stencila db snapshot

# Clean local blob cache
stencila db clean

# Remove orphaned remote blobs
stencila db gc
```

# Subcommands

| Command                   | Description                                      |
| ------------------------- | ------------------------------------------------ |
| [`init`](init.md)         | Initialize the workspace database                |
| [`push`](push.md)         | Push database state to Stencila Cloud            |
| [`pull`](pull.md)         | Pull database state from Stencila Cloud          |
| [`status`](status.md)     | Show database sync status                        |
| [`log`](log.md)           | Show changeset history from the manifest         |
| [`verify`](verify.md)     | Verify local database matches the manifest state |
| [`reset`](reset.md)       | Rebuild local database from the manifest         |
| [`snapshot`](snapshot.md) | Create a new baseline snapshot                   |
| [`clean`](clean.md)       | Clean up the local blob cache                    |
| [`gc`](gc.md)             | Remove orphaned remote blobs                     |
