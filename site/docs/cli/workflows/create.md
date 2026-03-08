---
title: "`stencila workflows create`"
description: Create a new workflow
---

Create a new workflow

Creates a new workflow directory with a template WORKFLOW.md in the workspace's `.stencila/workflows/` directory.

# Usage

```sh
stencila workflows create <NAME> <DESCRIPTION>
```

# Examples

```bash
# Create a new workflow in the workspace
stencila workflows create my-workflow "A multi-stage data pipeline"
```

# Arguments

| Name            | Description                              |
| --------------- | ---------------------------------------- |
| `<NAME>`        | The name for the new workflow.           |
| `<DESCRIPTION>` | A brief description of the new workflow. |
