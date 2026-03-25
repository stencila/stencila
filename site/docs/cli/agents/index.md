---
title: "`stencila agents`"
description: Manage agent definitions
---

Manage agent definitions

# Usage

```sh
stencila agents [COMMAND]
```

# Examples

```bash
# List all agents
stencila agents

# Show details about a specific agent
stencila agents show code-review

# Create a new agent in the workspace
stencila agents create my-agent "A helpful assistant"

# Create a new agent in user config
stencila agents create my-agent "A helpful assistant" --user

# Validate an agent by name, directory, or file path
stencila agents validate code-review

# Show how an agent session would be routed
stencila agents resolve code-engineer

# Run an agent with a prompt
stencila agents run code-engineer "What files are in this directory?"

# Dry run to see agent config and prompt
stencila agents run code-engineer "Hello" --dry-run
```

# Subcommands

| Command                         | Description                               |
| ------------------------------- | ----------------------------------------- |
| [`list`](list.md)               | List available agents                     |
| [`show`](show.md)               | Show an agent                             |
| [`create`](create.md)           | Create a new agent                        |
| [`validate`](validate.md)       | Validate an agent                         |
| [`resolve`](resolve.md)         | Show how an agent session would be routed |
| [`run`](run.md)                 | Run an agent with a prompt                |
| [`sessions`](sessions/index.md) | Manage agent sessions                     |
