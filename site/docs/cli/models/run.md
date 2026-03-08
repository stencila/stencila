---
title: "`stencila models run`"
description: Execute a prompt using a generative AI model
---

Execute a prompt using a generative AI model

Constructs a prompt from the provided text and file arguments, then streams the model's response to stdout. Arguments that correspond to existing file paths are read and included as file content.

# Usage

```sh
stencila models run [OPTIONS] [ARGS]...
```

# Examples

```bash
# Run with automatic model selection
stencila models run "Explain quantum computing"

# Run with a specific model
stencila models run "Write a haiku" --model gpt-4o

# Use a specific provider
stencila models run "Hello" --provider anthropic

# Multiple text arguments
stencila models run "Analyze this data:" "temperature: 23C, humidity: 65%"

# Mix text and file paths (files detected automatically)
stencila models run "Summarize:" report.txt

# Write output to a file
stencila models run "Generate a story" --output story.md

# Dry run to see prompt construction
stencila models run "Hello world" --dry-run
```

# Arguments

| Name     | Description                                              |
| -------- | -------------------------------------------------------- |
| `[ARGS]` | Text prompts and/or file paths (automatically detected). |

# Options

| Name             | Description                                                                   |
| ---------------- | ----------------------------------------------------------------------------- |
| `-m, --model`    | Model id to use (e.g. "gpt-4o", "claude-sonnet-4-5-20250929").                |
| `-p, --provider` | Provider name (e.g. "openai", "anthropic").                                   |
| `--system`       | System message to set context or behavior.                                    |
| `--temperature`  | Sampling temperature (0.0–2.0).                                               |
| `--max-tokens`   | Maximum tokens to generate.                                                   |
| `-o, --output`   | Write output to the specified file instead of stdout.                         |
| `--dry-run`      | Show prompt construction without executing. Possible values: `true`, `false`. |
