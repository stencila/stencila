---
title: "`stencila kernels info`"
description: Get information about a kernel
---

Get information about a kernel

Mainly used to check the version of the kernel runtime and operating system for debugging purpose.

# Usage

```sh
stencila kernels info <NAME>
```

# Examples

```bash
# Get information about the Python kernel
stencila kernels info python

# Get information about the R kernel
stencila kernels info r

# Get information about the JavaScript kernel
stencila kernels info javascript
```

# Arguments

| Name     | Description                                    |
| -------- | ---------------------------------------------- |
| `<NAME>` | The name of the kernel to get information for. |
