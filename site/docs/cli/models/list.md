---
title: "`stencila models list`"
description: List available models with their capabilities and pricing
---

List available models with their capabilities and pricing

# Usage

```sh
stencila models list [OPTIONS] [PREFIX]
```

# Examples

```bash
# List all models in table format
stencila models list

# Filter models by provider
stencila models list anthropic

# Filter models by ID prefix
stencila models list gpt-4

# Refresh from provider APIs before listing
stencila models list --live

# Output models as JSON
stencila models list --as json

# Output models as YAML
stencila models list --as yaml
```

# Arguments

| Name       | Description                                                         |
| ---------- | ------------------------------------------------------------------- |
| `[PREFIX]` | Filter models by provider or ID prefix (e.g. "anthropic", "gpt-4"). |

# Options

| Name       | Description                                                                                              |
| ---------- | -------------------------------------------------------------------------------------------------------- |
| `--live`   | Fetch current model listings from configured providers before listing. Possible values: `true`, `false`. |
| `-a, --as` | Output the list as JSON or YAML. Possible values: `json`, `yaml`, `toml`.                                |

# Notes

By default, model listings come from Stencila's embedded catalog.
Use --live to also query configured provider APIs and include newly discovered models.
Live listing only works for providers you have credentials for.
