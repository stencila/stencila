---
title: "`stencila agents create`"
description: Create a new agent
---

Create a new agent

Creates a new agent directory with a template AGENT.md. By default creates in the workspace's `.stencila/agents/` directory. Use `--user` to create in `~/.config/stencila/agents/` instead.

# Usage

```sh
stencila agents create [OPTIONS] <NAME> <DESCRIPTION>
```

# Examples

```bash
# Create a new agent in the workspace
stencila agents create my-agent "A helpful assistant"

# Create a new agent in user config
stencila agents create my-agent "A helpful assistant" --user
```

# Arguments

| Name            | Description                           |
| --------------- | ------------------------------------- |
| `<NAME>`        | The name for the new agent.           |
| `<DESCRIPTION>` | A brief description of the new agent. |

# Options

| Name     | Description                                                                             |
| -------- | --------------------------------------------------------------------------------------- |
| `--user` | Create in user config directory instead of workspace. Possible values: `true`, `false`. |
