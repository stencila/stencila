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

# Filter models by provider or ID prefix
stencila models list anthropic

# Run a prompt with automatic model selection
stencila models run "Explain photosynthesis"

# Run with a specific model
stencila models run "Write a poem" --model gpt-4o

# Mix text and file arguments
stencila models run "Summarize this file:" document.txt

# Dry run to see prompt construction
stencila models run "Hello" --dry-run
```

# Subcommands

| Command           | Description                                               |
| ----------------- | --------------------------------------------------------- |
| [`list`](list.md) | List available models with their capabilities and pricing |
| [`run`](run.md)   | Execute a prompt using a generative AI model              |
