---
title: "`stencila agents show`"
description: Show an agent
---

Show an agent

Displays the full content and metadata of a specific agent.

# Usage

```sh
stencila agents show [OPTIONS] <NAME>
```

# Examples

```bash
# Show an agent as Markdown
stencila agents show code-review

# Show an agent as JSON
stencila agents show code-review --as json
```

# Arguments

| Name     | Description                    |
| -------- | ------------------------------ |
| `<NAME>` | The name of the agent to show. |

# Options

| Name       | Description                                           |
| ---------- | ----------------------------------------------------- |
| `-a, --as` | The format to show the agent in. Default value: `md`. |
