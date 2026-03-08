---
title: "`stencila db clean`"
description: Clean up the local blob cache
---

Clean up the local blob cache

Removes cached blobs that are no longer referenced by the current manifest. Useful for reclaiming disk space after snapshot rotations.

# Usage

```sh
stencila db clean
```

# Examples

```bash
# Remove unreferenced cached blobs
stencila db clean
```
