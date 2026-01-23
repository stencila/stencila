---
title: "`stencila outputs list`"
description: List configured outputs
---

List configured outputs

# Usage

```sh
stencila outputs list [OPTIONS]
```

# Examples

```bash
# List configured outputs in table format
stencila outputs list

# List in JSON, YAML, or TOML format
stencila outputs list --as json
stencila outputs list --as yaml
stencila outputs list --as toml
```

# Options

| Name       | Description                                             |
| ---------- | ------------------------------------------------------- |
| `-a, --as` | Output format. Possible values: `json`, `yaml`, `toml`. |
