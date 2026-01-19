---
title: "`stencila kernels execute`"
description: Execute code in a kernel
---

Execute code in a kernel

Creates a temporary kernel instance, executes one or more lines of code, and returns any outputs and execution messages.

Mainly intended for quick testing of kernels during development.

# Usage

```sh
stencila kernels execute [OPTIONS] <NAME> <CODE>
```

# Examples

```bash
# Execute Python code
stencila kernels execute python "print('Hello World')"

# Execute multi-line code with escaped newlines
stencila kernels execute python "x = 5\nprint(x * 2)"

# Execute code in a sandboxed environment
stencila kernels execute python "import os\nprint(os.environ)" --box

# Use the exec alias
stencila kernels exec r "print(mean(c(1,2,3,4,5)))"
```

# Arguments

| Name     | Description                                |
| -------- | ------------------------------------------ |
| `<NAME>` | The name of the kernel to execute code in. |
| `<CODE>` | The code to execute.                       |

# Options

| Name              | Description                                                                                      |
| ----------------- | ------------------------------------------------------------------------------------------------ |
| `-b, --box <BOX>` | Execute code in a kernel instance with `Box` execution bounds. Possible values: `true`, `false`. |
