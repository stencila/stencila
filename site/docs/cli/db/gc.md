---
title: "`stencila db gc`"
description: Remove orphaned remote blobs
---

Remove orphaned remote blobs

Lists all blobs stored on Stencila Cloud for this workspace and removes any that are not referenced by the current manifest. This cleans up blobs left behind by rebased or force-pushed manifests.

# Usage

```sh
stencila db gc [OPTIONS]
```

# Examples

```bash
# Remove orphaned remote blobs
stencila db gc

# Preview what would be removed
stencila db gc --dry-run

# Non-interactive: fetch first, then GC
stencila db gc --fetch

# Non-interactive: skip fetch
stencila db gc --no-fetch
```

# Options

| Name         | Description                                                                                       |
| ------------ | ------------------------------------------------------------------------------------------------- |
| `--dry-run`  | Show what would be removed without actually deleting. Possible values: `true`, `false`.           |
| `--fetch`    | Run `git fetch --all` before scanning refs (without prompting). Possible values: `true`, `false`. |
| `--no-fetch` | Skip `git fetch` (without prompting). Possible values: `true`, `false`.                           |
