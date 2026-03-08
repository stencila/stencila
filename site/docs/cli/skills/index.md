---
title: "`stencila skills`"
description: Manage agent skills
---

Manage agent skills

# Usage

```sh
stencila skills [COMMAND]
```

# Examples

```bash
# List all skills in the current workspace
stencila skills

# Show details about a specific skill
stencila skills show data-analysis

# Validate a skill by name, directory, or file path
stencila skills validate data-analysis
stencila skills validate .stencila/skills/data-analysis

# Create a new skill
stencila skills create my-new-skill
```

# Subcommands

| Command                   | Description           |
| ------------------------- | --------------------- |
| [`list`](list.md)         | List available skills |
| [`show`](show.md)         | Show a skill          |
| [`validate`](validate.md) | Validate a skill      |
| [`create`](create.md)     | Create a new skill    |
