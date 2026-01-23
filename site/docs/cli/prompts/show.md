---
title: "`stencila prompts show`"
description: Show a prompt
---

Show a prompt

Displays the full content and metadata of a specific prompt in the requested format.

# Usage

```sh
stencila prompts show [OPTIONS] <NAME>
```

# Examples

```bash
# Show a prompt as Markdown
stencila prompts show edit-text

# Show a prompt as JSON
stencila prompts show create-table --to json
```

# Arguments

| Name     | Description                     |
| -------- | ------------------------------- |
| `<NAME>` | The name of the prompt to show. |

# Options

| Name       | Description                                            |
| ---------- | ------------------------------------------------------ |
| `-t, --to` | The format to show the prompt in. Default value: `md`. |
