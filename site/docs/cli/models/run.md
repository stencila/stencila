---
title: "`stencila models run`"
description: Execute a task using a generative AI model
---

Execute a task using a generative AI model

Primarily intended for testing model selection and routing. This command constructs a task from the provided inputs, selects an appropriate model, and displays both the constructed task and the generated output.

# Usage

```sh
stencila models run [OPTIONS] [ARGS]...
```

# Examples

```bash
# Run with automatic model selection
stencila models run "Explain quantum computing"

# Run with a specific model
stencila models run "Write a haiku" --model gpt-3.5-turbo

# Multiple text arguments
stencila models run "Analyze this data:" "temperature: 23°C, humidity: 65%"

# Mix text and file paths (files detected automatically)
stencila models run "Summarize:" report.txt

# Multiple files and text
stencila models run "Compare these:" version1.py version2.py

# Run a dry run to see task construction
stencila models run "Hello world" --dry-run

# Use the execute alias
stencila models execute "Summarize this text"
```

# Arguments

| Name     | Description                                              |
| -------- | -------------------------------------------------------- |
| `[ARGS]` | Text prompts and/or file paths (automatically detected). |

# Options

| Name                  | Description                                                                                     |
| --------------------- | ----------------------------------------------------------------------------------------------- |
| `-m, --model`         | Model id or pattern to select a specific model (e.g., "gpt-4o", "ollama/").                     |
| `-f, --format`        | Output format for generated content (json, markdown, yaml, etc.).                               |
| `-s, --schema`        | JSON schema name for structured output validation (e.g., "math-block-tex").                     |
| `--system`            | System message to set context or behavior for the model.                                        |
| `-o, --output`        | Write generated output to the specified file instead of stdout.                                 |
| `--dry-run <DRY_RUN>` | Show task construction and model selection without executing. Possible values: `true`, `false`. |

# Note

Arguments are automatically detected as file paths (if they exist) or treated as
text content. Images are detected by file extension. This command is primarily
for testing model routing and selection.
