---
title: "`stencila kernels packages`"
description: List packages available to a kernel
---

List packages available to a kernel

Mainly used to check libraries available to a kernel for debugging purpose.

# Usage

```sh
stencila kernels packages <NAME> [FILTER]
```

# Examples

```bash
# List all packages available to Python kernel
stencila kernels packages python

# Filter packages by name (case insensitive)
stencila kernels packages python numpy

# List R packages containing 'plot'
stencila kernels packages r plot
```

# Arguments

| Name       | Description                                  |
| ---------- | -------------------------------------------- |
| `<NAME>`   | The name of the kernel to list packages for. |
| `[FILTER]` | A filter on the name of the kernel.          |
