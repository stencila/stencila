---
title: "`stencila prompts list`"
description: List the prompts available
---

List the prompts available

Shows all available prompts with their names, descriptions, and versions.

# Usage

```sh
stencila prompts list [OPTIONS]
```

# Examples

```bash
# List all prompts in table format
stencila prompts list

# Output prompts as JSON
stencila prompts list --as json
```

# Options

| Name            | Description                                                               |
| --------------- | ------------------------------------------------------------------------- |
| `-a, --as <AS>` | Output the list as JSON or YAML. Possible values: `json`, `yaml`, `toml`. |
