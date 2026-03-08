---
title: "`stencila workflows show`"
description: Show a workflow
---

Show a workflow

Displays the full content and metadata of a specific workflow.

# Usage

```sh
stencila workflows show [OPTIONS] <NAME>
```

# Examples

```bash
# Show a workflow as Markdown
stencila workflows show data-pipeline

# Show a workflow as JSON
stencila workflows show data-pipeline --as json
```

# Arguments

| Name     | Description                       |
| -------- | --------------------------------- |
| `<NAME>` | The name of the workflow to show. |

# Options

| Name       | Description                                              |
| ---------- | -------------------------------------------------------- |
| `-a, --as` | The format to show the workflow in. Default value: `md`. |
