---
title: "`stencila formats structuring`"
description: Get a list of all structuring operations, or those that are the default for a format
---

Get a list of all structuring operations, or those that are the default for a format

# Usage

```sh
stencila formats structuring [OPTIONS] [FORMAT]
```

# Examples

```bash
# List all structuring operations
stencila formats structuring

# List the default structuring operations for DOCX
stencila formats structuring docx

# List all structuring operations with details for each
stencila formats structuring --verbose
```

# Arguments

| Name       | Description                                            |
| ---------- | ------------------------------------------------------ |
| `[FORMAT]` | The format to show default structuring operations for. |

# Options

| Name                      | Description                                                                             |
| ------------------------- | --------------------------------------------------------------------------------------- |
| `-v, --verbose <VERBOSE>` | Provide longer details on each structuring operation. Possible values: `true`, `false`. |
