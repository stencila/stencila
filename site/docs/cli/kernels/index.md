---
title: "`stencila kernels`"
description: Manage execution kernels
---

Manage execution kernels

# Usage

```sh
stencila kernels [COMMAND]
```

# Examples

```bash
# List all available kernels
stencila kernels

# Get information about a specific kernel
stencila kernels info python

# List packages available to a kernel
stencila kernels packages r

# Execute code in a kernel
stencila kernels execute python "print('Hello')"
```

# Subcommands

| Command                   | Description                            |
| ------------------------- | -------------------------------------- |
| [`list`](list.md)         | List the kernels available             |
| [`info`](info.md)         | Get information about a kernel         |
| [`packages`](packages.md) | List packages available to a kernel    |
| [`execute`](execute.md)   | Execute code in a kernel               |
| [`evaluate`](evaluate.md) | Evaluate a code expression in a kernel |
