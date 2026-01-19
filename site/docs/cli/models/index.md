---
title: "`stencila models`"
description: Manage and interact with generative AI models
---

Manage and interact with generative AI models

# Usage

```sh
stencila models [COMMAND]
```

# Examples

```bash
# List all available models
stencila models

# List models as JSON
stencila models list --as json

# Test a model with a prompt
stencila models run "Explain photosynthesis"

# Test a specific model
stencila models run "Write a poem" --model gpt-4o

# Run with multiple text arguments
stencila models run "Analyze this:" "Some data here"

# Mix text and file arguments
stencila models run "Summarize this file:" document.txt

# Multiple files and text
stencila models run "Compare these files:" file1.txt file2.txt

# Dry run to see task construction
stencila models run "Hello" --dry-run

Model Types
• builtin - Built into Stencila
• local - Running locally (e.g. Ollama)
• remote - Cloud-based APIs
• router - Routes to other models
• proxied - Proxied through another service
```

# Subcommands

| Command           | Description                                              |
| ----------------- | -------------------------------------------------------- |
| [`list`](list.md) | List available models with their status and capabilities |
| [`run`](run.md)   | Execute a task using a generative AI model               |
