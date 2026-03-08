---
title: "`stencila db snapshot`"
description: Create a new baseline snapshot
---

Create a new baseline snapshot

Forces creation of a new baseline snapshot, even if the schema hasn't changed. Useful when many changesets have accumulated and replay time is growing. Uploads the snapshot and resets the manifest's changeset list.

# Usage

```sh
stencila db snapshot [OPTIONS]
```

# Examples

```bash
# Create a new baseline snapshot
stencila db snapshot

# Create with a description
stencila db snapshot -m "compact after batch processing"
```

# Options

| Name            | Description                                                                                        |
| --------------- | -------------------------------------------------------------------------------------------------- |
| `-m, --message` | Optional message describing this snapshot.                                                         |
| `--force`       | Force snapshot even when local database is not at manifest head. Possible values: `true`, `false`. |
