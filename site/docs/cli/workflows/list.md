---
title: "`stencila workflows list`"
description: List available workflows
---

List available workflows

Shows workflows from `.stencila/workflows/`.

# Usage

```sh
stencila workflows list [OPTIONS]
```

# Examples

```bash
# List all workflows in table format
stencila workflows list

# Output workflows as JSON
stencila workflows list --as json
```

# Options

| Name       | Description                                                               |
| ---------- | ------------------------------------------------------------------------- |
| `-a, --as` | Output the list as JSON or YAML. Possible values: `json`, `yaml`, `toml`. |
