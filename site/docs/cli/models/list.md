---
title: "`stencila models list`"
description: List available models with their status and capabilities
---

List available models with their status and capabilities

# Usage

```sh
stencila models list [OPTIONS] [PREFIX]
```

# Examples

```bash
# List all models in table format
stencila models list

# Filter models by ID prefix
stencila models list google/gemini

# Output models as YAML
stencila models list --as yaml
```

# Arguments

| Name       | Description                                        |
| ---------- | -------------------------------------------------- |
| `[PREFIX]` | Filter models by ID prefix (e.g., "ollama/gemma"). |

# Options

| Name       | Description                                                               |
| ---------- | ------------------------------------------------------------------------- |
| `-a, --as` | Output the list as JSON or YAML. Possible values: `json`, `yaml`, `toml`. |
