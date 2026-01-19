---
title: "`stencila prompts`"
description: Manage prompts
---

Manage prompts

# Usage

```sh
stencila prompts [COMMAND]
```

# Examples

```bash
# List all available prompts
stencila prompts

# Show details about a specific prompt
stencila prompts show edit-text

# Infer which prompt would be used for a query
stencila prompts infer --instruction-type create "Make a table"

# Update builtin prompts from remote
stencila prompts update

# Reset prompts to embedded defaults
stencila prompts reset
```

# Subcommands

| Command               | Description                 |
| --------------------- | --------------------------- |
| [`list`](list.md)     | List the prompts available  |
| [`show`](show.md)     | Show a prompt               |
| [`infer`](infer.md)   | Infer a prompt from a query |
| [`update`](update.md) | Update builtin prompts      |
| [`reset`](reset.md)   | Reset builtin prompts       |
