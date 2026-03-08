---
title: "`stencila skills show`"
description: Show a skill
---

Show a skill

Displays the full content and metadata of a specific skill.

# Usage

```sh
stencila skills show [OPTIONS] <NAME>
```

# Examples

```bash
# Show a skill as Markdown
stencila skills show data-analysis

# Show a skill as JSON
stencila skills show data-analysis --as json
```

# Arguments

| Name     | Description                    |
| -------- | ------------------------------ |
| `<NAME>` | The name of the skill to show. |

# Options

| Name       | Description                                           |
| ---------- | ----------------------------------------------------- |
| `-a, --as` | The format to show the skill in. Default value: `md`. |
