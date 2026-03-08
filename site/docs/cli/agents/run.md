---
title: "`stencila agents run`"
description: Run an agent with a prompt
---

Run an agent with a prompt

Discovers a named agent definition, creates an agent session using the agent's configuration (model, provider, instructions, tool settings), and streams the response. Arguments that correspond to existing file paths are read and included as file content. Mainly for testing.

# Usage

```sh
stencila agents run [OPTIONS] <NAME> [ARGS]...
```

# Examples

```bash
# Run an agent with a prompt
stencila agents run code-engineer "What files are in this directory?"

# Mix text and file paths
stencila agents run code-review "Review this file:" src/main.rs

# Write output to a file
stencila agents run code-engineer "Generate a README" --output README.md

# Dry run to see agent config and prompt
stencila agents run code-engineer "Hello" --dry-run
```

# Arguments

| Name     | Description                                              |
| -------- | -------------------------------------------------------- |
| `<NAME>` | The name of the agent to run.                            |
| `[ARGS]` | Text prompts and/or file paths (automatically detected). |

# Options

| Name           | Description                                                                       |
| -------------- | --------------------------------------------------------------------------------- |
| `-o, --output` | Write output to the specified file instead of stdout.                             |
| `--dry-run`    | Show agent config and prompt without executing. Possible values: `true`, `false`. |
