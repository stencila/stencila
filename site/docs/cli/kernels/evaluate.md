---
title: "`stencila kernels evaluate`"
description: Evaluate a code expression in a kernel
---

Evaluate a code expression in a kernel

Creates a temporary kernel instance, evaluates the expression in it, and returns the output and any execution messages.

Mainly intended for quick testing of kernels during development.

# Usage

```sh
stencila kernels evaluate <NAME> <CODE>
```

# Examples

```bash
# Evaluate a Python expression
stencila kernels evaluate python "2 + 2"

# Evaluate an R expression
stencila kernels evaluate r "sqrt(16)"

# Evaluate a JavaScript expression
stencila kernels evaluate javascript "Math.PI * 2"

# Use the eval alias
stencila kernels eval python "sum([1, 2, 3, 4, 5])"
```

# Arguments

| Name     | Description                                 |
| -------- | ------------------------------------------- |
| `<NAME>` | The name of the kernel to evaluate code in. |
| `<CODE>` | The code expression to evaluate.            |
