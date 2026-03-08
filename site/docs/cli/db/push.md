---
title: "`stencila db push`"
description: Push database state to Stencila Cloud
---

Push database state to Stencila Cloud

# Usage

```sh
stencila db push [OPTIONS]
```

# Examples

```bash
# Push current database state
stencila db push

# Push with a description
stencila db push -m "add batch-1 results"
```

# Options

| Name            | Description                            |
| --------------- | -------------------------------------- |
| `-m, --message` | Optional message describing this push. |
