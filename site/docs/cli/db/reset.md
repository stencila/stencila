---
title: "`stencila db reset`"
description: Rebuild local database from the manifest
---

Rebuild local database from the manifest

Discards the local database and rebuilds it from scratch using the manifest (snapshot + all changesets). This is the escape hatch when the local database has diverged or become corrupted.

# Usage

```sh
stencila db reset
```

# Examples

```bash
# Rebuild database from manifest
stencila db reset
```
